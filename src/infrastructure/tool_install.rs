use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Deserialize;

const USER_AGENT: &str = "yt-dlp-gui-v2";
const TOOL_INSTALL_CANCELLED: &str = "Dependency deployment cancelled.";
const TOOL_DOWNLOAD_RETRY_ATTEMPTS: usize = 3;
const TOOL_DOWNLOAD_RETRY_BACKOFF_BASE_MS: u64 = 600;
const PORTABLE_TOOL_SEARCH_MAX_DEPTH: usize = 4;
const PORTABLE_TOOL_SEARCH_SKIPPED_DIRS: &[&str] = &[
    ".git",
    "cache",
    "data",
    "download",
    "downloads",
    "node_modules",
    "target",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DependencyTool {
    YtDlp,
    Ffmpeg,
    Aria2c,
    Deno,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolInstallStage {
    Preparing,
    Downloading,
    Extracting,
    Installing,
    Completed,
    Failed,
}

impl ToolInstallStage {
    pub fn label(self) -> &'static str {
        match self {
            Self::Preparing => "tool_install.stage.preparing",
            Self::Downloading => "tool_install.stage.downloading",
            Self::Extracting => "tool_install.stage.extracting",
            Self::Installing => "tool_install.stage.installing",
            Self::Completed => "tool_install.stage.completed",
            Self::Failed => "tool_install.stage.failed",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToolInstallProgress {
    pub tool: DependencyTool,
    pub stage: ToolInstallStage,
    pub percent: Option<u8>,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct InstalledDependencyTool {
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ToolInstallCancelHandle {
    cancelled: Arc<AtomicBool>,
}

impl ToolInstallCancelHandle {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub(crate) fn token(&self) -> ToolInstallCancelToken {
        ToolInstallCancelToken {
            cancelled: Arc::clone(&self.cancelled),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToolInstallCancelToken {
    cancelled: Arc<AtomicBool>,
}

impl ToolInstallCancelToken {
    fn check(&self) -> Result<(), String> {
        if self.cancelled.load(Ordering::Relaxed) {
            Err(TOOL_INSTALL_CANCELLED.to_owned())
        } else {
            Ok(())
        }
    }
}

impl DependencyTool {
    pub fn label(self) -> &'static str {
        match self {
            Self::YtDlp => "yt-dlp",
            Self::Ffmpeg => "FFmpeg",
            Self::Aria2c => "Aria2",
            Self::Deno => "Deno",
        }
    }

    pub fn executable_name(self) -> &'static str {
        match self {
            Self::YtDlp => "yt-dlp.exe",
            Self::Ffmpeg => "ffmpeg.exe",
            Self::Aria2c => "aria2c.exe",
            Self::Deno => "deno.exe",
        }
    }

    pub fn default_portable_path(self) -> &'static str {
        match self {
            Self::YtDlp => ".\\tools\\yt-dlp\\yt-dlp.exe",
            Self::Ffmpeg => ".\\tools\\ffmpeg\\ffmpeg.exe",
            Self::Aria2c => ".\\tools\\aria2c\\aria2c.exe",
            Self::Deno => ".\\tools\\deno\\deno.exe",
        }
    }

    pub fn install_dir_name(self) -> &'static str {
        match self {
            Self::YtDlp => "yt-dlp",
            Self::Ffmpeg => "ffmpeg",
            Self::Aria2c => "aria2c",
            Self::Deno => "deno",
        }
    }
}

#[derive(Deserialize)]
struct GithubRelease {
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

struct HttpResponse {
    reader: Box<dyn Read>,
    content_length: Option<u64>,
}

pub fn dependency_tool_exists(path: &str) -> bool {
    resolve_support_path(path).is_file()
}

pub fn dependency_tool_is_available(tool: DependencyTool, path: &str) -> bool {
    match tool {
        DependencyTool::Ffmpeg => {
            dependency_tool_exists(path) && ffprobe_companion_path(path).is_file()
        }
        _ => dependency_tool_exists(path),
    }
}

pub fn ffprobe_companion_path(ffmpeg_path: &str) -> PathBuf {
    let resolved_ffmpeg = if ffmpeg_path.trim().is_empty() {
        resolve_support_path(DependencyTool::Ffmpeg.default_portable_path())
    } else {
        resolve_support_path(ffmpeg_path)
    };

    resolved_ffmpeg
        .parent()
        .map(|parent| parent.join("ffprobe.exe"))
        .unwrap_or_else(|| resolve_support_path(".\\tools\\ffmpeg\\ffprobe.exe"))
}

pub fn detect_dependency_tool(tool: DependencyTool) -> Option<PathBuf> {
    find_dependency_tool_in_base_dirs(tool, &portable_tool_search_base_dirs())
        .or_else(|| find_dependency_tool_in_system_path(tool))
}

fn portable_tool_search_base_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(executable_dir) = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
    {
        dirs.push(executable_dir);
    }
    let portable_root = portable_root_dir();
    if !dirs.iter().any(|dir| dir == &portable_root) {
        dirs.push(portable_root);
    }
    dirs
}

fn find_dependency_tool_in_base_dirs(
    tool: DependencyTool,
    base_dirs: &[PathBuf],
) -> Option<PathBuf> {
    for base_dir in base_dirs {
        if let Some(path) = find_dependency_tool_under_base(tool, base_dir) {
            return Some(path);
        }
    }
    None
}

fn find_dependency_tool_under_base(tool: DependencyTool, base_dir: &Path) -> Option<PathBuf> {
    if !base_dir.is_dir() {
        return None;
    }

    if let Some(path) = find_dependency_tool_directly_in(tool, base_dir) {
        return Some(path);
    }

    let mut queue = VecDeque::new();
    queue.push_back((base_dir.to_path_buf(), 0_usize));
    while let Some((dir, depth)) = queue.pop_front() {
        if depth >= PORTABLE_TOOL_SEARCH_MAX_DEPTH {
            continue;
        }

        let mut child_dirs = fs::read_dir(&dir)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.flatten())
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                if !file_type.is_dir() || file_type.is_symlink() {
                    return None;
                }
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if PORTABLE_TOOL_SEARCH_SKIPPED_DIRS.contains(&name.as_str()) {
                    return None;
                }
                Some(entry.path())
            })
            .collect::<Vec<_>>();
        child_dirs.sort();

        for child_dir in child_dirs {
            if let Some(path) = find_dependency_tool_directly_in(tool, &child_dir) {
                return Some(path);
            }
            queue.push_back((child_dir, depth + 1));
        }
    }
    None
}

fn find_dependency_tool_directly_in(tool: DependencyTool, dir: &Path) -> Option<PathBuf> {
    for name in system_path_executable_candidates(tool) {
        let candidate = dir.join(name);
        if dependency_tool_candidate_is_available(tool, &candidate) {
            return Some(candidate);
        }
    }
    None
}

fn dependency_tool_candidate_is_available(tool: DependencyTool, candidate: &Path) -> bool {
    candidate.is_file()
        && (tool != DependencyTool::Ffmpeg || ffprobe_for_ffmpeg_path(candidate).is_file())
}

fn find_dependency_tool_in_system_path(tool: DependencyTool) -> Option<PathBuf> {
    let path_value = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_value) {
        for name in system_path_executable_candidates(tool) {
            let candidate = dir.join(name);
            if dependency_tool_candidate_is_available(tool, &candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(test)]
mod portable_detection_tests {
    use super::*;

    fn unique_test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "yt-dlp-gui-v2-{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    #[test]
    fn portable_detection_finds_tool_beside_gui_or_in_nested_folder() {
        let root = unique_test_root("portable-tool-detection");
        let nested = root.join("vendor").join("yt-dlp");
        fs::create_dir_all(&nested).expect("create nested tool folder");
        let yt_dlp = nested.join("yt-dlp.exe");
        fs::write(&yt_dlp, b"test").expect("create yt-dlp candidate");

        let found = find_dependency_tool_in_base_dirs(DependencyTool::YtDlp, &[root.clone()]);

        assert_eq!(found, Some(yt_dlp));
        fs::remove_dir_all(root).expect("remove test folder");
    }

    #[test]
    fn portable_ffmpeg_detection_requires_ffprobe_companion() {
        let root = unique_test_root("portable-ffmpeg-detection");
        fs::create_dir_all(&root).expect("create portable root");
        let ffmpeg = root.join("ffmpeg.exe");
        fs::write(&ffmpeg, b"test").expect("create ffmpeg candidate");

        assert_eq!(
            find_dependency_tool_in_base_dirs(DependencyTool::Ffmpeg, &[root.clone()]),
            None
        );

        fs::write(root.join("ffprobe.exe"), b"test").expect("create ffprobe companion");
        assert_eq!(
            find_dependency_tool_in_base_dirs(DependencyTool::Ffmpeg, &[root.clone()]),
            Some(ffmpeg)
        );
        fs::remove_dir_all(root).expect("remove test folder");
    }
}

fn system_path_executable_candidates(tool: DependencyTool) -> &'static [&'static str] {
    match tool {
        DependencyTool::YtDlp => &[
            "yt-dlp.exe",
            "yt-dlp",
            "yt-dlp.cmd",
            "yt-dlp.bat",
            "ytdl-patched.exe",
            "ytdl-patched",
            "ytdl-patched.cmd",
            "ytdl-patched.bat",
        ],
        DependencyTool::Ffmpeg => &["ffmpeg.exe", "ffmpeg"],
        DependencyTool::Aria2c => &["aria2c.exe", "aria2c"],
        DependencyTool::Deno => &["deno.exe", "deno", "deno.cmd", "deno.bat"],
    }
}

fn ffprobe_for_ffmpeg_path(ffmpeg_path: &Path) -> PathBuf {
    let file_name = if cfg!(target_os = "windows") {
        "ffprobe.exe"
    } else {
        "ffprobe"
    };
    ffmpeg_path
        .parent()
        .map(|parent| parent.join(file_name))
        .unwrap_or_else(|| PathBuf::from(file_name))
}

pub fn install_dependency_tool(tool: DependencyTool) -> Result<InstalledDependencyTool, String> {
    install_dependency_tool_with_progress(tool, |_| {})
}

pub fn install_dependency_tool_with_progress(
    tool: DependencyTool,
    progress: impl FnMut(ToolInstallProgress),
) -> Result<InstalledDependencyTool, String> {
    install_dependency_tool_with_progress_using_proxy(tool, None, None, progress)
}

pub fn install_dependency_tool_with_progress_using_proxy(
    tool: DependencyTool,
    proxy_url: Option<String>,
    cancel_token: Option<ToolInstallCancelToken>,
    mut progress: impl FnMut(ToolInstallProgress),
) -> Result<InstalledDependencyTool, String> {
    if !cfg!(target_os = "windows") {
        return Err("Dependency deployment currently only supports Windows.".to_owned());
    }

    let cancel_token = cancel_token.unwrap_or_else(|| ToolInstallCancelHandle::new().token());
    progress(ToolInstallProgress {
        tool,
        stage: ToolInstallStage::Preparing,
        percent: None,
        message: "tool_install.stage.preparing".to_owned(),
    });

    let install_dir = portable_root_dir()
        .join("tools")
        .join(tool.install_dir_name());
    let temp_root = tool_install_cache_dir().join(unique_temp_dir_name(tool));
    fs::create_dir_all(&install_dir).map_err(|error| {
        format!(
            "Could not create tools folder {}: {error}",
            install_dir.display()
        )
    })?;
    fs::create_dir_all(&temp_root).map_err(|error| {
        format!(
            "Could not create deployment temp folder {}: {error}",
            temp_root.display()
        )
    })?;

    let result = install_dependency_tool_native(
        tool,
        &install_dir,
        &temp_root,
        proxy_url.as_deref(),
        &cancel_token,
        &mut progress,
    );
    let _ = fs::remove_dir_all(&temp_root);
    let installed = result?;

    if !installed.path.is_file() {
        return Err(format!(
            "{} installation finished, but {} was not found.",
            tool.label(),
            installed.path.display()
        ));
    }

    progress(ToolInstallProgress {
        tool,
        stage: ToolInstallStage::Completed,
        percent: Some(100),
        message: "tool_install.stage.completed".to_owned(),
    });

    Ok(installed)
}

fn install_dependency_tool_native(
    tool: DependencyTool,
    install_dir: &Path,
    temp_root: &Path,
    proxy_url: Option<&str>,
    cancel_token: &ToolInstallCancelToken,
    progress: &mut impl FnMut(ToolInstallProgress),
) -> Result<InstalledDependencyTool, String> {
    match tool {
        DependencyTool::YtDlp => {
            install_yt_dlp(install_dir, temp_root, proxy_url, cancel_token, progress)
        }
        DependencyTool::Ffmpeg => {
            install_ffmpeg(install_dir, temp_root, proxy_url, cancel_token, progress)
        }
        DependencyTool::Aria2c => {
            install_aria2c(install_dir, temp_root, proxy_url, cancel_token, progress)
        }
        DependencyTool::Deno => {
            install_deno(install_dir, temp_root, proxy_url, cancel_token, progress)
        }
    }
}

fn install_yt_dlp(
    install_dir: &Path,
    temp_root: &Path,
    proxy_url: Option<&str>,
    cancel_token: &ToolInstallCancelToken,
    progress: &mut impl FnMut(ToolInstallProgress),
) -> Result<InstalledDependencyTool, String> {
    let temp_file = temp_root.join("yt-dlp.exe");
    download_file(
        DependencyTool::YtDlp,
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe",
        &temp_file,
        proxy_url,
        cancel_token,
        progress,
    )?;
    copy_installed_file(
        DependencyTool::YtDlp,
        &temp_file,
        &install_dir.join("yt-dlp.exe"),
        cancel_token,
        progress,
    )
}

fn install_ffmpeg(
    install_dir: &Path,
    temp_root: &Path,
    proxy_url: Option<&str>,
    cancel_token: &ToolInstallCancelToken,
    progress: &mut impl FnMut(ToolInstallProgress),
) -> Result<InstalledDependencyTool, String> {
    let zip_path = temp_root.join("ffmpeg.zip");
    download_file(
        DependencyTool::Ffmpeg,
        "https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip",
        &zip_path,
        proxy_url,
        cancel_token,
        progress,
    )?;
    let extract_dir = temp_root.join("extracted");
    extract_zip(
        DependencyTool::Ffmpeg,
        &zip_path,
        &extract_dir,
        cancel_token,
        progress,
    )?;
    let ffmpeg = find_file_recursive(&extract_dir, "ffmpeg.exe")
        .ok_or_else(|| "ffmpeg.exe not found in downloaded archive".to_owned())?;
    let installed = copy_installed_file(
        DependencyTool::Ffmpeg,
        &ffmpeg,
        &install_dir.join("ffmpeg.exe"),
        cancel_token,
        progress,
    )?;
    let ffprobe = find_file_recursive(&extract_dir, "ffprobe.exe")
        .ok_or_else(|| "ffprobe.exe not found in downloaded archive".to_owned())?;
    let installed_ffprobe = install_dir.join("ffprobe.exe");
    fs::copy(&ffprobe, &installed_ffprobe)
        .map_err(|error| format!("Could not install ffprobe.exe: {error}"))?;
    if !installed_ffprobe.is_file() {
        return Err(format!(
            "FFmpeg installation finished, but {} was not found.",
            installed_ffprobe.display()
        ));
    }
    Ok(installed)
}

fn install_aria2c(
    install_dir: &Path,
    temp_root: &Path,
    proxy_url: Option<&str>,
    cancel_token: &ToolInstallCancelToken,
    progress: &mut impl FnMut(ToolInstallProgress),
) -> Result<InstalledDependencyTool, String> {
    progress(ToolInstallProgress {
        tool: DependencyTool::Aria2c,
        stage: ToolInstallStage::Preparing,
        percent: None,
        message: "checking release".to_owned(),
    });
    let release: GithubRelease = request_json(
        "https://api.github.com/repos/aria2/aria2/releases/latest",
        proxy_url,
        cancel_token,
    )?;
    let asset = release
        .assets
        .into_iter()
        .find(|asset| {
            let name = asset.name.to_ascii_lowercase();
            name.contains("win-64bit") && name.ends_with(".zip")
        })
        .ok_or_else(|| "aria2 win-64bit zip asset not found in latest release".to_owned())?;

    let zip_path = temp_root.join(asset.name);
    download_file(
        DependencyTool::Aria2c,
        &asset.browser_download_url,
        &zip_path,
        proxy_url,
        cancel_token,
        progress,
    )?;
    let extract_dir = temp_root.join("extracted");
    extract_zip(
        DependencyTool::Aria2c,
        &zip_path,
        &extract_dir,
        cancel_token,
        progress,
    )?;
    let aria2c = find_file_recursive(&extract_dir, "aria2c.exe")
        .ok_or_else(|| "aria2c.exe not found in downloaded archive".to_owned())?;
    copy_installed_file(
        DependencyTool::Aria2c,
        &aria2c,
        &install_dir.join("aria2c.exe"),
        cancel_token,
        progress,
    )
}

fn install_deno(
    install_dir: &Path,
    temp_root: &Path,
    proxy_url: Option<&str>,
    cancel_token: &ToolInstallCancelToken,
    progress: &mut impl FnMut(ToolInstallProgress),
) -> Result<InstalledDependencyTool, String> {
    progress(ToolInstallProgress {
        tool: DependencyTool::Deno,
        stage: ToolInstallStage::Preparing,
        percent: None,
        message: "checking release".to_owned(),
    });
    let latest = request_text(
        "https://dl.deno.land/release-latest.txt",
        proxy_url,
        cancel_token,
    )?;
    let latest = latest.trim();
    if latest.is_empty() {
        return Err("Deno latest version is empty".to_owned());
    }

    let zip_url = format!("https://dl.deno.land/release/{latest}/deno-x86_64-pc-windows-msvc.zip");
    let zip_path = temp_root.join("deno.zip");
    download_file(
        DependencyTool::Deno,
        &zip_url,
        &zip_path,
        proxy_url,
        cancel_token,
        progress,
    )?;
    let extract_dir = temp_root.join("extracted");
    extract_zip(
        DependencyTool::Deno,
        &zip_path,
        &extract_dir,
        cancel_token,
        progress,
    )?;
    let deno = find_file_recursive(&extract_dir, "deno.exe")
        .ok_or_else(|| "deno.exe not found in downloaded archive".to_owned())?;
    copy_installed_file(
        DependencyTool::Deno,
        &deno,
        &install_dir.join("deno.exe"),
        cancel_token,
        progress,
    )
}

fn request_json<T: for<'de> Deserialize<'de>>(
    url: &str,
    proxy_url: Option<&str>,
    cancel_token: &ToolInstallCancelToken,
) -> Result<T, String> {
    let text = request_text(url, proxy_url, cancel_token)?;
    serde_json::from_str(&text)
        .map_err(|error| format!("Could not parse response from {url}: {error}"))
}

fn request_text(
    url: &str,
    proxy_url: Option<&str>,
    cancel_token: &ToolInstallCancelToken,
) -> Result<String, String> {
    cancel_token.check()?;
    let mut response = http_get(url, proxy_url)?;
    let mut text = String::new();
    response
        .reader
        .read_to_string(&mut text)
        .map_err(|error| format!("Could not read response from {url}: {error}"))?;
    cancel_token.check()?;
    Ok(text)
}

fn download_file(
    tool: DependencyTool,
    url: &str,
    destination: &Path,
    proxy_url: Option<&str>,
    cancel_token: &ToolInstallCancelToken,
    progress: &mut impl FnMut(ToolInstallProgress),
) -> Result<(), String> {
    let mut last_error = None;
    for attempt_index in 0..TOOL_DOWNLOAD_RETRY_ATTEMPTS {
        match download_file_once(tool, url, destination, proxy_url, cancel_token, progress) {
            Ok(()) => return Ok(()),
            Err(error) => {
                if error == TOOL_INSTALL_CANCELLED {
                    return Err(error);
                }
                last_error = Some(error);
                if attempt_index + 1 < TOOL_DOWNLOAD_RETRY_ATTEMPTS {
                    std::thread::sleep(Duration::from_millis(
                        TOOL_DOWNLOAD_RETRY_BACKOFF_BASE_MS * (attempt_index as u64 + 1),
                    ));
                }
            }
        }
    }

    Err(format!(
        "Could not download {url} after {} attempts: {}",
        TOOL_DOWNLOAD_RETRY_ATTEMPTS,
        last_error.unwrap_or_else(|| "unknown download error".to_owned())
    ))
}

fn download_file_once(
    tool: DependencyTool,
    url: &str,
    destination: &Path,
    proxy_url: Option<&str>,
    cancel_token: &ToolInstallCancelToken,
    progress: &mut impl FnMut(ToolInstallProgress),
) -> Result<(), String> {
    cancel_token.check()?;
    progress(ToolInstallProgress {
        tool,
        stage: ToolInstallStage::Downloading,
        percent: Some(0),
        message: "downloading".to_owned(),
    });

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Could not create download folder {}: {error}",
                parent.display()
            )
        })?;
    }

    let mut response = http_get(url, proxy_url)?;
    let total = response.content_length;
    let mut file = File::create(destination)
        .map_err(|error| format!("Could not create {}: {error}", destination.display()))?;
    let mut buffer = [0u8; 1024 * 1024];
    let mut downloaded = 0u64;
    let mut last_percent = Some(0u8);

    loop {
        cancel_token.check()?;
        let read = response
            .reader
            .read(&mut buffer)
            .map_err(|error| format!("Could not download {url}: {error}"))?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|error| format!("Could not write {}: {error}", destination.display()))?;
        downloaded += read as u64;
        if let Some(total) = total.filter(|value| *value > 0) {
            let percent = ((downloaded.saturating_mul(100)) / total).min(100) as u8;
            if Some(percent) != last_percent {
                progress(ToolInstallProgress {
                    tool,
                    stage: ToolInstallStage::Downloading,
                    percent: Some(percent),
                    message: "downloading".to_owned(),
                });
                last_percent = Some(percent);
            }
        } else if last_percent.is_some() {
            progress(ToolInstallProgress {
                tool,
                stage: ToolInstallStage::Downloading,
                percent: None,
                message: "downloading".to_owned(),
            });
            last_percent = None;
        }
    }
    file.flush()
        .map_err(|error| format!("Could not finish {}: {error}", destination.display()))?;
    cancel_token.check()?;
    progress(ToolInstallProgress {
        tool,
        stage: ToolInstallStage::Downloading,
        percent: Some(100),
        message: "downloaded".to_owned(),
    });
    Ok(())
}

fn http_get(url: &str, proxy_url: Option<&str>) -> Result<HttpResponse, String> {
    let mut builder = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(10)))
        .timeout_recv_response(Some(Duration::from_secs(10)))
        .timeout_recv_body(Some(Duration::from_secs(10)))
        .user_agent(USER_AGENT);
    if let Some(proxy_url) = proxy_url.map(str::trim).filter(|value| !value.is_empty()) {
        let proxy = ureq::Proxy::new(proxy_url)
            .map_err(|error| format!("Invalid proxy URL {proxy_url}: {error}"))?;
        builder = builder.proxy(Some(proxy));
    }

    let response = builder
        .build()
        .new_agent()
        .get(url)
        .call()
        .map_err(|error| format!("Could not download {url}: {error}"))?;
    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    Ok(HttpResponse {
        reader: Box::new(response.into_parts().1.into_reader()),
        content_length,
    })
}

fn extract_zip(
    tool: DependencyTool,
    zip_path: &Path,
    destination: &Path,
    cancel_token: &ToolInstallCancelToken,
    progress: &mut impl FnMut(ToolInstallProgress),
) -> Result<(), String> {
    cancel_token.check()?;
    progress(ToolInstallProgress {
        tool,
        stage: ToolInstallStage::Extracting,
        percent: Some(0),
        message: "extracting".to_owned(),
    });
    fs::create_dir_all(destination).map_err(|error| {
        format!(
            "Could not create extract folder {}: {error}",
            destination.display()
        )
    })?;

    let file = File::open(zip_path)
        .map_err(|error| format!("Could not open {}: {error}", zip_path.display()))?;
    let reader = BufReader::new(file);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|error| format!("Could not read zip: {error}"))?;
    let total = archive.len().max(1);
    let mut last_percent = 0u8;

    for index in 0..archive.len() {
        cancel_token.check()?;
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("Could not read zip entry: {error}"))?;
        let Some(enclosed_name) = entry.enclosed_name() else {
            continue;
        };
        let target = destination.join(enclosed_name);
        if entry.is_dir() {
            fs::create_dir_all(&target)
                .map_err(|error| format!("Could not create {}: {error}", target.display()))?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
            }
            let mut output = File::create(&target)
                .map_err(|error| format!("Could not create {}: {error}", target.display()))?;
            let mut buffer = [0u8; 1024 * 1024];
            loop {
                cancel_token.check()?;
                let read = entry
                    .read(&mut buffer)
                    .map_err(|error| format!("Could not read zip entry: {error}"))?;
                if read == 0 {
                    break;
                }
                output
                    .write_all(&buffer[..read])
                    .map_err(|error| format!("Could not extract {}: {error}", target.display()))?;
            }
        }

        let percent = (((index + 1) * 100) / total).min(100) as u8;
        if percent != last_percent {
            progress(ToolInstallProgress {
                tool,
                stage: ToolInstallStage::Extracting,
                percent: Some(percent),
                message: "extracting".to_owned(),
            });
            last_percent = percent;
        }
    }

    cancel_token.check()?;
    progress(ToolInstallProgress {
        tool,
        stage: ToolInstallStage::Extracting,
        percent: Some(100),
        message: "extracted".to_owned(),
    });
    Ok(())
}

fn copy_installed_file(
    tool: DependencyTool,
    source: &Path,
    destination: &Path,
    cancel_token: &ToolInstallCancelToken,
    progress: &mut impl FnMut(ToolInstallProgress),
) -> Result<InstalledDependencyTool, String> {
    cancel_token.check()?;
    progress(ToolInstallProgress {
        tool,
        stage: ToolInstallStage::Installing,
        percent: Some(0),
        message: "installing".to_owned(),
    });
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Could not create install folder {}: {error}",
                parent.display()
            )
        })?;
    }
    fs::copy(source, destination).map_err(|error| {
        format!(
            "Could not install {} to {}: {error}",
            source.display(),
            destination.display()
        )
    })?;
    cancel_token.check()?;
    progress(ToolInstallProgress {
        tool,
        stage: ToolInstallStage::Installing,
        percent: Some(100),
        message: "installed".to_owned(),
    });
    Ok(InstalledDependencyTool {
        path: destination.to_path_buf(),
    })
}

fn find_file_recursive(root: &Path, file_name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(file_name))
        {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_file_recursive(&path, file_name) {
                return Some(found);
            }
        }
    }
    None
}

fn unique_temp_dir_name(tool: DependencyTool) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    format!(
        "{}-{}-{timestamp}",
        tool.install_dir_name(),
        std::process::id()
    )
}

pub(crate) fn tool_install_cache_dir() -> PathBuf {
    portable_temp_cache_dir().join("tool-install")
}

pub(crate) fn portable_temp_cache_dir() -> PathBuf {
    portable_root_dir().join("cache").join("temp")
}

pub(crate) fn portable_root_dir() -> PathBuf {
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

pub(crate) fn resolve_support_path(path: &str) -> PathBuf {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return portable_root_dir();
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.is_absolute() {
        return candidate;
    }

    portable_root_dir().join(candidate)
}
