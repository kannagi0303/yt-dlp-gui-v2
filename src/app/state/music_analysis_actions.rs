use super::*;
use crate::app::music_mix_timeline::{MusicMixFrameClock, MusicMixFrameCount, MusicMixOutputFrame};
use crate::app::music_segment_selector::{
    self, PICK_MIN_CONFIDENCE as MUSIC_STAGE_PICK_MIN_CONFIDENCE,
    PRESENCE_MAX_SECONDS as MUSIC_STAGE_PRESENCE_MAX_SECONDS,
};
use crate::app::music_stream::MusicMixRenderMode;
use crate::app::state::music_runtime::{
    MusicChorusFallbackStage, MusicChorusHandoffBridge, MusicStageCueMemoryEntry,
    MusicStageCueMemoryStore, MusicStageMixExecutionRoute,
};
use std::sync::{Mutex, OnceLock};

const MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS: f64 = 0.12;
const MUSIC_CHORUS_REALTIME_FALLBACK_FADE_SECONDS: f64 = 0.68;
const MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS: f64 = 8.0;
const MUSIC_CHORUS_TRANSITION_FALLBACK_SECONDS: f64 = 8.8;
const MUSIC_CHORUS_TRANSITION_MIN_SECONDS: f64 = 5.2;
const MUSIC_CHORUS_STREAM_MIX_MIN_SECONDS: f64 = 0.72;
const MUSIC_CHORUS_STREAM_MIX_IDEAL_MIN_SECONDS: f64 = 1.2;
const MUSIC_CHORUS_STREAM_MIX_COMPACT_MAX_SECONDS: f64 = 4.8;
const MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS: f64 = 0.9;
const MUSIC_CHORUS_MIX_CAPSULE_IDEAL_MIN_SECONDS: f64 = 1.6;
const MUSIC_CHORUS_MIX_CAPSULE_MAX_SECONDS: f64 = 4.2;
const MUSIC_CHORUS_TRANSITION_MAX_SECONDS: f64 = 13.4;
const MUSIC_CHORUS_TRANSITION_MIN_BEATS: f64 = 12.0;
const MUSIC_CHORUS_TRANSITION_MAX_BEATS: f64 = 22.0;
const MUSIC_CHORUS_BEAT_SNAP_WINDOW_SECONDS: f64 = 0.95;
const MUSIC_CHORUS_B_TEMPO_MATCH_MIN_RATE: f64 = 0.976;
const MUSIC_CHORUS_B_TEMPO_MATCH_MAX_RATE: f64 = 1.024;
const MUSIC_CHORUS_A_TEMPO_MATCH_MIN_RATE: f64 = 0.986;
const MUSIC_CHORUS_A_TEMPO_MATCH_MAX_RATE: f64 = 1.014;
const MUSIC_CHORUS_TEMPO_MATCH_MIN_GAP: f64 = 0.006;
const MUSIC_CHORUS_PREVIEW_PREPARE_LEAD_SECONDS: f64 = 16.0;
const MUSIC_CHORUS_SHORT_TAIL_SEGMENT_SECONDS: f64 = 4.8;
const MUSIC_CHORUS_TAIL_DIRECT_HANDOFF_SECONDS: f64 = 24.0;
const MUSIC_CHORUS_TAIL_SILENCE_LOOKAHEAD_SECONDS: f64 = 16.0;
const MUSIC_CHORUS_TAIL_LYRIC_GAP_SECONDS: f64 = 4.8;
const MUSIC_CHORUS_TAIL_LYRIC_GRACE_SECONDS: f64 = 2.8;
const MUSIC_CHORUS_PLAIN_HANDOFF_STOP_GRACE_SECONDS: f64 = 0.85;
const MUSIC_RADIO_CUE_WAIT_FOR_PREVIEW_HOLD_SECONDS: f64 = 1.4;
const MUSIC_RADIO_CUE_READY_MIN_LEAD_SECONDS: f64 = 0.85;
const MUSIC_RADIO_CUE_READY_MAX_CUE_WINDOW_SECONDS: f64 = 5.0;
const MUSIC_STAGE_SHORT_HIGHLIGHT_READY_HOLD_SECONDS: f64 = 2.4;
const MUSIC_STAGE_SHORT_HIGHLIGHT_MAX_EXTENSION_SECONDS: f64 = 9.0;
const MUSIC_RADIO_CUE_PREPARE_TIMEOUT_SECONDS: f64 = 14.0;
const MUSIC_TRANSITION_CACHE_WAIT_HOLD_SECONDS: f64 = 4.0;
const MUSIC_PREPARED_MIX_START_EARLY_HOLD_SECONDS: f64 = 0.035;
const MUSIC_PREPARED_MIX_LATE_TRIM_MAX_SECONDS: f64 = 1.25;
const MUSIC_PREPARED_MIX_LATE_REPLAN_MIN_REMAINING_SECONDS: f64 = 4.0;
const MUSIC_PREPARED_MIX_LATE_REPLAN_PAD_SECONDS: f64 = 0.75;
const MUSIC_CHORUS_LYRIC_SNAP_WINDOW_SECONDS: f64 = 1.5;
const MUSIC_CHORUS_LYRIC_START_LEAD_SECONDS: f64 = 0.08;
const MUSIC_CHORUS_LYRIC_END_LEAD_SECONDS: f64 = 0.16;
const MUSIC_STAGE_ENTRY_MAX_PREROLL_RATIO: f64 = 1.25;
const MUSIC_STAGE_ENTRY_MAX_PREROLL_SECONDS: f64 = 12.0;
const MUSIC_CHORUS_A_TEMPO_LOCK_GAP: f64 = 0.055;
const MUSIC_CHORUS_A_TEMPO_LOCK_MAX_DRIFT: f64 = 0.0025;
const MUSIC_CHORUS_A_TEMPO_LOCK_HALF_DOUBLE_GAP: f64 = 0.018;
const MUSIC_CHORUS_A_TEMPO_LOCK_HALF_DOUBLE_MAX_DRIFT: f64 = 0.0015;
const MUSIC_STAGE_LOCAL_TEMPO_RADIUS_SECONDS: f64 = 18.0;
const MUSIC_STAGE_LOCAL_TEMPO_MIN_CONFIDENCE: f32 = 0.20;
const MUSIC_STAGE_LOCAL_TEMPO_BLEND_CONFIDENCE: f32 = 0.38;
const MUSIC_STAGE_LOCAL_TEMPO_STRONG_CONFIDENCE: f32 = 0.62;
const MUSIC_STAGE_MAP_SPAN_RUNTIME_LOG_DELTA_SECONDS: f64 = 0.75;
const MUSIC_STAGE_BPM_DISPLAY_JITTER_BPM: f32 = 1.25;
const MUSIC_STAGE_BPM_DISPLAY_ANIMATE_BPM: f32 = 3.0;
const MUSIC_STAGE_BPM_DISPLAY_ANIMATION_SECONDS: f64 = 0.72;
const MUSIC_STAGE_HIGHLIGHT_HEAD_SNAP_WINDOW_SECONDS: f64 = 6.0;
const MUSIC_STAGE_CUE_MEMORY_MAX_ENTRIES: usize = 512;
const MUSIC_STAGE_CUE_RUNWAY_SAFETY_SECONDS: f64 = 1.35;
const MUSIC_STAGE_POST_PROMOTE_MIN_DWELL_SECONDS: f64 = 34.0;
// Stability-first Stage Mix path.  When enabled, Stage Mix keeps Radio Cue's
// musical timing, but uses a callback-owned B-preview crossfade instead of a
// pre-rendered A->[mix]->B capsule.  This removes the multi-clock handoff
// source of A->MIX / MIX->B jumps while we keep the feature shippable.
const MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX: bool = true;
// Bold Stage Chain reset: do not promote a finite preview deck into the main
// player.  Instead, use the preview/cue analysis only to choose the next start
// point, then hand off to a real B playback stream with its own decoder.  This
// trades DJ-style overlap for the invariant we actually need first:
// Track(A) -> Mix(A,B) -> Track(B) must always continue as a full stream.
const MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY: bool = true;
// Stage Chain owns the full Track(A)->handoff->Track(B) route.  While this
// route is enabled, preview workers are planning hints only and must not
// prepare/arm finite B decks, because finite preview ownership was the old
// source of stale promote/late-trim interference.  Keep analysis/cue picking,
// but leave actual playback to the real stream handoff.
const MUSIC_STAGE_CHAIN_DIRECT_STREAM_DIRECTOR: bool = true;
const MUSIC_STAGE_LITE_TRANSITION_MIN_SECONDS: f64 = 2.8;
const MUSIC_STAGE_LITE_TRANSITION_MAX_SECONDS: f64 = 3.8;
// Stage Mix Lite promotes the decoded B preview deck into the main playback
// deck.  The deck must therefore contain more than the short musical overlap,
// otherwise playback can stop around highlight_end before the next cue fires.
const MUSIC_STAGE_LITE_PROMOTED_DECK_TARGET_SECONDS: f64 = 96.0;
const MUSIC_STAGE_LITE_PROMOTED_DECK_MIN_SECONDS: f64 = 48.0;
// Stage Chain invariant: when B becomes the next Track segment, it must
// start from a durable main-body runway, not a tail/final/outro fragment.
// If a late highlight would promote B near the song tail, pull B's entry
// earlier so the chain can continue as Track(A)->Mix(A,B)->Track(B).
const MUSIC_STAGE_CHAIN_ENTRY_PULLBACK_MIN_REMAINING_SECONDS: f64 = 84.0;
const MUSIC_STAGE_CHAIN_ENTRY_PULLBACK_SONG_SHARE: f64 = 0.48;
// Direct Stream handoff no longer needs a finite promoted preview deck.  Keep
// enough runway for the new Track(B) to breathe, but do not drag B all the way
// back to the old preview-deck safety window.  This makes the handoff land
// closer to the selected musical entry while preserving the chain invariant.
const MUSIC_STAGE_CHAIN_DIRECT_ENTRY_MIN_REMAINING_SECONDS: f64 = 54.0;
const MUSIC_STAGE_CHAIN_DIRECT_ENTRY_SONG_SHARE: f64 = 0.34;
// Radio Cue can use the same real-stream handoff as automatic Stage Chain.
// Do not wait for a finite B preview deck when the direct director owns
// playback; arm a short visible cue window and let Track(B)'s real decoder
// take over at the handoff.
const MUSIC_STAGE_CHAIN_DIRECT_RADIO_CUE: bool = true;
// Direct Stream handoff no longer needs a finite preview runway, so the next
// step is to make B's real stream entry sound intentional instead of merely
// safe.  After the runway guard chooses a valid B_start, snap it to a nearby
// lyric or functional-section boundary when that does not break the runway.
const MUSIC_STAGE_CHAIN_DIRECT_ENTRY_ANCHOR_WINDOW_SECONDS: f64 = 3.2;
const MUSIC_STAGE_CHAIN_DIRECT_ENTRY_PULLBACK_ANCHOR_WINDOW_SECONDS: f64 = 8.0;
// Direct Stream can start B from a real decoder, but it should not choose a
// final lyric/outro/silence tail as Track(B)'s new body.  If analysis/lyrics
// reveal a long quiet tail, pull B's entry back into the last stable body zone
// before applying lyric/section anchoring.
const MUSIC_STAGE_CHAIN_DIRECT_TAIL_ENTRY_MIN_REMAINING_SECONDS: f64 = 72.0;
const MUSIC_STAGE_CHAIN_DIRECT_TRAILING_SILENCE_MIN_SECONDS: f64 = 7.5;
const MUSIC_STAGE_CHAIN_DIRECT_LAST_LYRIC_BACKOFF_SECONDS: f64 = 26.0;
const MUSIC_STAGE_CHAIN_DIRECT_TAIL_SECTION_BACKOFF_SECONDS: f64 = 20.0;
// Some idol MVs keep real container duration after the actual music body has
// already ended.  Functional sections and lyrics may miss that dead-air tail,
// so Direct Stream also reads the analysis energy curve and protects both A's
// exit point and B's entry point from low-energy trailing silence.
const MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_MIN_SECONDS: f64 = 5.5;
const MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_ENTRY_BACKOFF_SECONDS: f64 = 24.0;
const MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_EXIT_GRACE_SECONDS: f64 = 1.2;
const MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_RELATIVE_RMS: f32 = 0.055;
const MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_PEAK_RMS: f32 = 0.032;
const MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_MIN_RMS: f32 = 0.0018;
// Do not let Direct Stage Chain plan from a final highlight/body tail even when
// the analyzer marks that tail as a strong highlight.  This is intentionally
// independent from the energy curve: some MVs keep noisy/low-level ambience in
// the last seconds, so the low-energy tail detector can miss the real musical
// ending.  Treat this as a conservative body fence for Track(A)'s mix-out and
// Track(B)'s entry selection.
const MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_SONG_SHARE: f64 = 0.78;
const MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_MIN_REMAINING_SECONDS: f64 = 42.0;
const MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_OUTRO_BACKOFF_SECONDS: f64 = 4.0;
const MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_TAIL_GRACE_SECONDS: f64 = 2.0;
// Now that Direct Stream owns playback, tempo/pair models can safely shape the
// handoff envelope again.  This does not time-stretch the real Track(B) stream;
// it only chooses a musically appropriate overlap/fade length from the same BPM,
// vocal-safety, harmonic and loudness models that old Prepared Mix used.
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MIN_SECONDS: f64 = 2.35;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MAX_SECONDS: f64 = 5.65;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_CLOSE_GAP: f64 = 0.035;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MEDIUM_GAP: f64 = 0.105;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_WIDE_GAP: f64 = 0.180;
// First safe real-tempo step for Direct Stream: let A do a tiny, eased tempo
// drift during the handoff fade and phase-compensate B's entry by a fraction of
// the modeled incoming rate.  B still becomes a normal real stream after the
// handoff, so this cannot resurrect finite preview/promote instability.
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE: bool = true;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_MIN_CONFIDENCE: f32 = 0.22;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_MIN_RATE_DELTA: f64 = 0.0075;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_ENTRY_FACTOR: f64 = 0.42;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_MAX_ENTRY_SHIFT_SECONDS: f64 = 0.58;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_INCOMING: bool = true;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_INCOMING_MIN_RATE_DELTA: f64 = 0.0065;
// User-facing Beat Match maps from a safe preview-like feather into
// stronger DJ-like tempo cooperation.  0.0 disables the tempo bridge, ~0.35 is
// close to the previous hidden pilot, and 1.0 is intentionally audible but
// still bounded so Direct Stream remains the playback owner.
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_STRENGTH_MIN_MULTIPLIER: f64 = 0.00;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_STRENGTH_MAX_MULTIPLIER: f64 = 3.20;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_INCOMING_SOFT_MAX_DELTA: f64 = 0.006;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_INCOMING_STRONG_MAX_DELTA: f64 = 0.072;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_OUTGOING_SOFT_MAX_DELTA: f64 = 0.003;
const MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_OUTGOING_STRONG_MAX_DELTA: f64 = 0.052;
// User-facing Mix Length controls the audible overlap/fade window after the
// tempo/vocal/harmonic model picks a safe seed.  50% preserves the model
// result, lower values make quick cuts, and higher values make the bridge long
// enough for the tempo feather to be easier to hear.
const MUSIC_STAGE_CHAIN_DIRECT_MIX_LENGTH_MIN_SECONDS: f64 = 1.45;
const MUSIC_STAGE_CHAIN_DIRECT_MIX_LENGTH_MAX_SECONDS: f64 = 9.60;
const MUSIC_STAGE_CHAIN_DIRECT_MIX_LENGTH_SHORT_MULTIPLIER: f64 = 0.58;
const MUSIC_STAGE_CHAIN_DIRECT_MIX_LENGTH_LONG_MULTIPLIER: f64 = 1.92;
// Release-candidate listening point found by long playback tests: Beat Match
// 50% + Mix Length 50% keeps Direct Stream continuity while preserving the
// model-picked overlap length instead of stretching every handoff into a wide
// mix.  When the user leaves B/M/C on this recommended point, Mix Assist may
// make tiny safety adjustments internally.  Setting Mix Assist to 0% keeps the
// sliders literal for lab testing.
const MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_NATURAL: bool = true;
const MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_BRIDGE_SLIDER: f32 = 0.50;
const MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_MIX_SLIDER: f32 = 0.50;
const MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_CURVE_SLIDER: f32 = 0.85;
const MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_SLIDER_EPSILON: f32 = 0.012;
// Stage Chain cue discipline: after a real stream handoff, let the new track
// breathe before planning the next transition.  Also avoid starting a handoff
// so late in A that the mix feels like it fires from the outro/tail.
const MUSIC_STAGE_CHAIN_POST_HANDOFF_BREATHE_SECONDS: f64 = 38.0;
const MUSIC_STAGE_CHAIN_EXIT_TAIL_GUARD_SECONDS: f64 = 16.0;
const MUSIC_CHORUS_STANDARD_STREAM_MAX_SECONDS: f64 = 6.2;
const MUSIC_CHORUS_REWARD_LONG_MIN_SECONDS: f64 = 5.8;
const MUSIC_CHORUS_REWARD_LONG_MAX_SECONDS: f64 = 10.0;
const MUSIC_CHORUS_REWARD_LONG_MIN_PAIR_CONFIDENCE: f32 = 0.18;
const MUSIC_CHORUS_REWARD_LONG_MAX_TEMPO_GAP: f64 = 0.245;
const MUSIC_CHORUS_REWARD_LONG_MIN_VOCAL_SAFETY: f32 = 0.10;
const MUSIC_CHORUS_REWARD_LONG_MIN_CUE_SCORE: f32 = 0.16;
const MUSIC_CHORUS_REWARD_LONG_MIN_SCORE: f32 = 0.285;
const MUSIC_CHORUS_REWARD_LONG_NEUTRAL_VOCAL_SAFETY: f32 = 0.46;
const MUSIC_CHORUS_REWARD_LONG_NEUTRAL_CUE_SCORE: f32 = 0.42;
const MUSIC_CHORUS_HARMONIC_MIN_CONFIDENCE: f32 = 0.14;
const MUSIC_CHORUS_REWARD_LONG_MIN_HARMONIC_SCORE: f32 = 0.30;
const MUSIC_CHORUS_REWARD_LONG_LOUDNESS_DELTA_HARD_LU: f32 = 8.0;
const MUSIC_CHORUS_REWARD_LONG_LOUDNESS_DELTA_SOFT_LU: f32 = 5.0;
const MUSIC_TEMPO_GRID_RATIO_MAX_EFFECTIVE_GAP: f64 = 0.245;
const MUSIC_TEMPO_GRID_COMPLEX_RATIO_PENALTY: f64 = 0.018;
const MUSIC_CHORUS_REWARD_TAIL_EXTENSION_MIN_SECONDS: f64 = 0.28;
const MUSIC_CHORUS_REWARD_TAIL_EXTENSION_MAX_SECONDS: f64 = 5.6;
const MUSIC_CHORUS_REWARD_TAIL_EXTENSION_SONG_END_GUARD_SECONDS: f64 = 0.12;
const MUSIC_CHORUS_REWARD_TAIL_ENERGY_REFERENCE_SECONDS: f64 = 7.5;
const MUSIC_CHORUS_REWARD_TAIL_ENERGY_LOOKAHEAD_SECONDS: f64 = 5.8;
const MUSIC_CHORUS_REWARD_TAIL_ENERGY_DIP_RATIO: f64 = 0.66;
const MUSIC_CHORUS_REWARD_TAIL_ENERGY_DIP_GRACE_SECONDS: f64 = 0.22;
const MUSIC_CHORUS_REWARD_TAIL_ENERGY_MIN_REFERENCE_RMS: f64 = 0.018;
const MUSIC_CHORUS_REWARD_PAYOFF_CLIFF_PHASE_MIN: f64 = 0.74;
const MUSIC_CHORUS_REWARD_PAYOFF_CLIFF_EXTRA_GRACE_SECONDS: f64 = 1.15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MusicMixWindowKind {
    Plain,
    Stream,
    RewardLong,
}

impl MusicMixWindowKind {
    fn key(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Stream => "stream",
            Self::RewardLong => "reward",
        }
    }

    fn detail_label(self) -> &'static str {
        match self {
            Self::Plain => "Plain Mix",
            Self::Stream => "Stream Mix",
            Self::RewardLong => "Reward Long Mix",
        }
    }
}

#[derive(Clone)]
struct CachedMusicAnalysisManifest {
    modified: Option<SystemTime>,
    manifest: crate::app::music_analysis::MusicAnalysisManifest,
}

#[derive(Clone, Copy, Debug)]
struct MusicChorusTempoSplit {
    incoming_rate: f64,
    outgoing_rate: f64,
    b_share: f64,
}

impl MusicChorusTempoSplit {
    fn neutral() -> Self {
        Self {
            incoming_rate: 1.0,
            outgoing_rate: 1.0,
            b_share: 0.5,
        }
    }
}

#[derive(Clone, Debug)]
struct MusicStageChainDirectTempoBridge {
    outgoing_rate: f64,
    incoming_rate: f64,
    entry_shift_seconds: f64,
    note: String,
}

#[derive(Clone, Debug)]
struct MusicChorusHarmonicCompatibility {
    score: f32,
    confidence: f32,
    label: String,
}

#[derive(Clone, Copy, Debug)]
struct MusicChorusStageTempoEstimate {
    bpm: f64,
    confidence: f32,
    local: bool,
}

#[derive(Clone, Copy, Debug)]
struct MusicTempoGridCompatibility {
    adjusted_next_bpm: f64,
    relative_gap: f64,
    effective_gap: f64,
    ratio_numerator: u32,
    ratio_denominator: u32,
}

#[derive(Clone, Debug)]
struct MusicStageHighlightDebugLabel {
    label: String,
    start_seconds: f64,
    end_seconds: f64,
    confidence: f32,
}

#[derive(Clone, Copy, Debug)]
enum MusicChorusStageTempoRole {
    Outgoing,
    Incoming,
}

static MUSIC_ANALYSIS_MANIFEST_CACHE: OnceLock<
    Mutex<HashMap<String, CachedMusicAnalysisManifest>>,
> = OnceLock::new();

fn music_mix_mode_from_flags(
    automix_enabled: bool,
    trim_enabled: bool,
    highlight_enabled: bool,
) -> MusicMixMode {
    if highlight_enabled {
        MusicMixMode::Highlight
    } else if trim_enabled {
        MusicMixMode::SkipQuietEdges
    } else if automix_enabled {
        MusicMixMode::FullSong
    } else {
        MusicMixMode::Off
    }
}

fn music_mix_flags_for_mode(mode: MusicMixMode) -> (bool, bool, bool) {
    (
        mode.enabled(),
        mode == MusicMixMode::SkipQuietEdges,
        mode == MusicMixMode::Highlight,
    )
}

fn music_full_song_playback_range(duration_seconds: f64) -> Option<(f64, f64)> {
    if duration_seconds.is_finite() && duration_seconds >= MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
        Some((0.0, duration_seconds))
    } else {
        None
    }
}

fn music_display_playback_range(
    mode: MusicMixMode,
    selected_range: Option<(f64, f64)>,
    active_flow_range: Option<(f64, f64)>,
) -> Option<(f64, f64)> {
    if mode == MusicMixMode::FullSong {
        selected_range
    } else {
        active_flow_range.or(selected_range)
    }
}

fn music_initial_cue_start_policy(
    mode: MusicMixMode,
    selected_start_seconds: f64,
    current_playback_seconds: f64,
    pending_start_seconds: Option<f64>,
) -> (f64, bool) {
    if let Some(pending_start_seconds) = pending_start_seconds {
        return (pending_start_seconds, true);
    }
    if mode == MusicMixMode::FullSong {
        return (
            current_playback_seconds
                .max(selected_start_seconds)
                .max(0.0),
            false,
        );
    }
    (selected_start_seconds, true)
}

fn music_segment_display_mix_window(
    mode: MusicMixMode,
    selected_range: Option<(f64, f64)>,
    segment_start_seconds: f64,
    segment_end_seconds: f64,
    transition_seconds: f64,
) -> Option<(f64, f64)> {
    let (range_floor, end_seconds) = if mode == MusicMixMode::FullSong {
        selected_range?
    } else {
        (segment_start_seconds, segment_end_seconds)
    };
    let end_seconds = end_seconds.max(range_floor);
    let start_seconds = (end_seconds - transition_seconds.max(0.0)).max(range_floor);
    (end_seconds > start_seconds).then_some((start_seconds, end_seconds))
}

fn music_player_aura_timing(
    playback_seconds: f64,
    first_beat_seconds: f64,
    beat_interval_seconds: f64,
    first_downbeat_seconds: Option<f64>,
    downbeat_confidence: f32,
) -> Option<(usize, f32, f32)> {
    if !playback_seconds.is_finite()
        || !first_beat_seconds.is_finite()
        || !beat_interval_seconds.is_finite()
        || beat_interval_seconds <= 0.0
    {
        return None;
    }
    let trusted_downbeat =
        first_downbeat_seconds.filter(|seconds| seconds.is_finite() && downbeat_confidence >= 0.18);
    let origin = trusted_downbeat.unwrap_or(first_beat_seconds);
    let beat_position = ((playback_seconds - origin) / beat_interval_seconds).max(0.0);
    let beat_floor = beat_position.floor();
    let active_beat = (beat_floor as usize) % 4;
    let beat_phase = (beat_position - beat_floor).clamp(0.0, 1.0) as f32;
    let downbeat_strength = if trusted_downbeat.is_some() && active_beat == 0 {
        downbeat_confidence.clamp(0.0, 1.0) * (1.0 - beat_phase).powi(2)
    } else {
        0.0
    };
    Some((active_beat, beat_phase, downbeat_strength))
}

fn music_energy_curve_value_at(
    points: &[crate::app::music_analysis::MusicEnergyPoint],
    playback_seconds: f64,
    track_rms: f32,
) -> f32 {
    if points.is_empty() || !playback_seconds.is_finite() {
        return 0.0;
    }
    let upper = points.partition_point(|point| point.time_seconds < playback_seconds);
    let raw = match (upper.checked_sub(1), points.get(upper)) {
        (Some(lower), Some(next)) => {
            let previous = &points[lower];
            let span = next.time_seconds - previous.time_seconds;
            if span > 0.0 {
                let ratio =
                    ((playback_seconds - previous.time_seconds) / span).clamp(0.0, 1.0) as f32;
                previous.rms + (next.rms - previous.rms) * ratio
            } else {
                next.rms
            }
        }
        (_, Some(next)) => next.rms,
        (Some(lower), None) => points[lower].rms,
        (None, None) => 0.0,
    };
    let reference = points
        .iter()
        .map(|point| point.rms)
        .fold(track_rms.max(0.0001), f32::max);
    (raw.max(0.0) / reference.max(0.0001))
        .sqrt()
        .clamp(0.0, 1.0)
}

fn music_energy_curve_momentum_at(
    points: &[crate::app::music_analysis::MusicEnergyPoint],
    playback_seconds: f64,
    track_rms: f32,
) -> f32 {
    const SAMPLE_RADIUS_SECONDS: f64 = 0.75;
    let before =
        music_energy_curve_value_at(points, playback_seconds - SAMPLE_RADIUS_SECONDS, track_rms);
    let after =
        music_energy_curve_value_at(points, playback_seconds + SAMPLE_RADIUS_SECONDS, track_rms);
    ((after - before) * 2.4).clamp(-1.0, 1.0)
}

fn music_curve_value_at(
    points: &[crate::app::music_analysis::MusicCurvePoint],
    playback_seconds: f64,
) -> f32 {
    if points.is_empty() || !playback_seconds.is_finite() {
        return 0.0;
    }
    let upper = points.partition_point(|point| point.time_seconds < playback_seconds);
    let value = match (upper.checked_sub(1), points.get(upper)) {
        (Some(lower), Some(next)) => {
            let previous = &points[lower];
            let span = next.time_seconds - previous.time_seconds;
            if span > 0.0 {
                let ratio =
                    ((playback_seconds - previous.time_seconds) / span).clamp(0.0, 1.0) as f32;
                previous.value + (next.value - previous.value) * ratio
            } else {
                next.value
            }
        }
        (_, Some(next)) => next.value,
        (Some(lower), None) => points[lower].value,
        (None, None) => 0.0,
    };
    value.clamp(0.0, 1.0)
}

fn music_spectrum_curve_value_at(
    points: &[crate::app::music_analysis::MusicSpectrumPoint],
    playback_seconds: f64,
) -> [f32; 8] {
    if points.is_empty() || !playback_seconds.is_finite() {
        return [0.0; 8];
    }
    let upper = points.partition_point(|point| point.time_seconds < playback_seconds);
    let (previous, next, ratio) = match (upper.checked_sub(1), points.get(upper)) {
        (Some(lower), Some(next)) => {
            let previous = &points[lower];
            let span = next.time_seconds - previous.time_seconds;
            let ratio = if span > 0.0 {
                ((playback_seconds - previous.time_seconds) / span).clamp(0.0, 1.0) as f32
            } else {
                1.0
            };
            (previous, next, ratio)
        }
        (_, Some(next)) => (next, next, 0.0),
        (Some(lower), None) => (&points[lower], &points[lower], 0.0),
        (None, None) => return [0.0; 8],
    };

    std::array::from_fn(|index| {
        let from = f32::from(previous.bands[index]) / 255.0;
        let to = f32::from(next.bands[index]) / 255.0;
        (from + (to - from) * ratio).clamp(0.0, 1.0)
    })
}

fn music_spectrum_peak_hold_at(
    points: &[crate::app::music_analysis::MusicSpectrumPoint],
    playback_seconds: f64,
) -> [f32; 8] {
    const HOLD_SECONDS: f64 = 0.20;
    const RELEASE_SECONDS: f64 = 0.32;
    const LOOKBACK_SECONDS: f64 = HOLD_SECONDS + RELEASE_SECONDS;

    let mut peaks = music_spectrum_curve_value_at(points, playback_seconds);
    if points.is_empty() || !playback_seconds.is_finite() {
        return peaks;
    }

    let start_seconds = (playback_seconds - LOOKBACK_SECONDS).max(0.0);
    let start = points.partition_point(|point| point.time_seconds < start_seconds);
    let end = points.partition_point(|point| point.time_seconds <= playback_seconds);
    for point in &points[start..end] {
        let age = (playback_seconds - point.time_seconds).max(0.0);
        let release_age = (age - HOLD_SECONDS).max(0.0);
        let release = (1.0 - release_age / RELEASE_SECONDS).clamp(0.0, 1.0) as f32;
        for (index, peak) in peaks.iter_mut().enumerate() {
            let candidate = f32::from(point.bands[index]) / 255.0 * release;
            *peak = peak.max(candidate);
        }
    }
    peaks
}

fn music_chroma_signature(
    harmonic: &crate::app::music_analysis::MusicHarmonicAnalysis,
) -> (f32, f32) {
    let mut x = 0.0_f32;
    let mut y = 0.0_f32;
    let mut total = 0.0_f32;
    for (index, weight) in harmonic.chroma.iter().take(12).enumerate() {
        let weight = weight.max(0.0);
        let angle = std::f32::consts::TAU * index as f32 / 12.0;
        x += weight * angle.cos();
        y += weight * angle.sin();
        total += weight;
    }

    if total > 0.000_001 {
        let coherence =
            ((x * x + y * y).sqrt() / total) * (0.35 + harmonic.confidence.clamp(0.0, 1.0) * 0.65);
        if x.abs() + y.abs() > 0.000_001 {
            let hue = (y.atan2(x) / std::f32::consts::TAU).rem_euclid(1.0);
            return (hue, coherence.clamp(0.0, 1.0));
        }
    }

    (
        harmonic
            .key_index
            .map(|key| f32::from(key % 12) / 12.0)
            .unwrap_or(0.58),
        (harmonic.confidence * 0.5).clamp(0.0, 1.0),
    )
}

fn music_player_aura_track_field(
    manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    playback_seconds: f64,
) -> MusicPlayerAuraTrackField {
    let mut field = MusicPlayerAuraTrackField::default();
    field.energy = music_energy_curve_value_at(
        &manifest.energy_curve,
        playback_seconds,
        manifest.loudness.rms,
    );
    field.energy_momentum = music_energy_curve_momentum_at(
        &manifest.energy_curve,
        playback_seconds,
        manifest.loudness.rms,
    );
    field.boundary = music_curve_value_at(&manifest.section_curves.boundary, playback_seconds);
    field.novelty =
        music_curve_value_at(&manifest.section_curves.structure.novelty, playback_seconds);
    field.recurrence = music_curve_value_at(
        &manifest.section_curves.structure.recurrence,
        playback_seconds,
    );
    field.chorusness = music_curve_value_at(&manifest.section_curves.chorusness, playback_seconds);
    (field.section_color_unit, field.section_color_strength) =
        music_player_aura_section_color(manifest, playback_seconds, field);
    field.spectrum_bands =
        music_spectrum_curve_value_at(&manifest.spectrum_curve, playback_seconds);
    field.spectrum_peaks = music_spectrum_peak_hold_at(&manifest.spectrum_curve, playback_seconds);
    (field.chroma_hue, field.chroma_coherence) = music_chroma_signature(&manifest.harmonic);
    let timing = manifest
        .tempo
        .beat_grid
        .as_ref()
        .map(|grid| (grid.first_beat_seconds, grid.interval_seconds))
        .or_else(|| {
            manifest
                .tempo
                .bpm
                .filter(|bpm| bpm.is_finite() && *bpm > 0.0)
                .map(|bpm| (0.0, 60.0 / f64::from(bpm)))
        });
    if let Some((first_beat_seconds, beat_interval_seconds)) = timing
        && let Some((active_beat, beat_phase, downbeat_strength)) = music_player_aura_timing(
            playback_seconds,
            first_beat_seconds,
            beat_interval_seconds,
            manifest
                .tempo
                .downbeat_grid
                .as_ref()
                .map(|grid| grid.first_downbeat_seconds),
            manifest
                .tempo
                .downbeat_grid
                .as_ref()
                .map(|grid| grid.confidence)
                .unwrap_or(0.0),
        )
    {
        field.bar_phase = ((active_beat as f32 + beat_phase) / 4.0).rem_euclid(1.0);
        field.beat_phase = beat_phase;
        field.downbeat_strength = downbeat_strength;
    }

    field
}

fn music_player_aura_section_color(
    manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    playback_seconds: f64,
    field: MusicPlayerAuraTrackField,
) -> (f32, f32) {
    let Some(segment) = manifest
        .sections
        .functional_segments
        .iter()
        .filter(|segment| {
            segment.start_seconds.is_finite()
                && segment.end_seconds.is_finite()
                && playback_seconds >= segment.start_seconds
                && playback_seconds < segment.end_seconds
        })
        .max_by(|left, right| {
            left.confidence
                .partial_cmp(&right.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    else {
        return (0.5, 0.0);
    };

    let duration = segment.end_seconds - segment.start_seconds;
    if !(3.0..=120.0).contains(&duration) {
        return (0.5, 0.0);
    }

    let structural_evidence = field.boundary.clamp(0.0, 1.0) * 0.20
        + field.novelty.clamp(0.0, 1.0) * 0.20
        + field.recurrence.clamp(0.0, 1.0) * 0.18
        + field.chorusness.clamp(0.0, 1.0) * 0.24
        + field.energy.clamp(0.0, 1.0) * 0.18;
    let strength =
        (segment.confidence.clamp(0.0, 1.0) * (0.48 + structural_evidence * 0.52)).clamp(0.0, 1.0);
    if strength < 0.22 {
        return (0.5, 0.0);
    }

    // Use a role-independent, time-anchored identity. Role names remain useful
    // analysis hints, but cannot make the palette flash when classification is
    // uncertain.
    let start_tick = (segment.start_seconds.max(0.0) * 20.0).round() as u64;
    let end_tick = (segment.end_seconds.max(0.0) * 20.0).round() as u64;
    let mut seed = start_tick ^ end_tick.rotate_left(23) ^ 0xA24B_AED4_963E_E407;
    seed ^= seed >> 30;
    seed = seed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    seed ^= seed >> 27;
    seed = seed.wrapping_mul(0x94D0_49BB_1331_11EB);
    seed ^= seed >> 31;
    ((seed as u32) as f32 / u32::MAX as f32, strength)
}

fn music_player_aura_mix_progress(
    current_output_frame: MusicMixOutputFrame,
    started_output_frame: MusicMixOutputFrame,
    duration_output_frames: MusicMixFrameCount,
) -> f32 {
    let elapsed_frames = current_output_frame.saturating_sub(started_output_frame);
    (elapsed_frames.get() as f32 / duration_output_frames.get().max(1) as f32).clamp(0.0, 1.0)
}

fn music_stage_segment_bpm_from_analysis(
    manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    playback_seconds: Option<f64>,
    display_highlight_range: Option<(f64, f64)>,
) -> Option<f32> {
    music_segment_selector::segment_bpm_from_analysis(
        manifest,
        playback_seconds,
        display_highlight_range,
    )
}

impl AppState {
    pub fn music_current_analysis_manifest(
        &self,
    ) -> Option<crate::app::music_analysis::MusicAnalysisManifest> {
        let item_id = self.music.music_player_current_item_id?;
        self.music_analysis_manifest_for_item(item_id)
    }

    pub fn music_player_aura_display(&self) -> MusicPlayerAuraDisplay {
        let mut display = MusicPlayerAuraDisplay {
            animating: self.music_player_is_playing(),
            ..Default::default()
        };
        let Some(item_id) = self.music.music_player_current_item_id else {
            return display;
        };
        let playback_seconds = self.music_current_playback_seconds().unwrap_or(0.0);

        if let Some(bridge) = self
            .music
            .music_chorus_handoff_bridge
            .as_ref()
            .filter(|bridge| bridge.target_item_id == item_id)
        {
            display.primary_item_id = Some(bridge.control.item_id);
            display.primary = Some(
                self.music_analysis_manifest_for_item(bridge.control.item_id)
                    .map(|manifest| {
                        music_player_aura_track_field(&manifest, bridge.control.playback_seconds())
                    })
                    .unwrap_or_default(),
            );
            display.secondary_item_id = Some(item_id);
            display.secondary = Some(
                self.music_analysis_manifest_for_item(item_id)
                    .map(|manifest| music_player_aura_track_field(&manifest, playback_seconds))
                    .unwrap_or_default(),
            );
            display.mix_progress = music_player_aura_mix_progress(
                bridge.control.output_frame_cursor(),
                bridge.visual_started_output_frame,
                bridge.visual_duration_output_frames,
            );
            return display;
        }

        display.primary_item_id = Some(item_id);
        display.primary = Some(
            self.music_analysis_manifest_for_item(item_id)
                .map(|manifest| music_player_aura_track_field(&manifest, playback_seconds))
                .unwrap_or_default(),
        );

        if let (Some(control), Some(fade)) = (
            self.music.music_playback.as_ref(),
            self.music.music_chorus_fade_out.as_ref(),
        ) && fade.item_id == control.item_id
            && fade.session_id == control.session_id
        {
            display.mix_progress = music_player_aura_mix_progress(
                control.output_frame_cursor(),
                fade.started_output_frame,
                fade.duration_output_frames,
            );
            display.secondary_item_id = fade.next_item_id;
            display.secondary = fade.next_item_id.and_then(|next_item_id| {
                let manifest = self.music_analysis_manifest_for_item(next_item_id)?;
                let next_playback_seconds = fade.next_start_seconds.unwrap_or(0.0)
                    + fade.duration_seconds * f64::from(display.mix_progress);
                Some(music_player_aura_track_field(
                    &manifest,
                    next_playback_seconds,
                ))
            });
        }

        display
    }

    pub fn music_current_analysis_progress(
        &self,
    ) -> Option<crate::app::music_analysis::MusicAnalysisProgressSnapshot> {
        let item_id = self.music.music_player_current_item_id?;
        let item = self.queue_item_by_id(item_id)?;
        let cache_key = item.music_cache_key.trim();
        if cache_key.is_empty() {
            return None;
        }
        let analysis_path = self
            .music_stream_cache_root()
            .join(sanitize_music_cache_key(cache_key))
            .join("analysis.yaml");
        crate::app::music_analysis::music_analysis_progress_for_path(&analysis_path)
    }

    pub fn music_automix_enabled(&self) -> bool {
        self.music.music_automix_enabled
    }

    fn music_mix_next_pending(&self) -> bool {
        self.music.music_automix_enabled && self.music.music_chorus_pending_mix_target.is_some()
    }
    pub fn music_stage_mix_render_mode(&self) -> MusicMixRenderMode {
        self.music.music_mix_render_mode
    }

    pub fn music_stage_mix_render_mode_label(&self) -> &'static str {
        self.music.music_mix_render_mode.label()
    }

    pub fn toggle_music_stage_mix_render_mode(&mut self) {
        self.music.music_mix_render_mode = match self.music.music_mix_render_mode {
            MusicMixRenderMode::HighQualityOffline => MusicMixRenderMode::Streaming,
            MusicMixRenderMode::Streaming => MusicMixRenderMode::HighQualityOffline,
        };
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;
        if let Some(control) = &self.music.music_playback {
            control.clear_crossfade_preview();
        }
        self.last_action = format!(
            "Stage Mix render: {}.",
            self.music.music_mix_render_mode.detail_label()
        );
    }

    pub fn music_mix_next_pending_item_indicator(&self, item_id: QueueItemId) -> bool {
        self.music_mix_next_pending()
            && self
                .music
                .music_chorus_pending_mix_target
                .as_ref()
                .is_some_and(|target| target.target_item_id == item_id)
    }

    pub(super) fn cancel_music_radio_cue_pending(&mut self) -> bool {
        let had_pending = self.music.music_chorus_pending_mix_target.take().is_some();
        if !had_pending {
            return false;
        }

        // Let the normal next-track predictor reclaim prefetch ownership. If
        // the cancelled explicit target differs, the next poll supersedes it.
        self.music.music_prefetch_for_current_item_id = None;
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;
        if self
            .music
            .music_chorus_mix_plan
            .as_ref()
            .is_some_and(|plan| plan.reason.contains("Mix next"))
        {
            self.music.music_chorus_mix_plan = None;
        }
        if let Some(control) = &self.music.music_playback {
            control.clear_crossfade_preview();
        }
        true
    }

    pub(super) fn cancel_music_radio_cue_pending_and_reanchor(&mut self) -> bool {
        let cancelled = self.cancel_music_radio_cue_pending();
        if cancelled {
            self.reanchor_current_music_mode_after_toggle_without_seek();
        }
        cancelled
    }

    pub(super) fn cancel_music_radio_cue_pending_with_message(&mut self, message: &str) -> bool {
        let cancelled = self.cancel_music_radio_cue_pending_and_reanchor();
        if cancelled {
            self.last_action = message.to_owned();
        }
        cancelled
    }

    pub fn music_trim_enabled(&self) -> bool {
        self.music.music_trim_enabled
    }

    pub fn music_chorus_flow_enabled(&self) -> bool {
        self.music.music_chorus_flow_enabled
    }

    pub fn music_stage_direct_tempo_bridge_strength(&self) -> f32 {
        self.music
            .music_stage_direct_tempo_bridge_strength
            .clamp(0.0, 1.0)
    }

    pub fn set_music_stage_direct_tempo_bridge_strength(&mut self, strength: f32) {
        let strength = strength.clamp(0.0, 1.0);
        if (self.music.music_stage_direct_tempo_bridge_strength - strength).abs() < 0.001 {
            return;
        }
        self.music.music_stage_direct_tempo_bridge_strength = strength;
        self.last_action = format!("Stage Chain Beat Match: {:.0}%.", strength * 100.0);
    }

    pub fn music_stage_direct_mix_length(&self) -> f32 {
        self.music.music_stage_direct_mix_length.clamp(0.0, 1.0)
    }

    pub fn set_music_stage_direct_mix_length(&mut self, length: f32) {
        let length = length.clamp(0.0, 1.0);
        if (self.music.music_stage_direct_mix_length - length).abs() < 0.001 {
            return;
        }
        self.music.music_stage_direct_mix_length = length;
        self.last_action = format!("Stage Chain Mix Length: {:.0}%.", length * 100.0);
    }

    pub fn music_stage_direct_mix_curve(&self) -> f32 {
        self.music.music_stage_direct_mix_curve.clamp(0.0, 1.0)
    }

    pub fn set_music_stage_direct_mix_curve(&mut self, curve: f32) {
        let curve = curve.clamp(0.0, 1.0);
        if (self.music.music_stage_direct_mix_curve - curve).abs() < 0.001 {
            return;
        }
        self.music.music_stage_direct_mix_curve = curve;
        self.last_action = format!("Stage Chain Mix Curve: {:.0}%.", curve * 100.0);
    }

    pub fn music_stage_direct_mix_assist(&self) -> f32 {
        self.music.music_stage_direct_mix_assist.clamp(0.0, 1.0)
    }

    pub fn set_music_stage_direct_mix_assist(&mut self, assist: f32) {
        let assist = assist.clamp(0.0, 1.0);
        if (self.music.music_stage_direct_mix_assist - assist).abs() < 0.001 {
            return;
        }
        self.music.music_stage_direct_mix_assist = assist;
        self.last_action = if assist <= 0.005 {
            "Stage Chain Mix Assist: manual sliders.".to_owned()
        } else {
            format!("Stage Chain Mix Assist: {:.0}%.", assist * 100.0)
        };
    }

    fn music_stage_direct_mix_assist_value(&self) -> f64 {
        self.music_stage_direct_mix_assist() as f64
    }

    fn music_stage_direct_mix_length_value(&self) -> f64 {
        self.music_stage_direct_mix_length() as f64
    }

    fn music_stage_direct_mix_curve_value(&self) -> f32 {
        self.music_stage_direct_mix_curve()
    }

    fn music_stage_direct_mix_length_multiplier_for(length: f64) -> f64 {
        music_segment_selector::mix_length_multiplier(
            length,
            music_segment_selector::MusicStageMixLengthMultiplierPolicy {
                short_multiplier: MUSIC_STAGE_CHAIN_DIRECT_MIX_LENGTH_SHORT_MULTIPLIER,
                long_multiplier: MUSIC_STAGE_CHAIN_DIRECT_MIX_LENGTH_LONG_MULTIPLIER,
            },
        )
    }

    fn music_stage_direct_tempo_bridge_strength_value(&self) -> f64 {
        self.music_stage_direct_tempo_bridge_strength() as f64
    }

    fn music_stage_direct_tempo_bridge_strength_multiplier_for(&self, strength: f64) -> f64 {
        music_segment_selector::tempo_bridge_strength_multiplier(
            strength,
            music_segment_selector::MusicStageTempoBridgeStrengthPolicy {
                min_multiplier: MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_STRENGTH_MIN_MULTIPLIER,
                max_multiplier: MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_STRENGTH_MAX_MULTIPLIER,
            },
        )
    }

    fn music_stage_direct_tempo_bridge_rate_bounds_for(
        &self,
        strength: f64,
    ) -> ((f64, f64), (f64, f64)) {
        music_segment_selector::tempo_bridge_rate_bounds(
            strength,
            music_segment_selector::MusicStageTempoBridgeRateBoundsPolicy {
                incoming_soft_max_delta:
                    MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_INCOMING_SOFT_MAX_DELTA,
                incoming_strong_max_delta:
                    MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_INCOMING_STRONG_MAX_DELTA,
                outgoing_soft_max_delta:
                    MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_OUTGOING_SOFT_MAX_DELTA,
                outgoing_strong_max_delta:
                    MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_OUTGOING_STRONG_MAX_DELTA,
            },
        )
    }

    fn music_stage_direct_tempo_bridge_clamp_incoming_rate(&self, rate: f64) -> f64 {
        let (_, incoming_bounds) = self.music_stage_direct_tempo_bridge_rate_bounds_for(
            self.music_stage_direct_tempo_bridge_strength_value(),
        );
        rate.clamp(incoming_bounds.0, incoming_bounds.1)
    }

    fn music_stage_direct_adaptive_natural_enabled(&self) -> bool {
        MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_NATURAL
            && self.music_stage_direct_mix_assist() > 0.005
            && (self.music_stage_direct_tempo_bridge_strength()
                - MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_BRIDGE_SLIDER)
                .abs()
                <= MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_SLIDER_EPSILON
            && (self.music_stage_direct_mix_length() - MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_MIX_SLIDER)
                .abs()
                <= MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_SLIDER_EPSILON
            && (self.music_stage_direct_mix_curve()
                - MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_CURVE_SLIDER)
                .abs()
                <= MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_SLIDER_EPSILON
    }

    pub fn music_auto_transition_enabled(&self) -> bool {
        self.music.music_automix_enabled
            || self.music.music_trim_enabled
            || self.music.music_chorus_flow_enabled
    }

    pub fn music_current_playback_seconds(&self) -> Option<f64> {
        let control = self.music.music_playback.as_ref()?;
        if let Some(fade) =
            self.music.music_chorus_fade_out.as_ref().filter(|fade| {
                fade.item_id == control.item_id && fade.session_id == control.session_id
            })
        {
            let fallback_ratio = || {
                let elapsed_frames = control
                    .output_frame_cursor()
                    .saturating_sub(fade.started_output_frame);
                elapsed_frames.get() as f64 / fade.duration_output_frames.get().max(1) as f64
            };
            // Prepared Mix may trim again inside the callback.  Use the deck's
            // real transition progress so the locked MIX lane reaches B at the
            // same moment as the audio owner instead of jumping on promotion.
            let ratio = if fade.crossfade_preview_started {
                control
                    .crossfade_preview_transition_progress_ratio()
                    .unwrap_or_else(fallback_ratio)
            } else {
                fallback_ratio()
            };
            let (start, end) = if fade.crossfade_preview_started
                && fade.mix_window_end_seconds > fade.mix_window_start_seconds
            {
                (fade.mix_window_start_seconds, fade.mix_window_end_seconds)
            } else {
                (
                    fade.start_playback_seconds,
                    fade.start_playback_seconds + fade.duration_seconds.max(0.0),
                )
            };
            let seconds = start + (end - start).max(0.0) * ratio.clamp(0.0, 1.0);
            if seconds.is_finite() && seconds >= 0.0 {
                return Some(seconds);
            }
        }

        Some(control.playback_seconds()).filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
    }

    pub fn music_stage_segment_bpm_display_text(
        &mut self,
        manifest: &crate::app::music_analysis::MusicAnalysisManifest,
        playback_seconds: Option<f64>,
        display_highlight_range: Option<(f64, f64)>,
    ) -> (String, bool) {
        let target_bpm = music_stage_segment_bpm_from_analysis(
            manifest,
            playback_seconds,
            display_highlight_range,
        );
        let Some(target_bpm) = target_bpm else {
            self.music.music_stage_bpm_display = Default::default();
            return ("BPM --".to_owned(), false);
        };

        let now = Instant::now();
        let display = &mut self.music.music_stage_bpm_display;
        if display.stable_bpm.is_none() {
            display.stable_bpm = Some(target_bpm);
            display.display_bpm = Some(target_bpm);
            return (format!("{target_bpm:.0} BPM"), false);
        }

        let active_target = display.animation_to_bpm.or(display.stable_bpm);
        if active_target
            .is_some_and(|active| (active - target_bpm).abs() >= MUSIC_STAGE_BPM_DISPLAY_JITTER_BPM)
        {
            let current_display = display
                .display_bpm
                .or(display.stable_bpm)
                .unwrap_or(target_bpm);
            if (current_display - target_bpm).abs() >= MUSIC_STAGE_BPM_DISPLAY_ANIMATE_BPM {
                display.animation_from_bpm = Some(current_display);
                display.animation_to_bpm = Some(target_bpm);
                display.animation_started_at = Some(now);
            } else {
                display.stable_bpm = Some(target_bpm);
                display.display_bpm = Some(target_bpm);
                display.animation_from_bpm = None;
                display.animation_to_bpm = None;
                display.animation_started_at = None;
            }
        }

        let mut animating = false;
        if let (Some(from), Some(to), Some(started_at)) = (
            display.animation_from_bpm,
            display.animation_to_bpm,
            display.animation_started_at,
        ) {
            let progress =
                (now - started_at).as_secs_f64() / MUSIC_STAGE_BPM_DISPLAY_ANIMATION_SECONDS;
            if progress >= 1.0 {
                display.stable_bpm = Some(to);
                display.display_bpm = Some(to);
                display.animation_from_bpm = None;
                display.animation_to_bpm = None;
                display.animation_started_at = None;
            } else {
                let progress = progress.clamp(0.0, 1.0) as f32;
                let eased = progress * progress * (3.0 - 2.0 * progress);
                display.display_bpm = Some(from + (to - from) * eased);
                animating = true;
            }
        } else if let Some(stable_bpm) = display.stable_bpm {
            display.display_bpm = Some(stable_bpm);
        }

        let bpm = display.display_bpm.unwrap_or(target_bpm);
        (format!("{bpm:.0} BPM"), animating)
    }

    pub fn music_chorus_flow_transition_text(&self) -> Option<String> {
        if let Some(fade) = self.music.music_chorus_fade_out.as_ref() {
            let reason = self
                .music
                .music_chorus_mix_plan
                .as_ref()
                .map(|plan| plan.reason.as_str())
                .unwrap_or("smooth crossfade");
            Some(format!("now {:.1}s · {reason}", fade.duration_seconds))
        } else if let Some(plan) = self.music.music_chorus_mix_plan.as_ref() {
            let ready_next_cue = plan.is_provisional_analysis_pending()
                && self
                    .music
                    .music_player_current_item_id
                    .and_then(|item_id| self.music_analysis_manifest_for_item(item_id))
                    .is_some();
            if ready_next_cue {
                return Some(format!(
                    "next {:.1}s · quick now · full map ready next cue",
                    plan.transition_seconds
                ));
            }
            Some(format!(
                "next {:.1}s · {}",
                plan.transition_seconds, plan.reason
            ))
        } else if self.music.music_chorus_fade_in.is_some() {
            Some("fade in".to_owned())
        } else if let Some(target) = self.music.music_chorus_pending_mix_target.as_ref() {
            let title = self
                .queue_item_by_id(target.target_item_id)
                .map(|item| item.title.as_str())
                .unwrap_or("next track");
            Some(format!("Mix next · {title}"))
        } else if self.music.music_chorus_pending_fade_in.is_some() {
            Some("cue next".to_owned())
        } else {
            None
        }
    }

    pub fn music_chorus_current_display_highlight_range(&self) -> Option<(f64, f64)> {
        let item_id = self.music.music_player_current_item_id?;
        let selected_range = self.music_automix_range_for_item(item_id);
        let active_flow_range = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|segment| segment.item_id == item_id)
            .map(|segment| (segment.start_seconds, segment.end_seconds));
        music_display_playback_range(self.music_mix_mode(), selected_range, active_flow_range)
    }

    pub fn music_current_provisional_highlight_range(&self) -> Option<(f64, f64)> {
        let item_id = self.music.music_player_current_item_id?;
        if self.music_analysis_manifest_for_item(item_id).is_some() {
            return None;
        }
        self.music_provisional_highlight_range_for_item(item_id)
    }

    pub fn music_current_duration_seconds(&self) -> Option<f64> {
        let item_id = self.music.music_player_current_item_id?;
        self.music
            .music_playback
            .as_ref()
            .filter(|control| control.item_id == item_id)
            .and_then(|control| control.duration_seconds())
            .or_else(|| {
                self.queue_item_by_id(item_id)
                    .and_then(|item| item.music_duration_seconds)
            })
            .or_else(|| {
                self.music_analysis_manifest_for_item(item_id)
                    .map(|manifest| manifest.duration_seconds)
            })
            .filter(|duration| duration.is_finite() && *duration > 0.0)
    }

    pub fn music_current_mix_window_seconds(&self) -> Option<(f64, f64)> {
        let control = self.music.music_playback.as_ref()?;
        if self.music.music_player_current_item_id != Some(control.item_id) {
            return None;
        }

        if let Some(fade) =
            self.music.music_chorus_fade_out.as_ref().filter(|fade| {
                fade.item_id == control.item_id && fade.session_id == control.session_id
            })
        {
            // Keep the developer MIX lane visually stable after handoff. The
            // audio path uses frame fields; this UI range is the plan snapshot.
            let start = fade.mix_window_start_seconds.max(0.0);
            let end = fade.mix_window_end_seconds.max(start);
            if end > start {
                return Some((start, end));
            }
        }

        if let Some(segment) = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|segment| {
                segment.item_id == control.item_id && segment.session_id == control.session_id
            })
        {
            return music_segment_display_mix_window(
                self.music_mix_mode(),
                self.music_automix_range_for_item(control.item_id),
                segment.start_seconds.max(0.0),
                segment.end_seconds.max(0.0),
                segment.transition_seconds,
            );
        }

        None
    }

    pub fn music_current_mix_window_kind_key(&self) -> &'static str {
        self.music_current_mix_render_key()
    }

    fn music_current_mix_render_key(&self) -> &'static str {
        let control = self.music.music_playback.as_ref();
        if let (Some(control), Some(fade)) = (control, self.music.music_chorus_fade_out.as_ref()) {
            if fade.item_id == control.item_id && fade.session_id == control.session_id {
                return fade.execution_route.render_key();
            }
        }

        if let (Some(control), Some(segment)) =
            (control, self.music.music_chorus_flow_segment.as_ref())
        {
            if segment.item_id == control.item_id && segment.session_id == control.session_id {
                if segment.fallback_stage.is_plain_crossfade() {
                    return if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY {
                        "guarded"
                    } else {
                        "fallback"
                    };
                }
            }
        }

        if self.music_chorus_plan_is_fallback_mix() {
            "fallback"
        } else if self.music_chorus_plan_is_prepared_mix() {
            "prepared"
        } else if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY {
            "direct"
        } else {
            "realtime"
        }
    }

    fn music_chorus_plan_is_prepared_mix(&self) -> bool {
        self.music
            .music_chorus_mix_plan
            .as_ref()
            .is_some_and(|plan| plan.reason.contains("Prepared Mix"))
    }

    fn music_chorus_plan_is_fallback_mix(&self) -> bool {
        self.music
            .music_chorus_mix_plan
            .as_ref()
            .is_some_and(|plan| {
                let reason = plan.reason.to_ascii_lowercase();
                reason.contains("fallback")
                    || reason.contains("plain mix")
                    || reason.contains("plain live")
                    || reason.contains("last-resort")
                    || reason.contains("preview rejected")
                    || reason.contains("preview stale")
                    || reason.contains("preview off")
            })
    }

    pub(super) fn reanchor_current_music_mode_after_toggle_without_seek(&mut self) -> bool {
        let Some(control) = self.music.music_playback.clone() else {
            return false;
        };

        // A mode toggle changes future selection/transition policy. It must not
        // reuse the initial-play cue path, because that path deliberately seeks
        // to a selected segment start. Keep the current playback cursor and
        // rebuild only the advisory segment that follows it.
        control.clear_crossfade_preview();
        control.set_volume(self.music.music_volume);
        let playback_seconds = control.playback_seconds().max(0.0);
        self.reanchor_music_chorus_flow_after_manual_seek(&control, playback_seconds);
        true
    }

    pub fn music_mix_mode(&self) -> MusicMixMode {
        music_mix_mode_from_flags(
            self.music.music_automix_enabled,
            self.music.music_trim_enabled,
            self.music.music_chorus_flow_enabled,
        )
    }

    pub fn set_music_mix_mode(&mut self, mode: MusicMixMode) {
        if self.music_mix_mode() == mode {
            return;
        }

        let (automix_enabled, trim_enabled, highlight_enabled) = music_mix_flags_for_mode(mode);
        self.music.music_automix_enabled = automix_enabled;
        self.music.music_trim_enabled = trim_enabled;
        self.music.music_chorus_flow_enabled = highlight_enabled;
        self.music.music_chorus_flow_segment = None;
        self.music.music_chorus_mix_plan = None;
        self.music.music_chorus_pending_mix_target = None;
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;
        self.music.music_stage_pick_selected.clear();
        self.clear_music_chorus_transition();

        if mode.enabled() {
            self.ensure_music_stage_analysis_for_active_targets();
        }
        self.reanchor_current_music_mode_after_toggle_without_seek();
        self.last_action = format!("Mix mode: {}.", mode.label());
    }

    pub(super) fn poll_music_chorus_flow(&mut self) {
        self.poll_music_playback_ready_handoff();
        self.poll_music_chorus_handoff_bridge();
        self.poll_music_chorus_fade_in();

        let Some(control) = self.music.music_playback.clone() else {
            return;
        };
        if self.music.music_player_current_item_id != Some(control.item_id) {
            return;
        }

        if !self.music_auto_transition_enabled() {
            self.cancel_music_radio_cue_pending();
            return;
        }

        self.sanitize_music_radio_cue_pending_for_control(&control);

        if control.is_paused() {
            return;
        }

        if let Some(deadline) = self.music.music_manual_seek_grace_until {
            if Instant::now() < deadline {
                return;
            }
            self.music.music_manual_seek_grace_until = None;
        }

        if self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .map_or(true, |segment| {
                segment.item_id != control.item_id || segment.session_id != control.session_id
            })
        {
            self.cue_music_highlight_for_control(control);
            return;
        }

        self.prefetch_music_transition_target_for_control(&control);
        self.poll_music_chorus_preview_job(&control);
        self.arm_music_radio_cue_when_preview_ready(&control);

        if self.poll_music_chorus_fade_out(&control) {
            return;
        }

        let Some(segment) = self.music.music_chorus_flow_segment.clone() else {
            return;
        };
        let playback_seconds = control.playback_seconds();
        let remaining_seconds = segment.end_seconds - playback_seconds;
        let transition_min_seconds = if segment.fallback_stage.is_plain_crossfade() {
            MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS
        } else {
            MUSIC_CHORUS_TRANSITION_MIN_SECONDS
        };
        let transition_seconds = segment
            .transition_seconds
            .max(transition_min_seconds)
            .min(MUSIC_CHORUS_TRANSITION_MAX_SECONDS);
        // Prepared Mix should be a proactive segment asset, not a last-second
        // rescue.  Start it as soon as the current A segment and next B target
        // are known; the helper is idempotent for the same item/session/target.
        if !music_stage_chain_direct_stream_director_enabled()
            && remaining_seconds > MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS
        {
            self.prepare_music_chorus_preview_for_control(&control, &segment);
        }
        if !music_stage_chain_direct_stream_director_enabled()
            && remaining_seconds <= transition_seconds + MUSIC_CHORUS_PREVIEW_PREPARE_LEAD_SECONDS
        {
            self.prepare_music_chorus_preview_for_control(&control, &segment);
        }

        if remaining_seconds <= transition_seconds.max(MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS) {
            if self.hold_music_stage_mix_until_preview_ready(&control, &segment, transition_seconds)
            {
                return;
            }
            self.begin_music_chorus_fade_out(&control);
        }
    }

    fn ensure_music_stage_analysis_for_active_targets(&mut self) {
        let Some(control) = self.music.music_playback.clone() else {
            return;
        };
        // Quick/provisional highlights are only a first-run bridge.  Keep the
        // real analyzer warm for the active track, the armed mix target, and
        // the next likely track, but do not force-replan an already playing
        // segment when the manifest finishes.  The next cue naturally adopts
        // the finished Music Map and avoids visible/aural range jumps.
        self.ensure_music_analysis_for_item_if_cached(control.item_id);

        if let Some(target_item_id) = self
            .music
            .music_chorus_pending_mix_target
            .as_ref()
            .filter(|target| {
                target.current_item_id == control.item_id && target.session_id == control.session_id
            })
            .map(|target| target.target_item_id)
        {
            self.ensure_music_analysis_for_item_if_cached(target_item_id);
        }

        if let Some(next_item_id) = self.peek_next_music_chorus_flow_item_id(control.item_id) {
            if next_item_id != control.item_id {
                self.ensure_music_analysis_for_item_if_cached(next_item_id);
            }
        }
    }

    fn ensure_music_analysis_for_item_if_cached(&mut self, item_id: QueueItemId) -> bool {
        let Some(item) = self.queue_item_by_id(item_id).cloned() else {
            return false;
        };
        let Some(media_path) = self.complete_music_cache_media_path(&item) else {
            return false;
        };
        let cache_key = item.music_cache_key.trim();
        if cache_key.is_empty() {
            return false;
        }
        let analysis_path = self
            .music_stream_cache_root()
            .join(sanitize_music_cache_key(cache_key))
            .join("analysis.yaml");
        crate::app::music_analysis::spawn_music_analysis_if_needed(
            media_path,
            item.music_stream_ext,
            analysis_path,
            item.music_duration_seconds,
        );
        true
    }

    fn cue_music_highlight_for_control(&mut self, control: MusicPlaybackControl) -> bool {
        self.ensure_music_stage_cue_memory_loaded();
        self.ensure_music_stage_analysis_for_active_targets();
        self.ensure_music_stage_pick_for_item(control.item_id, false);
        let Some(automix_segment) = self.music_automix_segment_for_item(control.item_id) else {
            return false;
        };
        let using_provisional_highlight = matches!(
            automix_segment.source,
            music_segment_selector::MusicPlayableSegmentSource::HighlightQuickEstimate
        );
        let (start_seconds, end_seconds) = automix_segment.as_range();
        let (start_seconds, mut end_seconds) =
            self.music_lyrics_safe_range_for_item(control.item_id, start_seconds, end_seconds);
        if end_seconds
            <= start_seconds
                + MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS
                + MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS
        {
            return false;
        }

        let pending_fade_in = self
            .music
            .music_chorus_pending_fade_in
            .as_ref()
            .is_some_and(|pending| pending.item_id == control.item_id);
        let pending_start_seconds = if self
            .music
            .music_chorus_pending_start
            .as_ref()
            .is_some_and(|pending| pending.item_id == control.item_id)
        {
            self.music.music_chorus_pending_start.take().map(|pending| {
                pending.start_seconds.clamp(
                    start_seconds,
                    (end_seconds - MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS).max(start_seconds),
                )
            })
        } else {
            None
        };
        let (playback_start_seconds, should_seek_to_cue_start) = music_initial_cue_start_policy(
            self.music_mix_mode(),
            start_seconds,
            control.playback_seconds(),
            pending_start_seconds,
        );

        let duration = control
            .duration_seconds()
            .or_else(|| {
                self.music_analysis_manifest_for_item(control.item_id)
                    .map(|manifest| manifest.duration_seconds)
            })
            .filter(|duration| duration.is_finite() && *duration > 0.0)
            .unwrap_or(end_seconds.max(1.0));
        let quick_focus_plan = if using_provisional_highlight {
            music_segment_selector::quick_focus_segment(duration)
        } else {
            None
        };
        if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY && pending_fade_in {
            let guarded_end = music_stage_chain_post_handoff_guarded_end_seconds(
                playback_start_seconds,
                end_seconds,
                duration,
            );
            if guarded_end > end_seconds + 0.050 {
                eprintln!(
                    "[music-stage-chain] post-handoff breathe item={} start={:.3}s old_end={:.3}s new_end={:.3}s breathe={:.3}s",
                    control.item_id,
                    playback_start_seconds,
                    end_seconds,
                    guarded_end,
                    (guarded_end
                        - playback_start_seconds
                        - MUSIC_STAGE_LITE_TRANSITION_MAX_SECONDS)
                        .max(0.0),
                );
                end_seconds = guarded_end;
            }
        }
        let seek_ratio = (playback_start_seconds / duration).clamp(0.0, 1.0) as f32;
        if should_seek_to_cue_start {
            control.seek_to_ratio(seek_ratio);
            self.music.music_seek_snap_ratio = Some(seek_ratio);
            self.music.music_seek_snap_deadline = Some(Instant::now() + Duration::from_millis(700));
        }
        let target_volume = self.music.music_volume;
        let active_fade_in = self
            .music
            .music_chorus_fade_in
            .as_ref()
            .is_some_and(|fade| {
                fade.item_id == control.item_id && fade.session_id == control.session_id
            });
        if pending_fade_in {
            control.set_volume(0.0);
        } else if !active_fade_in {
            control.set_volume(target_volume);
        }
        let segment_seconds = (end_seconds - playback_start_seconds).max(0.0);
        let near_tail_segment = duration - end_seconds <= MUSIC_CHORUS_TAIL_DIRECT_HANDOFF_SECONDS;
        let fallback_stage = if segment_seconds <= MUSIC_CHORUS_SHORT_TAIL_SEGMENT_SECONDS {
            MusicChorusFallbackStage::PlainCrossfade
        } else {
            MusicChorusFallbackStage::Normal
        };
        let transition_min_seconds = if fallback_stage.is_plain_crossfade() {
            MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS
        } else if segment_seconds <= MUSIC_CHORUS_SHORT_TAIL_SEGMENT_SECONDS * 2.5
            || near_tail_segment
        {
            MUSIC_CHORUS_STREAM_MIX_MIN_SECONDS
        } else {
            MUSIC_CHORUS_TRANSITION_MIN_SECONDS
        };
        let mut transition_seconds = self
            .music_chorus_transition_seconds_for_item(control.item_id)
            .min((segment_seconds * 0.45).max(transition_min_seconds))
            .clamp(transition_min_seconds, MUSIC_CHORUS_TRANSITION_MAX_SECONDS);
        if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY
            && !pending_fade_in
            && self.music_mix_mode() != MusicMixMode::FullSong
        {
            // Full Song owns the physical EOF. Tail/body fences are valid for
            // local-range modes, but shortening Full Song here both draws the
            // Mix lane in the middle and lets the runtime transition early.
            let guarded_end = self
                .music_stage_chain_direct_body_fence_safe_exit_end_for_item(
                    control.item_id,
                    playback_start_seconds,
                    end_seconds,
                    transition_seconds,
                    Some(duration),
                )
                .or_else(|| {
                    self.music_stage_chain_direct_energy_tail_safe_exit_end_for_item(
                        control.item_id,
                        playback_start_seconds,
                        end_seconds,
                        transition_seconds,
                        Some(duration),
                    )
                })
                .or_else(|| {
                    music_stage_chain_tail_guarded_exit_end_seconds(
                        playback_start_seconds,
                        end_seconds,
                        transition_seconds,
                        Some(duration),
                    )
                });
            if let Some(guarded_end) = guarded_end {
                if guarded_end + 0.050 < end_seconds {
                    eprintln!(
                        "[music-stage-chain] A exit tail guard item={} old_end={:.3}s new_end={:.3}s runway={:.3}s",
                        control.item_id,
                        end_seconds,
                        guarded_end,
                        (duration - guarded_end).max(0.0),
                    );
                    end_seconds = guarded_end;
                    let guarded_segment_seconds = (end_seconds - playback_start_seconds).max(0.0);
                    transition_seconds = transition_seconds
                        .min((guarded_segment_seconds * 0.45).max(transition_min_seconds))
                        .clamp(transition_min_seconds, MUSIC_CHORUS_TRANSITION_MAX_SECONDS);
                }
            }
        }
        self.music.music_chorus_flow_segment = Some(MusicChorusFlowSegment {
            item_id: control.item_id,
            session_id: control.session_id,
            start_seconds: playback_start_seconds,
            end_seconds,
            transition_seconds,
            hold_end_seconds: None,
            fallback_stage,
        });
        let mix_plan_reason = if fallback_stage.is_plain_crossfade() {
            "plain mix · short/near-tail segment".to_owned()
        } else if using_provisional_highlight {
            quick_focus_plan
                .map(|plan| {
                    format!(
                        "quick focus {} {:.2} · provisional analysis pending",
                        plan.kind.log_key(),
                        plan.confidence
                    )
                })
                .unwrap_or_else(|| "provisional highlight · analysis pending".to_owned())
        } else {
            "current track beat window".to_owned()
        };
        self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
            transition_seconds,
            confidence: self.music_chorus_tempo_confidence_for_item(control.item_id),
            reason: mix_plan_reason.clone(),
        });
        self.music.music_chorus_fade_out = None;
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;

        if using_provisional_highlight {
            let quick_kind = quick_focus_plan
                .map(|plan| plan.kind.log_key())
                .unwrap_or("unknown");
            let quick_confidence = quick_focus_plan
                .map(|plan| plan.confidence)
                .unwrap_or_default();
            eprintln!(
                "[music-stage-quick] provisional highlight item={} kind={} conf={:.2} range={:.3}-{:.3}s segment={:.3}s duration={:.3}s",
                control.item_id,
                quick_kind,
                quick_confidence,
                playback_start_seconds,
                end_seconds,
                (end_seconds - playback_start_seconds).max(0.0),
                duration,
            );
        }
        eprintln!(
            "[music-stage-decision] cue item={} session={} source={} range={:.3}-{:.3}s transition={:.3}s fallback={:?} reason={}",
            control.item_id,
            control.session_id,
            automix_segment.source.log_key(),
            playback_start_seconds,
            end_seconds,
            transition_seconds,
            fallback_stage,
            mix_plan_reason,
        );

        // v10.12.20: pre-arm the B transition as soon as A's Stage Mix
        // segment starts.  If there is no manual Radio Cue, the next track is
        // already known, so HQ Mix should render in the background during A
        // instead of waiting until the playhead is close to the orange mix
        // window.  This keeps the marker from retreating when the playhead
        // reaches it, and makes streaming/HQ behave like the same promised cue.
        if music_stage_chain_direct_stream_director_enabled() {
            self.music.music_chorus_preview_job = None;
            self.music.music_chorus_ready_preview = None;
        } else if let Some(segment) = self.music.music_chorus_flow_segment.clone() {
            self.prepare_music_chorus_preview_for_control(&control, &segment);
        }

        self.last_action = format!(
            "Stage Mix segment: {}–{} · {}{}",
            format_duration_seconds(start_seconds),
            format_duration_seconds(end_seconds),
            if music_stage_chain_direct_stream_director_enabled() {
                "direct stream ready"
            } else {
                "prearming B"
            },
            if using_provisional_highlight {
                " · provisional"
            } else {
                ""
            },
        );
        true
    }

    pub(super) fn reanchor_music_chorus_flow_after_manual_seek(
        &mut self,
        control: &MusicPlaybackControl,
        playback_seconds: f64,
    ) {
        self.ensure_music_stage_cue_memory_loaded();
        if !self.music_auto_transition_enabled() {
            self.music.music_chorus_flow_segment = None;
            self.music.music_chorus_mix_plan = None;
            return;
        }
        if self.music.music_player_current_item_id != Some(control.item_id) {
            return;
        }

        let playback_seconds = playback_seconds.max(0.0);
        let duration = control
            .duration_seconds()
            .or_else(|| {
                self.music_analysis_manifest_for_item(control.item_id)
                    .map(|manifest| manifest.duration_seconds)
            })
            .filter(|duration| duration.is_finite() && *duration > 0.0);
        let range_end = self
            .music_automix_range_for_item(control.item_id)
            .map(|(range_start, range_end)| {
                let (_range_start, range_end) =
                    self.music_lyrics_safe_range_for_item(control.item_id, range_start, range_end);
                range_end
            })
            .or(duration)
            .unwrap_or(playback_seconds + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS * 2.0);
        let fallback_end = duration.unwrap_or(range_end).max(range_end);
        let end_seconds = if range_end > playback_seconds + MUSIC_CHORUS_TRANSITION_MIN_SECONDS {
            range_end
        } else {
            fallback_end
        }
        .max(playback_seconds);
        let remaining_seconds = end_seconds - playback_seconds;
        // Keep an explicit short-tail segment instead of returning None. None
        // means "initial cue not planned" to the poller and would make it seek
        // backward to the selected range after the manual-seek grace expires.
        let fallback_stage = if remaining_seconds <= MUSIC_CHORUS_SHORT_TAIL_SEGMENT_SECONDS {
            MusicChorusFallbackStage::PlainCrossfade
        } else {
            MusicChorusFallbackStage::Normal
        };
        let transition_min_seconds = if fallback_stage.is_plain_crossfade() {
            MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS
        } else if remaining_seconds <= MUSIC_CHORUS_SHORT_TAIL_SEGMENT_SECONDS * 2.5 {
            MUSIC_CHORUS_STREAM_MIX_MIN_SECONDS
        } else {
            MUSIC_CHORUS_TRANSITION_MIN_SECONDS
        };
        let transition_seconds = self
            .music_chorus_transition_seconds_for_item(control.item_id)
            .min((remaining_seconds * 0.45).max(transition_min_seconds))
            .clamp(transition_min_seconds, MUSIC_CHORUS_TRANSITION_MAX_SECONDS);
        self.music.music_chorus_flow_segment = Some(MusicChorusFlowSegment {
            item_id: control.item_id,
            session_id: control.session_id,
            start_seconds: playback_seconds,
            end_seconds,
            transition_seconds,
            hold_end_seconds: None,
            fallback_stage,
        });
        self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
            transition_seconds,
            confidence: self.music_chorus_tempo_confidence_for_item(control.item_id),
            reason: if fallback_stage.is_plain_crossfade() {
                "plain mix · short manual window".to_owned()
            } else {
                "manual seek window".to_owned()
            },
        });
        self.music.music_chorus_fade_out = None;
        self.music.music_chorus_fade_in = None;
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;
        if music_stage_chain_direct_stream_director_enabled() {
            self.music.music_chorus_preview_job = None;
            self.music.music_chorus_ready_preview = None;
        } else if let Some(segment) = self.music.music_chorus_flow_segment.clone() {
            self.prepare_music_chorus_preview_for_control(control, &segment);
        }
    }

    fn hold_music_stage_mix_until_preview_ready(
        &mut self,
        control: &MusicPlaybackControl,
        segment: &MusicChorusFlowSegment,
        transition_seconds: f64,
    ) -> bool {
        // Radio Cue has its own readiness-based hold logic. This helper is for
        // automatic Stage Mix / Chorus Flow.  v10.12.21 adds a bounded fallback
        // ladder: HQ may retreat once into Stream; Stream may wait only inside
        // that promised window; if it is still not ready, use a plain crossfade
        // at the visible marker instead of moving the marker again.
        if let Some(target) = self.music_radio_cue_pending_for_control(control) {
            return !target.cue_armed;
        }

        let Some(next_item_id) = self.peek_next_music_chorus_flow_item_id(control.item_id) else {
            return false;
        };

        if music_stage_chain_direct_stream_director_enabled() {
            self.music.music_chorus_preview_job = None;
            self.music.music_chorus_ready_preview = None;
            if self.hold_music_transition_while_target_cache_prepares(
                control,
                next_item_id,
                transition_seconds,
                "Stage Chain · B cache preparing · A continues",
            ) {
                return true;
            }
            self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                transition_seconds,
                confidence: self.music_chorus_pair_confidence(control.item_id, next_item_id),
                reason: "Stage Chain · Direct Stream armed".to_owned(),
            });
            return false;
        }

        let preview_ready = self
            .music
            .music_chorus_ready_preview
            .as_ref()
            .is_some_and(|preview| {
                preview.current_item_id == control.item_id
                    && preview.session_id == control.session_id
                    && preview.next_item_id == next_item_id
            });
        if preview_ready {
            return false;
        }

        let playback_seconds = control.playback_seconds().max(0.0);
        let duration_end = control
            .duration_seconds()
            .or_else(|| {
                self.music_analysis_manifest_for_item(control.item_id)
                    .map(|manifest| manifest.duration_seconds)
            })
            .filter(|duration| duration.is_finite() && *duration > playback_seconds + 0.5);
        let segment_len = (segment.end_seconds - segment.start_seconds).max(0.0);
        let near_tail = duration_end.is_some_and(|duration| {
            duration - segment.end_seconds <= MUSIC_CHORUS_TAIL_DIRECT_HANDOFF_SECONDS
        });
        let short_segment = segment_len <= MUSIC_CHORUS_SHORT_TAIL_SEGMENT_SECONDS;
        let prepared_job_pending =
            self.music
                .music_chorus_preview_job
                .as_ref()
                .is_some_and(|job| {
                    job.current_item_id == control.item_id
                        && job.session_id == control.session_id
                        && job.next_item_id == next_item_id
                });
        let hard_end_allows_prepared_hold = duration_end.map_or(true, |duration| {
            (duration - MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS).max(playback_seconds)
                > playback_seconds + transition_seconds + 0.35
        });
        let prepared_hold_available = prepared_job_pending && hard_end_allows_prepared_hold;
        if active_state_is_normal_for_segment(
            &self.music.music_chorus_flow_segment,
            control,
            segment,
        ) && (short_segment || near_tail)
            && !prepared_hold_available
        {
            if let Some(active_segment) =
                self.music
                    .music_chorus_flow_segment
                    .as_mut()
                    .filter(|active| {
                        active.item_id == control.item_id && active.session_id == control.session_id
                    })
            {
                active_segment.transition_seconds =
                    transition_seconds.max(MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS);
                active_segment.hold_end_seconds = Some(segment.end_seconds);
                active_segment.fallback_stage = MusicChorusFallbackStage::PlainCrossfade;
            }
            let updated_segment = self.music.music_chorus_flow_segment.clone();
            if let Some(updated_segment) = updated_segment.as_ref() {
                if self
                    .music
                    .music_chorus_ready_preview
                    .as_ref()
                    .map_or(true, |preview| {
                        preview.current_item_id != control.item_id
                            || preview.session_id != control.session_id
                            || preview.next_item_id != next_item_id
                    })
                {
                    self.music.music_chorus_preview_job = None;
                    self.prepare_music_chorus_preview_for_target(
                        control,
                        updated_segment,
                        next_item_id,
                    );
                }
            }
            self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                transition_seconds,
                confidence: self.music_chorus_pair_confidence(control.item_id, next_item_id),
                reason: if near_tail {
                    "plain mix · short tail deadline".to_owned()
                } else {
                    "plain mix · short highlight deadline".to_owned()
                },
            });
            return false;
        }
        let active_state = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|active| {
                active.item_id == control.item_id && active.session_id == control.session_id
            })
            .map(|active| (active.hold_end_seconds, active.fallback_stage))
            .unwrap_or((segment.hold_end_seconds, segment.fallback_stage));

        if let Some(locked_end) = active_state.0 {
            let locked_mix_start =
                locked_end - transition_seconds.max(MUSIC_CHORUS_REALTIME_FALLBACK_FADE_SECONDS);
            if playback_seconds + MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS < locked_mix_start {
                return true;
            }

            // If the prepared worker is still running for the locked window, do
            // not immediately demote to plain-inline at the mix start. The
            // prepared preview may have been rendered for the correct guarded
            // end but delivered between UI ticks. Switching to plain here is
            // what produced short 2s capsules and apparent end-of-track stops.
            if prepared_job_pending && hard_end_allows_prepared_hold {
                self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                    transition_seconds,
                    confidence: self.music_chorus_pair_confidence(control.item_id, next_item_id),
                    reason: "Prepared Mix pending · holding locked window".to_owned(),
                });
                return true;
            }

            if let Some(active_segment) =
                self.music
                    .music_chorus_flow_segment
                    .as_mut()
                    .filter(|active| {
                        active.item_id == control.item_id && active.session_id == control.session_id
                    })
            {
                active_segment.fallback_stage = MusicChorusFallbackStage::PlainCrossfade;
                active_segment.transition_seconds = transition_seconds;
            }
            self.music.music_chorus_preview_job = None;
            self.music.music_chorus_ready_preview = None;
            self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                transition_seconds,
                confidence: self.music_chorus_pair_confidence(control.item_id, next_item_id),
                reason: "plain mix · stream window locked".to_owned(),
            });
            return false;
        }

        let requested_end = playback_seconds
            + transition_seconds.max(MUSIC_CHORUS_REALTIME_FALLBACK_FADE_SECONDS)
            + MUSIC_STAGE_SHORT_HIGHLIGHT_READY_HOLD_SECONDS;
        let original_highlight_end = self
            .music_automix_range_for_item(control.item_id)
            .map(|(range_start, range_end)| {
                let (_range_start, range_end) =
                    self.music_lyrics_safe_range_for_item(control.item_id, range_start, range_end);
                range_end
            })
            .unwrap_or(segment.end_seconds)
            .max(segment.start_seconds);
        let max_extension_end =
            original_highlight_end + MUSIC_STAGE_SHORT_HIGHLIGHT_MAX_EXTENSION_SECONDS;
        let mut hold_end = requested_end
            .min(max_extension_end)
            .max(segment.end_seconds + MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS)
            .min(max_extension_end);

        if let Some(duration) = duration_end {
            let hard_end =
                (duration - MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS).max(playback_seconds);
            if hold_end > hard_end {
                if hard_end <= playback_seconds + transition_seconds + 0.35 {
                    if let Some(active_segment) = self
                        .music
                        .music_chorus_flow_segment
                        .as_mut()
                        .filter(|active| {
                            active.item_id == control.item_id
                                && active.session_id == control.session_id
                        })
                    {
                        active_segment.end_seconds = hard_end.max(playback_seconds);
                        active_segment.transition_seconds = transition_seconds;
                        active_segment.hold_end_seconds = Some(hard_end.max(playback_seconds));
                        active_segment.fallback_stage = MusicChorusFallbackStage::PlainCrossfade;
                    }
                    self.music.music_chorus_preview_job = None;
                    self.music.music_chorus_ready_preview = None;
                    self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                        transition_seconds,
                        confidence: self
                            .music_chorus_pair_confidence(control.item_id, next_item_id),
                        reason: "plain mix · no room to retreat".to_owned(),
                    });
                    return false;
                }
                hold_end = hard_end;
            }
        }

        let use_stream_fallback = self.music_stage_mix_render_mode()
            == MusicMixRenderMode::HighQualityOffline
            && active_state.1 == MusicChorusFallbackStage::Normal;
        let next_fallback_stage = if active_state.1.is_plain_crossfade() {
            MusicChorusFallbackStage::PlainCrossfade
        } else {
            MusicChorusFallbackStage::StreamFallback
        };

        if let Some(active_segment) =
            self.music
                .music_chorus_flow_segment
                .as_mut()
                .filter(|active| {
                    active.item_id == control.item_id && active.session_id == control.session_id
                })
        {
            if active_segment.end_seconds < hold_end {
                active_segment.end_seconds = hold_end;
            }
            active_segment.transition_seconds = transition_seconds;
            active_segment.hold_end_seconds = Some(hold_end);
            active_segment.fallback_stage = next_fallback_stage;
        }

        // If HQ was late, drop its worker and immediately try the lighter
        // streaming renderer in the same, now locked, window.
        if use_stream_fallback {
            self.music.music_chorus_preview_job = None;
            self.music.music_chorus_ready_preview = None;
        }

        let updated_segment = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|active| {
                active.item_id == control.item_id && active.session_id == control.session_id
            })
            .cloned();
        if let Some(updated_segment) = updated_segment.as_ref() {
            if self.music.music_chorus_preview_job.is_none()
                && self.music.music_chorus_ready_preview.is_none()
            {
                self.prepare_music_chorus_preview_for_target(
                    control,
                    updated_segment,
                    next_item_id,
                );
            }
        }

        self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
            transition_seconds,
            confidence: self.music_chorus_pair_confidence(control.item_id, next_item_id),
            reason: if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY {
                "Stage Chain · Stream Handoff armed".to_owned()
            } else if next_fallback_stage.is_plain_crossfade() {
                "plain mix · waiting for B capsule".to_owned()
            } else if use_stream_fallback {
                "HQ late · stream fallback armed".to_owned()
            } else {
                "stage cue waiting for stream preview".to_owned()
            },
        });
        true
    }

    fn poll_music_chorus_preview_job(&mut self, control: &MusicPlaybackControl) {
        if music_stage_chain_direct_stream_director_enabled() {
            if self.music.music_chorus_preview_job.is_some()
                || self.music.music_chorus_ready_preview.is_some()
            {
                self.music.music_chorus_preview_job = None;
                self.music.music_chorus_ready_preview = None;
            }
            return;
        }

        let stale_ready = self
            .music
            .music_chorus_ready_preview
            .as_ref()
            .is_some_and(|preview| {
                preview.current_item_id != control.item_id
                    || preview.session_id != control.session_id
            });
        if stale_ready {
            self.music.music_chorus_ready_preview = None;
        }

        let Some(job) = self.music.music_chorus_preview_job.as_ref() else {
            return;
        };
        if job.current_item_id != control.item_id || job.session_id != control.session_id {
            self.music.music_chorus_preview_job = None;
            return;
        }

        let transition_seconds = job.transition_seconds;
        let next_item_id = job.next_item_id;
        let job_elapsed_seconds = job.started_at.elapsed().as_secs_f64();
        let message = job.receiver.try_recv();

        match message {
            Ok(Ok(preview)) => {
                if preview.current_item_id != control.item_id
                    || preview.session_id != control.session_id
                    || preview.next_item_id != next_item_id
                {
                    eprintln!(
                        "[music-stage] ignored stale preview result current={}:{} next={} active={}:{} next={}",
                        preview.current_item_id,
                        preview.session_id,
                        preview.next_item_id,
                        control.item_id,
                        control.session_id,
                        next_item_id
                    );
                    self.music.music_chorus_preview_job = None;
                    return;
                }
                let reason = self.music_chorus_transition_reason_with_rate(
                    preview.current_item_id,
                    preview.next_item_id,
                    preview.preview_rate,
                    preview.outgoing_rate,
                    preview.preserve_pitch,
                    preview.stretch_detail.as_deref(),
                );
                self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                    transition_seconds: preview.transition_seconds,
                    confidence: preview.plan_confidence,
                    reason,
                });
                self.music.music_chorus_ready_preview = Some(preview);
                self.music.music_chorus_preview_job = None;
            }
            Ok(Err(error)) => {
                self.music.music_chorus_preview_job = None;
                let should_try_stream = self.music_stage_mix_render_mode()
                    == MusicMixRenderMode::HighQualityOffline
                    && self
                        .music
                        .music_chorus_flow_segment
                        .as_ref()
                        .filter(|segment| {
                            segment.item_id == control.item_id
                                && segment.session_id == control.session_id
                        })
                        .is_some_and(|segment| {
                            segment.fallback_stage == MusicChorusFallbackStage::Normal
                        });
                if should_try_stream {
                    if let Some(segment) =
                        self.music
                            .music_chorus_flow_segment
                            .as_mut()
                            .filter(|segment| {
                                segment.item_id == control.item_id
                                    && segment.session_id == control.session_id
                            })
                    {
                        segment.fallback_stage = MusicChorusFallbackStage::StreamFallback;
                    }
                    let retry_segment = self.music.music_chorus_flow_segment.clone();
                    if let Some(segment) = retry_segment.as_ref() {
                        let _ = self.prepare_music_chorus_preview_for_control(control, segment);
                    }
                    self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                        transition_seconds,
                        confidence: 0.0,
                        reason: format!("HQ enhancer off · Stream Mix ({error})"),
                    });
                } else {
                    self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                        transition_seconds,
                        confidence: 0.0,
                        reason: format!("preview off · {error}"),
                    });
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                if job_elapsed_seconds > MUSIC_CHORUS_PREVIEW_PREPARE_LEAD_SECONDS * 1.75 {
                    self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                        transition_seconds,
                        confidence: 0.0,
                        reason: "preview preparing · Stream Mix worker".to_owned(),
                    });
                }
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.music.music_chorus_preview_job = None;
            }
        }
    }

    fn prepare_music_chorus_preview_for_control(
        &mut self,
        control: &MusicPlaybackControl,
        current_segment: &MusicChorusFlowSegment,
    ) -> bool {
        let Some(next_item_id) = self.pending_or_peek_next_music_item_id(control) else {
            return false;
        };
        self.prepare_music_chorus_preview_for_target(control, current_segment, next_item_id)
    }

    fn prefetch_music_transition_target_for_control(&mut self, control: &MusicPlaybackControl) {
        if self.music.music_prefetch_for_current_item_id == Some(control.item_id) {
            return;
        }
        let Some(next_item_id) = self.pending_or_peek_next_music_item_id(control) else {
            return;
        };
        if next_item_id == control.item_id || self.music_item_has_complete_cache(next_item_id) {
            return;
        }

        self.start_music_prefetch_for_item(next_item_id);
        self.music.music_prefetch_for_current_item_id = Some(control.item_id);
    }

    fn hold_music_transition_while_target_cache_prepares(
        &mut self,
        control: &MusicPlaybackControl,
        target_item_id: QueueItemId,
        transition_seconds: f64,
        reason: &str,
    ) -> bool {
        if self.music_item_has_complete_cache(target_item_id)
            || !self.music_prefetch_is_preparing_item(target_item_id)
        {
            return false;
        }

        let playback_seconds = control.playback_seconds().max(0.0);
        let duration_seconds = control.duration_seconds().or_else(|| {
            self.music_analysis_manifest_for_item(control.item_id)
                .map(|manifest| manifest.duration_seconds)
        });
        let Some(current_end_seconds) = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|segment| {
                segment.item_id == control.item_id && segment.session_id == control.session_id
            })
            .map(|segment| segment.end_seconds)
        else {
            return false;
        };
        let Some(hold_end_seconds) = music_transition_cache_hold_end_seconds(
            playback_seconds,
            current_end_seconds,
            transition_seconds,
            duration_seconds,
        ) else {
            return false;
        };

        if let Some(segment) = self
            .music
            .music_chorus_flow_segment
            .as_mut()
            .filter(|segment| {
                segment.item_id == control.item_id && segment.session_id == control.session_id
            })
        {
            segment.end_seconds = segment.end_seconds.max(hold_end_seconds);
            segment.hold_end_seconds = Some(
                segment
                    .hold_end_seconds
                    .unwrap_or(segment.end_seconds)
                    .max(hold_end_seconds),
            );
        }
        if let Some(target) = self
            .music
            .music_chorus_pending_mix_target
            .as_mut()
            .filter(|target| {
                target.current_item_id == control.item_id
                    && target.session_id == control.session_id
                    && target.target_item_id == target_item_id
            })
        {
            target.hold_end_seconds = Some(
                target
                    .hold_end_seconds
                    .unwrap_or(hold_end_seconds)
                    .max(hold_end_seconds),
            );
        }
        self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
            transition_seconds,
            confidence: self.music_chorus_pair_confidence(control.item_id, target_item_id),
            reason: reason.to_owned(),
        });
        true
    }

    fn prepare_music_chorus_preview_for_target(
        &mut self,
        control: &MusicPlaybackControl,
        current_segment: &MusicChorusFlowSegment,
        next_item_id: QueueItemId,
    ) -> bool {
        if self.music.music_chorus_fade_out.is_some() {
            return false;
        }
        if self
            .music
            .music_chorus_ready_preview
            .as_ref()
            .is_some_and(|preview| {
                preview.current_item_id == control.item_id
                    && preview.session_id == control.session_id
                    && preview.next_item_id == next_item_id
            })
        {
            return true;
        }
        if self
            .music
            .music_chorus_preview_job
            .as_ref()
            .is_some_and(|job| {
                job.current_item_id == control.item_id
                    && job.session_id == control.session_id
                    && job.next_item_id == next_item_id
            })
        {
            return true;
        }
        if self
            .music
            .music_chorus_preview_job
            .as_ref()
            .is_some_and(|job| {
                job.current_item_id == control.item_id && job.session_id == control.session_id
            })
        {
            // A different target was queued after the old worker started.  Drop the
            // stale receiver and let the old worker finish in the background; the
            // result will be ignored because nobody is listening anymore.
            self.music.music_chorus_preview_job = None;
        }
        if self
            .music
            .music_chorus_ready_preview
            .as_ref()
            .is_some_and(|preview| {
                preview.current_item_id == control.item_id
                    && preview.session_id == control.session_id
                    && preview.next_item_id != next_item_id
            })
        {
            self.music.music_chorus_ready_preview = None;
        }

        let replace_stage_pick = self
            .music
            .music_chorus_pending_mix_target
            .as_ref()
            .is_some_and(|target| target.target_item_id != next_item_id);
        self.ensure_music_stage_pick_for_item(next_item_id, replace_stage_pick);
        let Some((highlight_start, highlight_end)) = self
            .music_automix_range_for_item(next_item_id)
            .or_else(|| self.music_full_range_for_item(next_item_id))
        else {
            return false;
        };
        let Some(item) = self.queue_item_by_id(next_item_id).cloned() else {
            return false;
        };
        let Some(media_path) = self.complete_music_cache_media_path(&item) else {
            self.start_music_prefetch_for_item(next_item_id);
            self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                transition_seconds: current_segment.transition_seconds,
                confidence: self.music_chorus_pair_confidence(control.item_id, next_item_id),
                reason: "B cache preparing · preview pending".to_owned(),
            });
            return false;
        };
        let current_item = self.queue_item_by_id(control.item_id).cloned();
        let current_media_path = current_item
            .as_ref()
            .and_then(|item| self.complete_music_cache_media_path(item));
        let current_music_stream_ext = current_item
            .as_ref()
            .map(|item| item.music_stream_ext.clone())
            .unwrap_or_default();

        let current_end_seconds = self
            .music_chorus_stage_range_end_seconds_for_item(control.item_id)
            .map(|stage_end| stage_end.max(current_segment.end_seconds))
            .unwrap_or(current_segment.end_seconds);
        let current_len = current_end_seconds - current_segment.start_seconds;
        let next_len = highlight_end - highlight_start;
        let plain_mix_capsule = MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX
            || current_segment.fallback_stage.is_plain_crossfade();
        let (base_transition, mix_kind) = if plain_mix_capsule {
            (
                self.music_chorus_transition_seconds_between(control.item_id, next_item_id),
                MusicMixWindowKind::Plain,
            )
        } else {
            self.music_chorus_stream_transition_seconds_and_kind_between(
                control.item_id,
                next_item_id,
            )
        };
        let reward_tail_room = if !plain_mix_capsule && mix_kind == MusicMixWindowKind::RewardLong {
            self.music_chorus_reward_tail_extension_seconds_between(
                control.item_id,
                next_item_id,
                current_end_seconds,
                base_transition,
            )
        } else {
            0.0
        };
        let reward_transition_seed =
            if !plain_mix_capsule && mix_kind == MusicMixWindowKind::RewardLong {
                self.music_chorus_reward_transition_seed_seconds_between(
                    control.item_id,
                    next_item_id,
                    base_transition,
                    reward_tail_room,
                )
            } else {
                base_transition
            };
        let transition_current_len = current_len + reward_tail_room;
        let mut transition_seconds = if plain_mix_capsule {
            clamp_music_chorus_mix_capsule_transition_seconds(
                base_transition,
                current_len,
                next_len,
            )
        } else {
            clamp_music_chorus_transition_seconds(
                reward_transition_seed,
                transition_current_len,
                next_len,
            )
        };
        if MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX {
            transition_seconds = clamp_music_stage_lite_transition_seconds(
                transition_seconds,
                current_len,
                next_len,
            );
        }
        let mut planned_segment_end =
            if !plain_mix_capsule && mix_kind == MusicMixWindowKind::RewardLong {
                self.music_chorus_reward_extended_end_seconds(
                    control.item_id,
                    next_item_id,
                    current_end_seconds,
                    transition_seconds,
                )
            } else {
                current_end_seconds
            };
        // Codex/ひより guardrail: the Prepared Mix render start must use the
        // same segment end that the chorus flow will later use for handoff.
        // Reward-tail heuristics may return an earlier musical end than a
        // dwell/hold-extended segment. Rendering from that earlier end makes
        // the preview look ready but already late at handoff time, causing
        // late reject -> plain fallback -> apparent jump/stop.
        let locked_segment_end = current_segment
            .hold_end_seconds
            .unwrap_or(current_segment.end_seconds)
            .max(current_segment.end_seconds);
        if planned_segment_end + 0.001 < locked_segment_end {
            eprintln!(
                "[music-stage-prepared] align render window item={}->{} planned_end={:.3}s locked_end={:.3}s",
                control.item_id, next_item_id, planned_segment_end, locked_segment_end
            );
            planned_segment_end = locked_segment_end;
        }
        let mut entry_start = self.music_lyrics_safe_entry_start_for_item(
            next_item_id,
            self.music_automix_entry_start_for_item(
                next_item_id,
                highlight_start,
                highlight_end,
                transition_seconds,
            ),
            highlight_start,
            highlight_end,
        );
        let tempo_split = self.music_chorus_tempo_split_between(control.item_id, next_item_id);
        let transition_source_rate = if plain_mix_capsule {
            1.0
        } else {
            tempo_split.incoming_rate
        };
        let outgoing_transition_rate = if plain_mix_capsule {
            1.0
        } else {
            tempo_split.outgoing_rate
        };
        let plan_confidence = self.music_chorus_pair_confidence(control.item_id, next_item_id);
        let render_mode = if current_segment.fallback_stage.is_stream_fallback() {
            MusicMixRenderMode::Streaming
        } else {
            self.music_stage_mix_render_mode()
        };
        let planned_reason = self.music_chorus_transition_reason_with_rate(
            control.item_id,
            next_item_id,
            transition_source_rate,
            outgoing_transition_rate,
            !plain_mix_capsule,
            Some(if plain_mix_capsule {
                (MusicMixWindowKind::Plain).detail_label()
            } else if mix_kind == MusicMixWindowKind::RewardLong {
                mix_kind.detail_label()
            } else {
                render_mode.detail_label()
            }),
        );

        let (sender, receiver) = std::sync::mpsc::channel();
        let current_item_id = control.item_id;
        let session_id = control.session_id;
        let output_sample_rate = control.output_sample_rate();
        let output_channels = control.output_channels();
        let music_stream_ext = item.music_stream_ext.clone();
        let next_duration_seconds = self.music_chorus_duration_seconds_for_item(next_item_id);
        let original_entry_start = entry_start;
        entry_start = music_stage_chain_safe_entry_start_seconds(
            entry_start,
            transition_seconds,
            next_duration_seconds,
        );
        if MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX && (original_entry_start - entry_start).abs() > 0.050
        {
            let runway = next_duration_seconds
                .map(|duration| (duration - entry_start).max(0.0))
                .unwrap_or(0.0);
            eprintln!(
                "[music-stage-chain] entry pullback item={} old={:.3}s new={:.3}s runway={:.3}s",
                next_item_id, original_entry_start, entry_start, runway
            );
        }
        let base_source_duration_seconds = (highlight_end - entry_start).max(transition_seconds);
        let source_duration_seconds = music_stage_lite_promoted_deck_source_duration_seconds(
            entry_start,
            base_source_duration_seconds,
            transition_seconds,
            next_duration_seconds,
        );
        if MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX
            && source_duration_seconds > base_source_duration_seconds + 0.5
        {
            eprintln!(
                "[music-stage-lite] preview deck decode item={} entry={:.3}s duration={:.3}s base={:.3}s",
                next_item_id, entry_start, source_duration_seconds, base_source_duration_seconds
            );
        }
        let source_duration = Duration::from_secs_f64(source_duration_seconds);
        let transition_duration = Duration::from_secs_f64(transition_seconds);
        let current_mix_start_seconds = (planned_segment_end - transition_seconds).max(0.0);
        if MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX {
            eprintln!(
                "[music-stage-lite] preview item={}->{} transition={:.3}s A_start={:.3}s B_start={:.3}s owner=callback-only",
                control.item_id,
                next_item_id,
                transition_seconds,
                current_mix_start_seconds,
                entry_start
            );
        }
        let outgoing_highlight_end_phase = self
            .music_stage_outgoing_highlight_end_phase_for_transition(
                control.item_id,
                Some(current_segment),
                planned_segment_end,
                transition_seconds,
            );

        let spawn_result = std::thread::Builder::new()
            .name("music-automix-preview".to_owned())
            .spawn(move || {
                let build_realtime_preview = |fallback_note: Option<String>| {
                    crate::app::music_stream::decode_music_file_segment_for_mix(
                        &media_path,
                        &music_stream_ext,
                        entry_start,
                        source_duration,
                        transition_duration,
                        transition_source_rate,
                        output_sample_rate,
                        output_channels,
                        render_mode,
                    )
                    .map(|preview| {
                        let base_detail = if plain_mix_capsule {
                            Some((MusicMixWindowKind::Plain).detail_label().to_owned())
                        } else if mix_kind == MusicMixWindowKind::RewardLong {
                            Some(match preview.stretch_detail {
                                Some(detail) if !detail.trim().is_empty() => {
                                    format!("{} · {}", mix_kind.detail_label(), detail)
                                }
                                _ => mix_kind.detail_label().to_owned(),
                            })
                        } else {
                            preview.stretch_detail
                        };
                        let stretch_detail = match (fallback_note, base_detail) {
                            (Some(note), Some(detail)) if !detail.trim().is_empty() => {
                                Some(format!("{note} · {detail}"))
                            }
                            (Some(note), _) => Some(note),
                            (None, detail) => detail,
                        };

                        MusicChorusPreparedPreview {
                            current_item_id,
                            session_id,
                            next_item_id,
                            next_duration_seconds,
                            entry_start_seconds: entry_start,
                            transition_seconds,
                            transition_output_frames: preview.transition_output_frames,
                            transition_source_frames: preview.transition_source_frames,
                            source_start_frame: preview.source_start_frame,
                            prepared_mix: false,
                            prepared_mix_source_frames: MusicMixFrameCount::ZERO,
                            prepared_mix_resume_source_frame: None,
                            prepared_mix_start_seconds: None,
                            prepared_mix_b_samples: None,
                            source_sample_rate: preview.source_sample_rate,
                            plan_confidence,
                            preview_rate: preview.transition_source_rate,
                            outgoing_rate: outgoing_transition_rate,
                            preserve_pitch: preview.preserve_pitch && !plain_mix_capsule,
                            stretch_detail,
                            outgoing_highlight_end_phase,
                            samples: preview.samples,
                        }
                    })
                };

                let result = if plain_mix_capsule {
                    build_realtime_preview(None)
                } else if let Some(current_media_path) = current_media_path.as_ref() {
                    let render_started = Instant::now();
                    eprintln!(
                        "[music-stage-prepared] render start item={current_item_id}->{next_item_id} transition={transition_seconds:.3}s A_start={current_mix_start_seconds:.3}s B_start={entry_start:.3}s mode={}",
                        render_mode.label()
                    );
                    match crate::app::music_stream::render_music_prepared_mix_segment(
                        current_media_path,
                        &current_music_stream_ext,
                        current_mix_start_seconds,
                        &media_path,
                        &music_stream_ext,
                        entry_start,
                        source_duration,
                        transition_duration,
                        transition_source_rate,
                        output_sample_rate,
                        output_channels,
                        render_mode,
                    ) {
                        Ok(prepared) => {
                            let elapsed_ms = render_started.elapsed().as_secs_f64() * 1000.0;
                            eprintln!(
                                "[music-stage-prepared] rendered item={current_item_id}->{next_item_id} elapsed={elapsed_ms:.1}ms mix_frames={} samples={}",
                                prepared.mix_output_frames.get(),
                                prepared.samples.len()
                            );
                            let stretch_detail = if mix_kind == MusicMixWindowKind::RewardLong {
                                Some(match prepared.stretch_detail {
                                    Some(detail) if !detail.trim().is_empty() => {
                                        format!("{} · {}", mix_kind.detail_label(), detail)
                                    }
                                    _ => mix_kind.detail_label().to_owned(),
                                })
                            } else {
                                prepared.stretch_detail
                            };
                            Ok(MusicChorusPreparedPreview {
                                current_item_id,
                                session_id,
                                next_item_id,
                                next_duration_seconds,
                                entry_start_seconds: entry_start,
                                transition_seconds,
                                transition_output_frames: prepared.mix_output_frames,
                                transition_source_frames: prepared.mix_source_frames,
                                source_start_frame: prepared.b_resume_source_frame,
                                prepared_mix: true,
                                prepared_mix_source_frames: prepared.mix_source_frames,
                                prepared_mix_resume_source_frame: Some(
                                    prepared.b_resume_source_frame,
                                ),
                                prepared_mix_start_seconds: Some(current_mix_start_seconds),
                                prepared_mix_b_samples: Some(prepared.b_samples),
                                source_sample_rate: prepared.source_sample_rate,
                                plan_confidence,
                                preview_rate: prepared.transition_source_rate,
                                outgoing_rate: outgoing_transition_rate,
                                preserve_pitch: prepared.preserve_pitch,
                                stretch_detail,
                                outgoing_highlight_end_phase,
                                samples: prepared.samples,
                            })
                        }
                        Err(error) => {
                            let elapsed_ms = render_started.elapsed().as_secs_f64() * 1000.0;
                            eprintln!(
                                "[music-stage-prepared] fallback item={current_item_id}->{next_item_id} elapsed={elapsed_ms:.1}ms: {error}"
                            );
                            build_realtime_preview(Some(format!(
                                "Prepared Mix fallback · {error}"
                            )))
                        }
                    }
                } else {
                    eprintln!(
                        "[music-stage-prepared] fallback item={current_item_id}->{next_item_id}: current cache unavailable"
                    );
                    build_realtime_preview(Some(
                        "Prepared Mix fallback · current cache unavailable".to_owned(),
                    ))
                };
                let _ = sender.send(result);
            });

        if let Err(error) = spawn_result {
            self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                transition_seconds,
                confidence: plan_confidence,
                reason: format!("preview off · worker failed ({error})"),
            });
            return false;
        }

        self.music.music_chorus_preview_job = Some(MusicChorusPreviewJob {
            current_item_id,
            session_id,
            next_item_id,
            transition_seconds,
            started_at: Instant::now(),
            receiver,
        });
        self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
            transition_seconds,
            confidence: plan_confidence,
            reason: planned_reason,
        });
        if let Some(active_segment) = self.music.music_chorus_flow_segment.as_mut() {
            if active_segment.item_id == current_item_id && active_segment.session_id == session_id
            {
                active_segment.transition_seconds = transition_seconds;
                if mix_kind == MusicMixWindowKind::RewardLong {
                    active_segment.end_seconds =
                        active_segment.end_seconds.max(planned_segment_end);
                }
            }
        }
        true
    }

    fn try_start_plain_mix_capsule_preview_inline(
        &mut self,
        control: &MusicPlaybackControl,
        next_item_id: QueueItemId,
        transition_seconds: f64,
        planned_entry_start_seconds: Option<f64>,
    ) -> Option<(f64, f32, String, MusicMixOutputFrame, MusicMixFrameCount)> {
        let Some((highlight_start, highlight_end)) = self
            .music_automix_range_for_item(next_item_id)
            .or_else(|| self.music_full_range_for_item(next_item_id))
        else {
            return None;
        };
        let Some(item) = self.queue_item_by_id(next_item_id).cloned() else {
            return None;
        };
        let Some(media_path) = self.complete_music_cache_media_path(&item) else {
            self.start_music_prefetch_for_item(next_item_id);
            return None;
        };

        let mut entry_start = planned_entry_start_seconds.unwrap_or_else(|| {
            self.music_lyrics_safe_entry_start_for_item(
                next_item_id,
                self.music_automix_entry_start_for_item(
                    next_item_id,
                    highlight_start,
                    highlight_end,
                    transition_seconds,
                ),
                highlight_start,
                highlight_end,
            )
        });
        let next_duration_seconds = self.music_chorus_duration_seconds_for_item(next_item_id);
        let original_entry_start = entry_start;
        entry_start = music_stage_chain_safe_entry_start_seconds(
            entry_start,
            transition_seconds,
            next_duration_seconds,
        );
        if MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX && (original_entry_start - entry_start).abs() > 0.050
        {
            let runway = next_duration_seconds
                .map(|duration| (duration - entry_start).max(0.0))
                .unwrap_or(0.0);
            eprintln!(
                "[music-stage-chain] inline entry pullback item={} old={:.3}s new={:.3}s runway={:.3}s",
                next_item_id, original_entry_start, entry_start, runway
            );
        }
        let base_source_duration_seconds = (highlight_end - entry_start).max(transition_seconds);
        let source_duration_seconds = music_stage_lite_promoted_deck_source_duration_seconds(
            entry_start,
            base_source_duration_seconds,
            transition_seconds,
            next_duration_seconds,
        );
        if MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX
            && source_duration_seconds > base_source_duration_seconds + 0.5
        {
            eprintln!(
                "[music-stage-lite] promoted deck decode item={} entry={:.3}s duration={:.3}s base={:.3}s",
                next_item_id, entry_start, source_duration_seconds, base_source_duration_seconds
            );
        }
        let source_duration = Duration::from_secs_f64(source_duration_seconds);
        let transition_duration = Duration::from_secs_f64(transition_seconds);
        let output_sample_rate = control.output_sample_rate();
        let output_channels = control.output_channels();
        let music_stream_ext = item.music_stream_ext.clone();
        let preview = match crate::app::music_stream::decode_music_file_segment_for_mix(
            &media_path,
            &music_stream_ext,
            entry_start,
            source_duration,
            transition_duration,
            1.0,
            output_sample_rate,
            output_channels,
            MusicMixRenderMode::Streaming,
        ) {
            Ok(preview) => preview,
            Err(error) => {
                self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                    transition_seconds,
                    confidence: 0.0,
                    reason: format!("plain mix inline decode failed · {error}"),
                });
                return None;
            }
        };

        let transition_output_frames = preview.transition_output_frames;
        let started_output_frame = control.start_crossfade_preview(
            preview.samples,
            transition_output_frames,
            self.music.music_volume,
            preview.source_start_frame,
            preview.source_sample_rate,
            preview.transition_source_frames,
            next_duration_seconds,
            1.0,
            None,
        );
        let Some(started_output_frame) = started_output_frame else {
            self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                transition_seconds,
                confidence: 0.0,
                reason: "plain mix inline deck rejected".to_owned(),
            });
            return None;
        };

        Some((
            entry_start,
            self.music_chorus_pair_confidence(control.item_id, next_item_id),
            self.music_chorus_transition_reason_with_rate(
                control.item_id,
                next_item_id,
                1.0,
                1.0,
                false,
                Some("Plain Mix inline"),
            ),
            started_output_frame,
            transition_output_frames,
        ))
    }

    fn begin_music_chorus_fade_out(&mut self, control: &MusicPlaybackControl) {
        if self.music.music_chorus_fade_out.is_some() {
            return;
        }

        if music_stage_chain_direct_stream_director_enabled() {
            // Stage Chain stream handoff uses a real B stream instead of a
            // promoted finite preview deck. Drop preview state before planning
            // so this call cannot arm the old preview/promote path.
            self.music.music_chorus_preview_job = None;
            self.music.music_chorus_ready_preview = None;
        } else {
            self.poll_music_chorus_preview_job(control);
        }

        let pending_target = self
            .music
            .music_chorus_pending_mix_target
            .clone()
            .filter(|target| {
                target.current_item_id == control.item_id && target.session_id == control.session_id
            });
        let ready_preview_target = self
            .music
            .music_chorus_ready_preview
            .as_ref()
            .filter(|preview| {
                preview.current_item_id == control.item_id
                    && preview.session_id == control.session_id
            })
            .map(|preview| preview.next_item_id);
        let next = pending_target
            .as_ref()
            .map(|target| target.target_item_id)
            .or(ready_preview_target)
            .or_else(|| self.next_music_chorus_flow_item_id(control.item_id));
        let mut current_segment = self.music.music_chorus_flow_segment.clone();
        let mut transition_seconds = current_segment
            .as_ref()
            .map(|segment| segment.transition_seconds)
            .unwrap_or_else(|| self.music_chorus_transition_seconds_for_item(control.item_id));
        let mut next_start_seconds = None;
        let mut crossfade_preview_started = false;
        let mut prepared_mix_started = false;
        let mut crossfade_started_output_frame = None;
        let mut crossfade_duration_output_frames = None;
        let mut plan_confidence = self.music_chorus_tempo_confidence_for_item(control.item_id);
        let mut plan_reason = "current track beat window".to_owned();
        let mut direct_stage_chain_fade_lengths: Option<(f64, f64)> = None;
        let mut planned_segment_end = current_segment
            .as_ref()
            .map(|segment| segment.end_seconds)
            .unwrap_or_else(|| control.playback_seconds().max(0.0) + transition_seconds);
        let mut locked_plan_transition_seconds = transition_seconds;
        let mut locked_plan_segment_end = planned_segment_end;

        if let Some(next_item_id) = next {
            let replace_stage_pick = pending_target
                .as_ref()
                .is_some_and(|target| target.target_item_id != next_item_id);
            self.ensure_music_stage_pick_for_item(next_item_id, replace_stage_pick);
            if let Some((highlight_start, highlight_end)) = self
                .music_automix_range_for_item(next_item_id)
                .or_else(|| self.music_full_range_for_item(next_item_id))
            {
                let current_end_seconds = {
                    let segment_end = current_segment.as_ref().map(|segment| segment.end_seconds);
                    match (
                        self.music_chorus_stage_range_end_seconds_for_item(control.item_id),
                        segment_end,
                    ) {
                        (Some(stage_end), Some(segment_end)) => stage_end.max(segment_end),
                        (Some(stage_end), None) => stage_end,
                        (None, Some(segment_end)) => segment_end,
                        (None, None) => control.playback_seconds().max(0.0) + transition_seconds,
                    }
                };
                let current_len = current_segment
                    .as_ref()
                    .map(|segment| current_end_seconds - segment.start_seconds)
                    .unwrap_or(transition_seconds * 3.0);
                let next_len = highlight_end - highlight_start;
                direct_stage_chain_fade_lengths = Some((current_len, next_len));
                let plain_mix_capsule = MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX
                    || current_segment
                        .as_ref()
                        .is_some_and(|segment| segment.fallback_stage.is_plain_crossfade());
                let (base_transition, mix_kind) = if plain_mix_capsule {
                    (
                        self.music_chorus_transition_seconds_between(control.item_id, next_item_id),
                        MusicMixWindowKind::Plain,
                    )
                } else {
                    self.music_chorus_stream_transition_seconds_and_kind_between(
                        control.item_id,
                        next_item_id,
                    )
                };
                let reward_tail_room =
                    if !plain_mix_capsule && mix_kind == MusicMixWindowKind::RewardLong {
                        self.music_chorus_reward_tail_extension_seconds_between(
                            control.item_id,
                            next_item_id,
                            current_end_seconds,
                            base_transition,
                        )
                    } else {
                        0.0
                    };
                let reward_transition_seed =
                    if !plain_mix_capsule && mix_kind == MusicMixWindowKind::RewardLong {
                        self.music_chorus_reward_transition_seed_seconds_between(
                            control.item_id,
                            next_item_id,
                            base_transition,
                            reward_tail_room,
                        )
                    } else {
                        base_transition
                    };
                let transition_current_len = current_len + reward_tail_room;
                transition_seconds = if plain_mix_capsule {
                    clamp_music_chorus_mix_capsule_transition_seconds(
                        base_transition,
                        current_len,
                        next_len,
                    )
                } else {
                    clamp_music_chorus_transition_seconds(
                        reward_transition_seed,
                        transition_current_len,
                        next_len,
                    )
                };
                if MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX {
                    transition_seconds = clamp_music_stage_lite_transition_seconds(
                        transition_seconds,
                        current_len,
                        next_len,
                    );
                }
                planned_segment_end =
                    if !plain_mix_capsule && mix_kind == MusicMixWindowKind::RewardLong {
                        self.music_chorus_reward_extended_end_seconds(
                            control.item_id,
                            next_item_id,
                            current_end_seconds,
                            transition_seconds,
                        )
                    } else {
                        current_end_seconds
                    };
                if let Some(segment) = current_segment.as_ref() {
                    let locked_segment_end = segment
                        .hold_end_seconds
                        .unwrap_or(segment.end_seconds)
                        .max(segment.end_seconds);
                    if planned_segment_end + 0.001 < locked_segment_end {
                        eprintln!(
                            "[music-stage-prepared] align execute window item={}->{} planned_end={:.3}s locked_end={:.3}s",
                            control.item_id, next_item_id, planned_segment_end, locked_segment_end
                        );
                        planned_segment_end = locked_segment_end;
                    }
                }
                let mut entry_start = pending_target
                    .as_ref()
                    .and_then(|target| target.target_start_seconds)
                    .unwrap_or_else(|| {
                        self.music_lyrics_safe_entry_start_for_item(
                            next_item_id,
                            self.music_automix_entry_start_for_item(
                                next_item_id,
                                highlight_start,
                                highlight_end,
                                transition_seconds,
                            ),
                            highlight_start,
                            highlight_end,
                        )
                    });
                let next_duration_seconds =
                    self.music_chorus_duration_seconds_for_item(next_item_id);
                let original_entry_start = entry_start;
                entry_start = self.music_stage_chain_direct_entry_anchor_start_for_item(
                    next_item_id,
                    entry_start,
                    transition_seconds,
                    next_duration_seconds,
                );
                if MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX
                    && (original_entry_start - entry_start).abs() > 0.050
                {
                    let runway = next_duration_seconds
                        .map(|duration| (duration - entry_start).max(0.0))
                        .unwrap_or(0.0);
                    eprintln!(
                        "[music-stage-chain] execute entry pullback item={} old={:.3}s new={:.3}s runway={:.3}s",
                        next_item_id, original_entry_start, entry_start, runway
                    );
                }
                next_start_seconds = Some(entry_start);
                plan_confidence = self.music_chorus_pair_confidence(control.item_id, next_item_id);
                let tempo_split =
                    self.music_chorus_tempo_split_between(control.item_id, next_item_id);
                let plain_mix_capsule = MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX
                    || current_segment
                        .as_ref()
                        .is_some_and(|segment| segment.fallback_stage.is_plain_crossfade());
                let transition_source_rate = if plain_mix_capsule {
                    1.0
                } else {
                    tempo_split.incoming_rate
                };
                let outgoing_transition_rate = if plain_mix_capsule {
                    1.0
                } else {
                    tempo_split.outgoing_rate
                };
                plan_reason = self.music_chorus_transition_reason_with_rate(
                    control.item_id,
                    next_item_id,
                    transition_source_rate,
                    outgoing_transition_rate,
                    !plain_mix_capsule,
                    Some(if plain_mix_capsule {
                        (MusicMixWindowKind::Plain).detail_label()
                    } else {
                        mix_kind.detail_label()
                    }),
                );
                locked_plan_transition_seconds = transition_seconds;
                locked_plan_segment_end = planned_segment_end;

                if !music_stage_chain_direct_stream_director_enabled() {
                    if self.music.music_chorus_preview_job.is_none()
                        && self.music.music_chorus_ready_preview.is_none()
                    {
                        if let Some(segment) = current_segment.as_ref() {
                            self.prepare_music_chorus_preview_for_control(control, segment);
                        }
                        self.poll_music_chorus_preview_job(control);
                    }

                    let ready_preview = self.music.music_chorus_ready_preview.take();
                    if let Some(mut preview) = ready_preview {
                        let matches_transition = preview.current_item_id == control.item_id
                            && preview.session_id == control.session_id
                            && preview.next_item_id == next_item_id;
                        if matches_transition {
                            let mut preview_transition_frames = preview.transition_output_frames;
                            if preview.prepared_mix {
                                if let Some(prepared_start) = preview.prepared_mix_start_seconds {
                                    let playback_at_handoff = control.playback_seconds().max(0.0);
                                    let start_delta = playback_at_handoff - prepared_start;
                                    if start_delta < -MUSIC_PREPARED_MIX_START_EARLY_HOLD_SECONDS {
                                        self.music.music_chorus_ready_preview = Some(preview);
                                        self.music.music_chorus_mix_plan =
                                            Some(MusicChorusMixPlan {
                                                transition_seconds,
                                                confidence: plan_confidence,
                                                reason: format!(
                                                    "Prepared Mix cue wait · {:.0}ms early",
                                                    (-start_delta * 1000.0).round()
                                                ),
                                            });
                                        self.last_action =
                                            "Stage Mix: waiting for Prepared Mix frame start."
                                                .to_owned();
                                        return;
                                    }
                                    if start_delta > 0.0 {
                                        let requested_drop_frames =
                                            control.mix_frame_count_from_seconds(start_delta);
                                        let max_drop_frames =
                                            preview_transition_frames.get().saturating_sub(64);
                                        let drop_frames =
                                            requested_drop_frames.get().min(max_drop_frames);
                                        let remaining_after_drop_frames = preview_transition_frames
                                            .get()
                                            .saturating_sub(drop_frames);
                                        let remaining_after_drop_seconds =
                                            MusicMixFrameClock::new(control.output_sample_rate())
                                                .seconds_from_frame_count(MusicMixFrameCount::new(
                                                    remaining_after_drop_frames,
                                                ));
                                        if start_delta > MUSIC_PREPARED_MIX_LATE_TRIM_MAX_SECONDS
                                            && remaining_after_drop_seconds
                                                < MUSIC_PREPARED_MIX_LATE_REPLAN_MIN_REMAINING_SECONDS
                                        {
                                            let mut pushed_end = playback_at_handoff
                                                + transition_seconds
                                                + MUSIC_PREPARED_MIX_LATE_REPLAN_PAD_SECONDS;
                                            if let Some(duration) = control.duration_seconds() {
                                                if duration > playback_at_handoff {
                                                    let safe_end = (duration
                                                        - MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS)
                                                        .max(playback_at_handoff);
                                                    pushed_end = pushed_end.min(safe_end);
                                                }
                                            }
                                            if let Some(active_segment) =
                                                self.music.music_chorus_flow_segment.as_mut()
                                            {
                                                if active_segment.item_id == control.item_id
                                                    && active_segment.session_id == control.session_id
                                                {
                                                    active_segment.end_seconds =
                                                        active_segment.end_seconds.max(pushed_end);
                                                    active_segment.hold_end_seconds =
                                                        Some(active_segment.end_seconds);
                                                }
                                            }
                                            self.music.music_chorus_mix_plan = Some(
                                                MusicChorusMixPlan {
                                                    transition_seconds,
                                                    confidence: plan_confidence,
                                                    reason: format!(
                                                        "Prepared Mix too late · replan · {:.1}s late / {:.1}s left",
                                                        start_delta,
                                                        remaining_after_drop_seconds
                                                    ),
                                                },
                                            );
                                            self.last_action =
                                                "Stage Mix: discarded a late Prepared Mix and replanned the handoff."
                                                    .to_owned();
                                            eprintln!(
                                                "[music-stage-prepared] late reject item={}->{} late={start_delta:.3}s remaining={remaining_after_drop_seconds:.3}s pushed_end={pushed_end:.3}s",
                                                control.item_id,
                                                next_item_id
                                            );
                                            return;
                                        }
                                        let trimmed_frames = trim_prepared_mix_leading_frames(
                                            &mut preview.samples,
                                            control.output_channels(),
                                            MusicMixFrameCount::new(drop_frames),
                                        );
                                        if !trimmed_frames.is_zero() {
                                            let original_transition_frames =
                                                preview_transition_frames.get().max(1);
                                            preview_transition_frames = MusicMixFrameCount::new(
                                                preview_transition_frames
                                                    .get()
                                                    .saturating_sub(trimmed_frames.get())
                                                    .max(1),
                                            );
                                            let remaining_ratio = preview_transition_frames.get()
                                                as f64
                                                / original_transition_frames as f64;
                                            preview.transition_output_frames =
                                                preview_transition_frames;
                                            preview.prepared_mix_start_seconds = Some(
                                                prepared_start
                                                    + MusicMixFrameClock::new(
                                                        control.output_sample_rate(),
                                                    )
                                                    .seconds_from_frame_count(trimmed_frames),
                                            );
                                            preview.transition_seconds = MusicMixFrameClock::new(
                                                control.output_sample_rate(),
                                            )
                                            .seconds_from_frame_count(preview_transition_frames);
                                            preview.prepared_mix_source_frames = scale_frame_count(
                                                preview.prepared_mix_source_frames,
                                                remaining_ratio,
                                            );
                                            preview.transition_source_frames =
                                                preview.prepared_mix_source_frames;
                                            if start_delta
                                                > MUSIC_PREPARED_MIX_LATE_TRIM_MAX_SECONDS
                                            {
                                                eprintln!(
                                                    "[music-stage-prepared] large late trim item={}->{} late={start_delta:.3}s frames={}",
                                                    control.item_id,
                                                    next_item_id,
                                                    trimmed_frames.get()
                                                );
                                            } else {
                                                eprintln!(
                                                    "[music-stage-prepared] late trim item={}->{} late={start_delta:.3}s frames={}",
                                                    control.item_id,
                                                    next_item_id,
                                                    trimmed_frames.get()
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            transition_seconds = preview.transition_seconds;
                            if preview.prepared_mix {
                                if let Some(prepared_start) = preview.prepared_mix_start_seconds {
                                    let prepared_end = prepared_start + transition_seconds.max(0.0);
                                    if prepared_end.is_finite() && prepared_end > prepared_start {
                                        if (locked_plan_segment_end - prepared_end).abs() > 0.050 {
                                            eprintln!(
                                                "[music-stage-prepared] lock execute to rendered window item={}->{} old_end={:.3}s rendered_start={:.3}s rendered_end={:.3}s",
                                                control.item_id,
                                                next_item_id,
                                                locked_plan_segment_end,
                                                prepared_start,
                                                prepared_end
                                            );
                                        }
                                        planned_segment_end = prepared_end;
                                        locked_plan_segment_end = prepared_end;
                                        locked_plan_transition_seconds = transition_seconds;
                                    }
                                }
                            }
                            plan_confidence = preview.plan_confidence;
                            let preview_is_reward_long = preview
                                .stretch_detail
                                .as_deref()
                                .is_some_and(|detail| detail.contains("Reward Long Mix"));
                            plan_reason = self.music_chorus_transition_reason_with_rate(
                                control.item_id,
                                next_item_id,
                                preview.preview_rate,
                                preview.outgoing_rate,
                                preview.preserve_pitch || preview_is_reward_long,
                                preview.stretch_detail.as_deref(),
                            );
                            let preview_entry_start_seconds = preview.entry_start_seconds;
                            let preview_started_output_frame = if preview.prepared_mix {
                                let prepared_start_for_log = preview.prepared_mix_start_seconds;
                                let playback_before_handoff = control.playback_seconds().max(0.0);
                                if let Some(prepared_start) = preview.prepared_mix_start_seconds {
                                    let start_delta = playback_before_handoff - prepared_start;
                                    if start_delta.abs() > 0.55 {
                                        eprintln!(
                                            "[music-stage-prepared] start offset item={}->{} delta={start_delta:.3}s planned={prepared_start:.3}s",
                                            control.item_id, next_item_id
                                        );
                                    }
                                }
                                let prepared_start_text = prepared_start_for_log
                                    .map(|seconds| format!("{seconds:.3}s"))
                                    .unwrap_or_else(|| "n/a".to_owned());
                                let prepared_offset_millis = prepared_start_for_log
                                    .map(|seconds| (playback_before_handoff - seconds) * 1000.0)
                                    .unwrap_or(0.0);
                                eprintln!(
                                    "[music-stage-prepared] handoff item={}->{} mix_frames={} b_source_frames={} start={} playback={playback_before_handoff:.3}s offset={prepared_offset_millis:.1}ms",
                                    control.item_id,
                                    next_item_id,
                                    preview_transition_frames.get(),
                                    preview.prepared_mix_source_frames.get(),
                                    prepared_start_text
                                );
                                control.start_prepared_mix_handoff(
                                    preview.samples,
                                    preview.prepared_mix_b_samples,
                                    preview_transition_frames,
                                    self.music.music_volume,
                                    preview
                                        .prepared_mix_resume_source_frame
                                        .unwrap_or(preview.source_start_frame),
                                    preview.source_sample_rate,
                                    preview.next_duration_seconds,
                                    preview.prepared_mix_start_seconds,
                                )
                            } else {
                                control.start_crossfade_preview(
                                    preview.samples,
                                    preview_transition_frames,
                                    self.music.music_volume,
                                    preview.source_start_frame,
                                    preview.source_sample_rate,
                                    preview.transition_source_frames,
                                    preview.next_duration_seconds,
                                    preview.outgoing_rate,
                                    preview.outgoing_highlight_end_phase,
                                )
                            };
                            crossfade_preview_started = preview_started_output_frame.is_some();
                            if crossfade_preview_started {
                                prepared_mix_started = preview.prepared_mix;
                                crossfade_started_output_frame = preview_started_output_frame;
                                crossfade_duration_output_frames = Some(preview_transition_frames);
                                next_start_seconds = Some(preview_entry_start_seconds);
                            } else {
                                plan_reason = self.music_chorus_transition_reason_with_rate(
                                    control.item_id,
                                    next_item_id,
                                    transition_source_rate,
                                    outgoing_transition_rate,
                                    true,
                                    Some("preview rejected · realtime-safe"),
                                );
                            }
                        } else {
                            self.music.music_chorus_ready_preview = None;
                            plan_reason = self.music_chorus_transition_reason_with_rate(
                                control.item_id,
                                next_item_id,
                                transition_source_rate,
                                outgoing_transition_rate,
                                true,
                                Some("preview stale · realtime-safe"),
                            );
                            let _ = entry_start;
                        }
                    }
                } else {
                    self.music.music_chorus_preview_job = None;
                    self.music.music_chorus_ready_preview = None;
                    plan_reason = format!("Stage Chain · Direct Stream · {plan_reason}");
                }
            }
        }

        let mut plain_crossfade_fallback = MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX
            || current_segment
                .as_ref()
                .is_some_and(|segment| segment.fallback_stage.is_plain_crossfade());
        // Stage Mix must not start fading A unless B has an audio deck ready.
        // The only exception is the explicit plain fallback path below, which
        // has its own inline/last-resort handoff bridge.
        if !plain_crossfade_fallback && !crossfade_preview_started && next.is_some() {
            if let Some(segment) = current_segment.as_ref() {
                if self.hold_music_stage_mix_until_preview_ready(
                    control,
                    segment,
                    transition_seconds,
                ) {
                    self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                        transition_seconds,
                        confidence: plan_confidence,
                        reason: format!("waiting for B preview · {plan_reason}"),
                    });
                    self.last_action =
                        "Stage Mix: holding A until B preview is playable.".to_owned();
                    return;
                }
            }
            current_segment = self.music.music_chorus_flow_segment.clone();
            plain_crossfade_fallback = MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX
                || current_segment
                    .as_ref()
                    .is_some_and(|segment| segment.fallback_stage.is_plain_crossfade());
            if !plain_crossfade_fallback {
                self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                    transition_seconds,
                    confidence: plan_confidence,
                    reason: format!("preview not playable · hold retry · {plan_reason}"),
                });
                self.last_action =
                    "Stage Mix: skipped quick handoff because B preview is not playable."
                        .to_owned();
                return;
            }
        }
        if plain_crossfade_fallback
            && !crossfade_preview_started
            && !MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY
        {
            if let Some(next_item_id) = next {
                if let Some((
                    entry_start,
                    confidence,
                    reason,
                    started_output_frame,
                    duration_output_frames,
                )) = self.try_start_plain_mix_capsule_preview_inline(
                    control,
                    next_item_id,
                    transition_seconds,
                    next_start_seconds,
                ) {
                    crossfade_preview_started = true;
                    prepared_mix_started = false;
                    crossfade_started_output_frame = Some(started_output_frame);
                    crossfade_duration_output_frames = Some(duration_output_frames);
                    next_start_seconds = Some(entry_start);
                    plan_confidence = confidence;
                    plan_reason = reason;
                }
            }
        }

        let mut fade_duration_seconds = if crossfade_preview_started {
            transition_seconds
        } else if plain_crossfade_fallback {
            transition_seconds.clamp(
                MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS,
                MUSIC_CHORUS_MIX_CAPSULE_MAX_SECONDS,
            )
        } else {
            MUSIC_CHORUS_REALTIME_FALLBACK_FADE_SECONDS.min(transition_seconds)
        };
        if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY
            && plain_crossfade_fallback
            && !crossfade_preview_started
        {
            if let Some(next_item_id) = next {
                let (tempo_fade_seconds, tempo_note) = self
                    .music_stage_chain_direct_tempo_fade_seconds_between(
                        control.item_id,
                        next_item_id,
                        fade_duration_seconds,
                        direct_stage_chain_fade_lengths,
                    );
                if (tempo_fade_seconds - fade_duration_seconds).abs() > 0.050 {
                    eprintln!(
                        "[music-stage-chain] direct tempo fade item={}->{} old={:.3}s new={:.3}s {}",
                        control.item_id,
                        next_item_id,
                        fade_duration_seconds,
                        tempo_fade_seconds,
                        tempo_note.as_deref().unwrap_or("model=unavailable")
                    );
                }
                fade_duration_seconds = tempo_fade_seconds;
                if let Some(note) = tempo_note {
                    plan_reason = format!("{plan_reason} · {note}");
                }

                let (mix_length_seconds, mix_length_note) = self
                    .music_stage_chain_direct_mix_length_seconds(
                        control.item_id,
                        next_item_id,
                        fade_duration_seconds,
                        direct_stage_chain_fade_lengths,
                    );
                if (mix_length_seconds - fade_duration_seconds).abs() > 0.050 {
                    eprintln!(
                        "[music-stage-chain] direct mix length item={}->{} old={:.3}s new={:.3}s {}",
                        control.item_id,
                        next_item_id,
                        fade_duration_seconds,
                        mix_length_seconds,
                        mix_length_note.as_deref().unwrap_or("manual=neutral")
                    );
                }
                fade_duration_seconds = mix_length_seconds;
                if let Some(note) = mix_length_note {
                    plan_reason = format!("{plan_reason} · {note}");
                }
            }
        }
        let direct_tempo_bridge = if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY
            && plain_crossfade_fallback
            && !crossfade_preview_started
        {
            next.and_then(|next_item_id| {
                self.music_stage_chain_direct_tempo_bridge_between(
                    control.item_id,
                    next_item_id,
                    fade_duration_seconds,
                )
            })
        } else {
            None
        };
        if let Some(bridge) = direct_tempo_bridge.as_ref() {
            plan_reason = format!("{plan_reason} · {}", bridge.note);
        }

        if plain_crossfade_fallback {
            plan_reason = if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY {
                music_stage_chain_stream_handoff_reason(&plan_reason)
            } else if crossfade_preview_started {
                if MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX {
                    format!("Stage Mix Lite · callback-only · {plan_reason}")
                } else {
                    format!("plain mix · {plan_reason}")
                }
            } else if MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX {
                format!("Stage Mix Lite · live fallback · {plan_reason}")
            } else {
                format!("plain live fallback · {plan_reason}")
            };
        }

        if plain_crossfade_fallback && !crossfade_preview_started {
            if let Some(next_item_id) = next {
                self.music.music_chorus_preview_job = None;
                self.music.music_chorus_ready_preview = None;
                self.music.music_chorus_pending_mix_target = None;
                self.music.music_chorus_fade_out = None;
                self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                    transition_seconds: fade_duration_seconds,
                    confidence: plan_confidence,
                    reason: if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY {
                        plan_reason.clone()
                    } else {
                        format!("last-resort live handoff · {plan_reason}")
                    },
                });
                let next_duration_seconds =
                    self.music_chorus_duration_seconds_for_item(next_item_id);
                let next_start_before_tempo_bridge = next_start_seconds
                    .or_else(|| self.music_automix_entry_start_seconds_for_item(next_item_id))
                    .map(|start_seconds| {
                        let safe_start = self.music_stage_chain_direct_entry_anchor_start_for_item(
                            next_item_id,
                            start_seconds,
                            fade_duration_seconds,
                            next_duration_seconds,
                        );
                        if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY
                            && (safe_start - start_seconds).abs() > 0.050
                        {
                            let runway = next_duration_seconds
                                .map(|duration| (duration - safe_start).max(0.0))
                                .unwrap_or(0.0);
                            eprintln!(
                                "[music-stage-chain] stream entry pullback item={} old={:.3}s new={:.3}s runway={:.3}s",
                                next_item_id,
                                start_seconds,
                                safe_start,
                                runway
                            );
                        }
                        safe_start
                    });
                let next_start_seconds = next_start_before_tempo_bridge.map(|start_seconds| {
                    if let Some(bridge) = direct_tempo_bridge.as_ref() {
                        let shifted = self.music_stage_chain_direct_apply_tempo_bridge_entry_shift(
                            next_item_id,
                            start_seconds,
                            bridge.entry_shift_seconds,
                            fade_duration_seconds,
                            next_duration_seconds,
                        );
                        if (shifted - start_seconds).abs() > 0.040 {
                            eprintln!(
                                "[music-stage-chain] direct tempo bridge entry item={} old={:.3}s new={:.3}s shift={:+.3}s incoming_rate={:.4}",
                                next_item_id,
                                start_seconds,
                                shifted,
                                shifted - start_seconds,
                                bridge.incoming_rate
                            );
                        }
                        shifted
                    } else {
                        start_seconds
                    }
                });
                if let Some(start_seconds) = next_start_seconds {
                    self.prepare_music_chorus_start_for_item(next_item_id, start_seconds);
                }
                if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY {
                    eprintln!(
                        "[music-stage-chain] direct stream handoff item={}->{} fade={:.3}s B_start={} preview=disabled",
                        control.item_id,
                        next_item_id,
                        fade_duration_seconds,
                        next_start_seconds
                            .map(|seconds| format!("{seconds:.3}s"))
                            .unwrap_or_else(|| "auto".to_owned())
                    );
                }
                self.prepare_music_chorus_fade_in_for_item_with_duration_and_incoming_tempo(
                    next_item_id,
                    fade_duration_seconds,
                    direct_tempo_bridge
                        .as_ref()
                        .map(|bridge| bridge.incoming_rate)
                        .unwrap_or(1.0),
                );
                let fade_duration_output_frames =
                    control.mix_frame_count_from_seconds(fade_duration_seconds);
                let fade_started_output_frame = control.output_frame_cursor();
                if let Some(bridge) = direct_tempo_bridge.as_ref() {
                    control.start_outgoing_tempo_transition_from_output_frame(
                        bridge.outgoing_rate,
                        fade_duration_output_frames,
                        fade_started_output_frame,
                    );
                    eprintln!(
                        "[music-stage-chain] direct tempo bridge item={}->{} fade={:.3}s strength={:.0}% mix_len={:.0}% curve={:.0}% outgoing_rate={:.4} incoming_rate={:.4} entry_shift={:+.3}s {}",
                        control.item_id,
                        next_item_id,
                        fade_duration_seconds,
                        self.music_stage_direct_tempo_bridge_strength_value() * 100.0,
                        self.music_stage_direct_mix_length_value() * 100.0,
                        self.music_stage_direct_mix_curve_value() * 100.0,
                        bridge.outgoing_rate,
                        bridge.incoming_rate,
                        bridge.entry_shift_seconds,
                        bridge.note
                    );
                }
                control.fade_volume_to_from_output_frame_with_curve(
                    0.0,
                    fade_duration_output_frames,
                    fade_started_output_frame,
                    self.music_stage_direct_mix_curve_value(),
                );
                let bridge_lifetime_frames = control.mix_frame_count_from_seconds(
                    fade_duration_seconds + MUSIC_CHORUS_PLAIN_HANDOFF_STOP_GRACE_SECONDS,
                );
                self.music.music_chorus_handoff_bridge = Some(MusicChorusHandoffBridge {
                    control: control.clone(),
                    target_item_id: next_item_id,
                    stop_output_frame: Some(
                        fade_started_output_frame.saturating_add(bridge_lifetime_frames),
                    ),
                    visual_started_output_frame: fade_started_output_frame,
                    visual_duration_output_frames: fade_duration_output_frames,
                });
                self.emit_music_stage_transition_debug_stamp(
                    control.item_id,
                    next_item_id,
                    current_segment.as_ref(),
                    planned_segment_end,
                    next_start_seconds,
                    transition_seconds,
                    fade_duration_seconds,
                    false,
                    &plan_reason,
                );
                let session_id = self.next_music_playback_session_id();
                self.start_music_stream_playback_with_session(next_item_id, session_id);
                self.last_action = format!(
                    "Stage Chain: {:.1}s stream handoff ({plan_reason}).",
                    fade_duration_seconds
                );
                return;
            }
        }

        if let Some(active_segment) = self.music.music_chorus_flow_segment.as_mut() {
            if active_segment.item_id == control.item_id
                && active_segment.session_id == control.session_id
            {
                active_segment.transition_seconds = locked_plan_transition_seconds;
                if prepared_mix_started {
                    // Prepared Mix is sample-owned. Once a rendered segment is
                    // armed, the executed window must follow the prepared
                    // buffer, not a later reward-tail estimate; otherwise the
                    // UI/state waits for a second phantom transition after the
                    // audio already moved into B.
                    active_segment.end_seconds = locked_plan_segment_end;
                    active_segment.hold_end_seconds = Some(locked_plan_segment_end);
                } else if plan_reason.contains("Reward Long Mix") {
                    active_segment.end_seconds =
                        active_segment.end_seconds.max(locked_plan_segment_end);
                }
            }
        }

        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
            transition_seconds: fade_duration_seconds,
            confidence: plan_confidence,
            reason: plan_reason.clone(),
        });
        let fade_duration_output_frames = crossfade_duration_output_frames
            .unwrap_or_else(|| control.mix_frame_count_from_seconds(fade_duration_seconds));
        let fade_started_output_frame =
            crossfade_started_output_frame.unwrap_or_else(|| control.output_frame_cursor());
        let playback_seconds_at_fade = control.playback_seconds().max(0.0);
        let fallback_mix_window_end = playback_seconds_at_fade + fade_duration_seconds.max(0.0);
        let (mix_window_start_seconds, mix_window_end_seconds) = locked_stage_mix_window_seconds(
            locked_plan_segment_end,
            locked_plan_transition_seconds,
            fallback_mix_window_end,
        );
        let execution_route = MusicStageMixExecutionRoute::from_handoff_state(
            crossfade_preview_started,
            prepared_mix_started,
            plain_crossfade_fallback,
        );
        eprintln!(
            "[music-stage-exec] route locked item={} session={} route={} planned={:.3}s executed={:.3}s window={:.3}-{:.3}s frames={} reason={}",
            control.item_id,
            control.session_id,
            execution_route.label(),
            locked_plan_transition_seconds,
            fade_duration_seconds,
            mix_window_start_seconds,
            mix_window_end_seconds,
            fade_duration_output_frames.get(),
            plan_reason
        );
        if crossfade_preview_started {
            // Preview-deck handoffs are callback-owned.  The callback already
            // fades A against B using the deck envelope, and promotion preserves
            // the deck gain.  A second global fade-to-zero leaves stale volume
            // state that can sound like a second jump and can make promoted
            // Lite decks appear to stall near the old highlight tail.
            control.set_volume(self.music.music_volume);
        } else {
            control.fade_volume_to_from_output_frame(
                0.0,
                fade_duration_output_frames,
                fade_started_output_frame,
            );
        }
        self.music.music_chorus_fade_out = Some(MusicChorusFadeOut {
            item_id: control.item_id,
            session_id: control.session_id,
            execution_route,
            // This frame is shared with the audio fade state. Do not replace it
            // with Instant/UI time: callback progress is the transition truth.
            started_output_frame: fade_started_output_frame,
            duration_output_frames: fade_duration_output_frames,
            duration_seconds: fade_duration_seconds,
            planned_transition_seconds: locked_plan_transition_seconds,
            executed_transition_seconds: fade_duration_seconds,
            target_volume: self.music.music_volume,
            next_item_id: next,
            next_start_seconds,
            crossfade_preview_started,
            prepared_mix_started,
            plain_crossfade_fallback,
            start_playback_seconds: playback_seconds_at_fade,
            mix_window_start_seconds,
            mix_window_end_seconds,
        });
        if let Some(next_item_id) = next {
            self.emit_music_stage_transition_debug_stamp(
                control.item_id,
                next_item_id,
                current_segment.as_ref(),
                planned_segment_end,
                next_start_seconds,
                transition_seconds,
                fade_duration_seconds,
                crossfade_preview_started,
                &plan_reason,
            );
        }
        self.music.music_chorus_pending_mix_target = None;
        self.last_action = if crossfade_preview_started {
            format!(
                "Stage Mix: {:.1}s crossfade ({plan_reason}).",
                transition_seconds
            )
        } else {
            format!(
                "Stage Mix: {:.1}s quick handoff ({plan_reason}).",
                fade_duration_seconds
            )
        };
    }

    fn poll_music_chorus_fade_out(&mut self, control: &MusicPlaybackControl) -> bool {
        let Some(fade) = self.music.music_chorus_fade_out.clone() else {
            return false;
        };
        if fade.item_id != control.item_id || fade.session_id != control.session_id {
            self.music.music_chorus_fade_out = None;
            return false;
        }

        let elapsed_frames = control
            .output_frame_cursor()
            .saturating_sub(fade.started_output_frame);
        let ratio = elapsed_frames.get() as f64 / fade.duration_output_frames.get().max(1) as f64;
        let ratio = ratio.clamp(0.0, 1.0) as f32;

        let segment_end = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|segment| segment.item_id == control.item_id)
            .map(|segment| segment.end_seconds);
        let reached_segment_end = segment_end.is_some_and(|end| {
            control.playback_seconds() + MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS >= end
        });

        let transition_complete = if fade.crossfade_preview_started {
            control.crossfade_preview_transition_complete()
        } else {
            ratio >= 1.0 || (reached_segment_end && !fade.plain_crossfade_fallback)
        };
        if transition_complete {
            if should_zero_music_chorus_volume_before_advance(&fade) {
                control.set_volume(0.0);
            }
            self.advance_music_chorus_flow_from(control.item_id);
        }
        true
    }

    fn poll_music_chorus_fade_in(&mut self) {
        let Some(fade) = self.music.music_chorus_fade_in.clone() else {
            return;
        };
        let Some(control) = self.music.music_playback.clone() else {
            self.music.music_chorus_fade_in = None;
            return;
        };
        if fade.item_id != control.item_id || fade.session_id != control.session_id {
            self.music.music_chorus_fade_in = None;
            return;
        }

        let elapsed_frames = control
            .output_frame_cursor()
            .saturating_sub(fade.started_output_frame);
        let ratio = (elapsed_frames.get() as f64 / fade.duration_output_frames.get().max(1) as f64)
            .clamp(0.0, 1.0) as f32;
        if ratio >= 1.0 {
            control.set_volume(fade.target_volume);
            self.music.music_chorus_fade_in = None;
        }
    }

    fn sanitize_music_radio_cue_pending_for_control(&mut self, control: &MusicPlaybackControl) {
        let Some(target) = self.music.music_chorus_pending_mix_target.clone() else {
            return;
        };

        if !self.music_mix_next_pending() {
            self.cancel_music_radio_cue_pending_and_reanchor();
            return;
        }
        if target.current_item_id != control.item_id || target.session_id != control.session_id {
            self.cancel_music_radio_cue_pending_and_reanchor();
            return;
        }
        if control.is_paused() {
            self.cancel_music_radio_cue_pending_with_message(
                "Mix next cancelled while playback is paused.",
            );
            return;
        }
        if target.target_item_id == control.item_id {
            self.cancel_music_radio_cue_pending_and_reanchor();
            return;
        }
        if !self.music_item_can_play(target.target_item_id) {
            self.cancel_music_radio_cue_pending_with_message(
                "Mix next cancelled because the queued track is no longer playable.",
            );
        }
    }

    pub(super) fn request_music_mix_next_to_item(
        &mut self,
        item_id: QueueItemId,
        record_history: bool,
    ) -> bool {
        if !self.music.music_automix_enabled {
            return false;
        }
        if !self.music_item_can_play(item_id) {
            return false;
        }
        let Some(control) = self.music.music_playback.clone() else {
            return false;
        };
        if control.item_id == item_id {
            return false;
        }
        if control.is_paused() {
            return false;
        }

        if record_history {
            self.record_music_navigation_target(item_id);
        }
        self.request_music_scroll_to_item(item_id);
        self.cancel_music_radio_cue_pending();
        self.ensure_music_stage_pick_for_item(item_id, true);
        // Manual seeking temporarily suppresses automatic polling so the
        // selector cannot pull playback back to an old range. An explicit
        // Mix-next command is a newer user intent and must take ownership
        // immediately instead of waiting for that grace period to expire.
        self.music.music_manual_seek_grace_until = None;

        let target_start_seconds = self.music_automix_entry_start_seconds_for_item(item_id);
        let (transition_seconds, mix_kind) =
            self.music_chorus_stream_transition_seconds_and_kind_between(control.item_id, item_id);
        let confidence = self.music_chorus_pair_confidence(control.item_id, item_id);
        let tempo_split = self.music_chorus_tempo_split_between(control.item_id, item_id);
        let reason = self.music_chorus_transition_reason_with_rate(
            control.item_id,
            item_id,
            tempo_split.incoming_rate,
            tempo_split.outgoing_rate,
            true,
            Some(if mix_kind == MusicMixWindowKind::RewardLong {
                "Reward Long Mix · Mix next early cue"
            } else {
                "Mix next early cue"
            }),
        );

        self.music.music_chorus_pending_mix_target = Some(MusicChorusPendingMixTarget {
            current_item_id: control.item_id,
            session_id: control.session_id,
            target_item_id: item_id,
            target_start_seconds,
            transition_seconds,
            confidence,
            reason: reason.clone(),
            requested_at: Instant::now(),
            cue_armed: false,
            hold_end_seconds: None,
        });
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;
        if self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .is_none_or(|segment| {
                segment.item_id != control.item_id || segment.session_id != control.session_id
            })
        {
            self.music.music_chorus_flow_segment =
                self.music_radio_cue_prepare_segment_for_control(&control, transition_seconds);
        }
        self.start_music_prefetch_for_item(item_id);
        self.music.music_prefetch_for_current_item_id = Some(control.item_id);

        if music_stage_chain_direct_radio_cue_enabled() {
            let _ = self.arm_music_radio_cue_when_preview_ready(&control);
        } else {
            if let Some(segment) =
                self.music_radio_cue_prepare_segment_for_control(&control, transition_seconds)
            {
                self.prepare_music_chorus_preview_for_target(&control, &segment, item_id);
            }
            self.hold_music_radio_cue_until_preview_ready(&control, transition_seconds);
        }

        let title = self
            .queue_item_by_id(item_id)
            .map(|item| item.title.clone())
            .unwrap_or_else(|| "next track".to_owned());
        self.last_action = format!("Mix next preparing: {title}.");
        true
    }

    fn music_radio_cue_prepare_segment_for_control(
        &self,
        control: &MusicPlaybackControl,
        transition_seconds: f64,
    ) -> Option<MusicChorusFlowSegment> {
        if let Some(segment) = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|segment| {
                segment.item_id == control.item_id && segment.session_id == control.session_id
            })
            .cloned()
        {
            return Some(segment);
        }

        let playback_seconds = control.playback_seconds().max(0.0);
        let range_end = self
            .music_automix_range_for_item(control.item_id)
            .map(|(range_start, range_end)| {
                let (_range_start, range_end) =
                    self.music_lyrics_safe_range_for_item(control.item_id, range_start, range_end);
                range_end
            })
            .or_else(|| control.duration_seconds())
            .unwrap_or(playback_seconds + transition_seconds * 3.0);
        Some(MusicChorusFlowSegment {
            item_id: control.item_id,
            session_id: control.session_id,
            start_seconds: playback_seconds,
            end_seconds: range_end.max(playback_seconds + transition_seconds),
            transition_seconds,
            hold_end_seconds: None,
            fallback_stage: MusicChorusFallbackStage::Normal,
        })
    }

    fn hold_music_radio_cue_until_preview_ready(
        &mut self,
        control: &MusicPlaybackControl,
        transition_seconds: f64,
    ) {
        if music_stage_chain_direct_radio_cue_enabled() {
            return;
        }

        let Some((cue_armed, locked_hold_end)) = self
            .music_radio_cue_pending_for_control(control)
            .map(|target| (target.cue_armed, target.hold_end_seconds))
        else {
            return;
        };
        if cue_armed {
            return;
        }

        let playback_seconds = control.playback_seconds().max(0.0);
        let computed_hold_end = playback_seconds
            + transition_seconds.max(MUSIC_CHORUS_REALTIME_FALLBACK_FADE_SECONDS)
            + MUSIC_RADIO_CUE_WAIT_FOR_PREVIEW_HOLD_SECONDS;

        // v10.12.19: lock the Radio Cue waiting window the first time it is
        // extended.  HQ render can take a little longer than streaming, and
        // recomputing from the moving playhead made the visible mix window
        // retreat four or five times before the fallback fired.  The cue
        // marker must be a promise, not a moving target.
        let hold_end = if let Some(locked) = locked_hold_end {
            locked
        } else {
            let hold_end = computed_hold_end;
            if let Some(target) =
                self.music
                    .music_chorus_pending_mix_target
                    .as_mut()
                    .filter(|target| {
                        target.current_item_id == control.item_id
                            && target.session_id == control.session_id
                    })
            {
                target.hold_end_seconds = Some(hold_end);
            }
            hold_end
        };

        if let Some(segment) = self.music.music_chorus_flow_segment.as_mut() {
            if segment.item_id == control.item_id && segment.session_id == control.session_id {
                if segment.end_seconds < hold_end {
                    segment.end_seconds = hold_end;
                }
                segment.hold_end_seconds = Some(hold_end);
            }
        } else if let Some(segment) =
            self.music_radio_cue_prepare_segment_for_control(control, transition_seconds)
        {
            self.music.music_chorus_flow_segment = Some(MusicChorusFlowSegment {
                end_seconds: hold_end.max(segment.end_seconds),
                ..segment
            });
        }
    }

    fn arm_music_radio_cue_when_preview_ready(&mut self, control: &MusicPlaybackControl) -> bool {
        let Some(target) = self.music_radio_cue_pending_for_control(control).cloned() else {
            return false;
        };
        if target.cue_armed {
            return false;
        }

        if music_stage_chain_direct_radio_cue_enabled() {
            if self.hold_music_transition_while_target_cache_prepares(
                control,
                target.target_item_id,
                target.transition_seconds,
                "Mix next · B cache preparing · A continues",
            ) {
                return false;
            }
            return self.arm_music_radio_cue_direct_stream(control, &target);
        }

        let preview_ready = self
            .music
            .music_chorus_ready_preview
            .as_ref()
            .is_some_and(|preview| {
                preview.current_item_id == control.item_id
                    && preview.session_id == control.session_id
                    && preview.next_item_id == target.target_item_id
            });
        if !preview_ready {
            if target.requested_at.elapsed().as_secs_f64() > MUSIC_RADIO_CUE_PREPARE_TIMEOUT_SECONDS
            {
                // v10.12.17: never hard-open the queued track when preview rendering times out.
                // That old fallback could jump to B while the visible mix marker was still far
                // away, which felt like an instant cut before the mix zone.  Instead, arm a
                // short, visible streaming-safe cue window and let the normal fade-out path do
                // the handoff at that marker.
                let playback_seconds = control.playback_seconds().max(0.0);
                let transition_seconds = target.transition_seconds.clamp(
                    MUSIC_CHORUS_REALTIME_FALLBACK_FADE_SECONDS,
                    MUSIC_CHORUS_TRANSITION_MAX_SECONDS,
                );
                let fallback_lead = MUSIC_RADIO_CUE_READY_MIN_LEAD_SECONDS.max(1.2);
                let visible_end = self
                    .music
                    .music_chorus_flow_segment
                    .as_ref()
                    .filter(|segment| {
                        segment.item_id == control.item_id
                            && segment.session_id == control.session_id
                    })
                    .map(|segment| segment.end_seconds)
                    .or(target.hold_end_seconds);
                let minimum_safe_end = playback_seconds + transition_seconds + 0.35;
                let mut cue_end = visible_end
                    .filter(|end| *end > minimum_safe_end)
                    .unwrap_or(playback_seconds + transition_seconds + fallback_lead);

                if let Some(duration) = control
                    .duration_seconds()
                    .or_else(|| {
                        self.music_analysis_manifest_for_item(control.item_id)
                            .map(|manifest| manifest.duration_seconds)
                    })
                    .filter(|duration| duration.is_finite() && *duration > playback_seconds + 0.5)
                {
                    let hard_end = (duration - MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS)
                        .max(playback_seconds + transition_seconds + 0.35);
                    cue_end = cue_end.min(hard_end);
                }

                let start_seconds = self
                    .music
                    .music_chorus_flow_segment
                    .as_ref()
                    .filter(|segment| {
                        segment.item_id == control.item_id
                            && segment.session_id == control.session_id
                    })
                    .map(|segment| segment.start_seconds.min(playback_seconds))
                    .unwrap_or(playback_seconds);

                self.music.music_chorus_flow_segment = Some(MusicChorusFlowSegment {
                    item_id: control.item_id,
                    session_id: control.session_id,
                    start_seconds,
                    end_seconds: cue_end,
                    transition_seconds,
                    hold_end_seconds: Some(cue_end),
                    fallback_stage: MusicChorusFallbackStage::PlainCrossfade,
                });
                self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                    transition_seconds,
                    confidence: target.confidence,
                    reason: format!("Mix next fallback armed · {}", target.reason),
                });
                self.music.music_chorus_preview_job = None;
                self.music.music_chorus_ready_preview = None;
                if let Some(target) = self.music.music_chorus_pending_mix_target.as_mut() {
                    if target.current_item_id == control.item_id
                        && target.session_id == control.session_id
                    {
                        target.cue_armed = true;
                    }
                }
                let cue_delay = (cue_end - transition_seconds - playback_seconds).max(0.0);
                self.last_action = format!("Mix next armed · mix in {:.1}s.", cue_delay);
                return true;
            }
            if self.music.music_chorus_preview_job.is_none() {
                if let Some(segment) = self
                    .music_radio_cue_prepare_segment_for_control(control, target.transition_seconds)
                {
                    self.prepare_music_chorus_preview_for_target(
                        control,
                        &segment,
                        target.target_item_id,
                    );
                }
            }
            self.hold_music_radio_cue_until_preview_ready(control, target.transition_seconds);
            self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                transition_seconds: target.transition_seconds,
                confidence: target.confidence,
                reason: format!("Mix next preparing B · {}", target.reason),
            });
            return false;
        }

        let transition_seconds = self
            .music
            .music_chorus_ready_preview
            .as_ref()
            .map(|preview| preview.transition_seconds)
            .unwrap_or(target.transition_seconds)
            .clamp(
                MUSIC_CHORUS_TRANSITION_MIN_SECONDS,
                MUSIC_CHORUS_TRANSITION_MAX_SECONDS,
            );
        let playback_seconds = control.playback_seconds().max(0.0);
        let min_end =
            playback_seconds + transition_seconds + MUSIC_RADIO_CUE_READY_MIN_LEAD_SECONDS;
        let max_end =
            playback_seconds + transition_seconds + MUSIC_RADIO_CUE_READY_MAX_CUE_WINDOW_SECONDS;
        let range_end = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|segment| {
                segment.item_id == control.item_id && segment.session_id == control.session_id
            })
            .map(|segment| segment.end_seconds)
            .or_else(|| {
                self.music_automix_range_for_item(control.item_id).map(
                    |(range_start, range_end)| {
                        let (_range_start, range_end) = self.music_lyrics_safe_range_for_item(
                            control.item_id,
                            range_start,
                            range_end,
                        );
                        range_end
                    },
                )
            })
            .or_else(|| control.duration_seconds())
            .unwrap_or(max_end);
        let latest_end = range_end.max(min_end).min(max_end);
        let locked_ready_end = target
            .hold_end_seconds
            .filter(|end| *end > playback_seconds + transition_seconds + 0.25);
        let mut cue_end = if let Some(locked_end) = locked_ready_end {
            // v10.12.19: if Radio Cue had already promised a visible waiting
            // window, do not re-solve a new cue from the current playhead when
            // HQ preview finally becomes ready.  Re-solving here made the
            // marker hop backward right before the mix began.
            locked_end.clamp(min_end, latest_end.max(min_end))
        } else if latest_end > min_end {
            let desired_end =
                (playback_seconds + transition_seconds + 1.35).clamp(min_end, latest_end);
            self.music_lyrics_safe_mix_out_for_item(
                control.item_id,
                desired_end,
                min_end,
                latest_end,
            )
        } else {
            latest_end
        };
        cue_end = cue_end.clamp(min_end, latest_end.max(min_end));

        let segment = MusicChorusFlowSegment {
            item_id: control.item_id,
            session_id: control.session_id,
            start_seconds: self
                .music
                .music_chorus_flow_segment
                .as_ref()
                .map(|segment| segment.start_seconds.min(playback_seconds))
                .unwrap_or(playback_seconds),
            end_seconds: cue_end,
            transition_seconds,
            hold_end_seconds: target.hold_end_seconds.or(Some(cue_end)),
            fallback_stage: MusicChorusFallbackStage::Normal,
        };
        self.music.music_chorus_flow_segment = Some(segment);
        self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
            transition_seconds,
            confidence: target.confidence,
            reason: format!("Mix next armed · {}", target.reason),
        });
        if let Some(target) = self.music.music_chorus_pending_mix_target.as_mut() {
            if target.current_item_id == control.item_id && target.session_id == control.session_id
            {
                target.cue_armed = true;
            }
        }
        let cue_delay = (cue_end - transition_seconds - playback_seconds).max(0.0);
        self.last_action = format!("Mix next armed · mix in {:.1}s.", cue_delay);
        true
    }

    fn arm_music_radio_cue_direct_stream(
        &mut self,
        control: &MusicPlaybackControl,
        target: &MusicChorusPendingMixTarget,
    ) -> bool {
        if target.current_item_id != control.item_id || target.session_id != control.session_id {
            return false;
        }

        let playback_seconds = control.playback_seconds().max(0.0);
        let transition_seconds = target.transition_seconds.clamp(
            MUSIC_STAGE_LITE_TRANSITION_MIN_SECONDS,
            MUSIC_STAGE_LITE_TRANSITION_MAX_SECONDS,
        );
        let min_end =
            playback_seconds + transition_seconds + MUSIC_RADIO_CUE_READY_MIN_LEAD_SECONDS;
        let max_end =
            playback_seconds + transition_seconds + MUSIC_RADIO_CUE_READY_MAX_CUE_WINDOW_SECONDS;
        let range_end = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|segment| {
                segment.item_id == control.item_id && segment.session_id == control.session_id
            })
            .map(|segment| segment.end_seconds)
            .or_else(|| {
                self.music_automix_range_for_item(control.item_id).map(
                    |(range_start, range_end)| {
                        let (_range_start, range_end) = self.music_lyrics_safe_range_for_item(
                            control.item_id,
                            range_start,
                            range_end,
                        );
                        range_end
                    },
                )
            })
            .or_else(|| control.duration_seconds())
            .unwrap_or(max_end);
        let latest_end = range_end.max(min_end).min(max_end);
        let desired_end =
            (playback_seconds + transition_seconds + 1.20).clamp(min_end, latest_end.max(min_end));
        let cue_end = if latest_end > min_end {
            self.music_lyrics_safe_mix_out_for_item(
                control.item_id,
                desired_end,
                min_end,
                latest_end,
            )
        } else {
            latest_end
        }
        .clamp(min_end, latest_end.max(min_end));

        let next_duration_seconds =
            self.music_chorus_duration_seconds_for_item(target.target_item_id);
        let safe_target_start_seconds = target.target_start_seconds.map(|start_seconds| {
            self.music_stage_chain_direct_entry_anchor_start_for_item(
                target.target_item_id,
                start_seconds,
                transition_seconds,
                next_duration_seconds,
            )
        });
        let start_seconds = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|segment| {
                segment.item_id == control.item_id && segment.session_id == control.session_id
            })
            .map(|segment| segment.start_seconds.min(playback_seconds))
            .unwrap_or(playback_seconds);

        self.music.music_chorus_flow_segment = Some(MusicChorusFlowSegment {
            item_id: control.item_id,
            session_id: control.session_id,
            start_seconds,
            end_seconds: cue_end,
            transition_seconds,
            hold_end_seconds: Some(cue_end),
            fallback_stage: MusicChorusFallbackStage::PlainCrossfade,
        });
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;
        self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
            transition_seconds,
            confidence: target.confidence,
            reason: format!("Mix next · Stage Chain Direct Stream · {}", target.reason),
        });
        if let Some(pending) = self.music.music_chorus_pending_mix_target.as_mut() {
            if pending.current_item_id == control.item_id
                && pending.session_id == control.session_id
            {
                pending.cue_armed = true;
                pending.hold_end_seconds = Some(cue_end);
                pending.transition_seconds = transition_seconds;
                pending.target_start_seconds = safe_target_start_seconds;
            }
        }
        let cue_delay = (cue_end - transition_seconds - playback_seconds).max(0.0);
        eprintln!(
            "[music-stage-chain] radio cue direct item={}->{} fade={:.3}s cue_end={:.3}s B_start={} preview=disabled",
            control.item_id,
            target.target_item_id,
            transition_seconds,
            cue_end,
            safe_target_start_seconds
                .map(|seconds| format!("{seconds:.3}s"))
                .unwrap_or_else(|| "auto".to_owned())
        );
        self.last_action = format!("Mix next ready: mix in {:.1}s.", cue_delay);
        true
    }

    fn music_radio_cue_pending_for_control(
        &self,
        control: &MusicPlaybackControl,
    ) -> Option<&MusicChorusPendingMixTarget> {
        self.music
            .music_chorus_pending_mix_target
            .as_ref()
            .filter(|target| {
                target.current_item_id == control.item_id && target.session_id == control.session_id
            })
    }

    pub(super) fn prepare_music_mode_start_for_item(&mut self, item_id: QueueItemId) {
        if !self.music_auto_transition_enabled() {
            return;
        }
        if self
            .music
            .music_chorus_pending_start
            .as_ref()
            .is_some_and(|pending| pending.item_id == item_id)
        {
            return;
        }
        self.ensure_music_stage_pick_for_item(item_id, false);
        let Some((segment_start, segment_end)) = self.music_automix_range_for_item(item_id) else {
            return;
        };
        let transition_seconds = self.music_chorus_transition_seconds_for_item(item_id);
        let entry_start = self.music_automix_entry_start_for_item(
            item_id,
            segment_start,
            segment_end,
            transition_seconds,
        );
        if entry_start > 0.25 {
            self.prepare_music_chorus_start_for_item(item_id, entry_start);
        }
    }

    pub(super) fn prepare_music_chorus_fade_in_for_item(&mut self, item_id: QueueItemId) {
        self.prepare_music_chorus_fade_in_for_item_with_duration(
            item_id,
            self.music_chorus_transition_seconds_for_item(item_id),
        );
    }

    fn prepare_music_chorus_fade_in_for_item_with_duration(
        &mut self,
        item_id: QueueItemId,
        duration_seconds: f64,
    ) {
        self.prepare_music_chorus_fade_in_for_item_with_duration_and_incoming_tempo(
            item_id,
            duration_seconds,
            1.0,
        );
    }

    fn prepare_music_chorus_fade_in_for_item_with_duration_and_incoming_tempo(
        &mut self,
        item_id: QueueItemId,
        duration_seconds: f64,
        incoming_tempo_rate: f64,
    ) {
        let duration_seconds = duration_seconds.max(0.1);
        let incoming_tempo_rate = if MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_INCOMING
            && self.music_stage_direct_tempo_bridge_strength_value() > 0.005
            && incoming_tempo_rate.is_finite()
            && (incoming_tempo_rate - 1.0).abs()
                >= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_INCOMING_MIN_RATE_DELTA
        {
            self.music_stage_direct_tempo_bridge_clamp_incoming_rate(incoming_tempo_rate)
        } else {
            1.0
        };
        self.music.music_chorus_pending_fade_in = Some(MusicChorusPendingFadeIn {
            item_id,
            duration_seconds,
            target_volume: self.music.music_volume,
            incoming_tempo_rate,
        });
    }

    pub(super) fn prepare_music_chorus_start_for_item(
        &mut self,
        item_id: QueueItemId,
        start_seconds: f64,
    ) {
        self.music.music_chorus_pending_start = Some(MusicChorusPendingStart {
            item_id,
            start_seconds,
        });
    }

    pub(super) fn start_music_chorus_fade_in_if_pending(
        &mut self,
        item_id: QueueItemId,
        session_id: u64,
    ) {
        let pending_start = self
            .music
            .music_chorus_pending_start
            .as_ref()
            .filter(|pending| pending.item_id == item_id)
            .cloned();
        let pending_fade = match self.music.music_chorus_pending_fade_in.take() {
            Some(pending) if pending.item_id == item_id => Some(pending),
            Some(pending) => {
                self.music.music_chorus_pending_fade_in = Some(pending);
                None
            }
            None => None,
        };

        if pending_start.is_none() && pending_fade.is_none() {
            return;
        }

        if let Some(control) = self.music.music_playback.clone() {
            if control.item_id == item_id && control.session_id == session_id {
                let mut reanchor_start_seconds = None;
                if let Some(pending) = pending_start.as_ref() {
                    let duration = control.duration_seconds().or_else(|| {
                        self.music_analysis_manifest_for_item(item_id)
                            .map(|manifest| manifest.duration_seconds)
                    });
                    if let Some(duration) =
                        duration.filter(|duration| duration.is_finite() && *duration > 0.0)
                    {
                        let seek_ratio = (pending.start_seconds / duration).clamp(0.0, 1.0) as f32;
                        control.seek_to_ratio(seek_ratio);
                        self.music.music_seek_snap_ratio = Some(seek_ratio);
                        self.music.music_seek_snap_deadline =
                            Some(Instant::now() + Duration::from_millis(700));
                        reanchor_start_seconds = Some(pending.start_seconds);
                    }
                    self.music.music_chorus_pending_start = None;
                }

                if let Some(start_seconds) = reanchor_start_seconds {
                    self.reanchor_music_chorus_flow_after_manual_seek(&control, start_seconds);
                }

                if let Some(pending) = pending_fade.as_ref() {
                    control.set_volume(0.0);
                    let duration_output_frames =
                        control.mix_frame_count_from_seconds(pending.duration_seconds);
                    let started_output_frame = control.fade_volume_to_with_curve(
                        pending.target_volume,
                        Duration::from_secs_f64(pending.duration_seconds),
                        self.music_stage_direct_mix_curve_value(),
                    );
                    if (pending.incoming_tempo_rate - 1.0).abs()
                        >= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_INCOMING_MIN_RATE_DELTA
                    {
                        control.start_outgoing_tempo_transition_from_output_frame(
                            pending.incoming_tempo_rate,
                            duration_output_frames,
                            started_output_frame,
                        );
                        eprintln!(
                            "[music-stage-chain] direct mutual tempo bridge incoming item={} rate={:.4} duration={:.3}s strength={:.0}% mix_len={:.0}% curve={:.0}%",
                            item_id,
                            pending.incoming_tempo_rate,
                            pending.duration_seconds,
                            self.music_stage_direct_tempo_bridge_strength_value() * 100.0,
                            self.music_stage_direct_mix_length_value() * 100.0,
                            self.music_stage_direct_mix_curve_value() * 100.0
                        );
                    }
                    self.music.music_chorus_fade_in = Some(MusicChorusFadeIn {
                        item_id,
                        session_id,
                        started_output_frame,
                        duration_output_frames,
                        duration_seconds: pending.duration_seconds,
                        target_volume: pending.target_volume,
                    });
                    self.last_action = "Chorus Flow transition: cue start and fade in.".to_owned();
                }
            }
        }
    }

    pub(super) fn music_chorus_initial_volume_for_item(&self, item_id: QueueItemId) -> f32 {
        if self
            .music
            .music_chorus_pending_fade_in
            .as_ref()
            .is_some_and(|pending| pending.item_id == item_id)
        {
            0.0
        } else {
            self.music.music_volume
        }
    }

    pub(super) fn clear_music_chorus_transition(&mut self) {
        self.music.music_chorus_fade_out = None;
        self.music.music_chorus_fade_in = None;
        self.music.music_chorus_pending_fade_in = None;
        self.music.music_chorus_pending_start = None;
        self.music.music_chorus_mix_plan = None;
        self.music.music_chorus_pending_mix_target = None;
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;
        self.music.music_stage_presence_recent_seconds = None;
        self.music.music_stage_presence_last_seconds = None;
        self.music.music_stage_presence_short_run = 0;
        if let Some(bridge) = self.music.music_chorus_handoff_bridge.take() {
            bridge.control.stop();
        }
    }

    fn record_music_stage_presence_for_finished_item(
        &mut self,
        finished_item_id: QueueItemId,
        fade: Option<&MusicChorusFadeOut>,
    ) {
        self.ensure_music_stage_cue_memory_loaded();
        let Some(fade) = fade.filter(|fade| fade.item_id == finished_item_id) else {
            return;
        };
        let segment_start = self
            .music
            .music_chorus_flow_segment
            .as_ref()
            .filter(|segment| {
                segment.item_id == finished_item_id && segment.session_id == fade.session_id
            })
            .map(|segment| segment.start_seconds)
            .unwrap_or(fade.start_playback_seconds);
        let Some(presence_seconds) = music_segment_selector::presence_seconds_for_fade(
            segment_start,
            fade.start_playback_seconds,
            fade.duration_seconds,
        ) else {
            return;
        };
        let Some(history) = music_segment_selector::presence_history_after_finished_segment(
            music_segment_selector::MusicStagePresenceHistory {
                recent_seconds: self.music.music_stage_presence_recent_seconds,
                last_seconds: self.music.music_stage_presence_last_seconds,
                short_run: self.music.music_stage_presence_short_run,
            },
            presence_seconds,
        ) else {
            return;
        };
        self.music.music_stage_presence_recent_seconds = history.recent_seconds;
        self.music.music_stage_presence_last_seconds = history.last_seconds;
        self.music.music_stage_presence_short_run = history.short_run;

        if let Some(segment) = self
            .music
            .music_chorus_flow_segment
            .clone()
            .filter(|segment| {
                segment.item_id == finished_item_id && segment.session_id == fade.session_id
            })
        {
            self.record_music_stage_cue_memory_for_segment(
                finished_item_id,
                &segment,
                presence_seconds,
            );
        }
    }

    fn music_stage_cue_memory_path(&self) -> PathBuf {
        self.app_cache_root_path()
            .join("state")
            .join("music-cue-memory.yaml")
    }

    fn ensure_music_stage_cue_memory_loaded(&mut self) {
        if self.music.music_stage_cue_memory_loaded {
            return;
        }
        let path = self.music_stage_cue_memory_path();
        self.music.music_stage_cue_memory =
            read_yaml_file::<MusicStageCueMemoryStore>(&path).unwrap_or_default();
        self.music.music_stage_cue_memory.version = 1;
        self.music.music_stage_cue_memory_loaded = true;
    }

    fn save_music_stage_cue_memory(&self) {
        let path = self.music_stage_cue_memory_path();
        if let Err(error) = write_yaml_file(&path, &self.music.music_stage_cue_memory) {
            eprintln!("[music-stage] cue memory save skipped: {error}");
        }
    }

    fn music_stage_cue_memory_key_for_item(
        &self,
        item_id: QueueItemId,
        _manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    ) -> Option<String> {
        let item = self.queue_item_by_id(item_id)?;
        let track_key = if !item.music_cache_key.trim().is_empty() {
            item.music_cache_key.trim()
        } else if !item.source_url.trim().is_empty() {
            item.source_url.trim()
        } else {
            item.title.trim()
        };
        if track_key.is_empty() {
            return None;
        }
        let candidate_index = self
            .music
            .music_stage_pick_selected
            .get(&item_id)
            .map(|pick| pick.candidate_index)
            .unwrap_or(0);
        Some(format!("{track_key}#highlight:{candidate_index}"))
    }

    fn record_music_stage_cue_memory_for_segment(
        &mut self,
        item_id: QueueItemId,
        segment: &MusicChorusFlowSegment,
        effective_presence_seconds: f64,
    ) {
        if !effective_presence_seconds.is_finite()
            || effective_presence_seconds < MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS
        {
            return;
        }

        let Some(manifest) = self.music_analysis_manifest_for_item(item_id) else {
            return;
        };
        let Some(candidate) = self.selected_music_stage_highlight_candidate(item_id, &manifest)
        else {
            return;
        };
        let base_start = candidate
            .start_seconds
            .clamp(0.0, manifest.duration_seconds.max(0.0));
        let base_end = candidate
            .end_seconds
            .clamp(base_start, manifest.duration_seconds.max(base_start));
        let Some(key) = self.music_stage_cue_memory_key_for_item(item_id, &manifest) else {
            return;
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let Some(observation) = music_segment_selector::cue_memory_observation_for_segment(
            segment.start_seconds,
            segment.end_seconds,
            base_start,
            base_end,
            effective_presence_seconds,
            now,
        ) else {
            return;
        };

        let entry = self
            .music
            .music_stage_cue_memory
            .entries
            .entry(key)
            .or_default();
        let updated = music_segment_selector::cue_memory_updated_values(
            music_segment_selector::MusicStageCueMemoryValues {
                start_offset_seconds: entry.start_offset_seconds,
                end_offset_seconds: entry.end_offset_seconds,
                effective_presence_seconds: entry.effective_presence_seconds,
                confidence: entry.confidence,
                updates: entry.updates,
                updated_unix_seconds: entry.updated_unix_seconds,
            },
            observation,
        );
        entry.start_offset_seconds = updated.start_offset_seconds;
        entry.end_offset_seconds = updated.end_offset_seconds;
        entry.effective_presence_seconds = updated.effective_presence_seconds;
        entry.confidence = updated.confidence;
        entry.updates = updated.updates;
        entry.updated_unix_seconds = updated.updated_unix_seconds;

        self.prune_music_stage_cue_memory();
        self.save_music_stage_cue_memory();
    }

    fn prune_music_stage_cue_memory(&mut self) {
        if self.music.music_stage_cue_memory.entries.len() <= MUSIC_STAGE_CUE_MEMORY_MAX_ENTRIES {
            return;
        }
        let mut keys_by_age: Vec<(String, u64)> = self
            .music
            .music_stage_cue_memory
            .entries
            .iter()
            .map(|(key, entry)| (key.clone(), entry.updated_unix_seconds))
            .collect();
        keys_by_age.sort_by_key(|(_, updated)| *updated);
        let remove_count = keys_by_age
            .len()
            .saturating_sub(MUSIC_STAGE_CUE_MEMORY_MAX_ENTRIES);
        for (key, _) in keys_by_age.into_iter().take(remove_count) {
            self.music.music_stage_cue_memory.entries.remove(&key);
        }
    }

    fn advance_music_chorus_flow_from(&mut self, finished_item_id: QueueItemId) {
        let fade = self.music.music_chorus_fade_out.clone();
        self.record_music_stage_presence_for_finished_item(finished_item_id, fade.as_ref());
        let next = fade
            .as_ref()
            .and_then(|fade| fade.next_item_id)
            .or_else(|| self.next_music_chorus_flow_item_id(finished_item_id));
        // The leaving track's Stage Pick has already served this transition.
        // Clear it for normal track changes, but keep it for RepeatOne/same-song
        // handoff so A->same A does not feel like a new random cut every loop.
        if next != Some(finished_item_id) {
            self.music
                .music_stage_pick_selected
                .remove(&finished_item_id);
        }
        self.music.music_chorus_flow_segment = None;
        self.music.music_chorus_fade_out = None;
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;

        let Some(item_id) = next else {
            self.finish_music_chorus_flow_without_next_item(finished_item_id);
            return;
        };

        let used_crossfade_preview = fade
            .as_ref()
            .is_some_and(|fade| fade.crossfade_preview_started);
        if used_crossfade_preview {
            if let Some(control) = self.music.music_playback.clone() {
                let session_id = self.next_music_playback_session_id();
                if let Some(promoted_seconds) =
                    control.promote_crossfade_preview_to_main(item_id, session_id)
                {
                    let promoted_control = control.with_identity(item_id, session_id);
                    self.mark_music_playback_state(finished_item_id, CompactMusicState::Ready);
                    self.mark_music_playback_state(item_id, CompactMusicState::Playing);
                    self.music.music_playback = Some(promoted_control.clone());
                    self.music.music_player_current_item_id = Some(item_id);
                    self.music.music_player_error = None;
                    self.music.music_seek_snap_ratio = None;
                    self.music.music_seek_snap_deadline = None;
                    if let Some((highlight_start, highlight_end)) =
                        self.music_automix_range_for_item(item_id)
                    {
                        let (highlight_start, highlight_end) = self
                            .music_lyrics_safe_range_for_item(
                                item_id,
                                highlight_start,
                                highlight_end,
                            );
                        let start_seconds = promoted_seconds.max(highlight_start);
                        let mut transition_seconds =
                            self.music_chorus_transition_seconds_for_item(item_id).min(
                                ((highlight_end - start_seconds) * 0.45)
                                    .max(MUSIC_CHORUS_TRANSITION_MIN_SECONDS),
                            );
                        let mut plan_confidence =
                            self.music_chorus_tempo_confidence_for_item(item_id);
                        let mut plan_reason = "current track beat window".to_owned();
                        let mut planned_segment_end = highlight_end;
                        if let Some(next_item_id) = self.next_music_chorus_flow_item_id(item_id) {
                            if let Some((next_start, next_end)) =
                                self.music_automix_range_for_item(next_item_id)
                            {
                                let (next_start, next_end) = self.music_lyrics_safe_range_for_item(
                                    next_item_id,
                                    next_start,
                                    next_end,
                                );
                                let (base_transition, mix_kind) = self
                                    .music_chorus_stream_transition_seconds_and_kind_between(
                                        item_id,
                                        next_item_id,
                                    );
                                let reward_tail_room = if mix_kind == MusicMixWindowKind::RewardLong
                                {
                                    self.music_chorus_reward_tail_extension_seconds_between(
                                        item_id,
                                        next_item_id,
                                        highlight_end,
                                        base_transition,
                                    )
                                } else {
                                    0.0
                                };
                                let reward_transition_seed =
                                    if mix_kind == MusicMixWindowKind::RewardLong {
                                        self.music_chorus_reward_transition_seed_seconds_between(
                                            item_id,
                                            next_item_id,
                                            base_transition,
                                            reward_tail_room,
                                        )
                                    } else {
                                        base_transition
                                    };
                                transition_seconds = clamp_music_chorus_transition_seconds(
                                    reward_transition_seed,
                                    highlight_end - start_seconds + reward_tail_room,
                                    next_end - next_start,
                                );
                                planned_segment_end = if mix_kind == MusicMixWindowKind::RewardLong
                                {
                                    self.music_chorus_reward_extended_end_seconds(
                                        item_id,
                                        next_item_id,
                                        highlight_end,
                                        transition_seconds,
                                    )
                                } else {
                                    highlight_end
                                };
                                let tempo_split =
                                    self.music_chorus_tempo_split_between(item_id, next_item_id);
                                plan_confidence =
                                    self.music_chorus_pair_confidence(item_id, next_item_id);
                                plan_reason = self.music_chorus_transition_reason_with_rate(
                                    item_id,
                                    next_item_id,
                                    tempo_split.incoming_rate,
                                    tempo_split.outgoing_rate,
                                    true,
                                    Some(if mix_kind == MusicMixWindowKind::RewardLong {
                                        mix_kind.detail_label()
                                    } else {
                                        "Stream Mix"
                                    }),
                                );
                            }
                        }
                        let original_planned_segment_end = planned_segment_end;
                        let min_post_promote_segment_end = promoted_seconds
                            + MUSIC_STAGE_POST_PROMOTE_MIN_DWELL_SECONDS
                            + transition_seconds.max(MUSIC_CHORUS_REALTIME_FALLBACK_FADE_SECONDS);
                        let duration_limited_segment_end = promoted_control
                            .duration_seconds()
                            .filter(|duration| duration.is_finite() && *duration > promoted_seconds)
                            .map(|duration| {
                                min_post_promote_segment_end.min(
                                    (duration - MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS)
                                        .max(promoted_seconds),
                                )
                            })
                            .unwrap_or(min_post_promote_segment_end);
                        planned_segment_end = planned_segment_end
                            .max(duration_limited_segment_end)
                            .max(start_seconds + transition_seconds);
                        if planned_segment_end > original_planned_segment_end + 0.05 {
                            let guarded_mix_start =
                                planned_segment_end - transition_seconds.max(0.0);
                            eprintln!(
                                "[music-stage-dwell] post-promote guard item={} session={} playback={:.3}s mix_start={:.3}s end={:.3}s dwell={:.3}s",
                                item_id,
                                session_id,
                                promoted_seconds,
                                guarded_mix_start,
                                planned_segment_end,
                                (guarded_mix_start - promoted_seconds).max(0.0),
                            );
                        }
                        self.music.music_chorus_flow_segment = Some(MusicChorusFlowSegment {
                            item_id,
                            session_id,
                            start_seconds,
                            end_seconds: planned_segment_end,
                            transition_seconds,
                            hold_end_seconds: None,
                            fallback_stage: MusicChorusFallbackStage::Normal,
                        });
                        self.music.music_chorus_mix_plan = Some(MusicChorusMixPlan {
                            transition_seconds,
                            confidence: plan_confidence,
                            reason: plan_reason,
                        });
                    }
                    self.music.music_seek_snap_ratio = None;
                    self.music.music_seek_snap_deadline = None;
                    self.last_action =
                        "Chorus Flow transition: streaming deck promoted.".to_owned();
                    return;
                }
            }
        }

        let next_start_seconds = fade
            .as_ref()
            .and_then(|fade| fade.next_start_seconds)
            .or_else(|| self.music_automix_entry_start_seconds_for_item(item_id));
        if let Some(start_seconds) = next_start_seconds {
            self.prepare_music_chorus_start_for_item(item_id, start_seconds);
        }
        let fade_in_seconds = fade
            .as_ref()
            .filter(|fade| !fade.crossfade_preview_started)
            .map(|fade| fade.duration_seconds)
            .unwrap_or_else(|| self.music_chorus_transition_seconds_for_item(item_id));
        self.prepare_music_chorus_fade_in_for_item_with_duration(item_id, fade_in_seconds);
        self.start_music_stream_playback(item_id);
    }

    fn finish_music_chorus_flow_without_next_item(&mut self, finished_item_id: QueueItemId) {
        // A selected local range can finish before the decoder reaches the
        // physical end of the file. Treat that boundary as a real playback
        // completion when the playback order has no next item. Leaving the
        // control alive creates a silent/stale session that a later Mix-mode
        // toggle can reanchor and expose as an apparent restart.
        if self
            .music
            .music_playback
            .as_ref()
            .is_some_and(|control| control.item_id == finished_item_id)
        {
            if let Some(control) = self.music.music_playback.take() {
                control.stop();
            }
        }
        self.mark_music_playback_state(finished_item_id, CompactMusicState::Ready);
        self.music.music_chorus_flow_segment = None;
        self.clear_music_chorus_transition();
        self.music.music_player_error = None;

        // The user-selected Mix mode is persistent policy. Finishing the last
        // local range must not silently rewrite Highlight into Full song.
        self.last_action = "Chorus Flow finished.".to_owned();
    }

    pub(super) fn finish_music_chorus_transition_on_stream_finished(
        &mut self,
        item_id: QueueItemId,
        session_id: u64,
    ) -> bool {
        let stage_mix_fade_active = self
            .music
            .music_chorus_fade_out
            .as_ref()
            .is_some_and(|fade| fade.item_id == item_id && fade.session_id == session_id);
        if !stage_mix_fade_active {
            return false;
        }

        // Playback EOF and Stage Mix completion are separate signals.  Near the
        // mix window, the normal Finished event must not run the generic queue
        // advance path; Stage Mix owns the A -> [mix] -> B promotion/handoff.
        self.last_action = "Stage Mix: source ended during mix; completing transition.".to_owned();
        self.advance_music_chorus_flow_from(item_id);
        true
    }

    pub(super) fn release_music_chorus_handoff_bridge_if_ready(
        &mut self,
        item_id: QueueItemId,
        _session_id: u64,
    ) {
        if let Some(bridge) = self.music.music_chorus_handoff_bridge.as_mut() {
            if bridge.target_item_id == item_id && bridge.stop_output_frame.is_none() {
                bridge.control.fade_volume_to(
                    0.0,
                    Duration::from_secs_f64(MUSIC_PLAYBACK_READY_HANDOFF_FADE_SECONDS),
                );
                bridge.stop_output_frame =
                    Some(bridge.control.output_frame_cursor().saturating_add(
                        bridge.control.mix_frame_count_from_seconds(
                            MUSIC_PLAYBACK_READY_HANDOFF_FADE_SECONDS,
                        ),
                    ));
            }
        }
    }

    pub(super) fn release_music_playback_ready_handoff_if_ready(&mut self, item_id: QueueItemId) {
        if let Some(handoff) = self.music.music_playback_ready_handoff.as_mut() {
            if handoff.target_item_id == item_id && handoff.stop_output_frame.is_none() {
                handoff.control.fade_volume_to(
                    0.0,
                    Duration::from_secs_f64(MUSIC_PLAYBACK_READY_HANDOFF_FADE_SECONDS),
                );
                handoff.stop_output_frame =
                    Some(handoff.control.output_frame_cursor().saturating_add(
                        handoff.control.mix_frame_count_from_seconds(
                            MUSIC_PLAYBACK_READY_HANDOFF_FADE_SECONDS,
                        ),
                    ));
            }
        }
    }

    fn poll_music_playback_ready_handoff(&mut self) {
        let should_stop = self
            .music
            .music_playback_ready_handoff
            .as_ref()
            .is_some_and(|handoff| {
                let source_reached_eof = handoff
                    .control
                    .duration_seconds()
                    .is_some_and(|duration| handoff.control.playback_seconds() + 0.05 >= duration);
                source_reached_eof
                    || handoff.stop_output_frame.is_some_and(|stop_frame| {
                        handoff.control.output_frame_cursor() >= stop_frame
                    })
            });
        if !should_stop {
            return;
        }
        if let Some(handoff) = self.music.music_playback_ready_handoff.take() {
            handoff.control.stop();
            if self.music.music_player_current_item_id != Some(handoff.control.item_id)
                && self
                    .queue_item_by_id(handoff.control.item_id)
                    .is_some_and(|item| item.compact_music_state != Some(CompactMusicState::Failed))
            {
                self.mark_music_playback_state(handoff.control.item_id, CompactMusicState::Ready);
            }
        }
    }

    fn poll_music_chorus_handoff_bridge(&mut self) {
        let should_stop = self
            .music
            .music_chorus_handoff_bridge
            .as_ref()
            .is_some_and(|bridge| {
                bridge
                    .stop_output_frame
                    .is_some_and(|stop_frame| bridge.control.output_frame_cursor() >= stop_frame)
            });
        if !should_stop {
            return;
        }
        if let Some(bridge) = self.music.music_chorus_handoff_bridge.take() {
            bridge.control.stop();
            if self.music.music_player_current_item_id != Some(bridge.control.item_id)
                && self
                    .queue_item_by_id(bridge.control.item_id)
                    .is_some_and(|item| item.compact_music_state != Some(CompactMusicState::Failed))
            {
                self.mark_music_playback_state(bridge.control.item_id, CompactMusicState::Ready);
            }
        }
    }

    fn pending_or_peek_next_music_item_id(
        &mut self,
        control: &MusicPlaybackControl,
    ) -> Option<QueueItemId> {
        self.music
            .music_chorus_pending_mix_target
            .as_ref()
            .filter(|target| {
                target.current_item_id == control.item_id && target.session_id == control.session_id
            })
            .map(|target| target.target_item_id)
            .or_else(|| self.peek_next_music_chorus_flow_item_id(control.item_id))
    }

    fn peek_next_music_chorus_flow_item_id(
        &mut self,
        current_item_id: QueueItemId,
    ) -> Option<QueueItemId> {
        match self.music.music_playback_mode {
            MusicPlaybackMode::RepeatOne => Some(current_item_id),
            MusicPlaybackMode::Sequential => self.ordered_next_music_item_id(false),
            MusicPlaybackMode::RepeatAll | MusicPlaybackMode::Shuffle => {
                self.peek_next_music_item_id_for_prefetch(true)
            }
        }
    }

    fn next_music_chorus_flow_item_id(
        &mut self,
        current_item_id: QueueItemId,
    ) -> Option<QueueItemId> {
        match self.music.music_playback_mode {
            MusicPlaybackMode::RepeatOne => Some(current_item_id),
            MusicPlaybackMode::Sequential => self.next_music_item_id(false),
            MusicPlaybackMode::RepeatAll | MusicPlaybackMode::Shuffle => {
                self.next_music_item_id(true)
            }
        }
    }

    fn ensure_music_stage_pick_for_item(&mut self, item_id: QueueItemId, replace: bool) {
        if !self.music.music_chorus_flow_enabled {
            return;
        }
        if !replace && self.music.music_stage_pick_selected.contains_key(&item_id) {
            return;
        }
        let Some(manifest) = self.music_analysis_manifest_for_item(item_id) else {
            return;
        };
        if manifest.sections.highlight_candidates.len() <= 1 {
            self.music.music_stage_pick_selected.remove(&item_id);
            return;
        }

        self.music.music_stage_pick_serial =
            self.music.music_stage_pick_serial.wrapping_add(1).max(1);
        let seed = music_stage_pick_seed(
            item_id,
            self.music.music_stage_pick_serial,
            self.music.music_playback_session_id,
        );
        let Some(pick) = music_segment_selector::select_highlight_pick(&manifest, seed) else {
            self.music.music_stage_pick_selected.remove(&item_id);
            return;
        };
        let pick_score = manifest
            .sections
            .highlight_candidates
            .get(pick.candidate_index)
            .map(music_stage_pick_candidate_score)
            .unwrap_or(0.0);
        eprintln!(
            "[music-stage-pick] item={} candidate={} range={:.3}-{:.3}s score={:.2} replace={} candidates={}",
            item_id,
            music_stage_highlight_debug_label(pick.candidate_index),
            pick.start_seconds,
            pick.end_seconds,
            pick_score,
            replace,
            manifest.sections.highlight_candidates.len(),
        );
        self.music.music_stage_pick_selected.insert(item_id, pick);
    }

    fn music_stage_highlight_debug_label_for_item(
        &self,
        item_id: QueueItemId,
    ) -> Option<MusicStageHighlightDebugLabel> {
        let manifest = self.music_analysis_manifest_for_item(item_id)?;
        let candidate = self.selected_music_stage_highlight_candidate(item_id, &manifest)?;
        let candidate_index = manifest
            .sections
            .highlight_candidates
            .iter()
            .position(|entry| {
                (entry.start_seconds - candidate.start_seconds).abs() <= 1.5
                    && (entry.end_seconds - candidate.end_seconds).abs() <= 1.5
            })
            .unwrap_or_else(|| {
                manifest
                    .sections
                    .highlight_candidates
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| {
                        music_stage_pick_candidate_score(a)
                            .partial_cmp(&music_stage_pick_candidate_score(b))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(index, _)| index)
                    .unwrap_or(0)
            });

        Some(MusicStageHighlightDebugLabel {
            label: music_stage_highlight_debug_label(candidate_index),
            start_seconds: candidate.start_seconds,
            end_seconds: candidate.end_seconds,
            confidence: candidate.confidence,
        })
    }

    fn music_stage_transition_debug_item_stamp(&self, item_id: QueueItemId) -> String {
        let title = self
            .queue_item_by_id(item_id)
            .map(|item| item.title.trim().replace('\r', " ").replace('\n', " "))
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| "unknown track".to_owned());

        if let Some(highlight) = self.music_stage_highlight_debug_label_for_item(item_id) {
            format!(
                "{} {}({:.1}-{:.1}s conf={:.2})",
                title,
                highlight.label,
                highlight.start_seconds,
                highlight.end_seconds,
                highlight.confidence
            )
        } else {
            format!("{} ?(no-highlight)", title)
        }
    }

    fn emit_music_stage_transition_debug_stamp(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
        current_segment: Option<&MusicChorusFlowSegment>,
        planned_segment_end: f64,
        next_start_seconds: Option<f64>,
        transition_seconds: f64,
        fade_duration_seconds: f64,
        crossfade_preview_started: bool,
        plan_reason: &str,
    ) {
        let from_stamp = self.music_stage_transition_debug_item_stamp(current_item_id);
        let to_stamp = self.music_stage_transition_debug_item_stamp(next_item_id);
        let a_range = current_segment
            .map(|segment| format!("{:.1}-{:.1}s", segment.start_seconds, segment.end_seconds))
            .unwrap_or_else(|| "?".to_owned());
        let b_start = next_start_seconds
            .map(|seconds| format!("{seconds:.1}s"))
            .unwrap_or_else(|| "?".to_owned());
        let preview_state = if crossfade_preview_started {
            "preview"
        } else {
            "fallback"
        };
        let diagnostic_text = self.music_stage_transition_debug_diagnostic_text(
            current_item_id,
            current_segment,
            planned_segment_end,
            transition_seconds,
        );

        eprintln!(
            "[music-stage] MIX {from_stamp} -> {to_stamp} | mode={preview_state} transition={transition_seconds:.2}s fade={fade_duration_seconds:.2}s A_range={a_range} A_plan_end={planned_segment_end:.1}s B_start={b_start} | {plan_reason}{diagnostic_text}"
        );
    }

    fn music_stage_transition_debug_diagnostic_text(
        &self,
        current_item_id: QueueItemId,
        current_segment: Option<&MusicChorusFlowSegment>,
        planned_segment_end: f64,
        transition_seconds: f64,
    ) -> String {
        let Some(manifest) = self.music_analysis_manifest_for_item(current_item_id) else {
            return String::new();
        };
        if manifest.energy_curve.len() < 3 {
            return String::new();
        }
        let selected_end = self
            .music_automix_range_for_item(current_item_id)
            .map(|(_, end)| end)
            .or_else(|| current_segment.map(|segment| segment.end_seconds))
            .unwrap_or(planned_segment_end);
        let pre_rms = average_energy_curve_rms(&manifest, selected_end - 2.6, selected_end - 0.25);
        let post_rms = average_energy_curve_rms(&manifest, selected_end + 0.25, selected_end + 2.6);
        let Some((pre_rms, post_rms)) = pre_rms.zip(post_rms) else {
            return String::new();
        };
        let cliff_db = energy_diag_db(post_rms) - energy_diag_db(pre_rms);
        let mix_center_seconds = planned_segment_end - transition_seconds.max(0.0) * 0.5;
        let center_offset = mix_center_seconds - selected_end;
        let plan_tail = planned_segment_end - selected_end;
        let edge_phase = self.music_stage_outgoing_highlight_end_phase_for_transition(
            current_item_id,
            current_segment,
            planned_segment_end,
            transition_seconds,
        );
        let edge_text = edge_phase
            .map(|phase| format!(" Aend@mix{phase:.2}"))
            .unwrap_or_default();
        format!(
            " · diag Acliff {cliff_db:+.1}dB centerHL{center_offset:+.1}s tail{plan_tail:+.1}s{edge_text}"
        )
    }

    fn music_stage_outgoing_highlight_end_phase_for_transition(
        &self,
        current_item_id: QueueItemId,
        current_segment: Option<&MusicChorusFlowSegment>,
        planned_segment_end: f64,
        transition_seconds: f64,
    ) -> Option<f32> {
        if !transition_seconds.is_finite() || transition_seconds <= 0.25 {
            return None;
        }

        let selected_end = self
            .music_automix_range_for_item(current_item_id)
            .map(|(_, end)| end)
            .or_else(|| current_segment.map(|segment| segment.end_seconds))
            .unwrap_or(planned_segment_end);
        if !selected_end.is_finite() || !planned_segment_end.is_finite() {
            return None;
        }

        let transition_start = planned_segment_end - transition_seconds.max(0.0);
        let phase = (selected_end - transition_start) / transition_seconds;
        if !phase.is_finite() || !(0.0..=1.0).contains(&phase) {
            return None;
        }
        Some(phase as f32)
    }

    fn selected_music_stage_highlight_candidate<'a>(
        &self,
        item_id: QueueItemId,
        manifest: &'a crate::app::music_analysis::MusicAnalysisManifest,
    ) -> Option<&'a crate::app::music_analysis::MusicSectionCandidate> {
        if self.music.music_chorus_flow_enabled {
            if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY
                && music_stage_chain_direct_stream_director_enabled()
            {
                if let Some(candidate) =
                    self.music_stage_chain_direct_body_highlight_candidate(item_id, manifest)
                {
                    return Some(candidate);
                }
            }

            if let Some(pick) = self.music.music_stage_pick_selected.get(&item_id) {
                if let Some(candidate) = manifest
                    .sections
                    .highlight_candidates
                    .get(pick.candidate_index)
                {
                    if (candidate.start_seconds - pick.start_seconds).abs() <= 1.5
                        && (candidate.end_seconds - pick.end_seconds).abs() <= 1.5
                        && candidate.confidence >= MUSIC_STAGE_PICK_MIN_CONFIDENCE * 0.65
                    {
                        return Some(candidate);
                    }
                }
            }
        }
        music_segment_selector::best_highlight_candidate(manifest)
    }

    fn music_stage_chain_direct_body_highlight_candidate<'a>(
        &self,
        item_id: QueueItemId,
        manifest: &'a crate::app::music_analysis::MusicAnalysisManifest,
    ) -> Option<&'a crate::app::music_analysis::MusicSectionCandidate> {
        let transition_seconds = self.music_chorus_transition_seconds_for_item(item_id);
        let body_fence =
            Self::music_stage_chain_direct_body_fence_seconds(manifest, transition_seconds)?;
        let duration = manifest.duration_seconds.max(1.0);
        let latest_start = (body_fence - MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS * 0.55).max(0.0);

        let picked = music_segment_selector::select_direct_body_highlight_candidate(
            manifest,
            music_segment_selector::MusicDirectBodyHighlightPolicy {
                body_fence_seconds: body_fence,
                duration_seconds: duration,
                latest_start_seconds: latest_start,
                min_segment_seconds: MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS,
                min_confidence: MUSIC_STAGE_PICK_MIN_CONFIDENCE * 0.55,
                tail_grace_seconds: MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_TAIL_GRACE_SECONDS,
                late_midpoint_share: MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_SONG_SHARE,
            },
        );

        if let Some(candidate) = picked {
            if candidate.end_seconds
                > body_fence + MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_TAIL_GRACE_SECONDS
            {
                eprintln!(
                    "[music-stage-chain] direct body highlight item={} start={:.3}s end={:.3}s fence={:.3}s action=cap-end",
                    item_id, candidate.start_seconds, candidate.end_seconds, body_fence,
                );
            }
            return Some(candidate);
        }

        None
    }

    fn music_automix_entry_start_seconds_for_item(&self, item_id: QueueItemId) -> Option<f64> {
        let (segment_start, segment_end) = self.music_automix_range_for_item(item_id)?;
        let (segment_start, segment_end) =
            self.music_lyrics_safe_range_for_item(item_id, segment_start, segment_end);
        let transition_seconds = self.music_chorus_transition_seconds_for_item(item_id);
        Some(self.music_lyrics_safe_entry_start_for_item(
            item_id,
            self.music_automix_entry_start_for_item(
                item_id,
                segment_start,
                segment_end,
                transition_seconds,
            ),
            segment_start,
            segment_end,
        ))
    }

    fn music_lyrics_safe_range_for_item(
        &self,
        item_id: QueueItemId,
        start_seconds: f64,
        end_seconds: f64,
    ) -> (f64, f64) {
        if end_seconds <= start_seconds + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
            return (start_seconds, end_seconds);
        }

        let min_start = 0.0;
        let max_start =
            (end_seconds - MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS * 0.5).max(start_seconds);
        let min_end = (start_seconds + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS).min(end_seconds);
        let max_end = self
            .music_analysis_manifest_for_item(item_id)
            .map(|manifest| manifest.duration_seconds)
            .filter(|duration| duration.is_finite() && *duration > 0.0)
            .unwrap_or(end_seconds.max(start_seconds));

        let start = self.music_lyrics_safe_entry_start_for_item(
            item_id,
            start_seconds,
            min_start,
            max_start,
        );
        let mut end =
            self.music_lyrics_safe_mix_out_for_item(item_id, end_seconds, min_end, max_end);
        if let Some(tail_safe_end) =
            self.music_tail_safe_highlight_end_for_item(item_id, start, end)
        {
            end = tail_safe_end;
        }

        if end <= start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
            (start_seconds, end_seconds)
        } else {
            (start, end)
        }
    }

    fn music_stage_chain_direct_tempo_fade_seconds_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
        seed_seconds: f64,
        segment_lengths: Option<(f64, f64)>,
    ) -> (f64, Option<String>) {
        let mut seconds = seed_seconds.clamp(
            MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MIN_SECONDS,
            MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MAX_SECONDS,
        );

        if let Some((current_len, next_len)) = segment_lengths {
            let usable_len = current_len.max(0.0).min(next_len.max(0.0));
            if usable_len.is_finite() && usable_len > 0.0 {
                let max_by_segments = (usable_len * 0.72).clamp(
                    MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MIN_SECONDS,
                    MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MAX_SECONDS,
                );
                seconds = seconds
                    .min(max_by_segments)
                    .max(MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MIN_SECONDS.min(max_by_segments));
            }
        }

        if !music_stage_chain_direct_stream_director_enabled() {
            return (seconds, None);
        }

        let Some(current_tempo) = self.music_chorus_stage_tempo_for_item(
            current_item_id,
            MusicChorusStageTempoRole::Outgoing,
        ) else {
            return (seconds, None);
        };
        let Some(next_tempo) = self
            .music_chorus_stage_tempo_for_item(next_item_id, MusicChorusStageTempoRole::Incoming)
        else {
            return (seconds, None);
        };
        let confidence = ((current_tempo.confidence + next_tempo.confidence) * 0.5).clamp(0.0, 1.0);
        if confidence < 0.20 {
            return (seconds, None);
        }

        let current_bpm = current_tempo.bpm.clamp(50.0, 220.0);
        let next_bpm = next_tempo.bpm.clamp(50.0, 220.0);
        let grid = tempo_grid_compatibility_between(current_bpm, next_bpm);
        let gap = grid.effective_gap;
        let mut target: f64 = if gap <= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_CLOSE_GAP {
            5.15
        } else if gap <= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MEDIUM_GAP {
            4.45
        } else if gap <= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_WIDE_GAP {
            3.55
        } else {
            2.75
        };

        if confidence < 0.36 {
            target = target.min(3.35);
        } else if confidence > 0.66 && gap <= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MEDIUM_GAP {
            target += 0.18;
        }

        if let Some(vocal_safety) =
            self.music_chorus_vocal_overlap_safety_between(current_item_id, next_item_id)
        {
            if vocal_safety < 0.22 {
                target = target.min(2.85);
            } else if vocal_safety < 0.32 {
                target = target.min(3.45);
            } else if vocal_safety >= 0.52 && gap <= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MEDIUM_GAP
            {
                target += 0.16;
            }
        }

        if let Some(harmonic) =
            self.music_chorus_harmonic_compatibility_between(current_item_id, next_item_id)
        {
            if harmonic.confidence >= 0.42
                && harmonic.score >= 0.62
                && gap <= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MEDIUM_GAP
            {
                target += 0.14;
            } else if harmonic.confidence >= 0.42 && harmonic.score < 0.28 {
                target = target.min(3.25);
            }
        }

        let limited_target = if let Some((current_len, next_len)) = segment_lengths {
            let usable_len = current_len.max(0.0).min(next_len.max(0.0));
            if usable_len.is_finite() && usable_len > 0.0 {
                let max_by_segments = (usable_len * 0.72).clamp(
                    MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MIN_SECONDS,
                    MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MAX_SECONDS,
                );
                target.min(max_by_segments)
            } else {
                target
            }
        } else {
            target
        };
        seconds = (seconds * 0.32 + limited_target * 0.68).clamp(
            MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MIN_SECONDS,
            MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MAX_SECONDS,
        );

        let note = format!(
            "tempo-fade {:.0}->{:.0}bpm gap {:.3} conf {:.2}",
            current_bpm, grid.adjusted_next_bpm, gap, confidence
        );
        (seconds, Some(note))
    }

    fn music_stage_chain_direct_mix_length_seconds(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
        seed_seconds: f64,
        segment_lengths: Option<(f64, f64)>,
    ) -> (f64, Option<String>) {
        if !music_stage_chain_direct_stream_director_enabled() || !seed_seconds.is_finite() {
            return (seed_seconds, None);
        }

        let ui_length = self.music_stage_direct_mix_length_value();
        let (length, adaptive_note) = self.music_stage_chain_direct_adaptive_mix_length_between(
            current_item_id,
            next_item_id,
            ui_length,
        );
        let multiplier = Self::music_stage_direct_mix_length_multiplier_for(length);
        let mut max_seconds = MUSIC_STAGE_CHAIN_DIRECT_MIX_LENGTH_MAX_SECONDS;
        if let Some((current_len, next_len)) = segment_lengths {
            let usable_len = current_len.max(0.0).min(next_len.max(0.0));
            if usable_len.is_finite() && usable_len > 0.0 {
                max_seconds = max_seconds
                    .min((usable_len * 0.84).max(MUSIC_STAGE_CHAIN_DIRECT_MIX_LENGTH_MIN_SECONDS));
            }
        }

        let adjusted = (seed_seconds * multiplier).clamp(
            MUSIC_STAGE_CHAIN_DIRECT_MIX_LENGTH_MIN_SECONDS.min(max_seconds),
            max_seconds.max(MUSIC_STAGE_CHAIN_DIRECT_MIX_LENGTH_MIN_SECONDS),
        );
        let note_suffix = adaptive_note
            .map(|note| format!(" · {note}"))
            .unwrap_or_default();
        let note = if (adjusted - seed_seconds).abs() > 0.050 {
            Some(format!(
                "mix-length {:.0}% {:.2}x {:.2}s{}",
                length * 100.0,
                multiplier,
                adjusted,
                note_suffix
            ))
        } else {
            Some(format!(
                "mix-length {:.0}% {:.2}x{}",
                length * 100.0,
                multiplier,
                note_suffix
            ))
        };
        (adjusted, note)
    }

    fn music_stage_chain_direct_adaptive_mix_length_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
        ui_length: f64,
    ) -> (f64, Option<String>) {
        if !self.music_stage_direct_adaptive_natural_enabled() {
            return (ui_length, None);
        }

        let mut effective = ui_length;
        let mut reasons: Vec<&'static str> = Vec::new();
        let Some(current_tempo) = self.music_chorus_stage_tempo_for_item(
            current_item_id,
            MusicChorusStageTempoRole::Outgoing,
        ) else {
            return (ui_length, None);
        };
        let Some(next_tempo) = self
            .music_chorus_stage_tempo_for_item(next_item_id, MusicChorusStageTempoRole::Incoming)
        else {
            return (ui_length, None);
        };
        let confidence = ((current_tempo.confidence + next_tempo.confidence) * 0.5).clamp(0.0, 1.0);
        let grid = tempo_grid_compatibility_between(
            current_tempo.bpm.clamp(50.0, 220.0),
            next_tempo.bpm.clamp(50.0, 220.0),
        );

        if grid.effective_gap <= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_CLOSE_GAP && confidence >= 0.48
        {
            effective = effective.max(0.96);
            reasons.push("close-bpm");
        } else if grid.effective_gap >= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_WIDE_GAP {
            effective = effective.min(0.84);
            reasons.push("wide-bpm");
        }

        if let Some(vocal_safety) =
            self.music_chorus_vocal_overlap_safety_between(current_item_id, next_item_id)
        {
            if vocal_safety < 0.24 {
                effective = effective.min(0.72);
                reasons.push("vocal-safe");
            } else if vocal_safety >= 0.56
                && grid.effective_gap <= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MEDIUM_GAP
            {
                effective = effective.max(0.94);
                reasons.push("vocal-clear");
            }
        }

        if let Some(harmonic) =
            self.music_chorus_harmonic_compatibility_between(current_item_id, next_item_id)
        {
            if harmonic.confidence >= 0.42 && harmonic.score < 0.28 {
                effective = effective.min(0.76);
                reasons.push("key-risk");
            } else if harmonic.confidence >= 0.42
                && harmonic.score >= 0.64
                && grid.effective_gap <= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MEDIUM_GAP
            {
                effective = effective.max(0.98);
                reasons.push("key-fit");
            }
        }

        let assist = self.music_stage_direct_mix_assist_value();
        let target = effective.clamp(0.62, 1.00);
        let effective = (ui_length + (target - ui_length) * assist).clamp(0.0, 1.00);
        let note = if (effective - ui_length).abs() > 0.010 || !reasons.is_empty() {
            Some(format!(
                "mix-assist {:.0}% M{:.0}->{:.0} {}",
                assist * 100.0,
                ui_length * 100.0,
                effective * 100.0,
                reasons.join(",")
            ))
        } else {
            Some(format!("mix-assist {:.0}%", assist * 100.0))
        };
        (effective, note)
    }

    fn music_stage_chain_direct_adaptive_bridge_strength_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
        ui_strength: f64,
        tempo_gap: f64,
        confidence: f32,
    ) -> (f64, Option<String>) {
        if !self.music_stage_direct_adaptive_natural_enabled() {
            return (ui_strength, None);
        }

        let mut effective = ui_strength;
        let mut reasons: Vec<&'static str> = Vec::new();
        if tempo_gap <= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_CLOSE_GAP {
            effective -= 0.05;
            reasons.push("close-bpm");
        } else if tempo_gap >= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_WIDE_GAP {
            effective += 0.11;
            reasons.push("wide-bpm");
        } else if tempo_gap >= MUSIC_STAGE_CHAIN_DIRECT_TEMPO_FADE_MEDIUM_GAP {
            effective += 0.06;
            reasons.push("mid-bpm");
        }

        if confidence < 0.36 {
            effective -= 0.07;
            reasons.push("low-conf");
        } else if confidence >= 0.66 {
            effective += 0.04;
            reasons.push("confident");
        }

        if let Some(vocal_safety) =
            self.music_chorus_vocal_overlap_safety_between(current_item_id, next_item_id)
        {
            if vocal_safety < 0.24 {
                effective -= 0.08;
                reasons.push("vocal-safe");
            } else if vocal_safety >= 0.56 {
                effective += 0.03;
                reasons.push("vocal-clear");
            }
        }

        if let Some(harmonic) =
            self.music_chorus_harmonic_compatibility_between(current_item_id, next_item_id)
        {
            if harmonic.confidence >= 0.42 && harmonic.score < 0.28 {
                effective -= 0.05;
                reasons.push("key-risk");
            } else if harmonic.confidence >= 0.42 && harmonic.score >= 0.64 {
                effective += 0.03;
                reasons.push("key-fit");
            }
        }

        let assist = self.music_stage_direct_mix_assist_value();
        let mut target = effective.clamp(0.0, 1.0);
        if target > ui_strength {
            target = ui_strength;
            reasons.push("beat-cap");
        }
        let effective =
            (ui_strength + (target - ui_strength) * assist).clamp(0.0, ui_strength.max(0.0));
        let note = if (effective - ui_strength).abs() > 0.010 || !reasons.is_empty() {
            Some(format!(
                "mix-assist {:.0}% B{:.0}->{:.0} {}",
                assist * 100.0,
                ui_strength * 100.0,
                effective * 100.0,
                reasons.join(",")
            ))
        } else {
            Some(format!("mix-assist {:.0}%", assist * 100.0))
        };
        (effective, note)
    }

    fn music_stage_chain_direct_tempo_bridge_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
        fade_seconds: f64,
    ) -> Option<MusicStageChainDirectTempoBridge> {
        if !MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE
            || !music_stage_chain_direct_stream_director_enabled()
            || !fade_seconds.is_finite()
            || fade_seconds <= 0.0
        {
            return None;
        }

        let current_tempo = self.music_chorus_stage_tempo_for_item(
            current_item_id,
            MusicChorusStageTempoRole::Outgoing,
        )?;
        let next_tempo = self
            .music_chorus_stage_tempo_for_item(next_item_id, MusicChorusStageTempoRole::Incoming)?;
        let confidence = ((current_tempo.confidence + next_tempo.confidence) * 0.5).clamp(0.0, 1.0);
        if confidence < MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_MIN_CONFIDENCE {
            return None;
        }

        let ui_strength = self.music_stage_direct_tempo_bridge_strength_value();
        if ui_strength <= 0.005 {
            return None;
        }
        let current_bpm = current_tempo.bpm.clamp(50.0, 220.0);
        let next_bpm = next_tempo.bpm.clamp(50.0, 220.0);
        let grid = tempo_grid_compatibility_between(current_bpm, next_bpm);
        let (strength, adaptive_note) = self
            .music_stage_chain_direct_adaptive_bridge_strength_between(
                current_item_id,
                next_item_id,
                ui_strength,
                grid.effective_gap,
                confidence,
            );
        if strength <= 0.005 {
            return None;
        }
        let multiplier = self.music_stage_direct_tempo_bridge_strength_multiplier_for(strength);
        let (outgoing_bounds, incoming_bounds) =
            self.music_stage_direct_tempo_bridge_rate_bounds_for(strength);
        let split = self.music_chorus_tempo_split_between(current_item_id, next_item_id);
        let outgoing_rate = music_stage_chain_scale_tempo_rate(
            split.outgoing_rate,
            multiplier,
            outgoing_bounds.0,
            outgoing_bounds.1,
        );
        let incoming_rate = music_stage_chain_scale_tempo_rate(
            split.incoming_rate,
            multiplier,
            incoming_bounds.0,
            incoming_bounds.1,
        );
        let outgoing_delta = (outgoing_rate - 1.0).abs();
        let incoming_delta = (incoming_rate - 1.0).abs();
        if outgoing_delta < MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_MIN_RATE_DELTA
            && incoming_delta < MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_MIN_RATE_DELTA
        {
            return None;
        }

        let entry_factor = MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_ENTRY_FACTOR
            * (0.65 + strength * 0.70).clamp(0.45, 1.35);
        let max_entry_shift_seconds =
            (MUSIC_STAGE_CHAIN_DIRECT_TEMPO_BRIDGE_MAX_ENTRY_SHIFT_SECONDS
                * (0.75 + strength * 0.85))
                .clamp(0.28, 1.18);
        let entry_shift_seconds = ((incoming_rate - 1.0) * fade_seconds * entry_factor)
            .clamp(-max_entry_shift_seconds, max_entry_shift_seconds);
        let adaptive_suffix = adaptive_note
            .map(|note| format!(" · {note}"))
            .unwrap_or_default();
        let note = format!(
            "tempo-bridge {:.0}->{:.0}bpm strength {:.0}% A {:+.1}% B {:+.1}% Bphase {:+.2}s conf {:.2}{}",
            current_bpm,
            grid.adjusted_next_bpm,
            strength * 100.0,
            (outgoing_rate - 1.0) * 100.0,
            (incoming_rate - 1.0) * 100.0,
            entry_shift_seconds,
            confidence,
            adaptive_suffix
        );

        Some(MusicStageChainDirectTempoBridge {
            outgoing_rate,
            incoming_rate,
            entry_shift_seconds,
            note,
        })
    }

    fn music_stage_chain_direct_apply_tempo_bridge_entry_shift(
        &self,
        item_id: QueueItemId,
        start_seconds: f64,
        entry_shift_seconds: f64,
        transition_seconds: f64,
        track_duration_seconds: Option<f64>,
    ) -> f64 {
        if !entry_shift_seconds.is_finite() || entry_shift_seconds.abs() < 0.010 {
            return start_seconds;
        }

        let shifted = (start_seconds + entry_shift_seconds).max(0.0);
        // Re-run the same Direct Entry Anchor / tail / body protection after the
        // phase shift so the pilot never pushes B back into dead air.
        self.music_stage_chain_direct_entry_anchor_start_for_item(
            item_id,
            shifted,
            transition_seconds,
            track_duration_seconds,
        )
    }

    fn music_stage_chain_direct_entry_anchor_start_for_item(
        &self,
        item_id: QueueItemId,
        entry_start_seconds: f64,
        transition_seconds: f64,
        track_duration_seconds: Option<f64>,
    ) -> f64 {
        let guarded_start = music_stage_chain_safe_entry_start_seconds(
            entry_start_seconds,
            transition_seconds,
            track_duration_seconds,
        );

        if !music_stage_chain_direct_stream_director_enabled() {
            return guarded_start;
        }

        let target_start = self.music_stage_chain_direct_tail_safe_entry_start_for_item(
            item_id,
            guarded_start,
            transition_seconds,
            track_duration_seconds,
        );

        let pullback_applied = target_start + 0.050 < entry_start_seconds;
        let anchor_window = if pullback_applied {
            MUSIC_STAGE_CHAIN_DIRECT_ENTRY_PULLBACK_ANCHOR_WINDOW_SECONDS
        } else {
            MUSIC_STAGE_CHAIN_DIRECT_ENTRY_ANCHOR_WINDOW_SECONDS
        };
        let min_seconds = (target_start - anchor_window).max(0.0);
        let mut max_seconds = target_start + anchor_window;
        if let Some(duration) =
            track_duration_seconds.filter(|duration| duration.is_finite() && *duration > 0.0)
        {
            let latest_with_runway =
                music_stage_chain_direct_latest_entry_start_seconds(duration, transition_seconds);
            max_seconds = max_seconds.min(latest_with_runway.max(min_seconds));
        }
        if max_seconds < min_seconds + 0.001 {
            return target_start.max(0.0);
        }

        if let Some(anchor) = self.music_stage_chain_direct_lyric_or_section_anchor_start(
            item_id,
            target_start,
            min_seconds,
            max_seconds,
        ) {
            return anchor;
        }

        target_start.max(0.0)
    }

    fn music_stage_chain_direct_tail_safe_entry_start_for_item(
        &self,
        item_id: QueueItemId,
        entry_start_seconds: f64,
        transition_seconds: f64,
        track_duration_seconds: Option<f64>,
    ) -> f64 {
        if !entry_start_seconds.is_finite() {
            return 0.0;
        }

        let duration = track_duration_seconds.filter(|duration| {
            duration.is_finite()
                && *duration > MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS
                && *duration > transition_seconds + MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS
        });
        let body_fence = self
            .music_analysis_manifest_for_item(item_id)
            .and_then(|manifest| {
                Self::music_stage_chain_direct_body_fence_seconds(&manifest, transition_seconds)
            });
        let tail_section_start = duration.and_then(|duration| {
            self.music_stage_chain_direct_tail_section_start_for_item(
                item_id,
                duration,
                entry_start_seconds,
            )
        });
        let last_lyric_seconds = duration.and_then(|duration| {
            self.music_stage_chain_direct_last_lyric_seconds_for_item(item_id, duration)
        });
        let last_audible_seconds = duration.and_then(|duration| {
            self.music_stage_chain_direct_last_audible_seconds_for_item(item_id, duration)
        });
        let plan = music_segment_selector::direct_tail_safe_entry_start_seconds(
            entry_start_seconds,
            transition_seconds,
            duration,
            body_fence,
            tail_section_start,
            last_lyric_seconds,
            last_audible_seconds,
            music_segment_selector::MusicStageTailSafeEntryPolicy {
                min_segment_seconds: MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS,
                advance_guard_seconds: MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS,
                min_remaining_seconds: MUSIC_STAGE_CHAIN_DIRECT_TAIL_ENTRY_MIN_REMAINING_SECONDS,
                post_promote_min_dwell_seconds: MUSIC_STAGE_POST_PROMOTE_MIN_DWELL_SECONDS,
                extra_runway_seconds: 8.0,
                tail_section_backoff_seconds: MUSIC_STAGE_CHAIN_DIRECT_TAIL_SECTION_BACKOFF_SECONDS,
                trailing_silence_min_seconds: MUSIC_STAGE_CHAIN_DIRECT_TRAILING_SILENCE_MIN_SECONDS,
                last_lyric_backoff_seconds: MUSIC_STAGE_CHAIN_DIRECT_LAST_LYRIC_BACKOFF_SECONDS,
                energy_tail_min_seconds: MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_MIN_SECONDS,
                energy_tail_entry_backoff_seconds:
                    MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_ENTRY_BACKOFF_SECONDS,
            },
        );
        if plan.start_seconds + 0.050 < entry_start_seconds {
            eprintln!(
                "[music-stage-chain] direct tail entry guard item={} old={:.3}s new={:.3}s runway={:.3}s reason={}",
                item_id,
                entry_start_seconds,
                plan.start_seconds,
                duration
                    .map(|duration| (duration - plan.start_seconds).max(0.0))
                    .unwrap_or(0.0),
                plan.reason.log_key(),
            );
        }
        plan.start_seconds
    }

    fn music_stage_chain_direct_body_fence_safe_exit_end_for_item(
        &self,
        item_id: QueueItemId,
        playback_start_seconds: f64,
        segment_end_seconds: f64,
        transition_seconds: f64,
        duration_seconds: Option<f64>,
    ) -> Option<f64> {
        if !playback_start_seconds.is_finite()
            || !segment_end_seconds.is_finite()
            || !transition_seconds.is_finite()
        {
            return None;
        }

        let manifest = self.music_analysis_manifest_for_item(item_id)?;
        let duration = duration_seconds
            .filter(|duration| duration.is_finite() && *duration > 0.0)
            .unwrap_or(manifest.duration_seconds);
        if !duration.is_finite() || duration <= MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
            return None;
        }

        let body_fence =
            Self::music_stage_chain_direct_body_fence_seconds(&manifest, transition_seconds)?;
        if let Some(safe_end) = music_segment_selector::body_fence_safe_exit_end_seconds(
            playback_start_seconds,
            segment_end_seconds,
            transition_seconds,
            duration,
            body_fence,
            music_segment_selector::MusicStageBodyFenceExitPolicy {
                tail_grace_seconds: MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_TAIL_GRACE_SECONDS,
                transition_min_seconds: MUSIC_STAGE_LITE_TRANSITION_MIN_SECONDS,
                advance_guard_seconds: MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS,
            },
        ) {
            eprintln!(
                "[music-stage-chain] direct body exit guard item={} old_end={:.3}s new_end={:.3}s fence={:.3}s runway={:.3}s",
                item_id,
                segment_end_seconds,
                safe_end,
                body_fence,
                (duration - safe_end).max(0.0),
            );
            Some(safe_end)
        } else {
            None
        }
    }

    fn music_stage_chain_direct_body_fence_seconds(
        manifest: &crate::app::music_analysis::MusicAnalysisManifest,
        transition_seconds: f64,
    ) -> Option<f64> {
        music_segment_selector::direct_body_fence_seconds(
            manifest,
            transition_seconds,
            music_segment_selector::MusicStageBodyFencePolicy {
                min_segment_seconds: MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS,
                transition_min_seconds: MUSIC_STAGE_LITE_TRANSITION_MIN_SECONDS,
                min_remaining_seconds: MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_MIN_REMAINING_SECONDS,
                post_promote_min_dwell_seconds: MUSIC_STAGE_POST_PROMOTE_MIN_DWELL_SECONDS,
                song_share: MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_SONG_SHARE,
                outro_backoff_seconds: MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_OUTRO_BACKOFF_SECONDS,
            },
        )
    }

    fn music_stage_chain_direct_energy_tail_safe_exit_end_for_item(
        &self,
        item_id: QueueItemId,
        playback_start_seconds: f64,
        segment_end_seconds: f64,
        transition_seconds: f64,
        duration_seconds: Option<f64>,
    ) -> Option<f64> {
        if !playback_start_seconds.is_finite()
            || !segment_end_seconds.is_finite()
            || !transition_seconds.is_finite()
        {
            return None;
        }
        let duration = duration_seconds
            .or_else(|| self.music_chorus_duration_seconds_for_item(item_id))
            .filter(|duration| duration.is_finite() && *duration > 0.0)?;
        let last_audible_seconds =
            self.music_stage_chain_direct_last_audible_seconds_for_item(item_id, duration)?;
        if let Some(safe_end) = music_segment_selector::energy_tail_safe_exit_end_seconds(
            playback_start_seconds,
            segment_end_seconds,
            transition_seconds,
            duration,
            last_audible_seconds,
            music_segment_selector::MusicStageEnergyTailExitPolicy {
                min_tail_seconds: MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_MIN_SECONDS,
                exit_grace_seconds: MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_EXIT_GRACE_SECONDS,
                transition_min_seconds: MUSIC_STAGE_LITE_TRANSITION_MIN_SECONDS,
                advance_guard_seconds: MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS,
            },
        ) {
            eprintln!(
                "[music-stage-chain] energy tail exit guard item={} old_end={:.3}s new_end={:.3}s last_audible={:.3}s tail={:.3}s",
                item_id,
                segment_end_seconds,
                safe_end,
                last_audible_seconds,
                (duration - last_audible_seconds).max(0.0),
            );
            Some(safe_end)
        } else {
            None
        }
    }

    fn music_stage_chain_direct_last_audible_seconds_for_item(
        &self,
        item_id: QueueItemId,
        duration_seconds: f64,
    ) -> Option<f64> {
        let manifest = self.music_analysis_manifest_for_item(item_id)?;
        let duration = if duration_seconds.is_finite() && duration_seconds > 0.0 {
            duration_seconds
        } else {
            manifest.duration_seconds
        };
        if !duration.is_finite() || duration <= MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
            return None;
        }
        music_segment_selector::last_audible_seconds_from_energy(
            &manifest,
            duration,
            music_segment_selector::MusicStageEnergyTailPolicy {
                min_segment_seconds: MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS,
                relative_rms: MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_RELATIVE_RMS,
                peak_rms: MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_PEAK_RMS,
                min_rms: MUSIC_STAGE_CHAIN_DIRECT_ENERGY_TAIL_MIN_RMS,
            },
        )
    }

    fn music_stage_chain_direct_tail_section_start_for_item(
        &self,
        item_id: QueueItemId,
        duration_seconds: f64,
        entry_start_seconds: f64,
    ) -> Option<f64> {
        let manifest = self.music_analysis_manifest_for_item(item_id)?;
        manifest
            .sections
            .functional_segments
            .iter()
            .filter(|segment| {
                segment.start_seconds.is_finite()
                    && segment.start_seconds >= MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS
                    && segment.start_seconds <= duration_seconds
                    && (segment.start_seconds
                        >= entry_start_seconds - MUSIC_CHORUS_TAIL_SILENCE_LOOKAHEAD_SECONDS
                        || duration_seconds - segment.start_seconds
                            <= MUSIC_CHORUS_TAIL_DIRECT_HANDOFF_SECONDS
                                + MUSIC_CHORUS_TAIL_SILENCE_LOOKAHEAD_SECONDS)
                    && matches!(
                        segment.role,
                        crate::app::music_analysis::MusicFunctionalRole::Outro
                            | crate::app::music_analysis::MusicFunctionalRole::Silence
                    )
            })
            .map(|segment| segment.start_seconds)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    fn music_stage_chain_direct_last_lyric_seconds_for_item(
        &self,
        item_id: QueueItemId,
        duration_seconds: f64,
    ) -> Option<f64> {
        self.music_lrc_lines_for_item(item_id)?
            .into_iter()
            .map(|line| line.seconds)
            .filter(|seconds| seconds.is_finite())
            .filter(|seconds| {
                *seconds >= 0.0
                    && *seconds <= duration_seconds + MUSIC_CHORUS_LYRIC_SNAP_WINDOW_SECONDS
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    fn music_stage_chain_direct_lyric_or_section_anchor_start(
        &self,
        item_id: QueueItemId,
        target_seconds: f64,
        min_seconds: f64,
        max_seconds: f64,
    ) -> Option<f64> {
        if let Some(lyric_anchor) = self.music_lyrics_boundary_snap_for_item(
            item_id,
            target_seconds,
            min_seconds,
            max_seconds,
            true,
        ) {
            return Some(lyric_anchor);
        }

        let manifest = self.music_analysis_manifest_for_item(item_id)?;
        manifest
            .sections
            .functional_segments
            .iter()
            .filter(|segment| {
                segment.start_seconds.is_finite()
                    && segment.start_seconds >= min_seconds
                    && segment.start_seconds <= max_seconds
                    && !matches!(
                        segment.role,
                        crate::app::music_analysis::MusicFunctionalRole::Outro
                            | crate::app::music_analysis::MusicFunctionalRole::Silence
                    )
            })
            .min_by(|a, b| {
                let a_score = music_stage_chain_direct_entry_anchor_score(a, target_seconds);
                let b_score = music_stage_chain_direct_entry_anchor_score(b, target_seconds);
                a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|segment| segment.start_seconds.clamp(min_seconds, max_seconds))
    }

    fn music_tail_safe_highlight_end_for_item(
        &self,
        item_id: QueueItemId,
        start_seconds: f64,
        end_seconds: f64,
    ) -> Option<f64> {
        if !start_seconds.is_finite() || !end_seconds.is_finite() {
            return None;
        }
        if end_seconds <= start_seconds + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
            return None;
        }

        let manifest = self.music_analysis_manifest_for_item(item_id)?;
        let duration = manifest.duration_seconds;
        if !duration.is_finite() || duration <= 0.0 {
            return None;
        }

        let near_tail_by_duration = duration - end_seconds
            <= MUSIC_CHORUS_TAIL_DIRECT_HANDOFF_SECONDS
                + MUSIC_CHORUS_TAIL_SILENCE_LOOKAHEAD_SECONDS * 0.5;
        let tail_section_start = manifest
            .sections
            .functional_segments
            .iter()
            .filter(|segment| {
                matches!(
                    segment.role,
                    crate::app::music_analysis::MusicFunctionalRole::Outro
                        | crate::app::music_analysis::MusicFunctionalRole::Silence
                )
            })
            .map(|segment| segment.start_seconds)
            .filter(|seconds| seconds.is_finite())
            .filter(|seconds| {
                *seconds >= start_seconds + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS
                    && *seconds <= end_seconds + MUSIC_CHORUS_TAIL_SILENCE_LOOKAHEAD_SECONDS
            })
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        if !near_tail_by_duration && tail_section_start.is_none() {
            return None;
        }

        let last_lyric_seconds = self
            .music_lrc_lines_for_item(item_id)?
            .into_iter()
            .map(|line| line.seconds)
            .filter(|seconds| seconds.is_finite())
            .filter(|seconds| {
                *seconds >= start_seconds - 0.35
                    && *seconds <= end_seconds + MUSIC_CHORUS_LYRIC_SNAP_WINDOW_SECONDS
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))?;

        if end_seconds - last_lyric_seconds < MUSIC_CHORUS_TAIL_LYRIC_GAP_SECONDS {
            return None;
        }

        let mut capped_end =
            (last_lyric_seconds + MUSIC_CHORUS_TAIL_LYRIC_GRACE_SECONDS).min(end_seconds);
        if let Some(tail_start) = tail_section_start {
            capped_end = capped_end
                .min((tail_start - MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS).max(start_seconds));
        }
        capped_end = capped_end.clamp(
            start_seconds + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS,
            end_seconds,
        );

        if capped_end < end_seconds - 0.25 {
            Some(capped_end)
        } else {
            None
        }
    }

    fn music_lyrics_safe_entry_start_for_item(
        &self,
        item_id: QueueItemId,
        target_seconds: f64,
        min_seconds: f64,
        max_seconds: f64,
    ) -> f64 {
        self.music_lyrics_boundary_snap_for_item(
            item_id,
            target_seconds,
            min_seconds,
            max_seconds,
            true,
        )
        .unwrap_or(target_seconds)
    }

    fn music_lyrics_safe_mix_out_for_item(
        &self,
        item_id: QueueItemId,
        target_seconds: f64,
        min_seconds: f64,
        max_seconds: f64,
    ) -> f64 {
        self.music_lyrics_boundary_snap_for_item(
            item_id,
            target_seconds,
            min_seconds,
            max_seconds,
            false,
        )
        .unwrap_or(target_seconds)
    }

    fn music_lyrics_boundary_snap_for_item(
        &self,
        item_id: QueueItemId,
        target_seconds: f64,
        min_seconds: f64,
        max_seconds: f64,
        incoming: bool,
    ) -> Option<f64> {
        if !target_seconds.is_finite() {
            return None;
        }
        let lines = self.music_lrc_lines_for_item(item_id)?;
        let lead = if incoming {
            MUSIC_CHORUS_LYRIC_START_LEAD_SECONDS
        } else {
            MUSIC_CHORUS_LYRIC_END_LEAD_SECONDS
        };
        lines
            .iter()
            .map(|line| line.seconds - lead)
            .filter(|seconds| seconds.is_finite())
            .filter(|seconds| {
                *seconds >= min_seconds
                    && *seconds <= max_seconds
                    && (*seconds - target_seconds).abs() <= MUSIC_CHORUS_LYRIC_SNAP_WINDOW_SECONDS
            })
            .min_by(|a, b| {
                (*a - target_seconds)
                    .abs()
                    .partial_cmp(&(*b - target_seconds).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|seconds| seconds.clamp(min_seconds, max_seconds))
    }

    fn music_lrc_lines_for_item(&self, item_id: QueueItemId) -> Option<Vec<LrcLine>> {
        let item = self.queue_item_by_id(item_id)?;
        let cache_key = item.music_cache_key.trim();
        if cache_key.is_empty() {
            return None;
        }
        let path = music_lrc_cache_path(&self.music_stream_cache_root(), cache_key);
        if !path.is_file() {
            return None;
        }
        parse_lrc_file(&path).ok().filter(|lines| !lines.is_empty())
    }

    fn music_automix_segment_for_item(
        &self,
        item_id: QueueItemId,
    ) -> Option<music_segment_selector::MusicPlayableSegment> {
        if self.music.music_chorus_flow_enabled {
            return self.music_highlight_segment_for_item(item_id);
        }
        if self.music.music_trim_enabled {
            let (start, end) = self.music_trim_range_for_item(item_id)?;
            return music_segment_selector::MusicPlayableSegment::new(
                start,
                end,
                music_segment_selector::MusicPlayableSegmentSource::Trim,
            );
        }
        let (start, end) = self.music_full_range_for_item(item_id)?;
        music_segment_selector::MusicPlayableSegment::new(
            start,
            end,
            music_segment_selector::MusicPlayableSegmentSource::FullRange,
        )
    }

    fn music_highlight_segment_for_item(
        &self,
        item_id: QueueItemId,
    ) -> Option<music_segment_selector::MusicPlayableSegment> {
        let Some(manifest) = self.music_analysis_manifest_for_item(item_id) else {
            let (start, end) = self.music_provisional_highlight_range_for_item(item_id)?;
            return music_segment_selector::MusicPlayableSegment::new(
                start,
                end,
                music_segment_selector::MusicPlayableSegmentSource::HighlightQuickEstimate,
            );
        };
        let candidate = self.selected_music_stage_highlight_candidate(item_id, &manifest);
        let raw_plan =
            music_segment_selector::attention_highlight_range_plan(&manifest, candidate)?;
        let (start, end) = self.music_highlight_range_for_raw_plan(item_id, &manifest, raw_plan)?;
        music_segment_selector::MusicPlayableSegment::new(
            start,
            end,
            music_segment_selector::MusicPlayableSegmentSource::from_attention_highlight_source(
                raw_plan.source,
            ),
        )
    }

    fn music_automix_range_for_item(&self, item_id: QueueItemId) -> Option<(f64, f64)> {
        self.music_automix_segment_for_item(item_id)
            .map(|segment| segment.as_range())
    }

    fn music_full_range_for_item(&self, item_id: QueueItemId) -> Option<(f64, f64)> {
        if let Some(manifest) = self.music_analysis_manifest_for_item(item_id) {
            // Full song owns the physical playback range. Smart Mix may start
            // an overlap before EOF, but a safe Mix-out point must never become
            // a shortened playback boundary or make the last song end early.
            return music_full_song_playback_range(manifest.duration_seconds);
        }

        let duration = self
            .queue_item_by_id(item_id)
            .and_then(|item| item.music_duration_seconds)?;
        music_full_song_playback_range(duration)
    }

    fn music_provisional_highlight_range_for_item(
        &self,
        item_id: QueueItemId,
    ) -> Option<(f64, f64)> {
        let duration = self
            .queue_item_by_id(item_id)
            .and_then(|item| item.music_duration_seconds)
            .or_else(|| {
                self.music
                    .music_playback
                    .as_ref()
                    .filter(|control| control.item_id == item_id)
                    .and_then(|control| control.duration_seconds())
            })?;
        music_stage_provisional_highlight_range_for_duration(duration)
    }

    fn music_automix_low_energy_tail_start_seconds(
        manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    ) -> Option<f64> {
        music_segment_selector::low_energy_tail_start_seconds(
            manifest,
            MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS,
            MUSIC_CHORUS_TRANSITION_MIN_SECONDS,
        )
    }

    fn music_automix_low_energy_head_end_seconds(
        manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    ) -> Option<f64> {
        music_segment_selector::low_energy_head_end_seconds(manifest)
    }

    fn music_automix_attention_trim_range_plan(
        manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    ) -> Option<music_segment_selector::MusicAttentionTrimRangePlan> {
        music_segment_selector::attention_trim_range_plan(
            manifest,
            music_segment_selector::default_attention_trim_policy(
                MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS,
                MUSIC_CHORUS_TRANSITION_MIN_SECONDS,
            ),
        )
    }

    fn music_trim_range_for_item(&self, item_id: QueueItemId) -> Option<(f64, f64)> {
        let manifest = self.music_analysis_manifest_for_item(item_id)?;
        // Trim reads the same selector-side Attention Profile as highlight and
        // mix decisions.  The consumer projection is inverted: keep high
        // attention regions, and only shave head/tail regions that look like
        // low-content edge material.  Playback handoff still receives a plain
        // frame-addressed segment range after this point.
        let trim_plan = Self::music_automix_attention_trim_range_plan(&manifest)?;
        let mut start = trim_plan
            .start_seconds
            .clamp(0.0, manifest.duration_seconds.max(0.0));
        let mut end = trim_plan
            .end_seconds
            .clamp(start, manifest.duration_seconds.max(start));

        if let Some(point) = best_vocal_safe_mix_point_near(&manifest.mix_points.mix_in, start, 4.0)
        {
            if point.time_seconds <= start + 3.0 {
                start = point.time_seconds.clamp(0.0, end);
            }
        }
        if let Some(point) = best_vocal_safe_mix_point_near(&manifest.mix_points.mix_out, end, 6.0)
        {
            if point.time_seconds >= start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
                end = point
                    .time_seconds
                    .clamp(start, manifest.duration_seconds.max(start));
            }
        }
        if let Some(tail_start) = Self::music_automix_low_energy_tail_start_seconds(&manifest) {
            if tail_start > start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS && tail_start < end {
                end = tail_start;
            }
        }
        // Trim is intentionally still conservative, but its visible/playable
        // boundaries should land on nearby musical grid points when the
        // analyzer has a beat grid.  This improves head/tail feel without
        // replacing the existing intro/outro and vocal-safe decisions.
        if let Some(snapped_start) =
            snap_time_to_nearest_beat(start, &manifest, MUSIC_CHORUS_BEAT_SNAP_WINDOW_SECONDS)
        {
            if snapped_start <= end - MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
                start = snapped_start.clamp(0.0, end);
            }
        }
        if let Some(snapped_end) =
            snap_time_to_nearest_beat(end, &manifest, MUSIC_CHORUS_BEAT_SNAP_WINDOW_SECONDS)
        {
            if snapped_end >= start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
                end = snapped_end.clamp(start, manifest.duration_seconds.max(start));
            }
        }

        if end <= start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
            Some((0.0, manifest.duration_seconds))
        } else {
            Some((start, end))
        }
    }

    fn music_highlight_range_for_item(&self, item_id: QueueItemId) -> Option<(f64, f64)> {
        self.music_highlight_segment_for_item(item_id)
            .map(|segment| segment.as_range())
    }

    fn music_highlight_range_for_raw_plan(
        &self,
        item_id: QueueItemId,
        manifest: &crate::app::music_analysis::MusicAnalysisManifest,
        raw_plan: music_segment_selector::MusicAttentionHighlightRangePlan,
    ) -> Option<(f64, f64)> {
        let duration_seconds = manifest.duration_seconds.max(0.0);
        if let (Some(guarded_source), Some(guard_reason)) =
            (raw_plan.guarded_source, raw_plan.guard_reason)
        {
            let log_key = format!(
                "attention-guard:{item_id}:{}:{guard_reason}:{}:{:.2}:{:.2}",
                guarded_source.log_key(),
                raw_plan.source.log_key(),
                raw_plan.start_seconds,
                raw_plan.end_seconds
            );
            if music_attention_runtime_log_once(log_key) {
                eprintln!(
                    "[music-attention] runtime guard item={} rejected={} reason={} chosen={} chosen_conf={:.2} risk={:.2} kernel={} focus_score={} focus_rt={} overlap={} candidate_score={} map_score={}",
                    item_id,
                    guarded_source.log_key(),
                    guard_reason,
                    raw_plan.source.log_key(),
                    raw_plan.selection_confidence,
                    raw_plan.selection_risk,
                    raw_plan.selection_reason,
                    format_optional_score(raw_plan.focus_score),
                    format_optional_score(raw_plan.focus_runtime_score),
                    format_optional_score(raw_plan.focus_overlap),
                    format_optional_score(raw_plan.candidate_score),
                    format_optional_score(raw_plan.map_runtime_score.map(|score| score as f32)),
                );
            }
        }
        if let (
            music_segment_selector::MusicAttentionHighlightRangeSource::Candidate,
            Some(map_reason),
        ) = (raw_plan.source, raw_plan.rejected_map_reason)
        {
            let log_key = format!(
                "map-guard:{item_id}:{map_reason}:{}:{:.2}:{:.2}",
                raw_plan.source.log_key(),
                raw_plan.start_seconds,
                raw_plan.end_seconds
            );
            if music_attention_runtime_log_once(log_key) {
                eprintln!(
                    "[music-stage-map] runtime guard item={} rejected=music-map-span reason={} chosen={} chosen_conf={:.2} risk={:.2} kernel={} map_conf={} map_score={} candidate_score={}",
                    item_id,
                    map_reason,
                    raw_plan.source.log_key(),
                    raw_plan.selection_confidence,
                    raw_plan.selection_risk,
                    raw_plan.selection_reason,
                    format_optional_score(raw_plan.rejected_map_confidence),
                    format_optional_score(
                        raw_plan
                            .rejected_map_runtime_score
                            .map(|score| score as f32)
                    ),
                    format_optional_score(raw_plan.candidate_score),
                );
            }
        }
        if let (
            music_segment_selector::MusicAttentionHighlightRangeSource::MusicMapSpan,
            Some(reference_start),
            Some(reference_end),
            Some(lift_seconds),
            Some(peak_seconds),
            Some(confidence),
            Some(runtime_score),
        ) = (
            raw_plan.source,
            raw_plan.reference_start_seconds,
            raw_plan.reference_end_seconds,
            raw_plan.map_lift_seconds,
            raw_plan.map_peak_seconds,
            raw_plan.map_confidence,
            raw_plan.map_runtime_score,
        ) {
            // Keep this log sparse: it appears only when v8 Music Map changes
            // the old candidate boundary enough to be audible in playback.
            if (raw_plan.start_seconds - reference_start).abs()
                >= MUSIC_STAGE_MAP_SPAN_RUNTIME_LOG_DELTA_SECONDS
                || (raw_plan.end_seconds - reference_end).abs()
                    >= MUSIC_STAGE_MAP_SPAN_RUNTIME_LOG_DELTA_SECONDS
            {
                let log_key = format!(
                    "map-span:{item_id}:{}:{:.2}:{:.2}:{:.2}:{:.2}",
                    raw_plan.selection_reason,
                    reference_start,
                    reference_end,
                    raw_plan.start_seconds,
                    raw_plan.end_seconds
                );
                if music_attention_runtime_log_once(log_key) {
                    eprintln!(
                        "[music-stage-map] runtime span item={} source={} kernel={} risk={:.2} candidate={:.3}-{:.3}s span={:.3}-{:.3}s lift={:.3}s peak={:.3}s conf={:.2} score={:.2}",
                        item_id,
                        raw_plan.source.log_key(),
                        raw_plan.selection_reason,
                        raw_plan.selection_risk,
                        reference_start,
                        reference_end,
                        raw_plan.start_seconds,
                        raw_plan.end_seconds,
                        lift_seconds,
                        peak_seconds,
                        confidence,
                        runtime_score,
                    );
                }
            }
        }
        if let (
            music_segment_selector::MusicAttentionHighlightRangeSource::FocusZone,
            Some(reference_start),
            Some(reference_end),
            Some(score),
            Some(attention_score),
            Some(structural_score),
        ) = (
            raw_plan.source,
            raw_plan.reference_start_seconds,
            raw_plan.reference_end_seconds,
            raw_plan.focus_score,
            raw_plan.focus_attention_score,
            raw_plan.focus_structural_score,
        ) {
            if (raw_plan.start_seconds - reference_start).abs()
                >= MUSIC_STAGE_MAP_SPAN_RUNTIME_LOG_DELTA_SECONDS
                || (raw_plan.end_seconds - reference_end).abs()
                    >= MUSIC_STAGE_MAP_SPAN_RUNTIME_LOG_DELTA_SECONDS
            {
                let log_key = format!(
                    "focus-span:{item_id}:{}:{:.2}:{:.2}:{:.2}:{:.2}",
                    raw_plan.selection_reason,
                    reference_start,
                    reference_end,
                    raw_plan.start_seconds,
                    raw_plan.end_seconds
                );
                if music_attention_runtime_log_once(log_key) {
                    eprintln!(
                        "[music-attention] runtime focus item={} source={} kernel={} conf={:.2} risk={:.2} ref={:.3}-{:.3}s focus={:.3}-{:.3}s score={:.2} attention={:.2} structural={:.2}",
                        item_id,
                        raw_plan.source.log_key(),
                        raw_plan.selection_reason,
                        raw_plan.selection_confidence,
                        raw_plan.selection_risk,
                        reference_start,
                        reference_end,
                        raw_plan.start_seconds,
                        raw_plan.end_seconds,
                        score,
                        attention_score,
                        structural_score,
                    );
                }
            }
        }
        let (raw_start, raw_end) = (raw_plan.start_seconds, raw_plan.end_seconds);
        let raw_start = raw_start.clamp(0.0, duration_seconds);
        let raw_end = raw_end.clamp(raw_start, duration_seconds.max(raw_start));
        let mut start =
            snap_time_to_nearest_beat(raw_start, manifest, MUSIC_CHORUS_BEAT_SNAP_WINDOW_SECONDS)
                .unwrap_or(raw_start)
                .clamp(0.0, raw_end);
        let mut end =
            snap_time_to_nearest_beat(raw_end, manifest, MUSIC_CHORUS_BEAT_SNAP_WINDOW_SECONDS)
                .unwrap_or(raw_end)
                .clamp(start, manifest.duration_seconds.max(start));

        if let Some(point) =
            best_vocal_safe_mix_point_near(&manifest.mix_points.mix_in, raw_start, 5.0)
        {
            if point.time_seconds <= raw_start + 4.0 {
                start = point.time_seconds.clamp(0.0, raw_end);
            }
        }
        if let Some(point) =
            best_vocal_safe_mix_point_near(&manifest.mix_points.mix_out, raw_end, 7.5)
        {
            if point.time_seconds >= start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
                end = point
                    .time_seconds
                    .clamp(start, manifest.duration_seconds.max(start));
            }
        }

        if let Some(tail_start) = Self::music_automix_low_energy_tail_start_seconds(manifest) {
            if tail_start > start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS && tail_start < end {
                end = tail_start;
            }
        }

        if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY
            && music_stage_chain_direct_stream_director_enabled()
        {
            let transition_seconds = self.music_chorus_transition_seconds_for_item(item_id);
            if let Some(body_fence) =
                Self::music_stage_chain_direct_body_fence_seconds(manifest, transition_seconds)
            {
                if start >= body_fence - MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS * 0.65 {
                    let fallback_len = (raw_end - raw_start)
                        .clamp(MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS, 18.0)
                        .min(body_fence.max(MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS));
                    let new_start = (body_fence - fallback_len).max(0.0);
                    eprintln!(
                        "[music-stage-chain] direct body highlight synth item={} old_start={:.3}s old_end={:.3}s new_start={:.3}s new_end={:.3}s",
                        item_id, start, end, new_start, body_fence,
                    );
                    start = new_start;
                    end = body_fence;
                } else if end > body_fence + MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_TAIL_GRACE_SECONDS
                    && body_fence > start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS
                {
                    eprintln!(
                        "[music-stage-chain] direct body highlight cap item={} old_end={:.3}s new_end={:.3}s",
                        item_id, end, body_fence,
                    );
                    end = body_fence;
                }
            }
        }

        // v10.12.53: Presence Target + Delta Smoothing + Highlight Tail
        // Protection.  The tail/end of the selected highlight is the protected
        // payoff; balance should stretch or shrink the head/pre-roll instead of
        // cutting away the final hook or decision phrase.
        let balanced_range =
            self.music_stage_presence_balanced_range_for_item(item_id, manifest, start, end);
        start = balanced_range.0;
        end = balanced_range.1;

        if MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY
            && music_stage_chain_direct_stream_director_enabled()
        {
            let transition_seconds = self.music_chorus_transition_seconds_for_item(item_id);
            if let Some(body_fence) =
                Self::music_stage_chain_direct_body_fence_seconds(manifest, transition_seconds)
            {
                if end > body_fence + MUSIC_STAGE_CHAIN_DIRECT_BODY_FENCE_TAIL_GRACE_SECONDS
                    && body_fence > start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS
                {
                    eprintln!(
                        "[music-stage-chain] direct body highlight post-balance cap item={} old_end={:.3}s new_end={:.3}s",
                        item_id, end, body_fence,
                    );
                    end = body_fence;
                }
            }
        }

        if end <= start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
            Some((raw_start, raw_end))
        } else {
            Some((start, end))
        }
    }

    fn music_stage_presence_balanced_range_for_item(
        &self,
        item_id: QueueItemId,
        manifest: &crate::app::music_analysis::MusicAnalysisManifest,
        start_seconds: f64,
        current_end_seconds: f64,
    ) -> (f64, f64) {
        if !start_seconds.is_finite() || !current_end_seconds.is_finite() {
            return (start_seconds, current_end_seconds);
        }

        let duration = manifest.duration_seconds;
        if !duration.is_finite()
            || duration <= start_seconds + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS
        {
            return (start_seconds, current_end_seconds);
        }

        let mut protected_end = current_end_seconds.clamp(
            start_seconds,
            (duration - MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS).max(start_seconds),
        );
        if let Some(tail_start) = Self::music_automix_low_energy_tail_start_seconds(manifest) {
            if tail_start > start_seconds + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS
                && tail_start < protected_end
            {
                protected_end = tail_start;
            }
        }

        let target_seconds = self.music_stage_presence_target_seconds_for_item(item_id, manifest);
        let memory = self.music_stage_cue_memory_entry_for_item(item_id, manifest);
        let memory_weight = memory
            .as_ref()
            .and_then(|entry| music_stage_cue_memory_apply_weight(entry))
            .unwrap_or(0.0);
        let memory_presence = memory
            .as_ref()
            .map(|entry| entry.effective_presence_seconds)
            .filter(|seconds| seconds.is_finite() && *seconds > 0.0);
        let target_seconds = if let Some(memory_presence) = memory_presence {
            (target_seconds * (1.0 - memory_weight) + memory_presence * memory_weight).clamp(
                MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS,
                MUSIC_STAGE_PRESENCE_MAX_SECONDS,
            )
        } else {
            target_seconds
        };
        let target_start = (protected_end - target_seconds).clamp(0.0, protected_end);
        let memory_start = memory.as_ref().and_then(|entry| {
            entry.start_offset_seconds.is_finite().then(|| {
                (start_seconds + entry.start_offset_seconds).clamp(
                    0.0,
                    (protected_end - MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS).max(0.0),
                )
            })
        });
        let mut balanced_start = memory_start
            .map(|memory_start| target_start * (1.0 - memory_weight) + memory_start * memory_weight)
            .unwrap_or(target_start);

        // Prefer a real mix-in point near the computed head.  We only move the
        // head; the highlight tail remains protected so the final hook / phrase
        // landing is not sacrificed for presence balancing.  v10.12.54 applies
        // Cue Memory only as a soft head bias so learned comfort does not cut
        // the tail.
        if let Some(point) = best_vocal_safe_mix_point_near(
            &manifest.mix_points.mix_in,
            balanced_start,
            MUSIC_STAGE_HIGHLIGHT_HEAD_SNAP_WINDOW_SECONDS,
        ) {
            if point.time_seconds <= protected_end - MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
                balanced_start = point.time_seconds.clamp(0.0, protected_end);
            }
        }

        if protected_end <= balanced_start + MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS {
            balanced_start = (protected_end - MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS).max(0.0);
        }

        (balanced_start.min(protected_end), protected_end)
    }

    fn music_stage_cue_memory_entry_for_item(
        &self,
        item_id: QueueItemId,
        manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    ) -> Option<MusicStageCueMemoryEntry> {
        let key = self.music_stage_cue_memory_key_for_item(item_id, manifest)?;
        self.music.music_stage_cue_memory.entries.get(&key).cloned()
    }

    fn music_stage_presence_target_seconds_for_item(
        &self,
        _item_id: QueueItemId,
        manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    ) -> f64 {
        music_segment_selector::presence_target_seconds(
            manifest.duration_seconds,
            self.music.music_stage_presence_recent_seconds,
            self.music.music_stage_presence_last_seconds,
            self.music.music_stage_presence_short_run,
        )
    }

    fn music_automix_entry_start_for_item(
        &self,
        item_id: QueueItemId,
        segment_start: f64,
        segment_end: f64,
        transition_seconds: f64,
    ) -> f64 {
        if self.music.music_chorus_flow_enabled {
            self.music_chorus_entry_start_for_item(
                item_id,
                segment_start,
                segment_end,
                transition_seconds,
            )
        } else {
            segment_start
        }
    }

    fn music_chorus_entry_start_for_item(
        &self,
        item_id: QueueItemId,
        highlight_start: f64,
        highlight_end: f64,
        transition_seconds: f64,
    ) -> f64 {
        let Some(manifest) = self.music_analysis_manifest_for_item(item_id) else {
            return highlight_start;
        };
        let entry_floor = (highlight_start
            - (transition_seconds.max(1.0) * MUSIC_STAGE_ENTRY_MAX_PREROLL_RATIO)
                .min(MUSIC_STAGE_ENTRY_MAX_PREROLL_SECONDS))
        .max(0.0);
        let mut entry = manifest
            .sections
            .functional_segments
            .iter()
            .filter(|segment| {
                matches!(
                    &segment.role,
                    crate::app::music_analysis::MusicFunctionalRole::PreChorus
                        | crate::app::music_analysis::MusicFunctionalRole::Chorus
                        | crate::app::music_analysis::MusicFunctionalRole::FinalChorus
                ) && segment.end_seconds >= highlight_start - 1.0
                    && segment.start_seconds >= entry_floor - 0.75
                    && segment.start_seconds <= highlight_start + transition_seconds.max(1.0)
            })
            .min_by(|a, b| {
                (a.end_seconds - highlight_start)
                    .abs()
                    .partial_cmp(&(b.end_seconds - highlight_start).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|segment| segment.start_seconds)
            .unwrap_or_else(|| {
                snap_time_to_nearest_beat(
                    highlight_start,
                    &manifest,
                    MUSIC_CHORUS_BEAT_SNAP_WINDOW_SECONDS,
                )
                .unwrap_or(highlight_start)
            });
        if let Some(mix_in) = manifest
            .mix_points
            .mix_in
            .iter()
            .filter(|point| {
                point.time_seconds >= entry_floor - 0.35
                    && point.time_seconds <= highlight_start + transition_seconds.max(1.0)
            })
            .max_by(|a, b| {
                let a_vocal_penalty = if a.vocal_safety < 0.22 { 0.06 } else { 0.0 };
                let b_vocal_penalty = if b.vocal_safety < 0.22 { 0.06 } else { 0.0 };
                let a_score = a.confidence * 0.52 + a.vocal_safety * 0.48 - a_vocal_penalty;
                let b_score = b.confidence * 0.52 + b.vocal_safety * 0.48 - b_vocal_penalty;
                a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            entry = mix_in.time_seconds;
        }
        // v10.12.66: do not let a broad/merged chorus segment pull B back to
        // the first chorus when the selected Stage Pick is a later highlight.
        // A short pre-roll is useful, but a 100s+ jump makes B feel like it is
        // entering from the wrong scene.
        entry.clamp(
            entry_floor,
            (highlight_end - MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS * 0.5).max(highlight_start),
        )
    }

    fn music_chorus_transition_seconds_for_item(&self, item_id: QueueItemId) -> f64 {
        let Some(manifest) = self.music_analysis_manifest_for_item(item_id) else {
            return MUSIC_CHORUS_TRANSITION_FALLBACK_SECONDS;
        };
        transition_seconds_from_bpm(manifest.tempo.bpm, manifest.tempo.confidence)
    }

    fn music_chorus_stream_transition_seconds_and_kind_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> (f64, MusicMixWindowKind) {
        let base = self.music_chorus_transition_seconds_between(current_item_id, next_item_id);
        if self.music_chorus_reward_long_mix_allowed_between(current_item_id, next_item_id) {
            (
                base.clamp(
                    MUSIC_CHORUS_REWARD_LONG_MIN_SECONDS,
                    MUSIC_CHORUS_REWARD_LONG_MAX_SECONDS,
                ),
                MusicMixWindowKind::RewardLong,
            )
        } else {
            (
                base.min(MUSIC_CHORUS_STANDARD_STREAM_MAX_SECONDS)
                    .max(MUSIC_CHORUS_TRANSITION_MIN_SECONDS),
                MusicMixWindowKind::Stream,
            )
        }
    }

    fn music_chorus_reward_extended_end_seconds(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
        current_end_seconds: f64,
        transition_seconds: f64,
    ) -> f64 {
        let planned_end_seconds = current_end_seconds
            + self.music_chorus_reward_tail_extension_seconds_between(
                current_item_id,
                next_item_id,
                current_end_seconds,
                transition_seconds,
            );

        // v10.12.67: if the selected payoff/highlight end has an immediate
        // energy cliff, do not let a lyrics-safe end plus Reward tail place that
        // cliff near the audible crossfade centre.  That sounds like A was cut
        // even when the gain envelope is correct.  Pull the planned handoff
        // earlier so the cliff happens late in the fade, where B is already
        // standing up.
        self.music_chorus_reward_payoff_cliff_safe_end_seconds(
            current_item_id,
            planned_end_seconds,
            transition_seconds,
        )
        .unwrap_or(planned_end_seconds)
    }

    fn music_chorus_reward_payoff_cliff_safe_end_seconds(
        &self,
        current_item_id: QueueItemId,
        planned_end_seconds: f64,
        transition_seconds: f64,
    ) -> Option<f64> {
        if !planned_end_seconds.is_finite()
            || !transition_seconds.is_finite()
            || transition_seconds <= 0.0
        {
            return None;
        }

        let (_raw_start, raw_end_seconds) = self.music_automix_range_for_item(current_item_id)?;
        if !raw_end_seconds.is_finite() || planned_end_seconds <= raw_end_seconds {
            return None;
        }

        let safe_tail_before_cliff = self.music_chorus_reward_tail_energy_safe_extension_seconds(
            current_item_id,
            raw_end_seconds,
            transition_seconds,
        )?;

        let max_after_raw_by_phase =
            transition_seconds * (1.0 - MUSIC_CHORUS_REWARD_PAYOFF_CLIFF_PHASE_MIN).clamp(0.0, 1.0);
        let max_after_raw_by_energy =
            safe_tail_before_cliff + MUSIC_CHORUS_REWARD_PAYOFF_CLIFF_EXTRA_GRACE_SECONDS;
        let max_after_raw = max_after_raw_by_phase
            .min(max_after_raw_by_energy)
            .max(MUSIC_CHORUS_REWARD_TAIL_EXTENSION_MIN_SECONDS);
        let capped_end = (raw_end_seconds + max_after_raw).min(planned_end_seconds);

        if capped_end < planned_end_seconds - 0.20 {
            Some(capped_end)
        } else {
            None
        }
    }

    fn music_chorus_reward_transition_seed_seconds_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
        base_transition_seconds: f64,
        reward_tail_room_seconds: f64,
    ) -> f64 {
        if !base_transition_seconds.is_finite() {
            return base_transition_seconds;
        }

        let tempo_fit = self
            .music_chorus_reward_long_tempo_fit_between(current_item_id, next_item_id)
            .unwrap_or(0.0);
        let cue_score = self
            .music_chorus_reward_long_cue_score_between(current_item_id, next_item_id)
            .unwrap_or(MUSIC_CHORUS_REWARD_LONG_NEUTRAL_CUE_SCORE)
            .clamp(0.0, 1.0);
        let vocal_safety = self
            .music_chorus_vocal_overlap_safety_between(current_item_id, next_item_id)
            .unwrap_or(MUSIC_CHORUS_REWARD_LONG_NEUTRAL_VOCAL_SAFETY)
            .clamp(0.0, 1.0);

        // v10.12.46: choose Reward length from the actual A→B pair.
        // The four-song test cache shows two pairs with strong tempo fit and two
        // pairs with risky tempo fit.  Strong pairs can spend most of the tail
        // runway; risky pairs must not become a 10s green lane just because the
        // cue score is decent.
        let target_ceiling = if tempo_fit >= 0.82 && cue_score >= 0.42 {
            10.0
        } else if tempo_fit >= 0.68 && cue_score >= 0.36 {
            9.2
        } else if tempo_fit >= 0.52 && vocal_safety >= 0.18 {
            8.0
        } else {
            6.4
        };
        let tail_share = if tempo_fit >= 0.72 {
            1.0
        } else if tempo_fit >= 0.52 {
            0.72
        } else if tempo_fit >= 0.36 {
            0.48
        } else {
            0.30
        };

        (base_transition_seconds + reward_tail_room_seconds.max(0.0) * tail_share)
            .min(target_ceiling)
            .clamp(
                MUSIC_CHORUS_REWARD_LONG_MIN_SECONDS,
                MUSIC_CHORUS_REWARD_LONG_MAX_SECONDS,
            )
    }

    fn music_chorus_reward_long_tempo_fit_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> Option<f32> {
        let current_tempo = self.music_chorus_stage_tempo_for_item(
            current_item_id,
            MusicChorusStageTempoRole::Outgoing,
        )?;
        let next_tempo = self
            .music_chorus_stage_tempo_for_item(next_item_id, MusicChorusStageTempoRole::Incoming)?;
        let current_bpm = current_tempo.bpm.clamp(50.0, 220.0);
        let next_bpm = next_tempo.bpm.clamp(50.0, 220.0);
        let tempo_gap = reward_long_compatible_bpm_gap_f64(current_bpm, next_bpm);
        Some((1.0 - (tempo_gap / MUSIC_CHORUS_REWARD_LONG_MAX_TEMPO_GAP)).clamp(0.0, 1.0) as f32)
    }

    fn music_chorus_reward_tail_extension_seconds_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
        current_end_seconds: f64,
        transition_seconds: f64,
    ) -> f64 {
        if !current_end_seconds.is_finite() || !transition_seconds.is_finite() {
            return 0.0;
        }
        let Some(duration) = self.music_chorus_duration_seconds_for_item(current_item_id) else {
            return 0.0;
        };
        let available_tail = duration
            - current_end_seconds
            - MUSIC_CHORUS_REWARD_TAIL_EXTENSION_SONG_END_GUARD_SECONDS;
        if available_tail < MUSIC_CHORUS_REWARD_TAIL_EXTENSION_MIN_SECONDS {
            return 0.0;
        }

        let vocal_safety = self
            .music_chorus_vocal_overlap_safety_between(current_item_id, next_item_id)
            .unwrap_or(MUSIC_CHORUS_REWARD_LONG_NEUTRAL_VOCAL_SAFETY)
            .clamp(0.0, 1.0);
        let cue_score = self
            .music_chorus_reward_long_cue_score_between(current_item_id, next_item_id)
            .unwrap_or(MUSIC_CHORUS_REWARD_LONG_NEUTRAL_CUE_SCORE)
            .clamp(0.0, 1.0);

        // Reward Long Mix should not grow only by stealing time from the front of A.
        // Spend part of the extra budget as an A-tail runway after the selected
        // highlight: B can enter at the same psychological cue, while A keeps a
        // gentle residue behind it.  Low vocal/cue safety still shortens the tail,
        // but v10.12.45 keeps the cut softer so Reward does not collapse back
        // into a normal yellow Stream Mix whenever the analyzer is conservative.
        let safety_scale = if vocal_safety < 0.18 {
            0.64
        } else if vocal_safety < 0.30 {
            0.82
        } else if vocal_safety < 0.44 {
            0.94
        } else {
            1.0
        };
        let cue_scale = if cue_score < 0.22 {
            0.78
        } else if cue_score < 0.36 {
            0.92
        } else {
            1.0
        };
        let tempo_fit = self
            .music_chorus_reward_long_tempo_fit_between(current_item_id, next_item_id)
            .unwrap_or(0.0);
        let tempo_scale = if tempo_fit >= 0.72 {
            1.0
        } else if tempo_fit >= 0.52 {
            0.82
        } else if tempo_fit >= 0.36 {
            0.58
        } else {
            0.34
        };
        let mut target_tail = (transition_seconds * 0.56).clamp(
            MUSIC_CHORUS_REWARD_TAIL_EXTENSION_MIN_SECONDS,
            MUSIC_CHORUS_REWARD_TAIL_EXTENSION_MAX_SECONDS,
        ) * safety_scale
            * cue_scale
            * tempo_scale;

        // v10.12.59: late/final highlights can be loud up to the protected
        // payoff, then dip immediately after it.  Extending Reward into that
        // post-highlight valley makes the outgoing A side sound like it was
        // pulled away, even when the gain envelope is holding correctly.  Cap
        // the extra A-tail runway when the analysis energy curve shows an
        // immediate post-payoff collapse; keep the actual highlight tail
        // protected and let the mix spend more of its runway before the payoff.
        if let Some(energy_safe_tail) = self.music_chorus_reward_tail_energy_safe_extension_seconds(
            current_item_id,
            current_end_seconds,
            transition_seconds,
        ) {
            target_tail = target_tail.min(energy_safe_tail);
        }

        target_tail.min(available_tail).max(0.0)
    }

    fn music_chorus_reward_tail_energy_safe_extension_seconds(
        &self,
        current_item_id: QueueItemId,
        current_end_seconds: f64,
        transition_seconds: f64,
    ) -> Option<f64> {
        if !current_end_seconds.is_finite() || !transition_seconds.is_finite() {
            return None;
        }
        let manifest = self.music_analysis_manifest_for_item(current_item_id)?;
        if manifest.energy_curve.len() < 8 {
            return None;
        }

        let reference_start =
            (current_end_seconds - MUSIC_CHORUS_REWARD_TAIL_ENERGY_REFERENCE_SECONDS).max(0.0);
        let mut reference_sum = 0.0_f64;
        let mut reference_count = 0_usize;
        for point in manifest.energy_curve.iter().filter(|point| {
            point.time_seconds.is_finite()
                && point.time_seconds >= reference_start
                && point.time_seconds <= current_end_seconds
        }) {
            reference_sum += f64::from(point.rms.max(0.0));
            reference_count += 1;
        }
        if reference_count < 3 {
            return None;
        }

        let reference_rms = reference_sum / reference_count as f64;
        if reference_rms < MUSIC_CHORUS_REWARD_TAIL_ENERGY_MIN_REFERENCE_RMS {
            return None;
        }
        let dip_threshold = reference_rms * MUSIC_CHORUS_REWARD_TAIL_ENERGY_DIP_RATIO;
        let search_start = current_end_seconds + MUSIC_CHORUS_REWARD_TAIL_EXTENSION_MIN_SECONDS;
        let search_end = current_end_seconds
            + MUSIC_CHORUS_REWARD_TAIL_ENERGY_LOOKAHEAD_SECONDS
                .min(MUSIC_CHORUS_REWARD_TAIL_EXTENSION_MAX_SECONDS.max(transition_seconds * 0.72));

        manifest
            .energy_curve
            .iter()
            .filter(|point| {
                point.time_seconds.is_finite()
                    && point.time_seconds >= search_start
                    && point.time_seconds <= search_end
            })
            .find(|point| f64::from(point.rms.max(0.0)) <= dip_threshold)
            .map(|point| {
                (point.time_seconds
                    - current_end_seconds
                    - MUSIC_CHORUS_REWARD_TAIL_ENERGY_DIP_GRACE_SECONDS)
                    .clamp(0.0, MUSIC_CHORUS_REWARD_TAIL_EXTENSION_MAX_SECONDS)
            })
    }

    fn music_chorus_stage_range_end_seconds_for_item(&self, item_id: QueueItemId) -> Option<f64> {
        self.music_automix_range_for_item(item_id)
            .map(|(start, end)| self.music_lyrics_safe_range_for_item(item_id, start, end).1)
    }

    fn music_chorus_duration_seconds_for_item(&self, item_id: QueueItemId) -> Option<f64> {
        self.music_analysis_manifest_for_item(item_id)
            .map(|manifest| manifest.duration_seconds)
            .or_else(|| {
                self.queue_item_by_id(item_id)
                    .and_then(|item| item.music_duration_seconds)
            })
            .filter(|duration| duration.is_finite() && *duration > 0.0)
    }

    fn music_chorus_loudness_delta_lu_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> Option<f32> {
        let current = self.music_analysis_manifest_for_item(current_item_id)?;
        let next = self.music_analysis_manifest_for_item(next_item_id)?;
        let current_lufs = current.loudness.integrated_lufs;
        let next_lufs = next.loudness.integrated_lufs;
        if !current_lufs.is_finite() || !next_lufs.is_finite() {
            return None;
        }
        Some(next_lufs - current_lufs)
    }

    fn music_chorus_harmonic_compatibility_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> Option<MusicChorusHarmonicCompatibility> {
        let current = self.music_analysis_manifest_for_item(current_item_id)?;
        let next = self.music_analysis_manifest_for_item(next_item_id)?;
        harmonic_compatibility_between(&current.harmonic, &next.harmonic)
    }

    fn music_chorus_reward_long_mix_allowed_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> bool {
        let Some(current_tempo) = self.music_chorus_stage_tempo_for_item(
            current_item_id,
            MusicChorusStageTempoRole::Outgoing,
        ) else {
            return false;
        };
        let Some(next_tempo) = self
            .music_chorus_stage_tempo_for_item(next_item_id, MusicChorusStageTempoRole::Incoming)
        else {
            return false;
        };

        // v10.12.45: make Reward Long Mix practical, and keep the green lane anchored.
        // Real idol-pop caches often carry conservative tempo confidence, while
        // the green lane is also a visual/debug clue.  Keep hard safety rails for
        // obviously unsafe pairs, but score the middle generously so Reward can
        // appear as a practical 6-10s stage lane, not a rare laboratory result.
        let pair_confidence =
            ((current_tempo.confidence + next_tempo.confidence) * 0.5).clamp(0.0, 1.0);
        if pair_confidence < MUSIC_CHORUS_REWARD_LONG_MIN_PAIR_CONFIDENCE {
            return false;
        }

        let current_bpm = current_tempo.bpm.clamp(50.0, 220.0);
        let next_bpm = next_tempo.bpm.clamp(50.0, 220.0);
        let tempo_gap = reward_long_compatible_bpm_gap_f64(current_bpm, next_bpm);
        if tempo_gap > MUSIC_CHORUS_REWARD_LONG_MAX_TEMPO_GAP {
            return false;
        }

        // v10.12.52: Cue Runway Guard.  A green Reward lane needs enough room
        // for the incoming song to be understood after the mix starts.  If the
        // selected highlight region is too narrow, keep the pair in yellow Stream
        // Mix instead of letting Reward swallow B's whole appearance.
        let Some((current_start, current_end)) = self.music_automix_range_for_item(current_item_id)
        else {
            return false;
        };
        let Some((next_start, next_end)) = self.music_automix_range_for_item(next_item_id) else {
            return false;
        };
        let current_len = current_end - current_start;
        let next_len = next_end - next_start;
        let base_transition = self
            .music_chorus_transition_seconds_between(current_item_id, next_item_id)
            .clamp(
                MUSIC_CHORUS_REWARD_LONG_MIN_SECONDS,
                MUSIC_CHORUS_REWARD_LONG_MAX_SECONDS,
            );
        let usable_runway = current_len.max(0.0).min(next_len.max(0.0));
        if usable_runway < base_transition + MUSIC_STAGE_CUE_RUNWAY_SAFETY_SECONDS {
            return false;
        }

        let vocal_safety = self
            .music_chorus_vocal_overlap_safety_between(current_item_id, next_item_id)
            .unwrap_or(MUSIC_CHORUS_REWARD_LONG_NEUTRAL_VOCAL_SAFETY);
        if vocal_safety < MUSIC_CHORUS_REWARD_LONG_MIN_VOCAL_SAFETY {
            return false;
        }

        let cue_score = self
            .music_chorus_reward_long_cue_score_between(current_item_id, next_item_id)
            .unwrap_or(MUSIC_CHORUS_REWARD_LONG_NEUTRAL_CUE_SCORE);
        if cue_score < MUSIC_CHORUS_REWARD_LONG_MIN_CUE_SCORE {
            return false;
        }
        if let Some(harmonic) =
            self.music_chorus_harmonic_compatibility_between(current_item_id, next_item_id)
        {
            // Long overlaps expose key clashes much more than short Stream Mix
            // handoffs. Keep the harmonic model as a gate/diagnostic only; do
            // not pitch-shift user audio from this lightweight key estimate.
            if harmonic.confidence >= MUSIC_CHORUS_HARMONIC_MIN_CONFIDENCE
                && harmonic.score < MUSIC_CHORUS_REWARD_LONG_MIN_HARMONIC_SCORE
            {
                return false;
            }
        }
        if let Some(delta_lu) =
            self.music_chorus_loudness_delta_lu_between(current_item_id, next_item_id)
        {
            let abs_delta = delta_lu.abs();
            if abs_delta >= MUSIC_CHORUS_REWARD_LONG_LOUDNESS_DELTA_HARD_LU {
                return false;
            }
            if abs_delta >= MUSIC_CHORUS_REWARD_LONG_LOUDNESS_DELTA_SOFT_LU
                && cue_score < 0.36
                && vocal_safety < 0.34
            {
                return false;
            }
        }

        let tempo_fit =
            (1.0 - (tempo_gap / MUSIC_CHORUS_REWARD_LONG_MAX_TEMPO_GAP)).clamp(0.0, 1.0) as f32;
        let mut reward_score = pair_confidence * 0.20
            + tempo_fit * 0.38
            + vocal_safety.clamp(0.0, 1.0) * 0.14
            + cue_score.clamp(0.0, 1.0) * 0.28;
        if let Some(harmonic) =
            self.music_chorus_harmonic_compatibility_between(current_item_id, next_item_id)
        {
            reward_score += (harmonic.score - 0.50).clamp(-0.35, 0.35) * 0.08;
        }
        if let Some(delta_lu) =
            self.music_chorus_loudness_delta_lu_between(current_item_id, next_item_id)
        {
            reward_score -= (delta_lu.abs() / 10.0).clamp(0.0, 0.18);
        }

        // v10.12.46: Reward should be visible, but not chaotic.  The supplied
        // four-song cache has two clearly Reward-safe pairs and two pairs where
        // long green lanes become confusing because the tempo fit is weak.  Let
        // modest analyzer confidence pass when tempo/cue agree, but demote weak
        // tempo-fit pairs back to yellow unless their cue/vocal data is genuinely
        // excellent.
        if tempo_fit < 0.18 {
            return false;
        }
        if tempo_fit < 0.34 && (cue_score < 0.58 || vocal_safety < 0.32) {
            return false;
        }

        reward_score >= MUSIC_CHORUS_REWARD_LONG_MIN_SCORE
            || (tempo_fit >= 0.56 && reward_score >= MUSIC_CHORUS_REWARD_LONG_MIN_SCORE - 0.055)
            || (tempo_fit >= 0.74 && cue_score >= 0.22 && vocal_safety >= 0.12)
    }

    fn music_chorus_reward_long_cue_score_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> Option<f32> {
        let current = self.music_analysis_manifest_for_item(current_item_id)?;
        let next = self.music_analysis_manifest_for_item(next_item_id)?;
        let current_target = self
            .music_automix_range_for_item(current_item_id)
            .map(|(_, end)| end)
            .or_else(|| selected_highlight_end_seconds(&current));
        let next_target = self
            .music_automix_range_for_item(next_item_id)
            .map(|(start, _)| start)
            .or_else(|| selected_highlight_start_seconds(&next));
        let out = selected_mix_out_point_for_manifest(&current, current_target)?;
        let input = selected_mix_in_point_for_manifest(&next, next_target)?;
        let cue_score = out
            .perceptual_score
            .min(input.perceptual_score)
            .max(out.phrase_grid_fit.min(input.phrase_grid_fit) * 0.86)
            .max(out.vocal_handoff_score.min(input.vocal_handoff_score) * 0.92);
        Some(cue_score.clamp(0.0, 1.0))
    }

    fn music_chorus_transition_seconds_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> f64 {
        let current = self.music_chorus_stage_tempo_for_item(
            current_item_id,
            MusicChorusStageTempoRole::Outgoing,
        );
        let next = self
            .music_chorus_stage_tempo_for_item(next_item_id, MusicChorusStageTempoRole::Incoming);

        match (current, next) {
            (Some(a), Some(b)) if a.confidence > 0.2 || b.confidence > 0.2 => {
                let a_bpm = a.bpm.clamp(50.0, 220.0);
                let b_bpm = b.bpm.clamp(50.0, 220.0);
                let grid = tempo_grid_compatibility_between(a_bpm, b_bpm);
                let average_bpm = (a_bpm + grid.adjusted_next_bpm) * 0.5;
                let relative_gap = grid.effective_gap;
                let confidence = f64::from(((a.confidence + b.confidence) * 0.5).clamp(0.0, 1.0));
                let beats = dynamic_music_chorus_transition_beats(relative_gap, confidence);
                let mut seconds = ((60.0 / average_bpm) * beats).clamp(
                    MUSIC_CHORUS_TRANSITION_MIN_SECONDS,
                    MUSIC_CHORUS_TRANSITION_MAX_SECONDS,
                );
                seconds = cap_transition_seconds_for_tempo_gap(seconds, relative_gap);
                if let Some(vocal_safety) =
                    self.music_chorus_vocal_overlap_safety_between(current_item_id, next_item_id)
                {
                    // Preserve-pitch stretch solves pitch drift, but two lead vocals
                    // overlapping are where the listener most easily feels B's beat
                    // rushing in.  Give the handoff more phrase cushion instead of
                    // forcing a short overlap; segment-length clamping below still
                    // prevents tiny sections from being swallowed.
                    if vocal_safety < 0.22 {
                        seconds = seconds.max(8.4);
                    } else if vocal_safety < 0.32 {
                        seconds = seconds.max(7.2);
                    }
                }
                seconds.max(MUSIC_CHORUS_TRANSITION_MIN_SECONDS)
            }
            (Some(a), None) => transition_seconds_from_bpm(Some(a.bpm as f32), a.confidence),
            (None, Some(b)) => transition_seconds_from_bpm(Some(b.bpm as f32), b.confidence),
            _ => MUSIC_CHORUS_TRANSITION_FALLBACK_SECONDS,
        }
    }

    fn music_chorus_tempo_confidence_for_item(&self, item_id: QueueItemId) -> f32 {
        self.music_analysis_manifest_for_item(item_id)
            .map(|manifest| manifest.tempo.confidence)
            .unwrap_or(0.0)
    }

    fn music_chorus_pair_confidence(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> f32 {
        let current = self
            .music_chorus_stage_tempo_for_item(current_item_id, MusicChorusStageTempoRole::Outgoing)
            .map(|tempo| tempo.confidence)
            .unwrap_or_else(|| self.music_chorus_tempo_confidence_for_item(current_item_id));
        let next = self
            .music_chorus_stage_tempo_for_item(next_item_id, MusicChorusStageTempoRole::Incoming)
            .map(|tempo| tempo.confidence)
            .unwrap_or_else(|| self.music_chorus_tempo_confidence_for_item(next_item_id));
        ((current + next) * 0.5).clamp(0.0, 1.0)
    }

    fn music_chorus_stage_tempo_for_item(
        &self,
        item_id: QueueItemId,
        role: MusicChorusStageTempoRole,
    ) -> Option<MusicChorusStageTempoEstimate> {
        let manifest = self.music_analysis_manifest_for_item(item_id)?;
        let target = self.music_stage_tempo_target_seconds_for_item(item_id, &manifest, role);
        stage_tempo_estimate_for_manifest(&manifest, target)
    }

    fn music_stage_tempo_target_seconds_for_item(
        &self,
        item_id: QueueItemId,
        manifest: &crate::app::music_analysis::MusicAnalysisManifest,
        role: MusicChorusStageTempoRole,
    ) -> Option<f64> {
        match role {
            MusicChorusStageTempoRole::Outgoing => {
                let target = self
                    .music_automix_range_for_item(item_id)
                    .map(|(_, end)| end)
                    .or_else(|| selected_highlight_end_seconds(manifest));
                selected_mix_out_point_for_manifest(manifest, target)
                    .map(|point| point.time_seconds)
                    .or(target)
            }
            MusicChorusStageTempoRole::Incoming => {
                let target = self
                    .music_automix_range_for_item(item_id)
                    .map(|(start, _)| start)
                    .or_else(|| selected_highlight_start_seconds(manifest));
                selected_mix_in_point_for_manifest(manifest, target)
                    .map(|point| point.time_seconds)
                    .or(target)
            }
        }
    }

    fn music_chorus_vocal_overlap_safety_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> Option<f32> {
        let current = self.music_analysis_manifest_for_item(current_item_id)?;
        let next = self.music_analysis_manifest_for_item(next_item_id)?;
        let current_target = self
            .music_automix_range_for_item(current_item_id)
            .map(|(_, end)| end)
            .or_else(|| selected_highlight_end_seconds(&current));
        let next_target = self
            .music_automix_range_for_item(next_item_id)
            .map(|(start, _)| start)
            .or_else(|| selected_highlight_start_seconds(&next));
        let out = selected_mix_out_point_for_manifest(&current, current_target);
        let input = selected_mix_in_point_for_manifest(&next, next_target);
        match (out, input) {
            (Some(out), Some(input)) => Some(out.vocal_safety.min(input.vocal_safety)),
            (Some(out), None) => Some(out.vocal_safety),
            (None, Some(input)) => Some(input.vocal_safety),
            (None, None) => None,
        }
    }

    fn music_chorus_tempo_split_between(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> MusicChorusTempoSplit {
        let Some(current_tempo) = self.music_chorus_stage_tempo_for_item(
            current_item_id,
            MusicChorusStageTempoRole::Outgoing,
        ) else {
            return MusicChorusTempoSplit::neutral();
        };
        let Some(next_tempo) = self
            .music_chorus_stage_tempo_for_item(next_item_id, MusicChorusStageTempoRole::Incoming)
        else {
            return MusicChorusTempoSplit::neutral();
        };
        if current_tempo.confidence < 0.2 || next_tempo.confidence < 0.2 {
            return MusicChorusTempoSplit::neutral();
        }

        let current_bpm = current_tempo.bpm.clamp(50.0, 220.0);
        let next_bpm = next_tempo.bpm.clamp(50.0, 220.0);
        let grid = tempo_grid_compatibility_between(current_bpm, next_bpm);
        let raw_rate = current_bpm / grid.adjusted_next_bpm.max(1.0);
        if (raw_rate - 1.0).abs() < MUSIC_CHORUS_TEMPO_MATCH_MIN_GAP {
            return MusicChorusTempoSplit::neutral();
        }

        let relative_gap = grid.effective_gap;
        let mut b_share = adaptive_incoming_tempo_share(relative_gap);
        let mut a_share = (1.0 - b_share).clamp(0.35, 0.50);

        // v10.12.50: protect the outgoing deck from audible drag-down on weak or
        // cross-ratio tempo grids.  A is already playing in the listener's head;
        // when a pair such as 笑顔→僕のヒロイン requires the outgoing side to slow
        // down on a 3:2-like grid, even a mathematically small drift can feel like
        // a sudden brake.  Keep clean 1:1 / half-double matches cooperative, but
        // let B carry most correction on complex or wider-gap grids.
        let complex_grid_ratio = grid.ratio_numerator > 1 && grid.ratio_denominator > 1;
        let outgoing_would_slow_down = raw_rate > 1.0;
        if outgoing_would_slow_down && (complex_grid_ratio || relative_gap >= 0.10) {
            a_share = a_share.min(0.04);
            b_share = b_share.max(0.96);
        } else if outgoing_would_slow_down && relative_gap >= 0.07 {
            a_share = a_share.min(0.12);
            b_share = b_share.max(0.88);
        }

        let incoming_rate = raw_rate.powf(b_share).clamp(
            MUSIC_CHORUS_B_TEMPO_MATCH_MIN_RATE,
            MUSIC_CHORUS_B_TEMPO_MATCH_MAX_RATE,
        );
        let mut outgoing_rate = raw_rate.powf(-a_share).clamp(
            MUSIC_CHORUS_A_TEMPO_MATCH_MIN_RATE,
            MUSIC_CHORUS_A_TEMPO_MATCH_MAX_RATE,
        );

        // v10.12.51: perceptual drag floor.  Even after v10.12.50, the outgoing
        // deck can still audibly sag when a complex grid asks A to slow down and
        // B opens with a strong cue.  Keep A almost continuous in those cases;
        // let the incoming preview and the crossfade/tail do the musical work.
        if outgoing_would_slow_down {
            let drag_floor = if complex_grid_ratio {
                0.997
            } else if relative_gap >= 0.10 {
                0.996
            } else if relative_gap >= 0.07 {
                0.995
            } else {
                MUSIC_CHORUS_A_TEMPO_MATCH_MIN_RATE
            };
            outgoing_rate = outgoing_rate.max(drag_floor);
        }

        // v10.12.66: A-side tempo motion is much more audible than B-side
        // preserve-pitch preparation because A is the song the listener is
        // already standing on.  For medium/wide tempo gaps, keep A almost locked
        // and let the rendered incoming deck carry the remaining correction.
        //
        // v10.12.67: half/double-time pairs such as 81→167 can look compatible
        // mathematically, yet a small A-side drift still feels like a Doppler
        // wobble during vocal Reward bridges.  Lock A even on smaller
        // half/double residual gaps; B is the prepared side, so it should carry
        // the stretch.
        let half_double_grid_ratio = (grid.ratio_numerator == 1 && grid.ratio_denominator == 2)
            || (grid.ratio_numerator == 2 && grid.ratio_denominator == 1);
        if relative_gap >= MUSIC_CHORUS_A_TEMPO_LOCK_GAP
            || (half_double_grid_ratio && relative_gap >= MUSIC_CHORUS_A_TEMPO_LOCK_HALF_DOUBLE_GAP)
        {
            let max_drift =
                if half_double_grid_ratio && relative_gap < MUSIC_CHORUS_A_TEMPO_LOCK_GAP {
                    MUSIC_CHORUS_A_TEMPO_LOCK_HALF_DOUBLE_MAX_DRIFT
                } else {
                    MUSIC_CHORUS_A_TEMPO_LOCK_MAX_DRIFT
                };
            outgoing_rate = outgoing_rate.clamp(1.0 - max_drift, 1.0 + max_drift);
        }

        MusicChorusTempoSplit {
            incoming_rate,
            outgoing_rate,
            b_share,
        }
    }

    fn music_chorus_transition_reason_with_rate(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
        transition_source_rate: f64,
        outgoing_transition_rate: f64,
        preserve_pitch: bool,
        stretch_detail: Option<&str>,
    ) -> String {
        let current = self.music_analysis_manifest_for_item(current_item_id);
        let next = self.music_analysis_manifest_for_item(next_item_id);
        let current_stage_tempo = self.music_chorus_stage_tempo_for_item(
            current_item_id,
            MusicChorusStageTempoRole::Outgoing,
        );
        let next_stage_tempo = self
            .music_chorus_stage_tempo_for_item(next_item_id, MusicChorusStageTempoRole::Incoming);
        let tempo_text = stage_tempo_reason_text(current_stage_tempo, next_stage_tempo);
        let audit_text = self.music_chorus_pair_audit_text(current_item_id, next_item_id);
        let current_out = current.as_ref().and_then(|manifest| {
            let target = self
                .music_automix_range_for_item(current_item_id)
                .map(|(_, end)| end)
                .or_else(|| selected_highlight_end_seconds(manifest));
            selected_mix_out_point_for_manifest(manifest, target)
        });
        let next_in = next.as_ref().and_then(|manifest| {
            let target = self
                .music_automix_range_for_item(next_item_id)
                .map(|(start, _)| start)
                .or_else(|| selected_highlight_start_seconds(manifest));
            selected_mix_in_point_for_manifest(manifest, target)
        });
        let cue_text = match (current_out.as_ref(), next_in.as_ref()) {
            (Some(out), Some(input)) => {
                let pair_relay = out.vocal_handoff_score.min(input.vocal_handoff_score);
                let pair_magic = out
                    .perceptual_score
                    .min(input.perceptual_score)
                    .max(out.phrase_grid_fit.min(input.phrase_grid_fit) * 0.86)
                    .max(pair_relay * 0.92);
                let cue_label =
                    if pair_magic >= 0.58 && out.vocal_safety.min(input.vocal_safety) >= 0.34 {
                        "stage-cue"
                    } else if out.vocal_safety.min(input.vocal_safety) >= 0.36 {
                        "phrase-safe"
                    } else {
                        "phrase · vocal-risk"
                    };
                format!(
                    " · {cue_label} · vocal {:.2}/{:.2} · pcue {:.2}/{:.2} · relay {:.2}/{:.2}",
                    out.vocal_safety,
                    input.vocal_safety,
                    out.perceptual_score,
                    input.perceptual_score,
                    out.vocal_handoff_score,
                    input.vocal_handoff_score
                )
            }
            (Some(out), None) => format!(" · phrase out · vocal {:.2}", out.vocal_safety),
            (None, Some(input)) => format!(" · phrase in · vocal {:.2}", input.vocal_safety),
            (None, None) => String::new(),
        };
        let vocal_overlap_text = match (current_out.as_ref(), next_in.as_ref()) {
            (Some(out), Some(input)) => {
                let min_safety = out.vocal_safety.min(input.vocal_safety);
                if min_safety < 0.22 {
                    " · vocal-overlap short"
                } else if min_safety < 0.32 {
                    " · vocal-overlap guard"
                } else {
                    ""
                }
            }
            _ => "",
        };
        if (transition_source_rate - 1.0).abs() >= MUSIC_CHORUS_TEMPO_MATCH_MIN_GAP {
            let b_percent = (transition_source_rate - 1.0) * 100.0;
            let a_percent = (outgoing_transition_rate - 1.0) * 100.0;
            let split_text = self.music_chorus_tempo_split_between(current_item_id, next_item_id);
            let b_share_percent = split_text.b_share * 100.0;
            if preserve_pitch {
                let detail = stretch_detail.unwrap_or("preserve split");
                format!(
                    "{tempo_text}{cue_text}{vocal_overlap_text}{audit_text} · Tempo split {b_share_percent:.0}%B A {a_percent:+.1}% / B {b_percent:+.1}% · {detail}"
                )
            } else {
                format!(
                    "{tempo_text}{cue_text}{vocal_overlap_text}{audit_text} · Tempo split {b_share_percent:.0}%B A {a_percent:+.1}% / B {b_percent:+.1}%"
                )
            }
        } else if let Some(detail) = stretch_detail.filter(|detail| !detail.trim().is_empty()) {
            format!("{tempo_text}{cue_text}{vocal_overlap_text}{audit_text} · {detail}")
        } else {
            format!("{tempo_text}{cue_text}{vocal_overlap_text}{audit_text}")
        }
    }

    fn music_chorus_pair_audit_text(
        &self,
        current_item_id: QueueItemId,
        next_item_id: QueueItemId,
    ) -> String {
        let current = self.music_analysis_manifest_for_item(current_item_id);
        let next = self.music_analysis_manifest_for_item(next_item_id);
        let mut parts = Vec::new();

        if let Some(harmonic) =
            self.music_chorus_harmonic_compatibility_between(current_item_id, next_item_id)
        {
            parts.push(format!(
                "key {} {:.2}/{:.2}",
                harmonic.label, harmonic.score, harmonic.confidence
            ));
        }
        if let Some(delta_lu) =
            self.music_chorus_loudness_delta_lu_between(current_item_id, next_item_id)
        {
            parts.push(format!("LU {delta_lu:+.1}"));
        }
        if let Some(bar_confidence) = current
            .as_ref()
            .and_then(|manifest| manifest.tempo.downbeat_grid.as_ref())
            .zip(
                next.as_ref()
                    .and_then(|manifest| manifest.tempo.downbeat_grid.as_ref()),
            )
            .map(|(current, next)| current.confidence.min(next.confidence))
        {
            parts.push(format!("bar {:.2}", bar_confidence));
        }
        if let Some(true_peak_db) = current
            .as_ref()
            .map(|manifest| manifest.loudness.true_peak_db)
            .zip(next.as_ref().map(|manifest| manifest.loudness.true_peak_db))
            .map(|(current, next)| current.max(next))
            .filter(|value| value.is_finite())
        {
            parts.push(format!("TP {:.1}dB", true_peak_db));
        }

        if parts.is_empty() {
            String::new()
        } else {
            // Machine-readable enough for future Codex handoff, short enough to
            // stay inside the existing Stage Mix reason line.
            format!(" · audit {}", parts.join(" "))
        }
    }

    fn music_analysis_manifest_for_item(
        &self,
        item_id: QueueItemId,
    ) -> Option<crate::app::music_analysis::MusicAnalysisManifest> {
        let item = self.queue_item_by_id(item_id)?;
        let cache_key = item.music_cache_key.trim();
        if cache_key.is_empty() {
            return None;
        }

        let analysis_path = self
            .music_stream_cache_root()
            .join(sanitize_music_cache_key(cache_key))
            .join("analysis.yaml");
        let media_file_size = self
            .complete_music_cache_media_path(item)
            .and_then(|path| fs::metadata(path).ok())
            .map(|metadata| metadata.len());
        let modified = fs::metadata(&analysis_path)
            .and_then(|metadata| metadata.modified())
            .ok();
        let cache_id = analysis_path.to_string_lossy().into_owned();
        let cache = MUSIC_ANALYSIS_MANIFEST_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        if let Ok(cache) = cache.lock() {
            if let Some(cached) = cache.get(&cache_id) {
                if cached.modified == modified
                    && crate::app::music_analysis::music_analysis_manifest_is_current(
                        &cached.manifest,
                        media_file_size,
                    )
                {
                    return Some(cached.manifest.clone());
                }
            }
        }

        let manifest =
            read_yaml_file::<crate::app::music_analysis::MusicAnalysisManifest>(&analysis_path)?;
        if !crate::app::music_analysis::music_analysis_manifest_is_current(
            &manifest,
            media_file_size,
        ) {
            return None;
        }
        if let Ok(mut cache) = cache.lock() {
            if cache.len() > 64 {
                cache.clear();
            }
            cache.insert(
                cache_id,
                CachedMusicAnalysisManifest {
                    modified,
                    manifest: manifest.clone(),
                },
            );
        }
        Some(manifest)
    }
}

fn music_stage_highlight_debug_label(index: usize) -> String {
    if index < 26 {
        ((b'A' + index as u8) as char).to_string()
    } else {
        format!("H{}", index + 1)
    }
}

fn music_stage_cue_memory_apply_weight(entry: &MusicStageCueMemoryEntry) -> Option<f64> {
    music_segment_selector::cue_memory_apply_weight(
        entry.confidence,
        entry.effective_presence_seconds,
    )
}

fn music_stage_pick_candidate_score(
    candidate: &crate::app::music_analysis::MusicSectionCandidate,
) -> f64 {
    music_segment_selector::highlight_candidate_score(candidate)
}

fn music_stage_provisional_highlight_range_for_duration(
    duration_seconds: f64,
) -> Option<(f64, f64)> {
    music_segment_selector::provisional_highlight_segment(duration_seconds)
        .map(|segment| segment.as_range())
}

fn music_stage_map_span_is_runtime_eligible(
    span: &crate::app::music_analysis::MusicMapSpan,
    candidate: &crate::app::music_analysis::MusicSectionCandidate,
    duration_seconds: f64,
) -> bool {
    music_segment_selector::map_span_is_runtime_eligible(span, candidate, duration_seconds)
}

fn music_stage_map_span_runtime_reject_reason(
    span: &crate::app::music_analysis::MusicMapSpan,
    candidate: &crate::app::music_analysis::MusicSectionCandidate,
    duration_seconds: f64,
) -> Option<&'static str> {
    music_segment_selector::map_span_runtime_reject_reason(span, candidate, duration_seconds)
}

fn music_stage_map_span_candidate_overlap_ratio(
    span: &crate::app::music_analysis::MusicMapSpan,
    candidate: &crate::app::music_analysis::MusicSectionCandidate,
    duration_seconds: f64,
) -> f64 {
    music_segment_selector::map_span_candidate_overlap_ratio(span, candidate, duration_seconds)
}

fn music_stage_map_span_runtime_score(
    span: &crate::app::music_analysis::MusicMapSpan,
    candidate: &crate::app::music_analysis::MusicSectionCandidate,
    duration_seconds: f64,
) -> f64 {
    music_segment_selector::map_span_runtime_score(span, candidate, duration_seconds)
}

fn music_stage_pick_seed(item_id: QueueItemId, serial: u64, session_id: u64) -> u64 {
    music_segment_selector::stable_pick_seed(item_id, serial, session_id)
}

fn format_optional_score(score: Option<f32>) -> String {
    score
        .filter(|score| score.is_finite())
        .map(|score| format!("{score:.2}"))
        .unwrap_or_else(|| "n/a".to_owned())
}

fn music_attention_runtime_log_once(key: String) -> bool {
    static KEYS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    let Ok(mut keys) = KEYS.get_or_init(|| Mutex::new(Vec::new())).lock() else {
        return true;
    };
    if keys.iter().any(|existing| existing == &key) {
        return false;
    }
    if keys.len() >= 512 {
        keys.remove(0);
    }
    keys.push(key);
    true
}

fn active_state_is_normal_for_segment(
    active_segment: &Option<MusicChorusFlowSegment>,
    control: &MusicPlaybackControl,
    fallback_segment: &MusicChorusFlowSegment,
) -> bool {
    active_segment
        .as_ref()
        .filter(|active| {
            active.item_id == control.item_id && active.session_id == control.session_id
        })
        .map(|active| active.fallback_stage == MusicChorusFallbackStage::Normal)
        .unwrap_or(fallback_segment.fallback_stage == MusicChorusFallbackStage::Normal)
}

fn selected_highlight_start_seconds(
    manifest: &crate::app::music_analysis::MusicAnalysisManifest,
) -> Option<f64> {
    music_segment_selector::best_highlight_candidate(manifest)
        .map(|candidate| candidate.start_seconds)
}

fn selected_highlight_end_seconds(
    manifest: &crate::app::music_analysis::MusicAnalysisManifest,
) -> Option<f64> {
    music_segment_selector::best_highlight_candidate(manifest)
        .map(|candidate| candidate.end_seconds)
}

fn selected_mix_in_point_for_manifest(
    manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    target_seconds: Option<f64>,
) -> Option<crate::app::music_analysis::MusicMixPoint> {
    music_segment_selector::selected_mix_in_point_for_manifest(manifest, target_seconds)
}

fn selected_mix_out_point_for_manifest(
    manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    target_seconds: Option<f64>,
) -> Option<crate::app::music_analysis::MusicMixPoint> {
    music_segment_selector::selected_mix_out_point_for_manifest(manifest, target_seconds)
}

fn best_vocal_safe_mix_point_near<'a>(
    points: &'a [crate::app::music_analysis::MusicMixPoint],
    target_seconds: f64,
    window_seconds: f64,
) -> Option<&'a crate::app::music_analysis::MusicMixPoint> {
    music_segment_selector::best_vocal_safe_mix_point_near(points, target_seconds, window_seconds)
}

fn stage_tempo_estimate_for_manifest(
    manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    target_seconds: Option<f64>,
) -> Option<MusicChorusStageTempoEstimate> {
    let global_bpm = f64::from(manifest.tempo.bpm?).clamp(50.0, 220.0);
    let global_confidence = manifest.tempo.confidence.clamp(0.0, 1.0);
    let global = MusicChorusStageTempoEstimate {
        bpm: global_bpm,
        confidence: global_confidence,
        local: false,
    };
    let Some(target_seconds) = target_seconds else {
        return Some(global);
    };
    if manifest.tempo.tempo_map.is_empty() || !target_seconds.is_finite() {
        return Some(global);
    }

    let mut weighted_bpm = 0.0;
    let mut weighted_confidence = 0.0;
    let mut weight_total = 0.0;
    for point in &manifest.tempo.tempo_map {
        let Some(local_bpm) = point.bpm else {
            continue;
        };
        if !local_bpm.is_finite() || point.confidence <= 0.02 {
            continue;
        }
        let center = point.center_seconds;
        if !center.is_finite() {
            continue;
        }
        let inside_window =
            target_seconds >= point.start_seconds && target_seconds <= point.end_seconds;
        let distance = (center - target_seconds).abs();
        if !inside_window && distance > MUSIC_STAGE_LOCAL_TEMPO_RADIUS_SECONDS {
            continue;
        }
        let distance_weight = if inside_window {
            1.0
        } else {
            (1.0 - distance / MUSIC_STAGE_LOCAL_TEMPO_RADIUS_SECONDS).clamp(0.0, 1.0)
        };
        if distance_weight <= 0.0 {
            continue;
        }
        let stable_weight = if point.stable { 1.0 } else { 0.56 };
        let confidence = point.confidence.clamp(0.0, 1.0);
        let weight =
            (0.16 + f64::from(confidence)) * distance_weight * distance_weight * stable_weight;
        if weight <= 0.0 {
            continue;
        }
        let normalized_bpm =
            normalize_bpm_to_reference(f64::from(local_bpm), global_bpm).clamp(50.0, 220.0);
        weighted_bpm += normalized_bpm * weight;
        weighted_confidence += f64::from(confidence) * weight;
        weight_total += weight;
    }

    if weight_total <= 0.0001 {
        return Some(global);
    }

    let local_bpm = (weighted_bpm / weight_total).clamp(50.0, 220.0);
    let mut local_confidence = (weighted_confidence / weight_total).clamp(0.0, 1.0) as f32;
    let coverage_bonus = (weight_total / (weight_total + 1.4)).clamp(0.0, 1.0) as f32;
    local_confidence = (local_confidence * 0.72 + coverage_bonus * 0.28).clamp(0.0, 1.0);
    let local_gap = relative_bpm_gap_f64(local_bpm, global_bpm);

    if local_confidence < MUSIC_STAGE_LOCAL_TEMPO_MIN_CONFIDENCE || local_gap > 0.30 {
        return Some(global);
    }

    if local_confidence >= MUSIC_STAGE_LOCAL_TEMPO_STRONG_CONFIDENCE && local_gap <= 0.18 {
        return Some(MusicChorusStageTempoEstimate {
            bpm: local_bpm,
            confidence: local_confidence.max(global_confidence * 0.72),
            local: true,
        });
    }

    let blend = if local_confidence >= MUSIC_STAGE_LOCAL_TEMPO_BLEND_CONFIDENCE && local_gap <= 0.22
    {
        0.64
    } else if local_gap <= 0.16 {
        0.42
    } else {
        0.0
    };
    if blend <= 0.0 {
        return Some(global);
    }

    Some(MusicChorusStageTempoEstimate {
        bpm: (local_bpm * blend + global_bpm * (1.0 - blend)).clamp(50.0, 220.0),
        confidence: (local_confidence * blend as f32 + global_confidence * (1.0 - blend as f32))
            .clamp(0.0, 1.0),
        local: true,
    })
}

fn normalize_bpm_to_reference(bpm: f64, reference_bpm: f64) -> f64 {
    tempo_grid_compatibility_between(reference_bpm, bpm).adjusted_next_bpm
}

fn reward_long_compatible_bpm_gap_f64(a: f64, b: f64) -> f64 {
    tempo_grid_compatibility_between(a, b).effective_gap
}

fn tempo_grid_compatibility_between(a_bpm: f64, b_bpm: f64) -> MusicTempoGridCompatibility {
    let a = a_bpm.clamp(50.0, 220.0);
    let b = b_bpm.clamp(50.0, 220.0);
    let mut best = MusicTempoGridCompatibility {
        adjusted_next_bpm: b,
        relative_gap: relative_bpm_gap_f64(a, b),
        effective_gap: relative_bpm_gap_f64(a, b),
        ratio_numerator: 1,
        ratio_denominator: 1,
    };

    // v10.12.49: tempo compatibility is not only raw BPM or half/double-time.
    // A long mix can be musical when a small number of outgoing beats lands on
    // a small number of incoming beats.  Keep the ratio table intentionally
    // small: 1:1, half/double-time, and common simple cross-rhythmic grids.
    // Higher-complexity ratios get a small penalty so they can help borderline
    // pairs without overpowering a clean 1:1 or half/double match.
    const RATIO_CANDIDATES: &[(u32, u32)] = &[
        (1, 1),
        (1, 2),
        (2, 1),
        (2, 3),
        (3, 2),
        (3, 4),
        (4, 3),
        (4, 5),
        (5, 4),
    ];

    for &(numerator, denominator) in RATIO_CANDIDATES {
        let ratio = numerator as f64 / denominator as f64;
        let adjusted_next = b * ratio;
        if !adjusted_next.is_finite() || adjusted_next < 50.0 || adjusted_next > 220.0 {
            continue;
        }
        let relative_gap = relative_bpm_gap_f64(a, adjusted_next);
        let complexity = (numerator + denominator).saturating_sub(2) as f64;
        let penalty = if numerator == denominator {
            0.0
        } else if numerator == 1 || denominator == 1 {
            MUSIC_TEMPO_GRID_COMPLEX_RATIO_PENALTY * 0.55
        } else {
            MUSIC_TEMPO_GRID_COMPLEX_RATIO_PENALTY * complexity.min(6.0) / 3.0
        };
        let effective_gap = (relative_gap + penalty).clamp(0.0, 1.0);
        let best_complexity = best.ratio_numerator + best.ratio_denominator;
        let candidate_complexity = numerator + denominator;
        let is_better = effective_gap + 0.000_001 < best.effective_gap
            || ((effective_gap - best.effective_gap).abs() <= 0.000_001
                && candidate_complexity < best_complexity);
        if is_better {
            best = MusicTempoGridCompatibility {
                adjusted_next_bpm: adjusted_next,
                relative_gap,
                effective_gap,
                ratio_numerator: numerator,
                ratio_denominator: denominator,
            };
        }
    }

    if best.effective_gap > MUSIC_TEMPO_GRID_RATIO_MAX_EFFECTIVE_GAP {
        MusicTempoGridCompatibility {
            adjusted_next_bpm: b,
            relative_gap: relative_bpm_gap_f64(a, b),
            effective_gap: relative_bpm_gap_f64(a, b),
            ratio_numerator: 1,
            ratio_denominator: 1,
        }
    } else {
        best
    }
}

fn relative_bpm_gap_f64(a: f64, b: f64) -> f64 {
    ((a - b).abs() / ((a + b).abs() * 0.5).max(1.0)).clamp(0.0, 1.0)
}

fn average_energy_curve_rms(
    manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    start_seconds: f64,
    end_seconds: f64,
) -> Option<f32> {
    let start_seconds = start_seconds.max(0.0);
    let end_seconds = end_seconds.max(start_seconds);
    if end_seconds <= start_seconds {
        return None;
    }
    let mut sum = 0.0_f64;
    let mut count = 0_usize;
    for point in manifest.energy_curve.iter() {
        if point.time_seconds >= start_seconds && point.time_seconds <= end_seconds {
            sum += f64::from(point.rms.max(0.0));
            count = count.saturating_add(1);
        }
    }
    if count == 0 {
        None
    } else {
        Some((sum / count as f64) as f32)
    }
}

fn energy_diag_db(value: f32) -> f32 {
    20.0 * value.max(1.0e-6).log10()
}

fn stage_tempo_reason_text(
    current: Option<MusicChorusStageTempoEstimate>,
    next: Option<MusicChorusStageTempoEstimate>,
) -> String {
    match (current, next) {
        (Some(a), Some(b)) => {
            let label = if a.local || b.local {
                "local"
            } else {
                "global"
            };
            format!("{label} {:.0}→{:.0} BPM", a.bpm, b.bpm)
        }
        (Some(a), None) => {
            let label = if a.local { "local" } else { "global" };
            format!("{label} {:.0} BPM outgoing", a.bpm)
        }
        (None, Some(b)) => {
            let label = if b.local { "local" } else { "global" };
            format!("{label} {:.0} BPM incoming", b.bpm)
        }
        (None, None) => "energy fallback".to_owned(),
    }
}

fn transition_seconds_from_bpm(bpm: Option<f32>, confidence: f32) -> f64 {
    let Some(bpm) = bpm else {
        return MUSIC_CHORUS_TRANSITION_FALLBACK_SECONDS;
    };
    if confidence <= 0.12 {
        return MUSIC_CHORUS_TRANSITION_FALLBACK_SECONDS;
    }
    let beat_seconds = 60.0 / f64::from(bpm).clamp(50.0, 220.0);
    let beats = if confidence >= 0.72 {
        MUSIC_CHORUS_TRANSITION_MAX_BEATS
    } else if confidence >= 0.42 {
        14.0
    } else {
        MUSIC_CHORUS_TRANSITION_MIN_BEATS
    };
    (beat_seconds * beats).clamp(
        MUSIC_CHORUS_TRANSITION_MIN_SECONDS,
        MUSIC_CHORUS_TRANSITION_MAX_SECONDS,
    )
}

fn adaptive_incoming_tempo_share(relative_gap: f64) -> f64 {
    // Stage Mix now has perceptual cue selection and the unified [mix] capsule,
    // so tempo matching no longer needs to make B carry nearly all correction.
    // Split the motion around the psychological cue: B still gets a slightly
    // larger share because it is preserve-pitch rendered, while A only receives
    // a short eased drift during the overlap.
    if relative_gap <= 0.035 {
        0.50
    } else if relative_gap <= 0.075 {
        0.52
    } else if relative_gap <= 0.125 {
        0.55
    } else if relative_gap <= 0.18 {
        0.58
    } else {
        0.60
    }
}

fn cap_transition_seconds_for_tempo_gap(seconds: f64, relative_gap: f64) -> f64 {
    // Strong tempo gaps need a runway, not a harder stretch.  Keep an upper
    // bound so Stage Mix does not drag, but stop forcing the shortest mixes on
    // exactly the pairs where B would otherwise feel like it is sprinting.
    let capped = if relative_gap <= 0.035 {
        seconds
    } else if relative_gap <= 0.075 {
        seconds.min(10.8)
    } else if relative_gap <= 0.125 {
        seconds.min(10.0)
    } else if relative_gap <= 0.18 {
        seconds.min(8.8)
    } else {
        seconds.min(7.6)
    };
    capped.clamp(
        MUSIC_CHORUS_TRANSITION_MIN_SECONDS,
        MUSIC_CHORUS_TRANSITION_MAX_SECONDS,
    )
}

fn dynamic_music_chorus_transition_beats(relative_gap: f64, confidence: f64) -> f64 {
    let mut beats: f64 = if relative_gap <= 0.035 {
        20.0
    } else if relative_gap <= 0.075 {
        18.0
    } else if relative_gap <= 0.125 {
        16.0
    } else if relative_gap <= 0.18 {
        14.0
    } else {
        12.0
    };
    if confidence < 0.32 {
        beats = beats.min(12.0);
    } else if confidence < 0.55 {
        beats = beats.min(14.0);
    }
    beats.clamp(
        MUSIC_CHORUS_TRANSITION_MIN_BEATS,
        MUSIC_CHORUS_TRANSITION_MAX_BEATS,
    )
}

fn clamp_music_chorus_transition_seconds(seconds: f64, current_len: f64, next_len: f64) -> f64 {
    let usable_len = current_len.max(0.0).min(next_len.max(0.0));

    // Stream Mix is the main Stage Mix lane.  Normal material still keeps the
    // established 5s+ runway, but short highlights / tail handoffs may shrink
    // down near a 1–2 beat micro handoff instead of falling straight to Plain Mix.
    if usable_len <= MUSIC_CHORUS_SHORT_TAIL_SEGMENT_SECONDS * 2.5 {
        let max_by_segments = (usable_len * 0.72).clamp(
            MUSIC_CHORUS_STREAM_MIX_MIN_SECONDS,
            MUSIC_CHORUS_STREAM_MIX_COMPACT_MAX_SECONDS,
        );
        let floor = (if usable_len >= MUSIC_CHORUS_STREAM_MIX_COMPACT_MAX_SECONDS {
            MUSIC_CHORUS_STREAM_MIX_IDEAL_MIN_SECONDS
        } else {
            MUSIC_CHORUS_STREAM_MIX_MIN_SECONDS
        })
        .min(max_by_segments);
        return seconds
            .min(max_by_segments)
            .clamp(floor, max_by_segments)
            .max(MUSIC_CHORUS_STREAM_MIX_MIN_SECONDS);
    }

    let max_by_segments = (usable_len * 0.62).max(MUSIC_CHORUS_TRANSITION_MIN_SECONDS);
    seconds
        .clamp(
            MUSIC_CHORUS_TRANSITION_MIN_SECONDS,
            MUSIC_CHORUS_TRANSITION_MAX_SECONDS,
        )
        .min(max_by_segments)
        .max(MUSIC_CHORUS_TRANSITION_MIN_SECONDS)
}

fn clamp_music_chorus_mix_capsule_transition_seconds(
    seconds: f64,
    current_len: f64,
    next_len: f64,
) -> f64 {
    let usable_len = current_len.max(0.0).min(next_len.max(0.0));
    let preferred_floor = if usable_len >= MUSIC_CHORUS_MIX_CAPSULE_MAX_SECONDS {
        MUSIC_CHORUS_MIX_CAPSULE_IDEAL_MIN_SECONDS
    } else {
        MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS
    };
    let max_by_segments = if usable_len >= MUSIC_CHORUS_MIX_CAPSULE_MAX_SECONDS {
        MUSIC_CHORUS_MIX_CAPSULE_MAX_SECONDS.min(usable_len * 0.72)
    } else {
        (usable_len * 0.82).max(MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS)
    }
    .clamp(
        MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS,
        MUSIC_CHORUS_MIX_CAPSULE_MAX_SECONDS,
    );
    let floor = preferred_floor
        .min(max_by_segments)
        .max(MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS);
    seconds
        .clamp(floor, max_by_segments)
        .max(MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS)
}

fn clamp_music_stage_lite_transition_seconds(seconds: f64, current_len: f64, next_len: f64) -> f64 {
    let usable_len = current_len.max(0.0).min(next_len.max(0.0));
    let max_by_segments = if usable_len > 0.0 {
        (usable_len * 0.72).clamp(
            MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS,
            MUSIC_STAGE_LITE_TRANSITION_MAX_SECONDS,
        )
    } else {
        MUSIC_STAGE_LITE_TRANSITION_MAX_SECONDS
    };
    let floor = MUSIC_STAGE_LITE_TRANSITION_MIN_SECONDS
        .min(max_by_segments)
        .max(MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS);
    seconds
        .clamp(floor, max_by_segments)
        .max(MUSIC_CHORUS_MIX_CAPSULE_MIN_SECONDS)
}

fn locked_stage_mix_window_seconds(
    planned_segment_end: f64,
    planned_transition_seconds: f64,
    fallback_mix_window_end: f64,
) -> (f64, f64) {
    let end = if planned_segment_end.is_finite() && planned_segment_end > 0.0 {
        planned_segment_end
    } else {
        fallback_mix_window_end
    }
    .max(0.0);
    let start = (end - planned_transition_seconds.max(0.0))
        .max(0.0)
        .min(end);
    (start, end)
}

fn music_transition_cache_hold_end_seconds(
    playback_seconds: f64,
    current_end_seconds: f64,
    transition_seconds: f64,
    duration_seconds: Option<f64>,
) -> Option<f64> {
    let minimum_audible_end = playback_seconds.max(0.0)
        + transition_seconds.max(0.0)
        + MUSIC_TRANSITION_CACHE_WAIT_HOLD_SECONDS;
    let hard_end = duration_seconds
        .filter(|duration| duration.is_finite() && *duration > 0.0)
        .map(|duration| (duration - MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS).max(0.0));
    let hold_end = hard_end.map_or(current_end_seconds.max(minimum_audible_end), |hard_end| {
        current_end_seconds.max(minimum_audible_end).min(hard_end)
    });
    (hold_end > playback_seconds + transition_seconds + 0.25).then_some(hold_end)
}

fn trim_prepared_mix_leading_frames(
    samples: &mut Vec<f32>,
    channels: usize,
    frames: MusicMixFrameCount,
) -> MusicMixFrameCount {
    let channels = channels.max(1);
    let requested_frames = frames.get() as usize;
    if requested_frames == 0 || samples.is_empty() {
        return MusicMixFrameCount::ZERO;
    }
    let requested_samples = requested_frames.saturating_mul(channels);
    let removable_samples = requested_samples.min(samples.len());
    let aligned_samples = removable_samples - (removable_samples % channels);
    if aligned_samples == 0 {
        return MusicMixFrameCount::ZERO;
    }
    // Prepared Mix stores interleaved PCM. Dropping whole frames keeps channel
    // alignment and lets the rendered mix start at the actual A playback frame
    // when polling starts the handoff a callback late.
    samples.drain(0..aligned_samples);
    MusicMixFrameCount::new((aligned_samples / channels) as u64)
}

fn music_stage_chain_scale_tempo_rate(
    rate: f64,
    multiplier: f64,
    min_rate: f64,
    max_rate: f64,
) -> f64 {
    music_segment_selector::scale_tempo_rate(rate, multiplier, min_rate, max_rate)
}

fn music_stage_chain_direct_stream_director_enabled() -> bool {
    MUSIC_STAGE_CHAIN_STREAM_HANDOFF_ONLY && MUSIC_STAGE_CHAIN_DIRECT_STREAM_DIRECTOR
}

fn music_stage_chain_direct_radio_cue_enabled() -> bool {
    music_stage_chain_direct_stream_director_enabled() && MUSIC_STAGE_CHAIN_DIRECT_RADIO_CUE
}

fn music_stage_chain_stream_handoff_reason(plan_reason: &str) -> String {
    if plan_reason.contains("Stage Chain · Stream Handoff") {
        plan_reason.to_owned()
    } else if plan_reason.contains("Stage Chain · Direct Stream") {
        plan_reason.replace(
            "Stage Chain · Direct Stream",
            "Stage Chain · Stream Handoff",
        )
    } else {
        format!("Stage Chain · Stream Handoff · {plan_reason}")
    }
}

fn music_stage_chain_post_handoff_guarded_end_seconds(
    playback_start_seconds: f64,
    segment_end_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    music_segment_selector::post_handoff_guarded_end_seconds(
        playback_start_seconds,
        segment_end_seconds,
        duration_seconds,
        music_segment_selector::MusicStagePostHandoffGuardPolicy {
            advance_guard_seconds: MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS,
            post_handoff_breathe_seconds: MUSIC_STAGE_CHAIN_POST_HANDOFF_BREATHE_SECONDS,
            transition_min_seconds: MUSIC_STAGE_LITE_TRANSITION_MIN_SECONDS,
            transition_max_seconds: MUSIC_STAGE_LITE_TRANSITION_MAX_SECONDS,
        },
    )
}

fn music_stage_chain_tail_guarded_exit_end_seconds(
    playback_start_seconds: f64,
    segment_end_seconds: f64,
    transition_seconds: f64,
    duration_seconds: Option<f64>,
) -> Option<f64> {
    music_segment_selector::tail_guarded_exit_end_seconds(
        playback_start_seconds,
        segment_end_seconds,
        transition_seconds,
        duration_seconds,
        music_segment_selector::MusicStageTailExitGuardPolicy {
            exit_tail_guard_seconds: MUSIC_STAGE_CHAIN_EXIT_TAIL_GUARD_SECONDS,
            transition_min_seconds: MUSIC_STAGE_LITE_TRANSITION_MIN_SECONDS,
            advance_guard_seconds: MUSIC_CHORUS_FLOW_ADVANCE_GUARD_SECONDS,
        },
    )
}

fn music_stage_chain_direct_latest_entry_start_seconds(
    duration_seconds: f64,
    transition_seconds: f64,
) -> f64 {
    music_segment_selector::direct_latest_entry_start_seconds(
        duration_seconds,
        transition_seconds,
        music_segment_selector::MusicStageLatestEntryPolicy {
            min_remaining_seconds: MUSIC_STAGE_CHAIN_DIRECT_ENTRY_MIN_REMAINING_SECONDS,
            post_promote_min_dwell_seconds: MUSIC_STAGE_POST_PROMOTE_MIN_DWELL_SECONDS,
            extra_runway_seconds: 8.0,
        },
    )
}

fn music_stage_chain_direct_entry_anchor_score(
    segment: &crate::app::music_analysis::MusicFunctionalSegment,
    target_seconds: f64,
) -> f64 {
    music_segment_selector::direct_entry_anchor_score(segment, target_seconds)
}

fn music_stage_chain_safe_entry_start_seconds(
    entry_start_seconds: f64,
    transition_seconds: f64,
    track_duration_seconds: Option<f64>,
) -> f64 {
    music_segment_selector::safe_entry_start_seconds(
        entry_start_seconds,
        transition_seconds,
        track_duration_seconds,
        music_segment_selector::MusicStageSafeEntryPolicy {
            lite_enabled: MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX,
            direct_stream: music_stage_chain_direct_stream_director_enabled(),
            direct_min_remaining_seconds: MUSIC_STAGE_CHAIN_DIRECT_ENTRY_MIN_REMAINING_SECONDS,
            fallback_min_remaining_seconds: MUSIC_STAGE_CHAIN_ENTRY_PULLBACK_MIN_REMAINING_SECONDS,
            direct_song_share: MUSIC_STAGE_CHAIN_DIRECT_ENTRY_SONG_SHARE,
            fallback_song_share: MUSIC_STAGE_CHAIN_ENTRY_PULLBACK_SONG_SHARE,
            promoted_deck_target_seconds: MUSIC_STAGE_LITE_PROMOTED_DECK_TARGET_SECONDS,
            post_promote_min_dwell_seconds: MUSIC_STAGE_POST_PROMOTE_MIN_DWELL_SECONDS,
            extra_runway_seconds: 8.0,
        },
    )
}

fn music_stage_lite_promoted_deck_source_duration_seconds(
    entry_start_seconds: f64,
    base_source_duration_seconds: f64,
    transition_seconds: f64,
    track_duration_seconds: Option<f64>,
) -> f64 {
    if !MUSIC_STAGE_LITE_CALLBACK_ONLY_MIX {
        return base_source_duration_seconds.max(transition_seconds);
    }

    let base = base_source_duration_seconds.max(transition_seconds);
    let target = base
        .max(MUSIC_STAGE_LITE_PROMOTED_DECK_TARGET_SECONDS)
        .max(MUSIC_STAGE_POST_PROMOTE_MIN_DWELL_SECONDS + transition_seconds + 12.0);

    if let Some(duration) = track_duration_seconds.filter(|duration| {
        duration.is_finite() && *duration > entry_start_seconds && entry_start_seconds >= 0.0
    }) {
        let remaining = (duration - entry_start_seconds).max(transition_seconds);
        // Stage Mix Lite promotes this decoded B deck into the main playback
        // source. Decode through the rest of short/normal songs instead of only
        // the musical highlight; otherwise the promoted deck runs dry around
        // highlight_end and the UI appears to freeze at the same timestamp.
        return remaining.max(base);
    }

    target
        .max(MUSIC_STAGE_LITE_PROMOTED_DECK_MIN_SECONDS)
        .max(base)
}

fn scale_frame_count(frames: MusicMixFrameCount, ratio: f64) -> MusicMixFrameCount {
    if frames.is_zero() || !ratio.is_finite() {
        return MusicMixFrameCount::ZERO;
    }
    MusicMixFrameCount::new(
        (frames.get() as f64 * ratio.clamp(0.0, 1.0))
            .round()
            .clamp(0.0, u64::MAX as f64) as u64,
    )
}

fn should_zero_music_chorus_volume_before_advance(fade: &MusicChorusFadeOut) -> bool {
    // Realtime/Prepared preview decks are sample-owned PCM handoffs. The deck
    // already contains B continuation audio, and promotion preserves its gain.
    // Zeroing the whole output before promotion creates a MIX->B silence window
    // if the UI thread waits on locks or runs between audio callbacks.
    !fade.crossfade_preview_started
}

fn harmonic_compatibility_between(
    current: &crate::app::music_analysis::MusicHarmonicAnalysis,
    next: &crate::app::music_analysis::MusicHarmonicAnalysis,
) -> Option<MusicChorusHarmonicCompatibility> {
    let current_key = current.key_index?;
    let next_key = next.key_index?;
    let confidence = current.confidence.min(next.confidence).clamp(0.0, 1.0);
    if confidence < MUSIC_CHORUS_HARMONIC_MIN_CONFIDENCE {
        return None;
    }
    let current_scale = current.scale.as_deref().unwrap_or("unknown");
    let next_scale = next.scale.as_deref().unwrap_or("unknown");
    let interval = (i16::from(next_key) - i16::from(current_key)).rem_euclid(12) as u8;

    let (score, relation) = if interval == 0 && current_scale == next_scale {
        (0.98, "same-key")
    } else if interval == 0 {
        (0.82, "parallel")
    } else if is_relative_major_minor(current_key, current_scale, next_key, next_scale) {
        (0.90, "relative")
    } else if matches!(interval, 5 | 7) {
        (0.74, "fifth")
    } else if matches!(interval, 2 | 10) {
        (0.58, "neighbor")
    } else if matches!(interval, 1 | 11 | 6) {
        (0.24, "clash")
    } else {
        (0.42, "distant")
    };

    let current_name = current
        .key_name
        .as_deref()
        .unwrap_or_else(|| pitch_class_name_for_stage(current_key));
    let next_name = next
        .key_name
        .as_deref()
        .unwrap_or_else(|| pitch_class_name_for_stage(next_key));
    Some(MusicChorusHarmonicCompatibility {
        score,
        confidence,
        label: format!("{relation} {current_name}->{next_name}"),
    })
}

fn is_relative_major_minor(
    current_key: u8,
    current_scale: &str,
    next_key: u8,
    next_scale: &str,
) -> bool {
    let interval = (i16::from(next_key) - i16::from(current_key)).rem_euclid(12) as u8;
    (current_scale == "major" && next_scale == "minor" && interval == 9)
        || (current_scale == "minor" && next_scale == "major" && interval == 3)
}

fn pitch_class_name_for_stage(index: u8) -> &'static str {
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

fn snap_time_to_nearest_beat(
    time_seconds: f64,
    manifest: &crate::app::music_analysis::MusicAnalysisManifest,
    window_seconds: f64,
) -> Option<f64> {
    let grid = manifest.tempo.beat_grid.as_ref()?;
    if grid.interval_seconds <= 0.0 || !time_seconds.is_finite() {
        return None;
    }
    let steps = ((time_seconds - grid.first_beat_seconds) / grid.interval_seconds).round();
    let snapped = grid.first_beat_seconds + steps * grid.interval_seconds;
    if (snapped - time_seconds).abs() <= window_seconds {
        Some(snapped.clamp(0.0, manifest.duration_seconds.max(0.0)))
    } else {
        None
    }
}

#[cfg(test)]
mod music_stage_presence_tests {
    use super::*;

    #[test]
    fn mix_mode_is_one_complete_axis_without_invalid_combinations() {
        for mode in MusicMixMode::ALL {
            let (automix, trim, highlight) = music_mix_flags_for_mode(mode);
            assert_eq!(music_mix_mode_from_flags(automix, trim, highlight), mode);
            assert!(!(trim && highlight));
            assert_eq!(automix, mode.enabled());
        }
    }

    #[test]
    fn full_song_playback_range_keeps_the_physical_song_end() {
        assert_eq!(music_full_song_playback_range(246.75), Some((0.0, 246.75)));
        assert_eq!(music_full_song_playback_range(f64::NAN), None);
        assert_eq!(
            music_full_song_playback_range(MUSIC_CHORUS_FLOW_MIN_SEGMENT_SECONDS - 0.01),
            None
        );
    }

    #[test]
    fn full_song_display_range_ignores_runtime_handoff_segments() {
        let full_range = Some((0.0, 246.75));
        let runtime_segment = Some((118.0, 246.75));

        assert_eq!(
            music_display_playback_range(MusicMixMode::FullSong, full_range, runtime_segment,),
            full_range
        );
        assert_eq!(
            music_display_playback_range(
                MusicMixMode::SkipQuietEdges,
                Some((8.0, 238.0)),
                runtime_segment,
            ),
            runtime_segment
        );
    }

    #[test]
    fn late_full_song_cue_never_seeks_running_playback_back_to_zero() {
        assert_eq!(
            music_initial_cue_start_policy(MusicMixMode::FullSong, 0.0, 4.25, None),
            (4.25, false)
        );
        assert_eq!(
            music_initial_cue_start_policy(MusicMixMode::FullSong, 0.0, 4.25, Some(0.0)),
            (0.0, true)
        );
        assert_eq!(
            music_initial_cue_start_policy(MusicMixMode::SkipQuietEdges, 8.0, 4.25, None),
            (8.0, true)
        );
    }

    #[test]
    fn full_song_mix_window_stays_at_physical_tail() {
        assert_eq!(
            music_segment_display_mix_window(
                MusicMixMode::FullSong,
                Some((0.0, 240.0)),
                0.0,
                132.0,
                8.0,
            ),
            Some((232.0, 240.0))
        );
        assert_eq!(
            music_segment_display_mix_window(
                MusicMixMode::Highlight,
                Some((40.0, 150.0)),
                64.0,
                132.0,
                8.0,
            ),
            Some((124.0, 132.0))
        );
    }

    #[test]
    fn player_aura_tracks_four_beats_and_downbeat_decay() {
        assert_eq!(
            music_player_aura_timing(10.0, 10.0, 0.5, Some(10.0), 0.8),
            Some((0, 0.0, 0.8))
        );
        assert_eq!(
            music_player_aura_timing(11.25, 10.0, 0.5, Some(10.0), 0.8),
            Some((2, 0.5, 0.0))
        );
        assert_eq!(
            music_player_aura_timing(12.0, 10.0, 0.5, Some(10.0), 0.8),
            Some((0, 0.0, 0.8))
        );
    }

    #[test]
    fn player_aura_handoff_progress_uses_audio_output_frames() {
        assert_eq!(
            music_player_aura_mix_progress(
                MusicMixOutputFrame::new(12_400),
                MusicMixOutputFrame::new(10_000),
                MusicMixFrameCount::new(4_800),
            ),
            0.5
        );
        assert_eq!(
            music_player_aura_mix_progress(
                MusicMixOutputFrame::new(20_000),
                MusicMixOutputFrame::new(10_000),
                MusicMixFrameCount::new(4_800),
            ),
            1.0
        );
    }

    #[test]
    fn player_aura_energy_interpolates_current_analysis_curve() {
        let points = vec![
            crate::app::music_analysis::MusicEnergyPoint {
                time_seconds: 0.0,
                rms: 0.04,
                peak: 0.10,
            },
            crate::app::music_analysis::MusicEnergyPoint {
                time_seconds: 10.0,
                rms: 0.16,
                peak: 0.40,
            },
        ];
        let low = music_energy_curve_value_at(&points, 0.0, 0.08);
        let middle = music_energy_curve_value_at(&points, 5.0, 0.08);
        let high = music_energy_curve_value_at(&points, 10.0, 0.08);

        assert!(low < middle);
        assert!(middle < high);
        assert_eq!(high, 1.0);
    }

    #[test]
    fn player_aura_energy_momentum_preserves_rise_and_fall_direction() {
        let rising = vec![
            crate::app::music_analysis::MusicEnergyPoint {
                time_seconds: 0.0,
                rms: 0.02,
                peak: 0.08,
            },
            crate::app::music_analysis::MusicEnergyPoint {
                time_seconds: 2.0,
                rms: 0.18,
                peak: 0.42,
            },
        ];
        let falling = rising
            .iter()
            .rev()
            .cloned()
            .enumerate()
            .map(|(index, mut point)| {
                point.time_seconds = index as f64 * 2.0;
                point
            })
            .collect::<Vec<_>>();

        assert!(music_energy_curve_momentum_at(&rising, 1.0, 0.08) > 0.0);
        assert!(music_energy_curve_momentum_at(&falling, 1.0, 0.08) < 0.0);
    }

    #[test]
    fn player_aura_curve_interpolation_keeps_analysis_values_bounded() {
        let points = vec![
            crate::app::music_analysis::MusicCurvePoint {
                time_seconds: 0.0,
                value: 0.2,
            },
            crate::app::music_analysis::MusicCurvePoint {
                time_seconds: 10.0,
                value: 0.8,
            },
        ];

        assert!((music_curve_value_at(&points, 5.0) - 0.5).abs() < 0.000_001);
        assert_eq!(music_curve_value_at(&points, -1.0), 0.2);
        assert_eq!(music_curve_value_at(&points, 20.0), 0.8);
    }

    #[test]
    fn player_aura_spectrum_interpolates_cached_frequency_bands() {
        let points = vec![
            crate::app::music_analysis::MusicSpectrumPoint {
                time_seconds: 0.0,
                bands: [0; 8],
            },
            crate::app::music_analysis::MusicSpectrumPoint {
                time_seconds: 2.0,
                bands: [255, 128, 64, 0, 0, 0, 0, 255],
            },
        ];
        let middle = music_spectrum_curve_value_at(&points, 1.0);

        assert!((middle[0] - 0.5).abs() < 0.002);
        assert!((middle[1] - (64.0 / 255.0)).abs() < 0.002);
        assert!((middle[2] - (32.0 / 255.0)).abs() < 0.002);
        assert!((middle[7] - 0.5).abs() < 0.002);
    }

    #[test]
    fn player_aura_spectrum_peak_holds_then_releases_without_runtime_state() {
        let points = vec![
            crate::app::music_analysis::MusicSpectrumPoint {
                time_seconds: 0.0,
                bands: [0; 8],
            },
            crate::app::music_analysis::MusicSpectrumPoint {
                time_seconds: 1.0,
                bands: [255, 0, 0, 0, 0, 0, 0, 0],
            },
            crate::app::music_analysis::MusicSpectrumPoint {
                time_seconds: 1.1,
                bands: [0; 8],
            },
        ];

        let held = music_spectrum_peak_hold_at(&points, 1.15);
        let releasing = music_spectrum_peak_hold_at(&points, 1.45);
        let expired = music_spectrum_peak_hold_at(&points, 1.70);

        assert_eq!(held[0], 1.0);
        assert!((0.15..0.30).contains(&releasing[0]));
        assert_eq!(expired[0], 0.0);
    }

    #[test]
    fn player_aura_chroma_signature_maps_pitch_class_onto_color_circle() {
        let mut chroma = vec![0.0_f32; 12];
        chroma[3] = 1.0;
        let harmonic = crate::app::music_analysis::MusicHarmonicAnalysis {
            key_index: Some(3),
            key_name: None,
            scale: None,
            confidence: 1.0,
            chroma,
        };
        let (hue, coherence) = music_chroma_signature(&harmonic);

        assert!((hue - 0.25).abs() < 0.000_001);
        assert!((coherence - 1.0).abs() < 0.000_001);
    }

    fn test_music_stage_map_candidate(
        start_seconds: f64,
        end_seconds: f64,
        confidence: f32,
    ) -> crate::app::music_analysis::MusicSectionCandidate {
        crate::app::music_analysis::MusicSectionCandidate {
            start_seconds,
            end_seconds,
            confidence,
            reason: "test highlight".to_owned(),
            scores: crate::app::music_analysis::MusicSectionCandidateScores {
                total: confidence,
                chorusness: 0.84,
                repetition: 0.72,
                energy: 0.64,
                boundary: 0.70,
                duration: 0.82,
                segment_wholeness: 0.76,
                perceptual: 0.68,
                structural_recurrence: 0.74,
                ..Default::default()
            },
        }
    }

    fn test_music_stage_map_span(
        start_seconds: f64,
        lift_seconds: f64,
        peak_seconds: f64,
        end_seconds: f64,
        confidence: f32,
    ) -> crate::app::music_analysis::MusicMapSpan {
        crate::app::music_analysis::MusicMapSpan {
            start_seconds,
            lift_seconds,
            peak_seconds,
            end_seconds,
            confidence,
            reason_zh: "test span".to_owned(),
            listen_from_seconds: start_seconds,
        }
    }

    fn test_manifest_with_energy_curve(
        duration_seconds: f64,
        points: &[(f64, f32)],
    ) -> crate::app::music_analysis::MusicAnalysisManifest {
        crate::app::music_analysis::MusicAnalysisManifest {
            schema_version: 1,
            analyzer_version: 21,
            media_file_size: 0,
            updated_unix_seconds: 0,
            duration_seconds,
            sample_rate: 48_000,
            channels: 2,
            loudness: crate::app::music_analysis::MusicLoudnessAnalysis {
                rms: 0.12,
                peak: 0.24,
                rms_db: -18.0,
                peak_db: -9.0,
                integrated_lufs: -17.0,
                short_term_lufs: -16.0,
                true_peak: 0.25,
                true_peak_db: -8.8,
            },
            harmonic: crate::app::music_analysis::MusicHarmonicAnalysis::default(),
            tempo: crate::app::music_analysis::MusicTempoAnalysis {
                bpm: Some(120.0),
                confidence: 0.8,
                beat_grid: None,
                downbeat_grid: None,
                tempo_map: Vec::new(),
            },
            sections: crate::app::music_analysis::MusicSectionAnalysis {
                intro: None,
                outro: None,
                highlight_candidates: Vec::new(),
                functional_segments: Vec::new(),
                segment_tempo: Vec::new(),
                structure: crate::app::music_analysis::MusicStructureAnalysis::default(),
            },
            mix_points: crate::app::music_analysis::MusicMixPointAnalysis {
                mix_in: Vec::new(),
                mix_out: Vec::new(),
            },
            music_map: crate::app::music_analysis::StageMixMusicMap::default(),
            section_curves: crate::app::music_analysis::MusicSectionCurveAnalysis {
                hop_seconds: 1.0,
                chorusness: Vec::new(),
                boundary: Vec::new(),
                boundary_candidates: Vec::new(),
                structure: crate::app::music_analysis::MusicStructureAnalysis::default(),
            },
            energy_curve: points
                .iter()
                .map(
                    |(time_seconds, rms)| crate::app::music_analysis::MusicEnergyPoint {
                        time_seconds: *time_seconds,
                        rms: *rms,
                        peak: (*rms * 1.4).min(1.0),
                    },
                )
                .collect(),
            spectrum_curve: Vec::new(),
        }
    }

    #[test]
    fn player_aura_section_color_uses_structure_not_role_name() {
        let mut manifest = test_manifest_with_energy_curve(120.0, &[(0.0, 0.12)]);
        let field = MusicPlayerAuraTrackField {
            energy: 0.8,
            boundary: 0.6,
            novelty: 0.7,
            recurrence: 0.65,
            chorusness: 0.75,
            ..Default::default()
        };
        manifest.sections.functional_segments =
            vec![crate::app::music_analysis::MusicFunctionalSegment {
                start_seconds: 20.0,
                end_seconds: 44.0,
                role: crate::app::music_analysis::MusicFunctionalRole::Verse,
                confidence: 0.85,
                reason: "test".to_owned(),
            }];
        let verse = music_player_aura_section_color(&manifest, 30.0, field);
        manifest.sections.functional_segments[0].role =
            crate::app::music_analysis::MusicFunctionalRole::Chorus;
        let chorus_label = music_player_aura_section_color(&manifest, 30.0, field);

        assert_eq!(verse, chorus_label);
        assert!(verse.1 > 0.5);
    }

    #[test]
    fn music_stage_map_span_runtime_gate_accepts_overlapping_confident_span() {
        let candidate = test_music_stage_map_candidate(30.0, 60.0, 0.82);
        let span = test_music_stage_map_span(32.0, 33.0, 44.0, 58.0, 0.76);

        assert!(music_stage_map_span_is_runtime_eligible(
            &span, &candidate, 120.0
        ));
        assert!(music_stage_map_span_candidate_overlap_ratio(&span, &candidate, 120.0) >= 0.42);
    }

    #[test]
    fn music_stage_map_span_runtime_gate_rejects_far_span() {
        let candidate = test_music_stage_map_candidate(30.0, 60.0, 0.82);
        let span = test_music_stage_map_span(72.0, 73.0, 82.0, 94.0, 0.88);

        assert!(!music_stage_map_span_is_runtime_eligible(
            &span, &candidate, 120.0
        ));
        assert_eq!(
            music_stage_map_span_runtime_reject_reason(&span, &candidate, 120.0),
            Some("peak-outside-candidate")
        );
    }

    #[test]
    fn music_stage_map_span_runtime_gate_rejects_low_score_span() {
        let candidate = test_music_stage_map_candidate(30.0, 60.0, 0.10);
        let span = test_music_stage_map_span(17.0, 18.0, 32.0, 43.0, 0.67);

        assert_eq!(
            music_stage_map_span_runtime_reject_reason(&span, &candidate, 120.0),
            Some("low-score")
        );
        assert!(!music_stage_map_span_is_runtime_eligible(
            &span, &candidate, 120.0
        ));
    }

    #[test]
    fn music_stage_map_span_runtime_score_prefers_aligned_span() {
        let candidate = test_music_stage_map_candidate(30.0, 60.0, 0.82);
        let aligned = test_music_stage_map_span(31.5, 32.0, 44.0, 59.0, 0.76);
        let loose = test_music_stage_map_span(23.0, 24.0, 31.0, 51.0, 0.76);

        assert!(
            music_stage_map_span_runtime_score(&aligned, &candidate, 120.0)
                > music_stage_map_span_runtime_score(&loose, &candidate, 120.0)
        );
    }

    #[test]
    fn stage_pick_candidate_score_prefers_complete_highlight_over_loud_spike() {
        let mut spike = test_music_stage_map_candidate(18.0, 27.0, 0.86);
        spike.scores.chorusness = 0.22;
        spike.scores.repetition = 0.18;
        spike.scores.energy = 0.98;
        spike.scores.contrast = 0.92;
        spike.scores.boundary = 0.24;
        spike.scores.duration = 0.20;
        spike.scores.segment_wholeness = 0.12;
        spike.scores.perceptual = 0.18;
        spike.scores.structural_recurrence = 0.08;

        let mut complete = test_music_stage_map_candidate(42.0, 76.0, 0.74);
        complete.scores.chorusness = 0.84;
        complete.scores.repetition = 0.72;
        complete.scores.energy = 0.58;
        complete.scores.contrast = 0.66;
        complete.scores.boundary = 0.74;
        complete.scores.duration = 0.86;
        complete.scores.segment_wholeness = 0.88;
        complete.scores.perceptual = 0.76;
        complete.scores.structural_recurrence = 0.80;

        assert!(
            music_stage_pick_candidate_score(&complete) > music_stage_pick_candidate_score(&spike)
        );
    }

    #[test]
    fn stage_pick_seed_is_stable_for_same_inputs() {
        let first = music_stage_pick_seed(42, 7, 3);
        let second = music_stage_pick_seed(42, 7, 3);
        let changed = music_stage_pick_seed(42, 8, 3);

        assert_eq!(first, second);
        assert_ne!(first, changed);
    }

    #[test]
    fn trim_head_detector_skips_only_sustained_low_energy_intro() {
        let manifest = test_manifest_with_energy_curve(
            180.0,
            &[
                (0.0, 0.015),
                (1.0, 0.015),
                (2.2, 0.016),
                (3.0, 0.12),
                (4.0, 0.13),
                (5.0, 0.12),
            ],
        );

        let head_end = AppState::music_automix_low_energy_head_end_seconds(&manifest);
        assert!(head_end.is_some_and(|seconds| (seconds - 3.0).abs() < 0.000_001));
    }

    #[test]
    fn trim_head_detector_keeps_audible_intro() {
        let manifest = test_manifest_with_energy_curve(
            180.0,
            &[
                (0.0, 0.11),
                (1.0, 0.10),
                (2.2, 0.016),
                (3.0, 0.12),
                (4.0, 0.13),
                (5.0, 0.12),
            ],
        );

        assert_eq!(
            AppState::music_automix_low_energy_head_end_seconds(&manifest),
            None
        );
    }

    #[test]
    fn direct_mix_length_fifty_percent_preserves_model_seed() {
        let state = MusicState::new(1.0, MusicPlaybackMode::Sequential);

        assert_eq!(state.music_stage_direct_mix_length, 0.50);
        assert!(
            (state.music_stage_direct_mix_length - MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_MIX_SLIDER)
                .abs()
                <= MUSIC_STAGE_CHAIN_DIRECT_ADAPTIVE_SLIDER_EPSILON
        );
        assert!(
            (AppState::music_stage_direct_mix_length_multiplier_for(0.50) - 1.0).abs() < 0.000_001
        );
        assert!(AppState::music_stage_direct_mix_length_multiplier_for(1.00) > 1.0);
        assert!(AppState::music_stage_direct_mix_length_multiplier_for(0.00) < 1.0);
    }

    #[test]
    fn provisional_highlight_rejects_too_short_tracks() {
        assert_eq!(
            music_stage_provisional_highlight_range_for_duration(12.0),
            None
        );
    }

    #[test]
    fn provisional_highlight_uses_full_range_for_short_tracks() {
        assert_eq!(
            music_stage_provisional_highlight_range_for_duration(36.0),
            Some((0.0, 36.0))
        );
    }

    #[test]
    fn provisional_highlight_targets_first_main_body_for_long_tracks() {
        let (start, end) =
            music_stage_provisional_highlight_range_for_duration(240.0).expect("range");

        assert!((68.0..=78.0).contains(&start), "start={start}");
        assert!((116.0..=126.0).contains(&end), "end={end}");
        assert!(end - start >= 34.0);
        assert!(
            240.0 - end >= music_segment_selector::PROVISIONAL_HIGHLIGHT_REMAINING_GUARD_SECONDS
        );
    }

    #[test]
    fn short_run_recovery_does_not_trigger_after_one_short_track() {
        assert_eq!(
            music_segment_selector::presence_short_run_recovery_floor(0, 74.0),
            None
        );
        assert_eq!(
            music_segment_selector::presence_short_run_recovery_floor(1, 74.0),
            None
        );
    }

    #[test]
    fn short_run_recovery_breaks_consecutive_short_valley() {
        assert_eq!(
            music_segment_selector::presence_short_run_recovery_floor(2, 74.0),
            Some(music_segment_selector::PRESENCE_TARGET_SECONDS)
        );
        assert_eq!(
            music_segment_selector::presence_short_run_recovery_floor(4, 74.0),
            Some(music_segment_selector::PRESENCE_TARGET_SECONDS + 8.0)
        );
    }

    #[test]
    fn short_run_recovery_respects_per_track_max_target() {
        assert_eq!(
            music_segment_selector::presence_short_run_recovery_floor(5, 28.0),
            Some(28.0)
        );
    }

    #[test]
    fn prepared_mix_trim_drops_whole_interleaved_frames() {
        let mut samples = vec![0.0_f32, 0.1, 1.0, 1.1, 2.0, 2.1, 3.0, 3.1, 4.0, 4.1];

        let trimmed = trim_prepared_mix_leading_frames(&mut samples, 2, MusicMixFrameCount::new(2));

        assert_eq!(trimmed, MusicMixFrameCount::new(2));
        assert_eq!(samples, vec![2.0, 2.1, 3.0, 3.1, 4.0, 4.1]);
    }

    #[test]
    fn prepared_mix_source_frame_scale_is_bounded() {
        assert_eq!(
            scale_frame_count(MusicMixFrameCount::new(1_000), 0.375),
            MusicMixFrameCount::new(375)
        );
        assert_eq!(
            scale_frame_count(MusicMixFrameCount::new(1_000), 1.4),
            MusicMixFrameCount::new(1_000)
        );
    }

    #[test]
    fn locked_mix_window_uses_planned_transition_not_late_trimmed_execution() {
        let (start, end) = locked_stage_mix_window_seconds(187.4, 8.0, 185.0);

        assert!((start - 179.4).abs() < 0.000_001);
        assert!((end - 187.4).abs() < 0.000_001);
    }

    #[test]
    fn cache_wait_hold_extends_a_without_passing_track_end() {
        assert_eq!(
            music_transition_cache_hold_end_seconds(40.0, 42.0, 2.0, Some(120.0)),
            Some(46.0)
        );

        let clamped = music_transition_cache_hold_end_seconds(40.0, 42.0, 2.0, Some(45.0)).unwrap();
        assert!((clamped - 44.88).abs() < 0.000_001);
    }

    #[test]
    fn cache_wait_hold_declines_when_a_has_no_audible_room_left() {
        assert_eq!(
            music_transition_cache_hold_end_seconds(40.0, 42.0, 2.0, Some(42.1)),
            None
        );
    }

    #[test]
    fn preview_backed_mix_promotion_does_not_zero_volume_before_advance() {
        let mut fade = MusicChorusFadeOut {
            item_id: 1,
            session_id: 1,
            execution_route: MusicStageMixExecutionRoute::PreparedSegment,
            started_output_frame: MusicMixOutputFrame::ZERO,
            duration_output_frames: MusicMixFrameCount::new(480),
            duration_seconds: 0.01,
            planned_transition_seconds: 0.01,
            executed_transition_seconds: 0.01,
            target_volume: 1.0,
            next_item_id: Some(2),
            next_start_seconds: Some(10.0),
            crossfade_preview_started: true,
            prepared_mix_started: true,
            plain_crossfade_fallback: false,
            start_playback_seconds: 20.0,
            mix_window_start_seconds: 20.0,
            mix_window_end_seconds: 20.01,
        };

        assert!(!should_zero_music_chorus_volume_before_advance(&fade));

        fade.crossfade_preview_started = false;
        fade.prepared_mix_started = false;
        assert!(should_zero_music_chorus_volume_before_advance(&fade));
    }

    #[test]
    fn harmonic_compatibility_accepts_relative_keys() {
        let current = crate::app::music_analysis::MusicHarmonicAnalysis {
            key_index: Some(0),
            key_name: Some("C major".to_owned()),
            scale: Some("major".to_owned()),
            confidence: 0.72,
            chroma: Vec::new(),
        };
        let next = crate::app::music_analysis::MusicHarmonicAnalysis {
            key_index: Some(9),
            key_name: Some("A minor".to_owned()),
            scale: Some("minor".to_owned()),
            confidence: 0.68,
            chroma: Vec::new(),
        };

        let compatibility = harmonic_compatibility_between(&current, &next).unwrap();
        assert!(compatibility.score >= 0.85);
        assert!(compatibility.label.contains("relative"));
    }

    #[test]
    fn harmonic_compatibility_flags_semitone_clash() {
        let current = crate::app::music_analysis::MusicHarmonicAnalysis {
            key_index: Some(0),
            key_name: Some("C major".to_owned()),
            scale: Some("major".to_owned()),
            confidence: 0.72,
            chroma: Vec::new(),
        };
        let next = crate::app::music_analysis::MusicHarmonicAnalysis {
            key_index: Some(1),
            key_name: Some("C# major".to_owned()),
            scale: Some("major".to_owned()),
            confidence: 0.68,
            chroma: Vec::new(),
        };

        let compatibility = harmonic_compatibility_between(&current, &next).unwrap();
        assert!(compatibility.score < MUSIC_CHORUS_REWARD_LONG_MIN_HARMONIC_SCORE);
        assert!(compatibility.label.contains("clash"));
    }
}
