use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock, mpsc};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::sha256::sha256_hex_file;
use super::tool_install::{self, DependencyTool};
use super::yaml_store::{read_yaml_file, write_yaml_file};

const USER_AGENT: &str = "yt-dlp-gui";
const APP_OWNER: &str = "kannagi0303";
const APP_REPO: &str = "yt-dlp-gui-v2";
const DOWNLOAD_BUFFER: usize = 1024 * 1024;
const DOWNLOAD_RETRY_ATTEMPTS: usize = 3;
const DOWNLOAD_RETRY_BACKOFF_BASE_MS: u64 = 600;
const APPLY_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
const APPLY_POLL_INTERVAL: Duration = Duration::from_millis(250);
const YT_DLP_ASSET_NAME: &str = "yt-dlp.exe";
const YT_DLP_DIRECT_ASSET_URL: &str =
    "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
const YT_DLP_LATEST_RELEASE_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest";
const FFMPEG_ASSET_NAME: &str = "ffmpeg-master-latest-win64-gpl.zip";
const FFMPEG_DIRECT_ASSET_URL: &str = "https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip";
const FFMPEG_LATEST_RELEASE_URL: &str = "https://github.com/BtbN/FFmpeg-Builds/releases/tag/latest";
const DENO_ASSET_NAME: &str = "deno-x86_64-pc-windows-msvc.zip";
const DENO_DIRECT_ASSET_URL: &str =
    "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-pc-windows-msvc.zip";
const DENO_LATEST_RELEASE_URL: &str = "https://github.com/denoland/deno/releases/latest";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComponentUpdateProviderKind {
    AppGithubRelease,
    YtDlpNativeLikeRelease,
    FfmpegAutoBuildRelease,
    DenoGithubRelease,
    Aria2cGithubRelease,
}

#[derive(Clone, Copy, Debug)]
struct ComponentUpdateProvider {
    kind: ComponentUpdateProviderKind,
    owner: &'static str,
    repo: &'static str,
    asset_name: Option<&'static str>,
    direct_release_url: Option<&'static str>,
    direct_asset_url: Option<&'static str>,
}

fn component_update_provider(id: ManagedComponentId) -> ComponentUpdateProvider {
    match id {
        ManagedComponentId::App => ComponentUpdateProvider {
            kind: ComponentUpdateProviderKind::AppGithubRelease,
            owner: APP_OWNER,
            repo: APP_REPO,
            asset_name: None,
            direct_release_url: None,
            direct_asset_url: None,
        },
        ManagedComponentId::YtDlp => ComponentUpdateProvider {
            kind: ComponentUpdateProviderKind::YtDlpNativeLikeRelease,
            owner: "yt-dlp",
            repo: "yt-dlp",
            asset_name: Some(YT_DLP_ASSET_NAME),
            direct_release_url: Some(YT_DLP_LATEST_RELEASE_URL),
            direct_asset_url: Some(YT_DLP_DIRECT_ASSET_URL),
        },
        ManagedComponentId::Ffmpeg => ComponentUpdateProvider {
            kind: ComponentUpdateProviderKind::FfmpegAutoBuildRelease,
            owner: "BtbN",
            repo: "FFmpeg-Builds",
            asset_name: Some(FFMPEG_ASSET_NAME),
            direct_release_url: Some(FFMPEG_LATEST_RELEASE_URL),
            direct_asset_url: Some(FFMPEG_DIRECT_ASSET_URL),
        },
        ManagedComponentId::Deno => ComponentUpdateProvider {
            kind: ComponentUpdateProviderKind::DenoGithubRelease,
            owner: "denoland",
            repo: "deno",
            asset_name: Some(DENO_ASSET_NAME),
            direct_release_url: Some(DENO_LATEST_RELEASE_URL),
            direct_asset_url: Some(DENO_DIRECT_ASSET_URL),
        },
        ManagedComponentId::Aria2c => ComponentUpdateProvider {
            kind: ComponentUpdateProviderKind::Aria2cGithubRelease,
            owner: "aria2",
            repo: "aria2",
            asset_name: None,
            direct_release_url: None,
            direct_asset_url: None,
        },
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ManagedComponentId {
    App,
    YtDlp,
    Ffmpeg,
    Deno,
    Aria2c,
}

impl ManagedComponentId {
    pub const CORE_TOOLS: [Self; 3] = [Self::YtDlp, Self::Ffmpeg, Self::Deno];
    pub const ALL: [Self; 5] = [
        Self::App,
        Self::YtDlp,
        Self::Ffmpeg,
        Self::Deno,
        Self::Aria2c,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::App => "yt-dlp-gui",
            Self::YtDlp => "yt-dlp",
            Self::Ffmpeg => "FFmpeg",
            Self::Deno => "Deno",
            Self::Aria2c => "Aria2",
        }
    }

    pub fn is_optional(self) -> bool {
        matches!(self, Self::Aria2c)
    }

    pub fn as_dependency_tool(self) -> Option<DependencyTool> {
        match self {
            Self::App => None,
            Self::YtDlp => Some(DependencyTool::YtDlp),
            Self::Ffmpeg => Some(DependencyTool::Ffmpeg),
            Self::Deno => Some(DependencyTool::Deno),
            Self::Aria2c => Some(DependencyTool::Aria2c),
        }
    }

    pub fn for_dependency_tool(tool: DependencyTool) -> Self {
        match tool {
            DependencyTool::YtDlp => Self::YtDlp,
            DependencyTool::Ffmpeg => Self::Ffmpeg,
            DependencyTool::Aria2c => Self::Aria2c,
            DependencyTool::Deno => Self::Deno,
        }
    }

    fn dir_name(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::YtDlp => "yt-dlp",
            Self::Ffmpeg => "ffmpeg",
            Self::Deno => "deno",
            Self::Aria2c => "aria2c",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentOwnership {
    ManagedPortable,
    External,
    Missing,
    Unknown,
}

impl ComponentOwnership {
    pub fn label(self) -> &'static str {
        match self {
            Self::ManagedPortable => "v2 managed",
            Self::External => "external",
            Self::Missing => "missing",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentUpdateStatus {
    Unknown,
    Checking,
    UpToDate,
    UpdateAvailable,
    Missing,
    Downloading,
    Staged,
    PendingRestart,
    Applying,
    Installed,
    Skipped,
    Failed,
}

impl ComponentUpdateStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Checking => "checking",
            Self::UpToDate => "up to date",
            Self::UpdateAvailable => "update available",
            Self::Missing => "missing",
            Self::Downloading => "downloading",
            Self::Staged => "staged",
            Self::PendingRestart => "pending restart",
            Self::Applying => "applying",
            Self::Installed => "installed",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentUpdateEntry {
    pub id: ManagedComponentId,
    pub label: String,
    pub ownership: ComponentOwnership,
    pub status: ComponentUpdateStatus,
    pub local_version: Option<String>,
    pub latest_version: Option<String>,
    pub latest_url: Option<String>,
    pub release_notes_markdown: Option<String>,
    pub progress: Option<u8>,
    #[serde(default)]
    pub total_size_bytes: Option<u64>,
    pub message: String,
}

impl ComponentUpdateEntry {
    pub fn new(id: ManagedComponentId) -> Self {
        Self {
            id,
            label: id.label().to_owned(),
            ownership: ComponentOwnership::Unknown,
            status: ComponentUpdateStatus::Unknown,
            local_version: None,
            latest_version: None,
            latest_url: None,
            release_notes_markdown: None,
            progress: None,
            total_size_bytes: None,
            message: String::new(),
        }
    }

    pub fn has_update(&self) -> bool {
        matches!(self.status, ComponentUpdateStatus::UpdateAvailable)
    }

    pub fn can_apply_tool_update(&self) -> bool {
        self.ownership == ComponentOwnership::ManagedPortable
            && matches!(
                self.status,
                ComponentUpdateStatus::UpdateAvailable | ComponentUpdateStatus::Missing
            )
            && self.id != ManagedComponentId::App
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComponentUpdateSnapshot {
    pub entries: Vec<ComponentUpdateEntry>,
    pub selected: Option<ManagedComponentId>,
    pub running: bool,
    pub message: String,
    pub checked_at_unix: Option<u64>,
}

impl ComponentUpdateSnapshot {
    pub fn entry(&self, id: ManagedComponentId) -> Option<&ComponentUpdateEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn entry_mut(&mut self, id: ManagedComponentId) -> Option<&mut ComponentUpdateEntry> {
        self.entries.iter_mut().find(|entry| entry.id == id)
    }

    pub fn ensure_entry_mut(&mut self, id: ManagedComponentId) -> &mut ComponentUpdateEntry {
        if self.entry(id).is_none() {
            self.entries.push(ComponentUpdateEntry::new(id));
        }
        self.entry_mut(id).expect("entry just inserted")
    }

    pub fn selected_entry(&self) -> Option<&ComponentUpdateEntry> {
        self.selected
            .and_then(|id| self.entry(id))
            .or_else(|| self.entry(ManagedComponentId::App))
            .or_else(|| self.entries.first())
    }
}

#[derive(Clone, Debug)]
pub enum ComponentUpdateAction {
    CheckAll,
    UpdateAllManaged,
    UpdateMany(Vec<ManagedComponentId>),
    UpdateOne(ManagedComponentId),
}

#[derive(Clone, Debug)]
pub enum ComponentUpdateEvent {
    Snapshot(ComponentUpdateSnapshot),
    Finished(ComponentUpdateSnapshot),
}

#[derive(Clone, Debug)]
enum ParallelComponentUpdateEvent {
    Progress(ComponentUpdateEntry),
    Finished(ComponentUpdateEntry),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingAppUpdateManifest {
    pub state: PendingAppUpdateState,
    pub version: String,
    pub staged_exe: PathBuf,
    pub target_exe: PathBuf,
    pub backup_exe: PathBuf,
    pub release_url: Option<String>,
    pub downloaded_at_unix: u64,
    pub attempt_count: u32,
    pub last_error: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PendingAppUpdateState {
    DownloadedStaged,
    ApplyRequested,
    Applying,
    Applied,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct InstalledAppVersionManifest {
    version: String,
    installed_at_unix: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct InstalledComponentManifest {
    entries: Vec<InstalledComponentRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct InstalledComponentRecord {
    id: ManagedComponentId,
    repo_owner: String,
    repo_name: String,
    release_tag: String,
    release_url: Option<String>,
    asset_id: Option<u64>,
    asset_name: String,
    asset_url: String,
    asset_size: Option<u64>,
    asset_updated_at: Option<String>,
    executable_path: PathBuf,
    executable_size: u64,
    executable_modified_unix: Option<u64>,
    executable_hash_fnv1a64: String,
    reported_version: Option<String>,
    installed_at_unix: u64,
}

#[derive(Clone, Debug)]
enum InstalledReleaseIdentity {
    Verified {
        tag: String,
        asset_id: Option<u64>,
        asset_name: String,
        asset_size: Option<u64>,
        asset_updated_at: Option<String>,
    },
    NotRecorded,
    FingerprintChanged,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    html_url: Option<String>,
    body: Option<String>,
    draft: Option<bool>,
    prerelease: Option<bool>,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GithubReleaseAsset {
    id: Option<u64>,
    name: String,
    browser_download_url: String,
    size: Option<u64>,
    updated_at: Option<String>,
}

#[derive(Clone, Debug)]
struct RemoteRelease {
    tag: String,
    name: Option<String>,
    url: Option<String>,
    notes_markdown: Option<String>,
    asset: Option<ReleaseAsset>,
}

#[derive(Clone, Debug)]
struct ReleaseAsset {
    id: Option<u64>,
    name: String,
    url: String,
    size: Option<u64>,
    updated_at: Option<String>,
    checksum_sha256: Option<String>,
}

#[derive(Clone, Debug)]
struct DownloadedAsset {
    path: PathBuf,
    file_name: String,
    size_bytes: Option<u64>,
}

#[derive(Clone, Copy)]
struct DownloadAssetProgress {
    percent: Option<u8>,
    total_size_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DownloadPartMetadata {
    asset_id: Option<u64>,
    asset_name: String,
    asset_url: String,
    asset_size: Option<u64>,
    asset_updated_at: Option<String>,
    asset_checksum_sha256: Option<String>,
}

struct HttpResponse {
    reader: Box<dyn Read>,
    content_length: Option<u64>,
    status: u16,
}

pub struct AppInstanceGuard {
    path: PathBuf,
}

impl Drop for AppInstanceGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn register_app_instance() -> Option<AppInstanceGuard> {
    let dir = runtime_instance_dir();
    fs::create_dir_all(&dir).ok()?;
    let current_exe = std::env::current_exe().ok()?;
    let record = AppInstanceRecord {
        pid: std::process::id(),
        exe_path: current_exe,
        started_at_unix: now_unix(),
    };
    let path = dir.join(format!("{}.yaml", record.pid));
    write_yaml_file(&path, &record).ok()?;
    Some(AppInstanceGuard { path })
}

pub fn other_registered_app_instances() -> usize {
    let Ok(entries) = fs::read_dir(runtime_instance_dir()) else {
        return 0;
    };
    let current_pid = std::process::id();
    let now = now_unix();
    let mut count = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path_has_extension(&path, "yaml") {
            if runtime_instance_marker_is_stale(&path, now) {
                let _ = fs::remove_file(&path);
            } else {
                count += 1;
            }
            continue;
        }
        let Some(record) = read_yaml_file::<AppInstanceRecord>(&path) else {
            let _ = fs::remove_file(&path);
            continue;
        };
        if record.pid == current_pid {
            continue;
        }
        if now.saturating_sub(record.started_at_unix) > 24 * 60 * 60 {
            let _ = fs::remove_file(&path);
            continue;
        }
        count += 1;
    }
    count
}

fn path_has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn runtime_instance_marker_is_stale(path: &Path, now: u64) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|modified| now.saturating_sub(modified.as_secs()) > 24 * 60 * 60)
        .unwrap_or(true)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AppInstanceRecord {
    pid: u32,
    exe_path: PathBuf,
    started_at_unix: u64,
}

pub fn component_update_cache_snapshot() -> ComponentUpdateSnapshot {
    let path = manifest_cache_dir().join("state.yaml");
    let mut snapshot = read_yaml_file::<ComponentUpdateSnapshot>(&path).unwrap_or_default();
    reset_cached_remote_check_state_for_session(&mut snapshot);
    refresh_snapshot_local_component_state(&mut snapshot);
    snapshot
}

pub fn component_update_startup_snapshot() -> ComponentUpdateSnapshot {
    let path = manifest_cache_dir().join("state.yaml");
    let mut snapshot = read_yaml_file::<ComponentUpdateSnapshot>(&path).unwrap_or_default();
    reset_cached_remote_check_state_for_session(&mut snapshot);
    refresh_snapshot_local_component_presence(&mut snapshot);
    snapshot
}

fn reset_cached_remote_check_state_for_session(snapshot: &mut ComponentUpdateSnapshot) {
    snapshot.running = false;
    snapshot.message.clear();
    for entry in &mut snapshot.entries {
        reset_cached_entry_remote_check_state_for_session(entry);
    }
}

fn reset_cached_entry_remote_check_state_for_session(entry: &mut ComponentUpdateEntry) {
    if entry.id == ManagedComponentId::App && entry.status == ComponentUpdateStatus::PendingRestart
    {
        entry.progress = None;
        entry.total_size_bytes = None;
        return;
    }

    entry.latest_version = None;
    entry.latest_url = None;
    entry.release_notes_markdown = None;
    entry.progress = None;
    entry.total_size_bytes = None;

    entry.status = match entry.status {
        ComponentUpdateStatus::Missing => ComponentUpdateStatus::Missing,
        _ => ComponentUpdateStatus::Unknown,
    };

    entry.message = match entry.status {
        ComponentUpdateStatus::Missing => "not installed".to_owned(),
        _ => "not checked".to_owned(),
    };
}

pub fn save_component_update_cache(snapshot: &ComponentUpdateSnapshot) {
    let path = manifest_cache_dir().join("state.yaml");
    let _ = write_yaml_file(&path, snapshot);
}

pub fn pending_app_update_manifest() -> Option<PendingAppUpdateManifest> {
    read_yaml_file(&pending_app_update_manifest_path())
}

pub fn clear_pending_app_update_manifest() {
    let _ = fs::remove_file(pending_app_update_manifest_path());
}

pub fn run_component_update_action(
    action: ComponentUpdateAction,
    proxy_url: Option<String>,
    mut emit: impl FnMut(ComponentUpdateEvent),
) {
    let mut snapshot = component_update_cache_snapshot();
    snapshot.running = true;
    snapshot.message = match &action {
        ComponentUpdateAction::CheckAll => "checking updates".to_owned(),
        ComponentUpdateAction::UpdateAllManaged => "updating managed components".to_owned(),
        ComponentUpdateAction::UpdateMany(ids) => update_many_message(ids),
        ComponentUpdateAction::UpdateOne(id) => format!("updating {}", id.label()),
    };
    if snapshot.entries.is_empty() {
        snapshot.entries = ManagedComponentId::ALL
            .into_iter()
            .map(ComponentUpdateEntry::new)
            .collect();
    }
    mark_action_entries_started(&mut snapshot, &action);
    emit(ComponentUpdateEvent::Snapshot(snapshot.clone()));

    let result = match action {
        ComponentUpdateAction::CheckAll => {
            check_all_components(&mut snapshot, proxy_url.as_deref(), &mut emit)
        }
        ComponentUpdateAction::UpdateAllManaged => {
            update_all_managed(&mut snapshot, proxy_url.as_deref(), &mut emit)
        }
        ComponentUpdateAction::UpdateMany(ids) => {
            update_many_managed(&mut snapshot, ids, proxy_url.as_deref(), &mut emit)
        }
        ComponentUpdateAction::UpdateOne(id) => {
            update_one_component(&mut snapshot, id, proxy_url.as_deref(), &mut emit)
        }
    };

    if let Err(error) = result {
        snapshot.message = error;
    }
    snapshot.running = false;
    snapshot.checked_at_unix = Some(now_unix());
    save_component_update_cache(&snapshot);
    emit(ComponentUpdateEvent::Finished(snapshot));
}

fn update_many_message(ids: &[ManagedComponentId]) -> String {
    let ids = deduplicate_component_ids(ids.to_vec());
    if ids.is_empty() {
        "no managed components selected".to_owned()
    } else {
        format!(
            "updating {}",
            ids.iter()
                .map(|id| id.label())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn mark_action_entries_started(
    snapshot: &mut ComponentUpdateSnapshot,
    action: &ComponentUpdateAction,
) {
    let ids = component_update_action_initial_ids(action);
    let message = match action {
        ComponentUpdateAction::CheckAll => "checking",
        ComponentUpdateAction::UpdateAllManaged
        | ComponentUpdateAction::UpdateMany(_)
        | ComponentUpdateAction::UpdateOne(_) => "queued",
    };
    for id in ids.iter().copied() {
        let entry = snapshot.ensure_entry_mut(id);
        entry.status = ComponentUpdateStatus::Checking;
        entry.progress = None;
        entry.message = message.to_owned();
    }
    if matches!(
        action,
        ComponentUpdateAction::UpdateMany(_) | ComponentUpdateAction::UpdateOne(_)
    ) {
        snapshot.selected = ids.first().copied();
    }
}

fn component_update_action_initial_ids(action: &ComponentUpdateAction) -> Vec<ManagedComponentId> {
    match action {
        ComponentUpdateAction::CheckAll | ComponentUpdateAction::UpdateAllManaged => {
            ManagedComponentId::ALL.to_vec()
        }
        ComponentUpdateAction::UpdateMany(ids) => deduplicate_component_ids(ids.clone()),
        ComponentUpdateAction::UpdateOne(id) => vec![*id],
    }
}

fn deduplicate_component_ids(ids: Vec<ManagedComponentId>) -> Vec<ManagedComponentId> {
    let mut result = Vec::new();
    for id in ids {
        if !result.contains(&id) {
            result.push(id);
        }
    }
    result
}

fn refresh_snapshot_local_component_state(snapshot: &mut ComponentUpdateSnapshot) {
    for id in ManagedComponentId::ALL {
        let ownership = component_ownership(id);
        let local_version = local_component_version(id);
        let entry = snapshot.ensure_entry_mut(id);
        apply_local_component_probe(entry, ownership, local_version);
    }
}

fn refresh_snapshot_local_component_presence(snapshot: &mut ComponentUpdateSnapshot) {
    for id in ManagedComponentId::ALL {
        let ownership = component_ownership(id);
        let entry = snapshot.ensure_entry_mut(id);
        let cached_version =
            startup_cached_local_version(id, ownership, entry.local_version.clone());
        apply_local_component_probe(entry, ownership, cached_version);
    }
}

fn startup_cached_local_version(
    id: ManagedComponentId,
    ownership: ComponentOwnership,
    cached_version: Option<String>,
) -> Option<String> {
    match (id, ownership) {
        (ManagedComponentId::App, _) => current_app_build_version(),
        (_, ComponentOwnership::Missing) => None,
        _ => cached_version,
    }
}

fn apply_local_component_probe(
    entry: &mut ComponentUpdateEntry,
    ownership: ComponentOwnership,
    local_version: Option<String>,
) {
    let local_component_available =
        ownership != ComponentOwnership::Missing || local_version.is_some();

    entry.label = entry.id.label().to_owned();
    entry.ownership = ownership;
    entry.local_version = local_version;

    if local_component_available
        && matches!(
            entry.status,
            ComponentUpdateStatus::Missing | ComponentUpdateStatus::Failed
        )
    {
        entry.status = ComponentUpdateStatus::Unknown;
        entry.progress = None;
        entry.message = "not checked".to_owned();
    } else if !local_component_available {
        entry.status = ComponentUpdateStatus::Missing;
        entry.progress = None;
        entry.message = "not installed".to_owned();
    }
}

fn check_all_components(
    snapshot: &mut ComponentUpdateSnapshot,
    proxy_url: Option<&str>,
    emit: &mut impl FnMut(ComponentUpdateEvent),
) -> Result<(), String> {
    check_components(snapshot, &ManagedComponentId::ALL, proxy_url, emit)
}

fn check_components(
    snapshot: &mut ComponentUpdateSnapshot,
    ids: &[ManagedComponentId],
    proxy_url: Option<&str>,
    emit: &mut impl FnMut(ComponentUpdateEvent),
) -> Result<(), String> {
    let mut handles = Vec::new();
    for id in ids.iter().copied() {
        let proxy = proxy_url.map(str::to_owned);
        handles.push(thread::spawn(move || check_component(id, proxy.as_deref())));
    }

    for handle in handles {
        let entry = handle
            .join()
            .map_err(|_| "update check worker panicked".to_owned())?;
        snapshot.ensure_entry_mut(entry.id).clone_from(&entry);
        emit(ComponentUpdateEvent::Snapshot(snapshot.clone()));
    }
    snapshot.message = "update check complete".to_owned();
    Ok(())
}

fn update_all_managed(
    snapshot: &mut ComponentUpdateSnapshot,
    proxy_url: Option<&str>,
    emit: &mut impl FnMut(ComponentUpdateEvent),
) -> Result<(), String> {
    check_all_components(snapshot, proxy_url, emit)?;

    let targets = component_update_targets(snapshot, &ManagedComponentId::ALL, false);
    update_managed_targets(snapshot, targets, proxy_url, emit)
}

fn update_many_managed(
    snapshot: &mut ComponentUpdateSnapshot,
    ids: Vec<ManagedComponentId>,
    proxy_url: Option<&str>,
    emit: &mut impl FnMut(ComponentUpdateEvent),
) -> Result<(), String> {
    let ids = deduplicate_component_ids(ids);
    if ids.is_empty() {
        snapshot.message = "no managed components selected".to_owned();
        return Ok(());
    }

    check_components(snapshot, &ids, proxy_url, emit)?;
    let targets = component_update_targets(snapshot, &ids, true);
    update_managed_targets(snapshot, targets, proxy_url, emit)
}

fn component_update_targets(
    snapshot: &ComponentUpdateSnapshot,
    ids: &[ManagedComponentId],
    include_optional_missing: bool,
) -> Vec<ManagedComponentId> {
    ids.iter()
        .copied()
        .filter(|id| {
            snapshot.entry(*id).is_some_and(|entry| {
                if !include_optional_missing
                    && entry.id.is_optional()
                    && entry.ownership == ComponentOwnership::Missing
                {
                    return false;
                }

                matches!(
                    entry.status,
                    ComponentUpdateStatus::UpdateAvailable | ComponentUpdateStatus::Missing
                ) && matches!(
                    entry.ownership,
                    ComponentOwnership::ManagedPortable | ComponentOwnership::Missing
                )
            })
        })
        .collect()
}

fn update_managed_targets(
    snapshot: &mut ComponentUpdateSnapshot,
    targets: Vec<ManagedComponentId>,
    proxy_url: Option<&str>,
    emit: &mut impl FnMut(ComponentUpdateEvent),
) -> Result<(), String> {
    if targets.is_empty() {
        return Ok(());
    }

    let (progress_tx, progress_rx) = mpsc::channel();
    let mut handles = Vec::new();
    for id in targets.iter().copied() {
        let proxy = proxy_url.map(str::to_owned);
        if let Some(entry) = snapshot.entry_mut(id) {
            entry.status = ComponentUpdateStatus::Downloading;
            entry.progress = None;
            entry.message = "queued".to_owned();
        }
        let thread_tx = progress_tx.clone();
        handles.push(thread::spawn(move || {
            let progress_event_tx = thread_tx.clone();
            let entry = update_component_to_entry_with_progress(id, proxy.as_deref(), |entry| {
                let _ = progress_event_tx.send(ParallelComponentUpdateEvent::Progress(entry));
            });
            let _ = thread_tx.send(ParallelComponentUpdateEvent::Finished(entry));
        }));
    }
    drop(progress_tx);
    emit(ComponentUpdateEvent::Snapshot(snapshot.clone()));

    let mut finished = 0usize;
    while finished < handles.len() {
        match progress_rx.recv() {
            Ok(ParallelComponentUpdateEvent::Progress(entry)) => {
                snapshot.ensure_entry_mut(entry.id).clone_from(&entry);
                snapshot.selected = Some(entry.id);
                emit(ComponentUpdateEvent::Snapshot(snapshot.clone()));
            }
            Ok(ParallelComponentUpdateEvent::Finished(entry)) => {
                snapshot.ensure_entry_mut(entry.id).clone_from(&entry);
                snapshot.selected = Some(entry.id);
                finished += 1;
                emit(ComponentUpdateEvent::Snapshot(snapshot.clone()));
            }
            Err(_) => break,
        }
    }

    for handle in handles {
        handle
            .join()
            .map_err(|_| "component update worker panicked".to_owned())?;
    }
    Ok(())
}

fn update_component_to_entry_with_progress(
    id: ManagedComponentId,
    proxy_url: Option<&str>,
    mut progress: impl FnMut(ComponentUpdateEntry),
) -> ComponentUpdateEntry {
    let mut entry = check_component(id, proxy_url);
    progress(entry.clone());
    if matches!(
        entry.ownership,
        ComponentOwnership::External | ComponentOwnership::Unknown
    ) {
        entry.status = ComponentUpdateStatus::Skipped;
        entry.message = "external component is not managed by v2".to_owned();
        return entry;
    }
    if !matches!(
        entry.status,
        ComponentUpdateStatus::Missing | ComponentUpdateStatus::UpdateAvailable
    ) {
        return entry;
    }
    let result = (|| {
        let remote = resolve_remote_release(id, proxy_url)?;
        entry.latest_version = Some(remote_release_display_version(id, &remote));
        entry.latest_url = remote.url.clone();
        let asset = remote
            .asset
            .clone()
            .ok_or_else(|| format!("{} release asset not found", id.label()))?;
        entry.total_size_bytes = asset.size;
        entry.status = ComponentUpdateStatus::Downloading;
        entry.progress = Some(0);
        entry.message = "downloading".to_owned();
        progress(entry.clone());
        let downloaded = download_asset_resumable(id, &asset, proxy_url, |download_progress| {
            entry.status = ComponentUpdateStatus::Downloading;
            entry.progress = download_progress.percent;
            entry.total_size_bytes = download_progress.total_size_bytes;
            progress(entry.clone());
        })?;
        entry.total_size_bytes = entry.total_size_bytes.or(downloaded.size_bytes);
        match id {
            ManagedComponentId::App => {
                entry.status = ComponentUpdateStatus::Staged;
                entry.progress = Some(100);
                entry.message = "staging".to_owned();
                progress(entry.clone());
                let staged_exe = stage_app_update(&downloaded, &remote)?;
                write_pending_app_manifest(&remote, staged_exe)?;
                entry.status = ComponentUpdateStatus::PendingRestart;
                entry.message = "downloaded; restart required".to_owned();
            }
            _ => {
                entry.status = ComponentUpdateStatus::Applying;
                entry.progress = Some(0);
                entry.message = tool_apply_stage_message(&downloaded).to_owned();
                progress(entry.clone());
                apply_tool_update(id, &downloaded, |apply_percent| {
                    entry.status = ComponentUpdateStatus::Applying;
                    entry.progress = apply_percent;
                    entry.message = tool_apply_stage_message(&downloaded).to_owned();
                    progress(entry.clone());
                })?;
                write_installed_component_record(id, &remote, &asset)?;
                entry.local_version = local_component_version(id);
                entry.status = ComponentUpdateStatus::Installed;
                entry.progress = Some(100);
                entry.message = "installed".to_owned();
            }
        }
        Ok::<(), String>(())
    })();
    if let Err(error) = result {
        entry.status = ComponentUpdateStatus::Failed;
        entry.progress = None;
        entry.message = error;
    }
    entry
}

fn update_one_component(
    snapshot: &mut ComponentUpdateSnapshot,
    id: ManagedComponentId,
    proxy_url: Option<&str>,
    emit: &mut impl FnMut(ComponentUpdateEvent),
) -> Result<(), String> {
    snapshot.selected = Some(id);
    let entry = update_component_to_entry_with_progress(id, proxy_url, |entry| {
        snapshot.ensure_entry_mut(id).clone_from(&entry);
        snapshot.selected = Some(id);
        emit(ComponentUpdateEvent::Snapshot(snapshot.clone()));
    });
    snapshot.ensure_entry_mut(id).clone_from(&entry);
    save_component_update_cache(snapshot);
    emit(ComponentUpdateEvent::Snapshot(snapshot.clone()));
    Ok(())
}

fn check_component(id: ManagedComponentId, proxy_url: Option<&str>) -> ComponentUpdateEntry {
    let mut entry = ComponentUpdateEntry::new(id);
    entry.status = ComponentUpdateStatus::Checking;
    entry.ownership = component_ownership(id);
    entry.local_version = local_component_version(id);

    if entry.ownership == ComponentOwnership::Missing {
        entry.status = ComponentUpdateStatus::Missing;
    }

    match resolve_remote_release(id, proxy_url) {
        Ok(remote) => {
            let initial_release_identity = installed_release_identity(id);
            let reanchored_external_change = try_reanchor_external_component(
                id,
                entry.ownership,
                entry.local_version.as_deref(),
                &initial_release_identity,
                &remote,
            )
            .unwrap_or(false);
            let release_identity = installed_release_identity(id);
            let comparison_version = component_update_comparison_version(
                id,
                entry.local_version.as_deref(),
                &release_identity,
            );
            entry.latest_version = Some(remote_release_display_version(id, &remote));
            entry.latest_url = remote.url.clone();
            entry.total_size_bytes = remote.asset.as_ref().and_then(|asset| asset.size);
            entry.release_notes_markdown =
                release_notes_range_markdown(id, comparison_version.as_deref(), proxy_url)
                    .ok()
                    .or_else(|| {
                        Some(build_release_notes_markdown(
                            id,
                            comparison_version.as_deref(),
                            &remote,
                        ))
                    });
            if entry.ownership == ComponentOwnership::Missing {
                entry.status = ComponentUpdateStatus::Missing;
                entry.message = "not installed".to_owned();
            } else if remote_release_matches_comparison(
                comparison_version.as_deref(),
                &release_identity,
                &remote,
            ) {
                entry.status = ComponentUpdateStatus::UpToDate;
                entry.message = if reanchored_external_change {
                    "external update accepted by v2".to_owned()
                } else {
                    "up to date".to_owned()
                };
            } else if comparison_version.is_none()
                && entry.ownership == ComponentOwnership::ManagedPortable
            {
                entry.status = ComponentUpdateStatus::UpdateAvailable;
                entry.message = portable_unknown_release_identity_message(&release_identity);
            } else {
                entry.status = ComponentUpdateStatus::UpdateAvailable;
                entry.message = "update available".to_owned();
            }
        }
        Err(error) => {
            entry.status = ComponentUpdateStatus::Failed;
            entry.message = error;
        }
    }

    if id == ManagedComponentId::App {
        if let Some(manifest) = pending_app_update_manifest() {
            if matches!(
                manifest.state,
                PendingAppUpdateState::DownloadedStaged | PendingAppUpdateState::ApplyRequested
            ) {
                entry.latest_version = Some(manifest.version);
                entry.status = ComponentUpdateStatus::PendingRestart;
                entry.message = "downloaded; restart required".to_owned();
            }
        }
    }

    entry
}

fn resolve_remote_release(
    id: ManagedComponentId,
    proxy_url: Option<&str>,
) -> Result<RemoteRelease, String> {
    match id {
        ManagedComponentId::App => {
            github_latest_release(id, APP_OWNER, APP_REPO, proxy_url, select_app_asset)
        }
        ManagedComponentId::YtDlp => resolve_release_with_direct_asset_fallback(id, || {
            github_latest_release(id, "yt-dlp", "yt-dlp", proxy_url, |asset| {
                asset.name.eq_ignore_ascii_case(YT_DLP_ASSET_NAME)
            })
        }),
        ManagedComponentId::Ffmpeg => resolve_release_with_direct_asset_fallback(id, || {
            github_latest_release_required_asset(id, "BtbN", "FFmpeg-Builds", proxy_url, |asset| {
                asset.name.eq_ignore_ascii_case(FFMPEG_ASSET_NAME)
            })
        }),
        ManagedComponentId::Deno => resolve_release_with_direct_asset_fallback(id, || {
            github_latest_release(id, "denoland", "deno", proxy_url, |asset| {
                asset.name.eq_ignore_ascii_case(DENO_ASSET_NAME)
            })
        }),
        ManagedComponentId::Aria2c => {
            github_latest_release(id, "aria2", "aria2", proxy_url, |asset| {
                let name = asset.name.to_ascii_lowercase();
                name.contains("win-64bit") && name.ends_with(".zip")
            })
        }
    }
}

fn resolve_release_with_direct_asset_fallback(
    id: ManagedComponentId,
    resolve: impl FnOnce() -> Result<RemoteRelease, String>,
) -> Result<RemoteRelease, String> {
    resolve().or_else(|error| direct_latest_asset_release(id).ok_or(error))
}

fn direct_latest_asset_release(id: ManagedComponentId) -> Option<RemoteRelease> {
    let provider = component_update_provider(id);
    let release_url = provider.direct_release_url?;
    let asset_name = provider.asset_name?;
    let asset_url = provider.direct_asset_url?;
    Some(RemoteRelease {
        tag: direct_latest_release_tag(id),
        name: Some(format!("{} latest", id.label())),
        url: Some(release_url.to_owned()),
        notes_markdown: None,
        asset: Some(ReleaseAsset {
            id: None,
            name: asset_name.to_owned(),
            url: asset_url.to_owned(),
            size: None,
            updated_at: None,
            checksum_sha256: None,
        }),
    })
}

fn direct_latest_release_tag(id: ManagedComponentId) -> String {
    if component_release_tag_can_be_inferred_from_local_version(id) {
        if let Some(version) =
            local_component_version(id).filter(|version| !version.trim().is_empty())
        {
            return version;
        }
    }

    "latest".to_owned()
}

fn remote_release_display_version(id: ManagedComponentId, remote: &RemoteRelease) -> String {
    if id == ManagedComponentId::Ffmpeg && normalize_version(&remote.tag) == "latest" {
        return ffmpeg_daily_build_display_version(remote);
    }

    remote.tag.clone()
}

fn ffmpeg_daily_build_display_version(remote: &RemoteRelease) -> String {
    remote
        .asset
        .as_ref()
        .and_then(|asset| asset.updated_at.as_deref())
        .and_then(format_iso8601_date_prefix_as_display_version)
        .unwrap_or_else(|| "-".to_owned())
}

fn ffmpeg_build_date_display_version(local_version: Option<&str>) -> Option<String> {
    let value = local_version?.trim();
    let date = value
        .rsplit('-')
        .next()
        .filter(|part| part.len() == 8 && part.chars().all(|ch| ch.is_ascii_digit()))?;
    Some(format!("{}.{}.{}", &date[0..4], &date[4..6], &date[6..8]))
}

fn local_component_display_version(
    id: ManagedComponentId,
    local_version: Option<&str>,
) -> Option<String> {
    match id {
        ManagedComponentId::Ffmpeg => ffmpeg_build_date_display_version(local_version),
        _ => local_version.map(str::to_owned),
    }
}

fn format_iso8601_date_prefix_as_display_version(value: &str) -> Option<String> {
    let date = value.get(..10)?;
    let mut chars = date.chars();
    let year: String = chars.by_ref().take(4).collect();
    if year.len() != 4 || !year.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    if !matches!(chars.next(), Some('-')) {
        return None;
    }
    let month: String = chars.by_ref().take(2).collect();
    if month.len() != 2 || !month.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    if !matches!(chars.next(), Some('-')) {
        return None;
    }
    let day: String = chars.take(2).collect();
    if day.len() != 2 || !day.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    Some(format!("{year}.{month}.{day}"))
}

fn github_latest_release(
    id: ManagedComponentId,
    owner: &str,
    repo: &str,
    proxy_url: Option<&str>,
    select_asset: impl Fn(&GithubReleaseAsset) -> bool,
) -> Result<RemoteRelease, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let release: GithubRelease = request_json(&url, proxy_url)?;
    let asset = release
        .assets
        .iter()
        .find(|asset| select_asset(asset))
        .or_else(|| {
            release
                .assets
                .iter()
                .find(|asset| asset.name.ends_with(".zip") || asset.name.ends_with(".exe"))
        })
        .map(|asset| release_asset_from_github(id, &release.assets, asset, proxy_url));
    Ok(RemoteRelease {
        tag: release.tag_name,
        name: release.name,
        url: release.html_url,
        notes_markdown: release.body,
        asset,
    })
}

fn github_latest_release_required_asset(
    id: ManagedComponentId,
    owner: &str,
    repo: &str,
    proxy_url: Option<&str>,
    select_asset: impl Fn(&GithubReleaseAsset) -> bool,
) -> Result<RemoteRelease, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let release: GithubRelease = request_json(&url, proxy_url)?;
    let asset = release
        .assets
        .iter()
        .find(|asset| select_asset(asset))
        .map(|asset| release_asset_from_github(id, &release.assets, asset, proxy_url))
        .ok_or_else(|| format!("release asset not found for {owner}/{repo}"))?;

    Ok(RemoteRelease {
        tag: release.tag_name,
        name: release.name,
        url: release.html_url,
        notes_markdown: release.body,
        asset: Some(asset),
    })
}

fn release_asset_from_github(
    id: ManagedComponentId,
    release_assets: &[GithubReleaseAsset],
    asset: &GithubReleaseAsset,
    proxy_url: Option<&str>,
) -> ReleaseAsset {
    ReleaseAsset {
        id: asset.id,
        name: asset.name.clone(),
        url: asset.browser_download_url.clone(),
        size: asset.size,
        updated_at: asset.updated_at.clone(),
        checksum_sha256: component_release_asset_checksum_sha256(
            id,
            release_assets,
            asset,
            proxy_url,
        ),
    }
}

fn component_release_asset_checksum_sha256(
    id: ManagedComponentId,
    release_assets: &[GithubReleaseAsset],
    asset: &GithubReleaseAsset,
    proxy_url: Option<&str>,
) -> Option<String> {
    let provider = component_update_provider(id);
    match provider.kind {
        ComponentUpdateProviderKind::YtDlpNativeLikeRelease
        | ComponentUpdateProviderKind::FfmpegAutoBuildRelease
        | ComponentUpdateProviderKind::DenoGithubRelease => {}
        _ => return None,
    }

    for checksum_asset in checksum_assets_for_main_asset(release_assets, &asset.name) {
        let Ok(text) = request_text(&checksum_asset.browser_download_url, proxy_url) else {
            continue;
        };
        if let Some(hash) = parse_sha256_checksum_for_asset(&text, &asset.name) {
            return Some(hash);
        }
    }

    for url in checksum_sidecar_urls(&asset.browser_download_url) {
        let Ok(text) = request_text(&url, proxy_url) else {
            continue;
        };
        if let Some(hash) = parse_sha256_checksum_for_asset(&text, &asset.name) {
            return Some(hash);
        }
    }

    None
}

fn checksum_assets_for_main_asset<'a>(
    release_assets: &'a [GithubReleaseAsset],
    main_asset_name: &str,
) -> Vec<&'a GithubReleaseAsset> {
    let mut candidates = Vec::new();
    let main = main_asset_name.to_ascii_lowercase();
    for asset in release_assets {
        let name = asset.name.to_ascii_lowercase();
        if name == "sha2-256sums" || name == "checksums.sha256" || name.contains("checksum") {
            candidates.push(asset);
            continue;
        }
        if name.starts_with(&main) && name.contains("sha256") {
            candidates.push(asset);
        }
    }
    candidates
}

fn checksum_sidecar_urls(asset_url: &str) -> Vec<String> {
    [".sha256sum", ".sha256", ".sha256sums"]
        .into_iter()
        .map(|suffix| format!("{asset_url}{suffix}"))
        .collect()
}

fn parse_sha256_checksum_for_asset(text: &str, asset_name: &str) -> Option<String> {
    let wanted_name = asset_name.to_ascii_lowercase();
    let mut first_hash = None;
    let mut hash_count = 0usize;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut line_hash = None;
        let mut mentions_asset = false;
        for token in line.split_whitespace() {
            let cleaned = token
                .trim_start_matches('*')
                .trim_matches(|ch: char| ch == '(' || ch == ')' || ch == ':');
            if is_sha256_hex(cleaned) {
                hash_count = hash_count.saturating_add(1);
                line_hash = Some(cleaned.to_ascii_lowercase());
                if first_hash.is_none() {
                    first_hash = line_hash.clone();
                }
                continue;
            }
            let normalized = cleaned.replace('\\', "/").to_ascii_lowercase();
            if normalized == wanted_name || normalized.ends_with(&format!("/{wanted_name}")) {
                mentions_asset = true;
            }
        }
        if mentions_asset {
            if let Some(hash) = line_hash {
                return Some(hash);
            }
        }
    }

    (hash_count == 1).then_some(first_hash).flatten()
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn select_app_asset(asset: &GithubReleaseAsset) -> bool {
    let name = asset.name.to_ascii_lowercase();
    (name.contains("windows") || name.contains("win"))
        && (name.contains("x64") || name.contains("x86_64") || name.contains("amd64"))
        && (name.ends_with(".zip") || name.ends_with(".exe"))
}

fn github_repo_for(id: ManagedComponentId) -> Option<(&'static str, &'static str)> {
    let provider = component_update_provider(id);
    Some((provider.owner, provider.repo))
}

fn release_notes_range_markdown(
    id: ManagedComponentId,
    local: Option<&str>,
    proxy_url: Option<&str>,
) -> Result<String, String> {
    let Some((owner, repo)) = github_repo_for(id) else {
        return Err("component has no release notes source".to_owned());
    };
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=30");
    let releases: Vec<GithubRelease> = request_json(&url, proxy_url)?;
    let local_norm = local
        .map(normalize_version)
        .filter(|value| !value.is_empty());
    let mut selected = Vec::new();
    for release in releases {
        if release.draft.unwrap_or(false) || release.prerelease.unwrap_or(false) {
            continue;
        }
        if local_norm
            .as_ref()
            .is_some_and(|local| normalize_version(&release.tag_name) == *local)
        {
            break;
        }
        selected.push(release);
        if local_norm.is_none() || selected.len() >= 12 {
            break;
        }
    }
    if selected.is_empty() {
        return Err("no release notes in range".to_owned());
    }
    let mut markdown = String::new();
    for release in selected {
        if !markdown.trim().is_empty() {
            markdown.push_str("\n\n---\n\n");
        }
        let title = release
            .name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(release.tag_name.as_str());
        markdown.push_str("## ");
        markdown.push_str(title);
        markdown.push_str("\n\n");
        markdown.push_str(
            release
                .body
                .as_deref()
                .filter(|body| !body.trim().is_empty())
                .unwrap_or("No release notes."),
        );
    }
    Ok(markdown)
}

fn build_release_notes_markdown(
    _id: ManagedComponentId,
    _local: Option<&str>,
    remote: &RemoteRelease,
) -> String {
    let title = remote
        .name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(remote.tag.as_str());
    let body = remote
        .notes_markdown
        .as_deref()
        .filter(|body| !body.trim().is_empty())
        .unwrap_or("No release notes.");
    format!("## {title}\n\n{body}")
}

fn component_update_comparison_version(
    id: ManagedComponentId,
    local_version: Option<&str>,
    release_identity: &InstalledReleaseIdentity,
) -> Option<String> {
    match release_identity {
        InstalledReleaseIdentity::Verified { tag, .. } => Some(tag.clone()),
        InstalledReleaseIdentity::NotRecorded | InstalledReleaseIdentity::FingerprintChanged => {
            component_release_tag_can_be_inferred_from_local_version(id)
                .then(|| local_version.map(str::to_owned))
                .flatten()
        }
    }
}

fn remote_release_matches_comparison(
    comparison_version: Option<&str>,
    release_identity: &InstalledReleaseIdentity,
    remote: &RemoteRelease,
) -> bool {
    if !versions_equal(comparison_version, Some(remote.tag.as_str())) {
        return false;
    }

    let InstalledReleaseIdentity::Verified {
        asset_id,
        asset_name,
        asset_size,
        asset_updated_at,
        ..
    } = release_identity
    else {
        return true;
    };

    let Some(remote_asset) = remote.asset.as_ref() else {
        return true;
    };

    remote_asset.name == *asset_name
        && optional_identity_value_matches(*asset_id, remote_asset.id)
        && optional_identity_value_matches(*asset_size, remote_asset.size)
        && optional_identity_value_matches(
            asset_updated_at.as_deref(),
            remote_asset.updated_at.as_deref(),
        )
}

fn try_reanchor_external_component(
    id: ManagedComponentId,
    ownership: ComponentOwnership,
    local_version: Option<&str>,
    release_identity: &InstalledReleaseIdentity,
    remote: &RemoteRelease,
) -> Result<bool, String> {
    if id == ManagedComponentId::App
        || ownership != ComponentOwnership::ManagedPortable
        || !release_identity_needs_reanchor(release_identity)
        || !remote_release_matches_local_probe(id, local_version, remote)
    {
        return Ok(false);
    }

    let Some(asset) = remote.asset.as_ref() else {
        return Ok(false);
    };
    if !release_asset_has_complete_resume_identity(asset) {
        return Ok(false);
    }
    write_installed_component_record(id, remote, asset)?;
    Ok(true)
}

fn release_identity_needs_reanchor(identity: &InstalledReleaseIdentity) -> bool {
    matches!(
        identity,
        InstalledReleaseIdentity::NotRecorded | InstalledReleaseIdentity::FingerprintChanged
    )
}

fn remote_release_matches_local_probe(
    id: ManagedComponentId,
    local_version: Option<&str>,
    remote: &RemoteRelease,
) -> bool {
    if versions_equal(local_version, Some(remote.tag.as_str())) {
        return true;
    }

    let remote_display = remote_release_display_version(id, remote);
    if remote_display != "-" && versions_equal(local_version, Some(remote_display.as_str())) {
        return true;
    }

    let Some(local_display) = local_component_display_version(id, local_version) else {
        return false;
    };
    remote_display != "-" && versions_equal(Some(&local_display), Some(remote_display.as_str()))
}

fn optional_identity_value_matches<T: PartialEq>(installed: Option<T>, remote: Option<T>) -> bool {
    match (installed, remote) {
        (Some(installed), Some(remote)) => installed == remote,
        (None, Some(_)) => false,
        _ => true,
    }
}

fn component_release_tag_can_be_inferred_from_local_version(id: ManagedComponentId) -> bool {
    matches!(
        id,
        ManagedComponentId::App
            | ManagedComponentId::YtDlp
            | ManagedComponentId::Deno
            | ManagedComponentId::Aria2c
    )
}

fn portable_unknown_release_identity_message(identity: &InstalledReleaseIdentity) -> String {
    match identity {
        InstalledReleaseIdentity::FingerprintChanged => {
            "local file differs from the v2 installed release; update to re-anchor".to_owned()
        }
        InstalledReleaseIdentity::NotRecorded => {
            "release identity is not recorded; update to anchor this portable tool".to_owned()
        }
        InstalledReleaseIdentity::Verified { .. } => "update available".to_owned(),
    }
}

fn installed_release_identity(id: ManagedComponentId) -> InstalledReleaseIdentity {
    let Some(record) = installed_component_record(id) else {
        return InstalledReleaseIdentity::NotRecorded;
    };
    let Some(path) = managed_component_executable_path(id) else {
        return InstalledReleaseIdentity::NotRecorded;
    };
    if installed_record_matches_current_file(&record, &path) {
        InstalledReleaseIdentity::Verified {
            tag: record.release_tag,
            asset_id: record.asset_id,
            asset_name: record.asset_name,
            asset_size: record.asset_size,
            asset_updated_at: record.asset_updated_at,
        }
    } else {
        InstalledReleaseIdentity::FingerprintChanged
    }
}

fn installed_component_record(id: ManagedComponentId) -> Option<InstalledComponentRecord> {
    read_installed_component_manifest()
        .entries
        .into_iter()
        .find(|entry| entry.id == id)
}

fn read_installed_component_manifest() -> InstalledComponentManifest {
    read_yaml_file(&installed_component_manifest_path()).unwrap_or_default()
}

fn save_installed_component_manifest(manifest: &InstalledComponentManifest) -> Result<(), String> {
    write_yaml_file(&installed_component_manifest_path(), manifest)
}

fn write_installed_component_record(
    id: ManagedComponentId,
    remote: &RemoteRelease,
    asset: &ReleaseAsset,
) -> Result<(), String> {
    let _manifest_guard = installed_component_manifest_lock()
        .lock()
        .map_err(|_| "installed component manifest lock was poisoned".to_owned())?;
    let Some((owner, repo)) = github_repo_for(id) else {
        return Err(format!("{} has no GitHub release source", id.label()));
    };
    let path = managed_component_executable_path(id)
        .ok_or_else(|| format!("{} is not a managed portable tool", id.label()))?;
    let fingerprint = file_fingerprint(&path)?;
    let record = InstalledComponentRecord {
        id,
        repo_owner: owner.to_owned(),
        repo_name: repo.to_owned(),
        release_tag: installed_component_record_release_tag(id, remote),
        release_url: remote.url.clone(),
        asset_id: asset.id,
        asset_name: asset.name.clone(),
        asset_url: asset.url.clone(),
        asset_size: asset.size,
        asset_updated_at: asset.updated_at.clone(),
        executable_path: path,
        executable_size: fingerprint.size,
        executable_modified_unix: fingerprint.modified_unix,
        executable_hash_fnv1a64: fingerprint.hash_fnv1a64,
        reported_version: local_component_version(id),
        installed_at_unix: now_unix(),
    };

    let mut manifest = read_installed_component_manifest();
    manifest.entries.retain(|entry| entry.id != id);
    manifest.entries.push(record);
    save_installed_component_manifest(&manifest)
}

fn installed_component_manifest_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn installed_component_record_release_tag(
    id: ManagedComponentId,
    remote: &RemoteRelease,
) -> String {
    if remote.tag == "latest" && component_release_tag_can_be_inferred_from_local_version(id) {
        if let Some(version) =
            local_component_version(id).filter(|version| !version.trim().is_empty())
        {
            return version;
        }
    }

    remote.tag.clone()
}

fn installed_record_matches_current_file(record: &InstalledComponentRecord, path: &Path) -> bool {
    file_fingerprint(path).is_ok_and(|fingerprint| {
        fingerprint.size == record.executable_size
            && fingerprint.hash_fnv1a64 == record.executable_hash_fnv1a64
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileFingerprint {
    size: u64,
    modified_unix: Option<u64>,
    hash_fnv1a64: String,
}

fn file_fingerprint(path: &Path) -> Result<FileFingerprint, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Could not read metadata for {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("{} is not a file", path.display()));
    }

    let mut file =
        File::open(path).map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        for byte in &buffer[..read] {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    Ok(FileFingerprint {
        size: metadata.len(),
        modified_unix: metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs()),
        hash_fnv1a64: format!("{hash:016x}"),
    })
}

fn managed_component_executable_path(id: ManagedComponentId) -> Option<PathBuf> {
    id.as_dependency_tool().map(tool_path)
}

fn versions_equal(local: Option<&str>, remote: Option<&str>) -> bool {
    let Some(local) = local.map(normalize_version) else {
        return false;
    };
    let Some(remote) = remote.map(normalize_version) else {
        return false;
    };
    !local.is_empty() && local == remote
}

fn normalize_version(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('v')
        .trim_start_matches("release-")
        .to_ascii_lowercase()
}

fn component_ownership(id: ManagedComponentId) -> ComponentOwnership {
    if id == ManagedComponentId::App {
        return ComponentOwnership::ManagedPortable;
    }
    let Some(tool) = id.as_dependency_tool() else {
        return ComponentOwnership::Unknown;
    };
    let path = tool_path(tool);
    if !tool_install::dependency_tool_is_available(tool, &path.display().to_string()) {
        return ComponentOwnership::Missing;
    }
    let root = canonical_or_original(portable_root_dir());
    let resolved = canonical_or_original(path);
    if resolved.starts_with(root.join("tools")) {
        ComponentOwnership::ManagedPortable
    } else {
        ComponentOwnership::External
    }
}

fn local_component_version(id: ManagedComponentId) -> Option<String> {
    match id {
        ManagedComponentId::App => current_app_build_version(),
        ManagedComponentId::YtDlp => {
            command_first_line(tool_path(DependencyTool::YtDlp), &["--version"])
        }
        ManagedComponentId::Ffmpeg => {
            command_first_line(tool_path(DependencyTool::Ffmpeg), &["-version"])
                .and_then(|line| line.split_whitespace().nth(2).map(str::to_owned))
        }
        ManagedComponentId::Deno => {
            command_first_line(tool_path(DependencyTool::Deno), &["--version"])
                .and_then(|line| line.split_whitespace().nth(1).map(str::to_owned))
        }
        ManagedComponentId::Aria2c => {
            command_first_line(tool_path(DependencyTool::Aria2c), &["--version"])
                .and_then(|line| line.split_whitespace().nth(2).map(str::to_owned))
        }
    }
}

fn current_app_build_version() -> Option<String> {
    option_env!("YT_DLP_GUI_BUILD_DATE")
        .map(str::trim)
        .filter(|version| !version.is_empty())
        .map(str::to_owned)
        .or_else(installed_app_version)
}

fn command_first_line(path: PathBuf, args: &[&str]) -> Option<String> {
    if !path.is_file() {
        return None;
    }
    let mut command = Command::new(path);
    configure_hidden_command(&mut command);
    let output = command.args(args).output().ok()?;
    let text = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr).to_string()
    } else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

fn spawn_hidden(mut command: Command) -> Result<std::process::Child, std::io::Error> {
    configure_hidden_command(&mut command);
    command.spawn()
}

fn configure_hidden_command(command: &mut Command) {
    #[cfg(windows)]
    {
        // CREATE_NO_WINDOW. Required for console tools launched from a
        // windows-subsystem GUI; otherwise version checks flash a console.
        command.creation_flags(0x0800_0000);
    }

    #[cfg(not(windows))]
    {
        let _ = command;
    }
}

fn download_asset_resumable(
    id: ManagedComponentId,
    asset: &ReleaseAsset,
    proxy_url: Option<&str>,
    mut progress: impl FnMut(DownloadAssetProgress),
) -> Result<DownloadedAsset, String> {
    let mut last_error = None;
    for attempt_index in 0..DOWNLOAD_RETRY_ATTEMPTS {
        match download_asset_resumable_once(id, asset, proxy_url, &mut progress) {
            Ok(downloaded) => return Ok(downloaded),
            Err(error) => {
                last_error = Some(error);
                if attempt_index + 1 < DOWNLOAD_RETRY_ATTEMPTS {
                    thread::sleep(Duration::from_millis(
                        DOWNLOAD_RETRY_BACKOFF_BASE_MS * (attempt_index as u64 + 1),
                    ));
                }
            }
        }
    }

    Err(format!(
        "Could not download {} after {} attempts: {}",
        asset.name,
        DOWNLOAD_RETRY_ATTEMPTS,
        last_error.unwrap_or_else(|| "unknown download error".to_owned())
    ))
}

fn download_asset_resumable_once(
    id: ManagedComponentId,
    asset: &ReleaseAsset,
    proxy_url: Option<&str>,
    progress: &mut impl FnMut(DownloadAssetProgress),
) -> Result<DownloadedAsset, String> {
    let dir = update_download_dir().join(id.dir_name());
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Could not create {}: {error}", dir.display()))?;
    let final_path = dir.join(&asset.name);
    let part_path = dir.join(format!("{}.part", asset.name));
    let part_metadata_path = dir.join(format!("{}.part.yaml", asset.name));
    reset_stale_download_part(asset, &part_path, &part_metadata_path)?;
    let existing = part_path
        .metadata()
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    let mut response = http_get(&asset.url, proxy_url, (existing > 0).then_some(existing))?;
    let append = existing > 0 && response.status == 206;
    let already = if append { existing } else { 0 };
    let total = asset
        .size
        .or_else(|| response.content_length.map(|len| len + already));
    progress(DownloadAssetProgress {
        percent: total.map(|_| 0),
        total_size_bytes: total,
    });

    let mut file = if append {
        OpenOptions::new()
            .append(true)
            .open(&part_path)
            .map_err(|error| format!("Could not resume {}: {error}", part_path.display()))?
    } else {
        write_download_part_metadata(asset, &part_metadata_path)?;
        File::create(&part_path)
            .map_err(|error| format!("Could not create {}: {error}", part_path.display()))?
    };

    let mut downloaded = already;
    let mut buffer = [0u8; DOWNLOAD_BUFFER];
    let mut last_percent = None;
    loop {
        let read = response
            .reader
            .read(&mut buffer)
            .map_err(|error| format!("Could not download {}: {error}", asset.url))?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|error| format!("Could not write {}: {error}", part_path.display()))?;
        downloaded += read as u64;
        let percent = total
            .filter(|total| *total > 0)
            .map(|total| ((downloaded.saturating_mul(100)) / total).min(100) as u8);
        if percent != last_percent {
            progress(DownloadAssetProgress {
                percent,
                total_size_bytes: total,
            });
            last_percent = percent;
        }
    }
    file.flush()
        .map_err(|error| format!("Could not flush {}: {error}", part_path.display()))?;
    if let Some(expected) = total {
        if downloaded != expected {
            if downloaded > expected {
                let _ = fs::remove_file(&part_path);
                let _ = fs::remove_file(&part_metadata_path);
            }
            return Err(format!(
                "Downloaded size mismatch for {}: expected {} bytes, got {} bytes",
                asset.name, expected, downloaded
            ));
        }
    }
    if let Err(error) = verify_downloaded_asset_checksum(asset, &part_path) {
        let _ = fs::remove_file(&part_path);
        let _ = fs::remove_file(&part_metadata_path);
        return Err(error);
    }
    fs::rename(&part_path, &final_path)
        .or_else(|_| {
            let _ = fs::remove_file(&final_path);
            fs::rename(&part_path, &final_path)
        })
        .map_err(|error| format!("Could not finalize {}: {error}", final_path.display()))?;
    let _ = fs::remove_file(&part_metadata_path);
    let size_bytes = fs::metadata(&final_path)
        .ok()
        .map(|metadata| metadata.len());
    Ok(DownloadedAsset {
        path: final_path,
        file_name: asset.name.clone(),
        size_bytes,
    })
}

fn verify_downloaded_asset_checksum(asset: &ReleaseAsset, path: &Path) -> Result<(), String> {
    let Some(expected) = asset
        .checksum_sha256
        .as_deref()
        .map(str::trim)
        .filter(|value| is_sha256_hex(value))
    else {
        return Ok(());
    };
    let actual = sha256_hex_file(path)?;
    if actual.eq_ignore_ascii_case(expected) {
        return Ok(());
    }

    Err(format!(
        "SHA256 mismatch for {}: expected {}, got {}",
        asset.name, expected, actual
    ))
}

fn reset_stale_download_part(
    asset: &ReleaseAsset,
    part_path: &Path,
    metadata_path: &Path,
) -> Result<(), String> {
    if !part_path.is_file() {
        let _ = fs::remove_file(metadata_path);
        return Ok(());
    }

    if !release_asset_has_complete_resume_identity(asset) {
        fs::remove_file(part_path).map_err(|error| {
            format!(
                "Could not remove non-resumable {}: {error}",
                part_path.display()
            )
        })?;
        let _ = fs::remove_file(metadata_path);
        return Ok(());
    }

    if download_part_metadata_matches(asset, metadata_path) {
        return Ok(());
    }

    fs::remove_file(part_path)
        .map_err(|error| format!("Could not remove stale {}: {error}", part_path.display()))?;
    let _ = fs::remove_file(metadata_path);
    Ok(())
}

fn release_asset_has_complete_resume_identity(asset: &ReleaseAsset) -> bool {
    asset.id.is_some() && asset.size.is_some() && asset.updated_at.is_some()
}

fn write_download_part_metadata(asset: &ReleaseAsset, path: &Path) -> Result<(), String> {
    let metadata = DownloadPartMetadata {
        asset_id: asset.id,
        asset_name: asset.name.clone(),
        asset_url: asset.url.clone(),
        asset_size: asset.size,
        asset_updated_at: asset.updated_at.clone(),
        asset_checksum_sha256: asset.checksum_sha256.clone(),
    };
    write_yaml_file(path, &metadata)
}

fn download_part_metadata_matches(asset: &ReleaseAsset, path: &Path) -> bool {
    let Some(metadata) = read_yaml_file::<DownloadPartMetadata>(path) else {
        return false;
    };
    metadata.asset_id == asset.id
        && metadata.asset_name == asset.name
        && metadata.asset_url == asset.url
        && metadata.asset_size == asset.size
        && metadata.asset_updated_at == asset.updated_at
        && metadata.asset_checksum_sha256 == asset.checksum_sha256
}

fn tool_apply_stage_message(downloaded: &DownloadedAsset) -> &'static str {
    if downloaded.file_name.to_ascii_lowercase().ends_with(".zip") {
        "tool_install.stage.extracting"
    } else {
        "tool_install.stage.installing"
    }
}

fn stage_app_update(
    downloaded: &DownloadedAsset,
    remote: &RemoteRelease,
) -> Result<PathBuf, String> {
    let staged_dir = update_staged_dir().join("app");
    reset_dir(&staged_dir)?;
    let staged_exe = staged_dir.join(current_exe_file_name());
    if downloaded.file_name.to_ascii_lowercase().ends_with(".zip") {
        extract_zip(&downloaded.path, &staged_dir)?;
        let found = find_file_recursive(&staged_dir, &current_exe_file_name())
            .or_else(|| find_file_recursive(&staged_dir, "yt-dlp-gui.exe"))
            .ok_or_else(|| "new app executable not found in release archive".to_owned())?;
        if found != staged_exe {
            fs::copy(&found, &staged_exe)
                .map_err(|error| format!("Could not stage app executable: {error}"))?;
        }
    } else {
        fs::copy(&downloaded.path, &staged_exe)
            .map_err(|error| format!("Could not stage app executable: {error}"))?;
    }
    if !staged_exe.is_file() {
        return Err("staged app executable was not created".to_owned());
    }
    let _ = remote;
    Ok(staged_exe)
}

fn write_pending_app_manifest(remote: &RemoteRelease, staged_exe: PathBuf) -> Result<(), String> {
    let target_exe = std::env::current_exe()
        .map_err(|error| format!("Could not resolve current exe: {error}"))?;
    let backup_exe = update_backup_dir().join(format!(
        "{}.old.exe",
        target_exe
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("yt-dlp-gui")
    ));
    let manifest = PendingAppUpdateManifest {
        state: PendingAppUpdateState::DownloadedStaged,
        version: remote.tag.clone(),
        staged_exe,
        target_exe,
        backup_exe,
        release_url: remote.url.clone(),
        downloaded_at_unix: now_unix(),
        attempt_count: 0,
        last_error: None,
    };
    write_pending_manifest(&manifest)
}

fn apply_tool_update(
    id: ManagedComponentId,
    downloaded: &DownloadedAsset,
    mut progress: impl FnMut(Option<u8>),
) -> Result<(), String> {
    let tool = id
        .as_dependency_tool()
        .ok_or_else(|| "component is not a dependency tool".to_owned())?;
    let install_dir = portable_root_dir()
        .join("tools")
        .join(tool.install_dir_name());
    fs::create_dir_all(&install_dir)
        .map_err(|error| format!("Could not create {}: {error}", install_dir.display()))?;

    let apply_dir = update_staged_dir().join(id.dir_name()).join("apply");
    reset_dir(&apply_dir)?;
    let required_files = required_tool_file_names(id, tool);

    if downloaded.file_name.to_ascii_lowercase().ends_with(".zip") {
        let extract_dir = update_staged_dir().join(id.dir_name());
        reset_dir(&extract_dir)?;
        extract_zip_with_progress(&downloaded.path, &extract_dir, |percent| {
            progress(percent);
        })?;
        for file_name in required_files.iter().copied() {
            stage_required_file(&extract_dir, file_name, &apply_dir.join(file_name))?;
        }
    } else {
        progress(Some(0));
        let destination = apply_dir.join(tool.executable_name());
        copy_file_to_path(&downloaded.path, &destination)?;
    }

    promote_staged_tool_files(id, &apply_dir, &install_dir, &required_files)?;
    progress(Some(100));
    Ok(())
}

fn required_tool_file_names(id: ManagedComponentId, tool: DependencyTool) -> Vec<&'static str> {
    match id {
        ManagedComponentId::Ffmpeg => vec!["ffmpeg.exe", "ffprobe.exe"],
        ManagedComponentId::Deno => vec!["deno.exe"],
        ManagedComponentId::Aria2c => vec!["aria2c.exe"],
        ManagedComponentId::YtDlp => vec!["yt-dlp.exe"],
        ManagedComponentId::App => vec![tool.executable_name()],
    }
}

fn stage_required_file(root: &Path, file_name: &str, destination: &Path) -> Result<(), String> {
    let source = find_file_recursive(root, file_name)
        .ok_or_else(|| format!("{file_name} not found in staged archive"))?;
    copy_file_to_path(&source, destination)
}

fn copy_file_to_path(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }
    fs::copy(&source, destination).map_err(|error| {
        format!(
            "Could not install {} to {}: {error}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn promote_staged_tool_files(
    id: ManagedComponentId,
    staged_dir: &Path,
    install_dir: &Path,
    file_names: &[&str],
) -> Result<(), String> {
    for file_name in file_names {
        let source = staged_dir.join(file_name);
        if !source.is_file() {
            return Err(format!(
                "staged tool file was not created: {}",
                source.display()
            ));
        }
    }

    let backup_dir = staged_dir.join("backup-existing");
    reset_dir(&backup_dir)?;
    let mut backed_up = Vec::new();
    for file_name in file_names {
        let destination = install_dir.join(file_name);
        if destination.is_file() {
            let backup = backup_dir.join(file_name);
            copy_file_to_path(&destination, &backup)?;
            backed_up.push((destination, Some(backup)));
        } else {
            backed_up.push((destination, None));
        }
    }

    for file_name in file_names {
        let source = staged_dir.join(file_name);
        let destination = install_dir.join(file_name);
        if let Err(error) = copy_file_to_path(&source, &destination) {
            rollback_promoted_tool_files(&backed_up);
            return Err(error);
        }
    }

    if let Err(error) = verify_managed_tool_installation(id) {
        rollback_promoted_tool_files(&backed_up);
        return Err(error);
    }

    Ok(())
}

fn verify_managed_tool_installation(id: ManagedComponentId) -> Result<(), String> {
    let Some(tool) = id.as_dependency_tool() else {
        return Err("component is not a dependency tool".to_owned());
    };
    let path = tool_path(tool);
    if !tool_install::dependency_tool_is_available(tool, &path.display().to_string()) {
        return Err(format!(
            "{} executable did not pass availability check",
            id.label()
        ));
    }
    let Some(version) = local_component_version(id).filter(|version| !version.trim().is_empty())
    else {
        return Err(format!(
            "{} executable did not report a usable version",
            id.label()
        ));
    };

    if id == ManagedComponentId::Ffmpeg {
        let ffprobe_path = tool_path(DependencyTool::Ffmpeg)
            .parent()
            .map(|parent| parent.join("ffprobe.exe"))
            .unwrap_or_else(|| {
                portable_root_dir()
                    .join("tools")
                    .join("ffmpeg")
                    .join("ffprobe.exe")
            });
        if !ffprobe_path.is_file() {
            return Err("FFmpeg install is incomplete: ffprobe.exe was not found".to_owned());
        }
        let Some(ffprobe_version) = command_first_line(ffprobe_path, &["-version"])
            .and_then(|line| line.split_whitespace().nth(2).map(str::to_owned))
            .filter(|version| !version.trim().is_empty())
        else {
            return Err(
                "FFmpeg install is incomplete: ffprobe.exe did not report a version".to_owned(),
            );
        };
        if ffmpeg_build_date_display_version(Some(&version))
            != ffmpeg_build_date_display_version(Some(&ffprobe_version))
        {
            return Err("FFmpeg install is inconsistent: ffmpeg.exe and ffprobe.exe have different build dates".to_owned());
        }
    }

    Ok(())
}

fn rollback_promoted_tool_files(backed_up: &[(PathBuf, Option<PathBuf>)]) {
    for (destination, backup) in backed_up {
        if let Some(backup) = backup {
            if backup.is_file() {
                let _ = copy_file_to_path(backup, destination);
            }
        } else {
            let _ = fs::remove_file(destination);
        }
    }
}

fn prepare_update_runner() -> Result<PathBuf, String> {
    let source = std::env::current_exe()
        .map_err(|error| format!("Could not resolve updater runner executable: {error}"))?;
    if !source.is_file() {
        return Err(format!(
            "Updater runner source executable was not found: {}",
            source.display()
        ));
    }

    let runner_dir = update_runner_dir();
    reset_dir(&runner_dir)?;
    let runner_exe = runner_dir.join(
        source
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("yt-dlp-gui.exe")),
    );
    fs::copy(&source, &runner_exe).map_err(|error| {
        format!(
            "Could not prepare updater runner {} from {}: {error}",
            runner_exe.display(),
            source.display()
        )
    })?;
    Ok(runner_exe)
}

pub fn launch_pending_app_update(restart: bool) -> Result<(), String> {
    let mut manifest = pending_app_update_manifest()
        .ok_or_else(|| "No pending app update was found.".to_owned())?;
    if !manifest.staged_exe.is_file() {
        return Err(format!(
            "Staged executable was not found: {}",
            manifest.staged_exe.display()
        ));
    }
    if other_registered_app_instances() > 0 {
        return Err(
            "Another yt-dlp-gui instance is still running. Close it before applying the update."
                .to_owned(),
        );
    }
    manifest.state = PendingAppUpdateState::ApplyRequested;
    manifest.attempt_count = manifest.attempt_count.saturating_add(1);
    write_pending_manifest(&manifest)?;

    let runner_exe = prepare_update_runner()?;
    let after_apply = if restart { "restart" } else { "exit" };
    let mut command = Command::new(&runner_exe);
    command
        .arg("--apply-update")
        .arg("--parent-pid")
        .arg(std::process::id().to_string())
        .arg("--target-exe")
        .arg(&manifest.target_exe)
        .arg("--staged-exe")
        .arg(&manifest.staged_exe)
        .arg("--backup-exe")
        .arg(&manifest.backup_exe)
        .arg("--manifest")
        .arg(pending_app_update_manifest_path())
        .arg("--after-apply")
        .arg(after_apply);
    spawn_hidden(command).map_err(|error| format!("Could not launch staged updater: {error}"))?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct ApplyUpdateArgs {
    pub parent_pid: u32,
    pub target_exe: PathBuf,
    pub staged_exe: PathBuf,
    pub backup_exe: PathBuf,
    pub manifest: PathBuf,
    pub restart: bool,
}

pub fn apply_update_args_requested() -> bool {
    std::env::args_os()
        .skip(1)
        .any(|arg| arg.to_string_lossy() == "--apply-update")
}

pub fn parse_apply_update_args() -> Option<ApplyUpdateArgs> {
    let mut args = std::env::args_os().skip(1);
    let mut saw_apply = false;
    let mut parent_pid = None;
    let mut target_exe = None;
    let mut staged_exe = None;
    let mut backup_exe = None;
    let mut manifest = None;
    let mut restart = true;

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--apply-update" => saw_apply = true,
            "--parent-pid" => {
                parent_pid = args
                    .next()
                    .and_then(|value| value.to_string_lossy().parse::<u32>().ok())
            }
            "--target-exe" => target_exe = args.next().map(PathBuf::from),
            "--staged-exe" => staged_exe = args.next().map(PathBuf::from),
            "--backup-exe" => backup_exe = args.next().map(PathBuf::from),
            "--manifest" => manifest = args.next().map(PathBuf::from),
            "--after-apply" => {
                restart = args
                    .next()
                    .map(|value| value.to_string_lossy().eq_ignore_ascii_case("restart"))
                    .unwrap_or(true);
            }
            _ => {}
        }
    }

    saw_apply.then_some(ApplyUpdateArgs {
        parent_pid: parent_pid?,
        target_exe: target_exe?,
        staged_exe: staged_exe?,
        backup_exe: backup_exe?,
        manifest: manifest?,
        restart,
    })
}

pub fn resume_pending_app_update_on_launch() -> Result<bool, String> {
    let Some(manifest) = pending_app_update_manifest() else {
        return Ok(false);
    };
    if manifest.state != PendingAppUpdateState::ApplyRequested {
        return Ok(false);
    }
    if !manifest.staged_exe.is_file() {
        return Ok(false);
    }
    let runner_exe = prepare_update_runner()?;
    let mut command = Command::new(&runner_exe);
    command
        .arg("--apply-update")
        .arg("--parent-pid")
        .arg(std::process::id().to_string())
        .arg("--target-exe")
        .arg(&manifest.target_exe)
        .arg("--staged-exe")
        .arg(&manifest.staged_exe)
        .arg("--backup-exe")
        .arg(&manifest.backup_exe)
        .arg("--manifest")
        .arg(pending_app_update_manifest_path())
        .arg("--after-apply")
        .arg("restart");
    spawn_hidden(command)
        .map_err(|error| format!("Could not resume pending app update: {error}"))?;
    Ok(true)
}

pub fn run_apply_update(args: ApplyUpdateArgs) -> Result<(), String> {
    wait_for_parent_exit(args.parent_pid);
    let mut manifest = read_manifest_from(&args.manifest).unwrap_or(PendingAppUpdateManifest {
        state: PendingAppUpdateState::Applying,
        version: String::new(),
        staged_exe: args.staged_exe.clone(),
        target_exe: args.target_exe.clone(),
        backup_exe: args.backup_exe.clone(),
        release_url: None,
        downloaded_at_unix: now_unix(),
        attempt_count: 1,
        last_error: None,
    });
    manifest.state = PendingAppUpdateState::Applying;
    write_manifest_to(&args.manifest, &manifest)?;

    let replace_started = SystemTime::now();
    loop {
        match replace_target_executable(&args.target_exe, &args.staged_exe, &args.backup_exe) {
            Ok(()) => break,
            Err(error) => {
                if replace_started.elapsed().unwrap_or_default() >= APPLY_WAIT_TIMEOUT {
                    manifest.state = PendingAppUpdateState::Failed;
                    manifest.last_error = Some(error.clone());
                    let _ = write_manifest_to(&args.manifest, &manifest);
                    return Err(error);
                }
                thread::sleep(APPLY_POLL_INTERVAL);
            }
        }
    }

    manifest.state = PendingAppUpdateState::Applied;
    manifest.last_error = None;
    write_manifest_to(&args.manifest, &manifest)?;
    if !manifest.version.trim().is_empty() {
        let _ = write_installed_app_version(&manifest.version);
    }

    if args.restart {
        let mut command = Command::new(&args.target_exe);
        if let Some(parent) = args.target_exe.parent() {
            command.current_dir(parent);
        }
        spawn_hidden(command).map_err(|error| format!("Could not restart updated app: {error}"))?;
    }
    Ok(())
}

fn replace_target_executable(
    target_exe: &Path,
    staged_exe: &Path,
    backup_exe: &Path,
) -> Result<(), String> {
    if let Some(parent) = backup_exe.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create backup folder: {error}"))?;
    }
    if target_exe.is_file() {
        let _ = fs::remove_file(backup_exe);
        fs::rename(target_exe, backup_exe)
            .or_else(|_| {
                fs::copy(target_exe, backup_exe)?;
                fs::remove_file(target_exe)
            })
            .map_err(|error| format!("Could not backup current executable: {error}"))?;
    }
    fs::copy(staged_exe, target_exe).map_err(|error| {
        let _ = fs::copy(backup_exe, target_exe);
        format!("Could not install updated executable: {error}")
    })?;
    Ok(())
}

fn wait_for_parent_exit(_pid: u32) {
    // Portable, dependency-free fallback. On Windows the file lock is the final guard;
    // this delay gives the GUI process time to exit after spawning the staged updater.
    let started = SystemTime::now();
    loop {
        if started.elapsed().unwrap_or_default() >= APPLY_WAIT_TIMEOUT {
            break;
        }
        thread::sleep(APPLY_POLL_INTERVAL);
        // The parent process check intentionally stays conservative here because the
        // project avoids adding a new permanent updater binary or extra platform deps.
        break;
    }
}

pub fn cleanup_applied_update() {
    if let Some(manifest) = pending_app_update_manifest() {
        if manifest.state == PendingAppUpdateState::Applied {
            let _ = fs::remove_dir_all(update_staged_dir().join("app"));
            let _ = fs::remove_dir_all(update_runner_dir());
            clear_pending_app_update_manifest();
        }
    }
}

fn installed_app_version() -> Option<String> {
    let manifest =
        read_yaml_file::<InstalledAppVersionManifest>(&installed_app_version_manifest_path())?;
    let version = manifest.version.trim();
    (!version.is_empty()).then(|| version.to_owned())
}

fn write_installed_app_version(version: &str) -> Result<(), String> {
    let manifest = InstalledAppVersionManifest {
        version: version.to_owned(),
        installed_at_unix: now_unix(),
    };
    write_yaml_file(&installed_app_version_manifest_path(), &manifest)
}

fn write_pending_manifest(manifest: &PendingAppUpdateManifest) -> Result<(), String> {
    write_manifest_to(&pending_app_update_manifest_path(), manifest)
}

fn read_manifest_from(path: &Path) -> Option<PendingAppUpdateManifest> {
    read_yaml_file(path)
}

fn write_manifest_to(path: &Path, manifest: &PendingAppUpdateManifest) -> Result<(), String> {
    write_yaml_file(path, manifest)
}

fn request_json<T: for<'de> Deserialize<'de>>(
    url: &str,
    proxy_url: Option<&str>,
) -> Result<T, String> {
    let text = request_text(url, proxy_url)?;
    serde_json::from_str(&text)
        .map_err(|error| format!("Could not parse response from {url}: {error}"))
}

fn request_text(url: &str, proxy_url: Option<&str>) -> Result<String, String> {
    let mut response = http_get(url, proxy_url, None)?;
    let mut text = String::new();
    response
        .reader
        .read_to_string(&mut text)
        .map_err(|error| format!("Could not read response from {url}: {error}"))?;
    Ok(text)
}

fn http_get(
    url: &str,
    proxy_url: Option<&str>,
    range_start: Option<u64>,
) -> Result<HttpResponse, String> {
    let mut builder = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(10)))
        .timeout_recv_response(Some(Duration::from_secs(20)))
        .timeout_recv_body(Some(Duration::from_secs(20)))
        .user_agent(USER_AGENT);
    if let Some(proxy_url) = proxy_url.map(str::trim).filter(|value| !value.is_empty()) {
        let proxy = ureq::Proxy::new(proxy_url)
            .map_err(|error| format!("Invalid proxy URL {proxy_url}: {error}"))?;
        builder = builder.proxy(Some(proxy));
    }

    let agent = builder.build().new_agent();
    let mut request = agent.get(url);
    if let Some(start) = range_start {
        request = request.header("Range", format!("bytes={start}-"));
    }
    let response = request
        .call()
        .map_err(|error| format!("Could not download {url}: {error}"))?;
    let status = response.status().as_u16();
    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    Ok(HttpResponse {
        reader: Box::new(response.into_parts().1.into_reader()),
        content_length,
        status,
    })
}

fn extract_zip(zip_path: &Path, destination: &Path) -> Result<(), String> {
    extract_zip_with_progress(zip_path, destination, |_| {})
}

fn extract_zip_with_progress(
    zip_path: &Path,
    destination: &Path,
    mut progress: impl FnMut(Option<u8>),
) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("Could not create {}: {error}", destination.display()))?;
    let file = File::open(zip_path)
        .map_err(|error| format!("Could not open {}: {error}", zip_path.display()))?;
    let reader = BufReader::new(file);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|error| format!("Could not read zip: {error}"))?;
    let total = archive.len().max(1);
    progress(Some(0));
    for index in 0..archive.len() {
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
            let mut buffer = [0u8; DOWNLOAD_BUFFER];
            loop {
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
        progress(Some((((index + 1) * 100) / total).min(100) as u8));
    }
    Ok(())
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

fn reset_dir(path: &Path) -> Result<(), String> {
    let _ = fs::remove_dir_all(path);
    fs::create_dir_all(path)
        .map_err(|error| format!("Could not create {}: {error}", path.display()))
}

fn tool_path(tool: DependencyTool) -> PathBuf {
    tool_install::resolve_support_path(tool.default_portable_path())
}

fn portable_root_dir() -> PathBuf {
    tool_install::portable_root_dir()
}

fn canonical_or_original(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn current_exe_file_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "yt-dlp-gui.exe".to_owned())
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or_default()
}

fn manifest_cache_dir() -> PathBuf {
    portable_root_dir().join("cache").join("manifest")
}

fn manifest_transient_dir() -> PathBuf {
    manifest_temp_dir()
}

pub(crate) fn manifest_temp_dir() -> PathBuf {
    tool_install::portable_temp_cache_dir().join("manifest")
}

fn update_download_dir() -> PathBuf {
    manifest_transient_dir().join("downloads")
}

fn update_staged_dir() -> PathBuf {
    manifest_transient_dir().join("staged")
}

fn update_runner_dir() -> PathBuf {
    manifest_transient_dir().join("runner")
}

fn update_backup_dir() -> PathBuf {
    manifest_cache_dir().join("backup")
}

fn pending_app_update_manifest_path() -> PathBuf {
    manifest_cache_dir().join("pending_app_update.yaml")
}

fn installed_app_version_manifest_path() -> PathBuf {
    manifest_cache_dir().join("installed_app_version.yaml")
}

fn installed_component_manifest_path() -> PathBuf {
    manifest_cache_dir().join("installed.yaml")
}

fn runtime_instance_dir() -> PathBuf {
    tool_install::portable_temp_cache_dir()
        .join("runtime")
        .join("instances")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffmpeg_without_release_identity_has_no_safe_comparison_version() {
        let identity = InstalledReleaseIdentity::NotRecorded;

        assert_eq!(
            component_update_comparison_version(
                ManagedComponentId::Ffmpeg,
                Some("7.1.1"),
                &identity
            ),
            None
        );
    }

    #[test]
    fn yt_dlp_without_manifest_can_fallback_to_executable_version() {
        let identity = InstalledReleaseIdentity::NotRecorded;

        assert_eq!(
            component_update_comparison_version(
                ManagedComponentId::YtDlp,
                Some("2026.06.02"),
                &identity
            ),
            Some("2026.06.02".to_owned())
        );
    }

    #[test]
    fn direct_latest_asset_fallback_uses_fixed_prepare_tool_assets() {
        let cases = [
            (
                ManagedComponentId::YtDlp,
                YT_DLP_ASSET_NAME,
                YT_DLP_DIRECT_ASSET_URL,
            ),
            (
                ManagedComponentId::Ffmpeg,
                FFMPEG_ASSET_NAME,
                FFMPEG_DIRECT_ASSET_URL,
            ),
            (
                ManagedComponentId::Deno,
                DENO_ASSET_NAME,
                DENO_DIRECT_ASSET_URL,
            ),
        ];

        for (id, expected_name, expected_url) in cases {
            let remote = direct_latest_asset_release(id).expect("core tool fallback");
            let asset = remote.asset.expect("direct fallback asset");

            assert_eq!(asset.name, expected_name);
            assert_eq!(asset.url, expected_url);
            assert_eq!(asset.id, None);
            assert_eq!(asset.size, None);
        }
        assert!(direct_latest_asset_release(ManagedComponentId::App).is_none());
        assert!(direct_latest_asset_release(ManagedComponentId::Aria2c).is_none());
    }

    #[test]
    fn action_start_snapshot_replaces_stale_failed_targets() {
        let mut yt_dlp = ComponentUpdateEntry::new(ManagedComponentId::YtDlp);
        yt_dlp.status = ComponentUpdateStatus::Failed;
        yt_dlp.message = "old failed state".to_owned();
        let mut aria2c = ComponentUpdateEntry::new(ManagedComponentId::Aria2c);
        aria2c.status = ComponentUpdateStatus::Failed;
        aria2c.message = "not part of prepare".to_owned();
        let mut snapshot = ComponentUpdateSnapshot {
            entries: vec![yt_dlp, aria2c],
            selected: None,
            running: true,
            message: "updating yt-dlp".to_owned(),
            checked_at_unix: None,
        };

        mark_action_entries_started(
            &mut snapshot,
            &ComponentUpdateAction::UpdateMany(vec![
                ManagedComponentId::YtDlp,
                ManagedComponentId::Deno,
            ]),
        );

        let yt_dlp = snapshot
            .entry(ManagedComponentId::YtDlp)
            .expect("yt-dlp entry");
        assert_eq!(yt_dlp.status, ComponentUpdateStatus::Checking);
        assert_eq!(yt_dlp.message, "queued");
        assert_eq!(yt_dlp.progress, None);
        let deno = snapshot
            .entry(ManagedComponentId::Deno)
            .expect("deno entry");
        assert_eq!(deno.status, ComponentUpdateStatus::Checking);
        assert_eq!(deno.message, "queued");
        assert_eq!(snapshot.selected, Some(ManagedComponentId::YtDlp));
        assert_eq!(
            snapshot
                .entry(ManagedComponentId::Aria2c)
                .map(|entry| entry.status),
            Some(ComponentUpdateStatus::Failed)
        );
    }

    #[test]
    fn update_all_targets_skip_optional_missing_components() {
        let mut aria2c = ComponentUpdateEntry::new(ManagedComponentId::Aria2c);
        aria2c.ownership = ComponentOwnership::Missing;
        aria2c.status = ComponentUpdateStatus::Missing;
        let mut yt_dlp = ComponentUpdateEntry::new(ManagedComponentId::YtDlp);
        yt_dlp.ownership = ComponentOwnership::Missing;
        yt_dlp.status = ComponentUpdateStatus::Missing;
        let snapshot = ComponentUpdateSnapshot {
            entries: vec![aria2c, yt_dlp],
            selected: None,
            running: false,
            message: String::new(),
            checked_at_unix: None,
        };

        assert_eq!(
            component_update_targets(&snapshot, &ManagedComponentId::ALL, false),
            vec![ManagedComponentId::YtDlp]
        );
        assert_eq!(
            component_update_targets(&snapshot, &[ManagedComponentId::Aria2c], true),
            vec![ManagedComponentId::Aria2c]
        );
    }

    #[test]
    fn local_probe_restores_current_version_for_installed_tool_cache_entry() {
        let mut entry = ComponentUpdateEntry::new(ManagedComponentId::YtDlp);
        entry.ownership = ComponentOwnership::Missing;
        entry.status = ComponentUpdateStatus::Failed;
        entry.message = "old failure".to_owned();

        apply_local_component_probe(
            &mut entry,
            ComponentOwnership::ManagedPortable,
            Some("2026.03.17".to_owned()),
        );

        assert_eq!(entry.ownership, ComponentOwnership::ManagedPortable);
        assert_eq!(entry.local_version, Some("2026.03.17".to_owned()));
        assert_eq!(entry.status, ComponentUpdateStatus::Unknown);
        assert_eq!(entry.message, "not checked");
    }

    #[test]
    fn local_probe_downgrades_stale_up_to_date_for_missing_tool() {
        let mut entry = ComponentUpdateEntry::new(ManagedComponentId::YtDlp);
        entry.ownership = ComponentOwnership::ManagedPortable;
        entry.status = ComponentUpdateStatus::UpToDate;
        entry.local_version = Some("2026.03.17".to_owned());

        apply_local_component_probe(&mut entry, ComponentOwnership::Missing, None);

        assert_eq!(entry.ownership, ComponentOwnership::Missing);
        assert_eq!(entry.local_version, None);
        assert_eq!(entry.status, ComponentUpdateStatus::Missing);
        assert_eq!(entry.message, "not installed");
    }

    #[test]
    fn startup_presence_probe_keeps_cache_for_existing_tool_without_running_it() {
        assert_eq!(
            startup_cached_local_version(
                ManagedComponentId::YtDlp,
                ComponentOwnership::ManagedPortable,
                Some("2026.03.17".to_owned()),
            ),
            Some("2026.03.17".to_owned())
        );
        assert_eq!(
            startup_cached_local_version(
                ManagedComponentId::YtDlp,
                ComponentOwnership::Missing,
                Some("2026.03.17".to_owned()),
            ),
            None
        );
    }

    #[test]
    fn verified_release_identity_overrides_executable_version() {
        let identity = InstalledReleaseIdentity::Verified {
            tag: "autobuild-2026-06-02".to_owned(),
            asset_id: Some(42),
            asset_name: "ffmpeg-master-latest-win64-gpl.zip".to_owned(),
            asset_size: Some(1024),
            asset_updated_at: Some("2026-06-02T00:00:00Z".to_owned()),
        };

        assert_eq!(
            component_update_comparison_version(
                ManagedComponentId::Ffmpeg,
                Some("7.1.1"),
                &identity
            ),
            Some("autobuild-2026-06-02".to_owned())
        );
    }

    #[test]
    fn verified_release_identity_detects_changed_asset_metadata() {
        let identity = InstalledReleaseIdentity::Verified {
            tag: "latest".to_owned(),
            asset_id: Some(42),
            asset_name: "ffmpeg-master-latest-win64-gpl.zip".to_owned(),
            asset_size: Some(1024),
            asset_updated_at: Some("2026-06-02T00:00:00Z".to_owned()),
        };
        let remote = RemoteRelease {
            tag: "latest".to_owned(),
            name: None,
            url: None,
            notes_markdown: None,
            asset: Some(ReleaseAsset {
                id: Some(42),
                name: "ffmpeg-master-latest-win64-gpl.zip".to_owned(),
                url: "https://example.invalid/ffmpeg.zip".to_owned(),
                size: Some(2048),
                updated_at: Some("2026-06-03T00:00:00Z".to_owned()),
                checksum_sha256: None,
            }),
        };

        assert!(!remote_release_matches_comparison(
            Some("latest"),
            &identity,
            &remote
        ));
    }

    #[test]
    fn verified_release_identity_reanchors_when_remote_has_new_metadata() {
        let identity = InstalledReleaseIdentity::Verified {
            tag: "latest".to_owned(),
            asset_id: None,
            asset_name: "ffmpeg-master-latest-win64-gpl.zip".to_owned(),
            asset_size: None,
            asset_updated_at: None,
        };
        let remote = RemoteRelease {
            tag: "latest".to_owned(),
            name: None,
            url: None,
            notes_markdown: None,
            asset: Some(ReleaseAsset {
                id: Some(42),
                name: "ffmpeg-master-latest-win64-gpl.zip".to_owned(),
                url: "https://example.invalid/ffmpeg.zip".to_owned(),
                size: Some(2048),
                updated_at: Some("2026-06-03T00:00:00Z".to_owned()),
                checksum_sha256: None,
            }),
        };

        assert!(!remote_release_matches_comparison(
            Some("latest"),
            &identity,
            &remote
        ));
    }

    #[test]
    fn ffmpeg_latest_release_display_uses_yyyy_mm_dd_build_date() {
        let remote = RemoteRelease {
            tag: "latest".to_owned(),
            name: None,
            url: None,
            notes_markdown: None,
            asset: Some(ReleaseAsset {
                id: Some(42),
                name: "ffmpeg-master-latest-win64-gpl.zip".to_owned(),
                url: "https://example.invalid/ffmpeg.zip".to_owned(),
                size: Some(2048),
                updated_at: Some("2026-06-03T00:00:00Z".to_owned()),
                checksum_sha256: None,
            }),
        };

        assert_eq!(
            remote_release_display_version(ManagedComponentId::Ffmpeg, &remote),
            "2026.06.03"
        );
    }

    #[test]
    fn ffmpeg_latest_release_display_without_metadata_uses_placeholder() {
        let remote = RemoteRelease {
            tag: "latest".to_owned(),
            name: None,
            url: None,
            notes_markdown: None,
            asset: Some(ReleaseAsset {
                id: None,
                name: "ffmpeg-master-latest-win64-gpl.zip".to_owned(),
                url: "https://example.invalid/ffmpeg.zip".to_owned(),
                size: None,
                updated_at: None,
                checksum_sha256: None,
            }),
        };

        assert_eq!(
            remote_release_display_version(ManagedComponentId::Ffmpeg, &remote),
            "-"
        );
    }

    #[test]
    fn sha256_hex_file_matches_known_vector() {
        let dir = std::env::temp_dir().join(format!(
            "yt-dlp-gui-v2-sha256-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("create temp test dir");
        let path = dir.join("abc.txt");
        fs::write(&path, b"abc").expect("write test file");

        assert_eq!(
            sha256_hex_file(&path).expect("hash file"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn checksum_parser_matches_named_asset_line() {
        let text = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef  other.exe\n\
                   ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad *yt-dlp.exe\n";

        assert_eq!(
            parse_sha256_checksum_for_asset(text, "yt-dlp.exe"),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned())
        );
    }

    #[test]
    fn checksum_asset_candidates_ignore_other_platform_sidecars() {
        let assets = vec![
            GithubReleaseAsset {
                id: Some(1),
                name: "deno-aarch64-apple-darwin.zip.sha256sum".to_owned(),
                browser_download_url: "https://example.invalid/apple.sha256sum".to_owned(),
                size: Some(96),
                updated_at: None,
            },
            GithubReleaseAsset {
                id: Some(2),
                name: "deno-x86_64-pc-windows-msvc.zip.sha256sum".to_owned(),
                browser_download_url: "https://example.invalid/windows.sha256sum".to_owned(),
                size: Some(178),
                updated_at: None,
            },
            GithubReleaseAsset {
                id: Some(3),
                name: "deno-x86_64-pc-windows-msvc.zip".to_owned(),
                browser_download_url: "https://example.invalid/windows.zip".to_owned(),
                size: Some(42),
                updated_at: None,
            },
        ];

        let candidates = checksum_assets_for_main_asset(&assets, "deno-x86_64-pc-windows-msvc.zip");

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].name,
            "deno-x86_64-pc-windows-msvc.zip.sha256sum"
        );
    }

    #[test]
    fn direct_fallback_asset_does_not_resume_stale_part_without_identity() {
        let dir = std::env::temp_dir().join(format!(
            "yt-dlp-gui-v2-part-identity-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("create temp test dir");
        let part = dir.join("yt-dlp.exe.part");
        let metadata = dir.join("yt-dlp.exe.part.yaml");
        fs::write(&part, b"partial").expect("write partial file");
        fs::write(&metadata, "{}").expect("write metadata file");
        let asset = direct_latest_asset_release(ManagedComponentId::YtDlp)
            .expect("direct fallback")
            .asset
            .expect("direct fallback asset");

        reset_stale_download_part(&asset, &part, &metadata).expect("reset part");

        assert!(!part.exists());
        assert!(!metadata.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ffmpeg_external_probe_can_match_remote_build_date() {
        let remote = RemoteRelease {
            tag: "latest".to_owned(),
            name: None,
            url: None,
            notes_markdown: None,
            asset: Some(ReleaseAsset {
                id: Some(42),
                name: "ffmpeg-master-latest-win64-gpl.zip".to_owned(),
                url: "https://example.invalid/ffmpeg.zip".to_owned(),
                size: Some(2048),
                updated_at: Some("2026-06-03T00:00:00Z".to_owned()),
                checksum_sha256: None,
            }),
        };

        assert!(remote_release_matches_local_probe(
            ManagedComponentId::Ffmpeg,
            Some("N-124739-gbb5c461a47-20260603"),
            &remote
        ));
    }

    #[test]
    fn file_fingerprint_changes_when_file_is_replaced() {
        let dir = std::env::temp_dir().join(format!(
            "yt-dlp-gui-v2-fingerprint-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("create temp test dir");
        let path = dir.join("tool.exe");

        fs::write(&path, b"first-content").expect("write first test file");
        let first = file_fingerprint(&path).expect("fingerprint first file");

        fs::write(&path, b"second-content").expect("write second test file");
        let second = file_fingerprint(&path).expect("fingerprint second file");

        let _ = fs::remove_dir_all(&dir);
        assert_ne!(first.hash_fnv1a64, second.hash_fnv1a64);
    }
}
