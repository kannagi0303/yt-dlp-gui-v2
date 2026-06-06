pub(crate) mod app_identity;
mod component_update;
mod config;
pub(crate) mod cookie_site_index;
mod media_session;
mod notification;
mod output_actions;
mod portable_cleanup;
mod prepare_check;
mod sha256;
mod tool_install;
mod tools;
pub(crate) mod yaml_store;
mod youtube_login_rescue;

pub use component_update::{
    AppInstanceGuard, ComponentOwnership, ComponentUpdateAction, ComponentUpdateEntry,
    ComponentUpdateEvent, ComponentUpdateSnapshot, ComponentUpdateStatus, ManagedComponentId,
    apply_update_args_requested, cleanup_applied_update, component_update_startup_snapshot,
    launch_pending_app_update, parse_apply_update_args, register_app_instance,
    resume_pending_app_update_on_launch, run_apply_update, run_component_update_action,
};
pub use config::{
    AppConfig, AudioPolicy, CompatibilityTarget, ConfigFileOption, ContainerPolicy,
    FrameRatePolicy, OutputFileActionMode, PostProcessMode, ResolutionPolicy,
    SerializableCacheLocationMode, SubtitlePolicy, ThemeAccentColor, ThemeMode,
    TranscodeIntentMode, TranscodeIntentSettings, TranscodeSettingKey, VideoCodecPolicy,
    WindowPosition, WindowSize, YoutubeVideoPlaylistMode, available_yt_dlp_config_files,
    normalize_ui_scale_percent, yt_dlp_configs_dir_display,
};
pub use media_session::{
    MediaSession, MediaSessionCommand, MediaSessionPlaybackStatus, MediaSessionTimeline,
    MediaSessionTrack,
};
pub use notification::{send_download_failed_windows_toast, send_download_finished_windows_toast};

pub use output_actions::{
    open_output_file, open_output_folder, output_file_exists, output_parent_folder_exists,
};
pub use portable_cleanup::schedule_startup_transient_cleanup;
pub use prepare_check::{
    PrepareReport, PrepareRequirement, PrepareStatus, collect_dependency_presence_report,
    collect_prepare_report,
};
pub use tool_install::{
    DependencyTool, dependency_tool_exists, dependency_tool_is_available,
    detect_dependency_tool_in_system_path,
};
pub use tools::{
    AnalyzeError, AnalyzeOutput, BrowserCookieProfileOption, BrowserCookieSourceOption,
    CacheLocationMode, DownloadRequest, DownloadTargetKind, FINAL_OUTPUT_PATH_PREFIX, FileTimeMode,
    PreparedDownload, ToolPaths, YoutubePlaylistRisk, classify_youtube_playlist,
    configure_background_command, display_output_dir, humanize_yt_dlp_error,
    is_windows_known_folder_segment, looks_like_playlist_url, playlist_entry_url,
    resolve_output_dir, resolve_tool_path, youtube_url_force_single_video,
    youtube_url_has_video_and_playlist,
};
pub use youtube_login_rescue::{
    YoutubeLoginRescueBrowserInfo, YoutubeLoginRescueEvent,
    detect_default_youtube_login_rescue_browser, normalize_cookie_rescue_target_url,
    run_youtube_login_rescue_cookie_export,
};
