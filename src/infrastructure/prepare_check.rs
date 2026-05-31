use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::config::runtime_config_file_path;
use super::tool_install::{
    DependencyTool, dependency_tool_exists, dependency_tool_is_available, ffprobe_companion_path,
    portable_root_dir, resolve_support_path,
};
use super::tools::{CacheLocationMode, ToolPaths, resolve_output_dir};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrepareSeverity {
    Required,
    Recommended,
    Optional,
}

impl PrepareSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Required => "Required item",
            Self::Recommended => "Recommended item",
            Self::Optional => "Optional item",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrepareStatus {
    Ok,
    Missing,
    Warning,
    Failed,
}

impl PrepareStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ok => "prepare.status.ready",
            Self::Missing => "prepare.status.missing",
            Self::Warning => "prepare.status.warning",
            Self::Failed => "prepare.status.failed",
        }
    }

    pub fn is_ok(self) -> bool {
        self == Self::Ok
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrepareAction {
    InstallTool(DependencyTool),
}

#[derive(Clone, Debug)]
pub struct PrepareRequirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: PrepareSeverity,
    pub status: PrepareStatus,
    pub detail: String,
    pub recommendation: String,
    pub action: Option<PrepareAction>,
    pub default_selected: bool,
}

impl PrepareRequirement {
    pub fn needs_attention(&self) -> bool {
        !self.status.is_ok()
    }

    pub fn has_install_action(&self, tool: DependencyTool) -> bool {
        self.action == Some(PrepareAction::InstallTool(tool))
    }
}

#[derive(Clone, Debug, Default)]
pub struct PrepareReport {
    pub requirements: Vec<PrepareRequirement>,
}

impl PrepareReport {
    pub fn should_show_tab(&self) -> bool {
        self.requirements.iter().any(|item| {
            item.needs_attention()
                && matches!(
                    item.severity,
                    PrepareSeverity::Required | PrepareSeverity::Recommended
                )
        })
    }

    pub fn required_issue_count(&self) -> usize {
        self.requirements
            .iter()
            .filter(|item| item.severity == PrepareSeverity::Required && item.needs_attention())
            .count()
    }

    pub fn recommended_issue_count(&self) -> usize {
        self.requirements
            .iter()
            .filter(|item| item.severity == PrepareSeverity::Recommended && item.needs_attention())
            .count()
    }

    pub fn default_selected_tools(&self) -> Vec<DependencyTool> {
        self.requirements
            .iter()
            .filter(|item| item.default_selected && item.needs_attention())
            .filter_map(|item| match item.action {
                Some(PrepareAction::InstallTool(tool)) => Some(tool),
                None => None,
            })
            .collect()
    }
}

pub fn collect_prepare_report(tool_paths: &ToolPaths, download_dir: &str) -> PrepareReport {
    let mut requirements = Vec::new();

    push_tool_requirement(
        &mut requirements,
        DependencyTool::YtDlp,
        PrepareSeverity::Required,
        tool_paths.yt_dlp.as_str(),
        "Core video analysis and downloading.",
        true,
    );
    push_tool_requirement(
        &mut requirements,
        DependencyTool::Deno,
        PrepareSeverity::Recommended,
        tool_paths.deno.as_str(),
        "Improves YouTube analysis stability.",
        true,
    );
    push_tool_requirement(
        &mut requirements,
        DependencyTool::Ffmpeg,
        PrepareSeverity::Recommended,
        tool_paths.ffmpeg.as_str(),
        "Merges video/audio, converts formats, probes media info, and handles thumbnails/subtitles.",
        true,
    );
    let root = portable_root_dir();
    push_writable_requirement(
        &mut requirements,
        "app-root",
        "prepare.req.app_folder.title",
        &root,
        PrepareSeverity::Required,
        "prepare.req.app_folder.description",
    );
    push_config_file_requirement(&mut requirements, &runtime_config_file_path());
    push_writable_requirement(
        &mut requirements,
        "tools-dir",
        "prepare.req.tools_folder.title",
        &root.join("tools"),
        PrepareSeverity::Required,
        "prepare.req.tools_folder.description",
    );
    push_writable_requirement(
        &mut requirements,
        "tool-install-cache",
        "prepare.req.deployment_temp.title",
        &root.join("cache").join("tool-install"),
        PrepareSeverity::Required,
        "prepare.req.deployment_temp.description",
    );

    if tool_paths.cache_mode == CacheLocationMode::V2Cache {
        push_writable_requirement(
            &mut requirements,
            "yt-dlp-cache",
            "prepare.req.download_cache.title",
            &resolve_support_path(&tool_paths.cache_dir),
            PrepareSeverity::Required,
            "prepare.req.download_cache.description",
        );
    }

    match resolve_output_dir(download_dir) {
        Ok(path) => push_writable_requirement(
            &mut requirements,
            "output-dir",
            "prepare.req.output_folder.title",
            &path,
            PrepareSeverity::Required,
            "prepare.req.output_folder.description",
        ),
        Err(error) => requirements.push(PrepareRequirement {
            id: "output-dir".to_owned(),
            title: "prepare.req.output_folder.title".to_owned(),
            description: "prepare.req.output_folder.description".to_owned(),
            severity: PrepareSeverity::Required,
            status: PrepareStatus::Failed,
            detail: error,
            recommendation: "prepare.req.output_folder.recommendation".to_owned(),
            action: None,
            default_selected: false,
        }),
    }

    PrepareReport { requirements }
}

fn push_tool_requirement(
    requirements: &mut Vec<PrepareRequirement>,
    tool: DependencyTool,
    severity: PrepareSeverity,
    configured_path: &str,
    description: &str,
    default_selected: bool,
) {
    let installed = dependency_tool_is_available(tool, configured_path);
    let resolved = if configured_path.trim().is_empty() {
        resolve_support_path(tool.default_portable_path())
    } else {
        resolve_support_path(configured_path)
    };
    let detail = if installed {
        format!("Current path: {}", resolved.display())
    } else if tool == DependencyTool::Ffmpeg && dependency_tool_exists(configured_path) {
        let ffprobe = ffprobe_companion_path(configured_path);
        format!(
            "ffmpeg found: {}; ffprobe missing: {}",
            resolved.display(),
            ffprobe.display()
        )
    } else if configured_path.trim().is_empty() {
        format!("Default path: {}", tool.default_portable_path())
    } else {
        format!("Not found: {}", resolved.display())
    };

    requirements.push(PrepareRequirement {
        id: format!("tool-{}", tool.install_dir_name()),
        title: tool.label().to_owned(),
        description: description.to_owned(),
        severity,
        status: if installed {
            PrepareStatus::Ok
        } else {
            PrepareStatus::Missing
        },
        detail,
        recommendation: if installed {
            String::new()
        } else {
            format!("Can install to {}.", tool.default_portable_path())
        },
        action: Some(PrepareAction::InstallTool(tool)),
        default_selected,
    });
}

fn push_config_file_requirement(requirements: &mut Vec<PrepareRequirement>, path: &Path) {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let system = inspect_path_by_system_rules(parent);
    let write = probe_writable_config_file(path);

    let mut detail_parts = Vec::new();
    if !system.detail.is_empty() {
        detail_parts.push(format!("System check: {}", system.detail));
    }
    if !write.detail.is_empty() {
        detail_parts.push(format!("Save test: {}", write.detail));
    }

    let status = if write.ok {
        if system.warning {
            PrepareStatus::Warning
        } else {
            PrepareStatus::Ok
        }
    } else {
        PrepareStatus::Failed
    };

    let recommendation = if write.ok && !system.warning {
        String::new()
    } else if write.ok {
        system.recommendation
    } else if !write.recommendation.is_empty() {
        write.recommendation
    } else {
        "prepare.req.move_portable_folder".to_owned()
    };

    requirements.push(PrepareRequirement {
        id: "config-file".to_owned(),
        title: "prepare.req.config_file.title".to_owned(),
        description: "prepare.req.config_file.description"
            .to_owned(),
        severity: PrepareSeverity::Required,
        status,
        detail: detail_parts.join("; "),
        recommendation,
        action: None,
        default_selected: false,
    });
}

fn push_writable_requirement(
    requirements: &mut Vec<PrepareRequirement>,
    id: &str,
    title: &str,
    path: &Path,
    severity: PrepareSeverity,
    description: &str,
) {
    let system = inspect_path_by_system_rules(path);
    let write = probe_writable_dir(path);

    let mut detail_parts = Vec::new();
    if !system.detail.is_empty() {
        detail_parts.push(format!("System check: {}", system.detail));
    }
    if !write.detail.is_empty() {
        detail_parts.push(format!("Write test: {}", write.detail));
    }

    let ignore_path_warning = id == "output-dir";
    let status = if write.ok {
        if system.warning && !ignore_path_warning {
            PrepareStatus::Warning
        } else {
            PrepareStatus::Ok
        }
    } else {
        PrepareStatus::Failed
    };

    let recommendation = if write.ok && (!system.warning || ignore_path_warning) {
        String::new()
    } else if write.ok {
        system.recommendation
    } else if !write.recommendation.is_empty() {
        write.recommendation
    } else {
        "prepare.req.move_portable_folder"
            .to_owned()
    };

    requirements.push(PrepareRequirement {
        id: id.to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        severity,
        status,
        detail: detail_parts.join("; "),
        recommendation,
        action: None,
        default_selected: false,
    });
}

struct SystemPathInspection {
    warning: bool,
    detail: String,
    recommendation: String,
}

fn inspect_path_by_system_rules(path: &Path) -> SystemPathInspection {
    let mut warnings = Vec::new();
    let mut recommendation = String::new();

    if let Ok(metadata) = fs::metadata(path) {
        if metadata.permissions().readonly() {
            warnings.push("Folder is marked read-only".to_owned());
        }
    }

    #[cfg(target_os = "windows")]
    {
        let lowered = path
            .display()
            .to_string()
            .replace('/', "\\")
            .to_ascii_lowercase();
        if lowered.starts_with(r"\\") {
            warnings.push(
                "Located on a network path; permissions or file locks may affect it".to_owned(),
            );
        }
        if lowered.contains(r"\program files\")
            || lowered.contains(r"\program files (x86)\")
            || lowered.contains(r"\windows\")
            || lowered.ends_with(r"\windows")
        {
            warnings.push("Located in a Windows protected directory".to_owned());
            recommendation = "prepare.req.avoid_protected_folder".to_owned();
        } else if lowered.contains(r"\onedrive\") {
            warnings.push(
                "Located in a OneDrive sync path; sync locks or security blocking may occur"
                    .to_owned(),
            );
            recommendation =
                "prepare.req.move_non_synced_folder"
                    .to_owned();
        }
    }

    if recommendation.is_empty() && !warnings.is_empty() {
        recommendation = "prepare.req.generic_writable_recommendation".to_owned();
    }

    SystemPathInspection {
        warning: !warnings.is_empty(),
        detail: warnings.join("; "),
        recommendation,
    }
}

struct WriteProbeResult {
    ok: bool,
    detail: String,
    recommendation: String,
}

fn probe_writable_config_file(path: &Path) -> WriteProbeResult {
    if path.exists() && path.is_dir() {
        return WriteProbeResult {
            ok: false,
            detail: format!("{} is a folder", path.display()),
            recommendation: "prepare.req.config_not_folder".to_owned(),
        };
    }

    let Some(parent) = path.parent() else {
        return failed_write_probe(
            "Config file path could not be resolved",
            &std::io::Error::new(std::io::ErrorKind::NotFound, "missing parent directory"),
        );
    };

    if let Err(error) = fs::create_dir_all(parent) {
        return failed_write_probe("Could not create config folder", &error);
    }

    if path.exists() {
        match fs::metadata(path) {
            Ok(metadata) if metadata.permissions().readonly() => {
                return WriteProbeResult {
                    ok: false,
                    detail: "prepare.req.config_readonly".to_owned(),
                    recommendation: "prepare.req.config_readonly_recommendation".to_owned(),
                };
            }
            Ok(_) => {}
            Err(error) => return failed_write_probe("Could not read config file status", &error),
        }

        return match OpenOptions::new().write(true).open(path) {
            Ok(file) => {
                let _ = file.sync_all();
                WriteProbeResult {
                    ok: true,
                    detail: format!("Can save: {}", path.display()),
                    recommendation: String::new(),
                }
            }
            Err(error) => failed_write_probe("Could not open config file for writing", &error),
        };
    }

    probe_writable_dir(parent)
}

fn probe_writable_dir(path: &Path) -> WriteProbeResult {
    if path.exists() && !path.is_dir() {
        return WriteProbeResult {
            ok: false,
            detail: format!("{} is not a folder", path.display()),
            recommendation: "prepare.req.use_folder_path".to_owned(),
        };
    }

    let (probe_dir, target_exists) = if path.is_dir() {
        (path.to_path_buf(), true)
    } else {
        let Some(parent) = nearest_existing_parent(path) else {
            return failed_write_probe(
                "Could not find an existing parent folder",
                &std::io::Error::new(std::io::ErrorKind::NotFound, path.display().to_string()),
            );
        };
        (parent, false)
    };

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let base = format!(".yt-dlp-gui-write-test-{}-{stamp}", std::process::id());
    let test_path = probe_dir.join(format!("{base}.tmp"));
    let renamed_path = probe_dir.join(format!("{base}.ok"));

    let result = (|| -> Result<(), std::io::Error> {
        let mut file = File::create(&test_path)?;
        file.write_all(b"yt-dlp-gui write test")?;
        file.sync_all()?;
        drop(file);
        fs::rename(&test_path, &renamed_path)?;
        fs::remove_file(&renamed_path)?;
        Ok(())
    })();

    if let Err(error) = result {
        let _ = fs::remove_file(&test_path);
        let _ = fs::remove_file(&renamed_path);
        return failed_write_probe("Could not create, rename, or delete the test file", &error);
    }

    WriteProbeResult {
        ok: true,
        detail: if target_exists {
            format!("Writable: {}", path.display())
        } else {
            format!(
                "Parent writable: {}; {} will be created when needed",
                probe_dir.display(),
                path.display()
            )
        },
        recommendation: String::new(),
    }
}

fn nearest_existing_parent(path: &Path) -> Option<std::path::PathBuf> {
    let mut current = path.parent();
    while let Some(parent) = current {
        if parent.is_dir() {
            return Some(parent.to_path_buf());
        }
        current = parent.parent();
    }
    None
}

fn failed_write_probe(action: &str, error: &std::io::Error) -> WriteProbeResult {
    let (reason, recommendation) = classify_io_error(error);
    WriteProbeResult {
        ok: false,
        detail: format!("{action}: {reason} ({error})"),
        recommendation,
    }
}

fn classify_io_error(error: &std::io::Error) -> (String, String) {
    #[cfg(target_os = "windows")]
    if let Some(code) = error.raw_os_error() {
        return match code {
            3 => (
                "Path does not exist or the parent path is inaccessible".to_owned(),
                "prepare.req.drive_parent_exists".to_owned(),
            ),
            5 => (
                "Permission denied or blocked by Windows security settings".to_owned(),
                "prepare.req.permission_denied".to_owned(),
            ),
            32 => (
                "File or folder is being used by another program".to_owned(),
                "prepare.req.file_in_use".to_owned(),
            ),
            80 | 183 => (
                "Test file already exists or name conflict".to_owned(),
                "prepare.req.clear_write_test".to_owned(),
            ),
            112 => (
                "Not enough disk space".to_owned(),
                "prepare.req.free_disk_space".to_owned(),
            ),
            206 => (
                "Path is too long".to_owned(),
                "prepare.req.path_too_long".to_owned(),
            ),
            _ => (
                format!("Windows error code {code}"),
                "prepare.req.choose_writable_portable_folder".to_owned(),
            ),
        };
    }

    let reason = match error.kind() {
        std::io::ErrorKind::PermissionDenied => {
            "Permission denied or blocked by security settings".to_owned()
        }
        std::io::ErrorKind::NotFound => "Path does not exist".to_owned(),
        std::io::ErrorKind::AlreadyExists => "File already exists".to_owned(),
        _ => "Write failed".to_owned(),
    };
    (
        reason,
        "prepare.req.choose_writable_portable_folder".to_owned(),
    )
}
