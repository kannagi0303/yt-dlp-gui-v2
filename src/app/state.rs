use std::collections::{HashMap, VecDeque, hash_map::DefaultHasher};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use egui_commonmark::CommonMarkCache;
use serde_json::Value;

use crate::i18n::{self, Language, LanguageSelection};

pub use crate::app::app_mode::{AppMode, QueueDisplayMode};
use crate::app::batch_add_worker::{
    BatchAddEvent, request_batch_add_stop, run_batch_add_worker, terminate_child_process,
};
use crate::app::component_update_worker::run_component_update_worker;
use crate::app::download_worker::{
    DOWNLOAD_CANCELLED_MESSAGE, DownloadEvent, DownloadProgressDetail, DownloadProgressSlot,
    request_download_stop, run_download_worker,
};
pub use crate::app::format_picker_state::{
    DownloadRangePickerDraft, FormatPickerFilters, FormatPickerKind, FormatPickerState,
    FormatPickerViewMode, SectionPickerTab, SubtitlePickerTab,
};
use crate::app::media_probe::{ffprobe_companion_path_for_ffmpeg, probe_media_with_ffprobe};
pub use crate::app::metadata::sanitize_file_name_for_windows;
use crate::app::metadata::{
    PlaylistEntrySeed, display_file_stem, extract_chapters, extract_formats,
    extract_requested_filename, extract_requested_ids, extract_subtitle_tracks,
    first_audio_format_id, human_size_bytes, infer_title, normalize_duration_badge_text,
    playlist_entry_seed_from_json, requested_or_default_format_id, select_best_thumbnail_url,
    select_largest_thumbnail_url, video_resolution_area,
};
use crate::app::music_stream::{
    self, MusicPlaybackControl, MusicPlaybackEvent, ResolvedMusicStream,
};
use crate::app::post_process_worker::{
    POST_PROCESS_CANCELLED_MESSAGE, PostProcessEvent, request_post_process_stop,
    run_builtin_transcode_worker,
};
pub use crate::app::queue_status::{ItemTitleVisualState, QueueSummary};
use crate::app::queue_status::{
    QueueSummaryBucket, is_pending_download_state, item_can_enter_download_queue,
    item_latest_download_state, item_summary_bucket, selection_matches_completed,
};
use crate::app::thumbnail_worker::{
    ThumbnailFetchEvent, fetch_thumbnail_bytes, run_thumbnail_fetch_worker,
};
use crate::app::transcode_plan::resolve_transcode_plan;
use crate::domain::{
    CompactMusicState, CompletedSelection, CookiePolicy, DownloadContainerPreference,
    DownloadOptions, DownloadRangeSelection, DownloadTimeRange, FormatOption, MediaKind,
    MetadataState, QualityPreset, QueueItem, QueueItemId, QueueItemViewKind, SubtitleOption,
    SubtitleSource, ToolKind, VideoMetadata, WorkflowKind, WorkflowRun, WorkflowRunId,
    WorkflowState, codecs_support_webm_container, format_download_range_timestamp,
};
use crate::infrastructure::cookie_site_index::{
    CookieSiteIndexEntry, read_cookie_site_index_or_default, write_cookie_site_index,
};
use crate::infrastructure::yaml_store::{read_yaml_file, write_yaml_file};
use crate::infrastructure::{
    AnalyzeError, AnalyzeOutput, AppConfig, AppInstanceGuard, CacheLocationMode,
    ComponentUpdateAction, ComponentUpdateEntry, ComponentUpdateEvent, ComponentUpdateSnapshot,
    ComponentUpdateStatus, ConfigFileOption, DependencyTool, DownloadRequest, DownloadTargetKind,
    FINAL_OUTPUT_PATH_PREFIX, FileTimeMode, ManagedComponentId, MediaSessionCommand,
    MediaSessionPlaybackStatus, MediaSessionTimeline, MediaSessionTrack, OutputFileActionMode,
    PostProcessMode, PrepareReport, PrepareRequirement, PrepareStatus, PreparedDownload,
    SerializableCacheLocationMode, ThemeAccentColor, ThemeMode, ToolPaths, WindowPosition,
    WindowSize, YoutubeLoginRescueEvent, YoutubeVideoPlaylistMode, available_yt_dlp_config_files,
    classify_youtube_playlist, cleanup_applied_update, collect_dependency_presence_report,
    collect_prepare_report, component_update_startup_snapshot, configure_background_command,
    dependency_tool_exists, dependency_tool_is_available,
    detect_default_youtube_login_rescue_browser, detect_dependency_tool, display_output_dir,
    launch_pending_app_update, looks_like_playlist_url, normalize_cookie_rescue_target_url,
    normalize_ui_scale_percent, register_app_instance, resolve_output_dir, resolve_tool_path,
    run_tracked_command_output, run_youtube_login_rescue_cookie_export,
    schedule_startup_transient_cleanup, send_download_failed_windows_toast,
    send_download_finished_windows_toast, track_child_process, youtube_url_force_single_video,
    youtube_url_has_video_and_playlist, yt_dlp_configs_dir_display,
};

mod app_state;
mod background_poll_actions;
mod cache_actions;
mod cookie_actions;
mod cookie_rescue;
mod download_container_actions;
mod download_range_actions;
mod item_format_actions;
mod log;
mod log_actions;
mod music_actions;
mod music_analysis_actions;
mod music_batch_actions;
mod music_cache_playlist_actions;
mod music_context_actions;
mod music_download_actions;
mod music_event_actions;
mod music_lyrics_actions;
mod music_metadata_helpers;
mod music_navigation_actions;
mod music_player_actions;
mod music_prefetch_actions;
mod music_runtime;
mod music_types;
mod music_worker_helpers;
mod options_actions;
mod prepare_actions;
mod prepare_visibility_helpers;
mod queue_batch_helpers;
mod queue_input_actions;
mod queue_input_helpers;
mod queue_item_actions;
mod queue_worker_actions;
mod queue_workflow_actions;
mod state_helpers;
#[cfg(test)]
mod tests;
mod ui_types;
mod workflow_download_actions;
mod workflow_runtime;

pub use self::app_state::AppState;
use self::cookie_rescue::CookieRescueState;
pub use self::cookie_rescue::YoutubeLoginRescuePhase;
use self::log::{LogState, aggregate_tool_log_status, current_log_timestamp};
pub use self::log::{ToolLogAction, ToolLogStatus, ToolLogStep};
use self::music_metadata_helpers::*;
pub use self::music_runtime::MusicItemCacheActivity;
use self::music_runtime::{
    AudioCacheManifestSnapshot, AudioPlaylistItemSnapshot, AudioPlaylistSnapshot,
    CacheManagementSummary, CacheRemovalSummary, CachedLrcTrack, CompleteMusicCacheHit, LrcLine,
    MusicAudioExportPlan, MusicAudioQualityIntent, MusicAudioSourceProfile, MusicChorusFadeIn,
    MusicChorusFadeOut, MusicChorusFlowSegment, MusicChorusMixPlan, MusicChorusPendingFadeIn,
    MusicChorusPendingMixTarget, MusicChorusPendingStart, MusicChorusPreparedPreview,
    MusicChorusPreviewJob, MusicDownloadEvent, MusicDownloadJob, MusicLyricsCacheJob,
    MusicPlaybackReadyHandoff, MusicState, MusicStreamResolveEvent, MusicStreamSeed,
};
use self::music_types::music_online_target_format_selector;
pub use self::music_types::{
    MusicDownloadChoice, MusicDownloadFormat, MusicDownloadMode, MusicDownloadSourceKind,
    MusicMixMode, MusicOriginalPreference, MusicPlaybackMode,
};
use self::music_worker_helpers::*;
use self::prepare_visibility_helpers::*;
use self::queue_input_helpers::*;
use self::state_helpers::*;
use self::ui_types::ThumbnailCacheEntry;
pub use self::ui_types::{
    AboutDetailTarget, AdvanceDetailPage, AppTab, CookieFileSourceMode, CookieUsageMode,
    MusicLyricsDisplayLine, MusicPlayerAuraDisplay, MusicPlayerAuraPulse,
    MusicPlayerAuraTrackField, OptionsDetailPage, PrepareDetailPage, SavedCookieFile,
    ThumbnailRenderSource, YoutubePlaylistPrompt, YoutubePlaylistPromptKind,
};
use self::workflow_runtime::{ActiveWorkflow, AnalyzeResult};

const MUSIC_STREAM_CACHE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;
const MUSIC_LYRICS_DISPLAY_LEAD_SECONDS: f64 = 0.2;
const MUSIC_LYRICS_FADE_SECONDS: f64 = 0.14;
const MUSIC_PREFETCH_MIN_PLAY_SECONDS: f64 = 3.0;
const MUSIC_PREFETCH_DEFAULT_LEAD_SECONDS: f64 = 30.0;
const MUSIC_PREFETCH_MIN_LEAD_SECONDS: f64 = 20.0;
const MUSIC_PREFETCH_MAX_LEAD_SECONDS: f64 = 90.0;
const MUSIC_PREFETCH_SPEED_MULTIPLIER: f64 = 2.5;
const MUSIC_PREFETCH_STARTUP_SAFETY_SECONDS: f64 = 5.0;
const MUSIC_PLAYBACK_READY_HANDOFF_FADE_SECONDS: f64 = 0.32;
const MUSIC_PLAY_HISTORY_LIMIT: usize = 128;
