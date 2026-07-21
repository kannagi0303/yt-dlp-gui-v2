use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, TryLockError};
use std::time::{SystemTime, UNIX_EPOCH};

const DIAGNOSTICS_ARG: &str = "--startup-diagnostics";
const LOG_FILE_NAME: &str = "startup-diagnostics.log";

static DIAGNOSTICS: OnceLock<Option<DiagnosticsSink>> = OnceLock::new();

struct DiagnosticsSink {
    path: PathBuf,
    file: Mutex<File>,
}

pub fn enable_startup_diagnostics_if_requested() -> bool {
    let enabled = std::env::args_os()
        .skip(1)
        .any(|arg| arg.to_string_lossy() == DIAGNOSTICS_ARG);

    let _ = DIAGNOSTICS.set(enabled.then(open_diagnostics_sink).flatten());
    if is_startup_diagnostics_enabled() {
        record_startup_event(
            "diagnostics",
            format!(
                "enabled path={}",
                diagnostics_log_path()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<unavailable>".to_owned())
            ),
        );
        record_startup_event("diagnostics", format!("args={}", command_line_for_log()));
    }

    is_startup_diagnostics_enabled()
}

pub fn is_startup_diagnostics_enabled() -> bool {
    diagnostics_sink().is_some()
}

pub fn diagnostics_log_path() -> Option<PathBuf> {
    diagnostics_sink().map(|sink| sink.path.clone())
}

pub fn record_startup_checkpoint(label: impl AsRef<str>) {
    record_line(format!("startup: {}", label.as_ref()));
}

pub fn record_startup_event(label: impl AsRef<str>, message: impl AsRef<str>) {
    record_line(format!("{}: {}", label.as_ref(), message.as_ref()));
}

pub fn record_startup_error(label: impl AsRef<str>, error: impl std::fmt::Display) {
    record_line(format!("error: {}: {}", label.as_ref(), error));
}

pub fn record_panic_info(panic_info: &dyn std::fmt::Display) {
    record_line(format!("panic: {panic_info}"));
}

fn diagnostics_sink() -> Option<&'static DiagnosticsSink> {
    DIAGNOSTICS.get().and_then(Option::as_ref)
}

fn record_line(message: String) {
    let Some(sink) = diagnostics_sink() else {
        return;
    };
    let line = format!("[{}] {message}\n", unix_timestamp_millis());
    match sink.file.try_lock() {
        Ok(mut file) => {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
        Err(TryLockError::Poisoned(error)) => {
            let mut file = error.into_inner();
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
        Err(TryLockError::WouldBlock) => {}
    }
}

fn open_diagnostics_sink() -> Option<DiagnosticsSink> {
    for path in diagnostics_log_candidates() {
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
        {
            return Some(DiagnosticsSink {
                path,
                file: Mutex::new(file),
            });
        }
    }
    None
}

fn diagnostics_log_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            candidates.push(parent.join(LOG_FILE_NAME));
        }
    }
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join(LOG_FILE_NAME));
    }
    candidates.push(std::env::temp_dir().join(LOG_FILE_NAME));
    dedup_paths(candidates)
}

fn dedup_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique = Vec::new();
    for path in paths {
        if unique.iter().any(|candidate| candidate == &path) {
            continue;
        }
        unique.push(path);
    }
    unique
}

fn command_line_for_log() -> String {
    std::env::args_os()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
