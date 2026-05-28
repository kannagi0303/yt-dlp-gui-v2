use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::domain::{QueueItemId, WorkflowKind, WorkflowRunId};
use crate::infrastructure::{
    DownloadRequest, DownloadTargetKind, FINAL_OUTPUT_PATH_PREFIX, FileTimeMode, PreparedDownload,
    ToolPaths, configure_background_command, humanize_yt_dlp_error,
};

pub(super) const DOWNLOAD_CANCELLED_MESSAGE: &str = "Download cancelled.";

pub(super) struct DownloadResult {
    pub item_id: QueueItemId,
    pub workflow_id: WorkflowRunId,
    pub workflow_kind: WorkflowKind,
    pub target_kind: DownloadTargetKind,
    pub result: Result<String, String>,
}

pub(super) enum DownloadEvent {
    Metadata {
        item_id: QueueItemId,
        json: Value,
    },
    Progress {
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        slot: DownloadProgressSlot,
        percent: f32,
        detail: Option<DownloadProgressDetail>,
    },
    Finished(DownloadResult),
}

#[derive(Clone, Copy)]
pub(super) enum DownloadProgressSlot {
    Video,
    Audio,
    Subtitle,
    Both,
}

#[derive(Clone, Debug, Default)]
pub(super) struct DownloadProgressDetail {
    pub downloaded: Option<String>,
    pub total: Option<String>,
    pub speed: Option<String>,
    pub elapsed: Option<String>,
    pub frame: Option<String>,
    pub fps: Option<String>,
    pub time: Option<String>,
}

pub(super) fn run_download_worker(
    tool_paths: ToolPaths,
    request: DownloadRequest,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    workflow_kind: WorkflowKind,
    tx: mpsc::Sender<DownloadEvent>,
    child_handle: Arc<Mutex<Option<Child>>>,
    cancel_requested: Arc<AtomicBool>,
) {
    let first_attempt = run_download_attempt(
        &tool_paths,
        &request,
        item_id,
        workflow_id,
        tx.clone(),
        &child_handle,
        &cancel_requested,
    );

    let result = match first_attempt {
        Ok(output_path) => Ok(output_path),
        Err(error) if should_log_subtitle_429_diagnostics(&request, &error) => {
            log_subtitle_429_diagnostics(&request, &error);
            Err(error)
        }
        Err(error) if request_has_thumbnail(&request) && should_retry_without_thumbnail(&error) => {
            eprintln!("[download] retry without thumbnail: {error}");
            let mut retry_request = request.clone();
            retry_request.embed_thumbnail = false;
            retry_request.write_thumbnail = false;
            if cancel_requested.load(Ordering::Relaxed) {
                Err(DOWNLOAD_CANCELLED_MESSAGE.to_owned())
            } else {
                run_download_attempt(
                    &tool_paths,
                    &retry_request,
                    item_id,
                    workflow_id,
                    tx.clone(),
                    &child_handle,
                    &cancel_requested,
                )
            }
        }
        Err(error) => Err(error),
    };

    let _ = tx.send(DownloadEvent::Finished(DownloadResult {
        item_id,
        workflow_id,
        workflow_kind,
        target_kind: request.target_kind,
        result,
    }));
}

fn should_log_subtitle_429_diagnostics(request: &DownloadRequest, error: &str) -> bool {
    if request.target_kind != DownloadTargetKind::Subtitle {
        return false;
    }

    let normalized = error.to_ascii_lowercase();
    normalized.contains("http 429")
        || normalized.contains("http error 429")
        || normalized.contains("too many requests")
}

fn log_subtitle_429_diagnostics(request: &DownloadRequest, error: &str) {
    eprintln!(
        "[subtitle] yt-dlp subtitle path failed with YouTube HTTP 429; GUI direct downloader fallback is disabled"
    );
    eprintln!(
        "[subtitle] diagnostic target=subtitle lang={} output_ext={} source_ext={} auto_subs={} auto_translated={} cookies={} has_direct_url={}",
        request
            .subtitle_lang
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("-"),
        normalized_extension(&request.subtitle_ext).unwrap_or_else(|| "-".to_owned()),
        normalized_extension(&request.subtitle_source_ext).unwrap_or_else(|| "-".to_owned()),
        request.write_auto_subs,
        request.subtitle_is_auto_translated,
        request.use_cookies,
        request
            .subtitle_url
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
    );
    if let Some(url) = request
        .subtitle_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        eprintln!(
            "[subtitle] diagnostic timedtext {}",
            timedtext_url_diagnostic(url)
        );
    }
    if !request.use_cookies {
        eprintln!(
            "[subtitle] diagnostic cookies are disabled for this item; if a cookies.txt file is configured, reselect it or enable Cookie before exporting subtitles"
        );
    }
    eprintln!(
        "[subtitle] diagnostic yt-dlp_error={}",
        first_error_line(error)
    );
}

fn timedtext_url_diagnostic(url: &str) -> String {
    let host = url_host(url).unwrap_or_else(|| "-".to_owned());
    let lang = query_value(url, "lang").unwrap_or_else(|| "-".to_owned());
    let tlang = query_value(url, "tlang").unwrap_or_else(|| "-".to_owned());
    let fmt = query_value(url, "fmt").unwrap_or_else(|| "-".to_owned());
    let kind = query_value(url, "kind").unwrap_or_else(|| "-".to_owned());
    let expire = query_value(url, "expire").unwrap_or_else(|| "-".to_owned());
    let signed = query_value(url, "signature").is_some() || query_value(url, "sig").is_some();
    format!(
        "host={host} lang={lang} tlang={tlang} fmt={fmt} kind={kind} expire={expire} signed={signed}"
    )
}

fn url_host(url: &str) -> Option<String> {
    let rest = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    Some(host.to_owned())
}

fn query_value(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1.split('#').next().unwrap_or_default();
    for part in query.split('&') {
        let (name, value) = part.split_once('=').unwrap_or((part, ""));
        if name == key {
            return Some(value.to_owned());
        }
    }
    None
}

fn first_error_line(error: &str) -> String {
    error
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(error)
        .to_owned()
}

fn run_download_attempt(
    tool_paths: &ToolPaths,
    request: &DownloadRequest,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: mpsc::Sender<DownloadEvent>,
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<String, String> {
    let prepared = tool_paths.prepare_download(request)?;

    println!("[yt-dlp] output: {}", prepared.output_path.display());
    println!("[yt-dlp] command: {}", prepared.command_line);

    let PreparedDownload {
        mut command,
        output_path,
        ..
    } = prepared;

    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start yt-dlp: {error}"))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    if let Ok(mut guard) = child_handle.lock() {
        *guard = Some(child);
    }
    let progress_context = DownloadProgressContext::new(request);

    let stdout_handle = stdout.map(|stdout| {
        let tx = tx.clone();
        let context = progress_context.clone();
        thread::spawn(move || {
            read_download_stream(stdout, false, item_id, workflow_id, tx, context)
        })
    });
    let stderr_handle = stderr.map(|stderr| {
        let tx = tx.clone();
        let context = progress_context.clone();
        thread::spawn(move || read_download_stream(stderr, true, item_id, workflow_id, tx, context))
    });

    let status = wait_download_child(child_handle, cancel_requested);

    let stdout_lines = stdout_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    let stderr_lines = stderr_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();

    if cancel_requested.load(Ordering::Relaxed) {
        return Err(DOWNLOAD_CANCELLED_MESSAGE.to_owned());
    }

    match status {
        Some(Ok(status)) if status.success() => {
            finalize_download_output(tool_paths, request, &output_path, &stdout_lines)
        }
        Some(Ok(status)) => {
            let detail = stderr_lines
                .iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| format!("exit code {:?}", status.code()));
            let detail = humanize_download_error(request, &detail);
            Err(format!("yt-dlp download failed: {detail}"))
        }
        Some(Err(error)) => Err(format!("Could not wait for yt-dlp to finish: {error}")),
        None => Err("Could not wait for yt-dlp to finish: child process missing".to_owned()),
    }
}

pub(super) fn request_download_stop(
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) {
    cancel_requested.store(true, Ordering::Relaxed);
    if let Ok(mut guard) = child_handle.lock() {
        if let Some(child) = guard.as_mut() {
            terminate_child_process_tree(child);
        }
    }
}

fn wait_download_child(
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Option<std::io::Result<std::process::ExitStatus>> {
    let mut stop_attempts = 0usize;

    loop {
        if cancel_requested.load(Ordering::Relaxed) {
            if let Ok(mut guard) = child_handle.lock() {
                if let Some(child) = guard.as_mut() {
                    terminate_child_process_tree(child);
                    stop_attempts += 1;
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

        if stop_attempts > 40 {
            if let Ok(mut guard) = child_handle.lock() {
                if let Some(child) = guard.as_mut() {
                    let _ = child.kill();
                }
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(target_os = "windows")]
fn terminate_child_process_tree(child: &mut Child) {
    let mut command = std::process::Command::new("taskkill");
    configure_background_command(&mut command);
    let _ = command
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
}

#[cfg(not(target_os = "windows"))]
fn terminate_child_process_tree(child: &mut Child) {
    let _ = child.kill();
}

fn finalize_download_output(
    tool_paths: &ToolPaths,
    request: &DownloadRequest,
    output_path: &Path,
    stdout_lines: &[String],
) -> Result<String, String> {
    if request.target_kind != DownloadTargetKind::Subtitle {
        let actual_output_path = resolve_completed_output_path(output_path, stdout_lines);
        apply_file_time_mode(tool_paths, request, &actual_output_path, stdout_lines);
        return Ok(actual_output_path.display().to_string());
    }

    finalize_subtitle_export(request, output_path)
}

fn resolve_completed_output_path(expected_path: &Path, stdout_lines: &[String]) -> PathBuf {
    if let Some(reported_path) = reported_final_output_path(stdout_lines) {
        if reported_path.is_file() {
            return reported_path;
        }
        eprintln!(
            "[download] yt-dlp reported final path, but it does not exist: {}",
            reported_path.display()
        );
    }

    if expected_path.is_file() {
        return expected_path.to_path_buf();
    }

    find_same_stem_output_candidate(expected_path).unwrap_or_else(|| expected_path.to_path_buf())
}

fn reported_final_output_path(lines: &[String]) -> Option<PathBuf> {
    lines.iter().rev().find_map(|line| {
        let payload = line.trim().strip_prefix(FINAL_OUTPUT_PATH_PREFIX)?.trim();
        let parsed = serde_json::from_str::<String>(payload)
            .unwrap_or_else(|_| payload.trim_matches('"').to_owned());
        let trimmed = parsed.trim();
        (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
    })
}

fn apply_file_time_mode(
    tool_paths: &ToolPaths,
    request: &DownloadRequest,
    output_path: &Path,
    stdout_lines: &[String],
) {
    if tool_paths.file_time_mode != FileTimeMode::UseUploadDate {
        return;
    }
    if tool_paths.effective_config_owns_mtime() {
        eprintln!("[download] skip upload-date file time because yt-dlp config owns mtime");
        return;
    }
    if !output_path.is_file() {
        return;
    }

    let Some(upload_date) = request_upload_date(request, stdout_lines) else {
        eprintln!("[download] skip upload-date file time because upload_date is missing");
        return;
    };
    let Some(modified_time) = upload_date_to_system_time(&upload_date) else {
        eprintln!(
            "[download] skip upload-date file time because upload_date is invalid: {upload_date}"
        );
        return;
    };

    if let Err(error) = set_file_modified_time(output_path, modified_time) {
        eprintln!(
            "[download] could not set upload-date file time for {}: {error}",
            output_path.display()
        );
    }
}

fn request_upload_date(request: &DownloadRequest, stdout_lines: &[String]) -> Option<String> {
    request
        .upload_date
        .trim()
        .is_empty()
        .then(|| upload_date_from_stdout(stdout_lines))
        .flatten()
        .or_else(|| {
            let trimmed = request.upload_date.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_owned())
        })
}

fn upload_date_from_stdout(lines: &[String]) -> Option<String> {
    lines.iter().find_map(|line| {
        let json = serde_json::from_str::<Value>(line.trim()).ok()?;
        json.get("upload_date")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn upload_date_to_system_time(value: &str) -> Option<SystemTime> {
    let trimmed = value.trim();
    if trimmed.len() != 8 || !trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let year = trimmed[0..4].parse::<i32>().ok()?;
    let month = trimmed[4..6].parse::<u32>().ok()?;
    let day = trimmed[6..8].parse::<u32>().ok()?;
    if !is_valid_ymd(year, month, day) {
        return None;
    }
    let days = days_from_civil(year, month, day);
    if days < 0 {
        return None;
    }
    UNIX_EPOCH.checked_add(Duration::from_secs(days as u64 * 86_400))
}

fn is_valid_ymd(year: i32, month: u32, day: u32) -> bool {
    if !(1..=12).contains(&month) || day == 0 {
        return false;
    }
    day <= days_in_month(year, month)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(mut year: i32, month: u32, day: u32) -> i64 {
    year -= if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = month as i32;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i32 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era as i64 * 146_097 + day_of_era as i64 - 719_468
}

fn set_file_modified_time(path: &Path, modified_time: SystemTime) -> std::io::Result<()> {
    let file = fs::OpenOptions::new().write(true).open(path)?;
    let times = fs::FileTimes::new().set_modified(modified_time);
    file.set_times(times)
}

fn find_same_stem_output_candidate(expected_path: &Path) -> Option<PathBuf> {
    let parent = expected_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let expected_stem = expected_path.file_stem().and_then(|value| value.to_str())?;
    let expected_ext = expected_path
        .extension()
        .and_then(|value| value.to_str())
        .and_then(normalized_extension);
    let entries = fs::read_dir(parent).ok()?;
    let mut matches = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || is_temporary_output_path(&path) {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        if stem != expected_stem {
            continue;
        }

        let Some(extension) = path
            .extension()
            .and_then(|value| value.to_str())
            .and_then(normalized_extension)
        else {
            continue;
        };
        if !is_likely_media_output_extension(&extension) {
            continue;
        }

        let preferred_rank = if expected_ext.as_deref() == Some(extension.as_str()) {
            0
        } else {
            1
        };
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok());
        matches.push((preferred_rank, modified, path));
    }

    matches.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));
    matches.into_iter().map(|(_, _, path)| path).next()
}

fn is_temporary_output_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    let normalized = name.to_ascii_lowercase();
    normalized.ends_with(".part")
        || normalized.ends_with(".ytdl")
        || normalized.ends_with(".temp")
        || normalized.ends_with(".tmp")
}

fn is_likely_media_output_extension(extension: &str) -> bool {
    matches!(
        extension,
        "mkv"
            | "mp4"
            | "webm"
            | "m4v"
            | "mov"
            | "avi"
            | "flv"
            | "ogv"
            | "ogg"
            | "mp3"
            | "m4a"
            | "opus"
            | "aac"
            | "flac"
            | "wav"
    )
}

fn finalize_subtitle_export(
    request: &DownloadRequest,
    output_path: &Path,
) -> Result<String, String> {
    let parent = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let desired_stem = output_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            format!(
                "Could not determine subtitle output file name: {}",
                output_path.display()
            )
        })?;
    let desired_ext = output_path
        .extension()
        .and_then(|value| value.to_str())
        .and_then(normalized_extension)
        .or_else(|| normalized_extension(&request.subtitle_ext))
        .unwrap_or_else(|| "srt".to_owned());
    let subtitle_lang = request.subtitle_lang.as_deref().map(str::trim);

    for candidate in subtitle_output_candidates(parent, desired_stem, &desired_ext, subtitle_lang) {
        if candidate.is_file() {
            move_output_file(&candidate, output_path)?;
            return Ok(output_path.display().to_string());
        }
    }

    if output_path.is_file() {
        return Ok(output_path.display().to_string());
    }

    if let Some(candidate) = find_subtitle_output_candidate(
        parent,
        desired_stem,
        &desired_ext,
        subtitle_lang.filter(|value| !value.is_empty()),
    ) {
        move_output_file(&candidate, output_path)?;
        return Ok(output_path.display().to_string());
    }

    Err(format!(
        "yt-dlp finished, but the converted subtitle file was not found: {}",
        output_path.display()
    ))
}

fn subtitle_output_candidates(
    parent: &Path,
    desired_stem: &str,
    desired_ext: &str,
    subtitle_lang: Option<&str>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(lang) = subtitle_lang.filter(|value| !value.is_empty()) {
        candidates.push(parent.join(format!("{desired_stem}.{lang}.{desired_ext}")));
    }
    candidates
}

fn find_subtitle_output_candidate(
    parent: &Path,
    desired_stem: &str,
    desired_ext: &str,
    preferred_lang: Option<&str>,
) -> Option<PathBuf> {
    let preferred_stem = preferred_lang.map(|lang| format!("{desired_stem}.{lang}"));
    let mut matches = Vec::new();
    let entries = fs::read_dir(parent).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(extension) = path
            .extension()
            .and_then(|value| value.to_str())
            .and_then(normalized_extension)
        else {
            continue;
        };
        if extension != desired_ext {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        if stem != desired_stem && !stem.starts_with(&format!("{desired_stem}.")) {
            continue;
        }

        let preferred_rank = if preferred_stem.as_deref() == Some(stem) {
            0
        } else {
            1
        };
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok());
        matches.push((preferred_rank, modified, path));
    }

    matches.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));
    matches.into_iter().map(|(_, _, path)| path).next()
}

fn move_output_file(source: &Path, target: &Path) -> Result<(), String> {
    if source == target {
        return Ok(());
    }
    if target.exists() {
        fs::remove_file(target)
            .map_err(|error| format!("Could not overwrite existing subtitle file: {error}"))?;
    }
    match fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(_) => {
            fs::copy(source, target).map_err(|error| {
                format!("Could not copy subtitle file to target location: {error}")
            })?;
            fs::remove_file(source)
                .map_err(|error| format!("Could not remove temporary subtitle file: {error}"))
        }
    }
}

fn normalized_extension(value: &str) -> Option<String> {
    let extension = value.trim().trim_start_matches('.').to_ascii_lowercase();
    if extension.is_empty() {
        None
    } else {
        Some(extension)
    }
}

fn humanize_download_error(request: &DownloadRequest, detail: &str) -> String {
    if request.target_kind == DownloadTargetKind::Subtitle && detail.contains("HTTP Error 429") {
        if request.write_auto_subs && request.subtitle_is_auto_translated {
            return format!(
                "YouTube temporarily rejected the auto-translated subtitle request (HTTP 429 Too Many Requests). This is rate limiting on YouTube timedtext auto-translation. The GUI keeps the native yt-dlp flow and diagnostic output instead of switching to a custom downloader. Try enabling Cookie/cookies.txt for this item, or choose original automatic subtitles/original subtitles and retry. Original message: {detail}"
            );
        }
        return format!(
            "YouTube temporarily rejected the subtitle request (HTTP 429 Too Many Requests). The source subtitle file was not downloaded, so SRT conversion will not run. Retry later, or enable browser cookies before exporting. Original message: {detail}"
        );
    }
    humanize_yt_dlp_error(detail)
}

#[derive(Clone)]
struct DownloadProgressContext {
    target_kind: DownloadTargetKind,
    video_selector: String,
    audio_selector: String,
    shared_av_progress: bool,
    section_duration_seconds: Option<f32>,
}

impl DownloadProgressContext {
    fn new(request: &DownloadRequest) -> Self {
        Self {
            target_kind: request.target_kind,
            video_selector: request.video_selector.clone(),
            audio_selector: request.audio_selector.clone(),
            shared_av_progress: request.target_kind == DownloadTargetKind::Normal
                && request.is_muxed_video,
            section_duration_seconds: parse_section_duration_seconds(&request.download_sections),
        }
    }

    fn initial_slot(&self) -> DownloadProgressSlot {
        match self.target_kind {
            DownloadTargetKind::Normal => {
                if self.shared_av_progress {
                    DownloadProgressSlot::Both
                } else {
                    DownloadProgressSlot::Video
                }
            }
            DownloadTargetKind::Video => DownloadProgressSlot::Video,
            DownloadTargetKind::Audio => DownloadProgressSlot::Audio,
            DownloadTargetKind::Subtitle => DownloadProgressSlot::Subtitle,
        }
    }
}

fn read_download_stream(
    stream: impl std::io::Read,
    is_stderr: bool,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: mpsc::Sender<DownloadEvent>,
    context: DownloadProgressContext,
) -> Vec<String> {
    let mut reader = BufReader::new(stream);
    let mut current_slot = context.initial_slot();
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
                process_download_line(
                    &pending,
                    is_stderr,
                    item_id,
                    workflow_id,
                    &tx,
                    &context,
                    &mut current_slot,
                    &mut lines,
                );
                pending.clear();
            } else {
                pending.push(byte);
            }
        }
    }

    if !pending.is_empty() {
        process_download_line(
            &pending,
            is_stderr,
            item_id,
            workflow_id,
            &tx,
            &context,
            &mut current_slot,
            &mut lines,
        );
    }

    lines
}

fn request_has_thumbnail(request: &DownloadRequest) -> bool {
    request.write_thumbnail
}

fn should_retry_without_thumbnail(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("extracted extension")
        && normalized.contains("unusual")
        && normalized.contains("skipped for safety reasons")
}

fn process_download_line(
    bytes: &[u8],
    is_stderr: bool,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: &mpsc::Sender<DownloadEvent>,
    context: &DownloadProgressContext,
    current_slot: &mut DownloadProgressSlot,
    lines: &mut Vec<String>,
) {
    let line = String::from_utf8_lossy(bytes).trim().to_owned();
    if line.is_empty() {
        return;
    }
    lines.push(line.clone());

    if is_stderr {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }

    if let Ok(json) = serde_json::from_str::<Value>(&line) {
        let _ = tx.send(DownloadEvent::Metadata { item_id, json });
        return;
    }

    if let Some(slot) = detect_download_slot(&line, context) {
        *current_slot = slot;
        let _ = tx.send(DownloadEvent::Progress {
            item_id,
            workflow_id,
            slot,
            percent: 1.0,
            detail: None,
        });
    }

    let progress = parse_yt_dlp_progress_detail(&line)
        .or_else(|| parse_default_yt_dlp_progress_percent(&line).map(|percent| (percent, None)))
        .or_else(|| parse_ffmpeg_section_progress_detail(&line, context.section_duration_seconds));

    if let Some((percent, detail)) = progress {
        let _ = tx.send(DownloadEvent::Progress {
            item_id,
            workflow_id,
            slot: *current_slot,
            percent,
            detail,
        });
    }
}

fn detect_download_slot(
    line: &str,
    context: &DownloadProgressContext,
) -> Option<DownloadProgressSlot> {
    if !line.starts_with("[download] Destination:") {
        return None;
    }
    if context.shared_av_progress {
        return Some(DownloadProgressSlot::Both);
    }
    if context.target_kind == DownloadTargetKind::Audio {
        return Some(DownloadProgressSlot::Audio);
    }
    if context.target_kind == DownloadTargetKind::Video {
        return Some(DownloadProgressSlot::Video);
    }
    if context.target_kind == DownloadTargetKind::Subtitle {
        return Some(DownloadProgressSlot::Subtitle);
    }

    if line.contains(&format!(".f{}.", context.video_selector)) {
        return Some(DownloadProgressSlot::Video);
    }
    if line.contains(&format!(".f{}.", context.audio_selector)) {
        return Some(DownloadProgressSlot::Audio);
    }
    if context.target_kind == DownloadTargetKind::Normal {
        return Some(DownloadProgressSlot::Both);
    }
    None
}

fn parse_yt_dlp_progress_detail(line: &str) -> Option<(f32, Option<DownloadProgressDetail>)> {
    if !line.starts_with("[yt-dlp],") && !line.starts_with("[direct-subtitle],") {
        return None;
    }

    let mut parts = line.split(',');
    let _prefix = parts.next()?;
    let percent_text = parts.next()?.trim();
    let eta_text = parts.next().map(str::trim).unwrap_or_default();
    let downloaded_text = parts.next().map(str::trim).unwrap_or_default();
    let total_text = parts.next().map(str::trim).unwrap_or_default();
    let speed_text = parts.next().map(str::trim).unwrap_or_default();
    let elapsed_text = parts.next().map(str::trim).unwrap_or_default();

    let percent = percent_text.trim_end_matches('%').parse::<f32>().ok()?;
    let elapsed =
        normalize_progress_text(elapsed_text).or_else(|| normalize_progress_text(eta_text));
    let detail = DownloadProgressDetail {
        downloaded: normalize_progress_bytes(downloaded_text),
        total: normalize_progress_bytes(total_text),
        speed: normalize_progress_speed(speed_text),
        elapsed,
        ..Default::default()
    };

    Some((percent, Some(detail)))
}

fn parse_default_yt_dlp_progress_percent(line: &str) -> Option<f32> {
    let body = line.strip_prefix("[download]")?.trim_start();
    if body.starts_with("Destination:") {
        return None;
    }

    body.split_whitespace()
        .find_map(|part| part.trim().strip_suffix('%'))
        .and_then(|value| value.parse::<f32>().ok())
}

fn parse_ffmpeg_section_progress_detail(
    line: &str,
    duration_seconds: Option<f32>,
) -> Option<(f32, Option<DownloadProgressDetail>)> {
    let time_text = line
        .split_whitespace()
        .find_map(|part| part.strip_prefix("time="))?;
    let elapsed = parse_progress_timestamp_seconds(time_text)?;
    let duration = duration_seconds?;
    if duration <= 0.0 {
        return None;
    }

    let detail = DownloadProgressDetail {
        downloaded: extract_ffmpeg_value(line, "size=").map(str::to_owned),
        speed: extract_ffmpeg_value(line, "bitrate=").map(str::to_owned),
        frame: extract_ffmpeg_value(line, "frame=").map(str::to_owned),
        fps: extract_ffmpeg_value(line, "fps=").map(str::to_owned),
        time: Some(time_text.to_owned()),
        ..Default::default()
    };

    Some((
        ((elapsed / duration) * 100.0).clamp(1.0, 99.0),
        Some(detail),
    ))
}

fn extract_ffmpeg_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    line.split_whitespace()
        .find_map(|part| part.trim().strip_prefix(prefix))
        .filter(|value| !value.trim().is_empty() && *value != "N/A")
}

fn normalize_progress_text(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || matches!(value, "NA" | "N/A" | "Unknown" | "unknown") {
        None
    } else {
        Some(value.to_owned())
    }
}

fn normalize_progress_bytes(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || matches!(value, "NA" | "N/A") {
        return None;
    }
    let bytes = value.parse::<f64>().ok()?;
    Some(format_progress_bytes(bytes))
}

fn normalize_progress_speed(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || matches!(value, "NA" | "N/A") {
        return None;
    }
    if let Ok(bytes) = value.parse::<f64>() {
        return Some(format!("{}/s", format_progress_bytes(bytes)));
    }
    Some(value.to_owned())
}

fn format_progress_bytes(bytes: f64) -> String {
    if !bytes.is_finite() || bytes < 0.0 {
        return String::new();
    }
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes;
    let mut unit = UNITS[0];
    for next_unit in UNITS.iter().skip(1) {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = next_unit;
    }
    if unit == "B" {
        format!("{} {unit}", value.round() as u64)
    } else if value >= 100.0 {
        format!("{value:.0} {unit}")
    } else if value >= 10.0 {
        format!("{value:.1} {unit}")
    } else {
        format!("{value:.2} {unit}")
    }
}

fn parse_section_duration_seconds(download_sections: &str) -> Option<f32> {
    let section = download_sections.trim().strip_prefix('*')?;
    let (start, end) = section.split_once('-')?;
    let start = parse_progress_timestamp_seconds(start.trim())?;
    let end = parse_progress_timestamp_seconds(end.trim())?;
    (end > start).then_some(end - start)
}

fn parse_progress_timestamp_seconds(value: &str) -> Option<f32> {
    let mut parts = value.split(':').collect::<Vec<_>>();
    if parts.len() > 3 || parts.is_empty() {
        return None;
    }
    while parts.len() < 3 {
        parts.insert(0, "0");
    }

    let hours = parts[0].parse::<f32>().ok()?;
    let minutes = parts[1].parse::<f32>().ok()?;
    let seconds = parts[2].parse::<f32>().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}
