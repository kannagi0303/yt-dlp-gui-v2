mod config;
mod notification;
mod output_actions;
mod prepare_check;
mod tool_install;
mod tools;

pub use config::{
    AppConfig, AudioPolicy, CompatibilityTarget, ConfigFileOption, ContainerPolicy, FrameRatePolicy,
    OutputFileActionMode, ResolutionPolicy, SerializableCacheLocationMode, SubtitlePolicy,
    ThemeAccentColor, ThemeMode, TranscodeIntentMode,
    TranscodeIntentSettings, TranscodeSettingKey, VideoCodecPolicy, WindowPosition, WindowSize,
    YoutubeVideoPlaylistMode, available_yt_dlp_config_files, normalize_ui_scale_percent,
    yt_dlp_configs_dir_display,
};
pub use notification::{send_download_failed_windows_toast, send_download_finished_windows_toast};
pub use output_actions::{
    open_output_file, open_output_folder, output_file_exists, output_parent_folder_exists,
};
pub use prepare_check::{
    PrepareAction, PrepareReport, PrepareRequirement, PrepareSeverity, PrepareStatus,
    collect_prepare_report,
};
pub use tool_install::{
    DependencyTool, ToolInstallCancelHandle, ToolInstallProgress, ToolInstallStage,
    dependency_tool_exists, dependency_tool_is_available,
    install_dependency_tool_with_progress_using_proxy,
};
pub use tools::{
    BrowserCookieProfileOption, BrowserCookieSourceOption, CacheLocationMode, DownloadRequest,
    DownloadTargetKind, FINAL_OUTPUT_PATH_PREFIX, FileTimeMode, PreparedDownload, ToolPaths,
    YoutubePlaylistRisk, classify_youtube_playlist, configure_background_command,
    display_output_dir, humanize_yt_dlp_error, is_windows_known_folder_segment,
    looks_like_playlist_url, playlist_entry_url, resolve_output_dir,
    resolve_tool_path, youtube_url_force_single_video, youtube_url_has_video_and_playlist,
};
