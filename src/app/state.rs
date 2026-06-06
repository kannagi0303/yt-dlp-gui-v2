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
use serde::{Deserialize, Serialize};
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
    FormatPickerFilters, FormatPickerKind, FormatPickerState, FormatPickerViewMode,
    SubtitlePickerTab,
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
    self, MusicPlaybackControl, MusicPlaybackEvent, MusicPrefetchControl, ResolvedMusicStream,
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
    CompactMusicState, CompletedSelection, CookiePolicy, DownloadOptions, FormatOption, MediaKind,
    MetadataState, QualityPreset, QueueItem, QueueItemId, QueueItemViewKind, SubtitleOption,
    SubtitleSource, ToolKind, VideoMetadata, WorkflowKind, WorkflowRun, WorkflowRunId,
    WorkflowState,
};
use crate::infrastructure::cookie_site_index::{
    CookieSiteIndexEntry, read_cookie_site_index_or_default, write_cookie_site_index,
};
use crate::infrastructure::yaml_store::{read_yaml_file, write_yaml_file};
use crate::infrastructure::{
    AnalyzeError, AnalyzeOutput, AppConfig, AppInstanceGuard, CacheLocationMode,
    ComponentUpdateAction, ComponentUpdateEntry, ComponentUpdateEvent, ComponentUpdateSnapshot,
    ComponentUpdateStatus, ConfigFileOption, DependencyTool, DownloadRequest, DownloadTargetKind,
    FINAL_OUTPUT_PATH_PREFIX, FileTimeMode, ManagedComponentId, MediaSession, MediaSessionCommand,
    MediaSessionPlaybackStatus, MediaSessionTimeline, MediaSessionTrack, OutputFileActionMode,
    PostProcessMode, PrepareReport, PrepareRequirement, PrepareStatus, PreparedDownload,
    SerializableCacheLocationMode, ThemeAccentColor, ThemeMode, ToolPaths, WindowPosition,
    WindowSize, YoutubeLoginRescueBrowserInfo, YoutubeLoginRescueEvent, YoutubePlaylistRisk,
    YoutubeVideoPlaylistMode, available_yt_dlp_config_files, classify_youtube_playlist,
    cleanup_applied_update, collect_dependency_presence_report, collect_prepare_report,
    component_update_startup_snapshot, configure_background_command, dependency_tool_exists,
    dependency_tool_is_available, detect_default_youtube_login_rescue_browser,
    detect_dependency_tool_in_system_path, display_output_dir, launch_pending_app_update,
    looks_like_playlist_url, normalize_cookie_rescue_target_url, normalize_ui_scale_percent,
    register_app_instance, resolve_output_dir, resolve_tool_path,
    run_youtube_login_rescue_cookie_export, schedule_startup_transient_cleanup,
    send_download_failed_windows_toast, send_download_finished_windows_toast,
    youtube_url_force_single_video, youtube_url_has_video_and_playlist, yt_dlp_configs_dir_display,
};

const MUSIC_STREAM_CACHE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;
const MUSIC_LYRICS_DISPLAY_LEAD_SECONDS: f64 = 0.2;
const MUSIC_LYRICS_FADE_SECONDS: f64 = 0.14;
const MUSIC_PREFETCH_MIN_PLAY_SECONDS: f64 = 10.0;
const MUSIC_PREFETCH_DEFAULT_LEAD_SECONDS: f64 = 10.0;
const MUSIC_PREFETCH_MIN_LEAD_SECONDS: f64 = 10.0;
const MUSIC_PREFETCH_MAX_LEAD_SECONDS: f64 = 45.0;
const MUSIC_PREFETCH_SPEED_MULTIPLIER: f64 = 2.0;
const MUSIC_PLAY_HISTORY_LIMIT: usize = 128;
const MUSIC_ORIGINAL_AUTO_SELECTOR: &str = "bestaudio/best[acodec!=none]";
const MUSIC_ORIGINAL_MP3_SELECTOR: &str =
    "bestaudio[ext=mp3]/bestaudio[acodec^=mp3]/bestaudio/best[acodec!=none]";
const MUSIC_ORIGINAL_AAC_SELECTOR: &str = "bestaudio[ext=m4a]/bestaudio[acodec^=mp4a]/bestaudio[acodec^=aac]/bestaudio/best[acodec!=none]";
const MUSIC_ORIGINAL_OPUS_SELECTOR: &str = "bestaudio[acodec^=opus]/bestaudio[ext=opus]/bestaudio[ext=webm][acodec^=opus]/bestaudio/best[acodec!=none]";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTab {
    Prepare,
    Main,
    Advance,
    Options,
    About,
    Log,
}

#[derive(Clone, Debug)]
pub struct MusicLyricsDisplayLine {
    pub current: String,
    pub previous: Option<String>,
    pub fade: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionsDetailPage {
    Language,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrepareDetailPage {
    Language,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdvanceDetailPage {
    Transcode,
    CookieManager,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CookieUsageMode {
    Off,
    Browser,
    File,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CookieFileSourceMode {
    Custom,
    AutoSelect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedCookieFile {
    pub id: String,
    pub display_name: String,
    pub login_url: String,
    pub updated_unix: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AboutDetailTarget {
    App,
    Tool(ManagedComponentId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YoutubeLoginRescuePhase {
    Idle,
    Confirm,
    NoSupportedBrowser,
    Starting,
    WaitingForCdp,
    WaitingForCookie,
    CookieExported,
    Failed,
    Closed,
}

impl YoutubeLoginRescuePhase {
    pub fn is_blocking_prompt(self) -> bool {
        !matches!(self, Self::Idle | Self::Closed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicPlaybackMode {
    Sequential,
    RepeatAll,
    Shuffle,
    RepeatOne,
}

impl MusicPlaybackMode {
    fn next(self) -> Self {
        match self {
            Self::Sequential => Self::RepeatAll,
            Self::RepeatAll => Self::Shuffle,
            Self::Shuffle => Self::RepeatOne,
            Self::RepeatOne => Self::Sequential,
        }
    }

    fn label_key(self) -> &'static str {
        match self {
            Self::Sequential => "Sequence",
            Self::RepeatAll => "Repeat",
            Self::Shuffle => "Shuffle",
            Self::RepeatOne => "Repeat one",
        }
    }

    fn config_value(self) -> &'static str {
        match self {
            Self::Sequential => "sequential",
            Self::RepeatAll => "repeat_all",
            Self::Shuffle => "shuffle",
            Self::RepeatOne => "repeat_one",
        }
    }

    fn from_config_value(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "repeat_all" | "repeat" | "loop" => Self::RepeatAll,
            "shuffle" | "random" => Self::Shuffle,
            "repeat_one" | "single" | "one" => Self::RepeatOne,
            _ => Self::Sequential,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicDownloadFormat {
    Mp3,
    M4aAac,
    Opus,
    Flac,
    Wav,
}

impl MusicDownloadFormat {
    pub const SIMPLE_OUTPUTS: [Self; 3] = [Self::Mp3, Self::M4aAac, Self::Opus];

    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::M4aAac => "m4a",
            Self::Opus => "opus",
            Self::Flac => "flac",
            Self::Wav => "wav",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Mp3 => "MP3",
            Self::M4aAac => "AAC",
            Self::Opus => "Opus",
            Self::Flac => "FLAC",
            Self::Wav => "WAV",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicDownloadMode {
    Original,
    Unified,
}

impl MusicDownloadMode {
    pub const ALL: [Self; 2] = [Self::Original, Self::Unified];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Original => "Original file",
            Self::Unified => "Unified format",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicOriginalPreference {
    Auto,
    PreferMp3,
    PreferAac,
    PreferOpus,
}

impl MusicOriginalPreference {
    pub const ALL: [Self; 4] = [
        Self::Auto,
        Self::PreferOpus,
        Self::PreferAac,
        Self::PreferMp3,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Auto => "Best",
            Self::PreferMp3 => "MP3",
            Self::PreferAac => "AAC",
            Self::PreferOpus => "Opus",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MusicDownloadChoice {
    pub mode: MusicDownloadMode,
    pub original_preference: MusicOriginalPreference,
    pub unified_format: MusicDownloadFormat,
    pub embed_cover: bool,
    pub write_tags: bool,
}

impl Default for MusicDownloadChoice {
    fn default() -> Self {
        Self {
            mode: MusicDownloadMode::Original,
            original_preference: MusicOriginalPreference::Auto,
            unified_format: MusicDownloadFormat::M4aAac,
            embed_cover: true,
            write_tags: true,
        }
    }
}

impl MusicDownloadChoice {
    fn target_format(self) -> Option<MusicDownloadFormat> {
        match self.mode {
            MusicDownloadMode::Original => None,
            MusicDownloadMode::Unified => Some(self.unified_format),
        }
    }

    fn format_selector(self) -> &'static str {
        match self.mode {
            MusicDownloadMode::Original => match self.original_preference {
                MusicOriginalPreference::Auto => MUSIC_ORIGINAL_AUTO_SELECTOR,
                MusicOriginalPreference::PreferMp3 => MUSIC_ORIGINAL_MP3_SELECTOR,
                MusicOriginalPreference::PreferAac => MUSIC_ORIGINAL_AAC_SELECTOR,
                MusicOriginalPreference::PreferOpus => MUSIC_ORIGINAL_OPUS_SELECTOR,
            },
            MusicDownloadMode::Unified => music_online_target_format_selector(self.unified_format),
        }
    }

    fn selection_token(self) -> &'static str {
        match self.mode {
            MusicDownloadMode::Original => match self.original_preference {
                MusicOriginalPreference::Auto => "original:auto",
                MusicOriginalPreference::PreferMp3 => "original:mp3",
                MusicOriginalPreference::PreferAac => "original:aac",
                MusicOriginalPreference::PreferOpus => "original:opus",
            },
            MusicDownloadMode::Unified => self.unified_format.extension(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicDownloadSourceKind {
    CacheCopy,
    CacheConvert,
    /// yt-dlp selected an online source that already matches the requested audio codec.
    YtDlpOnlineTarget,
    YtDlpDownload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolLogStatus {
    Running,
    Success,
    Recovered,
    Failed,
    Skipped,
}

#[derive(Clone, Debug)]
pub struct ToolLogStep {
    pub id: u64,
    pub status: ToolLogStatus,
    pub tool: String,
    pub action: String,
    pub command: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ToolLogAction {
    pub id: u64,
    pub timestamp: String,
    pub status: ToolLogStatus,
    pub mode: String,
    pub action: String,
    pub steps: Vec<ToolLogStep>,
}

pub enum ThumbnailRenderSource {
    None,
    DirectUrl,
    Loading,
    Texture(eframe::egui::TextureHandle),
    Failed(String),
}

enum ThumbnailCacheEntry {
    Loading,
    Ready(eframe::egui::TextureHandle),
    Failed(String),
}

pub struct AppState {
    pub active_tab: AppTab,
    pub url_input: String,
    pub batch_input: String,
    pub batch_enabled: bool,
    pub monitor_clipboard: bool,
    last_clipboard_text: String,
    last_clipboard_check: Option<Instant>,
    clipboard_monitor_baseline_ready: bool,
    pub empty_item_preview: VideoMetadata,
    pub queue_items: Vec<QueueItem>,
    queue_display_mode: QueueDisplayMode,
    app_mode: AppMode,
    pub item_defaults: DownloadOptions,
    pub config: AppConfig,
    pending_ui_scale_percent: u16,
    pub options_detail_page: Option<OptionsDetailPage>,
    pub prepare_detail_page: Option<PrepareDetailPage>,
    pub advance_detail_page: Option<AdvanceDetailPage>,
    pub about_detail_target: AboutDetailTarget,
    pub tool_paths: ToolPaths,
    pub prepare_report: PrepareReport,
    prepare_tab_snoozed: bool,
    pub last_action: String,
    pub runtime_log: VecDeque<String>,
    pub tool_logs: VecDeque<ToolLogAction>,
    pub log_viewer_selected_step: Option<u64>,
    pub log_viewer_expanded_action: Option<u64>,
    tool_log_action_by_workflow: HashMap<WorkflowRunId, u64>,
    next_tool_log_action_id: u64,
    next_tool_log_step_id: u64,
    pub format_picker: FormatPickerState,
    pub is_adding_batch: bool,
    pub is_cancelling_batch_add: bool,
    pub youtube_playlist_prompt: Option<YoutubePlaylistPrompt>,
    pub youtube_login_rescue_phase: YoutubeLoginRescuePhase,
    pub youtube_login_rescue_browser: Option<YoutubeLoginRescueBrowserInfo>,
    pub youtube_login_rescue_site_name: Option<String>,
    pub youtube_login_rescue_target_url: String,
    pub youtube_login_rescue_target_error: Option<String>,
    pub youtube_login_rescue_clipboard_prefilled: bool,
    pub youtube_login_rescue_error: Option<String>,
    youtube_login_rescue_rx: Option<Receiver<YoutubeLoginRescueEvent>>,
    analyze_result_rx: Receiver<AnalyzeResult>,
    analyze_result_tx: Sender<AnalyzeResult>,
    batch_add_result_rx: Option<Receiver<BatchAddEvent>>,
    batch_add_child: Option<Arc<Mutex<Option<Child>>>>,
    batch_add_cancel_requested: Option<Arc<AtomicBool>>,
    batch_add_music_compact: bool,
    download_result_rx: Receiver<DownloadEvent>,
    download_result_tx: Sender<DownloadEvent>,
    post_process_result_rx: Receiver<PostProcessEvent>,
    post_process_result_tx: Sender<PostProcessEvent>,
    component_update_result_rx: Receiver<ComponentUpdateEvent>,
    component_update_result_tx: Sender<ComponentUpdateEvent>,
    thumbnail_result_rx: Receiver<ThumbnailFetchEvent>,
    thumbnail_result_tx: Sender<ThumbnailFetchEvent>,
    music_stream_result_rx: Receiver<MusicStreamResolveEvent>,
    music_stream_result_tx: Sender<MusicStreamResolveEvent>,
    music_playback_event_rx: Receiver<MusicPlaybackEvent>,
    music_playback_event_tx: Sender<MusicPlaybackEvent>,
    music_download_event_rx: Receiver<MusicDownloadEvent>,
    music_download_event_tx: Sender<MusicDownloadEvent>,
    music_playback: Option<MusicPlaybackControl>,
    music_player_current_item_id: Option<QueueItemId>,
    music_playback_session_id: u64,
    non_audio_queue_items: Vec<QueueItem>,
    audio_queue_items: Vec<QueueItem>,
    music_history_back: Vec<QueueItemId>,
    music_history_forward: Vec<QueueItemId>,
    music_reserved_next_item_id: Option<QueueItemId>,
    music_prefetch_active_item_id: Option<QueueItemId>,
    music_prefetch_control: Option<MusicPrefetchControl>,
    music_prefetch_pending_item_id: Option<QueueItemId>,
    music_prefetch_session_id: u64,
    music_prefetch_started_at: Option<Instant>,
    music_prefetch_lead_seconds: f64,
    music_prefetch_for_current_item_id: Option<QueueItemId>,
    music_scroll_to_item_id: Option<QueueItemId>,
    music_download_prompt_open: bool,
    music_download_prompt_choice: MusicDownloadChoice,
    active_music_download_choice: Option<MusicDownloadChoice>,
    music_player_error: Option<String>,
    music_volume: f32,
    music_playback_mode: MusicPlaybackMode,
    media_session: MediaSession,
    cache_management_summary: CacheManagementSummary,
    cache_management_summary_refreshed_at: Option<Instant>,
    music_seek_drag_ratio: Option<f32>,
    music_seek_snap_ratio: Option<f32>,
    music_seek_snap_deadline: Option<Instant>,
    music_lyrics_cache: HashMap<String, CachedLrcTrack>,
    music_lyrics_display_line: Option<String>,
    music_lyrics_previous_line: Option<String>,
    music_lyrics_transition_started_at: Option<Instant>,
    thumbnail_cache: HashMap<String, ThumbnailCacheEntry>,
    pub component_update_snapshot: ComponentUpdateSnapshot,
    pub about_markdown_cache: CommonMarkCache,
    app_instance_guard: Option<AppInstanceGuard>,
    font_content_revision: u64,
    active_workflows: HashMap<WorkflowRunId, ActiveWorkflow>,
    next_queue_item_id: QueueItemId,
    next_workflow_run_id: WorkflowRunId,
}

struct AnalyzeResult {
    source: String,
    target_item_id: Option<QueueItemId>,
    workflow_id: Option<WorkflowRunId>,
    used_cookies: bool,
    tool_log_action_id: Option<u64>,
    command_line: Option<String>,
    result: Result<Value, String>,
}

fn analyze_output_parts(
    result: Result<AnalyzeOutput, AnalyzeError>,
) -> (Result<Value, String>, Option<String>) {
    match result {
        Ok(output) => (Ok(output.json), Some(output.command_line)),
        Err(error) => (Err(error.message), error.command_line),
    }
}

fn download_target_log_action(target_kind: DownloadTargetKind) -> &'static str {
    match target_kind {
        DownloadTargetKind::Normal => "download",
        DownloadTargetKind::Video => "download video",
        DownloadTargetKind::Audio => "download audio",
        DownloadTargetKind::Subtitle => "download subtitle",
    }
}

fn music_source_kind_log_action(source_kind: MusicDownloadSourceKind) -> &'static str {
    match source_kind {
        MusicDownloadSourceKind::CacheCopy => "music cache copy",
        MusicDownloadSourceKind::CacheConvert => "music cache convert",
        MusicDownloadSourceKind::YtDlpOnlineTarget => "music download",
        MusicDownloadSourceKind::YtDlpDownload => "music download",
    }
}

fn aggregate_tool_log_status(steps: &[ToolLogStep]) -> ToolLogStatus {
    if steps.is_empty()
        || steps
            .iter()
            .any(|step| step.status == ToolLogStatus::Running)
    {
        ToolLogStatus::Running
    } else if steps
        .iter()
        .any(|step| step.status == ToolLogStatus::Failed)
    {
        ToolLogStatus::Failed
    } else if steps
        .iter()
        .any(|step| step.status == ToolLogStatus::Success)
    {
        ToolLogStatus::Success
    } else if steps
        .iter()
        .any(|step| step.status == ToolLogStatus::Recovered)
    {
        ToolLogStatus::Recovered
    } else {
        ToolLogStatus::Skipped
    }
}

fn current_log_timestamp() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let seconds_in_day = elapsed % 86_400;
    let hour = seconds_in_day / 3_600;
    let minute = (seconds_in_day % 3_600) / 60;
    let second = seconds_in_day % 60;
    format!("{hour:02}:{minute:02}:{second:02}")
}

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

enum MusicStreamResolveEvent {
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

enum MusicDownloadEvent {
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

struct MusicStreamSeed {
    source_url: String,
    title: String,
    album_title: String,
    thumbnail_url: String,
    thumbnail_hint: String,
    duration_text: String,
    duration_seconds: Option<f64>,
    direct_url: String,
    headers: Vec<(String, String)>,
    ext: String,
    format_id: String,
    acodec: String,
    expected_bytes: Option<u64>,
    cache_key: String,
    lyrics_track: Option<SubtitleOption>,
}

struct CompleteMusicCacheHit {
    cache_key: String,
    source_url: String,
    title: String,
    album_title: String,
    thumbnail_url: String,
    duration_seconds: Option<f64>,
    ext: String,
    format_id: String,
    acodec: String,
    expected_bytes: Option<u64>,
}

#[derive(Clone, Debug)]
struct CachedLrcTrack {
    path: PathBuf,
    modified: Option<SystemTime>,
    lines: Vec<LrcLine>,
    missing_checked_at: Option<Instant>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AudioPlaylistSnapshot {
    version: u32,
    items: Vec<AudioPlaylistItemSnapshot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AudioPlaylistItemSnapshot {
    source_url: String,
    title: String,
    #[serde(default)]
    album_title: String,
    #[serde(default)]
    thumbnail_hint: String,
    #[serde(default)]
    thumbnail_url: String,
    #[serde(default)]
    duration_text: String,
    duration_seconds: Option<f64>,
    #[serde(default)]
    stream_ext: String,
    #[serde(default)]
    stream_format_id: String,
    #[serde(default)]
    stream_acodec: String,
    expected_bytes: Option<u64>,
    #[serde(default)]
    cache_key: String,
    #[serde(default)]
    use_cookies: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct AudioCacheManifestSnapshot {
    source_url: String,
    title: String,
    album_title: String,
    duration_seconds: Option<f64>,
    ext: String,
    format_id: String,
    acodec: String,
    thumbnail_url: String,
    expected_bytes: Option<u64>,
    downloaded_bytes: Option<u64>,
    ranges: Vec<AudioCacheRangeSnapshot>,
    complete: bool,
    updated_unix_seconds: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AudioCacheRangeSnapshot {
    start: u64,
    end: u64,
}

#[derive(Clone, Debug)]
struct LrcLine {
    seconds: f64,
    text: String,
}

#[derive(Clone, Debug)]
struct MusicLyricsCacheJob {
    source_url: String,
    cache_key: String,
    language_code: String,
    use_cookies: bool,
}

#[derive(Clone, Debug, Default)]
struct CacheManagementSummary {
    total_bytes: u64,
    music_bytes: u64,
    expired_music_bytes: u64,
}

#[derive(Clone, Copy, Debug, Default)]
struct CacheRemovalSummary {
    bytes: u64,
    entries: u64,
}

fn restore_music_compact_item_from_cache_hit(item: &mut QueueItem, hit: &CompleteMusicCacheHit) {
    if !hit.source_url.trim().is_empty() {
        item.source_url = hit.source_url.clone();
    }
    if !hit.title.trim().is_empty() {
        item.title = hit.title.clone();
    }
    if !hit.album_title.trim().is_empty() {
        item.music_album_title = hit.album_title.clone();
    }
    if !hit.thumbnail_url.trim().is_empty() {
        item.thumbnail_url = hit.thumbnail_url.clone();
        item.thumbnail_hint = hit.thumbnail_url.clone();
    }
    if let Some(duration) = hit.duration_seconds {
        item.music_duration_seconds = Some(duration);
        item.duration_text = format_duration_seconds(duration);
    }
    item.music_stream_url.clear();
    item.music_stream_headers.clear();
    item.music_stream_ext = hit.ext.clone();
    item.music_stream_format_id = hit.format_id.clone();
    item.music_stream_acodec = hit.acodec.clone();
    item.music_stream_expected_bytes = hit.expected_bytes;
    item.music_cache_key = hit.cache_key.clone();
    item.compact_music_state = Some(CompactMusicState::Ready);
    item.last_error = None;
}

fn audio_playlist_item_snapshot(item: &QueueItem) -> AudioPlaylistItemSnapshot {
    AudioPlaylistItemSnapshot {
        source_url: item.source_url.clone(),
        title: item.title.clone(),
        album_title: item.music_album_title.clone(),
        thumbnail_hint: item.thumbnail_hint.clone(),
        thumbnail_url: item.thumbnail_url.clone(),
        duration_text: item.duration_text.clone(),
        duration_seconds: item.music_duration_seconds,
        stream_ext: item.music_stream_ext.clone(),
        stream_format_id: item.music_stream_format_id.clone(),
        stream_acodec: item.music_stream_acodec.clone(),
        expected_bytes: item.music_stream_expected_bytes,
        cache_key: item.music_cache_key.clone(),
        use_cookies: item.selection.use_cookies,
    }
}

pub struct YoutubePlaylistPrompt {
    pub source: String,
    pub kind: YoutubePlaylistPromptKind,
    pub risk: Option<YoutubePlaylistRisk>,
    pub music_compact: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YoutubePlaylistPromptKind {
    VideoAndPlaylist,
    HighRiskPlaylist,
}

struct ActiveWorkflow {
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    kind: WorkflowKind,
    tool: ToolKind,
    download_child: Option<Arc<Mutex<Option<Child>>>>,
    cancel_requested: Option<Arc<AtomicBool>>,
}

const BACKGROUND_DOWNLOAD_EVENT_BUDGET_PER_POLL: usize = 96;
const BACKGROUND_MUSIC_DOWNLOAD_EVENT_BUDGET_PER_POLL: usize = 24;

fn monotonic_progress(current: f32, next: f32) -> f32 {
    if next.is_finite() {
        current.max(next.clamp(0.0, 100.0))
    } else {
        current
    }
}

fn format_download_progress_detail(_language: Language, detail: &DownloadProgressDetail) -> String {
    let mut lines = Vec::new();

    push_detail_line(&mut lines, "Downloaded", detail.downloaded.as_deref());
    push_detail_line(&mut lines, "Total", detail.total.as_deref());
    push_detail_line(&mut lines, "Speed", detail.speed.as_deref());
    push_detail_line(&mut lines, "Elapsed", detail.elapsed.as_deref());
    push_detail_line(&mut lines, "Frame", detail.frame.as_deref());
    push_detail_line(&mut lines, "FPS", detail.fps.as_deref());
    push_detail_line(&mut lines, "Time", detail.time.as_deref());

    if lines.is_empty() {
        "Running".to_owned()
    } else {
        lines.join("\n")
    }
}

fn push_detail_line(lines: &mut Vec<String>, label: &str, value: Option<&str>) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    lines.push(format!("{label}\t{value}"));
}

fn single_mode_status_workflow_visible(run: &WorkflowRun, item: &QueueItem) -> bool {
    if !matches!(
        run.kind,
        WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
    ) {
        return false;
    }

    matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
        || !run.detail.trim().is_empty()
        || run.output_path.is_some()
        || run.error.is_some()
        || item.last_error.is_some()
}

fn workflow_tool_label(tool: &ToolKind) -> String {
    match tool {
        ToolKind::YtDlp => "yt-dlp".to_owned(),
        ToolKind::Ffmpeg => "FFMPEG".to_owned(),
        ToolKind::Aria2c => "aria2c".to_owned(),
        ToolKind::Other(label) => label.clone(),
    }
}

fn status_lines_contain(lines: &[(String, String)], label: &str) -> bool {
    lines.iter().any(|(candidate, _)| candidate == label)
}

fn queue_item_status_key(item: &QueueItem) -> &'static str {
    if let Some(run) = item.workflows.iter().rev().find(|run| {
        matches!(
            run.kind,
            WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
        ) && matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
    }) {
        return match run.state {
            WorkflowState::Queued => "Queued",
            WorkflowState::Running => "Running",
            _ => "Queued",
        };
    }

    if let Some(run) = item
        .workflows
        .iter()
        .rev()
        .find(|run| run.kind == WorkflowKind::DownloadMedia)
    {
        return match run.state {
            WorkflowState::Queued => "Queued",
            WorkflowState::Running => "Running",
            WorkflowState::Finished if item.last_error.is_some() => "Failed",
            WorkflowState::Finished => "Done",
            WorkflowState::Failed => "Failed",
            WorkflowState::Cancelled => "Cancelled",
        };
    }

    match &item.metadata_state {
        MetadataState::Idle => "Not started",
        MetadataState::Queued => "Waiting for analysis",
        MetadataState::Running => "Analyzing",
        MetadataState::Ready(_) => "Queued",
        MetadataState::Failed(_) => "Analysis failed",
    }
}

fn thumbnail_cache_key(url: &str, proxy_url: &str, no_check_certificates: bool) -> String {
    format!(
        "{}\n{}\n{}",
        proxy_url.trim(),
        no_check_certificates,
        url.trim()
    )
}

fn thumbnail_texture_id(key: &str) -> String {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    format!("proxied-thumbnail-{:016x}", hasher.finish())
}

fn thumbnail_needs_memory_loader(url: &str) -> bool {
    let url = url.trim();
    url.starts_with("http://") || url.starts_with("https://")
}

fn reset_item_for_new_work(item: &mut QueueItem, target_kind: DownloadTargetKind) {
    match target_kind {
        DownloadTargetKind::Normal => {
            item.progress.video = 0.0;
            item.progress.audio = 0.0;
            item.progress.subtitle = 0.0;
            item.progress.post_process = 0.0;
        }
        DownloadTargetKind::Video => item.progress.video = 0.0,
        DownloadTargetKind::Audio => item.progress.audio = 0.0,
        DownloadTargetKind::Subtitle => item.progress.subtitle = 0.0,
    }
    item.last_error = None;
}

impl AppState {
    pub fn new() -> Self {
        let (config, tool_paths) = AppConfig::load_runtime();
        Self::from_runtime(config, tool_paths)
    }

    pub fn from_runtime(mut config: AppConfig, tool_paths: ToolPaths) -> Self {
        let (analyze_result_tx, analyze_result_rx) = mpsc::channel();
        let (download_result_tx, download_result_rx) = mpsc::channel();
        let (post_process_result_tx, post_process_result_rx) = mpsc::channel();
        let (component_update_result_tx, component_update_result_rx) = mpsc::channel();
        let (thumbnail_result_tx, thumbnail_result_rx) = mpsc::channel();
        let (music_stream_result_tx, music_stream_result_rx) = mpsc::channel();
        let (music_playback_event_tx, music_playback_event_rx) = mpsc::channel();
        let (music_download_event_tx, music_download_event_rx) = mpsc::channel();
        let pending_ui_scale_percent = config.ui_scale_percent;
        let music_volume = config.music_volume.clamp(0.0, 1.0);
        let music_playback_mode = MusicPlaybackMode::from_config_value(&config.music_playback_mode);
        let app_mode = AppMode::from_config_value(&config.app_mode);
        let queue_display_mode = QueueDisplayMode::from_app_mode(app_mode);
        config.app_mode = app_mode.config_value().to_owned();
        config.queue_display_mode = queue_display_mode.config_value().to_owned();
        let mut state = Self {
            active_tab: AppTab::Main,
            url_input: String::new(),
            batch_input: String::new(),
            batch_enabled: true,
            monitor_clipboard: config.auto_paste_clipboard,
            last_clipboard_text: String::new(),
            last_clipboard_check: config.auto_paste_clipboard.then(Instant::now),
            clipboard_monitor_baseline_ready: false,
            empty_item_preview: VideoMetadata::empty_preview(),
            queue_items: Vec::new(),
            queue_display_mode,
            app_mode,
            item_defaults: {
                let mut defaults = DownloadOptions::default();
                defaults.output_dir = config.download_dir.clone();
                defaults.use_cookies = config.use_browser_cookies;
                defaults.use_aria2 = config.use_aria2;
                defaults.write_thumbnail = config.thumbnail_mode.writes();
                defaults.embed_thumbnail = config.thumbnail_mode.embeds();
                defaults.write_subtitles = config.subtitle_mode.writes();
                defaults.embed_subtitles = config.subtitle_mode.embeds();
                defaults.write_chapters = config.chapter_mode.writes();
                defaults.embed_chapters = config.chapter_mode.embeds();
                defaults
            },
            config,
            pending_ui_scale_percent,
            options_detail_page: None,
            prepare_detail_page: None,
            advance_detail_page: None,
            about_detail_target: AboutDetailTarget::App,
            tool_paths,
            prepare_report: PrepareReport::default(),
            prepare_tab_snoozed: false,
            last_action: String::new(),
            runtime_log: VecDeque::new(),
            tool_logs: VecDeque::new(),
            log_viewer_selected_step: None,
            log_viewer_expanded_action: None,
            tool_log_action_by_workflow: HashMap::new(),
            next_tool_log_action_id: 1,
            next_tool_log_step_id: 1,
            format_picker: FormatPickerState::default(),
            is_adding_batch: false,
            is_cancelling_batch_add: false,
            youtube_playlist_prompt: None,
            youtube_login_rescue_phase: YoutubeLoginRescuePhase::Idle,
            youtube_login_rescue_browser: None,
            youtube_login_rescue_site_name: None,
            youtube_login_rescue_target_url: String::new(),
            youtube_login_rescue_target_error: None,
            youtube_login_rescue_clipboard_prefilled: false,
            youtube_login_rescue_error: None,
            youtube_login_rescue_rx: None,
            analyze_result_rx,
            analyze_result_tx,
            batch_add_result_rx: None,
            batch_add_child: None,
            batch_add_cancel_requested: None,
            batch_add_music_compact: false,
            download_result_rx,
            download_result_tx,
            post_process_result_rx,
            post_process_result_tx,
            component_update_result_rx,
            component_update_result_tx,
            thumbnail_result_rx,
            thumbnail_result_tx,
            music_stream_result_rx,
            music_stream_result_tx,
            music_playback_event_rx,
            music_playback_event_tx,
            music_download_event_rx,
            music_download_event_tx,
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
            music_lyrics_cache: HashMap::new(),
            music_lyrics_display_line: None,
            music_lyrics_previous_line: None,
            music_lyrics_transition_started_at: None,
            thumbnail_cache: HashMap::new(),
            component_update_snapshot: component_update_startup_snapshot(),
            about_markdown_cache: CommonMarkCache::default(),
            app_instance_guard: register_app_instance(),
            font_content_revision: 1,
            active_workflows: HashMap::new(),
            next_queue_item_id: 1,
            next_workflow_run_id: 1,
        };

        cleanup_applied_update();
        state.restore_saved_audio_playlist();
        state.prepare_report = collect_dependency_presence_report(&state.tool_paths);
        state.sanitize_startup_prepare_component_update_snapshot();
        schedule_startup_transient_cleanup();
        if state.should_show_prepare_tab() {
            state.active_tab = AppTab::Prepare;
        }
        state
    }

    pub fn poll_clipboard_monitor(&mut self) {
        if !self.monitor_clipboard {
            return;
        }

        let now = Instant::now();
        if self
            .last_clipboard_check
            .is_some_and(|last| now.duration_since(last) < Duration::from_millis(800))
        {
            return;
        }
        self.last_clipboard_check = Some(now);

        let Some(text) = read_clipboard_text() else {
            return;
        };
        if !self.clipboard_monitor_baseline_ready {
            self.last_clipboard_text = text;
            self.clipboard_monitor_baseline_ready = true;
            return;
        }
        if text == self.last_clipboard_text {
            return;
        }
        self.last_clipboard_text = text.clone();

        if self.clipboard_monitor_input_blocked() {
            return;
        }

        let Some(url) = extract_monitored_youtube_url(&text) else {
            return;
        };
        if self.url_input.trim() == url && !self.config.clipboard_auto_add {
            return;
        }

        self.url_input = url.clone();
        if self.config.clipboard_auto_add {
            if let Err(error) = self.ensure_yt_dlp_ready() {
                self.last_action = error;
                return;
            }
            self.run_primary_url_action();
        } else {
            self.last_action = "Detected a YouTube URL from the clipboard.".to_owned();
        }
    }

    fn clipboard_monitor_input_blocked(&self) -> bool {
        self.url_input_locked() || self.format_picker.open
    }

    pub fn clipboard_monitor_enabled(&self) -> bool {
        self.monitor_clipboard
    }

    pub fn analyze_current_input(&mut self) {
        let Some(source) = self.primary_candidate_url() else {
            self.last_action = "There is no URL to analyze.".to_owned();
            return;
        };

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        self.last_action =
            i18n::format_fixed_english("Analyzing: {source}", &[("{source}", source.as_str())]);
        self.spawn_analyze_worker(source, None, None, false);
    }

    pub fn add_current_url_to_batch(&mut self) {
        if self.is_adding_batch {
            self.last_action = "Batch add is still running.".to_owned();
            return;
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        let source = self.url_input.trim();
        if source.is_empty() {
            self.last_action = "There is no URL to add.".to_owned();
            return;
        }

        let source = source.to_owned();
        if self.app_mode == AppMode::Origin {
            if youtube_url_has_video_and_playlist(&source) {
                let single_source =
                    youtube_url_force_single_video(&source).unwrap_or_else(|| source.clone());
                self.add_single_url_to_batch(single_source);
                return;
            }
            if looks_like_playlist_url(&source) {
                self.last_action = "Origin Mode does not support playlist URLs. Switch to Standard Mode to import a playlist."
                    .to_owned();
                return;
            }
            self.add_single_url_to_batch(source);
            return;
        }

        if youtube_url_has_video_and_playlist(&source) {
            match self.config.youtube_video_playlist_mode {
                YoutubeVideoPlaylistMode::Ask => {
                    let risk = if self.config.youtube_high_risk_playlist_prompt {
                        classify_youtube_playlist(&source)
                    } else {
                        None
                    };
                    self.youtube_playlist_prompt = Some(YoutubePlaylistPrompt {
                        source,
                        kind: YoutubePlaylistPromptKind::VideoAndPlaylist,
                        risk,
                        music_compact: false,
                    });
                    self.last_action =
                        "Detected a video URL that also contains a playlist.".to_owned();
                    return;
                }
                YoutubeVideoPlaylistMode::Video => {
                    let single_source =
                        youtube_url_force_single_video(&source).unwrap_or_else(|| source.clone());
                    self.add_single_url_to_batch(single_source);
                    return;
                }
                YoutubeVideoPlaylistMode::Ignore => {}
            }
        }

        if !looks_like_playlist_url(&source) {
            self.add_single_url_to_batch(source);
            return;
        }

        if self.config.youtube_high_risk_playlist_prompt {
            if let Some(risk) = classify_youtube_playlist(&source) {
                self.youtube_playlist_prompt = Some(YoutubePlaylistPrompt {
                    source,
                    kind: YoutubePlaylistPromptKind::HighRiskPlaylist,
                    risk: Some(risk),
                    music_compact: false,
                });
                self.last_action = i18n::format_fixed_english(
                    "Detected high-risk YouTube playlist: {kind}",
                    &[("{kind}", risk.kind.label())],
                );
                return;
            }
        }

        self.begin_batch_add(source);
    }

    pub fn run_primary_url_action(&mut self) {
        if self.app_mode == AppMode::Origin {
            self.add_current_url_to_batch();
        } else if self.queue_display_mode == QueueDisplayMode::Audio {
            self.add_current_url_to_music_compact_batch();
        } else if self.config.direct_download_on_add {
            self.immediate_download_current_url();
        } else {
            self.add_current_url_to_batch();
        }
    }

    pub fn primary_url_action_label_key(&self) -> &'static str {
        if self.is_adding_batch {
            if self.is_cancelling_batch_add {
                "action.stopping"
            } else {
                "action.stop"
            }
        } else if self.app_mode == AppMode::Origin {
            "action.analyze"
        } else if self.queue_display_mode == QueueDisplayMode::Audio {
            "action.add"
        } else if self.config.direct_download_on_add {
            "action.download"
        } else {
            "action.add"
        }
    }

    pub fn immediate_download_current_url(&mut self) {
        if self.is_adding_batch {
            self.last_action = "Batch add is still running.".to_owned();
            return;
        }
        let source = self.url_input.trim();
        if source.is_empty() {
            self.last_action = "There is no URL to download immediately.".to_owned();
            return;
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        let source = source.to_owned();
        let source = if youtube_url_has_video_and_playlist(&source) {
            youtube_url_force_single_video(&source).unwrap_or(source)
        } else {
            source
        };
        if looks_like_playlist_url(&source) {
            self.last_action = "Download now currently only handles one video URL.".to_owned();
            return;
        }

        let item_id = if self.app_mode == AppMode::Origin {
            if !self.active_workflows.is_empty() {
                self.last_action =
                    "Wait for the current Origin Mode item to finish first.".to_owned();
                return;
            }
            self.stop_music_playback();
            self.queue_items.clear();
            self.batch_input.clear();
            let item = self.build_queue_item_from_url(&source);
            let item_id = item.id;
            self.queue_items.push(item);
            item_id
        } else {
            self.ensure_queue_item_for_url(&source)
        };
        if self.app_mode != AppMode::Origin {
            self.url_input.clear();
        }
        let fallback_title = infer_title(&source, "Untitled task", "Imported {tail}");
        self.last_action = i18n::format_fixed_english(
            "Added and ready to download now: {title}",
            &[("{title}", fallback_title.as_str())],
        );
        let emit_json = self
            .queue_item_by_id(item_id)
            .is_some_and(|item| !item.metadata_loaded());
        let _ = self.start_download_task_at(item_id, emit_json);
    }

    pub fn confirm_youtube_playlist_prompt(&mut self) {
        let Some(prompt) = self.youtube_playlist_prompt.take() else {
            return;
        };
        if prompt.music_compact {
            self.begin_music_batch_add(prompt.source);
        } else {
            self.begin_batch_add(prompt.source);
        }
    }

    pub fn confirm_youtube_playlist_prompt_as_video(&mut self) {
        let Some(prompt) = self.youtube_playlist_prompt.take() else {
            return;
        };
        let source = youtube_url_force_single_video(&prompt.source).unwrap_or(prompt.source);
        if prompt.music_compact {
            self.add_single_music_compact_url(source);
        } else {
            self.add_single_url_to_batch(source);
        }
    }

    pub fn cancel_youtube_playlist_prompt(&mut self) {
        self.youtube_playlist_prompt = None;
        self.last_action = "Current action cancelled.".to_owned();
    }

    pub fn cancel_batch_add(&mut self) {
        self.is_cancelling_batch_add = true;
        if let Some(cancel_flag) = &self.batch_add_cancel_requested {
            cancel_flag.store(true, Ordering::Relaxed);
        }
        if let Some(child_handle) = &self.batch_add_child {
            request_batch_add_stop(child_handle);
        }
        self.last_action = "Stopping batch add...".to_owned();
    }

    pub fn poll_background_work(&mut self) {
        self.poll_media_session_commands();
        self.poll_youtube_login_rescue();

        loop {
            match self.analyze_result_rx.try_recv() {
                Ok(message) => {
                    let analyze_succeeded = message.result.is_ok();
                    if let (Some(action_id), Some(command_line)) =
                        (message.tool_log_action_id, message.command_line.as_ref())
                    {
                        self.push_tool_log_step(
                            action_id,
                            if analyze_succeeded {
                                ToolLogStatus::Success
                            } else {
                                ToolLogStatus::Failed
                            },
                            "yt-dlp",
                            "analyze",
                            command_line.clone(),
                        );
                    }

                    match message.result {
                        Ok(json) => {
                            if let Some(item_id) = message.target_item_id {
                                if let Some(item) = self.queue_item_mut_by_id(item_id) {
                                    item.cookie_policy = if message.used_cookies {
                                        CookiePolicy::Required
                                    } else {
                                        CookiePolicy::NotNeeded
                                    };
                                }
                            }
                            self.apply_analysis_json(
                                json,
                                Some(message.source),
                                message.target_item_id,
                                message.workflow_id,
                            );
                        }
                        Err(error) => {
                            let should_retry_with_cookies = message
                                .target_item_id
                                .and_then(|item_id| self.queue_item_by_id(item_id))
                                .is_some_and(|item| item.selection.use_cookies)
                                && !message.used_cookies
                                && message
                                    .target_item_id
                                    .and_then(|item_id| self.queue_item_by_id(item_id))
                                    .is_some_and(|item| {
                                        item.cookie_policy == CookiePolicy::Unknown
                                    })
                                && should_retry_analyze_with_cookies(&error);

                            if should_retry_with_cookies {
                                if let Some(item_id) = message.target_item_id {
                                    if let Some(item) = self.queue_item_mut_by_id(item_id) {
                                        item.cookie_policy = CookiePolicy::Required;
                                    }
                                }
                                self.last_action = i18n::format_fixed_english(
                                    "Retrying analysis with cookies: {source}",
                                    &[("{source}", message.source.as_str())],
                                );
                                self.spawn_analyze_worker(
                                    message.source,
                                    message.target_item_id,
                                    message.workflow_id,
                                    true,
                                );
                                continue;
                            }
                            eprintln!("[analyze] {error}");
                            if let Some(item_id) = message.target_item_id {
                                if let Some(item) = self.queue_item_mut_by_id(item_id) {
                                    item.metadata_state = MetadataState::Failed(error.clone());
                                    item.last_error = Some(error.clone());
                                    if let Some(workflow_id) = message.workflow_id {
                                        if let Some(run) = item
                                            .workflows
                                            .iter_mut()
                                            .find(|run| run.id == workflow_id)
                                        {
                                            run.state = WorkflowState::Failed;
                                            run.error = Some(error.clone());
                                        }
                                        self.unregister_active_workflow(workflow_id);
                                    }
                                }
                            }
                            self.last_action = error;
                        }
                    }
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        loop {
            match self.music_stream_result_rx.try_recv() {
                Ok(message) => self.apply_music_stream_result(message),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        loop {
            match self.music_playback_event_rx.try_recv() {
                Ok(event) => self.apply_music_playback_event(event),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        for _ in 0..BACKGROUND_MUSIC_DOWNLOAD_EVENT_BUDGET_PER_POLL {
            match self.music_download_event_rx.try_recv() {
                Ok(event) => self.apply_music_download_event(event),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        self.maybe_prefetch_next_music_item();

        if let Some(rx) = self.batch_add_result_rx.take() {
            let mut keep_rx = true;
            loop {
                match rx.try_recv() {
                    Ok(BatchAddEvent::ToolCommandFinished {
                        action_id,
                        command_line,
                        success,
                    }) => {
                        self.push_tool_log_step(
                            action_id,
                            self.tool_log_status_for_batch_step(success),
                            "yt-dlp",
                            "batch import",
                            command_line,
                        );
                    }
                    Ok(BatchAddEvent::ItemAdded { source, seed }) => {
                        let cancel_requested = self
                            .batch_add_cancel_requested
                            .as_ref()
                            .is_some_and(|flag| flag.load(Ordering::Relaxed));
                        if !cancel_requested {
                            if self.batch_add_music_compact {
                                self.append_music_compact_seed(seed);
                            } else {
                                self.append_batch_seed(&source, seed);
                            }
                        }
                    }
                    Ok(BatchAddEvent::Finished {
                        source,
                        added,
                        stopped_by_limit,
                    }) => {
                        self.is_adding_batch = false;
                        self.is_cancelling_batch_add = false;
                        self.batch_add_child = None;
                        self.batch_add_cancel_requested = None;
                        self.batch_add_music_compact = false;
                        keep_rx = false;
                        if added == 0 {
                            self.last_action = "No new items were found in the batch.".to_owned();
                        } else if stopped_by_limit {
                            self.last_action = i18n::format_fixed_english(
                                "Added {count} batch items from the playlist (limit applied).",
                                &[("{count}", &added.to_string())],
                            );
                        } else if added == 1 {
                            let fallback_title =
                                infer_title(&source, "Untitled task", "Imported {tail}");
                            self.last_action = i18n::format_fixed_english(
                                "Added to batch: {title}",
                                &[("{title}", fallback_title.as_str())],
                            );
                        } else {
                            self.last_action = i18n::format_fixed_english(
                                "Added {count} batch items from the playlist.",
                                &[("{count}", &added.to_string())],
                            );
                        }
                        self.url_input.clear();
                        break;
                    }
                    Ok(BatchAddEvent::Failed { error }) => {
                        self.is_adding_batch = false;
                        self.is_cancelling_batch_add = false;
                        self.batch_add_child = None;
                        self.batch_add_cancel_requested = None;
                        self.batch_add_music_compact = false;
                        keep_rx = false;
                        self.last_action = error;
                        break;
                    }
                    Ok(BatchAddEvent::Cancelled { added }) => {
                        self.is_adding_batch = false;
                        self.is_cancelling_batch_add = false;
                        self.batch_add_child = None;
                        self.batch_add_cancel_requested = None;
                        self.batch_add_music_compact = false;
                        keep_rx = false;
                        self.last_action = if added == 0 {
                            "Batch add cancelled.".to_owned()
                        } else {
                            i18n::format_fixed_english(
                                "Batch add cancelled; {count} items were added.",
                                &[("{count}", &added.to_string())],
                            )
                        };
                        self.url_input.clear();
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.is_adding_batch = false;
                        self.is_cancelling_batch_add = false;
                        self.batch_add_child = None;
                        self.batch_add_cancel_requested = None;
                        self.batch_add_music_compact = false;
                        keep_rx = false;
                        self.last_action = "Batch add was interrupted.".to_owned();
                        break;
                    }
                }
            }
            if keep_rx {
                self.batch_add_result_rx = Some(rx);
            }
        }

        loop {
            match self.component_update_result_rx.try_recv() {
                Ok(ComponentUpdateEvent::Snapshot(snapshot)) => {
                    self.component_update_snapshot = snapshot;
                    self.last_action = self.component_update_snapshot.message.clone();
                }
                Ok(ComponentUpdateEvent::Finished(snapshot)) => {
                    self.component_update_snapshot = snapshot;
                    self.last_action = self.component_update_snapshot.message.clone();
                    self.sync_available_managed_tool_paths_from_update_snapshot();
                    self.refresh_prepare_report();
                    if !self.should_show_prepare_tab() && self.active_tab == AppTab::Prepare {
                        self.active_tab = AppTab::Main;
                    }
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        for _ in 0..BACKGROUND_DOWNLOAD_EVENT_BUDGET_PER_POLL {
            match self.download_result_rx.try_recv() {
                Ok(DownloadEvent::Metadata { item_id, json }) => {
                    self.apply_analysis_json(json, None, Some(item_id), None);
                }
                Ok(DownloadEvent::ToolCommandFinished {
                    item_id: _,
                    workflow_id,
                    target_kind,
                    command_line,
                    success,
                    detail,
                }) => {
                    let action_id = self.workflow_tool_log_action(
                        workflow_id,
                        "origin",
                        download_target_log_action(target_kind),
                    );
                    self.push_tool_log_step_with_detail_without_failure_reveal(
                        action_id,
                        self.tool_log_status_for_workflow_step(workflow_id, success),
                        "yt-dlp",
                        "download",
                        command_line,
                        detail,
                    );
                }
                Ok(DownloadEvent::RecoveryStep {
                    item_id: _,
                    workflow_id,
                    target_kind,
                    action,
                    detail,
                    recover_previous_failure,
                    resolved_success,
                }) => {
                    let action_id = self.workflow_tool_log_action(
                        workflow_id,
                        "origin",
                        download_target_log_action(target_kind),
                    );
                    if recover_previous_failure {
                        self.mark_last_failed_tool_log_step_as_recoverable(action_id);
                    }
                    let status = if resolved_success {
                        ToolLogStatus::Success
                    } else {
                        ToolLogStatus::Skipped
                    };
                    self.push_tool_log_step(action_id, status, "v2", action, detail);
                }
                Ok(DownloadEvent::Progress {
                    item_id,
                    workflow_id,
                    slot,
                    percent,
                    detail,
                }) => {
                    if !self.is_current_download_progress_event(item_id, workflow_id) {
                        continue;
                    }
                    let language = self.language();
                    if let Some(item) = self.queue_item_mut_by_id(item_id) {
                        let display_percent = percent.clamp(0.0, 100.0);
                        if let Some(run) =
                            item.workflows.iter_mut().find(|run| run.id == workflow_id)
                        {
                            run.progress = monotonic_progress(run.progress, display_percent);
                            if let Some(detail) = detail.as_ref() {
                                run.detail = format_download_progress_detail(language, detail);
                            }
                        }
                        match slot {
                            DownloadProgressSlot::Video => {
                                item.progress.video =
                                    monotonic_progress(item.progress.video, display_percent);
                            }
                            DownloadProgressSlot::Audio => {
                                item.progress.audio =
                                    monotonic_progress(item.progress.audio, display_percent);
                            }
                            DownloadProgressSlot::Subtitle => {
                                item.progress.subtitle =
                                    monotonic_progress(item.progress.subtitle, display_percent);
                            }
                            DownloadProgressSlot::Both => {
                                item.progress.video =
                                    monotonic_progress(item.progress.video, display_percent);
                                item.progress.audio =
                                    monotonic_progress(item.progress.audio, display_percent);
                            }
                        }
                    }
                }
                Ok(DownloadEvent::Finished(message)) => {
                    let finished_item_id = message.item_id;
                    let notification_title = self
                        .queue_item_by_id(message.item_id)
                        .map(|item| item.title.trim().to_owned())
                        .filter(|title| !title.is_empty())
                        .unwrap_or_else(|| "Download item".to_owned());
                    let notification_result = message.result.clone();
                    let should_send_windows_toast =
                        message.workflow_kind == WorkflowKind::DownloadMedia;
                    self.unregister_active_workflow(message.workflow_id);
                    self.finish_workflow_tool_log(message.workflow_id);
                    let mut pending_post_process_input = None;
                    if let Some(item) = self.queue_item_mut_by_id(message.item_id) {
                        if let Some(run) = item
                            .workflows
                            .iter_mut()
                            .find(|run| run.id == message.workflow_id)
                        {
                            match &message.result {
                                Ok(output_path) => {
                                    run.state = WorkflowState::Finished;
                                    run.output_path = Some(output_path.clone());
                                    match message.target_kind {
                                        DownloadTargetKind::Normal => {
                                            item.progress.video = 100.0;
                                            item.progress.audio = 100.0;
                                            if let Some(actual_file_name) = Path::new(output_path)
                                                .file_name()
                                                .and_then(|value| value.to_str())
                                                .map(ToOwned::to_owned)
                                            {
                                                item.selection.file_name = actual_file_name;
                                            }
                                            item.completed_selection = Some(
                                                CompletedSelection::from_selection(&item.selection),
                                            );
                                        }
                                        DownloadTargetKind::Video => item.progress.video = 100.0,
                                        DownloadTargetKind::Audio => item.progress.audio = 100.0,
                                        DownloadTargetKind::Subtitle => {
                                            item.progress.subtitle = 100.0
                                        }
                                    }
                                    item.last_output_path = Some(output_path.clone());
                                    item.last_error = None;
                                    if message.workflow_kind == WorkflowKind::DownloadMedia
                                        && message.target_kind == DownloadTargetKind::Normal
                                    {
                                        pending_post_process_input = Some(output_path.clone());
                                    }
                                }
                                Err(error) if error == DOWNLOAD_CANCELLED_MESSAGE => {
                                    run.state = WorkflowState::Cancelled;
                                    run.error = None;
                                    item.last_error = None;
                                }
                                Err(error) => {
                                    run.state = WorkflowState::Failed;
                                    run.error = Some(error.clone());
                                    item.last_error = Some(error.clone());
                                }
                            }
                        }
                    }

                    let post_process_started =
                        pending_post_process_input
                            .as_deref()
                            .is_some_and(|output_path| {
                                self.maybe_start_builtin_transcode_post_process(
                                    message.item_id,
                                    output_path,
                                )
                            });

                    match message.result {
                        Ok(output_path) => {
                            self.push_runtime_log(format!("Download finished: {output_path}"));
                            if !post_process_started {
                                self.last_action.clear();
                            }
                        }
                        Err(error) if error == DOWNLOAD_CANCELLED_MESSAGE => {
                            self.push_runtime_log("Download cancelled".to_owned());
                            self.last_action = "Download stopped.".to_owned();
                        }
                        Err(error) => {
                            self.push_runtime_log(format!("Download failed: {error}"));
                            eprintln!("[download] {error}");
                            self.reveal_log_tab_for_tool_failure();
                            self.last_action = error;
                        }
                    }

                    if should_send_windows_toast && !post_process_started {
                        self.send_download_result_windows_toast(
                            notification_title,
                            notification_result,
                        );
                        self.start_next_queued_download_after(finished_item_id);
                    }
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        loop {
            match self.post_process_result_rx.try_recv() {
                Ok(PostProcessEvent::Progress {
                    item_id,
                    workflow_id,
                    percent,
                }) => {
                    if let Some(item) = self.queue_item_mut_by_id(item_id) {
                        item.progress.post_process = percent;
                        if let Some(run) =
                            item.workflows.iter_mut().find(|run| run.id == workflow_id)
                        {
                            run.progress = percent;
                        }
                    }
                }
                Ok(PostProcessEvent::ToolCommandFinished {
                    item_id: _,
                    workflow_id,
                    tool,
                    action,
                    command_line,
                    success,
                    detail,
                }) => {
                    let action_id =
                        self.workflow_tool_log_action(workflow_id, "origin", "post-process");
                    self.push_tool_log_step_with_detail_without_failure_reveal(
                        action_id,
                        self.tool_log_status_for_workflow_step(workflow_id, success),
                        tool,
                        action,
                        command_line,
                        detail,
                    );
                }
                Ok(PostProcessEvent::RecoveryStep {
                    item_id: _,
                    workflow_id,
                    action,
                    detail,
                    recover_previous_failure,
                    resolved_success,
                }) => {
                    let action_id =
                        self.workflow_tool_log_action(workflow_id, "origin", "post-process");
                    if recover_previous_failure {
                        self.mark_last_failed_tool_log_step_as_recoverable(action_id);
                    }
                    let status = if resolved_success {
                        ToolLogStatus::Success
                    } else {
                        ToolLogStatus::Skipped
                    };
                    self.push_tool_log_step(action_id, status, "v2", action, detail);
                }
                Ok(PostProcessEvent::Finished(message)) => {
                    let finished_item_id = message.item_id;
                    let notification_title = self
                        .queue_item_by_id(message.item_id)
                        .map(|item| item.title.trim().to_owned())
                        .filter(|title| !title.is_empty())
                        .unwrap_or_else(|| "Download item".to_owned());
                    let notification_result = message.result.clone();
                    self.unregister_active_workflow(message.workflow_id);

                    if let Some(item) = self.queue_item_mut_by_id(message.item_id) {
                        if let Some(run) = item
                            .workflows
                            .iter_mut()
                            .find(|run| run.id == message.workflow_id)
                        {
                            match &message.result {
                                Ok(output_path) => {
                                    run.state = WorkflowState::Finished;
                                    run.progress = 100.0;
                                    run.output_path = Some(output_path.clone());
                                    item.progress.post_process = 100.0;
                                    item.last_output_path = Some(output_path.clone());
                                    if let Some(actual_file_name) = Path::new(output_path)
                                        .file_name()
                                        .and_then(|value| value.to_str())
                                        .map(ToOwned::to_owned)
                                    {
                                        item.selection.file_name = actual_file_name;
                                    }
                                    item.completed_selection =
                                        Some(CompletedSelection::from_selection(&item.selection));
                                    item.last_error = None;
                                }
                                Err(error) if error == POST_PROCESS_CANCELLED_MESSAGE => {
                                    run.state = WorkflowState::Cancelled;
                                    run.error = None;
                                    item.last_error = None;
                                }
                                Err(error) => {
                                    run.state = WorkflowState::Failed;
                                    run.error = Some(error.clone());
                                    item.last_error = Some(error.clone());
                                    item.completed_selection = None;
                                }
                            }
                        }
                    }

                    match message.result {
                        Ok(output_path) => {
                            self.push_runtime_log(format!("Post-process finished: {output_path}"));
                            self.last_action.clear();
                        }
                        Err(error) if error == POST_PROCESS_CANCELLED_MESSAGE => {
                            self.push_runtime_log("Post-process cancelled".to_owned());
                            self.last_action = "Download stopped.".to_owned();
                        }
                        Err(error) => {
                            self.push_runtime_log(format!("Post-process failed: {error}"));
                            eprintln!("[post-process] {error}");
                            self.reveal_log_tab_for_tool_failure();
                            self.last_action = error;
                        }
                    }

                    self.send_download_result_windows_toast(
                        notification_title,
                        notification_result,
                    );
                    self.start_next_queued_download_after(finished_item_id);
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        self.sync_media_session();
    }

    pub fn queue_batch(&mut self) {
        let urls = self.parsed_batch_urls();
        let count = urls.len();
        if count == 0 {
            self.last_action = "There is no URL to add to the batch.".to_owned();
            return;
        }

        self.queue_items = urls
            .iter()
            .map(|url| self.build_queue_item_from_url(url))
            .collect();
        if self.queue_display_mode == QueueDisplayMode::Audio {
            self.prepare_queue_items_for_audio_mode();
            self.save_active_audio_playlist_if_needed();
        }
        self.last_action = i18n::format_fixed_english(
            "Added {count} queued items from batch input.",
            &[("{count}", &count.to_string())],
        );
    }

    pub fn app_mode(&self) -> AppMode {
        self.app_mode
    }

    pub fn set_app_mode(&mut self, mode: AppMode) {
        if self.app_mode == mode {
            return;
        }
        self.app_mode = mode;
        match mode {
            AppMode::Audio => {
                if self.queue_display_mode != QueueDisplayMode::Audio {
                    self.enter_audio_queue_context();
                    self.queue_display_mode = QueueDisplayMode::Audio;
                    self.config.queue_display_mode =
                        QueueDisplayMode::Audio.config_value().to_owned();
                }
            }
            AppMode::Origin | AppMode::Standard => {
                if self.queue_display_mode == QueueDisplayMode::Audio {
                    self.leave_audio_queue_context();
                    self.queue_display_mode = QueueDisplayMode::Normal;
                    self.config.queue_display_mode =
                        QueueDisplayMode::Normal.config_value().to_owned();
                }
            }
        }
        self.config.app_mode = mode.config_value().to_owned();
        let _ = self.config.save();
        self.last_action = self.ui_i18n_text_for_key(mode.label_key()).to_owned();
    }

    pub fn queue_display_mode(&self) -> QueueDisplayMode {
        self.queue_display_mode
    }

    pub fn queue_display_mode_is_audio(&self) -> bool {
        self.queue_display_mode == QueueDisplayMode::Audio
    }

    pub fn set_queue_display_mode(&mut self, mode: QueueDisplayMode) {
        if self.queue_display_mode == mode {
            return;
        }
        if mode == QueueDisplayMode::Audio {
            self.enter_audio_queue_context();
            self.app_mode = AppMode::Audio;
        } else {
            self.leave_audio_queue_context();
            if self.app_mode == AppMode::Audio {
                self.app_mode = AppMode::Standard;
            }
        }
        self.queue_display_mode = mode;
        self.config.queue_display_mode = mode.config_value().to_owned();
        self.config.app_mode = self.app_mode.config_value().to_owned();
        let _ = self.config.save();
        let mode_label_key = match mode {
            QueueDisplayMode::Normal => "Standard",
            QueueDisplayMode::Audio => "Audio",
        };
        self.last_action =
            i18n::format_fixed_english("List mode: {mode}", &[("{mode}", mode_label_key)]);
    }

    fn enter_audio_queue_context(&mut self) {
        if self.queue_display_mode == QueueDisplayMode::Audio {
            return;
        }
        self.non_audio_queue_items = std::mem::take(&mut self.queue_items);
        self.queue_items = std::mem::take(&mut self.audio_queue_items);
        for item in &mut self.queue_items {
            item.view_kind = QueueItemViewKind::MusicCompact;
            item.compact_music_state
                .get_or_insert(CompactMusicState::Ready);
        }
        self.prune_music_navigation_state();
        self.rebuild_batch_input_from_queue();
    }

    fn leave_audio_queue_context(&mut self) {
        if self.queue_display_mode != QueueDisplayMode::Audio {
            return;
        }
        self.stop_music_playback_for_audio_context();
        self.audio_queue_items = std::mem::take(&mut self.queue_items);
        self.save_audio_playlist_items(&self.audio_queue_items);
        self.queue_items = std::mem::take(&mut self.non_audio_queue_items);
        for item in &mut self.queue_items {
            item.view_kind = QueueItemViewKind::VideoCard;
            item.compact_music_state = None;
        }
        self.rebuild_batch_input_from_queue();
    }

    fn stop_music_playback_for_audio_context(&mut self) {
        if let Some(control) = self.music_playback.take() {
            control.stop();
            self.mark_music_playback_state(control.item_id, CompactMusicState::Ready);
        }
        self.music_player_current_item_id = None;
        self.music_player_error = None;
        self.music_reserved_next_item_id = None;
        self.cancel_music_prefetch();
        self.music_scroll_to_item_id = None;
        self.music_history_back.clear();
        self.music_history_forward.clear();
        self.media_session.clear();
    }

    pub fn music_player_visible(&self) -> bool {
        self.queue_display_mode == QueueDisplayMode::Audio && !self.queue_items.is_empty()
    }

    fn poll_media_session_commands(&mut self) {
        while let Some(command) = self.media_session.poll_command() {
            if self.queue_display_mode != QueueDisplayMode::Audio {
                continue;
            }
            match command {
                MediaSessionCommand::Play => {
                    if !self.music_player_is_playing() {
                        self.toggle_music_playback();
                    }
                }
                MediaSessionCommand::Pause => {
                    if self.music_player_is_playing() {
                        self.toggle_music_playback();
                    }
                }
                MediaSessionCommand::Previous => self.previous_music_item(),
                MediaSessionCommand::Next => self.next_music_item(),
                MediaSessionCommand::Stop => self.stop_music_playback(),
            }
        }
    }

    fn sync_media_session(&mut self) {
        if self.queue_display_mode != QueueDisplayMode::Audio {
            self.media_session.clear();
            return;
        }

        let Some(item_id) = self.music_player_current_item_id else {
            self.media_session.clear();
            return;
        };
        let Some(item) = self.queue_item_by_id(item_id) else {
            self.media_session.clear();
            return;
        };

        let duration_seconds = self
            .music_playback
            .as_ref()
            .and_then(MusicPlaybackControl::duration_seconds)
            .or(item.music_duration_seconds)
            .or_else(|| duration_text_to_seconds(&item.duration_text));
        let position_seconds = self
            .music_playback
            .as_ref()
            .map(MusicPlaybackControl::playback_seconds)
            .unwrap_or(0.0);
        let status = match item.compact_music_state.unwrap_or(CompactMusicState::Ready) {
            CompactMusicState::Resolving | CompactMusicState::Buffering => {
                MediaSessionPlaybackStatus::Changing
            }
            CompactMusicState::Playing => MediaSessionPlaybackStatus::Playing,
            CompactMusicState::Paused => MediaSessionPlaybackStatus::Paused,
            CompactMusicState::Failed => MediaSessionPlaybackStatus::Stopped,
            CompactMusicState::Ready => {
                if self.music_playback.is_some() {
                    if self.music_player_is_playing() {
                        MediaSessionPlaybackStatus::Playing
                    } else {
                        MediaSessionPlaybackStatus::Paused
                    }
                } else {
                    MediaSessionPlaybackStatus::Paused
                }
            }
        };

        let display_title = stable_media_session_title(&item.title, &item.source_url);
        let (artist, title) = split_artist_title_for_media_session(&display_title);
        let track = MediaSessionTrack {
            key: format!("{}:{}", item.id, item.source_url),
            title,
            artist,
            thumbnail_url: item.thumbnail_url.clone(),
            duration_seconds,
        };
        let timeline = MediaSessionTimeline {
            position_seconds,
            duration_seconds: track.duration_seconds,
        };
        self.media_session.update(&track, status, timeline);
    }

    pub fn queue_mode_downloads_as_audio(&self) -> bool {
        self.queue_display_mode == QueueDisplayMode::Audio
    }

    pub fn music_download_prompt_open(&self) -> bool {
        self.music_download_prompt_open
    }

    pub fn request_main_download(&mut self) {
        if self.queue_mode_downloads_as_audio() {
            self.prepare_queue_items_for_audio_mode();
            self.music_download_prompt_choice.mode = MusicDownloadMode::Original;
            self.music_download_prompt_choice.embed_cover = true;
            self.music_download_prompt_choice.write_tags = true;
            self.music_download_prompt_open = true;
        } else {
            self.start_single_download();
        }
    }

    pub fn music_download_prompt_choice(&self) -> MusicDownloadChoice {
        self.music_download_prompt_choice
    }

    pub fn set_music_download_prompt_mode(&mut self, mode: MusicDownloadMode) {
        self.music_download_prompt_choice.mode = mode;
    }

    pub fn set_music_download_original_preference(&mut self, preference: MusicOriginalPreference) {
        self.music_download_prompt_choice.original_preference = preference;
    }

    pub fn set_music_download_unified_format(&mut self, format: MusicDownloadFormat) {
        self.music_download_prompt_choice.unified_format = format;
    }

    pub fn set_music_download_embed_cover(&mut self, enabled: bool) {
        self.music_download_prompt_choice.embed_cover = enabled;
    }

    pub fn set_music_download_write_tags(&mut self, enabled: bool) {
        self.music_download_prompt_choice.write_tags = enabled;
    }

    pub fn cancel_music_download_prompt(&mut self) {
        self.music_download_prompt_open = false;
    }

    pub fn confirm_music_download_choice(&mut self) {
        let mut choice = self.music_download_prompt_choice;
        choice.mode = MusicDownloadMode::Original;
        choice.embed_cover = true;
        choice.write_tags = true;
        self.music_download_prompt_choice = choice;
        self.music_download_prompt_open = false;
        self.start_download_with_music_choice(choice);
    }

    fn start_download_with_music_choice(&mut self, choice: MusicDownloadChoice) {
        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }
        if self.has_running_download_workflow() {
            self.last_action =
                "A download is already running. Please wait for it to finish.".to_owned();
            return;
        }

        self.active_music_download_choice = Some(choice);
        self.enqueue_download_ready_items();

        let Some(item_id) = self
            .queue_items
            .iter()
            .find(|item| item_latest_download_state(item).is_some_and(is_pending_download_state))
            .map(|item| item.id)
        else {
            self.last_action = "There are no runnable batch items.".to_owned();
            return;
        };

        if self.queue_mode_downloads_as_audio() {
            let _ = self.start_music_download_task_at(item_id, choice);
        } else {
            let emit_json = self
                .queue_item_by_id(item_id)
                .is_some_and(|item| !item.metadata_loaded());
            let _ = self.start_download_task_at(item_id, emit_json);
        }
    }

    pub fn start_single_download(&mut self) {
        let Some(url) = self.primary_candidate_url() else {
            self.last_action = "There is no URL to download.".to_owned();
            return;
        };

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        if self.queue_items.is_empty() && !self.parsed_batch_urls().is_empty() {
            self.queue_batch();
        }

        if self.queue_items.is_empty() {
            self.queue_items = vec![self.build_queue_item_from_url(&url)];
        }

        if self.has_running_download_workflow() {
            self.last_action =
                "A download is already running. Please wait for it to finish.".to_owned();
            return;
        }

        self.enqueue_download_ready_items();

        let Some(item_id) = self
            .queue_items
            .iter()
            .find(|item| item_latest_download_state(item).is_some_and(is_pending_download_state))
            .map(|item| item.id)
        else {
            self.last_action = "There are no runnable batch items.".to_owned();
            return;
        };

        let emit_json = self
            .queue_item_by_id(item_id)
            .is_some_and(|item| !item.metadata_loaded());
        let _ = self.start_download_task_at(item_id, emit_json);
    }

    fn enqueue_download_ready_items(&mut self) {
        let ready_item_ids = self
            .queue_items
            .iter()
            .filter(|item| !item.source_url.trim().is_empty())
            .filter(|item| match item_latest_download_state(item) {
                None => true,
                Some(
                    WorkflowState::Failed | WorkflowState::Finished | WorkflowState::Cancelled,
                ) => true,
                Some(_) => false,
            })
            .map(|item| item.id)
            .collect::<Vec<_>>();

        for item_id in ready_item_ids {
            let workflow_id = self.alloc_workflow_run_id();
            let Some(item) = self.queue_item_mut_by_id(item_id) else {
                continue;
            };
            reset_item_for_new_work(item, DownloadTargetKind::Normal);
            item.completed_selection = None;
            let mut run = WorkflowRun::new(
                workflow_id,
                WorkflowKind::DownloadMedia,
                ToolKind::YtDlp,
                WorkflowState::Queued,
            );
            run.detail = item.source_url.clone();
            item.workflows.push(run);
        }
    }

    fn ensure_queue_item_for_url(&mut self, url: &str) -> QueueItemId {
        let source_key = canonical_queue_source_key(url);
        if let Some(item) = self
            .queue_items
            .iter()
            .find(|item| canonical_queue_source_key(&item.source_url) == source_key)
        {
            return item.id;
        }

        let item = self.build_queue_item_from_url(url);
        let item_id = item.id;
        self.queue_items.push(item);
        self.batch_input_push_unique(url);
        item_id
    }

    fn batch_input_push_unique(&mut self, url: &str) {
        let source_key = canonical_queue_source_key(url);
        if self
            .batch_input
            .lines()
            .map(str::trim)
            .any(|line| canonical_queue_source_key(line) == source_key)
        {
            return;
        }
        if !self.batch_input.trim().is_empty() {
            self.batch_input.push('\n');
        }
        self.batch_input.push_str(url);
    }

    fn alloc_queue_item_id(&mut self) -> QueueItemId {
        let id = self.next_queue_item_id;
        self.next_queue_item_id += 1;
        id
    }

    fn alloc_workflow_run_id(&mut self) -> WorkflowRunId {
        let id = self.next_workflow_run_id;
        self.next_workflow_run_id += 1;
        id
    }

    fn register_active_workflow(
        &mut self,
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        kind: WorkflowKind,
        tool: ToolKind,
    ) {
        self.active_workflows.insert(
            workflow_id,
            ActiveWorkflow {
                item_id,
                workflow_id,
                kind,
                tool,
                download_child: None,
                cancel_requested: None,
            },
        );
    }

    fn attach_active_download_process(
        &mut self,
        workflow_id: WorkflowRunId,
        child_handle: Arc<Mutex<Option<Child>>>,
        cancel_requested: Arc<AtomicBool>,
    ) {
        if let Some(workflow) = self.active_workflows.get_mut(&workflow_id) {
            workflow.download_child = Some(child_handle);
            workflow.cancel_requested = Some(cancel_requested);
        }
    }

    fn unregister_active_workflow(&mut self, workflow_id: WorkflowRunId) {
        self.active_workflows.remove(&workflow_id);
    }

    fn active_workflow_cancel_requested(&self, workflow_id: WorkflowRunId) -> bool {
        self.active_workflows
            .get(&workflow_id)
            .and_then(|workflow| workflow.cancel_requested.as_ref())
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
    }

    fn is_current_download_progress_event(
        &self,
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
    ) -> bool {
        self.active_workflows
            .get(&workflow_id)
            .is_some_and(|workflow| {
                workflow.item_id == item_id
                    && matches!(
                        workflow.kind,
                        WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia
                    )
            })
    }

    fn tool_log_status_for_workflow_step(
        &self,
        workflow_id: WorkflowRunId,
        success: bool,
    ) -> ToolLogStatus {
        if success {
            ToolLogStatus::Success
        } else if self.active_workflow_cancel_requested(workflow_id) {
            ToolLogStatus::Skipped
        } else {
            ToolLogStatus::Failed
        }
    }

    fn tool_log_status_for_batch_step(&self, success: bool) -> ToolLogStatus {
        if success {
            ToolLogStatus::Success
        } else if self.is_cancelling_batch_add
            || self
                .batch_add_cancel_requested
                .as_ref()
                .is_some_and(|flag| flag.load(Ordering::Relaxed))
        {
            ToolLogStatus::Skipped
        } else {
            ToolLogStatus::Failed
        }
    }

    fn has_running_download_workflow(&self) -> bool {
        self.active_workflows
            .values()
            .any(|workflow| workflow.kind == WorkflowKind::DownloadMedia)
    }

    fn maybe_start_builtin_transcode_post_process(
        &mut self,
        item_id: QueueItemId,
        input_path: &str,
    ) -> bool {
        if !self.config.post_download_conversion_enabled {
            return false;
        }
        let plan = resolve_transcode_plan(&self.config.transcode_intent);
        if !plan.is_executable() {
            return false;
        }
        let Some(profile) = plan.backend_profile else {
            return false;
        };

        let Some(item_index) = self.queue_item_index_by_id(item_id) else {
            return false;
        };
        let title = self.queue_items[item_index].title.clone();
        let workflow_id = self.alloc_workflow_run_id();
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::PostProcess,
            ToolKind::Ffmpeg,
        );

        if let Some(item) = self.queue_items.get_mut(item_index) {
            item.progress.post_process = 0.0;
            let mut run = WorkflowRun::new(
                workflow_id,
                WorkflowKind::PostProcess,
                ToolKind::Ffmpeg,
                WorkflowState::Running,
            );
            run.detail = input_path.to_owned();
            item.workflows.push(run);
        }

        self.last_action = i18n::format_fixed_english(
            "Converting with {profile}: {title}",
            &[("{title}", title.as_str()), ("{profile}", profile.label())],
        );
        self.push_runtime_log(format!(
            "Post-process started: {title} -> {}",
            profile.label()
        ));

        let tool_paths = self.tool_paths.clone();
        let settings = self.config.transcode_intent.clone();
        let tx = self.post_process_result_tx.clone();
        let input_path = input_path.to_owned();
        let temp_root = self.transcode_temp_root_path();
        let child_handle = Arc::new(Mutex::new(None));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.attach_active_download_process(
            workflow_id,
            child_handle.clone(),
            cancel_requested.clone(),
        );

        thread::spawn(move || {
            run_builtin_transcode_worker(
                tool_paths,
                settings,
                input_path,
                temp_root,
                item_id,
                workflow_id,
                tx,
                child_handle,
                cancel_requested,
            );
        });

        true
    }

    fn start_next_queued_download_after(&mut self, finished_item_id: QueueItemId) {
        if self.has_running_download_workflow() {
            return;
        }

        let Some(next_item_id) = self
            .queue_items
            .iter()
            .find(|item| {
                item.id != finished_item_id
                    && item_latest_download_state(item).is_some_and(is_pending_download_state)
            })
            .map(|item| item.id)
        else {
            return;
        };

        if self.queue_mode_downloads_as_audio() {
            if let Some(choice) = self.active_music_download_choice {
                let _ = self.start_music_download_task_at(next_item_id, choice);
                return;
            }
        }

        let emit_json = self
            .queue_item_by_id(next_item_id)
            .is_some_and(|item| !item.metadata_loaded());
        let _ = self.start_download_task_at(next_item_id, emit_json);
    }

    pub fn active_workflow_count(&self) -> usize {
        self.active_workflows.len()
    }

    pub fn item_has_running_workflow(&self, item_id: QueueItemId, kind: WorkflowKind) -> bool {
        self.active_workflows
            .values()
            .any(|workflow| workflow.item_id == item_id && workflow.kind == kind)
    }

    pub fn item_has_cancellable_download_workflow(&self, item_id: QueueItemId) -> bool {
        self.active_workflows.values().any(|workflow| {
            workflow.item_id == item_id
                && matches!(
                    workflow.kind,
                    WorkflowKind::DownloadMedia
                        | WorkflowKind::ExportMedia
                        | WorkflowKind::PostProcess
                )
                && workflow.download_child.is_some()
        })
    }

    pub fn cancel_item_download(&mut self, item_id: QueueItemId) {
        let workflows = self
            .active_workflows
            .values()
            .filter(|workflow| {
                workflow.item_id == item_id
                    && matches!(
                        workflow.kind,
                        WorkflowKind::DownloadMedia
                            | WorkflowKind::ExportMedia
                            | WorkflowKind::PostProcess
                    )
            })
            .map(|workflow| workflow.workflow_id)
            .collect::<Vec<_>>();

        if workflows.is_empty() {
            self.last_action = "There is no download to stop.".to_owned();
            return;
        }

        for workflow_id in workflows {
            self.request_active_download_stop(workflow_id);
        }
        self.last_action = "Stopping download...".to_owned();
    }

    fn request_active_download_stop(&self, workflow_id: WorkflowRunId) {
        let Some(workflow) = self.active_workflows.get(&workflow_id) else {
            return;
        };
        let (Some(child_handle), Some(cancel_requested)) = (
            workflow.download_child.as_ref(),
            workflow.cancel_requested.as_ref(),
        ) else {
            return;
        };
        if workflow.kind == WorkflowKind::PostProcess {
            request_post_process_stop(child_handle, cancel_requested);
        } else {
            request_download_stop(child_handle, cancel_requested);
        }
    }

    pub fn cleanup_active_download_processes(&mut self) {
        let workflows = self
            .active_workflows
            .values()
            .filter(|workflow| {
                matches!(
                    workflow.kind,
                    WorkflowKind::DownloadMedia
                        | WorkflowKind::ExportMedia
                        | WorkflowKind::PostProcess
                )
            })
            .map(|workflow| workflow.workflow_id)
            .collect::<Vec<_>>();
        for workflow_id in workflows {
            self.request_active_download_stop(workflow_id);
        }
    }

    pub fn item_is_busy(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        matches!(
            item.metadata_state,
            MetadataState::Queued | MetadataState::Running
        ) || item
            .workflows
            .iter()
            .any(|run| matches!(run.state, WorkflowState::Queued | WorkflowState::Running))
    }

    pub fn item_can_export(&self, item_index: usize, kind: DownloadTargetKind) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };
        if !item.metadata_loaded() || self.item_is_busy(item_index) {
            return false;
        }

        match kind {
            DownloadTargetKind::Video => !item.selection.video_selector.trim().is_empty(),
            DownloadTargetKind::Audio => {
                let (_, format_selector) = self.resolve_download_format_selection(
                    &item.selection.video_selector,
                    &item.selection.audio_selector,
                    item.metadata(),
                );
                !format_selector.trim().is_empty()
            }
            DownloadTargetKind::Subtitle => self
                .subtitle_track_by_id(&item.selection.subtitle_selector, item.metadata())
                .is_some(),
            DownloadTargetKind::Normal => false,
        }
    }

    fn queue_item_index_by_id(&self, item_id: QueueItemId) -> Option<usize> {
        self.queue_items.iter().position(|item| item.id == item_id)
    }

    fn queue_item_by_id(&self, item_id: QueueItemId) -> Option<&QueueItem> {
        self.queue_items.iter().find(|item| item.id == item_id)
    }

    fn queue_item_mut_by_id(&mut self, item_id: QueueItemId) -> Option<&mut QueueItem> {
        self.queue_items.iter_mut().find(|item| item.id == item_id)
    }

    fn should_use_cookies_for_item(&self, item_id: QueueItemId) -> bool {
        self.queue_item_by_id(item_id)
            .map(|item| item.selection.use_cookies)
            .unwrap_or(false)
    }

    fn mark_download_preflight_failed(&mut self, item_id: QueueItemId, error: &str) {
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            item.last_error = Some(error.to_owned());
            item.completed_selection = None;
            if let Some(run) = item.workflows.iter_mut().rev().find(|run| {
                run.kind == WorkflowKind::DownloadMedia
                    && matches!(run.state, WorkflowState::Queued | WorkflowState::Failed)
            }) {
                run.state = WorkflowState::Failed;
                run.progress = 0.0;
                run.error = Some(error.to_owned());
            }
        }
    }

    fn item_metadata(&self, item_index: usize) -> Option<&VideoMetadata> {
        self.queue_items
            .get(item_index)
            .and_then(QueueItem::metadata)
    }

    fn current_picker_metadata(&self) -> &VideoMetadata {
        self.format_picker
            .target_item_id
            .and_then(|index| self.item_metadata(index))
            .unwrap_or(&self.empty_item_preview)
    }

    pub fn item_thumbnail_url(&self, item_index: usize) -> &str {
        self.item_metadata(item_index)
            .map(|metadata| metadata.thumbnail_url.as_str())
            .or_else(|| {
                self.queue_items
                    .get(item_index)
                    .map(|item| item.thumbnail_url.as_str())
            })
            .unwrap_or_default()
    }

    pub fn item_thumbnail_hint(&self, item_index: usize) -> &str {
        self.item_metadata(item_index)
            .map(|metadata| metadata.thumbnail_hint.as_str())
            .or_else(|| {
                self.queue_items
                    .get(item_index)
                    .map(|item| item.thumbnail_hint.as_str())
            })
            .unwrap_or("item.thumbnail")
    }

    pub fn localized_thumbnail_hint<'a>(&self, hint: &'a str) -> std::borrow::Cow<'a, str> {
        match hint {
            "item.thumbnail" => {
                std::borrow::Cow::Borrowed(self.ui_i18n_text_for_key("item.thumbnail"))
            }
            "Thumbnail preview" => {
                std::borrow::Cow::Borrowed(self.ui_i18n_text_for_key("item.thumbnail_preview"))
            }
            _ => std::borrow::Cow::Borrowed(hint),
        }
    }

    pub fn item_duration_text(&self, item_index: usize) -> &str {
        self.item_metadata(item_index)
            .map(|metadata| metadata.duration_text.as_str())
            .or_else(|| {
                self.queue_items
                    .get(item_index)
                    .map(|item| item.duration_text.as_str())
            })
            .unwrap_or_default()
    }

    pub fn poll_thumbnail_work(&mut self, ctx: &eframe::egui::Context) {
        while let Ok(event) = self.thumbnail_result_rx.try_recv() {
            let entry = match event.result {
                Ok(image) => {
                    let texture = ctx.load_texture(
                        thumbnail_texture_id(&event.key),
                        image,
                        eframe::egui::TextureOptions::LINEAR,
                    );
                    ThumbnailCacheEntry::Ready(texture)
                }
                Err(error) => ThumbnailCacheEntry::Failed(error),
            };
            self.thumbnail_cache.insert(event.key, entry);
            ctx.request_repaint();
        }
    }

    pub fn has_loading_thumbnails(&self) -> bool {
        self.thumbnail_cache
            .values()
            .any(|entry| matches!(entry, ThumbnailCacheEntry::Loading))
    }

    pub fn thumbnail_render_source_for_url(
        &mut self,
        ctx: &eframe::egui::Context,
        url: &str,
    ) -> ThumbnailRenderSource {
        let url = url.trim();
        if url.is_empty() {
            return ThumbnailRenderSource::None;
        }

        let Some(proxy_url) = self.tool_paths.effective_proxy_url().map(str::to_owned) else {
            return ThumbnailRenderSource::DirectUrl;
        };

        self.thumbnail_render_source_with_proxy(ctx, url, proxy_url)
    }

    pub fn single_thumbnail_render_source_for_url(
        &mut self,
        ctx: &eframe::egui::Context,
        url: &str,
    ) -> ThumbnailRenderSource {
        let url = url.trim();
        if url.is_empty() {
            return ThumbnailRenderSource::None;
        }

        let proxy_url = self
            .tool_paths
            .effective_proxy_url()
            .map(str::to_owned)
            .unwrap_or_default();
        self.thumbnail_render_source_with_proxy(ctx, url, proxy_url)
    }

    fn thumbnail_render_source_with_proxy(
        &mut self,
        ctx: &eframe::egui::Context,
        url: &str,
        proxy_url: String,
    ) -> ThumbnailRenderSource {
        if !thumbnail_needs_memory_loader(url) {
            return ThumbnailRenderSource::DirectUrl;
        }

        self.poll_thumbnail_work(ctx);
        let no_check_certificates = self.tool_paths.no_check_certificates;
        let key = thumbnail_cache_key(url, &proxy_url, no_check_certificates);
        match self.thumbnail_cache.get(&key) {
            Some(ThumbnailCacheEntry::Ready(texture)) => {
                ThumbnailRenderSource::Texture(texture.clone())
            }
            Some(ThumbnailCacheEntry::Loading) => {
                ctx.request_repaint_after(Duration::from_millis(250));
                ThumbnailRenderSource::Loading
            }
            Some(ThumbnailCacheEntry::Failed(error)) => {
                ThumbnailRenderSource::Failed(error.clone())
            }
            None => {
                self.thumbnail_cache
                    .insert(key.clone(), ThumbnailCacheEntry::Loading);
                run_thumbnail_fetch_worker(
                    key,
                    url.to_owned(),
                    proxy_url,
                    no_check_certificates,
                    self.thumbnail_result_tx.clone(),
                );
                ctx.request_repaint_after(Duration::from_millis(250));
                ThumbnailRenderSource::Loading
            }
        }
    }

    pub fn has_active_work(&self) -> bool {
        self.is_adding_batch
            || !self.active_workflows.is_empty()
            || self.music_playback.is_some()
            || self.queue_items.iter().any(|item| {
                matches!(
                    item.compact_music_state,
                    Some(CompactMusicState::Resolving | CompactMusicState::Buffering)
                )
            })
            || self.component_update_snapshot.running
            || self.youtube_login_rescue_rx.is_some()
    }

    pub fn save_thumbnail_url_to_path(&mut self, url: &str, path: &Path) -> Result<(), String> {
        let url = url.trim();
        if url.is_empty() {
            return Err("Thumbnail load failed: empty URL".to_owned());
        }

        let proxy_url = self
            .tool_paths
            .effective_proxy_url()
            .map(str::to_owned)
            .unwrap_or_default();
        let bytes = fetch_thumbnail_bytes(url, &proxy_url, self.tool_paths.no_check_certificates)?;
        Self::save_thumbnail_bytes_as(&bytes, path)?;
        let display_path = path.display().to_string();
        self.last_action = i18n::format_fixed_english(
            "Thumbnail saved: {path}",
            &[("{path}", display_path.as_str())],
        );
        Ok(())
    }

    fn save_thumbnail_bytes_as(bytes: &[u8], path: &Path) -> Result<(), String> {
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            fs::write(path, bytes).map_err(|error| format!("Could not save thumbnail: {error}"))?;
            return Ok(());
        };

        match extension.trim().to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Self::save_thumbnail_as_jpeg(bytes, path),
            "png" => Self::save_thumbnail_as_image_format(bytes, path, image::ImageFormat::Png),
            "webp" => Self::save_thumbnail_as_image_format(bytes, path, image::ImageFormat::WebP),
            _ => {
                fs::write(path, bytes).map_err(|error| format!("Could not save thumbnail: {error}"))
            }
        }
    }

    fn save_thumbnail_as_jpeg(bytes: &[u8], path: &Path) -> Result<(), String> {
        let image = image::load_from_memory(bytes)
            .map_err(|error| format!("Could not decode thumbnail: {error}"))?;
        let rgb = image.to_rgb8();
        let mut file =
            fs::File::create(path).map_err(|error| format!("Could not save thumbnail: {error}"))?;
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, 92);
        encoder
            .encode_image(&rgb)
            .map_err(|error| format!("Could not encode thumbnail: {error}"))
    }

    fn save_thumbnail_as_image_format(
        bytes: &[u8],
        path: &Path,
        format: image::ImageFormat,
    ) -> Result<(), String> {
        let image = image::load_from_memory(bytes)
            .map_err(|error| format!("Could not decode thumbnail: {error}"))?;
        let mut file =
            fs::File::create(path).map_err(|error| format!("Could not save thumbnail: {error}"))?;
        image
            .write_to(&mut file, format)
            .map_err(|error| format!("Could not encode thumbnail: {error}"))
    }

    fn enqueue_item_analysis(&mut self, item_id: QueueItemId, source: String) {
        let workflow_id = self.alloc_workflow_run_id();
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::AnalyzeMetadata,
            ToolKind::YtDlp,
        );
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            item.metadata_state = MetadataState::Queued;
            item.last_error = None;
            let mut run = WorkflowRun::new(
                workflow_id,
                WorkflowKind::AnalyzeMetadata,
                ToolKind::YtDlp,
                WorkflowState::Queued,
            );
            run.detail = source.clone();
            item.workflows.push(run);
            item.metadata_state = MetadataState::Running;
            if let Some(run) = item
                .workflows
                .iter_mut()
                .rev()
                .find(|run| run.kind == WorkflowKind::AnalyzeMetadata)
            {
                run.state = WorkflowState::Running;
                run.detail = source.clone();
            }
        }

        self.last_action =
            i18n::format_fixed_english("Analyzing: {source}", &[("{source}", source.as_str())]);
        self.spawn_analyze_worker(
            source,
            Some(item_id),
            Some(workflow_id),
            self.should_use_cookies_for_item(item_id),
        );
    }

    fn spawn_analyze_worker(
        &mut self,
        source: String,
        target_item_id: Option<QueueItemId>,
        workflow_id: Option<WorkflowRunId>,
        use_cookies: bool,
    ) {
        if let Err(error) = self.ensure_yt_dlp_ready() {
            let _ = self.analyze_result_tx.send(AnalyzeResult {
                source,
                target_item_id,
                workflow_id,
                used_cookies: use_cookies,
                tool_log_action_id: None,
                command_line: None,
                result: Err(error),
            });
            return;
        }

        if let Err(error) = self.tool_paths.validate_cookie_setup(use_cookies) {
            let _ = self.analyze_result_tx.send(AnalyzeResult {
                source,
                target_item_id,
                workflow_id,
                used_cookies: use_cookies,
                tool_log_action_id: None,
                command_line: None,
                result: Err(error),
            });
            return;
        }

        let tool_log_action_id =
            Some(self.push_tool_log_action(self.app_mode.config_value(), "analyze"));

        let tool_paths = self.tool_paths.clone();
        let tx = self.analyze_result_tx.clone();
        let source_for_worker = source.clone();

        thread::spawn(move || {
            let (result, command_line) = analyze_output_parts(
                tool_paths.analyze_url_detailed(&source_for_worker, use_cookies),
            );
            let _ = tx.send(AnalyzeResult {
                source: source_for_worker,
                target_item_id,
                workflow_id,
                used_cookies: use_cookies,
                tool_log_action_id,
                command_line,
                result,
            });
        });
    }

    fn disable_missing_aria2_for_request(&self, request: &mut DownloadRequest) -> bool {
        if !request.use_aria2 || request.target_kind == DownloadTargetKind::Subtitle {
            return false;
        }

        if dependency_tool_exists(&self.tool_paths.aria2c) {
            return false;
        }

        request.use_aria2 = false;
        true
    }

    fn start_download_task_at(
        &mut self,
        item_id: QueueItemId,
        emit_json: bool,
    ) -> Result<(), String> {
        let Some(task_index) = self.queue_item_index_by_id(item_id) else {
            let error = "Target download item was not found.".to_owned();
            self.last_action = error.clone();
            return Err(error);
        };
        if self.has_running_download_workflow() {
            let error = "A download is already running. Please wait for it to finish.".to_owned();
            self.last_action = error.clone();
            return Err(error);
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.mark_download_preflight_failed(item_id, &error);
            self.last_action = error.clone();
            return Err(error);
        }

        let Some(item) = self.queue_items.get(task_index) else {
            let error = "Analyze the video before starting download.".to_owned();
            self.last_action = error.clone();
            return Err(error);
        };

        let title = item.title.clone();
        let source_url = item.source_url.clone();
        let (
            resolved_audio_selector,
            format_selector,
            resolved_audio_ext,
            subtitle_lang,
            subtitle_ext,
            subtitle_source_ext,
            subtitle_url,
            write_auto_subs,
            subtitle_is_auto_translated,
        ) = if item.metadata_loaded() {
            let (resolved_audio_selector, format_selector) = self
                .resolve_download_format_selection(
                    &item.selection.video_selector,
                    &item.selection.audio_selector,
                    item.metadata(),
                );
            let resolved_audio_ext =
                self.format_extension_by_id(&resolved_audio_selector, item.metadata());
            let subtitle_track = self
                .subtitle_track_by_id(&item.selection.subtitle_selector, item.metadata())
                .cloned();
            let subtitle_lang = subtitle_track
                .as_ref()
                .map(|track| track.download_language_code.clone());
            let subtitle_ext = subtitle_track
                .as_ref()
                .map(|track| track.ext.clone())
                .unwrap_or_default();
            let subtitle_source_ext = subtitle_ext.clone();
            let subtitle_url = subtitle_track.as_ref().map(|track| track.url.clone());
            let write_auto_subs = subtitle_track
                .as_ref()
                .is_some_and(|track| track.source == SubtitleSource::Automatic);
            let subtitle_is_auto_translated = subtitle_track.as_ref().is_some_and(|track| {
                track.source == SubtitleSource::Automatic && track.target_language_code.is_some()
            });
            (
                resolved_audio_selector,
                format_selector,
                resolved_audio_ext,
                subtitle_lang,
                subtitle_ext,
                subtitle_source_ext,
                subtitle_url,
                write_auto_subs,
                subtitle_is_auto_translated,
            )
        } else {
            (
                String::new(),
                String::new(),
                String::new(),
                None,
                String::new(),
                String::new(),
                None,
                false,
                false,
            )
        };

        let mut request = DownloadRequest {
            target_kind: DownloadTargetKind::Normal,
            url: source_url.clone(),
            format_selector,
            video_selector: item.selection.video_selector.clone(),
            audio_selector: resolved_audio_selector,
            is_muxed_video: item.metadata_loaded() && self.item_uses_muxed_video(task_index),
            video_ext: if item.metadata_loaded() {
                self.format_extension_by_id(&item.selection.video_selector, item.metadata())
            } else {
                String::new()
            },
            audio_ext: resolved_audio_ext,
            upload_date: item
                .metadata()
                .map(|metadata| metadata.upload_date_text.clone())
                .unwrap_or_default(),
            subtitle_lang,
            subtitle_ext,
            subtitle_source_ext,
            subtitle_url,
            write_auto_subs,
            subtitle_is_auto_translated,
            write_subtitles: item.selection.write_subtitles,
            embed_subtitles: item.selection.embed_subtitles,
            write_chapters: item.selection.write_chapters,
            embed_chapters: item.selection.embed_chapters,
            write_thumbnail: item.selection.write_thumbnail,
            embed_thumbnail: item.selection.embed_thumbnail,
            use_cookies: self.should_use_cookies_for_item(item_id),
            use_aria2: item.selection.use_aria2,
            emit_json,
            output_path: None,
            output_dir: item.selection.output_dir.clone(),
            file_name: if item.metadata_loaded() {
                item.selection.file_name.clone()
            } else {
                String::new()
            },
            download_sections: item.selection.download_sections.clone(),
        };

        let aria2_fallback = self.disable_missing_aria2_for_request(&mut request);

        if let Err(error) = self.tool_paths.validate_cookie_setup(request.use_cookies) {
            self.mark_download_preflight_failed(item_id, &error);
            self.last_action = error.clone();
            return Err(error);
        }

        let workflow_id = self.alloc_workflow_run_id();
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::DownloadMedia,
            ToolKind::YtDlp,
        );
        if let Some(item) = self.queue_items.get_mut(task_index) {
            reset_item_for_new_work(item, DownloadTargetKind::Normal);
            item.completed_selection = None;
            if let Some(run) = item.workflows.iter_mut().rev().find(|run| {
                run.kind == WorkflowKind::DownloadMedia
                    && matches!(run.state, WorkflowState::Queued | WorkflowState::Failed)
            }) {
                run.id = workflow_id;
                run.state = WorkflowState::Running;
                run.progress = 0.0;
                run.detail = source_url.clone();
                run.output_path = None;
                run.error = None;
            } else {
                let mut run = WorkflowRun::new(
                    workflow_id,
                    WorkflowKind::DownloadMedia,
                    ToolKind::YtDlp,
                    WorkflowState::Running,
                );
                run.detail = source_url.clone();
                item.workflows.push(run);
            }
        }
        self.last_action = if aria2_fallback {
            i18n::format_fixed_english(
                "Downloading: {title} (Aria2 not found; using yt-dlp native download)",
                &[("{title}", title.as_str())],
            )
        } else {
            i18n::format_fixed_english("Downloading: {title}", &[("{title}", title.as_str())])
        };

        let tool_paths = self.tool_paths.clone();
        let tx = self.download_result_tx.clone();
        let child_handle = Arc::new(Mutex::new(None));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.attach_active_download_process(
            workflow_id,
            child_handle.clone(),
            cancel_requested.clone(),
        );

        thread::spawn(move || {
            run_download_worker(
                tool_paths,
                request,
                item_id,
                workflow_id,
                WorkflowKind::DownloadMedia,
                tx,
                child_handle,
                cancel_requested,
            );
        });

        Ok(())
    }

    fn start_music_download_task_at(
        &mut self,
        item_id: QueueItemId,
        choice: MusicDownloadChoice,
    ) -> Result<(), String> {
        let Some(task_index) = self.queue_item_index_by_id(item_id) else {
            let error = "Target download item was not found.".to_owned();
            self.last_action = error.clone();
            return Err(error);
        };
        if self.has_running_download_workflow() {
            let error = "A download is already running. Please wait for it to finish.".to_owned();
            self.last_action = error.clone();
            return Err(error);
        }
        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.mark_download_preflight_failed(item_id, &error);
            self.last_action = error.clone();
            return Err(error);
        }

        self.prepare_queue_item_for_audio_mode(item_id);

        let Some(mut item) = self.queue_items.get(task_index).cloned() else {
            let error = "Analyze the video before starting download.".to_owned();
            self.last_action = error.clone();
            return Err(error);
        };

        if self.complete_music_cache_media_path(&item).is_none() {
            if let Some(hit) = self.complete_music_cache_hit_for_item(&item) {
                if let Some(target) = self.queue_items.get_mut(task_index) {
                    restore_music_compact_item_from_cache_hit(target, &hit);
                    item = target.clone();
                }
            }
        }

        let output_dir = resolve_output_dir(&item.selection.output_dir)
            .or_else(|_| resolve_output_dir(&self.item_defaults.output_dir))?;
        let cache_media_path = self.complete_music_cache_media_path(&item);
        let cover_path = self
            .music_cache_cover_path(&item)
            .filter(|path| path.is_file());
        let cover_cache_dir = self.music_cache_cover_write_dir(&item);
        let has_cover_source =
            choice.embed_cover && (cover_path.is_some() || !item.thumbnail_url.trim().is_empty());
        let source_kind = match cache_media_path.as_ref() {
            Some(path) if music_cache_can_be_copied_for_choice(choice, path, has_cover_source) => {
                MusicDownloadSourceKind::CacheCopy
            }
            Some(_) => MusicDownloadSourceKind::CacheConvert,
            None => MusicDownloadSourceKind::YtDlpDownload,
        };

        if source_kind == MusicDownloadSourceKind::CacheConvert {
            let ffmpeg = resolve_tool_path(&self.tool_paths.ffmpeg);
            if !ffmpeg.is_file() {
                let error = format!(
                    "ffmpeg.exe was not found: {}. Install FFmpeg from Options first.",
                    ffmpeg.display()
                );
                self.mark_download_preflight_failed(item_id, &error);
                self.last_action = error.clone();
                return Err(error);
            }
        }

        let workflow_id = self.alloc_workflow_run_id();
        let tool = music_download_tool_kind(source_kind);
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::DownloadMedia,
            tool.clone(),
        );
        if let Some(item) = self.queue_items.get_mut(task_index) {
            reset_item_for_new_work(item, DownloadTargetKind::Normal);
            item.completed_selection = None;
            item.selection.file_name = music_output_stem_template_for_title(&item.title);
            item.selection.audio_selector = choice.selection_token().to_owned();
            if let Some(run) = item.workflows.iter_mut().rev().find(|run| {
                run.kind == WorkflowKind::DownloadMedia
                    && matches!(run.state, WorkflowState::Queued | WorkflowState::Failed)
            }) {
                run.id = workflow_id;
                run.tool = tool;
                run.state = WorkflowState::Running;
                run.progress = 0.0;
                run.detail = item.source_url.clone();
                run.output_path = None;
                run.error = None;
            } else {
                let mut run = WorkflowRun::new(
                    workflow_id,
                    WorkflowKind::DownloadMedia,
                    tool,
                    WorkflowState::Running,
                );
                run.detail = item.source_url.clone();
                item.workflows.push(run);
            }
        }

        self.last_action = i18n::format_fixed_english(
            "Downloading music: {title}",
            &[("{title}", item.title.as_str())],
        );

        let job = MusicDownloadJob {
            item_id,
            workflow_id,
            source_url: item.source_url.clone(),
            title: item.title.clone(),
            album_title: item.music_album_title.clone(),
            output_dir,
            choice,
            source_acodec: item.music_stream_acodec.clone(),
            cache_media_path,
            cover_path,
            cover_cache_dir,
            thumbnail_url: item.thumbnail_url.clone(),
            use_cookies: self.should_use_cookies_for_item(item_id),
        };

        let tool_paths = self.tool_paths.clone();
        let tx = self.music_download_event_tx.clone();
        let child_handle = Arc::new(Mutex::new(None));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.attach_active_download_process(
            workflow_id,
            child_handle.clone(),
            cancel_requested.clone(),
        );

        thread::spawn(move || {
            run_music_download_worker(tool_paths, job, tx, child_handle, cancel_requested);
        });

        Ok(())
    }

    fn complete_music_cache_media_path(&self, item: &QueueItem) -> Option<PathBuf> {
        complete_music_cache_media_path_in_root(item, &self.music_stream_cache_root())
    }

    fn music_cache_cover_path(&self, item: &QueueItem) -> Option<PathBuf> {
        self.music_cache_cover_dirs(item)
            .into_iter()
            .find_map(|dir| first_music_cover_file_in_dir(&dir))
    }

    fn music_cache_cover_write_dir(&self, item: &QueueItem) -> Option<PathBuf> {
        let key = if item.music_cache_key.trim().is_empty() {
            music_cache_key(&item.source_url, "flat", "", "")
        } else {
            item.music_cache_key.clone()
        };
        (!key.trim().is_empty()).then(|| {
            self.music_stream_cache_root()
                .join("covers")
                .join(sanitize_music_cache_key(&key))
        })
    }

    fn music_cache_cover_dirs(&self, item: &QueueItem) -> Vec<PathBuf> {
        let cache_root = self.music_stream_cache_root();
        let mut dirs = Vec::new();
        if !item.music_cache_key.trim().is_empty() {
            let key = sanitize_music_cache_key(&item.music_cache_key);
            dirs.push(cache_root.join(&key));
            dirs.push(cache_root.join("covers").join(&key));
        }
        let flat_key = sanitize_music_cache_key(&music_cache_key(&item.source_url, "flat", "", ""));
        dirs.push(cache_root.join("covers").join(flat_key));
        dirs
    }

    pub fn item_export_initial_directory(&self, item_index: usize) -> Option<PathBuf> {
        let item = self.queue_items.get(item_index)?;
        resolve_output_dir(&item.selection.output_dir).ok()
    }

    pub fn item_export_default_name(
        &self,
        item_index: usize,
        kind: DownloadTargetKind,
    ) -> Option<String> {
        let item = self.queue_items.get(item_index)?;
        let base_name = if item.selection.file_name.trim().is_empty() {
            sanitize_file_name_for_windows(item.title.trim())
        } else {
            sanitize_file_name_for_windows(item.selection.file_name.trim())
        };
        let default_ext = self.item_export_default_extension(item_index, kind)?;
        Some(format!("{base_name}.{default_ext}"))
    }

    pub fn item_export_default_extension(
        &self,
        item_index: usize,
        kind: DownloadTargetKind,
    ) -> Option<String> {
        let item = self.queue_items.get(item_index)?;
        let metadata = item.metadata()?;
        match kind {
            DownloadTargetKind::Video => {
                let ext =
                    self.format_extension_by_id(&item.selection.video_selector, Some(metadata));
                normalized_export_extension(&ext).or_else(|| Some("mkv".to_owned()))
            }
            DownloadTargetKind::Audio => {
                let resolved_audio_selector = self
                    .resolve_download_format_selection(
                        &item.selection.video_selector,
                        &item.selection.audio_selector,
                        Some(metadata),
                    )
                    .0;
                let codec = self.format_codec_by_id(&resolved_audio_selector, Some(metadata));
                normalized_export_extension(&codec)
                    .or_else(|| {
                        let ext =
                            self.format_extension_by_id(&resolved_audio_selector, Some(metadata));
                        normalized_export_extension(&ext)
                    })
                    .or_else(|| Some("m4a".to_owned()))
            }
            DownloadTargetKind::Subtitle => Some("srt".to_owned()),
            DownloadTargetKind::Normal => None,
        }
    }

    pub fn start_item_export(
        &mut self,
        item_id: QueueItemId,
        kind: DownloadTargetKind,
        output_path: String,
    ) -> Result<(), String> {
        let Some(item_index) = self.queue_item_index_by_id(item_id) else {
            return Err("Target export item was not found.".to_owned());
        };
        if !self.item_can_export(item_index, kind) {
            return Err("This item cannot be exported right now.".to_owned());
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            if let Some(item) = self.queue_items.get_mut(item_index) {
                item.last_error = Some(error.clone());
            }
            self.last_action = error.clone();
            return Err(error);
        }

        let Some(item) = self.queue_items.get(item_index) else {
            return Err("Target export item was not found.".to_owned());
        };
        let Some(metadata) = item.metadata() else {
            return Err("Analyze the video before exporting.".to_owned());
        };
        let item_title = item.title.clone();
        let source_url = item.source_url.clone();
        let selected_video = item.selection.video_selector.clone();
        let selected_audio = item.selection.audio_selector.clone();
        let selected_subtitle_track = self
            .subtitle_track_by_id(&item.selection.subtitle_selector, Some(metadata))
            .cloned();
        let item_use_aria2 = item.selection.use_aria2;
        let item_write_thumbnail = item.selection.write_thumbnail;
        let item_embed_thumbnail = item.selection.embed_thumbnail;

        let (
            subtitle_lang,
            subtitle_ext,
            subtitle_source_ext,
            subtitle_url,
            write_auto_subs,
            subtitle_is_auto_translated,
        ) = if kind == DownloadTargetKind::Subtitle {
            let Some(track) = selected_subtitle_track.as_ref() else {
                return Err("Choose subtitles before exporting.".to_owned());
            };
            (
                Some(track.download_language_code.clone()),
                track.ext.clone(),
                track.ext.clone(),
                Some(track.url.clone()),
                track.source == SubtitleSource::Automatic,
                track.source == SubtitleSource::Automatic && track.target_language_code.is_some(),
            )
        } else {
            (None, String::new(), String::new(), None, false, false)
        };

        let target_path = normalize_export_target_path(
            &output_path,
            self.item_export_default_extension(item_index, kind)
                .as_deref(),
        );
        let export_ext = Path::new(&target_path)
            .extension()
            .and_then(|value| value.to_str())
            .and_then(normalized_export_extension)
            .ok_or_else(|| "Specify a file extension.".to_owned())?;
        validate_export_extension(kind, &export_ext)?;

        let (audio_selector, _) = self.resolve_download_format_selection(
            &selected_video,
            &selected_audio,
            Some(metadata),
        );
        let resolved_audio_ext = self.format_extension_by_id(&audio_selector, Some(metadata));
        let mut request = DownloadRequest {
            target_kind: kind,
            url: source_url.clone(),
            format_selector: match kind {
                DownloadTargetKind::Video => selected_video.clone(),
                DownloadTargetKind::Audio => audio_selector.clone(),
                DownloadTargetKind::Normal | DownloadTargetKind::Subtitle => String::new(),
            },
            video_selector: if kind == DownloadTargetKind::Subtitle {
                String::new()
            } else {
                selected_video.clone()
            },
            audio_selector: if kind == DownloadTargetKind::Subtitle {
                String::new()
            } else {
                audio_selector
            },
            is_muxed_video: false,
            video_ext: if kind == DownloadTargetKind::Subtitle {
                String::new()
            } else {
                self.format_extension_by_id(&selected_video, Some(metadata))
            },
            audio_ext: if kind == DownloadTargetKind::Subtitle {
                String::new()
            } else {
                resolved_audio_ext
            },
            upload_date: metadata.upload_date_text.clone(),
            subtitle_lang,
            subtitle_ext: if kind == DownloadTargetKind::Subtitle {
                export_ext.clone()
            } else {
                subtitle_ext
            },
            subtitle_source_ext,
            subtitle_url,
            write_auto_subs,
            subtitle_is_auto_translated,
            write_subtitles: false,
            embed_subtitles: false,
            write_chapters: false,
            embed_chapters: false,
            write_thumbnail: matches!(kind, DownloadTargetKind::Video) && item_write_thumbnail,
            embed_thumbnail: matches!(kind, DownloadTargetKind::Video) && item_embed_thumbnail,
            use_cookies: self.should_use_cookies_for_item(item_id),
            use_aria2: kind != DownloadTargetKind::Subtitle && item_use_aria2,
            emit_json: false,
            output_path: Some(target_path.clone()),
            output_dir: String::new(),
            file_name: String::new(),
            download_sections: item.selection.download_sections.clone(),
        };

        let aria2_fallback = self.disable_missing_aria2_for_request(&mut request);

        if let Err(error) = self.tool_paths.validate_cookie_setup(request.use_cookies) {
            if let Some(item) = self.queue_items.get_mut(item_index) {
                item.last_error = Some(error.clone());
            }
            return Err(error);
        }

        let workflow_id = self.alloc_workflow_run_id();
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::ExportMedia,
            ToolKind::YtDlp,
        );
        if let Some(item) = self.queue_items.get_mut(item_index) {
            reset_item_for_new_work(item, kind);
            let mut run = WorkflowRun::new(
                workflow_id,
                WorkflowKind::ExportMedia,
                ToolKind::YtDlp,
                WorkflowState::Running,
            );
            run.detail = target_path.clone();
            item.workflows.push(run);
            item.last_error = None;
        }

        let action_text = match kind {
            DownloadTargetKind::Video => i18n::format_fixed_english(
                "Exporting video: {title}",
                &[("{title}", item_title.as_str())],
            ),
            DownloadTargetKind::Audio => i18n::format_fixed_english(
                "Exporting audio: {title}",
                &[("{title}", item_title.as_str())],
            ),
            DownloadTargetKind::Normal => i18n::format_fixed_english(
                "Downloading: {title}",
                &[("{title}", item_title.as_str())],
            ),
            DownloadTargetKind::Subtitle => i18n::format_fixed_english(
                "Exporting subtitles: {title}",
                &[("{title}", item_title.as_str())],
            ),
        };
        self.last_action = if aria2_fallback {
            i18n::format_fixed_english(
                "{action} (Aria2 not found; using yt-dlp native download)",
                &[("{action}", action_text.as_str())],
            )
        } else {
            action_text
        };

        let tool_paths = self.tool_paths.clone();
        let tx = self.download_result_tx.clone();
        let child_handle = Arc::new(Mutex::new(None));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.attach_active_download_process(
            workflow_id,
            child_handle.clone(),
            cancel_requested.clone(),
        );
        thread::spawn(move || {
            run_download_worker(
                tool_paths,
                request,
                item_id,
                workflow_id,
                WorkflowKind::ExportMedia,
                tx,
                child_handle,
                cancel_requested,
            );
        });
        Ok(())
    }

    pub fn clear_queue(&mut self) {
        self.stop_music_playback();
        self.queue_items.clear();
        self.music_history_back.clear();
        self.music_history_forward.clear();
        self.music_reserved_next_item_id = None;
        self.last_action = "Queue cleared.".to_owned();
        self.save_active_audio_playlist_if_needed();
    }

    pub fn remove_queue_item(&mut self, item_id: QueueItemId) {
        let Some(index) = self.queue_item_index_by_id(item_id) else {
            return;
        };

        if self.item_is_busy(index) {
            self.last_action = "Running items cannot be removed.".to_owned();
            return;
        }

        if self.music_player_current_item_id == Some(item_id) {
            self.stop_music_playback();
        }

        let removed = self.queue_items.remove(index);
        let removed_source_key = canonical_queue_source_key(&removed.source_url);
        self.batch_input = self
            .batch_input
            .lines()
            .map(str::trim)
            .filter(|line| {
                !line.is_empty() && canonical_queue_source_key(line) != removed_source_key
            })
            .collect::<Vec<_>>()
            .join("\n");
        self.last_action =
            i18n::format_fixed_english("Removed: {title}", &[("{title}", removed.title.as_str())]);
        self.prune_music_navigation_state();
        self.save_active_audio_playlist_if_needed();
    }

    pub fn primary_candidate_url(&self) -> Option<String> {
        let direct = self.url_input.trim();
        if !direct.is_empty() {
            return Some(direct.to_owned());
        }

        self.batch_input
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(ToOwned::to_owned)
    }

    fn parsed_batch_urls(&self) -> Vec<String> {
        self.batch_input
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    fn append_batch_seed(&mut self, _source: &str, seed: PlaylistEntrySeed) {
        let source_key = canonical_queue_source_key(&seed.source_url);
        if self
            .queue_items
            .iter()
            .any(|item| canonical_queue_source_key(&item.source_url) == source_key)
        {
            return;
        }

        let source_url = seed.source_url.clone();
        let item = self.build_queue_item_from_seed(seed);
        self.queue_items.push(item);
        self.mark_font_content_changed();

        if !self
            .batch_input
            .lines()
            .map(str::trim)
            .any(|line| canonical_queue_source_key(line) == source_key)
        {
            self.batch_input_push_unique(&source_url);
        }
    }

    fn build_queue_item_from_url(&mut self, url: &str) -> QueueItem {
        let title = infer_title(url, "Untitled task", "Imported {tail}");
        let mut item = QueueItem::new(self.alloc_queue_item_id(), url, title);
        item.selection.quality = self.item_defaults.quality;
        item.selection.write_thumbnail = self.item_defaults.write_thumbnail;
        item.selection.embed_thumbnail = self.item_defaults.embed_thumbnail;
        item.selection.write_subtitles = self.item_defaults.write_subtitles;
        item.selection.embed_subtitles = self.item_defaults.embed_subtitles;
        item.selection.write_chapters = self.item_defaults.write_chapters;
        item.selection.embed_chapters = self.item_defaults.embed_chapters;
        item.selection.use_cookies = self.item_defaults.use_cookies;
        item.selection.use_aria2 = self.item_defaults.use_aria2;
        item.selection.output_dir = self.item_defaults.output_dir.clone();
        item.selection.download_sections.clear();
        item
    }

    fn build_queue_item_from_seed(&mut self, seed: PlaylistEntrySeed) -> QueueItem {
        let mut item = self.build_queue_item_from_url(&seed.source_url);
        if !seed.title.trim().is_empty() {
            item.title = seed.title;
        }
        item.music_album_title = seed.album_title;
        item.thumbnail_hint = seed.thumbnail_hint;
        item.thumbnail_url = seed.thumbnail_url;
        item.duration_text = seed.duration_text;
        item.metadata_state = MetadataState::Idle;
        if item.selection.file_name.trim().is_empty() {
            item.selection.file_name = sanitize_file_name_for_windows(item.title.trim());
        }
        item
    }

    fn apply_music_stream_result(&mut self, message: MusicStreamResolveEvent) {
        match message {
            MusicStreamResolveEvent::ToolCommandFinished {
                action_id,
                tool,
                action,
                command_line,
                success,
            } => {
                let status = if success {
                    ToolLogStatus::Success
                } else if action == "prefetch resolve" {
                    ToolLogStatus::Skipped
                } else {
                    ToolLogStatus::Failed
                };
                self.push_tool_log_step(action_id, status, tool, action, command_line);
            }
            MusicStreamResolveEvent::FlatImport { source, result } => match result {
                Ok(seeds) => {
                    let mut added = 0usize;
                    for seed in seeds {
                        if self.append_music_compact_seed(seed) {
                            added += 1;
                        }
                    }
                    self.music_player_error = None;
                    self.last_action = if added == 0 {
                        i18n::format_fixed_english(
                            "No music items could be added: {source}",
                            &[("{source}", source.as_str())],
                        )
                    } else {
                        i18n::format_fixed_english(
                            "Added {count} music items.",
                            &[("{count}", &added.to_string())],
                        )
                    };
                    self.save_active_audio_playlist_if_needed();
                }
                Err(error) => {
                    let message = i18n::format_fixed_english(
                        "Music list analysis failed: {error}",
                        &[("{error}", error.as_str())],
                    );
                    self.music_player_error = Some(message.clone());
                    self.push_runtime_log(message.clone());
                    self.last_action = message;
                    eprintln!("[music-stream] flat import failed: {error}");
                }
            },
            MusicStreamResolveEvent::FlatUpdate {
                item_id,
                source,
                result,
            } => {
                match result {
                    Ok(seed) => {
                        self.update_music_compact_item_from_seed(item_id, seed);
                        self.music_player_error = None;
                        self.save_active_audio_playlist_if_needed();
                    }
                    Err(error) => {
                        // Flat metadata is an enhancement for single-item compact rows.
                        // Keep the row playable by URL even when the fast metadata probe fails.
                        eprintln!(
                            "[music-stream] flat update skipped for item={item_id} source={source}: {error}"
                        );
                    }
                }
            }
            MusicStreamResolveEvent::Resolve {
                item_id,
                session_id,
                source,
                play_after_resolve,
                result,
            } => match result {
                Ok(seed) => {
                    let lyrics_track = seed.lyrics_track.clone();
                    let mut updated_item = None;
                    if let Some(item) = self.queue_item_mut_by_id(item_id) {
                        item.source_url = seed.source_url;
                        item.title = seed.title;
                        if !seed.album_title.trim().is_empty() {
                            item.music_album_title = seed.album_title;
                        }
                        if !seed.thumbnail_url.trim().is_empty() {
                            item.thumbnail_url = seed.thumbnail_url;
                        }
                        if !seed.thumbnail_hint.trim().is_empty() {
                            item.thumbnail_hint = seed.thumbnail_hint;
                        }
                        if !seed.duration_text.trim().is_empty() {
                            item.duration_text = seed.duration_text;
                        }
                        item.music_duration_seconds = seed.duration_seconds;
                        item.music_stream_url = seed.direct_url;
                        item.music_stream_headers = seed.headers;
                        item.music_stream_ext = seed.ext;
                        item.music_stream_format_id = seed.format_id;
                        item.music_stream_acodec = seed.acodec;
                        item.music_stream_expected_bytes = seed.expected_bytes;
                        item.music_cache_key = seed.cache_key;
                        item.metadata_state = MetadataState::Idle;
                        item.compact_music_state = Some(CompactMusicState::Ready);
                        item.last_error = None;
                        if item.selection.file_name.trim().is_empty() {
                            item.selection.file_name =
                                sanitize_file_name_for_windows(item.title.trim());
                        }
                        updated_item = Some(item.clone());
                    }
                    if let Some(item) = updated_item.as_ref() {
                        self.mark_font_content_changed();
                        self.cache_music_lyrics_for_item(item, lyrics_track.as_ref());
                    }
                    if !play_after_resolve {
                        let is_current_prefetch_resolve = self.music_prefetch_pending_item_id
                            == Some(item_id)
                            && self.music_prefetch_session_id == session_id;
                        if is_current_prefetch_resolve {
                            self.music_prefetch_pending_item_id = None;
                            if let Some(item) = updated_item {
                                self.start_resolved_music_prefetch(item);
                            }
                            self.save_active_audio_playlist_if_needed();
                        }
                        return;
                    }
                    self.music_player_error = None;
                    self.last_action = i18n::format_fixed_english(
                        "Music stream ready: {source}",
                        &[("{source}", source.as_str())],
                    );
                    self.save_active_audio_playlist_if_needed();
                    if play_after_resolve && self.is_current_music_session(item_id, session_id) {
                        self.start_music_stream_playback_with_session(item_id, session_id);
                    }
                }
                Err(error) => {
                    if !play_after_resolve {
                        let is_current_prefetch_resolve = self.music_prefetch_pending_item_id
                            == Some(item_id)
                            && self.music_prefetch_session_id == session_id;
                        if is_current_prefetch_resolve {
                            self.music_prefetch_pending_item_id = None;
                            eprintln!(
                                "[music-prefetch] resolve skipped for item={item_id}: {error}"
                            );
                        }
                        return;
                    }
                    if play_after_resolve && !self.is_current_music_session(item_id, session_id) {
                        if let Some(item) = self.queue_item_mut_by_id(item_id) {
                            item.compact_music_state = Some(CompactMusicState::Ready);
                            item.last_error = None;
                        }
                        eprintln!(
                            "[music-stream] ignored stale resolve failure for item={item_id}: {error}"
                        );
                        return;
                    }
                    if let Some(item) = self.queue_item_mut_by_id(item_id) {
                        item.metadata_state = MetadataState::Failed(error.clone());
                        item.compact_music_state = Some(CompactMusicState::Failed);
                        item.last_error = Some(error.clone());
                    }
                    let message = i18n::format_fixed_english(
                        "Music stream analysis failed: {error}",
                        &[("{error}", error.as_str())],
                    );
                    self.music_player_error = Some(message.clone());
                    self.push_runtime_log(message.clone());
                    eprintln!("[music-stream] resolve failed: {error}");
                    self.last_action = message;
                }
            },
        }
    }

    fn apply_music_download_event(&mut self, event: MusicDownloadEvent) {
        match event {
            MusicDownloadEvent::Progress {
                item_id,
                workflow_id,
                percent,
            } => {
                if let Some(item) = self.queue_item_mut_by_id(item_id) {
                    let display_percent = percent.clamp(0.0, 100.0);
                    item.progress.audio = monotonic_progress(item.progress.audio, display_percent);
                    if let Some(run) = item.workflows.iter_mut().find(|run| run.id == workflow_id) {
                        run.progress = monotonic_progress(run.progress, display_percent);
                    }
                }
            }
            MusicDownloadEvent::ToolCommandFinished {
                item_id: _,
                workflow_id,
                source_kind,
                tool,
                action,
                command_line,
                success,
            } => {
                let action_id = self.workflow_tool_log_action(
                    workflow_id,
                    "audio",
                    music_source_kind_log_action(source_kind),
                );
                self.push_tool_log_step(
                    action_id,
                    self.tool_log_status_for_workflow_step(workflow_id, success),
                    tool,
                    action,
                    command_line,
                );
            }
            MusicDownloadEvent::Finished {
                item_id,
                workflow_id,
                source_kind,
                result,
            } => {
                self.unregister_active_workflow(workflow_id);
                self.finish_workflow_tool_log(workflow_id);
                if let Some(item) = self.queue_item_mut_by_id(item_id) {
                    if let Some(run) = item.workflows.iter_mut().find(|run| run.id == workflow_id) {
                        match &result {
                            Ok(output_path) => {
                                run.state = WorkflowState::Finished;
                                run.progress = 100.0;
                                run.output_path = Some(output_path.clone());
                                item.progress.audio = 100.0;
                                item.progress.video = 100.0;
                                item.last_output_path = Some(output_path.clone());
                                if let Some(actual_file_name) = Path::new(output_path)
                                    .file_name()
                                    .and_then(|value| value.to_str())
                                    .map(ToOwned::to_owned)
                                {
                                    item.selection.file_name = actual_file_name;
                                }
                                item.completed_selection =
                                    Some(CompletedSelection::from_selection(&item.selection));
                                item.last_error = None;
                            }
                            Err(error) if error == DOWNLOAD_CANCELLED_MESSAGE => {
                                run.state = WorkflowState::Cancelled;
                                run.error = None;
                                item.last_error = None;
                            }
                            Err(error) => {
                                run.state = WorkflowState::Failed;
                                run.error = Some(error.clone());
                                item.last_error = Some(error.clone());
                                item.completed_selection = None;
                            }
                        }
                    }
                }

                match result {
                    Ok(output_path) => {
                        let source_label = match source_kind {
                            MusicDownloadSourceKind::CacheCopy => "cache copy",
                            MusicDownloadSourceKind::CacheConvert => "cache convert",
                            MusicDownloadSourceKind::YtDlpOnlineTarget => "yt-dlp online target",
                            MusicDownloadSourceKind::YtDlpDownload => "yt-dlp",
                        };
                        self.push_runtime_log(format!(
                            "Music download finished ({source_label}): {output_path}"
                        ));
                        self.last_action.clear();
                    }
                    Err(error) if error == DOWNLOAD_CANCELLED_MESSAGE => {
                        self.push_runtime_log("Music download cancelled".to_owned());
                        self.last_action = "Download stopped.".to_owned();
                    }
                    Err(error) => {
                        self.push_runtime_log(format!("Music download failed: {error}"));
                        eprintln!("[music-download] {error}");
                        self.last_action = error;
                    }
                }
                self.start_next_queued_download_after(item_id);
            }
        }
    }

    fn apply_music_playback_event(&mut self, event: MusicPlaybackEvent) {
        match event {
            MusicPlaybackEvent::ToolCommandFinished {
                item_id,
                session_id,
                tool,
                action,
                command_line,
                success,
            } => {
                if !self.is_current_music_session(item_id, session_id) {
                    return;
                }
                let action_id = self.push_tool_log_action("audio", "playback cache");
                self.push_tool_log_step(
                    action_id,
                    if success {
                        ToolLogStatus::Success
                    } else {
                        ToolLogStatus::Failed
                    },
                    tool,
                    action,
                    command_line,
                );
            }
            MusicPlaybackEvent::Started {
                item_id,
                session_id,
            } => {
                if !self.is_current_music_session(item_id, session_id) {
                    eprintln!(
                        "[music-stream] ignored stale playback start for item={item_id} session={session_id}"
                    );
                    return;
                }
                self.music_player_error = None;
                self.music_player_current_item_id = Some(item_id);
                self.mark_music_playback_state(item_id, CompactMusicState::Playing);
            }
            MusicPlaybackEvent::Finished {
                item_id,
                session_id,
            } => {
                if !self.is_current_music_session(item_id, session_id) {
                    eprintln!(
                        "[music-stream] ignored stale playback finish for item={item_id} session={session_id}"
                    );
                    return;
                }
                self.mark_music_playback_state(item_id, CompactMusicState::Ready);
                self.music_playback = None;
                self.music_player_error = None;
                self.last_action = "Playback finished.".to_owned();
                self.advance_music_after_finished(item_id);
            }
            MusicPlaybackEvent::Stopped {
                item_id,
                session_id,
            } => {
                if !self.is_current_music_session(item_id, session_id) {
                    if self.queue_item_by_id(item_id).is_some_and(|item| {
                        item.compact_music_state != Some(CompactMusicState::Failed)
                    }) {
                        self.mark_music_playback_state(item_id, CompactMusicState::Ready);
                    }
                    eprintln!(
                        "[music-stream] ignored stale playback stop for item={item_id} session={session_id}"
                    );
                    return;
                }
                self.music_playback = None;
                if self
                    .queue_item_by_id(item_id)
                    .is_some_and(|item| item.compact_music_state != Some(CompactMusicState::Failed))
                {
                    self.mark_music_playback_state(item_id, CompactMusicState::Ready);
                }
            }
            MusicPlaybackEvent::Failed {
                item_id,
                session_id,
                error,
            } => {
                if !self.is_current_music_session(item_id, session_id) {
                    // Stale event from a playback session that was replaced by user action.
                    // Do not poison the row/cache health for a deliberate track switch.
                    if self.queue_item_by_id(item_id).is_some_and(|item| {
                        item.compact_music_state != Some(CompactMusicState::Failed)
                    }) {
                        self.mark_music_playback_state(item_id, CompactMusicState::Ready);
                    }
                    eprintln!(
                        "[music-stream] ignored stale playback failure for item={item_id} session={session_id}: {error}"
                    );
                    return;
                }
                self.mark_music_playback_state(item_id, CompactMusicState::Failed);
                self.music_playback = None;
                self.music_player_current_item_id = None;
                if let Some(item) = self.queue_item_mut_by_id(item_id) {
                    item.last_error = Some(error.clone());
                }
                let message = i18n::format_fixed_english(
                    "Playback failed: {error}",
                    &[("{error}", error.as_str())],
                );
                self.music_player_error = Some(message.clone());
                self.push_runtime_log(message.clone());
                eprintln!("[music-stream] playback failed: {error}");
                self.last_action = message;
            }
            MusicPlaybackEvent::PrefetchToolCommandFinished {
                item_id,
                session_id,
                tool,
                action,
                command_line,
                success,
            } => {
                if !self.prefetch_event_is_current(item_id, session_id) {
                    return;
                }
                let action_id = self.push_tool_log_action("audio", "prefetch cache");
                let status = if success {
                    ToolLogStatus::Success
                } else {
                    ToolLogStatus::Skipped
                };
                self.push_tool_log_step(action_id, status, tool, action, command_line);
            }
            MusicPlaybackEvent::PrefetchFinished {
                item_id,
                session_id,
                success,
                error,
            } => {
                if !self.prefetch_event_is_current(item_id, session_id) {
                    return;
                }
                if success {
                    if let Some(started) = self.music_prefetch_started_at {
                        let elapsed = Instant::now().duration_since(started).as_secs_f64();
                        if elapsed.is_finite() && elapsed > 0.0 {
                            self.music_prefetch_lead_seconds =
                                (elapsed * MUSIC_PREFETCH_SPEED_MULTIPLIER).clamp(
                                    MUSIC_PREFETCH_MIN_LEAD_SECONDS,
                                    MUSIC_PREFETCH_MAX_LEAD_SECONDS,
                                );
                        }
                    }
                } else if let Some(error) = error {
                    eprintln!("[music-prefetch] cache skipped for item={item_id}: {error}");
                }
                if self.music_prefetch_active_item_id == Some(item_id) {
                    self.music_prefetch_control = None;
                }
                self.music_prefetch_active_item_id = None;
                self.music_prefetch_started_at = None;
            }
        }
    }

    fn is_current_music_session(&self, item_id: QueueItemId, session_id: u64) -> bool {
        self.music_player_current_item_id == Some(item_id)
            && self.music_playback_session_id == session_id
    }

    pub fn add_music_compact_from_current_url(&mut self) {
        self.add_current_url_to_music_compact_batch();
    }

    fn add_current_url_to_music_compact_batch(&mut self) {
        if self.is_adding_batch {
            self.last_action = "Batch add is still running.".to_owned();
            return;
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        let source = self.url_input.trim();
        if source.is_empty() {
            self.last_action = "There is no URL to add.".to_owned();
            return;
        }

        let source = source.to_owned();
        if youtube_url_has_video_and_playlist(&source) {
            match self.config.youtube_video_playlist_mode {
                YoutubeVideoPlaylistMode::Ask => {
                    let risk = if self.config.youtube_high_risk_playlist_prompt {
                        classify_youtube_playlist(&source)
                    } else {
                        None
                    };
                    self.youtube_playlist_prompt = Some(YoutubePlaylistPrompt {
                        source,
                        kind: YoutubePlaylistPromptKind::VideoAndPlaylist,
                        risk,
                        music_compact: true,
                    });
                    self.last_action =
                        "Detected a video URL that also contains a playlist.".to_owned();
                    return;
                }
                YoutubeVideoPlaylistMode::Video => {
                    let single_source =
                        youtube_url_force_single_video(&source).unwrap_or_else(|| source.clone());
                    self.add_single_music_compact_url(single_source);
                    return;
                }
                YoutubeVideoPlaylistMode::Ignore => {}
            }
        }

        self.music_player_error = None;
        if !looks_like_playlist_url(&source) {
            // Align with Add: single items enter the list immediately.
            // Music compact then uses a fast flat metadata update because it does not expose format choices.
            self.add_single_music_compact_url(source);
            return;
        }

        if self.config.youtube_high_risk_playlist_prompt {
            if let Some(risk) = classify_youtube_playlist(&source) {
                self.youtube_playlist_prompt = Some(YoutubePlaylistPrompt {
                    source,
                    kind: YoutubePlaylistPromptKind::HighRiskPlaylist,
                    risk: Some(risk),
                    music_compact: true,
                });
                self.last_action = i18n::format_fixed_english(
                    "Detected high-risk YouTube playlist: {kind}",
                    &[("{kind}", risk.kind.label())],
                );
                return;
            }
        }

        // Align with Add: playlist import locks the URL row and streams items in one-by-one.
        // The only difference is the inserted item view kind.
        self.begin_music_batch_add(source);
    }

    fn add_single_music_compact_url(&mut self, source: String) {
        let source_key = canonical_queue_source_key(&source);
        if let Some(existing_id) = self
            .queue_items
            .iter()
            .find(|item| canonical_queue_source_key(&item.source_url) == source_key)
            .map(|item| item.id)
        {
            let cache_hit = self
                .queue_item_by_id(existing_id)
                .and_then(|item| self.complete_music_cache_hit_for_item(item));
            if let Some(hit) = cache_hit {
                if let Some(item) = self.queue_item_mut_by_id(existing_id) {
                    restore_music_compact_item_from_cache_hit(item, &hit);
                }
                self.last_action =
                    "Music item is already in the list; local cache was used.".to_owned();
                self.save_active_audio_playlist_if_needed();
            } else {
                self.last_action = "Music item is already in the list.".to_owned();
                let use_cookies = self
                    .queue_item_by_id(existing_id)
                    .map(|item| item.selection.use_cookies)
                    .unwrap_or(self.item_defaults.use_cookies);
                self.spawn_music_flat_update_worker(existing_id, source.clone(), use_cookies);
            }
            self.url_input.clear();
            return;
        }

        let mut item = self.build_queue_item_from_url(&source);
        item.view_kind = QueueItemViewKind::MusicCompact;
        item.compact_music_state = Some(CompactMusicState::Ready);
        item.metadata_state = MetadataState::Idle;
        item.duration_text.clear();
        item.music_duration_seconds = None;
        item.music_stream_url.clear();
        item.music_stream_headers.clear();
        item.music_stream_ext.clear();
        item.music_stream_format_id.clear();
        item.music_stream_acodec.clear();
        item.music_stream_expected_bytes = None;
        item.music_cache_key = music_cache_key(&item.source_url, "flat", "", "");
        item.last_error = None;
        let cache_hit = self.complete_music_cache_hit_for_item(&item);
        if let Some(hit) = cache_hit.as_ref() {
            restore_music_compact_item_from_cache_hit(&mut item, hit);
        }
        let item_id = item.id;
        let title = item.title.clone();
        self.queue_items.push(item);
        self.batch_input_push_unique(&source);
        self.url_input.clear();
        self.last_action =
            i18n::format_fixed_english("Added to batch: {title}", &[("{title}", title.as_str())]);
        self.save_active_audio_playlist_if_needed();
        if cache_hit.is_none() {
            let use_cookies = self
                .queue_item_by_id(item_id)
                .map(|item| item.selection.use_cookies)
                .unwrap_or(self.item_defaults.use_cookies);
            self.spawn_music_flat_update_worker(item_id, source, use_cookies);
        } else {
            self.last_action = i18n::format_fixed_english(
                "Added music from local cache: {title}",
                &[("{title}", title.as_str())],
            );
        }
    }

    fn spawn_music_flat_update_worker(
        &mut self,
        item_id: QueueItemId,
        source: String,
        use_cookies: bool,
    ) {
        let tx = self.music_stream_result_tx.clone();
        let tool_paths = self.tool_paths.clone();
        let untitled_task = "Untitled task".to_owned();
        let imported_template = "Imported {tail}".to_owned();
        let tool_log_action_id = self.push_tool_log_action("audio", "flat update");
        thread::spawn(move || {
            let mut command_line = String::new();
            let result = (|| -> Result<PlaylistEntrySeed, String> {
                let mut command =
                    tool_paths.prepare_music_flat_update_command(&source, use_cookies)?;
                command_line = format_process_command_line(&command);
                command
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .stdin(Stdio::null());
                let mut child = command.spawn().map_err(|error| {
                    format!("Could not start yt-dlp music flat import: {error}")
                })?;
                let stdout = match child.stdout.take() {
                    Some(stdout) => stdout,
                    None => {
                        terminate_child_process(&mut child);
                        let _ = child.wait();
                        return Err("Could not read yt-dlp music flat output.".to_owned());
                    }
                };
                let stderr_handle = child.stderr.take().map(|mut stderr| {
                    thread::spawn(move || {
                        let mut stderr_text = String::new();
                        let _ = stderr.read_to_string(&mut stderr_text);
                        stderr_text
                    })
                });
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                let mut first_seed = None;
                loop {
                    line.clear();
                    let read = match reader.read_line(&mut line) {
                        Ok(read) => read,
                        Err(error) => {
                            terminate_child_process(&mut child);
                            let _ = child.wait();
                            return Err(format!(
                                "Could not read yt-dlp music flat output: {error}"
                            ));
                        }
                    };
                    if read == 0 {
                        break;
                    }
                    let raw = line.trim();
                    if raw.is_empty() {
                        continue;
                    }
                    let Ok(entry) = serde_json::from_str::<Value>(raw) else {
                        continue;
                    };
                    if let Some(mut seed) =
                        playlist_entry_seed_from_json(&entry, &untitled_task, &imported_template)
                    {
                        if let Some(thumbnail_url) = select_largest_thumbnail_url(&entry) {
                            seed.thumbnail_url = thumbnail_url;
                            seed.thumbnail_hint = "Thumbnail preview".to_owned();
                        }
                        first_seed = Some(seed);
                        terminate_child_process(&mut child);
                        break;
                    }
                }
                let status = child.wait().map_err(|error| {
                    format!("Could not wait for yt-dlp music flat import: {error}")
                })?;
                let stderr_text = stderr_handle
                    .and_then(|handle| handle.join().ok())
                    .unwrap_or_default();
                if let Some(seed) = first_seed {
                    return Ok(seed);
                }
                let detail = stderr_text.trim();
                if detail.is_empty() {
                    if status.success() {
                        Err("yt-dlp did not return a music entry.".to_owned())
                    } else {
                        Err(format!(
                            "yt-dlp music flat import failed: exit code {:?}",
                            status.code()
                        ))
                    }
                } else {
                    Err(format!("yt-dlp music flat import failed: {detail}"))
                }
            })();
            if command_line.trim().is_empty() && result.is_err() {
                command_line = "yt-dlp".to_owned();
            }
            if !command_line.trim().is_empty() {
                let _ = tx.send(MusicStreamResolveEvent::ToolCommandFinished {
                    action_id: tool_log_action_id,
                    tool: "yt-dlp".to_owned(),
                    action: "flat update".to_owned(),
                    command_line,
                    success: result.is_ok(),
                });
            }
            let _ = tx.send(MusicStreamResolveEvent::FlatUpdate {
                item_id,
                source,
                result,
            });
        });
    }

    fn begin_music_batch_add(&mut self, source: String) {
        self.begin_batch_add_with_kind(source, true);
    }

    fn update_music_compact_item_from_seed(
        &mut self,
        item_id: QueueItemId,
        seed: PlaylistEntrySeed,
    ) {
        let cache_key = music_cache_key(&seed.source_url, "flat", "", "");
        let mut changed = false;
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            item.source_url = seed.source_url;
            if !seed.title.trim().is_empty() {
                item.title = seed.title;
            }
            item.music_album_title = seed.album_title;
            if !seed.thumbnail_url.trim().is_empty() {
                item.thumbnail_url = seed.thumbnail_url;
            }
            if !seed.thumbnail_hint.trim().is_empty() {
                item.thumbnail_hint = seed.thumbnail_hint;
            }
            if !seed.duration_text.trim().is_empty() {
                item.duration_text = seed.duration_text;
            }
            item.metadata_state = MetadataState::Idle;
            item.compact_music_state = Some(CompactMusicState::Ready);
            item.music_duration_seconds = duration_text_to_seconds(&item.duration_text);
            item.music_stream_url.clear();
            item.music_stream_headers.clear();
            item.music_stream_ext.clear();
            item.music_stream_format_id.clear();
            item.music_stream_acodec.clear();
            item.music_stream_expected_bytes = None;
            item.music_cache_key = cache_key;
            item.last_error = None;
            if item.selection.file_name.trim().is_empty() {
                item.selection.file_name = sanitize_file_name_for_windows(item.title.trim());
            }
            changed = true;
        }
        if changed {
            self.mark_font_content_changed();
        }
        self.restore_music_compact_cache_hit_if_available(item_id);
        if let Some(item) = self.queue_item_by_id(item_id) {
            self.cache_music_cover_for_item(item);
        }
    }

    fn append_music_compact_seed(&mut self, seed: PlaylistEntrySeed) -> bool {
        let source_key = canonical_queue_source_key(&seed.source_url);
        if self
            .queue_items
            .iter()
            .any(|item| canonical_queue_source_key(&item.source_url) == source_key)
        {
            return false;
        }

        let mut item = self.build_queue_item_from_seed(seed);
        item.view_kind = QueueItemViewKind::MusicCompact;
        item.compact_music_state = Some(CompactMusicState::Ready);
        item.metadata_state = MetadataState::Idle;
        item.music_duration_seconds = duration_text_to_seconds(&item.duration_text);
        item.music_stream_url.clear();
        item.music_stream_headers.clear();
        item.music_stream_ext.clear();
        item.music_stream_format_id.clear();
        item.music_stream_acodec.clear();
        item.music_stream_expected_bytes = None;
        item.music_cache_key = music_cache_key(&item.source_url, "flat", "", "");
        item.last_error = None;
        let item_id = item.id;
        let source_url = item.source_url.clone();
        self.queue_items.push(item);
        self.mark_font_content_changed();
        self.restore_music_compact_cache_hit_if_available(item_id);
        if let Some(item) = self.queue_item_by_id(item_id) {
            self.cache_music_cover_for_item(item);
        }
        self.batch_input_push_unique(&source_url);
        true
    }

    fn cache_music_cover_for_item(&self, item: &QueueItem) {
        let url = item.thumbnail_url.trim();
        if url.is_empty() {
            return;
        }
        let key = if item.music_cache_key.trim().is_empty() {
            music_cache_key(&item.source_url, "flat", "", "")
        } else {
            item.music_cache_key.clone()
        };
        let dir = self
            .music_stream_cache_root()
            .join("covers")
            .join(sanitize_music_cache_key(&key));
        if self.music_cache_cover_dirs(item).into_iter().any(|dir| {
            first_music_cover_file_in_dir(&dir).is_some()
                && cached_music_cover_source_matches(&dir, url)
        }) {
            return;
        }
        let url = url.to_owned();
        thread::spawn(move || {
            if let Err(error) = download_music_cover_to_dir(&url, &dir) {
                eprintln!("[music-stream] flat cover cache skipped: {error}");
            }
        });
    }

    fn cache_music_lyrics_for_item(&self, item: &QueueItem, track: Option<&SubtitleOption>) {
        let Some(track) = track.filter(|track| track.source == SubtitleSource::Original) else {
            return;
        };
        let language_code = track.download_language_code.trim();
        if language_code.is_empty() {
            return;
        }
        let cache_key = if item.music_cache_key.trim().is_empty() {
            music_cache_key(&item.source_url, "lyrics", "", "")
        } else {
            item.music_cache_key.clone()
        };
        let lyrics_path = music_lrc_cache_path(&self.music_stream_cache_root(), &cache_key);
        if lyrics_path.is_file() {
            return;
        }
        let job = MusicLyricsCacheJob {
            source_url: item.source_url.clone(),
            cache_key,
            language_code: language_code.to_owned(),
            use_cookies: item.selection.use_cookies,
        };
        let tool_paths = self.tool_paths.clone();
        let cache_root = self.music_stream_cache_root();
        thread::spawn(move || {
            if let Err(error) = cache_music_lyrics_with_yt_dlp(&tool_paths, &cache_root, job) {
                eprintln!("[music-lyrics] cache skipped: {error}");
            }
        });
    }

    pub fn music_current_lyrics_display(&mut self) -> Option<MusicLyricsDisplayLine> {
        let current = self.current_music_lyrics_line_with_lead();
        self.update_music_lyrics_display_state(current)
    }

    fn current_music_lyrics_line_with_lead(&mut self) -> Option<String> {
        let control = self.music_playback.clone()?;
        let item = self.queue_item_by_id(control.item_id)?.clone();
        let cache_key = item.music_cache_key.trim();
        if cache_key.is_empty() {
            return None;
        }
        let seconds =
            if self.music_seek_drag_ratio.is_some() || self.music_seek_snap_ratio.is_some() {
                control
                    .duration_seconds()
                    .map(|duration| {
                        duration * f64::from(self.music_seek_display_ratio().clamp(0.0, 1.0))
                    })
                    .unwrap_or_else(|| control.playback_seconds())
            } else {
                control.playback_seconds()
            } + MUSIC_LYRICS_DISPLAY_LEAD_SECONDS;
        let path = music_lrc_cache_path(&self.music_stream_cache_root(), cache_key);
        if !path.is_file() {
            return None;
        }
        let lines = self.cached_music_lrc_lines(cache_key, path)?;
        current_lrc_line_text(&lines, seconds)
    }

    fn update_music_lyrics_display_state(
        &mut self,
        current: Option<String>,
    ) -> Option<MusicLyricsDisplayLine> {
        let now = Instant::now();
        match current {
            Some(current) => {
                if self.music_lyrics_display_line.as_deref() != Some(current.as_str()) {
                    self.music_lyrics_previous_line = self.music_lyrics_display_line.take();
                    self.music_lyrics_display_line = Some(current);
                    self.music_lyrics_transition_started_at = Some(now);
                }
            }
            None => {
                self.music_lyrics_display_line = None;
                self.music_lyrics_previous_line = None;
                self.music_lyrics_transition_started_at = None;
                return None;
            }
        }

        let fade = self
            .music_lyrics_transition_started_at
            .map(|started| {
                (now.duration_since(started).as_secs_f64() / MUSIC_LYRICS_FADE_SECONDS)
                    .clamp(0.0, 1.0) as f32
            })
            .unwrap_or(1.0);
        if fade >= 1.0 {
            self.music_lyrics_previous_line = None;
            self.music_lyrics_transition_started_at = None;
        }

        self.music_lyrics_display_line
            .as_ref()
            .map(|current| MusicLyricsDisplayLine {
                current: current.clone(),
                previous: self.music_lyrics_previous_line.clone(),
                fade,
            })
    }

    fn cached_music_lrc_lines(&mut self, cache_key: &str, path: PathBuf) -> Option<Vec<LrcLine>> {
        let metadata = fs::metadata(&path).ok();
        let modified = metadata.as_ref().and_then(|meta| meta.modified().ok());
        let now = Instant::now();
        if metadata.is_none() {
            let entry = self
                .music_lyrics_cache
                .entry(cache_key.to_owned())
                .or_insert_with(|| CachedLrcTrack {
                    path: path.clone(),
                    modified: None,
                    lines: Vec::new(),
                    missing_checked_at: Some(now),
                });
            let recently_checked = entry
                .missing_checked_at
                .is_some_and(|checked| now.duration_since(checked) < Duration::from_secs(1));
            if !recently_checked {
                entry.path = path;
                entry.modified = None;
                entry.lines.clear();
                entry.missing_checked_at = Some(now);
            }
            return None;
        }

        let reload = self
            .music_lyrics_cache
            .get(cache_key)
            .map_or(true, |entry| {
                entry.path != path || entry.modified != modified || entry.lines.is_empty()
            });
        if reload {
            let lines = parse_lrc_file(&path).unwrap_or_default();
            self.music_lyrics_cache.insert(
                cache_key.to_owned(),
                CachedLrcTrack {
                    path,
                    modified,
                    lines,
                    missing_checked_at: None,
                },
            );
        }
        self.music_lyrics_cache
            .get(cache_key)
            .map(|entry| entry.lines.clone())
            .filter(|lines| !lines.is_empty())
    }

    pub fn has_music_compact_items(&self) -> bool {
        self.queue_display_mode == QueueDisplayMode::Audio && !self.queue_items.is_empty()
    }

    pub fn music_item_cache_progress_ratio(&self, item_id: QueueItemId) -> f32 {
        if let Some(control) = self
            .music_playback
            .as_ref()
            .filter(|control| control.item_id == item_id)
        {
            return control.cache_progress_ratio();
        }
        self.music_cached_progress_for_item(item_id)
    }

    pub fn music_item_compact_progress_ratio(&self, item_id: QueueItemId) -> f32 {
        if let Some(progress) = self.music_item_active_download_progress_ratio(item_id) {
            return progress;
        }
        self.music_item_cache_progress_ratio(item_id)
    }

    pub fn music_item_compact_progress_visible(&self, item_id: QueueItemId) -> bool {
        if self
            .music_item_active_download_progress_ratio(item_id)
            .is_some()
        {
            return true;
        }
        self.music_playback
            .as_ref()
            .filter(|control| control.item_id == item_id)
            .is_some_and(|control| {
                let progress = control.cache_progress_ratio();
                !control.cache_is_complete() && progress > 0.0 && progress < 0.999
            })
    }

    fn music_item_active_download_progress_ratio(&self, item_id: QueueItemId) -> Option<f32> {
        let item = self.queue_item_by_id(item_id)?;
        let has_active_download = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item_id && workflow.kind == WorkflowKind::DownloadMedia
        });
        if !has_active_download {
            return None;
        }
        let progress = item
            .progress
            .audio
            .max(item.progress.video)
            .max(item.progress.post_process);
        Some(progress.clamp(0.0, 100.0) / 100.0)
    }

    pub fn music_item_compact_progress_status_text(&self, item_id: QueueItemId) -> Option<String> {
        self.music_item_active_download_progress_ratio(item_id)
            .map(|ratio| format!("{}%", (ratio * 100.0).round().clamp(0.0, 99.0) as u32))
    }

    pub fn music_item_playback_progress_ratio(&self, item_id: QueueItemId) -> f32 {
        self.music_playback
            .as_ref()
            .filter(|control| control.item_id == item_id)
            .map(MusicPlaybackControl::progress_ratio)
            .unwrap_or(0.0)
    }

    pub fn music_player_is_playing(&self) -> bool {
        self.music_playback
            .as_ref()
            .is_some_and(|control| !control.is_paused())
    }

    pub fn music_player_error_text(&self) -> Option<&str> {
        self.music_player_error
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub fn has_music_playback_activity(&self) -> bool {
        self.music_playback.is_some()
            || self.queue_items.iter().any(|item| {
                matches!(
                    item.compact_music_state,
                    Some(
                        CompactMusicState::Resolving
                            | CompactMusicState::Buffering
                            | CompactMusicState::Playing
                            | CompactMusicState::Paused
                    )
                )
            })
    }

    pub fn music_playback_progress_ratio(&self) -> f32 {
        self.music_playback
            .as_ref()
            .map(MusicPlaybackControl::progress_ratio)
            .unwrap_or(0.0)
    }

    pub fn music_playback_cache_progress_ratio(&self) -> f32 {
        self.music_playback
            .as_ref()
            .map(MusicPlaybackControl::cache_progress_ratio)
            .unwrap_or_else(|| {
                self.music_player_current_item_id
                    .map(|id| self.music_cached_progress_for_item(id))
                    .unwrap_or(0.0)
            })
    }

    pub fn music_seek_drag_ratio(&self) -> Option<f32> {
        self.music_seek_drag_ratio
    }

    pub fn music_seek_display_ratio(&mut self) -> f32 {
        if let Some(value) = self.music_seek_drag_ratio {
            return value.clamp(0.0, 1.0);
        }
        if let (Some(value), Some(deadline)) =
            (self.music_seek_snap_ratio, self.music_seek_snap_deadline)
        {
            if Instant::now() <= deadline {
                return value.clamp(0.0, 1.0);
            }
            self.music_seek_snap_ratio = None;
            self.music_seek_snap_deadline = None;
        }
        self.music_playback_progress_ratio()
    }

    pub fn set_music_seek_drag_ratio(&mut self, ratio: Option<f32>) {
        self.music_seek_drag_ratio = ratio.map(|value| value.clamp(0.0, 1.0));
        if ratio.is_some() {
            self.music_seek_snap_ratio = None;
            self.music_seek_snap_deadline = None;
        }
    }

    pub fn finish_music_seek_drag(&mut self, ratio: f32) {
        self.music_seek_drag_ratio = None;
        self.seek_music_playback_ratio(ratio);
    }

    pub fn seek_music_playback_ratio(&mut self, ratio: f32) {
        let Some(control) = self.music_playback.clone() else {
            return;
        };
        let requested = ratio.clamp(0.0, 1.0);
        let cache_ratio = control.cache_progress_ratio().clamp(0.0, 1.0);
        let safe_cache_ratio = if control.cache_is_complete() {
            1.0
        } else {
            // Avoid seeking exactly at the growing-file edge; keep a tiny guard
            // so UI over-drag snaps to a point that is actually buffered.
            (cache_ratio - 0.01).max(0.0)
        };
        let allowed = requested.min(safe_cache_ratio);
        if requested > allowed + f32::EPSILON {
            self.music_seek_snap_ratio = Some(allowed);
            self.music_seek_snap_deadline = Some(Instant::now() + Duration::from_millis(700));
            self.last_action =
                "Outside the cached range; moved back to a playable position.".to_owned();
        } else {
            self.music_seek_snap_ratio = None;
            self.music_seek_snap_deadline = None;
        }
        control.seek_to_ratio(allowed);
    }

    pub fn music_playback_time_text(&mut self) -> String {
        let Some(control) = self.music_playback.clone() else {
            return "00:00 / --:--".to_owned();
        };
        let duration = control.duration_seconds();
        let preview_ratio = (self.music_seek_drag_ratio.is_some()
            || self.music_seek_snap_ratio.is_some())
        .then(|| self.music_seek_display_ratio().clamp(0.0, 1.0));
        let current_seconds = match (preview_ratio, duration) {
            (Some(ratio), Some(duration)) => duration * f64::from(ratio),
            _ => control.playback_seconds(),
        };
        let current = format_duration_seconds(current_seconds);
        let total = duration
            .map(format_duration_seconds)
            .unwrap_or_else(|| "--:--".to_owned());
        format!("{current} / {total}")
    }

    pub fn music_volume(&self) -> f32 {
        self.music_volume
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        let next = volume.clamp(0.0, 1.0);
        if (self.music_volume - next).abs() < 0.001 {
            return;
        }
        self.music_volume = next;
        self.config.music_volume = next;
        let _ = self.config.save();
        if let Some(control) = &self.music_playback {
            control.set_volume(self.music_volume);
        }
    }

    pub fn toggle_music_playback(&mut self) {
        if let Some(control) = self.music_playback.clone() {
            if control.is_paused() {
                control.resume();
                self.mark_music_playback_state(control.item_id, CompactMusicState::Playing);
            } else {
                control.pause();
                self.mark_music_playback_state(control.item_id, CompactMusicState::Paused);
            }
            return;
        }

        if let Some(current) = self.music_player_current_item_id {
            if self.queue_item_by_id(current).is_some_and(|item| {
                matches!(
                    item.compact_music_state,
                    Some(CompactMusicState::Resolving | CompactMusicState::Buffering)
                )
            }) {
                self.last_action = "Music stream is still preparing.".to_owned();
                return;
            }
        }

        let item_id = self
            .music_player_current_item_id
            .filter(|id| self.music_item_can_play(*id))
            .or_else(|| {
                self.queue_items
                    .iter()
                    .find(|item| self.music_item_can_play(item.id))
                    .map(|item| item.id)
            });

        if let Some(item_id) = item_id {
            self.start_music_stream_playback(item_id);
        } else {
            self.last_action = "There are no playable music items.".to_owned();
        }
    }

    pub fn stop_music_playback(&mut self) {
        if let Some(control) = self.music_playback.take() {
            control.stop();
            self.mark_music_playback_state(control.item_id, CompactMusicState::Ready);
        }
        self.music_player_current_item_id = None;
        self.cancel_music_prefetch();
        self.media_session.clear();
    }

    fn cancel_music_prefetch(&mut self) {
        if let Some(control) = self.music_prefetch_control.take() {
            control.cancel();
        }
        self.music_prefetch_active_item_id = None;
        self.music_prefetch_pending_item_id = None;
        self.music_prefetch_for_current_item_id = None;
        self.music_prefetch_started_at = None;
        self.music_prefetch_session_id = self.music_prefetch_session_id.wrapping_add(1).max(1);
    }

    fn next_music_playback_session_id(&mut self) -> u64 {
        self.music_playback_session_id = self.music_playback_session_id.wrapping_add(1).max(1);
        self.music_playback_session_id
    }

    fn start_music_stream_playback(&mut self, item_id: QueueItemId) {
        self.start_music_stream_playback_recorded(item_id, true);
    }

    fn start_music_stream_playback_recorded(&mut self, item_id: QueueItemId, record_history: bool) {
        if record_history {
            self.record_music_navigation_target(item_id);
        }
        let session_id = self.next_music_playback_session_id();
        self.start_music_stream_playback_with_session(item_id, session_id);
    }

    fn start_music_stream_playback_with_session(&mut self, item_id: QueueItemId, session_id: u64) {
        if self.music_player_current_item_id != Some(item_id) {
            self.music_reserved_next_item_id = None;
            self.cancel_music_prefetch();
        }
        if let Some(previous_id) = self
            .music_player_current_item_id
            .filter(|id| *id != item_id)
        {
            if self
                .queue_item_by_id(previous_id)
                .is_some_and(|item| item.compact_music_state != Some(CompactMusicState::Failed))
            {
                self.mark_music_playback_state(previous_id, CompactMusicState::Ready);
            }
        }
        if let Some(control) = self.music_playback.take() {
            control.stop();
            self.mark_music_playback_state(control.item_id, CompactMusicState::Ready);
        }

        let Some(mut item) = self.queue_item_by_id(item_id).cloned() else {
            return;
        };
        self.music_playback_session_id = session_id;
        if item.music_stream_url.trim().is_empty() {
            if let Some(hit) = self.complete_music_cache_hit_for_item(&item) {
                if let Some(target) = self.queue_item_mut_by_id(item_id) {
                    restore_music_compact_item_from_cache_hit(target, &hit);
                    item = target.clone();
                }
                eprintln!(
                    "[music-stream] restored complete cache for item={} key={}",
                    item_id, hit.cache_key
                );
            } else {
                self.resolve_music_item_for_playback_with_session(item_id, session_id);
                return;
            }
        }

        self.music_player_error = None;
        let cache_root = self.music_stream_cache_root();
        let cache_dir = cache_root.join(sanitize_music_cache_key(&item.music_cache_key));
        let cache_media_path = cache_dir.join(format!(
            "audio.{}",
            sanitize_music_cache_ext(&item.music_stream_ext)
        ));
        let cache_command = if self.complete_music_cache_media_path(&item).is_some() {
            None
        } else {
            match self.tool_paths.prepare_music_stream_cache_command(
                &item.source_url,
                &cache_media_path,
                &item.music_stream_format_id,
                item.selection.use_cookies,
            ) {
                Ok(command) => Some(command),
                Err(error) => {
                    let message = i18n::format_fixed_english(
                        "Music cache preparation failed: {error}",
                        &[("{error}", error.as_str())],
                    );
                    self.music_player_error = Some(message.clone());
                    self.push_runtime_log(message.clone());
                    self.last_action = message;
                    return;
                }
            }
        };

        let lyrics_track = item
            .metadata()
            .and_then(primary_original_subtitle_track_from_metadata)
            .cloned();
        self.cache_music_lyrics_for_item(&item, lyrics_track.as_ref());

        let stream = ResolvedMusicStream {
            item_id,
            session_id,
            source_url: item.source_url.clone(),
            direct_url: item.music_stream_url.clone(),
            headers: item.music_stream_headers.clone(),
            title: item.title.clone(),
            album_title: item.music_album_title.clone(),
            thumbnail_url: item.thumbnail_url.clone(),
            duration_seconds: item.music_duration_seconds,
            ext: item.music_stream_ext.clone(),
            format_id: item.music_stream_format_id.clone(),
            acodec: item.music_stream_acodec.clone(),
            cache_key: item.music_cache_key.clone(),
            expected_bytes: item.music_stream_expected_bytes,
            cache_root,
            cache_command,
            volume: self.music_volume,
        };
        let control =
            music_stream::spawn_music_stream_playback(stream, self.music_playback_event_tx.clone());
        self.music_playback = Some(control);
        self.music_player_current_item_id = Some(item_id);
        self.mark_music_playback_state(item_id, CompactMusicState::Buffering);
        self.last_action = i18n::format_fixed_english(
            "Preparing playback: {title}",
            &[("{title}", item.title.as_str())],
        );
    }

    fn resolve_music_item_for_playback(&mut self, item_id: QueueItemId) {
        let session_id = self.next_music_playback_session_id();
        self.resolve_music_item_for_playback_with_session(item_id, session_id);
    }

    fn resolve_music_item_for_playback_with_session(
        &mut self,
        item_id: QueueItemId,
        session_id: u64,
    ) {
        let Some(item) = self.queue_item_by_id(item_id).cloned() else {
            return;
        };
        if item.source_url.trim().is_empty() {
            self.last_action = "Music item is missing a source URL.".to_owned();
            return;
        }
        self.mark_music_playback_state(item_id, CompactMusicState::Resolving);
        self.music_player_current_item_id = Some(item_id);
        self.music_playback_session_id = session_id;
        self.music_player_error = None;
        self.last_action = i18n::format_fixed_english(
            "Resolving music stream: {title}",
            &[("{title}", item.title.as_str())],
        );

        let tx = self.music_stream_result_tx.clone();
        let tool_paths = self.tool_paths.clone();
        let source = item.source_url.clone();
        let use_cookies = item.selection.use_cookies;
        let tool_log_action_id = self.push_tool_log_action("audio", "resolve stream");
        thread::spawn(move || {
            let (result, command_line, success) =
                match tool_paths.analyze_music_stream_url_detailed(&source, use_cookies) {
                    Ok(output) => {
                        let command_line = output.command_line.clone();
                        let result = music_stream_seed_from_json(&output.json, &source);
                        (result, Some(command_line), true)
                    }
                    Err(error) => (Err(error.message), error.command_line, false),
                };
            let command_line = command_line.or_else(|| (!success).then(|| "yt-dlp".to_owned()));
            if let Some(command_line) = command_line {
                let _ = tx.send(MusicStreamResolveEvent::ToolCommandFinished {
                    action_id: tool_log_action_id,
                    tool: "yt-dlp".to_owned(),
                    action: "resolve stream".to_owned(),
                    command_line,
                    success,
                });
            }
            let _ = tx.send(MusicStreamResolveEvent::Resolve {
                item_id,
                session_id,
                source,
                play_after_resolve: true,
                result,
            });
        });
    }

    fn maybe_prefetch_next_music_item(&mut self) {
        if self.queue_display_mode != QueueDisplayMode::Audio {
            return;
        }
        if self.music_prefetch_active_item_id.is_some()
            || self.music_prefetch_pending_item_id.is_some()
        {
            return;
        }
        let Some(control) = self.music_playback.clone() else {
            return;
        };
        if control.is_paused() {
            return;
        }
        let current_item_id = control.item_id;
        if self.music_player_current_item_id != Some(current_item_id) {
            return;
        }
        if self.music_prefetch_for_current_item_id == Some(current_item_id) {
            return;
        }
        let played = control.playback_seconds();
        if played < MUSIC_PREFETCH_MIN_PLAY_SECONDS {
            return;
        }
        let Some(duration) = control.duration_seconds().or_else(|| {
            self.queue_item_by_id(current_item_id)
                .and_then(|item| item.music_duration_seconds)
        }) else {
            return;
        };
        if duration <= 0.0 || !duration.is_finite() {
            return;
        }
        let lead = self.music_prefetch_lead_seconds.clamp(
            MUSIC_PREFETCH_MIN_LEAD_SECONDS,
            MUSIC_PREFETCH_MAX_LEAD_SECONDS,
        );
        let remaining = (duration - played).max(0.0);
        if remaining > lead {
            return;
        }

        let allow_wrap = matches!(
            self.music_playback_mode,
            MusicPlaybackMode::RepeatAll | MusicPlaybackMode::Shuffle
        );
        let Some(next_item_id) = self.peek_next_music_item_id_for_prefetch(allow_wrap) else {
            return;
        };
        if next_item_id == current_item_id {
            return;
        }
        self.music_prefetch_for_current_item_id = Some(current_item_id);
        self.start_music_prefetch_for_item(next_item_id);
    }

    fn start_music_prefetch_for_item(&mut self, item_id: QueueItemId) {
        let Some(item) = self.queue_item_by_id(item_id).cloned() else {
            return;
        };
        if self.complete_music_cache_media_path(&item).is_some() {
            return;
        }
        if let Some(hit) = self.complete_music_cache_hit_for_item(&item) {
            if let Some(target) = self.queue_item_mut_by_id(item_id) {
                restore_music_compact_item_from_cache_hit(target, &hit);
            }
            self.save_active_audio_playlist_if_needed();
            return;
        }
        if item.music_stream_url.trim().is_empty() || item.music_stream_format_id.trim().is_empty()
        {
            self.resolve_music_item_for_prefetch(item_id);
            return;
        }
        self.start_resolved_music_prefetch(item);
    }

    fn resolve_music_item_for_prefetch(&mut self, item_id: QueueItemId) {
        let Some(item) = self.queue_item_by_id(item_id).cloned() else {
            return;
        };
        if item.source_url.trim().is_empty() {
            return;
        }
        self.music_prefetch_pending_item_id = Some(item_id);
        let session_id = self.next_music_prefetch_session_id();
        let tx = self.music_stream_result_tx.clone();
        let tool_paths = self.tool_paths.clone();
        let source = item.source_url.clone();
        let use_cookies = item.selection.use_cookies;
        let tool_log_action_id = self.push_tool_log_action("audio", "prefetch resolve");
        thread::spawn(move || {
            let (result, command_line, success) =
                match tool_paths.analyze_music_stream_url_detailed(&source, use_cookies) {
                    Ok(output) => {
                        let command_line = output.command_line.clone();
                        let result = music_stream_seed_from_json(&output.json, &source);
                        (result, Some(command_line), true)
                    }
                    Err(error) => (Err(error.message), error.command_line, false),
                };
            let command_line = command_line.or_else(|| (!success).then(|| "yt-dlp".to_owned()));
            if let Some(command_line) = command_line {
                let _ = tx.send(MusicStreamResolveEvent::ToolCommandFinished {
                    action_id: tool_log_action_id,
                    tool: "yt-dlp".to_owned(),
                    action: "prefetch resolve".to_owned(),
                    command_line,
                    success,
                });
            }
            let _ = tx.send(MusicStreamResolveEvent::Resolve {
                item_id,
                session_id,
                source,
                play_after_resolve: false,
                result,
            });
        });
    }

    fn start_resolved_music_prefetch(&mut self, item: QueueItem) {
        if self.complete_music_cache_media_path(&item).is_some() {
            return;
        }
        let cache_root = self.music_stream_cache_root();
        let cache_dir = cache_root.join(sanitize_music_cache_key(&item.music_cache_key));
        let cache_media_path = cache_dir.join(format!(
            "audio.{}",
            sanitize_music_cache_ext(&item.music_stream_ext)
        ));
        let cache_command = match self.tool_paths.prepare_music_stream_cache_command(
            &item.source_url,
            &cache_media_path,
            &item.music_stream_format_id,
            item.selection.use_cookies,
        ) {
            Ok(command) => Some(command),
            Err(error) => {
                eprintln!(
                    "[music-prefetch] prepare skipped for item={}: {error}",
                    item.id
                );
                return;
            }
        };

        let session_id = self.next_music_prefetch_session_id();
        self.music_prefetch_active_item_id = Some(item.id);
        self.music_prefetch_started_at = Some(Instant::now());
        let stream = ResolvedMusicStream {
            item_id: item.id,
            session_id,
            source_url: item.source_url.clone(),
            direct_url: item.music_stream_url.clone(),
            headers: item.music_stream_headers.clone(),
            title: item.title.clone(),
            album_title: item.music_album_title.clone(),
            thumbnail_url: item.thumbnail_url.clone(),
            duration_seconds: item.music_duration_seconds,
            ext: item.music_stream_ext.clone(),
            format_id: item.music_stream_format_id.clone(),
            acodec: item.music_stream_acodec.clone(),
            cache_key: item.music_cache_key.clone(),
            expected_bytes: item.music_stream_expected_bytes,
            cache_root,
            cache_command,
            volume: 0.0,
        };
        let control =
            music_stream::spawn_music_stream_prefetch(stream, self.music_playback_event_tx.clone());
        self.music_prefetch_control = Some(control);
    }

    fn next_music_prefetch_session_id(&mut self) -> u64 {
        self.music_prefetch_session_id = self.music_prefetch_session_id.wrapping_add(1).max(1);
        self.music_prefetch_session_id
    }

    fn prefetch_event_is_current(&self, item_id: QueueItemId, session_id: u64) -> bool {
        self.music_prefetch_active_item_id == Some(item_id)
            && self.music_prefetch_session_id == session_id
    }

    fn music_item_can_play(&self, item_id: QueueItemId) -> bool {
        self.queue_display_mode == QueueDisplayMode::Audio
            && self.queue_item_by_id(item_id).is_some_and(|item| {
                matches!(
                    item.compact_music_state.unwrap_or(CompactMusicState::Ready),
                    CompactMusicState::Ready
                        | CompactMusicState::Playing
                        | CompactMusicState::Paused
                ) && !item.source_url.trim().is_empty()
            })
    }

    fn mark_music_playback_state(&mut self, item_id: QueueItemId, music_state: CompactMusicState) {
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            item.compact_music_state = Some(music_state);
        }
    }

    pub fn music_current_item_id(&self) -> Option<QueueItemId> {
        self.music_player_current_item_id
    }

    pub fn music_item_is_playing(&self, item_id: QueueItemId) -> bool {
        self.music_player_current_item_id == Some(item_id) && self.music_player_is_playing()
    }

    pub fn play_music_item(&mut self, item_id: QueueItemId) {
        if self.music_player_current_item_id == Some(item_id) {
            if self.music_playback.is_some() {
                self.toggle_music_playback();
            } else {
                self.last_action = "Music stream is still resolving.".to_owned();
            }
            return;
        }
        if self.music_item_can_play(item_id) {
            self.start_music_stream_playback(item_id);
        } else if let Some(item) = self.queue_item_by_id(item_id) {
            let message = match item.compact_music_state {
                Some(CompactMusicState::Resolving) => "Music stream is still resolving.".to_owned(),
                Some(CompactMusicState::Buffering) => "Music is buffering.".to_owned(),
                Some(CompactMusicState::Failed) => item
                    .last_error
                    .clone()
                    .unwrap_or_else(|| "This music item cannot be played right now.".to_owned()),
                _ => "Music stream is not ready yet.".to_owned(),
            };
            self.last_action = message;
        }
    }

    pub fn previous_music_item(&mut self) {
        let Some(item_id) = self.previous_music_item_id() else {
            self.last_action = "No previous track.".to_owned();
            return;
        };
        if let Some(current) = self
            .music_player_current_item_id
            .filter(|id| *id != item_id)
        {
            self.music_history_forward.push(current);
        }
        self.request_music_scroll_to_item(item_id);
        self.start_music_stream_playback_recorded(item_id, false);
    }

    pub fn next_music_item(&mut self) {
        let Some(item_id) = self.next_music_item_id(false) else {
            self.last_action = "No next track.".to_owned();
            return;
        };
        self.request_music_scroll_to_item(item_id);
        self.start_music_stream_playback_recorded(item_id, true);
    }

    fn request_music_scroll_to_item(&mut self, item_id: QueueItemId) {
        self.music_scroll_to_item_id = Some(item_id);
    }

    pub fn take_music_scroll_to_item_request(&mut self, item_id: QueueItemId) -> bool {
        if self.music_scroll_to_item_id == Some(item_id) {
            self.music_scroll_to_item_id = None;
            return true;
        }
        false
    }

    pub fn cycle_music_playback_mode(&mut self) {
        self.music_playback_mode = self.music_playback_mode.next();
        self.config.music_playback_mode = self.music_playback_mode.config_value().to_owned();
        let _ = self.config.save();
        let mode_label = self
            .ui_i18n_text_for_key(self.music_playback_mode.label_key())
            .to_owned();
        self.last_action =
            i18n::format_fixed_english("Playback mode: {mode}", &[("{mode}", mode_label.as_str())]);
    }

    pub fn music_playback_mode_text(&self) -> &'static str {
        self.ui_i18n_text_for_key(self.music_playback_mode.label_key())
    }

    pub fn music_playback_mode_kind(&self) -> MusicPlaybackMode {
        self.music_playback_mode
    }

    fn advance_music_after_finished(&mut self, finished_item_id: QueueItemId) {
        let next = match self.music_playback_mode {
            MusicPlaybackMode::RepeatOne => Some(finished_item_id),
            MusicPlaybackMode::Sequential => self.next_music_item_id(false),
            MusicPlaybackMode::RepeatAll | MusicPlaybackMode::Shuffle => {
                self.next_music_item_id(true)
            }
        };
        if let Some(item_id) = next {
            self.start_music_stream_playback(item_id);
        }
    }

    fn previous_music_item_id(&mut self) -> Option<QueueItemId> {
        while let Some(item_id) = self.music_history_back.pop() {
            if self.music_item_can_play(item_id) {
                return Some(item_id);
            }
        }

        if self.music_playback_mode == MusicPlaybackMode::Shuffle {
            return None;
        }

        let items = self.music_playable_item_ids();
        if items.is_empty() {
            return None;
        }
        let current = self.music_player_current_item_id?;
        let index = items.iter().position(|id| *id == current)?;
        if index > 0 {
            items.get(index - 1).copied()
        } else {
            items
                .last()
                .copied()
                .filter(|_| self.music_playback_mode == MusicPlaybackMode::RepeatAll)
        }
    }

    fn next_music_item_id(&mut self, allow_wrap: bool) -> Option<QueueItemId> {
        if self.music_playback_mode == MusicPlaybackMode::Shuffle {
            while let Some(item_id) = self.music_history_forward.pop() {
                if self.music_item_can_play(item_id) {
                    return Some(item_id);
                }
            }
            if let Some(item_id) = self
                .music_reserved_next_item_id
                .take()
                .filter(|id| self.music_item_can_play(*id))
            {
                return Some(item_id);
            }
            return self.random_music_next_item_id(allow_wrap);
        }
        self.ordered_next_music_item_id(allow_wrap)
    }

    fn peek_next_music_item_id_for_prefetch(&mut self, allow_wrap: bool) -> Option<QueueItemId> {
        if self.music_playback_mode == MusicPlaybackMode::Shuffle {
            if let Some(item_id) = self
                .music_reserved_next_item_id
                .filter(|id| self.music_item_can_play(*id))
            {
                return Some(item_id);
            }
            let item_id = self.random_music_next_item_id(allow_wrap)?;
            self.music_reserved_next_item_id = Some(item_id);
            return Some(item_id);
        }
        self.ordered_next_music_item_id(allow_wrap)
    }

    fn ordered_next_music_item_id(&self, allow_wrap: bool) -> Option<QueueItemId> {
        let items = self.music_playable_item_ids();
        if items.is_empty() {
            return None;
        }
        let Some(current) = self.music_player_current_item_id else {
            return items.first().copied();
        };
        let Some(index) = items.iter().position(|id| *id == current) else {
            return items.first().copied();
        };
        if let Some(next) = items.get(index + 1).copied() {
            return Some(next);
        }
        if allow_wrap || self.music_playback_mode == MusicPlaybackMode::RepeatAll {
            items.first().copied()
        } else {
            None
        }
    }

    fn random_music_next_item_id(&self, allow_wrap: bool) -> Option<QueueItemId> {
        let items = self.music_playable_item_ids();
        if items.is_empty() {
            return None;
        }
        let current = self.music_player_current_item_id;
        let candidates = items
            .iter()
            .copied()
            .filter(|id| Some(*id) != current)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return allow_wrap.then(|| items[0]);
        }
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as usize)
            .unwrap_or(0);
        candidates.get(seed % candidates.len()).copied()
    }

    fn record_music_navigation_target(&mut self, item_id: QueueItemId) {
        let Some(current) = self.music_player_current_item_id else {
            return;
        };
        if current == item_id {
            return;
        }
        if self.music_history_back.last().copied() != Some(current) {
            self.music_history_back.push(current);
        }
        if self.music_history_back.len() > MUSIC_PLAY_HISTORY_LIMIT {
            let excess = self.music_history_back.len() - MUSIC_PLAY_HISTORY_LIMIT;
            self.music_history_back.drain(0..excess);
        }
        self.music_history_forward.clear();
    }

    fn prune_music_navigation_state(&mut self) {
        let playable = self.music_playable_item_ids();
        self.music_history_back.retain(|id| playable.contains(id));
        self.music_history_forward
            .retain(|id| playable.contains(id));
        if self
            .music_reserved_next_item_id
            .is_some_and(|id| !playable.contains(&id))
        {
            self.music_reserved_next_item_id = None;
        }
    }

    fn music_playable_item_ids(&self) -> Vec<QueueItemId> {
        self.queue_items
            .iter()
            .filter(|item| self.music_item_can_play(item.id))
            .map(|item| item.id)
            .collect()
    }

    fn complete_music_cache_hit_for_item(&self, item: &QueueItem) -> Option<CompleteMusicCacheHit> {
        let source_key = canonical_queue_source_key(&item.source_url);
        let mut best: Option<(u64, CompleteMusicCacheHit)> = None;
        let root = self.music_stream_cache_root();
        let entries = fs::read_dir(&root).ok()?;
        for entry in entries.filter_map(Result::ok) {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let manifest_path = dir.join("manifest.yaml");
            let Some(manifest) = read_yaml_file::<AudioCacheManifestSnapshot>(&manifest_path)
            else {
                continue;
            };
            if !audio_cache_manifest_is_fresh(&manifest) {
                let _ = fs::remove_dir_all(&dir);
                continue;
            }
            if !manifest.complete {
                continue;
            }
            let manifest_source = manifest.source_url.trim().to_owned();
            if manifest_source.is_empty() {
                continue;
            };
            if canonical_queue_source_key(&manifest_source) != source_key {
                continue;
            }
            let ext = manifest.ext.trim().to_owned();
            if ext.trim().is_empty() {
                continue;
            }
            let media_path = dir.join(format!("audio.{}", sanitize_music_cache_ext(&ext)));
            let media_len = fs::metadata(&media_path)
                .map(|meta| meta.len())
                .unwrap_or(0);
            if media_len == 0 {
                continue;
            }
            let expected_bytes = manifest.expected_bytes;
            if expected_bytes.is_some_and(|expected| expected > media_len) {
                continue;
            }
            let cache_key = dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_owned();
            if cache_key.trim().is_empty() {
                continue;
            }
            let updated = manifest.updated_unix_seconds;
            let hit = CompleteMusicCacheHit {
                cache_key,
                source_url: manifest_source,
                title: manifest.title,
                album_title: manifest.album_title,
                thumbnail_url: manifest.thumbnail_url,
                duration_seconds: manifest.duration_seconds,
                ext,
                format_id: manifest.format_id,
                acodec: manifest.acodec,
                expected_bytes,
            };
            let replace_best = match best.as_ref() {
                Some((best_updated, _)) => updated >= *best_updated,
                None => true,
            };
            if replace_best {
                best = Some((updated, hit));
            }
        }
        best.map(|(_, hit)| hit)
    }

    fn prepare_queue_items_for_audio_mode(&mut self) {
        let ids = self
            .queue_items
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>();
        for item_id in ids {
            self.prepare_queue_item_for_audio_mode(item_id);
        }
    }

    fn prepare_queue_item_for_audio_mode(&mut self, item_id: QueueItemId) {
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            if item.compact_music_state.is_none() {
                item.compact_music_state = Some(CompactMusicState::Ready);
            }
            if item.music_cache_key.trim().is_empty() && !item.source_url.trim().is_empty() {
                item.music_cache_key = music_cache_key(&item.source_url, "flat", "", "");
            }
            if item.music_duration_seconds.is_none() && !item.duration_text.trim().is_empty() {
                item.music_duration_seconds = duration_text_to_seconds(&item.duration_text);
            }
        }

        self.restore_music_compact_cache_hit_if_available(item_id);
        if let Some(item) = self.queue_item_by_id(item_id) {
            self.cache_music_cover_for_item(item);
        }
    }

    fn restore_music_compact_cache_hit_if_available(&mut self, item_id: QueueItemId) -> bool {
        let hit = {
            self.queue_item_by_id(item_id)
                .and_then(|item| self.complete_music_cache_hit_for_item(item))
        };
        let Some(hit) = hit else {
            return false;
        };
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            restore_music_compact_item_from_cache_hit(item, &hit);
            self.mark_font_content_changed();
            return true;
        }
        false
    }

    pub fn music_item_has_complete_cache(&self, item_id: QueueItemId) -> bool {
        let Some(item) = self.queue_item_by_id(item_id) else {
            return false;
        };
        self.complete_music_cache_media_path(item).is_some()
    }

    fn music_cached_progress_for_item(&self, item_id: QueueItemId) -> f32 {
        let Some(item) = self.queue_item_by_id(item_id) else {
            return 0.0;
        };
        music_cached_progress_for_item_in_root(item, &self.music_stream_cache_root())
    }

    fn music_stream_cache_root(&self) -> PathBuf {
        self.audio_cache_root_path()
    }

    fn audio_cache_root_path(&self) -> PathBuf {
        self.app_cache_root_path().join("audio")
    }

    fn audio_playlist_snapshot_path(&self) -> PathBuf {
        self.app_cache_root_path().join("audio-playlist.yaml")
    }

    fn transcode_temp_root_path(&self) -> PathBuf {
        self.app_cache_root_path().join("transcode-temp")
    }

    fn restore_saved_audio_playlist(&mut self) {
        let items = self.load_audio_playlist_snapshot_items();
        if self.queue_display_mode == QueueDisplayMode::Audio {
            self.queue_items = items;
            self.rebuild_batch_input_from_queue();
        } else {
            self.audio_queue_items = items;
        }
    }

    fn load_audio_playlist_snapshot_items(&mut self) -> Vec<QueueItem> {
        let path = self.audio_playlist_snapshot_path();
        let Some(snapshot) = read_yaml_file::<AudioPlaylistSnapshot>(&path) else {
            return Vec::new();
        };
        snapshot
            .items
            .into_iter()
            .filter_map(|entry| self.queue_item_from_audio_playlist_snapshot(entry))
            .collect()
    }

    fn queue_item_from_audio_playlist_snapshot(
        &mut self,
        entry: AudioPlaylistItemSnapshot,
    ) -> Option<QueueItem> {
        let source = entry.source_url.trim().to_owned();
        if source.is_empty() {
            return None;
        }
        let mut item = self.build_queue_item_from_url(&source);
        item.view_kind = QueueItemViewKind::MusicCompact;
        item.compact_music_state = Some(CompactMusicState::Ready);
        item.metadata_state = MetadataState::Idle;
        if !entry.title.trim().is_empty() {
            item.title = entry.title;
        }
        item.music_album_title = entry.album_title;
        item.thumbnail_hint = if entry.thumbnail_hint.trim().is_empty() {
            "item.thumbnail".to_owned()
        } else {
            entry.thumbnail_hint
        };
        item.thumbnail_url = entry.thumbnail_url;
        item.duration_text = entry.duration_text;
        item.music_duration_seconds = entry
            .duration_seconds
            .or_else(|| duration_text_to_seconds(&item.duration_text));
        item.music_stream_url.clear();
        item.music_stream_headers.clear();
        item.music_stream_ext = entry.stream_ext;
        item.music_stream_format_id = entry.stream_format_id;
        item.music_stream_acodec = entry.stream_acodec;
        item.music_stream_expected_bytes = entry.expected_bytes;
        item.music_cache_key = if entry.cache_key.trim().is_empty() {
            music_cache_key(&item.source_url, "flat", "", "")
        } else {
            entry.cache_key
        };
        item.selection.use_cookies = entry.use_cookies;
        item.last_error = None;
        self.restore_music_compact_cache_hit_for_item(&mut item);
        Some(item)
    }

    fn save_active_audio_playlist_if_needed(&self) {
        if self.queue_display_mode == QueueDisplayMode::Audio {
            self.save_audio_playlist_items(&self.queue_items);
        } else {
            self.save_audio_playlist_items(&self.audio_queue_items);
        }
    }

    fn save_audio_playlist_items(&self, items: &[QueueItem]) {
        let snapshot = AudioPlaylistSnapshot {
            version: 1,
            items: items
                .iter()
                .filter(|item| item.view_kind == QueueItemViewKind::MusicCompact)
                .filter(|item| !item.source_url.trim().is_empty())
                .map(audio_playlist_item_snapshot)
                .collect(),
        };
        let path = self.audio_playlist_snapshot_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(error) = write_yaml_file(&path, &snapshot) {
            eprintln!("[music-playlist] save skipped: {error}");
        }
    }

    fn rebuild_batch_input_from_queue(&mut self) {
        self.batch_input = self
            .queue_items
            .iter()
            .filter(|item| !item.source_url.trim().is_empty())
            .map(|item| item.source_url.clone())
            .collect::<Vec<_>>()
            .join("\n");
    }

    fn restore_music_compact_cache_hit_for_item(&self, item: &mut QueueItem) {
        if let Some(hit) = self.complete_music_cache_hit_for_item(item) {
            restore_music_compact_item_from_cache_hit(item, &hit);
        }
    }

    pub fn queue_summary(&self) -> QueueSummary {
        let mut summary = QueueSummary::default();
        summary.total = self.queue_items.len();

        for item in &self.queue_items {
            match item_summary_bucket(item) {
                QueueSummaryBucket::Queued => summary.queued += 1,
                QueueSummaryBucket::Completed => summary.completed += 1,
                QueueSummaryBucket::Failed => summary.failed += 1,
            }
        }

        summary
    }

    pub fn has_pending_download_items(&self) -> bool {
        !self.has_running_download_workflow()
            && self.queue_items.iter().any(item_can_enter_download_queue)
    }

    pub fn required_dependency_notice(&self) -> Option<String> {
        self.ensure_yt_dlp_ready().err()
    }

    fn ensure_yt_dlp_ready(&self) -> Result<(), String> {
        self.tool_paths.validate_yt_dlp_available().map(|_| ())
    }

    pub fn available_quality_presets(&self) -> [QualityPreset; 4] {
        [
            QualityPreset::Best,
            QualityPreset::P1080,
            QualityPreset::P720,
            QualityPreset::AudioOnly,
        ]
    }

    pub fn resolved_output_dir_display(&self) -> String {
        if self.output_dir_locked_by_config() {
            return "Controlled by config".to_owned();
        }
        let path = self.item_defaults.output_dir.as_str();
        resolve_output_dir(path)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| path.to_owned())
    }

    pub fn output_dir_display(&self) -> String {
        if self.output_dir_locked_by_config() {
            return "Controlled by config".to_owned();
        }
        let path = self.item_defaults.output_dir.as_str();
        display_output_dir(path)
    }

    pub fn language(&self) -> Language {
        self.config.language.resolve()
    }

    pub fn language_selection(&self) -> LanguageSelection {
        self.config.language
    }

    pub fn font_content_revision(&self) -> u64 {
        self.font_content_revision
    }

    fn mark_font_content_changed(&mut self) {
        self.font_content_revision = self.font_content_revision.wrapping_add(1);
    }

    pub fn language_selection_display_name(&self) -> String {
        match self.language_selection() {
            LanguageSelection::Auto => format!(
                "{} ({})",
                LanguageSelection::Auto.native_name(),
                self.language().native_name()
            ),
            language => language.native_name().to_owned(),
        }
    }

    pub fn ui_i18n_text_for_key(&self, key: &'static str) -> &'static str {
        i18n::text(self.language(), key)
    }

    pub fn ui_i18n_text_with_replacements(
        &self,
        key: &'static str,
        replacements: &[(&str, &str)],
    ) -> String {
        i18n::format_text(self.language(), key, replacements)
    }

    pub fn localize_message(&self, value: &str) -> String {
        i18n::localize_message(self.language(), value)
    }

    pub fn set_language_selection(&mut self, language: LanguageSelection) {
        if self.config.language == language {
            return;
        }
        self.config.language = language;
        let _ = self.config.save();
    }

    pub fn open_options_detail_page(&mut self, page: OptionsDetailPage) {
        self.options_detail_page = Some(page);
    }

    pub fn close_options_detail_page(&mut self) {
        self.options_detail_page = None;
    }

    pub fn open_prepare_detail_page(&mut self, page: PrepareDetailPage) {
        self.prepare_detail_page = Some(page);
    }

    pub fn close_prepare_detail_page(&mut self) {
        self.prepare_detail_page = None;
    }

    pub fn open_advance_detail_page(&mut self, page: AdvanceDetailPage) {
        self.advance_detail_page = Some(page);
    }

    pub fn close_advance_detail_page(&mut self) {
        self.advance_detail_page = None;
    }

    pub fn set_last_action_message(&mut self, message: impl Into<String>) {
        self.last_action = message.into();
    }

    pub fn set_windows_toast_enabled(&mut self, enabled: bool) {
        self.config.windows_toast_enabled = enabled;
        let _ = self.config.save();
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        if self.config.theme_mode == mode {
            return;
        }
        self.config.theme_mode = mode;
        let _ = self.config.save();
    }

    pub fn set_theme_accent_color(&mut self, color: ThemeAccentColor) {
        if self.config.theme_accent_color == color {
            return;
        }
        self.config.theme_accent_color = color;
        let _ = self.config.save();
    }

    pub fn set_show_log_tab(&mut self, enabled: bool) {
        self.config.show_log_tab = enabled;
        if !enabled && self.active_tab == AppTab::Log {
            self.active_tab = AppTab::Options;
        }
        let _ = self.config.save();
    }

    pub fn set_transcode_intent(
        &mut self,
        settings: crate::infrastructure::TranscodeIntentSettings,
    ) {
        let settings = settings.normalized();
        if self.config.transcode_intent == settings {
            return;
        }
        self.config.transcode_intent = settings;
        let _ = self.config.save();
    }

    fn send_download_result_windows_toast(&self, title: String, result: Result<String, String>) {
        if !self.config.windows_toast_enabled {
            return;
        }
        let language = self.language();

        thread::spawn(move || {
            let result = match result {
                Ok(output_path) => send_download_finished_windows_toast(
                    language,
                    title.as_str(),
                    (!output_path.trim().is_empty()).then_some(output_path.as_str()),
                ),
                Err(error) => {
                    if error == DOWNLOAD_CANCELLED_MESSAGE {
                        Ok(())
                    } else {
                        send_download_failed_windows_toast(language, title.as_str(), error.as_str())
                    }
                }
            };

            if let Err(error) = result {
                eprintln!("[notification] Windows Toast failed: {error}");
            }
        });
    }

    pub fn set_output_dir(&mut self, path: impl Into<String>) {
        if self.output_dir_locked_by_config() {
            return;
        }
        let path = path.into();
        self.item_defaults.output_dir = path.clone();
        for item in &mut self.queue_items {
            item.selection.output_dir = path.clone();
        }
        self.config.set_download_dir(path);
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn output_dir_locked_by_config(&self) -> bool {
        self.tool_paths.effective_config_owns_output()
    }

    pub fn output_dir_config_source_display(&self) -> Option<String> {
        self.tool_paths
            .effective_config_path()
            .map(|path| path.display().to_string())
    }

    pub fn available_cache_location_modes(&self) -> [CacheLocationMode; 3] {
        [
            CacheLocationMode::YtDlpDefault,
            CacheLocationMode::V2Cache,
            CacheLocationMode::WindowsTemp,
        ]
    }

    pub fn set_cache_location_mode(&mut self, mode: CacheLocationMode) {
        self.tool_paths.cache_mode = mode;
        self.config.cache_location_mode = match mode {
            CacheLocationMode::YtDlpDefault => SerializableCacheLocationMode::YtDlpDefault,
            CacheLocationMode::V2Cache => SerializableCacheLocationMode::V2Cache,
            CacheLocationMode::WindowsTemp => SerializableCacheLocationMode::WindowsTemp,
        };
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn should_show_prepare_tab(&self) -> bool {
        !self.config.prepare_skipped
            && !self.prepare_tab_snoozed
            && self.prepare_report.should_show_tab()
    }

    pub fn prepare_requirements(&self) -> &[PrepareRequirement] {
        &self.prepare_report.requirements
    }

    pub fn refresh_prepare_report(&mut self) {
        self.prepare_report =
            collect_prepare_report(&self.tool_paths, &self.item_defaults.output_dir);
        if !self.should_show_prepare_tab() && self.active_tab == AppTab::Prepare {
            self.active_tab = AppTab::Main;
        }
    }

    fn sanitize_startup_prepare_component_update_snapshot(&mut self) {
        self.component_update_snapshot.running = false;

        for tool in self.prepare_install_order() {
            let id = ManagedComponentId::for_dependency_tool(tool);
            let Some(status) = self
                .component_update_snapshot
                .entry(id)
                .map(|entry| entry.status)
            else {
                continue;
            };
            if !matches!(
                status,
                ComponentUpdateStatus::Checking
                    | ComponentUpdateStatus::Downloading
                    | ComponentUpdateStatus::Staged
                    | ComponentUpdateStatus::Applying
                    | ComponentUpdateStatus::Failed
            ) {
                continue;
            }

            let installed = dependency_tool_is_available(tool, self.dependency_tool_path(tool));
            let entry = self.component_update_snapshot.ensure_entry_mut(id);
            entry.status = if installed {
                ComponentUpdateStatus::Unknown
            } else {
                ComponentUpdateStatus::Missing
            };
            entry.progress = None;
            entry.message = if installed {
                "not checked".to_owned()
            } else {
                "not installed".to_owned()
            };
        }
    }

    pub fn prepare_installable_tool_count(&self) -> usize {
        self.prepare_tools_to_install_all().len()
    }

    pub fn prepare_dependency_install_block_reason(&self) -> Option<String> {
        let blocking_issue = self.prepare_report.requirements.iter().find(|item| {
            item.action.is_none()
                && item.status == PrepareStatus::Failed
                && matches!(
                    item.id.as_str(),
                    "app-root" | "config-file" | "tools-dir" | "manifest-temp"
                )
        })?;

        Some(i18n::format_fixed_english(
            "Handle {items} before installing dependency tools.",
            &[("{items}", blocking_issue.title.as_str())],
        ))
    }

    fn component_update_block_reason(&self, target: Option<ManagedComponentId>) -> Option<String> {
        let needs_tools_dir = !matches!(target, Some(ManagedComponentId::App));
        let blocking_issue = self.prepare_report.requirements.iter().find(|item| {
            item.action.is_none()
                && item.status == PrepareStatus::Failed
                && (matches!(
                    item.id.as_str(),
                    "app-root" | "config-file" | "manifest-temp"
                ) || (needs_tools_dir && item.id == "tools-dir"))
        })?;

        Some(i18n::format_fixed_english(
            "Handle {items} before updating managed components.",
            &[("{items}", blocking_issue.title.as_str())],
        ))
    }

    pub fn prepare_footer_status_text(&self) -> Option<String> {
        if self.component_update_snapshot.running {
            return Some(self.prepare_component_update_status_summary_text());
        }

        let message = self.last_action.trim();
        if message.is_empty() {
            return None;
        }

        Some(match message {
            "checking updates" => self
                .ui_i18n_text_for_key("about.status.checking")
                .to_owned(),
            "updating managed components" => self.ui_i18n_text_for_key("about.running").to_owned(),
            "update check complete" => self
                .ui_i18n_text_for_key("tool_install.stage.completed")
                .to_owned(),
            _ if message.starts_with("updating ") => {
                self.ui_i18n_text_for_key("about.running").to_owned()
            }
            _ => self.localize_message(message),
        })
    }

    fn prepare_component_update_status_summary_text(&self) -> String {
        for status in [
            ComponentUpdateStatus::Applying,
            ComponentUpdateStatus::Downloading,
            ComponentUpdateStatus::Staged,
            ComponentUpdateStatus::Checking,
            ComponentUpdateStatus::UpdateAvailable,
            ComponentUpdateStatus::Missing,
        ] {
            if let Some(text) = self.prepare_first_component_status_text(status) {
                return text;
            }
        }

        self.ui_i18n_text_for_key("about.running").to_owned()
    }

    fn prepare_first_component_status_text(&self, status: ComponentUpdateStatus) -> Option<String> {
        for tool in self.prepare_install_order() {
            let entry = self
                .component_update_snapshot
                .entry(ManagedComponentId::for_dependency_tool(tool))?;
            if entry.status != status {
                continue;
            }

            let status_text = self.component_update_status_text(entry);
            return Some(format!("{}: {status_text}", tool.label()));
        }

        None
    }

    pub fn install_all_prepare_tools(&mut self) {
        if self.component_update_running() {
            self.last_action = "Update check is already running.".to_owned();
            return;
        }

        self.refresh_prepare_report();
        if let Some(reason) = self.prepare_dependency_install_block_reason() {
            self.last_action = reason;
            return;
        }

        let tools = self.prepare_tools_to_install_all();
        if tools.is_empty() {
            self.last_action = "There are no tools to install.".to_owned();
            return;
        }

        self.update_dependency_tools_for_prepare(tools);
    }

    pub fn snooze_prepare_tab(&mut self) {
        let previous_prepare_skipped = self.config.prepare_skipped;
        self.config.prepare_skipped = true;

        match self.config.save() {
            Ok(()) => {
                self.prepare_tab_snoozed = true;
                if self.active_tab == AppTab::Prepare {
                    self.active_tab = AppTab::Main;
                }
                self.last_action =
                    "Prepare page skipped. You can handle dependency deployment later in Options."
                        .to_owned();
            }
            Err(error) => {
                self.config.prepare_skipped = previous_prepare_skipped;
                self.prepare_tab_snoozed = false;
                let localized_error = self.localize_message(&error);
                self.last_action = i18n::format_fixed_english(
                    "Skip failed: {error}",
                    &[("{error}", localized_error.as_str())],
                );
                self.refresh_prepare_report();
            }
        }
    }

    pub fn reopen_prepare_tab(&mut self) {
        self.prepare_tab_snoozed = false;
        self.config.prepare_skipped = false;
        let _ = self.config.save();
        self.refresh_prepare_report();
        if self.should_show_prepare_tab() {
            self.active_tab = AppTab::Prepare;
        }
    }

    fn prepare_tools_to_install_all(&self) -> Vec<DependencyTool> {
        self.prepare_install_order()
            .into_iter()
            .filter(|tool| self.prepare_tool_needs_install(*tool))
            .collect()
    }

    fn prepare_tool_needs_install(&self, tool: DependencyTool) -> bool {
        self.prepare_report
            .requirements
            .iter()
            .any(|item| item.needs_attention() && item.has_install_action(tool))
    }

    fn prepare_install_order(&self) -> [DependencyTool; 3] {
        [
            DependencyTool::YtDlp,
            DependencyTool::Deno,
            DependencyTool::Ffmpeg,
        ]
    }

    pub fn select_about_detail(&mut self, target: AboutDetailTarget) {
        self.about_detail_target = target;
        match target {
            AboutDetailTarget::App => {
                self.component_update_snapshot.selected = Some(ManagedComponentId::App)
            }
            AboutDetailTarget::Tool(id) => self.component_update_snapshot.selected = Some(id),
        }
    }

    pub fn component_update_running(&self) -> bool {
        self.component_update_snapshot.running
    }

    pub fn component_update_attention_signal_visible(&self) -> bool {
        self.component_update_snapshot
            .entries
            .iter()
            .any(|entry| component_update_status_needs_attention_signal(entry.status))
    }

    pub fn check_component_updates(&mut self) {
        if self.component_update_running() {
            self.last_action = "Update check is already running.".to_owned();
            return;
        }
        let proxy_url = self.tool_paths.effective_proxy_url().map(str::to_owned);
        self.component_update_snapshot.running = true;
        self.component_update_snapshot.message = "checking updates".to_owned();
        run_component_update_worker(
            ComponentUpdateAction::CheckAll,
            proxy_url,
            self.component_update_result_tx.clone(),
        );
    }

    pub fn update_all_managed_components(&mut self) {
        if self.component_update_running() {
            self.last_action = "Update check is already running.".to_owned();
            return;
        }
        self.refresh_prepare_report();
        if let Some(reason) = self.component_update_block_reason(None) {
            self.last_action = reason;
            return;
        }
        let proxy_url = self.tool_paths.effective_proxy_url().map(str::to_owned);
        self.component_update_snapshot.running = true;
        self.component_update_snapshot.message = "updating managed components".to_owned();
        run_component_update_worker(
            ComponentUpdateAction::UpdateAllManaged,
            proxy_url,
            self.component_update_result_tx.clone(),
        );
    }

    fn update_dependency_tools_for_prepare(&mut self, tools: Vec<DependencyTool>) {
        let ids = tools
            .into_iter()
            .map(ManagedComponentId::for_dependency_tool)
            .collect::<Vec<_>>();
        if ids.is_empty() {
            self.last_action = "There are no tools to install.".to_owned();
            return;
        }

        let proxy_url = self.tool_paths.effective_proxy_url().map(str::to_owned);
        self.component_update_snapshot.running = true;
        self.component_update_snapshot.selected = ids.first().copied();
        self.component_update_snapshot.message = format!(
            "updating {}",
            ids.iter()
                .map(|id| id.label())
                .collect::<Vec<_>>()
                .join(", ")
        );
        for id in ids.iter().copied() {
            let entry = self.component_update_snapshot.ensure_entry_mut(id);
            entry.status = ComponentUpdateStatus::Checking;
            entry.progress = None;
            entry.message = "queued".to_owned();
        }
        run_component_update_worker(
            ComponentUpdateAction::UpdateMany(ids),
            proxy_url,
            self.component_update_result_tx.clone(),
        );
    }

    pub fn update_component(&mut self, id: ManagedComponentId) {
        if self.component_update_running() {
            self.last_action = "Update check is already running.".to_owned();
            return;
        }
        self.refresh_prepare_report();
        if let Some(reason) = self.component_update_block_reason(Some(id)) {
            self.last_action = reason;
            return;
        }
        let proxy_url = self.tool_paths.effective_proxy_url().map(str::to_owned);
        self.component_update_snapshot.running = true;
        self.component_update_snapshot.selected = Some(id);
        self.component_update_snapshot.message = format!("updating {}", id.label());
        run_component_update_worker(
            ComponentUpdateAction::UpdateOne(id),
            proxy_url,
            self.component_update_result_tx.clone(),
        );
    }

    pub fn restart_to_apply_app_update(&mut self) -> Result<(), String> {
        launch_pending_app_update(true)
    }

    pub fn app_update_pending_restart(&self) -> bool {
        self.component_update_snapshot
            .entry(ManagedComponentId::App)
            .is_some_and(|entry| entry.status == ComponentUpdateStatus::PendingRestart)
    }

    pub fn set_yt_dlp_path(&mut self, path: impl Into<String>) {
        self.config.set_yt_dlp_path(path);
        self.tool_paths.yt_dlp = self.config.yt_dlp_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn set_yt_dlp_config_path(&mut self, path: impl Into<String>) {
        self.config.set_yt_dlp_config_path(path);
        self.tool_paths.yt_dlp_config = self.config.yt_dlp_config_path.clone();
        let _ = self.config.save();
    }

    pub fn available_yt_dlp_config_files(&self) -> Vec<ConfigFileOption> {
        available_yt_dlp_config_files()
    }

    pub fn yt_dlp_configs_dir_display(&self) -> String {
        yt_dlp_configs_dir_display()
    }

    pub fn set_ffmpeg_path(&mut self, path: impl Into<String>) {
        self.config.set_ffmpeg_path(path);
        self.tool_paths.ffmpeg = self.config.ffmpeg_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn set_aria2c_path(&mut self, path: impl Into<String>) {
        self.config.set_aria2c_path(path);
        self.tool_paths.aria2c = self.config.aria2c_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn set_deno_path(&mut self, path: impl Into<String>) {
        self.config.set_deno_path(path);
        self.tool_paths.deno = self.config.deno_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    fn sync_available_managed_tool_paths_from_update_snapshot(&mut self) {
        let available_tools = self
            .component_update_snapshot
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry.status,
                    ComponentUpdateStatus::Installed | ComponentUpdateStatus::UpToDate
                )
            })
            .filter_map(|entry| entry.id.as_dependency_tool())
            .collect::<Vec<_>>();
        if available_tools.is_empty() {
            return;
        }

        let mut changed = false;
        for tool in available_tools {
            let current_path = self.dependency_tool_path(tool).to_owned();
            if !current_path.trim().is_empty()
                && dependency_tool_is_available(tool, current_path.as_str())
            {
                continue;
            }
            let managed_path = tool.default_portable_path().to_owned();
            if !dependency_tool_is_available(tool, managed_path.as_str()) {
                continue;
            }
            changed |= self.set_dependency_tool_path_without_refresh(tool, managed_path);
        }

        if changed {
            let _ = self.config.save();
        }
    }

    pub fn install_dependency_tool(&mut self, tool: DependencyTool) {
        if self.active_tab == AppTab::Prepare {
            if let Some(reason) = self.prepare_dependency_install_block_reason() {
                self.last_action = reason;
                return;
            }
        }
        self.update_component(ManagedComponentId::for_dependency_tool(tool));
    }

    pub fn dependency_tool_update_is_running(&self, tool: DependencyTool) -> bool {
        let id = ManagedComponentId::for_dependency_tool(tool);
        self.component_update_snapshot.running
            && self
                .component_update_snapshot
                .entry(id)
                .is_some_and(|entry| {
                    matches!(
                        entry.status,
                        ComponentUpdateStatus::Checking
                            | ComponentUpdateStatus::Downloading
                            | ComponentUpdateStatus::Staged
                            | ComponentUpdateStatus::Applying
                    )
                })
    }

    pub fn dependency_tool_update_status(
        &self,
        tool: DependencyTool,
    ) -> Option<ComponentUpdateStatus> {
        self.visible_prepare_dependency_update_entry(tool)
            .map(|entry| entry.status)
    }

    pub fn dependency_tool_update_status_text(&self, tool: DependencyTool) -> Option<String> {
        self.visible_prepare_dependency_update_entry(tool)
            .map(|entry| self.component_update_status_text(entry))
    }

    fn visible_prepare_dependency_update_entry(
        &self,
        tool: DependencyTool,
    ) -> Option<&ComponentUpdateEntry> {
        let id = ManagedComponentId::for_dependency_tool(tool);
        let entry = self.component_update_snapshot.entry(id)?;
        prepare_dependency_update_status_is_visible(
            entry.status,
            self.component_update_snapshot.running,
            self.dependency_tool_update_is_running(tool),
            self.dependency_tool_is_installed(tool),
        )
        .then_some(entry)
    }

    fn component_update_status_text(&self, entry: &ComponentUpdateEntry) -> String {
        match entry.status {
            ComponentUpdateStatus::Unknown => self.ui_i18n_text_for_key("about.status.unknown"),
            ComponentUpdateStatus::Checking => self.ui_i18n_text_for_key("about.status.checking"),
            ComponentUpdateStatus::UpToDate => self.ui_i18n_text_for_key("about.status.up_to_date"),
            ComponentUpdateStatus::UpdateAvailable => {
                self.ui_i18n_text_for_key("about.status.update_available")
            }
            ComponentUpdateStatus::Missing => self.ui_i18n_text_for_key("about.status.missing"),
            ComponentUpdateStatus::Downloading => {
                let text = if let Some(percent) = entry.progress {
                    let percent = percent.to_string();
                    self.ui_i18n_text_with_replacements(
                        "about.status.downloading_percent",
                        &[("{percent}", percent.as_str())],
                    )
                } else {
                    self.ui_i18n_text_for_key("about.status.downloading")
                        .to_owned()
                };
                return self.component_update_status_size_text(text, entry.total_size_bytes);
            }
            ComponentUpdateStatus::Staged => self.ui_i18n_text_for_key("about.status.staged"),
            ComponentUpdateStatus::PendingRestart => {
                self.ui_i18n_text_for_key("about.status.pending_restart")
            }
            ComponentUpdateStatus::Applying if !entry.message.trim().is_empty() => {
                let text = self.localize_message(&entry.message);
                return if let Some(percent) = entry.progress {
                    format!("{text} {percent}%")
                } else {
                    text
                };
            }
            ComponentUpdateStatus::Applying => self.ui_i18n_text_for_key("about.status.applying"),
            ComponentUpdateStatus::Installed => self.ui_i18n_text_for_key("about.status.installed"),
            ComponentUpdateStatus::Skipped => self.ui_i18n_text_for_key("about.status.skipped"),
            ComponentUpdateStatus::Failed => self.ui_i18n_text_for_key("about.status.failed"),
        }
        .to_owned()
    }

    fn component_update_status_size_text(&self, text: String, size_bytes: Option<u64>) -> String {
        match size_bytes {
            Some(size_bytes) => format!("{text} ({})", format_byte_size(size_bytes)),
            None => text,
        }
    }

    pub fn dependency_tool_path(&self, tool: DependencyTool) -> &str {
        match tool {
            DependencyTool::YtDlp => &self.tool_paths.yt_dlp,
            DependencyTool::Ffmpeg => &self.tool_paths.ffmpeg,
            DependencyTool::Aria2c => &self.tool_paths.aria2c,
            DependencyTool::Deno => &self.tool_paths.deno,
        }
    }

    pub fn dependency_tool_is_installed(&self, tool: DependencyTool) -> bool {
        dependency_tool_is_available(tool, self.dependency_tool_path(tool))
    }

    pub fn auto_detect_dependency_tool_path(&mut self, tool: DependencyTool) {
        match detect_dependency_tool_in_system_path(tool) {
            Some(path) => {
                let display_path = path.display().to_string();
                self.set_dependency_tool_path(tool, display_path.clone());
                self.last_action = i18n::format_fixed_english(
                    "{tool} detected from PATH: {path}",
                    &[("{tool}", tool.label()), ("{path}", display_path.as_str())],
                );
            }
            None => {
                self.last_action = i18n::format_fixed_english(
                    "{tool} was not found in system PATH.",
                    &[("{tool}", tool.label())],
                );
            }
        }
    }

    pub fn auto_detect_dependency_tool_paths(&mut self) {
        const TOOLS: [DependencyTool; 4] = [
            DependencyTool::YtDlp,
            DependencyTool::Deno,
            DependencyTool::Ffmpeg,
            DependencyTool::Aria2c,
        ];

        let mut detected = Vec::new();
        let mut missing = Vec::new();

        for tool in TOOLS {
            match detect_dependency_tool_in_system_path(tool) {
                Some(path) => {
                    let display_path = path.display().to_string();
                    self.set_dependency_tool_path(tool, display_path.clone());
                    detected.push(format!("{}: {}", tool.label(), display_path));
                }
                None => missing.push(tool.label()),
            }
        }

        if detected.is_empty() {
            self.last_action = "No dependency tools were found in system PATH.".to_owned();
            return;
        }

        let found_count = detected.len().to_string();
        let total_count = TOOLS.len().to_string();
        let mut message = i18n::format_fixed_english(
            "Detected {found}/{total} tools from PATH.",
            &[
                ("{found}", found_count.as_str()),
                ("{total}", total_count.as_str()),
            ],
        );
        message.push_str("\n");
        message.push_str(&detected.join("\n"));
        if !missing.is_empty() {
            message.push_str("\n");
            message.push_str(&i18n::format_fixed_english(
                "Not found in PATH: {tools}.",
                &[("{tools}", missing.join(", ").as_str())],
            ));
        }
        self.last_action = message;
    }

    fn set_dependency_tool_path(&mut self, tool: DependencyTool, path: String) {
        self.set_dependency_tool_path_without_refresh(tool, path);
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    fn set_dependency_tool_path_without_refresh(
        &mut self,
        tool: DependencyTool,
        path: String,
    ) -> bool {
        match tool {
            DependencyTool::YtDlp => {
                let before = self.config.yt_dlp_path.clone();
                self.config.set_yt_dlp_path(path);
                self.tool_paths.yt_dlp = self.config.yt_dlp_path.clone();
                before != self.config.yt_dlp_path
            }
            DependencyTool::Ffmpeg => {
                let before = self.config.ffmpeg_path.clone();
                self.config.set_ffmpeg_path(path);
                self.tool_paths.ffmpeg = self.config.ffmpeg_path.clone();
                before != self.config.ffmpeg_path
            }
            DependencyTool::Aria2c => {
                let before = self.config.aria2c_path.clone();
                self.config.set_aria2c_path(path);
                self.tool_paths.aria2c = self.config.aria2c_path.clone();
                before != self.config.aria2c_path
            }
            DependencyTool::Deno => {
                let before = self.config.deno_path.clone();
                self.config.set_deno_path(path);
                self.tool_paths.deno = self.config.deno_path.clone();
                before != self.config.deno_path
            }
        }
    }

    pub fn dependency_tool_status_text(&self, tool: DependencyTool) -> String {
        if let Some(status) = self.dependency_tool_update_status_text(tool) {
            return status;
        }
        if self.dependency_tool_is_installed(tool) {
            "Found".to_owned()
        } else {
            "Not found".to_owned()
        }
    }

    pub fn set_proxy_enabled(&mut self, enabled: bool) {
        self.config.proxy_enabled = enabled;
        self.tool_paths.proxy_enabled = enabled;
        let _ = self.config.save();
    }

    pub fn set_proxy_url(&mut self, value: impl Into<String>) {
        self.config.set_proxy_url(value);
        self.tool_paths.proxy_url = self.config.proxy_url.clone();
        self.tool_paths.proxy_enabled = self.config.proxy_enabled;
        let _ = self.config.save();
    }

    pub fn set_no_check_certificates(&mut self, enabled: bool) {
        self.config.no_check_certificates = enabled;
        self.tool_paths.no_check_certificates = enabled;
        let _ = self.config.save();
    }

    pub fn set_limit_rate(&mut self, value: impl Into<String>) {
        self.config.set_limit_rate(value);
        self.tool_paths.limit_rate = self.config.limit_rate.clone();
        let _ = self.config.save();
    }

    pub fn set_download_sections(&mut self, value: impl Into<String>) {
        self.config.set_download_sections(value);
        self.tool_paths.download_sections = self.config.download_sections.clone();
        let _ = self.config.save();
    }

    pub fn set_file_time_mode(&mut self, mode: FileTimeMode) {
        self.config.file_time_mode = mode;
        self.tool_paths.file_time_mode = mode;
        let _ = self.config.save();
    }

    pub fn set_auto_analyze(&mut self, enabled: bool) {
        self.config.auto_analyze = enabled;
        let _ = self.config.save();
    }

    pub fn set_keep_window_on_top(&mut self, enabled: bool) {
        self.config.keep_window_on_top = enabled;
        let _ = self.config.save();
    }

    pub fn pending_ui_scale_percent(&self) -> u16 {
        self.pending_ui_scale_percent
    }

    pub fn set_pending_ui_scale_percent(&mut self, value: u16) {
        self.pending_ui_scale_percent = normalize_ui_scale_percent(value);
    }

    pub fn ui_scale_has_pending_change(&self) -> bool {
        self.pending_ui_scale_percent != self.config.ui_scale_percent
    }

    pub fn apply_pending_ui_scale_percent(&mut self) {
        self.config.ui_scale_percent = self.pending_ui_scale_percent;
        let _ = self.config.save();
    }

    pub fn set_ui_scale_percent(&mut self, value: u16) {
        let normalized = normalize_ui_scale_percent(value);
        self.pending_ui_scale_percent = normalized;
        self.config.ui_scale_percent = normalized;
        let _ = self.config.save();
    }

    pub fn set_remember_window_position(&mut self, enabled: bool) {
        self.config.remember_window_position = enabled;
        if !enabled {
            self.config.window_position = None;
        }
        let _ = self.config.save();
    }

    pub fn set_remember_window_size(&mut self, enabled: bool) {
        self.config.remember_window_size = enabled;
        if !enabled {
            self.config.window_size = None;
        }
        let _ = self.config.save();
    }

    pub fn sync_window_state(&mut self, ctx: &eframe::egui::Context) {
        if !self.config.remember_window_position && !self.config.remember_window_size {
            return;
        }

        let viewport = ctx.input(|input| input.viewport().clone());
        if viewport.minimized.unwrap_or(false) || viewport.maximized.unwrap_or(false) {
            return;
        }

        let mut changed = false;
        if self.config.remember_window_position {
            if let Some(outer_rect) = viewport.outer_rect {
                if let Some(position) = WindowPosition::new(outer_rect.min.x, outer_rect.min.y) {
                    if self.config.window_position != Some(position) {
                        self.config.window_position = Some(position);
                        changed = true;
                    }
                }
            }
        }

        if self.config.remember_window_size {
            if let Some(inner_rect) = viewport.inner_rect {
                let size = inner_rect.size();
                if let Some(window_size) = WindowSize::new(size.x, size.y) {
                    if self.config.window_size != Some(window_size) {
                        self.config.window_size = Some(window_size);
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            return;
        }

        let _ = self.config.save();
    }

    pub fn set_batch_limit_enabled(&mut self, enabled: bool) {
        self.config.batch_limit_enabled = enabled;
        let _ = self.config.save();
    }

    pub fn set_direct_download_on_add(&mut self, enabled: bool) {
        self.config.direct_download_on_add = enabled;
        let _ = self.config.save();
    }

    pub fn set_output_file_action_mode(&mut self, mode: OutputFileActionMode) {
        self.config.output_file_action_mode = mode;
        let _ = self.config.save();
    }

    pub fn set_batch_limit_count(&mut self, count: usize) {
        self.config.batch_limit_count = count.max(1);
        let _ = self.config.save();
    }

    pub fn set_monitor_clipboard(&mut self, enabled: bool) {
        self.monitor_clipboard = enabled;
        self.config.auto_paste_clipboard = enabled;
        let _ = self.config.save();
        if enabled {
            self.last_clipboard_text = read_clipboard_text().unwrap_or_default();
            self.last_clipboard_check = Some(Instant::now());
            self.clipboard_monitor_baseline_ready = true;
            self.last_action = if self.config.clipboard_auto_add {
                "Clipboard monitor enabled; the next YouTube URL change will be added immediately."
                    .to_owned()
            } else {
                "Clipboard monitor enabled; the next YouTube URL change will fill the URL field."
                    .to_owned()
            };
        } else {
            self.clipboard_monitor_baseline_ready = false;
            self.last_action = "Clipboard monitor disabled.".to_owned();
        }
    }

    pub fn set_clipboard_auto_add(&mut self, enabled: bool) {
        self.config.clipboard_auto_add = enabled;
        let _ = self.config.save();
        if self.monitor_clipboard {
            self.last_clipboard_text = read_clipboard_text().unwrap_or_default();
            self.last_clipboard_check = Some(Instant::now());
            self.clipboard_monitor_baseline_ready = true;
            self.last_action = if enabled {
                "YouTube URLs will be added immediately after the clipboard changes.".to_owned()
            } else {
                "Clipboard changes will only fill the URL field.".to_owned()
            };
        }
    }

    pub fn set_youtube_high_risk_playlist_prompt(&mut self, enabled: bool) {
        self.config.youtube_high_risk_playlist_prompt = enabled;
        let _ = self.config.save();
    }

    pub fn set_youtube_video_playlist_mode(&mut self, mode: YoutubeVideoPlaylistMode) {
        self.config.youtube_video_playlist_mode = mode;
        let _ = self.config.save();
    }

    pub fn youtube_login_rescue_dialog_visible(&self) -> bool {
        self.youtube_login_rescue_phase.is_blocking_prompt()
    }

    pub fn open_youtube_login_rescue_prompt(&mut self) {
        self.youtube_login_rescue_rx = None;
        self.youtube_login_rescue_error = None;
        self.youtube_login_rescue_target_error = None;
        self.youtube_login_rescue_browser = None;
        self.youtube_login_rescue_site_name = None;
        self.youtube_login_rescue_clipboard_prefilled = false;
        self.prefill_youtube_login_rescue_target_url();
        self.open_youtube_login_rescue_prompt_with_current_target();
    }

    pub fn open_youtube_login_rescue_prompt_for_url(&mut self, target_url: String) {
        self.youtube_login_rescue_rx = None;
        self.youtube_login_rescue_error = None;
        self.youtube_login_rescue_target_error = None;
        self.youtube_login_rescue_browser = None;
        self.youtube_login_rescue_site_name = None;
        self.youtube_login_rescue_clipboard_prefilled = false;
        self.youtube_login_rescue_target_url = target_url;
        self.open_youtube_login_rescue_prompt_with_current_target();
    }

    fn open_youtube_login_rescue_prompt_with_current_target(&mut self) {
        match detect_default_youtube_login_rescue_browser() {
            Ok(Some(browser)) => {
                self.youtube_login_rescue_browser = Some(browser);
                self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::Confirm;
            }
            Ok(None) => {
                self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::NoSupportedBrowser;
            }
            Err(error) => {
                self.youtube_login_rescue_error = Some(error);
                self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::Failed;
            }
        }
    }

    fn prefill_youtube_login_rescue_target_url(&mut self) {
        if let Ok(url) = normalize_cookie_rescue_target_url(&self.url_input) {
            self.youtube_login_rescue_target_url = url;
            return;
        }

        if let Some(url) = read_clipboard_text()
            .and_then(|text| single_cookie_rescue_clipboard_url_candidate(&text))
            .and_then(|candidate| normalize_cookie_rescue_target_url(&candidate).ok())
        {
            self.youtube_login_rescue_target_url = url;
            self.youtube_login_rescue_clipboard_prefilled = true;
        }
    }

    pub fn paste_clipboard_to_youtube_login_rescue_target(&mut self) {
        match read_clipboard_text()
            .and_then(|text| single_cookie_rescue_clipboard_url_candidate(&text))
            .map(|candidate| normalize_cookie_rescue_target_url(&candidate))
        {
            Some(Ok(url)) => {
                self.youtube_login_rescue_target_url = url;
                self.youtube_login_rescue_target_error = None;
                self.youtube_login_rescue_clipboard_prefilled = true;
            }
            Some(Err(error)) => {
                self.youtube_login_rescue_target_error = Some(error);
            }
            None => {
                self.youtube_login_rescue_target_error =
                    Some("Clipboard does not contain a website URL.".to_owned());
            }
        }
    }

    pub fn set_youtube_login_rescue_target_url(&mut self, value: String) {
        self.youtube_login_rescue_target_url = value;
        self.youtube_login_rescue_target_error = None;
        self.youtube_login_rescue_clipboard_prefilled = false;
    }

    pub fn apply_youtube_login_rescue_dropped_paths(&mut self, paths: Vec<PathBuf>) {
        if let Some(url) = paths
            .iter()
            .find_map(|path| cookie_rescue_url_from_dropped_path(path))
            .and_then(|candidate| normalize_cookie_rescue_target_url(&candidate).ok())
        {
            self.youtube_login_rescue_target_url = url;
            self.youtube_login_rescue_target_error = None;
            self.youtube_login_rescue_clipboard_prefilled = false;
        }
    }

    fn cookie_rescue_profile_root_path(&self) -> PathBuf {
        self.app_cache_root_path()
            .join("temp")
            .join("cookie-rescue")
    }

    pub fn start_youtube_login_rescue(&mut self) {
        if self.youtube_login_rescue_rx.is_some() {
            return;
        }
        let Some(browser) = self.youtube_login_rescue_browser.clone() else {
            self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::NoSupportedBrowser;
            return;
        };

        let target_url =
            match normalize_cookie_rescue_target_url(&self.youtube_login_rescue_target_url) {
                Ok(url) => url,
                Err(error) => {
                    self.youtube_login_rescue_target_error = Some(error);
                    return;
                }
            };
        self.youtube_login_rescue_target_url = target_url.clone();
        self.youtube_login_rescue_target_error = None;

        let cookie_dir_path = cookie_rescue_cookie_dir_path();
        let profile_root_path = self.cookie_rescue_profile_root_path();
        let (tx, rx) = mpsc::channel();
        self.youtube_login_rescue_rx = Some(rx);
        self.youtube_login_rescue_error = None;
        self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::Starting;
        self.last_action = format!(
            "Opening {} for Cookie Rescue: {}",
            browser.display_name, target_url
        );

        thread::spawn(move || {
            run_youtube_login_rescue_cookie_export(
                browser,
                target_url,
                cookie_dir_path,
                profile_root_path,
                tx,
            );
        });
    }

    pub fn cancel_youtube_login_rescue_prompt(&mut self) {
        if self.youtube_login_rescue_rx.is_some() {
            return;
        }
        self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::Idle;
        self.youtube_login_rescue_browser = None;
        self.youtube_login_rescue_site_name = None;
        self.youtube_login_rescue_target_error = None;
        self.youtube_login_rescue_clipboard_prefilled = false;
        self.youtube_login_rescue_error = None;
    }

    pub fn close_youtube_login_rescue_browser(&mut self) {
        self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::Closed;
        self.youtube_login_rescue_site_name = None;
        self.youtube_login_rescue_target_error = None;
        self.youtube_login_rescue_clipboard_prefilled = false;
        self.youtube_login_rescue_error = None;
        self.youtube_login_rescue_rx = None;
        self.last_action = "Cookie Rescue closed.".to_owned();
    }

    pub fn retry_youtube_login_rescue_detection(&mut self) {
        self.open_youtube_login_rescue_prompt();
    }

    pub fn youtube_login_rescue_is_starting(&self) -> bool {
        self.youtube_login_rescue_rx.is_some()
    }

    fn poll_youtube_login_rescue(&mut self) {
        let Some(rx) = self.youtube_login_rescue_rx.take() else {
            return;
        };

        let mut keep_rx = true;
        loop {
            match rx.try_recv() {
                Ok(YoutubeLoginRescueEvent::CdpReady(browser)) => {
                    let browser_name = browser.display_name.clone();
                    self.youtube_login_rescue_browser = Some(browser);
                    self.youtube_login_rescue_error = None;
                    self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::WaitingForCookie;
                    self.last_action = format!(
                        "{browser_name} Cookie Rescue window is connected. Waiting for website cookies..."
                    );
                }
                Ok(YoutubeLoginRescueEvent::CookieExported(export)) => {
                    self.youtube_login_rescue_browser = Some(export.browser.clone());
                    self.youtube_login_rescue_site_name = Some(export.site_display_name.clone());
                    self.youtube_login_rescue_error = None;
                    self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::CookieExported;
                    self.set_use_browser_cookies(true);
                    self.set_browser_cookie_source("auto");
                    self.last_action = format!(
                        "{} cookies saved: {} cookies ({} login-like cookies).",
                        export.site_display_name,
                        export.exported_cookie_count,
                        export.auth_cookie_count
                    );
                    keep_rx = false;
                    break;
                }
                Ok(YoutubeLoginRescueEvent::Failed(error)) => {
                    self.youtube_login_rescue_error = Some(error.clone());
                    self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::Failed;
                    self.last_action = error;
                    keep_rx = false;
                    break;
                }
                Err(TryRecvError::Empty) => {
                    if matches!(
                        self.youtube_login_rescue_phase,
                        YoutubeLoginRescuePhase::Starting
                    ) {
                        self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::WaitingForCdp;
                    }
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    self.youtube_login_rescue_error =
                        Some("Cookie Rescue worker stopped before returning a result.".to_owned());
                    self.youtube_login_rescue_phase = YoutubeLoginRescuePhase::Failed;
                    keep_rx = false;
                    break;
                }
            }
        }

        if keep_rx {
            self.youtube_login_rescue_rx = Some(rx);
        }
    }

    pub fn available_browser_cookie_sources(
        &self,
    ) -> Vec<crate::infrastructure::BrowserCookieSourceOption> {
        self.tool_paths.available_browser_cookie_sources()
    }

    pub fn cookie_usage_mode(&self) -> CookieUsageMode {
        if !self.item_defaults.use_cookies {
            CookieUsageMode::Off
        } else if self.cookie_source_uses_file() || self.cookie_source_uses_auto() {
            CookieUsageMode::File
        } else {
            CookieUsageMode::Browser
        }
    }

    pub fn set_cookie_usage_mode(&mut self, mode: CookieUsageMode) {
        match mode {
            CookieUsageMode::Off => {
                self.set_use_browser_cookies(false);
            }
            CookieUsageMode::Browser => {
                self.set_use_browser_cookies(true);
                if self.cookie_source_uses_file() || self.cookie_source_uses_auto() {
                    self.set_browser_cookie_source(self.default_browser_cookie_source_value());
                }
            }
            CookieUsageMode::File => {
                self.set_use_browser_cookies(true);
                if !(self.cookie_source_uses_file() || self.cookie_source_uses_auto()) {
                    self.set_browser_cookie_source("file");
                }
            }
        }
    }

    fn default_browser_cookie_source_value(&self) -> String {
        self.available_browser_cookie_sources()
            .into_iter()
            .find(|option| option.value != "auto" && option.value != "file")
            .map(|option| option.value.to_owned())
            .unwrap_or_else(|| "chrome".to_owned())
    }

    pub fn cookie_file_source_mode(&self) -> CookieFileSourceMode {
        if self.cookie_source_uses_auto() {
            CookieFileSourceMode::AutoSelect
        } else {
            CookieFileSourceMode::Custom
        }
    }

    pub fn set_cookie_file_source_mode(&mut self, mode: CookieFileSourceMode) {
        self.set_use_browser_cookies(true);
        match mode {
            CookieFileSourceMode::Custom => self.set_browser_cookie_source("file"),
            CookieFileSourceMode::AutoSelect => self.set_browser_cookie_source("auto"),
        }
    }

    pub fn available_browser_cookie_profiles(
        &self,
    ) -> Vec<crate::infrastructure::BrowserCookieProfileOption> {
        self.tool_paths.available_browser_cookie_profiles()
    }

    pub fn set_browser_cookie_source(&mut self, source: impl Into<String>) {
        let source = source.into();
        self.tool_paths.browser_cookie_source = source.clone();
        self.config.browser_cookie_source = source;
        let profiles = self.tool_paths.available_browser_cookie_profiles();
        if self.cookie_source_uses_file()
            || self.cookie_source_uses_auto()
            || (!self.tool_paths.browser_cookie_profile.trim().is_empty()
                && !profiles
                    .iter()
                    .any(|option| option.value == self.tool_paths.browser_cookie_profile))
        {
            self.tool_paths.browser_cookie_profile.clear();
            self.config.browser_cookie_profile.clear();
        }
        let _ = self.config.save();
    }

    pub fn set_browser_cookie_profile(&mut self, profile: impl Into<String>) {
        let profile = profile.into();
        self.tool_paths.browser_cookie_profile = profile.clone();
        self.config.browser_cookie_profile = profile;
        let _ = self.config.save();
    }

    pub fn set_browser_cookie_file(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.tool_paths.browser_cookie_file = path.clone();
        self.config.browser_cookie_file = path;
        let _ = self.config.save();
    }

    pub fn cookie_source_uses_auto(&self) -> bool {
        self.tool_paths
            .browser_cookie_source
            .trim()
            .eq_ignore_ascii_case("auto")
    }

    pub fn cookie_source_uses_file(&self) -> bool {
        self.tool_paths
            .browser_cookie_source
            .trim()
            .eq_ignore_ascii_case("file")
    }

    pub fn saved_cookie_files(&self) -> Vec<SavedCookieFile> {
        let cookie_dir = cookie_rescue_cookie_dir_path();
        let mut entries = read_cookie_site_index_or_default(&cookie_dir)
            .sites
            .into_iter()
            .filter(|entry| !entry.id.trim().is_empty())
            .map(saved_cookie_file_from_index_entry)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            left.display_name
                .to_ascii_lowercase()
                .cmp(&right.display_name.to_ascii_lowercase())
        });
        entries
    }

    pub fn refresh_saved_cookie_file(&mut self, id: &str) {
        let cookie_dir = cookie_rescue_cookie_dir_path();
        let index = read_cookie_site_index_or_default(&cookie_dir);
        let Some(entry) = index.sites.iter().find(|entry| entry.id == id) else {
            self.last_action = "Cookie file entry was not found.".to_owned();
            return;
        };
        let login_url = entry.login_url.trim();
        if login_url.is_empty() {
            self.last_action = "Cookie file entry has no saved login URL.".to_owned();
            return;
        }
        self.open_youtube_login_rescue_prompt_for_url(login_url.to_owned());
    }

    pub fn delete_saved_cookie_file(&mut self, id: &str) {
        let cookie_dir = cookie_rescue_cookie_dir_path();
        let mut index = read_cookie_site_index_or_default(&cookie_dir);
        let Some(position) = index.sites.iter().position(|entry| entry.id == id) else {
            self.last_action = "Cookie file entry was not found.".to_owned();
            return;
        };
        let entry = index.sites.remove(position);
        if let Some(path) = cookie_file_path_owned_by_cookie_dir(&cookie_dir, &entry.cookie_file) {
            if path.is_file() {
                if let Err(error) = fs::remove_file(&path) {
                    self.last_action =
                        format!("Could not delete Cookie file {}: {error}", path.display());
                    return;
                }
            }
        }
        match write_cookie_site_index(&cookie_dir, &index) {
            Ok(()) => {
                self.last_action = format!(
                    "Cookie file removed: {}",
                    saved_cookie_file_from_index_entry(entry).display_name
                );
            }
            Err(error) => {
                self.last_action = error;
            }
        }
    }

    pub fn available_concurrent_fragment_values(&self) -> [usize; 4] {
        [1, 2, 4, 8]
    }

    pub fn set_concurrent_fragments(&mut self, value: usize) {
        let value = match value {
            1 | 2 | 4 | 8 => value,
            0 => 1,
            3 => 4,
            5..=7 => 8,
            _ => 8,
        };
        self.tool_paths.concurrent_fragments = value;
        self.config.concurrent_fragments = value;
        let _ = self.config.save();
    }

    pub fn set_youtube_subs_po_token(&mut self, token: impl Into<String>) {
        let token = token.into();
        self.tool_paths.youtube_subs_po_token = token.clone();
        self.config.youtube_subs_po_token = token;
        let _ = self.config.save();
    }

    pub fn set_youtube_extractor_args(&mut self, args: impl Into<String>) {
        let args = args.into();
        self.tool_paths.youtube_extractor_args = args.clone();
        self.config.youtube_extractor_args = args;
        let _ = self.config.save();
    }

    pub fn set_use_browser_cookies(&mut self, enabled: bool) {
        self.item_defaults.use_cookies = enabled;
        for item in &mut self.queue_items {
            item.selection.use_cookies = enabled;
        }
        self.config.use_browser_cookies = enabled;
        let _ = self.config.save();
    }

    pub fn set_use_aria2(&mut self, enabled: bool) {
        self.item_defaults.use_aria2 = enabled;
        for item in &mut self.queue_items {
            item.selection.use_aria2 = enabled;
        }
        self.config.use_aria2 = enabled;
        let _ = self.config.save();
    }

    pub fn set_thumbnail_post_process_mode(&mut self, mode: PostProcessMode) {
        self.item_defaults.write_thumbnail = mode.writes();
        self.item_defaults.embed_thumbnail = mode.embeds();
        for item in &mut self.queue_items {
            item.selection.write_thumbnail = mode.writes();
            item.selection.embed_thumbnail = mode.embeds();
        }
        self.config.thumbnail_mode = mode;
        let _ = self.config.save();
    }

    pub fn set_subtitle_post_process_mode(&mut self, mode: PostProcessMode) {
        self.item_defaults.write_subtitles = mode.writes();
        self.item_defaults.embed_subtitles = mode.embeds();
        for item in &mut self.queue_items {
            item.selection.write_subtitles = mode.writes();
            item.selection.embed_subtitles = mode.embeds();
        }
        self.config.subtitle_mode = mode;
        let _ = self.config.save();
    }

    pub fn set_chapter_post_process_mode(&mut self, mode: PostProcessMode) {
        self.item_defaults.write_chapters = mode.writes();
        self.item_defaults.embed_chapters = mode.embeds();
        for item in &mut self.queue_items {
            item.selection.write_chapters = mode.writes();
            item.selection.embed_chapters = mode.embeds();
        }
        self.config.chapter_mode = mode;
        let _ = self.config.save();
    }

    pub fn set_write_thumbnail(&mut self, enabled: bool) {
        let mode = if enabled {
            if self.item_defaults.embed_thumbnail {
                PostProcessMode::Embed
            } else {
                PostProcessMode::Download
            }
        } else {
            PostProcessMode::Off
        };
        self.set_thumbnail_post_process_mode(mode);
    }

    pub fn set_embed_thumbnail(&mut self, enabled: bool) {
        let mode = if enabled {
            PostProcessMode::Embed
        } else if self.item_defaults.write_thumbnail {
            PostProcessMode::Download
        } else {
            PostProcessMode::Off
        };
        self.set_thumbnail_post_process_mode(mode);
    }

    pub fn set_write_subtitles(&mut self, enabled: bool) {
        let mode = if enabled {
            if self.item_defaults.embed_subtitles {
                PostProcessMode::Embed
            } else {
                PostProcessMode::Download
            }
        } else {
            PostProcessMode::Off
        };
        self.set_subtitle_post_process_mode(mode);
    }

    pub fn set_embed_subtitles(&mut self, enabled: bool) {
        let mode = if enabled {
            PostProcessMode::Embed
        } else if self.item_defaults.write_subtitles {
            PostProcessMode::Download
        } else {
            PostProcessMode::Off
        };
        self.set_subtitle_post_process_mode(mode);
    }

    pub fn set_write_chapters(&mut self, enabled: bool) {
        let mode = if enabled {
            if self.item_defaults.embed_chapters {
                PostProcessMode::Embed
            } else {
                PostProcessMode::Download
            }
        } else {
            PostProcessMode::Off
        };
        self.set_chapter_post_process_mode(mode);
    }

    pub fn set_embed_chapters(&mut self, enabled: bool) {
        let mode = if enabled {
            PostProcessMode::Embed
        } else if self.item_defaults.write_chapters {
            PostProcessMode::Download
        } else {
            PostProcessMode::Off
        };
        self.set_chapter_post_process_mode(mode);
    }

    pub fn push_runtime_log(&mut self, message: impl Into<String>) {
        let message = message.into();
        if message.trim().is_empty() {
            return;
        }
        self.runtime_log.push_back(message);
        while self.runtime_log.len() > 20 {
            self.runtime_log.pop_front();
        }
    }

    pub fn push_tool_log_action(
        &mut self,
        mode: impl Into<String>,
        action: impl Into<String>,
    ) -> u64 {
        let id = self.next_tool_log_action_id;
        self.next_tool_log_action_id = self.next_tool_log_action_id.saturating_add(1);
        self.tool_logs.push_back(ToolLogAction {
            id,
            timestamp: current_log_timestamp(),
            status: ToolLogStatus::Running,
            mode: mode.into(),
            action: action.into(),
            steps: Vec::new(),
        });
        self.trim_tool_logs();
        id
    }

    fn mark_last_failed_tool_log_step_as_recoverable(&mut self, action_id: u64) {
        if let Some(parent) = self
            .tool_logs
            .iter_mut()
            .find(|entry| entry.id == action_id)
        {
            if let Some(step) = parent
                .steps
                .iter_mut()
                .rev()
                .find(|step| step.status == ToolLogStatus::Failed)
            {
                step.status = ToolLogStatus::Recovered;
            }
            parent.status = aggregate_tool_log_status(&parent.steps);
        }
    }

    pub fn push_tool_log_step(
        &mut self,
        action_id: u64,
        status: ToolLogStatus,
        tool: impl Into<String>,
        action: impl Into<String>,
        command: impl Into<String>,
    ) -> u64 {
        self.push_tool_log_step_internal(action_id, status, tool, action, command, None, true)
    }

    fn push_tool_log_step_with_detail_without_failure_reveal(
        &mut self,
        action_id: u64,
        status: ToolLogStatus,
        tool: impl Into<String>,
        action: impl Into<String>,
        command: impl Into<String>,
        detail: Option<String>,
    ) -> u64 {
        self.push_tool_log_step_internal(action_id, status, tool, action, command, detail, false)
    }

    fn push_tool_log_step_internal(
        &mut self,
        action_id: u64,
        status: ToolLogStatus,
        tool: impl Into<String>,
        action: impl Into<String>,
        command: impl Into<String>,
        detail: Option<String>,
        reveal_on_failure: bool,
    ) -> u64 {
        let id = self.next_tool_log_step_id;
        self.next_tool_log_step_id = self.next_tool_log_step_id.saturating_add(1);
        if let Some(parent) = self
            .tool_logs
            .iter_mut()
            .find(|entry| entry.id == action_id)
        {
            parent.steps.push(ToolLogStep {
                id,
                status,
                tool: tool.into(),
                action: action.into(),
                command: command.into(),
                detail,
            });
            parent.status = aggregate_tool_log_status(&parent.steps);
            if status == ToolLogStatus::Failed {
                self.log_viewer_expanded_action = Some(action_id);
                self.log_viewer_selected_step = Some(id);
                if reveal_on_failure {
                    self.reveal_log_tab_for_tool_failure();
                }
            }
        }
        id
    }

    pub fn workflow_tool_log_action(
        &mut self,
        workflow_id: WorkflowRunId,
        mode: impl Into<String>,
        action: impl Into<String>,
    ) -> u64 {
        if let Some(action_id) = self.tool_log_action_by_workflow.get(&workflow_id).copied() {
            if self.tool_logs.iter().any(|entry| entry.id == action_id) {
                return action_id;
            }
        }
        let action_id = self.push_tool_log_action(mode, action);
        self.tool_log_action_by_workflow
            .insert(workflow_id, action_id);
        action_id
    }

    pub fn finish_workflow_tool_log(&mut self, workflow_id: WorkflowRunId) {
        self.tool_log_action_by_workflow.remove(&workflow_id);
    }

    pub fn enter_log_tab(&mut self) {
        self.config.show_log_tab = true;
        self.active_tab = AppTab::Log;
        self.collapse_tool_log_viewer();
    }

    fn reveal_log_tab_for_tool_failure(&mut self) {
        self.enter_log_tab();
    }

    fn collapse_tool_log_viewer(&mut self) {
        self.log_viewer_expanded_action = None;
        self.log_viewer_selected_step = None;
    }

    fn trim_tool_logs(&mut self) {
        while self.tool_logs.len() > 20 {
            let removed = self.tool_logs.pop_front();
            if let Some(removed) = removed {
                if self.log_viewer_expanded_action == Some(removed.id) {
                    self.log_viewer_expanded_action = None;
                }
                if let Some(selected_step) = self.log_viewer_selected_step {
                    if removed.steps.iter().any(|step| step.id == selected_step) {
                        self.log_viewer_selected_step = None;
                    }
                }
                self.tool_log_action_by_workflow
                    .retain(|_, action_id| *action_id != removed.id);
            }
        }
    }

    pub fn set_post_download_conversion_enabled(&mut self, enabled: bool) {
        self.config.post_download_conversion_enabled = enabled;
        let _ = self.config.save();
    }

    pub fn set_enable_builtin_transcode_after_download(&mut self, enabled: bool) {
        self.set_post_download_conversion_enabled(enabled);
    }

    pub fn set_chapter_compatibility_mode(&mut self, enabled: bool) {
        self.tool_paths.chapter_compatibility_mode = enabled;
        self.config.chapter_compatibility_mode = enabled;
        let _ = self.config.save();
    }

    pub fn set_quality_preset(&mut self, preset: QualityPreset) {
        self.item_defaults.quality = preset;
        for item in &mut self.queue_items {
            item.selection.quality = preset;
        }
    }

    pub fn cache_location_display(&self) -> String {
        match self.tool_paths.cache_mode {
            CacheLocationMode::YtDlpDefault => "yt-dlp default".to_owned(),
            CacheLocationMode::V2Cache => {
                crate::infrastructure::resolve_output_dir(&self.tool_paths.cache_dir)
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|_| self.tool_paths.cache_dir.clone())
            }
            CacheLocationMode::WindowsTemp => std::env::temp_dir().display().to_string(),
        }
    }

    pub fn refresh_cache_management_summary_if_stale(&mut self) {
        if self
            .cache_management_summary_refreshed_at
            .is_some_and(|last| last.elapsed() < Duration::from_secs(2))
        {
            return;
        }
        self.refresh_cache_management_summary();
    }

    pub fn refresh_cache_management_summary(&mut self) {
        let root = self.app_cache_root_path();
        self.cache_management_summary = calculate_cache_management_summary(&root);
        self.cache_management_summary_refreshed_at = Some(Instant::now());
    }

    pub fn cache_management_usage_display(&self) -> String {
        let total = format_byte_size(self.cache_management_summary.total_bytes);
        let audio = format_byte_size(self.cache_management_summary.music_bytes);
        let expired = format_byte_size(self.cache_management_summary.expired_music_bytes);
        self.ui_i18n_text_with_replacements(
            "options.cache_usage_detail",
            &[
                ("{total}", total.as_str()),
                ("{audio}", audio.as_str()),
                ("{expired}", expired.as_str()),
            ],
        )
    }

    pub fn clear_expired_music_cache(&mut self) {
        let root = self.music_stream_cache_root();
        match remove_expired_music_cache_dirs(&root) {
            Ok(summary) => {
                self.refresh_cache_management_summary();
                let count = summary.entries.to_string();
                let size = format_byte_size(summary.bytes);
                self.last_action = i18n::format_fixed_english(
                    "Cleared {count} expired cache entries ({size}).",
                    &[("{count}", count.as_str()), ("{size}", size.as_str())],
                );
            }
            Err(error) => {
                let error = error.to_string();
                self.last_action = i18n::format_fixed_english(
                    "Cache cleanup failed: {error}",
                    &[("{error}", error.as_str())],
                );
            }
        }
    }

    pub fn clear_music_stream_cache(&mut self) {
        self.stop_music_playback();
        let root = self.music_stream_cache_root();
        match remove_path_contents_or_dir(&root) {
            Ok(summary) => {
                self.refresh_cache_management_summary();
                let count = summary.entries.to_string();
                let size = format_byte_size(summary.bytes);
                self.last_action = i18n::format_fixed_english(
                    "Cleared audio cache: {count} entries ({size}).",
                    &[("{count}", count.as_str()), ("{size}", size.as_str())],
                );
            }
            Err(error) => {
                let error = error.to_string();
                self.last_action = i18n::format_fixed_english(
                    "Cache cleanup failed: {error}",
                    &[("{error}", error.as_str())],
                );
            }
        }
    }

    pub fn clear_app_cache(&mut self) {
        self.stop_music_playback();
        let root = self.app_cache_root_path();
        match remove_safe_app_cache_contents(&root) {
            Ok(summary) => {
                self.refresh_cache_management_summary();
                let count = summary.entries.to_string();
                let size = format_byte_size(summary.bytes);
                self.last_action = i18n::format_fixed_english(
                    "Cleared app cache: {count} entries ({size}).",
                    &[("{count}", count.as_str()), ("{size}", size.as_str())],
                );
            }
            Err(error) => {
                let error = error.to_string();
                self.last_action = i18n::format_fixed_english(
                    "Cache cleanup failed: {error}",
                    &[("{error}", error.as_str())],
                );
            }
        }
    }

    fn app_cache_root_path(&self) -> PathBuf {
        crate::infrastructure::resolve_output_dir(&self.tool_paths.cache_dir)
            .unwrap_or_else(|_| PathBuf::from(&self.tool_paths.cache_dir))
    }

    fn begin_batch_add(&mut self, source: String) {
        self.begin_batch_add_with_kind(source, false);
    }

    fn begin_batch_add_with_kind(&mut self, source: String, music_compact: bool) {
        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.youtube_playlist_prompt = None;
            self.last_action = error;
            return;
        }

        self.youtube_playlist_prompt = None;
        self.is_adding_batch = true;
        self.is_cancelling_batch_add = false;
        self.batch_add_music_compact = music_compact;
        self.last_action =
            i18n::format_fixed_english("Adding: {source}", &[("{source}", source.as_str())]);

        let tool_paths = self.tool_paths.clone();
        let source_for_worker = source.clone();
        let (tx, rx) = mpsc::channel();
        self.batch_add_result_rx = Some(rx);
        let child_handle = Arc::new(Mutex::new(None));
        self.batch_add_child = Some(child_handle.clone());
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.batch_add_cancel_requested = Some(cancel_requested.clone());
        let limit = self
            .config
            .batch_limit_enabled
            .then_some(self.config.batch_limit_count.max(1));
        let untitled_task = "Untitled task".to_owned();
        let imported_template = "Imported {tail}".to_owned();
        let log_mode = if music_compact {
            "audio".to_owned()
        } else {
            self.app_mode.config_value().to_owned()
        };
        let tool_log_action_id = self.push_tool_log_action(log_mode, "batch import");

        thread::spawn(move || {
            run_batch_add_worker(
                tool_paths,
                source_for_worker,
                limit,
                untitled_task,
                imported_template,
                music_compact,
                tool_log_action_id,
                tx,
                child_handle,
                cancel_requested,
            );
        });
    }

    pub fn latest_download_status(&self) -> Option<String> {
        let item = self.queue_items.last()?;
        Some(format!(
            "{}: {} | {}",
            self.ui_i18n_text_for_key(queue_item_status_key(item)),
            item.title,
            item.last_output_path
                .as_deref()
                .or(item.last_error.as_deref())
                .unwrap_or(item.source_url.as_str())
        ))
    }

    pub fn item_status_text(&self, item_index: usize) -> &'static str {
        let Some(item) = self.queue_items.get(item_index) else {
            return "";
        };
        self.ui_i18n_text_for_key(queue_item_status_key(item))
    }

    pub fn single_mode_status_lines(&self) -> Vec<(String, String)> {
        let Some(item) = self.queue_items.first() else {
            return Vec::new();
        };

        let Some(run) = item
            .workflows
            .iter()
            .rev()
            .find(|run| single_mode_status_workflow_visible(run, item))
        else {
            return Vec::new();
        };

        let mut lines = Vec::new();
        lines.push(("Downloader".to_owned(), workflow_tool_label(&run.tool)));

        for line in run.detail.lines() {
            let Some((label, value)) = line.split_once('\t') else {
                continue;
            };
            let label = label.trim();
            let value = value.trim();
            if !label.is_empty() && !value.is_empty() {
                lines.push((label.to_owned(), value.to_owned()));
            }
        }

        if run.progress > 0.0 && run.progress < 100.0 && !status_lines_contain(&lines, "Progress") {
            lines.push((
                "Progress".to_owned(),
                format!("{:.0}%", run.progress.round()),
            ));
        }

        let status = match run.state {
            WorkflowState::Queued => self.ui_i18n_text_for_key("item.status.queued"),
            WorkflowState::Running => self.ui_i18n_text_for_key("item.status.running"),
            WorkflowState::Finished if item.last_error.is_some() => {
                self.ui_i18n_text_for_key("item.status.failed")
            }
            WorkflowState::Finished => self.ui_i18n_text_for_key("item.status.finished"),
            WorkflowState::Failed => self.ui_i18n_text_for_key("item.status.failed"),
            WorkflowState::Cancelled => self.ui_i18n_text_for_key("item.status.cancelled"),
        };
        lines.push(("Status".to_owned(), status.to_owned()));

        if let Some(error) = run
            .error
            .as_deref()
            .or(item.last_error.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            lines.push(("Error".to_owned(), error.to_owned()));
        }

        lines
    }

    pub fn item_title_text(&self, item_index: usize) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        if !item.title.trim().is_empty() {
            item.title.clone()
        } else {
            item.source_url.clone()
        }
    }

    pub fn item_title_is_loading(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };
        matches!(item.metadata_state, MetadataState::Running)
            || item
                .workflows
                .iter()
                .any(|run| run.state == WorkflowState::Running)
    }

    pub fn single_mode_analysis_running(&self) -> bool {
        if self.app_mode != AppMode::Origin {
            return false;
        }
        let Some(item) = self.queue_items.first() else {
            return false;
        };
        matches!(
            item.metadata_state,
            MetadataState::Queued | MetadataState::Running
        ) || item.workflows.iter().any(|run| {
            run.kind == WorkflowKind::AnalyzeMetadata
                && matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
        })
    }

    pub fn url_input_locked(&self) -> bool {
        self.is_adding_batch || self.youtube_playlist_prompt.is_some()
    }

    fn add_single_url_to_batch(&mut self, source: String) {
        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        let item_id = if self.app_mode == AppMode::Origin {
            if !self.active_workflows.is_empty() {
                self.last_action =
                    "Wait for the current Origin Mode item to finish first.".to_owned();
                return;
            }
            self.stop_music_playback();
            self.queue_items.clear();
            self.batch_input.clear();
            let item = self.build_queue_item_from_url(&source);
            let item_id = item.id;
            self.queue_items.push(item);
            item_id
        } else {
            self.ensure_queue_item_for_url(&source)
        };
        if self.app_mode != AppMode::Origin {
            self.url_input.clear();
        }
        let fallback_title = infer_title(&source, "Untitled task", "Imported {tail}");
        self.last_action = i18n::format_fixed_english(
            "Added to list: {title}",
            &[("{title}", fallback_title.as_str())],
        );
        self.enqueue_item_analysis(item_id, source);
    }

    pub fn item_title_visual_state(&self, item_index: usize) -> ItemTitleVisualState {
        let Some(item) = self.queue_items.get(item_index) else {
            return ItemTitleVisualState::Default;
        };

        if item.workflows.iter().any(|run| {
            matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
                && matches!(
                    run.kind,
                    WorkflowKind::DownloadMedia
                        | WorkflowKind::ExportMedia
                        | WorkflowKind::PostProcess
                )
        }) {
            return ItemTitleVisualState::Pending;
        }

        if item
            .completed_selection
            .as_ref()
            .is_some_and(|completed| selection_matches_completed(&item.selection, completed))
        {
            return ItemTitleVisualState::Completed;
        }

        if item.last_error.is_some() || matches!(item.metadata_state, MetadataState::Failed(_)) {
            return ItemTitleVisualState::Failed;
        }

        if matches!(
            item.metadata_state,
            MetadataState::Queued | MetadataState::Running
        ) {
            return ItemTitleVisualState::Pending;
        }

        if item.metadata_loaded() {
            return ItemTitleVisualState::Ready;
        }

        ItemTitleVisualState::Pending
    }

    pub fn item_error_text(&self, item_index: usize) -> Option<String> {
        let item = self.queue_items.get(item_index)?;
        if let Some(error) = item.last_error.as_ref() {
            return Some(error.clone());
        }

        if item_latest_download_state(item).is_some_and(|state| {
            matches!(
                state,
                WorkflowState::Queued | WorkflowState::Running | WorkflowState::Finished
            )
        }) {
            return None;
        }

        match &item.metadata_state {
            MetadataState::Failed(error) => Some(error.clone()),
            _ => None,
        }
    }

    pub fn item_output_file_path(&self, item_index: usize) -> Option<String> {
        let item = self.queue_items.get(item_index)?;
        if item.last_error.is_some() {
            return None;
        }
        if !item_latest_download_state(item).is_some_and(|state| state == WorkflowState::Finished) {
            return None;
        }
        item.last_output_path
            .clone()
            .filter(|path| !path.trim().is_empty())
    }

    pub fn item_progress(&self, item_index: usize, kind: FormatPickerKind) -> f32 {
        let Some(item) = self.queue_items.get(item_index) else {
            return 0.0;
        };
        let raw = if self.item_uses_muxed_video(item_index) {
            let shared = item.progress.video.max(item.progress.audio);
            match kind {
                FormatPickerKind::Video | FormatPickerKind::Audio => shared,
                FormatPickerKind::Subtitle => item.progress.subtitle,
                FormatPickerKind::Section => 0.0,
            }
        } else {
            match kind {
                FormatPickerKind::Video => item.progress.video,
                FormatPickerKind::Audio => item.progress.audio,
                FormatPickerKind::Subtitle => item.progress.subtitle,
                FormatPickerKind::Section => 0.0,
            }
        };
        raw.clamp(0.0, 100.0)
    }

    pub fn item_av_progress_visible(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        let has_active_download = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id && workflow.kind == WorkflowKind::DownloadMedia
        });
        if has_active_download {
            let video = item.progress.video;
            let audio = item.progress.audio;
            return video > 0.0 || audio > 0.0;
        }

        let has_active_export = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id && workflow.kind == WorkflowKind::ExportMedia
        });
        if has_active_export {
            let video = item.progress.video;
            let audio = item.progress.audio;
            let active_sides = [video, audio]
                .into_iter()
                .filter(|value| *value > 0.0)
                .collect::<Vec<_>>();
            if active_sides.is_empty() {
                return false;
            }
            return active_sides.iter().any(|value| *value < 100.0);
        }

        false
    }

    pub fn item_subtitle_progress_visible(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        let has_active_work = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id
                && matches!(
                    workflow.kind,
                    WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia
                )
        });
        if !has_active_work {
            return false;
        }

        item.progress.subtitle > 0.0 && item.progress.subtitle < 100.0
    }

    pub fn item_file_name_progress(&self, item_index: usize) -> f32 {
        self.queue_items
            .get(item_index)
            .map(|item| item.progress.post_process)
            .unwrap_or(0.0)
    }

    pub fn item_file_name_progress_visible(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        let has_active_post_process = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id && workflow.kind == WorkflowKind::PostProcess
        });
        has_active_post_process
            && item.progress.post_process > 0.0
            && item.progress.post_process < 100.0
    }

    fn resolve_download_format_selection(
        &self,
        video_selector: &str,
        audio_selector: &str,
        metadata: Option<&VideoMetadata>,
    ) -> (String, String) {
        if self.is_muxed_format(video_selector, metadata) {
            return (video_selector.to_owned(), video_selector.to_owned());
        }

        let resolved_audio = if audio_selector.trim().is_empty()
            || audio_selector == video_selector
            || self.is_muxed_format(audio_selector, metadata)
        {
            metadata
                .into_iter()
                .flat_map(|metadata| metadata.formats.iter())
                .find(|format| format.kind == MediaKind::Audio)
                .map(|format| format.id.clone())
                .unwrap_or_default()
        } else {
            audio_selector.to_owned()
        };

        if resolved_audio.is_empty() {
            (resolved_audio, video_selector.to_owned())
        } else {
            (
                resolved_audio.clone(),
                format!("{}+{}", video_selector.trim(), resolved_audio.trim()),
            )
        }
    }

    pub fn video_formats(&self) -> impl Iterator<Item = &FormatOption> {
        self.current_picker_metadata()
            .formats
            .iter()
            .filter(|format| matches!(format.kind, MediaKind::Video | MediaKind::Muxed))
    }

    pub fn audio_formats(&self) -> impl Iterator<Item = &FormatOption> {
        self.current_picker_metadata()
            .formats
            .iter()
            .filter(|format| format.kind == MediaKind::Audio)
    }

    pub fn subtitle_source_options(&self) -> Vec<SubtitleOption> {
        let mut items: Vec<SubtitleOption> = self
            .current_picker_metadata()
            .subtitle_tracks
            .iter()
            .cloned()
            .fold(Vec::new(), |mut acc, track| {
                if !acc
                    .iter()
                    .any(|item| item.source_key() == track.source_key())
                {
                    acc.push(track);
                }
                acc
            });

        items.sort_by(|left, right| {
            left.source
                .label()
                .cmp(right.source.label())
                .then_with(|| left.source_language_label.cmp(&right.source_language_label))
                .then_with(|| left.source_language_code.cmp(&right.source_language_code))
        });
        items
    }

    pub fn subtitle_translation_options(&self) -> Vec<SubtitleOption> {
        let source_key = self.current_subtitle_source_key();
        let mut items: Vec<SubtitleOption> = self
            .current_picker_metadata()
            .subtitle_tracks
            .iter()
            .filter(|track| track.source_key() == source_key)
            .cloned()
            .collect();

        items.sort_by(|left, right| {
            left.target_language_code
                .is_some()
                .cmp(&right.target_language_code.is_some())
                .then_with(|| left.target_label().cmp(&right.target_label()))
        });
        items
    }

    pub fn open_format_picker(&mut self, target_item_id: usize, kind: FormatPickerKind) {
        let selected_id = self
            .queue_items
            .get(target_item_id)
            .map(|item| match kind {
                FormatPickerKind::Video => item.selection.video_selector.as_str(),
                FormatPickerKind::Audio => item.selection.audio_selector.as_str(),
                FormatPickerKind::Subtitle => item.selection.subtitle_selector.as_str(),
                FormatPickerKind::Section => item.selection.download_sections.as_str(),
            })
            .unwrap_or_default()
            .to_owned();

        self.format_picker.open = true;
        self.format_picker.target_item_id = Some(target_item_id);
        self.format_picker.kind = Some(kind);
        self.format_picker.view_mode = if self.app_mode == AppMode::Origin
            && matches!(kind, FormatPickerKind::Video | FormatPickerKind::Audio)
        {
            FormatPickerViewMode::Table
        } else {
            FormatPickerViewMode::Filter
        };
        self.format_picker.filter_text.clear();
        self.format_picker.filters.clear();

        if kind == FormatPickerKind::Section {
            let options = self.download_section_picker_options();
            self.format_picker.selected_row = options
                .iter()
                .position(|(value, _label)| value == &selected_id)
                .or(Some(0));
            return;
        }

        if kind == FormatPickerKind::Subtitle {
            let subtitle_source = self
                .queue_items
                .get(target_item_id)
                .map(|item| item.selection.subtitle_source)
                .unwrap_or(SubtitleSource::None);
            self.format_picker.subtitle_tab = match subtitle_source {
                SubtitleSource::None => SubtitlePickerTab::None,
                SubtitleSource::Original => SubtitlePickerTab::Original,
                SubtitleSource::Automatic => SubtitlePickerTab::Automatic,
            };
            self.format_picker.subtitle_source_key = match subtitle_source {
                SubtitleSource::None => SubtitleSource::None.key().to_owned(),
                _ => self
                    .queue_items
                    .get(target_item_id)
                    .and_then(|item| {
                        self.subtitle_track_by_id(
                            &item.selection.subtitle_selector,
                            self.item_metadata(target_item_id),
                        )
                    })
                    .map(|track| track.source_key())
                    .unwrap_or_default(),
            };
            self.format_picker.selected_row = if subtitle_source == SubtitleSource::None {
                Some(0)
            } else {
                let options = if subtitle_source == SubtitleSource::Original {
                    self.subtitle_source_options()
                        .into_iter()
                        .filter(|track| track.source == SubtitleSource::Original)
                        .collect::<Vec<_>>()
                } else {
                    self.subtitle_translation_options()
                };
                options.iter().position(|option| option.id == selected_id)
            };
            return;
        }

        let options = self.format_picker_options(kind);
        let selected_row = options.iter().position(|option| option.id == selected_id);
        let selected_option = selected_row.and_then(|index| options.get(index));
        self.format_picker.selected_row = selected_row;

        if let Some(option) = selected_option {
            match kind {
                FormatPickerKind::Video => {
                    if !option.resolution.is_empty() {
                        self.format_picker.filters.resolution = Some(option.resolution.clone());
                    }
                    if !option.dynamic_range.is_empty() {
                        self.format_picker.filters.dynamic_range =
                            Some(option.dynamic_range.clone());
                    }
                    if !option.fps.is_empty() {
                        self.format_picker.filters.fps = Some(option.fps.clone());
                    }
                    if !option.codec.is_empty() {
                        self.format_picker.filters.codec = Some(option.codec.clone());
                    }
                }
                FormatPickerKind::Audio => {
                    if !option.sample_rate.is_empty() {
                        self.format_picker.filters.sample_rate = Some(option.sample_rate.clone());
                    }
                    if !option.codec.is_empty() {
                        self.format_picker.filters.codec = Some(option.codec.clone());
                    }
                }
                FormatPickerKind::Subtitle | FormatPickerKind::Section => {}
            }
        }
    }

    pub fn cancel_format_picker(&mut self) {
        self.format_picker.open = false;
        self.format_picker.target_item_id = None;
        self.format_picker.kind = None;
        self.format_picker.selected_row = None;
        self.format_picker.filter_text.clear();
        self.format_picker.filters.clear();
        self.format_picker.subtitle_source_key.clear();
        self.format_picker.subtitle_tab = SubtitlePickerTab::None;
    }

    pub fn confirm_format_picker_selection(&mut self, selected_format_id: &str) {
        let Some(target_item_id) = self.format_picker.target_item_id else {
            return;
        };
        let Some(kind) = self.format_picker.kind else {
            return;
        };
        let item_metadata = self.item_metadata(target_item_id);
        let is_muxed_selection = self.is_muxed_format(selected_format_id, item_metadata);
        let item_uses_muxed_video = self.item_uses_muxed_video(target_item_id);
        let replacement_audio_selector = if kind == FormatPickerKind::Video && !is_muxed_selection {
            self.replacement_audio_selector_for_video_change(
                target_item_id,
                selected_format_id,
                item_metadata,
            )
        } else {
            None
        };
        let selected_subtitle_source = self
            .subtitle_track_by_id(selected_format_id, item_metadata)
            .map(|track| track.source);
        let Some(item) = self.queue_items.get_mut(target_item_id) else {
            self.cancel_format_picker();
            return;
        };

        match kind {
            FormatPickerKind::Video => {
                item.selection.video_selector = selected_format_id.to_owned();
                if is_muxed_selection {
                    item.selection.audio_selector = selected_format_id.to_owned();
                } else if let Some(audio_selector) = replacement_audio_selector {
                    item.selection.audio_selector = audio_selector;
                }
            }
            FormatPickerKind::Audio => {
                if item_uses_muxed_video {
                    self.cancel_format_picker();
                    return;
                }
                item.selection.audio_selector = selected_format_id.to_owned();
            }
            FormatPickerKind::Subtitle => {
                if selected_format_id.is_empty() {
                    item.selection.subtitle_selector.clear();
                    item.selection.subtitle_source = SubtitleSource::None;
                } else {
                    item.selection.subtitle_selector = selected_format_id.to_owned();
                    item.selection.subtitle_source =
                        selected_subtitle_source.unwrap_or(item.selection.subtitle_source);
                }
            }
            FormatPickerKind::Section => {
                item.selection.download_sections = selected_format_id.trim().to_owned();
                item.completed_selection = None;
            }
        }

        self.last_action = match kind {
            FormatPickerKind::Section if selected_format_id.trim().is_empty() => {
                i18n::format_fixed_english(
                    "Download range set: Item {index} / Full video",
                    &[("{index}", &(target_item_id + 1).to_string())],
                )
            }
            FormatPickerKind::Section => i18n::format_fixed_english(
                "Download range set: Item {index} / {value}",
                &[
                    ("{index}", &(target_item_id + 1).to_string()),
                    ("{value}", selected_format_id),
                ],
            ),
            _ => i18n::format_fixed_english(
                "Format selection updated: Item {index} / {kind} / {value}",
                &[
                    ("{index}", &(target_item_id + 1).to_string()),
                    ("{kind}", kind.label()),
                    ("{value}", selected_format_id),
                ],
            ),
        };
        self.cancel_format_picker();
    }

    pub fn format_picker_options(&self, kind: FormatPickerKind) -> Vec<FormatOption> {
        let mut options: Vec<FormatOption> = match kind {
            FormatPickerKind::Video => self.video_formats().cloned().collect(),
            FormatPickerKind::Audio => self.audio_formats().cloned().collect(),
            FormatPickerKind::Subtitle | FormatPickerKind::Section => Vec::new(),
        };

        match kind {
            FormatPickerKind::Video => {
                options.sort_by(|left, right| {
                    video_resolution_area(right)
                        .cmp(&video_resolution_area(left))
                        .then_with(|| {
                            human_size_bytes(&right.filesize).cmp(&human_size_bytes(&left.filesize))
                        })
                        .then_with(|| left.id.cmp(&right.id))
                });
            }
            FormatPickerKind::Audio | FormatPickerKind::Subtitle | FormatPickerKind::Section => {}
        }

        options
    }

    pub fn selected_format_summary(&self, item_index: usize, kind: FormatPickerKind) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        if !item.metadata_loaded() {
            return self
                .ui_i18n_text_for_key("picker.waiting_analysis")
                .to_owned();
        }

        if kind == FormatPickerKind::Audio && self.item_uses_muxed_video(item_index) {
            return self
                .ui_i18n_text_for_key("picker.audio_from_video")
                .to_owned();
        }

        if kind == FormatPickerKind::Subtitle {
            return self.selected_subtitle_summary(item_index);
        }
        if kind == FormatPickerKind::Section {
            return self.selected_download_section_summary(item_index);
        }

        let selected_id = match kind {
            FormatPickerKind::Video => &item.selection.video_selector,
            FormatPickerKind::Audio => &item.selection.audio_selector,
            FormatPickerKind::Subtitle => &item.selection.subtitle_selector,
            FormatPickerKind::Section => &item.selection.download_sections,
        };

        self.format_label_by_id(selected_id, item.metadata())
            .unwrap_or_default()
            .to_owned()
    }

    pub fn format_picker_target_title(&self) -> Option<&str> {
        self.format_picker
            .target_item_id
            .and_then(|index| self.queue_items.get(index))
            .map(|item| item.title.as_str())
    }

    pub fn selected_subtitle_summary(&self, item_index: usize) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        if item.selection.subtitle_source == SubtitleSource::None
            || item.selection.subtitle_selector.is_empty()
        {
            return self
                .ui_i18n_text_for_key("picker.subtitle_tab.none")
                .to_owned();
        }

        self.subtitle_track_by_id(&item.selection.subtitle_selector, item.metadata())
            .map(|track| {
                format!(
                    "{} / {}",
                    self.subtitle_source_label(track.source),
                    self.localized_subtitle_target_label(track)
                )
            })
            .unwrap_or_else(|| self.ui_i18n_text_for_key("picker.not_selected").to_owned())
    }

    pub fn subtitle_source_label(&self, source: SubtitleSource) -> &'static str {
        match source {
            SubtitleSource::None => self.ui_i18n_text_for_key("picker.subtitle_tab.none"),
            SubtitleSource::Original => self.ui_i18n_text_for_key("picker.subtitle_tab.original"),
            SubtitleSource::Automatic => self.ui_i18n_text_for_key("picker.subtitle_tab.automatic"),
        }
    }

    pub fn localized_subtitle_target_label(&self, option: &SubtitleOption) -> String {
        match (&option.target_language_label, &option.target_language_code) {
            (Some(label), Some(code)) => format!("{label} ({code})"),
            (Some(label), None) => label.clone(),
            (None, Some(code)) => code.clone(),
            (None, None) => self
                .ui_i18n_text_for_key("picker.no_translation")
                .to_owned(),
        }
    }

    pub fn item_shows_subtitle_row(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        item.metadata()
            .is_some_and(|metadata| !metadata.subtitle_tracks.is_empty())
    }

    pub fn item_shows_download_section_row(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        item.metadata()
            .is_some_and(|metadata| !metadata.chapters.is_empty())
            || !item.selection.download_sections.trim().is_empty()
    }

    pub fn item_download_section_options(&self, item_index: usize) -> Vec<(String, String)> {
        self.queue_items
            .get(item_index)
            .and_then(QueueItem::metadata)
            .map(|metadata| {
                metadata
                    .chapters
                    .iter()
                    .map(|chapter| {
                        (
                            chapter.download_sections.clone(),
                            self.localized_chapter_label(chapter),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn download_section_picker_options(&self) -> Vec<(String, String)> {
        let Some(item_index) = self.format_picker.target_item_id else {
            return vec![(
                String::new(),
                self.ui_i18n_text_for_key("picker.full_video").to_owned(),
            )];
        };

        let mut options = Vec::with_capacity(1);
        options.push((
            String::new(),
            self.ui_i18n_text_for_key("picker.full_video").to_owned(),
        ));
        options.extend(self.item_download_section_options(item_index));
        options
    }

    pub fn selected_download_section_summary(&self, item_index: usize) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        let selected = item.selection.download_sections.trim();
        if selected.is_empty() {
            return self.ui_i18n_text_for_key("picker.full_video").to_owned();
        }

        item.metadata()
            .into_iter()
            .flat_map(|metadata| metadata.chapters.iter())
            .find(|chapter| chapter.download_sections.as_str() == selected)
            .map(|chapter| self.localized_chapter_label(chapter))
            .unwrap_or_else(|| selected.to_owned())
    }

    fn localized_chapter_label(&self, chapter: &crate::domain::ChapterOption) -> String {
        let range = match chapter.end_text.as_deref() {
            Some(end) if !end.is_empty() => format!("{}–{}", chapter.start_text, end),
            _ => format!(
                "{}–{}",
                chapter.start_text,
                self.ui_i18n_text_for_key("picker.until_end")
            ),
        };

        if chapter.title.trim().is_empty() {
            range
        } else {
            format!("{}  {}", range, chapter.title)
        }
    }

    pub fn set_item_download_sections(&mut self, item_index: usize, value: impl Into<String>) {
        let Some(item) = self.queue_items.get_mut(item_index) else {
            return;
        };
        let title = item.title.clone();
        let value = value.into().trim().to_owned();
        item.selection.download_sections = value.clone();
        item.completed_selection = None;
        self.last_action = if value.is_empty() {
            i18n::format_fixed_english(
                "Download range set: {title} / Full video",
                &[("{title}", title.as_str())],
            )
        } else {
            i18n::format_fixed_english(
                "Download range set: {title} / {value}",
                &[("{title}", title.as_str()), ("{value}", value.as_str())],
            )
        };
    }

    pub fn item_uses_seed_compact_layout(&self, item_index: usize) -> bool {
        self.queue_items
            .get(item_index)
            .is_some_and(|item| matches!(item.metadata_state, MetadataState::Idle))
    }

    pub fn subtitle_track_by_id<'a>(
        &'a self,
        id: &str,
        metadata: Option<&'a VideoMetadata>,
    ) -> Option<&'a SubtitleOption> {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.subtitle_tracks.iter())
            .find(|track| track.id == id)
    }

    pub fn current_subtitle_source_key(&self) -> String {
        if !self.format_picker.subtitle_source_key.is_empty() {
            return self.format_picker.subtitle_source_key.clone();
        }

        self.subtitle_source_options()
            .first()
            .map(|track| track.source_key())
            .unwrap_or_else(|| SubtitleSource::None.key().to_owned())
    }

    pub fn item_uses_muxed_video(&self, item_index: usize) -> bool {
        self.queue_items
            .get(item_index)
            .map(|item| self.is_muxed_format(&item.selection.video_selector, item.metadata()))
            .unwrap_or(false)
    }

    fn replacement_audio_selector_for_video_change(
        &self,
        item_index: usize,
        video_selector: &str,
        metadata: Option<&VideoMetadata>,
    ) -> Option<String> {
        let current_audio = self
            .queue_items
            .get(item_index)?
            .selection
            .audio_selector
            .trim();
        if !current_audio.is_empty()
            && current_audio != video_selector
            && !self.is_muxed_format(current_audio, metadata)
        {
            return None;
        }

        first_audio_format_id(metadata)
    }

    pub fn is_muxed_format(&self, format_id: &str, metadata: Option<&VideoMetadata>) -> bool {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == format_id)
            .map(|format| format.kind == MediaKind::Muxed)
            .unwrap_or(false)
    }

    pub fn format_label_by_id<'a>(
        &'a self,
        id: &str,
        metadata: Option<&'a VideoMetadata>,
    ) -> Option<&'a str> {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == id)
            .map(|format| format.label.as_str())
    }

    pub fn format_extension_by_id(&self, id: &str, metadata: Option<&VideoMetadata>) -> String {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == id)
            .map(|format| format.ext.clone())
            .unwrap_or_default()
    }

    pub fn format_codec_by_id(&self, id: &str, metadata: Option<&VideoMetadata>) -> String {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == id)
            .map(|format| format.codec.clone())
            .unwrap_or_default()
    }

    fn apply_analysis_json(
        &mut self,
        json: Value,
        analyzed_source: Option<String>,
        target_item_id: Option<QueueItemId>,
        workflow_id: Option<WorkflowRunId>,
    ) {
        if json.get("entries").and_then(Value::as_array).is_some() {
            let target = analyzed_source.unwrap_or_else(|| "playlist".to_owned());
            self.last_action = i18n::format_fixed_english(
                "Playlist is ignored for now: {target}",
                &[("{target}", target.as_str())],
            );
            return;
        }

        let title = json
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Untitled video")
            .to_owned();
        let webpage_url = json
            .get("webpage_url")
            .or_else(|| json.get("original_url"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let channel = json_str_field(&json, "channel").unwrap_or_default();
        let channel_url = json_str_field(&json, "channel_url").unwrap_or_default();
        let uploader = json_str_field(&json, "uploader").unwrap_or_default();
        let uploader_url = json_str_field(&json, "uploader_url").unwrap_or_default();
        let creator = json_str_field(&json, "creator").unwrap_or_default();
        let creator_url = json_str_field(&json, "creator_url").unwrap_or_default();
        let duration_text = json
            .get("duration_string")
            .and_then(Value::as_str)
            .map(normalize_duration_badge_text)
            .unwrap_or_default();
        let description = json
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let upload_date_text = json
            .get("upload_date")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let thumbnail_hint = json
            .get("thumbnail")
            .and_then(Value::as_str)
            .map(|_| "Thumbnail preview".to_owned())
            .unwrap_or_else(|| "item.thumbnail".to_owned());
        let thumbnail_url = select_best_thumbnail_url(&json).unwrap_or_default();

        let formats = extract_formats(&json);
        let requested_ids = extract_requested_ids(&json);
        let subtitle_tracks = extract_subtitle_tracks(&json);
        let chapters = extract_chapters(&json, |index| {
            let number = (index + 1).to_string();
            i18n::format_fixed_english("Chapter {index}", &[("{index}", number.as_str())])
        });

        let metadata = VideoMetadata {
            title: title.clone(),
            channel,
            channel_url,
            uploader,
            uploader_url,
            creator,
            creator_url,
            duration_text,
            webpage_url,
            description,
            view_count_text: json_number_or_str_field(&json, "view_count").unwrap_or_default(),
            upload_date_text,
            thumbnail_hint,
            thumbnail_url,
            formats: formats.clone(),
            subtitle_tracks: subtitle_tracks.clone(),
            chapters: chapters.clone(),
        };
        let default_video = requested_or_default_format_id(
            &formats,
            &requested_ids,
            &[MediaKind::Video, MediaKind::Muxed],
        );
        let default_audio = if formats
            .iter()
            .find(|format| format.id == default_video)
            .is_some_and(|format| format.kind == MediaKind::Muxed)
        {
            default_video.clone()
        } else {
            requested_or_default_format_id(&formats, &requested_ids, &[MediaKind::Audio])
        };

        let default_subtitle_source = SubtitleSource::None;
        let default_subtitle = String::new();
        let default_file_name = extract_requested_filename(&json)
            .or_else(|| {
                json.get("_filename")
                    .or_else(|| json.get("filename"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .map(|filename| sanitize_file_name_for_windows(&display_file_stem(&filename)))
            .unwrap_or_default();

        if let Some(item_id) = target_item_id {
            if let Some(item) = self.queue_item_mut_by_id(item_id) {
                item.title = title.clone();
                item.thumbnail_hint = "item.thumbnail".to_owned();
                item.thumbnail_url = metadata.thumbnail_url.clone();
                item.duration_text = metadata.duration_text.clone();
                item.metadata_state = MetadataState::Ready(metadata.clone());
                item.selection.video_selector = default_video.clone();
                item.selection.audio_selector = default_audio.clone();
                item.selection.subtitle_selector = default_subtitle.clone();
                item.selection.subtitle_source = default_subtitle_source;
                if item.selection.file_name.trim().is_empty() {
                    item.selection.file_name = default_file_name.clone();
                }
                let run_index = workflow_id
                    .and_then(|workflow_id| {
                        item.workflows.iter().position(|run| run.id == workflow_id)
                    })
                    .or_else(|| {
                        item.workflows
                            .iter()
                            .rposition(|run| run.kind == WorkflowKind::AnalyzeMetadata)
                    });
                if let Some(run) = run_index.and_then(|index| item.workflows.get_mut(index)) {
                    run.state = WorkflowState::Finished;
                    run.detail = analyzed_source
                        .clone()
                        .unwrap_or_else(|| metadata.webpage_url.clone());
                }
            }
            if let Some(workflow_id) = workflow_id {
                self.unregister_active_workflow(workflow_id);
            }
        } else {
            let item_source_url = analyzed_source
                .clone()
                .filter(|source| !source.trim().is_empty())
                .or_else(|| {
                    (!metadata.webpage_url.trim().is_empty()).then(|| metadata.webpage_url.clone())
                })
                .unwrap_or_else(|| self.url_input.trim().to_owned());
            let mut item =
                QueueItem::new(self.alloc_queue_item_id(), item_source_url, title.clone());
            item.selection.quality = self.item_defaults.quality;
            item.selection.write_thumbnail = self.item_defaults.write_thumbnail;
            item.selection.embed_thumbnail = self.item_defaults.embed_thumbnail;
            item.selection.write_subtitles = self.item_defaults.write_subtitles;
            item.selection.embed_subtitles = self.item_defaults.embed_subtitles;
            item.selection.write_chapters = self.item_defaults.write_chapters;
            item.selection.embed_chapters = self.item_defaults.embed_chapters;
            item.selection.use_cookies = self.item_defaults.use_cookies;
            item.selection.use_aria2 = self.item_defaults.use_aria2;
            item.selection.output_dir = self.item_defaults.output_dir.clone();
            item.metadata_state = MetadataState::Ready(metadata.clone());
            item.thumbnail_url = metadata.thumbnail_url.clone();
            item.duration_text = metadata.duration_text.clone();
            item.selection.video_selector = default_video.clone();
            item.selection.audio_selector = default_audio.clone();
            item.selection.subtitle_selector = default_subtitle.clone();
            item.selection.subtitle_source = default_subtitle_source;
            item.selection.file_name = default_file_name.clone();
            self.queue_items = vec![item];
        }

        let analyzed_target = analyzed_source
            .or_else(|| (!metadata.webpage_url.is_empty()).then(|| metadata.webpage_url.clone()))
            .unwrap_or_else(|| title.clone());
        self.last_action = i18n::format_fixed_english(
            "Analysis complete: {title}",
            &[("{title}", analyzed_target.as_str())],
        );
        self.mark_font_content_changed();
    }
}

fn first_cookie_rescue_url_candidate(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|part| trim_cookie_rescue_url_wrappers(part))
        .find(|candidate| normalize_cookie_rescue_target_url(candidate).is_ok())
        .map(ToOwned::to_owned)
}

fn single_cookie_rescue_clipboard_url_candidate(text: &str) -> Option<String> {
    let trimmed = trim_cookie_rescue_url_wrappers(text.trim());
    if trimmed.is_empty() || trimmed.len() > 2048 || trimmed.chars().any(char::is_whitespace) {
        return None;
    }
    normalize_cookie_rescue_target_url(trimmed)
        .ok()
        .map(|_| trimmed.to_owned())
}

fn trim_cookie_rescue_url_wrappers(value: &str) -> &str {
    value.trim_matches(|ch: char| {
        matches!(
            ch,
            '<' | '>' | '"' | '\'' | '`' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
        )
    })
}

fn cookie_rescue_url_from_dropped_path(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    if extension == "url" {
        let data = fs::read_to_string(path).ok()?;
        for line in data.lines() {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("URL=") {
                return Some(value.trim().to_owned());
            }
        }
        return None;
    }

    if extension == "txt" {
        let data = fs::read_to_string(path).ok()?;
        return first_cookie_rescue_url_candidate(&data);
    }

    None
}

fn saved_cookie_file_from_index_entry(entry: CookieSiteIndexEntry) -> SavedCookieFile {
    let display_name = entry.display_name.trim();
    SavedCookieFile {
        id: entry.id,
        display_name: if display_name.is_empty() {
            entry
                .match_domains
                .first()
                .cloned()
                .unwrap_or_else(|| "Cookie".to_owned())
        } else {
            display_name.to_owned()
        },
        login_url: entry.login_url,
        updated_unix: entry.updated_unix,
    }
}

fn cookie_file_path_owned_by_cookie_dir(cookie_dir: &Path, cookie_file: &str) -> Option<PathBuf> {
    let cookie_file = cookie_file.trim();
    if cookie_file.is_empty() {
        return None;
    }

    let candidate = PathBuf::from(cookie_file);
    let path = if candidate.is_absolute() {
        candidate
    } else {
        cookie_dir.join(candidate)
    };
    let normalized_cookie_dir = normalized_path_for_safety(cookie_dir);
    let normalized_path = normalized_path_for_safety(&path);
    normalized_path
        .starts_with(&normalized_cookie_dir)
        .then_some(path)
}

fn read_clipboard_text() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    clipboard.get_text().ok()
}

fn extract_monitored_youtube_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|candidate| {
            candidate
                .trim_matches(|ch: char| {
                    matches!(
                        ch,
                        '"' | '\''
                            | '`'
                            | '<'
                            | '>'
                            | '('
                            | ')'
                            | '['
                            | ']'
                            | '{'
                            | '}'
                            | '\u{ff0c}'
                            | '\u{3002}'
                            | '\u{3001}'
                            | '\u{ff1b}'
                            | ';'
                            | '\u{ff1a}'
                            | ':'
                            | ','
                    )
                })
                .to_owned()
        })
        .filter(|candidate| !candidate.is_empty())
        .find_map(|candidate| normalize_monitored_youtube_url(&candidate))
}

fn normalize_monitored_youtube_url(candidate: &str) -> Option<String> {
    let lowered = candidate.to_ascii_lowercase();
    if !(lowered.contains("youtube.com") || lowered.contains("youtu.be")) {
        return None;
    }

    if lowered.starts_with("http://") || lowered.starts_with("https://") {
        Some(candidate.to_owned())
    } else if lowered.starts_with("www.youtube.com")
        || lowered.starts_with("m.youtube.com")
        || lowered.starts_with("youtube.com")
        || lowered.starts_with("youtu.be")
    {
        Some(format!("https://{candidate}"))
    } else {
        None
    }
}

fn canonical_queue_source_key(url: &str) -> String {
    let trimmed = url.trim();
    if let Some(video_id) = youtube_video_id(trimmed) {
        return format!("youtube:video:{video_id}");
    }
    trimmed.to_ascii_lowercase()
}

fn youtube_video_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lowered = trimmed.to_ascii_lowercase();

    if lowered.contains("youtu.be/") {
        let (_, tail) = trimmed.split_once("youtu.be/")?;
        let id = tail
            .split(['?', '&', '#', '/'])
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        return Some(id.to_owned());
    }

    if lowered.contains("youtube.com/watch") || lowered.contains("m.youtube.com/watch") {
        let (_, tail) = trimmed.split_once("v=")?;
        let id = tail
            .split(['&', '#', '?', '/'])
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        return Some(id.to_owned());
    }

    None
}

fn should_retry_analyze_with_cookies(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("sign in to confirm you're not a bot")
        || normalized.contains("sign in to confirm you")
        || normalized.contains("use --cookies-from-browser")
        || normalized.contains("use --cookies for the authentication")
}

fn normalize_export_target_path(path: &str, default_extension: Option<&str>) -> String {
    let trimmed = path.trim();
    let mut target = PathBuf::from(trimmed);
    let has_extension = target
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| !value.trim().is_empty());
    if !has_extension {
        if let Some(extension) = default_extension.filter(|value| !value.trim().is_empty()) {
            target.set_extension(extension);
        }
    }
    target.display().to_string()
}

fn normalized_export_extension(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_start_matches('.').to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn validate_export_extension(kind: DownloadTargetKind, extension: &str) -> Result<(), String> {
    let valid = match kind {
        DownloadTargetKind::Video => matches!(extension, "mkv" | "mp4" | "webm" | "mov" | "flv"),
        DownloadTargetKind::Audio => {
            matches!(
                extension,
                "opus" | "aac" | "m4a" | "mp3" | "vorbis" | "alac" | "flac" | "wav"
            )
        }
        DownloadTargetKind::Subtitle => {
            matches!(
                extension,
                "srt"
                    | "vtt"
                    | "ass"
                    | "ssa"
                    | "lrc"
                    | "ttml"
                    | "dfxp"
                    | "srv1"
                    | "srv2"
                    | "srv3"
                    | "json3"
            )
        }
        DownloadTargetKind::Normal => true,
    };
    if valid {
        Ok(())
    } else {
        Err(match kind {
            DownloadTargetKind::Video => "Could not determine the video file extension.".to_owned(),
            DownloadTargetKind::Audio => "Could not determine the audio file extension.".to_owned(),
            DownloadTargetKind::Subtitle => {
                "Could not determine the subtitle file extension.".to_owned()
            }
            DownloadTargetKind::Normal => String::new(),
        })
    }
}

fn flat_music_entries_from_url(
    tool_paths: ToolPaths,
    source: &str,
    untitled_task: &str,
    imported_template: &str,
) -> Result<Vec<PlaylistEntrySeed>, String> {
    let mut command = tool_paths.prepare_batch_add_command(source)?;
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start yt-dlp music flat import: {error}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Could not read yt-dlp music flat output.".to_owned())?;
    let mut stderr = child.stderr.take();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    let mut seeds = Vec::new();

    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|error| format!("Could not read yt-dlp music flat output: {error}"))?;
        if read == 0 {
            break;
        }
        let raw = line.trim();
        if raw.is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_str::<Value>(raw) else {
            continue;
        };
        if let Some(mut seed) =
            playlist_entry_seed_from_json(&entry, untitled_task, imported_template)
        {
            if let Some(thumbnail_url) = select_largest_thumbnail_url(&entry) {
                seed.thumbnail_url = thumbnail_url;
                seed.thumbnail_hint = "Thumbnail preview".to_owned();
            }
            seeds.push(seed);
        }
    }

    let status = child
        .wait()
        .map_err(|error| format!("Could not wait for yt-dlp music flat import: {error}"))?;
    let mut stderr_text = String::new();
    if let Some(mut reader) = stderr.take() {
        let _ = reader.read_to_string(&mut stderr_text);
    }
    if !status.success() && seeds.is_empty() {
        let detail = stderr_text.trim();
        return Err(if detail.is_empty() {
            format!(
                "yt-dlp music flat import failed: exit code {:?}",
                status.code()
            )
        } else {
            format!("yt-dlp music flat import failed: {detail}")
        });
    }
    if seeds.is_empty() {
        return Err("yt-dlp did not return any music list entries.".to_owned());
    }
    Ok(seeds)
}

fn stable_media_session_title(title: &str, source_url: &str) -> String {
    let trimmed = title.trim();
    if !trimmed.is_empty() && !is_transient_media_session_title(trimmed) {
        return trimmed.to_owned();
    }

    let source = source_url.trim();
    if source.is_empty() {
        "Audio".to_owned()
    } else {
        source.to_owned()
    }
}

fn is_transient_media_session_title(value: &str) -> bool {
    const STATUS_KEYS: &[&str] = &[
        "music.status.resolving",
        "music.status.buffering",
        "music.status.ready",
        "music.status.caching",
        "music.status.playing",
    ];

    let trimmed = value.trim();
    STATUS_KEYS.iter().any(|key| {
        Language::ALL
            .iter()
            .any(|language| i18n::text(*language, key) == trimmed)
    }) || matches!(trimmed, "Buffering...")
}

fn split_artist_title_for_media_session(value: &str) -> (String, String) {
    let trimmed = value.trim();
    for separator in [" - ", " – ", " — "] {
        if let Some((artist, title)) = trimmed.split_once(separator) {
            let artist = artist.trim();
            let title = title.trim();
            if !artist.is_empty() && !title.is_empty() {
                return (artist.to_owned(), title.to_owned());
            }
        }
    }
    (String::new(), trimmed.to_owned())
}

fn duration_text_to_seconds(text: &str) -> Option<f64> {
    let mut total = 0_u64;
    let mut saw_part = false;
    for part in text.trim().split(':') {
        let value = part.trim().parse::<u64>().ok()?;
        total = total.saturating_mul(60).saturating_add(value);
        saw_part = true;
    }
    saw_part
        .then_some(total as f64)
        .filter(|value| *value > 0.0)
}

fn music_stream_seed_from_json(json: &Value, source: &str) -> Result<MusicStreamSeed, String> {
    let requested = json
        .get("requested_downloads")
        .and_then(Value::as_array)
        .and_then(|items| items.first());

    let direct_url = requested
        .and_then(|value| json_str_field(value, "url"))
        .or_else(|| json_str_field(json, "url"))
        .ok_or_else(|| "yt-dlp did not return a playable audio stream URL.".to_owned())?;

    let title = json_str_field(json, "title")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| source.to_owned());
    let album_title = music_album_title_from_json(json);
    let duration_seconds = json_f64_field(json, "duration");
    let duration_text = json_str_field(json, "duration_string")
        .as_deref()
        .map(normalize_duration_badge_text)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| duration_seconds.map(format_duration_seconds))
        .unwrap_or_default();
    let thumbnail_url = select_largest_thumbnail_url(json).unwrap_or_default();
    let ext = requested
        .and_then(|value| json_str_field(value, "ext"))
        .or_else(|| json_str_field(json, "ext"))
        .unwrap_or_default();
    let format_id = requested
        .and_then(|value| json_str_field(value, "format_id"))
        .or_else(|| json_str_field(json, "format_id"))
        .unwrap_or_default();
    let acodec = requested
        .and_then(|value| json_str_field(value, "acodec"))
        .or_else(|| json_str_field(json, "acodec"))
        .unwrap_or_default();
    let expected_bytes = requested
        .and_then(|value| json_u64_field(value, "filesize"))
        .or_else(|| requested.and_then(|value| json_u64_field(value, "filesize_approx")))
        .or_else(|| json_u64_field(json, "filesize"))
        .or_else(|| json_u64_field(json, "filesize_approx"));
    let cache_key = music_cache_key(source, &format_id, &ext, &acodec);
    let headers = requested
        .and_then(|value| value.get("http_headers"))
        .and_then(headers_from_json)
        .or_else(|| json.get("http_headers").and_then(headers_from_json))
        .unwrap_or_default();

    Ok(MusicStreamSeed {
        source_url: source.to_owned(),
        title,
        album_title,
        thumbnail_url,
        thumbnail_hint: "item.thumbnail".to_owned(),
        duration_text,
        duration_seconds,
        direct_url,
        headers,
        ext,
        format_id,
        acodec,
        expected_bytes,
        cache_key,
        lyrics_track: primary_original_subtitle_track_from_json(json),
    })
}

fn primary_original_subtitle_track_from_json(json: &Value) -> Option<SubtitleOption> {
    let tracks = extract_subtitle_tracks(json);
    primary_original_subtitle_track_from_tracks(json, tracks.into_iter())
}

fn primary_original_subtitle_track_from_metadata(
    metadata: &VideoMetadata,
) -> Option<&SubtitleOption> {
    let original_tracks = metadata
        .subtitle_tracks
        .iter()
        .filter(|track| is_direct_original_subtitle_track(track))
        .collect::<Vec<_>>();
    if original_tracks.is_empty() {
        return None;
    }

    let preferred_languages = metadata_language_candidates_from_metadata(metadata);
    for language in preferred_languages {
        if let Some(track) = original_tracks.iter().find(|track| {
            subtitle_language_matches(&track.download_language_code, &language)
                || subtitle_language_matches(&track.source_language_code, &language)
        }) {
            return Some(*track);
        }
    }

    original_tracks.into_iter().next()
}

fn primary_original_subtitle_track_from_tracks(
    json: &Value,
    tracks: impl Iterator<Item = SubtitleOption>,
) -> Option<SubtitleOption> {
    let original_tracks = tracks
        .filter(is_direct_original_subtitle_track)
        .collect::<Vec<_>>();
    if original_tracks.is_empty() {
        return None;
    }

    let preferred_languages = metadata_language_candidates(json);
    for language in preferred_languages {
        if let Some(track) = original_tracks.iter().find(|track| {
            subtitle_language_matches(&track.download_language_code, &language)
                || subtitle_language_matches(&track.source_language_code, &language)
        }) {
            return Some(track.clone());
        }
    }

    original_tracks.into_iter().next()
}

fn is_direct_original_subtitle_track(track: &SubtitleOption) -> bool {
    track.source == SubtitleSource::Original && track.target_language_code.is_none()
}

fn metadata_language_candidates(json: &Value) -> Vec<String> {
    let mut languages = Vec::new();
    for key in ["language", "original_language", "lang"] {
        if let Some(value) = json_str_field(json, key) {
            push_unique_language(&mut languages, value);
        }
    }
    push_text_inferred_language_candidates(
        &mut languages,
        [
            "track",
            "title",
            "fulltitle",
            "alt_title",
            "artist",
            "artists",
            "creator",
            "channel",
            "uploader",
        ]
        .into_iter()
        .filter_map(|key| json_str_field(json, key)),
    );
    languages
}

fn metadata_language_candidates_from_metadata(metadata: &VideoMetadata) -> Vec<String> {
    let mut languages = Vec::new();
    push_text_inferred_language_candidates(
        &mut languages,
        [
            metadata.title.as_str(),
            metadata.creator.as_str(),
            metadata.channel.as_str(),
            metadata.uploader.as_str(),
        ],
    );
    languages
}

fn push_text_inferred_language_candidates<T: AsRef<str>>(
    languages: &mut Vec<String>,
    texts: impl IntoIterator<Item = T>,
) {
    let mut saw_japanese_kana = false;
    let mut saw_hangul = false;
    let mut saw_thai = false;

    for text in texts {
        for ch in text.as_ref().chars() {
            let code = ch as u32;
            saw_japanese_kana |= (0x3040..=0x309f).contains(&code)
                || (0x30a0..=0x30ff).contains(&code)
                || (0xff66..=0xff9d).contains(&code);
            saw_hangul |= (0xac00..=0xd7af).contains(&code)
                || (0x1100..=0x11ff).contains(&code)
                || (0x3130..=0x318f).contains(&code);
            saw_thai |= (0x0e00..=0x0e7f).contains(&code);
        }
    }

    if saw_japanese_kana {
        push_unique_language(languages, "ja".to_owned());
    }
    if saw_hangul {
        push_unique_language(languages, "ko".to_owned());
    }
    if saw_thai {
        push_unique_language(languages, "th".to_owned());
    }
}

fn push_unique_language(languages: &mut Vec<String>, value: String) {
    let normalized = normalize_subtitle_language_code(&value);
    if normalized.is_empty() {
        return;
    }
    if !languages.iter().any(|item| item == &normalized) {
        languages.push(normalized);
    }
}

fn subtitle_language_matches(left: &str, right: &str) -> bool {
    let left = normalize_subtitle_language_code(left);
    let right = normalize_subtitle_language_code(right);
    !left.is_empty()
        && !right.is_empty()
        && (left == right
            || left.starts_with(&format!("{right}-"))
            || right.starts_with(&format!("{left}-")))
}

fn normalize_subtitle_language_code(value: &str) -> String {
    let normalized = value.trim().replace('_', "-").to_ascii_lowercase();
    match normalized.as_str() {
        "jp" | "jpn" | "japanese" => "ja".to_owned(),
        "kr" | "kor" | "korean" => "ko".to_owned(),
        "cn" | "chi" | "zho" | "chinese" => "zh".to_owned(),
        "tw" => "zh-tw".to_owned(),
        _ => normalized,
    }
}

fn complete_music_cache_media_path_in_root(item: &QueueItem, cache_root: &Path) -> Option<PathBuf> {
    if item.music_cache_key.trim().is_empty() || item.music_stream_ext.trim().is_empty() {
        return None;
    }
    let cache_dir = cache_root.join(sanitize_music_cache_key(&item.music_cache_key));
    let manifest_path = cache_dir.join("manifest.yaml");
    let manifest = read_yaml_file::<AudioCacheManifestSnapshot>(&manifest_path)?;
    if !audio_cache_manifest_is_fresh(&manifest) {
        let _ = fs::remove_dir_all(&cache_dir);
        return None;
    }
    if !manifest.complete {
        return None;
    }
    let path = cache_dir.join(format!(
        "audio.{}",
        sanitize_music_cache_ext(&item.music_stream_ext)
    ));
    let media_len = fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
    if media_len == 0 {
        return None;
    }
    let expected_bytes = manifest.expected_bytes.or(item.music_stream_expected_bytes);
    if expected_bytes.is_some_and(|expected| expected > media_len) {
        return None;
    }
    Some(path)
}

fn music_cached_progress_for_item_in_root(item: &QueueItem, cache_root: &Path) -> f32 {
    if item.music_cache_key.trim().is_empty() || item.music_stream_ext.trim().is_empty() {
        return 0.0;
    }
    let cache_dir = cache_root.join(sanitize_music_cache_key(&item.music_cache_key));
    let manifest_path = cache_dir.join("manifest.yaml");
    if let Some(ratio) =
        music_cache_manifest_progress_ratio(&manifest_path, item.music_stream_expected_bytes)
    {
        return ratio;
    }
    let path = cache_dir.join(format!(
        "audio.{}",
        sanitize_music_cache_ext(&item.music_stream_ext)
    ));
    let len = fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
    if let Some(expected) = item.music_stream_expected_bytes.filter(|value| *value > 0) {
        return (len as f32 / expected as f32).clamp(0.0, 1.0);
    }
    0.0
}

fn music_lrc_cache_path(cache_root: &Path, cache_key: &str) -> PathBuf {
    cache_root
        .join("lyrics")
        .join(sanitize_music_cache_key(cache_key))
        .join("lyrics.lrc")
}

fn cache_music_lyrics_with_yt_dlp(
    tool_paths: &ToolPaths,
    cache_root: &Path,
    job: MusicLyricsCacheJob,
) -> Result<(), String> {
    let lyrics_dir = cache_root
        .join("lyrics")
        .join(sanitize_music_cache_key(&job.cache_key));
    fs::create_dir_all(&lyrics_dir)
        .map_err(|error| format!("Could not create lyrics cache folder: {error}"))?;
    let target_path = lyrics_dir.join("lyrics.lrc");
    if target_path.is_file() {
        return Ok(());
    }
    let mut command = tool_paths.prepare_music_lyrics_cache_command(
        &job.source_url,
        &lyrics_dir,
        &job.language_code,
        job.use_cookies,
    )?;
    let output = command
        .output()
        .map_err(|error| format!("Could not start yt-dlp lyrics cache: {error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr)
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("unknown yt-dlp error")
            .to_owned();
        return Err(format!("yt-dlp lyrics cache failed: {detail}"));
    }
    let Some(candidate) = find_latest_file_in_dir(&lyrics_dir, "lrc") else {
        return Err("yt-dlp finished, but no LRC lyrics file was produced.".to_owned());
    };
    if candidate != target_path {
        if target_path.exists() {
            let _ = fs::remove_file(&target_path);
        }
        fs::rename(&candidate, &target_path)
            .or_else(|_| fs::copy(&candidate, &target_path).map(|_| ()))
            .map_err(|error| format!("Could not move LRC lyrics into cache: {error}"))?;
    }
    cleanup_music_lyrics_cache_dir(&lyrics_dir, &target_path);
    Ok(())
}

fn cleanup_music_lyrics_cache_dir(lyrics_dir: &Path, keep_path: &Path) {
    let Ok(entries) = fs::read_dir(lyrics_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path != keep_path && path.is_file() {
            let _ = fs::remove_file(path);
        }
    }
}

fn parse_lrc_file(path: &Path) -> Result<Vec<LrcLine>, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("Could not read LRC lyrics cache: {error}"))?;
    Ok(parse_lrc_text(&text))
}

fn parse_lrc_text(text: &str) -> Vec<LrcLine> {
    let mut lines = Vec::new();
    for raw_line in text.lines() {
        let mut rest = raw_line.trim();
        let mut timestamps = Vec::new();
        while let Some(stripped) = rest.strip_prefix('[') {
            let Some((timestamp, tail)) = stripped.split_once(']') else {
                break;
            };
            let Some(seconds) = parse_lrc_timestamp(timestamp) else {
                break;
            };
            timestamps.push(seconds);
            rest = tail.trim_start();
        }
        let text = rest.trim();
        if text.is_empty() || timestamps.is_empty() {
            continue;
        }
        for seconds in timestamps {
            lines.push(LrcLine {
                seconds,
                text: text.to_owned(),
            });
        }
    }
    lines.sort_by(|left, right| {
        left.seconds
            .partial_cmp(&right.seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    lines
}

fn parse_lrc_timestamp(value: &str) -> Option<f64> {
    let mut parts = value.trim().split(':').collect::<Vec<_>>();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    let seconds_text = parts.pop()?;
    let minutes = parts.pop()?.parse::<u64>().ok()?;
    let hours = parts
        .pop()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let seconds = seconds_text.replace(',', ".").parse::<f64>().ok()?;
    if !seconds.is_finite() || seconds < 0.0 {
        return None;
    }
    Some(hours as f64 * 3600.0 + minutes as f64 * 60.0 + seconds)
}

fn current_lrc_line_text(lines: &[LrcLine], seconds: f64) -> Option<String> {
    if !seconds.is_finite() || lines.is_empty() {
        return None;
    }
    let index = lines.partition_point(|line| line.seconds <= seconds.max(0.0));
    if index == 0 {
        return None;
    }
    lines
        .get(index - 1)
        .map(|line| line.text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn music_cache_key(source: &str, format_id: &str, ext: &str, acodec: &str) -> String {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    format_id.hash(&mut hasher);
    ext.hash(&mut hasher);
    acodec.hash(&mut hasher);
    format!("music_{:016x}", hasher.finish())
}

fn json_str_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn music_album_title_from_json(value: &Value) -> String {
    json_str_field(value, "album")
        .or_else(|| json_str_field(value, "playlist_title"))
        .or_else(|| json_str_field(value, "playlist"))
        .unwrap_or_default()
}

fn json_number_or_str_field(value: &Value, key: &str) -> Option<String> {
    let value = value.get(key)?;
    if let Some(text) = value.as_str() {
        let text = text.trim();
        return (!text.is_empty()).then(|| text.to_owned());
    }
    if let Some(number) = value.as_u64() {
        return Some(number.to_string());
    }
    if let Some(number) = value.as_i64() {
        return (number >= 0).then(|| number.to_string());
    }
    if let Some(number) = value
        .as_f64()
        .filter(|number| number.is_finite() && *number >= 0.0)
    {
        return Some(format!("{number:.0}"));
    }
    None
}

fn json_f64_field(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn json_u64_field(value: &Value, key: &str) -> Option<u64> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
}

fn headers_from_json(value: &Value) -> Option<Vec<(String, String)>> {
    let object = value.as_object()?;
    let mut headers = Vec::new();
    for (name, raw_value) in object {
        let Some(value) = raw_value
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        headers.push((name.clone(), value.to_owned()));
    }
    Some(headers)
}

fn format_duration_seconds(seconds: f64) -> String {
    if !seconds.is_finite() || seconds <= 0.0 {
        return "--:--".to_owned();
    }
    let total_seconds = seconds.round() as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

fn prepare_dependency_update_status_is_visible(
    status: ComponentUpdateStatus,
    snapshot_running: bool,
    tool_update_running: bool,
    tool_installed: bool,
) -> bool {
    if tool_update_running {
        return true;
    }

    match status {
        ComponentUpdateStatus::Installed | ComponentUpdateStatus::UpToDate => true,
        ComponentUpdateStatus::Failed => !tool_installed,
        ComponentUpdateStatus::Missing | ComponentUpdateStatus::UpdateAvailable => {
            snapshot_running && !tool_installed
        }
        _ => false,
    }
}

fn component_update_status_needs_attention_signal(status: ComponentUpdateStatus) -> bool {
    matches!(
        status,
        ComponentUpdateStatus::UpdateAvailable | ComponentUpdateStatus::PendingRestart
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::metadata::default_format_id;

    #[test]
    fn prepare_update_status_hides_stale_failed_for_installed_tool() {
        assert!(!prepare_dependency_update_status_is_visible(
            ComponentUpdateStatus::Failed,
            true,
            false,
            true,
        ));
    }

    #[test]
    fn prepare_update_status_keeps_failed_for_missing_tool() {
        assert!(prepare_dependency_update_status_is_visible(
            ComponentUpdateStatus::Failed,
            true,
            false,
            false,
        ));
    }

    #[test]
    fn prepare_update_status_keeps_running_states_visible() {
        assert!(prepare_dependency_update_status_is_visible(
            ComponentUpdateStatus::Downloading,
            true,
            true,
            true,
        ));
    }

    #[test]
    fn component_update_signal_tracks_update_attention_statuses() {
        assert!(component_update_status_needs_attention_signal(
            ComponentUpdateStatus::UpdateAvailable
        ));
        assert!(component_update_status_needs_attention_signal(
            ComponentUpdateStatus::PendingRestart
        ));
        assert!(!component_update_status_needs_attention_signal(
            ComponentUpdateStatus::UpToDate
        ));
        assert!(!component_update_status_needs_attention_signal(
            ComponentUpdateStatus::Missing
        ));
    }

    #[test]
    fn default_video_format_prefers_highest_resolution() {
        let formats = vec![
            FormatOption::video(
                "low",
                "low",
                MediaKind::Video,
                "640x360",
                "",
                "",
                "mp4",
                "h264",
                "10.00 MB",
            ),
            FormatOption::video(
                "high",
                "high",
                MediaKind::Video,
                "1920x1080",
                "",
                "",
                "mp4",
                "h264",
                "20.00 MB",
            ),
            FormatOption::video(
                "mid",
                "mid",
                MediaKind::Video,
                "1280x720",
                "",
                "",
                "mp4",
                "h264",
                "30.00 MB",
            ),
        ];

        assert_eq!(default_format_id(&formats, &[MediaKind::Video]), "high");
    }

    #[test]
    fn requested_format_still_wins_over_quality_guess() {
        let formats = vec![
            FormatOption::video(
                "requested",
                "requested",
                MediaKind::Video,
                "640x360",
                "",
                "",
                "mp4",
                "h264",
                "10.00 MB",
            ),
            FormatOption::video(
                "high",
                "high",
                MediaKind::Video,
                "1920x1080",
                "",
                "",
                "mp4",
                "h264",
                "20.00 MB",
            ),
        ];

        assert_eq!(
            requested_or_default_format_id(
                &formats,
                &[String::from("requested")],
                &[MediaKind::Video],
            ),
            "requested"
        );
    }

    #[test]
    fn display_file_stem_drops_extension_from_auto_name() {
        assert_eq!(
            display_file_stem(r"download\sample title [abc123].webm"),
            "sample title [abc123]"
        );
    }

    #[test]
    fn first_audio_format_ignores_muxed_formats() {
        let metadata = VideoMetadata {
            formats: vec![
                FormatOption::video(
                    "muxed",
                    "muxed",
                    MediaKind::Muxed,
                    "1280x720",
                    "",
                    "30",
                    "mp4",
                    "h264",
                    "10.00 MB",
                ),
                FormatOption::audio(
                    "audio",
                    "audio",
                    MediaKind::Audio,
                    "48000",
                    "m4a",
                    "aac",
                    "3.00 MB",
                ),
            ],
            ..VideoMetadata::empty_preview()
        };

        assert_eq!(
            first_audio_format_id(Some(&metadata)).as_deref(),
            Some("audio")
        );
    }

    #[test]
    fn music_audio_export_plan_copies_matching_opus_source() {
        let plan = resolve_music_audio_export_plan(
            MusicDownloadFormat::Opus,
            &MusicAudioSourceProfile::from_codec("opus"),
        );
        assert_eq!(plan.ffmpeg_args, vec!["-c:a".to_owned(), "copy".to_owned()]);
    }

    #[test]
    fn music_audio_export_plan_encodes_when_source_codec_differs() {
        let plan = resolve_music_audio_export_plan(
            MusicDownloadFormat::Opus,
            &MusicAudioSourceProfile::from_codec("aac"),
        );
        assert!(plan.ffmpeg_args.iter().any(|arg| arg == "libopus"));
    }

    #[test]
    fn music_audio_export_plan_treats_mp4a_as_aac_for_m4a() {
        let plan = resolve_music_audio_export_plan(
            MusicDownloadFormat::M4aAac,
            &MusicAudioSourceProfile::from_codec("mp4a.40.2"),
        );
        assert_eq!(plan.ffmpeg_args, vec!["-c:a".to_owned(), "copy".to_owned()]);
    }

    #[test]
    fn music_audio_export_plan_uses_source_bitrate_for_opus() {
        let source = MusicAudioSourceProfile {
            acodec: "aac".to_owned(),
            bitrate_kbps: Some(128),
            sample_rate: Some(48_000),
            channels: Some(2),
        };
        let plan = resolve_music_audio_export_plan(MusicDownloadFormat::Opus, &source);
        assert_eq!(
            plan.ffmpeg_args,
            vec![
                "-c:a".to_owned(),
                "libopus".to_owned(),
                "-b:a".to_owned(),
                "128k".to_owned(),
            ]
        );
    }

    #[test]
    fn music_audio_export_plan_reduces_mono_or_narrowband_bitrate() {
        let source = MusicAudioSourceProfile {
            acodec: "aac".to_owned(),
            bitrate_kbps: Some(160),
            sample_rate: Some(22_050),
            channels: Some(1),
        };
        let plan = resolve_music_audio_export_plan(MusicDownloadFormat::Opus, &source);
        assert_eq!(
            plan.ffmpeg_args,
            vec![
                "-c:a".to_owned(),
                "libopus".to_owned(),
                "-b:a".to_owned(),
                "64k".to_owned(),
            ]
        );
    }

    #[test]
    fn music_online_target_selector_prefers_requested_codec_before_fallback() {
        assert!(
            music_online_target_format_selector(MusicDownloadFormat::Opus)
                .starts_with("bestaudio[acodec^=opus]")
        );
        assert!(
            music_online_target_format_selector(MusicDownloadFormat::M4aAac)
                .starts_with("bestaudio[ext=m4a]")
        );
    }

    #[test]
    fn recovered_tool_log_step_does_not_keep_parent_failed() {
        let steps = vec![
            tool_log_step_for_test(ToolLogStatus::Recovered),
            tool_log_step_for_test(ToolLogStatus::Skipped),
            tool_log_step_for_test(ToolLogStatus::Success),
        ];

        assert_eq!(aggregate_tool_log_status(&steps), ToolLogStatus::Success);
    }

    #[test]
    fn unrecovered_failed_tool_log_step_keeps_parent_failed() {
        let steps = vec![
            tool_log_step_for_test(ToolLogStatus::Failed),
            tool_log_step_for_test(ToolLogStatus::Skipped),
            tool_log_step_for_test(ToolLogStatus::Success),
        ];

        assert_eq!(aggregate_tool_log_status(&steps), ToolLogStatus::Failed);
    }

    #[test]
    fn recovered_tool_log_without_later_success_is_recovered_not_failed() {
        let steps = vec![
            tool_log_step_for_test(ToolLogStatus::Recovered),
            tool_log_step_for_test(ToolLogStatus::Skipped),
        ];

        assert_eq!(aggregate_tool_log_status(&steps), ToolLogStatus::Recovered);
    }

    #[test]
    fn cache_summary_counts_only_flat_audio_namespace() {
        let root = std::env::temp_dir().join(format!(
            "yt-dlp-gui-v2-audio-cache-summary-test-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let current = root.join("audio").join("current");
        let legacy = root.join("music-stream").join("legacy");
        fs::create_dir_all(&current).expect("create current audio cache");
        fs::create_dir_all(&legacy).expect("create legacy audio cache");
        fs::write(current.join("audio.bin"), [1u8; 3]).expect("write current cache");
        fs::write(legacy.join("audio.bin"), [1u8; 5]).expect("write legacy cache");

        let summary = calculate_cache_management_summary(&root);

        assert_eq!(summary.music_bytes, 3);

        let _ = fs::remove_dir_all(root);
    }

    fn tool_log_step_for_test(status: ToolLogStatus) -> ToolLogStep {
        ToolLogStep {
            id: 0,
            status,
            tool: String::new(),
            action: String::new(),
            command: String::new(),
            detail: None,
        }
    }
}

struct MusicDownloadJob {
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    source_url: String,
    title: String,
    album_title: String,
    output_dir: PathBuf,
    choice: MusicDownloadChoice,
    source_acodec: String,
    cache_media_path: Option<PathBuf>,
    cover_path: Option<PathBuf>,
    cover_cache_dir: Option<PathBuf>,
    thumbnail_url: String,
    use_cookies: bool,
}

fn run_music_download_worker(
    tool_paths: ToolPaths,
    mut job: MusicDownloadJob,
    tx: Sender<MusicDownloadEvent>,
    child_handle: Arc<Mutex<Option<Child>>>,
    cancel_requested: Arc<AtomicBool>,
) {
    if job.choice.embed_cover && !job.cover_path.as_ref().is_some_and(|path| path.is_file()) {
        job.cover_path = ensure_music_download_cover_path(&job);
    }
    let has_cover =
        job.choice.embed_cover && job.cover_path.as_ref().is_some_and(|path| path.is_file());
    let online_target_available = job.choice.target_format().is_some_and(|format| {
        job.cache_media_path.as_ref().is_some_and(|path| {
            !music_cache_source_matches_target(format, path, &job.source_acodec)
        }) && online_music_target_source_available(&tool_paths, &job, format)
    });
    let source_kind = match job.cache_media_path.as_ref() {
        Some(path) if music_cache_can_be_copied_for_choice(job.choice, path, has_cover) => {
            MusicDownloadSourceKind::CacheCopy
        }
        Some(_) if online_target_available => MusicDownloadSourceKind::YtDlpOnlineTarget,
        Some(_) => MusicDownloadSourceKind::CacheConvert,
        None => MusicDownloadSourceKind::YtDlpDownload,
    };

    let result = match source_kind {
        MusicDownloadSourceKind::CacheCopy => {
            copy_music_cache_output(&job, tx.clone(), &cancel_requested)
        }
        MusicDownloadSourceKind::CacheConvert => convert_music_cache_output(
            &tool_paths,
            &job,
            tx.clone(),
            &child_handle,
            &cancel_requested,
        ),
        MusicDownloadSourceKind::YtDlpOnlineTarget | MusicDownloadSourceKind::YtDlpDownload => {
            download_music_output_with_yt_dlp(
                &tool_paths,
                &job,
                source_kind,
                tx.clone(),
                &child_handle,
                &cancel_requested,
            )
        }
    };

    let result = result.map(|path_text| {
        let output_path = PathBuf::from(&path_text);
        match ensure_music_download_album_metadata_written(
            &tool_paths,
            &job,
            output_path,
            source_kind,
            &tx,
        ) {
            Ok(path) => path.display().to_string(),
            Err(error) => {
                eprintln!("[music-download] album metadata pass skipped: {error}");
                path_text
            }
        }
    });

    let _ = tx.send(MusicDownloadEvent::Finished {
        item_id: job.item_id,
        workflow_id: job.workflow_id,
        source_kind,
        result,
    });
}

fn ensure_music_download_cover_path(job: &MusicDownloadJob) -> Option<PathBuf> {
    if let Some(path) = job.cover_path.as_ref().filter(|path| path.is_file()) {
        return Some(path.clone());
    }
    let dir = job.cover_cache_dir.as_ref()?;
    let url = job.thumbnail_url.trim();
    let cached = first_music_cover_file_in_dir(dir);
    if url.is_empty() {
        return cached;
    }
    if cached.is_some() && cached_music_cover_source_matches(dir, url) {
        return cached;
    }
    download_music_cover_to_dir(url, dir).ok().or(cached)
}

fn first_music_cover_file_in_dir(dir: &Path) -> Option<PathBuf> {
    [
        "cover.jpg",
        "cover.jpeg",
        "cover.png",
        "cover.webp",
        "cover.img",
    ]
    .into_iter()
    .map(|name| dir.join(name))
    .find(|path| path.is_file())
}

fn music_cover_source_url_path(dir: &Path) -> PathBuf {
    dir.join("source_url.txt")
}

fn cached_music_cover_source_matches(dir: &Path, url: &str) -> bool {
    fs::read_to_string(music_cover_source_url_path(dir))
        .map(|value| value.trim() == url.trim())
        .unwrap_or(false)
}

fn remove_cached_music_cover_files(dir: &Path) {
    for name in [
        "cover.jpg",
        "cover.jpeg",
        "cover.png",
        "cover.webp",
        "cover.img",
    ] {
        let _ = fs::remove_file(dir.join(name));
    }
}

fn download_music_cover_to_dir(url: &str, dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(dir)
        .map_err(|error| format!("Could not create music cover cache: {error}"))?;
    let mut response = ureq::get(url)
        .call()
        .map_err(|error| format!("Could not download music cover cache: {error}"))?;
    let status = response.status().as_u16();
    if status >= 400 {
        return Err(format!(
            "Could not download music cover cache: HTTP {status}"
        ));
    }
    let mut reader = response.body_mut().as_reader();
    let mut data = Vec::new();
    reader
        .read_to_end(&mut data)
        .map_err(|error| format!("Could not read music cover cache: {error}"))?;
    if data.is_empty() {
        return Err("Downloaded music cover cache is empty.".to_owned());
    }
    let extension = music_cover_extension_from_bytes(&data);
    let path = dir.join(format!("cover.{extension}"));
    remove_cached_music_cover_files(dir);
    fs::write(&path, data)
        .map_err(|error| format!("Could not write music cover cache: {error}"))?;
    let _ = fs::write(music_cover_source_url_path(dir), url);
    Ok(path)
}

fn music_cover_extension_from_bytes(data: &[u8]) -> &'static str {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if data.starts_with(b"\x89PNG\r\n\x1A\n") {
        "png"
    } else if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        "webp"
    } else {
        "img"
    }
}

fn send_music_tool_command_finished(
    tx: &Sender<MusicDownloadEvent>,
    job: &MusicDownloadJob,
    source_kind: MusicDownloadSourceKind,
    tool: &str,
    action: &str,
    command_line: String,
    success: bool,
) {
    let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
        item_id: job.item_id,
        workflow_id: job.workflow_id,
        source_kind,
        tool: tool.to_owned(),
        action: action.to_owned(),
        command_line,
        success,
    });
}

fn ensure_music_download_requested_extension(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    output_path: PathBuf,
    source_kind: MusicDownloadSourceKind,
    tx: &Sender<MusicDownloadEvent>,
) -> Result<PathBuf, String> {
    let Some(target_format) = job.choice.target_format() else {
        return Ok(output_path);
    };
    let current_ext = output_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if current_ext.eq_ignore_ascii_case(target_format.extension()) {
        return Ok(output_path);
    }
    // yt-dlp should normally return the requested extension, but some extract-audio
    // paths can leave a nearby container behind. Keep this pass generic so cached
    // and direct music downloads still honor the user-facing format choice.
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    if !ffmpeg.is_file() || !output_path.is_file() {
        return Ok(output_path);
    }
    let target_path =
        unique_music_output_path(&job.output_dir, &job.title, target_format.extension());
    let mut command = Command::new(&ffmpeg);
    configure_background_command(&mut command);
    command.arg("-y").arg("-i").arg(&output_path).args(
        resolve_music_audio_export_plan(
            target_format,
            &probe_music_audio_source_profile(tool_paths, &output_path, &job.source_acodec),
        )
        .ffmpeg_args,
    );
    if job.choice.write_tags {
        append_music_metadata_args(&mut command, job);
    }
    command.arg(&target_path);
    let command_line = format_process_command_line(&command);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            send_music_tool_command_finished(
                tx,
                job,
                source_kind,
                "ffmpeg",
                "extension pass",
                command_line,
                false,
            );
            return Err(format!("Could not start FFmpeg music output pass: {error}"));
        }
    };
    if !output.status.success() {
        send_music_tool_command_finished(
            tx,
            job,
            source_kind,
            "ffmpeg",
            "extension pass",
            command_line.clone(),
            false,
        );
        let _ = fs::remove_file(&target_path);
        let detail = String::from_utf8_lossy(&output.stderr)
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("unknown FFmpeg error")
            .to_owned();
        return Err(format!("FFmpeg music output pass failed: {detail}"));
    }
    send_music_tool_command_finished(
        tx,
        job,
        source_kind,
        "ffmpeg",
        "extension pass",
        command_line,
        true,
    );
    let _ = fs::remove_file(&output_path);
    Ok(target_path)
}

fn ensure_music_download_cover_embedded(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    output_path: PathBuf,
    source_kind: MusicDownloadSourceKind,
    tx: &Sender<MusicDownloadEvent>,
) -> Result<PathBuf, String> {
    if !job.choice.embed_cover || !music_output_path_supports_embedded_cover(&output_path) {
        return Ok(output_path);
    }
    let Some(cover_path) = job.cover_path.as_ref().filter(|path| path.is_file()) else {
        return Ok(output_path);
    };
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    if !ffmpeg.is_file() || !output_path.is_file() {
        return Ok(output_path);
    }
    let extension = output_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("m4a");
    let temp_path = output_path.with_extension(format!("cover-pass.{extension}"));
    let mut command = Command::new(&ffmpeg);
    configure_background_command(&mut command);
    command
        .arg("-y")
        .arg("-i")
        .arg(&output_path)
        .arg("-i")
        .arg(cover_path)
        .args(["-map", "0:a:0", "-map", "1:v:0", "-c:a", "copy"])
        .args([
            "-c:v",
            "mjpeg",
            "-disposition:v:0",
            "attached_pic",
            "-metadata:s:v",
            "title=Album cover",
            "-metadata:s:v",
            "comment=Cover (front)",
        ]);
    if job.choice.write_tags {
        append_music_metadata_args(&mut command, job);
    }
    command.arg(&temp_path);
    let command_line = format_process_command_line(&command);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            send_music_tool_command_finished(
                tx,
                job,
                source_kind,
                "ffmpeg",
                "embed cover",
                command_line,
                false,
            );
            return Err(format!("Could not start FFmpeg cover embed pass: {error}"));
        }
    };
    if !output.status.success() {
        send_music_tool_command_finished(
            tx,
            job,
            source_kind,
            "ffmpeg",
            "embed cover",
            command_line.clone(),
            false,
        );
        let _ = fs::remove_file(&temp_path);
        let detail = String::from_utf8_lossy(&output.stderr)
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("unknown FFmpeg error")
            .to_owned();
        return Err(format!("FFmpeg cover embed pass failed: {detail}"));
    }
    fs::remove_file(&output_path).map_err(|error| {
        format!("Could not replace music output after cover embed pass: {error}")
    })?;
    fs::rename(&temp_path, &output_path)
        .map_err(|error| format!("Could not move music output after cover embed pass: {error}"))?;
    send_music_tool_command_finished(
        tx,
        job,
        source_kind,
        "ffmpeg",
        "embed cover",
        command_line,
        true,
    );
    Ok(output_path)
}

fn ensure_music_download_album_metadata_written(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    output_path: PathBuf,
    source_kind: MusicDownloadSourceKind,
    tx: &Sender<MusicDownloadEvent>,
) -> Result<PathBuf, String> {
    if !job.choice.write_tags {
        return Ok(output_path);
    }
    if matches!(
        source_kind,
        MusicDownloadSourceKind::YtDlpDownload | MusicDownloadSourceKind::YtDlpOnlineTarget
    ) && job.album_title.trim().is_empty()
    {
        return Ok(output_path);
    }
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    if !ffmpeg.is_file() || !output_path.is_file() {
        return Ok(output_path);
    }
    let extension = output_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("m4a");
    let temp_path = output_path.with_extension(format!("metadata-pass.{extension}"));
    let mut command = Command::new(&ffmpeg);
    configure_background_command(&mut command);
    command
        .arg("-y")
        .arg("-i")
        .arg(&output_path)
        .args(["-map", "0", "-c", "copy"]);
    append_music_metadata_args(&mut command, job);
    command.arg(&temp_path);
    let command_line = format_process_command_line(&command);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            send_music_tool_command_finished(
                tx,
                job,
                source_kind,
                "ffmpeg",
                "write metadata",
                command_line,
                false,
            );
            return Err(format!(
                "Could not start FFmpeg album metadata pass: {error}"
            ));
        }
    };
    if !output.status.success() {
        send_music_tool_command_finished(
            tx,
            job,
            source_kind,
            "ffmpeg",
            "write metadata",
            command_line.clone(),
            false,
        );
        let _ = fs::remove_file(&temp_path);
        let detail = String::from_utf8_lossy(&output.stderr)
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("unknown FFmpeg error")
            .to_owned();
        return Err(format!("FFmpeg album metadata pass failed: {detail}"));
    }
    fs::remove_file(&output_path)
        .map_err(|error| format!("Could not replace music output after metadata pass: {error}"))?;
    fs::rename(&temp_path, &output_path)
        .map_err(|error| format!("Could not move music output after metadata pass: {error}"))?;
    send_music_tool_command_finished(
        tx,
        job,
        source_kind,
        "ffmpeg",
        "write metadata",
        command_line,
        true,
    );
    Ok(output_path)
}

fn copy_music_cache_output(
    job: &MusicDownloadJob,
    tx: Sender<MusicDownloadEvent>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<String, String> {
    let Some(source) = job.cache_media_path.as_ref() else {
        return Err("Music cache file is missing.".to_owned());
    };
    fs::create_dir_all(&job.output_dir)
        .map_err(|error| format!("Could not create music download folder: {error}"))?;
    let output_extension = music_output_extension_for_choice(job.choice, source);
    let output_path = unique_music_output_path(&job.output_dir, &job.title, &output_extension);
    let _ = tx.send(MusicDownloadEvent::Progress {
        item_id: job.item_id,
        workflow_id: job.workflow_id,
        percent: 5.0,
    });
    if cancel_requested.load(Ordering::Relaxed) {
        return Err(DOWNLOAD_CANCELLED_MESSAGE.to_owned());
    }
    fs::copy(source, &output_path)
        .map_err(|error| format!("Could not copy music cache: {error}"))?;
    let _ = tx.send(MusicDownloadEvent::Progress {
        item_id: job.item_id,
        workflow_id: job.workflow_id,
        percent: 100.0,
    });
    Ok(output_path.display().to_string())
}

fn convert_music_cache_output(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    tx: Sender<MusicDownloadEvent>,
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<String, String> {
    let Some(source) = job.cache_media_path.as_ref() else {
        return Err("Music cache file is missing.".to_owned());
    };
    fs::create_dir_all(&job.output_dir)
        .map_err(|error| format!("Could not create music download folder: {error}"))?;
    let output_extension = music_output_extension_for_choice(job.choice, source);
    let output_path = unique_music_output_path(&job.output_dir, &job.title, &output_extension);
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    if !ffmpeg.is_file() {
        return Err(format!(
            "ffmpeg.exe was not found: {}. Install FFmpeg from Options first.",
            ffmpeg.display()
        ));
    }

    let mut command = Command::new(&ffmpeg);
    configure_background_command(&mut command);
    command.arg("-y").arg("-i").arg(source);
    let has_cover = job.choice.embed_cover
        && job.cover_path.as_ref().is_some_and(|path| path.is_file())
        && music_extension_supports_embedded_cover(
            output_path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default(),
        );
    if has_cover {
        if let Some(cover) = job.cover_path.as_ref() {
            command.arg("-i").arg(cover);
        }
    }
    command.args(["-map", "0:a:0"]);
    if has_cover {
        command.args(["-map", "1:v:0"]);
    }
    let source_profile = probe_music_audio_source_profile(tool_paths, source, &job.source_acodec);
    let audio_args = if let Some(target_format) = job.choice.target_format() {
        resolve_music_audio_export_plan(target_format, &source_profile).ffmpeg_args
    } else {
        vec!["-c:a".to_owned(), "copy".to_owned()]
    };
    command.args(audio_args);
    if has_cover {
        command.args([
            "-c:v",
            "mjpeg",
            "-disposition:v:0",
            "attached_pic",
            "-metadata:s:v",
            "title=Album cover",
            "-metadata:s:v",
            "comment=Cover (front)",
        ]);
    }
    if job.choice.write_tags {
        append_music_metadata_args(&mut command, job);
    }
    command.arg(&output_path);
    let command_line = format_process_command_line(&command);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let _ = tx.send(MusicDownloadEvent::Progress {
        item_id: job.item_id,
        workflow_id: job.workflow_id,
        percent: 10.0,
    });
    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start FFmpeg music conversion: {error}"))?;
    let stderr = child.stderr.take();
    if let Ok(mut guard) = child_handle.lock() {
        *guard = Some(child);
    }
    let stderr_handle = stderr.map(|stderr| {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            reader.lines().map_while(Result::ok).collect::<Vec<_>>()
        })
    });
    let status = wait_music_child(child_handle, cancel_requested);
    let stderr_lines = stderr_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    if cancel_requested.load(Ordering::Relaxed) {
        let _ = fs::remove_file(&output_path);
        let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
            item_id: job.item_id,
            workflow_id: job.workflow_id,
            source_kind: MusicDownloadSourceKind::CacheConvert,
            tool: "ffmpeg".to_owned(),
            action: "convert".to_owned(),
            command_line,
            success: false,
        });
        return Err(DOWNLOAD_CANCELLED_MESSAGE.to_owned());
    }
    match status {
        Some(Ok(status)) if status.success() => {
            let _ = tx.send(MusicDownloadEvent::Progress {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                percent: 100.0,
            });
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind: MusicDownloadSourceKind::CacheConvert,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line,
                success: true,
            });
            Ok(output_path.display().to_string())
        }
        Some(Ok(status)) => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind: MusicDownloadSourceKind::CacheConvert,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line: command_line.clone(),
                success: false,
            });
            let detail = stderr_lines
                .iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| format!("exit code {:?}", status.code()));
            Err(format!("FFmpeg music conversion failed: {detail}"))
        }
        Some(Err(error)) => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind: MusicDownloadSourceKind::CacheConvert,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line: command_line.clone(),
                success: false,
            });
            Err(format!(
                "Could not wait for FFmpeg music conversion: {error}"
            ))
        }
        None => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind: MusicDownloadSourceKind::CacheConvert,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line,
                success: false,
            });
            Err("Could not wait for FFmpeg music conversion: child process missing".to_owned())
        }
    }
}

fn download_music_output_with_yt_dlp(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    source_kind: MusicDownloadSourceKind,
    tx: Sender<MusicDownloadEvent>,
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<String, String> {
    let prepared = tool_paths.prepare_music_audio_download_command(
        &job.source_url,
        &job.output_dir,
        job.choice
            .target_format()
            .map(MusicDownloadFormat::yt_dlp_audio_format),
        job.choice.format_selector(),
        job.choice.embed_cover,
        job.choice.write_tags,
        job.use_cookies,
    )?;
    println!(
        "[music-download] output: {}",
        prepared.output_path.display()
    );
    println!("[music-download] command: {}", prepared.command_line);
    let PreparedDownload {
        mut command,
        output_path,
        command_line,
    } = prepared;

    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start yt-dlp music download: {error}"))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    if let Ok(mut guard) = child_handle.lock() {
        *guard = Some(child);
    }

    let item_id = job.item_id;
    let workflow_id = job.workflow_id;
    let stdout_handle = stdout.map(|stdout| {
        let tx = tx.clone();
        thread::spawn(move || read_music_yt_dlp_stream(stdout, item_id, workflow_id, tx))
    });
    let stderr_handle = stderr.map(|stderr| {
        let tx = tx.clone();
        thread::spawn(move || read_music_yt_dlp_stream(stderr, item_id, workflow_id, tx))
    });

    let status = wait_music_child(child_handle, cancel_requested);
    let mut lines = stdout_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    lines.extend(
        stderr_handle
            .and_then(|handle| handle.join().ok())
            .unwrap_or_default(),
    );

    if cancel_requested.load(Ordering::Relaxed) {
        let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
            item_id: job.item_id,
            workflow_id: job.workflow_id,
            source_kind,
            tool: "yt-dlp".to_owned(),
            action: "download".to_owned(),
            command_line: command_line.clone(),
            success: false,
        });
        return Err(DOWNLOAD_CANCELLED_MESSAGE.to_owned());
    }

    match status {
        Some(Ok(status)) if status.success() => {
            let output_path = reported_music_final_output_path(&lines)
                .or_else(|| {
                    find_latest_music_download_output_for_choice(&job.output_dir, job.choice)
                })
                .unwrap_or(output_path);
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind,
                tool: "yt-dlp".to_owned(),
                action: "download".to_owned(),
                command_line: command_line.clone(),
                success: true,
            });
            let output_path = match ensure_music_download_requested_extension(
                &tool_paths,
                job,
                output_path.clone(),
                source_kind,
                &tx,
            ) {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("[music-download] requested extension pass skipped: {error}");
                    output_path
                }
            };
            let output_path = match ensure_music_download_cover_embedded(
                &tool_paths,
                job,
                output_path.clone(),
                source_kind,
                &tx,
            ) {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("[music-download] cover embed pass skipped: {error}");
                    output_path
                }
            };
            let _ = tx.send(MusicDownloadEvent::Progress {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                percent: 100.0,
            });
            Ok(output_path.display().to_string())
        }
        Some(Ok(status)) => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind,
                tool: "yt-dlp".to_owned(),
                action: "download".to_owned(),
                command_line: command_line.clone(),
                success: false,
            });
            let detail = lines
                .iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| format!("exit code {:?}", status.code()));
            Err(format!("yt-dlp music download failed: {detail}"))
        }
        Some(Err(error)) => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind,
                tool: "yt-dlp".to_owned(),
                action: "download".to_owned(),
                command_line: command_line.clone(),
                success: false,
            });
            Err(format!("Could not wait for yt-dlp music download: {error}"))
        }
        None => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind,
                tool: "yt-dlp".to_owned(),
                action: "download".to_owned(),
                command_line,
                success: false,
            });
            Err("Could not wait for yt-dlp music download: child process missing".to_owned())
        }
    }
}

fn read_music_yt_dlp_stream<R: Read>(
    reader: R,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: Sender<MusicDownloadEvent>,
) -> Vec<String> {
    let mut reader = BufReader::new(reader);
    let mut lines = Vec::new();
    let mut pending = Vec::new();
    let mut chunk = [0_u8; 4096];

    loop {
        let bytes_read = match std::io::Read::read(&mut reader, &mut chunk) {
            Ok(size) => size,
            Err(_) => break,
        };
        if bytes_read == 0 {
            break;
        }

        for &byte in &chunk[..bytes_read] {
            if matches!(byte, b'\n' | b'\r') {
                process_music_yt_dlp_line(&pending, item_id, workflow_id, &tx, &mut lines);
                pending.clear();
            } else {
                pending.push(byte);
            }
        }
    }

    if !pending.is_empty() {
        process_music_yt_dlp_line(&pending, item_id, workflow_id, &tx, &mut lines);
    }

    lines
}

fn process_music_yt_dlp_line(
    bytes: &[u8],
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: &Sender<MusicDownloadEvent>,
    lines: &mut Vec<String>,
) {
    let line = String::from_utf8_lossy(bytes).trim().to_owned();
    if line.is_empty() {
        return;
    }

    if let Some(percent) = parse_music_yt_dlp_progress_percent(&line) {
        let _ = tx.send(MusicDownloadEvent::Progress {
            item_id,
            workflow_id,
            percent,
        });
    }
    lines.push(line);
}

fn parse_music_yt_dlp_progress_percent(line: &str) -> Option<f32> {
    parse_music_progress_template_percent(line).or_else(|| parse_default_download_percent(line))
}

fn parse_music_progress_template_percent(line: &str) -> Option<f32> {
    let value = line
        .trim()
        .strip_prefix("[yt-dlp],")?
        .split(',')
        .next()?
        .trim();
    parse_percent_text(value)
}

fn parse_default_download_percent(line: &str) -> Option<f32> {
    let body = line.trim().strip_prefix("[download]")?.trim_start();
    if body.starts_with("Destination:") {
        return None;
    }

    body.split_whitespace()
        .find_map(|part| parse_percent_text(part.trim()))
}

fn parse_percent_text(value: &str) -> Option<f32> {
    value
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .ok()
        .map(|value| value.clamp(0.0, 100.0))
}

fn reported_music_final_output_path(lines: &[String]) -> Option<PathBuf> {
    lines.iter().rev().find_map(|line| {
        let payload = line.trim().strip_prefix(FINAL_OUTPUT_PATH_PREFIX)?.trim();
        let parsed = serde_json::from_str::<String>(payload)
            .unwrap_or_else(|_| payload.trim_matches('"').to_owned());
        let trimmed = parsed.trim();
        (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
    })
}

fn wait_music_child(
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Option<std::io::Result<std::process::ExitStatus>> {
    loop {
        if cancel_requested.load(Ordering::Relaxed) {
            if let Ok(mut guard) = child_handle.lock() {
                if let Some(child) = guard.as_mut() {
                    let _ = child.kill();
                }
            }
        }
        if let Ok(mut guard) = child_handle.lock() {
            let Some(child) = guard.as_mut() else {
                return None;
            };
            match child.try_wait() {
                Ok(Some(status)) => {
                    *guard = None;
                    return Some(Ok(status));
                }
                Ok(None) => {}
                Err(error) => {
                    *guard = None;
                    return Some(Err(error));
                }
            }
        } else {
            return None;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn unique_music_output_path(output_dir: &Path, title: &str, extension: &str) -> PathBuf {
    let stem = sanitize_file_name_for_windows(&music_output_stem_for_title(title));
    let base = if stem.trim().is_empty() {
        "music"
    } else {
        stem.trim()
    };
    let extension = extension.trim().trim_start_matches('.');
    let mut path = output_dir.join(format!("{base}.{extension}"));
    if !path.exists() {
        return path;
    }
    for index in 2..10_000 {
        path = output_dir.join(format!("{base} ({index}).{extension}"));
        if !path.exists() {
            return path;
        }
    }
    output_dir.join(format!(
        "{base}.{}.{}",
        unique_timestamp_suffix(),
        extension
    ))
}

fn find_latest_music_download_output_for_choice(
    dir: &Path,
    choice: MusicDownloadChoice,
) -> Option<PathBuf> {
    if let Some(format) = choice.target_format() {
        return find_latest_file_in_dir(dir, format.extension());
    }
    find_latest_music_original_output(dir)
}

fn find_latest_music_original_output(dir: &Path) -> Option<PathBuf> {
    const AUDIO_EXTENSIONS: [&str; 10] = [
        "m4a", "webm", "opus", "mp3", "aac", "flac", "wav", "ogg", "oga", "mka",
    ];
    AUDIO_EXTENSIONS
        .into_iter()
        .filter_map(|extension| find_latest_file_in_dir(dir, extension))
        .max_by_key(|path| {
            path.metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
        })
}

fn find_latest_file_in_dir(dir: &Path, extension: &str) -> Option<PathBuf> {
    let extension = extension.trim().trim_start_matches('.');
    fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(extension))
        })
        .max_by_key(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok())
}

fn music_output_stem_template_for_title(title: &str) -> String {
    let (artist, title) = split_artist_title(title);
    match artist {
        Some(artist) if !artist.trim().is_empty() => format!("{artist} - {title}"),
        _ => title.to_owned(),
    }
}

fn music_output_stem_for_title(title: &str) -> String {
    music_output_stem_template_for_title(title)
}

fn split_artist_title(title: &str) -> (Option<String>, String) {
    let trimmed = title.trim();
    if let Some((artist, title)) = trimmed.split_once(" - ") {
        let artist = artist.trim();
        let title = title.trim();
        if !artist.is_empty() && !title.is_empty() {
            return (Some(artist.to_owned()), title.to_owned());
        }
    }
    (None, trimmed.to_owned())
}

fn append_music_metadata_args(command: &mut Command, job: &MusicDownloadJob) {
    let (artist, title) = split_artist_title(&job.title);
    if let Some(artist) = artist
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        command.arg("-metadata").arg(format!("artist={artist}"));
    }
    let title = title.trim();
    if !title.is_empty() {
        command.arg("-metadata").arg(format!("title={title}"));
    }
    let album = job.album_title.trim();
    if !album.is_empty() {
        command.arg("-metadata").arg(format!("album={album}"));
    }
}

fn music_download_tool_kind(source_kind: MusicDownloadSourceKind) -> ToolKind {
    match source_kind {
        MusicDownloadSourceKind::CacheCopy => ToolKind::Other("cache".to_owned()),
        MusicDownloadSourceKind::CacheConvert => ToolKind::Ffmpeg,
        MusicDownloadSourceKind::YtDlpOnlineTarget | MusicDownloadSourceKind::YtDlpDownload => {
            ToolKind::YtDlp
        }
    }
}

fn music_cache_can_be_copied_for_choice(
    choice: MusicDownloadChoice,
    path: &Path,
    has_cover: bool,
) -> bool {
    if has_cover && music_output_path_supports_embedded_cover(path) {
        return false;
    }
    match choice.target_format() {
        Some(format) => music_download_format_matches_cache(format, path),
        None => true,
    }
}

fn music_download_format_matches_cache(format: MusicDownloadFormat, path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    let ext = ext.trim().trim_start_matches('.');
    ext.eq_ignore_ascii_case(format.extension())
}

fn music_output_extension_for_choice(choice: MusicDownloadChoice, source_path: &Path) -> String {
    choice
        .target_format()
        .map(MusicDownloadFormat::extension)
        .or_else(|| {
            source_path
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.trim().trim_start_matches('.'))
                .filter(|value| !value.is_empty())
        })
        .unwrap_or("audio")
        .to_owned()
}

fn music_output_path_supports_embedded_cover(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(music_extension_supports_embedded_cover)
}

fn music_extension_supports_embedded_cover(extension: &str) -> bool {
    matches!(
        extension
            .trim()
            .trim_start_matches('.')
            .to_ascii_lowercase()
            .as_str(),
        "mp3" | "m4a" | "flac"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MusicAudioQualityIntent {
    PreservePerceivedQuality,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct MusicAudioSourceProfile {
    acodec: String,
    bitrate_kbps: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u32>,
}

impl MusicAudioSourceProfile {
    fn from_codec(source_acodec: &str) -> Self {
        Self {
            acodec: source_acodec.to_owned(),
            bitrate_kbps: None,
            sample_rate: None,
            channels: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MusicAudioExportPlan {
    ffmpeg_args: Vec<String>,
}

fn resolve_music_audio_export_plan(
    format: MusicDownloadFormat,
    source: &MusicAudioSourceProfile,
) -> MusicAudioExportPlan {
    // Audio export follows a conservative source-aware model:
    // 1. never re-encode when the source codec already matches the target,
    // 2. prefer the online source selected by yt-dlp when it already exists,
    // 3. only then encode with a preserve-perceived-quality heuristic.
    //
    // The heuristic intentionally avoids exposing bitrate/sample-rate knobs in the UI.
    // It is based on public codec recommendations and listening-test consensus, not on
    // a promise of mathematically lossless output for lossy transcodes.
    if music_source_codec_matches_target_format(format, &source.acodec) {
        return MusicAudioExportPlan {
            ffmpeg_args: vec!["-c:a".to_owned(), "copy".to_owned()],
        };
    }

    MusicAudioExportPlan {
        ffmpeg_args: encode_music_audio_args_for_intent(
            format,
            MusicAudioQualityIntent::PreservePerceivedQuality,
            source,
        ),
    }
}

fn encode_music_audio_args_for_intent(
    format: MusicDownloadFormat,
    intent: MusicAudioQualityIntent,
    source: &MusicAudioSourceProfile,
) -> Vec<String> {
    match intent {
        MusicAudioQualityIntent::PreservePerceivedQuality => match format {
            MusicDownloadFormat::Mp3 => vec![
                "-c:a".to_owned(),
                "libmp3lame".to_owned(),
                "-q:a".to_owned(),
                mp3_quality_for_source(source).to_string(),
            ],
            MusicDownloadFormat::M4aAac => vec![
                "-c:a".to_owned(),
                "aac".to_owned(),
                "-b:a".to_owned(),
                format!("{}k", lossy_bitrate_for_source(source, 160, 192, 256, 320)),
            ],
            MusicDownloadFormat::Opus => vec![
                "-c:a".to_owned(),
                "libopus".to_owned(),
                "-b:a".to_owned(),
                format!("{}k", lossy_bitrate_for_source(source, 96, 128, 160, 192)),
            ],
            MusicDownloadFormat::Flac => vec![
                "-c:a".to_owned(),
                "flac".to_owned(),
                "-compression_level".to_owned(),
                "8".to_owned(),
            ],
            MusicDownloadFormat::Wav => {
                vec!["-c:a".to_owned(), "pcm_s16le".to_owned(), "-vn".to_owned()]
            }
        },
    }
}

fn mp3_quality_for_source(source: &MusicAudioSourceProfile) -> u8 {
    match source.bitrate_kbps {
        Some(value) if value <= 96 => 5,
        Some(value) if value <= 160 => 3,
        Some(value) if value <= 224 => 2,
        _ => 0,
    }
}

fn lossy_bitrate_for_source(
    source: &MusicAudioSourceProfile,
    low: u32,
    mid: u32,
    high: u32,
    max: u32,
) -> u32 {
    let Some(source_bitrate) = source.bitrate_kbps else {
        return high;
    };
    let mono_or_narrowband = source.channels == Some(1)
        || source
            .sample_rate
            .is_some_and(|sample_rate| sample_rate <= 24_000);
    let selected = if source_bitrate <= 96 {
        low
    } else if source_bitrate <= 160 {
        mid
    } else if source_bitrate <= 256 {
        high
    } else {
        max
    };
    if mono_or_narrowband {
        (selected / 2).max(64)
    } else {
        selected
    }
}

fn probe_music_audio_source_profile(
    tool_paths: &ToolPaths,
    input_path: &Path,
    fallback_acodec: &str,
) -> MusicAudioSourceProfile {
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    let ffprobe = ffprobe_companion_path_for_ffmpeg(&ffmpeg);
    let mut profile = MusicAudioSourceProfile::from_codec(fallback_acodec);
    let Ok(info) = probe_media_with_ffprobe(&ffprobe, input_path) else {
        return profile;
    };
    let Some(audio) = info.audio else {
        return profile;
    };
    if let Some(codec) = audio.codec.filter(|value| !value.trim().is_empty()) {
        profile.acodec = codec;
    }
    profile.bitrate_kbps = audio
        .bitrate_bps
        .map(|value| ((value as f64) / 1000.0).round().max(1.0) as u32);
    profile.sample_rate = audio.sample_rate;
    profile.channels = audio.channels;
    profile
}

fn music_cache_source_matches_target(
    format: MusicDownloadFormat,
    path: &Path,
    source_acodec: &str,
) -> bool {
    music_download_format_matches_cache(format, path)
        || music_source_codec_matches_target_format(format, source_acodec)
}

fn online_music_target_source_available(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    format: MusicDownloadFormat,
) -> bool {
    let selector = music_online_target_format_selector(format);
    let Ok(output) = tool_paths.analyze_music_stream_url_with_selector(
        &job.source_url,
        job.use_cookies,
        selector,
    ) else {
        return false;
    };
    let Ok(seed) = music_stream_seed_from_json(&output.json, &job.source_url) else {
        return false;
    };
    music_source_codec_matches_target_format(format, &seed.acodec)
}

fn music_online_target_format_selector(format: MusicDownloadFormat) -> &'static str {
    match format {
        MusicDownloadFormat::Mp3 => {
            "bestaudio[ext=mp3]/bestaudio[acodec^=mp3]/bestaudio/best[acodec!=none]"
        }
        MusicDownloadFormat::M4aAac => {
            "bestaudio[ext=m4a]/bestaudio[acodec^=mp4a]/bestaudio[acodec^=aac]/bestaudio/best[acodec!=none]"
        }
        MusicDownloadFormat::Opus => {
            "bestaudio[acodec^=opus]/bestaudio[ext=opus]/bestaudio[ext=webm][acodec^=opus]/bestaudio/best[acodec!=none]"
        }
        MusicDownloadFormat::Flac => {
            "bestaudio[ext=flac]/bestaudio[acodec^=flac]/bestaudio/best[acodec!=none]"
        }
        MusicDownloadFormat::Wav => {
            "bestaudio[ext=wav]/bestaudio[acodec^=pcm]/bestaudio/best[acodec!=none]"
        }
    }
}

fn music_source_codec_matches_target_format(
    format: MusicDownloadFormat,
    source_acodec: &str,
) -> bool {
    let normalized = normalize_music_source_codec(source_acodec);
    if normalized.is_empty() || normalized == "none" {
        return false;
    }
    match format {
        MusicDownloadFormat::Mp3 => normalized == "mp3",
        MusicDownloadFormat::M4aAac => normalized == "aac",
        MusicDownloadFormat::Opus => normalized == "opus",
        MusicDownloadFormat::Flac => normalized == "flac",
        MusicDownloadFormat::Wav => normalized.starts_with("pcm_"),
    }
}

fn normalize_music_source_codec(source_acodec: &str) -> String {
    let codec = source_acodec.trim().to_ascii_lowercase();
    if codec.starts_with("mp4a") || codec == "aac_latm" {
        "aac".to_owned()
    } else if codec.starts_with("opus") {
        "opus".to_owned()
    } else if codec.starts_with("mp3") || codec == "libmp3lame" {
        "mp3".to_owned()
    } else if codec.starts_with("flac") {
        "flac".to_owned()
    } else {
        codec
    }
}

impl MusicDownloadFormat {
    fn yt_dlp_audio_format(self) -> &'static str {
        self.extension()
    }
}

fn unique_timestamp_suffix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn music_cache_updated_is_fresh(updated_unix_seconds: u64) -> bool {
    updated_unix_seconds > 0
        && unique_timestamp_suffix().saturating_sub(updated_unix_seconds)
            <= MUSIC_STREAM_CACHE_TTL_SECONDS
}

fn audio_cache_manifest_is_fresh(manifest: &AudioCacheManifestSnapshot) -> bool {
    music_cache_updated_is_fresh(manifest.updated_unix_seconds)
}

fn sanitize_music_cache_key(value: &str) -> String {
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

fn music_cache_manifest_progress_ratio(
    manifest_path: &Path,
    fallback_expected_bytes: Option<u64>,
) -> Option<f32> {
    let manifest = read_yaml_file::<AudioCacheManifestSnapshot>(manifest_path)?;
    if !audio_cache_manifest_is_fresh(&manifest) {
        return None;
    }
    if manifest.complete {
        return Some(1.0);
    }
    let expected = manifest
        .expected_bytes
        .or(fallback_expected_bytes)
        .filter(|value| *value > 0)?;
    let range_bytes = manifest
        .ranges
        .iter()
        .map(|range| range.end.saturating_sub(range.start))
        .sum::<u64>();
    let downloaded = (range_bytes > 0)
        .then_some(range_bytes)
        .or(manifest.downloaded_bytes)?;
    Some((downloaded as f32 / expected as f32).clamp(0.0, 1.0))
}

fn sanitize_music_cache_ext(value: &str) -> String {
    let cleaned = value.trim().trim_start_matches('.');
    if cleaned.is_empty() {
        "bin".to_owned()
    } else {
        cleaned
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase()
    }
}

fn calculate_cache_management_summary(root: &Path) -> CacheManagementSummary {
    let total_bytes = dir_size_bytes(root);
    let music_root = root.join("audio");
    let music_bytes = dir_size_bytes(&music_root);
    let expired_music_bytes = expired_music_cache_size_bytes(&music_root);
    CacheManagementSummary {
        total_bytes,
        music_bytes,
        expired_music_bytes,
    }
}

fn dir_size_bytes(path: &Path) -> u64 {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return 0;
    };
    if metadata.is_file() {
        return metadata.len();
    }
    if !metadata.is_dir() {
        return 0;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| dir_size_bytes(&entry.path()))
        .sum()
}

fn expired_music_cache_size_bytes(root: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(root) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| music_cache_dir_is_expired(path))
        .map(|path| dir_size_bytes(&path))
        .sum()
}

fn remove_expired_music_cache_dirs(root: &Path) -> std::io::Result<CacheRemovalSummary> {
    let mut summary = CacheRemovalSummary::default();
    let Ok(entries) = fs::read_dir(root) else {
        return Ok(summary);
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !music_cache_dir_is_expired(&path) {
            continue;
        }
        let bytes = dir_size_bytes(&path);
        remove_path(&path)?;
        summary.bytes = summary.bytes.saturating_add(bytes);
        summary.entries = summary.entries.saturating_add(1);
    }
    Ok(summary)
}

fn music_cache_dir_is_expired(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    if path.file_name().and_then(|value| value.to_str()) == Some("covers") {
        return false;
    }

    let manifest_path = path.join("manifest.yaml");
    if let Some(manifest) = read_yaml_file::<AudioCacheManifestSnapshot>(&manifest_path) {
        return !audio_cache_manifest_is_fresh(&manifest);
    }

    path_modified_age_seconds(path).is_some_and(|age| age > MUSIC_STREAM_CACHE_TTL_SECONDS)
}

fn path_modified_age_seconds(path: &Path) -> Option<u64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    SystemTime::now()
        .duration_since(modified)
        .ok()
        .map(|duration| duration.as_secs())
}

fn remove_path_contents_or_dir(path: &Path) -> std::io::Result<CacheRemovalSummary> {
    let summary = CacheRemovalSummary {
        bytes: dir_size_bytes(path),
        entries: if path.exists() { 1 } else { 0 },
    };
    if path.exists() {
        remove_path(path)?;
    }
    Ok(summary)
}

fn remove_safe_app_cache_contents(path: &Path) -> std::io::Result<CacheRemovalSummary> {
    ensure_safe_app_cache_root(path)?;
    remove_dir_contents_collecting_summary(path)
}

fn ensure_safe_app_cache_root(path: &Path) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};

    let path = normalized_path_for_safety(path);
    if path.parent().is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Refusing to clear a filesystem root as cache.",
        ));
    }

    let mut protected_roots = Vec::new();
    if let Ok(current_dir) = std::env::current_dir() {
        protected_roots.push(normalized_path_for_safety(&current_dir));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            protected_roots.push(normalized_path_for_safety(parent));
        }
    }
    for var in ["USERPROFILE", "HOME"] {
        if let Ok(home) = std::env::var(var) {
            let home = normalized_path_for_safety(Path::new(&home));
            protected_roots.push(home.clone());
            for child in [
                "Desktop",
                "Downloads",
                "Documents",
                "Pictures",
                "Videos",
                "Music",
            ] {
                protected_roots.push(normalized_path_for_safety(&home.join(child)));
            }
        }
    }

    if protected_roots.iter().any(|protected| protected == &path) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Refusing to clear a protected folder as cache. Choose an app-owned cache folder.",
        ));
    }

    Ok(())
}

fn cookie_rescue_cookie_dir_path() -> PathBuf {
    app_portable_root_path().join("data").join("cookies")
}

fn app_portable_root_path() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    }

    #[cfg(not(debug_assertions))]
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn normalized_path_for_safety(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    absolute.canonicalize().unwrap_or(absolute)
}

fn remove_dir_contents_collecting_summary(path: &Path) -> std::io::Result<CacheRemovalSummary> {
    let mut summary = CacheRemovalSummary::default();
    let Ok(entries) = fs::read_dir(path) else {
        return Ok(summary);
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let bytes = dir_size_bytes(&path);
        remove_path(&path)?;
        summary.bytes = summary.bytes.saturating_add(bytes);
        summary.entries = summary.entries.saturating_add(1);
    }
    Ok(summary)
}

fn remove_path(path: &Path) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn format_byte_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0_usize;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
