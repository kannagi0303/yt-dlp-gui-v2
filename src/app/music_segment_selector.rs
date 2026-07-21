use crate::app::music_analysis::{
    MusicAnalysisManifest, MusicFunctionalRole, MusicFunctionalSegment, MusicMapSpan,
    MusicMixPoint, MusicSectionCandidate,
};
use crate::domain::QueueItemId;

pub(crate) const MIN_PLAYABLE_SEGMENT_SECONDS: f64 = 8.0;
pub(crate) const PROVISIONAL_HIGHLIGHT_REMAINING_GUARD_SECONDS: f64 = 18.0;
pub(crate) const PICK_MIN_CONFIDENCE: f32 = 0.42;
pub(crate) const PRESENCE_MIN_SECONDS: f64 = 18.0;
pub(crate) const PRESENCE_TARGET_SECONDS: f64 = 36.0;
pub(crate) const PRESENCE_MAX_SECONDS: f64 = 74.0;
pub(crate) const PRESENCE_FADE_SHARE: f64 = 0.42;

const PICK_SHORT_SPIKE_SECONDS: f64 = 16.0;
const PICK_COMPLETE_BODY_SECONDS: f64 = 24.0;
const PICK_OVERBROAD_SECONDS: f64 = 92.0;
const PICK_PRIMARY_WEIGHT_BOOST: f64 = 4.8;
const PICK_VARIANT_WEIGHT_BOOST: f64 = 0.72;
const PICK_VARIANT_SCORE_FLOOR: f32 = 0.58;
const PICK_VARIANT_CONFIDENCE_MARGIN: f32 = 0.18;
const MAP_SPAN_RUNTIME_MIN_CONFIDENCE: f32 = 0.66;
const MAP_SPAN_RUNTIME_MIN_SCORE: f64 = 0.72;
const MAP_SPAN_RUNTIME_MIN_OVERLAP: f64 = 0.42;
const MAP_SPAN_RUNTIME_PEAK_PAD_SECONDS: f64 = 2.0;
const PRESENCE_BALANCE_NUDGE_SECONDS: f64 = 8.0;
const PRESENCE_DELTA_SOFT_LIMIT_SECONDS: f64 = 13.0;
const PRESENCE_LONG_AFTER_SHORT_RELEASE_RATIO: f64 = 0.22;
const PRESENCE_AFTER_LONG_NUDGE_RATIO: f64 = 0.12;
const PRESENCE_AFTER_LONG_NUDGE_MAX_SECONDS: f64 = 5.0;
const PRESENCE_EWMA_ALPHA: f64 = 0.34;
const PRESENCE_SHORT_RUN_THRESHOLD_SECONDS: f64 = 24.0;
const PRESENCE_SHORT_RUN_RESET_SECONDS: f64 = 32.0;
const PRESENCE_SHORT_RUN_RECOVERY_STEP_SECONDS: f64 = 4.0;
const PRESENCE_SHORT_RUN_RECOVERY_MAX_SECONDS: f64 = 48.0;
const CUE_MEMORY_APPLY_MIN_CONFIDENCE: f32 = 0.18;
const CUE_MEMORY_APPLY_MAX_WEIGHT: f64 = 0.48;
const CUE_MEMORY_UPDATE_ALPHA_EARLY: f64 = 0.34;
const CUE_MEMORY_UPDATE_ALPHA_STABLE: f64 = 0.18;
const CUE_MEMORY_CONFIDENCE_GAIN: f32 = 0.14;
const CUE_MEMORY_MAX_START_OFFSET_SECONDS: f64 = 16.0;
const CUE_MEMORY_MAX_END_OFFSET_SECONDS: f64 = 4.0;
const PROVISIONAL_HIGHLIGHT_MIN_DURATION_SECONDS: f64 = 24.0;
const PROVISIONAL_HIGHLIGHT_FULL_RANGE_MAX_SECONDS: f64 = 45.0;
const PROVISIONAL_HIGHLIGHT_MIN_SECONDS: f64 = 18.0;
const PROVISIONAL_HIGHLIGHT_TARGET_SECONDS: f64 = 34.0;
const PROVISIONAL_HIGHLIGHT_MAX_SECONDS: f64 = 48.0;
const PROVISIONAL_HIGHLIGHT_START_SHARE: f64 = 0.30;
const PROVISIONAL_HIGHLIGHT_SHORT_START_SHARE: f64 = 0.25;
const PROVISIONAL_HIGHLIGHT_SHORT_LENGTH_SHARE: f64 = 0.48;
const PROVISIONAL_HIGHLIGHT_LONG_LENGTH_SHARE: f64 = 0.22;
const TRIM_HEAD_LOW_RUN_SECONDS: f64 = 2.0;
const TRIM_HEAD_AUDIBLE_RUN_SECONDS: f64 = 1.8;
const TRIM_HEAD_MAX_SECONDS: f64 = 24.0;
const ATTENTION_MIX_POINT_WINDOW_SECONDS: f64 = 5.0;
const ATTENTION_BOUNDARY_WINDOW_SECONDS: f64 = 3.0;
const ATTENTION_TRIM_DEFAULT_MIN_EDGE_SCORE: f32 = 0.46;
const ATTENTION_TRIM_DEFAULT_MAX_ATTENTION: f32 = 0.62;
const ATTENTION_TRIM_REASON_BONUS_LOW_ENERGY: f32 = 0.03;
const ATTENTION_TRIM_REASON_BONUS_INTRO_OUTRO: f32 = 0.08;
const ATTENTION_TRIM_REASON_BONUS_SILENCE: f32 = 0.16;
const ATTENTION_ZONE_MIN_SECONDS: f64 = 2.4;
const ATTENTION_ZONE_MIX_PAD_SECONDS: f64 = 2.8;
const ATTENTION_ZONE_DEFAULT_TRANSITION_SECONDS: f64 = 5.2;
const ATTENTION_ZONE_FOCUS_MIN_SCORE: f32 = 0.46;
const ATTENTION_ZONE_BUILD_MIN_SCORE: f32 = 0.30;
const ATTENTION_ZONE_MIX_MIN_SCORE: f32 = 0.36;
const ATTENTION_ZONE_FUNCTIONAL_FOCUS_MIN_CONFIDENCE: f32 = 0.52;
const ATTENTION_ZONE_FUNCTIONAL_EDGE_MIN_CONFIDENCE: f32 = 0.42;
const ATTENTION_FOCUS_RUNTIME_MIN_SCORE: f32 = 0.46;
const ATTENTION_FOCUS_RUNTIME_MIN_OVERLAP: f32 = 0.46;
const ATTENTION_FOCUS_RUNTIME_STRONG_SCORE: f32 = 0.58;
const ATTENTION_FOCUS_RUNTIME_STANDALONE_SCORE: f32 = 0.60;
const ATTENTION_FOCUS_RUNTIME_MIN_ATTENTION: f32 = 0.48;
const ATTENTION_FOCUS_RUNTIME_MIN_BODY_EVIDENCE: f32 = 0.42;
const ATTENTION_FOCUS_RUNTIME_MAP_MARGIN: f32 = 0.06;
const ATTENTION_FOCUS_RUNTIME_CANDIDATE_MARGIN: f32 = 0.04;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MusicPlayableSegmentSource {
    FullRange,
    Trim,
    HighlightAttentionFocus,
    HighlightMusicMapSpan,
    HighlightCandidate,
    HighlightQuickEstimate,
}

impl MusicPlayableSegmentSource {
    pub(crate) fn log_key(self) -> &'static str {
        match self {
            Self::FullRange => "full-range",
            Self::Trim => "trim",
            Self::HighlightAttentionFocus => "attention-focus",
            Self::HighlightMusicMapSpan => "music-map-span",
            Self::HighlightCandidate => "candidate",
            Self::HighlightQuickEstimate => "quick",
        }
    }

    pub(crate) fn from_attention_highlight_source(
        source: MusicAttentionHighlightRangeSource,
    ) -> Self {
        match source {
            MusicAttentionHighlightRangeSource::FocusZone => Self::HighlightAttentionFocus,
            MusicAttentionHighlightRangeSource::MusicMapSpan => Self::HighlightMusicMapSpan,
            MusicAttentionHighlightRangeSource::Candidate => Self::HighlightCandidate,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicPlayableSegment {
    pub(crate) start_seconds: f64,
    pub(crate) end_seconds: f64,
    pub(crate) source: MusicPlayableSegmentSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MusicQuickFocusKind {
    FullTrackShort,
    FirstMainBody,
}

impl MusicQuickFocusKind {
    pub(crate) fn log_key(self) -> &'static str {
        match self {
            Self::FullTrackShort => "full-track-short",
            Self::FirstMainBody => "first-main-body",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicQuickFocusPlan {
    pub(crate) segment: MusicPlayableSegment,
    pub(crate) confidence: f32,
    pub(crate) kind: MusicQuickFocusKind,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MusicStageHighlightPick {
    pub(crate) candidate_index: usize,
    pub(crate) start_seconds: f64,
    pub(crate) end_seconds: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct MusicStagePresenceHistory {
    pub(crate) recent_seconds: Option<f64>,
    pub(crate) last_seconds: Option<f64>,
    pub(crate) short_run: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct MusicStageCueMemoryValues {
    pub(crate) start_offset_seconds: f64,
    pub(crate) end_offset_seconds: f64,
    pub(crate) effective_presence_seconds: f64,
    pub(crate) confidence: f32,
    pub(crate) updates: u32,
    pub(crate) updated_unix_seconds: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageCueMemoryObservation {
    pub(crate) start_offset_seconds: f64,
    pub(crate) end_offset_seconds: f64,
    pub(crate) effective_presence_seconds: f64,
    pub(crate) updated_unix_seconds: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicDirectBodyHighlightPolicy {
    pub(crate) body_fence_seconds: f64,
    pub(crate) duration_seconds: f64,
    pub(crate) latest_start_seconds: f64,
    pub(crate) min_segment_seconds: f64,
    pub(crate) min_confidence: f32,
    pub(crate) tail_grace_seconds: f64,
    pub(crate) late_midpoint_share: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStagePostHandoffGuardPolicy {
    pub(crate) advance_guard_seconds: f64,
    pub(crate) post_handoff_breathe_seconds: f64,
    pub(crate) transition_min_seconds: f64,
    pub(crate) transition_max_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageTailExitGuardPolicy {
    pub(crate) exit_tail_guard_seconds: f64,
    pub(crate) transition_min_seconds: f64,
    pub(crate) advance_guard_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageLatestEntryPolicy {
    pub(crate) min_remaining_seconds: f64,
    pub(crate) post_promote_min_dwell_seconds: f64,
    pub(crate) extra_runway_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageSafeEntryPolicy {
    pub(crate) lite_enabled: bool,
    pub(crate) direct_stream: bool,
    pub(crate) direct_min_remaining_seconds: f64,
    pub(crate) fallback_min_remaining_seconds: f64,
    pub(crate) direct_song_share: f64,
    pub(crate) fallback_song_share: f64,
    pub(crate) promoted_deck_target_seconds: f64,
    pub(crate) post_promote_min_dwell_seconds: f64,
    pub(crate) extra_runway_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageBodyFencePolicy {
    pub(crate) min_segment_seconds: f64,
    pub(crate) transition_min_seconds: f64,
    pub(crate) min_remaining_seconds: f64,
    pub(crate) post_promote_min_dwell_seconds: f64,
    pub(crate) song_share: f64,
    pub(crate) outro_backoff_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageEnergyTailPolicy {
    pub(crate) min_segment_seconds: f64,
    pub(crate) relative_rms: f32,
    pub(crate) peak_rms: f32,
    pub(crate) min_rms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageBodyFenceExitPolicy {
    pub(crate) tail_grace_seconds: f64,
    pub(crate) transition_min_seconds: f64,
    pub(crate) advance_guard_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageEnergyTailExitPolicy {
    pub(crate) min_tail_seconds: f64,
    pub(crate) exit_grace_seconds: f64,
    pub(crate) transition_min_seconds: f64,
    pub(crate) advance_guard_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MusicStageTailSafeEntryReason {
    Runway,
    BodyFence,
    TailSection,
    LyricTail,
    EnergyTail,
}

impl MusicStageTailSafeEntryReason {
    pub(crate) fn log_key(self) -> &'static str {
        match self {
            Self::Runway => "runway",
            Self::BodyFence => "body-fence",
            Self::TailSection => "tail-section",
            Self::LyricTail => "lyric-tail",
            Self::EnergyTail => "energy-tail",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageTailSafeEntryPlan {
    pub(crate) start_seconds: f64,
    pub(crate) reason: MusicStageTailSafeEntryReason,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageTailSafeEntryPolicy {
    pub(crate) min_segment_seconds: f64,
    pub(crate) advance_guard_seconds: f64,
    pub(crate) min_remaining_seconds: f64,
    pub(crate) post_promote_min_dwell_seconds: f64,
    pub(crate) extra_runway_seconds: f64,
    pub(crate) tail_section_backoff_seconds: f64,
    pub(crate) trailing_silence_min_seconds: f64,
    pub(crate) last_lyric_backoff_seconds: f64,
    pub(crate) energy_tail_min_seconds: f64,
    pub(crate) energy_tail_entry_backoff_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageMixLengthMultiplierPolicy {
    pub(crate) short_multiplier: f64,
    pub(crate) long_multiplier: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageTempoBridgeStrengthPolicy {
    pub(crate) min_multiplier: f64,
    pub(crate) max_multiplier: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicStageTempoBridgeRateBoundsPolicy {
    pub(crate) incoming_soft_max_delta: f64,
    pub(crate) incoming_strong_max_delta: f64,
    pub(crate) outgoing_soft_max_delta: f64,
    pub(crate) outgoing_strong_max_delta: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MusicAttentionReasonFlags {
    pub(crate) low_energy: bool,
    pub(crate) highlight_overlap: bool,
    pub(crate) structural_role: bool,
    pub(crate) edge_role: bool,
    pub(crate) silence_role: bool,
    pub(crate) strong_boundary: bool,
    pub(crate) mix_entry: bool,
    pub(crate) mix_exit: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicAttentionProfile {
    pub(crate) start_seconds: f64,
    pub(crate) end_seconds: f64,
    pub(crate) attention_score: f32,
    pub(crate) emptiness_score: f32,
    pub(crate) edge_trim_score: f32,
    pub(crate) highlight_score: f32,
    pub(crate) structural_score: f32,
    pub(crate) energy_score: f32,
    pub(crate) boundary_strength: f32,
    pub(crate) entry_quality: f32,
    pub(crate) exit_quality: f32,
    pub(crate) reason_flags: MusicAttentionReasonFlags,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MusicAttentionTrimEdgeReason {
    IntroSection,
    OutroSection,
    SilenceSection,
    LowEnergyHead,
    LowEnergyTail,
}

impl MusicAttentionTrimEdgeReason {
    pub(crate) fn log_key(self) -> &'static str {
        match self {
            Self::IntroSection => "intro-section",
            Self::OutroSection => "outro-section",
            Self::SilenceSection => "silence-section",
            Self::LowEnergyHead => "low-energy-head",
            Self::LowEnergyTail => "low-energy-tail",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicAttentionTrimPolicy {
    pub(crate) min_segment_seconds: f64,
    pub(crate) min_transition_seconds: f64,
    pub(crate) max_head_seconds: f64,
    pub(crate) max_head_share: f64,
    pub(crate) min_edge_trim_score: f32,
    pub(crate) max_attention_score: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicAttentionTrimEdgePlan {
    pub(crate) boundary_seconds: f64,
    pub(crate) profile: MusicAttentionProfile,
    pub(crate) reason: MusicAttentionTrimEdgeReason,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicAttentionTrimRangePlan {
    pub(crate) start_seconds: f64,
    pub(crate) end_seconds: f64,
    pub(crate) head: Option<MusicAttentionTrimEdgePlan>,
    pub(crate) tail: Option<MusicAttentionTrimEdgePlan>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MusicAttentionZoneKind {
    Empty,
    Build,
    Focus,
    Entry,
    Exit,
    MixSafe,
    TailRisk,
}

impl MusicAttentionZoneKind {
    pub(crate) fn log_key(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Build => "build",
            Self::Focus => "focus",
            Self::Entry => "entry",
            Self::Exit => "exit",
            Self::MixSafe => "mix-safe",
            Self::TailRisk => "tail-risk",
        }
    }

    pub(crate) fn visual_priority(self) -> u8 {
        match self {
            Self::Empty => 10,
            Self::Build => 20,
            Self::Focus => 30,
            Self::Entry => 40,
            Self::Exit => 41,
            Self::MixSafe => 42,
            Self::TailRisk => 50,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicAttentionZone {
    pub(crate) start_seconds: f64,
    pub(crate) end_seconds: f64,
    pub(crate) kind: MusicAttentionZoneKind,
    pub(crate) score: f32,
    pub(crate) profile: MusicAttentionProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MusicAttentionHighlightRangeSource {
    FocusZone,
    MusicMapSpan,
    Candidate,
}

impl MusicAttentionHighlightRangeSource {
    pub(crate) fn log_key(self) -> &'static str {
        match self {
            Self::FocusZone => "focus-zone",
            Self::MusicMapSpan => "music-map-span",
            Self::Candidate => "candidate",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicAttentionHighlightRangePlan {
    pub(crate) start_seconds: f64,
    pub(crate) end_seconds: f64,
    pub(crate) source: MusicAttentionHighlightRangeSource,
    pub(crate) selection_confidence: f32,
    pub(crate) selection_risk: f32,
    pub(crate) selection_reason: &'static str,
    pub(crate) guarded_source: Option<MusicAttentionHighlightRangeSource>,
    pub(crate) guard_reason: Option<&'static str>,
    pub(crate) reference_start_seconds: Option<f64>,
    pub(crate) reference_end_seconds: Option<f64>,
    pub(crate) focus_score: Option<f32>,
    pub(crate) focus_runtime_score: Option<f32>,
    pub(crate) focus_overlap: Option<f32>,
    pub(crate) focus_attention_score: Option<f32>,
    pub(crate) focus_structural_score: Option<f32>,
    pub(crate) candidate_score: Option<f32>,
    pub(crate) map_lift_seconds: Option<f64>,
    pub(crate) map_peak_seconds: Option<f64>,
    pub(crate) map_confidence: Option<f32>,
    pub(crate) map_runtime_score: Option<f64>,
    pub(crate) rejected_map_reason: Option<&'static str>,
    pub(crate) rejected_map_confidence: Option<f32>,
    pub(crate) rejected_map_runtime_score: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicMapSpanRuntimeGuard {
    pub(crate) reason: &'static str,
    pub(crate) start_seconds: f64,
    pub(crate) end_seconds: f64,
    pub(crate) confidence: f32,
    pub(crate) runtime_score: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MusicFocusZoneRuntimeDecision {
    zone: MusicAttentionZone,
    runtime_score: f32,
    overlap: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MusicFocusZoneRuntimeGuard {
    zone: MusicAttentionZone,
    reason: &'static str,
    runtime_score: f32,
    overlap: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct MusicFocusZoneRuntimeSelection {
    accepted: Option<MusicFocusZoneRuntimeDecision>,
    rejected: Option<MusicFocusZoneRuntimeGuard>,
}

impl MusicPlayableSegment {
    pub(crate) fn new(
        start_seconds: f64,
        end_seconds: f64,
        source: MusicPlayableSegmentSource,
    ) -> Option<Self> {
        if !start_seconds.is_finite() || !end_seconds.is_finite() || end_seconds <= start_seconds {
            return None;
        }
        Some(Self {
            start_seconds,
            end_seconds,
            source,
        })
    }

    pub(crate) fn as_range(self) -> (f64, f64) {
        (self.start_seconds, self.end_seconds)
    }
}

pub(crate) fn default_attention_trim_policy(
    min_segment_seconds: f64,
    min_transition_seconds: f64,
) -> MusicAttentionTrimPolicy {
    MusicAttentionTrimPolicy {
        min_segment_seconds,
        min_transition_seconds,
        max_head_seconds: TRIM_HEAD_MAX_SECONDS,
        max_head_share: 0.35,
        min_edge_trim_score: ATTENTION_TRIM_DEFAULT_MIN_EDGE_SCORE,
        max_attention_score: ATTENTION_TRIM_DEFAULT_MAX_ATTENTION,
    }
}

pub(crate) fn attention_profile_for_candidate(
    candidate: &MusicSectionCandidate,
) -> MusicAttentionProfile {
    let start_seconds = candidate.start_seconds;
    let end_seconds = candidate.end_seconds.max(candidate.start_seconds);
    let highlight_score = raw_highlight_candidate_score(candidate) as f32;
    let energy_score = candidate.scores.energy.clamp(0.0, 1.0);
    let structural_score = candidate
        .scores
        .chorusness
        .max(candidate.scores.repetition)
        .max(candidate.scores.segment_wholeness)
        .max(candidate.scores.structural_recurrence)
        .max(candidate.scores.perceptual)
        .clamp(0.0, 1.0);
    let boundary_strength = candidate.scores.boundary.clamp(0.0, 1.0);
    let attention_score = highlight_score;
    let emptiness_score = ((1.0 - attention_score) * 0.58
        + (1.0 - structural_score) * 0.26
        + (1.0 - energy_score) * 0.16)
        .clamp(0.0, 1.0);
    let edge_trim_score = (emptiness_score * 0.70 + (1.0 - boundary_strength) * 0.12
        - attention_score * 0.18)
        .clamp(0.0, 1.0);

    MusicAttentionProfile {
        start_seconds,
        end_seconds,
        attention_score,
        emptiness_score,
        edge_trim_score,
        highlight_score,
        structural_score,
        energy_score,
        boundary_strength,
        entry_quality: 0.0,
        exit_quality: 0.0,
        reason_flags: MusicAttentionReasonFlags {
            low_energy: energy_score < 0.28,
            highlight_overlap: highlight_score >= 0.45,
            structural_role: structural_score >= 0.42,
            edge_role: false,
            silence_role: false,
            strong_boundary: boundary_strength >= 0.58,
            mix_entry: false,
            mix_exit: false,
        },
    }
}

pub(crate) fn attention_profile_for_range(
    manifest: &MusicAnalysisManifest,
    start_seconds: f64,
    end_seconds: f64,
) -> Option<MusicAttentionProfile> {
    let duration = manifest.duration_seconds.max(0.0);
    if !duration.is_finite() || duration <= 0.0 {
        return None;
    }
    let start = start_seconds.clamp(0.0, duration);
    let end = end_seconds.clamp(start, duration);
    if end <= start {
        return None;
    }

    let energy_score = attention_energy_score_for_range(manifest, start, end);
    let highlight_score = attention_highlight_score_for_range(manifest, start, end);
    let (structural_score, edge_role_score, silence_score) =
        attention_functional_scores_for_range(manifest, start, end);
    let boundary_strength = attention_boundary_strength_for_range(manifest, start, end);
    let entry_quality = attention_mix_quality_for_range(&manifest.mix_points.mix_in, start);
    let exit_quality = attention_mix_quality_for_range(&manifest.mix_points.mix_out, end);

    let attention_score = (highlight_score * 0.44
        + structural_score * 0.24
        + energy_score * 0.16
        + boundary_strength * 0.07
        + entry_quality * 0.045
        + exit_quality * 0.045)
        .clamp(0.0, 1.0);
    let low_energy_score = (1.0 - energy_score).clamp(0.0, 1.0);
    let low_structure_score = (1.0 - highlight_score.max(structural_score)).clamp(0.0, 1.0);
    let emptiness_score = (low_energy_score * 0.46
        + low_structure_score * 0.36
        + silence_score * 0.14
        + (1.0 - boundary_strength) * 0.04
        - attention_score * 0.13)
        .clamp(0.0, 1.0);
    let edge_trim_score = (emptiness_score * 0.50
        + edge_role_score * 0.30
        + silence_score * 0.15
        + (1.0 - highlight_score) * 0.05
        - attention_score * 0.10)
        .clamp(0.0, 1.0);

    Some(MusicAttentionProfile {
        start_seconds: start,
        end_seconds: end,
        attention_score,
        emptiness_score,
        edge_trim_score,
        highlight_score,
        structural_score,
        energy_score,
        boundary_strength,
        entry_quality,
        exit_quality,
        reason_flags: MusicAttentionReasonFlags {
            low_energy: energy_score < 0.30,
            highlight_overlap: highlight_score >= 0.36,
            structural_role: structural_score >= 0.30,
            edge_role: edge_role_score >= 0.36,
            silence_role: silence_score >= 0.35,
            strong_boundary: boundary_strength >= 0.55,
            mix_entry: entry_quality >= 0.42,
            mix_exit: exit_quality >= 0.42,
        },
    })
}

pub(crate) fn attention_trim_head_end_seconds(
    manifest: &MusicAnalysisManifest,
    policy: MusicAttentionTrimPolicy,
) -> Option<MusicAttentionTrimEdgePlan> {
    let duration = manifest.duration_seconds;
    if !duration.is_finite() || duration <= policy.min_segment_seconds {
        return None;
    }
    let max_head = attention_trim_head_limit_seconds(duration, policy);
    let mut candidates: Vec<(f64, MusicAttentionTrimEdgeReason)> = Vec::new();

    if let Some(intro) = manifest.sections.intro.as_ref() {
        candidates.push((
            intro.end_seconds,
            MusicAttentionTrimEdgeReason::IntroSection,
        ));
    }
    for segment in &manifest.sections.functional_segments {
        if segment.start_seconds <= 3.0
            && matches!(
                segment.role,
                MusicFunctionalRole::Intro | MusicFunctionalRole::Silence
            )
            && functional_edge_segment_is_trusted(segment)
        {
            let reason = if matches!(segment.role, MusicFunctionalRole::Silence) {
                MusicAttentionTrimEdgeReason::SilenceSection
            } else {
                MusicAttentionTrimEdgeReason::IntroSection
            };
            candidates.push((segment.end_seconds, reason));
        }
    }
    if let Some(head_end) = low_energy_head_end_seconds(manifest) {
        candidates.push((head_end, MusicAttentionTrimEdgeReason::LowEnergyHead));
    }

    candidates
        .into_iter()
        .filter(|(boundary, _)| {
            boundary.is_finite()
                && *boundary > 0.0
                && *boundary <= max_head
                && duration - *boundary >= policy.min_segment_seconds
        })
        .filter_map(|(boundary, reason)| {
            let profile = attention_profile_for_range(manifest, 0.0, boundary)?;
            attention_trim_edge_is_accepted(profile, reason, policy)
                .then_some((boundary, reason, profile))
        })
        .max_by(|a, b| {
            attention_trim_edge_sort_score(a.0, a.1, a.2, max_head)
                .partial_cmp(&attention_trim_edge_sort_score(b.0, b.1, b.2, max_head))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(
            |(boundary_seconds, reason, profile)| MusicAttentionTrimEdgePlan {
                boundary_seconds,
                profile,
                reason,
            },
        )
}

fn attention_trim_head_limit_seconds(duration: f64, policy: MusicAttentionTrimPolicy) -> f64 {
    if !duration.is_finite() || duration <= 0.0 {
        return 0.0;
    }
    let absolute_cap = policy.max_head_seconds.max(0.0);
    let share_cap = duration * policy.max_head_share.clamp(0.0, 0.75);
    match (absolute_cap > 0.0, share_cap > 0.0) {
        (true, true) => absolute_cap.min(share_cap),
        (true, false) => absolute_cap,
        (false, true) => share_cap,
        (false, false) => 0.0,
    }
}

pub(crate) fn attention_trim_tail_start_seconds(
    manifest: &MusicAnalysisManifest,
    policy: MusicAttentionTrimPolicy,
) -> Option<MusicAttentionTrimEdgePlan> {
    let duration = manifest.duration_seconds;
    if !duration.is_finite() || duration <= policy.min_segment_seconds {
        return None;
    }
    let mut candidates: Vec<(f64, MusicAttentionTrimEdgeReason)> = Vec::new();

    if let Some(outro) = manifest.sections.outro.as_ref() {
        candidates.push((
            outro.start_seconds,
            MusicAttentionTrimEdgeReason::OutroSection,
        ));
    }
    for segment in &manifest.sections.functional_segments {
        if matches!(
            segment.role,
            MusicFunctionalRole::Outro | MusicFunctionalRole::Silence
        ) && functional_edge_segment_is_trusted(segment)
        {
            let reason = if matches!(segment.role, MusicFunctionalRole::Silence) {
                MusicAttentionTrimEdgeReason::SilenceSection
            } else {
                MusicAttentionTrimEdgeReason::OutroSection
            };
            candidates.push((segment.start_seconds, reason));
        }
    }
    if let Some(tail_start) = low_energy_tail_start_seconds(
        manifest,
        policy.min_segment_seconds,
        policy.min_transition_seconds,
    ) {
        candidates.push((tail_start, MusicAttentionTrimEdgeReason::LowEnergyTail));
    }

    candidates
        .into_iter()
        .filter(|(boundary, _)| {
            boundary.is_finite()
                && *boundary >= policy.min_segment_seconds
                && duration - *boundary >= policy.min_transition_seconds.max(0.0)
        })
        .filter_map(|(boundary, reason)| {
            let profile = attention_profile_for_range(manifest, boundary, duration)?;
            attention_trim_edge_is_accepted(profile, reason, policy)
                .then_some((boundary, reason, profile))
        })
        .min_by(|a, b| {
            attention_trim_edge_sort_score(a.0, a.1, a.2, duration - a.0)
                .partial_cmp(&attention_trim_edge_sort_score(
                    b.0,
                    b.1,
                    b.2,
                    duration - b.0,
                ))
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse()
        })
        .map(
            |(boundary_seconds, reason, profile)| MusicAttentionTrimEdgePlan {
                boundary_seconds,
                profile,
                reason,
            },
        )
}

pub(crate) fn attention_trim_range_plan(
    manifest: &MusicAnalysisManifest,
    policy: MusicAttentionTrimPolicy,
) -> Option<MusicAttentionTrimRangePlan> {
    let duration = manifest.duration_seconds;
    if !duration.is_finite() || duration <= policy.min_segment_seconds {
        return None;
    }

    let head = attention_trim_head_end_seconds(manifest, policy);
    let tail = attention_trim_tail_start_seconds(manifest, policy);
    let start_seconds = head
        .map(|plan| plan.boundary_seconds)
        .unwrap_or(0.0)
        .clamp(0.0, duration);
    let end_seconds = tail
        .map(|plan| plan.boundary_seconds)
        .unwrap_or(duration)
        .clamp(start_seconds, duration.max(start_seconds));

    Some(MusicAttentionTrimRangePlan {
        start_seconds,
        end_seconds,
        head,
        tail,
    })
}

pub(crate) fn attention_zones_for_manifest(
    manifest: &MusicAnalysisManifest,
) -> Vec<MusicAttentionZone> {
    let duration = manifest.duration_seconds;
    if !duration.is_finite() || duration <= 0.0 {
        return Vec::new();
    }

    let mut zones = Vec::new();
    let trim_policy = default_attention_trim_policy(
        MIN_PLAYABLE_SEGMENT_SECONDS,
        ATTENTION_ZONE_DEFAULT_TRANSITION_SECONDS,
    );
    if let Some(trim_plan) = attention_trim_range_plan(manifest, trim_policy) {
        if let Some(head) = trim_plan.head {
            push_attention_zone(
                &mut zones,
                manifest,
                0.0,
                head.boundary_seconds,
                MusicAttentionZoneKind::Empty,
                head.profile
                    .edge_trim_score
                    .max(head.profile.emptiness_score),
            );
        }
        if let Some(tail) = trim_plan.tail {
            let kind = if matches!(tail.reason, MusicAttentionTrimEdgeReason::SilenceSection) {
                MusicAttentionZoneKind::Empty
            } else {
                MusicAttentionZoneKind::TailRisk
            };
            push_attention_zone(
                &mut zones,
                manifest,
                tail.boundary_seconds,
                duration,
                kind,
                tail.profile
                    .edge_trim_score
                    .max(tail.profile.emptiness_score),
            );
        }
    }

    for candidate in &manifest.sections.highlight_candidates {
        let candidate_score = highlight_candidate_score(candidate) as f32;
        if candidate_score < ATTENTION_ZONE_FOCUS_MIN_SCORE
            || candidate.end_seconds <= candidate.start_seconds + MIN_PLAYABLE_SEGMENT_SECONDS
        {
            continue;
        }
        let (start, end) = select_map_span_for_candidate(manifest, candidate)
            .map(|span| (span.start_seconds, span.end_seconds))
            .unwrap_or((candidate.start_seconds, candidate.end_seconds));
        push_attention_zone(
            &mut zones,
            manifest,
            start,
            end,
            MusicAttentionZoneKind::Focus,
            candidate_score,
        );
    }

    for segment in &manifest.sections.functional_segments {
        if segment.end_seconds <= segment.start_seconds + ATTENTION_ZONE_MIN_SECONDS {
            continue;
        }
        let Some(profile) =
            attention_profile_for_range(manifest, segment.start_seconds, segment.end_seconds)
        else {
            continue;
        };
        let structural_score = profile
            .attention_score
            .max(profile.structural_score * segment.confidence.clamp(0.0, 1.0));
        let segment_confidence = segment.confidence.clamp(0.0, 1.0);
        let kind = match segment.role {
            MusicFunctionalRole::Chorus | MusicFunctionalRole::FinalChorus => {
                let focus_evidence = segment_confidence
                    >= ATTENTION_ZONE_FUNCTIONAL_FOCUS_MIN_CONFIDENCE
                    || profile.highlight_score >= ATTENTION_ZONE_FOCUS_MIN_SCORE
                    || profile.boundary_strength >= 0.58;
                if structural_score >= ATTENTION_ZONE_FOCUS_MIN_SCORE && focus_evidence {
                    Some(MusicAttentionZoneKind::Focus)
                } else if structural_score >= ATTENTION_ZONE_BUILD_MIN_SCORE {
                    Some(MusicAttentionZoneKind::Build)
                } else {
                    None
                }
            }
            MusicFunctionalRole::PreChorus
            | MusicFunctionalRole::Bridge
            | MusicFunctionalRole::Verse
            | MusicFunctionalRole::Instrumental => (structural_score
                >= ATTENTION_ZONE_BUILD_MIN_SCORE)
                .then_some(MusicAttentionZoneKind::Build),
            MusicFunctionalRole::Intro => (profile.edge_trim_score
                >= ATTENTION_TRIM_DEFAULT_MIN_EDGE_SCORE
                && profile.attention_score < ATTENTION_ZONE_FOCUS_MIN_SCORE)
                .then_some(MusicAttentionZoneKind::Empty),
            MusicFunctionalRole::Outro => (segment_confidence
                >= ATTENTION_ZONE_FUNCTIONAL_EDGE_MIN_CONFIDENCE
                || profile.edge_trim_score >= ATTENTION_TRIM_DEFAULT_MIN_EDGE_SCORE
                || profile.emptiness_score >= 0.50)
                .then_some(MusicAttentionZoneKind::TailRisk),
            MusicFunctionalRole::Silence => (segment_confidence
                >= ATTENTION_ZONE_FUNCTIONAL_EDGE_MIN_CONFIDENCE
                || profile.emptiness_score >= 0.46)
                .then_some(MusicAttentionZoneKind::Empty),
        };
        if let Some(kind) = kind {
            push_attention_zone_with_profile(
                &mut zones,
                segment.start_seconds,
                segment.end_seconds,
                kind,
                structural_score
                    .max(profile.edge_trim_score)
                    .max(profile.emptiness_score * 0.82),
                profile,
            );
        }
    }

    push_mix_point_attention_zones(
        &mut zones,
        manifest,
        &manifest.mix_points.mix_in,
        MusicAttentionZoneKind::Entry,
    );
    push_mix_point_attention_zones(
        &mut zones,
        manifest,
        &manifest.mix_points.mix_out,
        MusicAttentionZoneKind::Exit,
    );

    compact_attention_zones(zones, duration)
}

pub(crate) fn best_focus_zone_for_manifest(
    manifest: &MusicAnalysisManifest,
) -> Option<MusicAttentionZone> {
    // Raw evidence helper: runtime highlight plans use
    // `select_focus_zone_runtime` so accepted/rejected gate decisions stay
    // explicit at the call site.
    attention_zones_for_manifest(manifest)
        .into_iter()
        .filter(|zone| {
            zone.kind == MusicAttentionZoneKind::Focus
                && zone.score >= ATTENTION_FOCUS_RUNTIME_MIN_SCORE
                && zone.end_seconds > zone.start_seconds + MIN_PLAYABLE_SEGMENT_SECONDS
        })
        .max_by(|a, b| {
            attention_focus_zone_runtime_score(*a, None)
                .partial_cmp(&attention_focus_zone_runtime_score(*b, None))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub(crate) fn focus_zone_for_candidate(
    manifest: &MusicAnalysisManifest,
    candidate: &MusicSectionCandidate,
) -> Option<MusicAttentionZone> {
    // Raw evidence helper: runtime highlight plans use
    // `select_focus_zone_runtime` so candidate fallback remains available when
    // the best raw Focus evidence is guarded.
    attention_zones_for_manifest(manifest)
        .into_iter()
        .filter(|zone| {
            zone.kind == MusicAttentionZoneKind::Focus
                && zone.score >= ATTENTION_FOCUS_RUNTIME_MIN_SCORE
                && range_overlap_ratio(
                    zone.start_seconds,
                    zone.end_seconds,
                    candidate.start_seconds,
                    candidate.end_seconds,
                ) >= ATTENTION_FOCUS_RUNTIME_MIN_OVERLAP
        })
        .max_by(|a, b| {
            attention_focus_zone_runtime_score(*a, Some(candidate))
                .partial_cmp(&attention_focus_zone_runtime_score(*b, Some(candidate)))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub(crate) fn attention_highlight_range_plan(
    manifest: &MusicAnalysisManifest,
    candidate: Option<&MusicSectionCandidate>,
) -> Option<MusicAttentionHighlightRangePlan> {
    let duration = manifest.duration_seconds;
    if !duration.is_finite() || duration <= 0.0 {
        return None;
    }
    let candidate_bounds = candidate.map(|candidate| {
        let start = candidate.start_seconds.clamp(0.0, duration);
        let end = candidate.end_seconds.clamp(start, duration.max(start));
        (start, end)
    });
    let selected_map_span =
        candidate.and_then(|candidate| select_map_span_for_candidate(manifest, candidate));
    let map_bounds = selected_map_span.map(|span| {
        let start = span.start_seconds.clamp(0.0, duration);
        let end = span.end_seconds.clamp(start, duration.max(start));
        (start, end)
    });
    let candidate_score = candidate.map(|candidate| highlight_candidate_score(candidate) as f32);
    let map_runtime_score = match (selected_map_span, candidate) {
        (Some(span), Some(candidate)) => Some(map_span_runtime_score(span, candidate, duration)),
        _ => None,
    };
    let rejected_map_guard = if selected_map_span.is_none() {
        candidate.and_then(|candidate| rejected_map_span_guard_for_candidate(manifest, candidate))
    } else {
        None
    };
    let mut guarded_source = None;
    let mut guard_reason = None;
    let mut rejected_focus_score = None;
    let mut rejected_focus_runtime_score = None;
    let mut rejected_focus_overlap = None;
    let mut rejected_focus_attention_score = None;
    let mut rejected_focus_structural_score = None;
    let focus_selection =
        select_focus_zone_runtime(manifest, candidate, candidate_score, map_runtime_score);
    if let Some(guard) = focus_selection.rejected {
        guarded_source = Some(MusicAttentionHighlightRangeSource::FocusZone);
        guard_reason = Some(guard.reason);
        rejected_focus_score = Some(guard.zone.score);
        rejected_focus_runtime_score = Some(guard.runtime_score);
        rejected_focus_overlap = Some(guard.overlap);
        rejected_focus_attention_score = Some(guard.zone.profile.attention_score);
        rejected_focus_structural_score = Some(guard.zone.profile.structural_score);
    }
    if let Some(decision) = focus_selection.accepted {
        let zone = decision.zone;
        let start = zone.start_seconds.clamp(0.0, duration);
        let end = zone.end_seconds.clamp(start, duration.max(start));
        return Some(MusicAttentionHighlightRangePlan {
            start_seconds: start,
            end_seconds: end,
            source: MusicAttentionHighlightRangeSource::FocusZone,
            selection_confidence: decision.runtime_score,
            selection_risk: attention_profile_runtime_risk_score(zone.profile),
            selection_reason: "focus-accepted",
            guarded_source: None,
            guard_reason: None,
            reference_start_seconds: candidate_bounds.or(map_bounds).map(|(start, _)| start),
            reference_end_seconds: candidate_bounds.or(map_bounds).map(|(_, end)| end),
            focus_score: Some(zone.score),
            focus_runtime_score: Some(decision.runtime_score),
            focus_overlap: Some(decision.overlap),
            focus_attention_score: Some(zone.profile.attention_score),
            focus_structural_score: Some(zone.profile.structural_score),
            candidate_score,
            map_lift_seconds: selected_map_span.map(|span| span.lift_seconds),
            map_peak_seconds: selected_map_span.map(|span| span.peak_seconds),
            map_confidence: selected_map_span.map(|span| span.confidence),
            map_runtime_score,
            rejected_map_reason: rejected_map_guard.map(|guard| guard.reason),
            rejected_map_confidence: rejected_map_guard.map(|guard| guard.confidence),
            rejected_map_runtime_score: rejected_map_guard.map(|guard| guard.runtime_score),
        });
    }
    if let (Some(span), Some(_candidate), Some((start, end))) =
        (selected_map_span, candidate, map_bounds)
    {
        let (candidate_start, candidate_end) = candidate_bounds.unwrap_or((start, end));
        return Some(MusicAttentionHighlightRangePlan {
            start_seconds: start,
            end_seconds: end,
            source: MusicAttentionHighlightRangeSource::MusicMapSpan,
            selection_confidence: map_runtime_score.unwrap_or(0.0).clamp(0.0, 1.0) as f32,
            selection_risk: if guarded_source.is_some() { 0.18 } else { 0.10 },
            selection_reason: if guarded_source.is_some() {
                "map-safer-than-focus"
            } else {
                "map-runtime-eligible"
            },
            guarded_source,
            guard_reason,
            reference_start_seconds: Some(candidate_start),
            reference_end_seconds: Some(candidate_end),
            focus_score: rejected_focus_score,
            focus_runtime_score: rejected_focus_runtime_score,
            focus_overlap: rejected_focus_overlap,
            focus_attention_score: rejected_focus_attention_score,
            focus_structural_score: rejected_focus_structural_score,
            candidate_score,
            map_lift_seconds: Some(span.lift_seconds),
            map_peak_seconds: Some(span.peak_seconds),
            map_confidence: Some(span.confidence),
            map_runtime_score,
            rejected_map_reason: rejected_map_guard.map(|guard| guard.reason),
            rejected_map_confidence: rejected_map_guard.map(|guard| guard.confidence),
            rejected_map_runtime_score: rejected_map_guard.map(|guard| guard.runtime_score),
        });
    }
    if let Some((start, end)) = candidate_bounds {
        return Some(MusicAttentionHighlightRangePlan {
            start_seconds: start,
            end_seconds: end,
            source: MusicAttentionHighlightRangeSource::Candidate,
            selection_confidence: candidate_score.unwrap_or(0.0).clamp(0.0, 1.0),
            selection_risk: if guarded_source.is_some() || rejected_map_guard.is_some() {
                0.26
            } else {
                0.18
            },
            selection_reason: if guarded_source.is_some() {
                "candidate-safer-than-focus"
            } else if rejected_map_guard.is_some() {
                "candidate-after-map-reject"
            } else {
                "candidate-fallback"
            },
            guarded_source,
            guard_reason,
            reference_start_seconds: None,
            reference_end_seconds: None,
            focus_score: rejected_focus_score,
            focus_runtime_score: rejected_focus_runtime_score,
            focus_overlap: rejected_focus_overlap,
            focus_attention_score: rejected_focus_attention_score,
            focus_structural_score: rejected_focus_structural_score,
            candidate_score,
            map_lift_seconds: None,
            map_peak_seconds: None,
            map_confidence: None,
            map_runtime_score: None,
            rejected_map_reason: rejected_map_guard.map(|guard| guard.reason),
            rejected_map_confidence: rejected_map_guard.map(|guard| guard.confidence),
            rejected_map_runtime_score: rejected_map_guard.map(|guard| guard.runtime_score),
        });
    }
    None
}

pub(crate) fn highlight_candidate_score(candidate: &MusicSectionCandidate) -> f64 {
    f64::from(attention_profile_for_candidate(candidate).highlight_score)
}

fn raw_highlight_candidate_score(candidate: &MusicSectionCandidate) -> f64 {
    let confidence = f64::from(candidate.confidence.max(0.05));
    let chorusness = f64::from(candidate.scores.chorusness.max(0.0));
    let repetition = f64::from(candidate.scores.repetition.max(0.0));
    let energy = f64::from(candidate.scores.energy.max(0.0));
    let contrast = f64::from(candidate.scores.contrast.max(0.0));
    let boundary = f64::from(candidate.scores.boundary.max(0.0));
    let duration = f64::from(candidate.scores.duration.max(0.0));
    let wholeness = f64::from(candidate.scores.segment_wholeness.max(0.0));
    let perceptual = f64::from(candidate.scores.perceptual.max(0.0));
    let recurrence = f64::from(candidate.scores.structural_recurrence.max(0.0));
    let length_seconds = (candidate.end_seconds - candidate.start_seconds).max(0.0);
    let short_spike_penalty = if length_seconds < PICK_SHORT_SPIKE_SECONDS {
        ((PICK_SHORT_SPIKE_SECONDS - length_seconds) / PICK_SHORT_SPIKE_SECONDS * 0.16)
            .clamp(0.0, 0.16)
    } else {
        0.0
    };
    let complete_body_bonus = if length_seconds >= PICK_COMPLETE_BODY_SECONDS
        && wholeness >= 0.55
        && (chorusness >= 0.55 || recurrence >= 0.50)
    {
        0.045
    } else {
        0.0
    };
    let overbroad_penalty = if length_seconds > PICK_OVERBROAD_SECONDS {
        ((length_seconds - PICK_OVERBROAD_SECONDS) / 80.0 * 0.10).clamp(0.0, 0.10)
    } else {
        0.0
    };

    (confidence * 0.42
        + chorusness * 0.17
        + repetition * 0.10
        + wholeness * 0.10
        + recurrence * 0.07
        + perceptual * 0.05
        + boundary * 0.04
        + duration * 0.03
        + contrast * 0.015
        + energy * 0.005
        + complete_body_bonus
        - short_spike_penalty
        - overbroad_penalty)
        .clamp(0.0, 1.0)
}

fn attention_energy_score_for_range(
    manifest: &MusicAnalysisManifest,
    start_seconds: f64,
    end_seconds: f64,
) -> f32 {
    let mut sum = 0.0_f64;
    let mut count = 0_usize;
    for point in &manifest.energy_curve {
        if point.time_seconds >= start_seconds && point.time_seconds <= end_seconds {
            sum += f64::from(point.rms.max(0.0));
            count += 1;
        }
    }
    if count == 0 {
        let midpoint = (start_seconds + end_seconds) * 0.5;
        if let Some(point) = manifest.energy_curve.iter().min_by(|a, b| {
            (a.time_seconds - midpoint)
                .abs()
                .partial_cmp(&(b.time_seconds - midpoint).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            sum = f64::from(point.rms.max(0.0));
            count = 1;
        }
    }
    if count == 0 {
        return 0.0;
    }

    let average_rms = sum / count as f64;
    let reference = f64::from(manifest.loudness.rms.max(0.000_001)) * 1.15;
    (average_rms / reference).clamp(0.0, 1.0) as f32
}

fn attention_highlight_score_for_range(
    manifest: &MusicAnalysisManifest,
    start_seconds: f64,
    end_seconds: f64,
) -> f32 {
    manifest
        .sections
        .highlight_candidates
        .iter()
        .map(|candidate| {
            range_overlap_ratio(
                start_seconds,
                end_seconds,
                candidate.start_seconds,
                candidate.end_seconds,
            ) * raw_highlight_candidate_score(candidate) as f32
        })
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0)
}

fn attention_functional_scores_for_range(
    manifest: &MusicAnalysisManifest,
    start_seconds: f64,
    end_seconds: f64,
) -> (f32, f32, f32) {
    let mut structural = 0.0_f32;
    let mut edge = 0.0_f32;
    let mut silence = 0.0_f32;
    for segment in &manifest.sections.functional_segments {
        let overlap = range_overlap_ratio(
            start_seconds,
            end_seconds,
            segment.start_seconds,
            segment.end_seconds,
        );
        if overlap <= 0.0 {
            continue;
        }
        let confidence = segment.confidence.clamp(0.0, 1.0);
        structural =
            structural.max(overlap * confidence * functional_role_attention_weight(&segment.role));
        edge = edge.max(overlap * confidence * functional_role_edge_trim_weight(&segment.role));
        if matches!(segment.role, MusicFunctionalRole::Silence) {
            silence = silence.max(overlap * confidence);
        }
    }
    (
        structural.clamp(0.0, 1.0),
        edge.clamp(0.0, 1.0),
        silence.clamp(0.0, 1.0),
    )
}

fn attention_boundary_strength_for_range(
    manifest: &MusicAnalysisManifest,
    start_seconds: f64,
    end_seconds: f64,
) -> f32 {
    let mut strength = 0.0_f32;
    for boundary in &manifest.section_curves.boundary_candidates {
        let start_distance = (boundary.time_seconds - start_seconds).abs();
        let end_distance = (boundary.time_seconds - end_seconds).abs();
        let distance = start_distance.min(end_distance);
        if distance <= ATTENTION_BOUNDARY_WINDOW_SECONDS {
            let proximity =
                (1.0 - distance / ATTENTION_BOUNDARY_WINDOW_SECONDS).clamp(0.0, 1.0) as f32;
            strength = strength.max(boundary.confidence.clamp(0.0, 1.0) * proximity);
        }
    }
    for segment in &manifest.sections.functional_segments {
        for boundary in [segment.start_seconds, segment.end_seconds] {
            let start_distance = (boundary - start_seconds).abs();
            let end_distance = (boundary - end_seconds).abs();
            let distance = start_distance.min(end_distance);
            if distance <= ATTENTION_BOUNDARY_WINDOW_SECONDS {
                let proximity =
                    (1.0 - distance / ATTENTION_BOUNDARY_WINDOW_SECONDS).clamp(0.0, 1.0) as f32;
                strength = strength.max(segment.confidence.clamp(0.0, 1.0) * proximity * 0.82);
            }
        }
    }
    strength.clamp(0.0, 1.0)
}

fn attention_mix_quality_for_range(points: &[MusicMixPoint], target_seconds: f64) -> f32 {
    points
        .iter()
        .filter(|point| {
            (point.time_seconds - target_seconds).abs() <= ATTENTION_MIX_POINT_WINDOW_SECONDS
        })
        .map(|point| {
            mix_point_runtime_score(point, target_seconds, ATTENTION_MIX_POINT_WINDOW_SECONDS)
        })
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0)
}

fn attention_trim_edge_is_accepted(
    profile: MusicAttentionProfile,
    reason: MusicAttentionTrimEdgeReason,
    policy: MusicAttentionTrimPolicy,
) -> bool {
    let min_score =
        (policy.min_edge_trim_score - attention_trim_reason_bonus(reason) * 0.42).clamp(0.32, 0.95);
    let attention_limit = if matches!(reason, MusicAttentionTrimEdgeReason::SilenceSection) {
        (policy.max_attention_score + 0.12).clamp(0.0, 1.0)
    } else {
        policy.max_attention_score.clamp(0.0, 1.0)
    };
    profile.edge_trim_score >= min_score && profile.attention_score <= attention_limit
}

fn attention_trim_edge_sort_score(
    boundary_seconds: f64,
    reason: MusicAttentionTrimEdgeReason,
    profile: MusicAttentionProfile,
    span_seconds: f64,
) -> f32 {
    let span_bonus = (boundary_seconds.max(span_seconds) / 60.0).clamp(0.0, 1.0) as f32 * 0.03;
    (profile.edge_trim_score + attention_trim_reason_bonus(reason) + span_bonus).clamp(0.0, 1.0)
}

fn attention_trim_reason_bonus(reason: MusicAttentionTrimEdgeReason) -> f32 {
    match reason {
        MusicAttentionTrimEdgeReason::IntroSection | MusicAttentionTrimEdgeReason::OutroSection => {
            ATTENTION_TRIM_REASON_BONUS_INTRO_OUTRO
        }
        MusicAttentionTrimEdgeReason::SilenceSection => ATTENTION_TRIM_REASON_BONUS_SILENCE,
        MusicAttentionTrimEdgeReason::LowEnergyHead
        | MusicAttentionTrimEdgeReason::LowEnergyTail => ATTENTION_TRIM_REASON_BONUS_LOW_ENERGY,
    }
}

fn functional_role_attention_weight(role: &MusicFunctionalRole) -> f32 {
    match role {
        MusicFunctionalRole::Intro => 0.18,
        MusicFunctionalRole::Verse => 0.46,
        MusicFunctionalRole::PreChorus => 0.62,
        MusicFunctionalRole::Chorus => 0.88,
        MusicFunctionalRole::FinalChorus => 0.92,
        MusicFunctionalRole::Bridge => 0.58,
        MusicFunctionalRole::Instrumental => 0.44,
        MusicFunctionalRole::Outro => 0.16,
        MusicFunctionalRole::Silence => 0.0,
    }
}

fn functional_role_edge_trim_weight(role: &MusicFunctionalRole) -> f32 {
    match role {
        MusicFunctionalRole::Intro => 0.70,
        MusicFunctionalRole::Outro => 0.76,
        MusicFunctionalRole::Silence => 1.0,
        MusicFunctionalRole::Instrumental => 0.22,
        MusicFunctionalRole::Verse => 0.12,
        MusicFunctionalRole::Bridge => 0.10,
        MusicFunctionalRole::PreChorus => 0.04,
        MusicFunctionalRole::Chorus | MusicFunctionalRole::FinalChorus => 0.0,
    }
}

fn functional_edge_segment_is_trusted(segment: &MusicFunctionalSegment) -> bool {
    let confidence = segment.confidence.clamp(0.0, 1.0);
    if matches!(segment.role, MusicFunctionalRole::Silence) {
        confidence >= 0.30
    } else {
        confidence >= ATTENTION_ZONE_FUNCTIONAL_EDGE_MIN_CONFIDENCE
    }
}

fn range_overlap_ratio(a_start: f64, a_end: f64, b_start: f64, b_end: f64) -> f32 {
    if !a_start.is_finite() || !a_end.is_finite() || !b_start.is_finite() || !b_end.is_finite() {
        return 0.0;
    }
    let a0 = a_start.min(a_end);
    let a1 = a_start.max(a_end);
    let b0 = b_start.min(b_end);
    let b1 = b_start.max(b_end);
    let overlap = (a1.min(b1) - a0.max(b0)).max(0.0);
    if overlap <= 0.0 {
        return 0.0;
    }
    let shorter = (a1 - a0).min(b1 - b0).max(0.000_001);
    (overlap / shorter).clamp(0.0, 1.0) as f32
}

fn select_focus_zone_runtime(
    manifest: &MusicAnalysisManifest,
    candidate: Option<&MusicSectionCandidate>,
    candidate_score: Option<f32>,
    map_runtime_score: Option<f64>,
) -> MusicFocusZoneRuntimeSelection {
    let mut selection = MusicFocusZoneRuntimeSelection::default();
    for zone in attention_zones_for_manifest(manifest) {
        if zone.kind != MusicAttentionZoneKind::Focus
            || zone.score < ATTENTION_FOCUS_RUNTIME_MIN_SCORE
            || zone.end_seconds <= zone.start_seconds + MIN_PLAYABLE_SEGMENT_SECONDS
        {
            continue;
        }
        let overlap = candidate
            .map(|candidate| {
                range_overlap_ratio(
                    zone.start_seconds,
                    zone.end_seconds,
                    candidate.start_seconds,
                    candidate.end_seconds,
                )
            })
            .unwrap_or(1.0);
        if candidate.is_some() && overlap < ATTENTION_FOCUS_RUNTIME_MIN_OVERLAP {
            continue;
        }

        let runtime_score = attention_focus_zone_runtime_score(zone, candidate);
        if let Some(reason) = attention_focus_zone_runtime_reject_reason(
            zone,
            candidate,
            runtime_score,
            overlap,
            candidate_score,
            map_runtime_score,
        ) {
            let guard = MusicFocusZoneRuntimeGuard {
                zone,
                reason,
                runtime_score,
                overlap,
            };
            if selection
                .rejected
                .is_none_or(|current| guard.runtime_score > current.runtime_score)
            {
                selection.rejected = Some(guard);
            }
            continue;
        }

        let decision = MusicFocusZoneRuntimeDecision {
            zone,
            runtime_score,
            overlap,
        };
        if selection
            .accepted
            .is_none_or(|current| decision.runtime_score > current.runtime_score)
        {
            selection.accepted = Some(decision);
        }
    }

    selection
}

fn attention_focus_zone_runtime_score(
    zone: MusicAttentionZone,
    candidate: Option<&MusicSectionCandidate>,
) -> f32 {
    let candidate_overlap = candidate
        .map(|candidate| {
            range_overlap_ratio(
                zone.start_seconds,
                zone.end_seconds,
                candidate.start_seconds,
                candidate.end_seconds,
            )
        })
        .unwrap_or(0.0);
    let duration = (zone.end_seconds - zone.start_seconds).max(0.0);
    let duration_score = if duration < PICK_COMPLETE_BODY_SECONDS {
        (duration / PICK_COMPLETE_BODY_SECONDS).clamp(0.0, 1.0) as f32
    } else if duration <= PRESENCE_MAX_SECONDS {
        1.0
    } else {
        (1.0 - ((duration - PRESENCE_MAX_SECONDS) / 80.0).clamp(0.0, 0.35)) as f32
    };

    (zone.score * 0.58
        + zone.profile.attention_score * 0.18
        + zone.profile.structural_score * 0.10
        + duration_score * 0.08
        + candidate_overlap * 0.06)
        .clamp(0.0, 1.0)
}

fn attention_profile_runtime_risk_score(profile: MusicAttentionProfile) -> f32 {
    let flag_risk = (if profile.reason_flags.silence_role {
        0.18
    } else {
        0.0
    }) + (if profile.reason_flags.edge_role {
        0.12
    } else {
        0.0
    }) + (if profile.reason_flags.low_energy {
        0.08
    } else {
        0.0
    });
    (profile.emptiness_score * 0.34
        + profile.edge_trim_score * 0.28
        + (1.0 - profile.attention_score).clamp(0.0, 1.0) * 0.20
        + (1.0 - profile.energy_score).clamp(0.0, 1.0) * 0.10
        + flag_risk)
        .clamp(0.0, 1.0)
}

fn attention_focus_zone_runtime_reject_reason(
    zone: MusicAttentionZone,
    candidate: Option<&MusicSectionCandidate>,
    focus_runtime_score: f32,
    focus_overlap: f32,
    candidate_score: Option<f32>,
    map_runtime_score: Option<f64>,
) -> Option<&'static str> {
    if !focus_runtime_score.is_finite() || focus_runtime_score < ATTENTION_FOCUS_RUNTIME_MIN_SCORE {
        return Some("weak-focus-score");
    }

    let body_evidence = zone
        .profile
        .highlight_score
        .max(zone.profile.structural_score)
        .max(zone.profile.boundary_strength * 0.72);
    if zone.profile.reason_flags.silence_role && zone.profile.attention_score < 0.68 {
        return Some("silence-risk");
    }
    if zone.profile.edge_trim_score >= 0.54 && zone.profile.attention_score < 0.64 {
        return Some("edge-risk");
    }
    if zone.profile.emptiness_score >= 0.50 && body_evidence < 0.62 {
        return Some("empty-risk");
    }
    if zone.profile.attention_score < ATTENTION_FOCUS_RUNTIME_MIN_ATTENTION
        && body_evidence < ATTENTION_FOCUS_RUNTIME_MIN_BODY_EVIDENCE
    {
        return Some("weak-attention-evidence");
    }

    if candidate.is_some() {
        if focus_overlap < ATTENTION_FOCUS_RUNTIME_MIN_OVERLAP.max(0.58) {
            return Some("weak-candidate-overlap");
        }
        if let Some(map_score) = map_runtime_score.map(|score| score as f32) {
            if map_score >= MAP_SPAN_RUNTIME_MIN_SCORE as f32
                && focus_runtime_score + ATTENTION_FOCUS_RUNTIME_MAP_MARGIN < map_score
            {
                return Some("safer-map-span");
            }
        }
        if let Some(candidate_score) = candidate_score {
            if candidate_score >= PICK_MIN_CONFIDENCE
                && focus_overlap < 0.92
                && focus_runtime_score + ATTENTION_FOCUS_RUNTIME_CANDIDATE_MARGIN < candidate_score
            {
                return Some("safer-candidate");
            }
        }
        if focus_runtime_score < ATTENTION_FOCUS_RUNTIME_STRONG_SCORE {
            return Some("weak-focus-score");
        }
    } else if focus_runtime_score < ATTENTION_FOCUS_RUNTIME_STANDALONE_SCORE {
        return Some("weak-standalone-focus");
    }

    None
}

fn push_attention_zone(
    zones: &mut Vec<MusicAttentionZone>,
    manifest: &MusicAnalysisManifest,
    start_seconds: f64,
    end_seconds: f64,
    kind: MusicAttentionZoneKind,
    score: f32,
) {
    let Some(profile) = attention_profile_for_range(manifest, start_seconds, end_seconds) else {
        return;
    };
    push_attention_zone_with_profile(zones, start_seconds, end_seconds, kind, score, profile);
}

fn push_attention_zone_with_profile(
    zones: &mut Vec<MusicAttentionZone>,
    start_seconds: f64,
    end_seconds: f64,
    kind: MusicAttentionZoneKind,
    score: f32,
    profile: MusicAttentionProfile,
) {
    if !start_seconds.is_finite()
        || !end_seconds.is_finite()
        || end_seconds <= start_seconds + ATTENTION_ZONE_MIN_SECONDS
        || !score.is_finite()
    {
        return;
    }
    zones.push(MusicAttentionZone {
        start_seconds,
        end_seconds,
        kind,
        score: score.clamp(0.0, 1.0),
        profile,
    });
}

fn push_mix_point_attention_zones(
    zones: &mut Vec<MusicAttentionZone>,
    manifest: &MusicAnalysisManifest,
    points: &[MusicMixPoint],
    kind: MusicAttentionZoneKind,
) {
    let mut scored: Vec<(f32, &MusicMixPoint)> = points
        .iter()
        .map(|point| {
            (
                mix_point_runtime_score(
                    point,
                    point.time_seconds,
                    ATTENTION_MIX_POINT_WINDOW_SECONDS,
                ),
                point,
            )
        })
        .filter(|(score, point)| {
            *score >= ATTENTION_ZONE_MIX_MIN_SCORE && point.time_seconds.is_finite()
        })
        .collect();
    scored.sort_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    for (score, point) in scored.into_iter().take(6) {
        let start = point.time_seconds - ATTENTION_ZONE_MIX_PAD_SECONDS;
        let end = point.time_seconds + ATTENTION_ZONE_MIX_PAD_SECONDS;
        push_attention_zone(zones, manifest, start, end, kind, score);
        if score >= 0.58 {
            push_attention_zone(
                zones,
                manifest,
                start,
                end,
                MusicAttentionZoneKind::MixSafe,
                score * 0.92,
            );
        }
    }
}

fn compact_attention_zones(
    mut zones: Vec<MusicAttentionZone>,
    duration_seconds: f64,
) -> Vec<MusicAttentionZone> {
    zones.retain(|zone| {
        zone.start_seconds.is_finite()
            && zone.end_seconds.is_finite()
            && zone.end_seconds > zone.start_seconds + ATTENTION_ZONE_MIN_SECONDS
            && zone.score > 0.0
    });
    for zone in zones.iter_mut() {
        zone.start_seconds = zone.start_seconds.clamp(0.0, duration_seconds.max(0.0));
        zone.end_seconds = zone
            .end_seconds
            .clamp(zone.start_seconds, duration_seconds.max(zone.start_seconds));
    }
    zones.retain(|zone| zone.end_seconds > zone.start_seconds + ATTENTION_ZONE_MIN_SECONDS);
    zones.sort_by(|a, b| {
        a.kind
            .visual_priority()
            .cmp(&b.kind.visual_priority())
            .then_with(|| {
                a.start_seconds
                    .partial_cmp(&b.start_seconds)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let mut compacted: Vec<MusicAttentionZone> = Vec::new();
    for zone in zones {
        if let Some(previous) = compacted.last_mut() {
            // Focus zones are selection evidence.  Do not merge adjacent
            // candidates into a wider synthetic focus span; the runtime gate
            // must evaluate the original evidence boundaries.
            if previous.kind == zone.kind
                && previous.kind != MusicAttentionZoneKind::Focus
                && zone.start_seconds <= previous.end_seconds + 1.0
            {
                previous.end_seconds = previous.end_seconds.max(zone.end_seconds);
                if zone.score > previous.score {
                    previous.score = zone.score;
                    previous.profile = zone.profile;
                }
                continue;
            }
        }
        compacted.push(zone);
    }

    compacted.sort_by(|a, b| {
        a.start_seconds
            .partial_cmp(&b.start_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.kind.visual_priority().cmp(&b.kind.visual_priority()))
    });
    compacted
}

pub(crate) fn best_highlight_candidate(
    manifest: &MusicAnalysisManifest,
) -> Option<&MusicSectionCandidate> {
    manifest
        .sections
        .highlight_candidates
        .iter()
        .max_by(|a, b| {
            highlight_candidate_score(a)
                .partial_cmp(&highlight_candidate_score(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub(crate) fn select_highlight_pick(
    manifest: &MusicAnalysisManifest,
    seed: u64,
) -> Option<MusicStageHighlightPick> {
    let candidates = &manifest.sections.highlight_candidates;
    if candidates.is_empty() {
        return None;
    }
    let primary_index = candidates
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            highlight_candidate_score(a)
                .partial_cmp(&highlight_candidate_score(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(index, _)| index)
        .unwrap_or(0);
    if candidates.len() == 1 {
        let candidate = &candidates[0];
        return Some(MusicStageHighlightPick {
            candidate_index: 0,
            start_seconds: candidate.start_seconds,
            end_seconds: candidate.end_seconds,
        });
    }

    let primary_candidate = &candidates[primary_index];
    let primary_score = highlight_candidate_score(primary_candidate);
    let mut weighted = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let length = candidate.end_seconds - candidate.start_seconds;
        if length < MIN_PLAYABLE_SEGMENT_SECONDS {
            continue;
        }
        let is_primary = index == primary_index;
        let candidate_score = highlight_candidate_score(candidate);
        if !is_primary {
            let close_to_primary = candidate.confidence + PICK_VARIANT_CONFIDENCE_MARGIN
                >= primary_candidate.confidence
                || candidate_score >= primary_score * 0.82;
            if !close_to_primary
                || candidate_score < f64::from(PICK_VARIANT_SCORE_FLOOR)
                || (candidate.confidence < PICK_MIN_CONFIDENCE
                    && candidate.scores.chorusness < 0.48
                    && candidate.scores.repetition < 0.40)
            {
                continue;
            }
        }

        let mut weight = candidate_score.powf(1.7);
        let position = ((candidate.start_seconds + candidate.end_seconds) * 0.5
            / manifest.duration_seconds.max(1.0))
        .clamp(0.0, 1.0);
        if !is_primary && position >= 0.66 {
            // Final chorus variants can appear occasionally, but they should not
            // overpower the primary highlight just because they are late.
            weight *= 1.10;
        }
        if is_primary {
            weight *= PICK_PRIMARY_WEIGHT_BOOST;
        } else {
            weight *= PICK_VARIANT_WEIGHT_BOOST;
        }
        if weight.is_finite() && weight > 0.0001 {
            weighted.push((index, weight));
        }
    }

    if weighted.is_empty() {
        let candidate = &candidates[primary_index];
        return Some(MusicStageHighlightPick {
            candidate_index: primary_index,
            start_seconds: candidate.start_seconds,
            end_seconds: candidate.end_seconds,
        });
    }

    let total = weighted.iter().map(|(_, weight)| *weight).sum::<f64>();
    if total <= 0.0001 || !total.is_finite() {
        let candidate = &candidates[primary_index];
        return Some(MusicStageHighlightPick {
            candidate_index: primary_index,
            start_seconds: candidate.start_seconds,
            end_seconds: candidate.end_seconds,
        });
    }

    let mut needle = unit_from_seed(seed) * total;
    let mut selected_index = primary_index;
    for (index, weight) in weighted {
        if needle <= weight {
            selected_index = index;
            break;
        }
        needle -= weight;
    }

    let candidate = &candidates[selected_index];
    Some(MusicStageHighlightPick {
        candidate_index: selected_index,
        start_seconds: candidate.start_seconds,
        end_seconds: candidate.end_seconds,
    })
}

pub(crate) fn select_direct_body_highlight_candidate<'a>(
    manifest: &'a MusicAnalysisManifest,
    policy: MusicDirectBodyHighlightPolicy,
) -> Option<&'a MusicSectionCandidate> {
    manifest
        .sections
        .highlight_candidates
        .iter()
        .filter(|candidate| {
            candidate.start_seconds.is_finite()
                && candidate.end_seconds.is_finite()
                && candidate.end_seconds > candidate.start_seconds + policy.min_segment_seconds
                && candidate.start_seconds <= policy.latest_start_seconds
                && candidate.confidence >= policy.min_confidence
        })
        .max_by(|a, b| {
            direct_body_highlight_score(a, policy)
                .partial_cmp(&direct_body_highlight_score(b, policy))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub(crate) fn direct_body_highlight_score(
    candidate: &MusicSectionCandidate,
    policy: MusicDirectBodyHighlightPolicy,
) -> f64 {
    let base = highlight_candidate_score(candidate);
    let midpoint = ((candidate.start_seconds + candidate.end_seconds) * 0.5)
        / policy.duration_seconds.max(1.0);
    let overflow = (candidate.end_seconds - policy.body_fence_seconds).max(0.0);
    let overflow_penalty = if overflow > policy.tail_grace_seconds {
        (overflow / 12.0).min(0.42)
    } else {
        0.0
    };
    let late_penalty = if midpoint >= policy.late_midpoint_share {
        ((midpoint - policy.late_midpoint_share) * 0.85).min(0.20)
    } else {
        0.0
    };
    (base - overflow_penalty - late_penalty).clamp(0.0, 1.0)
}

pub(crate) fn full_mix_out_seconds(
    manifest: &MusicAnalysisManifest,
    min_segment_seconds: f64,
    min_transition_seconds: f64,
) -> Option<f64> {
    let duration = manifest.duration_seconds;
    if !duration.is_finite() || duration < min_segment_seconds {
        return None;
    }

    let mut candidates: Vec<f64> = Vec::new();

    if let Some(outro) = manifest.sections.outro.as_ref() {
        if outro.start_seconds >= min_segment_seconds
            && duration - outro.start_seconds >= min_transition_seconds
        {
            candidates.push(outro.start_seconds);
        }
    }

    for segment in &manifest.sections.functional_segments {
        if matches!(
            segment.role,
            MusicFunctionalRole::Outro | MusicFunctionalRole::Silence
        ) && segment.start_seconds >= min_segment_seconds
            && duration - segment.start_seconds >= min_transition_seconds
        {
            candidates.push(segment.start_seconds);
        }
    }

    if let Some(tail_start) =
        low_energy_tail_start_seconds(manifest, min_segment_seconds, min_transition_seconds)
    {
        candidates.push(tail_start);
    }

    if let Some(best_late_mix_out) = manifest
        .mix_points
        .mix_out
        .iter()
        .filter(|point| {
            point.time_seconds >= duration * 0.58
                && point.time_seconds <= duration - min_transition_seconds
        })
        .max_by(|a, b| {
            let a_score = a.confidence * 0.72 + a.vocal_safety * 0.28;
            let b_score = b.confidence * 0.72 + b.vocal_safety * 0.28;
            a_score
                .partial_cmp(&b_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        candidates.push(best_late_mix_out.time_seconds);
    }

    candidates
        .into_iter()
        .filter(|value| value.is_finite())
        .filter(|value| *value >= min_segment_seconds)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}

pub(crate) fn direct_body_fence_seconds(
    manifest: &MusicAnalysisManifest,
    transition_seconds: f64,
    policy: MusicStageBodyFencePolicy,
) -> Option<f64> {
    let duration = manifest.duration_seconds;
    if !duration.is_finite() || duration <= policy.min_segment_seconds * 2.0 {
        return None;
    }

    let min_remaining = policy.min_remaining_seconds.max(
        transition_seconds.max(policy.transition_min_seconds)
            + policy.post_promote_min_dwell_seconds * 0.72,
    );
    let mut fence = (duration - min_remaining)
        .min(duration * policy.song_share)
        .max(policy.min_segment_seconds);

    if let Some(full_mix_out) = full_mix_out_seconds(
        manifest,
        policy.min_segment_seconds,
        policy.transition_min_seconds,
    ) {
        if full_mix_out.is_finite() {
            fence = fence
                .min((full_mix_out - policy.outro_backoff_seconds).max(policy.min_segment_seconds));
        }
    }

    if let Some(low_energy_tail) = low_energy_tail_start_seconds(
        manifest,
        policy.min_segment_seconds,
        policy.transition_min_seconds,
    ) {
        if low_energy_tail.is_finite() {
            fence = fence.min(
                (low_energy_tail - policy.outro_backoff_seconds).max(policy.min_segment_seconds),
            );
        }
    }

    for segment in &manifest.sections.functional_segments {
        if matches!(
            segment.role,
            MusicFunctionalRole::Outro | MusicFunctionalRole::Silence
        ) && segment.start_seconds.is_finite()
            && segment.start_seconds >= policy.min_segment_seconds
        {
            fence = fence.min(
                (segment.start_seconds - policy.outro_backoff_seconds)
                    .max(policy.min_segment_seconds),
            );
        }
    }

    if fence >= policy.min_segment_seconds && fence < duration {
        Some(fence)
    } else {
        None
    }
}

pub(crate) fn last_audible_seconds_from_energy(
    manifest: &MusicAnalysisManifest,
    duration_seconds: f64,
    policy: MusicStageEnergyTailPolicy,
) -> Option<f64> {
    let duration = if duration_seconds.is_finite() && duration_seconds > 0.0 {
        duration_seconds
    } else {
        manifest.duration_seconds
    };
    if !duration.is_finite() || duration <= policy.min_segment_seconds {
        return None;
    }
    let curve = &manifest.energy_curve;
    if curve.len() < 4 {
        return None;
    }

    let peak_rms = curve
        .iter()
        .map(|point| point.rms.max(0.0))
        .fold(0.0_f32, f32::max);
    let threshold = (manifest.loudness.rms.max(0.0) * policy.relative_rms)
        .max(peak_rms * policy.peak_rms)
        .max(policy.min_rms);
    let peak_threshold = (threshold * 1.8).max(policy.min_rms * 2.4);

    let mut hop_seconds = 0.5_f64;
    for window in curve.windows(2) {
        let delta = window[1].time_seconds - window[0].time_seconds;
        if delta.is_finite() && delta > 0.01 {
            hop_seconds = delta.clamp(0.08, 2.0);
            break;
        }
    }

    curve
        .iter()
        .rev()
        .find(|point| {
            point.time_seconds.is_finite()
                && point.time_seconds >= 0.0
                && point.time_seconds <= duration + hop_seconds
                && (point.rms >= threshold || point.peak >= peak_threshold)
        })
        .map(|point| (point.time_seconds + hop_seconds).min(duration))
}

pub(crate) fn body_fence_safe_exit_end_seconds(
    playback_start_seconds: f64,
    segment_end_seconds: f64,
    transition_seconds: f64,
    duration_seconds: f64,
    body_fence_seconds: f64,
    policy: MusicStageBodyFenceExitPolicy,
) -> Option<f64> {
    if !playback_start_seconds.is_finite()
        || !segment_end_seconds.is_finite()
        || !transition_seconds.is_finite()
        || !duration_seconds.is_finite()
        || !body_fence_seconds.is_finite()
        || duration_seconds <= 0.0
    {
        return None;
    }
    if segment_end_seconds <= body_fence_seconds + policy.tail_grace_seconds {
        return None;
    }

    let minimum_end = playback_start_seconds
        + transition_seconds.max(policy.transition_min_seconds)
        + policy.advance_guard_seconds;
    let safe_end = body_fence_seconds
        .min(duration_seconds - policy.advance_guard_seconds)
        .max(0.0);
    if safe_end > minimum_end && safe_end < segment_end_seconds {
        Some(safe_end)
    } else {
        None
    }
}

pub(crate) fn energy_tail_safe_exit_end_seconds(
    playback_start_seconds: f64,
    segment_end_seconds: f64,
    transition_seconds: f64,
    duration_seconds: f64,
    last_audible_seconds: f64,
    policy: MusicStageEnergyTailExitPolicy,
) -> Option<f64> {
    if !playback_start_seconds.is_finite()
        || !segment_end_seconds.is_finite()
        || !transition_seconds.is_finite()
        || !duration_seconds.is_finite()
        || !last_audible_seconds.is_finite()
        || duration_seconds <= 0.0
    {
        return None;
    }
    if duration_seconds - last_audible_seconds < policy.min_tail_seconds {
        return None;
    }
    if segment_end_seconds <= last_audible_seconds + policy.exit_grace_seconds {
        return None;
    }

    let safe_end = (last_audible_seconds + policy.exit_grace_seconds)
        .min(duration_seconds - policy.advance_guard_seconds)
        .max(0.0);
    let minimum_end = playback_start_seconds
        + transition_seconds.max(policy.transition_min_seconds)
        + policy.advance_guard_seconds;
    if safe_end > minimum_end && safe_end < segment_end_seconds {
        Some(safe_end)
    } else {
        None
    }
}

pub(crate) fn direct_tail_safe_entry_start_seconds(
    entry_start_seconds: f64,
    transition_seconds: f64,
    duration_seconds: Option<f64>,
    body_fence_seconds: Option<f64>,
    tail_section_start_seconds: Option<f64>,
    last_lyric_seconds: Option<f64>,
    last_audible_seconds: Option<f64>,
    policy: MusicStageTailSafeEntryPolicy,
) -> MusicStageTailSafeEntryPlan {
    let original_start = if entry_start_seconds.is_finite() {
        entry_start_seconds.max(0.0)
    } else {
        0.0
    };

    let Some(duration) = duration_seconds.filter(|duration| {
        duration.is_finite()
            && *duration > policy.min_segment_seconds
            && *duration > transition_seconds + policy.advance_guard_seconds
    }) else {
        return MusicStageTailSafeEntryPlan {
            start_seconds: original_start,
            reason: MusicStageTailSafeEntryReason::Runway,
        };
    };

    let direct_runway = policy
        .min_remaining_seconds
        .max(
            transition_seconds
                + policy.post_promote_min_dwell_seconds
                + policy.extra_runway_seconds,
        )
        .min((duration - policy.min_segment_seconds).max(0.0));
    let mut latest_body_entry = (duration - direct_runway).max(0.0);
    let mut reason = MusicStageTailSafeEntryReason::Runway;

    if let Some(body_fence) = body_fence_seconds.filter(|value| value.is_finite()) {
        if body_fence < latest_body_entry {
            latest_body_entry = body_fence;
            reason = MusicStageTailSafeEntryReason::BodyFence;
        }
    }

    if let Some(tail_start) = tail_section_start_seconds.filter(|value| value.is_finite()) {
        let safe_before_tail = (tail_start - policy.tail_section_backoff_seconds).max(0.0);
        if safe_before_tail < latest_body_entry {
            latest_body_entry = safe_before_tail;
            reason = MusicStageTailSafeEntryReason::TailSection;
        }
    }

    if let Some(last_lyric) = last_lyric_seconds.filter(|value| value.is_finite()) {
        if duration - last_lyric >= policy.trailing_silence_min_seconds {
            let safe_before_silence = (last_lyric - policy.last_lyric_backoff_seconds).max(0.0);
            if safe_before_silence < latest_body_entry {
                latest_body_entry = safe_before_silence;
                reason = MusicStageTailSafeEntryReason::LyricTail;
            }
        }
    }

    if let Some(last_audible) = last_audible_seconds.filter(|value| value.is_finite()) {
        if duration - last_audible >= policy.energy_tail_min_seconds {
            let safe_before_energy_tail =
                (last_audible - policy.energy_tail_entry_backoff_seconds).max(0.0);
            if safe_before_energy_tail < latest_body_entry {
                latest_body_entry = safe_before_energy_tail;
                reason = MusicStageTailSafeEntryReason::EnergyTail;
            }
        }
    }

    MusicStageTailSafeEntryPlan {
        start_seconds: latest_body_entry.min(original_start),
        reason,
    }
}

pub(crate) fn mix_length_multiplier(
    length: f64,
    policy: MusicStageMixLengthMultiplierPolicy,
) -> f64 {
    let length = length.clamp(0.0, 1.0);
    if length <= 0.50 {
        let ratio = (length / 0.50).clamp(0.0, 1.0);
        policy.short_multiplier + (1.0 - policy.short_multiplier) * ratio
    } else {
        let ratio = ((length - 0.50) / 0.50).clamp(0.0, 1.0);
        1.0 + (policy.long_multiplier - 1.0) * ratio
    }
}

pub(crate) fn tempo_bridge_strength_multiplier(
    strength: f64,
    policy: MusicStageTempoBridgeStrengthPolicy,
) -> f64 {
    let strength = strength.clamp(0.0, 1.0);
    let curved = strength.powf(0.82);
    (policy.min_multiplier + (policy.max_multiplier - policy.min_multiplier) * curved)
        .clamp(policy.min_multiplier, policy.max_multiplier)
}

pub(crate) fn tempo_bridge_rate_bounds(
    strength: f64,
    policy: MusicStageTempoBridgeRateBoundsPolicy,
) -> ((f64, f64), (f64, f64)) {
    let strength = strength.clamp(0.0, 1.0);
    let curved = strength.powf(0.88);
    let incoming_delta = (policy.incoming_soft_max_delta
        + (policy.incoming_strong_max_delta - policy.incoming_soft_max_delta) * curved)
        .clamp(
            policy.incoming_soft_max_delta,
            policy.incoming_strong_max_delta,
        );
    let outgoing_delta = (policy.outgoing_soft_max_delta
        + (policy.outgoing_strong_max_delta - policy.outgoing_soft_max_delta) * curved)
        .clamp(
            policy.outgoing_soft_max_delta,
            policy.outgoing_strong_max_delta,
        );
    (
        (1.0 - outgoing_delta, 1.0 + outgoing_delta),
        (1.0 - incoming_delta, 1.0 + incoming_delta),
    )
}

pub(crate) fn scale_tempo_rate(rate: f64, multiplier: f64, min_rate: f64, max_rate: f64) -> f64 {
    if !rate.is_finite() || !multiplier.is_finite() || multiplier <= 0.0 {
        return 1.0;
    }
    (1.0 + (rate - 1.0) * multiplier).clamp(min_rate, max_rate)
}

pub(crate) fn post_handoff_guarded_end_seconds(
    playback_start_seconds: f64,
    segment_end_seconds: f64,
    duration_seconds: f64,
    policy: MusicStagePostHandoffGuardPolicy,
) -> f64 {
    if !playback_start_seconds.is_finite()
        || !segment_end_seconds.is_finite()
        || !duration_seconds.is_finite()
        || duration_seconds <= playback_start_seconds + policy.advance_guard_seconds
    {
        return segment_end_seconds;
    }

    let minimum_end = playback_start_seconds
        + policy.post_handoff_breathe_seconds
        + policy.transition_max_seconds;
    let duration_cap = (duration_seconds - policy.advance_guard_seconds)
        .max(playback_start_seconds + policy.transition_min_seconds);
    segment_end_seconds.max(minimum_end.min(duration_cap))
}

pub(crate) fn tail_guarded_exit_end_seconds(
    playback_start_seconds: f64,
    segment_end_seconds: f64,
    transition_seconds: f64,
    duration_seconds: Option<f64>,
    policy: MusicStageTailExitGuardPolicy,
) -> Option<f64> {
    if !playback_start_seconds.is_finite()
        || !segment_end_seconds.is_finite()
        || !transition_seconds.is_finite()
    {
        return None;
    }
    let duration = duration_seconds.filter(|duration| duration.is_finite() && *duration > 0.0)?;
    if duration - segment_end_seconds > policy.exit_tail_guard_seconds {
        return None;
    }

    let safe_end = duration - policy.exit_tail_guard_seconds;
    let minimum_end = playback_start_seconds
        + transition_seconds.max(policy.transition_min_seconds)
        + policy.advance_guard_seconds;
    if safe_end > minimum_end && safe_end < segment_end_seconds {
        Some(safe_end)
    } else {
        None
    }
}

pub(crate) fn direct_latest_entry_start_seconds(
    duration_seconds: f64,
    transition_seconds: f64,
    policy: MusicStageLatestEntryPolicy,
) -> f64 {
    let target_runway = policy.min_remaining_seconds.max(
        transition_seconds + policy.post_promote_min_dwell_seconds + policy.extra_runway_seconds,
    );
    (duration_seconds - target_runway).max(0.0)
}

pub(crate) fn direct_entry_anchor_score(
    segment: &MusicFunctionalSegment,
    target_seconds: f64,
) -> f64 {
    let role_bias = match &segment.role {
        MusicFunctionalRole::Chorus | MusicFunctionalRole::FinalChorus => -0.55,
        MusicFunctionalRole::PreChorus => -0.32,
        MusicFunctionalRole::Verse => -0.12,
        MusicFunctionalRole::Bridge | MusicFunctionalRole::Instrumental => 0.08,
        MusicFunctionalRole::Intro => 0.22,
        MusicFunctionalRole::Outro | MusicFunctionalRole::Silence => 8.0,
    };
    (segment.start_seconds - target_seconds).abs() + role_bias - segment.confidence as f64 * 0.18
}

pub(crate) fn safe_entry_start_seconds(
    entry_start_seconds: f64,
    transition_seconds: f64,
    track_duration_seconds: Option<f64>,
    policy: MusicStageSafeEntryPolicy,
) -> f64 {
    if !policy.lite_enabled {
        return entry_start_seconds;
    }

    let Some(duration) = track_duration_seconds.filter(|duration| {
        duration.is_finite()
            && *duration > transition_seconds + policy.extra_runway_seconds
            && entry_start_seconds >= 0.0
    }) else {
        return entry_start_seconds.max(0.0);
    };

    let min_remaining = if policy.direct_stream {
        policy.direct_min_remaining_seconds
    } else {
        policy.fallback_min_remaining_seconds
    };
    let song_share = if policy.direct_stream {
        policy.direct_song_share
    } else {
        policy.fallback_song_share
    };

    let current_remaining = duration - entry_start_seconds;
    if current_remaining >= min_remaining {
        return entry_start_seconds.max(0.0);
    }

    let target_runway = policy
        .promoted_deck_target_seconds
        .min((duration * song_share).max(min_remaining))
        .max(
            transition_seconds
                + policy.post_promote_min_dwell_seconds
                + policy.extra_runway_seconds,
        );
    (duration - target_runway)
        .max(0.0)
        .min(entry_start_seconds.max(0.0))
}

pub(crate) fn selected_mix_in_point_for_manifest(
    manifest: &MusicAnalysisManifest,
    target_seconds: Option<f64>,
) -> Option<MusicMixPoint> {
    if let Some(target_seconds) = target_seconds {
        return best_vocal_safe_mix_point_near(&manifest.mix_points.mix_in, target_seconds, 6.0)
            .cloned()
            .or_else(|| closest_mix_point_to_target(&manifest.mix_points.mix_in, target_seconds));
    }
    manifest.mix_points.mix_in.first().cloned()
}

pub(crate) fn selected_mix_out_point_for_manifest(
    manifest: &MusicAnalysisManifest,
    target_seconds: Option<f64>,
) -> Option<MusicMixPoint> {
    if let Some(target_seconds) = target_seconds {
        return best_vocal_safe_mix_point_near(&manifest.mix_points.mix_out, target_seconds, 8.0)
            .cloned()
            .or_else(|| closest_mix_point_to_target(&manifest.mix_points.mix_out, target_seconds));
    }
    manifest.mix_points.mix_out.first().cloned()
}

pub(crate) fn segment_bpm_from_analysis(
    manifest: &MusicAnalysisManifest,
    playback_seconds: Option<f64>,
    display_highlight_range: Option<(f64, f64)>,
) -> Option<f32> {
    let focus_seconds = playback_seconds
        .filter(|seconds| *seconds >= 0.0 && *seconds <= manifest.duration_seconds.max(0.0))
        .or_else(|| display_highlight_range.map(|(start, end)| (start + end) * 0.5));

    if let Some(focus_seconds) = focus_seconds {
        if let Some(bpm) = manifest
            .sections
            .segment_tempo
            .iter()
            .filter(|segment| {
                segment.start_seconds <= focus_seconds && focus_seconds < segment.end_seconds
            })
            .filter_map(|segment| {
                segment
                    .bpm
                    .map(|bpm| (bpm, segment.confidence, segment.stable))
            })
            .max_by(|a, b| {
                let a_weight = a.1 + if a.2 { 0.12 } else { 0.0 };
                let b_weight = b.1 + if b.2 { 0.12 } else { 0.0 };
                a_weight
                    .partial_cmp(&b_weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(bpm, _, _)| bpm)
        {
            return Some(bpm);
        }

        if let Some(bpm) = manifest
            .sections
            .segment_tempo
            .iter()
            .filter_map(|segment| {
                let bpm = segment.bpm?;
                let distance = if focus_seconds < segment.start_seconds {
                    segment.start_seconds - focus_seconds
                } else if focus_seconds > segment.end_seconds {
                    focus_seconds - segment.end_seconds
                } else {
                    0.0
                };
                Some((bpm, distance, segment.confidence))
            })
            .min_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
            })
            .map(|(bpm, _, _)| bpm)
        {
            return Some(bpm);
        }
    }

    manifest.tempo.bpm
}

pub(crate) fn best_vocal_safe_mix_point_near<'a>(
    points: &'a [MusicMixPoint],
    target_seconds: f64,
    window_seconds: f64,
) -> Option<&'a MusicMixPoint> {
    points
        .iter()
        .filter(|point| {
            (point.time_seconds - target_seconds).abs() <= window_seconds
                && (point.vocal_safety >= 0.30 || point.confidence >= 0.72)
        })
        .max_by(|a, b| {
            mix_point_runtime_score(a, target_seconds, window_seconds)
                .partial_cmp(&mix_point_runtime_score(b, target_seconds, window_seconds))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn closest_mix_point_to_target(
    points: &[MusicMixPoint],
    target_seconds: f64,
) -> Option<MusicMixPoint> {
    points
        .iter()
        .min_by(|a, b| {
            (a.time_seconds - target_seconds)
                .abs()
                .partial_cmp(&(b.time_seconds - target_seconds).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
}

fn mix_point_runtime_score(point: &MusicMixPoint, target_seconds: f64, window_seconds: f64) -> f32 {
    let distance = (point.time_seconds - target_seconds).abs() / window_seconds.max(0.01);
    let vocal_penalty = if point.vocal_safety < 0.22 { 0.05 } else { 0.0 };
    let perceptual = if point.perceptual_score > 0.0 {
        point.perceptual_score
    } else {
        point.vocal_safety
    };
    let stage_phrase = point.phrase_grid_fit.max(point.phrase_closure * 0.72);
    let emotion = if point.emotional_continuity > 0.0 {
        point.emotional_continuity
    } else {
        point.expectation_safety
    };

    point.confidence * 0.26
        + point.vocal_safety * 0.27
        + perceptual * 0.22
        + stage_phrase * 0.11
        + point.masking_opportunity * 0.06
        + emotion * 0.06
        + point.vocal_handoff_score * 0.06
        - distance as f32 * 0.11
        - vocal_penalty
}

pub(crate) fn map_span_is_runtime_eligible(
    span: &MusicMapSpan,
    candidate: &MusicSectionCandidate,
    duration_seconds: f64,
) -> bool {
    map_span_runtime_reject_reason(span, candidate, duration_seconds).is_none()
}

pub(crate) fn select_map_span_for_candidate<'a>(
    manifest: &'a MusicAnalysisManifest,
    candidate: &MusicSectionCandidate,
) -> Option<&'a MusicMapSpan> {
    let duration_seconds = manifest.duration_seconds.max(0.0);
    manifest
        .music_map
        .highlight_span
        .iter()
        .filter(|span| map_span_is_runtime_eligible(span, candidate, duration_seconds))
        .max_by(|a, b| {
            map_span_runtime_score(a, candidate, duration_seconds)
                .partial_cmp(&map_span_runtime_score(b, candidate, duration_seconds))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub(crate) fn rejected_map_span_guard_for_candidate(
    manifest: &MusicAnalysisManifest,
    candidate: &MusicSectionCandidate,
) -> Option<MusicMapSpanRuntimeGuard> {
    let duration_seconds = manifest.duration_seconds.max(0.0);
    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return None;
    }
    manifest
        .music_map
        .highlight_span
        .iter()
        .filter_map(|span| {
            let reason = map_span_runtime_reject_reason(span, candidate, duration_seconds)?;
            let start_seconds = span.start_seconds.clamp(0.0, duration_seconds);
            let end_seconds = span.end_seconds.clamp(start_seconds, duration_seconds);
            Some(MusicMapSpanRuntimeGuard {
                reason,
                start_seconds,
                end_seconds,
                confidence: span.confidence,
                runtime_score: map_span_runtime_score(span, candidate, duration_seconds),
            })
        })
        .max_by(|a, b| {
            a.runtime_score
                .partial_cmp(&b.runtime_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub(crate) fn map_span_runtime_reject_reason(
    span: &MusicMapSpan,
    candidate: &MusicSectionCandidate,
    duration_seconds: f64,
) -> Option<&'static str> {
    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return Some("invalid-duration");
    }
    if span.confidence < MAP_SPAN_RUNTIME_MIN_CONFIDENCE {
        return Some("low-confidence");
    }
    let span_start = span.start_seconds.clamp(0.0, duration_seconds);
    let span_end = span.end_seconds.clamp(span_start, duration_seconds);
    let span_len = span_end - span_start;
    if span_len < MIN_PLAYABLE_SEGMENT_SECONDS * 0.85 {
        return Some("short-span");
    }
    if span_len > PRESENCE_MAX_SECONDS * 1.15 {
        return Some("long-span");
    }
    if !span.lift_seconds.is_finite() || !span.peak_seconds.is_finite() {
        return Some("invalid-anchor");
    }
    if span.peak_seconds < span_start || span.peak_seconds > span_end {
        return Some("peak-outside-span");
    }

    let candidate_start = candidate.start_seconds.clamp(0.0, duration_seconds);
    let candidate_end = candidate
        .end_seconds
        .clamp(candidate_start, duration_seconds.max(candidate_start));
    if candidate_end <= candidate_start + MIN_PLAYABLE_SEGMENT_SECONDS * 0.55 {
        return Some("short-candidate");
    }
    if span.peak_seconds < candidate_start - MAP_SPAN_RUNTIME_PEAK_PAD_SECONDS
        || span.peak_seconds > candidate_end + MAP_SPAN_RUNTIME_PEAK_PAD_SECONDS
    {
        return Some("peak-outside-candidate");
    }

    if map_span_candidate_overlap_ratio(span, candidate, duration_seconds)
        < MAP_SPAN_RUNTIME_MIN_OVERLAP
    {
        return Some("low-overlap");
    }
    if map_span_runtime_score(span, candidate, duration_seconds) < MAP_SPAN_RUNTIME_MIN_SCORE {
        return Some("low-score");
    }
    None
}

pub(crate) fn map_span_candidate_overlap_ratio(
    span: &MusicMapSpan,
    candidate: &MusicSectionCandidate,
    duration_seconds: f64,
) -> f64 {
    let span_start = span.start_seconds.clamp(0.0, duration_seconds);
    let span_end = span.end_seconds.clamp(span_start, duration_seconds);
    let candidate_start = candidate.start_seconds.clamp(0.0, duration_seconds);
    let candidate_end = candidate
        .end_seconds
        .clamp(candidate_start, duration_seconds.max(candidate_start));
    let overlap = (span_end.min(candidate_end) - span_start.max(candidate_start)).max(0.0);
    let shorter = (span_end - span_start)
        .min(candidate_end - candidate_start)
        .max(0.0001);
    (overlap / shorter).clamp(0.0, 1.0)
}

pub(crate) fn map_span_runtime_score(
    span: &MusicMapSpan,
    candidate: &MusicSectionCandidate,
    duration_seconds: f64,
) -> f64 {
    let overlap = map_span_candidate_overlap_ratio(span, candidate, duration_seconds);
    let confidence = f64::from(span.confidence.max(0.0));
    let candidate_score = highlight_candidate_score(candidate);
    let span_len = (span.end_seconds.clamp(0.0, duration_seconds)
        - span.start_seconds.clamp(0.0, duration_seconds))
    .max(0.0);
    let duration_score = map_span_runtime_duration_score(span_len);
    let start_proximity = (1.0
        - ((span.start_seconds - candidate.start_seconds).abs() / 8.0).clamp(0.0, 1.0))
    .clamp(0.0, 1.0);
    let peak_center =
        ((candidate.start_seconds + candidate.end_seconds) * 0.5).clamp(0.0, duration_seconds);
    let candidate_len = (candidate.end_seconds - candidate.start_seconds)
        .abs()
        .max(1.0);
    let peak_proximity = (1.0
        - ((span.peak_seconds - peak_center).abs() / candidate_len).clamp(0.0, 1.0))
    .clamp(0.0, 1.0);

    (confidence * 0.34
        + overlap * 0.30
        + candidate_score * 0.16
        + duration_score * 0.12
        + start_proximity * 0.05
        + peak_proximity * 0.03)
        .clamp(0.0, 1.0)
}

fn map_span_runtime_duration_score(span_len: f64) -> f64 {
    if span_len < MIN_PLAYABLE_SEGMENT_SECONDS {
        (span_len / MIN_PLAYABLE_SEGMENT_SECONDS).clamp(0.0, 1.0)
    } else if span_len <= PRESENCE_TARGET_SECONDS {
        1.0
    } else if span_len <= PRESENCE_MAX_SECONDS {
        (1.0 - (span_len - PRESENCE_TARGET_SECONDS)
            / (PRESENCE_MAX_SECONDS - PRESENCE_TARGET_SECONDS)
            * 0.35)
            .clamp(0.55, 1.0)
    } else {
        0.45
    }
}

pub(crate) fn quick_focus_segment(duration_seconds: f64) -> Option<MusicQuickFocusPlan> {
    if !duration_seconds.is_finite()
        || duration_seconds < PROVISIONAL_HIGHLIGHT_MIN_DURATION_SECONDS
    {
        return None;
    }
    if duration_seconds <= PROVISIONAL_HIGHLIGHT_FULL_RANGE_MAX_SECONDS {
        let segment = MusicPlayableSegment::new(
            0.0,
            duration_seconds,
            MusicPlayableSegmentSource::HighlightQuickEstimate,
        )?;
        return Some(MusicQuickFocusPlan {
            segment,
            confidence: 0.34,
            kind: MusicQuickFocusKind::FullTrackShort,
        });
    }

    let length_share = if duration_seconds < 70.0 {
        PROVISIONAL_HIGHLIGHT_SHORT_LENGTH_SHARE
    } else {
        PROVISIONAL_HIGHLIGHT_LONG_LENGTH_SHARE
    };
    let max_length = (duration_seconds
        - PROVISIONAL_HIGHLIGHT_REMAINING_GUARD_SECONDS
        - MIN_PLAYABLE_SEGMENT_SECONDS)
        .max(MIN_PLAYABLE_SEGMENT_SECONDS);
    let natural_segment_seconds = if duration_seconds < 70.0 {
        duration_seconds * length_share
    } else {
        (duration_seconds * length_share).max(PROVISIONAL_HIGHLIGHT_TARGET_SECONDS)
    };
    let segment_seconds = natural_segment_seconds
        .clamp(
            PROVISIONAL_HIGHLIGHT_MIN_SECONDS,
            PROVISIONAL_HIGHLIGHT_MAX_SECONDS,
        )
        .min(max_length);
    if segment_seconds < MIN_PLAYABLE_SEGMENT_SECONDS {
        return None;
    }

    let preferred_start_share = if duration_seconds < 70.0 {
        PROVISIONAL_HIGHLIGHT_SHORT_START_SHARE
    } else {
        PROVISIONAL_HIGHLIGHT_START_SHARE
    };
    let preferred_start = duration_seconds * preferred_start_share;
    let earliest_start = (duration_seconds * 0.16)
        .clamp(10.0, 34.0)
        .min((duration_seconds - segment_seconds).max(0.0));
    let latest_start =
        (duration_seconds - segment_seconds - PROVISIONAL_HIGHLIGHT_REMAINING_GUARD_SECONDS)
            .max(earliest_start)
            .min((duration_seconds - segment_seconds).max(earliest_start));
    let start_seconds = preferred_start.clamp(earliest_start, latest_start);
    let end_seconds = (start_seconds + segment_seconds).clamp(start_seconds, duration_seconds);

    if end_seconds <= start_seconds + MIN_PLAYABLE_SEGMENT_SECONDS {
        return None;
    }

    let segment = MusicPlayableSegment::new(
        start_seconds,
        end_seconds,
        MusicPlayableSegmentSource::HighlightQuickEstimate,
    )?;
    let duration_confidence = ((duration_seconds - PROVISIONAL_HIGHLIGHT_FULL_RANGE_MAX_SECONDS)
        / 180.0)
        .clamp(0.0, 1.0) as f32;
    Some(MusicQuickFocusPlan {
        segment,
        confidence: (0.42 + duration_confidence * 0.16).clamp(0.0, 0.62),
        kind: MusicQuickFocusKind::FirstMainBody,
    })
}

pub(crate) fn provisional_highlight_segment(duration_seconds: f64) -> Option<MusicPlayableSegment> {
    quick_focus_segment(duration_seconds).map(|plan| plan.segment)
}

pub(crate) fn presence_target_seconds(
    duration_seconds: f64,
    recent_presence_seconds: Option<f64>,
    last_presence_seconds: Option<f64>,
    short_run: u8,
) -> f64 {
    let duration = duration_seconds.max(MIN_PLAYABLE_SEGMENT_SECONDS);
    let max_target = PRESENCE_MAX_SECONDS.min((duration * 0.72).max(MIN_PLAYABLE_SEGMENT_SECONDS));
    let min_target = PRESENCE_MIN_SECONDS.min(max_target);
    let track_target = (duration * 0.24).clamp(min_target, PRESENCE_TARGET_SECONDS.min(max_target));
    let balance_nudge = recent_presence_seconds
        .map(|recent| {
            (PRESENCE_TARGET_SECONDS - recent).clamp(
                -PRESENCE_BALANCE_NUDGE_SECONDS,
                PRESENCE_BALANCE_NUDGE_SECONDS,
            ) * 0.42
        })
        .unwrap_or(0.0);
    let base_target = (track_target + balance_nudge).clamp(min_target, max_target);

    let smoothed_target = presence_delta_smoothed_target_seconds(
        base_target,
        min_target,
        max_target,
        last_presence_seconds,
    );
    let valley_recovery_floor = presence_short_run_recovery_floor(short_run, max_target);

    valley_recovery_floor
        .map(|floor| smoothed_target.max(floor))
        .unwrap_or(smoothed_target)
        .clamp(min_target, max_target)
}

pub(crate) fn presence_seconds_for_fade(
    segment_start_seconds: f64,
    fade_start_seconds: f64,
    fade_duration_seconds: f64,
) -> Option<f64> {
    if !segment_start_seconds.is_finite()
        || !fade_start_seconds.is_finite()
        || !fade_duration_seconds.is_finite()
    {
        return None;
    }
    let audible_until = fade_start_seconds + fade_duration_seconds.max(0.0) * PRESENCE_FADE_SHARE;
    let presence_seconds = (audible_until - segment_start_seconds).max(0.0);
    (presence_seconds >= 1.0).then_some(presence_seconds)
}

pub(crate) fn presence_history_after_finished_segment(
    history: MusicStagePresenceHistory,
    presence_seconds: f64,
) -> Option<MusicStagePresenceHistory> {
    if !presence_seconds.is_finite() || presence_seconds < 1.0 {
        return None;
    }

    let recent_seconds = Some(
        history
            .recent_seconds
            .map(|recent| {
                recent * (1.0 - PRESENCE_EWMA_ALPHA) + presence_seconds * PRESENCE_EWMA_ALPHA
            })
            .unwrap_or(presence_seconds),
    );
    let short_run = if presence_seconds < PRESENCE_SHORT_RUN_THRESHOLD_SECONDS {
        history.short_run.saturating_add(1).min(8)
    } else if presence_seconds >= PRESENCE_SHORT_RUN_RESET_SECONDS {
        0
    } else {
        history.short_run.saturating_sub(1)
    };

    Some(MusicStagePresenceHistory {
        recent_seconds,
        last_seconds: Some(presence_seconds),
        short_run,
    })
}

pub(crate) fn presence_delta_smoothed_target_seconds(
    base_target: f64,
    min_target: f64,
    max_target: f64,
    last_presence_seconds: Option<f64>,
) -> f64 {
    let Some(last_presence) = last_presence_seconds else {
        return base_target.clamp(min_target, max_target);
    };
    let last_presence = last_presence.clamp(MIN_PLAYABLE_SEGMENT_SECONDS, PRESENCE_MAX_SECONDS);

    let target = if last_presence + PRESENCE_DELTA_SOFT_LIMIT_SECONDS < base_target {
        // Previous song was short: do not let the next one suddenly dominate.
        // It may still be longer, but only by a soft step plus a small release.
        let overflow = base_target - last_presence - PRESENCE_DELTA_SOFT_LIMIT_SECONDS;
        last_presence
            + PRESENCE_DELTA_SOFT_LIMIT_SECONDS
            + overflow.max(0.0) * PRESENCE_LONG_AFTER_SHORT_RELEASE_RATIO
    } else if last_presence > base_target + PRESENCE_DELTA_SOFT_LIMIT_SECONDS {
        let deficit = last_presence - base_target - PRESENCE_DELTA_SOFT_LIMIT_SECONDS;
        base_target
            + (deficit.max(0.0) * PRESENCE_AFTER_LONG_NUDGE_RATIO)
                .min(PRESENCE_AFTER_LONG_NUDGE_MAX_SECONDS)
    } else {
        base_target
    };

    target.clamp(min_target, max_target)
}

pub(crate) fn presence_short_run_recovery_floor(short_run: u8, max_target: f64) -> Option<f64> {
    if short_run < 2 || !max_target.is_finite() || max_target <= 0.0 {
        return None;
    }

    // Two short Stage appearances are enough to break the valley. Longer runs
    // raise the floor gradually, but never past the track's own runway.
    let recovery_steps = short_run.saturating_sub(2).min(3);
    let floor = PRESENCE_TARGET_SECONDS
        + f64::from(recovery_steps) * PRESENCE_SHORT_RUN_RECOVERY_STEP_SECONDS;
    Some(
        floor
            .min(PRESENCE_SHORT_RUN_RECOVERY_MAX_SECONDS)
            .min(max_target)
            .max(PRESENCE_MIN_SECONDS.min(max_target)),
    )
}

pub(crate) fn cue_memory_apply_weight(
    confidence: f32,
    effective_presence_seconds: f64,
) -> Option<f64> {
    if confidence < CUE_MEMORY_APPLY_MIN_CONFIDENCE
        || !effective_presence_seconds.is_finite()
        || effective_presence_seconds < MIN_PLAYABLE_SEGMENT_SECONDS
    {
        return None;
    }
    let confidence = confidence.clamp(0.0, 1.0) as f64;
    Some((confidence * CUE_MEMORY_APPLY_MAX_WEIGHT).clamp(0.0, CUE_MEMORY_APPLY_MAX_WEIGHT))
}

pub(crate) fn cue_memory_observation_for_segment(
    segment_start_seconds: f64,
    segment_end_seconds: f64,
    base_start_seconds: f64,
    base_end_seconds: f64,
    effective_presence_seconds: f64,
    updated_unix_seconds: u64,
) -> Option<MusicStageCueMemoryObservation> {
    if !segment_start_seconds.is_finite()
        || !segment_end_seconds.is_finite()
        || !base_start_seconds.is_finite()
        || !base_end_seconds.is_finite()
        || !effective_presence_seconds.is_finite()
        || effective_presence_seconds < MIN_PLAYABLE_SEGMENT_SECONDS
    {
        return None;
    }

    Some(MusicStageCueMemoryObservation {
        start_offset_seconds: (segment_start_seconds - base_start_seconds).clamp(
            -CUE_MEMORY_MAX_START_OFFSET_SECONDS,
            CUE_MEMORY_MAX_START_OFFSET_SECONDS,
        ),
        end_offset_seconds: (segment_end_seconds - base_end_seconds).clamp(
            -CUE_MEMORY_MAX_END_OFFSET_SECONDS,
            CUE_MEMORY_MAX_END_OFFSET_SECONDS,
        ),
        effective_presence_seconds: effective_presence_seconds
            .clamp(MIN_PLAYABLE_SEGMENT_SECONDS, PRESENCE_MAX_SECONDS),
        updated_unix_seconds,
    })
}

pub(crate) fn cue_memory_updated_values(
    previous: MusicStageCueMemoryValues,
    observation: MusicStageCueMemoryObservation,
) -> MusicStageCueMemoryValues {
    let alpha = if previous.updates < 3 {
        CUE_MEMORY_UPDATE_ALPHA_EARLY
    } else {
        CUE_MEMORY_UPDATE_ALPHA_STABLE
    };

    let (start_offset_seconds, end_offset_seconds, effective_presence_seconds) =
        if previous.updates == 0 {
            (
                observation.start_offset_seconds,
                observation.end_offset_seconds,
                observation.effective_presence_seconds,
            )
        } else {
            (
                blend_seconds(
                    previous.start_offset_seconds,
                    observation.start_offset_seconds,
                    alpha,
                ),
                blend_seconds(
                    previous.end_offset_seconds,
                    observation.end_offset_seconds,
                    alpha,
                ),
                blend_seconds(
                    previous.effective_presence_seconds,
                    observation.effective_presence_seconds,
                    alpha,
                ),
            )
        };

    MusicStageCueMemoryValues {
        start_offset_seconds,
        end_offset_seconds,
        effective_presence_seconds,
        confidence: (previous.confidence * 0.88 + CUE_MEMORY_CONFIDENCE_GAIN).clamp(0.0, 1.0),
        updates: previous.updates.saturating_add(1),
        updated_unix_seconds: observation.updated_unix_seconds,
    }
}

pub(crate) fn low_energy_tail_start_seconds(
    manifest: &MusicAnalysisManifest,
    min_segment_seconds: f64,
    min_transition_seconds: f64,
) -> Option<f64> {
    let duration = manifest.duration_seconds;
    if manifest.energy_curve.len() < 6 || !duration.is_finite() {
        return None;
    }

    let track_floor_db = f64::from(manifest.loudness.rms_db) - 4.0;
    let search_start = duration * 0.58;
    let required_low_points = 5_usize;
    let mut low_run = 0_usize;
    let mut run_start = None;

    for point in &manifest.energy_curve {
        if point.time_seconds < search_start {
            continue;
        }
        let rms_db = 20.0 * f64::from(point.rms.max(1.0e-9)).log10();
        if rms_db <= track_floor_db {
            if low_run == 0 {
                run_start = Some(point.time_seconds);
            }
            low_run += 1;
            if low_run >= required_low_points {
                return run_start.filter(|start| {
                    *start >= min_segment_seconds && duration - *start >= min_transition_seconds
                });
            }
        } else {
            low_run = 0;
            run_start = None;
        }
    }

    None
}

pub(crate) fn low_energy_head_end_seconds(manifest: &MusicAnalysisManifest) -> Option<f64> {
    let duration = manifest.duration_seconds;
    if manifest.energy_curve.len() < 4 || !duration.is_finite() {
        return None;
    }

    let audible_floor_db = f64::from(manifest.loudness.rms_db) - 8.0;
    let max_search = TRIM_HEAD_MAX_SECONDS.min(duration * 0.35);
    if max_search <= TRIM_HEAD_LOW_RUN_SECONDS {
        return None;
    }

    let mut audible_run_start = None;
    for point in &manifest.energy_curve {
        if point.time_seconds > max_search {
            break;
        }
        let rms_db = 20.0 * f64::from(point.rms.max(1.0e-9)).log10();
        let audible = rms_db >= audible_floor_db;
        if audible && point.time_seconds < TRIM_HEAD_LOW_RUN_SECONDS {
            return None;
        }
        if audible {
            let run_start = *audible_run_start.get_or_insert(point.time_seconds);
            if point.time_seconds - run_start >= TRIM_HEAD_AUDIBLE_RUN_SECONDS {
                return (run_start >= TRIM_HEAD_LOW_RUN_SECONDS).then_some(run_start);
            }
        } else {
            audible_run_start = None;
        }
    }

    None
}

pub(crate) fn stable_pick_seed(item_id: QueueItemId, serial: u64, session_id: u64) -> u64 {
    splitmix64(item_id ^ serial.rotate_left(17) ^ session_id.rotate_left(29))
}

pub(crate) fn unit_from_seed(seed: u64) -> f64 {
    let value = splitmix64(seed) >> 11;
    (value as f64) / ((1_u64 << 53) as f64)
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = value;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn blend_seconds(old: f64, new: f64, alpha: f64) -> f64 {
    if !old.is_finite() {
        return new;
    }
    if !new.is_finite() {
        return old;
    }
    old * (1.0 - alpha) + new * alpha
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::music_analysis::MusicFunctionalRole;
    use crate::app::music_analysis::{
        MusicEnergyPoint, MusicFunctionalSegment, MusicHarmonicAnalysis, MusicLoudnessAnalysis,
        MusicMapSpan, MusicMixPoint, MusicMixPointAnalysis, MusicSectionAnalysis,
        MusicSectionCandidate, MusicSectionCandidateScores, MusicSectionCurveAnalysis,
        MusicSegmentTempo, MusicStructureAnalysis, MusicTempoAnalysis, StageMixMusicMap,
    };

    fn candidate(start_seconds: f64, end_seconds: f64, confidence: f32) -> MusicSectionCandidate {
        MusicSectionCandidate {
            start_seconds,
            end_seconds,
            confidence,
            reason: "test".to_owned(),
            scores: MusicSectionCandidateScores {
                total: confidence,
                chorusness: 0.80,
                repetition: 0.72,
                energy: 0.62,
                contrast: 0.66,
                boundary: 0.70,
                duration: 0.80,
                segment_wholeness: 0.78,
                perceptual: 0.70,
                structural_recurrence: 0.76,
                ..Default::default()
            },
        }
    }

    fn functional_segment(
        start_seconds: f64,
        end_seconds: f64,
        role: MusicFunctionalRole,
        confidence: f32,
    ) -> MusicFunctionalSegment {
        MusicFunctionalSegment {
            start_seconds,
            end_seconds,
            role,
            confidence,
            reason: "test".to_owned(),
        }
    }

    fn manifest_with_energy(points: &[(f64, f32)]) -> MusicAnalysisManifest {
        MusicAnalysisManifest {
            schema_version: 1,
            analyzer_version: 21,
            media_file_size: 0,
            updated_unix_seconds: 0,
            duration_seconds: 180.0,
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
            energy_curve: points
                .iter()
                .map(|(time_seconds, rms)| MusicEnergyPoint {
                    time_seconds: *time_seconds,
                    rms: *rms,
                    peak: (*rms * 1.4).min(1.0),
                })
                .collect(),
            spectrum_curve: Vec::new(),
        }
    }

    fn manifest_with_candidates(candidates: Vec<MusicSectionCandidate>) -> MusicAnalysisManifest {
        let mut manifest = manifest_with_energy(&[(0.0, 0.12), (1.0, 0.12), (2.0, 0.12)]);
        manifest.duration_seconds = 180.0;
        manifest.sections.highlight_candidates = candidates;
        manifest
    }

    #[test]
    fn candidate_score_prefers_complete_body_over_short_spike() {
        let mut spike = candidate(18.0, 27.0, 0.86);
        spike.scores.chorusness = 0.22;
        spike.scores.repetition = 0.18;
        spike.scores.energy = 0.98;
        spike.scores.contrast = 0.92;
        spike.scores.boundary = 0.24;
        spike.scores.duration = 0.20;
        spike.scores.segment_wholeness = 0.12;
        spike.scores.perceptual = 0.18;
        spike.scores.structural_recurrence = 0.08;

        let complete = candidate(42.0, 76.0, 0.74);

        assert!(highlight_candidate_score(&complete) > highlight_candidate_score(&spike));
    }

    #[test]
    fn provisional_highlight_is_bounded_and_leaves_runway() {
        let segment = provisional_highlight_segment(240.0).expect("segment");

        assert_eq!(
            segment.source,
            MusicPlayableSegmentSource::HighlightQuickEstimate
        );
        assert!(segment.end_seconds - segment.start_seconds >= 34.0);
        assert!(240.0 - segment.end_seconds >= PROVISIONAL_HIGHLIGHT_REMAINING_GUARD_SECONDS);
    }

    #[test]
    fn quick_focus_keeps_short_tracks_as_full_track_bridge() {
        let plan = quick_focus_segment(36.0).expect("plan");

        assert_eq!(plan.kind, MusicQuickFocusKind::FullTrackShort);
        assert_eq!(plan.segment.as_range(), (0.0, 36.0));
        assert!(plan.confidence < 0.40);
    }

    #[test]
    fn quick_focus_marks_long_tracks_as_first_main_body() {
        let plan = quick_focus_segment(240.0).expect("plan");

        assert_eq!(plan.kind, MusicQuickFocusKind::FirstMainBody);
        assert_eq!(
            plan.segment.source,
            MusicPlayableSegmentSource::HighlightQuickEstimate
        );
        assert!(plan.segment.start_seconds > 60.0);
        assert!(plan.segment.end_seconds < 140.0);
        assert!(plan.confidence >= 0.42);
    }

    #[test]
    fn head_detector_skips_only_sustained_low_energy_intro() {
        let manifest = manifest_with_energy(&[
            (0.0, 0.015),
            (1.0, 0.015),
            (2.2, 0.016),
            (3.0, 0.12),
            (4.0, 0.13),
            (5.0, 0.12),
        ]);

        let head_end = low_energy_head_end_seconds(&manifest);
        assert!(head_end.is_some_and(|seconds| (seconds - 3.0).abs() < 0.000_001));
    }

    #[test]
    fn attention_profile_marks_complete_highlight_as_high_attention() {
        let candidate = candidate(42.0, 76.0, 0.74);
        let profile = attention_profile_for_candidate(&candidate);

        assert!(profile.attention_score >= 0.65);
        assert!(profile.highlight_score >= 0.65);
        assert!(profile.edge_trim_score < 0.20);
        assert!(profile.reason_flags.highlight_overlap);
    }

    #[test]
    fn attention_trim_head_uses_empty_intro_edge() {
        let mut manifest = manifest_with_energy(&[
            (0.0, 0.015),
            (3.0, 0.016),
            (8.0, 0.015),
            (12.0, 0.12),
            (18.0, 0.13),
        ]);
        manifest.sections.functional_segments = vec![functional_segment(
            0.0,
            12.0,
            MusicFunctionalRole::Intro,
            0.80,
        )];

        let plan =
            attention_trim_head_end_seconds(&manifest, default_attention_trim_policy(8.0, 5.2))
                .expect("head trim");

        assert_eq!(plan.boundary_seconds, 12.0);
        assert_eq!(plan.reason, MusicAttentionTrimEdgeReason::IntroSection);
        assert!(plan.profile.edge_trim_score >= 0.46);
    }

    #[test]
    fn attention_trim_head_limit_uses_conservative_cap_for_long_intro() {
        let mut manifest =
            manifest_with_energy(&[(0.0, 0.012), (24.0, 0.014), (52.0, 0.16), (120.0, 0.18)]);
        manifest.sections.functional_segments = vec![functional_segment(
            0.0,
            52.0,
            MusicFunctionalRole::Intro,
            0.92,
        )];
        let policy = default_attention_trim_policy(8.0, 5.2);

        assert_eq!(attention_trim_head_limit_seconds(180.0, policy), 24.0);
        assert_eq!(attention_trim_head_end_seconds(&manifest, policy), None);
    }

    #[test]
    fn attention_trim_head_protects_high_attention_intro_overlap() {
        let mut manifest = manifest_with_energy(&[
            (0.0, 0.10),
            (3.0, 0.11),
            (8.0, 0.10),
            (12.0, 0.12),
            (18.0, 0.13),
        ]);
        manifest.sections.functional_segments = vec![functional_segment(
            0.0,
            12.0,
            MusicFunctionalRole::Intro,
            0.80,
        )];
        manifest.sections.highlight_candidates = vec![candidate(0.0, 18.0, 0.82)];

        assert_eq!(
            attention_trim_head_end_seconds(&manifest, default_attention_trim_policy(8.0, 5.2)),
            None
        );
    }

    #[test]
    fn attention_trim_head_keeps_audible_foreground_intro() {
        let mut manifest = manifest_with_energy(&[
            (0.0, 0.11),
            (3.0, 0.12),
            (8.0, 0.13),
            (14.0, 0.14),
            (24.0, 0.14),
        ]);
        manifest.sections.functional_segments = vec![functional_segment(
            0.0,
            14.0,
            MusicFunctionalRole::Intro,
            0.92,
        )];

        assert_eq!(
            attention_trim_head_end_seconds(&manifest, default_attention_trim_policy(8.0, 5.2)),
            None
        );
    }

    #[test]
    fn attention_trim_head_ignores_low_confidence_intro_label() {
        let mut manifest = manifest_with_energy(&[
            (0.0, 0.12),
            (3.0, 0.12),
            (8.0, 0.13),
            (12.0, 0.13),
            (24.0, 0.14),
        ]);
        manifest.sections.functional_segments = vec![functional_segment(
            0.0,
            12.0,
            MusicFunctionalRole::Intro,
            0.18,
        )];

        assert_eq!(
            attention_trim_head_end_seconds(&manifest, default_attention_trim_policy(8.0, 5.2)),
            None
        );
    }

    #[test]
    fn attention_trim_tail_uses_empty_outro_edge() {
        let mut manifest = manifest_with_energy(&[
            (0.0, 0.12),
            (90.0, 0.12),
            (132.0, 0.015),
            (150.0, 0.014),
            (178.0, 0.014),
        ]);
        manifest.sections.functional_segments = vec![functional_segment(
            132.0,
            180.0,
            MusicFunctionalRole::Outro,
            0.82,
        )];

        let plan =
            attention_trim_tail_start_seconds(&manifest, default_attention_trim_policy(8.0, 5.2))
                .expect("tail trim");

        assert_eq!(plan.boundary_seconds, 132.0);
        assert_eq!(plan.reason, MusicAttentionTrimEdgeReason::OutroSection);
        assert!(plan.profile.edge_trim_score >= 0.46);
    }

    #[test]
    fn attention_trim_tail_ignores_low_confidence_outro_label() {
        let mut manifest =
            manifest_with_energy(&[(0.0, 0.14), (80.0, 0.15), (132.0, 0.14), (178.0, 0.13)]);
        manifest.sections.functional_segments = vec![functional_segment(
            132.0,
            180.0,
            MusicFunctionalRole::Outro,
            0.18,
        )];

        assert_eq!(
            attention_trim_tail_start_seconds(&manifest, default_attention_trim_policy(8.0, 5.2)),
            None
        );
    }

    #[test]
    fn attention_trim_range_plan_combines_head_and_tail_projection() {
        let mut manifest = manifest_with_energy(&[
            (0.0, 0.015),
            (8.0, 0.015),
            (16.0, 0.12),
            (90.0, 0.12),
            (132.0, 0.015),
            (178.0, 0.014),
        ]);
        manifest.sections.functional_segments = vec![
            functional_segment(0.0, 16.0, MusicFunctionalRole::Intro, 0.80),
            functional_segment(132.0, 180.0, MusicFunctionalRole::Outro, 0.82),
        ];

        let plan = attention_trim_range_plan(&manifest, default_attention_trim_policy(8.0, 5.2))
            .expect("range plan");

        assert_eq!(plan.start_seconds, 16.0);
        assert_eq!(plan.end_seconds, 132.0);
        assert_eq!(
            plan.head.map(|edge| edge.reason),
            Some(MusicAttentionTrimEdgeReason::IntroSection)
        );
        assert_eq!(
            plan.tail.map(|edge| edge.reason),
            Some(MusicAttentionTrimEdgeReason::OutroSection)
        );
    }

    #[test]
    fn attention_zones_promote_highlight_to_focus() {
        let manifest = manifest_with_candidates(vec![candidate(42.0, 76.0, 0.74)]);

        let zones = attention_zones_for_manifest(&manifest);

        assert!(zones.iter().any(|zone| {
            zone.kind == MusicAttentionZoneKind::Focus
                && zone.start_seconds <= 42.0
                && zone.end_seconds >= 76.0
                && zone.score >= ATTENTION_ZONE_FOCUS_MIN_SCORE
        }));
    }

    #[test]
    fn best_focus_zone_can_use_functional_chorus_without_candidates() {
        let mut manifest =
            manifest_with_energy(&[(0.0, 0.16), (48.0, 0.24), (68.0, 0.28), (92.0, 0.22)]);
        manifest.sections.functional_segments = vec![functional_segment(
            48.0,
            92.0,
            MusicFunctionalRole::Chorus,
            0.84,
        )];

        let zone = best_focus_zone_for_manifest(&manifest).expect("focus zone");

        assert_eq!(zone.kind, MusicAttentionZoneKind::Focus);
        assert_eq!(zone.start_seconds, 48.0);
        assert_eq!(zone.end_seconds, 92.0);
        assert!(zone.score >= ATTENTION_FOCUS_RUNTIME_MIN_SCORE);
    }

    #[test]
    fn attention_highlight_plan_accepts_strong_standalone_focus() {
        let mut manifest =
            manifest_with_energy(&[(0.0, 0.16), (48.0, 0.24), (68.0, 0.28), (92.0, 0.22)]);
        manifest.sections.functional_segments = vec![functional_segment(
            48.0,
            92.0,
            MusicFunctionalRole::FinalChorus,
            0.88,
        )];

        let plan = attention_highlight_range_plan(&manifest, None).expect("plan");

        assert_eq!(plan.source, MusicAttentionHighlightRangeSource::FocusZone);
        assert_eq!(plan.start_seconds, 48.0);
        assert_eq!(plan.end_seconds, 92.0);
        assert_eq!(plan.selection_reason, "focus-accepted");
        assert!(plan.selection_confidence >= ATTENTION_FOCUS_RUNTIME_STANDALONE_SCORE);
        assert!(plan.guard_reason.is_none());
    }

    #[test]
    fn focus_zone_for_candidate_requires_runtime_overlap() {
        let mut manifest =
            manifest_with_energy(&[(0.0, 0.16), (40.0, 0.26), (60.0, 0.30), (120.0, 0.24)]);
        let selected = candidate(40.0, 78.0, 0.80);
        manifest.sections.highlight_candidates =
            vec![selected.clone(), candidate(112.0, 146.0, 0.78)];
        manifest.sections.functional_segments = vec![
            functional_segment(40.0, 82.0, MusicFunctionalRole::Chorus, 0.84),
            functional_segment(112.0, 146.0, MusicFunctionalRole::FinalChorus, 0.86),
        ];

        let zone = focus_zone_for_candidate(&manifest, &selected).expect("focus zone");

        assert_eq!(zone.kind, MusicAttentionZoneKind::Focus);
        assert!(zone.start_seconds <= selected.start_seconds + 0.001);
        assert!(zone.end_seconds >= selected.end_seconds - 0.001);
        assert!(zone.start_seconds < 100.0);
    }

    #[test]
    fn attention_highlight_plan_accepts_strong_focus_without_safer_map() {
        let selected = candidate(40.0, 78.0, 0.80);
        let manifest = manifest_with_candidates(vec![selected.clone()]);

        let plan = attention_highlight_range_plan(&manifest, Some(&selected)).expect("plan");

        assert_eq!(plan.source, MusicAttentionHighlightRangeSource::FocusZone);
        assert_eq!(plan.source.log_key(), "focus-zone");
        assert_eq!(plan.selection_reason, "focus-accepted");
        assert!(plan.focus_score.is_some());
        assert!(plan.guard_reason.is_none());
    }

    #[test]
    fn attention_highlight_plan_uses_accepted_focus_when_top_focus_is_guarded() {
        let selected = candidate(50.0, 90.0, 0.55);
        let risky_overlap = candidate(25.0, 70.0, 0.95);
        let accepted_overlap = candidate(52.0, 86.0, 0.58);
        let manifest = manifest_with_candidates(vec![risky_overlap, accepted_overlap]);

        let plan = attention_highlight_range_plan(&manifest, Some(&selected)).expect("plan");

        assert_eq!(plan.source, MusicAttentionHighlightRangeSource::FocusZone);
        assert_eq!(plan.selection_reason, "focus-accepted");
        assert!(plan.start_seconds >= 50.0);
        assert!(plan.end_seconds <= 90.0);
        assert!(plan.focus_overlap.is_some_and(|overlap| overlap >= 0.92));
    }

    #[test]
    fn attention_highlight_plan_keeps_safer_music_map_over_focus() {
        let selected = candidate(40.0, 78.0, 0.80);
        let mut manifest = manifest_with_candidates(vec![selected.clone()]);
        manifest.music_map.highlight_span = vec![MusicMapSpan {
            start_seconds: 42.0,
            lift_seconds: 48.0,
            peak_seconds: 61.0,
            end_seconds: 76.0,
            confidence: 0.88,
            reason_zh: "aligned".to_owned(),
            listen_from_seconds: 42.0,
        }];

        let plan = attention_highlight_range_plan(&manifest, Some(&selected)).expect("plan");

        assert_eq!(
            plan.source,
            MusicAttentionHighlightRangeSource::MusicMapSpan
        );
        assert_eq!(
            plan.guarded_source,
            Some(MusicAttentionHighlightRangeSource::FocusZone)
        );
        assert_eq!(plan.guard_reason, Some("safer-map-span"));
        assert_eq!(plan.selection_reason, "map-safer-than-focus");
        assert!(plan.selection_confidence >= MAP_SPAN_RUNTIME_MIN_SCORE as f32);
    }

    #[test]
    fn attention_highlight_plan_uses_music_map_when_focus_is_weak() {
        let mut selected = candidate(40.0, 78.0, 0.80);
        selected.scores.chorusness = 0.0;
        selected.scores.repetition = 0.0;
        selected.scores.energy = 0.0;
        selected.scores.contrast = 0.0;
        selected.scores.boundary = 0.0;
        selected.scores.duration = 0.0;
        selected.scores.segment_wholeness = 0.0;
        selected.scores.perceptual = 0.0;
        selected.scores.structural_recurrence = 0.0;
        let mut manifest = manifest_with_candidates(vec![selected.clone()]);
        manifest.music_map.highlight_span = vec![MusicMapSpan {
            start_seconds: 42.0,
            lift_seconds: 48.0,
            peak_seconds: 61.0,
            end_seconds: 76.0,
            confidence: 0.98,
            reason_zh: "aligned".to_owned(),
            listen_from_seconds: 42.0,
        }];

        let plan = attention_highlight_range_plan(&manifest, Some(&selected)).expect("plan");

        assert_eq!(
            plan.source,
            MusicAttentionHighlightRangeSource::MusicMapSpan
        );
        assert_eq!(plan.start_seconds, 42.0);
        assert_eq!(plan.end_seconds, 76.0);
        assert_eq!(plan.selection_reason, "map-runtime-eligible");
        assert!(plan.map_runtime_score.is_some());
    }

    #[test]
    fn attention_highlight_plan_falls_back_to_candidate_when_attention_is_low() {
        let mut selected = candidate(40.0, 78.0, 0.44);
        selected.scores.chorusness = 0.0;
        selected.scores.repetition = 0.0;
        selected.scores.energy = 0.12;
        selected.scores.contrast = 0.0;
        selected.scores.boundary = 0.0;
        selected.scores.duration = 0.20;
        selected.scores.segment_wholeness = 0.0;
        selected.scores.perceptual = 0.0;
        selected.scores.structural_recurrence = 0.0;
        let manifest = manifest_with_candidates(vec![selected.clone()]);

        let plan = attention_highlight_range_plan(&manifest, Some(&selected)).expect("plan");

        assert_eq!(plan.source, MusicAttentionHighlightRangeSource::Candidate);
        assert_eq!(plan.start_seconds, selected.start_seconds);
        assert_eq!(plan.end_seconds, selected.end_seconds);
        assert_eq!(plan.selection_reason, "candidate-fallback");
    }

    #[test]
    fn attention_highlight_plan_reports_rejected_music_map_reason() {
        let mut selected = candidate(40.0, 78.0, 0.44);
        selected.scores.chorusness = 0.0;
        selected.scores.repetition = 0.0;
        selected.scores.energy = 0.12;
        selected.scores.contrast = 0.0;
        selected.scores.boundary = 0.0;
        selected.scores.duration = 0.20;
        selected.scores.segment_wholeness = 0.0;
        selected.scores.perceptual = 0.0;
        selected.scores.structural_recurrence = 0.0;
        let mut manifest = manifest_with_candidates(vec![selected.clone()]);
        manifest.music_map.highlight_span = vec![MusicMapSpan {
            start_seconds: 42.0,
            lift_seconds: 48.0,
            peak_seconds: 61.0,
            end_seconds: 76.0,
            confidence: 0.40,
            reason_zh: "low-confidence".to_owned(),
            listen_from_seconds: 42.0,
        }];

        let plan = attention_highlight_range_plan(&manifest, Some(&selected)).expect("plan");

        assert_eq!(plan.source, MusicAttentionHighlightRangeSource::Candidate);
        assert_eq!(plan.rejected_map_reason, Some("low-confidence"));
        assert_eq!(plan.rejected_map_confidence, Some(0.40));
        assert_eq!(plan.selection_reason, "candidate-after-map-reject");
        assert!(plan.rejected_map_runtime_score.is_some());
    }

    #[test]
    fn focus_runtime_gate_rejects_edge_risk_without_strong_attention() {
        let profile = MusicAttentionProfile {
            start_seconds: 0.0,
            end_seconds: 18.0,
            attention_score: 0.46,
            emptiness_score: 0.52,
            edge_trim_score: 0.60,
            highlight_score: 0.24,
            structural_score: 0.28,
            energy_score: 0.20,
            boundary_strength: 0.20,
            entry_quality: 0.0,
            exit_quality: 0.0,
            reason_flags: MusicAttentionReasonFlags {
                edge_role: true,
                ..Default::default()
            },
        };
        let zone = MusicAttentionZone {
            start_seconds: 0.0,
            end_seconds: 18.0,
            kind: MusicAttentionZoneKind::Focus,
            score: 0.72,
            profile,
        };

        assert_eq!(
            attention_focus_zone_runtime_reject_reason(zone, None, 0.72, 1.0, None, None),
            Some("edge-risk")
        );
    }

    #[test]
    fn attention_zones_mark_empty_head_and_tail_risk() {
        let mut manifest = manifest_with_energy(&[
            (0.0, 0.015),
            (8.0, 0.015),
            (16.0, 0.12),
            (90.0, 0.12),
            (132.0, 0.015),
            (178.0, 0.014),
        ]);
        manifest.sections.functional_segments = vec![
            functional_segment(0.0, 16.0, MusicFunctionalRole::Intro, 0.80),
            functional_segment(132.0, 180.0, MusicFunctionalRole::Outro, 0.82),
        ];

        let zones = attention_zones_for_manifest(&manifest);

        assert!(zones.iter().any(|zone| {
            zone.kind == MusicAttentionZoneKind::Empty
                && zone.start_seconds <= 0.001
                && zone.end_seconds >= 16.0
        }));
        assert!(zones.iter().any(|zone| {
            zone.kind == MusicAttentionZoneKind::TailRisk
                && zone.start_seconds <= 132.0
                && zone.end_seconds >= 180.0
        }));
    }

    #[test]
    fn attention_zones_do_not_promote_low_confidence_chorus_to_focus() {
        let mut manifest =
            manifest_with_energy(&[(0.0, 0.16), (48.0, 0.20), (68.0, 0.22), (92.0, 0.20)]);
        manifest.sections.functional_segments = vec![functional_segment(
            48.0,
            92.0,
            MusicFunctionalRole::Chorus,
            0.20,
        )];

        let zones = attention_zones_for_manifest(&manifest);

        assert!(!zones.iter().any(|zone| {
            zone.kind == MusicAttentionZoneKind::Focus
                && zone.start_seconds <= 48.0
                && zone.end_seconds >= 92.0
        }));
    }

    #[test]
    fn attention_zones_ignore_low_confidence_outro_without_empty_evidence() {
        let mut manifest =
            manifest_with_energy(&[(0.0, 0.18), (90.0, 0.20), (132.0, 0.22), (178.0, 0.20)]);
        manifest.sections.functional_segments = vec![functional_segment(
            132.0,
            180.0,
            MusicFunctionalRole::Outro,
            0.12,
        )];

        let zones = attention_zones_for_manifest(&manifest);

        assert!(!zones.iter().any(|zone| {
            zone.kind == MusicAttentionZoneKind::TailRisk
                && zone.start_seconds <= 132.0
                && zone.end_seconds >= 180.0
        }));
    }

    #[test]
    fn attention_zones_expose_entry_and_exit_mix_points() {
        let mut manifest = manifest_with_candidates(vec![candidate(42.0, 76.0, 0.74)]);
        manifest.mix_points.mix_in = vec![mix_point(18.0, 0.82, 0.78)];
        manifest.mix_points.mix_out = vec![mix_point(118.0, 0.80, 0.76)];

        let zones = attention_zones_for_manifest(&manifest);

        assert!(zones.iter().any(|zone| {
            zone.kind == MusicAttentionZoneKind::Entry
                && zone.start_seconds < 18.0
                && zone.end_seconds > 18.0
        }));
        assert!(zones.iter().any(|zone| {
            zone.kind == MusicAttentionZoneKind::Exit
                && zone.start_seconds < 118.0
                && zone.end_seconds > 118.0
        }));
        assert!(
            zones
                .iter()
                .any(|zone| zone.kind == MusicAttentionZoneKind::MixSafe)
        );
    }

    #[test]
    fn attention_zones_do_not_let_intro_label_override_focus_evidence() {
        let mut manifest =
            manifest_with_energy(&[(0.0, 0.11), (6.0, 0.12), (12.0, 0.12), (30.0, 0.11)]);
        manifest.sections.functional_segments = vec![functional_segment(
            0.0,
            18.0,
            MusicFunctionalRole::Intro,
            0.90,
        )];
        manifest.sections.highlight_candidates = vec![candidate(0.0, 18.0, 0.82)];

        let zones = attention_zones_for_manifest(&manifest);

        assert!(zones.iter().any(|zone| {
            zone.kind == MusicAttentionZoneKind::Focus
                && zone.start_seconds <= 0.001
                && zone.end_seconds >= 18.0
        }));
        assert!(!zones.iter().any(|zone| {
            zone.kind == MusicAttentionZoneKind::Empty
                && zone.start_seconds <= 0.001
                && zone.end_seconds >= 18.0
        }));
    }

    #[test]
    fn pick_seed_is_stable_for_same_inputs() {
        assert_eq!(stable_pick_seed(42, 7, 3), stable_pick_seed(42, 7, 3));
        assert_ne!(stable_pick_seed(42, 7, 3), stable_pick_seed(42, 8, 3));
    }

    #[test]
    fn highlight_pick_ignores_tiny_loud_spike_when_full_body_exists() {
        let mut spike = candidate(20.0, 25.0, 0.92);
        spike.scores.chorusness = 0.12;
        spike.scores.repetition = 0.10;
        spike.scores.energy = 0.98;
        spike.scores.segment_wholeness = 0.10;

        let body = candidate(58.0, 92.0, 0.76);
        let manifest = manifest_with_candidates(vec![spike, body]);
        let pick = select_highlight_pick(&manifest, stable_pick_seed(7, 1, 1)).expect("pick");

        assert_eq!(pick.candidate_index, 1);
        assert_eq!(pick.start_seconds, 58.0);
        assert_eq!(pick.end_seconds, 92.0);
    }

    #[test]
    fn best_highlight_candidate_uses_selector_score_not_raw_confidence() {
        let mut spike = candidate(20.0, 25.0, 0.94);
        spike.scores.chorusness = 0.10;
        spike.scores.repetition = 0.10;
        spike.scores.energy = 0.98;
        spike.scores.segment_wholeness = 0.08;

        let body = candidate(58.0, 92.0, 0.72);
        let manifest = manifest_with_candidates(vec![spike, body]);
        let selected = best_highlight_candidate(&manifest).expect("candidate");

        assert_eq!(selected.start_seconds, 58.0);
        assert_eq!(selected.end_seconds, 92.0);
    }

    #[test]
    fn direct_body_highlight_policy_penalizes_tail_overflow() {
        let body = candidate(58.0, 92.0, 0.76);
        let mut tail = candidate(100.0, 136.0, 0.86);
        tail.scores.chorusness = 0.86;
        tail.scores.repetition = 0.80;
        tail.scores.segment_wholeness = 0.80;
        let mut manifest = manifest_with_candidates(vec![body, tail]);
        manifest.duration_seconds = 160.0;

        let selected = select_direct_body_highlight_candidate(
            &manifest,
            MusicDirectBodyHighlightPolicy {
                body_fence_seconds: 112.0,
                duration_seconds: 160.0,
                latest_start_seconds: 108.0,
                min_segment_seconds: 8.0,
                min_confidence: 0.20,
                tail_grace_seconds: 2.0,
                late_midpoint_share: 0.78,
            },
        )
        .expect("candidate");

        assert_eq!(selected.start_seconds, 58.0);
        assert_eq!(selected.end_seconds, 92.0);
    }

    #[test]
    fn tempo_scale_clamps_and_rejects_invalid_multipliers() {
        assert_eq!(scale_tempo_rate(1.08, 0.0, 0.94, 1.06), 1.0);
        assert_eq!(scale_tempo_rate(f64::NAN, 0.5, 0.94, 1.06), 1.0);

        let scaled = scale_tempo_rate(1.12, 0.5, 0.94, 1.06);
        assert!((scaled - 1.06).abs() < 0.000_001);

        let lowered = scale_tempo_rate(0.90, 0.5, 0.94, 1.06);
        assert!((lowered - 0.95).abs() < 0.000_001);
    }

    #[test]
    fn mix_length_multiplier_preserves_model_at_fifty_percent() {
        let policy = MusicStageMixLengthMultiplierPolicy {
            short_multiplier: 0.58,
            long_multiplier: 1.92,
        };

        assert!((mix_length_multiplier(0.50, policy) - 1.0).abs() < 0.000_001);
        assert!(mix_length_multiplier(0.0, policy) < 1.0);
        assert!(mix_length_multiplier(1.0, policy) > 1.0);
    }

    #[test]
    fn tempo_bridge_strength_and_bounds_expand_monotonically() {
        let strength_policy = MusicStageTempoBridgeStrengthPolicy {
            min_multiplier: 0.0,
            max_multiplier: 3.2,
        };
        assert_eq!(tempo_bridge_strength_multiplier(0.0, strength_policy), 0.0);
        assert!(
            tempo_bridge_strength_multiplier(1.0, strength_policy)
                > tempo_bridge_strength_multiplier(0.35, strength_policy)
        );

        let bounds_policy = MusicStageTempoBridgeRateBoundsPolicy {
            incoming_soft_max_delta: 0.006,
            incoming_strong_max_delta: 0.072,
            outgoing_soft_max_delta: 0.003,
            outgoing_strong_max_delta: 0.052,
        };
        let (soft_outgoing, soft_incoming) = tempo_bridge_rate_bounds(0.0, bounds_policy);
        let (strong_outgoing, strong_incoming) = tempo_bridge_rate_bounds(1.0, bounds_policy);

        assert!(strong_incoming.1 - 1.0 > soft_incoming.1 - 1.0);
        assert!(1.0 - strong_outgoing.0 > 1.0 - soft_outgoing.0);
    }

    #[test]
    fn post_handoff_guard_extends_short_segments_but_caps_at_duration() {
        let policy = MusicStagePostHandoffGuardPolicy {
            advance_guard_seconds: 0.12,
            post_handoff_breathe_seconds: 38.0,
            transition_min_seconds: 2.8,
            transition_max_seconds: 3.8,
        };

        let extended = post_handoff_guarded_end_seconds(100.0, 110.0, 200.0, policy);
        assert!((extended - 141.8).abs() < 0.000_001);

        let capped = post_handoff_guarded_end_seconds(100.0, 110.0, 130.0, policy);
        assert!((capped - 129.88).abs() < 0.000_001);
    }

    #[test]
    fn tail_guarded_exit_pulls_mix_out_before_track_tail() {
        let policy = MusicStageTailExitGuardPolicy {
            exit_tail_guard_seconds: 16.0,
            transition_min_seconds: 2.8,
            advance_guard_seconds: 0.12,
        };

        let guarded =
            tail_guarded_exit_end_seconds(20.0, 188.0, 4.0, Some(200.0), policy).expect("guard");
        assert!((guarded - 184.0).abs() < 0.000_001);

        assert_eq!(
            tail_guarded_exit_end_seconds(20.0, 170.0, 4.0, Some(200.0), policy),
            None
        );
    }

    #[test]
    fn direct_latest_entry_start_keeps_required_runway() {
        let latest = direct_latest_entry_start_seconds(
            220.0,
            5.0,
            MusicStageLatestEntryPolicy {
                min_remaining_seconds: 54.0,
                post_promote_min_dwell_seconds: 34.0,
                extra_runway_seconds: 8.0,
            },
        );

        assert!((latest - 166.0).abs() < 0.000_001);
    }

    #[test]
    fn direct_entry_anchor_score_prefers_near_musical_body_over_intro_or_tail() {
        let chorus = functional_segment(60.0, 92.0, MusicFunctionalRole::Chorus, 0.78);
        let verse = functional_segment(59.0, 84.0, MusicFunctionalRole::Verse, 0.86);
        let intro = functional_segment(58.0, 70.0, MusicFunctionalRole::Intro, 0.96);
        let outro = functional_segment(60.0, 72.0, MusicFunctionalRole::Outro, 0.99);

        let target = 60.0;
        let chorus_score = direct_entry_anchor_score(&chorus, target);
        assert!(chorus_score < direct_entry_anchor_score(&verse, target));
        assert!(chorus_score < direct_entry_anchor_score(&intro, target));
        assert!(direct_entry_anchor_score(&outro, target) > 7.0);
    }

    #[test]
    fn safe_entry_start_pulls_back_when_remaining_is_short() {
        let policy = MusicStageSafeEntryPolicy {
            lite_enabled: true,
            direct_stream: true,
            direct_min_remaining_seconds: 54.0,
            fallback_min_remaining_seconds: 84.0,
            direct_song_share: 0.34,
            fallback_song_share: 0.48,
            promoted_deck_target_seconds: 96.0,
            post_promote_min_dwell_seconds: 34.0,
            extra_runway_seconds: 8.0,
        };

        let pulled = safe_entry_start_seconds(190.0, 5.0, Some(220.0), policy);
        assert!((pulled - 145.2).abs() < 0.000_001);

        let disabled = safe_entry_start_seconds(
            190.0,
            5.0,
            Some(220.0),
            MusicStageSafeEntryPolicy {
                lite_enabled: false,
                ..policy
            },
        );
        assert_eq!(disabled, 190.0);
    }

    #[test]
    fn presence_target_uses_short_run_floor_after_repeated_short_appearances() {
        let target = presence_target_seconds(240.0, Some(18.0), Some(16.0), 4);

        assert!(target >= PRESENCE_TARGET_SECONDS + 8.0);
        assert!(target <= PRESENCE_MAX_SECONDS);
    }

    #[test]
    fn presence_delta_smoother_prevents_one_short_track_from_overcorrecting() {
        let smoothed =
            presence_delta_smoothed_target_seconds(56.0, PRESENCE_MIN_SECONDS, 74.0, Some(18.0));

        assert!(smoothed > 31.0);
        assert!(smoothed < 56.0);
    }

    #[test]
    fn presence_history_updates_recent_and_short_run_without_runtime_state() {
        let presence = presence_seconds_for_fade(10.0, 40.0, 10.0).expect("presence");
        assert!((presence - 34.2).abs() < 0.000_001);

        let history = presence_history_after_finished_segment(
            MusicStagePresenceHistory {
                recent_seconds: Some(20.0),
                last_seconds: Some(20.0),
                short_run: 1,
            },
            18.0,
        )
        .expect("history");

        assert_eq!(history.last_seconds, Some(18.0));
        assert_eq!(history.short_run, 2);
        assert!(history.recent_seconds.is_some_and(|recent| recent < 20.0));

        let reset = presence_history_after_finished_segment(history, 34.0).expect("reset");
        assert_eq!(reset.short_run, 0);
    }

    #[test]
    fn cue_memory_apply_weight_requires_confident_playable_presence() {
        assert_eq!(cue_memory_apply_weight(0.10, 40.0), None);
        assert_eq!(cue_memory_apply_weight(0.50, 4.0), None);

        let weight = cue_memory_apply_weight(0.50, 40.0).expect("weight");
        assert!((weight - 0.24).abs() < 0.000_001);
    }

    #[test]
    fn cue_memory_update_clamps_offsets_and_blends_after_first_observation() {
        let observation = cue_memory_observation_for_segment(20.0, 92.0, 40.0, 84.0, 90.0, 123)
            .expect("observation");
        assert_eq!(observation.start_offset_seconds, -16.0);
        assert_eq!(observation.end_offset_seconds, 4.0);
        assert_eq!(observation.effective_presence_seconds, PRESENCE_MAX_SECONDS);

        let first = cue_memory_updated_values(MusicStageCueMemoryValues::default(), observation);
        assert_eq!(first.start_offset_seconds, -16.0);
        assert_eq!(first.end_offset_seconds, 4.0);
        assert_eq!(first.updates, 1);
        assert_eq!(first.updated_unix_seconds, 123);

        let second_observation =
            cue_memory_observation_for_segment(48.0, 70.0, 40.0, 84.0, 20.0, 456).expect("second");
        let second = cue_memory_updated_values(first, second_observation);
        assert!(second.start_offset_seconds > -16.0);
        assert!(second.end_offset_seconds < 4.0);
        assert!(second.effective_presence_seconds < PRESENCE_MAX_SECONDS);
        assert_eq!(second.updates, 2);
        assert_eq!(second.updated_unix_seconds, 456);
        assert!(second.confidence > first.confidence);
    }

    #[test]
    fn map_span_gate_accepts_aligned_confident_span() {
        let candidate = candidate(40.0, 78.0, 0.80);
        let span = MusicMapSpan {
            start_seconds: 42.0,
            lift_seconds: 48.0,
            peak_seconds: 61.0,
            end_seconds: 76.0,
            confidence: 0.88,
            reason_zh: "test".to_owned(),
            listen_from_seconds: 42.0,
        };

        assert!(map_span_is_runtime_eligible(&span, &candidate, 140.0));
        assert!(map_span_candidate_overlap_ratio(&span, &candidate, 140.0) > 0.85);
    }

    #[test]
    fn map_span_score_prefers_aligned_peak_over_loose_span() {
        let candidate = candidate(40.0, 78.0, 0.80);
        let aligned = MusicMapSpan {
            start_seconds: 42.0,
            lift_seconds: 48.0,
            peak_seconds: 61.0,
            end_seconds: 76.0,
            confidence: 0.88,
            reason_zh: "aligned".to_owned(),
            listen_from_seconds: 42.0,
        };
        let loose = MusicMapSpan {
            start_seconds: 18.0,
            lift_seconds: 22.0,
            peak_seconds: 40.0,
            end_seconds: 64.0,
            confidence: 0.88,
            reason_zh: "loose".to_owned(),
            listen_from_seconds: 18.0,
        };

        assert!(
            map_span_runtime_score(&aligned, &candidate, 140.0)
                > map_span_runtime_score(&loose, &candidate, 140.0)
        );
    }

    #[test]
    fn map_span_selector_picks_best_runtime_eligible_span() {
        let candidate = candidate(40.0, 78.0, 0.80);
        let far = MusicMapSpan {
            start_seconds: 90.0,
            lift_seconds: 92.0,
            peak_seconds: 96.0,
            end_seconds: 104.0,
            confidence: 0.96,
            reason_zh: "far".to_owned(),
            listen_from_seconds: 90.0,
        };
        let aligned = MusicMapSpan {
            start_seconds: 42.0,
            lift_seconds: 48.0,
            peak_seconds: 61.0,
            end_seconds: 76.0,
            confidence: 0.82,
            reason_zh: "aligned".to_owned(),
            listen_from_seconds: 42.0,
        };
        let mut manifest = manifest_with_candidates(vec![candidate.clone()]);
        manifest.music_map.highlight_span = vec![far, aligned];

        let selected = select_map_span_for_candidate(&manifest, &candidate).expect("span");

        assert_eq!(selected.reason_zh, "aligned");
    }

    fn mix_point(time_seconds: f64, confidence: f32, vocal_safety: f32) -> MusicMixPoint {
        MusicMixPoint {
            time_seconds,
            confidence,
            reason: "test".to_owned(),
            phrase_snap_seconds: time_seconds,
            vocal_safety,
            perceptual_score: vocal_safety,
            phrase_closure: 0.60,
            masking_opportunity: 0.50,
            attention_safety: 0.50,
            expectation_safety: 0.50,
            phrase_grid_fit: 0.60,
            emotional_continuity: 0.50,
            vocal_handoff_score: vocal_safety,
        }
    }

    #[test]
    fn full_mix_out_uses_earliest_late_safety_cap() {
        let mut manifest = manifest_with_candidates(Vec::new());
        manifest.sections.functional_segments = vec![functional_segment(
            132.0,
            180.0,
            MusicFunctionalRole::Outro,
            0.80,
        )];
        manifest.mix_points.mix_out = vec![mix_point(118.0, 0.72, 0.62)];

        let selected = full_mix_out_seconds(&manifest, 8.0, 3.0).expect("mix out");

        assert_eq!(selected, 118.0);
    }

    #[test]
    fn direct_body_fence_stays_before_outro_with_backoff() {
        let mut manifest = manifest_with_candidates(Vec::new());
        manifest.sections.functional_segments = vec![functional_segment(
            132.0,
            180.0,
            MusicFunctionalRole::Outro,
            0.82,
        )];

        let fence = direct_body_fence_seconds(
            &manifest,
            4.0,
            MusicStageBodyFencePolicy {
                min_segment_seconds: 8.0,
                transition_min_seconds: 2.8,
                min_remaining_seconds: 42.0,
                post_promote_min_dwell_seconds: 34.0,
                song_share: 0.78,
                outro_backoff_seconds: 4.0,
            },
        )
        .expect("fence");

        assert_eq!(fence, 128.0);
    }

    #[test]
    fn last_audible_energy_point_ignores_low_energy_tail() {
        let mut manifest = manifest_with_energy(&[
            (0.0, 0.10),
            (60.0, 0.10),
            (100.0, 0.05),
            (101.0, 0.002),
            (102.0, 0.001),
            (103.0, 0.001),
            (104.0, 0.001),
        ]);
        manifest.duration_seconds = 120.0;

        let last = last_audible_seconds_from_energy(
            &manifest,
            120.0,
            MusicStageEnergyTailPolicy {
                min_segment_seconds: 8.0,
                relative_rms: 0.055,
                peak_rms: 0.032,
                min_rms: 0.0018,
            },
        )
        .expect("last audible");

        assert_eq!(last, 102.0);
    }

    #[test]
    fn body_fence_safe_exit_end_applies_only_after_tail_grace() {
        let policy = MusicStageBodyFenceExitPolicy {
            tail_grace_seconds: 2.0,
            transition_min_seconds: 2.8,
            advance_guard_seconds: 0.12,
        };

        let guarded = body_fence_safe_exit_end_seconds(100.0, 150.0, 4.0, 180.0, 128.0, policy)
            .expect("guard");
        assert_eq!(guarded, 128.0);

        assert_eq!(
            body_fence_safe_exit_end_seconds(100.0, 129.0, 4.0, 180.0, 128.0, policy),
            None
        );
    }

    #[test]
    fn energy_tail_safe_exit_end_requires_real_tail_gap() {
        let policy = MusicStageEnergyTailExitPolicy {
            min_tail_seconds: 5.5,
            exit_grace_seconds: 1.2,
            transition_min_seconds: 2.8,
            advance_guard_seconds: 0.12,
        };

        let guarded = energy_tail_safe_exit_end_seconds(80.0, 116.0, 4.0, 120.0, 102.0, policy)
            .expect("guard");
        assert!((guarded - 103.2).abs() < 0.000_001);

        assert_eq!(
            energy_tail_safe_exit_end_seconds(80.0, 119.0, 4.0, 120.0, 116.0, policy),
            None
        );
    }

    #[test]
    fn direct_tail_safe_entry_picks_most_conservative_tail_cue() {
        let policy = MusicStageTailSafeEntryPolicy {
            min_segment_seconds: 8.0,
            advance_guard_seconds: 0.12,
            min_remaining_seconds: 72.0,
            post_promote_min_dwell_seconds: 34.0,
            extra_runway_seconds: 8.0,
            tail_section_backoff_seconds: 20.0,
            trailing_silence_min_seconds: 7.5,
            last_lyric_backoff_seconds: 26.0,
            energy_tail_min_seconds: 5.5,
            energy_tail_entry_backoff_seconds: 24.0,
        };

        let plan = direct_tail_safe_entry_start_seconds(
            150.0,
            4.0,
            Some(180.0),
            Some(120.0),
            Some(118.0),
            Some(160.0),
            Some(110.0),
            policy,
        );

        assert_eq!(plan.start_seconds, 86.0);
        assert_eq!(plan.reason, MusicStageTailSafeEntryReason::EnergyTail);
    }

    #[test]
    fn direct_tail_safe_entry_returns_original_start_without_valid_duration() {
        let policy = MusicStageTailSafeEntryPolicy {
            min_segment_seconds: 8.0,
            advance_guard_seconds: 0.12,
            min_remaining_seconds: 72.0,
            post_promote_min_dwell_seconds: 34.0,
            extra_runway_seconds: 8.0,
            tail_section_backoff_seconds: 20.0,
            trailing_silence_min_seconds: 7.5,
            last_lyric_backoff_seconds: 26.0,
            energy_tail_min_seconds: 5.5,
            energy_tail_entry_backoff_seconds: 24.0,
        };

        let plan = direct_tail_safe_entry_start_seconds(
            32.0,
            4.0,
            None,
            Some(10.0),
            Some(11.0),
            Some(12.0),
            Some(13.0),
            policy,
        );

        assert_eq!(plan.start_seconds, 32.0);
        assert_eq!(plan.reason, MusicStageTailSafeEntryReason::Runway);
    }

    #[test]
    fn vocal_safe_mix_point_prefers_safe_point_inside_window() {
        let risky_near = mix_point(50.1, 0.60, 0.10);
        let safe_far = mix_point(53.0, 0.82, 0.74);
        let points = [risky_near, safe_far];
        let selected = best_vocal_safe_mix_point_near(&points, 50.0, 6.0).expect("point");

        assert_eq!(selected.time_seconds, 53.0);
    }

    #[test]
    fn selected_mix_point_falls_back_to_closest_when_no_safe_point_exists() {
        let mut manifest = manifest_with_candidates(Vec::new());
        manifest.mix_points.mix_in = vec![mix_point(20.0, 0.40, 0.12), mix_point(31.0, 0.42, 0.10)];

        let selected = selected_mix_in_point_for_manifest(&manifest, Some(29.0)).expect("point");

        assert_eq!(selected.time_seconds, 31.0);
    }

    #[test]
    fn segment_bpm_uses_playback_or_highlight_focus_before_global_tempo() {
        let mut manifest = manifest_with_candidates(Vec::new());
        manifest.tempo.bpm = Some(120.0);
        manifest.sections.segment_tempo = vec![
            MusicSegmentTempo {
                start_seconds: 0.0,
                end_seconds: 30.0,
                role: MusicFunctionalRole::Verse,
                bpm: Some(100.0),
                confidence: 0.60,
                stable: false,
            },
            MusicSegmentTempo {
                start_seconds: 30.0,
                end_seconds: 60.0,
                role: MusicFunctionalRole::Chorus,
                bpm: Some(132.0),
                confidence: 0.50,
                stable: true,
            },
        ];

        assert_eq!(
            segment_bpm_from_analysis(&manifest, Some(42.0), None),
            Some(132.0)
        );
        assert_eq!(
            segment_bpm_from_analysis(&manifest, None, Some((10.0, 20.0))),
            Some(100.0)
        );
        assert_eq!(
            segment_bpm_from_analysis(&manifest, None, None),
            Some(120.0)
        );
    }
}
