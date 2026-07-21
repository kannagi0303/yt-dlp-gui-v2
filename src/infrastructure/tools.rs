use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::os::raw::c_void;
use std::process::{Command, Stdio};
#[cfg(target_os = "windows")]
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::cookie_site_index::read_cookie_site_index;
use super::process_guard::run_tracked_command_output;

#[cfg(target_os = "windows")]
use std::os::windows::ffi::{OsStrExt, OsStringExt};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const DEFAULT_FORMAT_SELECTOR: &str = "bestvideo*+bestaudio/best";
const MUSIC_STREAM_FORMAT_SELECTOR: &str = "bestaudio/best[acodec!=none]";
const SECTION_FORMAT_SELECTOR: &str =
    "best[protocol!*=dash][vcodec!=none][acodec!=none]/best[vcodec!=none][acodec!=none]/best";
const COOKIE_SOURCE_AUTO: &str = "auto";
const COOKIE_SOURCE_FILE: &str = "file";
pub const FINAL_OUTPUT_PATH_PREFIX: &str = "__YTDLPGUI_FINAL_PATH__=";
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
#[cfg(target_os = "windows")]
const SHGFI_DISPLAYNAME: u32 = 0x0000_0200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrowserCookieSourceOption {
    pub value: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserCookieProfileOption {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileTimeMode {
    #[serde(
        rename = "none",
        alias = "preserve_modified",
        alias = "Modified",
        alias = "modified"
    )]
    None,
    #[serde(rename = "use_upload_date", alias = "Upload", alias = "upload")]
    UseUploadDate,
    #[serde(
        rename = "use_download_time",
        alias = "use_current",
        alias = "Created",
        alias = "created"
    )]
    UseDownloadTime,
}

impl Default for FileTimeMode {
    fn default() -> Self {
        Self::None
    }
}

impl FileTimeMode {
    pub fn label_key(self) -> &'static str {
        match self {
            Self::None => "advance.file_time.none",
            Self::UseUploadDate => "advance.file_time.upload_date",
            Self::UseDownloadTime => "advance.file_time.download_time",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheLocationMode {
    YtDlpDefault,
    V2Cache,
    WindowsTemp,
}

impl CacheLocationMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::YtDlpDefault => "Default",
            Self::V2Cache => "yt-dlp-gui",
            Self::WindowsTemp => "Windows",
        }
    }
}

#[derive(Clone)]
pub struct DownloadRequest {
    pub target_kind: DownloadTargetKind,
    pub url: String,
    pub format_selector: String,
    pub video_selector: String,
    pub audio_selector: String,
    pub is_muxed_video: bool,
    pub video_ext: String,
    pub audio_ext: String,
    pub merge_output_container: Option<String>,
    pub upload_date: String,
    pub subtitle_lang: Option<String>,
    pub subtitle_ext: String,
    pub subtitle_source_ext: String,
    pub subtitle_url: Option<String>,
    pub write_auto_subs: bool,
    pub subtitle_is_auto_translated: bool,
    pub write_subtitles: bool,
    pub embed_subtitles: bool,
    pub write_chapters: bool,
    pub embed_chapters: bool,
    pub write_thumbnail: bool,
    pub embed_thumbnail: bool,
    pub use_cookies: bool,
    pub use_aria2: bool,
    pub emit_json: bool,
    pub output_path: Option<String>,
    pub output_dir: String,
    pub file_name: String,
    pub download_section_args: Vec<String>,
}

pub struct PreparedDownload {
    pub command: Command,
    pub output_path: PathBuf,
    pub command_line: String,
}

pub struct AnalyzeOutput {
    pub json: Value,
    pub command_line: String,
}

pub struct AnalyzeError {
    pub message: String,
    pub command_line: Option<String>,
}

impl AnalyzeError {
    fn new(message: impl Into<String>, command_line: Option<String>) -> Self {
        Self {
            message: message.into(),
            command_line,
        }
    }
}

struct OutputPlan {
    output_argument: String,
    final_output_path: PathBuf,
    remux_extension: Option<String>,
    extract_audio: bool,
    audio_format: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadTargetKind {
    Normal,
    Video,
    Audio,
    Subtitle,
}

#[derive(Clone)]
pub struct ToolPaths {
    pub yt_dlp: String,
    pub yt_dlp_config: String,
    pub ffmpeg: String,
    pub aria2c: String,
    pub deno: String,
    pub cache_mode: CacheLocationMode,
    pub cache_dir: String,
    pub browser_cookie_source: String,
    pub browser_cookie_profile: String,
    pub browser_cookie_file: String,
    pub youtube_subs_po_token: String,
    pub youtube_extractor_args: String,
    pub concurrent_fragments: usize,
    pub live_from_start: bool,
    pub proxy_enabled: bool,
    pub proxy_url: String,
    pub no_check_certificates: bool,
    pub limit_rate: String,
    pub download_sections: String,
    pub chapter_compatibility_mode: bool,
    pub file_time_mode: FileTimeMode,
}

impl Default for ToolPaths {
    fn default() -> Self {
        Self {
            yt_dlp: ".\\tools\\yt-dlp\\yt-dlp.exe".to_owned(),
            yt_dlp_config: String::new(),
            ffmpeg: ".\\tools\\ffmpeg\\ffmpeg.exe".to_owned(),
            aria2c: ".\\tools\\aria2c\\aria2c.exe".to_owned(),
            deno: ".\\tools\\deno\\deno.exe".to_owned(),
            cache_mode: CacheLocationMode::V2Cache,
            cache_dir: ".\\cache".to_owned(),
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
            file_time_mode: FileTimeMode::None,
        }
    }
}

impl ToolPaths {
    pub fn has_explicit_config(&self) -> bool {
        !self.yt_dlp_config.trim().is_empty() && resolve_support_path(&self.yt_dlp_config).is_file()
    }

    pub fn effective_config_path(&self) -> Option<PathBuf> {
        self.explicit_config_path()
            .or_else(|| self.portable_config_path())
    }

    pub fn effective_config_owns_output(&self) -> bool {
        self.effective_config_path()
            .as_deref()
            .is_some_and(config_contains_output_arg)
    }

    pub fn effective_config_owns_format(&self) -> bool {
        self.effective_config_path()
            .as_deref()
            .is_some_and(config_contains_format_arg)
    }

    pub fn effective_config_owns_limit_rate(&self) -> bool {
        self.effective_config_path()
            .as_deref()
            .is_some_and(config_contains_limit_rate_arg)
    }

    pub fn effective_config_owns_download_sections(&self) -> bool {
        self.effective_config_path()
            .as_deref()
            .is_some_and(config_contains_download_sections_arg)
    }

    pub fn effective_config_owns_live_from_start(&self) -> bool {
        self.effective_config_path()
            .as_deref()
            .is_some_and(config_contains_live_from_start_arg)
    }

    pub fn effective_config_owns_mtime(&self) -> bool {
        self.effective_config_path()
            .as_deref()
            .is_some_and(config_contains_mtime_arg)
    }

    pub fn effective_proxy_url(&self) -> Option<&str> {
        let proxy_url = self.proxy_url.trim();
        (self.proxy_enabled && !proxy_url.is_empty()).then_some(proxy_url)
    }

    pub fn available_browser_cookie_sources(&self) -> Vec<BrowserCookieSourceOption> {
        let mut items = Vec::new();
        items.push(BrowserCookieSourceOption {
            value: COOKIE_SOURCE_AUTO,
            label: "Auto by website",
        });
        items.push(BrowserCookieSourceOption {
            value: COOKIE_SOURCE_FILE,
            label: "Use file (cookies.txt)",
        });
        let mut has_browser_source = false;
        for option in browser_cookie_source_candidates() {
            if browser_cookie_source_exists(option.value) {
                has_browser_source = true;
                items.push(option);
            }
        }
        if !has_browser_source {
            items.push(BrowserCookieSourceOption {
                value: "chrome",
                label: "Chrome",
            });
        }
        items
    }

    pub fn available_browser_cookie_profiles(&self) -> Vec<BrowserCookieProfileOption> {
        if self.uses_cookie_file_source() || self.uses_auto_cookie_source() {
            return Vec::new();
        }
        browser_cookie_profiles(&self.browser_cookie_source)
    }

    pub fn prepare_batch_add_command(&self, url: &str) -> Result<Command, String> {
        let tool_path = self.validate_yt_dlp_available()?;

        let mut command = Command::new(&tool_path);
        configure_background_command(&mut command);
        self.apply_common_yt_dlp_args(&mut command);
        command
            .arg("--flat-playlist")
            .arg("--dump-json")
            .arg("--lazy-playlist")
            .arg("--no-warnings")
            .arg("--ignore-errors")
            .arg(url);

        println!(
            "[batch-add] command: {}",
            format_command_line(&tool_path, &command)
        );

        Ok(command)
    }

    pub fn prepare_music_flat_update_command(
        &self,
        url: &str,
        use_cookies: bool,
    ) -> Result<Command, String> {
        let tool_path = self.validate_yt_dlp_available()?;

        let mut command = Command::new(&tool_path);
        configure_background_command(&mut command);
        self.apply_common_yt_dlp_args(&mut command);
        self.apply_cookie_args(&mut command, use_cookies, Some(url))?;
        command
            .arg("--flat-playlist")
            .arg("--dump-json")
            .arg("--lazy-playlist")
            .arg("--playlist-items")
            .arg("1")
            .arg("--no-warnings")
            .arg("--ignore-errors")
            .arg(url);

        println!(
            "[music-flat-update] command: {}",
            format_command_line(&tool_path, &command)
        );

        Ok(command)
    }

    pub fn analyze_url(&self, url: &str, use_cookies: bool) -> Result<AnalyzeOutput, String> {
        self.analyze_url_detailed(url, use_cookies)
            .map_err(|error| error.message)
    }

    pub fn analyze_url_detailed(
        &self,
        url: &str,
        use_cookies: bool,
    ) -> Result<AnalyzeOutput, AnalyzeError> {
        let tool_path = self
            .validate_yt_dlp_available()
            .map_err(|error| AnalyzeError::new(error, None))?;

        if self.effective_config_owns_format() {
            return self.run_analyze_command(&tool_path, url, use_cookies, None);
        }

        self.run_analyze_command(&tool_path, url, use_cookies, Some(DEFAULT_FORMAT_SELECTOR))
    }

    pub fn analyze_music_stream_url(
        &self,
        url: &str,
        use_cookies: bool,
    ) -> Result<AnalyzeOutput, String> {
        self.analyze_music_stream_url_with_selector(url, use_cookies, MUSIC_STREAM_FORMAT_SELECTOR)
    }

    pub fn analyze_music_stream_url_detailed(
        &self,
        url: &str,
        use_cookies: bool,
    ) -> Result<AnalyzeOutput, AnalyzeError> {
        self.analyze_music_stream_url_with_selector_detailed(
            url,
            use_cookies,
            MUSIC_STREAM_FORMAT_SELECTOR,
        )
    }

    pub fn analyze_music_stream_url_with_selector(
        &self,
        url: &str,
        use_cookies: bool,
        format_selector: &str,
    ) -> Result<AnalyzeOutput, String> {
        self.analyze_music_stream_url_with_selector_detailed(url, use_cookies, format_selector)
            .map_err(|error| error.message)
    }

    pub fn analyze_music_stream_url_with_selector_detailed(
        &self,
        url: &str,
        use_cookies: bool,
        format_selector: &str,
    ) -> Result<AnalyzeOutput, AnalyzeError> {
        let tool_path = self
            .validate_yt_dlp_available()
            .map_err(|error| AnalyzeError::new(error, None))?;
        self.run_analyze_command(
            &tool_path,
            url,
            use_cookies,
            Some(format_selector.trim()).filter(|value| !value.is_empty()),
        )
    }

    pub fn prepare_music_stream_cache_command(
        &self,
        url: &str,
        output_path: &Path,
        format_selector: &str,
        use_cookies: bool,
    ) -> Result<Command, String> {
        let tool_path = self.validate_yt_dlp_available()?;
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create music cache folder: {error}"))?;
        }

        let mut command = Command::new(&tool_path);
        configure_background_command(&mut command);
        self.apply_common_yt_dlp_args(&mut command);
        command
            .arg("--no-playlist")
            .arg("--no-simulate")
            .arg("--no-part")
            .arg("--force-overwrites")
            .arg("--windows-filenames")
            .arg("--newline")
            .arg("--progress")
            .arg("--output")
            .arg(output_path);

        let selector = format_selector.trim();
        if !self.effective_config_owns_format() {
            command.arg("--format").arg(if selector.is_empty() {
                MUSIC_STREAM_FORMAT_SELECTOR
            } else {
                selector
            });
        }

        self.apply_cookie_args(&mut command, use_cookies, Some(url))?;
        self.apply_youtube_extractor_args(&mut command);
        command.arg(url);

        let command_line = format_command_line(&tool_path, &command);
        command
            .env("PYTHONIOENCODING", "utf-8")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        println!("[music-cache] command: {command_line}");
        Ok(command)
    }

    pub fn prepare_music_audio_download_command(
        &self,
        url: &str,
        output_dir: &Path,
        audio_format: Option<&str>,
        format_selector: &str,
        embed_cover: bool,
        write_tags: bool,
        use_cookies: bool,
    ) -> Result<PreparedDownload, String> {
        let tool_path = self.validate_yt_dlp_available()?;
        fs::create_dir_all(output_dir)
            .map_err(|error| format!("Could not create music download folder: {error}"))?;

        let target_format = audio_format.and_then(normalized_extension);
        let output_template = output_dir
            .join("%(artist)s - %(title)s.%(ext)s")
            .display()
            .to_string();
        let expected_output = target_format
            .as_ref()
            .map(|format| output_dir.join(format!("%(artist)s - %(title)s.{format}")))
            .unwrap_or_else(|| output_dir.join("%(artist)s - %(title)s.%(ext)s"));

        let mut command = Command::new(&tool_path);
        configure_background_command(&mut command);
        self.apply_common_yt_dlp_args(&mut command);
        command
            .arg("--no-playlist")
            .arg("--continue")
            .arg("--force-overwrites")
            .arg("--windows-filenames")
            .arg("--progress")
            .arg("--newline")
            .arg("--progress-template")
            .arg("[yt-dlp],%(progress._percent_str)s,%(progress._eta_str)s,%(progress.downloaded_bytes)s,%(progress.total_bytes)s,%(progress.speed)s,%(progress.eta)s")
            .arg("--no-simulate");

        if let Some(target_format) = target_format.as_ref() {
            command
                .arg("--extract-audio")
                .arg("--audio-format")
                .arg(target_format)
                .arg("--audio-quality")
                .arg("0");
        }

        if write_tags {
            command
                .arg("--embed-metadata")
                .arg("--metadata-from-title")
                .arg("%(artist)s - %(title)s");
            for parse_metadata_arg in music_album_parse_metadata_args() {
                command.arg("--parse-metadata").arg(parse_metadata_arg);
            }
        }

        if embed_cover
            && target_format
                .as_deref()
                .is_some_and(music_audio_download_format_supports_thumbnail)
        {
            command
                .arg("--embed-thumbnail")
                .arg("--convert-thumbnails")
                .arg("jpg");
        }

        command
            .arg("--print")
            .arg(format!("after_move:{FINAL_OUTPUT_PATH_PREFIX}%(filepath)j"))
            .arg("--output")
            .arg(output_template);

        if !self.effective_config_owns_format() {
            let selector = if format_selector.trim().is_empty() {
                target_format
                    .as_deref()
                    .map(music_audio_download_format_selector)
                    .unwrap_or(MUSIC_STREAM_FORMAT_SELECTOR)
            } else {
                format_selector
            };
            command.arg("--format").arg(selector);
        }

        self.apply_cookie_args(&mut command, use_cookies, Some(url))?;
        self.apply_youtube_extractor_args(&mut command);
        command.arg(url);

        let command_line = format_command_line(&tool_path, &command);
        command
            .env("PYTHONIOENCODING", "utf-8")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        Ok(PreparedDownload {
            command,
            output_path: expected_output,
            command_line,
        })
    }

    pub fn prepare_music_lyrics_cache_command(
        &self,
        url: &str,
        lyrics_dir: &Path,
        language_code: &str,
        use_cookies: bool,
    ) -> Result<Command, String> {
        let tool_path = self.validate_yt_dlp_available()?;
        fs::create_dir_all(lyrics_dir)
            .map_err(|error| format!("Could not create lyrics cache folder: {error}"))?;

        let output_template = lyrics_dir.join("lyrics.%(ext)s").display().to_string();
        let mut command = Command::new(&tool_path);
        configure_background_command(&mut command);
        self.apply_common_yt_dlp_args(&mut command);
        command
            .arg("--no-playlist")
            .arg("--skip-download")
            .arg("--force-overwrites")
            .arg("--windows-filenames")
            .arg("--write-subs")
            .arg("--sub-langs")
            .arg(language_code.trim())
            .arg("--sub-format")
            .arg("vtt/srt/ttml/srv3/srv2/srv1/best")
            .arg("--convert-subs")
            .arg("lrc")
            .arg("--output")
            .arg(output_template);

        self.apply_cookie_args(&mut command, use_cookies, Some(url))?;
        self.apply_youtube_extractor_args(&mut command);
        command.arg(url);

        command
            .env("PYTHONIOENCODING", "utf-8")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        println!(
            "[music-lyrics] command: {}",
            format_command_line(&tool_path, &command)
        );
        Ok(command)
    }

    pub fn prepare_download(&self, request: &DownloadRequest) -> Result<PreparedDownload, String> {
        let tool_path = self.validate_yt_dlp_available()?;

        let output_plan = build_output_plan(request)?;

        let mut command = Command::new(&tool_path);
        configure_background_command(&mut command);
        self.apply_common_yt_dlp_args(&mut command);
        command
            .arg("--no-playlist")
            .arg("--force-overwrites")
            .arg("--windows-filenames")
            .arg("--progress")
            .arg("--newline")
            .arg("--progress-template")
            .arg("[yt-dlp],%(info.format_id)s,%(progress._percent_str)s,%(progress._eta_str)s,%(progress.downloaded_bytes)s,%(progress.total_bytes)s,%(progress.speed)s,%(progress.eta)s");

        if request.target_kind != DownloadTargetKind::Subtitle {
            command
                .arg("--no-simulate")
                .arg("--print")
                .arg(format!("after_move:{FINAL_OUTPUT_PATH_PREFIX}%(filepath)j"));
        }

        if !self.effective_config_owns_output() || request.output_path.is_some() {
            command.arg("--output").arg(&output_plan.output_argument);
        }

        if request.target_kind != DownloadTargetKind::Subtitle
            && !request.use_aria2
            && !self.has_explicit_config()
            && self.concurrent_fragments > 1
        {
            command
                .arg("--concurrent-fragments")
                .arg(self.concurrent_fragments.to_string());
        }

        apply_live_from_start_arg(
            &mut command,
            request.target_kind,
            self.live_from_start,
            self.effective_config_owns_live_from_start(),
        );

        if request.target_kind != DownloadTargetKind::Subtitle
            && !self.effective_config_owns_limit_rate()
            && !self.limit_rate.trim().is_empty()
        {
            command.arg("--limit-rate").arg(self.limit_rate.trim());
        }

        let has_item_download_section = apply_download_section_args(
            &mut command,
            request.target_kind,
            &request.download_section_args,
            self.effective_config_owns_download_sections(),
        );

        let use_section_compatibility_mode = has_item_download_section
            && self.chapter_compatibility_mode
            && !self.has_explicit_config();

        if use_section_compatibility_mode {
            // Section downloads can trigger yt-dlp's direct-merge path, where the
            // process appears to stop at `[download] Destination:` and does not
            // emit normal per-format progress.  Force the classic separate
            // download + merge path for item-level chapter/range downloads.
            command.arg("--compat-options").arg("no-direct-merge");
        }

        if !self.effective_config_owns_mtime()
            && self.file_time_mode == FileTimeMode::UseDownloadTime
        {
            command.arg("--no-mtime");
        }

        if request.target_kind == DownloadTargetKind::Subtitle {
            command.arg("--skip-download");
        }

        if request.emit_json {
            command.arg("--print-json");
        }

        if output_plan.extract_audio {
            command.arg("--extract-audio");
            if let Some(audio_format) = output_plan.audio_format.as_deref() {
                command.arg("--audio-format").arg(audio_format);
            }
        }

        if request.target_kind != DownloadTargetKind::Subtitle
            && !self.effective_config_owns_format()
        {
            let requested_selector = request.format_selector.trim();
            let use_section_safe_selector = use_section_compatibility_mode
                && request.target_kind == DownloadTargetKind::Normal
                && (requested_selector.is_empty()
                    || selector_uses_separate_streams(requested_selector));
            let format_selector = if use_section_safe_selector {
                // Time-range downloads are handled by FFmpeg.  YouTube DASH
                // video+audio pairs can sit at `[download] Destination:` for a
                // long time with no visible progress, so prefer a muxed single
                // file for item-level chapter/range downloads.
                SECTION_FORMAT_SELECTOR
            } else if requested_selector.is_empty() {
                DEFAULT_FORMAT_SELECTOR
            } else {
                requested_selector
            };
            command.arg("--format").arg(format_selector);
        }

        apply_merge_output_container_arg(
            &mut command,
            request.target_kind,
            request.merge_output_container.as_deref(),
        );

        let forces_webm_container = request.target_kind == DownloadTargetKind::Normal
            && request
                .merge_output_container
                .as_deref()
                .is_some_and(|container| container.trim().eq_ignore_ascii_case("webm"));
        if forces_webm_container {
            // WebM cannot carry the cover-art attachment used by yt-dlp's
            // thumbnail embedder. The explicit per-item container choice owns
            // this conflict even when a loaded config requests embedding.
            command.arg("--no-embed-thumbnail");
        }

        if !self.has_explicit_config() {
            if let Some(remux_extension) = output_plan.remux_extension.as_deref() {
                command.arg("--remux-video").arg(remux_extension);
            }
        }

        self.apply_cookie_args(
            &mut command,
            request.use_cookies,
            Some(request.url.as_str()),
        )?;
        self.apply_youtube_extractor_args(&mut command);

        if !self.has_explicit_config() && request.use_aria2 {
            let aria2_path = resolve_tool_path(&self.aria2c);
            if aria2_path.is_file() {
                command.arg("--external-downloader").arg(aria2_path);
                if let Some(args) =
                    aria2_downloader_args(self.effective_proxy_url(), self.no_check_certificates)
                {
                    command.arg("--external-downloader-args").arg(args);
                }
            }
        }

        let should_write_subtitles = request.target_kind == DownloadTargetKind::Subtitle
            || (request.write_subtitles && !self.has_explicit_config());
        apply_subtitle_download_args(&mut command, request, should_write_subtitles);

        if !self.has_explicit_config() {
            if request.write_chapters && request.target_kind == DownloadTargetKind::Normal {
                command.arg("--split-chapters");
                command
                    .arg("--output")
                    .arg(chapter_output_argument(&output_plan.final_output_path));
            }

            if request.write_chapters
                && request.embed_chapters
                && request.target_kind != DownloadTargetKind::Subtitle
            {
                command.arg("--embed-chapters");
            }

            if request.write_thumbnail && request.embed_thumbnail && !forces_webm_container {
                command
                    .arg("--embed-thumbnail")
                    .arg("--convert-thumbnails")
                    .arg("jpg");
            } else if request.write_thumbnail {
                command
                    .arg("--write-thumbnail")
                    .arg("--convert-thumbnails")
                    .arg("jpg");
            }
        }

        command.arg(&request.url);

        let command_line = format_command_line(&tool_path, &command);
        command
            .env("PYTHONIOENCODING", "utf-8")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        Ok(PreparedDownload {
            command,
            output_path: output_plan.final_output_path,
            command_line,
        })
    }

    pub fn validate_yt_dlp_available(&self) -> Result<PathBuf, String> {
        let tool_path = resolve_tool_path(&self.yt_dlp);
        if tool_path.is_file() {
            return Ok(tool_path);
        }

        Err(format!(
            "yt-dlp was not found: {}. Install yt-dlp first, or handle dependency deployment in Options.",
            tool_path.display()
        ))
    }

    pub fn validate_cookie_setup(&self, use_cookies: bool) -> Result<(), String> {
        if !use_cookies || self.has_explicit_config() {
            return Ok(());
        }

        if self.uses_auto_cookie_source() {
            return Ok(());
        }

        if self.uses_cookie_file_source() {
            if self.cookie_file_path()?.is_some() {
                return Ok(());
            }
            return Err("Cookies are enabled and the cookie source is file, but no valid Netscape cookies.txt is selected.".to_owned());
        }

        let browser_cookie_arg =
            cookie_source_argument(&self.browser_cookie_source, &self.browser_cookie_profile);
        if browser_cookie_arg.trim().is_empty() {
            return Err(
                "Cookies are enabled, but no browser or cookies.txt source is selected.".to_owned(),
            );
        }

        Ok(())
    }

    fn apply_cookie_args(
        &self,
        command: &mut Command,
        use_cookies: bool,
        url: Option<&str>,
    ) -> Result<(), String> {
        self.validate_cookie_setup(use_cookies)?;
        if !use_cookies || self.has_explicit_config() {
            return Ok(());
        }

        if self.uses_auto_cookie_source() {
            if let Some(url) = url {
                if let Some(cookie_file) = auto_cookie_file_for_url(url)? {
                    command.arg("--cookies").arg(cookie_file);
                }
            }
            return Ok(());
        }

        if self.uses_cookie_file_source() {
            if let Some(cookie_file) = self.cookie_file_path()? {
                command.arg("--cookies").arg(cookie_file);
            }
            return Ok(());
        }

        let browser_cookie_arg =
            cookie_source_argument(&self.browser_cookie_source, &self.browser_cookie_profile);
        command
            .arg("--cookies-from-browser")
            .arg(browser_cookie_arg.trim());
        Ok(())
    }

    fn uses_auto_cookie_source(&self) -> bool {
        self.browser_cookie_source
            .trim()
            .eq_ignore_ascii_case(COOKIE_SOURCE_AUTO)
    }

    fn uses_cookie_file_source(&self) -> bool {
        self.browser_cookie_source
            .trim()
            .eq_ignore_ascii_case(COOKIE_SOURCE_FILE)
    }

    fn apply_youtube_extractor_args(&self, command: &mut Command) {
        if self.has_explicit_config() {
            return;
        }

        if let Some(args) = self.youtube_extractor_args_argument() {
            command.arg("--extractor-args").arg(args);
        }
    }

    fn youtube_extractor_args_argument(&self) -> Option<String> {
        let mut args = Vec::new();
        if let Some(po_token) = normalized_youtube_subs_po_token(&self.youtube_subs_po_token) {
            args.push(format!("po_token={po_token}"));
        }
        if let Some(raw_args) = normalized_youtube_extractor_args(&self.youtube_extractor_args) {
            args.push(raw_args);
        }

        if args.is_empty() {
            None
        } else {
            Some(format!("youtube:{}", args.join(";")))
        }
    }

    fn cookie_file_path(&self) -> Result<Option<PathBuf>, String> {
        let trimmed = self.browser_cookie_file.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let path = resolve_support_path(trimmed);
        if path.is_file() {
            Ok(Some(path))
        } else {
            Err(format!(
                "Cookie file was not found: {}. Choose a Netscape cookies.txt again, or change the cookie source back to browser.",
                path.display()
            ))
        }
    }

    fn apply_common_yt_dlp_args(&self, command: &mut Command) {
        if let Some(config_path) = self.effective_config_path() {
            command.arg("--config-location").arg(config_path);
        } else {
            command.arg("--ignore-config");
        }

        if let Some(proxy_url) = self.effective_proxy_url() {
            command.arg("--proxy").arg(proxy_url);
        }

        if self.no_check_certificates {
            command.arg("--no-check-certificates");
        }

        let ffmpeg_path = resolve_tool_path(&self.ffmpeg);
        if ffmpeg_path.is_file() {
            command.arg("--ffmpeg-location").arg(ffmpeg_path);
        }

        let deno_path = resolve_tool_path(&self.deno);
        if deno_path.is_file() {
            command
                .arg("--js-runtimes")
                .arg(format!("deno:{}", deno_path.display()));
        }

        match self.cache_mode {
            CacheLocationMode::YtDlpDefault => {}
            CacheLocationMode::V2Cache => {
                let cache_root = resolve_support_path(&self.cache_dir);
                let yt_dlp_cache_dir = cache_root.join("yt-dlp");
                let yt_dlp_temp_dir = cache_root.join("yt-dlp-temp");
                command.arg("--cache-dir").arg(&yt_dlp_cache_dir);
                command
                    .arg("-P")
                    .arg(format!("temp:{}", yt_dlp_temp_dir.display()));
            }
            CacheLocationMode::WindowsTemp => {
                let temp_dir = std::env::temp_dir();
                command.arg("--cache-dir").arg(&temp_dir);
                command
                    .arg("-P")
                    .arg(format!("temp:{}", temp_dir.display()));
            }
        }
    }

    fn explicit_config_path(&self) -> Option<PathBuf> {
        let trimmed = self.yt_dlp_config.trim();
        if trimmed.is_empty() {
            return None;
        }

        let path = resolve_support_path(trimmed);
        path.is_file().then_some(path)
    }

    fn portable_config_path(&self) -> Option<PathBuf> {
        let tool_path = resolve_tool_path(&self.yt_dlp);
        let parent = tool_path.parent()?;
        let config_path = parent.join("yt-dlp.conf");
        config_path.is_file().then_some(config_path)
    }
}

impl ToolPaths {
    fn run_analyze_command(
        &self,
        tool_path: &Path,
        url: &str,
        use_cookies: bool,
        format_selector: Option<&str>,
    ) -> Result<AnalyzeOutput, AnalyzeError> {
        let mut command = Command::new(tool_path);
        configure_background_command(&mut command);
        self.apply_common_yt_dlp_args(&mut command);
        command
            .arg("--dump-single-json")
            .arg("--no-warnings")
            .arg("--no-playlist");

        if let Some(selector) = format_selector.filter(|value| !value.trim().is_empty()) {
            command.arg("-f").arg(selector);
        }

        self.apply_cookie_args(&mut command, use_cookies, Some(url))
            .map_err(|error| AnalyzeError::new(error, None))?;
        self.apply_youtube_extractor_args(&mut command);
        command.arg(url);

        let command_line = format_command_line(tool_path, &command);
        println!("[analyze] command: {command_line}");

        let output =
            run_tracked_command_output(&mut command, "yt-dlp analyze").map_err(|error| {
                AnalyzeError::new(
                    format!("Could not start yt-dlp: {error}"),
                    Some(command_line.clone()),
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            let detail = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                format!("exit code {:?}", output.status.code())
            };
            let detail = humanize_yt_dlp_error(&detail);
            return Err(AnalyzeError::new(
                format!("yt-dlp analysis failed: {detail}"),
                Some(command_line),
            ));
        }

        let json = serde_json::from_slice::<Value>(&output.stdout).map_err(|error| {
            AnalyzeError::new(
                format!("Could not parse yt-dlp JSON: {error}"),
                Some(command_line.clone()),
            )
        })?;
        Ok(AnalyzeOutput { json, command_line })
    }
}

pub fn humanize_yt_dlp_error(detail: &str) -> String {
    if is_chromium_cookie_database_copy_error(detail) {
        return format!(
            r#"Could not read the Chromium/Chrome cookie database directly. This is usually because the browser locked the Network\Cookies database, not because the checkbox state is wrong. Close the browser and retry, or change Cookie source to Use file (cookies.txt) in Advanced. Original message: {detail}"#
        );
    }
    if is_youtube_subtitle_429_error(detail) {
        return format!(
            r#"YouTube rejected the subtitle request (HTTP 429 Too Many Requests). This often happens on the YouTube auto-translation timedtext endpoint. cookies.txt can provide login state, but may not satisfy PO Token / rate-limit requirements for that endpoint. The GUI keeps the native yt-dlp flow and diagnostic logs instead of switching to a custom downloader. Original message: {detail}"#
        );
    }
    detail.to_owned()
}

fn is_youtube_subtitle_429_error(detail: &str) -> bool {
    let normalized = detail.to_ascii_lowercase();
    normalized.contains("unable to download video subtitles")
        && normalized.contains("http error 429")
}

fn is_chromium_cookie_database_copy_error(detail: &str) -> bool {
    let normalized = detail.to_ascii_lowercase();
    normalized.contains("could not copy chrome cookie database")
        || (normalized.contains("could not copy") && normalized.contains("cookie database"))
}

pub fn configure_background_command(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn config_contains_output_arg(path: &Path) -> bool {
    config_contains_any_arg(
        path,
        &["-o", "--output", "-P", "--paths"],
        &["--output=", "--paths="],
    )
}

fn config_contains_format_arg(path: &Path) -> bool {
    config_contains_any_arg(path, &["-f", "--format"], &["--format="])
}

fn config_contains_limit_rate_arg(path: &Path) -> bool {
    config_contains_any_arg(path, &["-r", "--limit-rate"], &["--limit-rate="])
}

fn selector_uses_separate_streams(selector: &str) -> bool {
    selector.split('/').any(|part| part.contains('+'))
}

fn aria2_downloader_args(proxy_url: Option<&str>, no_check_certificates: bool) -> Option<String> {
    let mut args = Vec::new();
    if let Some(proxy_url) = proxy_url.map(str::trim).filter(|value| !value.is_empty()) {
        args.push(format!("--all-proxy={proxy_url}"));
    }
    if no_check_certificates {
        args.push("--check-certificate=false".to_owned());
    }
    (!args.is_empty()).then(|| format!("aria2c:{}", args.join(" ")))
}

fn config_contains_download_sections_arg(path: &Path) -> bool {
    config_contains_any_arg(path, &["--download-sections"], &["--download-sections="])
}

fn config_contains_live_from_start_arg(path: &Path) -> bool {
    config_contains_any_arg(path, &["--live-from-start", "--no-live-from-start"], &[])
}

fn apply_download_section_args(
    command: &mut Command,
    target_kind: DownloadTargetKind,
    download_section_args: &[String],
    config_owns_download_sections: bool,
) -> bool {
    let has_item_download_section =
        target_kind != DownloadTargetKind::Subtitle && !download_section_args.is_empty();
    if has_item_download_section && !config_owns_download_sections {
        for section in download_section_args {
            command.arg("--download-sections").arg(section.trim());
        }
    }
    has_item_download_section
}

fn apply_live_from_start_arg(
    command: &mut Command,
    target_kind: DownloadTargetKind,
    enabled: bool,
    config_owns_setting: bool,
) {
    if target_kind != DownloadTargetKind::Subtitle && enabled && !config_owns_setting {
        command.arg("--live-from-start");
    }
}

fn apply_merge_output_container_arg(
    command: &mut Command,
    target_kind: DownloadTargetKind,
    container: Option<&str>,
) {
    if target_kind != DownloadTargetKind::Normal {
        return;
    }
    if let Some(container) = container.and_then(normalized_download_merge_container) {
        command.arg("--merge-output-format").arg(container);
    }
}

fn apply_subtitle_download_args(
    command: &mut Command,
    request: &DownloadRequest,
    should_write_subtitles: bool,
) {
    if !should_write_subtitles {
        return;
    }
    let Some(subtitle_lang) = request.subtitle_lang.as_deref() else {
        return;
    };

    command.arg("--sub-langs").arg(subtitle_lang);
    if request.write_auto_subs {
        command.arg("--write-auto-subs");
        if request.subtitle_is_auto_translated {
            command.arg("--sleep-subtitles").arg("1");
        }
    } else if request.target_kind == DownloadTargetKind::Subtitle || !request.embed_subtitles {
        // yt-dlp keeps the sidecar when --write-subs and --embed-subs are
        // explicitly combined. Embed mode intentionally relies on --embed-subs
        // to request normal subtitles and remove its temporary sidecar.
        command.arg("--write-subs");
    }

    let subtitle_ext =
        normalized_extension(&request.subtitle_ext).unwrap_or_else(|| "srt".to_owned());
    if request.target_kind == DownloadTargetKind::Subtitle {
        command
            .arg("--sub-format")
            .arg(subtitle_format_preference(&subtitle_ext));
        if subtitle_conversion_is_supported(&subtitle_ext) {
            command.arg("--convert-subs").arg(&subtitle_ext);
        }
    } else if request.embed_subtitles {
        command.arg("--embed-subs");
    } else {
        command.arg("--convert-subs").arg("srt");
    }
}

fn config_contains_mtime_arg(path: &Path) -> bool {
    config_contains_any_arg(path, &["--mtime", "--no-mtime"], &["--mtime="])
}

fn config_contains_any_arg(path: &Path, exact_args: &[&str], prefix_args: &[&str]) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };

    content.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && trimmed.split_whitespace().any(|token| {
                exact_args.contains(&token)
                    || prefix_args.iter().any(|prefix| token.starts_with(prefix))
            })
    })
}

fn auto_cookie_file_for_url(url: &str) -> Result<Option<PathBuf>, String> {
    let Some(host) = auto_cookie_host_from_url(url) else {
        return Ok(None);
    };

    let cookie_dir = portable_root_dir().join("data").join("cookies");
    let Some(index) = read_cookie_site_index(&cookie_dir)? else {
        return Ok(None);
    };

    for site in index.sites {
        let cookie_file = site.cookie_file.trim();
        if cookie_file.is_empty() {
            continue;
        }

        let matches = site
            .match_domains
            .iter()
            .map(String::as_str)
            .any(|domain| auto_cookie_domain_matches(&host, domain));
        if !matches {
            continue;
        }

        let candidate = PathBuf::from(cookie_file);
        let path = if candidate.is_absolute() {
            candidate
        } else {
            cookie_dir.join(candidate)
        };
        if path.is_file() {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn auto_cookie_host_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let rest = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next()?.trim();
    if authority.is_empty() {
        return None;
    }
    let host = authority
        .rsplit('@')
        .next()
        .unwrap_or(authority)
        .split(':')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if host.is_empty() { None } else { Some(host) }
}

fn auto_cookie_domain_matches(host: &str, domain: &str) -> bool {
    let domain = domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if domain.is_empty() {
        return false;
    }
    host == domain || host.ends_with(&format!(".{domain}"))
}

fn cookie_source_argument(source: &str, profile: &str) -> String {
    let source = source.trim();
    if source.is_empty() {
        return String::new();
    }
    let profile = profile.trim();
    if profile.is_empty() {
        source.to_owned()
    } else {
        format!("{source}:{profile}")
    }
}

fn browser_cookie_source_candidates() -> [BrowserCookieSourceOption; 7] {
    [
        BrowserCookieSourceOption {
            value: "brave",
            label: "Brave",
        },
        BrowserCookieSourceOption {
            value: "chrome",
            label: "Chrome",
        },
        BrowserCookieSourceOption {
            value: "chromium",
            label: "Chromium",
        },
        BrowserCookieSourceOption {
            value: "edge",
            label: "Edge",
        },
        BrowserCookieSourceOption {
            value: "firefox",
            label: "Firefox",
        },
        BrowserCookieSourceOption {
            value: "opera",
            label: "Opera",
        },
        BrowserCookieSourceOption {
            value: "vivaldi",
            label: "Vivaldi",
        },
    ]
}

fn browser_cookie_source_exists(browser: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        let roaming = std::env::var_os("APPDATA").map(PathBuf::from);
        match browser {
            "brave" => local
                .map(|base| base.join(r"BraveSoftware\Brave-Browser\User Data"))
                .is_some_and(|path| path.exists()),
            "chrome" => local
                .map(|base| base.join(r"Google\Chrome\User Data"))
                .is_some_and(|path| path.exists()),
            "chromium" => local
                .map(|base| base.join(r"Chromium\User Data"))
                .is_some_and(|path| path.exists()),
            "edge" => local
                .map(|base| base.join(r"Microsoft\Edge\User Data"))
                .is_some_and(|path| path.exists()),
            "firefox" => roaming
                .map(|base| base.join(r"Mozilla\Firefox\Profiles"))
                .is_some_and(|path| path.exists()),
            "opera" => roaming
                .map(|base| base.join(r"Opera Software\Opera Stable"))
                .is_some_and(|path| path.exists()),
            "vivaldi" => local
                .map(|base| base.join(r"Vivaldi\User Data"))
                .is_some_and(|path| path.exists()),
            _ => false,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = browser;
        false
    }
}

fn browser_cookie_profiles(browser: &str) -> Vec<BrowserCookieProfileOption> {
    #[cfg(target_os = "windows")]
    {
        let Some(base_dir) = browser_cookie_base_dir(browser) else {
            return Vec::new();
        };
        let mut items = Vec::new();
        let chromium_profile_names = chromium_profile_display_names(&base_dir);

        let single_profile = matches!(browser, "opera");
        if single_profile {
            if base_dir.exists() {
                items.push(BrowserCookieProfileOption {
                    value: "Default".to_owned(),
                    label: chromium_profile_names
                        .get("Default")
                        .cloned()
                        .unwrap_or_else(|| "Default".to_owned()),
                });
            }
            return items;
        }

        let Ok(entries) = fs::read_dir(base_dir) else {
            return items;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if name == "Default" || name == "Guest Profile" || name.starts_with("Profile ") {
                let label = chromium_profile_names
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.to_owned());
                items.push(BrowserCookieProfileOption {
                    value: name.to_owned(),
                    label,
                });
            }
        }

        if browser == "firefox" {
            items.clear();
            let Ok(entries) = fs::read_dir(browser_cookie_base_dir(browser).unwrap()) else {
                return items;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                    continue;
                };
                items.push(BrowserCookieProfileOption {
                    value: name.to_owned(),
                    label: name.to_owned(),
                });
            }
        }

        items.sort_by(|left, right| left.label.cmp(&right.label));
        items
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = browser;
        Vec::new()
    }
}

#[cfg(target_os = "windows")]
fn browser_cookie_base_dir(browser: &str) -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let roaming = std::env::var_os("APPDATA").map(PathBuf::from);
    match browser {
        "brave" => local.map(|base| base.join(r"BraveSoftware\Brave-Browser\User Data")),
        "chrome" => local.map(|base| base.join(r"Google\Chrome\User Data")),
        "chromium" => local.map(|base| base.join(r"Chromium\User Data")),
        "edge" => local.map(|base| base.join(r"Microsoft\Edge\User Data")),
        "firefox" => roaming.map(|base| base.join(r"Mozilla\Firefox\Profiles")),
        "opera" => roaming.map(|base| base.join(r"Opera Software\Opera Stable")),
        "vivaldi" => local.map(|base| base.join(r"Vivaldi\User Data")),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn chromium_profile_display_names(base_dir: &Path) -> std::collections::HashMap<String, String> {
    let mut names = std::collections::HashMap::new();
    let local_state_path = base_dir.join("Local State");
    let Ok(content) = fs::read_to_string(local_state_path) else {
        return names;
    };
    let Ok(json) = serde_json::from_str::<Value>(&content) else {
        return names;
    };
    let Some(info_cache) = json
        .get("profile")
        .and_then(|profile| profile.get("info_cache"))
        .and_then(Value::as_object)
    else {
        return names;
    };

    for (folder, info) in info_cache {
        let display_name = info
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| {
                info.get("shortcut_name")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
            });
        let user_name = info
            .get("user_name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        let Some(display_name) = display_name else {
            continue;
        };
        let label = match user_name {
            Some(user_name) if !user_name.eq_ignore_ascii_case(&display_name) => {
                format!("{display_name} ({user_name})")
            }
            _ => display_name,
        };
        names.insert(folder.to_owned(), label);
    }

    names
}

pub fn looks_like_playlist_url(url: &str) -> bool {
    url.contains("list=") || url.contains("/playlist")
}

pub fn youtube_url_has_video_and_playlist(url: &str) -> bool {
    looks_like_youtube_url(url) && youtube_video_id(url).is_some() && youtube_list_id(url).is_some()
}

pub fn youtube_url_force_single_video(url: &str) -> Option<String> {
    let video_id = youtube_video_id(url)?;
    Some(format!("https://www.youtube.com/watch?v={video_id}"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YoutubePlaylistKind {
    ChannelPopularOrGenerated,
    YoutubeMixRadio,
    YoutubeMusicAlbum,
    LikedVideos,
    FavoritesLegacy,
}

impl YoutubePlaylistKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::ChannelPopularOrGenerated => "YouTube generated channel playlist",
            Self::YoutubeMixRadio => "YouTube Mix / Radio",
            Self::YoutubeMusicAlbum => "YouTube Music album/collection",
            Self::LikedVideos => "Liked videos",
            Self::FavoritesLegacy => "Legacy favorites playlist",
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::ChannelPopularOrGenerated => "options.playlist_risk.kind.channel_generated",
            Self::YoutubeMixRadio => "options.playlist_risk.kind.youtube_mix_radio",
            Self::YoutubeMusicAlbum => "options.playlist_risk.kind.youtube_music_album",
            Self::LikedVideos => "options.playlist_risk.kind.liked_videos",
            Self::FavoritesLegacy => "options.playlist_risk.kind.favorites_legacy",
        }
    }

    pub fn note_key(self) -> &'static str {
        match self {
            Self::ChannelPopularOrGenerated => "options.playlist_risk.note.channel_generated",
            Self::YoutubeMixRadio => "options.playlist_risk.note.youtube_mix_radio",
            Self::YoutubeMusicAlbum => "options.playlist_risk.note.youtube_music_album",
            Self::LikedVideos => "options.playlist_risk.note.liked_videos",
            Self::FavoritesLegacy => "options.playlist_risk.note.favorites_legacy",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YoutubePlaylistRisk {
    pub kind: YoutubePlaylistKind,
    pub note: &'static str,
}

pub fn classify_youtube_playlist(url: &str) -> Option<YoutubePlaylistRisk> {
    if !looks_like_youtube_url(url) {
        return None;
    }

    let list_id = youtube_list_id(url)?;
    let uppercase = list_id.to_ascii_uppercase();

    let risk = if uppercase.starts_with("RDCLAK5UY") || uppercase.starts_with("RDCMUC") {
        YoutubePlaylistRisk {
            kind: YoutubePlaylistKind::YoutubeMixRadio,
            note: "This Mix / Radio playlist may contain many items and can change over time.",
        }
    } else if uppercase.starts_with("UULP") || uppercase.starts_with("UUSH") {
        YoutubePlaylistRisk {
            kind: YoutubePlaylistKind::ChannelPopularOrGenerated,
            note: "Treat this YouTube-generated channel playlist conservatively.",
        }
    } else if uppercase.starts_with("RD") {
        YoutubePlaylistRisk {
            kind: YoutubePlaylistKind::YoutubeMixRadio,
            note: "This Mix / Radio playlist may contain many items and can change over time.",
        }
    } else if uppercase == "LL" {
        YoutubePlaylistRisk {
            kind: YoutubePlaylistKind::LikedVideos,
            note: "Liked videos usually require login or cookies.",
        }
    } else if uppercase.starts_with("FL") {
        YoutubePlaylistRisk {
            kind: YoutubePlaylistKind::FavoritesLegacy,
            note: "This is a legacy favorites playlist style and may not be stable now.",
        }
    } else if uppercase.starts_with("OLAK5UY") {
        YoutubePlaylistRisk {
            kind: YoutubePlaylistKind::YoutubeMusicAlbum,
            note: "This is usually a YouTube Music album or collection.",
        }
    } else {
        return None;
    };

    Some(risk)
}

fn looks_like_youtube_url(url: &str) -> bool {
    let normalized = url.to_ascii_lowercase();
    normalized.contains("youtube.com") || normalized.contains("youtu.be")
}

fn youtube_list_id(url: &str) -> Option<&str> {
    let (_, tail) = url.split_once("list=")?;
    tail.split(['&', '#', '?'])
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn youtube_video_id(url: &str) -> Option<&str> {
    let normalized = url.to_ascii_lowercase();
    if normalized.contains("youtu.be/") {
        let (_, tail) = url.split_once("youtu.be/")?;
        return tail
            .split(['&', '#', '?', '/'])
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty());
    }

    let (_, tail) = url.split_once("v=")?;
    tail.split(['&', '#', '?', '/'])
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub fn playlist_entry_url(entry: &Value) -> Option<String> {
    entry
        .get("webpage_url")
        .or_else(|| entry.get("original_url"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            let raw_url = entry.get("url").and_then(Value::as_str)?;
            if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
                Some(raw_url.to_owned())
            } else {
                let id = entry.get("id").and_then(Value::as_str).unwrap_or(raw_url);
                Some(format!("https://www.youtube.com/watch?v={id}"))
            }
        })
}

fn build_output_plan(request: &DownloadRequest) -> Result<OutputPlan, String> {
    if let Some(output_path) = request
        .output_path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        return build_explicit_output_plan(request, output_path);
    }

    let output_dir = resolve_output_dir(&request.output_dir)?;
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("Could not create download folder: {error}"))?;
    Ok(build_directory_output_plan(request, &output_dir))
}

fn build_directory_output_plan(request: &DownloadRequest, output_dir: &Path) -> OutputPlan {
    let expected_extension = determine_expected_container(request);
    let (base_name, forced_extension) = split_file_name_parts(request.file_name.trim());
    let base_name = if base_name.is_empty() {
        "%(title)s".to_owned()
    } else {
        base_name
    };

    let has_explicit_merge_container = request
        .merge_output_container
        .as_deref()
        .and_then(normalized_download_merge_container)
        .is_some();
    let final_extension = if has_explicit_merge_container {
        expected_extension.clone()
    } else {
        forced_extension
            .clone()
            .unwrap_or_else(|| expected_extension.clone())
    };
    let final_output_path = output_dir.join(format!("{base_name}.{final_extension}"));
    let output_base_name =
        output_base_name_with_section_number(&base_name, request.download_section_args.len());

    if !has_explicit_merge_container
        && request.target_kind != DownloadTargetKind::Subtitle
        && forced_extension
            .as_deref()
            .is_some_and(|ext| !ext.eq_ignore_ascii_case(&expected_extension))
    {
        OutputPlan {
            output_argument: output_dir
                .join(format!("{output_base_name}.%(ext)s"))
                .display()
                .to_string(),
            final_output_path,
            remux_extension: Some(final_extension),
            extract_audio: false,
            audio_format: None,
        }
    } else {
        OutputPlan {
            output_argument: output_argument_for_kind(
                request.target_kind,
                &output_dir.join(format!("{output_base_name}.{final_extension}")),
            ),
            final_output_path,
            remux_extension: None,
            extract_audio: false,
            audio_format: None,
        }
    }
}

fn normalized_youtube_subs_po_token(value: &str) -> Option<String> {
    let mut token = value.trim();
    if token.is_empty() {
        return None;
    }
    if let Some(rest) = token.strip_prefix("youtube:") {
        token = rest.trim();
    }
    if let Some(rest) = token.strip_prefix("po_token=") {
        token = rest.trim();
    }
    if token.is_empty() {
        return None;
    }

    if token.contains('+') {
        Some(token.to_owned())
    } else {
        Some(format!("web.subs+{token}"))
    }
}

fn normalized_youtube_extractor_args(value: &str) -> Option<String> {
    let mut args = value.trim();
    if args.is_empty() {
        return None;
    }
    if let Some(rest) = args.strip_prefix("--extractor-args") {
        args = rest.trim().trim_matches('"').trim_matches('\'');
    }
    if let Some(rest) = args.strip_prefix("youtube:") {
        args = rest.trim();
    }
    if args.is_empty() {
        None
    } else {
        Some(args.to_owned())
    }
}

fn subtitle_format_preference(extension: &str) -> String {
    if subtitle_conversion_is_supported(extension) {
        format!("{extension}/best")
    } else {
        extension.to_owned()
    }
}

fn subtitle_conversion_is_supported(extension: &str) -> bool {
    matches!(extension, "ass" | "lrc" | "srt" | "vtt")
}

fn determine_expected_container(request: &DownloadRequest) -> String {
    if request.target_kind == DownloadTargetKind::Normal {
        if let Some(container) = request
            .merge_output_container
            .as_deref()
            .and_then(normalized_download_merge_container)
        {
            return container;
        }
    }

    match request.target_kind {
        DownloadTargetKind::Audio => {
            return normalized_extension(&request.audio_ext).unwrap_or_else(|| "m4a".to_owned());
        }
        DownloadTargetKind::Video => {
            return normalized_extension(&request.video_ext).unwrap_or_else(|| "mkv".to_owned());
        }
        DownloadTargetKind::Subtitle => {
            return normalized_extension(&request.subtitle_ext).unwrap_or_else(|| "srt".to_owned());
        }
        DownloadTargetKind::Normal => {}
    }

    if request.is_muxed_video {
        return normalized_extension(&request.video_ext).unwrap_or_else(|| "mkv".to_owned());
    }

    let video_ext = normalized_extension(&request.video_ext);
    let audio_ext = normalized_extension(&request.audio_ext);

    match (video_ext.as_deref(), audio_ext.as_deref()) {
        (Some("webm"), Some("webm")) => "webm".to_owned(),
        (Some("mp4"), Some("m4a" | "aac")) => "mp4".to_owned(),
        _ => "mkv".to_owned(),
    }
}

fn build_explicit_output_plan(
    request: &DownloadRequest,
    output_path: &str,
) -> Result<OutputPlan, String> {
    let requested_path = PathBuf::from(output_path);
    let parent = requested_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&parent)
        .map_err(|error| format!("Could not create download folder: {error}"))?;

    let expected_extension = determine_expected_container(request);
    let (base_name, forced_extension) = split_file_name_parts(
        requested_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default(),
    );
    let final_extension = forced_extension
        .clone()
        .unwrap_or_else(|| expected_extension.clone());
    let final_output_path = parent.join(format!("{base_name}.{final_extension}"));
    let output_base_name =
        output_base_name_with_section_number(&base_name, request.download_section_args.len());
    let output_path = parent.join(format!("{output_base_name}.{final_extension}"));

    match request.target_kind {
        DownloadTargetKind::Audio => Ok(OutputPlan {
            output_argument: parent.join(output_base_name).display().to_string(),
            final_output_path,
            remux_extension: None,
            extract_audio: true,
            audio_format: Some(final_extension),
        }),
        DownloadTargetKind::Video | DownloadTargetKind::Subtitle | DownloadTargetKind::Normal => {
            let remux_extension = forced_extension
                .as_deref()
                .filter(|_| request.target_kind != DownloadTargetKind::Subtitle)
                .filter(|ext| !ext.eq_ignore_ascii_case(&expected_extension))
                .map(ToOwned::to_owned);
            Ok(OutputPlan {
                output_argument: output_argument_for_kind(request.target_kind, &output_path),
                final_output_path,
                remux_extension,
                extract_audio: false,
                audio_format: None,
            })
        }
    }
}

fn output_base_name_with_section_number(base_name: &str, section_count: usize) -> String {
    if section_count > 1 {
        format!("{base_name} - %(section_number)02d")
    } else {
        base_name.to_owned()
    }
}

fn chapter_output_argument(path: &Path) -> String {
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("chapter");
    let template = format!("{stem} - %(section_number)03d. %(section_title)s.%(ext)s");
    format!("chapter:{}", parent.join(template).display())
}

fn output_argument_for_kind(kind: DownloadTargetKind, path: &Path) -> String {
    match kind {
        DownloadTargetKind::Subtitle => format!("subtitle:{}", path.display()),
        DownloadTargetKind::Normal | DownloadTargetKind::Video | DownloadTargetKind::Audio => {
            path.display().to_string()
        }
    }
}

fn split_file_name_parts(file_name: &str) -> (String, Option<String>) {
    if file_name.is_empty() {
        return (String::new(), None);
    }

    let path = Path::new(file_name);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .and_then(normalized_extension)
        .filter(|extension| is_known_output_extension(extension));

    if let Some(extension) = extension {
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or(file_name)
            .to_owned();
        (stem, Some(extension))
    } else {
        (file_name.to_owned(), None)
    }
}

fn is_known_output_extension(extension: &str) -> bool {
    matches!(
        extension,
        "mp4"
            | "mkv"
            | "webm"
            | "mov"
            | "flv"
            | "avi"
            | "mp3"
            | "m4a"
            | "flac"
            | "wav"
            | "opus"
            | "aac"
            | "vorbis"
            | "alac"
            | "srt"
            | "vtt"
            | "ass"
            | "ssa"
            | "lrc"
            | "ttml"
            | "dfxp"
            | "json3"
            | "srv3"
            | "srv2"
            | "srv1"
    )
}

fn music_audio_download_format_selector(target_format: &str) -> &'static str {
    match target_format
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
        .as_str()
    {
        "mp3" => "bestaudio[ext=mp3]/bestaudio[acodec^=mp3]/bestaudio/best[acodec!=none]",
        "m4a" | "aac" => {
            "bestaudio[ext=m4a]/bestaudio[acodec^=mp4a]/bestaudio[acodec^=aac]/bestaudio/best[acodec!=none]"
        }
        "opus" => {
            "bestaudio[acodec^=opus]/bestaudio[ext=opus]/bestaudio[ext=webm][acodec^=opus]/bestaudio/best[acodec!=none]"
        }
        "flac" => "bestaudio[ext=flac]/bestaudio[acodec^=flac]/bestaudio/best[acodec!=none]",
        "wav" => "bestaudio[ext=wav]/bestaudio[acodec^=pcm]/bestaudio/best[acodec!=none]",
        _ => MUSIC_STREAM_FORMAT_SELECTOR,
    }
}

fn music_audio_download_format_supports_thumbnail(format: &str) -> bool {
    matches!(
        format.trim().to_ascii_lowercase().as_str(),
        "mp3" | "m4a" | "flac"
    )
}

fn music_album_parse_metadata_args() -> [&'static str; 2] {
    [
        "%(playlist|)s:%(meta_album)s",
        "%(playlist_title|)s:%(meta_album)s",
    ]
}

fn normalized_extension(extension: &str) -> Option<String> {
    let trimmed = extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn normalized_download_merge_container(container: &str) -> Option<String> {
    normalized_extension(container).filter(|container| matches!(container.as_str(), "mkv" | "webm"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subtitle_test_request(write_auto_subs: bool, embed_subtitles: bool) -> DownloadRequest {
        DownloadRequest {
            target_kind: DownloadTargetKind::Normal,
            url: "https://example.invalid/video".to_owned(),
            format_selector: String::new(),
            video_selector: String::new(),
            audio_selector: String::new(),
            is_muxed_video: true,
            video_ext: "mkv".to_owned(),
            audio_ext: String::new(),
            merge_output_container: None,
            upload_date: String::new(),
            subtitle_lang: Some("en".to_owned()),
            subtitle_ext: "vtt".to_owned(),
            subtitle_source_ext: "vtt".to_owned(),
            subtitle_url: None,
            write_auto_subs,
            subtitle_is_auto_translated: false,
            write_subtitles: true,
            embed_subtitles,
            write_chapters: false,
            embed_chapters: false,
            write_thumbnail: false,
            embed_thumbnail: false,
            use_cookies: false,
            use_aria2: false,
            emit_json: false,
            output_path: None,
            output_dir: ".".to_owned(),
            file_name: "video".to_owned(),
            download_section_args: Vec::new(),
        }
    }

    fn command_args(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect()
    }

    #[test]
    fn live_from_start_applies_only_to_media_downloads() {
        let mut media = Command::new("yt-dlp");
        apply_live_from_start_arg(&mut media, DownloadTargetKind::Normal, true, false);
        assert_eq!(command_args(&media), vec!["--live-from-start"]);

        let mut subtitle = Command::new("yt-dlp");
        apply_live_from_start_arg(&mut subtitle, DownloadTargetKind::Subtitle, true, false);
        assert!(command_args(&subtitle).is_empty());

        let mut config_owned = Command::new("yt-dlp");
        apply_live_from_start_arg(&mut config_owned, DownloadTargetKind::Normal, true, true);
        assert!(command_args(&config_owned).is_empty());
    }

    #[test]
    fn multiple_item_ranges_repeat_download_sections_in_timeline_order() {
        let mut command = Command::new("yt-dlp");
        let ranges = vec![
            "*00:00:10-00:00:20".to_owned(),
            "*00:00:30-00:00:50".to_owned(),
        ];

        assert!(apply_download_section_args(
            &mut command,
            DownloadTargetKind::Normal,
            &ranges,
            false,
        ));
        assert_eq!(
            command_args(&command),
            vec![
                "--download-sections",
                "*00:00:10-00:00:20",
                "--download-sections",
                "*00:00:30-00:00:50",
            ]
        );
    }

    #[test]
    fn multiple_item_ranges_add_section_number_to_output_template() {
        let mut request = subtitle_test_request(false, false);
        request.target_kind = DownloadTargetKind::Normal;
        request.subtitle_lang = None;
        request.video_ext = "mkv".to_owned();
        request.download_section_args = vec![
            "*00:00:10-00:00:20".to_owned(),
            "*00:00:30-00:00:50".to_owned(),
        ];

        let plan = build_directory_output_plan(&request, Path::new("output"));

        assert!(
            plan.output_argument
                .contains("video - %(section_number)02d.mkv")
        );
        assert!(
            plan.final_output_path
                .ends_with(Path::new("output").join("video.mkv"))
        );
    }

    #[test]
    fn explicit_webm_container_controls_merge_and_expected_output() {
        let mut request = subtitle_test_request(false, false);
        request.merge_output_container = Some("webm".to_owned());
        request.video_ext = "webm".to_owned();
        request.audio_ext = "webm".to_owned();
        request.file_name = "video.mkv".to_owned();
        let plan = build_directory_output_plan(&request, Path::new("output"));
        assert!(
            plan.final_output_path
                .ends_with(Path::new("output").join("video.webm"))
        );

        let mut command = Command::new("yt-dlp");
        apply_merge_output_container_arg(
            &mut command,
            DownloadTargetKind::Normal,
            request.merge_output_container.as_deref(),
        );
        assert_eq!(
            command_args(&command),
            vec!["--merge-output-format", "webm"]
        );
    }

    #[test]
    fn merge_container_is_not_applied_to_standalone_exports() {
        let mut command = Command::new("yt-dlp");
        apply_merge_output_container_arg(&mut command, DownloadTargetKind::Video, Some("webm"));
        assert!(command_args(&command).is_empty());

        apply_merge_output_container_arg(&mut command, DownloadTargetKind::Normal, Some("invalid"));
        assert!(command_args(&command).is_empty());
    }

    #[test]
    fn embedded_normal_subtitles_do_not_request_a_persistent_sidecar() {
        let request = subtitle_test_request(false, true);
        let mut command = Command::new("yt-dlp");

        apply_subtitle_download_args(&mut command, &request, true);

        let args = command_args(&command);
        assert!(args.iter().any(|arg| arg == "--embed-subs"));
        assert!(!args.iter().any(|arg| arg == "--write-subs"));
    }

    #[test]
    fn embedded_auto_subtitles_keep_auto_source_selection_and_embed() {
        let request = subtitle_test_request(true, true);
        let mut command = Command::new("yt-dlp");

        apply_subtitle_download_args(&mut command, &request, true);

        let args = command_args(&command);
        assert!(args.iter().any(|arg| arg == "--write-auto-subs"));
        assert!(args.iter().any(|arg| arg == "--embed-subs"));
        assert!(!args.iter().any(|arg| arg == "--write-subs"));
    }

    #[test]
    fn downloaded_subtitles_still_request_a_sidecar() {
        let request = subtitle_test_request(false, false);
        let mut command = Command::new("yt-dlp");

        apply_subtitle_download_args(&mut command, &request, true);

        let args = command_args(&command);
        assert!(args.iter().any(|arg| arg == "--write-subs"));
        assert!(!args.iter().any(|arg| arg == "--embed-subs"));
    }

    #[test]
    fn music_album_parse_metadata_args_use_current_yt_dlp_from_to_syntax() {
        assert_eq!(
            music_album_parse_metadata_args(),
            [
                "%(playlist|)s:%(meta_album)s",
                "%(playlist_title|)s:%(meta_album)s",
            ]
        );
    }

    #[test]
    fn v2_cache_mode_keeps_yt_dlp_cache_and_temp_namespaced() {
        let cache_root = std::env::temp_dir().join(format!(
            "yt-dlp-gui-v2-cache-args-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let tool_paths = ToolPaths {
            cache_mode: CacheLocationMode::V2Cache,
            cache_dir: cache_root.display().to_string(),
            ffmpeg: "missing-ffmpeg.exe".to_owned(),
            deno: "missing-deno.exe".to_owned(),
            ..ToolPaths::default()
        };
        let mut command = Command::new("yt-dlp");

        tool_paths.apply_common_yt_dlp_args(&mut command);

        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert!(args.windows(2).any(|pair| pair[0] == "--cache-dir"
            && pair[1] == cache_root.join("yt-dlp").display().to_string()));
        assert!(args.windows(2).any(|pair| pair[0] == "-P"
            && pair[1] == format!("temp:{}", cache_root.join("yt-dlp-temp").display())));
    }
}

pub fn resolve_tool_path(path: &str) -> PathBuf {
    resolve_support_path(path)
}

fn resolve_support_path(path: &str) -> PathBuf {
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

fn candidate_base_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![portable_root_dir()];
    if let Some(exe_dir) = current_executable_dir() {
        if !dirs.iter().any(|dir| dir == &exe_dir) {
            dirs.push(exe_dir);
        }
    }

    dirs
}

fn portable_root_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    }

    #[cfg(not(debug_assertions))]
    current_executable_dir()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn current_executable_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

fn format_command_line(program: &Path, command: &Command) -> String {
    let args = command
        .get_args()
        .map(|arg| quote_arg(&arg.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ");
    if args.is_empty() {
        quote_arg(&program.display().to_string())
    } else {
        format!("{} {}", quote_arg(&program.display().to_string()), args)
    }
}

fn quote_arg(value: &str) -> String {
    if value.contains([' ', '\t', '"']) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_owned()
    }
}

pub fn resolve_output_dir(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Download folder cannot be empty.".to_owned());
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(special) = resolve_windows_special_dir(trimmed) {
            return Ok(special);
        }
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.is_absolute() {
        Ok(candidate)
    } else {
        Ok(resolve_support_path(trimmed))
    }
}

pub fn display_output_dir(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Some(portable_display) = display_portable_relative_dir(trimmed) {
        return portable_display;
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(label) = windows_known_folder_display_name(trimmed) {
            return label;
        }

        if let Some(label) = match_windows_special_dir_path(trimmed) {
            return label;
        }
    }

    let resolved = resolve_support_path(trimmed);
    if resolved == portable_root_dir() {
        return "yt-dlp-gui".to_owned();
    }

    trimmed.to_owned()
}

fn display_portable_relative_dir(path: &str) -> Option<String> {
    let normalized = path.trim().replace('/', "\\");
    if normalized == "." || normalized == ".\\" {
        return Some("yt-dlp-gui".to_owned());
    }

    let suffix = normalized
        .strip_prefix(".\\")
        .or_else(|| normalized.strip_prefix('.'))?
        .trim_start_matches('\\');
    if suffix.is_empty() {
        Some("yt-dlp-gui".to_owned())
    } else {
        Some(format!("yt-dlp-gui\\{suffix}"))
    }
}

#[cfg(target_os = "windows")]
#[derive(Clone)]
struct WindowsKnownFolder {
    canonical_name: &'static str,
    display_name: String,
    path: PathBuf,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[cfg(target_os = "windows")]
const FOLDERID_DESKTOP: Guid = Guid {
    data1: 0xB4BFCC3A,
    data2: 0xDB2C,
    data3: 0x424C,
    data4: [0xB0, 0x29, 0x7F, 0xE9, 0x9A, 0x87, 0xC6, 0x41],
};
#[cfg(target_os = "windows")]
const FOLDERID_DOCUMENTS: Guid = Guid {
    data1: 0xFDD39AD0,
    data2: 0x238F,
    data3: 0x46AF,
    data4: [0xAD, 0xB4, 0x6C, 0x85, 0x48, 0x03, 0x69, 0xC7],
};
#[cfg(target_os = "windows")]
const FOLDERID_DOWNLOADS: Guid = Guid {
    data1: 0x374DE290,
    data2: 0x123F,
    data3: 0x4565,
    data4: [0x91, 0x64, 0x39, 0xC4, 0x92, 0x5E, 0x46, 0x7B],
};
#[cfg(target_os = "windows")]
const FOLDERID_MUSIC: Guid = Guid {
    data1: 0x4BD8D571,
    data2: 0x6D19,
    data3: 0x48D3,
    data4: [0xBE, 0x97, 0x42, 0x22, 0x20, 0x08, 0x04, 0xE3],
};
#[cfg(target_os = "windows")]
const FOLDERID_VIDEOS: Guid = Guid {
    data1: 0x18989B1D,
    data2: 0x99B5,
    data3: 0x455B,
    data4: [0x84, 0x1C, 0xAB, 0x7C, 0x74, 0xE4, 0xDD, 0xFC],
};

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
unsafe extern "system" {
    fn SHGetKnownFolderPath(
        rfid: *const Guid,
        dw_flags: u32,
        h_token: *mut c_void,
        ppsz_path: *mut *mut u16,
    ) -> i32;

    fn SHGetFileInfoW(
        psz_path: *const u16,
        dw_file_attributes: u32,
        psfi: *mut ShFileInfoW,
        cb_file_info: u32,
        u_flags: u32,
    ) -> usize;
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct ShFileInfoW {
    h_icon: *mut c_void,
    i_icon: i32,
    dw_attributes: u32,
    sz_display_name: [u16; 260],
    sz_type_name: [u16; 80],
}

#[cfg(target_os = "windows")]
impl Default for ShFileInfoW {
    fn default() -> Self {
        Self {
            h_icon: std::ptr::null_mut(),
            i_icon: 0,
            dw_attributes: 0,
            sz_display_name: [0; 260],
            sz_type_name: [0; 80],
        }
    }
}

#[cfg(target_os = "windows")]
#[link(name = "ole32")]
unsafe extern "system" {
    fn CoTaskMemFree(pv: *mut c_void);
}

#[cfg(target_os = "windows")]
fn resolve_windows_special_dir(path: &str) -> Option<PathBuf> {
    windows_known_folders()
        .iter()
        .find(|folder| windows_known_folder_name_matches(folder, path))
        .map(|folder| folder.path.clone())
}

#[cfg(target_os = "windows")]
pub fn is_windows_known_folder_segment(segment: &str) -> bool {
    let trimmed = segment.trim();
    if trimmed == "yt-dlp-gui" {
        return true;
    }
    windows_known_folders()
        .iter()
        .any(|folder| windows_known_folder_name_matches(folder, trimmed))
}

#[cfg(not(target_os = "windows"))]
pub fn is_windows_known_folder_segment(segment: &str) -> bool {
    segment.trim() == "yt-dlp-gui"
}

#[cfg(target_os = "windows")]
fn windows_known_folder_display_name(path: &str) -> Option<String> {
    windows_known_folders()
        .iter()
        .find(|folder| windows_known_folder_name_matches(folder, path))
        .map(|folder| folder.display_name.clone())
}

#[cfg(target_os = "windows")]
fn match_windows_special_dir_path(path: &str) -> Option<String> {
    let candidate = PathBuf::from(path);
    if !candidate.is_absolute() {
        return None;
    }

    if candidate == portable_root_dir() {
        return Some("yt-dlp-gui".to_owned());
    }

    windows_known_folders()
        .iter()
        .find(|folder| same_windows_path(&candidate, &folder.path))
        .map(|folder| folder.display_name.clone())
}

#[cfg(target_os = "windows")]
static WINDOWS_KNOWN_FOLDERS: OnceLock<Vec<WindowsKnownFolder>> = OnceLock::new();

#[cfg(target_os = "windows")]
fn windows_known_folders() -> &'static [WindowsKnownFolder] {
    WINDOWS_KNOWN_FOLDERS
        .get_or_init(load_windows_known_folders)
        .as_slice()
}

#[cfg(target_os = "windows")]
fn load_windows_known_folders() -> Vec<WindowsKnownFolder> {
    [
        ("Desktop", FOLDERID_DESKTOP),
        ("Documents", FOLDERID_DOCUMENTS),
        ("Downloads", FOLDERID_DOWNLOADS),
        ("Music", FOLDERID_MUSIC),
        ("Videos", FOLDERID_VIDEOS),
    ]
    .into_iter()
    .filter_map(|(canonical_name, guid)| {
        let path = sh_get_known_folder_path(guid)?;
        let display_name = shell_display_name_for_path(&path)
            .or_else(|| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .filter(|name| !name.trim().is_empty())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| canonical_name.to_owned());
        Some(WindowsKnownFolder {
            canonical_name,
            display_name,
            path,
        })
    })
    .collect()
}

#[cfg(target_os = "windows")]
fn windows_known_folder_name_matches(folder: &WindowsKnownFolder, input: &str) -> bool {
    input.eq_ignore_ascii_case(folder.display_name.as_str())
        || input.eq_ignore_ascii_case(folder.canonical_name)
}

#[cfg(target_os = "windows")]
fn same_windows_path(a: &Path, b: &Path) -> bool {
    a.to_string_lossy()
        .eq_ignore_ascii_case(&b.to_string_lossy())
}

#[cfg(target_os = "windows")]
fn shell_display_name_for_path(path: &Path) -> Option<String> {
    let wide_path = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut info = ShFileInfoW::default();
    let result = unsafe {
        SHGetFileInfoW(
            wide_path.as_ptr(),
            0,
            &mut info as *mut ShFileInfoW,
            std::mem::size_of::<ShFileInfoW>() as u32,
            SHGFI_DISPLAYNAME,
        )
    };
    if result == 0 {
        return None;
    }

    let len = info
        .sz_display_name
        .iter()
        .position(|ch| *ch == 0)
        .unwrap_or(info.sz_display_name.len());
    let display_name = OsString::from_wide(&info.sz_display_name[..len])
        .to_string_lossy()
        .trim()
        .to_owned();
    if display_name.is_empty() {
        None
    } else {
        Some(display_name)
    }
}

#[cfg(target_os = "windows")]
fn sh_get_known_folder_path(guid: Guid) -> Option<PathBuf> {
    unsafe {
        let mut raw_path: *mut u16 = std::ptr::null_mut();
        let hr = SHGetKnownFolderPath(
            &guid as *const Guid,
            0,
            std::ptr::null_mut(),
            &mut raw_path as *mut *mut u16,
        );
        if hr < 0 || raw_path.is_null() {
            return None;
        }

        let mut len = 0usize;
        while *raw_path.add(len) != 0 {
            len += 1;
        }
        let wide = std::slice::from_raw_parts(raw_path, len);
        let path = PathBuf::from(OsString::from_wide(wide));
        CoTaskMemFree(raw_path.cast::<c_void>());
        Some(path)
    }
}
