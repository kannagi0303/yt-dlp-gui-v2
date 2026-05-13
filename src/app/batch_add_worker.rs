use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::Value;

use crate::app::metadata::{PlaylistEntrySeed, playlist_entry_seed_from_json};
use crate::infrastructure::{ToolPaths, configure_background_command};

pub(super) enum BatchAddEvent {
    ItemAdded {
        source: String,
        seed: PlaylistEntrySeed,
    },
    Finished {
        source: String,
        added: usize,
        stopped_by_limit: bool,
    },
    Failed {
        error: String,
    },
    Cancelled {
        added: usize,
    },
}

pub(super) fn run_batch_add_worker(
    tool_paths: ToolPaths,
    source: String,
    limit: Option<usize>,
    untitled_task: String,
    imported_template: String,
    tx: Sender<BatchAddEvent>,
    child_handle: Arc<Mutex<Option<Child>>>,
    cancel_requested: Arc<AtomicBool>,
) {
    let mut command = match tool_paths.prepare_batch_add_command(&source) {
        Ok(command) => command,
        Err(error) => {
            let _ = tx.send(BatchAddEvent::Failed { error });
            return;
        }
    };

    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            let _ = tx.send(BatchAddEvent::Failed {
                error: format!("Could not start yt-dlp: {error}"),
            });
            return;
        }
    };

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = tx.send(BatchAddEvent::Failed {
                error: "Could not read yt-dlp playlist output.".to_owned(),
            });
            return;
        }
    };
    let mut stderr = child.stderr.take();

    if let Ok(mut guard) = child_handle.lock() {
        *guard = Some(child);
    }

    let mut reader = BufReader::new(stdout);
    let mut added = 0usize;
    let mut buffer = String::new();
    let mut stopped_by_limit = false;

    loop {
        if cancel_requested.load(Ordering::Relaxed) {
            break;
        }

        buffer.clear();
        let bytes_read = match reader.read_line(&mut buffer) {
            Ok(bytes) => bytes,
            Err(error) => {
                let _ = tx.send(BatchAddEvent::Failed {
                    error: format!("Could not read yt-dlp playlist output: {error}"),
                });
                clear_batch_add_child(&child_handle);
                return;
            }
        };
        if bytes_read == 0 {
            break;
        }

        if cancel_requested.load(Ordering::Relaxed) {
            break;
        }

        let line = buffer.trim();
        if line.is_empty() {
            continue;
        }

        let Ok(entry) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        let Some(seed) = playlist_entry_seed_from_json(&entry, &untitled_task, &imported_template)
        else {
            continue;
        };

        if cancel_requested.load(Ordering::Relaxed) {
            break;
        }

        added += 1;
        let _ = tx.send(BatchAddEvent::ItemAdded {
            source: source.clone(),
            seed,
        });

        if limit.is_some_and(|value| added >= value) {
            stopped_by_limit = true;
            request_batch_add_stop(&child_handle);
            break;
        }
    }

    if cancel_requested.load(Ordering::Relaxed) {
        request_batch_add_stop(&child_handle);
    }

    let status =
        take_and_wait_batch_add_child(&child_handle, cancel_requested.load(Ordering::Relaxed));
    let was_cancelled = cancel_requested.load(Ordering::Relaxed);

    if let Some(mut stderr_reader) = stderr.take() {
        let mut stderr_text = String::new();
        let _ = stderr_reader.read_to_string(&mut stderr_text);
        if let Some(status) = status.as_ref() {
            if !status.success() && !was_cancelled && !stopped_by_limit {
                let detail = stderr_text.trim();
                let error = if detail.is_empty() {
                    format!("yt-dlp batch import failed: exit code {:?}", status.code())
                } else {
                    format!("yt-dlp batch import failed: {detail}")
                };
                let _ = tx.send(BatchAddEvent::Failed { error });
                return;
            }
        }
    }

    let event = if was_cancelled {
        let _ = source;
        BatchAddEvent::Cancelled { added }
    } else {
        BatchAddEvent::Finished {
            source,
            added,
            stopped_by_limit,
        }
    };
    let _ = tx.send(event);
}

fn clear_batch_add_child(child_handle: &Arc<Mutex<Option<Child>>>) {
    if let Ok(mut guard) = child_handle.lock() {
        *guard = None;
    }
}

pub(super) fn request_batch_add_stop(child_handle: &Arc<Mutex<Option<Child>>>) {
    if let Ok(mut guard) = child_handle.lock() {
        if let Some(child) = guard.as_mut() {
            terminate_child_process(child);
        }
    }
}

fn take_and_wait_batch_add_child(
    child_handle: &Arc<Mutex<Option<Child>>>,
    force_stop: bool,
) -> Option<std::process::ExitStatus> {
    let mut child = child_handle.lock().ok()?.take()?;
    if force_stop {
        terminate_child_process(&mut child);
        for _ in 0..40 {
            if let Ok(Some(status)) = child.try_wait() {
                return Some(status);
            }
            thread::sleep(Duration::from_millis(50));
        }
        terminate_child_process(&mut child);
    }
    child.wait().ok()
}

#[cfg(target_os = "windows")]
fn terminate_child_process(child: &mut Child) {
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
fn terminate_child_process(child: &mut Child) {
    let _ = child.kill();
}
