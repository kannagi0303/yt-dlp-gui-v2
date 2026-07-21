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

use super::download_resilience::{
    DownloadAttemptContext, DownloadAttemptFailure, DownloadErrorKind, DownloadResiliencePolicy,
    RecoveryDecision,
};
use crate::domain::{QueueItemId, WorkflowKind, WorkflowRunId};
use crate::infrastructure::{
    DownloadRequest, DownloadTargetKind, FINAL_OUTPUT_PATH_PREFIX, FileTimeMode, PreparedDownload,
    ToolPaths, configure_background_command, humanize_yt_dlp_error, track_child_process,
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
    ToolCommandFinished {
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        target_kind: DownloadTargetKind,
        command_line: String,
        success: bool,
        detail: Option<String>,
    },
    RecoveryStep {
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        target_kind: DownloadTargetKind,
        action: String,
        detail: String,
        recover_previous_failure: bool,
        resolved_success: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

fn emit_download_tool_finished(
    tx: &mpsc::Sender<DownloadEvent>,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    target_kind: DownloadTargetKind,
    command_line: String,
    success: bool,
    detail: Option<String>,
) {
    let _ = tx.send(DownloadEvent::ToolCommandFinished {
        item_id,
        workflow_id,
        target_kind,
        command_line,
        success,
        detail,
    });
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
    let policy = DownloadResiliencePolicy::default();
    let mut context = DownloadAttemptContext::new();
    let mut current_request = request.clone();

    let result = loop {
        let attempt = run_download_attempt(
            &tool_paths,
            &current_request,
            item_id,
            workflow_id,
            tx.clone(),
            &child_handle,
            &cancel_requested,
        );

        let failure = match attempt {
            Ok(output_path) => break Ok(output_path),
            Err(failure) => failure,
        };

        if failure.kind == DownloadErrorKind::Cancelled {
            break Err(failure.message);
        }

        if should_log_subtitle_429_diagnostics(&current_request, &failure.message) {
            log_subtitle_429_diagnostics(&current_request, &failure.message);
        }

        let decision = policy.decide(
            failure.kind,
            &context,
            request_has_thumbnail(&current_request),
            failure.recovered_output_path.is_some(),
            format_fallback_request(&current_request).is_some(),
        );

        match decision {
            RecoveryDecision::RetryWithoutThumbnail { action, detail } => {
                eprintln!("[download] {action}: {}", failure.message);
                emit_recovery_step(
                    &tx,
                    item_id,
                    workflow_id,
                    current_request.target_kind,
                    action,
                    detail,
                    true,
                    false,
                );
                context.mark_thumbnail_retry();
                current_request.embed_thumbnail = false;
                current_request.write_thumbnail = false;
                if cancel_requested.load(Ordering::Relaxed) {
                    break Err(DOWNLOAD_CANCELLED_MESSAGE.to_owned());
                }
            }
            RecoveryDecision::RetryWithFormatFallback { action, detail } => {
                let Some(fallback_request) = format_fallback_request(&current_request) else {
                    break Err(failure.message);
                };
                eprintln!("[download] {action}: {}", failure.message);
                emit_recovery_step(
                    &tx,
                    item_id,
                    workflow_id,
                    current_request.target_kind,
                    action,
                    detail,
                    true,
                    false,
                );
                context.mark_format_fallback();
                current_request = fallback_request;
                if cancel_requested.load(Ordering::Relaxed) {
                    break Err(DOWNLOAD_CANCELLED_MESSAGE.to_owned());
                }
            }
            RecoveryDecision::KeepMainOutput { action, detail } => {
                let Some(output_path) = failure.recovered_output_path else {
                    break Err(failure.message);
                };
                eprintln!("[download] {action}: {}", failure.message);
                emit_recovery_step(
                    &tx,
                    item_id,
                    workflow_id,
                    current_request.target_kind,
                    action,
                    detail,
                    true,
                    true,
                );
                break Ok(output_path);
            }
            RecoveryDecision::LogOnly { action, detail } => {
                emit_recovery_step(
                    &tx,
                    item_id,
                    workflow_id,
                    current_request.target_kind,
                    action,
                    detail,
                    false,
                    false,
                );
                break Err(failure.message);
            }
            RecoveryDecision::DoNotRecover => break Err(failure.message),
        }
    };

    let _ = tx.send(DownloadEvent::Finished(DownloadResult {
        item_id,
        workflow_id,
        workflow_kind,
        target_kind: request.target_kind,
        result,
    }));
}

fn emit_recovery_step(
    tx: &mpsc::Sender<DownloadEvent>,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    target_kind: DownloadTargetKind,
    action: &str,
    detail: &str,
    recover_previous_failure: bool,
    resolved_success: bool,
) {
    let _ = tx.send(DownloadEvent::RecoveryStep {
        item_id,
        workflow_id,
        target_kind,
        action: action.to_owned(),
        detail: detail.to_owned(),
        recover_previous_failure,
        resolved_success,
    });
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

// i18n boundary:
// This worker may emit localized app summaries through AppState/UI, but command
// lines and yt-dlp stdout/stderr captured here must remain raw technical text.
fn run_download_attempt(
    tool_paths: &ToolPaths,
    request: &DownloadRequest,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: mpsc::Sender<DownloadEvent>,
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<String, DownloadAttemptFailure> {
    let prepared = tool_paths
        .prepare_download(request)
        .map_err(DownloadAttemptFailure::from_tool_setup_error)?;

    println!("[yt-dlp] output: {}", prepared.output_path.display());
    println!("[yt-dlp] command: {}", prepared.command_line);

    let PreparedDownload {
        mut command,
        output_path,
        command_line,
    } = prepared;

    let mut child = command.spawn().map_err(|error| {
        DownloadAttemptFailure::new(
            DownloadErrorKind::ToolMissingOrBroken,
            format!("Could not start yt-dlp: {error}"),
        )
    })?;

    let _process_guard = track_child_process(&child, "yt-dlp download");

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
        emit_download_tool_finished(
            &tx,
            item_id,
            workflow_id,
            request.target_kind,
            command_line.clone(),
            false,
            Some(DOWNLOAD_CANCELLED_MESSAGE.to_owned()),
        );
        return Err(DownloadAttemptFailure::new(
            DownloadErrorKind::Cancelled,
            DOWNLOAD_CANCELLED_MESSAGE,
        ));
    }

    match status {
        Some(Ok(status)) if status.success() => {
            let result = finalize_download_output(tool_paths, request, &output_path, &stdout_lines);
            let success = result.is_ok();
            emit_download_tool_finished(
                &tx,
                item_id,
                workflow_id,
                request.target_kind,
                command_line,
                success,
                result.as_ref().err().cloned(),
            );
            result.map_err(|error| {
                let failure = DownloadAttemptFailure::from_attempt_output(
                    &stdout_lines,
                    &stderr_lines,
                    error,
                );
                attach_recovered_output_path(
                    tool_paths,
                    request,
                    &output_path,
                    &stdout_lines,
                    failure,
                )
            })
        }
        Some(Ok(status)) => {
            let detail = stderr_lines
                .iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| format!("exit code {:?}", status.code()));
            let detail = humanize_download_error(request, &detail);
            let message = format!("yt-dlp download failed: {detail}");
            emit_download_tool_finished(
                &tx,
                item_id,
                workflow_id,
                request.target_kind,
                command_line.clone(),
                false,
                Some(message.clone()),
            );
            let failure =
                DownloadAttemptFailure::from_attempt_output(&stdout_lines, &stderr_lines, message);
            Err(attach_recovered_output_path(
                tool_paths,
                request,
                &output_path,
                &stdout_lines,
                failure,
            ))
        }
        Some(Err(error)) => {
            let message = format!("Could not wait for yt-dlp to finish: {error}");
            emit_download_tool_finished(
                &tx,
                item_id,
                workflow_id,
                request.target_kind,
                command_line.clone(),
                false,
                Some(message.clone()),
            );
            Err(DownloadAttemptFailure::new(
                DownloadErrorKind::Unknown,
                message,
            ))
        }
        None => {
            let message = "Could not wait for yt-dlp to finish: child process missing".to_owned();
            emit_download_tool_finished(
                &tx,
                item_id,
                workflow_id,
                request.target_kind,
                command_line,
                false,
                Some(message.clone()),
            );
            Err(DownloadAttemptFailure::new(
                DownloadErrorKind::Unknown,
                message,
            ))
        }
    }
}

fn attach_recovered_output_path(
    tool_paths: &ToolPaths,
    request: &DownloadRequest,
    output_path: &Path,
    stdout_lines: &[String],
    failure: DownloadAttemptFailure,
) -> DownloadAttemptFailure {
    if failure.kind != DownloadErrorKind::PostprocessMetadataFailure
        || request.target_kind == DownloadTargetKind::Subtitle
    {
        return failure;
    }

    let actual_output_path = resolve_completed_output_path(output_path, stdout_lines);
    if !actual_output_path.is_file() {
        return failure;
    }

    apply_file_time_mode(tool_paths, request, &actual_output_path, stdout_lines);
    failure.with_recovered_output_path(actual_output_path.display().to_string())
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
        let mut actual_output_paths = reported_final_output_paths(stdout_lines)
            .into_iter()
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        if actual_output_paths.is_empty() {
            actual_output_paths.push(resolve_completed_output_path(output_path, stdout_lines));
        }
        for actual_output_path in &actual_output_paths {
            apply_file_time_mode(tool_paths, request, actual_output_path, stdout_lines);
        }
        let representative_output_path = actual_output_paths
            .last()
            .cloned()
            .unwrap_or_else(|| output_path.to_path_buf());
        return Ok(representative_output_path.display().to_string());
    }

    finalize_subtitle_export(request, output_path)
}

fn resolve_completed_output_path(expected_path: &Path, stdout_lines: &[String]) -> PathBuf {
    let reported_paths = reported_final_output_paths(stdout_lines);
    if let Some(reported_path) = reported_paths.iter().rev().find(|path| path.is_file()) {
        return reported_path.clone();
    }
    for reported_path in reported_paths {
        if !reported_path.is_file() {
            eprintln!(
                "[download] yt-dlp reported final path, but it does not exist: {}",
                reported_path.display()
            );
        }
    }

    if expected_path.is_file() {
        return expected_path.to_path_buf();
    }

    find_same_stem_output_candidate(expected_path).unwrap_or_else(|| expected_path.to_path_buf())
}

fn reported_final_output_paths(lines: &[String]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for line in lines {
        let Some(payload) = line.trim().strip_prefix(FINAL_OUTPUT_PATH_PREFIX) else {
            continue;
        };
        let payload = payload.trim();
        let parsed = serde_json::from_str::<String>(payload)
            .unwrap_or_else(|_| payload.trim_matches('"').to_owned());
        let trimmed = parsed.trim();
        if trimmed.is_empty() {
            continue;
        }
        let path = PathBuf::from(trimmed);
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    paths
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
    format_selector: String,
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
            format_selector: request.format_selector.clone(),
            section_duration_seconds: parse_section_duration_seconds(
                &request.download_section_args,
            ),
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

// i18n-exempt:
// yt-dlp writes progress, JSON, warnings, and errors to stdout/stderr. Capture
// those lines raw. Parse them only to derive app-owned status summaries; do not
// translate or rewrite the original tool text.
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
    let mut progress_state = DownloadProgressEmitState::default();
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
                    &mut progress_state,
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
            &mut progress_state,
            &mut lines,
        );
    }

    lines
}

fn request_has_thumbnail(request: &DownloadRequest) -> bool {
    request.write_thumbnail
}

fn format_fallback_request(request: &DownloadRequest) -> Option<DownloadRequest> {
    if request.target_kind == DownloadTargetKind::Subtitle {
        return None;
    }

    let fallback_selector = fallback_format_selector(request.target_kind)?;
    if request.format_selector.trim() == fallback_selector {
        return None;
    }

    let mut fallback = request.clone();
    fallback.format_selector = fallback_selector.to_owned();
    fallback.video_selector.clear();
    fallback.audio_selector.clear();
    fallback.is_muxed_video = request.target_kind == DownloadTargetKind::Normal;
    Some(fallback)
}

fn fallback_format_selector(target_kind: DownloadTargetKind) -> Option<&'static str> {
    match target_kind {
        DownloadTargetKind::Normal => Some("bestvideo*+bestaudio/best"),
        DownloadTargetKind::Video => {
            Some("bestvideo*[vcodec!=none]/bestvideo/best[vcodec!=none]/best")
        }
        DownloadTargetKind::Audio => Some("bestaudio/best[acodec!=none]"),
        DownloadTargetKind::Subtitle => None,
    }
}

#[derive(Default)]
struct DownloadProgressEmitState {
    video: SlotProgressEmitState,
    audio: SlotProgressEmitState,
    subtitle: SlotProgressEmitState,
    both: SlotProgressEmitState,
}

#[derive(Default)]
struct SlotProgressEmitState {
    percent: Option<f32>,
}

impl DownloadProgressEmitState {
    fn should_emit(&mut self, slot: DownloadProgressSlot, percent: f32) -> bool {
        if !percent.is_finite() {
            return false;
        }
        let percent = percent.clamp(0.0, 100.0);
        let slot_state = match slot {
            DownloadProgressSlot::Video => &mut self.video,
            DownloadProgressSlot::Audio => &mut self.audio,
            DownloadProgressSlot::Subtitle => &mut self.subtitle,
            DownloadProgressSlot::Both => &mut self.both,
        };
        slot_state.percent = Some(percent);
        true
    }
}

fn emit_download_progress(
    tx: &mpsc::Sender<DownloadEvent>,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    slot: DownloadProgressSlot,
    percent: f32,
    detail: Option<DownloadProgressDetail>,
    progress_state: &mut DownloadProgressEmitState,
) {
    let percent = percent.clamp(0.0, 100.0);
    if !progress_state.should_emit(slot, percent) {
        return;
    }
    let _ = tx.send(DownloadEvent::Progress {
        item_id,
        workflow_id,
        slot,
        percent,
        detail,
    });
}

fn process_download_line(
    bytes: &[u8],
    is_stderr: bool,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: &mpsc::Sender<DownloadEvent>,
    context: &DownloadProgressContext,
    current_slot: &mut DownloadProgressSlot,
    progress_state: &mut DownloadProgressEmitState,
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
        emit_download_progress(tx, item_id, workflow_id, slot, 1.0, None, progress_state);
    }

    if let Some(progress) = parse_yt_dlp_progress_detail(&line) {
        let slot = if progress.format_id.is_some() {
            match download_slot_for_format_id(progress.format_id.as_deref(), context) {
                Some(slot) => {
                    *current_slot = slot;
                    slot
                }
                None if context.target_kind == DownloadTargetKind::Subtitle => {
                    DownloadProgressSlot::Subtitle
                }
                None => {
                    // yt-dlp may report auxiliary downloads such as subtitles or thumbnails
                    // with format_id=NA. Those tiny files can hit 100% before the real
                    // video/audio streams start, so never let them advance the media rows.
                    return;
                }
            }
        } else {
            *current_slot
        };
        emit_download_progress(
            tx,
            item_id,
            workflow_id,
            slot,
            progress.percent,
            progress.detail,
            progress_state,
        );
        return;
    }

    let progress = parse_default_yt_dlp_progress_percent(&line)
        .map(|percent| (percent, None))
        .or_else(|| parse_ffmpeg_section_progress_detail(&line, context.section_duration_seconds));

    if let Some((percent, detail)) = progress {
        emit_download_progress(
            tx,
            item_id,
            workflow_id,
            *current_slot,
            percent,
            detail,
            progress_state,
        );
    }
}

fn download_slot_for_format_id(
    format_id: Option<&str>,
    context: &DownloadProgressContext,
) -> Option<DownloadProgressSlot> {
    let format_id = format_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| !matches!(*value, "NA" | "N/A" | "None" | "none" | "null"))?;

    let video_selector = context.video_selector.trim();
    let audio_selector = context.audio_selector.trim();

    if context.shared_av_progress
        && (!video_selector.is_empty() && format_id == video_selector
            || !audio_selector.is_empty() && format_id == audio_selector
            || format_selector_contains_id(&context.format_selector, format_id))
    {
        return Some(DownloadProgressSlot::Both);
    }
    if !video_selector.is_empty() && format_id == video_selector {
        return Some(DownloadProgressSlot::Video);
    }
    if !audio_selector.is_empty() && format_id == audio_selector {
        return Some(DownloadProgressSlot::Audio);
    }

    if video_selector.is_empty() && audio_selector.is_empty() {
        return fallback_slot_for_generic_selector(context.target_kind);
    }

    // Keep progress routing resilient even if the display selectors and the exact
    // yt-dlp format selector drift apart. The command may still be an explicit
    // split selector such as "313+251", and yt-dlp reports those concrete ids in
    // the progress template. In that case, the first selector is the video side
    // and the second selector is the audio side.
    if let Some((video_id, audio_id)) = split_explicit_av_format_selector(&context.format_selector)
    {
        if format_id == video_id {
            return Some(DownloadProgressSlot::Video);
        }
        if format_id == audio_id {
            return Some(DownloadProgressSlot::Audio);
        }
    }

    None
}

fn fallback_slot_for_generic_selector(
    target_kind: DownloadTargetKind,
) -> Option<DownloadProgressSlot> {
    match target_kind {
        DownloadTargetKind::Normal => Some(DownloadProgressSlot::Both),
        DownloadTargetKind::Video => Some(DownloadProgressSlot::Video),
        DownloadTargetKind::Audio => Some(DownloadProgressSlot::Audio),
        DownloadTargetKind::Subtitle => None,
    }
}

fn split_explicit_av_format_selector(selector: &str) -> Option<(&str, &str)> {
    let mut parts = selector
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(primary_format_selector_token);
    let video = parts.next()?;
    let audio = parts.next()?;
    if parts.next().is_some() || video.is_empty() || audio.is_empty() {
        return None;
    }
    Some((video, audio))
}

fn format_selector_contains_id(selector: &str, format_id: &str) -> bool {
    selector
        .split('+')
        .map(str::trim)
        .map(primary_format_selector_token)
        .any(|part| part == format_id)
}

fn primary_format_selector_token(selector: &str) -> &str {
    selector.split('/').next().unwrap_or(selector).trim()
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

struct ParsedYtDlpProgress {
    format_id: Option<String>,
    percent: f32,
    detail: Option<DownloadProgressDetail>,
}

fn parse_yt_dlp_progress_detail(line: &str) -> Option<ParsedYtDlpProgress> {
    if !line.starts_with("[yt-dlp],") && !line.starts_with("[direct-subtitle],") {
        return None;
    }

    let fields = line.split(',').skip(1).map(str::trim).collect::<Vec<_>>();
    let (format_id, percent_index) = parse_progress_template_format_field(&fields)?;

    let percent_text = fields.get(percent_index).copied().unwrap_or_default();
    let eta_text = fields.get(percent_index + 1).copied().unwrap_or_default();
    let downloaded_text = fields.get(percent_index + 2).copied().unwrap_or_default();
    let total_text = fields.get(percent_index + 3).copied().unwrap_or_default();
    let speed_text = fields.get(percent_index + 4).copied().unwrap_or_default();
    let elapsed_text = fields.get(percent_index + 5).copied().unwrap_or_default();

    let percent = parse_progress_percent_text(percent_text)?;
    let elapsed =
        normalize_progress_text(elapsed_text).or_else(|| normalize_progress_text(eta_text));
    let detail = DownloadProgressDetail {
        downloaded: normalize_progress_bytes(downloaded_text),
        total: normalize_progress_bytes(total_text),
        speed: normalize_progress_speed(speed_text),
        elapsed,
        ..Default::default()
    };

    Some(ParsedYtDlpProgress {
        format_id,
        percent,
        detail: Some(detail),
    })
}

fn parse_progress_template_format_field(fields: &[&str]) -> Option<(Option<String>, usize)> {
    let first = fields.first().copied()?;

    // Normal video downloads include an explicit format-id field:
    // [yt-dlp],%(info.format_id)s,%(progress._percent_str)s,...
    // Format ids are often numeric (313, 251, ...), so do not detect the
    // presence of the field by checking whether the first token parses as a
    // number. That misreads format id 313 as 313% and immediately fills the UI
    // progress bar. The older audio-only template has one fewer field and starts
    // directly with the percent text.
    if fields.len() >= 7 {
        return Some((Some(first.to_owned()), 1));
    }

    Some((None, 0))
}

fn parse_progress_percent_text(value: &str) -> Option<f32> {
    value.trim().trim_end_matches('%').parse::<f32>().ok()
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

fn parse_section_duration_seconds(download_sections: &[String]) -> Option<f32> {
    if download_sections.is_empty() {
        return None;
    }

    download_sections.iter().try_fold(0.0, |total, section| {
        let section = section.trim().strip_prefix('*')?;
        let (start, end) = section.split_once('-')?;
        let start = parse_progress_timestamp_seconds(start.trim())?;
        let end = parse_progress_timestamp_seconds(end.trim())?;
        (end > start).then_some(total + end - start)
    })
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

#[cfg(test)]
mod progress_parser_tests {
    use super::*;

    #[test]
    fn parses_numeric_format_id_without_treating_it_as_percent() {
        let progress = parse_yt_dlp_progress_detail(
            "[yt-dlp],313,  0.0%,10:48,1024,311411571,480475.14218592684,648",
        )
        .expect("progress template line should parse");

        assert_eq!(progress.format_id.as_deref(), Some("313"));
        assert_eq!(progress.percent, 0.0);
        let detail = progress.detail.expect("detail should be preserved");
        assert_eq!(detail.downloaded.as_deref(), Some("1.00 KiB"));
    }

    #[test]
    fn parses_audio_format_id_as_format_not_percent() {
        let progress = parse_yt_dlp_progress_detail(
            "[yt-dlp],251, 53.1%,00:00,2096128,3947431,64250621.58285831,0",
        )
        .expect("audio progress template line should parse");

        assert_eq!(progress.format_id.as_deref(), Some("251"));
        assert!((progress.percent - 53.1).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_legacy_template_without_format_id() {
        let progress = parse_yt_dlp_progress_detail("[yt-dlp], 12.5%,00:05,1024,8192,2048,5")
            .expect("legacy progress template line should parse");

        assert_eq!(progress.format_id, None);
        assert!((progress.percent - 12.5).abs() < f32::EPSILON);
    }

    #[test]
    fn treats_na_as_non_media_format_id_for_routing() {
        let context = DownloadProgressContext {
            target_kind: DownloadTargetKind::Normal,
            video_selector: "313".to_owned(),
            audio_selector: "251".to_owned(),
            shared_av_progress: false,
            format_selector: "313+251".to_owned(),
            section_duration_seconds: None,
        };

        assert_eq!(
            download_slot_for_format_id(Some("313"), &context),
            Some(DownloadProgressSlot::Video)
        );
        assert_eq!(
            download_slot_for_format_id(Some("251"), &context),
            Some(DownloadProgressSlot::Audio)
        );
        assert_eq!(download_slot_for_format_id(Some("NA"), &context), None);
    }

    #[test]
    fn routes_generic_fallback_progress_to_target_slot() {
        let audio_context = DownloadProgressContext {
            target_kind: DownloadTargetKind::Audio,
            video_selector: String::new(),
            audio_selector: String::new(),
            shared_av_progress: false,
            format_selector: "bestaudio/best[acodec!=none]".to_owned(),
            section_duration_seconds: None,
        };
        assert_eq!(
            download_slot_for_format_id(Some("251"), &audio_context),
            Some(DownloadProgressSlot::Audio)
        );

        let normal_context = DownloadProgressContext {
            target_kind: DownloadTargetKind::Normal,
            video_selector: String::new(),
            audio_selector: String::new(),
            shared_av_progress: true,
            format_selector: "bestvideo*+bestaudio/best".to_owned(),
            section_duration_seconds: None,
        };
        assert_eq!(
            download_slot_for_format_id(Some("399"), &normal_context),
            Some(DownloadProgressSlot::Both)
        );
    }

    #[test]
    fn sums_multiple_closed_download_section_durations() {
        let sections = vec![
            "*00:00:10-00:00:20".to_owned(),
            "*00:00:30-00:00:55.500".to_owned(),
        ];

        assert_eq!(parse_section_duration_seconds(&sections), Some(35.5));
    }

    #[test]
    fn collects_all_unique_reported_final_output_paths() {
        let lines = vec![
            format!("{FINAL_OUTPUT_PATH_PREFIX}\"C:\\\\out\\\\video - 01.mkv\""),
            format!("{FINAL_OUTPUT_PATH_PREFIX}\"C:\\\\out\\\\video - 02.mkv\""),
            format!("{FINAL_OUTPUT_PATH_PREFIX}\"C:\\\\out\\\\video - 02.mkv\""),
        ];

        assert_eq!(
            reported_final_output_paths(&lines),
            vec![
                PathBuf::from(r"C:\out\video - 01.mkv"),
                PathBuf::from(r"C:\out\video - 02.mkv"),
            ]
        );
    }
}
