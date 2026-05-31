use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use serde::{Deserialize, Serialize};
use symphonia::core::audio::sample::Sample;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::Time;
use symphonia_adapter_libopus::OpusDecoder;

const INITIAL_CACHE_BUFFER_BYTES: u64 = 384 * 1024;
const INITIAL_CACHE_WAIT_TIMEOUT: Duration = Duration::from_secs(20);
const CACHE_WAIT_STEP: Duration = Duration::from_millis(40);
const HTTP_READ_BUFFER_SIZE: usize = 128 * 1024;
const NO_SEEK_MILLIS: u64 = u64::MAX;
const MUSIC_STREAM_CACHE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

pub struct ResolvedMusicStream {
    pub item_id: u64,
    pub session_id: u64,
    pub source_url: String,
    pub direct_url: String,
    pub headers: Vec<(String, String)>,
    pub title: String,
    pub album_title: String,
    pub thumbnail_url: String,
    pub duration_seconds: Option<f64>,
    pub ext: String,
    pub format_id: String,
    pub acodec: String,
    pub cache_key: String,
    pub expected_bytes: Option<u64>,
    pub cache_root: PathBuf,
    pub cache_command: Option<Command>,
    pub volume: f32,
}

#[derive(Debug)]
pub enum MusicPlaybackEvent {
    ToolCommandFinished {
        item_id: u64,
        session_id: u64,
        tool: String,
        action: String,
        command_line: String,
        success: bool,
    },
    Started {
        item_id: u64,
        session_id: u64,
    },
    Finished {
        item_id: u64,
        session_id: u64,
    },
    Stopped {
        item_id: u64,
        session_id: u64,
    },
    Failed {
        item_id: u64,
        session_id: u64,
        error: String,
    },
    PrefetchToolCommandFinished {
        item_id: u64,
        session_id: u64,
        tool: String,
        action: String,
        command_line: String,
        success: bool,
    },
    PrefetchFinished {
        item_id: u64,
        session_id: u64,
        success: bool,
        error: Option<String>,
    },
}

#[derive(Clone)]
pub struct MusicPlaybackControl {
    pub item_id: u64,
    pub session_id: u64,
    shared: Arc<SharedPlaybackState>,
    cache_state: Arc<CacheTransferState>,
}

impl MusicPlaybackControl {
    pub fn pause(&self) {
        self.shared.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.shared.paused.store(false, Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.shared.stop_requested.store(true, Ordering::Relaxed);
    }

    pub fn set_volume(&self, volume: f32) {
        self.shared
            .volume_bits
            .store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.shared.paused.load(Ordering::Relaxed)
    }

    pub fn progress_ratio(&self) -> f32 {
        let Some(duration) = self.duration_seconds() else {
            return 0.0;
        };
        if duration <= 0.0 {
            return 0.0;
        }
        (self.playback_seconds() / duration).clamp(0.0, 1.0) as f32
    }

    pub fn cache_progress_ratio(&self) -> f32 {
        self.cache_state.progress_ratio()
    }

    pub fn cache_is_complete(&self) -> bool {
        self.cache_state.complete.load(Ordering::Relaxed)
    }

    pub fn seek_to_ratio(&self, ratio: f32) {
        let Some(duration) = self.duration_seconds() else {
            return;
        };
        if duration <= 0.0 {
            return;
        }
        let allowed_ratio = if self.cache_is_complete() {
            ratio.clamp(0.0, 1.0)
        } else {
            ratio.clamp(0.0, self.cache_progress_ratio().clamp(0.0, 1.0))
        };
        let target = duration * f64::from(allowed_ratio);
        self.shared
            .seek_target_millis
            .store((target * 1000.0).round().max(0.0) as u64, Ordering::Relaxed);
    }

    pub fn playback_seconds(&self) -> f64 {
        let sample_rate = self.shared.sample_rate.load(Ordering::Relaxed).max(1) as f64;
        let channels = self.shared.channels.load(Ordering::Relaxed).max(1) as f64;
        self.shared.samples_played.load(Ordering::Relaxed) as f64 / sample_rate / channels
    }

    pub fn duration_seconds(&self) -> Option<f64> {
        let bits = self.shared.duration_bits.load(Ordering::Relaxed);
        let duration = f64::from_bits(bits);
        duration
            .is_finite()
            .then_some(duration)
            .filter(|value| *value > 0.0)
    }
}

#[derive(Clone)]
pub struct MusicPrefetchControl {
    pub item_id: u64,
    pub session_id: u64,
    cancel_requested: Arc<AtomicBool>,
}

impl MusicPrefetchControl {
    pub fn cancel(&self) {
        self.cancel_requested.store(true, Ordering::Relaxed);
    }
}

struct SharedPlaybackState {
    buffer: Mutex<VecDeque<f32>>,
    stop_requested: AtomicBool,
    paused: AtomicBool,
    volume_bits: AtomicU32,
    samples_played: AtomicU64,
    sample_rate: AtomicU32,
    channels: AtomicU32,
    duration_bits: AtomicU64,
    seek_target_millis: AtomicU64,
    partial_seek_enabled: AtomicBool,
}

impl SharedPlaybackState {
    fn new(volume: f32, duration_seconds: Option<f64>) -> Self {
        Self {
            buffer: Mutex::new(VecDeque::new()),
            stop_requested: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            volume_bits: AtomicU32::new(volume.clamp(0.0, 1.0).to_bits()),
            samples_played: AtomicU64::new(0),
            sample_rate: AtomicU32::new(44_100),
            channels: AtomicU32::new(2),
            duration_bits: AtomicU64::new(duration_seconds.unwrap_or(0.0).to_bits()),
            seek_target_millis: AtomicU64::new(NO_SEEK_MILLIS),
            partial_seek_enabled: AtomicBool::new(false),
        }
    }
}

#[derive(Debug)]
struct CacheTransferState {
    downloaded_bytes: AtomicU64,
    expected_bytes: AtomicU64,
    complete: AtomicBool,
    failed: AtomicBool,
    ranges: Mutex<Vec<MusicCacheRange>>,
    error: Mutex<Option<String>>,
}

impl Default for CacheTransferState {
    fn default() -> Self {
        Self {
            downloaded_bytes: AtomicU64::new(0),
            expected_bytes: AtomicU64::new(0),
            complete: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            ranges: Mutex::new(Vec::new()),
            error: Mutex::new(None),
        }
    }
}

impl CacheTransferState {
    fn expected_bytes(&self) -> Option<u64> {
        let value = self.expected_bytes.load(Ordering::Relaxed);
        (value > 0).then_some(value)
    }

    fn progress_ratio(&self) -> f32 {
        if self.complete.load(Ordering::Relaxed) {
            return 1.0;
        }
        let Some(expected) = self.expected_bytes() else {
            return 0.0;
        };
        if expected == 0 {
            return 0.0;
        }
        (self.downloaded_bytes.load(Ordering::Relaxed) as f32 / expected as f32).clamp(0.0, 1.0)
    }

    fn set_expected_bytes(&self, value: Option<u64>) {
        self.expected_bytes
            .store(value.unwrap_or(0), Ordering::Relaxed);
    }

    fn set_downloaded_bytes(&self, value: u64) {
        self.downloaded_bytes.store(value, Ordering::Relaxed);
    }

    fn seed_ranges(&self, ranges: Vec<MusicCacheRange>) {
        if let Ok(mut slot) = self.ranges.lock() {
            *slot = normalize_ranges(ranges);
            self.downloaded_bytes
                .store(total_range_bytes(&slot), Ordering::Relaxed);
        }
    }

    fn ranges_snapshot(&self) -> Vec<MusicCacheRange> {
        self.ranges
            .lock()
            .map(|ranges| ranges.clone())
            .unwrap_or_default()
    }

    fn add_range(&self, start: u64, end: u64) {
        if end <= start {
            return;
        }
        if let Ok(mut ranges) = self.ranges.lock() {
            ranges.push(MusicCacheRange { start, end });
            *ranges = normalize_ranges(std::mem::take(&mut *ranges));
            self.downloaded_bytes
                .store(total_range_bytes(&ranges), Ordering::Relaxed);
        }
    }

    fn available_end_from(&self, position: u64) -> Option<u64> {
        self.ranges.lock().ok().and_then(|ranges| {
            ranges
                .iter()
                .find(|range| range.start <= position && position < range.end)
                .map(|range| range.end)
        })
    }

    fn contiguous_end_from_start(&self) -> u64 {
        let Ok(ranges) = self.ranges.lock() else {
            return 0;
        };
        let mut end = 0_u64;
        for range in ranges.iter() {
            if range.start > end {
                break;
            }
            end = end.max(range.end);
        }
        end
    }

    fn is_fully_cached(&self) -> bool {
        let Some(expected) = self.expected_bytes() else {
            return false;
        };
        expected > 0 && self.contiguous_end_from_start() >= expected
    }

    fn set_complete(&self, value: bool) {
        self.complete.store(value, Ordering::Relaxed);
    }

    fn set_error(&self, error: String) {
        self.failed.store(true, Ordering::Relaxed);
        if let Ok(mut slot) = self.error.lock() {
            *slot = Some(error);
        }
    }

    fn error_text(&self) -> Option<String> {
        self.error.lock().ok().and_then(|slot| slot.clone())
    }
}

#[derive(Clone, Debug)]
struct MusicCachePaths {
    dir: PathBuf,
    media: PathBuf,
    cover: PathBuf,
    manifest: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct MusicCacheManifest {
    source_url: String,
    title: String,
    #[serde(default)]
    album_title: String,
    duration_seconds: Option<f64>,
    ext: String,
    format_id: String,
    acodec: String,
    #[serde(default)]
    thumbnail_url: String,
    #[serde(default)]
    cover_file: String,
    expected_bytes: Option<u64>,
    downloaded_bytes: u64,
    #[serde(default)]
    ranges: Vec<MusicCacheRange>,
    complete: bool,
    updated_unix_seconds: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct MusicCacheRange {
    start: u64,
    end: u64,
}

pub fn spawn_music_stream_playback(
    stream: ResolvedMusicStream,
    event_tx: Sender<MusicPlaybackEvent>,
) -> MusicPlaybackControl {
    let shared = Arc::new(SharedPlaybackState::new(
        stream.volume,
        stream.duration_seconds,
    ));
    let cache_state = Arc::new(CacheTransferState::default());
    let control = MusicPlaybackControl {
        item_id: stream.item_id,
        session_id: stream.session_id,
        shared: shared.clone(),
        cache_state: cache_state.clone(),
    };

    thread::spawn(move || {
        let item_id = stream.item_id;
        let session_id = stream.session_id;
        let shared_for_error = shared.clone();
        let result = run_stream_playback(stream, shared, cache_state, event_tx.clone());
        if let Err(error) = result {
            if shared_for_error.stop_requested.load(Ordering::Relaxed) {
                let _ = event_tx.send(MusicPlaybackEvent::Stopped {
                    item_id,
                    session_id,
                });
            } else {
                let _ = event_tx.send(MusicPlaybackEvent::Failed {
                    item_id,
                    session_id,
                    error,
                });
            }
        }
    });

    control
}

pub fn spawn_music_stream_prefetch(
    mut stream: ResolvedMusicStream,
    event_tx: Sender<MusicPlaybackEvent>,
) -> MusicPrefetchControl {
    let cancel_requested = Arc::new(AtomicBool::new(false));
    let control = MusicPrefetchControl {
        item_id: stream.item_id,
        session_id: stream.session_id,
        cancel_requested: cancel_requested.clone(),
    };
    thread::spawn(move || {
        let item_id = stream.item_id;
        let session_id = stream.session_id;
        let result = run_music_stream_prefetch(&mut stream, event_tx.clone(), cancel_requested);
        let (success, error) = match result {
            Ok(()) => (true, None),
            Err(error) => (false, Some(error)),
        };
        let _ = event_tx.send(MusicPlaybackEvent::PrefetchFinished {
            item_id,
            session_id,
            success,
            error,
        });
    });
    control
}

fn run_music_stream_prefetch(
    stream: &mut ResolvedMusicStream,
    event_tx: Sender<MusicPlaybackEvent>,
    cancel_requested: Arc<AtomicBool>,
) -> Result<(), String> {
    let paths = music_cache_paths(stream)?;
    fs::create_dir_all(&paths.dir)
        .map_err(|error| format!("Could not create music stream cache: {error}"))?;
    if existing_cache_manifest_is_not_fresh(&paths) {
        let _ = fs::remove_file(&paths.media);
        let _ = fs::remove_file(&paths.cover);
        let _ = fs::remove_file(&paths.manifest);
    }
    cache_cover_image_if_needed(stream, &paths);

    let existing_bytes = fs::metadata(&paths.media)
        .map(|meta| meta.len())
        .unwrap_or(0);
    if cached_media_is_complete(&paths, stream, existing_bytes) {
        return Ok(());
    }

    let cache_state = Arc::new(CacheTransferState::default());
    if let Some(expected) = stream.expected_bytes.filter(|value| *value > 0) {
        cache_state.set_expected_bytes(Some(expected));
    }
    cache_state.seed_ranges(manifest_ranges_for_existing_cache(&paths, existing_bytes));

    if let Some(command) = stream.cache_command.take() {
        let command_line = format_process_command_line(&command);
        let manifest_info = MusicCacheManifestInfo::from_stream(stream);
        let result = run_yt_dlp_cache_downloader(
            command,
            paths,
            cache_state,
            manifest_info,
            Some(cancel_requested.clone()),
        );
        let success = result.is_ok();
        let _ = event_tx.send(MusicPlaybackEvent::PrefetchToolCommandFinished {
            item_id: stream.item_id,
            session_id: stream.session_id,
            tool: "yt-dlp".to_owned(),
            action: "prefetch cache".to_owned(),
            command_line,
            success,
        });
        return result;
    }

    let fallback_stream = MusicHttpCacheDownloadInfo::from_stream(stream);
    run_http_cache_downloader(fallback_stream, paths, cache_state, Some(cancel_requested))
}

fn music_codec_registry() -> &'static CodecRegistry {
    static CODEC_REGISTRY: OnceLock<CodecRegistry> = OnceLock::new();
    CODEC_REGISTRY.get_or_init(|| {
        let mut registry = CodecRegistry::new();
        symphonia::default::register_enabled_codecs(&mut registry);
        registry.register_audio_decoder::<OpusDecoder>();
        registry
    })
}

fn run_stream_playback(
    mut stream: ResolvedMusicStream,
    shared: Arc<SharedPlaybackState>,
    cache_state: Arc<CacheTransferState>,
    event_tx: Sender<MusicPlaybackEvent>,
) -> Result<(), String> {
    let paths = music_cache_paths(&stream)?;
    fs::create_dir_all(&paths.dir)
        .map_err(|error| format!("Could not create music stream cache: {error}"))?;
    if existing_cache_manifest_is_not_fresh(&paths) {
        let _ = fs::remove_file(&paths.media);
        let _ = fs::remove_file(&paths.cover);
        let _ = fs::remove_file(&paths.manifest);
    }
    cache_cover_image_if_needed(&stream, &paths);

    if let Some(expected) = stream.expected_bytes.filter(|value| *value > 0) {
        cache_state.set_expected_bytes(Some(expected));
    }

    let existing_bytes = fs::metadata(&paths.media)
        .map(|meta| meta.len())
        .unwrap_or(0);
    cache_state.seed_ranges(manifest_ranges_for_existing_cache(&paths, existing_bytes));

    let cached_complete = cached_media_is_complete(&paths, &stream, existing_bytes);
    if cached_complete {
        cache_state.set_complete(true);
        eprintln!(
            "[music-stream] cache hit item={} key={} bytes={}",
            stream.item_id, stream.cache_key, existing_bytes
        );
    } else if let Some(command) = stream.cache_command.take() {
        let command_line = format_process_command_line(&command);
        let downloader_paths = paths.clone();
        let downloader_state = cache_state.clone();
        let manifest_info = MusicCacheManifestInfo::from_stream(&stream);
        let log_tx = event_tx.clone();
        let log_item_id = stream.item_id;
        let log_session_id = stream.session_id;
        thread::spawn(move || {
            let result = run_yt_dlp_cache_downloader(
                command,
                downloader_paths,
                downloader_state.clone(),
                manifest_info,
                None,
            );
            let _ = log_tx.send(MusicPlaybackEvent::ToolCommandFinished {
                item_id: log_item_id,
                session_id: log_session_id,
                tool: "yt-dlp".to_owned(),
                action: "playback cache".to_owned(),
                command_line,
                success: result.is_ok(),
            });
            if let Err(error) = result {
                eprintln!("[music-stream] yt-dlp cache download failed: {error}");
                downloader_state.set_error(error);
            }
        });
    } else {
        let downloader_paths = paths.clone();
        let downloader_state = cache_state.clone();
        let fallback_stream = MusicHttpCacheDownloadInfo::from_stream(&stream);
        thread::spawn(move || {
            if let Err(error) = run_http_cache_downloader(
                fallback_stream,
                downloader_paths,
                downloader_state.clone(),
                None,
            ) {
                eprintln!("[music-stream] fallback cache download failed: {error}");
                downloader_state.set_error(error);
            }
        });
    }

    wait_for_initial_cache(&paths.media, &cache_state, &shared)?;
    if shared.stop_requested.load(Ordering::Relaxed) {
        let _ = event_tx.send(MusicPlaybackEvent::Stopped {
            item_id: stream.item_id,
            session_id: stream.session_id,
        });
        return Ok(());
    }

    eprintln!(
        "[music-stream] playback open item={} ext={} title={} cache={} direct_url_len={} headers={}",
        stream.item_id,
        stream.ext,
        stream.title,
        paths.media.display(),
        stream.direct_url.len(),
        stream.headers.len()
    );

    let mut format = probe_growing_music_format(
        &paths.media,
        &stream.ext,
        cache_state.clone(),
        shared.clone(),
    )?;

    let (track_id, mut decoder) = {
        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| "No playable audio track was found.".to_owned())?;
        let codec_params = track
            .codec_params
            .as_ref()
            .ok_or_else(|| "Audio codec parameters are missing.".to_owned())?;
        let audio_params = codec_params
            .audio()
            .ok_or_else(|| "Audio codec parameters are missing.".to_owned())?;
        let decoder = music_codec_registry()
            .make_audio_decoder(audio_params, &AudioDecoderOptions::default())
            .map_err(|error| format!("Could not create audio decoder: {error}"))?;
        (track.id, decoder)
    };

    let mut output_stream: Option<Stream> = None;
    let mut sample_buffer: Vec<f32> = Vec::new();
    let _ = event_tx.send(MusicPlaybackEvent::Started {
        item_id: stream.item_id,
        session_id: stream.session_id,
    });

    loop {
        if shared.stop_requested.load(Ordering::Relaxed) {
            let _ = event_tx.send(MusicPlaybackEvent::Stopped {
                item_id: stream.item_id,
                session_id: stream.session_id,
            });
            return Ok(());
        }

        if let Some(target_seconds) = take_seek_target_seconds(&shared) {
            if let Ok(mut buffer) = shared.buffer.lock() {
                buffer.clear();
            }
            let sample_rate = shared.sample_rate.load(Ordering::Relaxed).max(1) as f64;
            let channels = shared.channels.load(Ordering::Relaxed).max(1) as f64;
            let seek_seconds = target_seconds.max(0.0);
            if !seek_seconds.is_finite() {
                continue;
            }
            let Some(seek_time) = Time::try_from_secs_f64(seek_seconds) else {
                continue;
            };
            match format.seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time: seek_time,
                    track_id: Some(track_id),
                },
            ) {
                Ok(_) => {
                    shared.samples_played.store(
                        (seek_seconds * sample_rate * channels).round().max(0.0) as u64,
                        Ordering::Relaxed,
                    );
                }
                Err(error) => {
                    eprintln!("[music-stream] seek ignored: {error}");
                }
            }
        }

        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(_error) if shared.stop_requested.load(Ordering::Relaxed) => {
                let _ = event_tx.send(MusicPlaybackEvent::Stopped {
                    item_id: stream.item_id,
                    session_id: stream.session_id,
                });
                return Ok(());
            }
            Err(error) => return Err(format!("Could not read audio packet: {error}")),
        };

        if packet.track_id != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(error) => return Err(format!("Could not decode audio packet: {error}")),
        };

        let spec = decoded.spec();
        let source_channels = spec.channels().count().max(1);
        let source_sample_rate = spec.rate().max(1);
        let output_channels = source_channels.min(2).max(1);
        shared
            .sample_rate
            .store(source_sample_rate, Ordering::Relaxed);
        shared
            .channels
            .store(output_channels as u32, Ordering::Relaxed);

        if output_stream.is_none() {
            let stream = build_output_stream(shared.clone(), source_sample_rate, output_channels)?;
            stream
                .play()
                .map_err(|error| format!("Could not start audio output: {error}"))?;
            output_stream = Some(stream);
        }

        sample_buffer.resize(decoded.samples_interleaved(), f32::MID);
        decoded.copy_to_slice_interleaved(&mut sample_buffer);
        queue_samples(
            &shared,
            &sample_buffer,
            source_channels,
            output_channels,
            source_sample_rate,
        );
    }

    wait_for_buffer_drain(&shared);
    if !shared.stop_requested.load(Ordering::Relaxed) {
        let _ = event_tx.send(MusicPlaybackEvent::Finished {
            item_id: stream.item_id,
            session_id: stream.session_id,
        });
    }
    Ok(())
}

fn probe_growing_music_format(
    media_path: &Path,
    ext: &str,
    cache_state: Arc<CacheTransferState>,
    shared: Arc<SharedPlaybackState>,
) -> Result<Box<dyn FormatReader>, String> {
    let started = SystemTime::now();
    let mut last_logged_error = String::new();

    loop {
        if shared.stop_requested.load(Ordering::Relaxed) {
            return Err("Music playback was stopped before stream probing.".to_owned());
        }

        let source = GrowingCacheSource::open(
            media_path.to_path_buf(),
            cache_state.clone(),
            shared.clone(),
        )?;
        let mss = MediaSourceStream::new(Box::new(source), Default::default());

        let mut hint = Hint::new();
        if !ext.trim().is_empty() {
            hint.with_extension(ext.trim());
        }

        match symphonia::default::get_probe().probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        ) {
            Ok(format) => return Ok(format),
            Err(error) => {
                let message = error.to_string();
                if !music_probe_error_should_retry(&message, &cache_state, started) {
                    return Err(format!("Could not read stream format: {message}"));
                }

                if message != last_logged_error {
                    eprintln!("[music-stream] probe retry while cache grows: {message}");
                    last_logged_error = message;
                }
                thread::sleep(CACHE_WAIT_STEP);
            }
        }
    }
}

fn music_probe_error_should_retry(
    message: &str,
    cache_state: &CacheTransferState,
    started: SystemTime,
) -> bool {
    if cache_state.failed.load(Ordering::Relaxed) {
        return false;
    }
    if cache_state.complete.load(Ordering::Relaxed) {
        return false;
    }
    if started.elapsed().unwrap_or_default() >= INITIAL_CACHE_WAIT_TIMEOUT {
        return false;
    }

    let message = message.to_ascii_lowercase();
    message.contains("missing segment")
        || message.contains("unexpected eof")
        || message.contains("end of stream")
        || message.contains("eof")
        || message.contains("incomplete")
}

fn music_cache_paths(stream: &ResolvedMusicStream) -> Result<MusicCachePaths, String> {
    let key = sanitize_cache_key(&stream.cache_key);
    let ext = sanitize_cache_ext(&stream.ext);
    let dir = stream.cache_root.join(key);
    Ok(MusicCachePaths {
        media: dir.join(format!("audio.{ext}")),
        cover: dir.join("cover.img"),
        manifest: dir.join("manifest.json"),
        dir,
    })
}

fn sanitize_cache_key(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "unknown".to_owned();
    }
    trimmed
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn sanitize_cache_ext(value: &str) -> String {
    let trimmed = value.trim().trim_start_matches('.');
    if trimmed.is_empty() {
        "bin".to_owned()
    } else {
        trimmed
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase()
    }
}

#[derive(Clone)]
struct MusicCacheManifestInfo {
    item_id: u64,
    source_url: String,
    title: String,
    album_title: String,
    duration_seconds: Option<f64>,
    ext: String,
    format_id: String,
    acodec: String,
    thumbnail_url: String,
    cache_key: String,
    expected_bytes: Option<u64>,
}

impl MusicCacheManifestInfo {
    fn from_stream(stream: &ResolvedMusicStream) -> Self {
        Self {
            item_id: stream.item_id,
            source_url: stream.source_url.clone(),
            title: stream.title.clone(),
            album_title: stream.album_title.clone(),
            duration_seconds: stream.duration_seconds,
            ext: stream.ext.clone(),
            format_id: stream.format_id.clone(),
            acodec: stream.acodec.clone(),
            thumbnail_url: stream.thumbnail_url.clone(),
            cache_key: stream.cache_key.clone(),
            expected_bytes: stream.expected_bytes,
        }
    }
}

#[derive(Clone)]
struct MusicHttpCacheDownloadInfo {
    item_id: u64,
    source_url: String,
    direct_url: String,
    headers: Vec<(String, String)>,
    title: String,
    album_title: String,
    duration_seconds: Option<f64>,
    ext: String,
    format_id: String,
    acodec: String,
    thumbnail_url: String,
    cache_key: String,
    expected_bytes: Option<u64>,
}

impl MusicHttpCacheDownloadInfo {
    fn from_stream(stream: &ResolvedMusicStream) -> Self {
        Self {
            item_id: stream.item_id,
            source_url: stream.source_url.clone(),
            direct_url: stream.direct_url.clone(),
            headers: stream.headers.clone(),
            title: stream.title.clone(),
            album_title: stream.album_title.clone(),
            duration_seconds: stream.duration_seconds,
            ext: stream.ext.clone(),
            format_id: stream.format_id.clone(),
            acodec: stream.acodec.clone(),
            thumbnail_url: stream.thumbnail_url.clone(),
            cache_key: stream.cache_key.clone(),
            expected_bytes: stream.expected_bytes,
        }
    }

    fn manifest_info(&self) -> MusicCacheManifestInfo {
        MusicCacheManifestInfo {
            item_id: self.item_id,
            source_url: self.source_url.clone(),
            title: self.title.clone(),
            album_title: self.album_title.clone(),
            duration_seconds: self.duration_seconds,
            ext: self.ext.clone(),
            format_id: self.format_id.clone(),
            acodec: self.acodec.clone(),
            thumbnail_url: self.thumbnail_url.clone(),
            cache_key: self.cache_key.clone(),
            expected_bytes: self.expected_bytes,
        }
    }
}

fn existing_cache_manifest_is_not_fresh(paths: &MusicCachePaths) -> bool {
    if !paths.media.is_file() {
        return false;
    }
    let Ok(data) = fs::read_to_string(&paths.manifest) else {
        return true;
    };
    let Ok(manifest) = serde_json::from_str::<MusicCacheManifest>(&data) else {
        return true;
    };
    !cache_manifest_updated_is_fresh(manifest.updated_unix_seconds)
}

fn cached_media_is_complete(
    paths: &MusicCachePaths,
    stream: &ResolvedMusicStream,
    media_len: u64,
) -> bool {
    if media_len == 0 {
        return false;
    }
    let Ok(data) = fs::read_to_string(&paths.manifest) else {
        return false;
    };
    let Ok(manifest) = serde_json::from_str::<MusicCacheManifest>(&data) else {
        return false;
    };
    if !cache_manifest_updated_is_fresh(manifest.updated_unix_seconds) {
        return false;
    }
    let ranges = normalize_ranges(manifest.ranges);
    let ranges_cover_expected = manifest
        .expected_bytes
        .filter(|expected| *expected > 0)
        .is_some_and(|expected| {
            ranges
                .first()
                .is_some_and(|range| range.start == 0 && range.end >= expected)
        });
    manifest.complete
        && manifest.source_url == stream.source_url
        && manifest.ext == stream.ext
        && manifest
            .expected_bytes
            .map_or(true, |expected| expected <= media_len)
        && (ranges_cover_expected
            || manifest
                .expected_bytes
                .map_or(false, |expected| expected == media_len))
}

// i18n-exempt:
// Music stream/cache commands are technical evidence. Keep executable names, CLI
// options, URLs, format IDs, codecs, and paths raw for debugging/searchability.
fn format_process_command_line(command: &Command) -> String {
    let program = quote_command_arg(&command.get_program().to_string_lossy());
    let args = command
        .get_args()
        .map(|arg| quote_command_arg(&arg.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ");
    if args.is_empty() {
        program
    } else {
        format!("{program} {args}")
    }
}

fn quote_command_arg(value: &str) -> String {
    if value.contains([' ', '\t', '"']) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_owned()
    }
}

fn music_cache_cancel_requested(cancel_requested: Option<&Arc<AtomicBool>>) -> bool {
    cancel_requested.is_some_and(|flag| flag.load(Ordering::Relaxed))
}

fn run_yt_dlp_cache_downloader(
    mut command: Command,
    paths: MusicCachePaths,
    cache_state: Arc<CacheTransferState>,
    manifest_info: MusicCacheManifestInfo,
    cancel_requested: Option<Arc<AtomicBool>>,
) -> Result<(), String> {
    fs::create_dir_all(&paths.dir)
        .map_err(|error| format!("Could not create music stream cache: {error}"))?;

    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start yt-dlp music cache download: {error}"))?;

    loop {
        if music_cache_cancel_requested(cancel_requested.as_ref()) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Music cache download cancelled.".to_owned());
        }
        update_cache_progress_from_file(&paths.media, &cache_state, manifest_info.expected_bytes);
        match child
            .try_wait()
            .map_err(|error| format!("Could not poll yt-dlp music cache download: {error}"))?
        {
            Some(status) => {
                update_cache_progress_from_file(
                    &paths.media,
                    &cache_state,
                    manifest_info.expected_bytes,
                );
                let final_len = fs::metadata(&paths.media)
                    .map(|meta| meta.len())
                    .unwrap_or(0);
                if status.success() {
                    let expected = cache_state
                        .expected_bytes()
                        .or(manifest_info.expected_bytes)
                        .or_else(|| (final_len > 0).then_some(final_len));
                    cache_state.set_expected_bytes(expected);
                    if final_len > 0 {
                        cache_state.seed_ranges(vec![MusicCacheRange {
                            start: 0,
                            end: final_len,
                        }]);
                    }
                    cache_state.set_complete(final_len > 0);
                    write_cache_manifest(
                        &paths,
                        &manifest_info,
                        final_len > 0,
                        cache_state.expected_bytes(),
                        cache_state.ranges_snapshot(),
                    )?;
                    eprintln!(
                        "[music-stream] yt-dlp cache complete item={} key={} bytes={}",
                        manifest_info.item_id, manifest_info.cache_key, final_len
                    );
                    return Ok(());
                }

                let mut stderr_text = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = stderr.read_to_string(&mut stderr_text);
                }
                let detail = stderr_text.trim();
                let message = if detail.is_empty() {
                    format!(
                        "yt-dlp music cache download failed: exit code {:?}",
                        status.code()
                    )
                } else {
                    format!("yt-dlp music cache download failed: {detail}")
                };
                let _ = write_cache_manifest(
                    &paths,
                    &manifest_info,
                    false,
                    cache_state
                        .expected_bytes()
                        .or(manifest_info.expected_bytes),
                    cache_state.ranges_snapshot(),
                );
                return Err(message);
            }
            None => thread::sleep(CACHE_WAIT_STEP),
        }
    }
}

fn update_cache_progress_from_file(
    media_path: &Path,
    cache_state: &CacheTransferState,
    expected_bytes: Option<u64>,
) {
    if let Some(expected) = expected_bytes.filter(|value| *value > 0) {
        cache_state.set_expected_bytes(Some(expected));
    }
    if let Ok(metadata) = fs::metadata(media_path) {
        let len = metadata.len();
        if len > 0 {
            cache_state.seed_ranges(vec![MusicCacheRange { start: 0, end: len }]);
            if cache_state
                .expected_bytes()
                .is_some_and(|expected| len >= expected)
            {
                cache_state.set_complete(true);
            }
        }
    }
}

fn run_http_cache_downloader(
    stream: MusicHttpCacheDownloadInfo,
    paths: MusicCachePaths,
    cache_state: Arc<CacheTransferState>,
    cancel_requested: Option<Arc<AtomicBool>>,
) -> Result<(), String> {
    fs::create_dir_all(&paths.dir)
        .map_err(|error| format!("Could not create music stream cache: {error}"))?;
    let manifest_info = stream.manifest_info();

    let mut retry_count = 0_u32;
    loop {
        if music_cache_cancel_requested(cancel_requested.as_ref()) {
            return Err("Music cache download cancelled.".to_owned());
        }
        if cache_state.is_fully_cached() {
            cache_state.set_complete(true);
            write_cache_manifest(
                &paths,
                &manifest_info,
                true,
                cache_state.expected_bytes(),
                cache_state.ranges_snapshot(),
            )?;
            eprintln!(
                "[music-stream] cache complete item={} key={} bytes={}",
                stream.item_id,
                stream.cache_key,
                cache_state.downloaded_bytes.load(Ordering::Relaxed)
            );
            return Ok(());
        }

        // Cache growth is intentionally contiguous. Until the file is fully cached,
        // the UI clamps seek targets to the cached range instead of requesting
        // random HTTP ranges from providers that may ignore or mishandle Range.
        let start_offset = cache_state.contiguous_end_from_start();
        if cache_state
            .expected_bytes()
            .is_some_and(|expected| start_offset >= expected)
        {
            cache_state.set_complete(cache_state.is_fully_cached());
            write_cache_manifest(
                &paths,
                &manifest_info,
                cache_state.complete.load(Ordering::Relaxed),
                cache_state.expected_bytes(),
                cache_state.ranges_snapshot(),
            )?;
            return Ok(());
        }

        match download_cache_range(
            &stream,
            &paths,
            &cache_state,
            start_offset,
            cancel_requested.as_ref(),
        ) {
            Ok(DownloadRangeOutcome::CompletedRange) => {
                retry_count = 0;
                write_cache_manifest(
                    &paths,
                    &manifest_info,
                    cache_state.is_fully_cached(),
                    cache_state.expected_bytes(),
                    cache_state.ranges_snapshot(),
                )?;
                if cache_state.is_fully_cached() || cache_state.expected_bytes().is_none() {
                    cache_state.set_complete(true);
                    write_cache_manifest(
                        &paths,
                        &manifest_info,
                        true,
                        cache_state.expected_bytes(),
                        cache_state.ranges_snapshot(),
                    )?;
                    eprintln!(
                        "[music-stream] cache complete item={} key={} bytes={}",
                        stream.item_id,
                        stream.cache_key,
                        cache_state.downloaded_bytes.load(Ordering::Relaxed)
                    );
                    return Ok(());
                }
            }
            Err(error) => {
                let _ = write_cache_manifest(
                    &paths,
                    &manifest_info,
                    false,
                    cache_state.expected_bytes(),
                    cache_state.ranges_snapshot(),
                );
                let cached = cache_state.contiguous_end_from_start();
                if cached > 0 && retry_count < 8 {
                    retry_count += 1;
                    eprintln!(
                        "[music-stream] cache download interrupted; retrying ({retry_count}/8): {error}"
                    );
                    if music_cache_cancel_requested(cancel_requested.as_ref()) {
                        return Err("Music cache download cancelled.".to_owned());
                    }
                    thread::sleep(Duration::from_millis(700));
                    continue;
                }
                return Err(error);
            }
        }
    }
}

enum DownloadRangeOutcome {
    CompletedRange,
}

fn download_cache_range(
    stream: &MusicHttpCacheDownloadInfo,
    paths: &MusicCachePaths,
    cache_state: &CacheTransferState,
    start_offset: u64,
    cancel_requested: Option<&Arc<AtomicBool>>,
) -> Result<DownloadRangeOutcome, String> {
    if music_cache_cancel_requested(cancel_requested) {
        return Err("Music cache download cancelled.".to_owned());
    }
    let mut request = ureq::get(&stream.direct_url);
    for (name, value) in &stream.headers {
        if !name.trim().is_empty() && !value.trim().is_empty() {
            request = request.header(name.trim(), value.trim());
        }
    }
    if start_offset > 0 {
        request = request.header("Range", format!("bytes={start_offset}-"));
    }

    let response = request
        .call()
        .map_err(|error| format!("Could not open audio stream cache download: {error}"))?;
    let status = response.status().as_u16();
    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    let content_range = response
        .headers()
        .get("content-range")
        .and_then(|value| value.to_str().ok());
    if let Some(total) =
        total_length_from_headers(status, start_offset, content_length, content_range)
    {
        cache_state.set_expected_bytes(Some(total));
    }

    let range_start = if start_offset > 0 && status != 206 {
        // Server ignored Range. Treat the response as a full-file restart from byte 0.
        cache_state.seed_ranges(Vec::new());
        0
    } else {
        start_offset
    };

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&paths.media)
        .map_err(|error| format!("Could not open music cache file: {error}"))?;
    file.seek(SeekFrom::Start(range_start))
        .map_err(|error| format!("Could not seek music cache file: {error}"))?;
    if range_start == 0 && start_offset > 0 && status != 206 {
        file.set_len(0)
            .map_err(|error| format!("Could not reset music cache file: {error}"))?;
    }

    let mut reader = response.into_parts().1.into_reader();
    let mut buffer = vec![0_u8; HTTP_READ_BUFFER_SIZE];
    let mut cursor = range_start;

    loop {
        if music_cache_cancel_requested(cancel_requested) {
            return Err("Music cache download cancelled.".to_owned());
        }
        let read = reader
            .read(&mut buffer)
            .map_err(|error| format!("Could not read audio stream cache: {error}"))?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|error| format!("Could not write music cache file: {error}"))?;
        let next_cursor = cursor.saturating_add(read as u64);
        cache_state.add_range(cursor, next_cursor);
        cursor = next_cursor;
    }

    let _ = file.flush();
    Ok(DownloadRangeOutcome::CompletedRange)
}

fn total_length_from_headers(
    status: u16,
    start_offset: u64,
    content_length: Option<u64>,
    content_range: Option<&str>,
) -> Option<u64> {
    if status == 206 {
        if let Some(total) = content_range.and_then(parse_content_range_total) {
            return Some(total);
        }
        return content_length.map(|len| start_offset.saturating_add(len));
    }
    content_length
}

fn parse_content_range_total(value: &str) -> Option<u64> {
    let (_, total) = value.rsplit_once('/')?;
    total.trim().parse::<u64>().ok()
}

fn write_cache_manifest(
    paths: &MusicCachePaths,
    info: &MusicCacheManifestInfo,
    complete: bool,
    expected_bytes: Option<u64>,
    ranges: Vec<MusicCacheRange>,
) -> Result<(), String> {
    let ranges = normalize_ranges(ranges);
    let manifest = MusicCacheManifest {
        source_url: info.source_url.clone(),
        title: info.title.clone(),
        album_title: info.album_title.clone(),
        duration_seconds: info.duration_seconds,
        ext: info.ext.clone(),
        format_id: info.format_id.clone(),
        acodec: info.acodec.clone(),
        thumbnail_url: info.thumbnail_url.clone(),
        cover_file: "cover.img".to_owned(),
        expected_bytes,
        downloaded_bytes: total_range_bytes(&ranges),
        ranges,
        complete,
        updated_unix_seconds: unix_seconds_now(),
    };
    let data = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("Could not encode music cache manifest: {error}"))?;
    fs::write(&paths.manifest, data)
        .map_err(|error| format!("Could not write music cache manifest: {error}"))
}

fn cache_cover_image_if_needed(stream: &ResolvedMusicStream, paths: &MusicCachePaths) {
    let url = stream.thumbnail_url.trim();
    if url.is_empty() || paths.cover.exists() {
        return;
    }
    let cover_path = paths.cover.clone();
    let url = url.to_owned();
    thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let mut response = ureq::get(&url)
                .call()
                .map_err(|error| format!("Could not download cover image: {error}"))?;
            let status = response.status().as_u16();
            if status >= 400 {
                return Err(format!("Could not download cover image: HTTP {status}"));
            }
            let mut reader = response.body_mut().as_reader();
            let mut data = Vec::new();
            reader
                .read_to_end(&mut data)
                .map_err(|error| format!("Could not read cover image: {error}"))?;
            if data.is_empty() {
                return Err("Cover image response was empty.".to_owned());
            }
            fs::write(&cover_path, data)
                .map_err(|error| format!("Could not write cover image cache: {error}"))
        })();
        if let Err(error) = result {
            eprintln!("[music-stream] cover cache skipped: {error}");
        }
    });
}

fn unix_seconds_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn cache_manifest_updated_is_fresh(updated_unix_seconds: u64) -> bool {
    updated_unix_seconds > 0
        && unix_seconds_now().saturating_sub(updated_unix_seconds) <= MUSIC_STREAM_CACHE_TTL_SECONDS
}

fn normalize_ranges(mut ranges: Vec<MusicCacheRange>) -> Vec<MusicCacheRange> {
    ranges.retain(|range| range.end > range.start);
    ranges.sort_by_key(|range| (range.start, range.end));
    let mut merged: Vec<MusicCacheRange> = Vec::new();
    for range in ranges {
        if let Some(last) = merged.last_mut() {
            if range.start <= last.end {
                last.end = last.end.max(range.end);
                continue;
            }
        }
        merged.push(range);
    }
    merged
}

fn total_range_bytes(ranges: &[MusicCacheRange]) -> u64 {
    ranges
        .iter()
        .map(|range| range.end.saturating_sub(range.start))
        .sum()
}

fn manifest_ranges_for_existing_cache(
    paths: &MusicCachePaths,
    media_len: u64,
) -> Vec<MusicCacheRange> {
    if media_len == 0 {
        return Vec::new();
    }
    if let Ok(data) = fs::read_to_string(&paths.manifest) {
        if let Ok(manifest) = serde_json::from_str::<MusicCacheManifest>(&data) {
            let ranges = normalize_ranges(manifest.ranges);
            if !ranges.is_empty() {
                return ranges;
            }
        }
    }
    if media_len > 0 {
        vec![MusicCacheRange {
            start: 0,
            end: media_len,
        }]
    } else {
        Vec::new()
    }
}

fn wait_for_initial_cache(
    _path: &Path,
    cache_state: &CacheTransferState,
    shared: &SharedPlaybackState,
) -> Result<(), String> {
    let started = SystemTime::now();
    loop {
        if shared.stop_requested.load(Ordering::Relaxed) {
            return Ok(());
        }
        let available = cache_state.available_end_from(0).unwrap_or(0);
        let target = cache_state
            .expected_bytes()
            .map(|expected| expected.min(INITIAL_CACHE_BUFFER_BYTES))
            .unwrap_or(INITIAL_CACHE_BUFFER_BYTES);
        if available > 0 && (available >= target || cache_state.complete.load(Ordering::Relaxed)) {
            return Ok(());
        }
        if cache_state.failed.load(Ordering::Relaxed) && available == 0 {
            return Err(cache_state
                .error_text()
                .unwrap_or_else(|| "Music stream cache download failed.".to_owned()));
        }
        if started.elapsed().unwrap_or_default() >= INITIAL_CACHE_WAIT_TIMEOUT && available > 0 {
            return Ok(());
        }
        thread::sleep(CACHE_WAIT_STEP);
    }
}

struct GrowingCacheSource {
    path: PathBuf,
    file: File,
    position: u64,
    cache_state: Arc<CacheTransferState>,
    shared: Arc<SharedPlaybackState>,
}

impl GrowingCacheSource {
    fn open(
        path: PathBuf,
        cache_state: Arc<CacheTransferState>,
        shared: Arc<SharedPlaybackState>,
    ) -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|error| format!("Could not open music stream cache for playback: {error}"))?;
        Ok(Self {
            path,
            file,
            position: 0,
            cache_state,
            shared,
        })
    }
}

impl MediaSource for GrowingCacheSource {
    fn is_seekable(&self) -> bool {
        // Expose random seek only after the cache is complete. Some providers
        // do not provide stable Range semantics, and MP4/M4A readers may seek
        // near the tail while probing. During cache growth, UI drags are
        // allowed visually, but decoder-level seek remains best-effort/ignored.
        self.cache_state.complete.load(Ordering::Relaxed)
    }

    fn byte_len(&self) -> Option<u64> {
        if self.cache_state.complete.load(Ordering::Relaxed) {
            return self
                .cache_state
                .expected_bytes()
                .or_else(|| fs::metadata(&self.path).map(|meta| meta.len()).ok());
        }

        // While the file is still growing, do not expose the current on-disk
        // length as the final media length. Container readers such as WebM/MKV
        // may treat a short temporary length as EOF during probing and fail with
        // errors like "mkv: missing segment element" even though yt-dlp will
        // finish writing a valid file a moment later.
        None
    }
}

impl Read for GrowingCacheSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            if self.shared.stop_requested.load(Ordering::Relaxed) {
                return Ok(0);
            }
            if let Some(available_end) = self.cache_state.available_end_from(self.position) {
                self.file.seek(SeekFrom::Start(self.position))?;
                let max_read = ((available_end - self.position) as usize).min(buf.len());
                let read = self.file.read(&mut buf[..max_read])?;
                self.position = self.position.saturating_add(read as u64);
                return Ok(read);
            }
            if self.cache_state.complete.load(Ordering::Relaxed) {
                return Ok(0);
            }
            if self.cache_state.failed.load(Ordering::Relaxed) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    self.cache_state
                        .error_text()
                        .unwrap_or_else(|| "music stream cache failed".to_owned()),
                ));
            }
            thread::sleep(CACHE_WAIT_STEP);
        }
    }
}

impl Seek for GrowingCacheSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let next = match pos {
            SeekFrom::Start(value) => value,
            SeekFrom::Current(offset) => {
                if offset.is_negative() {
                    self.position.saturating_sub(offset.unsigned_abs())
                } else {
                    self.position.saturating_add(offset as u64)
                }
            }
            SeekFrom::End(offset) => {
                let len =
                    self.cache_state
                        .expected_bytes()
                        .or_else(|| {
                            self.cache_state.complete.load(Ordering::Relaxed).then(|| {
                                fs::metadata(&self.path).map(|meta| meta.len()).unwrap_or(0)
                            })
                        })
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::Unsupported,
                                "music cache length is unknown before completion",
                            )
                        })?;
                if offset.is_negative() {
                    len.saturating_sub(offset.unsigned_abs())
                } else {
                    len.saturating_add(offset as u64)
                }
            }
        };
        if !self.cache_state.complete.load(Ordering::Relaxed) {
            let available = self.cache_state.contiguous_end_from_start();
            self.position = next.min(available);
            return Ok(self.position);
        }
        self.position = next;
        Ok(self.position)
    }
}

fn take_seek_target_seconds(shared: &SharedPlaybackState) -> Option<f64> {
    let millis = shared
        .seek_target_millis
        .swap(NO_SEEK_MILLIS, Ordering::Relaxed);
    (millis != NO_SEEK_MILLIS).then_some(millis as f64 / 1000.0)
}

fn build_output_stream(
    shared: Arc<SharedPlaybackState>,
    sample_rate: u32,
    channels: usize,
) -> Result<Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "No default audio output device was found.".to_owned())?;
    let supported = device
        .default_output_config()
        .map_err(|error| format!("Could not read default audio output config: {error}"))?;
    let sample_format = supported.sample_format();
    let mut config: cpal::StreamConfig = supported.into();
    config.channels = channels.clamp(1, u16::MAX as usize) as u16;
    config.sample_rate = sample_rate;

    match sample_format {
        SampleFormat::F32 => {
            let shared = shared.clone();
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _| write_output_f32(data, &shared),
                    move |error| eprintln!("[music-stream] output error: {error}"),
                    None,
                )
                .map_err(|error| format!("Could not build f32 audio output: {error}"))
        }
        SampleFormat::I16 => {
            let shared = shared.clone();
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [i16], _| write_output_i16(data, &shared),
                    move |error| eprintln!("[music-stream] output error: {error}"),
                    None,
                )
                .map_err(|error| format!("Could not build i16 audio output: {error}"))
        }
        SampleFormat::U16 => {
            let shared = shared.clone();
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [u16], _| write_output_u16(data, &shared),
                    move |error| eprintln!("[music-stream] output error: {error}"),
                    None,
                )
                .map_err(|error| format!("Could not build u16 audio output: {error}"))
        }
        other => Err(format!("Unsupported output sample format: {other:?}")),
    }
}

fn write_output_f32(data: &mut [f32], shared: &SharedPlaybackState) {
    write_output_samples(data, shared, |value| value);
}

fn write_output_i16(data: &mut [i16], shared: &SharedPlaybackState) {
    write_output_samples(data, shared, |value| {
        (value.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
    });
}

fn write_output_u16(data: &mut [u16], shared: &SharedPlaybackState) {
    write_output_samples(data, shared, |value| {
        (((value.clamp(-1.0, 1.0) + 1.0) * 0.5) * u16::MAX as f32) as u16
    });
}

fn write_output_samples<T>(data: &mut [T], shared: &SharedPlaybackState, convert: impl Fn(f32) -> T)
where
    T: Copy,
{
    let stopped = shared.stop_requested.load(Ordering::Relaxed);
    let paused = shared.paused.load(Ordering::Relaxed);
    let volume = f32::from_bits(shared.volume_bits.load(Ordering::Relaxed));
    let silence = convert(0.0);

    if stopped || paused {
        data.fill(silence);
        return;
    }

    let mut consumed = 0_u64;
    if let Ok(mut buffer) = shared.buffer.lock() {
        for out in data.iter_mut() {
            if let Some(sample) = buffer.pop_front() {
                *out = convert(sample * volume);
                consumed += 1;
            } else {
                *out = silence;
            }
        }
    } else {
        data.fill(silence);
    }

    if consumed > 0 {
        shared.samples_played.fetch_add(consumed, Ordering::Relaxed);
    }
}

fn queue_samples(
    shared: &SharedPlaybackState,
    samples: &[f32],
    source_channels: usize,
    output_channels: usize,
    sample_rate: u32,
) {
    let max_buffered_samples = sample_rate as usize * output_channels * 10;
    while !shared.stop_requested.load(Ordering::Relaxed)
        && shared
            .buffer
            .lock()
            .map(|buffer| buffer.len() > max_buffered_samples)
            .unwrap_or(false)
    {
        thread::sleep(Duration::from_millis(20));
    }

    if let Ok(mut buffer) = shared.buffer.lock() {
        for frame in samples.chunks(source_channels.max(1)) {
            for channel in 0..output_channels {
                let sample = if source_channels == 1 {
                    frame.first().copied().unwrap_or(0.0)
                } else {
                    frame
                        .get(channel)
                        .copied()
                        .or_else(|| frame.last().copied())
                        .unwrap_or(0.0)
                };
                buffer.push_back(sample);
            }
        }
    }
}

fn wait_for_buffer_drain(shared: &SharedPlaybackState) {
    while !shared.stop_requested.load(Ordering::Relaxed)
        && shared
            .buffer
            .lock()
            .map(|buffer| !buffer.is_empty())
            .unwrap_or(false)
    {
        thread::sleep(Duration::from_millis(40));
    }
}
