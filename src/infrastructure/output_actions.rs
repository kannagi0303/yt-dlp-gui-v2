use std::path::{Path, PathBuf};
use std::process::Command;

pub fn output_file_exists(path: &str) -> bool {
    output_path(path).is_file()
}

pub fn output_parent_folder_exists(path: &str) -> bool {
    let path = output_path(path);
    output_parent_folder(&path).is_some_and(|folder| folder.is_dir())
}

pub fn open_output_file(path: &str) -> Result<(), String> {
    let path = output_path(path);
    if !path.is_file() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    open_file_with_system(&path)
}

pub fn open_output_folder(path: &str) -> Result<(), String> {
    let path = output_path(path);

    if path.is_file() {
        return open_folder_selecting_file(&path);
    }

    let Some(folder) = output_parent_folder(&path).filter(|folder| folder.is_dir()) else {
        return Err(format!("File location does not exist: {}", path.display()));
    };

    open_folder(&folder)
}

fn output_path(path: &str) -> PathBuf {
    PathBuf::from(path.trim())
}

fn output_parent_folder(path: &Path) -> Option<PathBuf> {
    if path.is_dir() {
        return Some(path.to_path_buf());
    }
    path.parent().map(Path::to_path_buf)
}

#[cfg(target_os = "windows")]
fn open_file_with_system(path: &Path) -> Result<(), String> {
    Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open file: {error}"))
}

#[cfg(target_os = "windows")]
fn open_folder_selecting_file(path: &Path) -> Result<(), String> {
    Command::new("explorer.exe")
        .arg(format!("/select,{}", path.display()))
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open containing folder: {error}"))
}

#[cfg(target_os = "windows")]
fn open_folder(path: &Path) -> Result<(), String> {
    Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open folder: {error}"))
}

#[cfg(target_os = "macos")]
fn open_file_with_system(path: &Path) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open file: {error}"))
}

#[cfg(target_os = "macos")]
fn open_folder_selecting_file(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        return open_folder(parent);
    }
    open_file_with_system(path)
}

#[cfg(target_os = "macos")]
fn open_folder(path: &Path) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open folder: {error}"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_file_with_system(path: &Path) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open file: {error}"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_folder_selecting_file(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        return open_folder(parent);
    }
    open_file_with_system(path)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_folder(path: &Path) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open folder: {error}"))
}
