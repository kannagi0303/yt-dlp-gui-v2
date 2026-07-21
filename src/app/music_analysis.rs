use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use symphonia::core::audio::sample::Sample;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia_adapter_libopus::OpusDecoder;

use crate::infrastructure::yaml_store::{read_yaml_file, write_yaml_file};

const MUSIC_ANALYSIS_SCHEMA_VERSION: u32 = 1;
const MUSIC_ANALYSIS_ANALYZER_VERSION: u32 = 21;
const ANALYSIS_FRAME_SIZE: usize = 2048;
const MAX_ENERGY_CURVE_POINTS: usize = 256;
const MAX_SPECTRUM_CURVE_POINTS: usize = 1536;
const MAX_HIGHLIGHT_CANDIDATES: usize = 4;
const SECTION_CURVE_HOP_SECONDS: f64 = 1.0;
const MIN_HIGHLIGHT_SEGMENT_SECONDS: f64 = 8.0;
const MAX_BOUNDARY_CANDIDATES: usize = 18;
const MAX_MUSIC_MAP_ROLE_POINTS: usize = 2;
const MUSIC_MAP_EARLY_HOOK_FLOOR_SECONDS: f64 = 28.0;
const MUSIC_MAP_EARLY_HOOK_FLOOR_RATIO: f64 = 0.16;
const MAX_MUSIC_MAP_HUMAN_CHECKS: usize = 2;
const MUSIC_MAP_MIN_HOOK_CONFIDENCE: f32 = 0.52;
const MUSIC_MAP_MIN_HIGHLIGHT_SPAN_CONFIDENCE: f32 = 0.62;
const MUSIC_MAP_MIN_HIGHLIGHT_CONFIDENCE: f32 = 0.66;
const MUSIC_MAP_MIN_ENTRY_CONFIDENCE: f32 = 0.64;
const MUSIC_MAP_MIN_EXIT_CONFIDENCE: f32 = 0.62;
const MUSIC_MAP_MIN_RISK_CONFIDENCE: f32 = 0.58;
const MUSIC_MAP_ATTENTION_WINDOW_SECONDS: f64 = 8.0;
const MUSIC_MAP_ATTENTION_HOP_SECONDS: f64 = 2.0;
const MAX_FUNCTIONAL_SEGMENTS: usize = 24;
const HIGHLIGHT_SEGMENT_ANCHOR_MIN_CONFIDENCE: f32 = 0.58;
const HIGHLIGHT_SEGMENT_OVERLAP_MIN: f32 = 0.28;
const FUNCTIONAL_BOUNDARY_MIN_CONFIDENCE: f32 = 0.22;
const FUNCTIONAL_CUT_MIN_GAP_SECONDS: f64 = 4.0;
const MAX_CHORUS_FUNCTIONAL_SEGMENT_SECONDS: f64 = 34.0;
const LONG_FUNCTIONAL_GAP_SECONDS: f64 = 32.0;
const MIX_POINT_VOCAL_SAFE_TARGET: f32 = 0.44;
const MIX_POINT_PERCEPTUAL_TARGET: f32 = 0.50;
const TEMPO_MAP_WINDOW_SECONDS: f64 = 18.0;
const TEMPO_MAP_HOP_SECONDS: f64 = 4.0;
const TEMPO_MAP_MIN_WINDOW_SECONDS: f64 = 10.0;
const MAX_TEMPO_MAP_POINTS: usize = 96;
const TEMPO_LOCAL_MIN_CONFIDENCE: f32 = 0.18;
const TONAL_ANALYSIS_STRIDE_FRAMES: usize = 4;
const TONAL_MIN_FRAME_RMS: f32 = 0.004;
const SPECTRUM_ANALYSIS_STRIDE_FRAMES: usize = 2;
const SPECTRUM_MIN_FRAME_RMS: f32 = 0.003;
const SPECTRUM_BAND_FREQUENCIES: [[f64; 2]; 8] = [
    [55.0, 82.0],
    [110.0, 165.0],
    [220.0, 330.0],
    [440.0, 660.0],
    [880.0, 1320.0],
    [1760.0, 2640.0],
    [3520.0, 5280.0],
    [7040.0, 10560.0],
];
const LUFS_K_WEIGHTING_PROXY_OFFSET_DB: f32 = -0.691;
const STRUCTURE_RECURRENCE_WINDOW_BINS: usize = 16;
const STRUCTURE_NOVELTY_WINDOW_BINS: usize = 8;
const MUSIC_ANALYSIS_PROGRESS_FINISHED_TTL: Duration = Duration::from_secs(8);

#[derive(Clone, Debug)]
pub(crate) struct MusicAnalysisProgressSnapshot {
    pub percent: f32,
    pub stage: String,
    pub finished: bool,
    pub error: Option<String>,
    updated_at: Instant,
}

static MUSIC_ANALYSIS_PROGRESS: OnceLock<Mutex<HashMap<String, MusicAnalysisProgressSnapshot>>> =
    OnceLock::new();

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicAnalysisManifest {
    pub schema_version: u32,
    pub analyzer_version: u32,
    pub media_file_size: u64,
    pub updated_unix_seconds: u64,
    pub duration_seconds: f64,
    pub sample_rate: u32,
    pub channels: u32,
    pub loudness: MusicLoudnessAnalysis,
    #[serde(default)]
    pub harmonic: MusicHarmonicAnalysis,
    pub tempo: MusicTempoAnalysis,
    pub sections: MusicSectionAnalysis,
    pub mix_points: MusicMixPointAnalysis,
    #[serde(default)]
    pub music_map: StageMixMusicMap,
    pub section_curves: MusicSectionCurveAnalysis,
    pub energy_curve: Vec<MusicEnergyPoint>,
    #[serde(default)]
    pub spectrum_curve: Vec<MusicSpectrumPoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicLoudnessAnalysis {
    pub rms: f32,
    pub peak: f32,
    pub rms_db: f32,
    pub peak_db: f32,
    #[serde(default)]
    pub integrated_lufs: f32,
    #[serde(default)]
    pub short_term_lufs: f32,
    #[serde(default)]
    pub true_peak: f32,
    #[serde(default)]
    pub true_peak_db: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct MusicHarmonicAnalysis {
    pub key_index: Option<u8>,
    pub key_name: Option<String>,
    pub scale: Option<String>,
    pub confidence: f32,
    #[serde(default)]
    pub chroma: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicTempoAnalysis {
    pub bpm: Option<f32>,
    pub confidence: f32,
    pub beat_grid: Option<MusicBeatGrid>,
    #[serde(default)]
    pub downbeat_grid: Option<MusicDownbeatGrid>,
    #[serde(default)]
    pub tempo_map: Vec<MusicTempoPoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicTempoPoint {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub center_seconds: f64,
    pub bpm: Option<f32>,
    pub confidence: f32,
    pub stable: bool,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicBeatGrid {
    pub first_beat_seconds: f64,
    pub interval_seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicDownbeatGrid {
    pub first_downbeat_seconds: f64,
    pub bar_interval_seconds: f64,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicSectionAnalysis {
    pub intro: Option<MusicTimeRange>,
    pub outro: Option<MusicTimeRange>,
    pub highlight_candidates: Vec<MusicSectionCandidate>,
    #[serde(default)]
    pub functional_segments: Vec<MusicFunctionalSegment>,
    #[serde(default)]
    pub segment_tempo: Vec<MusicSegmentTempo>,
    #[serde(default)]
    pub structure: MusicStructureAnalysis,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicSegmentTempo {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub role: MusicFunctionalRole,
    pub bpm: Option<f32>,
    pub confidence: f32,
    pub stable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicSectionCandidate {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub confidence: f32,
    pub reason: String,
    #[serde(default)]
    pub scores: MusicSectionCandidateScores,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct MusicSectionCandidateScores {
    pub total: f32,
    pub chorusness: f32,
    pub repetition: f32,
    pub energy: f32,
    pub contrast: f32,
    pub boundary: f32,
    pub position: f32,
    pub density: f32,
    pub duration: f32,
    #[serde(default)]
    pub segment_wholeness: f32,
    #[serde(default)]
    pub perceptual: f32,
    #[serde(default)]
    pub structural_recurrence: f32,
    #[serde(default)]
    pub structural_novelty: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicFunctionalSegment {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub role: MusicFunctionalRole,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MusicFunctionalRole {
    Intro,
    Verse,
    PreChorus,
    Chorus,
    FinalChorus,
    Bridge,
    Instrumental,
    Outro,
    Silence,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicMixPointAnalysis {
    pub mix_in: Vec<MusicMixPoint>,
    pub mix_out: Vec<MusicMixPoint>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct StageMixMusicMap {
    pub debug_only: bool,
    #[serde(default)]
    pub hook_start: Vec<MusicMapPoint>,
    #[serde(default)]
    pub highlight_span: Vec<MusicMapSpan>,
    #[serde(default)]
    pub highlight_peak: Vec<MusicMapPoint>,
    #[serde(default)]
    pub entry_safe: Vec<MusicMapPoint>,
    #[serde(default)]
    pub exit_safe: Vec<MusicMapPoint>,
    #[serde(default)]
    pub post_peak_valley: Vec<MusicMapPoint>,
    #[serde(default)]
    pub tail_or_silence_risk: Vec<MusicMapPoint>,
    #[serde(default)]
    pub human_check_queue: Vec<MusicMapHumanCheck>,
    #[serde(default)]
    pub summary_zh: String,
    #[serde(default)]
    pub source_candidate_count: u32,
    #[serde(default)]
    pub suppressed_candidate_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicMapSpan {
    pub start_seconds: f64,
    pub lift_seconds: f64,
    pub peak_seconds: f64,
    pub end_seconds: f64,
    pub confidence: f32,
    pub reason_zh: String,
    #[serde(default)]
    pub listen_from_seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicMapPoint {
    pub role: MusicMapRole,
    pub time_seconds: f64,
    pub confidence: f32,
    pub reason_zh: String,
    #[serde(default)]
    pub lyric_text: Option<String>,
    #[serde(default)]
    pub listen_from_seconds: f64,
    #[serde(default)]
    pub listen_to_seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicMapHumanCheck {
    pub time_seconds: f64,
    pub question_zh: String,
    pub why_ask: String,
    pub expected_labels: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MusicMapRole {
    HookStart,
    HighlightPeak,
    EntrySafe,
    ExitSafe,
    PostPeakValley,
    TailOrSilenceRisk,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicMixPoint {
    pub time_seconds: f64,
    pub confidence: f32,
    pub reason: String,
    #[serde(default)]
    pub phrase_snap_seconds: f64,
    #[serde(default)]
    pub vocal_safety: f32,
    #[serde(default)]
    pub perceptual_score: f32,
    #[serde(default)]
    pub phrase_closure: f32,
    #[serde(default)]
    pub masking_opportunity: f32,
    #[serde(default)]
    pub attention_safety: f32,
    #[serde(default)]
    pub expectation_safety: f32,
    #[serde(default)]
    pub phrase_grid_fit: f32,
    #[serde(default)]
    pub emotional_continuity: f32,
    #[serde(default)]
    pub vocal_handoff_score: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicTimeRange {
    pub start_seconds: f64,
    pub end_seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicEnergyPoint {
    pub time_seconds: f64,
    pub rms: f32,
    pub peak: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicSpectrumPoint {
    pub time_seconds: f64,
    pub bands: [u8; 8],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicSectionCurveAnalysis {
    pub hop_seconds: f64,
    pub chorusness: Vec<MusicCurvePoint>,
    pub boundary: Vec<MusicCurvePoint>,
    pub boundary_candidates: Vec<MusicBoundaryCandidate>,
    #[serde(default)]
    pub structure: MusicStructureAnalysis,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct MusicStructureAnalysis {
    #[serde(default)]
    pub recurrence: Vec<MusicCurvePoint>,
    #[serde(default)]
    pub novelty: Vec<MusicCurvePoint>,
    #[serde(default)]
    pub novelty_boundaries: Vec<MusicBoundaryCandidate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicCurvePoint {
    pub time_seconds: f64,
    pub value: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MusicBoundaryCandidate {
    pub time_seconds: f64,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Clone, Debug)]
struct AnalysisFrame {
    time_seconds: f64,
    rms: f32,
    peak: f32,
    chroma: [f32; 12],
    spectrum: [f32; 8],
    spectrum_sampled: bool,
}

#[derive(Clone, Debug)]
struct MusicMapAttentionContext {
    global_rms: f32,
    strongest_window_score: f32,
}

pub(crate) fn spawn_music_analysis_if_needed(
    media_path: PathBuf,
    ext: String,
    analysis_path: PathBuf,
    duration_hint_seconds: Option<f64>,
) {
    if analysis_is_current(&media_path, &analysis_path) {
        clear_music_analysis_progress(&analysis_path);
        return;
    }
    if music_analysis_progress_for_path(&analysis_path).is_some() {
        return;
    }

    set_music_analysis_progress(&analysis_path, 0.01, "queued", false, None);
    thread::spawn(move || {
        let mut progress = |percent: f32, stage: &'static str| {
            set_music_analysis_progress(&analysis_path, percent, stage, false, None);
        };
        if let Err(error) = analyze_music_file_if_needed_with_progress(
            &media_path,
            &ext,
            &analysis_path,
            duration_hint_seconds,
            Some(&mut progress),
        ) {
            set_music_analysis_progress(&analysis_path, 1.0, "failed", true, Some(error.clone()));
            eprintln!("[music-analysis] skipped {}: {error}", media_path.display());
        } else {
            set_music_analysis_progress(&analysis_path, 1.0, "complete", true, None);
        }
    });
}

pub(crate) fn analyze_music_file_if_needed(
    media_path: &Path,
    ext: &str,
    analysis_path: &Path,
    duration_hint_seconds: Option<f64>,
) -> Result<(), String> {
    analyze_music_file_if_needed_with_progress(
        media_path,
        ext,
        analysis_path,
        duration_hint_seconds,
        None,
    )
}

fn analyze_music_file_if_needed_with_progress(
    media_path: &Path,
    ext: &str,
    analysis_path: &Path,
    duration_hint_seconds: Option<f64>,
    mut progress: Option<&mut dyn FnMut(f32, &'static str)>,
) -> Result<(), String> {
    if analysis_is_current(media_path, analysis_path) {
        emit_analysis_progress(&mut progress, 1.0, "current");
        return Ok(());
    }
    let manifest = analyze_music_file(media_path, ext, duration_hint_seconds, &mut progress)?;
    emit_analysis_progress(&mut progress, 0.97, "writing manifest");
    write_yaml_file(analysis_path, &manifest)
        .map_err(|error| format!("Could not write music analysis manifest: {error}"))
}

pub(crate) fn music_analysis_progress_for_path(
    analysis_path: &Path,
) -> Option<MusicAnalysisProgressSnapshot> {
    let key = music_analysis_progress_key(analysis_path);
    let registry = MUSIC_ANALYSIS_PROGRESS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut registry = registry.lock().ok()?;
    let snapshot = registry.get(&key).cloned()?;
    if snapshot.finished && snapshot.updated_at.elapsed() > MUSIC_ANALYSIS_PROGRESS_FINISHED_TTL {
        registry.remove(&key);
        return None;
    }
    Some(snapshot)
}

fn set_music_analysis_progress(
    analysis_path: &Path,
    percent: f32,
    stage: &str,
    finished: bool,
    error: Option<String>,
) {
    let key = music_analysis_progress_key(analysis_path);
    let registry = MUSIC_ANALYSIS_PROGRESS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut registry) = registry.lock() {
        registry.insert(
            key,
            MusicAnalysisProgressSnapshot {
                percent: percent.clamp(0.0, 1.0),
                stage: stage.to_owned(),
                finished,
                error,
                updated_at: Instant::now(),
            },
        );
    }
}

fn clear_music_analysis_progress(analysis_path: &Path) {
    let key = music_analysis_progress_key(analysis_path);
    let registry = MUSIC_ANALYSIS_PROGRESS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut registry) = registry.lock() {
        registry.remove(&key);
    }
}

fn music_analysis_progress_key(analysis_path: &Path) -> String {
    analysis_path.to_string_lossy().into_owned()
}

fn emit_analysis_progress(
    progress: &mut Option<&mut dyn FnMut(f32, &'static str)>,
    percent: f32,
    stage: &'static str,
) {
    if let Some(callback) = progress.as_deref_mut() {
        callback(percent.clamp(0.0, 1.0), stage);
    }
}

fn analysis_is_current(media_path: &Path, analysis_path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(media_path) else {
        return false;
    };
    let Some(manifest) = read_yaml_file::<MusicAnalysisManifest>(analysis_path) else {
        return false;
    };
    music_analysis_manifest_is_current(&manifest, Some(metadata.len()))
}

pub(crate) fn music_analysis_manifest_is_current(
    manifest: &MusicAnalysisManifest,
    media_file_size: Option<u64>,
) -> bool {
    manifest.schema_version == MUSIC_ANALYSIS_SCHEMA_VERSION
        && manifest.analyzer_version == MUSIC_ANALYSIS_ANALYZER_VERSION
        && media_file_size.map_or(true, |size| manifest.media_file_size == size)
        && manifest.duration_seconds.is_finite()
        && manifest.duration_seconds > 0.0
}

fn analysis_codec_registry() -> &'static CodecRegistry {
    static CODEC_REGISTRY: OnceLock<CodecRegistry> = OnceLock::new();
    CODEC_REGISTRY.get_or_init(|| {
        let mut registry = CodecRegistry::new();
        symphonia::default::register_enabled_codecs(&mut registry);
        registry.register_audio_decoder::<OpusDecoder>();
        registry
    })
}

fn analyze_music_file(
    media_path: &Path,
    ext: &str,
    duration_hint_seconds: Option<f64>,
    progress: &mut Option<&mut dyn FnMut(f32, &'static str)>,
) -> Result<MusicAnalysisManifest, String> {
    emit_analysis_progress(progress, 0.03, "opening audio");
    let media_file_size = fs::metadata(media_path)
        .map_err(|error| format!("Could not read media metadata: {error}"))?
        .len();
    let source = File::open(media_path)
        .map_err(|error| format!("Could not open media for analysis: {error}"))?;
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
        .map_err(|error| format!("Could not read media format for analysis: {error}"))?;

    let (track_id, mut decoder) = {
        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| "No audio track was found for analysis.".to_owned())?;
        let codec_params = track
            .codec_params
            .as_ref()
            .ok_or_else(|| "Audio codec parameters are missing for analysis.".to_owned())?;
        let audio_params = codec_params
            .audio()
            .ok_or_else(|| "Audio codec parameters are missing for analysis.".to_owned())?;
        let decoder = analysis_codec_registry()
            .make_audio_decoder(audio_params, &AudioDecoderOptions::default())
            .map_err(|error| format!("Could not create audio decoder for analysis: {error}"))?;
        (track.id, decoder)
    };

    let mut builder = AnalysisBuilder::new(duration_hint_seconds);
    let mut sample_buffer: Vec<f32> = Vec::new();
    let mut last_decode_percent = 0.04_f32;
    emit_analysis_progress(progress, last_decode_percent, "decoding audio");

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
            Err(error) => return Err(format!("Could not read packet for analysis: {error}")),
        };

        if packet.track_id != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(error) => return Err(format!("Could not decode packet for analysis: {error}")),
        };

        let spec = decoded.spec();
        let channels = spec.channels().count().max(1);
        let sample_rate = spec.rate().max(1);
        sample_buffer.resize(decoded.samples_interleaved(), f32::MID);
        decoded.copy_to_slice_interleaved(&mut sample_buffer);
        builder.push_interleaved(&sample_buffer, channels, sample_rate);
        if let Some(ratio) = builder.duration_progress_ratio() {
            let percent = 0.05 + ratio * 0.62;
            if percent - last_decode_percent >= 0.015 {
                last_decode_percent = percent;
                emit_analysis_progress(progress, percent, "decoding audio");
            }
        }
    }

    builder.finish(media_file_size, progress)
}

struct AnalysisBuilder {
    duration_hint_seconds: Option<f64>,
    sample_rate: u32,
    channels: u32,
    total_mono_samples: u64,
    sum_squares: f64,
    peak: f32,
    true_peak: f32,
    previous_mono_sample: Option<f32>,
    frame_samples: Vec<f32>,
    frames: Vec<AnalysisFrame>,
}

impl AnalysisBuilder {
    fn new(duration_hint_seconds: Option<f64>) -> Self {
        Self {
            duration_hint_seconds,
            sample_rate: 0,
            channels: 0,
            total_mono_samples: 0,
            sum_squares: 0.0,
            peak: 0.0,
            true_peak: 0.0,
            previous_mono_sample: None,
            frame_samples: Vec::with_capacity(ANALYSIS_FRAME_SIZE),
            frames: Vec::new(),
        }
    }

    fn push_interleaved(&mut self, samples: &[f32], channels: usize, sample_rate: u32) {
        if samples.is_empty() {
            return;
        }
        if self.sample_rate == 0 {
            self.sample_rate = sample_rate.max(1);
        }
        if self.channels == 0 {
            self.channels = channels.max(1) as u32;
        }

        for frame in samples.chunks(channels.max(1)) {
            let mono = frame.iter().copied().sum::<f32>() / frame.len().max(1) as f32;
            self.sum_squares += f64::from(mono) * f64::from(mono);
            self.peak = self.peak.max(mono.abs());
            self.true_peak = self
                .true_peak
                .max(true_peak_proxy_sample(self.previous_mono_sample, mono));
            self.previous_mono_sample = Some(mono);
            self.total_mono_samples = self.total_mono_samples.saturating_add(1);
            self.frame_samples.push(mono);
            if self.frame_samples.len() >= ANALYSIS_FRAME_SIZE {
                self.flush_frame();
            }
        }
    }

    fn duration_progress_ratio(&self) -> Option<f32> {
        let duration = self
            .duration_hint_seconds
            .filter(|duration| duration.is_finite() && *duration > 0.0)?;
        if self.sample_rate == 0 {
            return None;
        }
        let decoded_seconds = self.total_mono_samples as f64 / self.sample_rate.max(1) as f64;
        Some((decoded_seconds / duration).clamp(0.0, 1.0) as f32)
    }

    fn flush_frame(&mut self) {
        if self.frame_samples.is_empty() {
            return;
        }
        let sample_rate = self.sample_rate.max(1) as f64;
        let frame_len = self.frame_samples.len().max(1) as f64;
        let frame_start_samples = self
            .total_mono_samples
            .saturating_sub(self.frame_samples.len() as u64);
        let time_seconds = frame_start_samples as f64 / sample_rate;
        let sum_squares = self
            .frame_samples
            .iter()
            .map(|sample| f64::from(*sample) * f64::from(*sample))
            .sum::<f64>();
        let peak = self
            .frame_samples
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0_f32, f32::max);
        // Lightweight harmonic handoff support: capture low-rate chroma only on
        // every few analysis frames. Stage Mix uses this as a compatibility
        // prior, not as a DJ-grade key lock or pitch-shift instruction.
        let rms = (sum_squares / frame_len).sqrt() as f32;
        let chroma = if self.frames.len() % TONAL_ANALYSIS_STRIDE_FRAMES == 0
            && peak >= TONAL_MIN_FRAME_RMS
        {
            estimate_frame_chroma(&self.frame_samples, self.sample_rate)
        } else {
            [0.0; 12]
        };
        let spectrum_sampled = self.frames.len() % SPECTRUM_ANALYSIS_STRIDE_FRAMES == 0;
        let spectrum = if spectrum_sampled && rms >= SPECTRUM_MIN_FRAME_RMS {
            estimate_frame_spectrum(&self.frame_samples, self.sample_rate)
        } else {
            [0.0; 8]
        };
        self.frames.push(AnalysisFrame {
            time_seconds,
            rms,
            peak,
            chroma,
            spectrum,
            spectrum_sampled,
        });
        self.frame_samples.clear();
    }

    fn finish(
        mut self,
        media_file_size: u64,
        progress: &mut Option<&mut dyn FnMut(f32, &'static str)>,
    ) -> Result<MusicAnalysisManifest, String> {
        self.flush_frame();
        if self.total_mono_samples == 0 || self.sample_rate == 0 {
            return Err("No audio samples were decoded for analysis.".to_owned());
        }

        emit_analysis_progress(progress, 0.70, "measuring loudness");
        let duration_seconds = self
            .duration_hint_seconds
            .filter(|duration| duration.is_finite() && *duration > 0.0)
            .unwrap_or_else(|| self.total_mono_samples as f64 / self.sample_rate.max(1) as f64);
        let rms = (self.sum_squares / self.total_mono_samples.max(1) as f64).sqrt() as f32;
        let loudness = estimate_loudness_analysis(&self.frames, rms, self.peak, self.true_peak);
        emit_analysis_progress(progress, 0.76, "estimating key");
        let harmonic = estimate_harmonic_analysis(&self.frames);
        emit_analysis_progress(progress, 0.81, "estimating tempo");
        let tempo = estimate_tempo(&self.frames, self.sample_rate, duration_seconds);
        emit_analysis_progress(progress, 0.86, "finding sections");
        let section_curves = estimate_section_curves(&self.frames, duration_seconds);
        let mut sections = estimate_sections(&self.frames, duration_seconds, &section_curves);
        emit_analysis_progress(progress, 0.91, "mapping segment BPM");
        sections.segment_tempo = estimate_segment_tempo(&sections.functional_segments, &tempo);
        emit_analysis_progress(progress, 0.94, "selecting mix points");
        let mix_points = estimate_mix_points(
            &sections,
            &tempo,
            duration_seconds,
            &section_curves,
            &self.frames,
        );
        emit_analysis_progress(progress, 0.95, "building music map");
        let music_map = estimate_stage_mix_music_map(
            &sections,
            &mix_points,
            &tempo,
            duration_seconds,
            &section_curves,
            &self.frames,
        );
        emit_analysis_progress(progress, 0.96, "finalizing manifest");
        let energy_curve = downsample_energy_curve(&self.frames);
        let spectrum_curve = downsample_spectrum_curve(&self.frames);

        Ok(MusicAnalysisManifest {
            schema_version: MUSIC_ANALYSIS_SCHEMA_VERSION,
            analyzer_version: MUSIC_ANALYSIS_ANALYZER_VERSION,
            media_file_size,
            updated_unix_seconds: unix_seconds_now(),
            duration_seconds,
            sample_rate: self.sample_rate,
            channels: self.channels.max(1),
            loudness,
            harmonic,
            tempo,
            sections,
            mix_points,
            music_map,
            section_curves,
            energy_curve,
            spectrum_curve,
        })
    }
}

fn true_peak_proxy_sample(previous: Option<f32>, current: f32) -> f32 {
    let current_peak = current.abs();
    let Some(previous) = previous else {
        return current_peak;
    };
    let endpoint_peak = previous.abs().max(current_peak);
    let slope = (current - previous).abs();

    // This is a cheap analysis-time true-peak proxy, not a full oversampled
    // ITU meter. It deliberately overestimates sharp edges slightly so Stage
    // Mix avoids adding overlap gain when a decoded file is already close to
    // inter-sample clipping.
    endpoint_peak.max(((previous + current) * 0.5).abs() + slope * 0.125)
}

fn estimate_loudness_analysis(
    frames: &[AnalysisFrame],
    rms: f32,
    peak: f32,
    true_peak: f32,
) -> MusicLoudnessAnalysis {
    let ungated_lufs = rms_to_lufs_proxy(rms);
    let relative_gate = (ungated_lufs - 10.0).max(-70.0);
    let mut gated_sum = 0.0_f64;
    let mut gated_count = 0_u32;
    for frame in frames {
        if rms_to_lufs_proxy(frame.rms) >= relative_gate {
            gated_sum += f64::from(frame.rms) * f64::from(frame.rms);
            gated_count = gated_count.saturating_add(1);
        }
    }
    let integrated_rms = if gated_count > 0 {
        (gated_sum / f64::from(gated_count)).sqrt() as f32
    } else {
        rms
    };
    let short_term_lufs = estimate_short_term_lufs_proxy(frames).unwrap_or(ungated_lufs);
    let true_peak = true_peak.max(peak);

    MusicLoudnessAnalysis {
        rms,
        peak,
        rms_db: amplitude_to_db(rms),
        peak_db: amplitude_to_db(peak),
        integrated_lufs: rms_to_lufs_proxy(integrated_rms),
        short_term_lufs,
        true_peak,
        true_peak_db: amplitude_to_db(true_peak),
    }
}

fn estimate_short_term_lufs_proxy(frames: &[AnalysisFrame]) -> Option<f32> {
    if frames.is_empty() {
        return None;
    }
    let mut best = -120.0_f32;
    for start in 0..frames.len() {
        let start_time = frames[start].time_seconds;
        let mut sum = 0.0_f64;
        let mut count = 0_u32;
        for frame in frames[start..].iter() {
            if frame.time_seconds - start_time > 3.0 {
                break;
            }
            sum += f64::from(frame.rms) * f64::from(frame.rms);
            count = count.saturating_add(1);
        }
        if count > 0 {
            let window_rms = (sum / f64::from(count)).sqrt() as f32;
            best = best.max(rms_to_lufs_proxy(window_rms));
        }
    }
    Some(best)
}

fn rms_to_lufs_proxy(rms: f32) -> f32 {
    (amplitude_to_db(rms) + LUFS_K_WEIGHTING_PROXY_OFFSET_DB).clamp(-120.0, 12.0)
}

fn estimate_frame_chroma(samples: &[f32], sample_rate: u32) -> [f32; 12] {
    if samples.len() < 128 || sample_rate == 0 {
        return [0.0; 12];
    }
    let mut chroma = [0.0_f32; 12];
    for midi in 36_i32..=83_i32 {
        let frequency = 440.0_f64 * 2.0_f64.powf((f64::from(midi) - 69.0) / 12.0);
        if frequency >= sample_rate as f64 * 0.45 {
            continue;
        }
        let class = midi.rem_euclid(12) as usize;
        chroma[class] += goertzel_power(samples, sample_rate, frequency);
    }
    normalize_chroma(&mut chroma);
    chroma
}

fn estimate_frame_spectrum(samples: &[f32], sample_rate: u32) -> [f32; 8] {
    if samples.len() < 128 || sample_rate == 0 {
        return [0.0; 8];
    }

    let nyquist_guard = f64::from(sample_rate) * 0.45;
    let mut bands = [0.0_f32; 8];
    for (band_index, frequencies) in SPECTRUM_BAND_FREQUENCIES.iter().enumerate() {
        let mut total = 0.0_f32;
        let mut count = 0_u32;
        for frequency in frequencies {
            if *frequency >= nyquist_guard {
                continue;
            }
            total += goertzel_power(samples, sample_rate, *frequency);
            count = count.saturating_add(1);
        }
        if count > 0 {
            bands[band_index] = total / count as f32;
        }
    }

    let peak = bands.iter().copied().fold(0.0_f32, f32::max);
    if peak <= 0.000_001 {
        return [0.0; 8];
    }
    for band in &mut bands {
        *band = (*band / peak).sqrt().clamp(0.0, 1.0);
    }
    bands
}

fn goertzel_power(samples: &[f32], sample_rate: u32, frequency: f64) -> f32 {
    if frequency <= 0.0 || sample_rate == 0 {
        return 0.0;
    }
    let omega = 2.0 * std::f64::consts::PI * frequency / f64::from(sample_rate);
    let coeff = 2.0 * omega.cos();
    let len = samples.len().max(1) as f64;
    let mut q1 = 0.0_f64;
    let mut q2 = 0.0_f64;
    for (index, sample) in samples.iter().enumerate() {
        let phase = index as f64 / (len - 1.0).max(1.0);
        let window = 0.5 - 0.5 * (2.0 * std::f64::consts::PI * phase).cos();
        let q0 = coeff * q1 - q2 + f64::from(*sample) * window;
        q2 = q1;
        q1 = q0;
    }
    let power = q1 * q1 + q2 * q2 - coeff * q1 * q2;
    (power.max(0.0) / len).sqrt() as f32
}

fn normalize_chroma(chroma: &mut [f32; 12]) {
    let sum = chroma.iter().copied().sum::<f32>();
    if sum <= 0.000001 {
        return;
    }
    for value in chroma.iter_mut() {
        *value = (*value / sum).clamp(0.0, 1.0);
    }
}

fn estimate_harmonic_analysis(frames: &[AnalysisFrame]) -> MusicHarmonicAnalysis {
    let mut chroma = [0.0_f32; 12];
    let mut total_weight = 0.0_f32;
    for frame in frames {
        let weight = frame.rms.max(0.0);
        if weight <= TONAL_MIN_FRAME_RMS {
            continue;
        }
        let frame_chroma_sum = frame.chroma.iter().copied().sum::<f32>();
        if frame_chroma_sum <= 0.000001 {
            continue;
        }
        for (index, value) in frame.chroma.iter().enumerate() {
            chroma[index] += *value * weight;
        }
        total_weight += weight;
    }
    if total_weight <= 0.000001 {
        return MusicHarmonicAnalysis::default();
    }
    normalize_chroma(&mut chroma);

    let mut best: Option<(f32, u8, &'static str)> = None;
    let mut second = 0.0_f32;
    for tonic in 0..12_u8 {
        for (scale, profile) in [
            ("major", KRUMHANSL_MAJOR_PROFILE),
            ("minor", KRUMHANSL_MINOR_PROFILE),
        ] {
            let score = key_profile_score(&chroma, &profile, tonic);
            if best.map_or(true, |(best_score, _, _)| score > best_score) {
                if let Some((best_score, _, _)) = best {
                    second = second.max(best_score);
                }
                best = Some((score, tonic, scale));
            } else {
                second = second.max(score);
            }
        }
    }

    let Some((best_score, tonic, scale)) = best else {
        return MusicHarmonicAnalysis::default();
    };
    let confidence = if best_score <= 0.000001 {
        0.0
    } else {
        ((best_score - second) / best_score).clamp(0.0, 1.0)
    };
    let key_name = format!("{} {}", pitch_class_name(tonic), scale);

    MusicHarmonicAnalysis {
        key_index: Some(tonic),
        key_name: Some(key_name),
        scale: Some(scale.to_owned()),
        confidence,
        chroma: chroma.to_vec(),
    }
}

const KRUMHANSL_MAJOR_PROFILE: [f32; 12] = [
    6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
];
const KRUMHANSL_MINOR_PROFILE: [f32; 12] = [
    6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
];

fn key_profile_score(chroma: &[f32; 12], profile: &[f32; 12], tonic: u8) -> f32 {
    let profile_sum = profile.iter().copied().sum::<f32>().max(0.0001);
    let mut score = 0.0_f32;
    for class in 0..12_usize {
        let relative = (class + 12 - tonic as usize) % 12;
        score += chroma[class] * (profile[relative] / profile_sum);
    }
    score.clamp(0.0, 1.0)
}

fn pitch_class_name(index: u8) -> &'static str {
    match index % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "Eb",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "Ab",
        9 => "A",
        10 => "Bb",
        _ => "B",
    }
}

fn estimate_tempo(
    frames: &[AnalysisFrame],
    sample_rate: u32,
    duration_seconds: f64,
) -> MusicTempoAnalysis {
    if frames.len() < 16 || sample_rate == 0 {
        return MusicTempoAnalysis {
            bpm: None,
            confidence: 0.0,
            beat_grid: None,
            downbeat_grid: None,
            tempo_map: Vec::new(),
        };
    }

    let onset = onset_envelope(frames);
    let frame_duration = ANALYSIS_FRAME_SIZE as f64 / sample_rate.max(1) as f64;
    let mut best_bpm = None;
    let mut best_score = 0.0_f64;
    let mut total_score = 0.0_f64;
    let mut score_count = 0_u32;

    for bpm in 70..=180 {
        let lag = (60.0 / bpm as f64 / frame_duration).round() as usize;
        if lag == 0 || lag >= onset.len() {
            continue;
        }
        let mut score = 0.0_f64;
        for index in lag..onset.len() {
            score += f64::from(onset[index]) * f64::from(onset[index - lag]);
        }
        total_score += score;
        score_count += 1;
        if score > best_score {
            best_score = score;
            best_bpm = Some(bpm as f32);
        }
    }

    let Some(bpm) = best_bpm else {
        return MusicTempoAnalysis {
            bpm: None,
            confidence: 0.0,
            beat_grid: None,
            downbeat_grid: None,
            tempo_map: Vec::new(),
        };
    };

    let average_score = if score_count > 0 {
        total_score / f64::from(score_count)
    } else {
        0.0
    };
    let confidence = if best_score <= 0.0 || average_score <= 0.0 {
        0.0
    } else {
        ((best_score / average_score - 1.0) / 5.0).clamp(0.0, 1.0) as f32
    };
    let interval_seconds = 60.0 / f64::from(bpm);
    let first_beat_seconds = strongest_first_beat_time(frames, &onset, interval_seconds);

    let tempo_map = estimate_tempo_map(
        frames,
        &onset,
        sample_rate,
        duration_seconds,
        bpm,
        confidence,
    );
    let downbeat_grid = estimate_downbeat_grid(
        frames,
        &onset,
        first_beat_seconds,
        interval_seconds,
        duration_seconds,
    );

    MusicTempoAnalysis {
        bpm: Some(bpm),
        confidence,
        beat_grid: Some(MusicBeatGrid {
            first_beat_seconds,
            interval_seconds,
        }),
        downbeat_grid,
        tempo_map,
    }
}

fn onset_envelope(frames: &[AnalysisFrame]) -> Vec<f32> {
    let mut previous = frames.first().map(|frame| frame.rms).unwrap_or(0.0);
    frames
        .iter()
        .map(|frame| {
            let delta = (frame.rms - previous).max(0.0);
            previous = frame.rms;
            delta
        })
        .collect()
}

fn strongest_first_beat_time(
    frames: &[AnalysisFrame],
    onset: &[f32],
    interval_seconds: f64,
) -> f64 {
    let search_end = interval_seconds.max(0.25) * 2.0;
    let mut best = (0.0_f64, 0.0_f32);
    for (frame, value) in frames.iter().zip(onset.iter()) {
        if frame.time_seconds > search_end {
            break;
        }
        if *value > best.1 {
            best = (frame.time_seconds, *value);
        }
    }
    best.0.max(0.0)
}

fn estimate_downbeat_grid(
    frames: &[AnalysisFrame],
    onset: &[f32],
    first_beat_seconds: f64,
    interval_seconds: f64,
    duration_seconds: f64,
) -> Option<MusicDownbeatGrid> {
    if frames.len() < 24
        || onset.len() != frames.len()
        || interval_seconds <= 0.0
        || duration_seconds < interval_seconds * 8.0
    {
        return None;
    }
    let bar_interval = interval_seconds * 4.0;
    let mut best: Option<(f32, f64)> = None;
    let mut second = 0.0_f32;

    for phase in 0..4 {
        let mut first_downbeat = first_beat_seconds + f64::from(phase) * interval_seconds;
        while first_downbeat >= bar_interval {
            first_downbeat -= bar_interval;
        }

        let mut downbeat_sum = 0.0_f32;
        let mut offbeat_sum = 0.0_f32;
        let mut bars = 0_u32;
        let mut time = first_downbeat;
        while time < duration_seconds {
            if time >= 0.0 {
                downbeat_sum += onset_near_time(frames, onset, time, interval_seconds * 0.18);
                for beat in 1..4 {
                    let beat_time = time + f64::from(beat) * interval_seconds;
                    if beat_time < duration_seconds {
                        offbeat_sum +=
                            onset_near_time(frames, onset, beat_time, interval_seconds * 0.18);
                    }
                }
                bars = bars.saturating_add(1);
            }
            time += bar_interval;
        }
        if bars < 3 {
            continue;
        }
        let downbeat_avg = downbeat_sum / bars.max(1) as f32;
        let offbeat_avg = offbeat_sum / (bars.saturating_mul(3)).max(1) as f32;
        let accent_ratio = downbeat_avg / offbeat_avg.max(0.0001);
        let score =
            (downbeat_avg * 0.66 + (accent_ratio - 1.0).clamp(0.0, 2.5) * 0.34).clamp(0.0, 4.0);

        if best.map_or(true, |(best_score, _)| score > best_score) {
            if let Some((best_score, _)) = best {
                second = second.max(best_score);
            }
            best = Some((score, first_downbeat));
        } else {
            second = second.max(score);
        }
    }

    let (best_score, first_downbeat) = best?;
    if best_score <= 0.0001 {
        return None;
    }
    let confidence = ((best_score - second) / best_score).clamp(0.0, 1.0);
    Some(MusicDownbeatGrid {
        first_downbeat_seconds: first_downbeat.max(0.0),
        bar_interval_seconds: bar_interval,
        confidence,
    })
}

fn onset_near_time(frames: &[AnalysisFrame], onset: &[f32], time_seconds: f64, window: f64) -> f32 {
    let mut best = 0.0_f32;
    for (frame, value) in frames.iter().zip(onset.iter()) {
        if (frame.time_seconds - time_seconds).abs() <= window.max(0.01) {
            best = best.max(*value);
        }
        if frame.time_seconds > time_seconds + window {
            break;
        }
    }
    best
}

fn estimate_tempo_map(
    frames: &[AnalysisFrame],
    onset: &[f32],
    sample_rate: u32,
    duration_seconds: f64,
    global_bpm: f32,
    global_confidence: f32,
) -> Vec<MusicTempoPoint> {
    if frames.len() < 32
        || onset.len() < 32
        || sample_rate == 0
        || duration_seconds < TEMPO_MAP_MIN_WINDOW_SECONDS
    {
        return Vec::new();
    }

    let frame_duration = ANALYSIS_FRAME_SIZE as f64 / sample_rate.max(1) as f64;
    if !frame_duration.is_finite() || frame_duration <= 0.0 {
        return Vec::new();
    }

    let window_seconds = TEMPO_MAP_WINDOW_SECONDS
        .min(duration_seconds)
        .max(TEMPO_MAP_MIN_WINDOW_SECONDS);
    let mut hop_seconds = TEMPO_MAP_HOP_SECONDS;
    let estimated_points = (duration_seconds / hop_seconds).ceil().max(1.0) as usize;
    if estimated_points > MAX_TEMPO_MAP_POINTS {
        hop_seconds = (duration_seconds / MAX_TEMPO_MAP_POINTS as f64).max(TEMPO_MAP_HOP_SECONDS);
    }

    let mut raw = Vec::new();
    let mut start_seconds = 0.0_f64;
    while start_seconds < duration_seconds && raw.len() < MAX_TEMPO_MAP_POINTS {
        let end_seconds = (start_seconds + window_seconds).min(duration_seconds);
        if end_seconds - start_seconds < TEMPO_MAP_MIN_WINDOW_SECONDS {
            break;
        }
        let start_index = seconds_to_frame_index(start_seconds, frame_duration, onset.len());
        let end_index =
            seconds_to_frame_index(end_seconds, frame_duration, onset.len()).max(start_index + 1);
        let (bpm, confidence) =
            estimate_local_bpm(onset, start_index, end_index, frame_duration, global_bpm);
        raw.push(MusicTempoPoint {
            start_seconds,
            end_seconds,
            center_seconds: (start_seconds + end_seconds) * 0.5,
            bpm,
            confidence,
            stable: bpm.is_some() && confidence >= TEMPO_LOCAL_MIN_CONFIDENCE,
            source: "local".to_owned(),
        });
        start_seconds += hop_seconds;
    }

    smooth_tempo_map(raw, global_bpm, global_confidence)
}

fn seconds_to_frame_index(seconds: f64, frame_duration: f64, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (seconds / frame_duration)
        .floor()
        .clamp(0.0, len.saturating_sub(1) as f64) as usize
}

fn estimate_local_bpm(
    onset: &[f32],
    start_index: usize,
    end_index: usize,
    frame_duration: f64,
    global_bpm: f32,
) -> (Option<f32>, f32) {
    let start_index = start_index.min(onset.len());
    let end_index = end_index.min(onset.len());
    if end_index.saturating_sub(start_index) < 16 {
        return (None, 0.0);
    }

    let mut best_bpm = None;
    let mut best_score = 0.0_f64;
    let mut total_score = 0.0_f64;
    let mut score_count = 0_u32;

    for bpm in 70..=180 {
        let lag = (60.0 / bpm as f64 / frame_duration).round() as usize;
        if lag == 0 || start_index + lag >= end_index {
            continue;
        }
        let mut score = 0.0_f64;
        for index in (start_index + lag)..end_index {
            score += f64::from(onset[index]) * f64::from(onset[index - lag]);
        }
        total_score += score;
        score_count = score_count.saturating_add(1);
        if score > best_score {
            best_score = score;
            best_bpm = Some(bpm as f32);
        }
    }

    let Some(raw_bpm) = best_bpm else {
        return (None, 0.0);
    };
    let average_score = if score_count > 0 {
        total_score / f64::from(score_count)
    } else {
        0.0
    };
    let confidence = if best_score <= 0.0 || average_score <= 0.0 {
        0.0
    } else {
        ((best_score / average_score - 1.0) / 5.5).clamp(0.0, 1.0) as f32
    };

    (
        Some(normalize_local_bpm_to_global(raw_bpm, global_bpm)),
        confidence,
    )
}

fn normalize_local_bpm_to_global(local_bpm: f32, global_bpm: f32) -> f32 {
    let global_bpm = global_bpm.clamp(50.0, 220.0);
    let candidates = [local_bpm, local_bpm * 2.0, local_bpm * 0.5];
    candidates
        .into_iter()
        .filter(|candidate| candidate.is_finite() && *candidate >= 50.0 && *candidate <= 220.0)
        .min_by(|a, b| {
            relative_bpm_gap(*a, global_bpm)
                .partial_cmp(&relative_bpm_gap(*b, global_bpm))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(local_bpm)
}

fn smooth_tempo_map(
    raw: Vec<MusicTempoPoint>,
    global_bpm: f32,
    global_confidence: f32,
) -> Vec<MusicTempoPoint> {
    if raw.is_empty() {
        return raw;
    }

    let mut output = Vec::with_capacity(raw.len());
    for index in 0..raw.len() {
        let median = neighbor_tempo_median(&raw, index).unwrap_or(global_bpm);
        let mut point = raw[index].clone();
        let raw_bpm = point.bpm.unwrap_or(median);
        let raw_gap = relative_bpm_gap(raw_bpm, median);
        let global_gap = relative_bpm_gap(median, global_bpm);

        let use_raw = point.confidence >= TEMPO_LOCAL_MIN_CONFIDENCE && raw_gap <= 0.10;
        let smoothed = if use_raw {
            raw_bpm * 0.70 + median * 0.30
        } else if global_confidence >= 0.42 && global_gap > 0.16 {
            median * 0.65 + global_bpm * 0.35
        } else {
            median
        };

        point.bpm = Some(smoothed.clamp(50.0, 220.0));
        point.stable = point.confidence >= TEMPO_LOCAL_MIN_CONFIDENCE
            && relative_bpm_gap(smoothed, global_bpm) <= 0.22;
        point.source = if use_raw {
            "local_smoothed".to_owned()
        } else if point.confidence > 0.0 {
            "neighbor_smoothed".to_owned()
        } else {
            "global_fallback".to_owned()
        };
        output.push(point);
    }
    output
}

fn neighbor_tempo_median(points: &[MusicTempoPoint], center: usize) -> Option<f32> {
    let start = center.saturating_sub(2);
    let end = (center + 3).min(points.len());
    let mut values: Vec<f32> = points[start..end]
        .iter()
        .filter(|point| point.confidence >= 0.10)
        .filter_map(|point| point.bpm)
        .filter(|bpm| bpm.is_finite())
        .collect();
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(values[values.len() / 2])
}

fn relative_bpm_gap(a: f32, b: f32) -> f32 {
    ((a - b).abs() / ((a + b) * 0.5).max(1.0)).clamp(0.0, 1.0)
}

fn estimate_segment_tempo(
    segments: &[MusicFunctionalSegment],
    tempo: &MusicTempoAnalysis,
) -> Vec<MusicSegmentTempo> {
    if segments.is_empty() || tempo.tempo_map.is_empty() {
        return Vec::new();
    }

    segments
        .iter()
        .map(|segment| {
            let mut weighted_sum = 0.0_f32;
            let mut weight_total = 0.0_f32;
            let mut stable_count = 0_u32;
            for point in &tempo.tempo_map {
                if point.center_seconds < segment.start_seconds
                    || point.center_seconds > segment.end_seconds
                {
                    continue;
                }
                let Some(bpm) = point.bpm else {
                    continue;
                };
                let weight = point.confidence.max(0.05);
                weighted_sum += bpm * weight;
                weight_total += weight;
                if point.stable {
                    stable_count = stable_count.saturating_add(1);
                }
            }
            let bpm = (weight_total > 0.0).then_some(weighted_sum / weight_total);
            let confidence = if weight_total > 0.0 {
                (weight_total / 3.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            MusicSegmentTempo {
                start_seconds: segment.start_seconds,
                end_seconds: segment.end_seconds,
                role: segment.role.clone(),
                bpm,
                confidence,
                stable: bpm.is_some()
                    && stable_count > 0
                    && confidence >= TEMPO_LOCAL_MIN_CONFIDENCE,
            }
        })
        .collect()
}

#[derive(Clone, Debug)]
struct EnergyBin {
    time_seconds: f64,
    energy: f32,
    peak: f32,
}

fn estimate_section_curves(
    frames: &[AnalysisFrame],
    duration_seconds: f64,
) -> MusicSectionCurveAnalysis {
    let bins = build_energy_bins(frames, duration_seconds);
    if bins.is_empty() {
        return MusicSectionCurveAnalysis {
            hop_seconds: SECTION_CURVE_HOP_SECONDS,
            chorusness: Vec::new(),
            boundary: Vec::new(),
            boundary_candidates: Vec::new(),
            structure: MusicStructureAnalysis::default(),
        };
    }

    let global_energy = bins.iter().map(|bin| bin.energy).sum::<f32>() / bins.len().max(1) as f32;
    let max_energy = bins
        .iter()
        .map(|bin| bin.energy)
        .fold(0.0_f32, f32::max)
        .max(0.0001);
    let structure = estimate_structure_analysis(&bins, duration_seconds);
    let mut chorusness = Vec::with_capacity(bins.len());
    let mut boundary = Vec::with_capacity(bins.len());

    for index in 0..bins.len() {
        let local_energy = smooth_energy(&bins, index, 3);
        let energy_score = (local_energy / max_energy).clamp(0.0, 1.0);
        let contrast_score = local_contrast_at(&bins, index, global_energy);
        let repetition_score = repetition_score_at(&bins, index);
        let stability_score = local_stability_score(&bins, index);
        let recurrence_score = structure
            .recurrence
            .get(index)
            .map(|point| point.value)
            .unwrap_or(0.0);
        let novelty_score = structure
            .novelty
            .get(index)
            .map(|point| point.value)
            .unwrap_or(0.0);
        let position =
            (bins[index].time_seconds / duration_seconds.max(1.0)).clamp(0.0, 1.0) as f32;
        let position_score = chorus_position_prior(position);
        let chorus_score = energy_score * 0.25
            + repetition_score * 0.23
            + recurrence_score * 0.17
            + contrast_score * 0.14
            + stability_score * 0.10
            + position_score * 0.11;
        let boundary_score =
            boundary_score_at(&bins, index, global_energy).max(novelty_score * 0.78);
        chorusness.push(MusicCurvePoint {
            time_seconds: bins[index].time_seconds,
            value: chorus_score.clamp(0.0, 1.0),
        });
        boundary.push(MusicCurvePoint {
            time_seconds: bins[index].time_seconds,
            value: boundary_score.clamp(0.0, 1.0),
        });
    }

    let boundary_candidates = boundary_candidates_from_curve(
        &boundary,
        duration_seconds,
        "local energy/novelty boundary",
    );
    MusicSectionCurveAnalysis {
        hop_seconds: SECTION_CURVE_HOP_SECONDS,
        chorusness,
        boundary,
        boundary_candidates,
        structure,
    }
}

fn estimate_structure_analysis(
    bins: &[EnergyBin],
    duration_seconds: f64,
) -> MusicStructureAnalysis {
    if bins.is_empty() {
        return MusicStructureAnalysis::default();
    }
    let recurrence = structure_recurrence_curve(bins);
    let novelty = structure_novelty_curve(bins);
    let novelty_boundaries =
        boundary_candidates_from_curve(&novelty, duration_seconds, "self-similarity novelty");
    MusicStructureAnalysis {
        recurrence,
        novelty,
        novelty_boundaries,
    }
}

fn structure_recurrence_curve(bins: &[EnergyBin]) -> Vec<MusicCurvePoint> {
    let mut out = Vec::with_capacity(bins.len());
    for index in 0..bins.len() {
        let start = index.saturating_sub(STRUCTURE_RECURRENCE_WINDOW_BINS / 2);
        let len = STRUCTURE_RECURRENCE_WINDOW_BINS
            .min(bins.len().saturating_sub(start))
            .max(1);
        let mut best = 0.0_f32;
        let min_offset = len.max(12);
        let max_offset = bins.len().saturating_sub(len + 1).min(112);
        if min_offset <= max_offset {
            for offset in (min_offset..=max_offset).step_by(2) {
                for other_start in [start.checked_sub(offset), Some(start + offset)] {
                    let Some(other_start) = other_start else {
                        continue;
                    };
                    if other_start + len >= bins.len() {
                        continue;
                    }
                    best = best.max(energy_window_similarity(bins, start, other_start, len));
                }
            }
        }
        out.push(MusicCurvePoint {
            time_seconds: bins[index].time_seconds,
            value: best.clamp(0.0, 1.0),
        });
    }
    out
}

fn structure_novelty_curve(bins: &[EnergyBin]) -> Vec<MusicCurvePoint> {
    let mut out = Vec::with_capacity(bins.len());
    for index in 0..bins.len() {
        let left_start = index.saturating_sub(STRUCTURE_NOVELTY_WINDOW_BINS);
        let left_end = index;
        let right_start = (index + 1).min(bins.len());
        let right_end = (index + 1 + STRUCTURE_NOVELTY_WINDOW_BINS).min(bins.len());
        let novelty = if left_end <= left_start || right_end <= right_start {
            0.0
        } else {
            structure_window_distance(bins, left_start, left_end, right_start, right_end)
        };
        out.push(MusicCurvePoint {
            time_seconds: bins[index].time_seconds,
            value: novelty.clamp(0.0, 1.0),
        });
    }
    out
}

fn structure_window_distance(
    bins: &[EnergyBin],
    a_start: usize,
    a_end: usize,
    b_start: usize,
    b_end: usize,
) -> f32 {
    let len = (a_end - a_start).min(b_end - b_start);
    if len == 0 {
        return 0.0;
    }
    let a_avg = local_average_energy(bins, a_start, a_end).max(0.0001);
    let b_avg = local_average_energy(bins, b_start, b_end).max(0.0001);
    let mut diff = 0.0_f32;
    for offset in 0..len {
        let a = bins[a_end - len + offset].energy / a_avg;
        let b = bins[b_start + offset].energy / b_avg;
        diff += (a - b).abs();
    }
    (diff / len as f32 / 1.6).clamp(0.0, 1.0)
}

fn build_energy_bins(frames: &[AnalysisFrame], duration_seconds: f64) -> Vec<EnergyBin> {
    if frames.is_empty() || duration_seconds <= 0.0 {
        return Vec::new();
    }
    let bin_count = (duration_seconds / SECTION_CURVE_HOP_SECONDS)
        .ceil()
        .max(1.0) as usize;
    let mut sums = vec![0.0_f32; bin_count];
    let mut peaks = vec![0.0_f32; bin_count];
    let mut counts = vec![0_u32; bin_count];
    for frame in frames {
        let index = (frame.time_seconds / SECTION_CURVE_HOP_SECONDS)
            .floor()
            .clamp(0.0, (bin_count.saturating_sub(1)) as f64) as usize;
        sums[index] += frame.rms;
        peaks[index] = peaks[index].max(frame.peak);
        counts[index] = counts[index].saturating_add(1);
    }
    (0..bin_count)
        .map(|index| {
            let count = counts[index].max(1) as f32;
            EnergyBin {
                time_seconds: index as f64 * SECTION_CURVE_HOP_SECONDS,
                energy: sums[index] / count,
                peak: peaks[index],
            }
        })
        .collect()
}

fn smooth_energy(bins: &[EnergyBin], center: usize, radius: usize) -> f32 {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(bins.len());
    bins[start..end].iter().map(|bin| bin.energy).sum::<f32>() / (end - start).max(1) as f32
}

fn local_average_energy(bins: &[EnergyBin], start: usize, end: usize) -> f32 {
    if start >= end || start >= bins.len() {
        return 0.0;
    }
    let end = end.min(bins.len());
    bins[start..end].iter().map(|bin| bin.energy).sum::<f32>() / (end - start).max(1) as f32
}

fn local_contrast_at(bins: &[EnergyBin], index: usize, global_energy: f32) -> f32 {
    let center = smooth_energy(bins, index, 3);
    let left = local_average_energy(bins, index.saturating_sub(14), index.saturating_sub(6));
    let right = local_average_energy(
        bins,
        (index + 6).min(bins.len()),
        (index + 14).min(bins.len()),
    );
    let context = if left > 0.0 && right > 0.0 {
        (left + right) * 0.5
    } else if left > 0.0 {
        left
    } else if right > 0.0 {
        right
    } else {
        global_energy
    };
    ((center - context) / global_energy.max(0.0001)).clamp(0.0, 1.6) / 1.6
}

fn local_energy_contrast(bins: &[EnergyBin], start: usize, end: usize, global_energy: f32) -> f32 {
    let center = local_average_energy(bins, start, end);
    let left = local_average_energy(bins, start.saturating_sub(18), start.saturating_sub(4));
    let right = local_average_energy(bins, (end + 4).min(bins.len()), (end + 18).min(bins.len()));
    let context = match (left > 0.0, right > 0.0) {
        (true, true) => (left + right) * 0.5,
        (true, false) => left,
        (false, true) => right,
        (false, false) => global_energy,
    };
    ((center - context) / global_energy.max(0.0001)).clamp(0.0, 1.6) / 1.6
}

fn repetition_score_at(bins: &[EnergyBin], index: usize) -> f32 {
    let window = 12_usize;
    let start = index.saturating_sub(window / 2);
    repetition_score_for_window(bins, start, window)
}

fn repetition_score_for_window(bins: &[EnergyBin], start: usize, window: usize) -> f32 {
    if bins.len() < window * 3 || start + window >= bins.len() {
        return 0.0;
    }

    // Borrow the useful part of time-lag chorus detectors: do not only test a few
    // hand-picked verse/chorus distances. Scan plausible section lags and reward
    // a stable repeated contour. This is still very light-weight and keeps v2
    // portable, but it makes chorus/highlight evidence less dependent on one
    // fixed song form.
    let min_offset = (window + 4).max(16);
    let max_offset = (bins.len().saturating_sub(window + 1)).min(128);
    if min_offset > max_offset {
        return 0.0;
    }

    let mut best = 0.0_f32;
    for offset in (min_offset..=max_offset).step_by(2) {
        for other_start in [start.checked_sub(offset), Some(start + offset)] {
            let Some(other_start) = other_start else {
                continue;
            };
            if other_start + window >= bins.len() {
                continue;
            }
            let similarity = energy_window_similarity(bins, start, other_start, window);
            let lag_prior = (1.0 - ((offset as f32 - 48.0).abs() / 96.0).clamp(0.0, 1.0) * 0.18)
                .clamp(0.72, 1.0);
            best = best.max(similarity * lag_prior);
        }
    }
    best
}

fn energy_window_similarity(bins: &[EnergyBin], a_start: usize, b_start: usize, len: usize) -> f32 {
    let a_avg = local_average_energy(bins, a_start, a_start + len).max(0.0001);
    let b_avg = local_average_energy(bins, b_start, b_start + len).max(0.0001);
    let mut diff = 0.0_f32;
    for offset in 0..len {
        let a = bins[a_start + offset].energy / a_avg;
        let b = bins[b_start + offset].energy / b_avg;
        diff += (a - b).abs();
    }
    (1.0 - diff / len.max(1) as f32).clamp(0.0, 1.0)
}

fn local_stability_score(bins: &[EnergyBin], index: usize) -> f32 {
    let start = index.saturating_sub(5);
    let end = (index + 6).min(bins.len());
    if end <= start + 1 {
        return 0.0;
    }
    let avg = local_average_energy(bins, start, end).max(0.0001);
    let variance = bins[start..end]
        .iter()
        .map(|bin| {
            let delta = bin.energy - avg;
            delta * delta
        })
        .sum::<f32>()
        / (end - start).max(1) as f32;
    let cv = variance.sqrt() / avg;
    (1.0 - cv).clamp(0.0, 1.0)
}

fn boundary_score_at(bins: &[EnergyBin], index: usize, global_energy: f32) -> f32 {
    let left = local_average_energy(bins, index.saturating_sub(8), index.saturating_sub(1));
    let right = local_average_energy(
        bins,
        (index + 1).min(bins.len()),
        (index + 8).min(bins.len()),
    );
    if left <= 0.0 || right <= 0.0 {
        return 0.0;
    }
    ((right - left).abs() / global_energy.max(0.0001)).clamp(0.0, 2.0) / 2.0
}

fn boundary_candidates_from_curve(
    boundary: &[MusicCurvePoint],
    duration_seconds: f64,
    reason: &str,
) -> Vec<MusicBoundaryCandidate> {
    let mut scored = Vec::new();
    for index in 1..boundary.len().saturating_sub(1) {
        let value = boundary[index].value;
        if value < boundary[index - 1].value || value < boundary[index + 1].value || value < 0.18 {
            continue;
        }
        let time_seconds = boundary[index].time_seconds;
        if time_seconds < 4.0 || time_seconds > duration_seconds - 4.0 {
            continue;
        }
        scored.push((value, time_seconds));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut candidates = Vec::new();
    for (value, time_seconds) in scored {
        if candidates.len() >= MAX_BOUNDARY_CANDIDATES {
            break;
        }
        if candidates.iter().any(|candidate: &MusicBoundaryCandidate| {
            (candidate.time_seconds - time_seconds).abs() < 5.0
        }) {
            continue;
        }
        candidates.push(MusicBoundaryCandidate {
            time_seconds,
            confidence: value.clamp(0.0, 1.0),
            reason: reason.to_owned(),
        });
    }
    candidates.sort_by(|a, b| {
        a.time_seconds
            .partial_cmp(&b.time_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates
}

fn average_curve_value(curve: &[MusicCurvePoint], start_seconds: f64, end_seconds: f64) -> f32 {
    let mut sum = 0.0_f32;
    let mut count = 0_u32;
    for point in curve {
        if point.time_seconds >= start_seconds && point.time_seconds <= end_seconds {
            sum += point.value;
            count = count.saturating_add(1);
        }
    }
    if count == 0 { 0.0 } else { sum / count as f32 }
}

fn nearest_boundary_score(
    section_curves: &MusicSectionCurveAnalysis,
    time_seconds: f64,
    window_seconds: f64,
) -> f32 {
    section_curves
        .boundary_candidates
        .iter()
        .filter(|candidate| (candidate.time_seconds - time_seconds).abs() <= window_seconds)
        .map(|candidate| candidate.confidence)
        .fold(0.0_f32, f32::max)
}

fn nearest_structure_novelty_score(
    section_curves: &MusicSectionCurveAnalysis,
    start_seconds: f64,
    end_seconds: f64,
) -> f32 {
    section_curves
        .structure
        .novelty_boundaries
        .iter()
        .filter(|candidate| {
            (candidate.time_seconds - start_seconds).abs() <= 4.0
                || (candidate.time_seconds - end_seconds).abs() <= 4.0
        })
        .map(|candidate| candidate.confidence)
        .fold(0.0_f32, f32::max)
}

fn snap_to_nearby_boundary(
    section_curves: &MusicSectionCurveAnalysis,
    time_seconds: f64,
    window_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    section_curves
        .boundary_candidates
        .iter()
        .filter(|candidate| (candidate.time_seconds - time_seconds).abs() <= window_seconds)
        .max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|candidate| candidate.time_seconds)
        .unwrap_or(time_seconds)
        .clamp(0.0, duration_seconds.max(0.0))
}

fn chorus_position_prior(position: f32) -> f32 {
    let main = 1.0 - ((position - 0.56).abs() / 0.42).clamp(0.0, 1.0);
    let late = 0.78 * (1.0 - ((position - 0.73).abs() / 0.20).clamp(0.0, 1.0));
    main.max(late).clamp(0.0, 1.0)
}

fn duration_fit_score(duration_seconds: f64) -> f32 {
    let target = 34.0_f64;
    (1.0 - ((duration_seconds - target).abs() / 24.0).clamp(0.0, 1.0)) as f32
}

fn segment_wholeness_score(
    section_curves: &MusicSectionCurveAnalysis,
    start_seconds: f64,
    end_seconds: f64,
    avg_chorusness: f32,
) -> f32 {
    let length = end_seconds - start_seconds;
    if length <= 4.0 {
        return 0.0;
    }

    let start_boundary = nearest_boundary_score(section_curves, start_seconds, 4.0);
    let end_boundary = nearest_boundary_score(section_curves, end_seconds, 4.0);
    let edge_score = (start_boundary * 0.48 + end_boundary * 0.52).clamp(0.0, 1.0);
    let inside_start = start_seconds + length * 0.22;
    let inside_end = end_seconds - length * 0.22;
    let internal_boundary = if inside_end > inside_start {
        average_curve_value(&section_curves.boundary, inside_start, inside_end)
    } else {
        0.0
    };
    let internal_calm = (1.0 - (internal_boundary / 0.34).clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let chorus_consistency = curve_consistency_score(
        &section_curves.chorusness,
        start_seconds,
        end_seconds,
        avg_chorusness,
    );
    (edge_score * 0.42 + internal_calm * 0.30 + chorus_consistency * 0.28).clamp(0.0, 1.0)
}

fn curve_consistency_score(
    curve: &[MusicCurvePoint],
    start_seconds: f64,
    end_seconds: f64,
    mean: f32,
) -> f32 {
    if curve.is_empty() || end_seconds <= start_seconds {
        return 0.0;
    }
    let mut deviation = 0.0_f32;
    let mut count = 0_u32;
    for point in curve {
        if point.time_seconds >= start_seconds && point.time_seconds <= end_seconds {
            deviation += (point.value - mean).abs();
            count = count.saturating_add(1);
        }
    }
    if count == 0 {
        return 0.0;
    }
    let avg_deviation = deviation / count as f32;
    (1.0 - (avg_deviation / 0.32).clamp(0.0, 1.0)).clamp(0.0, 1.0)
}

fn perceptual_segment_score(
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    start_seconds: f64,
    end_seconds: f64,
    duration_seconds: f64,
) -> f32 {
    if end_seconds <= start_seconds + 4.0 {
        return 0.0;
    }
    let entry = perceptual_cue_score_at(
        frames,
        section_curves,
        None,
        start_seconds,
        start_seconds,
        1.0,
        duration_seconds,
        PerceptualCueRole::MixIn,
    );
    let exit = perceptual_cue_score_at(
        frames,
        section_curves,
        None,
        end_seconds,
        end_seconds,
        1.0,
        duration_seconds,
        PerceptualCueRole::MixOut,
    );
    (entry.total * 0.46 + exit.total * 0.54).clamp(0.0, 1.0)
}

fn highlight_reason(chorusness: f32, repetition: f32, boundary: f32, contrast: f32) -> String {
    let mut reasons = Vec::new();
    if chorusness >= 0.48 {
        reasons.push("chorusness");
    }
    if repetition >= 0.42 {
        reasons.push("repetition");
    }
    if boundary >= 0.28 {
        reasons.push("boundary");
    }
    if contrast >= 0.24 {
        reasons.push("contrast");
    }
    if reasons.is_empty() {
        "energy phrase".to_owned()
    } else {
        reasons.join(" + ")
    }
}

fn estimate_sections(
    frames: &[AnalysisFrame],
    duration_seconds: f64,
    section_curves: &MusicSectionCurveAnalysis,
) -> MusicSectionAnalysis {
    if frames.is_empty() {
        return MusicSectionAnalysis {
            intro: None,
            outro: None,
            highlight_candidates: Vec::new(),
            functional_segments: Vec::new(),
            segment_tempo: Vec::new(),
            structure: MusicStructureAnalysis::default(),
        };
    }

    let silence_threshold = silence_threshold(frames);
    let intro_end = first_sustained_energy_time(frames, silence_threshold)
        .unwrap_or(0.0)
        .clamp(0.0, duration_seconds);
    let outro_start = last_sustained_energy_time(frames, silence_threshold)
        .unwrap_or(duration_seconds)
        .clamp(0.0, duration_seconds);

    let intro = (intro_end > 0.25).then_some(MusicTimeRange {
        start_seconds: 0.0,
        end_seconds: intro_end,
    });
    let outro = (outro_start + 0.25 < duration_seconds).then_some(MusicTimeRange {
        start_seconds: outro_start,
        end_seconds: duration_seconds,
    });
    let raw_highlight_candidates =
        estimate_highlight_candidates(frames, duration_seconds, section_curves);
    let initial_functional_segments = estimate_functional_segments(
        frames,
        duration_seconds,
        intro.as_ref(),
        outro.as_ref(),
        &raw_highlight_candidates,
        section_curves,
    );
    let highlight_candidates = align_highlight_candidates_to_functional_segments(
        raw_highlight_candidates,
        &initial_functional_segments,
        duration_seconds,
    );
    let functional_segments = estimate_functional_segments(
        frames,
        duration_seconds,
        intro.as_ref(),
        outro.as_ref(),
        &highlight_candidates,
        section_curves,
    );
    let highlight_candidates = align_highlight_candidates_to_functional_segments(
        highlight_candidates,
        &functional_segments,
        duration_seconds,
    );

    MusicSectionAnalysis {
        intro,
        outro,
        highlight_candidates,
        functional_segments,
        segment_tempo: Vec::new(),
        structure: section_curves.structure.clone(),
    }
}

fn silence_threshold(frames: &[AnalysisFrame]) -> f32 {
    let mean = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    (mean * 0.12).clamp(0.003, 0.04)
}

fn first_sustained_energy_time(frames: &[AnalysisFrame], threshold: f32) -> Option<f64> {
    frames
        .windows(3)
        .find(|window| window.iter().all(|frame| frame.rms > threshold))
        .and_then(|window| window.first())
        .map(|frame| frame.time_seconds)
}

fn last_sustained_energy_time(frames: &[AnalysisFrame], threshold: f32) -> Option<f64> {
    frames
        .windows(3)
        .rev()
        .find(|window| window.iter().all(|frame| frame.rms > threshold))
        .and_then(|window| window.last())
        .map(|frame| frame.time_seconds)
}

fn estimate_highlight_candidates(
    frames: &[AnalysisFrame],
    duration_seconds: f64,
    section_curves: &MusicSectionCurveAnalysis,
) -> Vec<MusicSectionCandidate> {
    if frames.is_empty() || duration_seconds <= 20.0 {
        return Vec::new();
    }

    let energy_bins = build_energy_bins(frames, duration_seconds);
    if energy_bins.len() < 12 {
        return Vec::new();
    }
    let window_seconds = (duration_seconds * 0.15).clamp(24.0, 46.0);
    let window_bins = (window_seconds / SECTION_CURVE_HOP_SECONDS)
        .round()
        .max(12.0) as usize;
    if energy_bins.len() <= window_bins {
        return Vec::new();
    }
    let start_limit = energy_bins.len().saturating_sub(window_bins);
    let global_energy =
        energy_bins.iter().map(|bin| bin.energy).sum::<f32>() / energy_bins.len().max(1) as f32;
    let global_peak = energy_bins
        .iter()
        .map(|bin| bin.peak)
        .fold(0.0_f32, f32::max)
        .max(0.0001);
    let mut scored = Vec::new();

    for start in 0..=start_limit {
        let end = start + window_bins;
        let raw_start_seconds = energy_bins[start].time_seconds;
        let raw_end_seconds = (energy_bins[end.saturating_sub(1)].time_seconds
            + SECTION_CURVE_HOP_SECONDS)
            .min(duration_seconds);
        if raw_start_seconds < duration_seconds * 0.08 || raw_end_seconds > duration_seconds * 0.96
        {
            continue;
        }

        let avg_energy = energy_bins[start..end]
            .iter()
            .map(|bin| bin.energy)
            .sum::<f32>()
            / window_bins as f32;
        let avg_peak = energy_bins[start..end]
            .iter()
            .map(|bin| bin.peak)
            .sum::<f32>()
            / window_bins as f32;
        let avg_chorusness = average_curve_value(
            &section_curves.chorusness,
            raw_start_seconds,
            raw_end_seconds,
        );
        let repetition = repetition_score_for_window(&energy_bins, start, window_bins);
        let contrast = local_energy_contrast(&energy_bins, start, end, global_energy);
        let boundary_start = nearest_boundary_score(section_curves, raw_start_seconds, 5.0);
        let boundary_end = nearest_boundary_score(section_curves, raw_end_seconds, 5.0);
        let boundary_score = (boundary_start + boundary_end) * 0.5;
        let center =
            ((raw_start_seconds + raw_end_seconds) * 0.5 / duration_seconds.max(1.0)) as f32;
        let position_score = chorus_position_prior(center);
        let density_score = (avg_peak / global_peak).clamp(0.0, 1.0);
        let energy_score = (avg_energy / global_energy.max(0.0001)).clamp(0.0, 2.5) / 2.5;
        let duration_score = duration_fit_score(raw_end_seconds - raw_start_seconds);
        let segment_wholeness = segment_wholeness_score(
            section_curves,
            raw_start_seconds,
            raw_end_seconds,
            avg_chorusness,
        );
        let structural_recurrence = average_curve_value(
            &section_curves.structure.recurrence,
            raw_start_seconds,
            raw_end_seconds,
        );
        let structural_novelty =
            nearest_structure_novelty_score(section_curves, raw_start_seconds, raw_end_seconds);
        let perceptual_score = perceptual_segment_score(
            frames,
            section_curves,
            raw_start_seconds,
            raw_end_seconds,
            duration_seconds,
        );

        let score = avg_chorusness * 0.28
            + repetition * 0.22
            + structural_recurrence * 0.12
            + energy_score * 0.10
            + contrast * 0.09
            + boundary_score * 0.09
            + segment_wholeness * 0.07
            + perceptual_score * 0.05
            + position_score * 0.03
            + structural_novelty * 0.02
            + density_score * 0.01
            + duration_score * 0.01;
        let start_seconds =
            snap_to_nearby_boundary(section_curves, raw_start_seconds, 4.0, duration_seconds);
        let end_seconds =
            snap_to_nearby_boundary(section_curves, raw_end_seconds, 4.0, duration_seconds)
                .max(start_seconds + MIN_HIGHLIGHT_SEGMENT_SECONDS);
        let reason = highlight_reason(avg_chorusness, repetition, boundary_score, contrast);
        let scores = MusicSectionCandidateScores {
            total: score,
            chorusness: avg_chorusness.clamp(0.0, 1.0),
            repetition: repetition.clamp(0.0, 1.0),
            energy: energy_score.clamp(0.0, 1.0),
            contrast: contrast.clamp(0.0, 1.0),
            boundary: boundary_score.clamp(0.0, 1.0),
            position: position_score.clamp(0.0, 1.0),
            density: density_score.clamp(0.0, 1.0),
            duration: duration_score.clamp(0.0, 1.0),
            segment_wholeness: segment_wholeness.clamp(0.0, 1.0),
            perceptual: perceptual_score.clamp(0.0, 1.0),
            structural_recurrence: structural_recurrence.clamp(0.0, 1.0),
            structural_novelty: structural_novelty.clamp(0.0, 1.0),
        };
        scored.push((
            score,
            start_seconds,
            end_seconds.min(duration_seconds),
            reason,
            scores,
        ));
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let best_score = scored.first().map(|item| item.0).unwrap_or(0.0).max(0.0001);
    let mut candidates: Vec<MusicSectionCandidate> = Vec::new();
    for (score, start_seconds, end_seconds, reason, scores) in scored {
        if candidates.len() >= MAX_HIGHLIGHT_CANDIDATES {
            break;
        }
        if end_seconds <= start_seconds + MIN_HIGHLIGHT_SEGMENT_SECONDS {
            continue;
        }
        if candidates.iter().any(|candidate| {
            ranges_overlap(
                start_seconds,
                end_seconds,
                candidate.start_seconds,
                candidate.end_seconds,
            )
        }) {
            continue;
        }
        candidates.push(MusicSectionCandidate {
            start_seconds,
            end_seconds,
            confidence: (score / best_score).clamp(0.0, 1.0),
            reason,
            scores,
        });
    }
    candidates.sort_by(|a, b| {
        a.start_seconds
            .partial_cmp(&b.start_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates
}
fn ranges_overlap(a_start: f64, a_end: f64, b_start: f64, b_end: f64) -> bool {
    let overlap = a_end.min(b_end) - a_start.max(b_start);
    let min_len = (a_end - a_start).min(b_end - b_start).max(0.0);
    overlap > min_len * 0.35
}

fn estimate_functional_segments(
    frames: &[AnalysisFrame],
    duration_seconds: f64,
    intro: Option<&MusicTimeRange>,
    outro: Option<&MusicTimeRange>,
    highlight_candidates: &[MusicSectionCandidate],
    section_curves: &MusicSectionCurveAnalysis,
) -> Vec<MusicFunctionalSegment> {
    if duration_seconds <= 0.0 {
        return Vec::new();
    }

    let mut cuts = vec![0.0_f64, duration_seconds];
    if let Some(intro) = intro {
        cuts.push(intro.end_seconds.clamp(0.0, duration_seconds));
    }
    if let Some(outro) = outro {
        cuts.push(outro.start_seconds.clamp(0.0, duration_seconds));
    }
    for candidate in highlight_candidates {
        cuts.push(candidate.start_seconds.clamp(0.0, duration_seconds));
        cuts.push(candidate.end_seconds.clamp(0.0, duration_seconds));
    }
    for candidate in section_curves.boundary_candidates.iter() {
        if candidate.confidence >= FUNCTIONAL_BOUNDARY_MIN_CONFIDENCE {
            cuts.push(candidate.time_seconds.clamp(0.0, duration_seconds));
        }
    }
    for candidate in section_curves.structure.novelty_boundaries.iter() {
        if candidate.confidence >= FUNCTIONAL_BOUNDARY_MIN_CONFIDENCE {
            cuts.push(candidate.time_seconds.clamp(0.0, duration_seconds));
        }
    }
    add_chorus_curve_transition_cuts(&mut cuts, section_curves, duration_seconds);
    add_energy_valley_cuts(&mut cuts, frames, duration_seconds);
    refine_long_functional_cut_gaps(&mut cuts, frames, section_curves, duration_seconds);

    let deduped = normalized_functional_cuts(cuts, duration_seconds);

    let energy_bins = build_energy_bins(frames, duration_seconds);
    let global_energy =
        frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let global_bin_energy =
        energy_bins.iter().map(|bin| bin.energy).sum::<f32>() / energy_bins.len().max(1) as f32;
    let mut segments = Vec::new();
    for pair in deduped.windows(2) {
        let start = pair[0].clamp(0.0, duration_seconds);
        let end = pair[1].clamp(start, duration_seconds);
        if end <= start + 2.0 {
            continue;
        }
        let role = classify_functional_segment(
            start,
            end,
            duration_seconds,
            intro,
            outro,
            highlight_candidates,
            section_curves,
            frames,
            &energy_bins,
            global_energy,
            global_bin_energy,
        );
        push_merged_functional_segment(&mut segments, role, start, end);
        if segments.len() >= MAX_FUNCTIONAL_SEGMENTS {
            break;
        }
    }
    finalize_functional_segments(segments, duration_seconds, highlight_candidates)
}

fn add_chorus_curve_transition_cuts(
    cuts: &mut Vec<f64>,
    section_curves: &MusicSectionCurveAnalysis,
    duration_seconds: f64,
) {
    let points = &section_curves.chorusness;
    for pair in points.windows(2) {
        let prev = &pair[0];
        let next = &pair[1];
        let crosses_core =
            (prev.value < 0.50 && next.value >= 0.50) || (prev.value >= 0.50 && next.value < 0.50);
        let crosses_soft =
            (prev.value < 0.38 && next.value >= 0.38) || (prev.value >= 0.38 && next.value < 0.38);
        if crosses_core || crosses_soft {
            let time = ((prev.time_seconds + next.time_seconds) * 0.5).clamp(0.0, duration_seconds);
            if time > duration_seconds * 0.06 && time < duration_seconds * 0.96 {
                cuts.push(time);
            }
        }
    }
}

fn add_energy_valley_cuts(cuts: &mut Vec<f64>, frames: &[AnalysisFrame], duration_seconds: f64) {
    if frames.len() < 9 || duration_seconds <= 32.0 {
        return;
    }
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let mut valleys = Vec::new();
    for window in frames.windows(7) {
        let center = &window[3];
        let time = center.time_seconds;
        if time < duration_seconds * 0.08 || time > duration_seconds * 0.94 {
            continue;
        }
        let left = window[..3].iter().map(|frame| frame.rms).sum::<f32>() / 3.0;
        let right = window[4..].iter().map(|frame| frame.rms).sum::<f32>() / 3.0;
        let context = ((left + right) * 0.5).max(global).max(0.0001);
        let dip = (1.0 - (center.rms / context).clamp(0.0, 1.25) / 1.25).clamp(0.0, 1.0);
        let edge = ((right - left).abs() / global.max(0.0001)).clamp(0.0, 2.0) / 2.0;
        let score = dip * 0.72 + edge * 0.28;
        if score >= 0.20 {
            valleys.push((score, time));
        }
    }
    valleys.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut accepted: Vec<f64> = Vec::new();
    for (_score, time) in valleys {
        if accepted.len() >= 12 {
            break;
        }
        if accepted.iter().all(|other| (time - *other).abs() >= 10.0) {
            accepted.push(time);
            cuts.push(time);
        }
    }
}

fn refine_long_functional_cut_gaps(
    cuts: &mut Vec<f64>,
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    duration_seconds: f64,
) {
    for _ in 0..3 {
        let deduped = normalized_functional_cuts(cuts.clone(), duration_seconds);
        let mut added = false;
        for pair in deduped.windows(2) {
            let start = pair[0];
            let end = pair[1];
            if end - start <= LONG_FUNCTIONAL_GAP_SECONDS {
                continue;
            }
            let midpoint = (start + end) * 0.5;
            let split = best_structural_split_time(frames, section_curves, start, end)
                .unwrap_or(midpoint)
                .clamp(start + 8.0, end - 8.0);
            if split.is_finite() {
                cuts.push(split);
                added = true;
            }
        }
        if !added {
            break;
        }
    }
}

fn best_structural_split_time(
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    start: f64,
    end: f64,
) -> Option<f64> {
    if end <= start + 16.0 {
        return None;
    }
    let midpoint = (start + end) * 0.5;
    let search_start = start + (end - start) * 0.25;
    let search_end = end - (end - start) * 0.25;
    let mut best: Option<(f32, f64)> = None;
    for boundary in section_curves.boundary_candidates.iter() {
        if boundary.time_seconds < search_start || boundary.time_seconds > search_end {
            continue;
        }
        let center_bias = 1.0
            - ((boundary.time_seconds - midpoint).abs() / ((end - start) * 0.5)).clamp(0.0, 1.0)
                as f32;
        let score = boundary.confidence * 0.72 + center_bias * 0.28;
        if best.map_or(true, |(best_score, _)| score > best_score) {
            best = Some((score, boundary.time_seconds));
        }
    }
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    for frame in frames.iter() {
        if frame.time_seconds < search_start || frame.time_seconds > search_end {
            continue;
        }
        let safety = vocal_cut_safety_score(frames, frame.time_seconds);
        let calm = (1.0 - (frame.rms / global.max(0.0001)).clamp(0.0, 1.4) / 1.4).clamp(0.0, 1.0);
        let center_bias = 1.0
            - ((frame.time_seconds - midpoint).abs() / ((end - start) * 0.5)).clamp(0.0, 1.0)
                as f32;
        let score = safety * 0.48 + calm * 0.26 + center_bias * 0.26;
        if best.map_or(true, |(best_score, _)| score > best_score) {
            best = Some((score, frame.time_seconds));
        }
    }
    best.map(|(_, time)| time)
}

fn normalized_functional_cuts(mut cuts: Vec<f64>, duration_seconds: f64) -> Vec<f64> {
    cuts.retain(|cut| cut.is_finite());
    cuts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut deduped = Vec::new();
    for cut in cuts {
        let cut = cut.clamp(0.0, duration_seconds);
        if deduped.last().map_or(true, |last: &f64| {
            (cut - *last).abs() >= FUNCTIONAL_CUT_MIN_GAP_SECONDS
        }) {
            deduped.push(cut);
        } else if let Some(last) = deduped.last_mut() {
            *last = (*last + cut) * 0.5;
        }
    }
    if deduped.first().map_or(true, |first| *first > 0.2) {
        deduped.insert(0, 0.0);
    }
    if deduped
        .last()
        .map_or(true, |last| (duration_seconds - *last).abs() > 0.2)
    {
        deduped.push(duration_seconds);
    }
    deduped
}

fn push_merged_functional_segment(
    segments: &mut Vec<MusicFunctionalSegment>,
    next: FunctionalSegmentDraft,
    start_seconds: f64,
    end_seconds: f64,
) {
    if let Some(previous) = segments.last_mut() {
        let combined_length = end_seconds - previous.start_seconds;
        let protected_chorus = matches!(
            &previous.role,
            MusicFunctionalRole::Chorus
                | MusicFunctionalRole::FinalChorus
                | MusicFunctionalRole::PreChorus
        ) || matches!(
            &next.role,
            MusicFunctionalRole::Chorus
                | MusicFunctionalRole::FinalChorus
                | MusicFunctionalRole::PreChorus
        );
        if previous.role == next.role
            && start_seconds - previous.end_seconds <= 3.5
            && (!protected_chorus || combined_length <= MAX_CHORUS_FUNCTIONAL_SEGMENT_SECONDS)
        {
            previous.end_seconds = end_seconds;
            previous.confidence = ((previous.confidence + next.confidence) * 0.5).clamp(0.0, 1.0);
            if !previous.reason.contains(&next.reason) {
                previous.reason = format!("{} / {}", previous.reason, next.reason);
            }
            return;
        }
    }
    segments.push(MusicFunctionalSegment {
        start_seconds,
        end_seconds,
        role: next.role,
        confidence: next.confidence,
        reason: next.reason,
    });
}

fn finalize_functional_segments(
    segments: Vec<MusicFunctionalSegment>,
    duration_seconds: f64,
    highlight_candidates: &[MusicSectionCandidate],
) -> Vec<MusicFunctionalSegment> {
    let mut split_segments = Vec::new();
    for segment in segments {
        if segment.end_seconds <= segment.start_seconds + 1.5 {
            continue;
        }
        if matches!(
            &segment.role,
            MusicFunctionalRole::Chorus | MusicFunctionalRole::FinalChorus
        ) && segment.end_seconds - segment.start_seconds > MAX_CHORUS_FUNCTIONAL_SEGMENT_SECONDS
        {
            let split = (segment.start_seconds + MAX_CHORUS_FUNCTIONAL_SEGMENT_SECONDS)
                .min(segment.end_seconds - 8.0)
                .clamp(segment.start_seconds + 8.0, duration_seconds);
            split_segments.push(MusicFunctionalSegment {
                start_seconds: segment.start_seconds,
                end_seconds: split,
                role: segment.role.clone(),
                confidence: segment.confidence,
                reason: format!("{} · capped", segment.reason),
            });
            if segment.end_seconds > split + 3.0 {
                split_segments.push(MusicFunctionalSegment {
                    start_seconds: split,
                    end_seconds: segment.end_seconds,
                    role: if split / duration_seconds.max(1.0) > 0.66 {
                        MusicFunctionalRole::FinalChorus
                    } else {
                        MusicFunctionalRole::Bridge
                    },
                    confidence: (segment.confidence * 0.82).clamp(0.0, 1.0),
                    reason: "long chorus region split".to_owned(),
                });
            }
        } else {
            split_segments.push(segment);
        }
    }

    let primary_start =
        primary_highlight_candidate(highlight_candidates).map(|candidate| candidate.start_seconds);
    let mut normalized = Vec::new();
    for mut segment in split_segments {
        if matches!(&segment.role, MusicFunctionalRole::PreChorus) {
            let valid_pre = primary_start.is_some_and(|start| {
                let distance = start - segment.end_seconds;
                (-1.5..=20.0).contains(&distance)
            });
            if !valid_pre {
                segment.role = MusicFunctionalRole::Verse;
                segment.confidence = (segment.confidence * 0.84).clamp(0.0, 1.0);
                segment.reason = "pre-chorus candidate outside selected highlight".to_owned();
            }
        }
        push_normalized_functional_segment(&mut normalized, segment);
    }

    let mut compact: Vec<MusicFunctionalSegment> = Vec::new();
    for segment in normalized {
        if segment.end_seconds <= segment.start_seconds + 2.5 {
            continue;
        }
        if let Some(previous) = compact.last_mut() {
            let gap = segment.start_seconds - previous.end_seconds;
            if previous.role == segment.role && gap.abs() <= 2.0 {
                previous.end_seconds = segment.end_seconds;
                previous.confidence =
                    ((previous.confidence + segment.confidence) * 0.5).clamp(0.0, 1.0);
                continue;
            }
        }
        compact.push(segment);
    }
    let mut section_graph =
        smooth_functional_segment_sequence(compact, duration_seconds, highlight_candidates);
    section_graph.truncate(MAX_FUNCTIONAL_SEGMENTS);
    section_graph
}

fn smooth_functional_segment_sequence(
    mut segments: Vec<MusicFunctionalSegment>,
    duration_seconds: f64,
    highlight_candidates: &[MusicSectionCandidate],
) -> Vec<MusicFunctionalSegment> {
    if segments.is_empty() {
        return segments;
    }

    // Sequence-level prior: sections just before a strong chorus often function as
    // pre-chorus, and the last chorus-like section in the last third is usually final.
    for index in 0..segments.len() {
        if matches!(&segments[index].role, MusicFunctionalRole::Chorus)
            && segments[index].start_seconds >= duration_seconds * 0.66
        {
            segments[index].role = MusicFunctionalRole::FinalChorus;
            segments[index].reason = format!("{} · late chorus", segments[index].reason);
        }
    }

    for index in 0..segments.len().saturating_sub(1) {
        let next_is_chorus = matches!(
            &segments[index + 1].role,
            MusicFunctionalRole::Chorus | MusicFunctionalRole::FinalChorus
        );
        let length = segments[index].end_seconds - segments[index].start_seconds;
        if next_is_chorus
            && matches!(
                &segments[index].role,
                MusicFunctionalRole::Verse | MusicFunctionalRole::Bridge
            )
            && (5.0..=24.0).contains(&length)
            && segments[index + 1].start_seconds - segments[index].end_seconds <= 3.0
        {
            segments[index].role = MusicFunctionalRole::PreChorus;
            segments[index].confidence = (segments[index].confidence * 0.92 + 0.08).clamp(0.0, 1.0);
            segments[index].reason = "sequence prior before chorus".to_owned();
        }
    }

    // If a highlight lands inside a non-chorus section, only relabel when the
    // overlap is strong and the candidate itself has real chorus/repetition evidence.
    // This keeps highlight and structure related without letting highlight paint
    // huge areas as chorus.
    for candidate in highlight_candidates
        .iter()
        .filter(|candidate| candidate.confidence >= 0.82)
    {
        if candidate.scores.chorusness < 0.56 && candidate.scores.repetition < 0.54 {
            continue;
        }
        for segment in segments.iter_mut() {
            let overlap = segment_overlap_ratio(
                segment.start_seconds,
                segment.end_seconds,
                candidate.start_seconds,
                candidate.end_seconds,
            );
            let segment_length = segment.end_seconds - segment.start_seconds;
            if overlap >= 0.55
                && segment_length <= MAX_CHORUS_FUNCTIONAL_SEGMENT_SECONDS + 6.0
                && matches!(
                    &segment.role,
                    MusicFunctionalRole::Verse | MusicFunctionalRole::Bridge
                )
            {
                segment.role = if segment.start_seconds >= duration_seconds * 0.66 {
                    MusicFunctionalRole::FinalChorus
                } else {
                    MusicFunctionalRole::Chorus
                };
                segment.confidence =
                    (segment.confidence * 0.65 + candidate.confidence * 0.35).clamp(0.0, 1.0);
                segment.reason = "highlight-supported chorus section".to_owned();
            }
        }
    }

    compact_adjacent_functional_segments(segments)
}

fn reconcile_functional_segments_with_highlights(
    segments: Vec<MusicFunctionalSegment>,
    duration_seconds: f64,
    highlight_candidates: &[MusicSectionCandidate],
) -> Vec<MusicFunctionalSegment> {
    if segments.is_empty() || highlight_candidates.is_empty() {
        return segments;
    }

    let mut output = segments;
    let mut strong_candidates: Vec<&MusicSectionCandidate> = highlight_candidates
        .iter()
        .filter(|candidate| {
            candidate.confidence >= HIGHLIGHT_SEGMENT_ANCHOR_MIN_CONFIDENCE
                && candidate.end_seconds > candidate.start_seconds + MIN_HIGHLIGHT_SEGMENT_SECONDS
        })
        .collect();
    strong_candidates.sort_by(|a, b| {
        a.start_seconds
            .partial_cmp(&b.start_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for candidate in strong_candidates {
        output = anchor_functional_segment_to_highlight(output, candidate, duration_seconds);
    }
    compact_adjacent_functional_segments(output)
}

fn anchor_functional_segment_to_highlight(
    segments: Vec<MusicFunctionalSegment>,
    candidate: &MusicSectionCandidate,
    duration_seconds: f64,
) -> Vec<MusicFunctionalSegment> {
    let start = candidate.start_seconds.clamp(0.0, duration_seconds);
    let end = candidate.end_seconds.clamp(start, duration_seconds);
    if end <= start + MIN_HIGHLIGHT_SEGMENT_SECONDS {
        return segments;
    }

    let best_overlap = segments
        .iter()
        .map(|segment| {
            segment_overlap_ratio(segment.start_seconds, segment.end_seconds, start, end)
        })
        .fold(0.0_f32, f32::max);
    if best_overlap < HIGHLIGHT_SEGMENT_OVERLAP_MIN {
        return segments;
    }

    let mut anchored = Vec::new();
    let mut inserted = false;
    for segment in segments {
        if segment.end_seconds <= start + 0.20 || segment.start_seconds >= end - 0.20 {
            anchored.push(segment);
            continue;
        }

        if segment.start_seconds < start - 1.5 {
            anchored.push(MusicFunctionalSegment {
                start_seconds: segment.start_seconds,
                end_seconds: start,
                role: segment.role.clone(),
                confidence: (segment.confidence * 0.88).clamp(0.0, 1.0),
                reason: format!("{} · before stage highlight", segment.reason),
            });
        }

        if !inserted {
            let role = if start / duration_seconds.max(1.0) >= 0.68 {
                MusicFunctionalRole::FinalChorus
            } else {
                MusicFunctionalRole::Chorus
            };
            anchored.push(MusicFunctionalSegment {
                start_seconds: start,
                end_seconds: end,
                role,
                confidence: (0.58
                    + candidate.confidence * 0.30
                    + candidate.scores.chorusness * 0.08)
                    .clamp(0.0, 1.0),
                reason: format!("stage highlight anchor · {}", candidate.reason),
            });
            inserted = true;
        }

        if segment.end_seconds > end + 1.5 {
            anchored.push(MusicFunctionalSegment {
                start_seconds: end,
                end_seconds: segment.end_seconds,
                role: segment.role,
                confidence: (segment.confidence * 0.88).clamp(0.0, 1.0),
                reason: format!("{} · after stage highlight", segment.reason),
            });
        }
    }
    anchored
}

fn compact_adjacent_functional_segments(
    segments: Vec<MusicFunctionalSegment>,
) -> Vec<MusicFunctionalSegment> {
    let mut compact = Vec::new();
    for segment in segments {
        if segment.end_seconds <= segment.start_seconds + 2.0 {
            continue;
        }
        push_normalized_functional_segment(&mut compact, segment);
    }
    compact
}

fn segment_overlap_ratio(a_start: f64, a_end: f64, b_start: f64, b_end: f64) -> f32 {
    let overlap = a_end.min(b_end) - a_start.max(b_start);
    let denom = (a_end - a_start).min(b_end - b_start).max(0.0001);
    (overlap.max(0.0) / denom).clamp(0.0, 1.0) as f32
}

fn push_normalized_functional_segment(
    segments: &mut Vec<MusicFunctionalSegment>,
    mut segment: MusicFunctionalSegment,
) {
    if let Some(previous) = segments.last_mut() {
        if segment.start_seconds < previous.end_seconds {
            segment.start_seconds = previous.end_seconds;
        }
        if segment.end_seconds <= segment.start_seconds + 1.5 {
            return;
        }
        if previous.role == segment.role && segment.start_seconds - previous.end_seconds <= 2.0 {
            previous.end_seconds = segment.end_seconds;
            previous.confidence =
                ((previous.confidence + segment.confidence) * 0.5).clamp(0.0, 1.0);
            return;
        }
    }
    segments.push(segment);
}

#[derive(Clone, Debug)]
struct FunctionalSegmentDraft {
    role: MusicFunctionalRole,
    confidence: f32,
    reason: String,
}

#[derive(Clone, Debug)]
struct FunctionalEvidence {
    chorusness: f32,
    repetition: f32,
    structural_recurrence: f32,
    structural_novelty: f32,
    energy: f32,
    contrast: f32,
    boundary: f32,
    position: f32,
    duration: f32,
    segment_wholeness: f32,
    highlight_overlap: f32,
}

impl FunctionalEvidence {
    fn chorus_score(&self) -> f32 {
        (self.chorusness * 0.34
            + self.repetition * 0.23
            + self.structural_recurrence * 0.13
            + self.energy * 0.10
            + self.contrast * 0.08
            + self.boundary.max(self.structural_novelty) * 0.07
            + self.segment_wholeness * 0.08
            + self.position * 0.05
            + self.duration * 0.03
            + self.highlight_overlap * 0.01)
            .clamp(0.0, 1.0)
    }
}

fn classify_functional_segment(
    start: f64,
    end: f64,
    duration_seconds: f64,
    intro: Option<&MusicTimeRange>,
    outro: Option<&MusicTimeRange>,
    highlight_candidates: &[MusicSectionCandidate],
    section_curves: &MusicSectionCurveAnalysis,
    frames: &[AnalysisFrame],
    energy_bins: &[EnergyBin],
    global_energy: f32,
    global_bin_energy: f32,
) -> FunctionalSegmentDraft {
    let length = end - start;
    let mid = (start + end) * 0.5;
    let position = (mid / duration_seconds.max(1.0)).clamp(0.0, 1.0) as f32;
    let avg_energy = average_frame_rms(frames, start, end);
    let relative_energy = (avg_energy / global_energy.max(0.0001)).clamp(0.0, 2.0) / 2.0;
    let avg_chorusness = average_curve_value(&section_curves.chorusness, start, end);
    let boundary_start = nearest_boundary_score(section_curves, start, 3.0);
    let boundary_end = nearest_boundary_score(section_curves, end, 3.0);
    let boundary = boundary_start.max(boundary_end);
    let repetition = section_repetition_score(energy_bins, start, end);
    let contrast = section_energy_contrast(energy_bins, start, end, global_bin_energy);
    let duration = duration_fit_score(length);
    let segment_wholeness = segment_wholeness_score(section_curves, start, end, avg_chorusness);
    let structural_recurrence =
        average_curve_value(&section_curves.structure.recurrence, start, end);
    let structural_novelty = nearest_structure_novelty_score(section_curves, start, end);
    let primary = primary_highlight_candidate(highlight_candidates);
    let primary_overlap = primary
        .map(|candidate| candidate_overlap_ratio(candidate, start, end))
        .unwrap_or(0.0);
    let strong_highlight_overlap =
        highlight_overlap_ratio_by_confidence(highlight_candidates, start, end, 0.88);
    let highlight_overlap = primary_overlap.max(strong_highlight_overlap);
    let next_strong_highlight_distance = next_strong_highlight_distance(highlight_candidates, end);
    let evidence = FunctionalEvidence {
        chorusness: avg_chorusness.clamp(0.0, 1.0),
        repetition: repetition.clamp(0.0, 1.0),
        structural_recurrence: structural_recurrence.clamp(0.0, 1.0),
        structural_novelty: structural_novelty.clamp(0.0, 1.0),
        energy: relative_energy.clamp(0.0, 1.0),
        contrast: contrast.clamp(0.0, 1.0),
        boundary: boundary.clamp(0.0, 1.0),
        position: chorus_position_prior(position),
        duration,
        segment_wholeness: segment_wholeness.clamp(0.0, 1.0),
        highlight_overlap: highlight_overlap.clamp(0.0, 1.0),
    };
    let chorus_score = evidence.chorus_score();

    if relative_energy < 0.08 && length >= 3.0 {
        return FunctionalSegmentDraft {
            role: MusicFunctionalRole::Silence,
            confidence: (0.72 + (0.08 - relative_energy) * 2.0).clamp(0.0, 1.0),
            reason: "low energy gap".to_owned(),
        };
    }
    if intro.is_some_and(|intro| end <= intro.end_seconds + 2.5) || start < duration_seconds * 0.06
    {
        return FunctionalSegmentDraft {
            role: MusicFunctionalRole::Intro,
            confidence: (0.62 + boundary * 0.24).clamp(0.0, 1.0),
            reason: "intro boundary".to_owned(),
        };
    }
    if outro.is_some_and(|outro| start >= outro.start_seconds - 2.5)
        || start > duration_seconds * 0.90
    {
        return FunctionalSegmentDraft {
            role: MusicFunctionalRole::Outro,
            confidence: (0.60 + boundary * 0.24).clamp(0.0, 1.0),
            reason: "outro boundary".to_owned(),
        };
    }

    // Functional sections should be a song-structure skeleton first. Highlight is
    // only supporting evidence here; it must not paint a weak, very long area as
    // chorus by itself. This follows the boundary -> section -> label order used
    // by stronger structure analyzers.
    let strong_structural_evidence =
        repetition >= 0.48 || structural_recurrence >= 0.55 || segment_wholeness >= 0.46;
    let confident_chorus = chorus_score >= 0.51
        && (avg_chorusness >= 0.42 || strong_structural_evidence || highlight_overlap >= 0.44)
        && segment_wholeness >= 0.30
        && length <= MAX_CHORUS_FUNCTIONAL_SEGMENT_SECONDS + 8.0;
    let possible_chorus = chorus_score >= 0.44
        && (avg_chorusness >= 0.50 || repetition >= 0.56)
        && segment_wholeness >= 0.36
        && relative_energy >= 0.16
        && length <= MAX_CHORUS_FUNCTIONAL_SEGMENT_SECONDS;

    if confident_chorus || possible_chorus {
        let late = position >= 0.70 || start >= duration_seconds * 0.68;
        return FunctionalSegmentDraft {
            role: if late {
                MusicFunctionalRole::FinalChorus
            } else {
                MusicFunctionalRole::Chorus
            },
            confidence: (0.44 + chorus_score * 0.42 + boundary * 0.08 + highlight_overlap * 0.06)
                .clamp(0.0, 1.0),
            reason: if late {
                format!(
                    "section graph final chorus · c{:.2} r{:.2} rec{:.2} w{:.2}",
                    avg_chorusness, repetition, structural_recurrence, segment_wholeness
                )
            } else {
                format!(
                    "section graph chorus · c{:.2} r{:.2} rec{:.2} w{:.2}",
                    avg_chorusness, repetition, structural_recurrence, segment_wholeness
                )
            },
        };
    }

    if next_strong_highlight_distance.is_finite()
        && (0.0..=18.0).contains(&next_strong_highlight_distance)
        && chorus_score >= 0.30
        && segment_wholeness >= 0.24
        && relative_energy >= 0.16
        && length <= 24.0
    {
        return FunctionalSegmentDraft {
            role: MusicFunctionalRole::PreChorus,
            confidence: (0.42
                + (1.0 - (next_strong_highlight_distance / 18.0) as f32) * 0.24
                + chorus_score * 0.20
                + boundary * 0.10)
                .clamp(0.0, 1.0),
            reason: "ramp before chorus-like section".to_owned(),
        };
    }

    if position > 0.55
        && avg_chorusness < 0.42
        && repetition < 0.46
        && relative_energy >= 0.22
        && length >= 8.0
    {
        return FunctionalSegmentDraft {
            role: MusicFunctionalRole::Bridge,
            confidence: (0.44 + boundary * 0.20 + contrast * 0.16 + relative_energy * 0.10)
                .clamp(0.0, 1.0),
            reason: "late contrast section".to_owned(),
        };
    }
    if avg_chorusness < 0.28 && repetition < 0.34 && relative_energy >= 0.42 && length <= 18.0 {
        return FunctionalSegmentDraft {
            role: MusicFunctionalRole::Instrumental,
            confidence: (0.40 + relative_energy * 0.24 + contrast * 0.16).clamp(0.0, 1.0),
            reason: "non-vocal hook energy".to_owned(),
        };
    }
    FunctionalSegmentDraft {
        role: MusicFunctionalRole::Verse,
        confidence: (0.46
            + boundary * 0.14
            + (1.0 - chorus_score).clamp(0.0, 1.0) * 0.14
            + relative_energy * 0.08)
            .clamp(0.0, 1.0),
        reason: "verse-like section".to_owned(),
    }
}

fn section_repetition_score(bins: &[EnergyBin], start_seconds: f64, end_seconds: f64) -> f32 {
    if bins.len() < 24 || end_seconds <= start_seconds + 4.0 {
        return 0.0;
    }
    let start = (start_seconds / SECTION_CURVE_HOP_SECONDS).floor().max(0.0) as usize;
    let end = (end_seconds / SECTION_CURVE_HOP_SECONDS)
        .ceil()
        .max(start as f64 + 1.0) as usize;
    let window = (end.saturating_sub(start)).clamp(10, 40);
    let centered_start = if end.saturating_sub(start) > window {
        start + (end - start - window) / 2
    } else {
        start
    };
    repetition_score_for_window(
        bins,
        centered_start.min(bins.len().saturating_sub(1)),
        window,
    )
}

fn section_energy_contrast(
    bins: &[EnergyBin],
    start_seconds: f64,
    end_seconds: f64,
    global_energy: f32,
) -> f32 {
    if bins.is_empty() || end_seconds <= start_seconds {
        return 0.0;
    }
    let start = (start_seconds / SECTION_CURVE_HOP_SECONDS)
        .floor()
        .clamp(0.0, bins.len().saturating_sub(1) as f64) as usize;
    let end = (end_seconds / SECTION_CURVE_HOP_SECONDS)
        .ceil()
        .clamp((start + 1) as f64, bins.len() as f64) as usize;
    local_energy_contrast(bins, start, end, global_energy)
}

fn next_strong_highlight_distance(
    highlight_candidates: &[MusicSectionCandidate],
    end_seconds: f64,
) -> f64 {
    highlight_candidates
        .iter()
        .filter(|candidate| {
            candidate.confidence >= 0.70 && candidate.start_seconds >= end_seconds - 1.0
        })
        .map(|candidate| candidate.start_seconds - end_seconds)
        .filter(|distance| *distance >= -1.5)
        .fold(f64::INFINITY, f64::min)
}

fn align_highlight_candidates_to_functional_segments(
    candidates: Vec<MusicSectionCandidate>,
    functional_segments: &[MusicFunctionalSegment],
    duration_seconds: f64,
) -> Vec<MusicSectionCandidate> {
    if candidates.is_empty() || functional_segments.is_empty() {
        return candidates;
    }

    let mut aligned = Vec::new();
    for mut candidate in candidates {
        let best_chorus = functional_segments
            .iter()
            .filter(|segment| {
                matches!(
                    &segment.role,
                    MusicFunctionalRole::Chorus | MusicFunctionalRole::FinalChorus
                )
            })
            .map(|segment| {
                let overlap = segment_overlap_ratio(
                    candidate.start_seconds,
                    candidate.end_seconds,
                    segment.start_seconds,
                    segment.end_seconds,
                );
                (overlap, segment)
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((overlap, segment)) = best_chorus {
            if overlap >= 0.30 {
                let clipped_start = candidate
                    .start_seconds
                    .max(segment.start_seconds)
                    .clamp(0.0, duration_seconds);
                let clipped_end = candidate
                    .end_seconds
                    .min(segment.end_seconds)
                    .clamp(clipped_start, duration_seconds);
                if clipped_end >= clipped_start + MIN_HIGHLIGHT_SEGMENT_SECONDS {
                    candidate.start_seconds = clipped_start;
                    candidate.end_seconds = clipped_end;
                    candidate.reason = format!("{} · functional chorus", candidate.reason);
                }
                let boost = if matches!(&segment.role, MusicFunctionalRole::FinalChorus) {
                    0.08
                } else {
                    0.06
                };
                candidate.confidence =
                    (candidate.confidence + boost + overlap * 0.08).clamp(0.0, 1.0);
            } else if candidate.scores.chorusness < 0.58 || candidate.scores.repetition < 0.52 {
                // A highlight that does not land near any chorus-like structural section can
                // still be an interesting moment, but it should not dominate Stage Pick.
                candidate.confidence = (candidate.confidence * 0.82).clamp(0.0, 1.0);
                candidate.reason = format!("{} · outside functional chorus", candidate.reason);
            }
        }
        aligned.push(candidate);
    }

    aligned.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    aligned.truncate(MAX_HIGHLIGHT_CANDIDATES);
    aligned.sort_by(|a, b| {
        a.start_seconds
            .partial_cmp(&b.start_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    aligned
}

fn primary_highlight_candidate(
    highlight_candidates: &[MusicSectionCandidate],
) -> Option<&MusicSectionCandidate> {
    highlight_candidates.iter().max_by(|a, b| {
        a.confidence
            .partial_cmp(&b.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn candidate_overlap_ratio(candidate: &MusicSectionCandidate, start: f64, end: f64) -> f32 {
    let length = (end - start).max(0.0001);
    let overlap = candidate.end_seconds.min(end) - candidate.start_seconds.max(start);
    (overlap.max(0.0) / length).clamp(0.0, 1.0) as f32
}

fn highlight_overlap_ratio_by_confidence(
    highlight_candidates: &[MusicSectionCandidate],
    start: f64,
    end: f64,
    min_confidence: f32,
) -> f32 {
    highlight_candidates
        .iter()
        .filter(|candidate| candidate.confidence >= min_confidence)
        .map(|candidate| candidate_overlap_ratio(candidate, start, end))
        .fold(0.0_f32, f32::max)
}

fn average_frame_rms(frames: &[AnalysisFrame], start: f64, end: f64) -> f32 {
    if frames.is_empty() || end <= start {
        return 0.0;
    }
    let mut sum = 0.0_f32;
    let mut count = 0_u32;
    for frame in frames {
        if frame.time_seconds >= start && frame.time_seconds <= end {
            sum += frame.rms;
            count = count.saturating_add(1);
        }
    }
    if count == 0 { 0.0 } else { sum / count as f32 }
}

#[derive(Clone, Debug)]
struct PerceptualCuePoint {
    time_seconds: f64,
    total: f32,
    vocal_safety: f32,
    phrase_closure: f32,
    masking_opportunity: f32,
    attention_safety: f32,
    expectation_safety: f32,
    phrase_grid_fit: f32,
    emotional_continuity: f32,
    vocal_handoff_score: f32,
}

#[derive(Clone, Copy, Debug)]
enum PerceptualCueRole {
    MixIn,
    MixOut,
}

fn phrase_safe_time_near(
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    tempo: &MusicTempoAnalysis,
    target_seconds: f64,
    search_window_seconds: f64,
    duration_seconds: f64,
    role: PerceptualCueRole,
) -> PerceptualCuePoint {
    let mut best = perceptual_cue_score_at(
        frames,
        section_curves,
        Some(tempo),
        target_seconds.clamp(0.0, duration_seconds),
        target_seconds,
        search_window_seconds,
        duration_seconds,
        role,
    );
    let steps = 24;
    for step in 0..=steps {
        let ratio = step as f64 / steps as f64;
        let time = (target_seconds - search_window_seconds + search_window_seconds * 2.0 * ratio)
            .clamp(0.0, duration_seconds);
        let candidate = perceptual_cue_score_at(
            frames,
            section_curves,
            Some(tempo),
            time,
            target_seconds,
            search_window_seconds,
            duration_seconds,
            role,
        );
        if candidate.total > best.total {
            best = candidate;
        }
    }
    best
}

fn perceptual_cue_score_at(
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    tempo: Option<&MusicTempoAnalysis>,
    time_seconds: f64,
    target_seconds: f64,
    search_window_seconds: f64,
    duration_seconds: f64,
    role: PerceptualCueRole,
) -> PerceptualCuePoint {
    let time_seconds = time_seconds.clamp(0.0, duration_seconds.max(0.0));
    let vocal_safety = vocal_cut_safety_score(frames, time_seconds);
    let boundary = nearest_boundary_score(section_curves, time_seconds, 2.4);
    let distance = ((time_seconds - target_seconds).abs() / search_window_seconds.max(0.01))
        .clamp(0.0, 1.0) as f32;
    let phrase_proximity = 1.0 - distance;
    let masking_opportunity = masking_opportunity_score(frames, time_seconds);
    let attention_safety = attention_safety_score(frames, time_seconds);
    let expectation_safety = expectation_safety_score(frames, time_seconds, boundary);
    let phrase_grid_fit = tempo
        .map(|tempo| stage_phrase_alignment_score(time_seconds, tempo))
        .unwrap_or(phrase_proximity);
    let emotional_continuity = emotional_continuity_score(frames, time_seconds);
    let breath_pocket = breathing_pocket_score(frames, time_seconds);
    let vocal_handoff_score = vocal_relay_handoff_score(frames, time_seconds, role);
    let phrase_closure = (boundary * 0.46
        + phrase_grid_fit * 0.24
        + phrase_proximity * 0.14
        + expectation_safety * 0.16)
        .clamp(0.0, 1.0);
    let distance_penalty = distance * 0.12;
    let total = (vocal_safety * 0.23
        + phrase_closure * 0.24
        + masking_opportunity * 0.16
        + attention_safety * 0.14
        + expectation_safety * 0.08
        + emotional_continuity * 0.08
        + vocal_handoff_score * 0.06
        + breath_pocket * 0.04
        + phrase_grid_fit * 0.03
        - distance_penalty)
        .clamp(0.0, 1.0);

    PerceptualCuePoint {
        time_seconds,
        total,
        vocal_safety,
        phrase_closure,
        masking_opportunity,
        attention_safety,
        expectation_safety,
        phrase_grid_fit,
        emotional_continuity,
        vocal_handoff_score,
    }
}

fn stage_phrase_alignment_score(time_seconds: f64, tempo: &MusicTempoAnalysis) -> f32 {
    if !time_seconds.is_finite() {
        return 0.0;
    };
    let beat_position = if let (Some(beat_grid), Some(downbeat_grid)) =
        (tempo.beat_grid.as_ref(), tempo.downbeat_grid.as_ref())
    {
        if beat_grid.interval_seconds > 0.0 && downbeat_grid.confidence >= 0.18 {
            (time_seconds - downbeat_grid.first_downbeat_seconds) / beat_grid.interval_seconds
        } else if beat_grid.interval_seconds > 0.0 {
            (time_seconds - beat_grid.first_beat_seconds) / beat_grid.interval_seconds
        } else {
            return 0.0;
        }
    } else if let Some(grid) = tempo.beat_grid.as_ref() {
        if grid.interval_seconds <= 0.0 {
            return 0.0;
        }
        (time_seconds - grid.first_beat_seconds) / grid.interval_seconds
    } else {
        return 0.0;
    };
    if !beat_position.is_finite() {
        return 0.0;
    }

    let bar = beat_grid_mod_score(beat_position, 4.0, 0.72);
    let two_bar = beat_grid_mod_score(beat_position, 8.0, 0.92);
    let four_bar = beat_grid_mod_score(beat_position, 16.0, 1.12);
    (bar * 0.46 + two_bar * 0.30 + four_bar * 0.24).clamp(0.0, 1.0)
}

fn beat_grid_mod_score(beat_position: f64, period_beats: f64, soft_window_beats: f64) -> f32 {
    if period_beats <= 0.0 || soft_window_beats <= 0.0 || !beat_position.is_finite() {
        return 0.0;
    }
    let phase = beat_position.rem_euclid(period_beats);
    let distance = phase.min(period_beats - phase);
    (1.0 - (distance / soft_window_beats).clamp(0.0, 1.0)) as f32
}

fn breathing_pocket_score(frames: &[AnalysisFrame], time_seconds: f64) -> f32 {
    if frames.is_empty() || !time_seconds.is_finite() {
        return 0.0;
    }
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let center = average_frame_rms(frames, time_seconds - 0.18, time_seconds + 0.18);
    let before = average_frame_rms(frames, time_seconds - 0.95, time_seconds - 0.30);
    let after = average_frame_rms(frames, time_seconds + 0.30, time_seconds + 0.95);
    let context = ((before + after) * 0.5).max(global).max(0.0001);
    let pocket = (1.0 - (center / context).clamp(0.0, 1.15) / 1.15).clamp(0.0, 1.0);
    let support = (context / global.max(0.0001)).clamp(0.0, 1.55) / 1.55;
    let symmetry = (1.0 - ((before - after).abs() / context).clamp(0.0, 1.0)).clamp(0.0, 1.0);
    (pocket * 0.54 + support * 0.24 + symmetry * 0.22).clamp(0.0, 1.0)
}

fn emotional_continuity_score(frames: &[AnalysisFrame], time_seconds: f64) -> f32 {
    if frames.is_empty() || !time_seconds.is_finite() {
        return 0.0;
    }
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let early = average_frame_rms(frames, time_seconds - 4.20, time_seconds - 2.10);
    let before = average_frame_rms(frames, time_seconds - 1.55, time_seconds - 0.25);
    let after = average_frame_rms(frames, time_seconds + 0.30, time_seconds + 2.20);
    let floor = global.max(0.0001);
    let pre_rise = ((before - early) / floor).clamp(0.0, 1.6) / 1.6;
    let release = ((before - after) / floor).clamp(0.0, 1.5) / 1.5;
    let harsh_lift = ((after - before) / floor).clamp(0.0, 1.8) / 1.8;
    let stable_after =
        (1.0 - ((after - before).abs() / floor).clamp(0.0, 1.8) / 1.8).clamp(0.0, 1.0);
    (0.48 + release * 0.24 + stable_after * 0.18 - pre_rise * 0.18 - harsh_lift * 0.22)
        .clamp(0.0, 1.0)
}

fn masking_opportunity_score(frames: &[AnalysisFrame], time_seconds: f64) -> f32 {
    if frames.is_empty() || !time_seconds.is_finite() {
        return 0.0;
    }
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let context = average_frame_rms(frames, time_seconds - 1.10, time_seconds + 1.10);
    let center = average_frame_rms(frames, time_seconds - 0.22, time_seconds + 0.22);
    let density = (context / global.max(0.0001)).clamp(0.0, 1.7) / 1.7;
    let texture = (1.0
        - ((center - context).abs() / context.max(global).max(0.0001)).clamp(0.0, 1.0))
    .clamp(0.0, 1.0);
    (density * 0.64 + texture * 0.36).clamp(0.0, 1.0)
}

fn attention_safety_score(frames: &[AnalysisFrame], time_seconds: f64) -> f32 {
    if frames.is_empty() || !time_seconds.is_finite() {
        return 0.0;
    }
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let center = average_frame_rms(frames, time_seconds - 0.25, time_seconds + 0.25);
    let before = average_frame_rms(frames, time_seconds - 1.20, time_seconds - 0.35);
    let after = average_frame_rms(frames, time_seconds + 0.35, time_seconds + 1.20);
    let context = ((before + after) * 0.5).max(global).max(0.0001);
    let salience = ((center / context).clamp(0.0, 1.8) / 1.8).clamp(0.0, 1.0);
    let edge = ((after - before).abs() / global.max(0.0001)).clamp(0.0, 2.0) / 2.0;
    let cut_risk = (salience * 0.58 + edge * 0.42).clamp(0.0, 1.0);
    (1.0 - cut_risk).clamp(0.0, 1.0)
}

fn expectation_safety_score(frames: &[AnalysisFrame], time_seconds: f64, boundary: f32) -> f32 {
    if frames.is_empty() || !time_seconds.is_finite() {
        return 0.0;
    }
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let before = average_frame_rms(frames, time_seconds - 2.40, time_seconds - 0.45);
    let after = average_frame_rms(frames, time_seconds + 0.45, time_seconds + 2.40);
    let rising_tension = ((after - before) / global.max(0.0001)).clamp(0.0, 1.5) / 1.5;
    let release = ((before - after) / global.max(0.0001)).clamp(0.0, 1.5) / 1.5;
    (0.62 - rising_tension * 0.34 + release * 0.18 + boundary * 0.30).clamp(0.0, 1.0)
}

fn vocal_relay_handoff_score(
    frames: &[AnalysisFrame],
    time_seconds: f64,
    role: PerceptualCueRole,
) -> f32 {
    if frames.is_empty() || !time_seconds.is_finite() {
        return 0.0;
    }
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let global = global.max(0.0001);
    let before_far = average_frame_rms(frames, time_seconds - 2.20, time_seconds - 1.05);
    let before_near = average_frame_rms(frames, time_seconds - 0.78, time_seconds - 0.18);
    let center = average_frame_rms(frames, time_seconds - 0.16, time_seconds + 0.16);
    let after_near = average_frame_rms(frames, time_seconds + 0.18, time_seconds + 0.78);
    let after_far = average_frame_rms(frames, time_seconds + 1.05, time_seconds + 2.20);
    let pocket = (1.0 - (center / before_near.max(after_near).max(global)).clamp(0.0, 1.25) / 1.25)
        .clamp(0.0, 1.0);
    let calm_center = (1.0 - (center / global.max(0.0001)).clamp(0.0, 1.8) / 1.8).clamp(0.0, 1.0);
    match role {
        PerceptualCueRole::MixOut => {
            // A good outgoing handoff feels like the singer has just handed the
            // phrase away: there was activity before the cue, then a short breath
            // or release where B can take over.
            let tail_release = ((before_far.max(before_near) - after_near.min(after_far)) / global)
                .clamp(0.0, 1.6)
                / 1.6;
            let not_mid_syllable = (1.0
                - (before_near - after_near).abs() / global.max(0.0001) * 0.18)
                .clamp(0.0, 1.0);
            (pocket * 0.42 + tail_release * 0.34 + calm_center * 0.14 + not_mid_syllable * 0.10)
                .clamp(0.0, 1.0)
        }
        PerceptualCueRole::MixIn => {
            // A good incoming handoff starts just before the next vocal/lead hook
            // has room to enter.  Do not require actual source separation; use the
            // local energy contour as a lightweight proxy for phrase pickup.
            let entry_lift = ((after_near.max(after_far) - before_near.min(before_far)) / global)
                .clamp(0.0, 1.6)
                / 1.6;
            let pre_breath = (1.0 - (before_near / after_near.max(global)).clamp(0.0, 1.3) / 1.3)
                .clamp(0.0, 1.0);
            (pocket * 0.30 + entry_lift * 0.38 + pre_breath * 0.22 + calm_center * 0.10)
                .clamp(0.0, 1.0)
        }
    }
}

fn vocal_cut_safety_score(frames: &[AnalysisFrame], time_seconds: f64) -> f32 {
    if frames.is_empty() || !time_seconds.is_finite() {
        return 0.0;
    }
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let center = average_frame_rms(frames, time_seconds - 0.28, time_seconds + 0.28);
    let before = average_frame_rms(frames, time_seconds - 1.35, time_seconds - 0.35);
    let after = average_frame_rms(frames, time_seconds + 0.35, time_seconds + 1.35);
    let context = ((before + after) * 0.5).max(global).max(0.0001);
    let dip_score = (1.0 - (center / context).clamp(0.0, 1.2) / 1.2).clamp(0.0, 1.0);
    let edge_score = ((after - before).abs() / global.max(0.0001)).clamp(0.0, 2.0) / 2.0;
    let calm_score = (1.0 - ((center - context).abs() / context).clamp(0.0, 1.0)).clamp(0.0, 1.0);
    (dip_score * 0.52 + edge_score * 0.30 + calm_score * 0.18).clamp(0.0, 1.0)
}

fn snap_to_nearest_phrase(
    time_seconds: f64,
    tempo: &MusicTempoAnalysis,
    duration_seconds: f64,
) -> Option<f64> {
    let grid = tempo.beat_grid.as_ref()?;
    if grid.interval_seconds <= 0.0 || !time_seconds.is_finite() {
        return None;
    }
    let phrase_seconds = grid.interval_seconds * 4.0;
    if phrase_seconds <= 0.0 {
        return None;
    }
    let phrase_origin = tempo
        .downbeat_grid
        .as_ref()
        .filter(|downbeat| downbeat.confidence >= 0.18)
        .map(|downbeat| downbeat.first_downbeat_seconds)
        .unwrap_or(grid.first_beat_seconds);
    let steps = ((time_seconds - phrase_origin) / phrase_seconds).round();
    let snapped = (phrase_origin + steps * phrase_seconds).clamp(0.0, duration_seconds.max(0.0));
    let window = phrase_seconds.min(2.4).max(0.75);
    if (snapped - time_seconds).abs() <= window {
        Some(snapped)
    } else {
        None
    }
}

fn mix_point_reason(
    prefix: &str,
    phrase_delta: f64,
    cue: &PerceptualCuePoint,
    fallback_reason: &str,
) -> String {
    if cue.total >= MIX_POINT_PERCEPTUAL_TARGET && cue.vocal_safety >= MIX_POINT_VOCAL_SAFE_TARGET {
        format!(
            "{prefix} · perceptual {:.2} · vocal {:.2} · relay {:.2} · grid {:.2} · flow {:.2} · phrase {phrase_delta:+.2}s",
            cue.total,
            cue.vocal_safety,
            cue.vocal_handoff_score,
            cue.phrase_grid_fit,
            cue.emotional_continuity
        )
    } else if cue.vocal_safety >= MIX_POINT_VOCAL_SAFE_TARGET {
        format!(
            "{prefix} · vocal-safe {:.2} · perceptual {:.2} · relay {:.2} · grid {:.2} · phrase {phrase_delta:+.2}s",
            cue.vocal_safety, cue.total, cue.vocal_handoff_score, cue.phrase_grid_fit
        )
    } else {
        format!(
            "{fallback_reason} · vocal risk {:.2} · perceptual {:.2} · relay {:.2} · grid {:.2} · phrase {phrase_delta:+.2}s",
            cue.vocal_safety, cue.total, cue.vocal_handoff_score, cue.phrase_grid_fit
        )
    }
}

fn estimate_mix_points(
    sections: &MusicSectionAnalysis,
    tempo: &MusicTempoAnalysis,
    duration_seconds: f64,
    section_curves: &MusicSectionCurveAnalysis,
    frames: &[AnalysisFrame],
) -> MusicMixPointAnalysis {
    let raw_in = sections
        .intro
        .as_ref()
        .map(|intro| intro.end_seconds)
        .unwrap_or(0.0)
        .clamp(0.0, duration_seconds);
    let raw_out = sections
        .outro
        .as_ref()
        .map(|outro| outro.start_seconds)
        .unwrap_or_else(|| (duration_seconds - 12.0).max(duration_seconds * 0.82))
        .clamp(0.0, duration_seconds);

    let mut mix_in = Vec::new();
    let mut mix_out = Vec::new();
    for candidate in sections.highlight_candidates.iter() {
        let start = snap_to_nearby_boundary(
            section_curves,
            candidate.start_seconds,
            3.0,
            duration_seconds,
        );
        let end =
            snap_to_nearby_boundary(section_curves, candidate.end_seconds, 3.0, duration_seconds);

        let phrase_start = snap_to_nearest_phrase(start, tempo, duration_seconds)
            .unwrap_or_else(|| align_to_next_beat(start, tempo).clamp(0.0, duration_seconds));
        let phrase_end = snap_to_nearest_phrase(end, tempo, duration_seconds)
            .unwrap_or_else(|| align_to_previous_beat(end, tempo).clamp(0.0, duration_seconds));
        let safe_start = phrase_safe_time_near(
            frames,
            section_curves,
            tempo,
            phrase_start,
            2.4,
            duration_seconds,
            PerceptualCueRole::MixIn,
        );
        let safe_end = phrase_safe_time_near(
            frames,
            section_curves,
            tempo,
            phrase_end,
            2.8,
            duration_seconds,
            PerceptualCueRole::MixOut,
        );
        let phrase_start_delta = safe_start.time_seconds - start;
        let phrase_end_delta = safe_end.time_seconds - end;

        mix_in.push(MusicMixPoint {
            time_seconds: safe_start.time_seconds.clamp(0.0, duration_seconds),
            confidence: (candidate.confidence * 0.64 + safe_start.total * 0.28 + 0.08)
                .clamp(0.0, 1.0),
            reason: mix_point_reason(
                "phrase-safe chorus in",
                phrase_start_delta,
                &safe_start,
                "highlight start boundary",
            ),
            phrase_snap_seconds: phrase_start_delta,
            vocal_safety: safe_start.vocal_safety,
            perceptual_score: safe_start.total,
            phrase_closure: safe_start.phrase_closure,
            masking_opportunity: safe_start.masking_opportunity,
            attention_safety: safe_start.attention_safety,
            expectation_safety: safe_start.expectation_safety,
            phrase_grid_fit: safe_start.phrase_grid_fit,
            emotional_continuity: safe_start.emotional_continuity,
            vocal_handoff_score: safe_start.vocal_handoff_score,
        });
        mix_out.push(MusicMixPoint {
            time_seconds: safe_end.time_seconds.clamp(0.0, duration_seconds),
            confidence: (candidate.confidence * 0.64 + safe_end.total * 0.28 + 0.08)
                .clamp(0.0, 1.0),
            reason: mix_point_reason(
                "phrase-safe chorus out",
                phrase_end_delta,
                &safe_end,
                "highlight end boundary",
            ),
            phrase_snap_seconds: phrase_end_delta,
            vocal_safety: safe_end.vocal_safety,
            perceptual_score: safe_end.total,
            phrase_closure: safe_end.phrase_closure,
            masking_opportunity: safe_end.masking_opportunity,
            attention_safety: safe_end.attention_safety,
            expectation_safety: safe_end.expectation_safety,
            phrase_grid_fit: safe_end.phrase_grid_fit,
            emotional_continuity: safe_end.emotional_continuity,
            vocal_handoff_score: safe_end.vocal_handoff_score,
        });
    }

    if mix_in.is_empty() {
        let phrase_in = snap_to_nearest_phrase(raw_in, tempo, duration_seconds)
            .unwrap_or_else(|| align_to_next_beat(raw_in, tempo).clamp(0.0, duration_seconds));
        let safe_in = phrase_safe_time_near(
            frames,
            section_curves,
            tempo,
            phrase_in,
            2.4,
            duration_seconds,
            PerceptualCueRole::MixIn,
        );
        mix_in.push(MusicMixPoint {
            time_seconds: safe_in.time_seconds.clamp(0.0, duration_seconds),
            confidence: ((if tempo.beat_grid.is_some() {
                0.58
            } else {
                0.36
            }) + safe_in.total * 0.24)
                .clamp(0.0, 1.0),
            reason: mix_point_reason(
                "phrase-safe intro in",
                safe_in.time_seconds - raw_in,
                &safe_in,
                "intro end aligned to beat grid",
            ),
            phrase_snap_seconds: safe_in.time_seconds - raw_in,
            vocal_safety: safe_in.vocal_safety,
            perceptual_score: safe_in.total,
            phrase_closure: safe_in.phrase_closure,
            masking_opportunity: safe_in.masking_opportunity,
            attention_safety: safe_in.attention_safety,
            expectation_safety: safe_in.expectation_safety,
            phrase_grid_fit: safe_in.phrase_grid_fit,
            emotional_continuity: safe_in.emotional_continuity,
            vocal_handoff_score: safe_in.vocal_handoff_score,
        });
    }
    let phrase_out = snap_to_nearest_phrase(raw_out, tempo, duration_seconds)
        .unwrap_or_else(|| align_to_previous_beat(raw_out, tempo).clamp(0.0, duration_seconds));
    let safe_out = phrase_safe_time_near(
        frames,
        section_curves,
        tempo,
        phrase_out,
        2.8,
        duration_seconds,
        PerceptualCueRole::MixOut,
    );
    mix_out.push(MusicMixPoint {
        time_seconds: safe_out.time_seconds.clamp(0.0, duration_seconds),
        confidence: ((if tempo.beat_grid.is_some() {
            0.50
        } else {
            0.30
        }) + safe_out.total * 0.26)
            .clamp(0.0, 1.0),
        reason: mix_point_reason(
            "phrase-safe outro out",
            safe_out.time_seconds - raw_out,
            &safe_out,
            "outro start aligned to beat grid",
        ),
        phrase_snap_seconds: safe_out.time_seconds - raw_out,
        vocal_safety: safe_out.vocal_safety,
        perceptual_score: safe_out.total,
        phrase_closure: safe_out.phrase_closure,
        masking_opportunity: safe_out.masking_opportunity,
        attention_safety: safe_out.attention_safety,
        expectation_safety: safe_out.expectation_safety,
        phrase_grid_fit: safe_out.phrase_grid_fit,
        emotional_continuity: safe_out.emotional_continuity,
        vocal_handoff_score: safe_out.vocal_handoff_score,
    });
    mix_in.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    mix_out.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    mix_in.truncate(4);
    mix_out.truncate(4);

    MusicMixPointAnalysis { mix_in, mix_out }
}

fn estimate_stage_mix_music_map(
    sections: &MusicSectionAnalysis,
    mix_points: &MusicMixPointAnalysis,
    tempo: &MusicTempoAnalysis,
    duration_seconds: f64,
    section_curves: &MusicSectionCurveAnalysis,
    frames: &[AnalysisFrame],
) -> StageMixMusicMap {
    let mut map = StageMixMusicMap {
        debug_only: false,
        summary_zh: "Stage Mix 音樂地圖 v8：高精度顯示，可信 highlight span 可作為播放選段提示；不接管播放心臟"
            .to_owned(),
        ..StageMixMusicMap::default()
    };

    let mut source_count = 0_u32;
    let attention_context = music_map_attention_context(frames, section_curves, duration_seconds);
    for candidate in &sections.highlight_candidates {
        source_count = source_count.saturating_add(1);
        push_music_map_point(
            &mut map.hook_start,
            MusicMapRole::HookStart,
            candidate.start_seconds,
            music_map_hook_start_confidence(candidate, duration_seconds),
            music_map_hook_start_reason(candidate, duration_seconds),
            duration_seconds,
        );
        let highlight_span = music_map_highlight_span_from_candidate(
            candidate,
            &sections.functional_segments,
            frames,
            section_curves,
            &attention_context,
            duration_seconds,
        );
        let peak_time = highlight_span
            .as_ref()
            .map(|span| span.peak_seconds)
            .unwrap_or_else(|| {
                refined_music_map_peak_time(candidate, frames, section_curves, duration_seconds)
            });
        push_music_map_point(
            &mut map.highlight_peak,
            MusicMapRole::HighlightPeak,
            peak_time,
            music_map_highlight_peak_confidence(candidate, peak_time, duration_seconds),
            music_map_highlight_peak_reason(candidate, peak_time, duration_seconds),
            duration_seconds,
        );
        if let Some(span) = highlight_span {
            map.highlight_span.push(span);
        }
        let valley_time = (peak_time + 7.5)
            .min(candidate.end_seconds - 1.0)
            .max(candidate.start_seconds);
        if valley_time > peak_time + 1.0 {
            let valley_risk = post_peak_valley_risk(frames, peak_time, valley_time);
            if valley_risk >= 0.24 {
                push_music_map_point(
                    &mut map.post_peak_valley,
                    MusicMapRole::PostPeakValley,
                    valley_time,
                    valley_risk,
                    "疑似高潮後低谷：保留給 debug，避免誤當高亮或進場".to_owned(),
                    duration_seconds,
                );
            }
        }
    }

    for segment in &sections.functional_segments {
        source_count = source_count.saturating_add(1);
        match segment.role {
            MusicFunctionalRole::Chorus | MusicFunctionalRole::FinalChorus => {
                if let Some(confidence) =
                    music_map_functional_hook_confidence(segment, duration_seconds)
                {
                    push_music_map_point(
                        &mut map.hook_start,
                        MusicMapRole::HookStart,
                        segment.start_seconds,
                        confidence,
                        format!(
                            "{}前緣：功能段落候選；v7 只保留短段落或 final highlight，避免長段落假前緣",
                            if matches!(segment.role, MusicFunctionalRole::FinalChorus) {
                                "final highlight "
                            } else {
                                "副歌/高亮"
                            }
                        ),
                        duration_seconds,
                    );
                }
            }
            MusicFunctionalRole::Outro | MusicFunctionalRole::Silence => {
                push_music_map_point(
                    &mut map.tail_or_silence_risk,
                    MusicMapRole::TailOrSilenceRisk,
                    segment.start_seconds,
                    (segment.confidence + 0.18).clamp(0.0, 1.0),
                    "疑似尾段/靜音風險：不能直接當進場或高亮前緣".to_owned(),
                    duration_seconds,
                );
            }
            _ => {}
        }
    }

    for boundary in section_curves
        .boundary_candidates
        .iter()
        .chain(section_curves.structure.novelty_boundaries.iter())
    {
        source_count = source_count.saturating_add(1);
        if boundary.confidence < 0.34 {
            continue;
        }
        let near_highlight = sections.highlight_candidates.iter().any(|candidate| {
            (candidate.start_seconds - boundary.time_seconds).abs() <= 6.0
                || (candidate.end_seconds - boundary.time_seconds).abs() <= 6.0
        });
        if near_highlight {
            push_music_map_point(
                &mut map.hook_start,
                MusicMapRole::HookStart,
                boundary.time_seconds,
                (boundary.confidence * 0.62 + 0.12).clamp(0.0, 1.0),
                "段落邊界靠近高亮：可能是副歌/高亮前緣".to_owned(),
                duration_seconds,
            );
        }
    }

    for point in &mix_points.mix_in {
        source_count = source_count.saturating_add(1);
        if music_map_tail_risk(sections, point.time_seconds, duration_seconds) >= 0.55 {
            push_music_map_point(
                &mut map.tail_or_silence_risk,
                MusicMapRole::TailOrSilenceRisk,
                point.time_seconds,
                music_map_tail_risk(sections, point.time_seconds, duration_seconds),
                "可疑進場點落在尾段/靜音風險附近，降級為風險標記".to_owned(),
                duration_seconds,
            );
            continue;
        }
        let entry_confidence = music_map_entry_safe_confidence(point, sections, duration_seconds);
        push_music_map_point(
            &mut map.entry_safe,
            MusicMapRole::EntrySafe,
            point.time_seconds,
            entry_confidence,
            format!(
                "可進場點：高精度顯示，vocal {:.2} / perceptual {:.2} / grid {:.2}",
                point.vocal_safety, point.perceptual_score, point.phrase_grid_fit
            ),
            duration_seconds,
        );
    }
    for point in &mix_points.mix_out {
        source_count = source_count.saturating_add(1);
        let exit_confidence = music_map_exit_safe_confidence(point, sections, duration_seconds);
        push_music_map_point(
            &mut map.exit_safe,
            MusicMapRole::ExitSafe,
            point.time_seconds,
            exit_confidence,
            format!(
                "可退場點：高精度顯示，vocal {:.2} / perceptual {:.2} / relay {:.2}",
                point.vocal_safety, point.perceptual_score, point.vocal_handoff_score
            ),
            duration_seconds,
        );
    }

    add_near_miss_hook_points(&mut map, sections, tempo, duration_seconds, frames);
    map.hook_start =
        compact_music_map_points(std::mem::take(&mut map.hook_start), duration_seconds);
    map.highlight_span =
        compact_music_map_spans(std::mem::take(&mut map.highlight_span), duration_seconds);
    map.highlight_peak =
        compact_music_map_points(std::mem::take(&mut map.highlight_peak), duration_seconds);
    map.entry_safe =
        compact_music_map_points(std::mem::take(&mut map.entry_safe), duration_seconds);
    map.exit_safe = compact_music_map_points(std::mem::take(&mut map.exit_safe), duration_seconds);
    map.post_peak_valley =
        compact_music_map_points(std::mem::take(&mut map.post_peak_valley), duration_seconds);
    map.tail_or_silence_risk = compact_music_map_points(
        std::mem::take(&mut map.tail_or_silence_risk),
        duration_seconds,
    );
    map.suppressed_candidate_count = source_count.saturating_sub(
        (map.hook_start.len()
            + map.highlight_span.len()
            + map.highlight_peak.len()
            + map.entry_safe.len()
            + map.exit_safe.len()
            + map.post_peak_valley.len()
            + map.tail_or_silence_risk.len()) as u32,
    );
    map.source_candidate_count = source_count;
    map.human_check_queue = select_music_map_human_checks(&map, duration_seconds);
    map
}

fn music_map_functional_hook_confidence(
    segment: &MusicFunctionalSegment,
    duration_seconds: f64,
) -> Option<f32> {
    let segment_len = (segment.end_seconds - segment.start_seconds).max(0.0);
    let is_final = matches!(segment.role, MusicFunctionalRole::FinalChorus);

    if !is_final {
        let too_long = segment_len > MAX_CHORUS_FUNCTIONAL_SEGMENT_SECONDS * 1.35;
        let too_early = segment.start_seconds < music_map_early_hook_floor(duration_seconds);
        if too_long || too_early {
            return None;
        }
    }

    let role_boost = if is_final { 0.02 } else { 0.04 };
    Some((segment.confidence + role_boost).clamp(0.0, 1.0))
}

fn music_map_early_hook_floor(duration_seconds: f64) -> f64 {
    (duration_seconds * MUSIC_MAP_EARLY_HOOK_FLOOR_RATIO)
        .min(MUSIC_MAP_EARLY_HOOK_FLOOR_SECONDS)
        .max(12.0)
}

fn push_music_map_point(
    points: &mut Vec<MusicMapPoint>,
    role: MusicMapRole,
    time_seconds: f64,
    confidence: f32,
    reason_zh: String,
    duration_seconds: f64,
) {
    if !time_seconds.is_finite() || duration_seconds <= 0.0 {
        return;
    }
    let time_seconds = time_seconds.clamp(0.0, duration_seconds);
    let listen_from_seconds = (time_seconds - 6.0).clamp(0.0, duration_seconds);
    let listen_to_seconds = (time_seconds + 3.0).clamp(listen_from_seconds, duration_seconds);
    points.push(MusicMapPoint {
        role,
        time_seconds,
        confidence: confidence.clamp(0.0, 1.0),
        reason_zh,
        lyric_text: None,
        listen_from_seconds,
        listen_to_seconds,
    });
}

fn music_map_role_min_confidence(role: &MusicMapRole) -> f32 {
    match role {
        MusicMapRole::HookStart => MUSIC_MAP_MIN_HOOK_CONFIDENCE,
        MusicMapRole::HighlightPeak => MUSIC_MAP_MIN_HIGHLIGHT_CONFIDENCE,
        MusicMapRole::EntrySafe => MUSIC_MAP_MIN_ENTRY_CONFIDENCE,
        MusicMapRole::ExitSafe => MUSIC_MAP_MIN_EXIT_CONFIDENCE,
        MusicMapRole::PostPeakValley | MusicMapRole::TailOrSilenceRisk => {
            MUSIC_MAP_MIN_RISK_CONFIDENCE
        }
    }
}

fn compact_music_map_points(
    mut points: Vec<MusicMapPoint>,
    duration_seconds: f64,
) -> Vec<MusicMapPoint> {
    points.retain(|point| {
        point.time_seconds >= 4.0
            && point.time_seconds <= duration_seconds - 2.0
            && point.confidence >= music_map_role_min_confidence(&point.role)
    });
    points.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut compacted: Vec<MusicMapPoint> = Vec::new();
    for point in points {
        if compacted
            .iter()
            .any(|kept| (kept.time_seconds - point.time_seconds).abs() <= 5.0)
        {
            continue;
        }
        compacted.push(point);
        if compacted.len() >= MAX_MUSIC_MAP_ROLE_POINTS {
            break;
        }
    }
    compacted.sort_by(|a, b| {
        a.time_seconds
            .partial_cmp(&b.time_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    compacted
}

fn compact_music_map_spans(
    mut spans: Vec<MusicMapSpan>,
    duration_seconds: f64,
) -> Vec<MusicMapSpan> {
    spans.retain(|span| {
        span.start_seconds.is_finite()
            && span.end_seconds.is_finite()
            && span.peak_seconds.is_finite()
            && span.start_seconds >= 4.0
            && span.end_seconds <= duration_seconds
            && span.end_seconds > span.start_seconds + 3.0
            && span.confidence >= MUSIC_MAP_MIN_HIGHLIGHT_SPAN_CONFIDENCE
    });
    spans.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut compacted: Vec<MusicMapSpan> = Vec::new();
    for span in spans {
        if compacted.iter().any(|kept| {
            (kept.peak_seconds - span.peak_seconds).abs() <= 7.0
                || music_map_span_overlap_ratio(kept, &span) >= 0.48
        }) {
            continue;
        }
        compacted.push(span);
        if compacted.len() >= MAX_MUSIC_MAP_ROLE_POINTS {
            break;
        }
    }
    compacted.sort_by(|a, b| {
        a.start_seconds
            .partial_cmp(&b.start_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    compacted
}

fn music_map_span_overlap_ratio(a: &MusicMapSpan, b: &MusicMapSpan) -> f64 {
    let start = a.start_seconds.max(b.start_seconds);
    let end = a.end_seconds.min(b.end_seconds);
    let overlap = (end - start).max(0.0);
    let shorter = (a.end_seconds - a.start_seconds)
        .min(b.end_seconds - b.start_seconds)
        .max(0.0001);
    (overlap / shorter).clamp(0.0, 1.0)
}

fn music_map_attention_context(
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    duration_seconds: f64,
) -> MusicMapAttentionContext {
    let global_rms = global_frame_rms(frames);
    let mut strongest_window_score = 0.0_f32;
    if duration_seconds.is_finite() && duration_seconds > MUSIC_MAP_ATTENTION_WINDOW_SECONDS {
        let half_window = MUSIC_MAP_ATTENTION_WINDOW_SECONDS * 0.5;
        let mut center = half_window.max(0.0);
        while center <= duration_seconds - half_window {
            let start = (center - half_window).clamp(0.0, duration_seconds);
            let end = (center + half_window).clamp(start, duration_seconds);
            strongest_window_score = strongest_window_score.max(music_map_attention_window_score(
                frames,
                section_curves,
                start,
                end,
                global_rms,
                duration_seconds,
            ));
            center += MUSIC_MAP_ATTENTION_HOP_SECONDS;
        }
    }
    MusicMapAttentionContext {
        global_rms,
        strongest_window_score: strongest_window_score.max(0.0001),
    }
}

fn music_map_attention_window_score(
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    start_seconds: f64,
    end_seconds: f64,
    global_rms: f32,
    duration_seconds: f64,
) -> f32 {
    if end_seconds <= start_seconds || duration_seconds <= 0.0 {
        return 0.0;
    }
    let local_energy = average_frame_rms(frames, start_seconds, end_seconds);
    let local_energy_norm = (local_energy / global_rms.max(0.0001)).clamp(0.0, 2.2) / 2.2;
    let chorusness = average_curve_value(&section_curves.chorusness, start_seconds, end_seconds);
    let recurrence = average_curve_value(
        &section_curves.structure.recurrence,
        start_seconds,
        end_seconds,
    );
    let novelty = nearest_structure_novelty_score(section_curves, start_seconds, end_seconds);
    let boundary = average_curve_value(&section_curves.boundary, start_seconds, end_seconds)
        .max(nearest_boundary_score(section_curves, start_seconds, 4.0))
        .max(nearest_boundary_score(section_curves, end_seconds, 4.0))
        .max(novelty);
    let contrast = highlight_peak_window_contrast(frames, start_seconds, end_seconds, global_rms);
    let stability = highlight_peak_energy_stability(frames, start_seconds, end_seconds);
    let position =
        highlight_span_position_score((start_seconds + end_seconds) * 0.5, duration_seconds);

    (chorusness * 0.27
        + recurrence * 0.22
        + local_energy_norm * 0.16
        + boundary * 0.12
        + contrast * 0.10
        + stability * 0.08
        + position * 0.05)
        .clamp(0.0, 1.0)
}

fn music_map_highlight_span_attention_score(
    candidate: &MusicSectionCandidate,
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    attention_context: &MusicMapAttentionContext,
    start_seconds: f64,
    peak_seconds: f64,
    end_seconds: f64,
    duration_seconds: f64,
) -> f32 {
    let span_score = music_map_attention_window_score(
        frames,
        section_curves,
        start_seconds,
        end_seconds,
        attention_context.global_rms,
        duration_seconds,
    );
    let peak_window = music_map_peak_window_seconds(end_seconds - start_seconds);
    let peak_start = (peak_seconds - peak_window * 0.5).clamp(start_seconds, end_seconds);
    let peak_end = (peak_seconds + peak_window * 0.5).clamp(peak_start, end_seconds);
    let peak_attention = music_map_attention_window_score(
        frames,
        section_curves,
        peak_start,
        peak_end,
        attention_context.global_rms,
        duration_seconds,
    );
    let relative_focus =
        (span_score / attention_context.strongest_window_score.max(0.0001)).clamp(0.0, 1.0);
    let candidate_body = (candidate.scores.chorusness * 0.24
        + candidate.scores.repetition * 0.18
        + candidate.scores.structural_recurrence * 0.18
        + candidate.scores.segment_wholeness * 0.16
        + candidate.scores.perceptual * 0.12
        + candidate.scores.boundary * 0.08
        + candidate.scores.duration * 0.04)
        .clamp(0.0, 1.0);
    let position_penalty = music_map_highlight_span_attention_position_penalty(
        start_seconds,
        end_seconds,
        duration_seconds,
    );

    (span_score * 0.34 + peak_attention * 0.18 + relative_focus * 0.18 + candidate_body * 0.30
        - position_penalty)
        .clamp(0.0, 1.0)
}

fn music_map_highlight_span_attention_position_penalty(
    start_seconds: f64,
    end_seconds: f64,
    duration_seconds: f64,
) -> f32 {
    if duration_seconds <= 0.0 || end_seconds <= start_seconds {
        return 0.18;
    }
    let midpoint =
        ((start_seconds + end_seconds) * 0.5 / duration_seconds.max(1.0)).clamp(0.0, 1.0);
    let mut penalty = 0.0_f32;
    if start_seconds < music_map_early_hook_floor(duration_seconds) * 0.65 {
        penalty += 0.10;
    }
    if midpoint > 0.86 || end_seconds > duration_seconds * 0.94 {
        penalty += 0.10;
    }
    if end_seconds - start_seconds < MIN_HIGHLIGHT_SEGMENT_SECONDS {
        penalty += 0.08;
    }
    penalty.clamp(0.0, 0.24)
}

fn music_map_hook_start_confidence(
    candidate: &MusicSectionCandidate,
    duration_seconds: f64,
) -> f32 {
    let position = (candidate.start_seconds / duration_seconds.max(1.0)).clamp(0.0, 1.0) as f32;
    let early_penalty = if position < 0.12 { 0.24 } else { 0.0 };
    let tail_penalty = if position > 0.82 { 0.16 } else { 0.0 };
    (candidate.confidence * 0.58
        + candidate.scores.boundary * 0.16
        + candidate.scores.structural_novelty * 0.10
        + candidate.scores.segment_wholeness * 0.10
        + candidate.scores.perceptual * 0.10
        - early_penalty
        - tail_penalty)
        .clamp(0.0, 1.0)
}

fn music_map_hook_start_reason(candidate: &MusicSectionCandidate, duration_seconds: f64) -> String {
    let position = (candidate.start_seconds / duration_seconds.max(1.0)).clamp(0.0, 1.0);
    if position < 0.12 {
        "早段高亮前緣候選：需小心 intro 假高亮".to_owned()
    } else if position > 0.82 {
        "後段高亮前緣候選：可能是 final highlight，不自動當進場".to_owned()
    } else {
        format!(
            "副歌/高亮前緣：{}，boundary {:.2}，recurrence {:.2}",
            candidate.reason, candidate.scores.boundary, candidate.scores.structural_recurrence
        )
    }
}

fn music_map_highlight_peak_confidence(
    candidate: &MusicSectionCandidate,
    peak_time: f64,
    duration_seconds: f64,
) -> f32 {
    let position = (peak_time / duration_seconds.max(1.0)).clamp(0.0, 1.0) as f32;
    let early_penalty = if position < 0.10 { 0.22 } else { 0.0 };
    (candidate.confidence * 0.60
        + candidate.scores.chorusness * 0.12
        + candidate.scores.energy * 0.12
        + candidate.scores.repetition * 0.10
        + candidate.scores.structural_recurrence * 0.08
        - early_penalty)
        .clamp(0.0, 1.0)
}

fn music_map_highlight_peak_reason(
    candidate: &MusicSectionCandidate,
    peak_time: f64,
    duration_seconds: f64,
) -> String {
    let position = (peak_time / duration_seconds.max(1.0)).clamp(0.0, 1.0);
    if position > 0.82 {
        "高亮中心：後段/final highlight 可保留，但不直接當進場".to_owned()
    } else {
        format!(
            "高亮中心：chorus {:.2} / repetition {:.2} / energy {:.2}",
            candidate.scores.chorusness, candidate.scores.repetition, candidate.scores.energy
        )
    }
}

fn music_map_highlight_span_from_candidate(
    candidate: &MusicSectionCandidate,
    functional_segments: &[MusicFunctionalSegment],
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    attention_context: &MusicMapAttentionContext,
    duration_seconds: f64,
) -> Option<MusicMapSpan> {
    let (start_seconds, end_seconds, clip_note) = music_map_highlight_span_bounds(
        candidate,
        functional_segments,
        section_curves,
        duration_seconds,
    )?;
    let peak_seconds = refined_music_map_peak_time_in_span(
        candidate,
        frames,
        section_curves,
        start_seconds,
        end_seconds,
        duration_seconds,
    );
    let lift_seconds = music_map_lift_time(start_seconds, end_seconds, frames, duration_seconds);
    let span_len = (end_seconds - start_seconds).max(0.0);
    let peak_score = highlight_peak_window_score(
        candidate,
        frames,
        section_curves,
        peak_seconds,
        music_map_peak_window_seconds(span_len),
        duration_seconds,
    );
    let position_score =
        highlight_span_position_score((start_seconds + end_seconds) * 0.5, duration_seconds);
    let duration_score = highlight_span_duration_score(span_len);
    let attention_score = music_map_highlight_span_attention_score(
        candidate,
        frames,
        section_curves,
        attention_context,
        start_seconds,
        peak_seconds,
        end_seconds,
        duration_seconds,
    );
    let confidence = (candidate.confidence * 0.30
        + attention_score * 0.30
        + peak_score * 0.18
        + duration_score * 0.09
        + position_score * 0.07
        + candidate.scores.segment_wholeness * 0.05
        + candidate.scores.perceptual * 0.03)
        .clamp(0.0, 1.0);

    Some(MusicMapSpan {
        start_seconds,
        lift_seconds,
        peak_seconds,
        end_seconds,
        confidence,
        reason_zh: format!(
            "主高亮段：v8 attention span；{clip_note}；attention {:.2} / peak {:.2} / chorus {:.2} / recurrence {:.2}",
            attention_score,
            peak_score,
            candidate.scores.chorusness,
            candidate.scores.structural_recurrence
        ),
        listen_from_seconds: (lift_seconds - 4.0).clamp(0.0, duration_seconds),
    })
}

fn music_map_highlight_span_bounds(
    candidate: &MusicSectionCandidate,
    functional_segments: &[MusicFunctionalSegment],
    section_curves: &MusicSectionCurveAnalysis,
    duration_seconds: f64,
) -> Option<(f64, f64, &'static str)> {
    let mut start = candidate.start_seconds.clamp(0.0, duration_seconds);
    let mut end = candidate.end_seconds.clamp(start, duration_seconds);
    let mut note = "候選段落原始範圍";
    let candidate_len = (end - start).max(0.0);
    if candidate_len < MIN_HIGHLIGHT_SEGMENT_SECONDS * 0.55 {
        return None;
    }

    let best_functional = functional_segments
        .iter()
        .filter(|segment| {
            matches!(
                segment.role,
                MusicFunctionalRole::Chorus | MusicFunctionalRole::FinalChorus
            )
        })
        .filter_map(|segment| {
            let overlap_start = start.max(segment.start_seconds);
            let overlap_end = end.min(segment.end_seconds);
            let overlap = (overlap_end - overlap_start).max(0.0);
            (overlap >= MIN_HIGHLIGHT_SEGMENT_SECONDS * 0.70
                || overlap / candidate_len.max(0.0001) >= 0.42)
                .then_some((segment, overlap))
        })
        .max_by(|(a_segment, a_overlap), (b_segment, b_overlap)| {
            let a_score = *a_overlap as f32 * a_segment.confidence.max(0.15);
            let b_score = *b_overlap as f32 * b_segment.confidence.max(0.15);
            a_score
                .partial_cmp(&b_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    if let Some((segment, _)) = best_functional {
        let clipped_start = start.max(segment.start_seconds);
        let clipped_end = end.min(segment.end_seconds);
        if clipped_end - clipped_start >= MIN_HIGHLIGHT_SEGMENT_SECONDS * 0.65 {
            start = clipped_start;
            end = clipped_end;
            note = if matches!(segment.role, MusicFunctionalRole::FinalChorus) {
                "貼齊 final highlight 功能段"
            } else {
                "貼齊 chorus-like 功能段"
            };
        }
    }

    if end - start < MIN_HIGHLIGHT_SEGMENT_SECONDS * 0.55 {
        return None;
    }

    let boundary_start = snap_to_nearby_boundary(section_curves, start, 4.0, duration_seconds);
    let boundary_end = snap_to_nearby_boundary(section_curves, end, 4.0, duration_seconds);
    if (boundary_start - start).abs() > 0.001 || (boundary_end - end).abs() > 0.001 {
        let snapped_start = boundary_start.clamp(0.0, duration_seconds);
        let snapped_end = boundary_end.clamp(snapped_start, duration_seconds);
        if snapped_end - snapped_start >= MIN_HIGHLIGHT_SEGMENT_SECONDS * 0.65 {
            start = snapped_start;
            end = snapped_end;
            note = if note == "候選段落原始範圍" {
                "貼齊強段落邊界"
            } else {
                "貼齊功能段與強段落邊界"
            };
        }
    }

    if end - start < MIN_HIGHLIGHT_SEGMENT_SECONDS * 0.55 {
        return None;
    }
    Some((start, end, note))
}

fn music_map_lift_time(
    span_start: f64,
    span_end: f64,
    frames: &[AnalysisFrame],
    duration_seconds: f64,
) -> f64 {
    if frames.is_empty() {
        return span_start.clamp(0.0, duration_seconds);
    }
    let search_start = (span_start - 4.0).clamp(0.0, duration_seconds);
    let search_end = (span_start + 4.0).clamp(search_start, span_end.min(duration_seconds));
    let global = global_frame_rms(frames);
    let mut best_time = span_start;
    let mut best_score = 0.0_f32;
    for frame in frames {
        if frame.time_seconds < search_start || frame.time_seconds > search_end {
            continue;
        }
        let before = average_frame_rms(frames, frame.time_seconds - 2.4, frame.time_seconds - 0.45);
        let after = average_frame_rms(frames, frame.time_seconds + 0.35, frame.time_seconds + 2.4);
        let lift = ((after - before) / global.max(0.0001)).clamp(0.0, 1.8) / 1.8;
        let start_proximity =
            (1.0 - ((frame.time_seconds - span_start).abs() / 4.0).clamp(0.0, 1.0)) as f32;
        let score = lift * 0.78 + start_proximity * 0.22;
        if score > best_score {
            best_score = score;
            best_time = frame.time_seconds;
        }
    }
    if best_score < 0.08 {
        span_start.clamp(0.0, duration_seconds)
    } else {
        best_time.clamp(0.0, duration_seconds)
    }
}

fn refined_music_map_peak_time(
    candidate: &MusicSectionCandidate,
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    duration_seconds: f64,
) -> f64 {
    refined_music_map_peak_time_in_span(
        candidate,
        frames,
        section_curves,
        candidate.start_seconds,
        candidate.end_seconds,
        duration_seconds,
    )
}

fn refined_music_map_peak_time_in_span(
    candidate: &MusicSectionCandidate,
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    span_start: f64,
    span_end: f64,
    duration_seconds: f64,
) -> f64 {
    let span_start = span_start.clamp(0.0, duration_seconds);
    let span_end = span_end.clamp(span_start, duration_seconds);
    let span_len = span_end - span_start;
    if frames.is_empty() || span_len <= 0.0 {
        return ((span_start + span_end) * 0.5).clamp(0.0, duration_seconds);
    }

    let window_seconds = music_map_peak_window_seconds(span_len);
    let half_window = window_seconds * 0.5;
    let min_center = (span_start + half_window).min(span_end);
    let max_center = (span_end - half_window).max(span_start);
    let mut best_time = ((span_start + span_end) * 0.5).clamp(0.0, duration_seconds);
    let mut best_score = f32::NEG_INFINITY;
    for frame in frames {
        if frame.time_seconds < span_start || frame.time_seconds > span_end {
            continue;
        }
        let center = frame.time_seconds.clamp(min_center, max_center);
        let score = highlight_peak_window_score(
            candidate,
            frames,
            section_curves,
            center,
            window_seconds,
            duration_seconds,
        );
        if score > best_score {
            best_score = score;
            best_time = center;
        }
    }
    best_time.clamp(0.0, duration_seconds)
}

fn music_map_peak_window_seconds(span_len: f64) -> f64 {
    if span_len <= 0.0 {
        return 3.0;
    }
    (span_len * 0.34).clamp(3.0, 6.0).min(span_len.max(1.0))
}

fn highlight_peak_window_score(
    candidate: &MusicSectionCandidate,
    frames: &[AnalysisFrame],
    section_curves: &MusicSectionCurveAnalysis,
    center_seconds: f64,
    window_seconds: f64,
    duration_seconds: f64,
) -> f32 {
    let half_window = window_seconds.max(0.5) * 0.5;
    let start = (center_seconds - half_window).clamp(0.0, duration_seconds);
    let end = (center_seconds + half_window).clamp(start, duration_seconds);
    let global = global_frame_rms(frames);
    let local_energy = average_frame_rms(frames, start, end);
    let local_energy_norm = (local_energy / global.max(0.0001)).clamp(0.0, 2.2) / 2.2;
    let chorusness = average_curve_value(&section_curves.chorusness, start, end)
        .max(candidate.scores.chorusness * 0.62);
    let structural_recurrence =
        average_curve_value(&section_curves.structure.recurrence, start, end)
            .max(candidate.scores.structural_recurrence * 0.70);
    let structural_novelty = nearest_structure_novelty_score(section_curves, start, end)
        .max(candidate.scores.structural_novelty * 0.72);
    let boundary =
        average_curve_value(&section_curves.boundary, start, end).max(structural_novelty);
    let local_contrast = highlight_peak_window_contrast(frames, start, end, global);
    let energy_stability = highlight_peak_energy_stability(frames, start, end);
    let position = highlight_peak_position_score(candidate, center_seconds);
    let perceptual = candidate
        .scores
        .perceptual
        .max(candidate.scores.segment_wholeness * 0.82);
    let short_spike_penalty =
        highlight_peak_short_spike_penalty(frames, center_seconds, start, end);
    let intro_outro_penalty = highlight_peak_intro_outro_penalty(center_seconds, duration_seconds);

    (local_energy_norm * 0.22
        + chorusness * 0.20
        + structural_recurrence * 0.16
        + candidate.scores.repetition * 0.12
        + local_contrast * 0.10
        + energy_stability * 0.08
        + position * 0.07
        + perceptual.max(boundary) * 0.05
        - short_spike_penalty
        - intro_outro_penalty)
        .clamp(0.0, 1.0)
}

fn highlight_peak_window_contrast(
    frames: &[AnalysisFrame],
    start: f64,
    end: f64,
    global: f32,
) -> f32 {
    let center = average_frame_rms(frames, start, end);
    let before = average_frame_rms(frames, start - 6.0, start - 1.2);
    let after = average_frame_rms(frames, end + 1.2, end + 6.0);
    let context = if before > 0.0 && after > 0.0 {
        (before + after) * 0.5
    } else {
        before.max(after).max(global)
    };
    ((center - context) / global.max(0.0001)).clamp(0.0, 1.6) / 1.6
}

fn highlight_peak_energy_stability(frames: &[AnalysisFrame], start: f64, end: f64) -> f32 {
    if frames.is_empty() || end <= start {
        return 0.0;
    }
    let mut values = Vec::new();
    for frame in frames {
        if frame.time_seconds >= start && frame.time_seconds <= end {
            values.push(frame.rms.max(0.0));
        }
    }
    if values.len() < 2 {
        return 0.0;
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    if mean <= 0.0001 {
        return 0.0;
    }
    let variance = values
        .iter()
        .map(|value| {
            let delta = *value - mean;
            delta * delta
        })
        .sum::<f32>()
        / values.len() as f32;
    let coefficient = variance.sqrt() / mean.max(0.0001);
    (1.0 - (coefficient / 0.90).clamp(0.0, 1.0)).clamp(0.0, 1.0)
}

fn highlight_peak_position_score(candidate: &MusicSectionCandidate, time_seconds: f64) -> f32 {
    let midpoint = (candidate.start_seconds + candidate.end_seconds) * 0.5;
    let half_len = ((candidate.end_seconds - candidate.start_seconds) * 0.5).max(1.0);
    (1.0 - ((time_seconds - midpoint).abs() / half_len).clamp(0.0, 1.0)) as f32
}

fn highlight_peak_short_spike_penalty(
    frames: &[AnalysisFrame],
    center_seconds: f64,
    window_start: f64,
    window_end: f64,
) -> f32 {
    let window = average_frame_rms(frames, window_start, window_end);
    if window <= 0.0001 {
        return 0.0;
    }
    let center = average_frame_rms(frames, center_seconds - 0.22, center_seconds + 0.22);
    let ratio = center / window.max(0.0001);
    ((ratio - 1.65) / 1.35).clamp(0.0, 1.0) * 0.18
}

fn highlight_peak_intro_outro_penalty(time_seconds: f64, duration_seconds: f64) -> f32 {
    let position = (time_seconds / duration_seconds.max(1.0)).clamp(0.0, 1.0);
    if position < 0.10 {
        0.18
    } else if position < 0.16 {
        0.08
    } else if position > 0.90 {
        0.16
    } else if position > 0.82 {
        0.06
    } else {
        0.0
    }
}

fn highlight_span_position_score(midpoint_seconds: f64, duration_seconds: f64) -> f32 {
    let position = (midpoint_seconds / duration_seconds.max(1.0)).clamp(0.0, 1.0);
    if (0.16..=0.78).contains(&position) {
        1.0
    } else if position < 0.16 {
        (position / 0.16).clamp(0.0, 1.0) as f32
    } else {
        ((1.0 - position) / 0.22).clamp(0.0, 1.0) as f32
    }
}

fn highlight_span_duration_score(span_len: f64) -> f32 {
    if span_len < 6.0 {
        (span_len / 6.0).clamp(0.0, 1.0) as f32
    } else if span_len <= 28.0 {
        1.0
    } else if span_len <= 48.0 {
        (1.0 - (span_len - 28.0) / 28.0).clamp(0.35, 1.0) as f32
    } else {
        0.32
    }
}

fn global_frame_rms(frames: &[AnalysisFrame]) -> f32 {
    if frames.is_empty() {
        return 0.0001;
    }
    (frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32).max(0.0001)
}

fn post_peak_valley_risk(frames: &[AnalysisFrame], peak_time: f64, valley_time: f64) -> f32 {
    if frames.is_empty() || valley_time <= peak_time {
        return 0.0;
    }
    let peak = average_frame_rms(frames, peak_time - 0.8, peak_time + 0.8);
    let valley = average_frame_rms(frames, valley_time - 0.9, valley_time + 0.9);
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let drop = ((peak - valley) / global.max(0.0001)).clamp(0.0, 2.0) / 2.0;
    let still_loud = (valley / global.max(0.0001)).clamp(0.0, 1.6) / 1.6;
    (drop * 0.62 + still_loud * 0.38).clamp(0.0, 1.0)
}

fn music_map_entry_safe_confidence(
    point: &MusicMixPoint,
    sections: &MusicSectionAnalysis,
    duration_seconds: f64,
) -> f32 {
    let tail_penalty = music_map_tail_risk(sections, point.time_seconds, duration_seconds) * 0.34;
    let position = (point.time_seconds / duration_seconds.max(1.0)).clamp(0.0, 1.0) as f32;
    let early_penalty = if position < 0.10 { 0.10 } else { 0.0 };
    (point.confidence * 0.40
        + point.vocal_safety * 0.18
        + point.perceptual_score * 0.18
        + point.phrase_grid_fit * 0.16
        + point.emotional_continuity * 0.08
        - tail_penalty
        - early_penalty)
        .clamp(0.0, 1.0)
}

fn music_map_exit_safe_confidence(
    point: &MusicMixPoint,
    sections: &MusicSectionAnalysis,
    duration_seconds: f64,
) -> f32 {
    let tail_bonus =
        (music_map_tail_risk(sections, point.time_seconds, duration_seconds) * 0.10).min(0.08);
    (point.confidence * 0.44
        + point.vocal_safety * 0.16
        + point.perceptual_score * 0.16
        + point.vocal_handoff_score * 0.14
        + point.phrase_closure * 0.10
        + tail_bonus)
        .clamp(0.0, 1.0)
}

fn music_map_tail_risk(
    sections: &MusicSectionAnalysis,
    time_seconds: f64,
    duration_seconds: f64,
) -> f32 {
    if duration_seconds <= 0.0 || time_seconds > duration_seconds * 0.88 {
        return 0.66;
    }
    if let Some(outro) = sections.outro.as_ref() {
        if time_seconds >= outro.start_seconds - 2.0 {
            return 0.80;
        }
    }
    for segment in &sections.functional_segments {
        if time_seconds >= segment.start_seconds - 1.5 && time_seconds <= segment.end_seconds + 1.5
        {
            if matches!(
                segment.role,
                MusicFunctionalRole::Outro | MusicFunctionalRole::Silence
            ) {
                return (segment.confidence + 0.24).clamp(0.0, 1.0);
            }
        }
    }
    0.0
}

fn add_near_miss_hook_points(
    map: &mut StageMixMusicMap,
    sections: &MusicSectionAnalysis,
    tempo: &MusicTempoAnalysis,
    duration_seconds: f64,
    frames: &[AnalysisFrame],
) {
    for candidate in &sections.highlight_candidates {
        for offset in [2.0_f64, 4.0, 8.0, 12.0] {
            let mut time = candidate.start_seconds - offset;
            if let Some(snapped) = snap_to_nearest_phrase(time, tempo, duration_seconds) {
                if snapped <= candidate.start_seconds + 0.5 {
                    time = snapped;
                }
            }
            if time < 4.0 || time > duration_seconds - 4.0 {
                continue;
            }
            let lift = entry_lift_score(frames, time);
            let grid = stage_phrase_alignment_score(time, tempo);
            let score =
                (candidate.confidence * 0.36 + lift * 0.34 + grid * 0.20 + 0.02).clamp(0.0, 1.0);
            if score >= MUSIC_MAP_MIN_HOOK_CONFIDENCE {
                push_music_map_point(
                    &mut map.hook_start,
                    MusicMapRole::HookStart,
                    time,
                    score,
                    format!(
                        "near-miss 前緣：在高亮候選前 {:.0}s，lift {:.2}，grid {:.2}",
                        offset, lift, grid
                    ),
                    duration_seconds,
                );
            }
        }
    }
}

fn entry_lift_score(frames: &[AnalysisFrame], time_seconds: f64) -> f32 {
    if frames.is_empty() || !time_seconds.is_finite() {
        return 0.0;
    }
    let global = frames.iter().map(|frame| frame.rms).sum::<f32>() / frames.len().max(1) as f32;
    let before = average_frame_rms(frames, time_seconds - 2.4, time_seconds - 0.45);
    let after = average_frame_rms(frames, time_seconds + 0.45, time_seconds + 2.4);
    ((after - before) / global.max(0.0001)).clamp(0.0, 1.8) as f32 / 1.8
}

fn select_music_map_human_checks(
    map: &StageMixMusicMap,
    duration_seconds: f64,
) -> Vec<MusicMapHumanCheck> {
    let mut checks = Vec::new();
    if let Some(point) = highest_confidence_point(&map.hook_start) {
        checks.push(MusicMapHumanCheck {
            time_seconds: point.time_seconds,
            question_zh: "這段是主高亮區前緣嗎？".to_owned(),
            why_ask: "v8 只保留少量 debug 校正點，不把使用者變成標註員".to_owned(),
            expected_labels: vec![
                "v".to_owned(),
                "e".to_owned(),
                "x".to_owned(),
                "?".to_owned(),
            ],
        });
    }
    let final_peak = map
        .highlight_peak
        .iter()
        .filter(|point| {
            point.time_seconds >= duration_seconds * 0.78
                && !checks
                    .iter()
                    .any(|check| (check.time_seconds - point.time_seconds).abs() < 5.0)
        })
        .max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    let peak = final_peak.or_else(|| highest_confidence_point(&map.highlight_peak));
    if let Some(point) = peak {
        if !checks
            .iter()
            .any(|check| (check.time_seconds - point.time_seconds).abs() < 5.0)
        {
            checks.push(MusicMapHumanCheck {
                time_seconds: point.time_seconds,
                question_zh: "這個 peak 是高亮中心，還是短爆點/尾段？".to_owned(),
                why_ask: "v8 peak 使用多特徵小窗；人工只看明顯錯誤".to_owned(),
                expected_labels: vec![
                    "v".to_owned(),
                    "e".to_owned(),
                    "x".to_owned(),
                    "?".to_owned(),
                ],
            });
        }
    }
    checks.truncate(MAX_MUSIC_MAP_HUMAN_CHECKS);
    checks
}

fn highest_confidence_point(points: &[MusicMapPoint]) -> Option<&MusicMapPoint> {
    points.iter().max_by(|a, b| {
        a.confidence
            .partial_cmp(&b.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn align_to_next_beat(time_seconds: f64, tempo: &MusicTempoAnalysis) -> f64 {
    let Some(grid) = tempo.beat_grid.as_ref() else {
        return time_seconds;
    };
    if grid.interval_seconds <= 0.0 {
        return time_seconds;
    }
    if time_seconds <= grid.first_beat_seconds {
        return grid.first_beat_seconds;
    }
    let steps = ((time_seconds - grid.first_beat_seconds) / grid.interval_seconds).ceil();
    grid.first_beat_seconds + steps * grid.interval_seconds
}

fn align_to_previous_beat(time_seconds: f64, tempo: &MusicTempoAnalysis) -> f64 {
    let Some(grid) = tempo.beat_grid.as_ref() else {
        return time_seconds;
    };
    if grid.interval_seconds <= 0.0 || time_seconds <= grid.first_beat_seconds {
        return time_seconds;
    }
    let steps = ((time_seconds - grid.first_beat_seconds) / grid.interval_seconds).floor();
    grid.first_beat_seconds + steps * grid.interval_seconds
}

fn downsample_energy_curve(frames: &[AnalysisFrame]) -> Vec<MusicEnergyPoint> {
    if frames.is_empty() {
        return Vec::new();
    }
    let chunk_size = (frames.len() as f64 / MAX_ENERGY_CURVE_POINTS as f64)
        .ceil()
        .max(1.0) as usize;
    frames
        .chunks(chunk_size)
        .filter_map(|chunk| {
            let first = chunk.first()?;
            let rms = chunk.iter().map(|frame| frame.rms).sum::<f32>() / chunk.len().max(1) as f32;
            let peak = chunk.iter().map(|frame| frame.peak).fold(0.0_f32, f32::max);
            Some(MusicEnergyPoint {
                time_seconds: first.time_seconds,
                rms,
                peak,
            })
        })
        .collect()
}

fn downsample_spectrum_curve(frames: &[AnalysisFrame]) -> Vec<MusicSpectrumPoint> {
    let sampled = frames
        .iter()
        .filter(|frame| frame.spectrum_sampled)
        .collect::<Vec<_>>();
    if sampled.is_empty() {
        return Vec::new();
    }

    let chunk_size = (sampled.len() as f64 / MAX_SPECTRUM_CURVE_POINTS as f64)
        .ceil()
        .max(1.0) as usize;
    sampled
        .chunks(chunk_size)
        .filter_map(|chunk| {
            let first = chunk.first()?;
            let mut bands = [0_u8; 8];
            for (band_index, output) in bands.iter_mut().enumerate() {
                let average = chunk
                    .iter()
                    .map(|frame| frame.spectrum[band_index])
                    .sum::<f32>()
                    / chunk.len().max(1) as f32;
                *output = (average.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            Some(MusicSpectrumPoint {
                time_seconds: first.time_seconds,
                bands,
            })
        })
        .collect()
}

fn amplitude_to_db(value: f32) -> f32 {
    if value <= 0.000001 {
        -120.0
    } else {
        (20.0 * value.max(0.000001).log10()).clamp(-120.0, 12.0)
    }
}

fn unix_seconds_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod analysis_model_tests {
    use super::*;

    fn test_frame(time_seconds: f64, rms: f32, peak: f32, chroma: [f32; 12]) -> AnalysisFrame {
        AnalysisFrame {
            time_seconds,
            rms,
            peak,
            chroma,
            spectrum: [0.0; 8],
            spectrum_sampled: true,
        }
    }

    #[test]
    fn loudness_proxy_gates_silent_tail() {
        let mut frames = Vec::new();
        for index in 0..12 {
            frames.push(test_frame(index as f64 * 0.5, 0.20, 0.25, [0.0; 12]));
        }
        for index in 12..28 {
            frames.push(test_frame(index as f64 * 0.5, 0.001, 0.002, [0.0; 12]));
        }

        let loudness = estimate_loudness_analysis(&frames, 0.08, 0.25, 0.27);

        assert!(loudness.integrated_lufs > rms_to_lufs_proxy(0.08));
        assert_eq!(loudness.true_peak, 0.27);
    }

    #[test]
    fn harmonic_analysis_detects_major_triad_key() {
        let mut c_major = [0.0_f32; 12];
        c_major[0] = 0.52;
        c_major[4] = 0.28;
        c_major[7] = 0.20;
        let frames = (0..12)
            .map(|index| test_frame(index as f64, 0.12, 0.18, c_major))
            .collect::<Vec<_>>();

        let harmonic = estimate_harmonic_analysis(&frames);

        assert_eq!(harmonic.key_index, Some(0));
        assert_eq!(harmonic.scale.as_deref(), Some("major"));
        assert!(harmonic.confidence > 0.05);
    }

    #[test]
    fn spectrum_analysis_separates_low_and_high_frequency_energy() {
        let sine = |frequency: f64| {
            (0..ANALYSIS_FRAME_SIZE)
                .map(|index| {
                    let phase = std::f64::consts::TAU * frequency * index as f64 / 48_000.0;
                    phase.sin() as f32
                })
                .collect::<Vec<_>>()
        };
        let low = estimate_frame_spectrum(&sine(110.0), 48_000);
        let high = estimate_frame_spectrum(&sine(3520.0), 48_000);

        assert!(low[1] > 0.90);
        assert!(low[1] > low[6]);
        assert!(high[6] > 0.90);
        assert!(high[6] > high[1]);
    }

    #[test]
    fn spectrum_curve_keeps_sampled_silence_and_quantizes_band_shape() {
        let mut active = test_frame(0.0, 0.2, 0.4, [0.0; 12]);
        active.spectrum = [1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let silent = test_frame(0.1, 0.0, 0.0, [0.0; 12]);
        let curve = downsample_spectrum_curve(&[active, silent]);

        assert_eq!(curve.len(), 2);
        assert_eq!(curve[0].bands[0], 255);
        assert_eq!(curve[0].bands[1], 128);
        assert_eq!(curve[1].bands, [0; 8]);
    }

    #[test]
    fn downbeat_grid_prefers_accented_bar_phase() {
        let interval = 0.5;
        let frames = (0..64)
            .map(|index| test_frame(index as f64 * interval, 0.10, 0.12, [0.0; 12]))
            .collect::<Vec<_>>();
        let onset = (0..64)
            .map(|index| {
                if index % 4 == 2 {
                    1.0
                } else if index % 2 == 0 {
                    0.25
                } else {
                    0.02
                }
            })
            .collect::<Vec<_>>();

        let grid = estimate_downbeat_grid(&frames, &onset, 0.0, interval, 32.0).unwrap();

        assert!((grid.first_downbeat_seconds - 1.0).abs() <= 0.01);
        assert!(grid.confidence > 0.10);
    }

    fn test_section_curves() -> MusicSectionCurveAnalysis {
        let mut chorusness = Vec::new();
        let mut recurrence = Vec::new();
        let mut novelty = Vec::new();
        let mut boundary = Vec::new();
        for second in 0..=70 {
            let time = second as f64;
            let stable_highlight = (32..=39).contains(&second);
            chorusness.push(MusicCurvePoint {
                time_seconds: time,
                value: if stable_highlight { 0.86 } else { 0.22 },
            });
            recurrence.push(MusicCurvePoint {
                time_seconds: time,
                value: if stable_highlight { 0.78 } else { 0.18 },
            });
            novelty.push(MusicCurvePoint {
                time_seconds: time,
                value: if second == 31 { 0.68 } else { 0.12 },
            });
            boundary.push(MusicCurvePoint {
                time_seconds: time,
                value: if second == 31 { 0.62 } else { 0.10 },
            });
        }

        MusicSectionCurveAnalysis {
            hop_seconds: 1.0,
            chorusness,
            boundary,
            boundary_candidates: vec![MusicBoundaryCandidate {
                time_seconds: 31.0,
                confidence: 0.62,
                reason: "test boundary".to_owned(),
            }],
            structure: MusicStructureAnalysis {
                recurrence,
                novelty,
                novelty_boundaries: vec![MusicBoundaryCandidate {
                    time_seconds: 31.0,
                    confidence: 0.68,
                    reason: "test novelty".to_owned(),
                }],
            },
        }
    }

    fn test_highlight_candidate() -> MusicSectionCandidate {
        MusicSectionCandidate {
            start_seconds: 20.0,
            end_seconds: 50.0,
            confidence: 0.82,
            reason: "test chorusness".to_owned(),
            scores: MusicSectionCandidateScores {
                total: 0.82,
                chorusness: 0.72,
                repetition: 0.70,
                energy: 0.64,
                contrast: 0.54,
                boundary: 0.58,
                position: 0.70,
                density: 0.62,
                duration: 0.88,
                segment_wholeness: 0.70,
                perceptual: 0.66,
                structural_recurrence: 0.74,
                structural_novelty: 0.52,
            },
        }
    }

    fn test_manifest_with_analyzer_version(
        analyzer_version: u32,
        media_file_size: u64,
    ) -> MusicAnalysisManifest {
        MusicAnalysisManifest {
            schema_version: MUSIC_ANALYSIS_SCHEMA_VERSION,
            analyzer_version,
            media_file_size,
            updated_unix_seconds: 0,
            duration_seconds: 80.0,
            sample_rate: 48_000,
            channels: 2,
            loudness: MusicLoudnessAnalysis {
                rms: 0.12,
                peak: 0.24,
                rms_db: -18.0,
                peak_db: -9.0,
                integrated_lufs: -17.0,
                short_term_lufs: -16.0,
                true_peak: 0.25,
                true_peak_db: -8.8,
            },
            harmonic: MusicHarmonicAnalysis::default(),
            tempo: MusicTempoAnalysis {
                bpm: Some(120.0),
                confidence: 0.8,
                beat_grid: None,
                downbeat_grid: None,
                tempo_map: Vec::new(),
            },
            sections: MusicSectionAnalysis {
                intro: None,
                outro: None,
                highlight_candidates: Vec::new(),
                functional_segments: Vec::new(),
                segment_tempo: Vec::new(),
                structure: MusicStructureAnalysis::default(),
            },
            mix_points: MusicMixPointAnalysis {
                mix_in: Vec::new(),
                mix_out: Vec::new(),
            },
            music_map: StageMixMusicMap::default(),
            section_curves: MusicSectionCurveAnalysis {
                hop_seconds: 1.0,
                chorusness: Vec::new(),
                boundary: Vec::new(),
                boundary_candidates: Vec::new(),
                structure: MusicStructureAnalysis::default(),
            },
            energy_curve: Vec::new(),
            spectrum_curve: Vec::new(),
        }
    }

    #[test]
    fn manifest_current_check_rejects_old_analyzer_and_size_mismatch() {
        let current = test_manifest_with_analyzer_version(MUSIC_ANALYSIS_ANALYZER_VERSION, 42);
        let old = test_manifest_with_analyzer_version(MUSIC_ANALYSIS_ANALYZER_VERSION - 1, 42);

        assert!(music_analysis_manifest_is_current(&current, Some(42)));
        assert!(!music_analysis_manifest_is_current(&old, Some(42)));
        assert!(!music_analysis_manifest_is_current(&current, Some(43)));
        assert!(music_analysis_manifest_is_current(&current, None));
    }

    fn test_attention_candidate(
        start_seconds: f64,
        end_seconds: f64,
        confidence: f32,
        chorusness: f32,
        repetition: f32,
        recurrence: f32,
    ) -> MusicSectionCandidate {
        MusicSectionCandidate {
            start_seconds,
            end_seconds,
            confidence,
            reason: "test attention".to_owned(),
            scores: MusicSectionCandidateScores {
                total: confidence,
                chorusness,
                repetition,
                energy: 0.58,
                contrast: 0.48,
                boundary: 0.52,
                position: 0.70,
                density: 0.58,
                duration: 0.82,
                segment_wholeness: (chorusness * 0.70 + recurrence * 0.30).clamp(0.0, 1.0),
                perceptual: (chorusness * 0.55 + repetition * 0.45).clamp(0.0, 1.0),
                structural_recurrence: recurrence,
                structural_novelty: 0.44,
            },
        }
    }

    #[test]
    fn music_map_peak_prefers_stable_highlight_window_over_short_spike() {
        let mut frames = Vec::new();
        for second in 0..=70 {
            let rms = if second == 25 {
                0.96
            } else if (32..=39).contains(&second) {
                0.46
            } else {
                0.12
            };
            frames.push(test_frame(second as f64, rms, rms * 1.12, [0.0; 12]));
        }
        let candidate = test_highlight_candidate();
        let section_curves = test_section_curves();

        let peak = refined_music_map_peak_time(&candidate, &frames, &section_curves, 70.0);

        assert!(
            (32.0..=39.0).contains(&peak),
            "peak should land in stable highlight body, got {peak}"
        );
    }

    #[test]
    fn music_map_attention_span_prefers_complete_highlight_over_short_spike() {
        let mut frames = Vec::new();
        for second in 0..=80 {
            let rms = if second == 18 {
                0.96
            } else if (42..=51).contains(&second) {
                0.44
            } else {
                0.11
            };
            frames.push(test_frame(second as f64, rms, rms * 1.12, [0.0; 12]));
        }
        let mut chorusness = Vec::new();
        let mut recurrence = Vec::new();
        let mut novelty = Vec::new();
        let mut boundary = Vec::new();
        for second in 0..=80 {
            let time = second as f64;
            let stable_highlight = (42..=51).contains(&second);
            chorusness.push(MusicCurvePoint {
                time_seconds: time,
                value: if stable_highlight { 0.88 } else { 0.18 },
            });
            recurrence.push(MusicCurvePoint {
                time_seconds: time,
                value: if stable_highlight { 0.82 } else { 0.14 },
            });
            novelty.push(MusicCurvePoint {
                time_seconds: time,
                value: if second == 40 { 0.70 } else { 0.08 },
            });
            boundary.push(MusicCurvePoint {
                time_seconds: time,
                value: if second == 40 { 0.66 } else { 0.08 },
            });
        }
        let section_curves = MusicSectionCurveAnalysis {
            hop_seconds: 1.0,
            chorusness,
            boundary,
            boundary_candidates: vec![MusicBoundaryCandidate {
                time_seconds: 40.0,
                confidence: 0.66,
                reason: "test boundary".to_owned(),
            }],
            structure: MusicStructureAnalysis {
                recurrence,
                novelty,
                novelty_boundaries: vec![MusicBoundaryCandidate {
                    time_seconds: 40.0,
                    confidence: 0.70,
                    reason: "test novelty".to_owned(),
                }],
            },
        };
        let attention_context = music_map_attention_context(&frames, &section_curves, 80.0);
        let spike_candidate = test_attention_candidate(12.0, 24.0, 0.86, 0.22, 0.18, 0.14);
        let stable_candidate = test_attention_candidate(36.0, 58.0, 0.74, 0.82, 0.76, 0.80);

        let spike_span = music_map_highlight_span_from_candidate(
            &spike_candidate,
            &[],
            &frames,
            &section_curves,
            &attention_context,
            80.0,
        )
        .expect("spike candidate span");
        let stable_span = music_map_highlight_span_from_candidate(
            &stable_candidate,
            &[],
            &frames,
            &section_curves,
            &attention_context,
            80.0,
        )
        .expect("stable candidate span");

        assert!(
            stable_span.confidence > spike_span.confidence,
            "stable confidence {} should beat spike confidence {}",
            stable_span.confidence,
            spike_span.confidence
        );
    }

    #[test]
    fn music_map_highlight_span_bounds_snap_to_nearby_strong_boundaries() {
        let mut chorusness = Vec::new();
        let mut boundary = Vec::new();
        for second in 0..=90 {
            let time = second as f64;
            chorusness.push(MusicCurvePoint {
                time_seconds: time,
                value: if (28..=60).contains(&second) {
                    0.82
                } else {
                    0.18
                },
            });
            boundary.push(MusicCurvePoint {
                time_seconds: time,
                value: if second == 28 || second == 60 {
                    0.76
                } else {
                    0.08
                },
            });
        }
        let section_curves = MusicSectionCurveAnalysis {
            hop_seconds: 1.0,
            chorusness,
            boundary,
            boundary_candidates: vec![
                MusicBoundaryCandidate {
                    time_seconds: 28.0,
                    confidence: 0.76,
                    reason: "test start boundary".to_owned(),
                },
                MusicBoundaryCandidate {
                    time_seconds: 60.0,
                    confidence: 0.78,
                    reason: "test end boundary".to_owned(),
                },
            ],
            structure: MusicStructureAnalysis::default(),
        };
        let candidate = test_attention_candidate(31.0, 58.0, 0.76, 0.82, 0.74, 0.70);

        let (start, end, note) =
            music_map_highlight_span_bounds(&candidate, &[], &section_curves, 90.0)
                .expect("span bounds");

        assert_eq!(start, 28.0);
        assert_eq!(end, 60.0);
        assert_eq!(note, "貼齊強段落邊界");
    }

    #[test]
    fn music_map_v8_builds_runtime_gated_highlight_span() {
        let frames = (0..=70)
            .map(|second| {
                let rms = if (32..=39).contains(&second) {
                    0.46
                } else {
                    0.12
                };
                test_frame(second as f64, rms, rms * 1.12, [0.0; 12])
            })
            .collect::<Vec<_>>();
        let candidate = test_highlight_candidate();
        let sections = MusicSectionAnalysis {
            intro: None,
            outro: None,
            highlight_candidates: vec![candidate],
            functional_segments: vec![MusicFunctionalSegment {
                start_seconds: 30.0,
                end_seconds: 42.0,
                role: MusicFunctionalRole::Chorus,
                confidence: 0.84,
                reason: "test chorus".to_owned(),
            }],
            segment_tempo: Vec::new(),
            structure: MusicStructureAnalysis::default(),
        };
        let section_curves = test_section_curves();
        let map = estimate_stage_mix_music_map(
            &sections,
            &MusicMixPointAnalysis {
                mix_in: Vec::new(),
                mix_out: Vec::new(),
            },
            &MusicTempoAnalysis {
                bpm: Some(120.0),
                confidence: 0.8,
                beat_grid: None,
                downbeat_grid: None,
                tempo_map: Vec::new(),
            },
            70.0,
            &section_curves,
            &frames,
        );

        assert!(!map.debug_only);
        assert_eq!(
            map.human_check_queue.len().min(2),
            map.human_check_queue.len()
        );
        let span = map.highlight_span.first().expect("v8 highlight span");
        assert!(span.start_seconds >= 30.0);
        assert!(span.end_seconds <= 42.0);
        assert!(span.peak_seconds >= span.start_seconds);
        assert!(span.peak_seconds <= span.end_seconds);
        assert!(
            map.highlight_peak
                .iter()
                .any(|point| { (point.time_seconds - span.peak_seconds).abs() <= 0.001 })
        );
    }
}
