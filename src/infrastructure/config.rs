use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::i18n::LanguageSelection;

use super::tools::{CacheLocationMode, FileTimeMode, ToolPaths};

#[derive(Clone, Debug)]
pub struct ConfigFileOption {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(alias = "TargetPath")]
    pub download_dir: String,
    #[serde(alias = "PathTEMP")]
    pub cache_dir: String,
    pub cache_location_mode: SerializableCacheLocationMode,
    #[serde(alias = "PathYTDLP")]
    pub yt_dlp_path: String,
    pub yt_dlp_config_path: String,
    #[serde(alias = "PathFFMPEG")]
    pub ffmpeg_path: String,
    #[serde(alias = "PathAria2")]
    pub aria2c_path: String,
    #[serde(alias = "PathDeno")]
    pub deno_path: String,
    pub use_browser_cookies: bool,
    pub browser_cookie_source: String,
    pub browser_cookie_profile: String,
    pub browser_cookie_file: String,
    pub youtube_subs_po_token: String,
    pub youtube_extractor_args: String,
    pub concurrent_fragments: usize,
    #[serde(alias = "ProxyEnabled")]
    pub proxy_enabled: bool,
    #[serde(alias = "ProxyUrl")]
    pub proxy_url: String,
    pub no_check_certificates: bool,
    #[serde(alias = "LimitRate")]
    pub limit_rate: String,
    #[serde(alias = "TimeRange")]
    pub download_sections: String,
    pub chapter_compatibility_mode: bool,
    #[serde(alias = "ModifiedType")]
    pub file_time_mode: FileTimeMode,
    #[serde(alias = "UseAria2")]
    pub use_aria2: bool,
    #[serde(alias = "SaveThumbnail")]
    pub write_thumbnail: bool,
    #[serde(alias = "EmbedThumbnail")]
    pub embed_thumbnail: bool,
    pub write_subtitles: bool,
    #[serde(alias = "EmbedSubtitles")]
    pub embed_subtitles: bool,
    pub write_chapters: bool,
    pub embed_chapters: bool,
    #[serde(alias = "IsMonitor")]
    pub auto_paste_clipboard: bool,
    pub clipboard_auto_add: bool,
    #[serde(alias = "AutoDownloadAnalysed")]
    pub auto_analyze: bool,
    pub direct_download_on_add: bool,
    pub output_file_action_mode: OutputFileActionMode,
    pub batch_limit_enabled: bool,
    pub batch_limit_count: usize,
    pub youtube_video_playlist_mode: YoutubeVideoPlaylistMode,
    pub youtube_high_risk_playlist_prompt: bool,
    pub windows_toast_enabled: bool,
    pub language: LanguageSelection,
    pub ui_scale_percent: u16,
    #[serde(alias = "AlwaysOnTop")]
    pub keep_window_on_top: bool,
    pub remember_window_position: bool,
    pub remember_window_size: bool,
    #[serde(
        default,
        rename = "persist_window_state",
        alias = "RememberWindowStatePosition",
        skip_serializing
    )]
    legacy_persist_window_state: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_position: Option<WindowPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_size: Option<WindowSize>,
    pub enable_batch_panel: bool,
    pub prepare_skipped: bool,
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct WindowPosition {
    pub x: f32,
    pub y: f32,
}

impl WindowPosition {
    pub fn new(x: f32, y: f32) -> Option<Self> {
        if x.is_finite() && y.is_finite() {
            Some(Self { x, y })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl WindowSize {
    pub fn new(width: f32, height: f32) -> Option<Self> {
        if width.is_finite() && height.is_finite() && width >= 320.0 && height >= 240.0 {
            Some(Self { width, height })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum YoutubeVideoPlaylistMode {
    #[serde(rename = "ask")]
    Ask,
    #[serde(rename = "video")]
    Video,
    #[serde(rename = "ignore")]
    Ignore,
}

impl Default for YoutubeVideoPlaylistMode {
    fn default() -> Self {
        Self::Ask
    }
}

impl YoutubeVideoPlaylistMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ask => "config.youtube_playlist_mode.ask",
            Self::Video => "config.youtube_playlist_mode.video",
            Self::Ignore => "config.youtube_playlist_mode.ignore",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFileActionMode {
    #[serde(rename = "menu")]
    Menu,
    #[serde(rename = "open_folder")]
    OpenFolder,
    #[serde(rename = "open_file")]
    OpenFile,
}

impl Default for OutputFileActionMode {
    fn default() -> Self {
        Self::Menu
    }
}

impl OutputFileActionMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Menu => "config.output_action.menu",
            Self::OpenFolder => "config.output_action.open_folder",
            Self::OpenFile => "config.output_action.open_file",
        }
    }

    pub fn variants() -> [Self; 3] {
        [Self::Menu, Self::OpenFolder, Self::OpenFile]
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            download_dir: String::new(),
            cache_dir: String::new(),
            cache_location_mode: SerializableCacheLocationMode::V2Cache,
            yt_dlp_path: String::new(),
            yt_dlp_config_path: String::new(),
            ffmpeg_path: String::new(),
            aria2c_path: String::new(),
            deno_path: String::new(),
            use_browser_cookies: false,
            browser_cookie_source: "chrome".to_owned(),
            browser_cookie_profile: String::new(),
            browser_cookie_file: String::new(),
            youtube_subs_po_token: String::new(),
            youtube_extractor_args: String::new(),
            concurrent_fragments: 1,
            proxy_enabled: false,
            proxy_url: String::new(),
            no_check_certificates: false,
            limit_rate: String::new(),
            download_sections: String::new(),
            chapter_compatibility_mode: true,
            file_time_mode: FileTimeMode::None,
            use_aria2: false,
            write_thumbnail: false,
            embed_thumbnail: false,
            write_subtitles: false,
            embed_subtitles: false,
            write_chapters: false,
            embed_chapters: false,
            auto_paste_clipboard: false,
            clipboard_auto_add: false,
            auto_analyze: false,
            direct_download_on_add: false,
            output_file_action_mode: OutputFileActionMode::Menu,
            batch_limit_enabled: false,
            batch_limit_count: 100,
            youtube_video_playlist_mode: YoutubeVideoPlaylistMode::Ask,
            youtube_high_risk_playlist_prompt: true,
            windows_toast_enabled: false,
            language: LanguageSelection::Auto,
            ui_scale_percent: 100,
            keep_window_on_top: false,
            remember_window_position: false,
            remember_window_size: false,
            legacy_persist_window_state: None,
            window_position: None,
            window_size: None,
            enable_batch_panel: true,
            prepare_skipped: false,
            config_path: None,
        }
    }
}

impl AppConfig {
    pub fn load_runtime() -> (Self, ToolPaths) {
        let config_path = config_file_path();
        let mut config = fs::read_to_string(&config_path)
            .ok()
            .and_then(|content| serde_yaml::from_str::<Self>(&content).ok())
            .unwrap_or_default();
        config.config_path = Some(config_path);
        config.normalize();
        let tool_paths = config.tool_paths();
        let _ = config.save();
        (config, tool_paths)
    }

    pub fn save(&self) -> Result<(), String> {
        let Some(path) = self.config_path.as_ref() else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create config folder: {error}"))?;
        }

        let content = serde_yaml::to_string(self)
            .map_err(|error| format!("Could not serialize config file: {error}"))?;
        fs::write(path, content).map_err(|error| format!("Could not write config file: {error}"))
    }

    pub fn set_download_dir(&mut self, path: impl Into<String>) {
        self.download_dir = normalize_stored_path(&path.into());
    }

    pub fn set_yt_dlp_path(&mut self, path: impl Into<String>) {
        self.yt_dlp_path = normalize_stored_path(&path.into());
    }

    pub fn set_yt_dlp_config_path(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.yt_dlp_config_path = if path.trim().is_empty() {
            String::new()
        } else {
            normalize_stored_path(&path)
        };
    }

    pub fn set_ffmpeg_path(&mut self, path: impl Into<String>) {
        self.ffmpeg_path = normalize_stored_path(&path.into());
    }

    pub fn set_aria2c_path(&mut self, path: impl Into<String>) {
        self.aria2c_path = normalize_stored_path(&path.into());
    }

    pub fn set_deno_path(&mut self, path: impl Into<String>) {
        self.deno_path = normalize_stored_path(&path.into());
    }

    pub fn set_proxy_url(&mut self, value: impl Into<String>) {
        self.proxy_url = value.into().trim().to_owned();
    }

    pub fn set_limit_rate(&mut self, value: impl Into<String>) {
        self.limit_rate = value.into().trim().to_owned();
    }

    pub fn set_download_sections(&mut self, value: impl Into<String>) {
        self.download_sections = value.into().trim().to_owned();
    }

    fn tool_paths(&self) -> ToolPaths {
        ToolPaths {
            yt_dlp: self.yt_dlp_path.clone(),
            yt_dlp_config: self.yt_dlp_config_path.clone(),
            ffmpeg: self.ffmpeg_path.clone(),
            aria2c: self.aria2c_path.clone(),
            deno: self.deno_path.clone(),
            cache_mode: self.cache_location_mode.into_runtime(),
            cache_dir: self.cache_dir.clone(),
            browser_cookie_source: self.browser_cookie_source.clone(),
            browser_cookie_profile: self.browser_cookie_profile.clone(),
            browser_cookie_file: self.browser_cookie_file.clone(),
            youtube_subs_po_token: self.youtube_subs_po_token.clone(),
            youtube_extractor_args: self.youtube_extractor_args.clone(),
            concurrent_fragments: self.concurrent_fragments,
            proxy_enabled: self.proxy_enabled,
            proxy_url: self.proxy_url.clone(),
            no_check_certificates: self.no_check_certificates,
            limit_rate: self.limit_rate.clone(),
            download_sections: self.download_sections.clone(),
            chapter_compatibility_mode: self.chapter_compatibility_mode,
            file_time_mode: self.file_time_mode,
        }
    }

    fn normalize(&mut self) {
        let app_dir = app_dir();

        if self.download_dir.trim().is_empty() || !resolved_path_exists(&self.download_dir) {
            self.download_dir = portable_path_string(&app_dir.join("download"));
        } else {
            self.download_dir = normalize_stored_path(&self.download_dir);
        }

        if self.cache_dir.trim().is_empty() || !resolved_path_exists(&self.cache_dir) {
            self.cache_dir = portable_path_string(&app_dir.join("cache"));
        } else {
            self.cache_dir = normalize_stored_path(&self.cache_dir);
        }

        normalize_tool_path(&mut self.yt_dlp_path, tool_kind::YT_DLP);
        normalize_tool_path(&mut self.ffmpeg_path, tool_kind::FFMPEG);
        normalize_tool_path(&mut self.aria2c_path, tool_kind::ARIA2C);
        normalize_tool_path(&mut self.deno_path, tool_kind::DENO);

        if !self.browser_cookie_file.trim().is_empty() {
            self.browser_cookie_file = normalize_stored_path(&self.browser_cookie_file);
        }
        self.youtube_subs_po_token = self.youtube_subs_po_token.trim().to_owned();
        self.youtube_extractor_args = self.youtube_extractor_args.trim().to_owned();
        self.concurrent_fragments = normalize_concurrent_fragments(self.concurrent_fragments);
        self.proxy_url = self.proxy_url.trim().to_owned();
        self.limit_rate = self.limit_rate.trim().to_owned();
        self.download_sections = self.download_sections.trim().to_owned();

        if let Some(legacy) = self.legacy_persist_window_state.take() {
            self.remember_window_position = legacy;
            self.remember_window_size = legacy;
        }

        self.ui_scale_percent = normalize_ui_scale_percent(self.ui_scale_percent);

        if let Some(position) = self.window_position {
            if WindowPosition::new(position.x, position.y).is_none() {
                self.window_position = None;
            }
        }
        if let Some(size) = self.window_size {
            if WindowSize::new(size.width, size.height).is_none() {
                self.window_size = None;
            }
        }

        if self.write_thumbnail
            && self.embed_thumbnail
            && self.write_subtitles
            && !self.embed_subtitles
            && !self.write_chapters
            && !self.embed_chapters
        {
            self.write_thumbnail = false;
            self.embed_thumbnail = false;
            self.write_subtitles = false;
        }
    }
}

fn normalize_tool_path(path: &mut String, kind: ToolKind) {
    if path.trim().is_empty() || !resolved_file_exists(path.as_str()) {
        *path = discover_tool(kind)
            .map(|path| portable_path_string(&path))
            .unwrap_or_else(|| kind.default_portable_path().to_owned());
    } else {
        *path = normalize_stored_path(path.as_str());
    }
}

pub fn normalize_ui_scale_percent(value: u16) -> u16 {
    value.clamp(80, 200)
}

fn normalize_concurrent_fragments(value: usize) -> usize {
    match value {
        1 | 2 | 4 | 8 => value,
        0 => 1,
        3 => 4,
        5..=7 => 8,
        _ => 8,
    }
}

pub fn available_yt_dlp_config_files() -> Vec<ConfigFileOption> {
    let dir = yt_dlp_configs_dir();
    let files = if dir.is_dir() {
        fs::read_dir(&dir)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.flatten())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut items = files
        .into_iter()
        .map(|path| ConfigFileOption {
            name: path
                .file_stem()
                .map(|value| value.to_string_lossy().to_string())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| path.display().to_string()),
            path: portable_path_string(&path),
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.name.cmp(&right.name).then(left.path.cmp(&right.path)));
    items
}

pub fn yt_dlp_configs_dir_display() -> String {
    portable_path_string(&yt_dlp_configs_dir())
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SerializableCacheLocationMode {
    #[serde(rename = "yt_dlp_default")]
    YtDlpDefault,
    #[serde(rename = "v2_cache")]
    V2Cache,
    #[serde(rename = "windows_temp")]
    WindowsTemp,
}

impl Default for SerializableCacheLocationMode {
    fn default() -> Self {
        Self::V2Cache
    }
}

impl SerializableCacheLocationMode {
    fn into_runtime(self) -> CacheLocationMode {
        match self {
            Self::YtDlpDefault => CacheLocationMode::YtDlpDefault,
            Self::V2Cache => CacheLocationMode::V2Cache,
            Self::WindowsTemp => CacheLocationMode::WindowsTemp,
        }
    }
}

pub(crate) fn runtime_config_file_path() -> PathBuf {
    config_file_path()
}

fn config_file_path() -> PathBuf {
    let app_dir = portable_root_dir();
    let file_name = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
        })
        .filter(|stem| !stem.is_empty())
        .map(|stem| format!("{stem}.yaml"))
        .unwrap_or_else(|| "yt-dlp-gui.yaml".to_owned());
    app_dir.join(file_name)
}

fn yt_dlp_configs_dir() -> PathBuf {
    portable_root_dir().join("configs")
}

fn app_dir() -> PathBuf {
    portable_root_dir()
}

fn portable_root_dir() -> PathBuf {
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

fn executable_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(portable_root_dir)
}

fn candidate_base_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![portable_root_dir()];
    let exe_dir = executable_dir();
    if !dirs.iter().any(|dir| dir == &exe_dir) {
        dirs.push(exe_dir);
    }
    dirs
}

fn resolved_path(path: &str) -> PathBuf {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return portable_root_dir();
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.is_absolute() {
        return candidate;
    }

    for base in candidate_base_dirs() {
        let joined = base.join(&candidate);
        if joined.exists() {
            return joined;
        }
    }

    portable_root_dir().join(candidate)
}

fn resolved_path_exists(path: &str) -> bool {
    resolved_path(path).exists()
}

fn resolved_file_exists(path: &str) -> bool {
    resolved_path(path).is_file()
}

fn normalize_stored_path(path: &str) -> String {
    portable_path_string(&resolved_path(path))
}

fn portable_path_string(path: &Path) -> String {
    let app_dir = app_dir();
    if let Ok(relative) = path.strip_prefix(&app_dir) {
        let relative = relative.display().to_string().replace('/', "\\");
        if relative.is_empty() {
            ".\\".to_owned()
        } else {
            format!(".\\{relative}")
        }
    } else {
        path.display().to_string()
    }
}

fn discover_tool(kind: ToolKind) -> Option<PathBuf> {
    for base in candidate_base_dirs() {
        let tools_dir = base.join("tools");
        if !tools_dir.is_dir() {
            continue;
        }

        let mut stack = vec![tools_dir];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = fs::read_dir(dir) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if !path.is_file() {
                    continue;
                }

                let file_name = path
                    .file_name()
                    .map(|value| value.to_string_lossy().to_ascii_lowercase())
                    .unwrap_or_default();
                if kind.matches(&file_name) {
                    return Some(path);
                }
            }
        }
    }

    None
}

#[derive(Clone, Copy)]
enum ToolKind {
    YtDlp,
    Ffmpeg,
    Aria2c,
    Deno,
}

impl ToolKind {
    fn matches(self, file_name: &str) -> bool {
        match self {
            Self::YtDlp => {
                (file_name.starts_with("yt-dlp") || file_name.starts_with("ytdl-patched"))
                    && file_name.ends_with(".exe")
            }
            Self::Ffmpeg => file_name == "ffmpeg.exe",
            Self::Aria2c => file_name.starts_with("aria2") && file_name.ends_with(".exe"),
            Self::Deno => file_name == "deno.exe",
        }
    }

    fn default_portable_path(self) -> &'static str {
        match self {
            Self::YtDlp => ".\\tools\\yt-dlp\\yt-dlp.exe",
            Self::Ffmpeg => ".\\tools\\ffmpeg\\ffmpeg.exe",
            Self::Aria2c => ".\\tools\\aria2c\\aria2c.exe",
            Self::Deno => ".\\tools\\deno\\deno.exe",
        }
    }
}

mod tool_kind {
    use super::ToolKind;

    pub const YT_DLP: ToolKind = ToolKind::YtDlp;
    pub const FFMPEG: ToolKind = ToolKind::Ffmpeg;
    pub const ARIA2C: ToolKind = ToolKind::Aria2c;
    pub const DENO: ToolKind = ToolKind::Deno;
}
