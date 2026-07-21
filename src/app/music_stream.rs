use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

use crate::app::music_analysis::spawn_music_analysis_if_needed;
use crate::app::music_mix_timeline::{
    MusicMixFrameClock, MusicMixFrameCount, MusicMixOutputFrame, MusicMixSourceFrame,
    MusicMixSourcePosition, source_position_after_rendered_frames,
};
use crate::app::music_pcm_reservoir::{MusicPcmReservoir, MusicPcmReservoirCoverage};
use crate::infrastructure::yaml_store::{read_yaml_file, write_yaml_file};

const INITIAL_CACHE_BUFFER_BYTES: u64 = 384 * 1024;
const INITIAL_CACHE_WAIT_TIMEOUT: Duration = Duration::from_secs(20);
const CACHE_WAIT_STEP: Duration = Duration::from_millis(40);
const HTTP_READ_BUFFER_SIZE: usize = 128 * 1024;
const NO_SEEK_MILLIS: u64 = u64::MAX;
const MUSIC_STREAM_CACHE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;
const MUSIC_MIX_NORMAL_ANCHOR_MIN_SECONDS: f64 = 0.34;
const MUSIC_MIX_NORMAL_ANCHOR_MAX_SECONDS: f64 = 0.82;
const MUSIC_MIX_NORMAL_ANCHOR_RATIO: f64 = 0.15;
const MUSIC_MIX_OUTGOING_NORMAL_EDGE_RATIO: f64 = 0.22;
const MUSIC_MIX_TEMPO_JND_RATE: f64 = 0.0065;
const MUSIC_MIX_TEMPO_DEADBAND_RATE: f64 = 0.0052;
const MUSIC_MIX_TEMPO_SOFT_KNEE_RATE: f64 = 0.018;
const MUSIC_MIX_TEMPO_FEATHER_MIN_GAP: f64 = 0.0014;
const MUSIC_MIX_TEMPO_FEATHER_MAX_STEP: f64 = 0.0026;
const MUSIC_MIX_MICRO_STRETCH_BYPASS_SECONDS: f64 = 1.35;
const MUSIC_BRIDGE_PREVIEW_AUDIBLE_RMS: f32 = 0.0008;
const MUSIC_BRIDGE_WEAK_A_TAIL_RMS: f32 = 0.0014;
const MUSIC_LONG_BRIDGE_COMPLEMENTARY_A_FADE_MIN_MILLIS: u64 = 6000;
const MUSIC_ENERGY_DUCK_A_BED_MIN_TRANSITION_MILLIS: u64 = 6000;
const MUSIC_ENERGY_DUCK_A_BED_START_PHASE: f32 = 0.30;
const MUSIC_ENERGY_DUCK_A_BED_FULL_PHASE: f32 = 0.70;
const MUSIC_ENERGY_DUCK_A_BED_RATIO_START: f32 = 0.75;
const MUSIC_ENERGY_DUCK_A_BED_RATIO_FULL: f32 = 1.85;
const MUSIC_ENERGY_DUCK_A_BED_MAX_DEPTH: f32 = 0.18;
const MUSIC_ENERGY_DUCK_A_BED_MIN_FLOOR: f32 = 0.42;
const MUSIC_A_BED_SAFETY_CENTER_START_PHASE: f32 = 0.24;
const MUSIC_A_BED_SAFETY_CENTER_FULL_PHASE: f32 = 0.42;
const MUSIC_A_BED_SAFETY_RELEASE_START_PHASE: f32 = 0.72;
const MUSIC_A_BED_SAFETY_RELEASE_END_PHASE: f32 = 0.96;
const MUSIC_A_BED_SAFETY_WEAK_FLOOR: f32 = 0.62;
const MUSIC_A_BED_SAFETY_STRONG_FLOOR: f32 = 0.48;
const MUSIC_A_BED_BOUNDARY_CUSHION_MIN_TRANSITION_MILLIS: u64 = 6000;
const MUSIC_A_BED_BOUNDARY_CUSHION_MIN_PHASE: f32 = 0.38;
const MUSIC_A_BED_BOUNDARY_CUSHION_MAX_PHASE: f32 = 0.84;
const MUSIC_A_BED_BOUNDARY_CUSHION_WIDTH_PHASE: f32 = 0.17;
const MUSIC_A_BED_BOUNDARY_CUSHION_WEAK_FLOOR: f32 = 0.70;
const MUSIC_A_BED_BOUNDARY_CUSHION_STRONG_FLOOR: f32 = 0.54;
const MUSIC_A_TAIL_RELEASE_RATIO: f32 = 0.45;
const MUSIC_A_TAIL_RELEASE_MIN_SECONDS: f32 = 2.5;
const MUSIC_A_TAIL_RELEASE_MAX_SECONDS: f32 = 5.0;
const MUSIC_A_TAIL_RELEASE_WEAK_B_BONUS: f32 = 0.14;
const MUSIC_A_TAIL_RELEASE_STRONG_B_SHAVE: f32 = 0.10;
const MUSIC_A_TAIL_RELEASE_MIN_START_PHASE: f32 = 0.42;
const MUSIC_A_TAIL_RELEASE_MAX_START_PHASE: f32 = 0.68;
const MUSIC_TRANSITION_LOAD_DIAG_AFTER_MILLIS: u64 = 2_500;
const MUSIC_TRANSITION_LOAD_LATE_GRACE_MILLIS: u64 = 18;
const MUSIC_TRANSITION_LOAD_MIN_GAP_FACTOR: u64 = 3;
const MUSIC_PREPARED_MIX_LIVE_A_MIN_HEAD_MILLIS: u64 = 160;
const MUSIC_PREPARED_MIX_LIVE_A_HEAD_BLEND_MILLIS: u64 = 72;
const MUSIC_PREPARED_MIX_EDGE_GUARD_MILLIS: u64 = 420;
const MUSIC_PREPARED_MIX_LATE_GUARD_PAD_MILLIS: u64 = 96;
const MUSIC_PCM_RESERVOIR_SECONDS: u64 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicMixRenderMode {
    HighQualityOffline,
    Streaming,
}

impl MusicMixRenderMode {
    fn uses_high_quality_offline(self) -> bool {
        matches!(self, Self::HighQualityOffline)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::HighQualityOffline => "HQ Mix",
            Self::Streaming => "Stream Mix",
        }
    }

    pub fn detail_label(self) -> &'static str {
        match self {
            Self::HighQualityOffline => "HQ offline render",
            Self::Streaming => "streaming render",
        }
    }
}

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
            .volume_fade_started_output_frame
            .store(0, Ordering::Relaxed);
        self.shared
            .volume_fade_duration_frames
            .store(0, Ordering::Relaxed);
        self.shared
            .volume_fade_curve_bits
            .store(0.0_f32.to_bits(), Ordering::Relaxed);
        self.shared
            .volume_bits
            .store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn fade_volume_to(&self, target_volume: f32, duration: Duration) -> MusicMixOutputFrame {
        self.fade_volume_to_with_curve(target_volume, duration, 0.0)
    }

    pub(crate) fn fade_volume_to_with_curve(
        &self,
        target_volume: f32,
        duration: Duration,
        mix_curve: f32,
    ) -> MusicMixOutputFrame {
        let started_output_frame = self.output_frame_cursor();
        let duration_frames = self.mix_frame_count_from_duration(duration);
        self.fade_volume_to_from_output_frame_with_curve(
            target_volume,
            duration_frames,
            started_output_frame,
            mix_curve,
        )
    }

    pub(crate) fn fade_volume_to_from_output_frame(
        &self,
        target_volume: f32,
        duration_frames: MusicMixFrameCount,
        started_output_frame: MusicMixOutputFrame,
    ) -> MusicMixOutputFrame {
        self.fade_volume_to_from_output_frame_with_curve(
            target_volume,
            duration_frames,
            started_output_frame,
            0.0,
        )
    }

    pub(crate) fn fade_volume_to_from_output_frame_with_curve(
        &self,
        target_volume: f32,
        duration_frames: MusicMixFrameCount,
        started_output_frame: MusicMixOutputFrame,
        mix_curve: f32,
    ) -> MusicMixOutputFrame {
        // Codex handoff: callers that coordinate this fade with a preview deck
        // must pass the deck's exact start frame. Sampling the cursor again here
        // lets UI/worker delay shift A's fade relative to B's envelope.
        let target_volume = target_volume.clamp(0.0, 1.0);
        let current_volume = effective_output_volume(&self.shared);
        self.shared
            .volume_fade_from_bits
            .store(current_volume.to_bits(), Ordering::Relaxed);
        self.shared
            .volume_fade_to_bits
            .store(target_volume.to_bits(), Ordering::Relaxed);
        self.shared
            .volume_fade_duration_frames
            .store(duration_frames.get().max(1), Ordering::Relaxed);
        self.shared
            .volume_fade_started_output_frame
            .store(started_output_frame.get(), Ordering::Relaxed);
        self.shared
            .volume_fade_curve_bits
            .store(mix_curve.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        started_output_frame
    }

    pub(crate) fn start_outgoing_tempo_transition_from_output_frame(
        &self,
        target_rate: f64,
        duration_frames: MusicMixFrameCount,
        started_output_frame: MusicMixOutputFrame,
    ) {
        let target_rate = target_rate.clamp(0.92, 1.08);
        if (target_rate - 1.0).abs() < MUSIC_MIX_TEMPO_DEADBAND_RATE || duration_frames.get() == 0 {
            self.shared
                .outgoing_transition_rate_bits
                .store(1.0_f64.to_bits(), Ordering::Relaxed);
            self.shared
                .outgoing_transition_started_output_frame
                .store(0, Ordering::Relaxed);
            self.shared
                .outgoing_transition_duration_frames
                .store(0, Ordering::Relaxed);
            self.shared
                .outgoing_transition_phase_bits
                .store(0.0_f64.to_bits(), Ordering::Relaxed);
            return;
        }

        self.shared
            .outgoing_transition_rate_bits
            .store(target_rate.to_bits(), Ordering::Relaxed);
        self.shared
            .outgoing_transition_started_output_frame
            .store(started_output_frame.get(), Ordering::Relaxed);
        self.shared
            .outgoing_transition_duration_frames
            .store(duration_frames.get().max(1), Ordering::Relaxed);
        self.shared
            .outgoing_transition_phase_bits
            .store(0.0_f64.to_bits(), Ordering::Relaxed);
    }

    pub fn output_sample_rate(&self) -> u32 {
        self.shared.sample_rate.load(Ordering::Relaxed).max(1)
    }

    pub fn output_channels(&self) -> usize {
        self.shared.channels.load(Ordering::Relaxed).max(1) as usize
    }

    pub(crate) fn output_frame_cursor(&self) -> MusicMixOutputFrame {
        MusicMixOutputFrame::new(self.shared.output_frames_rendered.load(Ordering::Relaxed))
    }

    pub(crate) fn mix_frame_count_from_duration(&self, duration: Duration) -> MusicMixFrameCount {
        MusicMixFrameClock::new(self.output_sample_rate()).frame_count_from_duration(duration)
    }

    pub(crate) fn mix_frame_count_from_seconds(&self, seconds: f64) -> MusicMixFrameCount {
        MusicMixFrameClock::new(self.output_sample_rate()).frame_count_from_seconds(seconds)
    }

    pub fn start_crossfade_preview(
        &self,
        samples: Vec<f32>,
        transition_frames: MusicMixFrameCount,
        target_volume: f32,
        track_start_source_frame: MusicMixSourceFrame,
        source_sample_rate: u32,
        transition_source_frames: MusicMixFrameCount,
        track_duration_seconds: Option<f64>,
        outgoing_transition_rate: f64,
        outgoing_highlight_end_phase: Option<f32>,
    ) -> Option<MusicMixOutputFrame> {
        if samples.is_empty() || transition_frames.is_zero() {
            return None;
        }
        let duration_millis = (MusicMixFrameClock::new(self.output_sample_rate())
            .seconds_from_frame_count(transition_frames)
            * 1000.0)
            .round()
            .clamp(1.0, u64::MAX as f64) as u64;
        reset_transition_load_diagnostics(&self.shared, duration_millis);
        let outgoing_rate = outgoing_transition_rate.clamp(0.92, 1.08);
        let transition_start_frame = self.output_frame_cursor();
        self.shared
            .outgoing_transition_rate_bits
            .store(outgoing_rate.to_bits(), Ordering::Relaxed);
        self.shared
            .outgoing_transition_started_output_frame
            .store(transition_start_frame.get(), Ordering::Relaxed);
        self.shared
            .outgoing_transition_duration_frames
            .store(transition_frames.get(), Ordering::Relaxed);
        self.shared
            .outgoing_transition_phase_bits
            .store(0.0_f64.to_bits(), Ordering::Relaxed);
        let deck = CrossfadePreviewDeck {
            mode: CrossfadeDeckMode::RealtimePreview,
            buffer: samples.into_iter().collect(),
            transition_output_frames: transition_frames,
            target_volume: target_volume.clamp(0.0, 1.0),
            track_start_source_frame,
            source_sample_rate: source_sample_rate.max(1),
            transition_source_frames,
            prepared_mix_resume_source_frame: None,
            prepared_mix_start_seconds: None,
            prepared_mix_alignment_applied: true,
            track_duration_seconds: track_duration_seconds
                .filter(|duration| duration.is_finite() && *duration > 0.0),
            output_frames_consumed: MusicMixFrameCount::ZERO,
            rendered_frames_consumed: MusicMixFrameCount::ZERO,
            release_started_output_frame: None,
            release_duration_frames: MusicMixFrameCount::ZERO,
            release_from_gain: 0.0,
            outgoing_transition_rate: 1.0,
            outgoing_transition_phase_frames: 0.0,
            outgoing_transition_started_output_frame: None,
            outgoing_transition_duration_frames: MusicMixFrameCount::ZERO,
            outgoing_highlight_end_phase: outgoing_highlight_end_phase
                .filter(|phase| phase.is_finite())
                .map(|phase| phase.clamp(0.0, 1.0)),
        };
        self.shared
            .crossfade_decks
            .lock()
            .map(|mut decks| {
                if let Some(main) = decks.main.as_mut() {
                    main.outgoing_transition_rate = outgoing_rate;
                    main.outgoing_transition_phase_frames = 0.0;
                    main.outgoing_transition_started_output_frame =
                        Some(main.output_frames_consumed);
                    main.outgoing_transition_duration_frames = transition_frames;
                }
                decks.next = Some(deck);
                transition_start_frame
            })
            .ok()
    }

    pub fn start_prepared_mix_handoff(
        &self,
        samples: Vec<f32>,
        prepared_mix_b_samples: Option<Vec<f32>>,
        mix_frames: MusicMixFrameCount,
        target_volume: f32,
        b_resume_source_frame: MusicMixSourceFrame,
        source_sample_rate: u32,
        track_duration_seconds: Option<f64>,
        prepared_mix_start_seconds: Option<f64>,
    ) -> Option<MusicMixOutputFrame> {
        if samples.is_empty() || mix_frames.is_zero() {
            return None;
        }
        let handoff_started = Instant::now();
        let output_sample_rate = self.output_sample_rate();
        let output_channels = self.output_channels();
        let duration_millis = (MusicMixFrameClock::new(output_sample_rate)
            .seconds_from_frame_count(mix_frames)
            * 1000.0)
            .round()
            .clamp(1.0, u64::MAX as f64) as u64;
        reset_transition_load_diagnostics(&self.shared, duration_millis);
        self.shared
            .outgoing_transition_rate_bits
            .store(1.0_f64.to_bits(), Ordering::Relaxed);
        self.shared
            .outgoing_transition_started_output_frame
            .store(0, Ordering::Relaxed);
        self.shared
            .outgoing_transition_duration_frames
            .store(0, Ordering::Relaxed);
        self.shared
            .outgoing_transition_phase_bits
            .store(0.0_f64.to_bits(), Ordering::Relaxed);
        let (mut reservoir_snapshot, pcm_reservoir_status) = if prepared_mix_b_samples.is_some() {
            snapshot_prepared_mix_pcm_reservoir_window(
                &self.shared,
                self.item_id,
                self.session_id,
                prepared_mix_start_seconds,
                output_sample_rate,
                output_channels,
                mix_frames,
            )
        } else {
            (None, "unused".to_owned())
        };
        let snapshot_started = Instant::now();
        let (snapshot_main_queue, snapshot_streaming_main, live_snapshot) = {
            let buffer = self.shared.buffer.lock().ok()?;
            let decks = self.shared.crossfade_decks.lock().ok()?;
            let snapshot_main_queue = buffer.len();
            let snapshot_streaming_main = decks
                .main
                .as_ref()
                .map(|deck| deck.buffer.len())
                .unwrap_or(0);
            let live_snapshot = reservoir_snapshot.take().or_else(|| {
                prepared_mix_b_samples.as_ref().and_then(|_| {
                    snapshot_prepared_mix_main_deck_range(
                        decks.main.as_ref(),
                        prepared_mix_start_seconds,
                        output_sample_rate,
                        output_channels,
                        mix_frames,
                    )
                    .or_else(|| {
                        snapshot_prepared_mix_streaming_queue_range(
                            &buffer,
                            self.shared
                                .decoder_queued_source_frame
                                .load(Ordering::Relaxed),
                            prepared_mix_start_seconds,
                            output_sample_rate,
                            output_channels,
                            mix_frames,
                        )
                    })
                    .or_else(|| {
                        snapshot_prepared_mix_live_a_window(
                            &buffer,
                            decks.main.as_ref(),
                            output_channels,
                            output_sample_rate,
                            mix_frames,
                        )
                    })
                })
            });
            (snapshot_main_queue, snapshot_streaming_main, live_snapshot)
        };
        let snapshot_micros = elapsed_micros(snapshot_started);
        // Prepared Mix is armed from the UI/audio control thread, so the audio
        // callback may already have played past the planned start by the time
        // the segment queue is installed. Convert that lateness into extra
        // frame guard so callback alignment can discard frames inside raw A.
        let handoff_late_seconds =
            prepared_mix_handoff_late_seconds(prepared_mix_start_seconds, self.playback_seconds());
        let edge_guard_frames = prepared_mix_edge_guard_frames_for_late(
            output_sample_rate,
            mix_frames,
            handoff_late_seconds,
        );
        let build_started = Instant::now();
        let live_rebuild = prepared_mix_b_samples.as_ref().zip(live_snapshot).and_then(
            |(b_samples, live_snapshot)| {
                build_prepared_mix_from_live_a_snapshot(
                    &samples,
                    live_snapshot,
                    b_samples,
                    output_channels,
                    output_sample_rate,
                    mix_frames,
                    edge_guard_frames,
                )
            },
        );
        let build_micros = elapsed_micros(build_started);
        let live_rebuild_frames = live_rebuild
            .as_ref()
            .map(|rebuilt| rebuilt.live_a_frames)
            .unwrap_or(MusicMixFrameCount::ZERO);
        let live_rebuild_label = live_rebuild
            .as_ref()
            .map(|rebuilt| format!("{}-{}", rebuilt.source_label, rebuilt.kind.label()))
            .unwrap_or_else(|| "fallback".to_owned());
        let samples = live_rebuild
            .map(|rebuilt| rebuilt.samples)
            .unwrap_or(samples);
        // Keep callback alignment active even after a live-A rebuild. Snapshot
        // and render are intentionally outside audio locks, so callbacks may
        // play more A before this deck is armed; the callback must drop those
        // already-rendered frames from the segment queue.
        let prepared_mix_alignment_applied = false;
        let deck = CrossfadePreviewDeck {
            mode: CrossfadeDeckMode::PreparedMix,
            buffer: samples.into_iter().collect(),
            transition_output_frames: mix_frames,
            target_volume: target_volume.clamp(0.0, 1.0),
            track_start_source_frame: b_resume_source_frame,
            source_sample_rate: source_sample_rate.max(1),
            transition_source_frames: MusicMixFrameCount::ZERO,
            prepared_mix_resume_source_frame: Some(b_resume_source_frame),
            prepared_mix_start_seconds: prepared_mix_start_seconds
                .filter(|seconds| seconds.is_finite() && *seconds >= 0.0),
            prepared_mix_alignment_applied,
            track_duration_seconds: track_duration_seconds
                .filter(|duration| duration.is_finite() && *duration > 0.0),
            output_frames_consumed: MusicMixFrameCount::ZERO,
            rendered_frames_consumed: MusicMixFrameCount::ZERO,
            release_started_output_frame: None,
            release_duration_frames: MusicMixFrameCount::ZERO,
            release_from_gain: 0.0,
            outgoing_transition_rate: 1.0,
            outgoing_transition_phase_frames: 0.0,
            outgoing_transition_started_output_frame: None,
            outgoing_transition_duration_frames: MusicMixFrameCount::ZERO,
            outgoing_highlight_end_phase: None,
        };
        self.shared
            .discard_decoder_samples
            .store(true, Ordering::Relaxed);
        let install_started = Instant::now();
        let mut buffer = self.shared.buffer.lock().ok()?;
        let mut decks = self.shared.crossfade_decks.lock().ok()?;
        let transition_start_frame = self.output_frame_cursor();
        let cleared_main_queue = buffer.len();
        let cleared_streaming_main = decks
            .main
            .as_ref()
            .map(|deck| deck.buffer.len())
            .unwrap_or(0);
        // Prepared Mix is a pre-rendered A->[mix]->B segment queue. Once it is
        // armed, live A queues are stale sources; keeping or crossfading them
        // can replay a tiny earlier A fragment when the rendered segment starts.
        buffer.clear();
        decks.main = None;
        decks.next = Some(deck);
        drop(decks);
        drop(buffer);
        let install_micros = elapsed_micros(install_started);
        let handoff_micros = elapsed_micros(handoff_started);
        eprintln!(
            "[music-stage-prepared] handoff armed item={} session={} mode=segment-queue mix_frames={} guard_frames={} guard_late={:.1}ms start={:.3}s live_a={} live_frames={} pcm={} snap_main={} snap_deck={} cleared_main={} cleared_deck={} timing snapshot={:.1}ms build={:.1}ms install={:.1}ms total={:.1}ms align=callback",
            self.item_id,
            self.session_id,
            mix_frames.get(),
            edge_guard_frames.get(),
            handoff_late_seconds * 1000.0,
            prepared_mix_start_seconds.unwrap_or(-1.0),
            live_rebuild_label,
            live_rebuild_frames.get(),
            pcm_reservoir_status,
            snapshot_main_queue,
            snapshot_streaming_main,
            cleared_main_queue,
            cleared_streaming_main,
            micros_to_millis(snapshot_micros),
            micros_to_millis(build_micros),
            micros_to_millis(install_micros),
            micros_to_millis(handoff_micros)
        );
        Some(transition_start_frame)
    }

    pub fn fade_crossfade_preview_to_silence(&self, duration: Duration) {
        let duration_frames = self.mix_frame_count_from_duration(duration);
        if let Ok(mut decks) = self.shared.crossfade_decks.lock() {
            if let Some(deck) = decks.next.as_mut() {
                deck.release_from_gain = deck.gain_at_consumed();
                deck.release_started_output_frame = Some(deck.output_frames_consumed);
                deck.release_duration_frames = duration_frames;
            }
        }
    }

    pub fn clear_crossfade_preview(&self) {
        self.shared
            .outgoing_transition_rate_bits
            .store(1.0_f64.to_bits(), Ordering::Relaxed);
        self.shared
            .outgoing_transition_started_output_frame
            .store(0, Ordering::Relaxed);
        self.shared
            .outgoing_transition_duration_frames
            .store(0, Ordering::Relaxed);
        self.shared
            .outgoing_transition_phase_bits
            .store(0.0_f64.to_bits(), Ordering::Relaxed);
        clear_transition_load_diagnostics(&self.shared);
        if let Ok(mut decks) = self.shared.crossfade_decks.lock() {
            decks.next = None;
        }
    }

    pub fn has_promoted_crossfade_main(&self) -> bool {
        self.shared
            .crossfade_decks
            .lock()
            .map(|decks| decks.main.is_some())
            .unwrap_or(false)
    }

    pub(crate) fn crossfade_preview_transition_complete(&self) -> bool {
        self.shared
            .crossfade_decks
            .lock()
            .map(|decks| {
                decks
                    .next
                    .as_ref()
                    .is_some_and(CrossfadePreviewDeck::transition_complete)
            })
            .unwrap_or(false)
    }

    pub(crate) fn crossfade_preview_transition_progress_ratio(&self) -> Option<f64> {
        self.shared.crossfade_decks.lock().ok().and_then(|decks| {
            let deck = decks.next.as_ref()?;
            let total = deck.transition_output_frames.get().max(1);
            Some((deck.output_frames_consumed.get() as f64 / total as f64).clamp(0.0, 1.0))
        })
    }

    pub fn promote_crossfade_preview_to_main(
        &self,
        promoted_item_id: u64,
        promoted_session_id: u64,
    ) -> Option<f64> {
        // The chorus mixer must not hand off through a new playback stream.
        // Promote the already-playing crossfade deck inside the same output stream,
        // so the audio callback always sees one continuous session-lifetime stream.
        let promo_started = Instant::now();
        let buffer_lock_started = Instant::now();
        let mut buffer = self.shared.buffer.lock().ok()?;
        let buffer_lock_micros = buffer_lock_started
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        let deck_lock_started = Instant::now();
        let mut decks = self.shared.crossfade_decks.lock().ok()?;
        let deck_lock_micros = deck_lock_started
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        let main_queue_before_clear = buffer.len();
        let mut next = decks.next.take()?;
        if !next.transition_complete() {
            decks.next = Some(next);
            return None;
        }
        let next_queue_before_promote = next.buffer.len();
        next.outgoing_transition_rate = 1.0;
        next.outgoing_transition_phase_frames = 0.0;
        next.outgoing_transition_started_output_frame = None;
        next.outgoing_transition_duration_frames = MusicMixFrameCount::ZERO;
        self.shared
            .outgoing_transition_rate_bits
            .store(1.0_f64.to_bits(), Ordering::Relaxed);
        self.shared
            .outgoing_transition_started_output_frame
            .store(0, Ordering::Relaxed);
        self.shared
            .outgoing_transition_duration_frames
            .store(0, Ordering::Relaxed);
        self.shared
            .outgoing_transition_phase_bits
            .store(0.0_f64.to_bits(), Ordering::Relaxed);

        let output_sample_rate = self.shared.sample_rate.load(Ordering::Relaxed).max(1);
        let output_channels = self.shared.channels.load(Ordering::Relaxed).max(1) as usize;
        let sample_rate = output_sample_rate as f64;
        let channels = output_channels as f64;
        let source_position = next.source_position(output_sample_rate);
        let playback_seconds = source_position.seconds();
        let target_volume = next.target_volume;
        if let Some(duration_seconds) = next.track_duration_seconds {
            self.shared
                .duration_bits
                .store(duration_seconds.to_bits(), Ordering::Relaxed);
        }
        let preview_path_gain = next.gain_at_consumed()
            * f32::from_bits(self.shared.preview_level_guard_bits.load(Ordering::Relaxed))
                .clamp(0.82, 1.04)
            * f32::from_bits(
                self.shared
                    .crossfade_compensation_bits
                    .load(Ordering::Relaxed),
            )
            .clamp(0.92, 1.075);
        let handoff_gain = preview_path_gain.clamp(
            (target_volume * 0.32).min(target_volume),
            target_volume.max(0.0001),
        );
        let promoted_mode = next.mode.label();
        let promoted_output_frames = next.output_frames_consumed;
        let promoted_rendered_frames = next.rendered_frames_consumed;
        let promoted_transition_frames = next.transition_output_frames;
        let promoted_reservoir_start_frame = source_position.frame;
        let promoted_reservoir_samples: Vec<f32> = next
            .buffer
            .iter()
            .take(playback_pcm_reservoir_max_samples(
                output_sample_rate,
                output_channels,
            ))
            .copied()
            .collect();

        // This discard is now legal only after the sample-driven transition
        // boundary above. Before this guard, UI wall-clock completion could clear
        // still-audible A frames and create the reported sudden cut.
        buffer.clear();
        decks.main = Some(next);
        drop(decks);
        drop(buffer);

        append_playback_pcm_reservoir(
            &self.shared,
            promoted_item_id,
            promoted_session_id,
            promoted_reservoir_start_frame,
            &promoted_reservoir_samples,
            output_channels,
            output_sample_rate,
        );
        self.shared
            .discard_decoder_samples
            .store(true, Ordering::Relaxed);
        self.shared.samples_played.store(
            (playback_seconds * sample_rate * channels).round().max(0.0) as u64,
            Ordering::Relaxed,
        );
        // Keep the last preview block and the promoted B main deck at the same
        // perceived level.  If promotion happens a hair before the fade-in has
        // mathematically reached target_volume, jumping straight to full volume
        // is heard as the mix->B "bump" the user reported.  Start from the exact
        // preview gain and release gently to the real player volume.
        self.set_volume(handoff_gain);
        if (target_volume - handoff_gain).abs() >= 0.006 {
            self.fade_volume_to(target_volume, Duration::from_millis(220));
        } else {
            self.set_volume(target_volume);
        }

        eprintln!(
            "[music-stage-promote] mode={} source_frame={} playback={:.3}s remaining_frames={} consumed={} rendered={} transition={} gain={:.3}->{:.3}",
            promoted_mode,
            source_position.frame.get(),
            playback_seconds,
            next_queue_before_promote / output_channels.max(1),
            promoted_output_frames.get(),
            promoted_rendered_frames.get(),
            promoted_transition_frames.get(),
            handoff_gain,
            target_volume
        );

        let promo_micros = promo_started
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        let main_queue_millis =
            samples_to_millis(main_queue_before_clear, output_sample_rate, output_channels);
        let next_queue_millis = samples_to_millis(
            next_queue_before_promote,
            output_sample_rate,
            output_channels,
        );
        if let Some(summary) = transition_load_diagnostic_summary(
            &self.shared,
            promo_micros,
            buffer_lock_micros,
            deck_lock_micros,
            main_queue_millis,
            next_queue_millis,
        ) {
            eprintln!("[music-stage-load] {summary}");
        }
        clear_transition_load_diagnostics(&self.shared);
        Some(playback_seconds)
    }
    pub fn with_identity(&self, item_id: u64, session_id: u64) -> Self {
        Self {
            item_id,
            session_id,
            shared: self.shared.clone(),
            cache_state: self.cache_state.clone(),
        }
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
        if let Ok(decks) = self.shared.crossfade_decks.lock() {
            if let Some(deck) = decks.main.as_ref() {
                return deck
                    .source_position(self.shared.sample_rate.load(Ordering::Relaxed).max(1))
                    .seconds();
            }
        }
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
    crossfade_decks: Mutex<CrossfadeDecks>,
    pcm_reservoir: Mutex<MusicPcmReservoir>,
    pcm_reservoir_item_id: AtomicU64,
    pcm_reservoir_session_id: AtomicU64,
    decoder_queued_source_frame: AtomicU64,
    discard_decoder_samples: AtomicBool,
    stop_requested: AtomicBool,
    paused: AtomicBool,
    volume_bits: AtomicU32,
    volume_fade_from_bits: AtomicU32,
    volume_fade_to_bits: AtomicU32,
    volume_fade_started_output_frame: AtomicU64,
    volume_fade_duration_frames: AtomicU64,
    volume_fade_curve_bits: AtomicU32,
    preview_level_guard_bits: AtomicU32,
    crossfade_compensation_bits: AtomicU32,
    reward_energy_duck_bits: AtomicU32,
    samples_played: AtomicU64,
    output_frames_rendered: AtomicU64,
    sample_rate: AtomicU32,
    channels: AtomicU32,
    duration_bits: AtomicU64,
    seek_target_millis: AtomicU64,
    partial_seek_enabled: AtomicBool,
    outgoing_transition_rate_bits: AtomicU64,
    outgoing_transition_started_output_frame: AtomicU64,
    outgoing_transition_duration_frames: AtomicU64,
    outgoing_transition_phase_bits: AtomicU64,
    transition_load_diag_started_millis: AtomicU64,
    transition_load_diag_duration_millis: AtomicU64,
    transition_load_diag_last_callback_millis: AtomicU64,
    transition_load_diag_callback_count: AtomicU64,
    transition_load_diag_late_count: AtomicU64,
    transition_load_diag_source_underfill_count: AtomicU64,
    transition_load_diag_max_gap_millis: AtomicU64,
    transition_load_diag_max_work_micros: AtomicU64,
    transition_load_diag_min_main_queue_millis: AtomicU64,
    transition_load_diag_min_deck_queue_millis: AtomicU64,
    transition_load_diag_min_preview_queue_millis: AtomicU64,
}

impl SharedPlaybackState {
    fn new(volume: f32, duration_seconds: Option<f64>, item_id: u64, session_id: u64) -> Self {
        Self {
            buffer: Mutex::new(VecDeque::new()),
            crossfade_decks: Mutex::new(CrossfadeDecks::default()),
            pcm_reservoir: Mutex::new(MusicPcmReservoir::new(Duration::from_secs(
                MUSIC_PCM_RESERVOIR_SECONDS,
            ))),
            pcm_reservoir_item_id: AtomicU64::new(item_id),
            pcm_reservoir_session_id: AtomicU64::new(session_id),
            decoder_queued_source_frame: AtomicU64::new(0),
            discard_decoder_samples: AtomicBool::new(false),
            stop_requested: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            volume_bits: AtomicU32::new(volume.clamp(0.0, 1.0).to_bits()),
            volume_fade_from_bits: AtomicU32::new(volume.clamp(0.0, 1.0).to_bits()),
            volume_fade_to_bits: AtomicU32::new(volume.clamp(0.0, 1.0).to_bits()),
            volume_fade_started_output_frame: AtomicU64::new(0),
            volume_fade_duration_frames: AtomicU64::new(0),
            volume_fade_curve_bits: AtomicU32::new(0.0_f32.to_bits()),
            preview_level_guard_bits: AtomicU32::new(1.0_f32.to_bits()),
            crossfade_compensation_bits: AtomicU32::new(1.0_f32.to_bits()),
            reward_energy_duck_bits: AtomicU32::new(0.0_f32.to_bits()),
            samples_played: AtomicU64::new(0),
            output_frames_rendered: AtomicU64::new(0),
            sample_rate: AtomicU32::new(44_100),
            channels: AtomicU32::new(2),
            duration_bits: AtomicU64::new(duration_seconds.unwrap_or(0.0).to_bits()),
            seek_target_millis: AtomicU64::new(NO_SEEK_MILLIS),
            partial_seek_enabled: AtomicBool::new(false),
            outgoing_transition_rate_bits: AtomicU64::new(1.0_f64.to_bits()),
            outgoing_transition_started_output_frame: AtomicU64::new(0),
            outgoing_transition_duration_frames: AtomicU64::new(0),
            outgoing_transition_phase_bits: AtomicU64::new(0.0_f64.to_bits()),
            transition_load_diag_started_millis: AtomicU64::new(0),
            transition_load_diag_duration_millis: AtomicU64::new(0),
            transition_load_diag_last_callback_millis: AtomicU64::new(0),
            transition_load_diag_callback_count: AtomicU64::new(0),
            transition_load_diag_late_count: AtomicU64::new(0),
            transition_load_diag_source_underfill_count: AtomicU64::new(0),
            transition_load_diag_max_gap_millis: AtomicU64::new(0),
            transition_load_diag_max_work_micros: AtomicU64::new(0),
            transition_load_diag_min_main_queue_millis: AtomicU64::new(u64::MAX),
            transition_load_diag_min_deck_queue_millis: AtomicU64::new(u64::MAX),
            transition_load_diag_min_preview_queue_millis: AtomicU64::new(u64::MAX),
        }
    }
}

#[derive(Default)]
struct CrossfadeDecks {
    main: Option<CrossfadePreviewDeck>,
    next: Option<CrossfadePreviewDeck>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrossfadeDeckMode {
    RealtimePreview,
    PreparedMix,
}

impl CrossfadeDeckMode {
    fn label(self) -> &'static str {
        match self {
            Self::RealtimePreview => "realtime",
            Self::PreparedMix => "prepared",
        }
    }
}

struct CrossfadePreviewDeck {
    mode: CrossfadeDeckMode,
    buffer: VecDeque<f32>,
    transition_output_frames: MusicMixFrameCount,
    target_volume: f32,
    track_start_source_frame: MusicMixSourceFrame,
    source_sample_rate: u32,
    transition_source_frames: MusicMixFrameCount,
    prepared_mix_resume_source_frame: Option<MusicMixSourceFrame>,
    prepared_mix_start_seconds: Option<f64>,
    prepared_mix_alignment_applied: bool,
    track_duration_seconds: Option<f64>,
    // Output frames drive envelopes and promotion. Rendered frames drive source
    // position because a later outgoing tempo rate may consume them faster or
    // slower than the device writes output frames.
    output_frames_consumed: MusicMixFrameCount,
    rendered_frames_consumed: MusicMixFrameCount,
    release_started_output_frame: Option<MusicMixFrameCount>,
    release_duration_frames: MusicMixFrameCount,
    release_from_gain: f32,
    outgoing_transition_rate: f64,
    outgoing_transition_phase_frames: f64,
    outgoing_transition_started_output_frame: Option<MusicMixFrameCount>,
    outgoing_transition_duration_frames: MusicMixFrameCount,
    outgoing_highlight_end_phase: Option<f32>,
}

impl CrossfadePreviewDeck {
    fn gain_at_consumed(&self) -> f32 {
        if let Some(release_start) = self.release_started_output_frame {
            let elapsed = self
                .output_frames_consumed
                .get()
                .saturating_sub(release_start.get());
            if elapsed >= self.release_duration_frames.get() {
                return 0.0;
            }
            let ratio = elapsed as f32 / self.release_duration_frames.get().max(1) as f32;
            return (self.release_from_gain * (1.0 - smooth_audio_fade(ratio))).clamp(0.0, 1.0);
        }

        if self.transition_complete() {
            return self.target_volume;
        }
        let ratio = self.output_frames_consumed.get() as f32
            / self.transition_output_frames.get().max(1) as f32;
        self.target_volume * crossfade_equal_power_fade_in(ratio)
    }

    fn source_position(&self, output_sample_rate: u32) -> MusicMixSourcePosition {
        if self.mode == CrossfadeDeckMode::PreparedMix {
            let source_clock = MusicMixFrameClock::new(self.source_sample_rate);
            let rendered_clock = MusicMixFrameClock::new(output_sample_rate);
            let mix_elapsed = self
                .rendered_frames_consumed
                .min(self.transition_output_frames);
            let post_mix_elapsed = MusicMixFrameCount::new(
                self.rendered_frames_consumed
                    .get()
                    .saturating_sub(mix_elapsed.get()),
            );
            let post_mix_source_elapsed = source_clock.frame_count_from_seconds(
                rendered_clock.seconds_from_frame_count(post_mix_elapsed),
            );
            return MusicMixSourcePosition {
                frame: self
                    .prepared_mix_resume_source_frame
                    .unwrap_or(self.track_start_source_frame)
                    .saturating_add(post_mix_source_elapsed),
                sample_rate: source_clock.sample_rate(),
            };
        }

        source_position_after_rendered_frames(
            self.track_start_source_frame,
            self.source_sample_rate,
            self.rendered_frames_consumed,
            output_sample_rate,
            self.transition_output_frames,
            self.transition_source_frames,
        )
    }

    fn transition_complete(&self) -> bool {
        self.output_frames_consumed >= self.transition_output_frames
    }

    fn release_finished(&self) -> bool {
        self.release_started_output_frame
            .is_some_and(|release_start| {
                self.output_frames_consumed
                    .get()
                    .saturating_sub(release_start.get())
                    >= self.release_duration_frames.get()
            })
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
    analysis: PathBuf,
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
        stream.item_id,
        stream.session_id,
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
        let _ = fs::remove_file(&paths.analysis);
    }
    cache_cover_image_if_needed(stream, &paths);

    let existing_bytes = fs::metadata(&paths.media)
        .map(|meta| meta.len())
        .unwrap_or(0);
    if cached_media_is_complete(&paths, stream, existing_bytes) {
        schedule_music_analysis_for_stream(&paths, stream);
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
        let _ = fs::remove_file(&paths.analysis);
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
        schedule_music_analysis_for_stream(&paths, &stream);
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
    let mut source_timeline_sample_rate = 48_000_u32;
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
            let output_sample_rate = shared.sample_rate.load(Ordering::Relaxed).max(1);
            let output_channels = shared.channels.load(Ordering::Relaxed).max(1) as usize;
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
                    let seek_source_frame = MusicMixFrameClock::new(source_timeline_sample_rate)
                        .source_frame_from_seconds(seek_seconds);
                    shared.samples_played.store(
                        (seek_seconds * f64::from(output_sample_rate) * output_channels as f64)
                            .round()
                            .max(0.0) as u64,
                        Ordering::Relaxed,
                    );
                    shared
                        .decoder_queued_source_frame
                        .store(seek_source_frame.get(), Ordering::Relaxed);
                    reset_playback_pcm_reservoir(
                        &shared,
                        stream.item_id,
                        stream.session_id,
                        source_timeline_sample_rate,
                        output_channels,
                        seek_source_frame,
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
        source_timeline_sample_rate = source_sample_rate;

        if output_stream.is_none() {
            let output = build_output_stream(
                shared.clone(),
                source_sample_rate,
                source_channels.min(2).max(1),
            )?;
            shared
                .sample_rate
                .store(output.sample_rate, Ordering::Relaxed);
            shared
                .channels
                .store(output.channels as u32, Ordering::Relaxed);
            output
                .stream
                .play()
                .map_err(|error| format!("Could not start audio output: {error}"))?;
            output_stream = Some(output.stream);
        }
        let output_sample_rate = shared.sample_rate.load(Ordering::Relaxed).max(1);
        let output_channels = shared.channels.load(Ordering::Relaxed).max(1) as usize;

        sample_buffer.resize(decoded.samples_interleaved(), f32::MID);
        decoded.copy_to_slice_interleaved(&mut sample_buffer);
        queue_samples(
            &shared,
            stream.item_id,
            stream.session_id,
            &sample_buffer,
            source_channels,
            output_channels,
            source_sample_rate,
            output_sample_rate,
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
        manifest: dir.join("manifest.yaml"),
        analysis: dir.join("analysis.yaml"),
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
    let Some(manifest) = read_yaml_file::<MusicCacheManifest>(&paths.manifest) else {
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
    let Some(manifest) = read_yaml_file::<MusicCacheManifest>(&paths.manifest) else {
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
    if range_start == 0 {
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

fn schedule_music_analysis(paths: &MusicCachePaths, info: &MusicCacheManifestInfo) {
    spawn_music_analysis_if_needed(
        paths.media.clone(),
        info.ext.clone(),
        paths.analysis.clone(),
        info.duration_seconds,
    );
}

fn schedule_music_analysis_for_stream(paths: &MusicCachePaths, stream: &ResolvedMusicStream) {
    spawn_music_analysis_if_needed(
        paths.media.clone(),
        stream.ext.clone(),
        paths.analysis.clone(),
        stream.duration_seconds,
    );
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
    write_yaml_file(&paths.manifest, &manifest)
        .map_err(|error| format!("Could not write music cache manifest: {error}"))?;
    if complete {
        schedule_music_analysis(paths, info);
    }
    Ok(())
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
    if let Some(manifest) = read_yaml_file::<MusicCacheManifest>(&paths.manifest) {
        let ranges = normalize_ranges(manifest.ranges);
        if !ranges.is_empty() {
            return ranges;
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

/// Pre-rendered B audio plus the frame mapping required for gapless promotion.
///
/// `transition_output_frames` controls the device envelope. The corresponding
/// `transition_source_frames` records how much original B audio the render
/// consumed, including natural-speed anchors around preserve-pitch regions.
pub struct MusicMixPreviewSegment {
    pub samples: Vec<f32>,
    pub transition_output_frames: MusicMixFrameCount,
    pub transition_source_frames: MusicMixFrameCount,
    pub source_start_frame: MusicMixSourceFrame,
    pub source_sample_rate: u32,
    pub transition_source_rate: f64,
    pub preserve_pitch: bool,
    pub stretch_detail: Option<String>,
}

/// Fully rendered A->[mix]->B capsule for the single output callback.
///
/// The first `mix_output_frames` are already mixed A+B audio.  Remaining frames
/// are B continuation frames and become the promoted main deck after the frame
/// boundary.  Keep all boundaries in frames so Stage Mix never has to infer the
/// handoff point from UI time or wall-clock progress.
pub struct MusicPreparedMixSegment {
    pub samples: Vec<f32>,
    pub b_samples: Vec<f32>,
    pub mix_output_frames: MusicMixFrameCount,
    pub mix_source_frames: MusicMixFrameCount,
    pub b_resume_source_frame: MusicMixSourceFrame,
    pub source_sample_rate: u32,
    pub transition_source_rate: f64,
    pub preserve_pitch: bool,
    pub stretch_detail: Option<String>,
}

pub fn decode_music_file_segment_for_mix(
    media_path: &Path,
    ext: &str,
    start_seconds: f64,
    duration: Duration,
    transition_duration: Duration,
    transition_source_rate: f64,
    output_sample_rate: u32,
    output_channels: usize,
    render_mode: MusicMixRenderMode,
) -> Result<MusicMixPreviewSegment, String> {
    let source = File::open(media_path)
        .map_err(|error| format!("Could not open crossfade preview media: {error}"))?;
    let mss = MediaSourceStream::new(Box::new(source), Default::default());

    let mut hint = Hint::new();
    if !ext.trim().is_empty() {
        hint.with_extension(ext.trim());
    }

    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|error| format!("Could not read crossfade preview format: {error}"))?;

    let (track_id, track_time_base, mut decoder) = {
        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| "No audio track was found for crossfade preview.".to_owned())?;
        let codec_params = track.codec_params.as_ref().ok_or_else(|| {
            "Audio codec parameters are missing for crossfade preview.".to_owned()
        })?;
        let audio_params = codec_params.audio().ok_or_else(|| {
            "Audio codec parameters are missing for crossfade preview.".to_owned()
        })?;
        let decoder = music_codec_registry()
            .make_audio_decoder(audio_params, &AudioDecoderOptions::default())
            .map_err(|error| format!("Could not create crossfade preview decoder: {error}"))?;
        (track.id, track.time_base, decoder)
    };

    let requested_start_seconds = start_seconds.max(0.0);
    let Some(seek_time) = Time::try_from_secs_f64(requested_start_seconds) else {
        return Err("Invalid crossfade preview start time.".to_owned());
    };
    let seeked = format
        .seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time: seek_time,
                track_id: Some(track_id),
            },
        )
        .map_err(|error| format!("Could not seek crossfade preview: {error}"))?;
    let actual_seek_seconds = track_time_base
        .and_then(|time_base| time_base.calc_time(seeked.actual_ts))
        .map(|time| time.as_secs_f64())
        .filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
        .unwrap_or(requested_start_seconds);
    let seek_trim_seconds = (requested_start_seconds - actual_seek_seconds).max(0.0);
    let effective_source_start_seconds = actual_seek_seconds.max(requested_start_seconds);

    let output_channels = output_channels.clamp(1, 2);
    let output_sample_rate = output_sample_rate.max(1);
    let source_segment_duration = duration.as_secs_f64().max(0.05);
    let transition_seconds = transition_duration
        .as_secs_f64()
        .clamp(0.0, source_segment_duration);
    let transition_source_rate = transition_source_rate.clamp(0.965, 1.035);
    let output_duration = (transition_seconds
        + (source_segment_duration - transition_seconds * transition_source_rate).max(0.0))
    .max(0.05);
    let source_duration = (transition_seconds * transition_source_rate
        + (output_duration - transition_seconds).max(0.0))
    .max(source_segment_duration.min(transition_seconds.max(0.05)));

    let mut source_frames: Vec<f32> = Vec::new();
    let mut sample_buffer: Vec<f32> = Vec::new();
    let mut source_sample_rate = output_sample_rate;
    let mut collected_frames = 0_usize;
    let mut remaining_seek_trim_frames = None;

    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(error) => return Err(format!("Could not read crossfade preview packet: {error}")),
        };
        if packet.track_id != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(error) => {
                return Err(format!(
                    "Could not decode crossfade preview packet: {error}"
                ));
            }
        };

        let spec = decoded.spec();
        let source_channels = spec.channels().count().max(1);
        source_sample_rate = spec.rate().max(1);
        let trim_frames = remaining_seek_trim_frames.get_or_insert_with(|| {
            (seek_trim_seconds * source_sample_rate as f64)
                .round()
                .clamp(0.0, usize::MAX as f64) as usize
        });
        let max_source_frames = (source_duration * source_sample_rate as f64).ceil() as usize + 4;
        sample_buffer.resize(decoded.samples_interleaved(), f32::MID);
        decoded.copy_to_slice_interleaved(&mut sample_buffer);
        let decoded_frames = sample_buffer.len() / source_channels;
        let skipped_frames = (*trim_frames).min(decoded_frames);
        *trim_frames = trim_frames.saturating_sub(skipped_frames);

        // Accurate container seek lands on a packet boundary at or before the
        // requested cue. Decode and discard the exact leading source frames so
        // B audio and its recorded source frame begin at the same position.
        for frame in sample_buffer.chunks(source_channels).skip(skipped_frames) {
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
                source_frames.push(sample);
            }
            collected_frames += 1;
            if collected_frames >= max_source_frames {
                break;
            }
        }
        if collected_frames >= max_source_frames {
            break;
        }
    }

    if source_frames.is_empty() {
        return Err("Crossfade preview segment is empty.".to_owned());
    }

    Ok(build_preserve_pitch_mix_preview(
        &source_frames,
        source_sample_rate,
        output_sample_rate,
        output_channels,
        effective_source_start_seconds,
        source_segment_duration,
        transition_seconds,
        transition_source_rate,
        render_mode,
    ))
}

pub fn render_music_prepared_mix_segment(
    current_media_path: &Path,
    current_ext: &str,
    current_mix_start_seconds: f64,
    next_media_path: &Path,
    next_ext: &str,
    next_entry_start_seconds: f64,
    next_source_duration: Duration,
    transition_duration: Duration,
    transition_source_rate: f64,
    output_sample_rate: u32,
    output_channels: usize,
    render_mode: MusicMixRenderMode,
) -> Result<MusicPreparedMixSegment, String> {
    let output_sample_rate = output_sample_rate.max(1);
    let output_channels = output_channels.clamp(1, 2);
    let transition_seconds = transition_duration.as_secs_f64().max(0.05);
    let next_preview = decode_music_file_segment_for_mix(
        next_media_path,
        next_ext,
        next_entry_start_seconds,
        next_source_duration,
        transition_duration,
        transition_source_rate,
        output_sample_rate,
        output_channels,
        render_mode,
    )?;
    let mix_output_frames = next_preview.transition_output_frames;
    if mix_output_frames.is_zero() || next_preview.samples.is_empty() {
        return Err("Prepared Mix rejected an empty B preview.".to_owned());
    }

    let current_mix_duration = Duration::from_secs_f64(
        MusicMixFrameClock::new(output_sample_rate)
            .seconds_from_frame_count(mix_output_frames)
            .max(transition_seconds),
    );
    let current_segment = decode_music_file_segment_for_mix(
        current_media_path,
        current_ext,
        current_mix_start_seconds.max(0.0),
        current_mix_duration,
        Duration::ZERO,
        1.0,
        output_sample_rate,
        output_channels,
        MusicMixRenderMode::Streaming,
    )?;

    let edge_guard_frames = prepared_mix_edge_guard_frames(output_sample_rate, mix_output_frames);
    let mut samples = build_prepared_mix_samples_with_guards(
        &current_segment.samples,
        &next_preview.samples,
        output_channels,
        mix_output_frames,
        edge_guard_frames,
    );
    sanitize_mix_preview_samples(
        &mut samples,
        output_channels,
        output_sample_rate,
        MusicMixFrameClock::new(output_sample_rate).seconds_from_frame_count(mix_output_frames),
    );
    if samples.is_empty() {
        return Err("Prepared Mix renderer produced no samples.".to_owned());
    }

    let b_resume_position = source_position_after_rendered_frames(
        next_preview.source_start_frame,
        next_preview.source_sample_rate,
        mix_output_frames,
        output_sample_rate,
        mix_output_frames,
        next_preview.transition_source_frames,
    );
    let guard_detail = prepared_mix_guard_detail(output_sample_rate, edge_guard_frames);
    let stretch_detail = Some(match next_preview.stretch_detail {
        Some(detail) if !detail.trim().is_empty() => {
            format!("Prepared Mix · {guard_detail} · {detail}")
        }
        _ => format!("Prepared Mix · {guard_detail}"),
    });

    Ok(MusicPreparedMixSegment {
        samples,
        b_samples: next_preview.samples,
        mix_output_frames,
        mix_source_frames: next_preview.transition_source_frames,
        b_resume_source_frame: b_resume_position.frame,
        source_sample_rate: next_preview.source_sample_rate,
        transition_source_rate: next_preview.transition_source_rate,
        preserve_pitch: next_preview.preserve_pitch,
        stretch_detail,
    })
}

fn build_prepared_mix_samples(
    current_samples: &[f32],
    next_samples: &[f32],
    channels: usize,
    mix_frames: MusicMixFrameCount,
) -> Vec<f32> {
    build_prepared_mix_samples_with_guards(
        current_samples,
        next_samples,
        channels,
        mix_frames,
        MusicMixFrameCount::ZERO,
    )
}

fn build_prepared_mix_samples_with_guards(
    current_samples: &[f32],
    next_samples: &[f32],
    channels: usize,
    mix_frames: MusicMixFrameCount,
    edge_guard_frames: MusicMixFrameCount,
) -> Vec<f32> {
    let channels = channels.max(1);
    let mix_frames = mix_frames.get() as usize;
    let edge_guard_frames = edge_guard_frames.get() as usize;
    let current_frames = current_samples.len() / channels;
    let next_frames = next_samples.len() / channels;
    let total_frames = next_frames.max(mix_frames);
    if total_frames == 0 {
        return Vec::new();
    }

    let edge_guard_frames = edge_guard_frames.min(mix_frames / 4);
    let head_guard_frames = edge_guard_frames;
    let tail_guard_frames = edge_guard_frames.min(mix_frames.saturating_sub(head_guard_frames));
    let fade_start_frame = head_guard_frames;
    let fade_end_frame = mix_frames.saturating_sub(tail_guard_frames);
    let fade_frames = fade_end_frame.saturating_sub(fade_start_frame);
    let mut output = Vec::with_capacity(total_frames.saturating_mul(channels));
    let fade_denominator = fade_frames.saturating_sub(1).max(1) as f32;
    for frame in 0..total_frames {
        let in_mix = frame < mix_frames;
        let ratio = if !in_mix || frame >= fade_end_frame {
            1.0
        } else if frame <= fade_start_frame {
            0.0
        } else {
            frame.saturating_sub(fade_start_frame) as f32 / fade_denominator
        };
        let current_gain = if in_mix && frame < fade_end_frame {
            crossfade_equal_power_fade_out(ratio)
        } else {
            0.0
        };
        let next_gain = if !in_mix || frame >= fade_end_frame {
            1.0
        } else if frame < fade_start_frame {
            0.0
        } else {
            crossfade_equal_power_fade_in(ratio)
        };

        for channel in 0..channels {
            let current = if frame < current_frames {
                current_samples
                    .get(frame.saturating_mul(channels) + channel)
                    .copied()
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            let next = if frame < next_frames {
                next_samples
                    .get(frame.saturating_mul(channels) + channel)
                    .copied()
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            output.push(soft_limit_audio_sample(
                current * current_gain + next * next_gain,
            ));
        }
    }
    output
}

fn prepared_mix_edge_guard_frames(
    output_sample_rate: u32,
    mix_frames: MusicMixFrameCount,
) -> MusicMixFrameCount {
    prepared_mix_edge_guard_frames_for_late(output_sample_rate, mix_frames, 0.0)
}

fn prepared_mix_edge_guard_frames_for_late(
    output_sample_rate: u32,
    mix_frames: MusicMixFrameCount,
    late_seconds: f64,
) -> MusicMixFrameCount {
    if mix_frames.is_zero() {
        return MusicMixFrameCount::ZERO;
    }
    let late_millis = if late_seconds.is_finite() && late_seconds > 0.0 {
        (late_seconds * 1000.0).ceil().clamp(0.0, u64::MAX as f64) as u64
    } else {
        0
    };
    // Guard frames are structural PCM padding, not a cosmetic fade. They keep
    // both external boundaries exact while giving late callback alignment a raw
    // A/B area to trim into before the processed crossfade body starts.
    let requested_millis = MUSIC_PREPARED_MIX_EDGE_GUARD_MILLIS
        .max(late_millis.saturating_add(MUSIC_PREPARED_MIX_LATE_GUARD_PAD_MILLIS));
    let requested = MusicMixFrameClock::new(output_sample_rate)
        .frame_count_from_duration(Duration::from_millis(requested_millis));
    let max_each_side = MusicMixFrameCount::new(mix_frames.get() / 4);
    requested.min(max_each_side)
}

fn prepared_mix_handoff_late_seconds(
    prepared_mix_start_seconds: Option<f64>,
    playback_seconds: f64,
) -> f64 {
    prepared_mix_start_seconds
        .filter(|seconds| seconds.is_finite())
        .map(|start| (playback_seconds - start).max(0.0))
        .filter(|late| late.is_finite())
        .unwrap_or(0.0)
}

fn prepared_mix_guard_detail(output_sample_rate: u32, guard_frames: MusicMixFrameCount) -> String {
    let guard_millis =
        MusicMixFrameClock::new(output_sample_rate).seconds_from_frame_count(guard_frames) * 1000.0;
    format!("guard {:.0}ms", guard_millis)
}

struct PreparedMixLiveARebuild {
    samples: Vec<f32>,
    live_a_frames: MusicMixFrameCount,
    kind: PreparedMixLiveARebuildKind,
    source_label: &'static str,
}

struct PreparedMixLiveASnapshot {
    samples: Vec<f32>,
    live_a_frames: MusicMixFrameCount,
    source_label: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreparedMixLiveARebuildKind {
    FullWindow,
    BoundaryHead,
}

impl PreparedMixLiveARebuildKind {
    fn label(self) -> &'static str {
        match self {
            Self::FullWindow => "window",
            Self::BoundaryHead => "head",
        }
    }
}

fn snapshot_prepared_mix_live_a_window(
    streaming_queue: &VecDeque<f32>,
    main_deck: Option<&CrossfadePreviewDeck>,
    channels: usize,
    output_sample_rate: u32,
    mix_frames: MusicMixFrameCount,
) -> Option<PreparedMixLiveASnapshot> {
    let channels = channels.max(1);
    let mix_frame_count = mix_frames.get() as usize;
    if mix_frame_count == 0 {
        return None;
    }

    let min_head_frames = MusicMixFrameClock::new(output_sample_rate)
        .frame_count_from_duration(Duration::from_millis(
            MUSIC_PREPARED_MIX_LIVE_A_MIN_HEAD_MILLIS,
        ))
        .get()
        .min(mix_frames.get()) as usize;
    let live_frames =
        live_mix_window_available_frames(streaming_queue, main_deck, channels).min(mix_frame_count);
    if live_frames < min_head_frames.max(1) {
        return None;
    }

    let current_samples = snapshot_live_mix_window_from_queues(
        streaming_queue,
        main_deck,
        channels,
        MusicMixFrameCount::new(live_frames as u64),
    )?;
    Some(PreparedMixLiveASnapshot {
        samples: current_samples,
        live_a_frames: MusicMixFrameCount::new(live_frames as u64),
        source_label: "queue",
    })
}

fn snapshot_prepared_mix_pcm_reservoir_window(
    shared: &SharedPlaybackState,
    item_id: u64,
    session_id: u64,
    prepared_mix_start_seconds: Option<f64>,
    output_sample_rate: u32,
    output_channels: usize,
    mix_frames: MusicMixFrameCount,
) -> (Option<PreparedMixLiveASnapshot>, String) {
    let Some(start_seconds) =
        prepared_mix_start_seconds.filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
    else {
        return (None, "missing-start".to_owned());
    };
    if mix_frames.is_zero() {
        return (None, "empty-mix".to_owned());
    }
    let requested_start =
        MusicMixFrameClock::new(output_sample_rate).source_frame_from_seconds(start_seconds);
    let Ok(reservoir) = shared.pcm_reservoir.lock() else {
        return (None, "lock-failed".to_owned());
    };
    let coverage = reservoir.coverage();
    let status = playback_pcm_reservoir_status(coverage, requested_start, mix_frames);
    let owner_item_id = shared.pcm_reservoir_item_id.load(Ordering::Relaxed);
    let owner_session_id = shared.pcm_reservoir_session_id.load(Ordering::Relaxed);
    if owner_item_id != item_id || owner_session_id != session_id {
        return (
            None,
            format!(
                "owner-miss:req={item_id}/{session_id} cover={owner_item_id}/{owner_session_id} {status}"
            ),
        );
    }
    let Some(snapshot) = reservoir.snapshot_range(requested_start, mix_frames) else {
        return (None, format!("miss:{status}"));
    };
    if snapshot.sample_rate != output_sample_rate || snapshot.channels != output_channels.max(1) {
        return (
            None,
            format!(
                "format-miss:{}Hz/{}ch:{}",
                snapshot.sample_rate, snapshot.channels, status
            ),
        );
    }

    (
        Some(PreparedMixLiveASnapshot {
            samples: snapshot.samples,
            live_a_frames: snapshot.frame_count,
            source_label: "reservoir",
        }),
        format!("hit:{status}"),
    )
}

fn snapshot_prepared_mix_main_deck_range(
    main_deck: Option<&CrossfadePreviewDeck>,
    prepared_mix_start_seconds: Option<f64>,
    output_sample_rate: u32,
    channels: usize,
    mix_frames: MusicMixFrameCount,
) -> Option<PreparedMixLiveASnapshot> {
    let deck = main_deck?;
    let start_seconds =
        prepared_mix_start_seconds.filter(|seconds| seconds.is_finite() && *seconds >= 0.0)?;
    let channels = channels.max(1);
    let front_seconds = deck.source_position(output_sample_rate).seconds();
    let offset_seconds = start_seconds - front_seconds;
    if offset_seconds < -0.010 {
        return None;
    }
    let offset_frames = MusicMixFrameClock::new(output_sample_rate)
        .frame_count_from_seconds(offset_seconds.max(0.0))
        .get() as usize;
    snapshot_interleaved_queue_range(
        &deck.buffer,
        channels,
        offset_frames,
        mix_frames,
        "deck-range",
    )
}

fn snapshot_prepared_mix_streaming_queue_range(
    streaming_queue: &VecDeque<f32>,
    queued_end_frame: u64,
    prepared_mix_start_seconds: Option<f64>,
    output_sample_rate: u32,
    channels: usize,
    mix_frames: MusicMixFrameCount,
) -> Option<PreparedMixLiveASnapshot> {
    let start_seconds =
        prepared_mix_start_seconds.filter(|seconds| seconds.is_finite() && *seconds >= 0.0)?;
    let channels = channels.max(1);
    let queued_frames = streaming_queue.len() / channels;
    let queued_start_frame = queued_end_frame.saturating_sub(queued_frames as u64);
    let requested_start = MusicMixFrameClock::new(output_sample_rate)
        .source_frame_from_seconds(start_seconds)
        .get();
    if requested_start < queued_start_frame {
        return None;
    }
    let offset_frames = requested_start.saturating_sub(queued_start_frame) as usize;
    snapshot_interleaved_queue_range(
        streaming_queue,
        channels,
        offset_frames,
        mix_frames,
        "stream-range",
    )
}

fn snapshot_interleaved_queue_range(
    queue: &VecDeque<f32>,
    channels: usize,
    offset_frames: usize,
    frame_count: MusicMixFrameCount,
    source_label: &'static str,
) -> Option<PreparedMixLiveASnapshot> {
    let channels = channels.max(1);
    let requested_frames = frame_count.get() as usize;
    if requested_frames == 0 {
        return None;
    }
    let offset_samples = offset_frames.saturating_mul(channels);
    let requested_samples = requested_frames.saturating_mul(channels);
    if offset_samples.saturating_add(requested_samples) > queue.len() {
        return None;
    }
    Some(PreparedMixLiveASnapshot {
        samples: queue
            .iter()
            .skip(offset_samples)
            .take(requested_samples)
            .copied()
            .collect(),
        live_a_frames: frame_count,
        source_label,
    })
}

fn build_prepared_mix_from_live_a_snapshot(
    prepared_samples: &[f32],
    live_snapshot: PreparedMixLiveASnapshot,
    next_samples: &[f32],
    channels: usize,
    output_sample_rate: u32,
    mix_frames: MusicMixFrameCount,
    edge_guard_frames: MusicMixFrameCount,
) -> Option<PreparedMixLiveARebuild> {
    let channels = channels.max(1);
    let mix_frame_count = mix_frames.get() as usize;
    let prepared_frame_count = prepared_samples.len() / channels;
    if mix_frame_count == 0
        || prepared_frame_count == 0
        || next_samples.len() / channels < mix_frame_count
    {
        return None;
    }

    let live_frames = live_snapshot.live_a_frames.get() as usize;
    if live_frames == 0 {
        return None;
    }

    let live_mix_samples = build_prepared_mix_samples_with_guards(
        &live_snapshot.samples,
        next_samples,
        channels,
        mix_frames,
        edge_guard_frames,
    );
    let mut samples = prepared_samples.to_vec();
    let replace_samples = live_frames
        .saturating_mul(channels)
        .min(samples.len())
        .min(live_mix_samples.len());
    if replace_samples == 0 {
        return None;
    }
    samples[..replace_samples].copy_from_slice(&live_mix_samples[..replace_samples]);

    let kind = if live_frames >= mix_frame_count {
        PreparedMixLiveARebuildKind::FullWindow
    } else {
        blend_live_a_head_to_prepared_body(
            &mut samples,
            &live_mix_samples,
            prepared_samples,
            channels,
            live_frames,
            output_sample_rate,
        );
        PreparedMixLiveARebuildKind::BoundaryHead
    };
    Some(PreparedMixLiveARebuild {
        samples,
        live_a_frames: live_snapshot.live_a_frames,
        kind,
        source_label: live_snapshot.source_label,
    })
}

fn live_mix_window_available_frames(
    streaming_queue: &VecDeque<f32>,
    main_deck: Option<&CrossfadePreviewDeck>,
    channels: usize,
) -> usize {
    let channels = channels.max(1);
    let streaming_frames = streaming_queue.len() / channels;
    let deck_frames = main_deck
        .map(|deck| deck.buffer.len() / channels)
        .unwrap_or(0);
    streaming_frames.max(deck_frames)
}

fn snapshot_live_mix_window_from_queues(
    streaming_queue: &VecDeque<f32>,
    main_deck: Option<&CrossfadePreviewDeck>,
    channels: usize,
    frames: MusicMixFrameCount,
) -> Option<Vec<f32>> {
    let channels = channels.max(1);
    let requested_frames = frames.get() as usize;
    if requested_frames == 0 {
        return None;
    }

    let deck_queue = main_deck.map(|deck| &deck.buffer);
    if live_mix_window_available_frames(streaming_queue, main_deck, channels) < requested_frames {
        return None;
    }

    let mut output = Vec::with_capacity(requested_frames.saturating_mul(channels));
    for frame in 0..requested_frames {
        for channel in 0..channels {
            let index = frame.saturating_mul(channels).saturating_add(channel);
            let streaming = streaming_queue.get(index).copied().unwrap_or(0.0);
            let deck = deck_queue
                .and_then(|queue| queue.get(index))
                .copied()
                .unwrap_or(0.0);
            output.push(streaming + deck);
        }
    }
    Some(output)
}

fn blend_live_a_head_to_prepared_body(
    samples: &mut [f32],
    live_mix_samples: &[f32],
    prepared_samples: &[f32],
    channels: usize,
    live_frames: usize,
    output_sample_rate: u32,
) {
    let channels = channels.max(1);
    let frame_count = samples.len() / channels;
    if live_frames == 0 || live_frames >= frame_count {
        return;
    }
    let blend_frames = MusicMixFrameClock::new(output_sample_rate)
        .frame_count_from_duration(Duration::from_millis(
            MUSIC_PREPARED_MIX_LIVE_A_HEAD_BLEND_MILLIS,
        ))
        .get() as usize;
    let blend_frames = blend_frames
        .min(live_frames)
        .min(frame_count.saturating_sub(live_frames).max(1));
    if blend_frames == 0 {
        return;
    }
    let blend_start = live_frames.saturating_sub(blend_frames);
    let denominator = blend_frames.max(1) as f32;
    for offset in 0..blend_frames {
        let ratio = (offset + 1) as f32 / denominator;
        let fade = smooth_audio_fade(ratio.clamp(0.0, 1.0));
        let frame = blend_start + offset;
        for channel in 0..channels {
            let index = frame.saturating_mul(channels).saturating_add(channel);
            if index >= samples.len()
                || index >= live_mix_samples.len()
                || index >= prepared_samples.len()
            {
                continue;
            }
            samples[index] =
                live_mix_samples[index] * (1.0 - fade) + prepared_samples[index] * fade;
        }
    }
}

fn build_preserve_pitch_mix_preview(
    source: &[f32],
    source_sample_rate: u32,
    output_sample_rate: u32,
    channels: usize,
    source_start_seconds: f64,
    source_segment_duration_seconds: f64,
    transition_seconds: f64,
    transition_source_rate: f64,
    render_mode: MusicMixRenderMode,
) -> MusicMixPreviewSegment {
    let channels = channels.max(1);
    let source_sample_rate = source_sample_rate.max(1);
    let output_sample_rate = output_sample_rate.max(1);
    let source_frame_count = source.len() / channels;
    if source_frame_count == 0 {
        return MusicMixPreviewSegment {
            samples: Vec::new(),
            transition_output_frames: MusicMixFrameCount::ZERO,
            transition_source_frames: MusicMixFrameCount::ZERO,
            source_start_frame: MusicMixSourceFrame::new(0),
            source_sample_rate,
            transition_source_rate: 1.0,
            preserve_pitch: false,
            stretch_detail: Some("empty preview".to_owned()),
        };
    }

    let requested_rate_raw = transition_source_rate.clamp(0.965, 1.035);
    let artifact_risk = mix_preview_artifact_risk_score(source, channels);
    let requested_rate = smooth_stage_mix_requested_rate(requested_rate_raw, artifact_risk);
    let rate_delta = (requested_rate - 1.0).abs();
    let tempo_smoothing_suffix =
        tempo_smoothing_detail(requested_rate_raw, requested_rate, artifact_risk);
    let transition_seconds = transition_seconds
        .max(0.0)
        .min(source_segment_duration_seconds.max(0.0));
    let micro_stream_mix = transition_seconds <= MUSIC_MIX_MICRO_STRETCH_BYPASS_SECONDS;
    // Compact 1-2 beat handoffs are too short for the preserve-pitch stretcher to
    // hide its analysis/re-synthesis edge reliably.  Let the beat/cue alignment
    // carry these micro transitions and keep B at natural speed; longer Stream
    // Mix still uses preserve-pitch.
    let should_preserve_pitch = !micro_stream_mix && rate_delta >= MUSIC_MIX_TEMPO_JND_RATE;
    let normal_anchor_seconds = if should_preserve_pitch {
        mix_preview_normal_anchor_seconds(transition_seconds, requested_rate)
    } else {
        0.0
    };
    let head_anchor_seconds = normal_anchor_seconds.min((transition_seconds * 0.28).max(0.0));
    let exit_anchor_seconds =
        normal_anchor_seconds.min((transition_seconds - head_anchor_seconds - 0.35).max(0.0));
    let stretch_output_seconds = if should_preserve_pitch {
        (transition_seconds - head_anchor_seconds - exit_anchor_seconds).max(0.0)
    } else {
        transition_seconds
    };
    // The B side is faded in during the mix.  Hide more of the tempo correction
    // while B is still quiet, then ease the later/high-volume part closer to
    // normal speed.  This keeps the handoff musical without putting a hard
    // preserve-pitch seam right before B becomes the main deck.
    let gain_aware_stretch = should_preserve_pitch
        && stretch_output_seconds >= 1.35
        && rate_delta >= MUSIC_MIX_TEMPO_JND_RATE;
    let early_stretch_output_seconds = if gain_aware_stretch {
        let early_ratio = if rate_delta >= 0.025 { 0.50 } else { 0.46 };
        (stretch_output_seconds * early_ratio)
            .clamp(0.55, (stretch_output_seconds - 0.55).max(0.55))
    } else {
        stretch_output_seconds
    };
    let late_stretch_output_seconds = if gain_aware_stretch {
        (stretch_output_seconds - early_stretch_output_seconds).max(0.0)
    } else {
        0.0
    };
    let early_stretch_rate = if gain_aware_stretch {
        perceptual_early_transition_rate(requested_rate)
    } else {
        requested_rate
    };
    let late_stretch_rate = if gain_aware_stretch {
        perceptual_late_transition_rate(requested_rate)
    } else {
        requested_rate
    };

    let head_anchor_frames = (head_anchor_seconds * source_sample_rate as f64)
        .round()
        .clamp(0.0, source_frame_count as f64) as usize;
    let transition_start_sample = if should_preserve_pitch {
        head_anchor_frames
            .saturating_mul(channels)
            .min(source.len())
    } else {
        0
    };
    let source_transition_seconds = if gain_aware_stretch {
        early_stretch_output_seconds * early_stretch_rate
            + late_stretch_output_seconds * late_stretch_rate
    } else if should_preserve_pitch {
        stretch_output_seconds * requested_rate
    } else {
        stretch_output_seconds
    }
    .clamp(0.0, source_segment_duration_seconds.max(0.0));
    let source_transition_frames = (source_transition_seconds * source_sample_rate as f64)
        .round()
        .clamp(
            0.0,
            source_frame_count.saturating_sub(head_anchor_frames) as f64,
        ) as usize;
    let early_source_frames = if gain_aware_stretch {
        (early_stretch_output_seconds * early_stretch_rate * source_sample_rate as f64)
            .round()
            .clamp(0.0, source_transition_frames as f64) as usize
    } else {
        source_transition_frames
    };
    let transition_end_frame = if should_preserve_pitch {
        head_anchor_frames.saturating_add(source_transition_frames)
    } else {
        source_transition_frames
    }
    .min(source_frame_count);
    let transition_end_sample = transition_end_frame
        .saturating_mul(channels)
        .min(source.len());
    let transition_source = &source[transition_start_sample..transition_end_sample];
    let mut tail_start_sample = transition_end_sample;

    let mut output = Vec::new();
    let mut applied_rate = 1.0_f64;
    let mut preserve_pitch = false;
    let mut transition_source_frames_for_timeline = source_transition_frames;
    let mut stretch_detail = None;

    if should_preserve_pitch && head_anchor_frames > 0 {
        let head_end_sample = head_anchor_frames
            .saturating_mul(channels)
            .min(source.len());
        output.extend(resample_interleaved_frames(
            &source[..head_end_sample],
            source_sample_rate,
            output_sample_rate,
            channels,
            Some(
                (head_anchor_seconds * output_sample_rate as f64)
                    .round()
                    .max(1.0) as usize,
            ),
        ));
    }

    if !transition_source.is_empty() && should_preserve_pitch {
        let stretch_result = if gain_aware_stretch && late_stretch_output_seconds > 0.05 {
            append_gain_aware_stretched_preview_part(
                &mut output,
                transition_source,
                source_sample_rate,
                output_sample_rate,
                channels,
                early_stretch_rate,
                late_stretch_rate,
                early_source_frames,
                stretch_output_seconds,
                render_mode,
            )
            .map(|result| {
                if head_anchor_seconds > 0.0 {
                    let seam_frame = (head_anchor_seconds * output_sample_rate as f64)
                        .round()
                        .max(1.0) as usize;
                    let fade_frames = ((output_sample_rate.max(1) as f64) * 0.052)
                        .round()
                        .clamp(64.0, 3072.0) as usize;
                    smooth_preview_seam(&mut output, channels, seam_frame, fade_frames);
                }
                (result.preserve_pitch, result.detail)
            })
        } else {
            append_stretched_preview_part(
                &mut output,
                transition_source,
                source_sample_rate,
                output_sample_rate,
                channels,
                requested_rate,
                stretch_output_seconds,
                render_mode,
            )
            .map(|result| {
                if head_anchor_seconds > 0.0 {
                    let seam_frame = (head_anchor_seconds * output_sample_rate as f64)
                        .round()
                        .max(1.0) as usize;
                    let fade_frames = ((output_sample_rate.max(1) as f64) * 0.052)
                        .round()
                        .clamp(64.0, 3072.0) as usize;
                    smooth_preview_seam(&mut output, channels, seam_frame, fade_frames);
                }
                (result.preserve_pitch, result.detail)
            })
        };

        match stretch_result {
            Ok((used_preserve_pitch, detail)) => {
                preserve_pitch = used_preserve_pitch;
                stretch_detail = Some(format!(
                    "{} · gain-aware edge {:.2}s{}",
                    detail, normal_anchor_seconds, tempo_smoothing_suffix
                ));
            }
            Err(error) => {
                stretch_detail = Some(if error.contains("realtime budget") {
                    "Tempo off · stretch budget".to_owned()
                } else {
                    format!("Tempo off · stretch failed ({error})")
                });
                let plain_transition_frames = (transition_seconds * source_sample_rate as f64)
                    .round()
                    .clamp(0.0, source_frame_count as f64)
                    as usize;
                let plain_transition_end = plain_transition_frames
                    .saturating_mul(channels)
                    .min(source.len());
                output.clear();
                tail_start_sample = plain_transition_end;
                transition_source_frames_for_timeline = plain_transition_frames;
                output.extend(resample_interleaved_frames(
                    &source[..plain_transition_end],
                    source_sample_rate,
                    output_sample_rate,
                    channels,
                    Some(
                        (transition_seconds * output_sample_rate as f64)
                            .round()
                            .max(1.0) as usize,
                    ),
                ));
                applied_rate = 1.0;
                preserve_pitch = false;
            }
        }
    } else if !transition_source.is_empty() {
        output.extend(resample_interleaved_frames(
            transition_source,
            source_sample_rate,
            output_sample_rate,
            channels,
            Some(
                (source_transition_seconds * output_sample_rate as f64)
                    .round()
                    .max(1.0) as usize,
            ),
        ));
        if micro_stream_mix && rate_delta >= MUSIC_MIX_TEMPO_JND_RATE {
            stretch_detail = Some(format!(
                "Micro Stream Mix · stretch bypass{}",
                tempo_smoothing_suffix
            ));
        } else if !tempo_smoothing_suffix.is_empty() {
            stretch_detail = Some(format!("Tempo gentle{}", tempo_smoothing_suffix));
        }
    }

    if preserve_pitch && transition_seconds > 0.0 {
        let exit_anchor_frames = (exit_anchor_seconds * source_sample_rate as f64)
            .round()
            .clamp(0.0, source_frame_count as f64) as usize;
        transition_source_frames_for_timeline = head_anchor_frames
            .saturating_add(source_transition_frames)
            .saturating_add(exit_anchor_frames)
            .min(source_frame_count);
        applied_rate = (transition_source_frames_for_timeline as f64
            / source_sample_rate as f64
            / transition_seconds)
            .clamp(0.92, 1.08);
    }

    // v10.12.15: the stretch -> normal-B bridge must be time-aligned.  The
    // previous trial used only a tiny pre-roll under a much longer seam, which
    // effectively collapsed part of B's head into the overlap and could sound as
    // if a beat/consonant was eaten.  Use a pre-roll that matches the actual seam
    // duration, so the crossfade blends duplicate time instead of adjacent time.
    let tail_preroll_seconds = mix_preview_tail_seam_seconds(transition_seconds);
    let tail_preroll_frames = ((source_sample_rate.max(1) as f64) * tail_preroll_seconds)
        .round()
        .clamp(0.0, 4096.0) as usize;
    let tail_start_with_preroll = tail_start_sample
        .saturating_sub(tail_preroll_frames.saturating_mul(channels))
        .min(source.len());
    let tail_source = &source[tail_start_with_preroll..];
    if !tail_source.is_empty() {
        let tail_output = resample_interleaved_frames(
            tail_source,
            source_sample_rate,
            output_sample_rate,
            channels,
            None,
        );
        append_preview_tail_with_seam_crossfade(
            &mut output,
            &tail_output,
            channels,
            output_sample_rate,
            transition_seconds,
        );
    }

    // Keep B on one gain timeline. A rendered post-boundary lift followed by a
    // timed release creates an unrelated second fade around deck promotion.
    // The diagnostic below still reports the boundary delta for future fixes.
    sanitize_mix_preview_samples(
        &mut output,
        channels,
        output_sample_rate,
        transition_seconds,
    );

    let diagnostic_detail = mix_preview_diagnostic_detail(
        &output,
        output_sample_rate,
        channels,
        transition_seconds,
        head_anchor_seconds,
        exit_anchor_seconds,
        source_transition_frames,
        source_sample_rate,
        tail_preroll_seconds,
        requested_rate_raw,
        requested_rate,
        preserve_pitch,
    );
    stretch_detail = Some(match stretch_detail {
        Some(detail) if !detail.trim().is_empty() => {
            format!("{detail} · {diagnostic_detail}")
        }
        _ if preserve_pitch => format!("preserve active · {diagnostic_detail}"),
        _ => format!("preserve off · {diagnostic_detail}"),
    });

    MusicMixPreviewSegment {
        samples: output,
        transition_output_frames: MusicMixFrameClock::new(output_sample_rate)
            .frame_count_from_seconds(transition_seconds),
        transition_source_frames: MusicMixFrameCount::new(
            transition_source_frames_for_timeline as u64,
        ),
        source_start_frame: MusicMixFrameClock::new(source_sample_rate)
            .source_frame_from_seconds(source_start_seconds),
        source_sample_rate,
        transition_source_rate: if preserve_pitch { applied_rate } else { 1.0 },
        preserve_pitch,
        stretch_detail,
    }
}

struct StretchedPreviewPart {
    preserve_pitch: bool,
    detail: String,
}

fn append_stretched_preview_part(
    output: &mut Vec<f32>,
    source: &[f32],
    source_sample_rate: u32,
    output_sample_rate: u32,
    channels: usize,
    rate: f64,
    output_seconds: f64,
    render_mode: MusicMixRenderMode,
) -> Result<StretchedPreviewPart, String> {
    if source.is_empty() || output_seconds <= 0.0 {
        return Ok(StretchedPreviewPart {
            preserve_pitch: false,
            detail: "empty tempo part".to_owned(),
        });
    }

    let expected_frames = (output_seconds * output_sample_rate.max(1) as f64)
        .round()
        .max(1.0) as usize;
    let stretched = if render_mode.uses_high_quality_offline() {
        crate::app::music_timestretch::stretch_mix_transition_preserve_pitch_high_quality(
            source,
            source_sample_rate,
            channels,
            rate,
        )
        .or_else(|_| {
            crate::app::music_timestretch::stretch_mix_transition_preserve_pitch_to_frames(
                source,
                source_sample_rate,
                channels,
                rate,
                (output_seconds * source_sample_rate.max(1) as f64)
                    .round()
                    .max(1.0) as usize,
            )
        })?
    } else {
        let mut result =
            crate::app::music_timestretch::stretch_mix_transition_preserve_pitch_to_frames(
                source,
                source_sample_rate,
                channels,
                rate,
                (output_seconds * source_sample_rate.max(1) as f64)
                    .round()
                    .max(1.0) as usize,
            )?;
        if result.preserve_pitch {
            result.detail = format!("{} · Stream Mix", result.detail);
        }
        result
    };
    let samples = resample_interleaved_frames(
        &stretched.samples,
        source_sample_rate,
        output_sample_rate,
        channels,
        Some(expected_frames),
    );
    output.extend(samples);
    Ok(StretchedPreviewPart {
        preserve_pitch: stretched.preserve_pitch,
        detail: stretched.detail,
    })
}

fn append_gain_aware_stretched_preview_part(
    output: &mut Vec<f32>,
    source: &[f32],
    source_sample_rate: u32,
    output_sample_rate: u32,
    channels: usize,
    early_rate: f64,
    late_rate: f64,
    rate_change_source_frames: usize,
    output_seconds: f64,
    render_mode: MusicMixRenderMode,
) -> Result<StretchedPreviewPart, String> {
    if source.is_empty() || output_seconds <= 0.0 {
        return Ok(StretchedPreviewPart {
            preserve_pitch: false,
            detail: "empty tempo part".to_owned(),
        });
    }

    let expected_frames = (output_seconds * output_sample_rate.max(1) as f64)
        .round()
        .max(1.0) as usize;
    let source_frames = source.len() / channels.max(1);
    let split_frame = rate_change_source_frames.min(source_frames.saturating_sub(1));
    let rate_changes =
        perceptual_tempo_feather_rate_changes(source_frames, split_frame, early_rate, late_rate);
    let static_rate = (source_frames as f64 / source_sample_rate.max(1) as f64 / output_seconds)
        .clamp(0.965, 1.035);
    let stretched = if render_mode.uses_high_quality_offline() {
        crate::app::music_timestretch::stretch_mix_transition_preserve_pitch_high_quality(
            source,
            source_sample_rate,
            channels,
            static_rate,
        )
        .or_else(|_| {
            crate::app::music_timestretch::stretch_mix_transition_preserve_pitch_dynamic_to_frames(
                source,
                source_sample_rate,
                channels,
                early_rate,
                &rate_changes,
                (output_seconds * source_sample_rate.max(1) as f64)
                    .round()
                    .max(1.0) as usize,
            )
        })?
    } else {
        let mut result =
            crate::app::music_timestretch::stretch_mix_transition_preserve_pitch_dynamic_to_frames(
                source,
                source_sample_rate,
                channels,
                early_rate,
                &rate_changes,
                (output_seconds * source_sample_rate.max(1) as f64)
                    .round()
                    .max(1.0) as usize,
            )?;
        if result.preserve_pitch {
            result.detail = format!("{} · Stream Mix", result.detail);
        }
        result
    };
    let samples = resample_interleaved_frames(
        &stretched.samples,
        source_sample_rate,
        output_sample_rate,
        channels,
        Some(expected_frames),
    );
    output.extend(samples);
    Ok(StretchedPreviewPart {
        preserve_pitch: stretched.preserve_pitch,
        detail: stretched.detail,
    })
}

fn smooth_stage_mix_requested_rate(requested_rate: f64, artifact_risk: f64) -> f64 {
    let requested_rate = requested_rate.clamp(0.965, 1.035);
    let delta = requested_rate - 1.0;
    let abs_delta = delta.abs();
    if abs_delta <= MUSIC_MIX_TEMPO_DEADBAND_RATE {
        return 1.0;
    }

    // Soft-knee the small corrections first.  The psychological cue model still
    // decides when the capsule happens; this only prevents the preserve-pitch
    // lane from chasing sub-JND BPM differences and bringing compression noise
    // to the front.
    let knee = if abs_delta < MUSIC_MIX_TEMPO_SOFT_KNEE_RATE {
        let t = ((abs_delta - MUSIC_MIX_TEMPO_DEADBAND_RATE)
            / (MUSIC_MIX_TEMPO_SOFT_KNEE_RATE - MUSIC_MIX_TEMPO_DEADBAND_RATE).max(0.0001))
        .clamp(0.0, 1.0);
        0.42 + smoother_audio_fade(t) * 0.42
    } else {
        let t = ((abs_delta - MUSIC_MIX_TEMPO_SOFT_KNEE_RATE)
            / (0.035 - MUSIC_MIX_TEMPO_SOFT_KNEE_RATE))
            .clamp(0.0, 1.0);
        0.84 + smoother_audio_fade(t) * 0.08
    };
    let artifact_guard = 1.0 - artifact_risk.clamp(0.0, 1.0) * 0.18;
    let mut smoothed_delta = delta * knee * artifact_guard;
    if abs_delta >= 0.009 && smoothed_delta.abs() < MUSIC_MIX_TEMPO_JND_RATE {
        smoothed_delta = delta.signum() * MUSIC_MIX_TEMPO_JND_RATE.min(abs_delta);
    }
    let smoothed = 1.0 + smoothed_delta;
    if smoothed_delta.abs() < MUSIC_MIX_TEMPO_JND_RATE {
        1.0
    } else {
        smoothed.clamp(0.965, 1.035)
    }
}

fn tempo_smoothing_detail(raw_rate: f64, smoothed_rate: f64, artifact_risk: f64) -> String {
    let raw_delta = (raw_rate.clamp(0.965, 1.035) - 1.0).abs();
    let smooth_delta = (smoothed_rate.clamp(0.965, 1.035) - 1.0).abs();
    if raw_delta < MUSIC_MIX_TEMPO_JND_RATE && artifact_risk < 0.35 {
        return String::new();
    }
    if (raw_delta - smooth_delta).abs() < 0.0012 && artifact_risk < 0.35 {
        return String::new();
    }

    let mut parts = Vec::new();
    if smooth_delta <= f64::EPSILON && raw_delta >= MUSIC_MIX_TEMPO_JND_RATE {
        parts.push("tempo deadband".to_owned());
    } else if smooth_delta + 0.0012 < raw_delta {
        parts.push(format!(
            "tempo gentle {:+.1}%→{:+.1}%",
            (raw_rate - 1.0) * 100.0,
            (smoothed_rate - 1.0) * 100.0
        ));
    }
    if artifact_risk >= 0.35 {
        parts.push("artifact guard".to_owned());
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" · {}", parts.join(" · "))
    }
}

fn mix_preview_artifact_risk_score(source: &[f32], channels: usize) -> f64 {
    let channels = channels.max(1);
    let frames = source.len() / channels;
    if frames < 512 {
        return 0.0;
    }

    let stride = (frames / 48_000).max(1);
    let mut count = 0_usize;
    let mut sum_sq = 0.0_f64;
    let mut peak = 0.0_f64;
    let mut near_clip = 0_usize;
    for frame in (0..frames).step_by(stride) {
        let base = frame.saturating_mul(channels);
        let mut mono = 0.0_f64;
        for channel in 0..channels {
            mono += source.get(base + channel).copied().unwrap_or(0.0) as f64;
        }
        mono /= channels as f64;
        let abs = mono.abs();
        peak = peak.max(abs);
        sum_sq += mono * mono;
        if abs >= 0.965 {
            near_clip = near_clip.saturating_add(1);
        }
        count = count.saturating_add(1);
    }
    if count == 0 {
        return 0.0;
    }

    let rms = (sum_sq / count as f64).sqrt();
    if rms <= 1.0e-6 || peak <= 0.05 {
        return 0.0;
    }

    let crest = peak / rms.max(1.0e-6);
    let limited_risk = if peak >= 0.72 && crest <= 2.35 {
        ((2.35 - crest) / 0.85).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let clip_risk = (near_clip as f64 / count as f64 * 22.0).clamp(0.0, 1.0);
    (limited_risk * 0.55 + clip_risk * 0.45).clamp(0.0, 1.0)
}

fn smoother_audio_fade(ratio: f64) -> f64 {
    let t = ratio.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn perceptual_early_transition_rate(requested_rate: f64) -> f64 {
    // A longer transition makes pitch artifacts easier to hide, but a full BPM
    // correction can make B's beat feel like it is pulling ahead.  Since cue
    // selection now does more psychological work, keep even the low-volume lane
    // closer to B's natural groove.
    let delta = requested_rate.clamp(0.965, 1.035) - 1.0;
    let strength = if delta.abs() >= 0.025 { 0.62 } else { 0.68 };
    1.0 + delta * strength
}

fn perceptual_late_transition_rate(requested_rate: f64) -> f64 {
    // B is much more exposed near the end of fade-in, so this part should feel
    // nearly natural.  Leave only a hint of correction for continuity.
    1.0 + (requested_rate.clamp(0.965, 1.035) - 1.0) * 0.08
}

fn perceptual_tempo_feather_rate_changes(
    source_frames: usize,
    first_change_frame: usize,
    early_rate: f64,
    late_rate: f64,
) -> Vec<(usize, f64)> {
    if source_frames < 8 {
        return Vec::new();
    }

    let early_rate = early_rate.clamp(0.965, 1.035);
    let late_rate = late_rate.clamp(0.965, 1.035);
    let delta = late_rate - early_rate;
    if delta.abs() < MUSIC_MIX_TEMPO_FEATHER_MIN_GAP {
        return Vec::new();
    }

    // One abrupt rate step is easy to notice even when pitch is preserved.  Treat
    // tempo like a perceptual envelope: do most correction while B is quiet, then
    // feather toward the exposed late rate through several small JND-sized moves.
    let start_frame = first_change_frame
        .max((source_frames as f64 * 0.18).round() as usize)
        .min(source_frames.saturating_sub(1));
    let usable_frames = source_frames.saturating_sub(start_frame);
    if usable_frames < 8 {
        return vec![(start_frame, late_rate)];
    }

    let steps = if source_frames >= 144_000 {
        7
    } else if source_frames >= 96_000 {
        6
    } else if source_frames >= 48_000 {
        5
    } else {
        4
    };
    let mut changes = Vec::with_capacity(steps);
    let mut last_frame = 0_usize;
    let mut last_rate = early_rate;
    for step in 1..=steps {
        let t = step as f64 / steps as f64;
        // Bias the first changes slightly earlier: they are masked by the fade-in
        // and reduce the size of every later, more audible correction.  v10.12.39
        // keeps the same cue timing, but turns the rate lane into a softer curve
        // with smaller steps so Stream Mix does not feel like it is wobbling while
        // B is being stretched.
        let frame_curve = smoother_audio_fade(t);
        let frame = start_frame
            .saturating_add((usable_frames as f64 * frame_curve).round() as usize)
            .min(source_frames.saturating_sub(1));
        if frame <= last_frame {
            continue;
        }

        let ease = smoother_audio_fade(t);
        let mut target_rate = early_rate + delta * ease;
        if t >= 0.68 {
            // Final exposed region should feel closer to natural motion than math
            // would suggest.  Keep only a hint of correction near the handoff.
            let tail_relax = smoother_audio_fade(((t - 0.68) / 0.32).clamp(0.0, 1.0));
            target_rate = target_rate * (1.0 - tail_relax * 0.24) + 1.0 * tail_relax * 0.24;
        }
        target_rate = target_rate.clamp(0.965, 1.035);
        let max_step =
            MUSIC_MIX_TEMPO_FEATHER_MAX_STEP * if delta.abs() >= 0.020 { 1.25 } else { 1.0 };
        let rate_delta = (target_rate - last_rate).clamp(-max_step, max_step);
        let rate = (last_rate + rate_delta).clamp(0.965, 1.035);
        if (rate - last_rate).abs() >= MUSIC_MIX_TEMPO_FEATHER_MIN_GAP || step == steps {
            changes.push((frame, rate));
            last_frame = frame;
            last_rate = rate;
        }
    }

    changes
}

fn mix_preview_normal_anchor_seconds(transition_seconds: f64, requested_rate: f64) -> f64 {
    let rate_delta = (requested_rate - 1.0).abs();
    if transition_seconds < 1.2 || rate_delta < 0.006 {
        return 0.0;
    }

    // Keep a little more original-speed audio on both sides of the transformed
    // center when tempo matching is audible.  This makes the preview layout more
    // like: B normal head -> stretched middle -> B normal tail, so a possible
    // stretch/tail seam is not sitting directly on the listener's perceived handoff.
    let ratio = if rate_delta >= 0.045 {
        0.18
    } else if rate_delta >= 0.025 {
        0.16
    } else {
        MUSIC_MIX_NORMAL_ANCHOR_RATIO
    };
    let max_seconds = if rate_delta >= 0.045 {
        0.92
    } else if rate_delta >= 0.025 {
        MUSIC_MIX_NORMAL_ANCHOR_MAX_SECONDS
    } else {
        0.62
    };

    (transition_seconds * ratio)
        .clamp(MUSIC_MIX_NORMAL_ANCHOR_MIN_SECONDS, max_seconds)
        .min((transition_seconds * 0.24).max(0.0))
}

fn mix_preview_tail_seam_seconds(transition_seconds: f64) -> f64 {
    // Short enough to avoid eating musical time, long enough to hide residual
    // stretcher boundary noise.  The same duration is used for source pre-roll
    // and output crossfade so the bridge is sample-continuous in perceived time.
    if transition_seconds >= 10.5 {
        0.046
    } else if transition_seconds >= 7.5 {
        0.040
    } else {
        0.034
    }
}

fn append_preview_tail_with_seam_crossfade(
    output: &mut Vec<f32>,
    tail: &[f32],
    channels: usize,
    sample_rate: u32,
    transition_seconds: f64,
) {
    let channels = channels.max(1);
    if tail.is_empty() {
        return;
    }
    let output_frames = output.len() / channels;
    let tail_frames = tail.len() / channels;

    // v10.12.15: keep this seam short and exactly matched to the source
    // pre-roll.  A long seam with too little duplicate material shortens the B
    // head perceptually, which is heard as a 1-2 ms cut or a swallowed beat.
    let seam_seconds = mix_preview_tail_seam_seconds(transition_seconds);
    let seam_frames = ((sample_rate.max(1) as f64) * seam_seconds)
        .round()
        .clamp(192.0, 4096.0) as usize;
    let overlap_frames = seam_frames.min(output_frames).min(tail_frames);
    if overlap_frames < 24 {
        output.extend_from_slice(tail);
        return;
    }

    let output_start = output.len().saturating_sub(overlap_frames * channels);
    for frame in 0..overlap_frames {
        let ratio = frame as f32 / overlap_frames.saturating_sub(1).max(1) as f32;
        let fade = smooth_audio_fade(ratio);
        for channel in 0..channels {
            let out_index = output_start + frame * channels + channel;
            let tail_index = frame * channels + channel;
            if let (Some(out_sample), Some(tail_sample)) =
                (output.get_mut(out_index), tail.get(tail_index))
            {
                *out_sample = *out_sample * (1.0 - fade) + *tail_sample * fade;
            }
        }
    }

    let tail_start = overlap_frames.saturating_mul(channels).min(tail.len());
    output.extend_from_slice(&tail[tail_start..]);
}

fn mix_preview_diagnostic_detail(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
    transition_seconds: f64,
    head_anchor_seconds: f64,
    exit_anchor_seconds: f64,
    source_transition_frames: usize,
    source_sample_rate: u32,
    tail_preroll_seconds: f64,
    requested_rate_raw: f64,
    requested_rate: f64,
    preserve_pitch: bool,
) -> String {
    let seam_delta_db =
        mix_preview_seam_delta_db(samples, sample_rate, channels, transition_seconds, 0.080);
    let source_transition_seconds =
        source_transition_frames as f64 / source_sample_rate.max(1) as f64;
    let smoothed_percent = (requested_rate - 1.0) * 100.0;
    let raw_percent = (requested_rate_raw - 1.0) * 100.0;
    let mode = if preserve_pitch { "pp" } else { "no-pp" };
    format!(
        "diag {mode} seamΔ{seam_delta_db:+.1}dB/{:.0}ms anchor {:.2}+{:.2}s src {:.2}s raw {raw_percent:+.1}%→{smoothed_percent:+.1}%",
        tail_preroll_seconds * 1000.0,
        head_anchor_seconds,
        exit_anchor_seconds,
        source_transition_seconds,
    )
}

fn mix_preview_seam_delta_db(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
    transition_seconds: f64,
    window_seconds: f64,
) -> f32 {
    let channels = channels.max(1);
    let sample_rate = sample_rate.max(1);
    let frame_count = samples.len() / channels;
    if frame_count == 0 {
        return 0.0;
    }
    let seam_frame = (transition_seconds.max(0.0) * sample_rate as f64)
        .round()
        .clamp(0.0, frame_count as f64) as usize;
    let window_frames = ((sample_rate as f64) * window_seconds.max(0.005))
        .round()
        .clamp(96.0, 4096.0) as usize;
    let before_start = seam_frame.saturating_sub(window_frames);
    let after_start = seam_frame.min(frame_count);
    let before_rms = rms_interleaved_window(samples, before_start, window_frames, channels);
    let after_rms = rms_interleaved_window(samples, after_start, window_frames, channels);
    rms_delta_db(after_rms, before_rms)
}

fn rms_delta_db(after_rms: f32, before_rms: f32) -> f32 {
    amplitude_to_db_for_diag(after_rms) - amplitude_to_db_for_diag(before_rms)
}

fn amplitude_to_db_for_diag(value: f32) -> f32 {
    let value = value.max(1.0e-6);
    20.0 * value.log10()
}

fn rms_interleaved_window(
    samples: &[f32],
    start_frame: usize,
    frames: usize,
    channels: usize,
) -> f32 {
    let channels = channels.max(1);
    if samples.is_empty() || frames == 0 {
        return 0.0;
    }
    let frame_count = samples.len() / channels;
    if start_frame >= frame_count {
        return 0.0;
    }
    let end_frame = start_frame.saturating_add(frames).min(frame_count);
    let mut sum = 0.0_f64;
    let mut count = 0_usize;
    for frame in start_frame..end_frame {
        for channel in 0..channels {
            if let Some(sample) = samples.get(frame.saturating_mul(channels) + channel) {
                sum += f64::from(*sample) * f64::from(*sample);
                count = count.saturating_add(1);
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        (sum / count as f64).sqrt() as f32
    }
}

fn sanitize_mix_preview_samples(
    samples: &mut [f32],
    channels: usize,
    sample_rate: u32,
    transition_seconds: f64,
) {
    let channels = channels.max(1);
    for sample in samples.iter_mut() {
        if !sample.is_finite() || sample.abs() < 1.0e-20 {
            *sample = 0.0;
        } else {
            *sample = sample.clamp(-1.25, 1.25);
        }
    }

    let frame_count = samples.len() / channels;
    if frame_count == 0 {
        return;
    }

    let fade_frames = ((sample_rate.max(1) as f64) * 0.072)
        .round()
        .clamp(64.0, 4096.0) as usize;
    remove_preview_edge_dc(samples, channels, 0, fade_frames, true);

    // v10.12.35: do not hide Stream Mix boundary artifacts with an extra
    // capsule-edge gain fade.  The StreamProcessor path now exposes exact-sized
    // output through a reservoir, so this sanitizer only removes non-finite/DC
    // edge issues and keeps the musical envelope owned by the mix capsule.

    let seam_frame = (transition_seconds.max(0.0) * sample_rate.max(1) as f64).round() as usize;
    if seam_frame > 0 && seam_frame < frame_count {
        smooth_preview_seam(samples, channels, seam_frame, fade_frames);
    }
}

fn remove_preview_edge_dc(
    samples: &mut [f32],
    channels: usize,
    start_frame: usize,
    fade_frames: usize,
    from_start: bool,
) {
    let channels = channels.max(1);
    let frame_count = samples.len() / channels;
    if frame_count == 0 || fade_frames == 0 || start_frame >= frame_count {
        return;
    }

    for channel in 0..channels {
        let edge_index = if from_start {
            start_frame.saturating_mul(channels) + channel
        } else {
            frame_count.saturating_sub(1).saturating_mul(channels) + channel
        };
        let offset = samples.get(edge_index).copied().unwrap_or(0.0);
        if offset.abs() < 1.0e-5 {
            continue;
        }
        let available = if from_start {
            frame_count.saturating_sub(start_frame)
        } else {
            start_frame.saturating_add(1)
        };
        let frames = fade_frames.min(available);
        for frame in 0..frames {
            let ratio = frame as f32 / frames.max(1) as f32;
            let fade = if from_start {
                1.0 - smooth_audio_fade(ratio)
            } else {
                smooth_audio_fade(ratio)
            };
            let target_frame = if from_start {
                start_frame + frame
            } else {
                start_frame.saturating_sub(frames.saturating_sub(1).saturating_sub(frame))
            };
            if let Some(sample) = samples.get_mut(target_frame.saturating_mul(channels) + channel) {
                *sample -= offset * fade;
            }
        }
    }
}

fn smooth_preview_seam(
    samples: &mut [f32],
    channels: usize,
    seam_frame: usize,
    fade_frames: usize,
) {
    let channels = channels.max(1);
    let frame_count = samples.len() / channels;
    if seam_frame == 0 || seam_frame >= frame_count || fade_frames == 0 {
        return;
    }

    let before_frames = fade_frames.min(seam_frame);
    let after_frames = fade_frames.min(frame_count.saturating_sub(seam_frame));
    if before_frames == 0 || after_frames == 0 {
        return;
    }

    for channel in 0..channels {
        let before = samples
            .get(seam_frame.saturating_sub(1).saturating_mul(channels) + channel)
            .copied()
            .unwrap_or(0.0);
        let after = samples
            .get(seam_frame.saturating_mul(channels) + channel)
            .copied()
            .unwrap_or(0.0);
        let discontinuity = after - before;
        if discontinuity.abs() < 1.0e-4 {
            continue;
        }

        // Split the seam correction across both sides of the stretch→tail join.
        // Correcting only the post-seam side can still leave a short high-frequency
        // step right when the crossfade finishes, which is heard as a small "beep".
        for frame in 0..before_frames {
            let ratio = frame as f32 / before_frames.max(1) as f32;
            let weight = smooth_audio_fade(ratio) * 0.5;
            let index = (seam_frame - before_frames + frame).saturating_mul(channels) + channel;
            if let Some(sample) = samples.get_mut(index) {
                *sample += discontinuity * weight;
            }
        }
        for frame in 0..after_frames {
            let ratio = frame as f32 / after_frames.max(1) as f32;
            let weight = (1.0 - smooth_audio_fade(ratio)) * 0.5;
            let index = (seam_frame + frame).saturating_mul(channels) + channel;
            if let Some(sample) = samples.get_mut(index) {
                *sample -= discontinuity * weight;
            }
        }
    }
}

fn resample_interleaved_frames(
    source: &[f32],
    source_sample_rate: u32,
    output_sample_rate: u32,
    channels: usize,
    target_frames: Option<usize>,
) -> Vec<f32> {
    let channels = channels.max(1);
    let source_frame_count = source.len() / channels;
    if source_frame_count == 0 {
        return Vec::new();
    }
    let output_frame_count = target_frames.unwrap_or_else(|| {
        ((source_frame_count as f64 / source_sample_rate.max(1) as f64)
            * output_sample_rate.max(1) as f64)
            .round()
            .max(1.0) as usize
    });
    let mut output = Vec::with_capacity(output_frame_count * channels);
    let source_rate = source_sample_rate.max(1) as f64;
    let output_rate = output_sample_rate.max(1) as f64;
    let fit_to_target_frames = target_frames.is_some();

    for output_frame in 0..output_frame_count {
        let source_position = if fit_to_target_frames {
            // When a caller supplies target_frames, that buffer is a musical/time
            // capsule with an exact length.  Map the whole source slice into that
            // exact frame count instead of continuing to step by sample-rate ratio.
            // Otherwise a stretched buffer whose produced length is slightly
            // different from the requested duration can clamp/repeat its final
            // frame near the seam, which is heard as a tiny beep/pop.
            if output_frame_count <= 1 || source_frame_count <= 1 {
                0.0
            } else {
                output_frame as f64 * source_frame_count.saturating_sub(1) as f64
                    / output_frame_count.saturating_sub(1) as f64
            }
        } else {
            output_frame as f64 * source_rate / output_rate
        };
        let left = source_position.floor().max(0.0) as usize;
        let right = (left + 1).min(source_frame_count.saturating_sub(1));
        let left = left.min(source_frame_count.saturating_sub(1));
        let frac = (source_position - left as f64).clamp(0.0, 1.0) as f32;
        for channel in 0..channels {
            let a = source
                .get(left * channels + channel)
                .copied()
                .unwrap_or(0.0);
            let b = source.get(right * channels + channel).copied().unwrap_or(a);
            output.push(a + (b - a) * frac);
        }
    }
    output
}

struct MusicOutputStream {
    stream: Stream,
    sample_rate: u32,
    channels: usize,
}

fn build_output_stream(
    shared: Arc<SharedPlaybackState>,
    sample_rate: u32,
    channels: usize,
) -> Result<MusicOutputStream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "No default audio output device was found.".to_owned())?;
    let supported = device
        .default_output_config()
        .map_err(|error| format!("Could not read default audio output config: {error}"))?;
    let sample_format = supported.sample_format();
    let default_config = supported.config();
    let mut requested_config = default_config.clone();
    requested_config.channels = channels.clamp(1, u16::MAX as usize) as u16;
    requested_config.sample_rate = sample_rate;

    match build_output_stream_for_config(
        &device,
        shared.clone(),
        requested_config.clone(),
        sample_format,
    ) {
        Ok(stream) => Ok(MusicOutputStream {
            stream,
            sample_rate: requested_config.sample_rate,
            channels: usize::from(requested_config.channels),
        }),
        Err(requested_error) if requested_config != default_config => {
            eprintln!(
                "[music-stream] requested output {}Hz/{}ch rejected; using device mix {}Hz/{}ch: {}",
                requested_config.sample_rate,
                requested_config.channels,
                default_config.sample_rate,
                default_config.channels,
                requested_error
            );
            let stream = build_output_stream_for_config(
                &device,
                shared,
                default_config.clone(),
                sample_format,
            )
            .map_err(|default_error| {
                format!("{requested_error}; device mix format also failed: {default_error}")
            })?;
            Ok(MusicOutputStream {
                stream,
                sample_rate: default_config.sample_rate,
                channels: usize::from(default_config.channels),
            })
        }
        Err(error) => Err(error),
    }
}

fn build_output_stream_for_config(
    device: &cpal::Device,
    shared: Arc<SharedPlaybackState>,
    config: cpal::StreamConfig,
    sample_format: SampleFormat,
) -> Result<Stream, String> {
    match sample_format {
        SampleFormat::F32 => {
            let shared = shared.clone();
            device
                .build_output_stream(
                    config,
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
                    config,
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
                    config,
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

fn effective_output_volume(shared: &SharedPlaybackState) -> f32 {
    let duration_frames = shared.volume_fade_duration_frames.load(Ordering::Relaxed);
    let started_frame = shared
        .volume_fade_started_output_frame
        .load(Ordering::Relaxed);
    if duration_frames == 0 {
        return f32::from_bits(shared.volume_bits.load(Ordering::Relaxed)).clamp(0.0, 1.0);
    }

    let output_frame = shared.output_frames_rendered.load(Ordering::Relaxed);
    let elapsed_frames = output_frame.saturating_sub(started_frame);
    let from = f32::from_bits(shared.volume_fade_from_bits.load(Ordering::Relaxed)).clamp(0.0, 1.0);
    let to = f32::from_bits(shared.volume_fade_to_bits.load(Ordering::Relaxed)).clamp(0.0, 1.0);
    if elapsed_frames >= duration_frames {
        shared.volume_bits.store(to.to_bits(), Ordering::Relaxed);
        shared
            .volume_fade_started_output_frame
            .store(0, Ordering::Relaxed);
        shared
            .volume_fade_duration_frames
            .store(0, Ordering::Relaxed);
        shared
            .volume_fade_curve_bits
            .store(0.0_f32.to_bits(), Ordering::Relaxed);
        return to;
    }

    let ratio = elapsed_frames as f32 / duration_frames.max(1) as f32;
    let mix_curve =
        f32::from_bits(shared.volume_fade_curve_bits.load(Ordering::Relaxed)).clamp(0.0, 1.0);
    if to <= 0.001 && from > to {
        return (from * direct_stage_mix_curve_fade_out(ratio, mix_curve)).clamp(0.0, 1.0);
    }
    if from <= 0.001 && to > from {
        return (to * direct_stage_mix_curve_fade_in(ratio, mix_curve)).clamp(0.0, 1.0);
    }
    let curve = smooth_audio_fade(ratio);
    (from + (to - from) * curve).clamp(0.0, 1.0)
}

fn smooth_audio_fade(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn direct_stage_mix_curve_fade_in(value: f32, mix_curve: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    let smooth = smooth_audio_fade(t);
    let equal = crossfade_equal_power_fade_in(t);
    // Show Blend keeps the same safe Direct Stream owner, but lets B arrive
    // earlier so long overlaps can sound more like a DJ layer instead of only a
    // handoff valley.  It is intentionally bounded; the slider blends into it.
    let show = equal.powf(0.78).clamp(0.0, 1.0);
    blend_stage_mix_curve(smooth, equal, show, mix_curve)
}

fn direct_stage_mix_curve_fade_out(value: f32, mix_curve: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    let equal = crossfade_equal_power_fade_out(t);
    // Current release candidate already uses equal-power on A-out.  For the
    // show side of the slider, let A hold a tiny bit longer instead of dropping
    // early; B's curve does most of the audible blend change.
    let show = equal.powf(0.82).clamp(0.0, 1.0);
    blend_stage_mix_curve(equal, equal, show, mix_curve)
}

fn blend_stage_mix_curve(smooth: f32, equal: f32, show: f32, mix_curve: f32) -> f32 {
    let curve = mix_curve.clamp(0.0, 1.0);
    if curve <= 0.5 {
        let t = curve / 0.5;
        (smooth + (equal - smooth) * t).clamp(0.0, 1.0)
    } else {
        let t = (curve - 0.5) / 0.5;
        (equal + (show - equal) * t).clamp(0.0, 1.0)
    }
}

fn crossfade_equal_power_fade_in(value: f32) -> f32 {
    let t = smooth_audio_fade(value);
    (t * std::f32::consts::FRAC_PI_2).sin().clamp(0.0, 1.0)
}

fn crossfade_equal_power_fade_out(value: f32) -> f32 {
    let t = smooth_audio_fade(value);
    (t * std::f32::consts::FRAC_PI_2).cos().clamp(0.0, 1.0)
}

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

fn reset_transition_load_diagnostics(shared: &SharedPlaybackState, duration_millis: u64) {
    let now_millis = current_time_millis();
    shared
        .transition_load_diag_started_millis
        .store(now_millis, Ordering::Relaxed);
    shared
        .transition_load_diag_duration_millis
        .store(duration_millis.max(1), Ordering::Relaxed);
    shared
        .transition_load_diag_last_callback_millis
        .store(0, Ordering::Relaxed);
    shared
        .transition_load_diag_callback_count
        .store(0, Ordering::Relaxed);
    shared
        .transition_load_diag_late_count
        .store(0, Ordering::Relaxed);
    shared
        .transition_load_diag_source_underfill_count
        .store(0, Ordering::Relaxed);
    shared
        .transition_load_diag_max_gap_millis
        .store(0, Ordering::Relaxed);
    shared
        .transition_load_diag_max_work_micros
        .store(0, Ordering::Relaxed);
    shared
        .transition_load_diag_min_main_queue_millis
        .store(u64::MAX, Ordering::Relaxed);
    shared
        .transition_load_diag_min_deck_queue_millis
        .store(u64::MAX, Ordering::Relaxed);
    shared
        .transition_load_diag_min_preview_queue_millis
        .store(u64::MAX, Ordering::Relaxed);
}

fn clear_transition_load_diagnostics(shared: &SharedPlaybackState) {
    shared
        .transition_load_diag_started_millis
        .store(0, Ordering::Relaxed);
    shared
        .transition_load_diag_duration_millis
        .store(0, Ordering::Relaxed);
    shared
        .transition_load_diag_last_callback_millis
        .store(0, Ordering::Relaxed);
}

fn transition_load_diag_is_active(shared: &SharedPlaybackState, now_millis: u64) -> bool {
    let started = shared
        .transition_load_diag_started_millis
        .load(Ordering::Relaxed);
    if started == 0 {
        return false;
    }
    let duration = shared
        .transition_load_diag_duration_millis
        .load(Ordering::Relaxed)
        .max(1);
    now_millis.saturating_sub(started)
        <= duration.saturating_add(MUSIC_TRANSITION_LOAD_DIAG_AFTER_MILLIS)
}

fn observe_transition_load_callback(
    shared: &SharedPlaybackState,
    callback_started_millis: u64,
    expected_gap_millis: u64,
    work_micros: u64,
    main_queue_millis: Option<u64>,
    deck_queue_millis: Option<u64>,
    preview_queue_millis: Option<u64>,
    source_underfilled: bool,
) {
    if !transition_load_diag_is_active(shared, callback_started_millis) {
        return;
    }

    shared
        .transition_load_diag_callback_count
        .fetch_add(1, Ordering::Relaxed);
    shared
        .transition_load_diag_max_work_micros
        .fetch_max(work_micros, Ordering::Relaxed);

    if source_underfilled {
        shared
            .transition_load_diag_source_underfill_count
            .fetch_add(1, Ordering::Relaxed);
    }

    if let Some(value) = main_queue_millis {
        shared
            .transition_load_diag_min_main_queue_millis
            .fetch_min(value, Ordering::Relaxed);
    }
    if let Some(value) = deck_queue_millis {
        shared
            .transition_load_diag_min_deck_queue_millis
            .fetch_min(value, Ordering::Relaxed);
    }
    if let Some(value) = preview_queue_millis {
        shared
            .transition_load_diag_min_preview_queue_millis
            .fetch_min(value, Ordering::Relaxed);
    }

    let previous = shared
        .transition_load_diag_last_callback_millis
        .swap(callback_started_millis, Ordering::Relaxed);
    if previous > 0 {
        let gap = callback_started_millis.saturating_sub(previous);
        shared
            .transition_load_diag_max_gap_millis
            .fetch_max(gap, Ordering::Relaxed);
        let late_threshold = expected_gap_millis
            .saturating_mul(MUSIC_TRANSITION_LOAD_MIN_GAP_FACTOR)
            .saturating_add(MUSIC_TRANSITION_LOAD_LATE_GRACE_MILLIS);
        if gap > late_threshold {
            shared
                .transition_load_diag_late_count
                .fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn transition_load_diagnostic_summary(
    shared: &SharedPlaybackState,
    promo_micros: u64,
    buffer_lock_micros: u64,
    deck_lock_micros: u64,
    promote_main_queue_millis: u64,
    promote_next_queue_millis: u64,
) -> Option<String> {
    let started = shared
        .transition_load_diag_started_millis
        .load(Ordering::Relaxed);
    if started == 0 {
        return None;
    }

    let callbacks = shared
        .transition_load_diag_callback_count
        .load(Ordering::Relaxed);
    let late = shared
        .transition_load_diag_late_count
        .load(Ordering::Relaxed);
    let underfill = shared
        .transition_load_diag_source_underfill_count
        .load(Ordering::Relaxed);
    let max_gap = shared
        .transition_load_diag_max_gap_millis
        .load(Ordering::Relaxed);
    let max_work = shared
        .transition_load_diag_max_work_micros
        .load(Ordering::Relaxed);
    let main_min = load_diag_min_text(
        shared
            .transition_load_diag_min_main_queue_millis
            .load(Ordering::Relaxed),
    );
    let deck_min = load_diag_min_text(
        shared
            .transition_load_diag_min_deck_queue_millis
            .load(Ordering::Relaxed),
    );
    let preview_min = load_diag_min_text(
        shared
            .transition_load_diag_min_preview_queue_millis
            .load(Ordering::Relaxed),
    );

    Some(format!(
        "promo={:.1}ms locks={:.1}/{:.1}ms cb={callbacks} late={late} maxgap={max_gap}ms maxwork={max_work}us underfill={underfill} qmin main={main_min} deck={deck_min} next={preview_min} promoteq main={}ms next={}ms",
        micros_to_millis(promo_micros),
        micros_to_millis(buffer_lock_micros),
        micros_to_millis(deck_lock_micros),
        promote_main_queue_millis,
        promote_next_queue_millis,
    ))
}

fn micros_to_millis(value: u64) -> f64 {
    value as f64 / 1000.0
}

fn elapsed_micros(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

fn load_diag_min_text(value: u64) -> String {
    if value == u64::MAX {
        "n/a".to_owned()
    } else {
        format!("{value}ms")
    }
}

fn samples_to_millis(samples: usize, sample_rate: u32, channels: usize) -> u64 {
    let channels = channels.max(1);
    let sample_rate = sample_rate.max(1) as f64;
    let frames = samples / channels;
    ((frames as f64 / sample_rate) * 1000.0)
        .round()
        .clamp(0.0, u64::MAX as f64) as u64
}

fn align_prepared_mix_deck_to_playback_cursor(
    deck: &mut CrossfadePreviewDeck,
    shared: &SharedPlaybackState,
    output_sample_rate: u32,
    output_channels: usize,
) {
    if deck.mode != CrossfadeDeckMode::PreparedMix || deck.prepared_mix_alignment_applied {
        return;
    }
    deck.prepared_mix_alignment_applied = true;

    let Some(prepared_start_seconds) = deck.prepared_mix_start_seconds else {
        return;
    };
    let output_sample_rate = output_sample_rate.max(1);
    let output_channels = output_channels.max(1);
    let playback_seconds = shared.samples_played.load(Ordering::Relaxed) as f64
        / output_sample_rate as f64
        / output_channels as f64;
    let late_seconds = playback_seconds - prepared_start_seconds;
    if !late_seconds.is_finite() || late_seconds <= 0.0 {
        return;
    }

    let requested_frames =
        MusicMixFrameClock::new(output_sample_rate).frame_count_from_seconds(late_seconds);
    let available_frames = deck.buffer.len() / output_channels;
    let max_queue_drop = available_frames.saturating_sub(64);
    let max_transition_drop = deck.transition_output_frames.get().saturating_sub(64) as usize;
    let drop_frames = requested_frames
        .get()
        .min(max_queue_drop.min(max_transition_drop) as u64);
    let trimmed_frames = trim_interleaved_queue_front(
        &mut deck.buffer,
        output_channels,
        MusicMixFrameCount::new(drop_frames),
    );
    if trimmed_frames.is_zero() {
        return;
    }

    deck.transition_output_frames = MusicMixFrameCount::new(
        deck.transition_output_frames
            .get()
            .saturating_sub(trimmed_frames.get())
            .max(1),
    );
    deck.prepared_mix_start_seconds = Some(
        prepared_start_seconds
            + MusicMixFrameClock::new(output_sample_rate).seconds_from_frame_count(trimmed_frames),
    );
    let late_millis = late_seconds * 1000.0;
    eprintln!(
        "[music-stage-prepared] callback align late={late_millis:.1}ms frames={} playback={playback_seconds:.3}s start={prepared_start_seconds:.3}s",
        trimmed_frames.get()
    );
}

fn trim_interleaved_queue_front(
    queue: &mut VecDeque<f32>,
    channels: usize,
    frames: MusicMixFrameCount,
) -> MusicMixFrameCount {
    let channels = channels.max(1);
    let requested_samples = (frames.get() as usize).saturating_mul(channels);
    if requested_samples == 0 || queue.is_empty() {
        return MusicMixFrameCount::ZERO;
    }
    let removable_samples = requested_samples.min(queue.len());
    let aligned_samples = removable_samples - (removable_samples % channels);
    for _ in 0..aligned_samples {
        let _ = queue.pop_front();
    }
    MusicMixFrameCount::new((aligned_samples / channels) as u64)
}

fn write_output_samples<T>(data: &mut [T], shared: &SharedPlaybackState, convert: impl Fn(f32) -> T)
where
    T: Copy,
{
    let callback_work_started = Instant::now();
    let callback_started_millis = current_time_millis();
    let stopped = shared.stop_requested.load(Ordering::Relaxed);
    let paused = shared.paused.load(Ordering::Relaxed);
    let volume = effective_output_volume(shared);
    let silence = convert(0.0);

    if stopped || paused {
        data.fill(silence);
        return;
    }

    let output_channels = shared.channels.load(Ordering::Relaxed).max(1) as usize;
    let output_sample_rate = shared.sample_rate.load(Ordering::Relaxed).max(1);
    let callback_frames = MusicMixFrameCount::new((data.len() / output_channels.max(1)) as u64);
    let expected_gap_millis =
        samples_to_millis(data.len(), output_sample_rate, output_channels).max(1);
    let mut consumed = 0_u64;
    let mut main_written_frames = MusicMixFrameCount::ZERO;
    let mut main_queue_after_millis = None;
    let mut deck_queue_after_millis = None;
    let mut preview_queue_after_millis = None;
    let mut source_underfilled = false;
    let mut main_samples = vec![0.0_f32; data.len()];
    let outgoing_rate = active_outgoing_transition_rate(shared);
    if let Ok(mut buffer) = shared.buffer.lock() {
        let main_queue_before_samples = buffer.len();
        if (outgoing_rate - 1.0).abs() >= 0.002 {
            let mut phase = f64::from_bits(
                shared
                    .outgoing_transition_phase_bits
                    .load(Ordering::Relaxed),
            );
            let fill = fill_interleaved_from_queue_with_rate(
                &mut buffer,
                &mut main_samples,
                output_channels,
                outgoing_rate,
                &mut phase,
            );
            consumed = fill.consumed_samples;
            main_written_frames = fill.written_frames;
            shared
                .outgoing_transition_phase_bits
                .store(phase.to_bits(), Ordering::Relaxed);
        } else {
            let fill = fill_interleaved_from_queue(&mut buffer, &mut main_samples, output_channels);
            consumed = fill.consumed_samples;
            main_written_frames = fill.written_frames;
        }
        if main_queue_before_samples > 0 && main_written_frames < callback_frames {
            source_underfilled = true;
        }
        main_queue_after_millis = Some(samples_to_millis(
            buffer.len(),
            output_sample_rate,
            output_channels,
        ));
    }

    let mut streaming_main_consumed = 0_u64;
    let mut streaming_main_written_frames = MusicMixFrameCount::ZERO;
    let mut streaming_main_samples = vec![0.0_f32; data.len()];
    let mut preview_raw_samples = vec![0.0_f32; data.len()];
    let mut preview_gain_envelope = vec![0.0_f32; data.len()];
    let mut preview_gain = 0.0_f32;
    let mut preview_target_volume = 0.0_f32;
    let mut preview_raw_rms = 0.0_f32;
    let mut preview_transition_frames = MusicMixFrameCount::ZERO;
    let mut outgoing_highlight_end_phase = None;
    let mut prepared_mix_preview_active = false;
    if let Ok(mut decks) = shared.crossfade_decks.lock() {
        if let Some(deck) = decks.main.as_mut() {
            let deck_queue_before_samples = deck.buffer.len();
            let rate = active_deck_outgoing_rate(deck);
            if (rate - 1.0).abs() >= 0.002 {
                let fill = fill_interleaved_from_queue_with_rate(
                    &mut deck.buffer,
                    &mut streaming_main_samples,
                    output_channels,
                    rate,
                    &mut deck.outgoing_transition_phase_frames,
                );
                streaming_main_consumed = fill.consumed_samples;
                streaming_main_written_frames = fill.written_frames;
                deck.output_frames_consumed = deck
                    .output_frames_consumed
                    .saturating_add(fill.written_frames);
                deck.rendered_frames_consumed = deck
                    .rendered_frames_consumed
                    .saturating_add(fill.consumed_frames);
            } else {
                let fill = fill_interleaved_from_queue(
                    &mut deck.buffer,
                    &mut streaming_main_samples,
                    output_channels,
                );
                streaming_main_consumed = fill.consumed_samples;
                streaming_main_written_frames = fill.written_frames;
                deck.output_frames_consumed = deck
                    .output_frames_consumed
                    .saturating_add(fill.written_frames);
                deck.rendered_frames_consumed = deck
                    .rendered_frames_consumed
                    .saturating_add(fill.consumed_frames);
            }
            if deck_queue_before_samples > 0 && streaming_main_written_frames < callback_frames {
                source_underfilled = true;
            }
            deck_queue_after_millis = Some(samples_to_millis(
                deck.buffer.len(),
                output_sample_rate,
                output_channels,
            ));
            if deck.buffer.is_empty() || deck.release_finished() {
                decks.main = None;
            }
        }
        if let Some(deck) = decks.next.as_mut() {
            align_prepared_mix_deck_to_playback_cursor(
                deck,
                shared,
                output_sample_rate,
                output_channels,
            );
            let preview_queue_before_samples = deck.buffer.len();
            let deck_mode = deck.mode;
            preview_target_volume = deck.target_volume;
            preview_transition_frames = deck.transition_output_frames;
            outgoing_highlight_end_phase = deck.outgoing_highlight_end_phase;
            let start_output_frame = deck.output_frames_consumed;
            let fill = fill_interleaved_from_queue(
                &mut deck.buffer,
                &mut preview_raw_samples,
                output_channels,
            );
            let preview_consumed = fill.consumed_samples;
            let preview_written_frames = fill.written_frames;
            deck.output_frames_consumed = deck
                .output_frames_consumed
                .saturating_add(preview_written_frames);
            deck.rendered_frames_consumed = deck
                .rendered_frames_consumed
                .saturating_add(fill.consumed_frames);

            if preview_consumed > 0 {
                if deck_mode == CrossfadeDeckMode::PreparedMix {
                    // Prepared Mix samples already contain the whole A+B capsule.
                    // The callback must not apply the realtime preview fade-in
                    // again or the rendered mix will dip at the handoff.
                    prepared_mix_preview_active = true;
                    let valid_samples = (preview_written_frames.get() as usize)
                        .saturating_mul(output_channels)
                        .min(preview_gain_envelope.len());
                    for slot in preview_gain_envelope.iter_mut().take(valid_samples) {
                        *slot = deck.target_volume;
                    }
                } else {
                    fill_preview_gain_envelope(
                        &mut preview_gain_envelope,
                        output_channels,
                        start_output_frame,
                        preview_written_frames,
                        deck,
                    );
                }
                preview_raw_rms = rms_audio_samples_for_frames(
                    &preview_raw_samples,
                    preview_written_frames,
                    output_channels,
                );
                preview_gain = metric_gain_from_envelope(&preview_gain_envelope, output_channels);
            }

            if preview_queue_before_samples > 0 && preview_written_frames < callback_frames {
                source_underfilled = true;
            }
            preview_queue_after_millis = Some(samples_to_millis(
                deck.buffer.len(),
                output_sample_rate,
                output_channels,
            ));
            if deck.release_finished() || (deck.buffer.is_empty() && !deck.transition_complete()) {
                decks.next = None;
            }
        }
    }

    if prepared_mix_preview_active {
        shared
            .crossfade_compensation_bits
            .store(1.0_f32.to_bits(), Ordering::Relaxed);
        shared
            .preview_level_guard_bits
            .store(1.0_f32.to_bits(), Ordering::Relaxed);
        shared
            .reward_energy_duck_bits
            .store(0.0_f32.to_bits(), Ordering::Relaxed);
        // Prepared Mix already contains the rendered A tail, overlap, and B
        // entry. Treat it as the sole segment source here; mixing live A again
        // reintroduces the exact A rollback this path is meant to remove.
        for (index, out) in data.iter_mut().enumerate() {
            let deck_gain = preview_gain_envelope.get(index).copied().unwrap_or(0.0);
            let prepared = preview_raw_samples[index] * deck_gain;
            *out = convert(soft_limit_audio_sample(prepared));
        }
        shared.samples_played.fetch_add(
            callback_frames.get().saturating_mul(output_channels as u64),
            Ordering::Relaxed,
        );
        shared
            .output_frames_rendered
            .fetch_add(callback_frames.get(), Ordering::Relaxed);

        let work_micros = callback_work_started
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        observe_transition_load_callback(
            shared,
            callback_started_millis,
            expected_gap_millis,
            work_micros,
            main_queue_after_millis,
            deck_queue_after_millis,
            preview_queue_after_millis,
            source_underfilled,
        );
        return;
    }

    let main_rms_frames = main_written_frames.max(streaming_main_written_frames);
    let main_raw_rms_for_mix = rms_audio_sample_pairs_for_frames(
        &main_samples,
        &streaming_main_samples,
        main_rms_frames,
        output_channels,
    );
    // Underfill release samples and zero padding are continuity output, not
    // loudness evidence. Keep that callback on deterministic frame envelopes;
    // feeding its RMS into adaptive gains caused false B lifts and A drops.
    let adaptive_gain_allowed = !source_underfilled;
    let energy_duck_target = if adaptive_gain_allowed && preview_gain > 0.0001 {
        reward_energy_duck_target(
            volume,
            preview_gain,
            preview_target_volume,
            main_raw_rms_for_mix,
            preview_raw_rms,
            preview_transition_frames,
            output_sample_rate,
            outgoing_highlight_end_phase,
        )
    } else {
        0.0
    };
    let energy_duck_amount = if adaptive_gain_allowed {
        smooth_reward_energy_duck(&shared.reward_energy_duck_bits, energy_duck_target)
    } else {
        shared
            .reward_energy_duck_bits
            .store(0.0_f32.to_bits(), Ordering::Relaxed);
        0.0
    };
    let outgoing_main_gain = if adaptive_gain_allowed && preview_gain > 0.0001 {
        reward_energy_aware_outgoing_main_gain(
            volume,
            preview_gain,
            preview_target_volume,
            main_raw_rms_for_mix,
            preview_raw_rms,
            preview_transition_frames,
            output_sample_rate,
            energy_duck_amount,
            outgoing_highlight_end_phase,
        )
    } else {
        volume
    };

    // v10.12.31: keep the preview-side vocal guard, but add a tiny, smoothed
    // floor support only when the overlap really sags.  It is intentionally
    // capped and reduced while the preview guard is active, so B vocals do not
    // jump forward just because the bus is being filled.
    let (crossfade_compensation, preview_level_guard) = if adaptive_gain_allowed
        && preview_gain > 0.0001
    {
        let main_raw_rms = main_raw_rms_for_mix;
        let guard_target = preview_relative_level_guard(
            main_raw_rms,
            preview_raw_rms,
            outgoing_main_gain,
            preview_gain,
        );
        let preview_level_guard =
            smooth_mix_level_guard(&shared.preview_level_guard_bits, guard_target);
        let compensation_target = crossfade_floor_support_compensation(
            main_raw_rms,
            preview_raw_rms,
            outgoing_main_gain,
            preview_gain,
            preview_target_volume,
            preview_level_guard,
        );
        (
            smooth_mix_bus_compensation(&shared.crossfade_compensation_bits, compensation_target),
            preview_level_guard,
        )
    } else if source_underfilled {
        shared
            .crossfade_compensation_bits
            .store(1.0_f32.to_bits(), Ordering::Relaxed);
        shared
            .preview_level_guard_bits
            .store(1.0_f32.to_bits(), Ordering::Relaxed);
        (1.0, 1.0)
    } else {
        (
            smooth_mix_bus_compensation(&shared.crossfade_compensation_bits, 1.0),
            smooth_mix_level_guard(&shared.preview_level_guard_bits, 1.0),
        )
    };

    for (index, out) in data.iter_mut().enumerate() {
        let sample_preview_gain = preview_gain_envelope.get(index).copied().unwrap_or(0.0);
        let outgoing_main_gain = if adaptive_gain_allowed && sample_preview_gain > 0.0001 {
            reward_energy_aware_outgoing_main_gain(
                volume,
                sample_preview_gain,
                preview_target_volume,
                main_raw_rms_for_mix,
                preview_raw_rms,
                preview_transition_frames,
                output_sample_rate,
                energy_duck_amount,
                outgoing_highlight_end_phase,
            )
        } else {
            volume
        };
        let main = (main_samples[index] + streaming_main_samples[index]) * outgoing_main_gain;
        let preview = preview_raw_samples[index] * sample_preview_gain * preview_level_guard;
        let mixed = (main + preview) * crossfade_compensation;
        *out = convert(soft_limit_audio_sample(mixed));
    }

    let total_consumed = consumed.saturating_add(streaming_main_consumed);
    if total_consumed > 0 {
        shared
            .samples_played
            .fetch_add(total_consumed, Ordering::Relaxed);
    }
    shared
        .output_frames_rendered
        .fetch_add(callback_frames.get(), Ordering::Relaxed);

    let work_micros = callback_work_started
        .elapsed()
        .as_micros()
        .min(u128::from(u64::MAX)) as u64;
    observe_transition_load_callback(
        shared,
        callback_started_millis,
        expected_gap_millis,
        work_micros,
        main_queue_after_millis,
        deck_queue_after_millis,
        preview_queue_after_millis,
        source_underfilled,
    );
}

fn fill_preview_gain_envelope(
    envelope: &mut [f32],
    channels: usize,
    start_output_frame: MusicMixFrameCount,
    written_frames: MusicMixFrameCount,
    deck: &CrossfadePreviewDeck,
) {
    let channels = channels.max(1);
    if envelope.is_empty() || written_frames.is_zero() {
        return;
    }

    let output_frames = envelope.len() / channels;
    let written_frames = (written_frames.get() as usize).min(output_frames);
    if written_frames == 0 {
        return;
    }

    if let Some(release_start) = deck.release_started_output_frame {
        let release_elapsed_frames = start_output_frame.get().saturating_sub(release_start.get());
        let release_duration_frames = deck.release_duration_frames.get().max(1);
        for frame in 0..written_frames {
            let elapsed = release_elapsed_frames.saturating_add(frame as u64);
            let gain = if elapsed >= release_duration_frames {
                0.0
            } else {
                let ratio = elapsed as f32 / release_duration_frames as f32;
                (deck.release_from_gain * (1.0 - smooth_audio_fade(ratio))).clamp(0.0, 1.0)
            };
            for channel in 0..channels {
                if let Some(slot) = envelope.get_mut(frame.saturating_mul(channels) + channel) {
                    *slot = gain;
                }
            }
        }
        return;
    }

    let duration_frames = deck.transition_output_frames.get().max(1);
    let start_frame = start_output_frame.get();
    for frame in 0..written_frames {
        let playback_frame = start_frame.saturating_add(frame as u64);
        let ratio = playback_frame as f32 / duration_frames.max(1) as f32;
        let gain = if playback_frame >= duration_frames {
            deck.target_volume
        } else {
            deck.target_volume * crossfade_equal_power_fade_in(ratio)
        }
        .clamp(0.0, 1.0);
        for channel in 0..channels {
            if let Some(slot) = envelope.get_mut(frame.saturating_mul(channels) + channel) {
                *slot = gain;
            }
        }
    }
}

fn metric_gain_from_envelope(envelope: &[f32], channels: usize) -> f32 {
    let channels = channels.max(1);
    let frame_count = envelope.len() / channels;
    if frame_count == 0 {
        return 0.0;
    }

    // Use a near-centre gain for the block-level RMS guards.  The audio path
    // itself uses the per-frame envelope below; this scalar is only for slow
    // loudness guard decisions.
    let mid_frame = frame_count / 2;
    let mid_gain = envelope
        .get(mid_frame.saturating_mul(channels))
        .copied()
        .unwrap_or(0.0);
    if mid_gain > 0.0001 {
        return mid_gain;
    }

    envelope.iter().copied().fold(0.0_f32, f32::max)
}

fn active_outgoing_transition_rate(shared: &SharedPlaybackState) -> f64 {
    let started_frame = shared
        .outgoing_transition_started_output_frame
        .load(Ordering::Relaxed);
    let duration_frames = shared
        .outgoing_transition_duration_frames
        .load(Ordering::Relaxed);
    if duration_frames == 0 {
        return 1.0;
    }
    let output_frame = shared.output_frames_rendered.load(Ordering::Relaxed);
    let elapsed_frames = output_frame.saturating_sub(started_frame);
    if elapsed_frames >= duration_frames {
        shared
            .outgoing_transition_rate_bits
            .store(1.0_f64.to_bits(), Ordering::Relaxed);
        shared
            .outgoing_transition_started_output_frame
            .store(0, Ordering::Relaxed);
        shared
            .outgoing_transition_duration_frames
            .store(0, Ordering::Relaxed);
        shared
            .outgoing_transition_phase_bits
            .store(0.0_f64.to_bits(), Ordering::Relaxed);
        return 1.0;
    }
    let target_rate = f64::from_bits(shared.outgoing_transition_rate_bits.load(Ordering::Relaxed))
        .clamp(0.92, 1.08);
    let elapsed_ratio = elapsed_frames as f64 / duration_frames.max(1) as f64;
    eased_tempo_rate(target_rate, elapsed_ratio)
}

fn active_deck_outgoing_rate(deck: &mut CrossfadePreviewDeck) -> f64 {
    let Some(started_frame) = deck.outgoing_transition_started_output_frame else {
        return 1.0;
    };
    if deck.outgoing_transition_duration_frames.is_zero() {
        return 1.0;
    }
    let elapsed_frames = deck
        .output_frames_consumed
        .get()
        .saturating_sub(started_frame.get());
    if elapsed_frames >= deck.outgoing_transition_duration_frames.get() {
        deck.outgoing_transition_rate = 1.0;
        deck.outgoing_transition_phase_frames = 0.0;
        deck.outgoing_transition_started_output_frame = None;
        deck.outgoing_transition_duration_frames = MusicMixFrameCount::ZERO;
        return 1.0;
    }
    let elapsed_ratio =
        elapsed_frames as f64 / deck.outgoing_transition_duration_frames.get().max(1) as f64;
    eased_tempo_rate(
        deck.outgoing_transition_rate.clamp(0.92, 1.08),
        elapsed_ratio,
    )
}

fn eased_tempo_rate(target_rate: f64, elapsed_ratio: f64) -> f64 {
    let ratio = elapsed_ratio.clamp(0.0, 1.0);
    if (target_rate - 1.0).abs() < 0.0005 {
        return 1.0;
    }
    // Keep the first/last edge of the audible mix natural.  Only the center of
    // the crossfade does the A-side micro drift, so the ear hears A enter and
    // leave the mix at normal speed rather than as one long transformed region.
    let edge = MUSIC_MIX_OUTGOING_NORMAL_EDGE_RATIO.clamp(0.0, 0.32);
    if ratio <= edge || ratio >= 1.0 - edge {
        return 1.0;
    }
    let inner_ratio = ((ratio - edge) / (1.0 - edge * 2.0)).clamp(0.0, 1.0);
    let bell = (std::f64::consts::PI * inner_ratio).sin().max(0.0);
    let smooth_bell = bell * bell * (3.0 - 2.0 * bell);
    1.0 + (target_rate - 1.0) * smooth_bell
}

fn level_neutral_outgoing_main_gain(
    current_gain: f32,
    preview_gain: f32,
    preview_target_volume: f32,
) -> f32 {
    let base = preview_target_volume
        .clamp(0.0, 1.0)
        .max(current_gain)
        .max(0.0001);
    let phase = (preview_gain / preview_target_volume.max(0.0001)).clamp(0.0, 1.0);

    // v10.12.14 trial: after the cue/tempo/HQ-render layers became smarter, the
    // volume layer should stop behaving like a second director.  Ignore the
    // regular main-deck fade while a preview is active and use one predictable
    // equal-power handoff instead.  This intentionally removes the previous
    // A-hold / B-guard / bus-compensation tug-of-war that could make the mix
    // feel like A disappears, B steps in early, or the centre pulls away.
    (base * crossfade_equal_power_fade_out(phase)).clamp(0.0, base)
}

fn complementary_preview_phase_outgoing_main_gain(
    current_gain: f32,
    preview_gain: f32,
    preview_target_volume: f32,
) -> f32 {
    let base = preview_target_volume
        .clamp(0.0, 1.0)
        .max(current_gain)
        .max(0.0001);
    let phase = (preview_gain / preview_target_volume.max(0.0001)).clamp(0.0, 1.0);

    // v10.12.61: the preview gain envelope is already an equal-power fade-in.
    // Feeding that audible gain back through another smooth/equal-power fade-out
    // makes A fall much faster than the timeline suggests: around the visual
    // bridge midpoint B is already ~0.70, but A could be pushed near ~0.30.
    // Long Reward bridges need the mathematical complement of the actual B gain
    // instead, so A stays present through the middle and only leaves near the
    // real bridge end.
    //
    // v10.12.63: include medium-long Reward bridges too.  A 6.4s bridge still
    // felt like A was cut near the centre, because it remained just below the
    // old 6.5s threshold and therefore used the steeper legacy curve.
    let complement = (1.0 - phase * phase).max(0.0).sqrt();
    (base * complement).clamp(0.0, base)
}

fn preview_equal_power_timeline_progress(preview_gain: f32, preview_target_volume: f32) -> f32 {
    let phase = (preview_gain / preview_target_volume.max(0.0001)).clamp(0.0, 1.0);

    // v10.12.62: keep A-tail hold release decisions on the fade timeline, not
    // on the already-audible equal-power gain.  At the visual bridge midpoint
    // the preview gain is ~0.70, so using raw gain as a release phase makes the
    // A hold begin releasing near the middle.  Convert the equal-power gain back
    // to its underlying fade progress before checking release thresholds.
    (phase.asin() / std::f32::consts::FRAC_PI_2).clamp(0.0, 1.0)
}

fn reward_energy_duck_target(
    current_gain: f32,
    preview_gain: f32,
    preview_target_volume: f32,
    main_raw_rms: f32,
    preview_raw_rms: f32,
    transition_frames: MusicMixFrameCount,
    output_sample_rate: u32,
    outgoing_highlight_end_phase: Option<f32>,
) -> f32 {
    let transition_millis = transition_millis_from_frames(transition_frames, output_sample_rate);
    if transition_millis < MUSIC_ENERGY_DUCK_A_BED_MIN_TRANSITION_MILLIS
        || preview_target_volume <= 0.001
        || preview_raw_rms <= MUSIC_BRIDGE_PREVIEW_AUDIBLE_RMS
        || main_raw_rms <= MUSIC_BRIDGE_WEAK_A_TAIL_RMS
    {
        return 0.0;
    }

    let timeline_progress =
        preview_equal_power_timeline_progress(preview_gain, preview_target_volume);
    if timeline_progress >= 0.98 {
        return 0.0;
    }

    let bed_gain = complementary_preview_phase_outgoing_main_gain(
        current_gain,
        preview_gain,
        preview_target_volume,
    );
    let preview_audible_rms = preview_raw_rms * preview_gain.max(0.0);
    let main_bed_rms = main_raw_rms * bed_gain.max(0.08);
    let audible_ratio = preview_audible_rms / main_bed_rms.max(1.0e-7);

    let energy_ratio = ((audible_ratio - MUSIC_ENERGY_DUCK_A_BED_RATIO_START)
        / (MUSIC_ENERGY_DUCK_A_BED_RATIO_FULL - MUSIC_ENERGY_DUCK_A_BED_RATIO_START).max(0.001))
    .clamp(0.0, 1.0);
    let energy_strength = smooth_audio_fade(energy_ratio);

    let phase_ratio = ((timeline_progress - MUSIC_ENERGY_DUCK_A_BED_START_PHASE)
        / (MUSIC_ENERGY_DUCK_A_BED_FULL_PHASE - MUSIC_ENERGY_DUCK_A_BED_START_PHASE).max(0.001))
    .clamp(0.0, 1.0);
    let phase_strength = smooth_audio_fade(phase_ratio);

    // v10.12.64 Energy-Ducked A Bed:
    // Do not keep raising a fixed A-tail hold floor.  Keep A as the bed while B
    // is still forming, then duck A only after the incoming B side has real
    // audible energy.  The duck itself is also smoothed across callbacks, so the
    // centre breathes instead of making either a hard valley or a permanently
    // full two-song overlay.
    let base_target = (energy_strength * phase_strength).clamp(0.0, 1.0);
    let boundary_window = reward_boundary_cushion_window(
        timeline_progress,
        outgoing_highlight_end_phase,
        transition_millis,
    );
    if boundary_window <= 0.0 {
        return base_target;
    }

    // v10.12.70 Boundary Cushion:
    // If the outgoing highlight edge lands near the active handoff zone, do not
    // let a still-weak B side earn a sudden A duck exactly on that musical role
    // change.  Strong B energy can still take over; weak B must wait.
    let weak_b_guard = 1.0 - energy_strength;
    (base_target * (1.0 - boundary_window * weak_b_guard * 0.72)).clamp(0.0, 1.0)
}

fn reward_energy_aware_outgoing_main_gain(
    current_gain: f32,
    preview_gain: f32,
    preview_target_volume: f32,
    main_raw_rms: f32,
    preview_raw_rms: f32,
    transition_frames: MusicMixFrameCount,
    output_sample_rate: u32,
    energy_duck_amount: f32,
    outgoing_highlight_end_phase: Option<f32>,
) -> f32 {
    let transition_millis = transition_millis_from_frames(transition_frames, output_sample_rate);
    let legacy_base_gain =
        level_neutral_outgoing_main_gain(current_gain, preview_gain, preview_target_volume);
    if transition_millis < MUSIC_LONG_BRIDGE_COMPLEMENTARY_A_FADE_MIN_MILLIS
        || preview_target_volume <= 0.001
    {
        return legacy_base_gain;
    }

    let bed_gain = complementary_preview_phase_outgoing_main_gain(
        current_gain,
        preview_gain,
        preview_target_volume,
    )
    .max(legacy_base_gain);

    if preview_raw_rms <= MUSIC_BRIDGE_PREVIEW_AUDIBLE_RMS
        || main_raw_rms <= MUSIC_BRIDGE_WEAK_A_TAIL_RMS
    {
        return bed_gain;
    }

    let base = preview_target_volume
        .clamp(0.0, 1.0)
        .max(current_gain)
        .max(0.0001);
    let timeline_progress =
        preview_equal_power_timeline_progress(preview_gain, preview_target_volume);
    let preview_audible_rms = preview_raw_rms * preview_gain.max(0.0);
    let main_bed_rms = main_raw_rms * bed_gain.max(0.08);
    let audible_ratio = preview_audible_rms / main_bed_rms.max(1.0e-7);
    let duck_depth = (MUSIC_ENERGY_DUCK_A_BED_MAX_DEPTH * energy_duck_amount).clamp(0.0, 0.24);
    let ducked_gain = bed_gain * (1.0 - duck_depth);

    // v10.12.69 Experimental A-Bed Safety Net:
    // Some Reward bridges still sounded like the outgoing A bed was switched off
    // near the centre even when diagnostics showed no timestretch seam and no A
    // energy cliff.  Treat the centre as a protected handoff zone: if B has not
    // earned enough audible energy yet, A is allowed to stay as a stronger bed.
    // This is intentionally a little "dual-main" in the middle; the final tail
    // blend below still forces a clean handoff instead of letting A hang forever.
    let centre_attack = ((timeline_progress - MUSIC_A_BED_SAFETY_CENTER_START_PHASE)
        / (MUSIC_A_BED_SAFETY_CENTER_FULL_PHASE - MUSIC_A_BED_SAFETY_CENTER_START_PHASE)
            .max(0.001))
    .clamp(0.0, 1.0);
    let centre_release = ((timeline_progress - MUSIC_A_BED_SAFETY_RELEASE_START_PHASE)
        / (MUSIC_A_BED_SAFETY_RELEASE_END_PHASE - MUSIC_A_BED_SAFETY_RELEASE_START_PHASE)
            .max(0.001))
    .clamp(0.0, 1.0);
    let centre_window =
        smooth_audio_fade(centre_attack) * (1.0 - smooth_audio_fade(centre_release));
    let b_strength = ((audible_ratio - MUSIC_ENERGY_DUCK_A_BED_RATIO_START)
        / (MUSIC_ENERGY_DUCK_A_BED_RATIO_FULL - MUSIC_ENERGY_DUCK_A_BED_RATIO_START).max(0.001))
    .clamp(0.0, 1.0);
    let b_strength_curve = smooth_audio_fade(b_strength);
    let tail_release_start_phase =
        reward_long_a_tail_release_start_phase(transition_millis, b_strength_curve);
    let tail_release_ratio = ((timeline_progress - tail_release_start_phase)
        / (1.0 - tail_release_start_phase).max(0.001))
    .clamp(0.0, 1.0);
    let tail_release_curve = crossfade_equal_power_fade_out(tail_release_ratio);
    let bed_floor = base * MUSIC_ENERGY_DUCK_A_BED_MIN_FLOOR;
    let safety_floor_ratio = MUSIC_A_BED_SAFETY_WEAK_FLOOR
        + (MUSIC_A_BED_SAFETY_STRONG_FLOOR - MUSIC_A_BED_SAFETY_WEAK_FLOOR) * b_strength_curve;
    let centre_safety_floor = base * safety_floor_ratio * centre_window;

    let boundary_window = reward_boundary_cushion_window(
        timeline_progress,
        outgoing_highlight_end_phase,
        transition_millis,
    );
    let boundary_floor_ratio = MUSIC_A_BED_BOUNDARY_CUSHION_WEAK_FLOOR
        + (MUSIC_A_BED_BOUNDARY_CUSHION_STRONG_FLOOR - MUSIC_A_BED_BOUNDARY_CUSHION_WEAK_FLOOR)
            * b_strength_curve;
    let boundary_safety_floor = base * boundary_floor_ratio * boundary_window;

    let ducked_bed_gain = ducked_gain
        .max(bed_floor)
        .max(centre_safety_floor)
        .max(boundary_safety_floor)
        .clamp(0.0, base);

    // v10.12.71 Long A Tail Release / No Hard Pull:
    // The old tail safety behaved like a late hard director: once the last
    // 20~25% arrived it blended A back to the steep legacy fade.  If A was still
    // inside the listener's attention window, that could feel like the outgoing
    // song was pulled away.  Treat the tail as a long release instead: A starts
    // fading over roughly 2.5~5s, adjusted by how ready B is, and only reaches
    // silence at the very end.  This keeps the handoff decisive without a wall.
    (ducked_bed_gain * tail_release_curve).clamp(0.0, base)
}

fn transition_millis_from_frames(
    transition_frames: MusicMixFrameCount,
    output_sample_rate: u32,
) -> u64 {
    (MusicMixFrameClock::new(output_sample_rate).seconds_from_frame_count(transition_frames)
        * 1000.0)
        .round()
        .clamp(0.0, u64::MAX as f64) as u64
}

fn reward_long_a_tail_release_start_phase(transition_millis: u64, b_strength_curve: f32) -> f32 {
    let transition_seconds = (transition_millis as f32 / 1000.0).max(0.001);
    let b_strength_curve = b_strength_curve.clamp(0.0, 1.0);
    let weak_b = 1.0 - b_strength_curve;
    let neutral_release_seconds = (transition_seconds * MUSIC_A_TAIL_RELEASE_RATIO).clamp(
        MUSIC_A_TAIL_RELEASE_MIN_SECONDS,
        MUSIC_A_TAIL_RELEASE_MAX_SECONDS,
    );
    let release_seconds = (neutral_release_seconds
        * (1.0 + weak_b * MUSIC_A_TAIL_RELEASE_WEAK_B_BONUS
            - b_strength_curve * MUSIC_A_TAIL_RELEASE_STRONG_B_SHAVE))
        .clamp(
            MUSIC_A_TAIL_RELEASE_MIN_SECONDS.min(transition_seconds * 0.5),
            MUSIC_A_TAIL_RELEASE_MAX_SECONDS.min(transition_seconds * 0.72),
        );
    (1.0 - release_seconds / transition_seconds).clamp(
        MUSIC_A_TAIL_RELEASE_MIN_START_PHASE,
        MUSIC_A_TAIL_RELEASE_MAX_START_PHASE,
    )
}

fn reward_boundary_cushion_window(
    timeline_progress: f32,
    outgoing_highlight_end_phase: Option<f32>,
    transition_millis: u64,
) -> f32 {
    if transition_millis < MUSIC_A_BED_BOUNDARY_CUSHION_MIN_TRANSITION_MILLIS {
        return 0.0;
    }

    let Some(phase) = outgoing_highlight_end_phase.filter(|phase| phase.is_finite()) else {
        return 0.0;
    };
    if !(MUSIC_A_BED_BOUNDARY_CUSHION_MIN_PHASE..=MUSIC_A_BED_BOUNDARY_CUSHION_MAX_PHASE)
        .contains(&phase)
    {
        return 0.0;
    }

    let distance = (timeline_progress - phase).abs();
    if distance >= MUSIC_A_BED_BOUNDARY_CUSHION_WIDTH_PHASE {
        return 0.0;
    }

    smooth_audio_fade(1.0 - distance / MUSIC_A_BED_BOUNDARY_CUSHION_WIDTH_PHASE)
}

#[derive(Clone, Copy, Debug, Default)]
struct AudioQueueFillResult {
    consumed_samples: u64,
    consumed_frames: MusicMixFrameCount,
    written_frames: MusicMixFrameCount,
}

fn fill_interleaved_from_queue(
    queue: &mut VecDeque<f32>,
    output: &mut [f32],
    channels: usize,
) -> AudioQueueFillResult {
    let channels = channels.max(1);
    if queue.is_empty() || output.is_empty() {
        return AudioQueueFillResult::default();
    }

    let output_frames = output.len() / channels;
    let available_frames = queue.len() / channels;
    let frames_to_write = output_frames.min(available_frames);
    if frames_to_write == 0 {
        return AudioQueueFillResult::default();
    }

    for frame in 0..frames_to_write {
        for channel in 0..channels {
            let out_index = frame * channels + channel;
            if let Some(value) = queue.pop_front() {
                output[out_index] = value;
            }
        }
    }

    let written_samples = frames_to_write.saturating_mul(channels).min(output.len());
    if written_samples < output.len() {
        apply_audio_underfill_release(output, written_samples, channels);
    }

    AudioQueueFillResult {
        consumed_samples: written_samples as u64,
        consumed_frames: MusicMixFrameCount::new(frames_to_write as u64),
        written_frames: MusicMixFrameCount::new(frames_to_write as u64),
    }
}

fn apply_audio_underfill_release(output: &mut [f32], written_samples: usize, channels: usize) {
    let channels = channels.max(1);
    let output_frames = output.len() / channels;
    let written_frames = (written_samples / channels).min(output_frames);
    if written_frames == 0 || written_frames >= output_frames {
        return;
    }

    // Keep callback buffers frame-complete.  When a preview/main deck ends in
    // the middle of an output callback, the tail of the block used to remain
    // hard zero: [audio][zero tail].  A following normal block could then sound
    // like a short beep/pop.  Fill only a tiny release from the previous frame;
    // the remaining tail stays silent, but the boundary is no longer vertical.
    let release_frames = output_frames.saturating_sub(written_frames).min(32);
    if release_frames == 0 {
        return;
    }

    for frame_offset in 0..release_frames {
        let ratio = (frame_offset + 1) as f32 / release_frames as f32;
        let gain = crossfade_equal_power_fade_out(ratio);
        let frame = written_frames + frame_offset;
        for channel in 0..channels {
            let last_index = (written_frames - 1) * channels + channel;
            let prev_index = written_frames
                .saturating_sub(2)
                .saturating_mul(channels)
                .saturating_add(channel);
            let out_index = frame * channels + channel;
            if out_index < output.len() && last_index < output.len() {
                let last = output[last_index];
                let previous = output.get(prev_index).copied().unwrap_or(last);
                let slope = (last - previous).clamp(-0.08, 0.08);
                let projected =
                    (last + slope * (frame_offset + 1).min(4) as f32).clamp(-1.25, 1.25);
                output[out_index] = projected * gain;
            }
        }
    }
}

fn fill_interleaved_from_queue_with_rate(
    queue: &mut VecDeque<f32>,
    output: &mut [f32],
    channels: usize,
    rate: f64,
    phase_frames: &mut f64,
) -> AudioQueueFillResult {
    let channels = channels.max(1);
    if queue.is_empty() || output.is_empty() {
        return AudioQueueFillResult::default();
    }
    let output_frames = output.len() / channels;
    if output_frames == 0 {
        return AudioQueueFillResult::default();
    }
    let frame_count = queue.len() / channels;
    if frame_count == 0 {
        return AudioQueueFillResult::default();
    }
    let rate = rate.clamp(0.965, 1.035);
    let mut phase = (*phase_frames).max(0.0);

    let mut written_frames = 0_usize;
    for frame_index in 0..output_frames {
        let source_frame = phase.floor().max(0.0) as usize;
        let frac = (phase - source_frame as f64).clamp(0.0, 1.0) as f32;
        for channel in 0..channels {
            let out_index = frame_index * channels + channel;
            let current = source_frame
                .checked_mul(channels)
                .and_then(|base| base.checked_add(channel))
                .and_then(|index| queue.get(index))
                .copied()
                .unwrap_or(0.0);
            let next = (source_frame + 1)
                .checked_mul(channels)
                .and_then(|base| base.checked_add(channel))
                .and_then(|index| queue.get(index))
                .copied()
                .unwrap_or(current);
            output[out_index] = current + (next - current) * frac;
        }
        written_frames = frame_index.saturating_add(1);
        phase += rate;
        if phase >= frame_count.saturating_sub(1) as f64 {
            break;
        }
    }

    let discard_frames = phase.floor().max(0.0) as usize;
    let discard_samples = discard_frames.saturating_mul(channels).min(queue.len());
    for _ in 0..discard_samples {
        let _ = queue.pop_front();
    }
    *phase_frames = (phase - discard_frames as f64).clamp(0.0, 1.0);

    let written_samples = written_frames.saturating_mul(channels).min(output.len());
    if written_samples < output.len() {
        apply_audio_underfill_release(output, written_samples, channels);
    }

    AudioQueueFillResult {
        consumed_samples: discard_samples as u64,
        consumed_frames: MusicMixFrameCount::new(discard_frames as u64),
        written_frames: MusicMixFrameCount::new(written_frames as u64),
    }
}

fn rms_audio_samples_for_frames(
    samples: &[f32],
    valid_frames: MusicMixFrameCount,
    channels: usize,
) -> f32 {
    let valid_samples = (valid_frames.get() as usize)
        .saturating_mul(channels.max(1))
        .min(samples.len());
    if valid_samples == 0 {
        return 0.0;
    }
    let sum = samples
        .iter()
        .take(valid_samples)
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum::<f64>();
    (sum / valid_samples as f64).sqrt() as f32
}

fn rms_audio_sample_pairs_for_frames(
    a: &[f32],
    b: &[f32],
    valid_frames: MusicMixFrameCount,
    channels: usize,
) -> f32 {
    let valid_samples = (valid_frames.get() as usize).saturating_mul(channels.max(1));
    let len = a.len().min(b.len()).min(valid_samples);
    if len == 0 {
        return 0.0;
    }
    let sum = a
        .iter()
        .zip(b.iter())
        .take(len)
        .map(|(left, right)| {
            let sample = f64::from(*left + *right);
            sample * sample
        })
        .sum::<f64>();
    (sum / len.max(1) as f64).sqrt() as f32
}

fn preview_relative_level_guard(
    main_raw_rms: f32,
    preview_raw_rms: f32,
    main_gain: f32,
    preview_gain: f32,
) -> f32 {
    let main_loudness = main_raw_rms * main_gain;
    let preview_loudness = preview_raw_rms * preview_gain;
    if preview_gain < 0.075 || main_loudness <= 1.0e-5 || preview_loudness <= 1.0e-5 {
        return 1.0;
    }

    // This is a relative vocal-presence guard, not a loudness normalizer.  When A
    // is still audible, a louder B vocal can feel like a bump even if the total
    // mix bus is level.  Attenuate only the preview side, and release naturally as
    // A fades out so B can still become the main song.
    let a_presence = (main_gain / 0.55).clamp(0.0, 1.0);
    if a_presence <= 0.05 {
        return 1.0;
    }
    let centre_weight = (main_gain * preview_gain * 4.0).clamp(0.0, 1.0) * a_presence;
    if centre_weight <= 0.001 {
        return 1.0;
    }

    let allowed_preview = main_loudness * (1.06 + 0.24 * (1.0 - a_presence));
    if preview_loudness <= allowed_preview {
        return 1.0;
    }

    let raw = (allowed_preview / preview_loudness).clamp(0.90, 1.0);
    1.0 + (raw - 1.0) * centre_weight * 0.58
}

fn crossfade_floor_support_compensation(
    main_raw_rms: f32,
    preview_raw_rms: f32,
    main_gain: f32,
    preview_gain: f32,
    target_volume: f32,
    preview_level_guard: f32,
) -> f32 {
    let main_loudness = main_raw_rms * main_gain;
    let preview_loudness = preview_raw_rms * preview_gain * preview_level_guard;
    if preview_gain < 0.045
        || main_gain < 0.045
        || main_loudness <= 1.0e-5
        || preview_loudness <= 1.0e-5
    {
        return 1.0;
    }

    let target_volume = target_volume.clamp(0.0, 1.0).max(0.0001);
    let phase = (preview_gain / target_volume).clamp(0.0, 1.0);
    let centre_weight = (phase * (1.0 - phase) * 4.0).clamp(0.0, 1.0);
    if centre_weight <= 0.001 {
        return 1.0;
    }

    let reference = (main_raw_rms.max(preview_raw_rms) * target_volume)
        .max(main_loudness.max(preview_loudness));
    let current = (main_loudness.powi(2) + preview_loudness.powi(2)).sqrt();
    if reference <= 1.0e-5 || current >= reference * 0.965 {
        return 1.0;
    }

    // Only fill a real overlap valley.  If the preview guard is actively
    // suppressing a forward vocal, reduce support so the old "B suddenly gets
    // loud" problem does not come back.
    let guard_release = ((preview_level_guard - 0.86) / 0.14).clamp(0.0, 1.0);
    let needed = ((reference * 0.975) / current).clamp(1.0, 1.075);
    1.0 + (needed - 1.0) * centre_weight * guard_release * 0.82
}

fn crossfade_loudness_compensation(
    main_raw_rms: f32,
    preview_raw_rms: f32,
    main_gain: f32,
    preview_gain: f32,
    _target_volume: f32,
) -> f32 {
    let main_loudness = main_raw_rms * main_gain;
    let preview_loudness = preview_raw_rms * preview_gain;
    let reference = main_loudness.max(preview_loudness);
    let current = (main_loudness.powi(2) + preview_loudness.powi(2)).sqrt();

    if preview_gain < 0.055 || reference <= 1.0e-5 || current <= 1.0e-5 {
        return 1.0;
    }

    // v10.12.9: mix-bus levelling is attenuation-only.  The old centre support
    // could gently boost the middle of the crossfade; on vocal-heavy songs that
    // can be perceived as the singer suddenly stepping forward.  Keep the magic
    // cue, but never add loudness inside the overlap.
    let centre_weight = (main_gain * preview_gain * 4.0).clamp(0.0, 1.0);
    let overlap_ceiling = reference * (1.20 - 0.03 * centre_weight);
    if current <= overlap_ceiling {
        return 1.0;
    }

    let raw = (overlap_ceiling / current).clamp(0.92, 1.0);
    1.0 + (raw - 1.0) * centre_weight * 0.75
}

fn smooth_mix_bus_compensation(state: &AtomicU32, target: f32) -> f32 {
    let target = target.clamp(1.0, 1.075);
    let current = f32::from_bits(state.load(Ordering::Relaxed)).clamp(1.0, 1.075);
    let delta = target - current;
    // Support can rise slowly enough to avoid a vocal pop, then release a little
    // faster so the next non-overlap block returns to neutral.
    let step = if delta > 0.0 { 0.055 } else { 0.12 };
    let smoothed = (current + delta * step).clamp(1.0, 1.075);
    state.store(smoothed.to_bits(), Ordering::Relaxed);
    smoothed
}

fn smooth_mix_level_guard(state: &AtomicU32, target: f32) -> f32 {
    let target = target.clamp(0.86, 1.0);
    let current = f32::from_bits(state.load(Ordering::Relaxed)).clamp(0.86, 1.0);
    let delta = target - current;
    // Smooth the guard itself so block-to-block RMS changes do not create a
    // centre "pull away → come back" valley.  Attenuation may react a little
    // faster than release, but both remain gentle.
    let step = if delta < 0.0 { 0.11 } else { 0.085 };
    let smoothed = (current + delta * step).clamp(0.86, 1.0);
    state.store(smoothed.to_bits(), Ordering::Relaxed);
    smoothed
}

fn smooth_reward_energy_duck(state: &AtomicU32, target: f32) -> f32 {
    let target = target.clamp(0.0, 1.0);
    let current = f32::from_bits(state.load(Ordering::Relaxed)).clamp(0.0, 1.0);
    let delta = target - current;
    // Duck gain-reduction should not snap in or snap out.  Let B earn the duck
    // over a few callbacks, then release A even more gently if B energy dips.
    let step = if delta > 0.0 { 0.075 } else { 0.045 };
    let smoothed = (current + delta * step).clamp(0.0, 1.0);
    state.store(smoothed.to_bits(), Ordering::Relaxed);
    smoothed
}

fn soft_limit_audio_sample(value: f32) -> f32 {
    const KNEE: f32 = 0.92;
    const KNEE_RANGE: f32 = 1.0 - KNEE;

    let value = value.clamp(-4.0, 4.0);
    let magnitude = value.abs();
    if magnitude <= KNEE {
        value
    } else {
        // This rational knee is continuous and has slope 1 at KNEE. The old
        // >1.0 branch restarted near 0.5, so an overlap peak could sound like a
        // sudden volume collapse exactly when both songs became energetic.
        let over = magnitude - KNEE;
        let limited = KNEE + KNEE_RANGE * over / (over + KNEE_RANGE);
        value.signum() * limited
    }
}

fn append_playback_pcm_reservoir(
    shared: &SharedPlaybackState,
    item_id: u64,
    session_id: u64,
    start_frame: MusicMixSourceFrame,
    samples: &[f32],
    channels: usize,
    sample_rate: u32,
) {
    if samples.is_empty() {
        return;
    }
    if let Ok(mut reservoir) = shared.pcm_reservoir.lock() {
        reset_playback_pcm_reservoir_owner_if_needed(
            shared,
            &mut reservoir,
            item_id,
            session_id,
            sample_rate,
            channels,
            start_frame,
        );
        reservoir.append_interleaved(start_frame, samples, channels, sample_rate);
    }
}

fn reset_playback_pcm_reservoir(
    shared: &SharedPlaybackState,
    item_id: u64,
    session_id: u64,
    sample_rate: u32,
    channels: usize,
    start_frame: MusicMixSourceFrame,
) {
    if let Ok(mut reservoir) = shared.pcm_reservoir.lock() {
        set_playback_pcm_reservoir_owner(shared, item_id, session_id);
        reservoir.clear_from(sample_rate, channels, start_frame);
    }
}

fn reset_playback_pcm_reservoir_owner_if_needed(
    shared: &SharedPlaybackState,
    reservoir: &mut MusicPcmReservoir,
    item_id: u64,
    session_id: u64,
    sample_rate: u32,
    channels: usize,
    start_frame: MusicMixSourceFrame,
) {
    let current_item_id = shared.pcm_reservoir_item_id.load(Ordering::Relaxed);
    let current_session_id = shared.pcm_reservoir_session_id.load(Ordering::Relaxed);
    if current_item_id == item_id && current_session_id == session_id {
        return;
    }

    // The audio output stream can survive across A->[mix]->B promotions, but
    // source frames are track-local.  Reset ownership before comparing frame
    // numbers so a lower B frame is not discarded as an "older" A frame.
    eprintln!(
        "[music-stage-pcm] reservoir owner reset old={current_item_id}/{current_session_id} new={item_id}/{session_id} start_frame={}",
        start_frame.get()
    );
    set_playback_pcm_reservoir_owner(shared, item_id, session_id);
    reservoir.clear_from(sample_rate, channels, start_frame);
}

fn set_playback_pcm_reservoir_owner(shared: &SharedPlaybackState, item_id: u64, session_id: u64) {
    shared
        .pcm_reservoir_item_id
        .store(item_id, Ordering::Relaxed);
    shared
        .pcm_reservoir_session_id
        .store(session_id, Ordering::Relaxed);
}

fn playback_pcm_reservoir_status(
    coverage: Option<MusicPcmReservoirCoverage>,
    requested_start: MusicMixSourceFrame,
    requested_frames: MusicMixFrameCount,
) -> String {
    let requested_end = requested_start.get().saturating_add(requested_frames.get());
    match coverage {
        Some(coverage) => format!(
            "req={}..{} cover={}..{}",
            requested_start.get(),
            requested_end,
            coverage.start_frame.get(),
            coverage.end_frame.get()
        ),
        None => format!(
            "req={}..{} cover=empty",
            requested_start.get(),
            requested_end
        ),
    }
}

fn playback_pcm_reservoir_max_samples(sample_rate: u32, channels: usize) -> usize {
    (sample_rate.max(1) as usize)
        .saturating_mul(channels.max(1))
        .saturating_mul(MUSIC_PCM_RESERVOIR_SECONDS as usize)
}

fn convert_interleaved_channels(
    samples: &[f32],
    source_channels: usize,
    output_channels: usize,
) -> Vec<f32> {
    let source_channels = source_channels.max(1);
    let output_channels = output_channels.max(1);
    let mut converted =
        Vec::with_capacity((samples.len() / source_channels).saturating_mul(output_channels));
    for frame in samples.chunks_exact(source_channels) {
        if output_channels == 1 {
            converted.push(frame.iter().copied().sum::<f32>() / source_channels as f32);
            continue;
        }
        for channel in 0..output_channels {
            let sample = if source_channels == 1 && channel < 2 {
                frame[0]
            } else {
                frame.get(channel).copied().unwrap_or(0.0)
            };
            converted.push(sample);
        }
    }
    converted
}

fn queue_samples(
    shared: &SharedPlaybackState,
    item_id: u64,
    session_id: u64,
    samples: &[f32],
    source_channels: usize,
    output_channels: usize,
    source_sample_rate: u32,
    output_sample_rate: u32,
) {
    if shared.discard_decoder_samples.load(Ordering::Relaxed) {
        return;
    }
    let max_buffered_samples = output_sample_rate as usize * output_channels * 10;
    while !shared.stop_requested.load(Ordering::Relaxed)
        && shared
            .buffer
            .lock()
            .map(|buffer| buffer.len() > max_buffered_samples)
            .unwrap_or(false)
    {
        thread::sleep(Duration::from_millis(20));
    }

    let source_channels = source_channels.max(1);
    let output_channels = output_channels.max(1);
    let converted = convert_interleaved_channels(samples, source_channels, output_channels);
    let source_frames = converted.len() / output_channels;
    if source_frames == 0 {
        return;
    }

    if let Ok(mut buffer) = shared.buffer.lock() {
        if shared.discard_decoder_samples.load(Ordering::Relaxed) {
            return;
        }
        let start_frame = MusicMixSourceFrame::new(
            shared
                .decoder_queued_source_frame
                .fetch_add(source_frames as u64, Ordering::Relaxed),
        );
        append_playback_pcm_reservoir(
            shared,
            item_id,
            session_id,
            start_frame,
            &converted,
            output_channels,
            source_sample_rate,
        );
        if source_sample_rate == output_sample_rate {
            buffer.extend(converted);
        } else {
            buffer.extend(resample_interleaved_frames(
                &converted,
                source_sample_rate,
                output_sample_rate,
                output_channels,
                None,
            ));
        }
    }
}

fn wait_for_buffer_drain(shared: &SharedPlaybackState) {
    while !shared.stop_requested.load(Ordering::Relaxed) && output_has_buffered_audio(shared) {
        thread::sleep(Duration::from_millis(40));
    }
}

fn output_has_buffered_audio(shared: &SharedPlaybackState) -> bool {
    let main_has_audio = shared
        .buffer
        .lock()
        .map(|buffer| !buffer.is_empty())
        .unwrap_or(false);
    if main_has_audio {
        return true;
    }
    shared
        .crossfade_decks
        .lock()
        .map(|decks| {
            decks
                .main
                .as_ref()
                .is_some_and(|deck| !deck.buffer.is_empty())
                || decks
                    .next
                    .as_ref()
                    .is_some_and(|deck| !deck.buffer.is_empty())
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod mix_execution_tests {
    use super::*;

    #[test]
    fn device_channel_conversion_keeps_front_stereo_and_silences_extra_outputs() {
        let converted = convert_interleaved_channels(&[0.25, -0.5, 0.75, -1.0], 2, 4);

        assert_eq!(converted, vec![0.25, -0.5, 0.0, 0.0, 0.75, -1.0, 0.0, 0.0]);
    }

    #[test]
    fn device_channel_conversion_downmixes_to_mono() {
        let converted = convert_interleaved_channels(&[0.5, -0.25, 1.0, 0.0], 2, 1);

        assert_eq!(converted, vec![0.125, 0.5]);
    }

    #[test]
    fn device_rate_fallback_preserves_playback_duration() {
        let source = vec![0.25_f32; 44_100 * 2];
        let output = resample_interleaved_frames(&source, 44_100, 48_000, 2, None);

        assert_eq!(output.len(), 48_000 * 2);
    }

    fn test_crossfade_deck(
        mode: CrossfadeDeckMode,
        samples: Vec<f32>,
        transition_frames: MusicMixFrameCount,
    ) -> CrossfadePreviewDeck {
        CrossfadePreviewDeck {
            mode,
            buffer: samples.into_iter().collect(),
            transition_output_frames: transition_frames,
            target_volume: 1.0,
            track_start_source_frame: MusicMixSourceFrame::new(0),
            source_sample_rate: 44_100,
            transition_source_frames: transition_frames,
            prepared_mix_resume_source_frame: (mode == CrossfadeDeckMode::PreparedMix)
                .then_some(MusicMixSourceFrame::new(transition_frames.get())),
            prepared_mix_start_seconds: (mode == CrossfadeDeckMode::PreparedMix).then_some(0.0),
            prepared_mix_alignment_applied: mode != CrossfadeDeckMode::PreparedMix,
            track_duration_seconds: Some(120.0),
            output_frames_consumed: MusicMixFrameCount::ZERO,
            rendered_frames_consumed: MusicMixFrameCount::ZERO,
            release_started_output_frame: None,
            release_duration_frames: MusicMixFrameCount::ZERO,
            release_from_gain: 0.0,
            outgoing_transition_rate: 1.0,
            outgoing_transition_phase_frames: 0.0,
            outgoing_transition_started_output_frame: None,
            outgoing_transition_duration_frames: MusicMixFrameCount::ZERO,
            outgoing_highlight_end_phase: None,
        }
    }

    #[test]
    fn soft_limiter_has_no_full_scale_volume_collapse() {
        let at_full_scale = soft_limit_audio_sample(1.0);
        let just_over = soft_limit_audio_sample(1.0001);

        assert!(at_full_scale > 0.95);
        assert!(just_over >= at_full_scale);
        assert!((just_over - at_full_scale).abs() < 0.001);
        assert_eq!(
            soft_limit_audio_sample(-1.0001),
            -soft_limit_audio_sample(1.0001)
        );
    }

    #[test]
    fn soft_limiter_is_continuous_and_monotonic_through_knee() {
        let values = [0.9199, 0.92, 0.9201, 1.0, 1.5, 4.0];
        let limited = values.map(soft_limit_audio_sample);

        assert!((limited[1] - limited[0]).abs() < 0.001);
        assert!((limited[2] - limited[1]).abs() < 0.001);
        assert!(limited.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(limited.last().copied().unwrap_or_default() < 1.0);
    }

    #[test]
    fn rms_ignores_callback_padding_after_valid_frames() {
        let samples = [0.5_f32, -0.5, 0.5, -0.5, 0.0, 0.0, 0.0, 0.0];
        let rms = rms_audio_samples_for_frames(&samples, MusicMixFrameCount::new(2), 2);

        assert!((rms - 0.5).abs() < 0.000_001);
    }

    #[test]
    fn prepared_mix_buffer_starts_as_current_and_continues_as_next() {
        let current = vec![0.40_f32; 4 * 2];
        let mut next = vec![0.0_f32; 6 * 2];
        for frame in 0..6 {
            for channel in 0..2 {
                next[frame * 2 + channel] = 0.10 + frame as f32 * 0.05;
            }
        }

        let mixed = build_prepared_mix_samples(&current, &next, 2, MusicMixFrameCount::new(4));

        assert_eq!(mixed.len(), next.len());
        assert!((mixed[0] - current[0]).abs() < 0.000_001);
        assert!((mixed[4 * 2] - next[4 * 2]).abs() < 0.000_001);
        assert!((mixed[5 * 2 + 1] - next[5 * 2 + 1]).abs() < 0.000_001);
    }

    #[test]
    fn prepared_mix_guard_keeps_head_raw_a_and_tail_raw_b() {
        let mut current = Vec::new();
        let mut next = Vec::new();
        for frame in 0..12 {
            let current_left = 0.40 + frame as f32 * 0.01;
            let next_left = 0.10 + frame as f32 * 0.02;
            current.push(current_left);
            current.push(current_left + 0.001);
            next.push(next_left);
            next.push(next_left + 0.001);
        }

        let mixed = build_prepared_mix_samples_with_guards(
            &current,
            &next,
            2,
            MusicMixFrameCount::new(12),
            MusicMixFrameCount::new(3),
        );

        assert_eq!(mixed.len(), next.len());
        assert!((mixed[0] - current[0]).abs() < 0.000_001);
        assert!((mixed[2 * 2] - current[2 * 2]).abs() < 0.000_001);
        assert!((mixed[9 * 2] - next[9 * 2]).abs() < 0.000_001);
        assert!((mixed[11 * 2 + 1] - next[11 * 2 + 1]).abs() < 0.000_001);
    }

    #[test]
    fn prepared_mix_guard_expands_for_late_handoff_but_stays_bounded() {
        let base =
            prepared_mix_edge_guard_frames_for_late(1_000, MusicMixFrameCount::new(4_000), 0.0);
        let late =
            prepared_mix_edge_guard_frames_for_late(1_000, MusicMixFrameCount::new(4_000), 0.750);
        let capped =
            prepared_mix_edge_guard_frames_for_late(1_000, MusicMixFrameCount::new(4_000), 10.0);

        assert_eq!(base, MusicMixFrameCount::new(420));
        assert_eq!(
            late,
            MusicMixFrameCount::new(750 + MUSIC_PREPARED_MIX_LATE_GUARD_PAD_MILLIS)
        );
        assert_eq!(capped, MusicMixFrameCount::new(1_000));
    }

    #[test]
    fn prepared_mix_keeps_b_when_current_tail_is_short() {
        let current = vec![0.35_f32; 2];
        let next = vec![0.12_f32; 5 * 2];

        let mixed = build_prepared_mix_samples(&current, &next, 2, MusicMixFrameCount::new(4));

        assert_eq!(mixed.len(), next.len());
        assert!((mixed[0] - current[0]).abs() < 0.000_001);
        assert!((mixed[4 * 2] - next[4 * 2]).abs() < 0.000_001);
    }

    #[test]
    fn prepared_mix_handoff_replaces_live_sources_with_segment_queue() {
        let shared = Arc::new(SharedPlaybackState::new(0.8, Some(180.0), 11, 22));
        shared.sample_rate.store(44_100, Ordering::Relaxed);
        shared.channels.store(2, Ordering::Relaxed);
        {
            let mut buffer = shared.buffer.lock().unwrap();
            buffer.extend([0.7_f32; 8]);
        }
        {
            let mut decks = shared.crossfade_decks.lock().unwrap();
            decks.main = Some(test_crossfade_deck(
                CrossfadeDeckMode::RealtimePreview,
                vec![0.6_f32; 8],
                MusicMixFrameCount::new(4),
            ));
        }
        let control = MusicPlaybackControl {
            item_id: 11,
            session_id: 22,
            shared: Arc::clone(&shared),
            cache_state: Arc::new(CacheTransferState::default()),
        };

        let started = control.start_prepared_mix_handoff(
            vec![0.25_f32; 10],
            None,
            MusicMixFrameCount::new(5),
            0.8,
            MusicMixSourceFrame::new(1_000),
            44_100,
            Some(180.0),
            Some(0.0),
        );

        assert!(started.is_some());
        assert!(shared.discard_decoder_samples.load(Ordering::Relaxed));
        assert!(shared.buffer.lock().unwrap().is_empty());
        let decks = shared.crossfade_decks.lock().unwrap();
        assert!(decks.main.is_none());
        let next = decks.next.as_ref().expect("prepared mix deck");
        assert_eq!(next.mode, CrossfadeDeckMode::PreparedMix);
        assert_eq!(next.buffer.len(), 10);
    }

    #[test]
    fn prepared_mix_handoff_rebuilds_a_from_live_pcm_window() {
        let shared = Arc::new(SharedPlaybackState::new(1.0, Some(180.0), 11, 22));
        shared.sample_rate.store(1_000, Ordering::Relaxed);
        shared.channels.store(2, Ordering::Relaxed);
        {
            let mut buffer = shared.buffer.lock().unwrap();
            for frame in 0..200 {
                let left = 0.90 - frame as f32 * 0.001;
                buffer.push_back(left);
                buffer.push_back(left + 0.01);
            }
        }
        let control = MusicPlaybackControl {
            item_id: 11,
            session_id: 22,
            shared: Arc::clone(&shared),
            cache_state: Arc::new(CacheTransferState::default()),
        };
        let stale_worker_mix = vec![0.10_f32; 200 * 2];
        let prepared_b = vec![0.0_f32; 200 * 2];

        let started = control.start_prepared_mix_handoff(
            stale_worker_mix,
            Some(prepared_b),
            MusicMixFrameCount::new(200),
            1.0,
            MusicMixSourceFrame::new(1_000),
            1_000,
            Some(180.0),
            Some(0.0),
        );

        assert!(started.is_some());
        assert!(shared.buffer.lock().unwrap().is_empty());
        {
            let decks = shared.crossfade_decks.lock().unwrap();
            let next = decks.next.as_ref().expect("prepared mix deck");
            assert!(!next.prepared_mix_alignment_applied);
        }
        let mut output = vec![0.0_f32; 8];
        write_output_samples(&mut output, &shared, |sample| sample);

        assert!((output[0] - 0.90).abs() < 0.000_001);
        assert!((output[1] - 0.91).abs() < 0.000_001);
        assert_ne!(output[0], 0.10);
    }

    #[test]
    fn prepared_mix_handoff_prefers_frame_addressed_pcm_reservoir() {
        let shared = Arc::new(SharedPlaybackState::new(1.0, Some(180.0), 11, 22));
        shared.sample_rate.store(1_000, Ordering::Relaxed);
        shared.channels.store(2, Ordering::Relaxed);
        shared.samples_played.store(50 * 2, Ordering::Relaxed);
        let mut reservoir_samples = Vec::new();
        for frame in 0..120 {
            let left = 0.70 + frame as f32 * 0.001;
            reservoir_samples.push(left);
            reservoir_samples.push(left + 0.01);
        }
        append_playback_pcm_reservoir(
            &shared,
            11,
            22,
            MusicMixSourceFrame::new(50),
            &reservoir_samples,
            2,
            1_000,
        );
        let control = MusicPlaybackControl {
            item_id: 11,
            session_id: 22,
            shared: Arc::clone(&shared),
            cache_state: Arc::new(CacheTransferState::default()),
        };
        let stale_worker_mix = vec![0.10_f32; 100 * 2];
        let prepared_b = vec![0.0_f32; 100 * 2];

        let started = control.start_prepared_mix_handoff(
            stale_worker_mix,
            Some(prepared_b),
            MusicMixFrameCount::new(100),
            1.0,
            MusicMixSourceFrame::new(1_000),
            1_000,
            Some(180.0),
            Some(0.050),
        );

        assert!(started.is_some());
        let mut output = vec![0.0_f32; 8];
        write_output_samples(&mut output, &shared, |sample| sample);

        assert!((output[0] - 0.70).abs() < 0.000_001);
        assert!((output[1] - 0.71).abs() < 0.000_001);
        assert_ne!(output[0], 0.10);
    }

    #[test]
    fn pcm_reservoir_owner_reset_keeps_promoted_track_frames_local() {
        let shared = Arc::new(SharedPlaybackState::new(1.0, Some(180.0), 11, 22));
        shared.sample_rate.store(1_000, Ordering::Relaxed);
        shared.channels.store(2, Ordering::Relaxed);

        let old_track_samples = vec![0.10_f32; 120 * 2];
        append_playback_pcm_reservoir(
            &shared,
            11,
            22,
            MusicMixSourceFrame::new(10_000),
            &old_track_samples,
            2,
            1_000,
        );

        let mut promoted_track_samples = Vec::new();
        for frame in 0..120 {
            let left = 0.80 + frame as f32 * 0.001;
            promoted_track_samples.push(left);
            promoted_track_samples.push(left + 0.01);
        }
        append_playback_pcm_reservoir(
            &shared,
            12,
            23,
            MusicMixSourceFrame::new(50),
            &promoted_track_samples,
            2,
            1_000,
        );

        let (snapshot, status) = snapshot_prepared_mix_pcm_reservoir_window(
            &shared,
            12,
            23,
            Some(0.050),
            1_000,
            2,
            MusicMixFrameCount::new(100),
        );
        let snapshot = snapshot.expect("promoted track range should be frame-local");

        assert!(status.starts_with("hit:"), "{status}");
        assert_eq!(snapshot.live_a_frames, MusicMixFrameCount::new(100));
        assert!((snapshot.samples[0] - 0.80).abs() < 0.000_001);
        assert!((snapshot.samples[1] - 0.81).abs() < 0.000_001);

        let (stale_snapshot, stale_status) = snapshot_prepared_mix_pcm_reservoir_window(
            &shared,
            11,
            22,
            Some(10.0),
            1_000,
            2,
            MusicMixFrameCount::new(50),
        );

        assert!(stale_snapshot.is_none());
        assert!(stale_status.starts_with("owner-miss:"), "{stale_status}");
    }

    #[test]
    fn prepared_mix_handoff_rebuilds_a_head_when_full_live_window_is_short() {
        let shared = Arc::new(SharedPlaybackState::new(1.0, Some(180.0), 11, 22));
        shared.sample_rate.store(1_000, Ordering::Relaxed);
        shared.channels.store(2, Ordering::Relaxed);
        {
            let mut buffer = shared.buffer.lock().unwrap();
            for frame in 0..200 {
                let left = 0.90 - frame as f32 * 0.001;
                buffer.push_back(left);
                buffer.push_back(left + 0.01);
            }
        }
        let control = MusicPlaybackControl {
            item_id: 11,
            session_id: 22,
            shared: Arc::clone(&shared),
            cache_state: Arc::new(CacheTransferState::default()),
        };
        let stale_worker_mix = vec![0.10_f32; 400 * 2];
        let prepared_b = vec![0.0_f32; 400 * 2];

        let started = control.start_prepared_mix_handoff(
            stale_worker_mix,
            Some(prepared_b),
            MusicMixFrameCount::new(400),
            1.0,
            MusicMixSourceFrame::new(1_000),
            1_000,
            Some(180.0),
            Some(0.0),
        );

        assert!(started.is_some());
        assert!(shared.buffer.lock().unwrap().is_empty());
        let decks = shared.crossfade_decks.lock().unwrap();
        let next = decks.next.as_ref().expect("prepared mix deck");
        assert!(!next.prepared_mix_alignment_applied);
        assert_eq!(next.buffer.len(), 400 * 2);
        assert!((next.buffer[0] - 0.90).abs() < 0.000_001);
        assert!((next.buffer[1] - 0.91).abs() < 0.000_001);
        assert_ne!(next.buffer[0], 0.10);
        assert!((next.buffer[250 * 2] - 0.10).abs() < 0.000_001);
    }

    #[test]
    fn prepared_mix_live_rebuild_alignment_drops_frames_played_after_snapshot() {
        let shared = Arc::new(SharedPlaybackState::new(1.0, Some(180.0), 11, 22));
        shared.sample_rate.store(1_000, Ordering::Relaxed);
        shared.channels.store(2, Ordering::Relaxed);
        {
            let mut buffer = shared.buffer.lock().unwrap();
            for frame in 0..200 {
                let left = 0.90 - frame as f32 * 0.001;
                buffer.push_back(left);
                buffer.push_back(left + 0.01);
            }
        }
        let control = MusicPlaybackControl {
            item_id: 11,
            session_id: 22,
            shared: Arc::clone(&shared),
            cache_state: Arc::new(CacheTransferState::default()),
        };
        let stale_worker_mix = vec![0.10_f32; 200 * 2];
        let prepared_b = vec![0.0_f32; 200 * 2];

        let started = control.start_prepared_mix_handoff(
            stale_worker_mix,
            Some(prepared_b),
            MusicMixFrameCount::new(200),
            1.0,
            MusicMixSourceFrame::new(1_000),
            1_000,
            Some(180.0),
            Some(0.0),
        );

        assert!(started.is_some());
        shared.samples_played.store(24 * 2, Ordering::Relaxed);
        let mut output = vec![0.0_f32; 8];
        write_output_samples(&mut output, &shared, |sample| sample);

        assert!((output[0] - 0.876).abs() < 0.000_001);
        assert!((output[1] - 0.886).abs() < 0.000_001);
        let decks = shared.crossfade_decks.lock().unwrap();
        let next = decks.next.as_ref().expect("prepared deck remains");
        assert!(next.prepared_mix_alignment_applied);
        assert_eq!(next.transition_output_frames, MusicMixFrameCount::new(176));
    }

    #[test]
    fn prepared_mix_callback_outputs_prepared_pcm_without_live_a_seam() {
        let shared = SharedPlaybackState::new(1.0, Some(180.0), 11, 22);
        shared.sample_rate.store(44_100, Ordering::Relaxed);
        shared.channels.store(2, Ordering::Relaxed);
        {
            let mut buffer = shared.buffer.lock().unwrap();
            buffer.extend([1.0_f32; 8]);
        }
        {
            let mut decks = shared.crossfade_decks.lock().unwrap();
            decks.next = Some(test_crossfade_deck(
                CrossfadeDeckMode::PreparedMix,
                vec![0.25_f32; 8],
                MusicMixFrameCount::new(4),
            ));
        }
        let mut output = vec![0.0_f32; 8];

        write_output_samples(&mut output, &shared, |sample| sample);

        assert!(
            output
                .iter()
                .all(|sample| (*sample - 0.25).abs() < 0.000_001)
        );
    }

    #[test]
    fn prepared_mix_callback_aligns_first_frame_to_current_playback_cursor() {
        let shared = SharedPlaybackState::new(1.0, Some(180.0), 11, 22);
        shared.sample_rate.store(44_100, Ordering::Relaxed);
        shared.channels.store(2, Ordering::Relaxed);
        shared.samples_played.store(2 * 2, Ordering::Relaxed);
        let mut prepared = Vec::new();
        for frame in 0..128 {
            prepared.push(frame as f32 * 0.01);
            prepared.push(frame as f32 * 0.01);
        }
        {
            let mut decks = shared.crossfade_decks.lock().unwrap();
            decks.next = Some(test_crossfade_deck(
                CrossfadeDeckMode::PreparedMix,
                prepared,
                MusicMixFrameCount::new(128),
            ));
        }
        let mut output = vec![0.0_f32; 8];

        write_output_samples(&mut output, &shared, |sample| sample);

        let expected = [0.02_f32, 0.02, 0.03, 0.03, 0.04, 0.04, 0.05, 0.05];
        for (actual, expected) in output.iter().zip(expected) {
            assert!((*actual - expected).abs() < 0.000_001);
        }
        let decks = shared.crossfade_decks.lock().unwrap();
        let next = decks.next.as_ref().expect("prepared deck remains");
        assert_eq!(next.transition_output_frames, MusicMixFrameCount::new(126));
    }
}
