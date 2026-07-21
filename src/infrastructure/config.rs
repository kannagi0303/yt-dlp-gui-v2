use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::i18n::LanguageSelection;

use super::tool_install::{DependencyTool, detect_dependency_tool};
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
    pub live_from_start: bool,
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
    pub always_show_download_range: bool,
    #[serde(alias = "ModifiedType")]
    pub file_time_mode: FileTimeMode,
    #[serde(alias = "UseAria2")]
    pub use_aria2: bool,
    pub thumbnail_mode: PostProcessMode,
    pub subtitle_mode: PostProcessMode,
    pub chapter_mode: PostProcessMode,
    #[serde(alias = "apple_tv_hevc_post_process")]
    pub post_download_conversion_enabled: bool,
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
    pub music_volume: f32,
    pub music_playback_mode: String,
    #[serde(skip_serializing)]
    pub queue_display_mode: String,
    pub app_mode: String,
    pub show_log_tab: bool,
    pub transcode_intent: TranscodeIntentSettings,
    pub language: LanguageSelection,
    pub theme_mode: ThemeMode,
    pub theme_accent_color: ThemeAccentColor,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PostProcessMode {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "download")]
    Download,
    #[serde(rename = "embed")]
    Embed,
}

impl Default for PostProcessMode {
    fn default() -> Self {
        Self::Off
    }
}

impl PostProcessMode {
    pub fn writes(self) -> bool {
        matches!(self, Self::Download | Self::Embed)
    }

    pub fn embeds(self) -> bool {
        matches!(self, Self::Embed)
    }

    pub fn variants() -> [Self; 3] {
        [Self::Off, Self::Download, Self::Embed]
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Off => "advance.none",
            Self::Download => "advance.download",
            Self::Embed => "advance.embed",
        }
    }
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
            Self::Ask => "Ask",
            Self::Video => "Video",
            Self::Ignore => "Ignore",
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
            Self::Menu => "Show menu",
            Self::OpenFolder => "Open folder",
            Self::OpenFile => "Open file",
        }
    }

    pub fn variants() -> [Self; 3] {
        [Self::Menu, Self::OpenFolder, Self::OpenFile]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeMode {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::System
    }
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "Follow system",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn variants() -> [Self; 3] {
        [Self::System, Self::Light, Self::Dark]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeAccentColor {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "purple")]
    Purple,
    #[serde(rename = "pink")]
    Pink,
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "orange")]
    Orange,
    #[serde(rename = "slate")]
    Slate,
}

impl Default for ThemeAccentColor {
    fn default() -> Self {
        Self::Off
    }
}

impl ThemeAccentColor {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::System => "Blue",
            Self::Blue => "Soft blue",
            Self::Purple => "Purple",
            Self::Pink => "Pink",
            Self::Green => "Green",
            Self::Orange => "Orange",
            Self::Slate => "Slate",
        }
    }

    pub fn variants() -> [Self; 8] {
        [
            Self::Off,
            Self::System,
            Self::Blue,
            Self::Purple,
            Self::Pink,
            Self::Green,
            Self::Orange,
            Self::Slate,
        ]
    }

    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Off | Self::System | Self::Blue => (74, 144, 226),
            Self::Purple => (145, 111, 224),
            Self::Pink => (220, 104, 158),
            Self::Green => (83, 164, 112),
            Self::Orange => (224, 136, 69),
            Self::Slate => (120, 132, 150),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TranscodeIntentSettings {
    #[serde(skip_serializing)]
    pub intent_mode: TranscodeIntentMode,
    #[serde(skip_serializing)]
    pub compatibility_target: CompatibilityTarget,
    pub video_codec_policy: VideoCodecPolicy,
    pub container_policy: ContainerPolicy,
    #[serde(skip_serializing)]
    pub encoder_policy: EncoderPolicy,
    #[serde(skip_serializing)]
    pub quality_target: QualityTarget,
    #[serde(skip_serializing)]
    pub size_ratio_percent: u8,
    #[serde(skip_serializing)]
    pub target_size_mb: u32,
    #[serde(skip_serializing)]
    pub resolution_policy: ResolutionPolicy,
    #[serde(skip_serializing)]
    pub frame_rate_policy: FrameRatePolicy,
    #[serde(skip_serializing)]
    pub encode_effort: EncodeEffort,
    #[serde(skip_serializing)]
    pub pass_policy: PassPolicy,
    pub audio_policy: AudioPolicy,
    pub subtitle_policy: SubtitlePolicy,
    #[serde(skip_serializing)]
    pub hdr_policy: HdrPolicy,
    #[serde(skip_serializing)]
    pub locked_keys: Vec<TranscodeSettingKey>,
}

impl Default for TranscodeIntentSettings {
    fn default() -> Self {
        Self {
            intent_mode: TranscodeIntentMode::ReduceSize,
            compatibility_target: CompatibilityTarget::MostDevices,
            video_codec_policy: VideoCodecPolicy::Auto,
            container_policy: ContainerPolicy::Auto,
            encoder_policy: EncoderPolicy::Auto,
            quality_target: QualityTarget::High,
            size_ratio_percent: 100,
            target_size_mb: 900,
            resolution_policy: ResolutionPolicy::AutoBalance,
            frame_rate_policy: FrameRatePolicy::Source,
            encode_effort: EncodeEffort::Normal,
            pass_policy: PassPolicy::Auto,
            audio_policy: AudioPolicy::Auto,
            subtitle_policy: SubtitlePolicy::Preserve,
            hdr_policy: HdrPolicy::Compatible,
            locked_keys: Vec::new(),
        }
    }
}

impl TranscodeIntentSettings {
    pub fn normalized(mut self) -> Self {
        self.size_ratio_percent = self.size_ratio_percent.clamp(5, 100);
        self.target_size_mb = self.target_size_mb.clamp(1, 999_999);
        self.locked_keys.sort();
        self.locked_keys.dedup();
        self
    }

    pub fn is_locked(&self, key: TranscodeSettingKey) -> bool {
        self.locked_keys.contains(&key)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TranscodeIntentMode {
    #[serde(rename = "reduce_size")]
    ReduceSize,
    #[serde(rename = "quality_first")]
    QualityFirst,
    #[serde(rename = "target_size")]
    TargetSize,
    #[serde(rename = "fast_transcode")]
    FastTranscode,
    #[serde(rename = "device_compat")]
    DeviceCompat,
}

impl Default for TranscodeIntentMode {
    fn default() -> Self {
        Self::ReduceSize
    }
}

impl TranscodeIntentMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::ReduceSize => "Smaller file",
            Self::QualityFirst => "Quality first",
            Self::TargetSize => "Size target",
            Self::FastTranscode => "Format",
            Self::DeviceCompat => "Compatibility target",
        }
    }

    pub fn variants() -> [Self; 5] {
        [
            Self::ReduceSize,
            Self::QualityFirst,
            Self::TargetSize,
            Self::FastTranscode,
            Self::DeviceCompat,
        ]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompatibilityTarget {
    #[serde(rename = "most_devices")]
    MostDevices,
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "mac")]
    Mac,
    #[serde(rename = "apple")]
    Apple,
    #[serde(rename = "apple_tv_legacy")]
    AppleTvLegacy,
    #[serde(rename = "apple_tv_modern")]
    AppleTvModern,
    #[serde(rename = "iphone_ipad")]
    IphoneIpad,
    #[serde(rename = "android_tv")]
    AndroidTv,
    #[serde(rename = "android_phone_tablet")]
    AndroidPhoneTablet,
    #[serde(rename = "browser_mp4")]
    BrowserMp4,
    #[serde(rename = "tv_nas")]
    TvNas,
    #[serde(rename = "old_device")]
    OldDevice,
}

impl Default for CompatibilityTarget {
    fn default() -> Self {
        Self::MostDevices
    }
}

impl CompatibilityTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::MostDevices => "Most devices / not sure",
            Self::Windows => "Windows PC",
            Self::Mac => "Mac",
            Self::Apple => "Apple devices",
            Self::AppleTvLegacy => "Apple TV legacy",
            Self::AppleTvModern => "Apple TV modern",
            Self::IphoneIpad => "iPhone / iPad",
            Self::AndroidTv => "Android TV / Chromecast",
            Self::AndroidPhoneTablet => "Android phone / tablet",
            Self::BrowserMp4 => "Browser-safe MP4",
            Self::TvNas => "Generic TV / NAS",
            Self::OldDevice => "Old TV / USB playback",
        }
    }

    pub fn variants() -> [Self; 12] {
        [
            Self::MostDevices,
            Self::Windows,
            Self::Mac,
            Self::Apple,
            Self::AppleTvLegacy,
            Self::AppleTvModern,
            Self::IphoneIpad,
            Self::AndroidTv,
            Self::AndroidPhoneTablet,
            Self::BrowserMp4,
            Self::TvNas,
            Self::OldDevice,
        ]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VideoCodecPolicy {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "h264")]
    H264,
    #[serde(rename = "hevc")]
    Hevc,
    #[serde(rename = "av1")]
    Av1,
}

impl Default for VideoCodecPolicy {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContainerPolicy {
    #[serde(rename = "auto", alias = "source")]
    Auto,
    #[serde(rename = "mp4")]
    Mp4,
    #[serde(rename = "mkv")]
    Mkv,
    #[serde(rename = "mov")]
    Mov,
}

impl Default for ContainerPolicy {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncoderPolicy {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "hardware_first")]
    HardwareFirst,
    #[serde(rename = "software")]
    Software,
}

impl Default for EncoderPolicy {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum QualityTarget {
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "near_original")]
    NearOriginal,
}

impl Default for QualityTarget {
    fn default() -> Self {
        Self::High
    }
}

impl QualityTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::High => "High quality",
            Self::NearOriginal => "Near original",
        }
    }

    pub fn variants() -> [Self; 3] {
        [Self::Standard, Self::High, Self::NearOriginal]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResolutionPolicy {
    #[serde(rename = "auto_balance")]
    AutoBalance,
    #[serde(rename = "keep_original")]
    KeepOriginal,
    #[serde(rename = "max_1080p")]
    Max1080p,
    #[serde(rename = "max_720p")]
    Max720p,
}

impl Default for ResolutionPolicy {
    fn default() -> Self {
        Self::AutoBalance
    }
}

impl ResolutionPolicy {
    pub fn label(self) -> &'static str {
        match self {
            Self::AutoBalance => "Auto balance",
            Self::KeepOriginal => "Keep original",
            Self::Max1080p => "Max 1080p",
            Self::Max720p => "Max 720p",
        }
    }

    pub fn variants() -> [Self; 4] {
        [
            Self::AutoBalance,
            Self::KeepOriginal,
            Self::Max1080p,
            Self::Max720p,
        ]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FrameRatePolicy {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "fps_24")]
    Fps24,
    #[serde(rename = "fps_25")]
    Fps25,
    #[serde(rename = "fps_30")]
    Fps30,
    #[serde(rename = "fps_60")]
    Fps60,
}

impl Default for FrameRatePolicy {
    fn default() -> Self {
        Self::Source
    }
}

impl FrameRatePolicy {
    pub fn label(self) -> &'static str {
        match self {
            Self::Source => "Source",
            Self::Fps24 => "Up to 24 fps",
            Self::Fps25 => "Up to 25 fps",
            Self::Fps30 => "Up to 30 fps",
            Self::Fps60 => "Up to 60 fps",
        }
    }

    pub fn variants() -> [Self; 5] {
        [
            Self::Source,
            Self::Fps24,
            Self::Fps25,
            Self::Fps30,
            Self::Fps60,
        ]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncodeEffort {
    #[serde(rename = "fast")]
    Fast,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "detailed")]
    Detailed,
    #[serde(rename = "extreme")]
    Extreme,
}

impl Default for EncodeEffort {
    fn default() -> Self {
        Self::Normal
    }
}

impl EncodeEffort {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fast => "Fast",
            Self::Normal => "Normal",
            Self::Detailed => "Detailed",
            Self::Extreme => "Extreme",
        }
    }

    pub fn variants() -> [Self; 4] {
        [Self::Fast, Self::Normal, Self::Detailed, Self::Extreme]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PassPolicy {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "one_pass")]
    OnePass,
    #[serde(rename = "two_pass")]
    TwoPass,
}

impl Default for PassPolicy {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioPolicy {
    #[serde(rename = "auto", alias = "preserve_surround")]
    Auto,
    #[serde(rename = "aac", alias = "compatible")]
    Aac,
    #[serde(rename = "opus")]
    Opus,
    #[serde(rename = "flac")]
    Flac,
}

impl Default for AudioPolicy {
    fn default() -> Self {
        Self::Auto
    }
}

impl AudioPolicy {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Source",
            Self::Aac => "AAC",
            Self::Opus => "Opus",
            Self::Flac => "FLAC",
        }
    }

    pub fn variants() -> [Self; 4] {
        [Self::Auto, Self::Aac, Self::Opus, Self::Flac]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubtitlePolicy {
    #[serde(rename = "preserve", alias = "remove")]
    Preserve,
    #[serde(rename = "embed", alias = "compatible")]
    Embed,
    #[serde(rename = "burn")]
    Burn,
}

impl Default for SubtitlePolicy {
    fn default() -> Self {
        Self::Preserve
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HdrPolicy {
    #[serde(rename = "compatible")]
    Compatible,
    #[serde(rename = "preserve_hdr")]
    PreserveHdr,
}

impl Default for HdrPolicy {
    fn default() -> Self {
        Self::Compatible
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TranscodeSettingKey {
    #[serde(rename = "compatibility_target")]
    CompatibilityTarget,
    #[serde(rename = "video_codec_policy")]
    VideoCodecPolicy,
    #[serde(rename = "container_policy")]
    ContainerPolicy,
    #[serde(rename = "encoder_policy")]
    EncoderPolicy,
    #[serde(rename = "quality_target")]
    QualityTarget,
    #[serde(rename = "size_ratio")]
    SizeRatio,
    #[serde(rename = "target_size")]
    TargetSize,
    #[serde(rename = "resolution_policy")]
    ResolutionPolicy,
    #[serde(rename = "frame_rate_policy")]
    FrameRatePolicy,
    #[serde(rename = "encode_effort")]
    EncodeEffort,
    #[serde(rename = "pass_policy")]
    PassPolicy,
    #[serde(rename = "audio_policy")]
    AudioPolicy,
}

impl TranscodeSettingKey {
    pub fn label(self) -> &'static str {
        match self {
            Self::CompatibilityTarget => "Compatibility",
            Self::VideoCodecPolicy => "Video codec",
            Self::ContainerPolicy => "Container",
            Self::EncoderPolicy => "Encoder",
            Self::QualityTarget => "Quality",
            Self::SizeRatio => "Size ratio",
            Self::TargetSize => "Target size",
            Self::ResolutionPolicy => "Resolution",
            Self::FrameRatePolicy => "FPS limit",
            Self::EncodeEffort => "Effort",
            Self::PassPolicy => "Size control",
            Self::AudioPolicy => "Audio",
        }
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
            live_from_start: true,
            proxy_enabled: false,
            proxy_url: String::new(),
            no_check_certificates: false,
            limit_rate: String::new(),
            download_sections: String::new(),
            chapter_compatibility_mode: true,
            always_show_download_range: false,
            file_time_mode: FileTimeMode::None,
            use_aria2: false,
            thumbnail_mode: PostProcessMode::Off,
            subtitle_mode: PostProcessMode::Off,
            chapter_mode: PostProcessMode::Off,
            post_download_conversion_enabled: false,
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
            music_volume: 0.80,
            music_playback_mode: "sequential".to_owned(),
            queue_display_mode: "normal".to_owned(),
            app_mode: "origin".to_owned(),
            show_log_tab: true,
            transcode_intent: TranscodeIntentSettings::default(),
            language: LanguageSelection::Auto,
            theme_mode: ThemeMode::System,
            theme_accent_color: ThemeAccentColor::Off,
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

fn explicit_app_mode_from_config_content(content: &str) -> Option<String> {
    let value = serde_yaml::from_str::<serde_yaml::Value>(content).ok()?;
    let mapping = value.as_mapping()?;
    for key in ["app_mode", "appMode", "AppMode", "app-mode"] {
        if let Some(raw) = mapping
            .get(&serde_yaml::Value::String(key.to_owned()))
            .and_then(|value| value.as_str())
        {
            return Some(normalize_app_mode_value(raw));
        }
    }
    None
}

fn normalize_app_mode_value(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "origin" => "origin".to_owned(),
        "standard" => "standard".to_owned(),
        "audio" => "audio".to_owned(),
        _ => "origin".to_owned(),
    }
}

fn queue_display_mode_from_app_mode_value(value: &str) -> String {
    if value == "audio" {
        "audio".to_owned()
    } else {
        "normal".to_owned()
    }
}

impl AppConfig {
    pub fn load_runtime() -> (Self, ToolPaths) {
        let config_path = config_file_path();
        let raw_content = fs::read_to_string(&config_path).ok();
        let explicit_app_mode = raw_content
            .as_deref()
            .and_then(explicit_app_mode_from_config_content);
        let mut config = raw_content
            .as_deref()
            .and_then(|content| serde_yaml::from_str::<Self>(content).ok())
            .unwrap_or_default();
        config.config_path = Some(config_path);
        config.normalize();
        if let Some(app_mode) = explicit_app_mode {
            config.app_mode = app_mode;
        }
        config.queue_display_mode = queue_display_mode_from_app_mode_value(&config.app_mode);
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
            live_from_start: self.live_from_start,
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

        normalize_tool_path(&mut self.yt_dlp_path, DependencyTool::YtDlp);
        normalize_tool_path(&mut self.ffmpeg_path, DependencyTool::Ffmpeg);
        normalize_tool_path(&mut self.aria2c_path, DependencyTool::Aria2c);
        normalize_tool_path(&mut self.deno_path, DependencyTool::Deno);

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
        self.music_volume = self.music_volume.clamp(0.0, 1.0);
        self.music_playback_mode = match self
            .music_playback_mode
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "repeat_all" | "repeat" | "loop" => "repeat_all".to_owned(),
            "shuffle" | "random" => "shuffle".to_owned(),
            "repeat_one" | "single" | "one" => "repeat_one".to_owned(),
            _ => "sequential".to_owned(),
        };
        self.app_mode = normalize_app_mode_value(&self.app_mode);
        self.queue_display_mode = queue_display_mode_from_app_mode_value(&self.app_mode);
        self.transcode_intent = self.transcode_intent.clone().normalized();

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
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AppConfig, PostProcessMode, normalize_app_mode_value,
        queue_display_mode_from_app_mode_value,
    };

    #[test]
    fn app_config_defaults_to_origin_app_mode() {
        assert_eq!(AppConfig::default().app_mode, "origin");
    }

    #[test]
    fn app_mode_normalization_rejects_legacy_aliases() {
        assert_eq!(normalize_app_mode_value("origin"), "origin");
        assert_eq!(normalize_app_mode_value("standard"), "standard");
        assert_eq!(normalize_app_mode_value("audio"), "audio");

        assert_eq!(normalize_app_mode_value("single"), "origin");
        assert_eq!(normalize_app_mode_value("normal"), "origin");
        assert_eq!(normalize_app_mode_value("garbage"), "origin");
    }

    #[test]
    fn queue_display_mode_is_derived_from_app_mode() {
        assert_eq!(queue_display_mode_from_app_mode_value("audio"), "audio");
        assert_eq!(queue_display_mode_from_app_mode_value("origin"), "normal");
        assert_eq!(queue_display_mode_from_app_mode_value("standard"), "normal");
    }

    #[test]
    fn serialized_config_writes_app_mode_without_queue_display_mode() {
        for app_mode in ["origin", "standard", "audio"] {
            let mut config = AppConfig::default();
            config.app_mode = app_mode.to_owned();
            config.queue_display_mode = queue_display_mode_from_app_mode_value(app_mode);

            let yaml = serde_yaml::to_string(&config).expect("serialize config");

            assert!(yaml.contains(&format!("app_mode: {app_mode}")));
            assert!(!yaml.contains("queue_display_mode"));
        }
    }

    #[test]
    fn serialized_config_writes_post_process_modes_without_legacy_bools() {
        let mut config = AppConfig::default();
        config.thumbnail_mode = PostProcessMode::Embed;
        config.subtitle_mode = PostProcessMode::Download;
        config.chapter_mode = PostProcessMode::Off;

        let yaml = serde_yaml::to_string(&config).expect("serialize config");

        assert!(yaml.contains("thumbnail_mode: embed"));
        assert!(yaml.contains("subtitle_mode: download"));
        assert!(yaml.contains("chapter_mode: off"));
        assert!(!yaml.contains("write_thumbnail"));
        assert!(!yaml.contains("embed_thumbnail"));
        assert!(!yaml.contains("write_subtitles"));
        assert!(!yaml.contains("embed_subtitles"));
        assert!(!yaml.contains("write_chapters"));
        assert!(!yaml.contains("embed_chapters"));
    }

    #[test]
    fn live_from_start_defaults_on_and_round_trips_when_disabled() {
        assert!(AppConfig::default().live_from_start);

        let mut config = AppConfig::default();
        config.live_from_start = false;
        let yaml = serde_yaml::to_string(&config).expect("serialize config");
        let restored = serde_yaml::from_str::<AppConfig>(&yaml).expect("deserialize config");

        assert!(yaml.contains("live_from_start: false"));
        assert!(!restored.live_from_start);
    }

    #[test]
    fn always_show_download_range_defaults_off_and_round_trips_when_enabled() {
        assert!(!AppConfig::default().always_show_download_range);

        let mut config = AppConfig::default();
        config.always_show_download_range = true;
        let yaml = serde_yaml::to_string(&config).expect("serialize config");
        let restored = serde_yaml::from_str::<AppConfig>(&yaml).expect("deserialize config");

        assert!(yaml.contains("always_show_download_range: true"));
        assert!(restored.always_show_download_range);
    }
}

fn normalize_tool_path(path: &mut String, kind: DependencyTool) {
    if path.trim().is_empty() || !resolved_file_exists(path.as_str()) {
        *path = detect_dependency_tool(kind)
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
