use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Instant, SystemTime};

use serde::{Deserialize, Serialize};

use crate::app::music_mix_timeline::{
    MusicMixFrameCount, MusicMixOutputFrame, MusicMixSourceFrame,
};
use crate::app::music_segment_selector::MusicStageHighlightPick;
use crate::app::music_stream::{MusicMixRenderMode, MusicPlaybackControl, MusicPrefetchControl};
use crate::domain::{QueueItem, QueueItemId, SubtitleOption, WorkflowRunId};
use crate::infrastructure::MediaSession;

use super::{
    MUSIC_PREFETCH_DEFAULT_LEAD_SECONDS, MusicDownloadChoice, MusicDownloadSourceKind,
    MusicPlaybackMode, PlaylistEntrySeed,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicItemCacheActivity {
    Preparing,
    Caching,
}

pub(super) struct MusicState {
    pub(super) music_playback: Option<MusicPlaybackControl>,
    pub(super) music_player_current_item_id: Option<QueueItemId>,
    pub(super) music_playback_session_id: u64,
    pub(super) non_audio_queue_items: Vec<QueueItem>,
    pub(super) audio_queue_items: Vec<QueueItem>,
    pub(super) music_history_back: Vec<QueueItemId>,
    pub(super) music_history_forward: Vec<QueueItemId>,
    pub(super) music_reserved_next_item_id: Option<QueueItemId>,
    pub(super) music_prefetch_active_item_id: Option<QueueItemId>,
    pub(super) music_prefetch_control: Option<MusicPrefetchControl>,
    pub(super) music_prefetch_pending_item_id: Option<QueueItemId>,
    pub(super) music_prefetch_session_id: u64,
    pub(super) music_prefetch_started_at: Option<Instant>,
    pub(super) music_prefetch_lead_seconds: f64,
    pub(super) music_prefetch_for_current_item_id: Option<QueueItemId>,
    pub(super) music_scroll_to_item_id: Option<QueueItemId>,
    pub(super) music_download_prompt_open: bool,
    pub(super) music_download_prompt_choice: MusicDownloadChoice,
    pub(super) active_music_download_choice: Option<MusicDownloadChoice>,
    pub(super) music_player_error: Option<String>,
    pub(super) music_volume: f32,
    pub(super) music_playback_mode: MusicPlaybackMode,
    pub(super) media_session: MediaSession,
    pub(super) cache_management_summary: CacheManagementSummary,
    pub(super) cache_management_summary_refreshed_at: Option<Instant>,
    pub(super) music_seek_drag_ratio: Option<f32>,
    pub(super) music_seek_snap_ratio: Option<f32>,
    pub(super) music_seek_snap_deadline: Option<Instant>,
    pub(super) music_manual_seek_grace_until: Option<Instant>,
    pub(super) music_automix_enabled: bool,
    pub(super) music_mix_render_mode: MusicMixRenderMode,
    pub(super) music_trim_enabled: bool,
    pub(super) music_chorus_flow_enabled: bool,
    pub(super) music_smooth_seek: Option<MusicSmoothSeek>,
    pub(super) music_chorus_flow_segment: Option<MusicChorusFlowSegment>,
    pub(super) music_chorus_mix_plan: Option<MusicChorusMixPlan>,
    pub(super) music_chorus_fade_out: Option<MusicChorusFadeOut>,
    pub(super) music_chorus_fade_in: Option<MusicChorusFadeIn>,
    pub(super) music_chorus_pending_fade_in: Option<MusicChorusPendingFadeIn>,
    pub(super) music_chorus_pending_start: Option<MusicChorusPendingStart>,
    pub(super) music_playback_ready_handoff: Option<MusicPlaybackReadyHandoff>,
    pub(super) music_chorus_handoff_bridge: Option<MusicChorusHandoffBridge>,
    pub(super) music_chorus_pending_mix_target: Option<MusicChorusPendingMixTarget>,
    pub(super) music_chorus_preview_job: Option<MusicChorusPreviewJob>,
    pub(super) music_chorus_ready_preview: Option<MusicChorusPreparedPreview>,
    pub(super) music_stage_pick_selected: HashMap<QueueItemId, MusicStageHighlightPick>,
    pub(super) music_stage_pick_serial: u64,
    pub(super) music_stage_presence_recent_seconds: Option<f64>,
    pub(super) music_stage_presence_last_seconds: Option<f64>,
    pub(super) music_stage_presence_short_run: u8,
    pub(super) music_stage_bpm_display: MusicStageBpmDisplayState,
    pub(super) music_stage_direct_tempo_bridge_strength: f32,
    pub(super) music_stage_direct_mix_length: f32,
    pub(super) music_stage_direct_mix_curve: f32,
    pub(super) music_stage_direct_mix_assist: f32,
    pub(super) music_stage_cue_memory: MusicStageCueMemoryStore,
    pub(super) music_stage_cue_memory_loaded: bool,
    pub(super) music_lyrics_cache: HashMap<String, CachedLrcTrack>,
    pub(super) music_lyrics_display_line: Option<String>,
    pub(super) music_lyrics_previous_line: Option<String>,
    pub(super) music_lyrics_transition_started_at: Option<Instant>,
}

impl MusicState {
    pub(super) fn new(music_volume: f32, music_playback_mode: MusicPlaybackMode) -> Self {
        Self {
            music_playback: None,
            music_player_current_item_id: None,
            music_playback_session_id: 0,
            non_audio_queue_items: Vec::new(),
            audio_queue_items: Vec::new(),
            music_history_back: Vec::new(),
            music_history_forward: Vec::new(),
            music_reserved_next_item_id: None,
            music_prefetch_active_item_id: None,
            music_prefetch_control: None,
            music_prefetch_pending_item_id: None,
            music_prefetch_session_id: 0,
            music_prefetch_started_at: None,
            music_prefetch_lead_seconds: MUSIC_PREFETCH_DEFAULT_LEAD_SECONDS,
            music_prefetch_for_current_item_id: None,
            music_scroll_to_item_id: None,
            music_download_prompt_open: false,
            music_download_prompt_choice: MusicDownloadChoice::default(),
            active_music_download_choice: None,
            music_player_error: None,
            music_volume,
            music_playback_mode,
            media_session: MediaSession::new(),
            cache_management_summary: CacheManagementSummary::default(),
            cache_management_summary_refreshed_at: None,
            music_seek_drag_ratio: None,
            music_seek_snap_ratio: None,
            music_seek_snap_deadline: None,
            music_manual_seek_grace_until: None,
            music_automix_enabled: false,
            music_mix_render_mode: MusicMixRenderMode::Streaming,
            music_trim_enabled: false,
            music_chorus_flow_enabled: false,
            music_smooth_seek: None,
            music_chorus_flow_segment: None,
            music_chorus_mix_plan: None,
            music_chorus_fade_out: None,
            music_chorus_fade_in: None,
            music_chorus_pending_fade_in: None,
            music_chorus_pending_start: None,
            music_playback_ready_handoff: None,
            music_chorus_handoff_bridge: None,
            music_chorus_pending_mix_target: None,
            music_chorus_preview_job: None,
            music_chorus_ready_preview: None,
            music_stage_pick_selected: HashMap::new(),
            music_stage_pick_serial: 0,
            music_stage_presence_recent_seconds: None,
            music_stage_presence_last_seconds: None,
            music_stage_presence_short_run: 0,
            music_stage_bpm_display: MusicStageBpmDisplayState::default(),
            music_stage_direct_tempo_bridge_strength: 0.50,
            music_stage_direct_mix_length: 0.50,
            music_stage_direct_mix_curve: 0.85,
            music_stage_direct_mix_assist: 1.00,
            music_stage_cue_memory: MusicStageCueMemoryStore::default(),
            music_stage_cue_memory_loaded: false,
            music_lyrics_cache: HashMap::new(),
            music_lyrics_display_line: None,
            music_lyrics_previous_line: None,
            music_lyrics_transition_started_at: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct MusicStageBpmDisplayState {
    pub(super) stable_bpm: Option<f32>,
    pub(super) display_bpm: Option<f32>,
    pub(super) animation_from_bpm: Option<f32>,
    pub(super) animation_to_bpm: Option<f32>,
    pub(super) animation_started_at: Option<Instant>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MusicSmoothSeekPhase {
    FadeOut,
    FadeIn,
}

#[derive(Clone, Debug)]
pub(super) struct MusicSmoothSeek {
    pub(super) item_id: QueueItemId,
    pub(super) session_id: u64,
    pub(super) target_seconds: f64,
    pub(super) duration_seconds: f64,
    pub(super) phase: MusicSmoothSeekPhase,
    pub(super) started_at: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MusicChorusFallbackStage {
    Normal,
    StreamFallback,
    PlainCrossfade,
}

impl MusicChorusFallbackStage {
    pub(super) fn is_stream_fallback(self) -> bool {
        matches!(self, Self::StreamFallback | Self::PlainCrossfade)
    }

    pub(super) fn is_plain_crossfade(self) -> bool {
        matches!(self, Self::PlainCrossfade)
    }
}

#[derive(Clone, Debug)]
pub(super) struct MusicChorusFlowSegment {
    pub(super) item_id: QueueItemId,
    pub(super) session_id: u64,
    pub(super) start_seconds: f64,
    pub(super) end_seconds: f64,
    pub(super) transition_seconds: f64,
    pub(super) hold_end_seconds: Option<f64>,
    pub(super) fallback_stage: MusicChorusFallbackStage,
}

#[derive(Clone, Debug)]
pub(super) struct MusicChorusMixPlan {
    pub(super) transition_seconds: f64,
    pub(super) confidence: f32,
    pub(super) reason: String,
}

impl MusicChorusMixPlan {
    pub(super) fn is_provisional_analysis_pending(&self) -> bool {
        self.reason.contains("provisional") && self.reason.contains("analysis pending")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MusicStageMixExecutionRoute {
    PreparedSegment,
    RealtimePreview,
    PlainInline,
    DirectFallback,
}

impl MusicStageMixExecutionRoute {
    pub(super) fn from_handoff_state(
        crossfade_preview_started: bool,
        prepared_mix_started: bool,
        plain_crossfade_fallback: bool,
    ) -> Self {
        if crossfade_preview_started && prepared_mix_started {
            Self::PreparedSegment
        } else if crossfade_preview_started && plain_crossfade_fallback {
            Self::PlainInline
        } else if crossfade_preview_started {
            Self::RealtimePreview
        } else {
            Self::DirectFallback
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::PreparedSegment => "prepared-segment",
            Self::RealtimePreview => "realtime-preview",
            Self::PlainInline => "plain-inline",
            Self::DirectFallback => "direct-fallback",
        }
    }

    pub(super) fn render_key(self) -> &'static str {
        match self {
            Self::PreparedSegment => "prepared",
            Self::RealtimePreview => "guarded",
            Self::PlainInline => "fallback",
            Self::DirectFallback => "direct",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct MusicChorusFadeOut {
    pub(super) item_id: QueueItemId,
    pub(super) session_id: u64,
    pub(super) execution_route: MusicStageMixExecutionRoute,
    pub(super) started_output_frame: MusicMixOutputFrame,
    pub(super) duration_output_frames: MusicMixFrameCount,
    // Seconds remain only for UI/debug text and next-track planning. Playback
    // completion must use the frame fields above.
    pub(super) duration_seconds: f64,
    pub(super) planned_transition_seconds: f64,
    pub(super) executed_transition_seconds: f64,
    pub(super) target_volume: f32,
    pub(super) next_item_id: Option<QueueItemId>,
    pub(super) next_start_seconds: Option<f64>,
    pub(super) crossfade_preview_started: bool,
    pub(super) prepared_mix_started: bool,
    pub(super) plain_crossfade_fallback: bool,
    pub(super) start_playback_seconds: f64,
    // UI-only plan snapshot. The visible MIX lane must stay anchored when the
    // playback cursor reaches it; execution timing is owned by the frame fields.
    pub(super) mix_window_start_seconds: f64,
    pub(super) mix_window_end_seconds: f64,
}

#[derive(Clone, Debug)]
pub(super) struct MusicChorusFadeIn {
    pub(super) item_id: QueueItemId,
    pub(super) session_id: u64,
    pub(super) started_output_frame: MusicMixOutputFrame,
    pub(super) duration_output_frames: MusicMixFrameCount,
    // UI/debug mirror; never use this field as the execution clock.
    pub(super) duration_seconds: f64,
    pub(super) target_volume: f32,
}

#[derive(Clone, Debug)]
pub(super) struct MusicChorusPendingFadeIn {
    pub(super) item_id: QueueItemId,
    pub(super) duration_seconds: f64,
    pub(super) target_volume: f32,
    // Direct Tempo Bridge: when Track(B) becomes the real stream, it may carry
    // a very short incoming tempo feather during the fade-in.  This stays on
    // the new playback control and returns to normal speed automatically.
    pub(super) incoming_tempo_rate: f64,
}

#[derive(Clone, Debug)]
pub(super) struct MusicChorusPendingStart {
    pub(super) item_id: QueueItemId,
    pub(super) start_seconds: f64,
}

#[derive(Clone)]
pub(super) struct MusicPlaybackReadyHandoff {
    pub(super) control: MusicPlaybackControl,
    pub(super) target_item_id: QueueItemId,
    pub(super) stop_output_frame: Option<MusicMixOutputFrame>,
}

#[derive(Clone)]
pub(super) struct MusicChorusHandoffBridge {
    pub(super) control: MusicPlaybackControl,
    pub(super) target_item_id: QueueItemId,
    pub(super) stop_output_frame: Option<MusicMixOutputFrame>,
    // Keep Aura on the same output-frame clock as Direct Stream Handoff.
    pub(super) visual_started_output_frame: MusicMixOutputFrame,
    pub(super) visual_duration_output_frames: MusicMixFrameCount,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct MusicStageCueMemoryStore {
    #[serde(default = "default_music_stage_cue_memory_version")]
    pub(super) version: u32,
    #[serde(default)]
    pub(super) entries: HashMap<String, MusicStageCueMemoryEntry>,
}

impl Default for MusicStageCueMemoryStore {
    fn default() -> Self {
        Self {
            version: default_music_stage_cue_memory_version(),
            entries: HashMap::new(),
        }
    }
}

fn default_music_stage_cue_memory_version() -> u32 {
    1
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(super) struct MusicStageCueMemoryEntry {
    #[serde(default)]
    pub(super) start_offset_seconds: f64,
    #[serde(default)]
    pub(super) end_offset_seconds: f64,
    #[serde(default)]
    pub(super) effective_presence_seconds: f64,
    #[serde(default)]
    pub(super) confidence: f32,
    #[serde(default)]
    pub(super) updates: u32,
    #[serde(default)]
    pub(super) updated_unix_seconds: u64,
}

#[derive(Clone, Debug)]
pub(super) struct MusicChorusPendingMixTarget {
    pub(super) current_item_id: QueueItemId,
    pub(super) session_id: u64,
    pub(super) target_item_id: QueueItemId,
    pub(super) target_start_seconds: Option<f64>,
    pub(super) transition_seconds: f64,
    pub(super) confidence: f32,
    pub(super) reason: String,
    pub(super) requested_at: Instant,
    pub(super) cue_armed: bool,
    pub(super) hold_end_seconds: Option<f64>,
}

#[derive(Clone, Debug)]
pub(super) struct MusicChorusPreparedPreview {
    pub(super) current_item_id: QueueItemId,
    pub(super) session_id: u64,
    pub(super) next_item_id: QueueItemId,
    pub(super) next_duration_seconds: Option<f64>,
    pub(super) entry_start_seconds: f64,
    pub(super) transition_seconds: f64,
    // The worker crosses into playback ownership here. Preserve these explicit
    // frame units; do not reconstruct them from transition_seconds/preview_rate.
    pub(super) transition_output_frames: MusicMixFrameCount,
    pub(super) transition_source_frames: MusicMixFrameCount,
    pub(super) source_start_frame: MusicMixSourceFrame,
    pub(super) prepared_mix: bool,
    pub(super) prepared_mix_source_frames: MusicMixFrameCount,
    pub(super) prepared_mix_resume_source_frame: Option<MusicMixSourceFrame>,
    pub(super) prepared_mix_start_seconds: Option<f64>,
    pub(super) prepared_mix_b_samples: Option<Vec<f32>>,
    pub(super) source_sample_rate: u32,
    pub(super) plan_confidence: f32,
    pub(super) preview_rate: f64,
    pub(super) outgoing_rate: f64,
    pub(super) preserve_pitch: bool,
    pub(super) stretch_detail: Option<String>,
    pub(super) outgoing_highlight_end_phase: Option<f32>,
    pub(super) samples: Vec<f32>,
}

pub(super) struct MusicChorusPreviewJob {
    pub(super) current_item_id: QueueItemId,
    pub(super) session_id: u64,
    pub(super) next_item_id: QueueItemId,
    pub(super) transition_seconds: f64,
    pub(super) started_at: Instant,
    pub(super) receiver: Receiver<Result<MusicChorusPreparedPreview, String>>,
}

pub(super) enum MusicStreamResolveEvent {
    ToolCommandFinished {
        action_id: u64,
        tool: String,
        action: String,
        command_line: String,
        success: bool,
    },
    FlatImport {
        source: String,
        result: Result<Vec<PlaylistEntrySeed>, String>,
    },
    FlatUpdate {
        item_id: QueueItemId,
        source: String,
        result: Result<PlaylistEntrySeed, String>,
    },
    Resolve {
        item_id: QueueItemId,
        session_id: u64,
        source: String,
        play_after_resolve: bool,
        result: Result<MusicStreamSeed, String>,
    },
}

pub(super) enum MusicDownloadEvent {
    Progress {
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        percent: f32,
    },
    ToolCommandFinished {
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        source_kind: MusicDownloadSourceKind,
        tool: String,
        action: String,
        command_line: String,
        success: bool,
    },
    Finished {
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        source_kind: MusicDownloadSourceKind,
        result: Result<String, String>,
    },
}

pub(super) struct MusicStreamSeed {
    pub(super) source_url: String,
    pub(super) title: String,
    pub(super) album_title: String,
    pub(super) thumbnail_url: String,
    pub(super) thumbnail_hint: String,
    pub(super) duration_text: String,
    pub(super) duration_seconds: Option<f64>,
    pub(super) direct_url: String,
    pub(super) headers: Vec<(String, String)>,
    pub(super) ext: String,
    pub(super) format_id: String,
    pub(super) acodec: String,
    pub(super) expected_bytes: Option<u64>,
    pub(super) cache_key: String,
    pub(super) lyrics_track: Option<SubtitleOption>,
}

pub(super) struct CompleteMusicCacheHit {
    pub(super) cache_key: String,
    pub(super) source_url: String,
    pub(super) title: String,
    pub(super) album_title: String,
    pub(super) thumbnail_url: String,
    pub(super) duration_seconds: Option<f64>,
    pub(super) ext: String,
    pub(super) format_id: String,
    pub(super) acodec: String,
    pub(super) expected_bytes: Option<u64>,
}

#[derive(Clone, Debug)]
pub(super) struct CachedLrcTrack {
    pub(super) path: PathBuf,
    pub(super) modified: Option<SystemTime>,
    pub(super) lines: Vec<LrcLine>,
    pub(super) missing_checked_at: Option<Instant>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct AudioPlaylistSnapshot {
    pub(super) version: u32,
    pub(super) items: Vec<AudioPlaylistItemSnapshot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct AudioPlaylistItemSnapshot {
    pub(super) source_url: String,
    pub(super) title: String,
    #[serde(default)]
    pub(super) album_title: String,
    #[serde(default)]
    pub(super) thumbnail_hint: String,
    #[serde(default)]
    pub(super) thumbnail_url: String,
    #[serde(default)]
    pub(super) duration_text: String,
    pub(super) duration_seconds: Option<f64>,
    #[serde(default)]
    pub(super) stream_ext: String,
    #[serde(default)]
    pub(super) stream_format_id: String,
    #[serde(default)]
    pub(super) stream_acodec: String,
    pub(super) expected_bytes: Option<u64>,
    #[serde(default)]
    pub(super) cache_key: String,
    #[serde(default)]
    pub(super) use_cookies: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub(super) struct AudioCacheManifestSnapshot {
    pub(super) source_url: String,
    pub(super) title: String,
    pub(super) album_title: String,
    pub(super) duration_seconds: Option<f64>,
    pub(super) ext: String,
    pub(super) format_id: String,
    pub(super) acodec: String,
    pub(super) thumbnail_url: String,
    pub(super) expected_bytes: Option<u64>,
    pub(super) downloaded_bytes: Option<u64>,
    pub(super) ranges: Vec<AudioCacheRangeSnapshot>,
    pub(super) complete: bool,
    pub(super) updated_unix_seconds: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct AudioCacheRangeSnapshot {
    pub(super) start: u64,
    pub(super) end: u64,
}

#[derive(Clone, Debug)]
pub(super) struct LrcLine {
    pub(super) seconds: f64,
    pub(super) text: String,
}

#[derive(Clone, Debug)]
pub(super) struct MusicLyricsCacheJob {
    pub(super) source_url: String,
    pub(super) cache_key: String,
    pub(super) language_code: String,
    pub(super) use_cookies: bool,
}

#[derive(Clone, Debug, Default)]
pub(super) struct CacheManagementSummary {
    pub(super) total_bytes: u64,
    pub(super) music_bytes: u64,
    pub(super) expired_music_bytes: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CacheRemovalSummary {
    pub(super) bytes: u64,
    pub(super) entries: u64,
}

pub(super) struct MusicDownloadJob {
    pub(super) item_id: QueueItemId,
    pub(super) workflow_id: WorkflowRunId,
    pub(super) source_url: String,
    pub(super) title: String,
    pub(super) album_title: String,
    pub(super) output_dir: PathBuf,
    pub(super) choice: MusicDownloadChoice,
    pub(super) source_acodec: String,
    pub(super) cache_media_path: Option<PathBuf>,
    pub(super) cover_path: Option<PathBuf>,
    pub(super) cover_cache_dir: Option<PathBuf>,
    pub(super) thumbnail_url: String,
    pub(super) use_cookies: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MusicAudioQualityIntent {
    PreservePerceivedQuality,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct MusicAudioSourceProfile {
    pub(super) acodec: String,
    pub(super) bitrate_kbps: Option<u32>,
    pub(super) sample_rate: Option<u32>,
    pub(super) channels: Option<u32>,
}

impl MusicAudioSourceProfile {
    pub(super) fn from_codec(source_acodec: &str) -> Self {
        Self {
            acodec: source_acodec.to_owned(),
            bitrate_kbps: None,
            sample_rate: None,
            channels: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MusicAudioExportPlan {
    pub(super) ffmpeg_args: Vec<String>,
}
