use std::collections::HashMap;
use std::io;
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Mutex, Once, OnceLock};

use super::startup_diagnostics;

static TRACKED_CHILDREN: OnceLock<Mutex<HashMap<u32, String>>> = OnceLock::new();
static PANIC_HOOK_INSTALLED: Once = Once::new();

fn tracked_children() -> &'static Mutex<HashMap<u32, String>> {
    TRACKED_CHILDREN.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug)]
pub struct TrackedChildProcess {
    pid: u32,
    active: bool,
}

impl Drop for TrackedChildProcess {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if let Ok(mut children) = tracked_children().lock() {
            children.remove(&self.pid);
        }
        self.active = false;
    }
}

pub fn track_child_process(child: &Child, label: impl Into<String>) -> TrackedChildProcess {
    let pid = child.id();
    if let Ok(mut children) = tracked_children().lock() {
        children.insert(pid, label.into());
    }
    TrackedChildProcess { pid, active: true }
}

pub fn run_tracked_command_output(
    command: &mut Command,
    label: impl Into<String>,
) -> io::Result<Output> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command.spawn()?;
    let _guard = track_child_process(&child, label);
    child.wait_with_output()
}

pub fn force_cleanup_tracked_processes() {
    let children = match tracked_children().lock() {
        Ok(mut children) => children.drain().collect::<Vec<_>>(),
        Err(error) => error.into_inner().drain().collect::<Vec<_>>(),
    };

    for (pid, label) in children {
        eprintln!("[process-cleanup] terminate child process tree pid={pid} label={label}");
        terminate_process_tree(pid);
    }
}

pub fn install_process_cleanup_panic_hook() {
    PANIC_HOOK_INSTALLED.call_once(|| {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            startup_diagnostics::record_panic_info(panic_info);
            force_cleanup_tracked_processes();
            previous_hook(panic_info);
        }));
    });
}

#[cfg(target_os = "windows")]
fn terminate_process_tree(pid: u32) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let _ = Command::new("taskkill")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(target_os = "windows"))]
fn terminate_process_tree(pid: u32) {
    let _ = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
