use std::collections::{HashMap, VecDeque, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::i18n::{self, Language, LanguageSelection};

use crate::app::batch_add_worker::{BatchAddEvent, request_batch_add_stop, run_batch_add_worker};
use crate::app::download_worker::{
    DOWNLOAD_CANCELLED_MESSAGE, DownloadEvent, DownloadProgressSlot, request_download_stop,
    run_download_worker,
};
pub use crate::app::format_picker_state::{
    FormatPickerFilters, FormatPickerKind, FormatPickerState, FormatPickerViewMode,
    SubtitlePickerTab,
};
pub use crate::app::metadata::sanitize_file_name_for_windows;
use crate::app::metadata::{
    PlaylistEntrySeed, display_file_stem, extract_chapters, extract_formats,
    extract_requested_filename, extract_requested_ids, extract_subtitle_tracks,
    first_audio_format_id, human_size_bytes, infer_title, normalize_duration_badge_text,
    requested_or_default_format_id, select_best_thumbnail_url, video_resolution_area,
};
use crate::app::post_process_worker::{
    POST_PROCESS_CANCELLED_MESSAGE, PostProcessEvent, request_post_process_stop,
    run_builtin_transcode_worker,
};
use crate::app::transcode_plan::resolve_transcode_plan;
pub use crate::app::queue_status::{ItemTitleVisualState, QueueSummary};
use crate::app::queue_status::{
    QueueSummaryBucket, is_pending_download_state, item_can_enter_download_queue,
    item_latest_download_state, item_summary_bucket, selection_matches_completed,
};
use crate::app::thumbnail_worker::{ThumbnailFetchEvent, run_thumbnail_fetch_worker};
use crate::app::tool_install_worker::{ToolInstallEvent, run_tool_install_worker};
use crate::domain::{
    CompletedSelection, CookiePolicy, DownloadOptions, FormatOption, MediaKind, MetadataState,
    QualityPreset, QueueItem, QueueItemId, SubtitleOption, SubtitleSource, ToolKind, VideoMetadata,
    WorkflowKind, WorkflowRun, WorkflowRunId, WorkflowState,
};
use crate::infrastructure::{
    AppConfig, CacheLocationMode, ConfigFileOption, DependencyTool, DownloadRequest,
    DownloadTargetKind, FileTimeMode, OutputFileActionMode, PrepareAction, PrepareReport,
    PrepareRequirement, PrepareStatus, SerializableCacheLocationMode, ToolInstallCancelHandle,
    ThemeAccentColor, ThemeMode, ToolInstallProgress, ToolInstallStage, ToolPaths, WindowPosition, WindowSize,
    YoutubePlaylistRisk, YoutubeVideoPlaylistMode, available_yt_dlp_config_files,
    classify_youtube_playlist, collect_prepare_report, dependency_tool_exists,
    dependency_tool_is_available, display_output_dir,
    looks_like_playlist_url, normalize_ui_scale_percent, resolve_output_dir,
    send_download_failed_windows_toast, send_download_finished_windows_toast,
    youtube_url_force_single_video, youtube_url_has_video_and_playlist, yt_dlp_configs_dir_display,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTab {
    Prepare,
    Main,
    Advance,
    Options,
    Processing,
    Log,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionsDetailPage {
    Language,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrepareDetailPage {
    Language,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessingDetailPage {
    Transcode,
}

pub enum ThumbnailRenderSource {
    None,
    DirectUrl,
    Loading,
    Texture(eframe::egui::TextureHandle),
    Failed(String),
}

enum ThumbnailCacheEntry {
    Loading,
    Ready(eframe::egui::TextureHandle),
    Failed(String),
}

pub struct AppState {
    pub active_tab: AppTab,
    pub url_input: String,
    pub batch_input: String,
    pub batch_enabled: bool,
    pub monitor_clipboard: bool,
    last_clipboard_text: String,
    last_clipboard_check: Option<Instant>,
    pub empty_item_preview: VideoMetadata,
    pub queue_items: Vec<QueueItem>,
    pub item_defaults: DownloadOptions,
    pub config: AppConfig,
    pending_ui_scale_percent: u16,
    pub options_detail_page: Option<OptionsDetailPage>,
    pub prepare_detail_page: Option<PrepareDetailPage>,
    pub processing_detail_page: Option<ProcessingDetailPage>,
    pub tool_paths: ToolPaths,
    pub prepare_report: PrepareReport,
    prepare_tool_selection: Vec<DependencyTool>,
    prepare_tab_snoozed: bool,
    pending_dependency_installs: VecDeque<DependencyTool>,
    pub last_action: String,
    pub runtime_log: VecDeque<String>,
    pub format_picker: FormatPickerState,
    pub is_adding_batch: bool,
    pub is_cancelling_batch_add: bool,
    pub youtube_playlist_prompt: Option<YoutubePlaylistPrompt>,
    analyze_result_rx: Receiver<AnalyzeResult>,
    analyze_result_tx: Sender<AnalyzeResult>,
    batch_add_result_rx: Option<Receiver<BatchAddEvent>>,
    batch_add_child: Option<Arc<Mutex<Option<Child>>>>,
    batch_add_cancel_requested: Option<Arc<AtomicBool>>,
    download_result_rx: Receiver<DownloadEvent>,
    download_result_tx: Sender<DownloadEvent>,
    post_process_result_rx: Receiver<PostProcessEvent>,
    post_process_result_tx: Sender<PostProcessEvent>,
    tool_install_result_rx: Receiver<ToolInstallEvent>,
    tool_install_result_tx: Sender<ToolInstallEvent>,
    thumbnail_result_rx: Receiver<ThumbnailFetchEvent>,
    thumbnail_result_tx: Sender<ThumbnailFetchEvent>,
    thumbnail_cache: HashMap<String, ThumbnailCacheEntry>,
    installing_dependency_tool: Option<DependencyTool>,
    tool_install_cancel_handle: Option<ToolInstallCancelHandle>,
    dependency_install_progress: HashMap<DependencyTool, ToolInstallProgress>,
    active_workflows: HashMap<WorkflowRunId, ActiveWorkflow>,
    next_queue_item_id: QueueItemId,
    next_workflow_run_id: WorkflowRunId,
}

struct AnalyzeResult {
    source: String,
    target_item_id: Option<QueueItemId>,
    workflow_id: Option<WorkflowRunId>,
    used_cookies: bool,
    result: Result<Value, String>,
}

pub struct YoutubePlaylistPrompt {
    pub source: String,
    pub kind: YoutubePlaylistPromptKind,
    pub risk: Option<YoutubePlaylistRisk>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YoutubePlaylistPromptKind {
    VideoAndPlaylist,
    HighRiskPlaylist,
}

struct ActiveWorkflow {
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    kind: WorkflowKind,
    tool: ToolKind,
    download_child: Option<Arc<Mutex<Option<Child>>>>,
    cancel_requested: Option<Arc<AtomicBool>>,
}

fn progress_status_text(language: Language, progress: &ToolInstallProgress) -> String {
    let stage = i18n::text(language, progress.stage.label());
    match progress.percent {
        Some(percent)
            if matches!(
                progress.stage,
                ToolInstallStage::Downloading
                    | ToolInstallStage::Extracting
                    | ToolInstallStage::Installing
            ) =>
        {
            format!("{stage} {percent}%")
        }
        _ => stage.to_owned(),
    }
}

fn monotonic_progress(current: f32, next: f32) -> f32 {
    if next.is_finite() {
        current.max(next.clamp(0.0, 100.0))
    } else {
        current
    }
}

fn progress_summary_text(language: Language, progress: &ToolInstallProgress) -> String {
    let stage = i18n::text(language, progress.stage.label());
    match progress.percent {
        Some(percent)
            if matches!(
                progress.stage,
                ToolInstallStage::Downloading
                    | ToolInstallStage::Extracting
                    | ToolInstallStage::Installing
            ) =>
        {
            format!("{} {stage} {percent}%", progress.tool.label())
        }
        _ => format!("{} {stage}", progress.tool.label()),
    }
}

fn queue_item_status_key(item: &QueueItem) -> &'static str {
    if let Some(run) = item.workflows.iter().rev().find(|run| {
        matches!(
            run.kind,
            WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
        ) && matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
    }) {
        return match run.state {
            WorkflowState::Queued => "item.status.queued",
            WorkflowState::Running => "item.status.running",
            _ => "item.status.queued",
        };
    }

    if let Some(run) = item
        .workflows
        .iter()
        .rev()
        .find(|run| run.kind == WorkflowKind::DownloadMedia)
    {
        return match run.state {
            WorkflowState::Queued => "item.status.queued",
            WorkflowState::Running => "item.status.running",
            WorkflowState::Finished if item.last_error.is_some() => "item.status.failed",
            WorkflowState::Finished => "item.status.finished",
            WorkflowState::Failed => "item.status.failed",
            WorkflowState::Cancelled => "item.status.cancelled",
        };
    }

    match &item.metadata_state {
        MetadataState::Idle => "item.status.idle",
        MetadataState::Queued => "item.status.waiting_analysis",
        MetadataState::Running => "item.status.analyzing",
        MetadataState::Ready(_) => "item.status.queued",
        MetadataState::Failed(_) => "item.status.analysis_failed",
    }
}

fn thumbnail_cache_key(url: &str, proxy_url: &str, no_check_certificates: bool) -> String {
    format!(
        "{}\n{}\n{}",
        proxy_url.trim(),
        no_check_certificates,
        url.trim()
    )
}

fn thumbnail_texture_id(key: &str) -> String {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    format!("proxied-thumbnail-{:016x}", hasher.finish())
}

fn thumbnail_needs_memory_loader(url: &str) -> bool {
    let url = url.trim();
    url.starts_with("http://") || url.starts_with("https://")
}

fn reset_item_for_new_work(item: &mut QueueItem, target_kind: DownloadTargetKind) {
    match target_kind {
        DownloadTargetKind::Normal => {
            item.progress.video = 0.0;
            item.progress.audio = 0.0;
            item.progress.subtitle = 0.0;
            item.progress.post_process = 0.0;
        }
        DownloadTargetKind::Video => item.progress.video = 0.0,
        DownloadTargetKind::Audio => item.progress.audio = 0.0,
        DownloadTargetKind::Subtitle => item.progress.subtitle = 0.0,
    }
    item.last_error = None;
}

impl AppState {
    pub fn new() -> Self {
        let (config, tool_paths) = AppConfig::load_runtime();
        let (analyze_result_tx, analyze_result_rx) = mpsc::channel();
        let (download_result_tx, download_result_rx) = mpsc::channel();
        let (post_process_result_tx, post_process_result_rx) = mpsc::channel();
        let (tool_install_result_tx, tool_install_result_rx) = mpsc::channel();
        let (thumbnail_result_tx, thumbnail_result_rx) = mpsc::channel();
        let pending_ui_scale_percent = config.ui_scale_percent;
        let mut state = Self {
            active_tab: AppTab::Main,
            url_input: String::new(),
            batch_input: String::new(),
            batch_enabled: true,
            monitor_clipboard: config.auto_paste_clipboard,
            last_clipboard_text: String::new(),
            last_clipboard_check: None,
            empty_item_preview: VideoMetadata::empty_preview(),
            queue_items: Vec::new(),
            item_defaults: {
                let mut defaults = DownloadOptions::default();
                defaults.output_dir = config.download_dir.clone();
                defaults.use_cookies = config.use_browser_cookies;
                defaults.use_aria2 = config.use_aria2;
                defaults.write_thumbnail = config.write_thumbnail;
                defaults.embed_thumbnail = config.embed_thumbnail;
                defaults.write_subtitles = config.write_subtitles;
                defaults.embed_subtitles = config.embed_subtitles;
                defaults.write_chapters = config.write_chapters;
                defaults.embed_chapters = config.embed_chapters;
                defaults
            },
            config,
            pending_ui_scale_percent,
            options_detail_page: None,
            prepare_detail_page: None,
            processing_detail_page: None,
            tool_paths,
            prepare_report: PrepareReport::default(),
            prepare_tool_selection: Vec::new(),
            prepare_tab_snoozed: false,
            pending_dependency_installs: VecDeque::new(),
            last_action: String::new(),
            runtime_log: VecDeque::new(),
            format_picker: FormatPickerState::default(),
            is_adding_batch: false,
            is_cancelling_batch_add: false,
            youtube_playlist_prompt: None,
            analyze_result_rx,
            analyze_result_tx,
            batch_add_result_rx: None,
            batch_add_child: None,
            batch_add_cancel_requested: None,
            download_result_rx,
            download_result_tx,
            post_process_result_rx,
            post_process_result_tx,
            tool_install_result_rx,
            tool_install_result_tx,
            thumbnail_result_rx,
            thumbnail_result_tx,
            thumbnail_cache: HashMap::new(),
            installing_dependency_tool: None,
            tool_install_cancel_handle: None,
            dependency_install_progress: HashMap::new(),
            active_workflows: HashMap::new(),
            next_queue_item_id: 1,
            next_workflow_run_id: 1,
        };

        state.refresh_prepare_report();
        state.reset_prepare_tool_selection_to_defaults();
        state.prime_clipboard_monitor_baseline();
        if state.should_show_prepare_tab() {
            state.active_tab = AppTab::Prepare;
        }
        state
    }

    fn prime_clipboard_monitor_baseline(&mut self) {
        if !self.monitor_clipboard {
            return;
        }

        self.last_clipboard_text = read_clipboard_text().unwrap_or_default();
        self.last_clipboard_check = Some(Instant::now());
    }

    pub fn poll_clipboard_monitor(&mut self) {
        if !self.monitor_clipboard {
            return;
        }

        let now = Instant::now();
        if self
            .last_clipboard_check
            .is_some_and(|last| now.duration_since(last) < Duration::from_millis(800))
        {
            return;
        }
        self.last_clipboard_check = Some(now);

        let Some(text) = read_clipboard_text() else {
            return;
        };
        if text == self.last_clipboard_text {
            return;
        }
        self.last_clipboard_text = text.clone();

        if self.clipboard_monitor_input_blocked() {
            return;
        }

        let Some(url) = extract_monitored_youtube_url(&text) else {
            return;
        };
        if self.url_input.trim() == url && !self.config.clipboard_auto_add {
            return;
        }

        self.url_input = url.clone();
        if self.config.clipboard_auto_add {
            if let Err(error) = self.ensure_yt_dlp_ready() {
                self.last_action = error;
                return;
            }
            self.run_primary_url_action();
        } else {
            self.last_action = self.tr("state.clipboard_detected_url").to_owned();
        }
    }

    fn clipboard_monitor_input_blocked(&self) -> bool {
        self.url_input_locked() || self.format_picker.open
    }

    pub fn clipboard_monitor_enabled(&self) -> bool {
        self.monitor_clipboard
    }

    pub fn analyze_current_input(&mut self) {
        let Some(source) = self.primary_candidate_url() else {
            self.last_action = self.tr("state.no_url_to_analyze").to_owned();
            return;
        };

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        self.last_action = self.trf("state.analyzing_source", &[("{source}", source.as_str())]);
        self.spawn_analyze_worker(source, None, None, false);
    }

    pub fn add_current_url_to_batch(&mut self) {
        if self.is_adding_batch {
            self.last_action = self.tr("state.batch_add_running").to_owned();
            return;
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        let source = self.url_input.trim();
        if source.is_empty() {
            self.last_action = self.tr("state.no_url_to_add").to_owned();
            return;
        }

        let source = source.to_owned();
        if youtube_url_has_video_and_playlist(&source) {
            match self.config.youtube_video_playlist_mode {
                YoutubeVideoPlaylistMode::Ask => {
                    let risk = if self.config.youtube_high_risk_playlist_prompt {
                        classify_youtube_playlist(&source)
                    } else {
                        None
                    };
                    self.youtube_playlist_prompt = Some(YoutubePlaylistPrompt {
                        source,
                        kind: YoutubePlaylistPromptKind::VideoAndPlaylist,
                        risk,
                    });
                    self.last_action = self.tr("state.video_url_contains_playlist").to_owned();
                    return;
                }
                YoutubeVideoPlaylistMode::Video => {
                    let single_source =
                        youtube_url_force_single_video(&source).unwrap_or_else(|| source.clone());
                    self.add_single_url_to_batch(single_source);
                    return;
                }
                YoutubeVideoPlaylistMode::Ignore => {}
            }
        }

        if !looks_like_playlist_url(&source) {
            self.add_single_url_to_batch(source);
            return;
        }

        if self.config.youtube_high_risk_playlist_prompt {
            if let Some(risk) = classify_youtube_playlist(&source) {
                self.youtube_playlist_prompt = Some(YoutubePlaylistPrompt {
                    source,
                    kind: YoutubePlaylistPromptKind::HighRiskPlaylist,
                    risk: Some(risk),
                });
                self.last_action = self.trf(
                    "state.detected_high_risk_playlist",
                    &[("{kind}", self.tr(risk.kind.label()))],
                );
                return;
            }
        }

        self.begin_batch_add(source);
    }

    pub fn run_primary_url_action(&mut self) {
        if self.config.direct_download_on_add {
            self.immediate_download_current_url();
        } else {
            self.add_current_url_to_batch();
        }
    }

    pub fn primary_url_action_label(&self) -> &'static str {
        if self.is_adding_batch {
            if self.is_cancelling_batch_add {
                self.tr("action.stopping")
            } else {
                self.tr("action.stop")
            }
        } else if self.config.direct_download_on_add {
            self.tr("action.download")
        } else {
            self.tr("action.add")
        }
    }

    pub fn immediate_download_current_url(&mut self) {
        if self.is_adding_batch {
            self.last_action = self.tr("state.batch_add_running").to_owned();
            return;
        }
        let source = self.url_input.trim();
        if source.is_empty() {
            self.last_action = self.tr("state.no_url_to_download_now").to_owned();
            return;
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        let source = source.to_owned();
        let source = if youtube_url_has_video_and_playlist(&source) {
            youtube_url_force_single_video(&source).unwrap_or(source)
        } else {
            source
        };
        if looks_like_playlist_url(&source) {
            self.last_action = self.tr("state.download_now_single_video_only").to_owned();
            return;
        }

        let item_id = self.ensure_queue_item_for_url(&source);
        self.url_input.clear();
        let fallback_title = infer_title(
            &source,
            self.tr("state.untitled_task"),
            self.tr("state.imported_source"),
        );
        self.last_action = self.trf(
            "state.added_ready_download_now",
            &[("{title}", fallback_title.as_str())],
        );
        let emit_json = self
            .queue_item_by_id(item_id)
            .is_some_and(|item| !item.metadata_loaded());
        let _ = self.start_download_task_at(item_id, emit_json);
    }

    pub fn confirm_youtube_playlist_prompt(&mut self) {
        let Some(prompt) = self.youtube_playlist_prompt.take() else {
            return;
        };
        self.begin_batch_add(prompt.source);
    }

    pub fn confirm_youtube_playlist_prompt_as_video(&mut self) {
        let Some(prompt) = self.youtube_playlist_prompt.take() else {
            return;
        };
        let source = youtube_url_force_single_video(&prompt.source).unwrap_or(prompt.source);
        self.add_single_url_to_batch(source);
    }

    pub fn cancel_youtube_playlist_prompt(&mut self) {
        self.youtube_playlist_prompt = None;
        self.last_action = self.tr("state.current_action_cancelled").to_owned();
    }

    pub fn cancel_batch_add(&mut self) {
        self.is_cancelling_batch_add = true;
        if let Some(cancel_flag) = &self.batch_add_cancel_requested {
            cancel_flag.store(true, Ordering::Relaxed);
        }
        if let Some(child_handle) = &self.batch_add_child {
            request_batch_add_stop(child_handle);
        }
        self.last_action = self.tr("state.stopping_batch_add").to_owned();
    }

    pub fn poll_background_work(&mut self) {
        loop {
            match self.analyze_result_rx.try_recv() {
                Ok(message) => match message.result {
                    Ok(json) => {
                        if let Some(item_id) = message.target_item_id {
                            if let Some(item) = self.queue_item_mut_by_id(item_id) {
                                item.cookie_policy = if message.used_cookies {
                                    CookiePolicy::Required
                                } else {
                                    CookiePolicy::NotNeeded
                                };
                            }
                        }
                        self.apply_analysis_json(
                            json,
                            Some(message.source),
                            message.target_item_id,
                            message.workflow_id,
                        );
                    }
                    Err(error) => {
                        let should_retry_with_cookies = message
                            .target_item_id
                            .and_then(|item_id| self.queue_item_by_id(item_id))
                            .is_some_and(|item| item.selection.use_cookies)
                            && !message.used_cookies
                            && message
                                .target_item_id
                                .and_then(|item_id| self.queue_item_by_id(item_id))
                                .is_some_and(|item| item.cookie_policy == CookiePolicy::Unknown)
                            && should_retry_analyze_with_cookies(&error);

                        if should_retry_with_cookies {
                            if let Some(item_id) = message.target_item_id {
                                if let Some(item) = self.queue_item_mut_by_id(item_id) {
                                    item.cookie_policy = CookiePolicy::Required;
                                }
                            }
                            self.last_action = self.trf(
                                "state.retrying_analysis_cookie",
                                &[("{source}", message.source.as_str())],
                            );
                            self.spawn_analyze_worker(
                                message.source,
                                message.target_item_id,
                                message.workflow_id,
                                true,
                            );
                            continue;
                        }
                        eprintln!("[analyze] {error}");
                        if let Some(item_id) = message.target_item_id {
                            if let Some(item) = self.queue_item_mut_by_id(item_id) {
                                item.metadata_state = MetadataState::Failed(error.clone());
                                item.last_error = Some(error.clone());
                                if let Some(workflow_id) = message.workflow_id {
                                    if let Some(run) =
                                        item.workflows.iter_mut().find(|run| run.id == workflow_id)
                                    {
                                        run.state = WorkflowState::Failed;
                                        run.error = Some(error.clone());
                                    }
                                    self.unregister_active_workflow(workflow_id);
                                }
                            }
                        }
                        self.last_action = error;
                    }
                },
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        if let Some(rx) = self.batch_add_result_rx.take() {
            let mut keep_rx = true;
            loop {
                match rx.try_recv() {
                    Ok(BatchAddEvent::ItemAdded { source, seed }) => {
                        let cancel_requested = self
                            .batch_add_cancel_requested
                            .as_ref()
                            .is_some_and(|flag| flag.load(Ordering::Relaxed));
                        if !cancel_requested {
                            self.append_batch_seed(&source, seed);
                        }
                    }
                    Ok(BatchAddEvent::Finished {
                        source,
                        added,
                        stopped_by_limit,
                    }) => {
                        self.is_adding_batch = false;
                        self.is_cancelling_batch_add = false;
                        self.batch_add_child = None;
                        self.batch_add_cancel_requested = None;
                        keep_rx = false;
                        if added == 0 {
                            self.last_action = self.tr("state.batch_no_new_items").to_owned();
                        } else if stopped_by_limit {
                            self.last_action = self.trf(
                                "state.playlist_added_limited",
                                &[("{count}", &added.to_string())],
                            );
                        } else if added == 1 {
                            let fallback_title = infer_title(
                                &source,
                                self.tr("state.untitled_task"),
                                self.tr("state.imported_source"),
                            );
                            self.last_action = self.trf(
                                "state.batch_added_title",
                                &[("{title}", fallback_title.as_str())],
                            );
                        } else {
                            self.last_action = self
                                .trf("state.playlist_added", &[("{count}", &added.to_string())]);
                        }
                        self.url_input.clear();
                        break;
                    }
                    Ok(BatchAddEvent::Failed { error }) => {
                        self.is_adding_batch = false;
                        self.is_cancelling_batch_add = false;
                        self.batch_add_child = None;
                        self.batch_add_cancel_requested = None;
                        keep_rx = false;
                        self.last_action = error;
                        break;
                    }
                    Ok(BatchAddEvent::Cancelled { added }) => {
                        self.is_adding_batch = false;
                        self.is_cancelling_batch_add = false;
                        self.batch_add_child = None;
                        self.batch_add_cancel_requested = None;
                        keep_rx = false;
                        self.last_action = if added == 0 {
                            self.tr("state.batch_add_cancelled").to_owned()
                        } else {
                            self.trf(
                                "state.batch_add_cancelled_with_count",
                                &[("{count}", &added.to_string())],
                            )
                        };
                        self.url_input.clear();
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.is_adding_batch = false;
                        self.is_cancelling_batch_add = false;
                        self.batch_add_child = None;
                        self.batch_add_cancel_requested = None;
                        keep_rx = false;
                        self.last_action = self.tr("state.batch_add_interrupted").to_owned();
                        break;
                    }
                }
            }
            if keep_rx {
                self.batch_add_result_rx = Some(rx);
            }
        }

        loop {
            match self.tool_install_result_rx.try_recv() {
                Ok(ToolInstallEvent::Progress(progress)) => {
                    self.last_action = progress_summary_text(self.language(), &progress);
                    self.dependency_install_progress
                        .insert(progress.tool, progress);
                }
                Ok(ToolInstallEvent::Finished { tool, result }) => {
                    self.installing_dependency_tool = None;
                    self.tool_install_cancel_handle = None;
                    match result {
                        Ok(path) => {
                            self.dependency_install_progress.insert(
                                tool,
                                ToolInstallProgress {
                                    tool,
                                    stage: ToolInstallStage::Completed,
                                    percent: Some(100),
                                    message: self.tr("state.deployment_complete").to_owned(),
                                },
                            );
                            match tool {
                                DependencyTool::YtDlp => self.set_yt_dlp_path(path),
                                DependencyTool::Ffmpeg => self.set_ffmpeg_path(path),
                                DependencyTool::Aria2c => self.set_aria2c_path(path),
                                DependencyTool::Deno => self.set_deno_path(path),
                            }
                            self.refresh_prepare_report();
                            if let Some(next_tool) = self.pending_dependency_installs.pop_front() {
                                self.begin_dependency_install(next_tool);
                            } else {
                                self.last_action =
                                    self.trf("state.tool_deployed", &[("{tool}", tool.label())]);
                                if !self.should_show_prepare_tab()
                                    && self.active_tab == AppTab::Prepare
                                {
                                    self.active_tab = AppTab::Main;
                                }
                            }
                        }
                        Err(error) => {
                            self.dependency_install_progress.insert(
                                tool,
                                ToolInstallProgress {
                                    tool,
                                    stage: ToolInstallStage::Failed,
                                    percent: None,
                                    message: error.clone(),
                                },
                            );
                            self.pending_dependency_installs.clear();
                            self.refresh_prepare_report();
                            if !self.should_show_prepare_tab() && self.active_tab == AppTab::Prepare
                            {
                                self.active_tab = AppTab::Main;
                            }
                            let localized_error = self.localize_message(&error);
                            self.last_action = self.trf(
                                "state.tool_deploy_failed",
                                &[
                                    ("{tool}", tool.label()),
                                    ("{error}", localized_error.as_str()),
                                ],
                            );
                        }
                    }
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        loop {
            match self.download_result_rx.try_recv() {
                Ok(DownloadEvent::Metadata { item_id, json }) => {
                    self.apply_analysis_json(json, None, Some(item_id), None);
                }
                Ok(DownloadEvent::Progress {
                    item_id,
                    workflow_id,
                    slot,
                    percent,
                }) => {
                    if let Some(item) = self.queue_item_mut_by_id(item_id) {
                        let display_percent = percent.clamp(0.0, 100.0);
                        if let Some(run) =
                            item.workflows.iter_mut().find(|run| run.id == workflow_id)
                        {
                            run.progress = monotonic_progress(run.progress, display_percent);
                        }
                        match slot {
                            DownloadProgressSlot::Video => {
                                item.progress.video =
                                    monotonic_progress(item.progress.video, display_percent);
                            }
                            DownloadProgressSlot::Audio => {
                                item.progress.audio =
                                    monotonic_progress(item.progress.audio, display_percent);
                            }
                            DownloadProgressSlot::Subtitle => {
                                item.progress.subtitle =
                                    monotonic_progress(item.progress.subtitle, display_percent);
                            }
                            DownloadProgressSlot::Both => {
                                item.progress.video =
                                    monotonic_progress(item.progress.video, display_percent);
                                item.progress.audio =
                                    monotonic_progress(item.progress.audio, display_percent);
                            }
                        }
                    }
                }
                Ok(DownloadEvent::Finished(message)) => {
                    let finished_item_id = message.item_id;
                    let notification_title = self
                        .queue_item_by_id(message.item_id)
                        .map(|item| item.title.trim().to_owned())
                        .filter(|title| !title.is_empty())
                        .unwrap_or_else(|| self.tr("state.download_item_fallback").to_owned());
                    let notification_result = message.result.clone();
                    let should_send_windows_toast =
                        message.workflow_kind == WorkflowKind::DownloadMedia;
                    self.unregister_active_workflow(message.workflow_id);
                    let mut pending_post_process_input = None;
                    if let Some(item) = self.queue_item_mut_by_id(message.item_id) {
                        if let Some(run) = item
                            .workflows
                            .iter_mut()
                            .find(|run| run.id == message.workflow_id)
                        {
                            match &message.result {
                                Ok(output_path) => {
                                    run.state = WorkflowState::Finished;
                                    run.output_path = Some(output_path.clone());
                                    match message.target_kind {
                                        DownloadTargetKind::Normal => {
                                            item.progress.video = 100.0;
                                            item.progress.audio = 100.0;
                                            if let Some(actual_file_name) = Path::new(output_path)
                                                .file_name()
                                                .and_then(|value| value.to_str())
                                                .map(ToOwned::to_owned)
                                            {
                                                item.selection.file_name = actual_file_name;
                                            }
                                            item.completed_selection = Some(
                                                CompletedSelection::from_selection(&item.selection),
                                            );
                                        }
                                        DownloadTargetKind::Video => item.progress.video = 100.0,
                                        DownloadTargetKind::Audio => item.progress.audio = 100.0,
                                        DownloadTargetKind::Subtitle => {
                                            item.progress.subtitle = 100.0
                                        }
                                    }
                                    item.last_output_path = Some(output_path.clone());
                                    item.last_error = None;
                                    if message.workflow_kind == WorkflowKind::DownloadMedia
                                        && message.target_kind == DownloadTargetKind::Normal
                                    {
                                        pending_post_process_input = Some(output_path.clone());
                                    }
                                }
                                Err(error) if error == DOWNLOAD_CANCELLED_MESSAGE => {
                                    run.state = WorkflowState::Cancelled;
                                    run.error = None;
                                    item.last_error = None;
                                }
                                Err(error) => {
                                    run.state = WorkflowState::Failed;
                                    run.error = Some(error.clone());
                                    item.last_error = Some(error.clone());
                                }
                            }
                        }
                    }

                    let post_process_started = pending_post_process_input
                        .as_deref()
                        .is_some_and(|output_path| {
                            self.maybe_start_builtin_transcode_post_process(
                                message.item_id,
                                output_path,
                            )
                        });

                    match message.result {
                        Ok(output_path) => {
                            self.push_runtime_log(format!("Download finished: {output_path}"));
                            if !post_process_started {
                                self.last_action.clear();
                            }
                        }
                        Err(error) if error == DOWNLOAD_CANCELLED_MESSAGE => {
                            self.push_runtime_log("Download cancelled".to_owned());
                            self.last_action = self.tr("state.download_stopped").to_owned();
                        }
                        Err(error) => {
                            self.push_runtime_log(format!("Download failed: {error}"));
                            eprintln!("[download] {error}");
                            self.last_action = error;
                        }
                    }

                    if should_send_windows_toast && !post_process_started {
                        self.send_download_result_windows_toast(
                            notification_title,
                            notification_result,
                        );
                        self.start_next_queued_download_after(finished_item_id);
                    }
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        loop {
            match self.post_process_result_rx.try_recv() {
                Ok(PostProcessEvent::Progress {
                    item_id,
                    workflow_id,
                    percent,
                }) => {
                    if let Some(item) = self.queue_item_mut_by_id(item_id) {
                        item.progress.post_process = percent;
                        if let Some(run) =
                            item.workflows.iter_mut().find(|run| run.id == workflow_id)
                        {
                            run.progress = percent;
                        }
                    }
                }
                Ok(PostProcessEvent::Finished(message)) => {
                    let finished_item_id = message.item_id;
                    let notification_title = self
                        .queue_item_by_id(message.item_id)
                        .map(|item| item.title.trim().to_owned())
                        .filter(|title| !title.is_empty())
                        .unwrap_or_else(|| self.tr("state.download_item_fallback").to_owned());
                    let notification_result = message.result.clone();
                    self.unregister_active_workflow(message.workflow_id);

                    if let Some(item) = self.queue_item_mut_by_id(message.item_id) {
                        if let Some(run) = item
                            .workflows
                            .iter_mut()
                            .find(|run| run.id == message.workflow_id)
                        {
                            match &message.result {
                                Ok(output_path) => {
                                    run.state = WorkflowState::Finished;
                                    run.progress = 100.0;
                                    run.output_path = Some(output_path.clone());
                                    item.progress.post_process = 100.0;
                                    item.last_output_path = Some(output_path.clone());
                                    if let Some(actual_file_name) = Path::new(output_path)
                                        .file_name()
                                        .and_then(|value| value.to_str())
                                        .map(ToOwned::to_owned)
                                    {
                                        item.selection.file_name = actual_file_name;
                                    }
                                    item.completed_selection =
                                        Some(CompletedSelection::from_selection(&item.selection));
                                    item.last_error = None;
                                }
                                Err(error) if error == POST_PROCESS_CANCELLED_MESSAGE => {
                                    run.state = WorkflowState::Cancelled;
                                    run.error = None;
                                    item.last_error = None;
                                }
                                Err(error) => {
                                    run.state = WorkflowState::Failed;
                                    run.error = Some(error.clone());
                                    item.last_error = Some(error.clone());
                                    item.completed_selection = None;
                                }
                            }
                        }
                    }

                    match message.result {
                        Ok(output_path) => {
                            self.push_runtime_log(format!("Post-process finished: {output_path}"));
                            self.last_action.clear();
                        }
                        Err(error) if error == POST_PROCESS_CANCELLED_MESSAGE => {
                            self.push_runtime_log("Post-process cancelled".to_owned());
                            self.last_action = self.tr("state.download_stopped").to_owned();
                        }
                        Err(error) => {
                            self.push_runtime_log(format!("Post-process failed: {error}"));
                            eprintln!("[post-process] {error}");
                            self.last_action = error;
                        }
                    }

                    self.send_download_result_windows_toast(notification_title, notification_result);
                    self.start_next_queued_download_after(finished_item_id);
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
    }

    pub fn queue_batch(&mut self) {
        let urls = self.parsed_batch_urls();
        let count = urls.len();
        if count == 0 {
            self.last_action = self.tr("state.no_url_to_add_batch").to_owned();
            return;
        }

        self.queue_items = urls
            .iter()
            .map(|url| self.build_queue_item_from_url(url))
            .collect();
        self.last_action = self.trf(
            "state.batch_input_added",
            &[("{count}", &count.to_string())],
        );
    }

    pub fn start_single_download(&mut self) {
        let Some(url) = self.primary_candidate_url() else {
            self.last_action = self.tr("state.no_url_to_download").to_owned();
            return;
        };

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        if self.queue_items.is_empty() && !self.parsed_batch_urls().is_empty() {
            self.queue_batch();
        }

        if self.queue_items.is_empty() {
            self.queue_items = vec![self.build_queue_item_from_url(&url)];
        }

        if self.has_running_download_workflow() {
            self.last_action = self.tr("state.download_already_running").to_owned();
            return;
        }

        self.enqueue_download_ready_items();

        let Some(item_id) = self
            .queue_items
            .iter()
            .find(|item| item_latest_download_state(item).is_some_and(is_pending_download_state))
            .map(|item| item.id)
        else {
            self.last_action = self.tr("state.no_runnable_batch_items").to_owned();
            return;
        };

        let emit_json = self
            .queue_item_by_id(item_id)
            .is_some_and(|item| !item.metadata_loaded());
        let _ = self.start_download_task_at(item_id, emit_json);
    }

    fn enqueue_download_ready_items(&mut self) {
        let ready_item_ids = self
            .queue_items
            .iter()
            .filter(|item| !item.source_url.trim().is_empty())
            .filter(|item| match item_latest_download_state(item) {
                None => true,
                Some(
                    WorkflowState::Failed | WorkflowState::Finished | WorkflowState::Cancelled,
                ) => true,
                Some(_) => false,
            })
            .map(|item| item.id)
            .collect::<Vec<_>>();

        for item_id in ready_item_ids {
            let workflow_id = self.alloc_workflow_run_id();
            let Some(item) = self.queue_item_mut_by_id(item_id) else {
                continue;
            };
            reset_item_for_new_work(item, DownloadTargetKind::Normal);
            item.completed_selection = None;
            let mut run = WorkflowRun::new(
                workflow_id,
                WorkflowKind::DownloadMedia,
                ToolKind::YtDlp,
                WorkflowState::Queued,
            );
            run.detail = item.source_url.clone();
            item.workflows.push(run);
        }
    }

    fn ensure_queue_item_for_url(&mut self, url: &str) -> QueueItemId {
        let source_key = canonical_queue_source_key(url);
        if let Some(item) = self
            .queue_items
            .iter()
            .find(|item| canonical_queue_source_key(&item.source_url) == source_key)
        {
            return item.id;
        }

        let item = self.build_queue_item_from_url(url);
        let item_id = item.id;
        self.queue_items.push(item);
        self.batch_input_push_unique(url);
        item_id
    }

    fn batch_input_push_unique(&mut self, url: &str) {
        let source_key = canonical_queue_source_key(url);
        if self
            .batch_input
            .lines()
            .map(str::trim)
            .any(|line| canonical_queue_source_key(line) == source_key)
        {
            return;
        }
        if !self.batch_input.trim().is_empty() {
            self.batch_input.push('\n');
        }
        self.batch_input.push_str(url);
    }

    fn alloc_queue_item_id(&mut self) -> QueueItemId {
        let id = self.next_queue_item_id;
        self.next_queue_item_id += 1;
        id
    }

    fn alloc_workflow_run_id(&mut self) -> WorkflowRunId {
        let id = self.next_workflow_run_id;
        self.next_workflow_run_id += 1;
        id
    }

    fn register_active_workflow(
        &mut self,
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        kind: WorkflowKind,
        tool: ToolKind,
    ) {
        self.active_workflows.insert(
            workflow_id,
            ActiveWorkflow {
                item_id,
                workflow_id,
                kind,
                tool,
                download_child: None,
                cancel_requested: None,
            },
        );
    }

    fn attach_active_download_process(
        &mut self,
        workflow_id: WorkflowRunId,
        child_handle: Arc<Mutex<Option<Child>>>,
        cancel_requested: Arc<AtomicBool>,
    ) {
        if let Some(workflow) = self.active_workflows.get_mut(&workflow_id) {
            workflow.download_child = Some(child_handle);
            workflow.cancel_requested = Some(cancel_requested);
        }
    }

    fn unregister_active_workflow(&mut self, workflow_id: WorkflowRunId) {
        self.active_workflows.remove(&workflow_id);
    }

    fn has_running_download_workflow(&self) -> bool {
        self.active_workflows
            .values()
            .any(|workflow| workflow.kind == WorkflowKind::DownloadMedia)
    }

    fn maybe_start_builtin_transcode_post_process(
        &mut self,
        item_id: QueueItemId,
        input_path: &str,
    ) -> bool {
        if !self.config.post_download_conversion_enabled {
            return false;
        }
        let plan = resolve_transcode_plan(&self.config.transcode_intent);
        if !plan.is_executable() {
            return false;
        }
        let Some(profile) = plan.backend_profile else {
            return false;
        };

        let Some(item_index) = self.queue_item_index_by_id(item_id) else {
            return false;
        };
        let title = self.queue_items[item_index].title.clone();
        let workflow_id = self.alloc_workflow_run_id();
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::PostProcess,
            ToolKind::Ffmpeg,
        );

        if let Some(item) = self.queue_items.get_mut(item_index) {
            item.progress.post_process = 0.0;
            let mut run = WorkflowRun::new(
                workflow_id,
                WorkflowKind::PostProcess,
                ToolKind::Ffmpeg,
                WorkflowState::Running,
            );
            run.detail = input_path.to_owned();
            item.workflows.push(run);
        }

        self.last_action = self.trf(
            "state.transcode_post_processing_title",
            &[("{title}", title.as_str()), ("{profile}", profile.label())],
        );
        self.push_runtime_log(format!("Post-process started: {title} -> {}", profile.label()));

        let tool_paths = self.tool_paths.clone();
        let settings = self.config.transcode_intent.clone();
        let tx = self.post_process_result_tx.clone();
        let input_path = input_path.to_owned();
        let child_handle = Arc::new(Mutex::new(None));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.attach_active_download_process(
            workflow_id,
            child_handle.clone(),
            cancel_requested.clone(),
        );

        thread::spawn(move || {
            run_builtin_transcode_worker(
                tool_paths,
                settings,
                input_path,
                item_id,
                workflow_id,
                tx,
                child_handle,
                cancel_requested,
            );
        });

        true
    }

    fn start_next_queued_download_after(&mut self, finished_item_id: QueueItemId) {
        if self.has_running_download_workflow() {
            return;
        }

        let Some(next_item_id) = self
            .queue_items
            .iter()
            .find(|item| {
                item.id != finished_item_id
                    && item_latest_download_state(item).is_some_and(is_pending_download_state)
            })
            .map(|item| item.id)
        else {
            return;
        };

        let emit_json = self
            .queue_item_by_id(next_item_id)
            .is_some_and(|item| !item.metadata_loaded());
        let _ = self.start_download_task_at(next_item_id, emit_json);
    }

    pub fn active_workflow_count(&self) -> usize {
        self.active_workflows.len()
    }

    pub fn item_has_running_workflow(&self, item_id: QueueItemId, kind: WorkflowKind) -> bool {
        self.active_workflows
            .values()
            .any(|workflow| workflow.item_id == item_id && workflow.kind == kind)
    }

    pub fn item_has_cancellable_download_workflow(&self, item_id: QueueItemId) -> bool {
        self.active_workflows.values().any(|workflow| {
            workflow.item_id == item_id
                && matches!(
                    workflow.kind,
                    WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
                )
                && workflow.download_child.is_some()
        })
    }

    pub fn cancel_item_download(&mut self, item_id: QueueItemId) {
        let workflows = self
            .active_workflows
            .values()
            .filter(|workflow| {
                workflow.item_id == item_id
                    && matches!(
                        workflow.kind,
                        WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
                    )
            })
            .map(|workflow| workflow.workflow_id)
            .collect::<Vec<_>>();

        if workflows.is_empty() {
            self.last_action = self.tr("state.no_download_to_stop").to_owned();
            return;
        }

        for workflow_id in workflows {
            self.request_active_download_stop(workflow_id);
        }
        self.last_action = self.tr("state.stopping_download").to_owned();
    }

    fn request_active_download_stop(&self, workflow_id: WorkflowRunId) {
        let Some(workflow) = self.active_workflows.get(&workflow_id) else {
            return;
        };
        let (Some(child_handle), Some(cancel_requested)) = (
            workflow.download_child.as_ref(),
            workflow.cancel_requested.as_ref(),
        ) else {
            return;
        };
        if workflow.kind == WorkflowKind::PostProcess {
            request_post_process_stop(child_handle, cancel_requested);
        } else {
            request_download_stop(child_handle, cancel_requested);
        }
    }

    pub fn cleanup_active_download_processes(&mut self) {
        let workflows = self
            .active_workflows
            .values()
            .filter(|workflow| {
                matches!(
                    workflow.kind,
                    WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
                )
            })
            .map(|workflow| workflow.workflow_id)
            .collect::<Vec<_>>();
        for workflow_id in workflows {
            self.request_active_download_stop(workflow_id);
        }
    }

    pub fn cleanup_active_tool_install(&mut self) {
        self.pending_dependency_installs.clear();
        if let Some(cancel_handle) = self.tool_install_cancel_handle.take() {
            cancel_handle.cancel();
        }
    }

    pub fn item_is_busy(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        matches!(
            item.metadata_state,
            MetadataState::Queued | MetadataState::Running
        ) || item
            .workflows
            .iter()
            .any(|run| matches!(run.state, WorkflowState::Queued | WorkflowState::Running))
    }

    pub fn item_can_export(&self, item_index: usize, kind: DownloadTargetKind) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };
        if !item.metadata_loaded() || self.item_is_busy(item_index) {
            return false;
        }

        match kind {
            DownloadTargetKind::Video => !item.selection.video_selector.trim().is_empty(),
            DownloadTargetKind::Audio => {
                let (_, format_selector) = self.resolve_download_format_selection(
                    &item.selection.video_selector,
                    &item.selection.audio_selector,
                    item.metadata(),
                );
                !format_selector.trim().is_empty()
            }
            DownloadTargetKind::Subtitle => self
                .subtitle_track_by_id(&item.selection.subtitle_selector, item.metadata())
                .is_some(),
            DownloadTargetKind::Normal => false,
        }
    }

    fn queue_item_index_by_id(&self, item_id: QueueItemId) -> Option<usize> {
        self.queue_items.iter().position(|item| item.id == item_id)
    }

    fn queue_item_by_id(&self, item_id: QueueItemId) -> Option<&QueueItem> {
        self.queue_items.iter().find(|item| item.id == item_id)
    }

    fn queue_item_mut_by_id(&mut self, item_id: QueueItemId) -> Option<&mut QueueItem> {
        self.queue_items.iter_mut().find(|item| item.id == item_id)
    }

    fn should_use_cookies_for_item(&self, item_id: QueueItemId) -> bool {
        self.queue_item_by_id(item_id)
            .map(|item| item.selection.use_cookies)
            .unwrap_or(false)
    }

    fn mark_download_preflight_failed(&mut self, item_id: QueueItemId, error: &str) {
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            item.last_error = Some(error.to_owned());
            item.completed_selection = None;
            if let Some(run) = item.workflows.iter_mut().rev().find(|run| {
                run.kind == WorkflowKind::DownloadMedia
                    && matches!(run.state, WorkflowState::Queued | WorkflowState::Failed)
            }) {
                run.state = WorkflowState::Failed;
                run.progress = 0.0;
                run.error = Some(error.to_owned());
            }
        }
    }

    fn item_metadata(&self, item_index: usize) -> Option<&VideoMetadata> {
        self.queue_items
            .get(item_index)
            .and_then(QueueItem::metadata)
    }

    fn current_picker_metadata(&self) -> &VideoMetadata {
        self.format_picker
            .target_item_id
            .and_then(|index| self.item_metadata(index))
            .unwrap_or(&self.empty_item_preview)
    }

    pub fn item_thumbnail_url(&self, item_index: usize) -> &str {
        self.item_metadata(item_index)
            .map(|metadata| metadata.thumbnail_url.as_str())
            .or_else(|| {
                self.queue_items
                    .get(item_index)
                    .map(|item| item.thumbnail_url.as_str())
            })
            .unwrap_or_default()
    }

    pub fn item_thumbnail_hint(&self, item_index: usize) -> &str {
        self.item_metadata(item_index)
            .map(|metadata| metadata.thumbnail_hint.as_str())
            .or_else(|| {
                self.queue_items
                    .get(item_index)
                    .map(|item| item.thumbnail_hint.as_str())
            })
            .unwrap_or("item.thumbnail")
    }

    pub fn localized_thumbnail_hint<'a>(&self, hint: &'a str) -> std::borrow::Cow<'a, str> {
        match hint {
            "item.thumbnail" => std::borrow::Cow::Borrowed(self.tr("item.thumbnail")),
            "item.thumbnail_preview" => {
                std::borrow::Cow::Borrowed(self.tr("item.thumbnail_preview"))
            }
            _ => std::borrow::Cow::Borrowed(hint),
        }
    }

    pub fn item_duration_text(&self, item_index: usize) -> &str {
        self.item_metadata(item_index)
            .map(|metadata| metadata.duration_text.as_str())
            .or_else(|| {
                self.queue_items
                    .get(item_index)
                    .map(|item| item.duration_text.as_str())
            })
            .unwrap_or_default()
    }

    pub fn poll_thumbnail_work(&mut self, ctx: &eframe::egui::Context) {
        while let Ok(event) = self.thumbnail_result_rx.try_recv() {
            let entry = match event.result {
                Ok(image) => {
                    let texture = ctx.load_texture(
                        thumbnail_texture_id(&event.key),
                        image,
                        eframe::egui::TextureOptions::LINEAR,
                    );
                    ThumbnailCacheEntry::Ready(texture)
                }
                Err(error) => ThumbnailCacheEntry::Failed(error),
            };
            self.thumbnail_cache.insert(event.key, entry);
            ctx.request_repaint();
        }
    }

    pub fn has_loading_thumbnails(&self) -> bool {
        self.thumbnail_cache
            .values()
            .any(|entry| matches!(entry, ThumbnailCacheEntry::Loading))
    }

    pub fn thumbnail_render_source_for_url(
        &mut self,
        ctx: &eframe::egui::Context,
        url: &str,
    ) -> ThumbnailRenderSource {
        let url = url.trim();
        if url.is_empty() {
            return ThumbnailRenderSource::None;
        }

        let Some(proxy_url) = self.tool_paths.effective_proxy_url().map(str::to_owned) else {
            return ThumbnailRenderSource::DirectUrl;
        };

        if !thumbnail_needs_memory_loader(url) {
            return ThumbnailRenderSource::DirectUrl;
        }

        self.poll_thumbnail_work(ctx);
        let no_check_certificates = self.tool_paths.no_check_certificates;
        let key = thumbnail_cache_key(url, &proxy_url, no_check_certificates);
        match self.thumbnail_cache.get(&key) {
            Some(ThumbnailCacheEntry::Ready(texture)) => {
                ThumbnailRenderSource::Texture(texture.clone())
            }
            Some(ThumbnailCacheEntry::Loading) => {
                ctx.request_repaint_after(Duration::from_millis(250));
                ThumbnailRenderSource::Loading
            }
            Some(ThumbnailCacheEntry::Failed(error)) => {
                ThumbnailRenderSource::Failed(error.clone())
            }
            None => {
                self.thumbnail_cache
                    .insert(key.clone(), ThumbnailCacheEntry::Loading);
                run_thumbnail_fetch_worker(
                    key,
                    url.to_owned(),
                    proxy_url,
                    no_check_certificates,
                    self.thumbnail_result_tx.clone(),
                );
                ctx.request_repaint_after(Duration::from_millis(250));
                ThumbnailRenderSource::Loading
            }
        }
    }

    pub fn has_active_work(&self) -> bool {
        self.is_adding_batch
            || !self.active_workflows.is_empty()
            || self.installing_dependency_tool.is_some()
            || !self.pending_dependency_installs.is_empty()
    }

    fn enqueue_item_analysis(&mut self, item_id: QueueItemId, source: String) {
        let workflow_id = self.alloc_workflow_run_id();
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::AnalyzeMetadata,
            ToolKind::YtDlp,
        );
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            item.metadata_state = MetadataState::Queued;
            item.last_error = None;
            let mut run = WorkflowRun::new(
                workflow_id,
                WorkflowKind::AnalyzeMetadata,
                ToolKind::YtDlp,
                WorkflowState::Queued,
            );
            run.detail = source.clone();
            item.workflows.push(run);
            item.metadata_state = MetadataState::Running;
            if let Some(run) = item
                .workflows
                .iter_mut()
                .rev()
                .find(|run| run.kind == WorkflowKind::AnalyzeMetadata)
            {
                run.state = WorkflowState::Running;
                run.detail = source.clone();
            }
        }

        self.last_action = self.trf("state.analyzing_source", &[("{source}", source.as_str())]);
        self.spawn_analyze_worker(
            source,
            Some(item_id),
            Some(workflow_id),
            self.should_use_cookies_for_item(item_id),
        );
    }

    fn spawn_analyze_worker(
        &mut self,
        source: String,
        target_item_id: Option<QueueItemId>,
        workflow_id: Option<WorkflowRunId>,
        use_cookies: bool,
    ) {
        if let Err(error) = self.ensure_yt_dlp_ready() {
            let _ = self.analyze_result_tx.send(AnalyzeResult {
                source,
                target_item_id,
                workflow_id,
                used_cookies: use_cookies,
                result: Err(error),
            });
            return;
        }

        if let Err(error) = self.tool_paths.validate_cookie_setup(use_cookies) {
            let _ = self.analyze_result_tx.send(AnalyzeResult {
                source,
                target_item_id,
                workflow_id,
                used_cookies: use_cookies,
                result: Err(error),
            });
            return;
        }

        let tool_paths = self.tool_paths.clone();
        let tx = self.analyze_result_tx.clone();
        let source_for_worker = source.clone();

        thread::spawn(move || {
            let result = tool_paths.analyze_url(&source_for_worker, use_cookies);
            let _ = tx.send(AnalyzeResult {
                source: source_for_worker,
                target_item_id,
                workflow_id,
                used_cookies: use_cookies,
                result,
            });
        });
    }

    fn disable_missing_aria2_for_request(&self, request: &mut DownloadRequest) -> bool {
        if !request.use_aria2 || request.target_kind == DownloadTargetKind::Subtitle {
            return false;
        }

        if dependency_tool_exists(&self.tool_paths.aria2c) {
            return false;
        }

        request.use_aria2 = false;
        true
    }

    fn start_download_task_at(
        &mut self,
        item_id: QueueItemId,
        emit_json: bool,
    ) -> Result<(), String> {
        let Some(task_index) = self.queue_item_index_by_id(item_id) else {
            let error = self.tr("state.target_download_not_found").to_owned();
            self.last_action = error.clone();
            return Err(error);
        };
        if self.has_running_download_workflow() {
            let error = self.tr("state.download_already_running").to_owned();
            self.last_action = error.clone();
            return Err(error);
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.mark_download_preflight_failed(item_id, &error);
            self.last_action = error.clone();
            return Err(error);
        }

        let Some(item) = self.queue_items.get(task_index) else {
            let error = self.tr("state.analyze_before_download").to_owned();
            self.last_action = error.clone();
            return Err(error);
        };

        let title = item.title.clone();
        let source_url = item.source_url.clone();
        let (
            resolved_audio_selector,
            format_selector,
            resolved_audio_ext,
            subtitle_lang,
            subtitle_ext,
            subtitle_source_ext,
            subtitle_url,
            write_auto_subs,
            subtitle_is_auto_translated,
        ) = if item.metadata_loaded() {
            let (resolved_audio_selector, format_selector) = self
                .resolve_download_format_selection(
                    &item.selection.video_selector,
                    &item.selection.audio_selector,
                    item.metadata(),
                );
            let resolved_audio_ext =
                self.format_extension_by_id(&resolved_audio_selector, item.metadata());
            let subtitle_track = self
                .subtitle_track_by_id(&item.selection.subtitle_selector, item.metadata())
                .cloned();
            let subtitle_lang = subtitle_track
                .as_ref()
                .map(|track| track.download_language_code.clone());
            let subtitle_ext = subtitle_track
                .as_ref()
                .map(|track| track.ext.clone())
                .unwrap_or_default();
            let subtitle_source_ext = subtitle_ext.clone();
            let subtitle_url = subtitle_track.as_ref().map(|track| track.url.clone());
            let write_auto_subs = subtitle_track
                .as_ref()
                .is_some_and(|track| track.source == SubtitleSource::Automatic);
            let subtitle_is_auto_translated = subtitle_track.as_ref().is_some_and(|track| {
                track.source == SubtitleSource::Automatic && track.target_language_code.is_some()
            });
            (
                resolved_audio_selector,
                format_selector,
                resolved_audio_ext,
                subtitle_lang,
                subtitle_ext,
                subtitle_source_ext,
                subtitle_url,
                write_auto_subs,
                subtitle_is_auto_translated,
            )
        } else {
            (
                String::new(),
                String::new(),
                String::new(),
                None,
                String::new(),
                String::new(),
                None,
                false,
                false,
            )
        };

        let mut request = DownloadRequest {
            target_kind: DownloadTargetKind::Normal,
            url: source_url.clone(),
            format_selector,
            video_selector: item.selection.video_selector.clone(),
            audio_selector: resolved_audio_selector,
            is_muxed_video: item.metadata_loaded() && self.item_uses_muxed_video(task_index),
            video_ext: if item.metadata_loaded() {
                self.format_extension_by_id(&item.selection.video_selector, item.metadata())
            } else {
                String::new()
            },
            audio_ext: resolved_audio_ext,
            upload_date: item
                .metadata()
                .map(|metadata| metadata.upload_date_text.clone())
                .unwrap_or_default(),
            subtitle_lang,
            subtitle_ext,
            subtitle_source_ext,
            subtitle_url,
            write_auto_subs,
            subtitle_is_auto_translated,
            write_subtitles: item.selection.write_subtitles,
            embed_subtitles: item.selection.embed_subtitles,
            write_chapters: item.selection.write_chapters,
            embed_chapters: item.selection.embed_chapters,
            write_thumbnail: item.selection.write_thumbnail,
            embed_thumbnail: item.selection.embed_thumbnail,
            use_cookies: self.should_use_cookies_for_item(item_id),
            use_aria2: item.selection.use_aria2,
            emit_json,
            output_path: None,
            output_dir: item.selection.output_dir.clone(),
            file_name: if item.metadata_loaded() {
                item.selection.file_name.clone()
            } else {
                String::new()
            },
            download_sections: item.selection.download_sections.clone(),
        };

        let aria2_fallback = self.disable_missing_aria2_for_request(&mut request);

        if let Err(error) = self.tool_paths.validate_cookie_setup(request.use_cookies) {
            self.mark_download_preflight_failed(item_id, &error);
            self.last_action = error.clone();
            return Err(error);
        }

        let workflow_id = self.alloc_workflow_run_id();
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::DownloadMedia,
            ToolKind::YtDlp,
        );
        if let Some(item) = self.queue_items.get_mut(task_index) {
            reset_item_for_new_work(item, DownloadTargetKind::Normal);
            item.completed_selection = None;
            if let Some(run) = item.workflows.iter_mut().rev().find(|run| {
                run.kind == WorkflowKind::DownloadMedia
                    && matches!(run.state, WorkflowState::Queued | WorkflowState::Failed)
            }) {
                run.id = workflow_id;
                run.state = WorkflowState::Running;
                run.progress = 0.0;
                run.detail = source_url.clone();
                run.output_path = None;
                run.error = None;
            } else {
                let mut run = WorkflowRun::new(
                    workflow_id,
                    WorkflowKind::DownloadMedia,
                    ToolKind::YtDlp,
                    WorkflowState::Running,
                );
                run.detail = source_url.clone();
                item.workflows.push(run);
            }
        }
        self.last_action = if aria2_fallback {
            self.trf(
                "state.downloading_title_aria2_fallback",
                &[("{title}", title.as_str())],
            )
        } else {
            self.trf("state.downloading_title", &[("{title}", title.as_str())])
        };

        let tool_paths = self.tool_paths.clone();
        let tx = self.download_result_tx.clone();
        let child_handle = Arc::new(Mutex::new(None));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.attach_active_download_process(
            workflow_id,
            child_handle.clone(),
            cancel_requested.clone(),
        );

        thread::spawn(move || {
            run_download_worker(
                tool_paths,
                request,
                item_id,
                workflow_id,
                WorkflowKind::DownloadMedia,
                tx,
                child_handle,
                cancel_requested,
            );
        });

        Ok(())
    }

    pub fn item_export_initial_directory(&self, item_index: usize) -> Option<PathBuf> {
        let item = self.queue_items.get(item_index)?;
        resolve_output_dir(&item.selection.output_dir).ok()
    }

    pub fn item_export_default_name(
        &self,
        item_index: usize,
        kind: DownloadTargetKind,
    ) -> Option<String> {
        let item = self.queue_items.get(item_index)?;
        let base_name = if item.selection.file_name.trim().is_empty() {
            sanitize_file_name_for_windows(item.title.trim())
        } else {
            sanitize_file_name_for_windows(item.selection.file_name.trim())
        };
        let default_ext = self.item_export_default_extension(item_index, kind)?;
        Some(format!("{base_name}.{default_ext}"))
    }

    pub fn item_export_default_extension(
        &self,
        item_index: usize,
        kind: DownloadTargetKind,
    ) -> Option<String> {
        let item = self.queue_items.get(item_index)?;
        let metadata = item.metadata()?;
        match kind {
            DownloadTargetKind::Video => {
                let ext =
                    self.format_extension_by_id(&item.selection.video_selector, Some(metadata));
                normalized_export_extension(&ext).or_else(|| Some("mkv".to_owned()))
            }
            DownloadTargetKind::Audio => {
                let resolved_audio_selector = self
                    .resolve_download_format_selection(
                        &item.selection.video_selector,
                        &item.selection.audio_selector,
                        Some(metadata),
                    )
                    .0;
                let codec = self.format_codec_by_id(&resolved_audio_selector, Some(metadata));
                normalized_export_extension(&codec)
                    .or_else(|| {
                        let ext =
                            self.format_extension_by_id(&resolved_audio_selector, Some(metadata));
                        normalized_export_extension(&ext)
                    })
                    .or_else(|| Some("m4a".to_owned()))
            }
            DownloadTargetKind::Subtitle => Some("srt".to_owned()),
            DownloadTargetKind::Normal => None,
        }
    }

    pub fn start_item_export(
        &mut self,
        item_id: QueueItemId,
        kind: DownloadTargetKind,
        output_path: String,
    ) -> Result<(), String> {
        let Some(item_index) = self.queue_item_index_by_id(item_id) else {
            return Err(self.tr("state.target_export_not_found").to_owned());
        };
        if !self.item_can_export(item_index, kind) {
            return Err(self.tr("state.cannot_export_item").to_owned());
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            if let Some(item) = self.queue_items.get_mut(item_index) {
                item.last_error = Some(error.clone());
            }
            self.last_action = error.clone();
            return Err(error);
        }

        let Some(item) = self.queue_items.get(item_index) else {
            return Err(self.tr("state.target_export_not_found").to_owned());
        };
        let Some(metadata) = item.metadata() else {
            return Err(self.tr("state.analyze_before_export").to_owned());
        };
        let item_title = item.title.clone();
        let source_url = item.source_url.clone();
        let selected_video = item.selection.video_selector.clone();
        let selected_audio = item.selection.audio_selector.clone();
        let selected_subtitle_track = self
            .subtitle_track_by_id(&item.selection.subtitle_selector, Some(metadata))
            .cloned();
        let item_use_aria2 = item.selection.use_aria2;
        let item_write_thumbnail = item.selection.write_thumbnail;
        let item_embed_thumbnail = item.selection.embed_thumbnail;

        let (
            subtitle_lang,
            subtitle_ext,
            subtitle_source_ext,
            subtitle_url,
            write_auto_subs,
            subtitle_is_auto_translated,
        ) = if kind == DownloadTargetKind::Subtitle {
            let Some(track) = selected_subtitle_track.as_ref() else {
                return Err(self.tr("state.choose_subtitles_before_export").to_owned());
            };
            (
                Some(track.download_language_code.clone()),
                track.ext.clone(),
                track.ext.clone(),
                Some(track.url.clone()),
                track.source == SubtitleSource::Automatic,
                track.source == SubtitleSource::Automatic && track.target_language_code.is_some(),
            )
        } else {
            (None, String::new(), String::new(), None, false, false)
        };

        let target_path = normalize_export_target_path(
            &output_path,
            self.item_export_default_extension(item_index, kind)
                .as_deref(),
        );
        let export_ext = Path::new(&target_path)
            .extension()
            .and_then(|value| value.to_str())
            .and_then(normalized_export_extension)
            .ok_or_else(|| self.tr("state.specify_file_extension").to_owned())?;
        validate_export_extension(self.language(), kind, &export_ext)?;

        let (audio_selector, _) = self.resolve_download_format_selection(
            &selected_video,
            &selected_audio,
            Some(metadata),
        );
        let resolved_audio_ext = self.format_extension_by_id(&audio_selector, Some(metadata));
        let mut request = DownloadRequest {
            target_kind: kind,
            url: source_url.clone(),
            format_selector: match kind {
                DownloadTargetKind::Video => selected_video.clone(),
                DownloadTargetKind::Audio => audio_selector.clone(),
                DownloadTargetKind::Normal | DownloadTargetKind::Subtitle => String::new(),
            },
            video_selector: if kind == DownloadTargetKind::Subtitle {
                String::new()
            } else {
                selected_video.clone()
            },
            audio_selector: if kind == DownloadTargetKind::Subtitle {
                String::new()
            } else {
                audio_selector
            },
            is_muxed_video: false,
            video_ext: if kind == DownloadTargetKind::Subtitle {
                String::new()
            } else {
                self.format_extension_by_id(&selected_video, Some(metadata))
            },
            audio_ext: if kind == DownloadTargetKind::Subtitle {
                String::new()
            } else {
                resolved_audio_ext
            },
            upload_date: metadata.upload_date_text.clone(),
            subtitle_lang,
            subtitle_ext: if kind == DownloadTargetKind::Subtitle {
                export_ext.clone()
            } else {
                subtitle_ext
            },
            subtitle_source_ext,
            subtitle_url,
            write_auto_subs,
            subtitle_is_auto_translated,
            write_subtitles: false,
            embed_subtitles: false,
            write_chapters: false,
            embed_chapters: false,
            write_thumbnail: matches!(kind, DownloadTargetKind::Video) && item_write_thumbnail,
            embed_thumbnail: matches!(kind, DownloadTargetKind::Video) && item_embed_thumbnail,
            use_cookies: self.should_use_cookies_for_item(item_id),
            use_aria2: kind != DownloadTargetKind::Subtitle && item_use_aria2,
            emit_json: false,
            output_path: Some(target_path.clone()),
            output_dir: String::new(),
            file_name: String::new(),
            download_sections: item.selection.download_sections.clone(),
        };

        let aria2_fallback = self.disable_missing_aria2_for_request(&mut request);

        if let Err(error) = self.tool_paths.validate_cookie_setup(request.use_cookies) {
            if let Some(item) = self.queue_items.get_mut(item_index) {
                item.last_error = Some(error.clone());
            }
            return Err(error);
        }

        let workflow_id = self.alloc_workflow_run_id();
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::ExportMedia,
            ToolKind::YtDlp,
        );
        if let Some(item) = self.queue_items.get_mut(item_index) {
            reset_item_for_new_work(item, kind);
            let mut run = WorkflowRun::new(
                workflow_id,
                WorkflowKind::ExportMedia,
                ToolKind::YtDlp,
                WorkflowState::Running,
            );
            run.detail = target_path.clone();
            item.workflows.push(run);
            item.last_error = None;
        }

        let action_text = match kind {
            DownloadTargetKind::Video => {
                self.trf("state.exporting_video", &[("{title}", item_title.as_str())])
            }
            DownloadTargetKind::Audio => {
                self.trf("state.exporting_audio", &[("{title}", item_title.as_str())])
            }
            DownloadTargetKind::Normal => self.trf(
                "state.downloading_title",
                &[("{title}", item_title.as_str())],
            ),
            DownloadTargetKind::Subtitle => self.trf(
                "state.exporting_subtitles",
                &[("{title}", item_title.as_str())],
            ),
        };
        self.last_action = if aria2_fallback {
            self.trf(
                "state.action_aria2_fallback",
                &[("{action}", action_text.as_str())],
            )
        } else {
            action_text
        };

        let tool_paths = self.tool_paths.clone();
        let tx = self.download_result_tx.clone();
        let child_handle = Arc::new(Mutex::new(None));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.attach_active_download_process(
            workflow_id,
            child_handle.clone(),
            cancel_requested.clone(),
        );
        thread::spawn(move || {
            run_download_worker(
                tool_paths,
                request,
                item_id,
                workflow_id,
                WorkflowKind::ExportMedia,
                tx,
                child_handle,
                cancel_requested,
            );
        });
        Ok(())
    }

    pub fn clear_queue(&mut self) {
        self.queue_items.clear();
        self.last_action = self.tr("state.cleared_queue").to_owned();
    }

    pub fn remove_queue_item(&mut self, item_id: QueueItemId) {
        let Some(index) = self.queue_item_index_by_id(item_id) else {
            return;
        };

        if self.item_is_busy(index) {
            self.last_action = self.tr("state.cannot_remove_running_item").to_owned();
            return;
        }

        let removed = self.queue_items.remove(index);
        let removed_source_key = canonical_queue_source_key(&removed.source_url);
        self.batch_input = self
            .batch_input
            .lines()
            .map(str::trim)
            .filter(|line| {
                !line.is_empty() && canonical_queue_source_key(line) != removed_source_key
            })
            .collect::<Vec<_>>()
            .join("\n");
        self.last_action = self.trf("state.removed_item", &[("{title}", removed.title.as_str())]);
    }

    pub fn primary_candidate_url(&self) -> Option<String> {
        let direct = self.url_input.trim();
        if !direct.is_empty() {
            return Some(direct.to_owned());
        }

        self.batch_input
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(ToOwned::to_owned)
    }

    fn parsed_batch_urls(&self) -> Vec<String> {
        self.batch_input
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    fn append_batch_seed(&mut self, _source: &str, seed: PlaylistEntrySeed) {
        let source_key = canonical_queue_source_key(&seed.source_url);
        if self
            .queue_items
            .iter()
            .any(|item| canonical_queue_source_key(&item.source_url) == source_key)
        {
            return;
        }

        let source_url = seed.source_url.clone();
        let item = self.build_queue_item_from_seed(seed);
        self.queue_items.push(item);

        if !self
            .batch_input
            .lines()
            .map(str::trim)
            .any(|line| canonical_queue_source_key(line) == source_key)
        {
            self.batch_input_push_unique(&source_url);
        }
    }

    fn build_queue_item_from_url(&mut self, url: &str) -> QueueItem {
        let title = infer_title(
            url,
            self.tr("state.untitled_task"),
            self.tr("state.imported_source"),
        );
        let mut item = QueueItem::new(self.alloc_queue_item_id(), url, title);
        item.selection.quality = self.item_defaults.quality;
        item.selection.write_thumbnail = self.item_defaults.write_thumbnail;
        item.selection.embed_thumbnail = self.item_defaults.embed_thumbnail;
        item.selection.write_subtitles = self.item_defaults.write_subtitles;
        item.selection.embed_subtitles = self.item_defaults.embed_subtitles;
        item.selection.write_chapters = self.item_defaults.write_chapters;
        item.selection.embed_chapters = self.item_defaults.embed_chapters;
        item.selection.use_cookies = self.item_defaults.use_cookies;
        item.selection.use_aria2 = self.item_defaults.use_aria2;
        item.selection.output_dir = self.item_defaults.output_dir.clone();
        item.selection.download_sections.clear();
        item
    }

    fn build_queue_item_from_seed(&mut self, seed: PlaylistEntrySeed) -> QueueItem {
        let mut item = self.build_queue_item_from_url(&seed.source_url);
        if !seed.title.trim().is_empty() {
            item.title = seed.title;
        }
        item.thumbnail_hint = seed.thumbnail_hint;
        item.thumbnail_url = seed.thumbnail_url;
        item.duration_text = seed.duration_text;
        item.metadata_state = MetadataState::Idle;
        if item.selection.file_name.trim().is_empty() {
            item.selection.file_name = sanitize_file_name_for_windows(item.title.trim());
        }
        item
    }

    pub fn queue_summary(&self) -> QueueSummary {
        let mut summary = QueueSummary::default();
        summary.total = self.queue_items.len();

        for item in &self.queue_items {
            match item_summary_bucket(item) {
                QueueSummaryBucket::Queued => summary.queued += 1,
                QueueSummaryBucket::Completed => summary.completed += 1,
                QueueSummaryBucket::Failed => summary.failed += 1,
            }
        }

        summary
    }

    pub fn has_pending_download_items(&self) -> bool {
        !self.has_running_download_workflow()
            && self.queue_items.iter().any(item_can_enter_download_queue)
    }

    pub fn required_dependency_notice(&self) -> Option<String> {
        self.ensure_yt_dlp_ready().err()
    }

    fn ensure_yt_dlp_ready(&self) -> Result<(), String> {
        self.tool_paths.validate_yt_dlp_available().map(|_| ())
    }

    pub fn available_quality_presets(&self) -> [QualityPreset; 4] {
        [
            QualityPreset::Best,
            QualityPreset::P1080,
            QualityPreset::P720,
            QualityPreset::AudioOnly,
        ]
    }

    pub fn resolved_output_dir_display(&self) -> String {
        if self.output_dir_locked_by_config() {
            return self.tr("state.controlled_by_config").to_owned();
        }
        let path = self.item_defaults.output_dir.as_str();
        resolve_output_dir(path)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| path.to_owned())
    }

    pub fn output_dir_display(&self) -> String {
        if self.output_dir_locked_by_config() {
            return self.tr("state.controlled_by_config").to_owned();
        }
        let path = self.item_defaults.output_dir.as_str();
        display_output_dir(path)
    }

    pub fn language(&self) -> Language {
        self.config.language.resolve()
    }

    pub fn language_selection(&self) -> LanguageSelection {
        self.config.language
    }

    pub fn language_selection_display_name(&self) -> String {
        match self.language_selection() {
            LanguageSelection::Auto => format!(
                "{} ({})",
                LanguageSelection::Auto.native_name(),
                self.language().native_name()
            ),
            language => language.native_name().to_owned(),
        }
    }

    pub fn tr(&self, key: &'static str) -> &'static str {
        i18n::text(self.language(), key)
    }

    pub fn trf(&self, key: &'static str, args: &[(&str, &str)]) -> String {
        i18n::format_text(self.language(), key, args)
    }

    pub fn localize_message(&self, value: &str) -> String {
        i18n::localize_message(self.language(), value)
    }

    pub fn set_language_selection(&mut self, language: LanguageSelection) {
        if self.config.language == language {
            return;
        }
        self.config.language = language;
        let _ = self.config.save();
    }

    pub fn open_options_detail_page(&mut self, page: OptionsDetailPage) {
        self.options_detail_page = Some(page);
    }

    pub fn close_options_detail_page(&mut self) {
        self.options_detail_page = None;
    }

    pub fn open_prepare_detail_page(&mut self, page: PrepareDetailPage) {
        self.prepare_detail_page = Some(page);
    }

    pub fn close_prepare_detail_page(&mut self) {
        self.prepare_detail_page = None;
    }

    pub fn open_processing_detail_page(&mut self, page: ProcessingDetailPage) {
        self.processing_detail_page = Some(page);
    }

    pub fn close_processing_detail_page(&mut self) {
        self.processing_detail_page = None;
    }

    pub fn set_last_action_message(&mut self, message: impl Into<String>) {
        self.last_action = message.into();
    }

    pub fn set_windows_toast_enabled(&mut self, enabled: bool) {
        self.config.windows_toast_enabled = enabled;
        let _ = self.config.save();
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        if self.config.theme_mode == mode {
            return;
        }
        self.config.theme_mode = mode;
        let _ = self.config.save();
    }

    pub fn set_theme_accent_color(&mut self, color: ThemeAccentColor) {
        if self.config.theme_accent_color == color {
            return;
        }
        self.config.theme_accent_color = color;
        let _ = self.config.save();
    }

    pub fn set_enable_processing_tab(&mut self, enabled: bool) {
        self.config.enable_processing_tab = enabled;
        if !enabled && self.active_tab == AppTab::Processing {
            self.active_tab = AppTab::Options;
        }
        let _ = self.config.save();
    }

    pub fn set_show_log_tab(&mut self, enabled: bool) {
        self.config.show_log_tab = enabled;
        if !enabled && self.active_tab == AppTab::Log {
            self.active_tab = AppTab::Options;
        }
        let _ = self.config.save();
    }

    pub fn set_transcode_intent(
        &mut self,
        settings: crate::infrastructure::TranscodeIntentSettings,
    ) {
        let settings = settings.normalized();
        if self.config.transcode_intent == settings {
            return;
        }
        self.config.transcode_intent = settings;
        let _ = self.config.save();
    }

    fn send_download_result_windows_toast(&self, title: String, result: Result<String, String>) {
        if !self.config.windows_toast_enabled {
            return;
        }
        let language = self.language();

        thread::spawn(move || {
            let result = match result {
                Ok(output_path) => send_download_finished_windows_toast(
                    language,
                    title.as_str(),
                    (!output_path.trim().is_empty()).then_some(output_path.as_str()),
                ),
                Err(error) => {
                    if error == DOWNLOAD_CANCELLED_MESSAGE {
                        Ok(())
                    } else {
                        send_download_failed_windows_toast(language, title.as_str(), error.as_str())
                    }
                }
            };

            if let Err(error) = result {
                eprintln!("[notification] Windows Toast failed: {error}");
            }
        });
    }

    pub fn set_output_dir(&mut self, path: impl Into<String>) {
        if self.output_dir_locked_by_config() {
            return;
        }
        let path = path.into();
        self.item_defaults.output_dir = path.clone();
        for item in &mut self.queue_items {
            item.selection.output_dir = path.clone();
        }
        self.config.set_download_dir(path);
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn output_dir_locked_by_config(&self) -> bool {
        self.tool_paths.effective_config_owns_output()
    }

    pub fn output_dir_config_source_display(&self) -> Option<String> {
        self.tool_paths
            .effective_config_path()
            .map(|path| path.display().to_string())
    }

    pub fn available_cache_location_modes(&self) -> [CacheLocationMode; 3] {
        [
            CacheLocationMode::YtDlpDefault,
            CacheLocationMode::V2Cache,
            CacheLocationMode::WindowsTemp,
        ]
    }

    pub fn set_cache_location_mode(&mut self, mode: CacheLocationMode) {
        self.tool_paths.cache_mode = mode;
        self.config.cache_location_mode = match mode {
            CacheLocationMode::YtDlpDefault => SerializableCacheLocationMode::YtDlpDefault,
            CacheLocationMode::V2Cache => SerializableCacheLocationMode::V2Cache,
            CacheLocationMode::WindowsTemp => SerializableCacheLocationMode::WindowsTemp,
        };
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn should_show_prepare_tab(&self) -> bool {
        !self.config.prepare_skipped
            && !self.prepare_tab_snoozed
            && self.prepare_report.should_show_tab()
    }

    pub fn prepare_requirements(&self) -> &[PrepareRequirement] {
        &self.prepare_report.requirements
    }

    pub fn refresh_prepare_report(&mut self) {
        self.prepare_report =
            collect_prepare_report(&self.tool_paths, &self.item_defaults.output_dir);
        let actionable_tools = self
            .prepare_report
            .requirements
            .iter()
            .filter(|item| item.needs_attention())
            .filter_map(|item| match item.action {
                Some(PrepareAction::InstallTool(tool)) => Some(tool),
                None => None,
            })
            .collect::<Vec<_>>();
        self.prepare_tool_selection
            .retain(|tool| actionable_tools.contains(tool));
        if !self.should_show_prepare_tab() && self.active_tab == AppTab::Prepare {
            self.active_tab = AppTab::Main;
        }
    }

    pub fn reset_prepare_tool_selection_to_defaults(&mut self) {
        self.prepare_tool_selection = self.prepare_report.default_selected_tools();
    }

    pub fn prepare_tool_is_selected(&self, tool: DependencyTool) -> bool {
        self.prepare_tool_selection.contains(&tool)
    }

    pub fn set_prepare_tool_selected(&mut self, tool: DependencyTool, selected: bool) {
        if selected {
            if !self.prepare_tool_selection.contains(&tool) {
                self.prepare_tool_selection.push(tool);
            }
        } else {
            self.prepare_tool_selection
                .retain(|selected_tool| *selected_tool != tool);
        }
    }

    pub fn selected_prepare_install_count(&self) -> usize {
        self.prepare_selected_tools_to_install().len()
    }

    pub fn prepare_installable_tool_count(&self) -> usize {
        self.prepare_tools_to_install_all().len()
    }

    pub fn prepare_dependency_install_block_reason(&self) -> Option<String> {
        let blocking_issue = self.prepare_report.requirements.iter().find(|item| {
            item.action.is_none()
                && item.status == PrepareStatus::Failed
                && matches!(
                    item.id.as_str(),
                    "app-root" | "config-file" | "tools-dir" | "tool-install-cache"
                )
        })?;

        Some(self.trf(
            "state.install_blocked_by_prepare",
            &[("{items}", blocking_issue.title.as_str())],
        ))
    }

    pub fn install_all_prepare_tools(&mut self) {
        if let Some(active) = self.installing_dependency_tool {
            self.last_action = self.trf(
                "state.tool_deployment_running",
                &[("{tool}", active.label())],
            );
            return;
        }

        if let Some(reason) = self.prepare_dependency_install_block_reason() {
            self.last_action = reason;
            return;
        }

        let tools = self.prepare_tools_to_install_all();
        if tools.is_empty() {
            self.last_action = self.tr("state.no_tools_to_install").to_owned();
            return;
        }

        self.pending_dependency_installs = tools.into_iter().collect();
        if let Some(tool) = self.pending_dependency_installs.pop_front() {
            self.begin_dependency_install(tool);
        }
    }

    pub fn install_selected_prepare_tools(&mut self) {
        if let Some(active) = self.installing_dependency_tool {
            self.last_action = self.trf(
                "state.tool_deployment_running",
                &[("{tool}", active.label())],
            );
            return;
        }

        if let Some(reason) = self.prepare_dependency_install_block_reason() {
            self.last_action = reason;
            return;
        }

        let tools = self.prepare_selected_tools_to_install();
        if tools.is_empty() {
            self.last_action = self.tr("state.no_selected_tools_to_install").to_owned();
            return;
        }

        self.pending_dependency_installs = tools.into_iter().collect();
        if let Some(tool) = self.pending_dependency_installs.pop_front() {
            self.begin_dependency_install(tool);
        }
    }

    pub fn snooze_prepare_tab(&mut self) {
        let previous_prepare_skipped = self.config.prepare_skipped;
        self.config.prepare_skipped = true;

        match self.config.save() {
            Ok(()) => {
                self.prepare_tab_snoozed = true;
                if self.active_tab == AppTab::Prepare {
                    self.active_tab = AppTab::Main;
                }
                self.last_action = self.tr("state.prepare_skipped").to_owned();
            }
            Err(error) => {
                self.config.prepare_skipped = previous_prepare_skipped;
                self.prepare_tab_snoozed = false;
                let localized_error = self.localize_message(&error);
                self.last_action = self.trf(
                    "state.skip_failed",
                    &[("{error}", localized_error.as_str())],
                );
                self.refresh_prepare_report();
            }
        }
    }

    pub fn reopen_prepare_tab(&mut self) {
        self.prepare_tab_snoozed = false;
        self.config.prepare_skipped = false;
        let _ = self.config.save();
        self.refresh_prepare_report();
        if self.should_show_prepare_tab() {
            self.active_tab = AppTab::Prepare;
        }
    }

    fn prepare_selected_tools_to_install(&self) -> Vec<DependencyTool> {
        self.prepare_install_order()
            .into_iter()
            .filter(|tool| self.prepare_tool_selection.contains(tool))
            .filter(|tool| self.prepare_tool_needs_install(*tool))
            .collect()
    }

    fn prepare_tools_to_install_all(&self) -> Vec<DependencyTool> {
        self.prepare_install_order()
            .into_iter()
            .filter(|tool| self.prepare_tool_needs_install(*tool))
            .collect()
    }

    fn prepare_tool_needs_install(&self, tool: DependencyTool) -> bool {
        self.prepare_report
            .requirements
            .iter()
            .any(|item| item.needs_attention() && item.has_install_action(tool))
    }

    fn prepare_install_order(&self) -> [DependencyTool; 3] {
        [
            DependencyTool::YtDlp,
            DependencyTool::Deno,
            DependencyTool::Ffmpeg,
        ]
    }

    pub fn set_yt_dlp_path(&mut self, path: impl Into<String>) {
        self.config.set_yt_dlp_path(path);
        self.tool_paths.yt_dlp = self.config.yt_dlp_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn set_yt_dlp_config_path(&mut self, path: impl Into<String>) {
        self.config.set_yt_dlp_config_path(path);
        self.tool_paths.yt_dlp_config = self.config.yt_dlp_config_path.clone();
        let _ = self.config.save();
    }

    pub fn available_yt_dlp_config_files(&self) -> Vec<ConfigFileOption> {
        available_yt_dlp_config_files()
    }

    pub fn yt_dlp_configs_dir_display(&self) -> String {
        yt_dlp_configs_dir_display()
    }

    pub fn set_ffmpeg_path(&mut self, path: impl Into<String>) {
        self.config.set_ffmpeg_path(path);
        self.tool_paths.ffmpeg = self.config.ffmpeg_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn set_aria2c_path(&mut self, path: impl Into<String>) {
        self.config.set_aria2c_path(path);
        self.tool_paths.aria2c = self.config.aria2c_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn set_deno_path(&mut self, path: impl Into<String>) {
        self.config.set_deno_path(path);
        self.tool_paths.deno = self.config.deno_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn install_dependency_tool(&mut self, tool: DependencyTool) {
        if let Some(active) = self.installing_dependency_tool {
            self.last_action = self.trf(
                "state.tool_deployment_running",
                &[("{tool}", active.label())],
            );
            return;
        }
        if self.active_tab == AppTab::Prepare {
            if let Some(reason) = self.prepare_dependency_install_block_reason() {
                self.last_action = reason;
                return;
            }
        }
        self.pending_dependency_installs.clear();
        self.begin_dependency_install(tool);
    }

    fn begin_dependency_install(&mut self, tool: DependencyTool) {
        self.installing_dependency_tool = Some(tool);
        self.dependency_install_progress.insert(
            tool,
            ToolInstallProgress {
                tool,
                stage: ToolInstallStage::Preparing,
                percent: None,
                message: self.tr("state.preparing_deployment").to_owned(),
            },
        );
        self.last_action = self.trf(
            "state.tool_downloading_deploying",
            &[("{tool}", tool.label())],
        );
        let proxy_url = self.tool_paths.effective_proxy_url().map(str::to_owned);
        self.tool_install_cancel_handle = Some(run_tool_install_worker(
            tool,
            proxy_url,
            self.tool_install_result_tx.clone(),
        ));
    }

    pub fn installing_dependency_tool(&self) -> Option<DependencyTool> {
        self.installing_dependency_tool
    }

    pub fn dependency_tool_path(&self, tool: DependencyTool) -> &str {
        match tool {
            DependencyTool::YtDlp => &self.tool_paths.yt_dlp,
            DependencyTool::Ffmpeg => &self.tool_paths.ffmpeg,
            DependencyTool::Aria2c => &self.tool_paths.aria2c,
            DependencyTool::Deno => &self.tool_paths.deno,
        }
    }

    pub fn dependency_tool_is_installed(&self, tool: DependencyTool) -> bool {
        dependency_tool_is_available(tool, self.dependency_tool_path(tool))
    }

    pub fn dependency_tool_status_text(&self, tool: DependencyTool) -> String {
        if let Some(progress) = self.dependency_install_progress.get(&tool) {
            if self.installing_dependency_tool == Some(tool)
                || matches!(progress.stage, ToolInstallStage::Failed)
            {
                return progress_status_text(self.language(), progress);
            }
        }
        if self.dependency_tool_is_installed(tool) {
            self.tr("state.found").to_owned()
        } else {
            self.tr("state.not_found").to_owned()
        }
    }

    pub fn dependency_tool_install_progress(
        &self,
        tool: DependencyTool,
    ) -> Option<&ToolInstallProgress> {
        self.dependency_install_progress.get(&tool)
    }

    pub fn set_proxy_enabled(&mut self, enabled: bool) {
        self.config.proxy_enabled = enabled;
        self.tool_paths.proxy_enabled = enabled;
        let _ = self.config.save();
    }

    pub fn set_proxy_url(&mut self, value: impl Into<String>) {
        self.config.set_proxy_url(value);
        self.tool_paths.proxy_url = self.config.proxy_url.clone();
        self.tool_paths.proxy_enabled = self.config.proxy_enabled;
        let _ = self.config.save();
    }

    pub fn set_no_check_certificates(&mut self, enabled: bool) {
        self.config.no_check_certificates = enabled;
        self.tool_paths.no_check_certificates = enabled;
        let _ = self.config.save();
    }

    pub fn set_limit_rate(&mut self, value: impl Into<String>) {
        self.config.set_limit_rate(value);
        self.tool_paths.limit_rate = self.config.limit_rate.clone();
        let _ = self.config.save();
    }

    pub fn set_download_sections(&mut self, value: impl Into<String>) {
        self.config.set_download_sections(value);
        self.tool_paths.download_sections = self.config.download_sections.clone();
        let _ = self.config.save();
    }

    pub fn set_file_time_mode(&mut self, mode: FileTimeMode) {
        self.config.file_time_mode = mode;
        self.tool_paths.file_time_mode = mode;
        let _ = self.config.save();
    }

    pub fn set_auto_analyze(&mut self, enabled: bool) {
        self.config.auto_analyze = enabled;
        let _ = self.config.save();
    }

    pub fn set_keep_window_on_top(&mut self, enabled: bool) {
        self.config.keep_window_on_top = enabled;
        let _ = self.config.save();
    }

    pub fn pending_ui_scale_percent(&self) -> u16 {
        self.pending_ui_scale_percent
    }

    pub fn set_pending_ui_scale_percent(&mut self, value: u16) {
        self.pending_ui_scale_percent = normalize_ui_scale_percent(value);
    }

    pub fn ui_scale_has_pending_change(&self) -> bool {
        self.pending_ui_scale_percent != self.config.ui_scale_percent
    }

    pub fn apply_pending_ui_scale_percent(&mut self) {
        self.config.ui_scale_percent = self.pending_ui_scale_percent;
        let _ = self.config.save();
    }

    pub fn set_ui_scale_percent(&mut self, value: u16) {
        let normalized = normalize_ui_scale_percent(value);
        self.pending_ui_scale_percent = normalized;
        self.config.ui_scale_percent = normalized;
        let _ = self.config.save();
    }

    pub fn set_remember_window_position(&mut self, enabled: bool) {
        self.config.remember_window_position = enabled;
        if !enabled {
            self.config.window_position = None;
        }
        let _ = self.config.save();
    }

    pub fn set_remember_window_size(&mut self, enabled: bool) {
        self.config.remember_window_size = enabled;
        if !enabled {
            self.config.window_size = None;
        }
        let _ = self.config.save();
    }

    pub fn sync_window_state(&mut self, ctx: &eframe::egui::Context) {
        if !self.config.remember_window_position && !self.config.remember_window_size {
            return;
        }

        let viewport = ctx.input(|input| input.viewport().clone());
        if viewport.minimized.unwrap_or(false) || viewport.maximized.unwrap_or(false) {
            return;
        }

        let mut changed = false;
        if self.config.remember_window_position {
            if let Some(outer_rect) = viewport.outer_rect {
                if let Some(position) = WindowPosition::new(outer_rect.min.x, outer_rect.min.y) {
                    if self.config.window_position != Some(position) {
                        self.config.window_position = Some(position);
                        changed = true;
                    }
                }
            }
        }

        if self.config.remember_window_size {
            if let Some(inner_rect) = viewport.inner_rect {
                let size = inner_rect.size();
                if let Some(window_size) = WindowSize::new(size.x, size.y) {
                    if self.config.window_size != Some(window_size) {
                        self.config.window_size = Some(window_size);
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            return;
        }

        let _ = self.config.save();
    }

    pub fn set_batch_limit_enabled(&mut self, enabled: bool) {
        self.config.batch_limit_enabled = enabled;
        let _ = self.config.save();
    }

    pub fn set_direct_download_on_add(&mut self, enabled: bool) {
        self.config.direct_download_on_add = enabled;
        let _ = self.config.save();
    }

    pub fn set_output_file_action_mode(&mut self, mode: OutputFileActionMode) {
        self.config.output_file_action_mode = mode;
        let _ = self.config.save();
    }

    pub fn set_batch_limit_count(&mut self, count: usize) {
        self.config.batch_limit_count = count.max(1);
        let _ = self.config.save();
    }

    pub fn set_monitor_clipboard(&mut self, enabled: bool) {
        self.monitor_clipboard = enabled;
        self.config.auto_paste_clipboard = enabled;
        let _ = self.config.save();
        if enabled {
            self.last_clipboard_text = read_clipboard_text().unwrap_or_default();
            self.last_clipboard_check = Some(Instant::now());
            self.last_action = if self.config.clipboard_auto_add {
                self.tr("state.clipboard_monitor_enabled_auto_add")
                    .to_owned()
            } else {
                self.tr("state.clipboard_monitor_enabled_fill").to_owned()
            };
        } else {
            self.last_action = self.tr("state.clipboard_monitor_disabled").to_owned();
        }
    }

    pub fn set_clipboard_auto_add(&mut self, enabled: bool) {
        self.config.clipboard_auto_add = enabled;
        let _ = self.config.save();
        if self.monitor_clipboard {
            self.last_clipboard_text = read_clipboard_text().unwrap_or_default();
            self.last_clipboard_check = Some(Instant::now());
            self.last_action = if enabled {
                self.tr("state.clipboard_will_auto_add").to_owned()
            } else {
                self.tr("state.clipboard_will_fill_only").to_owned()
            };
        }
    }

    pub fn set_youtube_high_risk_playlist_prompt(&mut self, enabled: bool) {
        self.config.youtube_high_risk_playlist_prompt = enabled;
        let _ = self.config.save();
    }

    pub fn set_youtube_video_playlist_mode(&mut self, mode: YoutubeVideoPlaylistMode) {
        self.config.youtube_video_playlist_mode = mode;
        let _ = self.config.save();
    }

    pub fn available_browser_cookie_sources(
        &self,
    ) -> Vec<crate::infrastructure::BrowserCookieSourceOption> {
        self.tool_paths.available_browser_cookie_sources()
    }

    pub fn available_browser_cookie_profiles(
        &self,
    ) -> Vec<crate::infrastructure::BrowserCookieProfileOption> {
        self.tool_paths.available_browser_cookie_profiles()
    }

    pub fn set_browser_cookie_source(&mut self, source: impl Into<String>) {
        let source = source.into();
        self.tool_paths.browser_cookie_source = source.clone();
        self.config.browser_cookie_source = source;
        let profiles = self.tool_paths.available_browser_cookie_profiles();
        if self.cookie_source_uses_file()
            || (!self.tool_paths.browser_cookie_profile.trim().is_empty()
                && !profiles
                    .iter()
                    .any(|option| option.value == self.tool_paths.browser_cookie_profile))
        {
            self.tool_paths.browser_cookie_profile.clear();
            self.config.browser_cookie_profile.clear();
        }
        let _ = self.config.save();
    }

    pub fn set_browser_cookie_profile(&mut self, profile: impl Into<String>) {
        let profile = profile.into();
        self.tool_paths.browser_cookie_profile = profile.clone();
        self.config.browser_cookie_profile = profile;
        let _ = self.config.save();
    }

    pub fn set_browser_cookie_file(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.tool_paths.browser_cookie_file = path.clone();
        self.config.browser_cookie_file = path;
        let _ = self.config.save();
    }

    pub fn cookie_source_uses_file(&self) -> bool {
        self.tool_paths
            .browser_cookie_source
            .trim()
            .eq_ignore_ascii_case("file")
    }

    pub fn available_concurrent_fragment_values(&self) -> [usize; 4] {
        [1, 2, 4, 8]
    }

    pub fn set_concurrent_fragments(&mut self, value: usize) {
        let value = match value {
            1 | 2 | 4 | 8 => value,
            0 => 1,
            3 => 4,
            5..=7 => 8,
            _ => 8,
        };
        self.tool_paths.concurrent_fragments = value;
        self.config.concurrent_fragments = value;
        let _ = self.config.save();
    }

    pub fn set_youtube_subs_po_token(&mut self, token: impl Into<String>) {
        let token = token.into();
        self.tool_paths.youtube_subs_po_token = token.clone();
        self.config.youtube_subs_po_token = token;
        let _ = self.config.save();
    }

    pub fn set_youtube_extractor_args(&mut self, args: impl Into<String>) {
        let args = args.into();
        self.tool_paths.youtube_extractor_args = args.clone();
        self.config.youtube_extractor_args = args;
        let _ = self.config.save();
    }

    pub fn set_use_browser_cookies(&mut self, enabled: bool) {
        self.item_defaults.use_cookies = enabled;
        for item in &mut self.queue_items {
            item.selection.use_cookies = enabled;
        }
        self.config.use_browser_cookies = enabled;
        let _ = self.config.save();
    }

    pub fn set_use_aria2(&mut self, enabled: bool) {
        self.item_defaults.use_aria2 = enabled;
        for item in &mut self.queue_items {
            item.selection.use_aria2 = enabled;
        }
        self.config.use_aria2 = enabled;
        let _ = self.config.save();
    }

    pub fn set_write_thumbnail(&mut self, enabled: bool) {
        self.item_defaults.write_thumbnail = enabled;
        for item in &mut self.queue_items {
            item.selection.write_thumbnail = enabled;
        }
        self.config.write_thumbnail = enabled;
        let _ = self.config.save();
    }

    pub fn set_embed_thumbnail(&mut self, enabled: bool) {
        self.item_defaults.embed_thumbnail = enabled;
        for item in &mut self.queue_items {
            item.selection.embed_thumbnail = enabled;
        }
        self.config.embed_thumbnail = enabled;
        let _ = self.config.save();
    }

    pub fn set_write_subtitles(&mut self, enabled: bool) {
        self.item_defaults.write_subtitles = enabled;
        for item in &mut self.queue_items {
            item.selection.write_subtitles = enabled;
        }
        self.config.write_subtitles = enabled;
        let _ = self.config.save();
    }

    pub fn set_embed_subtitles(&mut self, enabled: bool) {
        self.item_defaults.embed_subtitles = enabled;
        for item in &mut self.queue_items {
            item.selection.embed_subtitles = enabled;
        }
        self.config.embed_subtitles = enabled;
        let _ = self.config.save();
    }

    pub fn set_write_chapters(&mut self, enabled: bool) {
        self.item_defaults.write_chapters = enabled;
        for item in &mut self.queue_items {
            item.selection.write_chapters = enabled;
        }
        self.config.write_chapters = enabled;
        let _ = self.config.save();
    }

    pub fn set_embed_chapters(&mut self, enabled: bool) {
        self.item_defaults.embed_chapters = enabled;
        for item in &mut self.queue_items {
            item.selection.embed_chapters = enabled;
        }
        self.config.embed_chapters = enabled;
        let _ = self.config.save();
    }


    pub fn push_runtime_log(&mut self, message: impl Into<String>) {
        let message = message.into();
        if message.trim().is_empty() {
            return;
        }
        self.runtime_log.push_front(message);
        while self.runtime_log.len() > 120 {
            self.runtime_log.pop_back();
        }
    }

    pub fn set_post_download_conversion_enabled(&mut self, enabled: bool) {
        self.config.post_download_conversion_enabled = enabled;
        let _ = self.config.save();
    }

    pub fn set_enable_builtin_transcode_after_download(&mut self, enabled: bool) {
        self.set_post_download_conversion_enabled(enabled);
    }

    pub fn set_chapter_compatibility_mode(&mut self, enabled: bool) {
        self.tool_paths.chapter_compatibility_mode = enabled;
        self.config.chapter_compatibility_mode = enabled;
        let _ = self.config.save();
    }

    pub fn set_quality_preset(&mut self, preset: QualityPreset) {
        self.item_defaults.quality = preset;
        for item in &mut self.queue_items {
            item.selection.quality = preset;
        }
    }

    pub fn cache_location_display(&self) -> String {
        match self.tool_paths.cache_mode {
            CacheLocationMode::YtDlpDefault => self.tr("state.cache_yt_dlp_default").to_owned(),
            CacheLocationMode::V2Cache => {
                crate::infrastructure::resolve_output_dir(&self.tool_paths.cache_dir)
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|_| self.tool_paths.cache_dir.clone())
            }
            CacheLocationMode::WindowsTemp => std::env::temp_dir().display().to_string(),
        }
    }

    fn begin_batch_add(&mut self, source: String) {
        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.youtube_playlist_prompt = None;
            self.last_action = error;
            return;
        }

        self.youtube_playlist_prompt = None;
        self.is_adding_batch = true;
        self.is_cancelling_batch_add = false;
        self.last_action = self.trf("state.adding_source", &[("{source}", source.as_str())]);

        let tool_paths = self.tool_paths.clone();
        let source_for_worker = source.clone();
        let (tx, rx) = mpsc::channel();
        self.batch_add_result_rx = Some(rx);
        let child_handle = Arc::new(Mutex::new(None));
        self.batch_add_child = Some(child_handle.clone());
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.batch_add_cancel_requested = Some(cancel_requested.clone());
        let limit = self
            .config
            .batch_limit_enabled
            .then_some(self.config.batch_limit_count.max(1));
        let untitled_task = self.tr("state.untitled_task").to_owned();
        let imported_template = self.tr("state.imported_source").to_owned();

        thread::spawn(move || {
            run_batch_add_worker(
                tool_paths,
                source_for_worker,
                limit,
                untitled_task,
                imported_template,
                tx,
                child_handle,
                cancel_requested,
            );
        });
    }

    pub fn latest_download_status(&self) -> Option<String> {
        let item = self.queue_items.last()?;
        Some(format!(
            "{}: {} | {}",
            self.tr(queue_item_status_key(item)),
            item.title,
            item.last_output_path
                .as_deref()
                .or(item.last_error.as_deref())
                .unwrap_or(item.source_url.as_str())
        ))
    }

    pub fn item_status_text(&self, item_index: usize) -> &'static str {
        let Some(item) = self.queue_items.get(item_index) else {
            return "";
        };
        self.tr(queue_item_status_key(item))
    }

    pub fn item_title_text(&self, item_index: usize) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        if !item.title.trim().is_empty() {
            item.title.clone()
        } else {
            item.source_url.clone()
        }
    }

    pub fn item_title_is_loading(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };
        matches!(item.metadata_state, MetadataState::Running)
            || item
                .workflows
                .iter()
                .any(|run| run.state == WorkflowState::Running)
    }

    pub fn url_input_locked(&self) -> bool {
        self.is_adding_batch || self.youtube_playlist_prompt.is_some()
    }

    fn add_single_url_to_batch(&mut self, source: String) {
        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        let item_id = self.ensure_queue_item_for_url(&source);
        self.url_input.clear();
        let fallback_title = infer_title(
            &source,
            self.tr("state.untitled_task"),
            self.tr("state.imported_source"),
        );
        self.last_action = self.trf(
            "state.added_to_list",
            &[("{title}", fallback_title.as_str())],
        );
        self.enqueue_item_analysis(item_id, source);
    }

    pub fn item_title_visual_state(&self, item_index: usize) -> ItemTitleVisualState {
        let Some(item) = self.queue_items.get(item_index) else {
            return ItemTitleVisualState::Default;
        };

        if item.workflows.iter().any(|run| {
            matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
                && matches!(
                    run.kind,
                    WorkflowKind::DownloadMedia
                        | WorkflowKind::ExportMedia
                        | WorkflowKind::PostProcess
                )
        })
        {
            return ItemTitleVisualState::Pending;
        }

        if item
            .completed_selection
            .as_ref()
            .is_some_and(|completed| selection_matches_completed(&item.selection, completed))
        {
            return ItemTitleVisualState::Completed;
        }

        if item.last_error.is_some() || matches!(item.metadata_state, MetadataState::Failed(_)) {
            return ItemTitleVisualState::Failed;
        }

        if matches!(
            item.metadata_state,
            MetadataState::Queued | MetadataState::Running
        ) {
            return ItemTitleVisualState::Pending;
        }

        if item.metadata_loaded() {
            return ItemTitleVisualState::Ready;
        }

        ItemTitleVisualState::Pending
    }

    pub fn item_error_text(&self, item_index: usize) -> Option<String> {
        let item = self.queue_items.get(item_index)?;
        if let Some(error) = item.last_error.as_ref() {
            return Some(error.clone());
        }

        if item_latest_download_state(item).is_some_and(|state| {
            matches!(
                state,
                WorkflowState::Queued | WorkflowState::Running | WorkflowState::Finished
            )
        }) {
            return None;
        }

        match &item.metadata_state {
            MetadataState::Failed(error) => Some(error.clone()),
            _ => None,
        }
    }

    pub fn item_output_file_path(&self, item_index: usize) -> Option<String> {
        let item = self.queue_items.get(item_index)?;
        if item.last_error.is_some() {
            return None;
        }
        if !item_latest_download_state(item).is_some_and(|state| state == WorkflowState::Finished) {
            return None;
        }
        item.last_output_path
            .clone()
            .filter(|path| !path.trim().is_empty())
    }

    pub fn item_progress(&self, item_index: usize, kind: FormatPickerKind) -> f32 {
        let Some(item) = self.queue_items.get(item_index) else {
            return 0.0;
        };
        if self.item_uses_muxed_video(item_index) {
            let shared = item.progress.video.max(item.progress.audio);
            return match kind {
                FormatPickerKind::Video | FormatPickerKind::Audio => shared,
                FormatPickerKind::Subtitle => item.progress.subtitle,
                FormatPickerKind::Section => 0.0,
            };
        }
        match kind {
            FormatPickerKind::Video => item.progress.video,
            FormatPickerKind::Audio => item.progress.audio,
            FormatPickerKind::Subtitle => item.progress.subtitle,
            FormatPickerKind::Section => 0.0,
        }
    }

    pub fn item_av_progress_visible(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        let has_active_download = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id && workflow.kind == WorkflowKind::DownloadMedia
        });
        if has_active_download {
            let video = item.progress.video;
            let audio = item.progress.audio;
            return (video > 0.0 || audio > 0.0) && !(video >= 100.0 && audio >= 100.0);
        }

        let has_active_export = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id && workflow.kind == WorkflowKind::ExportMedia
        });
        if has_active_export {
            let video = item.progress.video;
            let audio = item.progress.audio;
            let active_sides = [video, audio]
                .into_iter()
                .filter(|value| *value > 0.0)
                .collect::<Vec<_>>();
            if active_sides.is_empty() {
                return false;
            }
            return active_sides.iter().any(|value| *value < 100.0);
        }

        false
    }

    pub fn item_subtitle_progress_visible(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        let has_active_work = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id
                && matches!(
                    workflow.kind,
                    WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia
                )
        });
        if !has_active_work {
            return false;
        }

        item.progress.subtitle > 0.0 && item.progress.subtitle < 100.0
    }

    pub fn item_file_name_progress(&self, item_index: usize) -> f32 {
        self.queue_items
            .get(item_index)
            .map(|item| item.progress.post_process)
            .unwrap_or(0.0)
    }

    pub fn item_file_name_progress_visible(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        let has_active_post_process = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id && workflow.kind == WorkflowKind::PostProcess
        });
        has_active_post_process
            && item.progress.post_process > 0.0
            && item.progress.post_process < 100.0
    }

    fn resolve_download_format_selection(
        &self,
        video_selector: &str,
        audio_selector: &str,
        metadata: Option<&VideoMetadata>,
    ) -> (String, String) {
        if self.is_muxed_format(video_selector, metadata) {
            return (video_selector.to_owned(), video_selector.to_owned());
        }

        let resolved_audio = if audio_selector.trim().is_empty()
            || audio_selector == video_selector
            || self.is_muxed_format(audio_selector, metadata)
        {
            metadata
                .into_iter()
                .flat_map(|metadata| metadata.formats.iter())
                .find(|format| format.kind == MediaKind::Audio)
                .map(|format| format.id.clone())
                .unwrap_or_default()
        } else {
            audio_selector.to_owned()
        };

        if resolved_audio.is_empty() {
            (resolved_audio, video_selector.to_owned())
        } else {
            (
                resolved_audio.clone(),
                format!("{}+{}", video_selector.trim(), resolved_audio.trim()),
            )
        }
    }

    pub fn video_formats(&self) -> impl Iterator<Item = &FormatOption> {
        self.current_picker_metadata()
            .formats
            .iter()
            .filter(|format| matches!(format.kind, MediaKind::Video | MediaKind::Muxed))
    }

    pub fn audio_formats(&self) -> impl Iterator<Item = &FormatOption> {
        self.current_picker_metadata()
            .formats
            .iter()
            .filter(|format| format.kind == MediaKind::Audio)
    }

    pub fn subtitle_source_options(&self) -> Vec<SubtitleOption> {
        let mut items: Vec<SubtitleOption> = self
            .current_picker_metadata()
            .subtitle_tracks
            .iter()
            .cloned()
            .fold(Vec::new(), |mut acc, track| {
                if !acc
                    .iter()
                    .any(|item| item.source_key() == track.source_key())
                {
                    acc.push(track);
                }
                acc
            });

        items.sort_by(|left, right| {
            left.source
                .label()
                .cmp(right.source.label())
                .then_with(|| left.source_language_label.cmp(&right.source_language_label))
                .then_with(|| left.source_language_code.cmp(&right.source_language_code))
        });
        items
    }

    pub fn subtitle_translation_options(&self) -> Vec<SubtitleOption> {
        let source_key = self.current_subtitle_source_key();
        let mut items: Vec<SubtitleOption> = self
            .current_picker_metadata()
            .subtitle_tracks
            .iter()
            .filter(|track| track.source_key() == source_key)
            .cloned()
            .collect();

        items.sort_by(|left, right| {
            left.target_language_code
                .is_some()
                .cmp(&right.target_language_code.is_some())
                .then_with(|| left.target_label().cmp(&right.target_label()))
        });
        items
    }

    pub fn open_format_picker(&mut self, target_item_id: usize, kind: FormatPickerKind) {
        let selected_id = self
            .queue_items
            .get(target_item_id)
            .map(|item| match kind {
                FormatPickerKind::Video => item.selection.video_selector.as_str(),
                FormatPickerKind::Audio => item.selection.audio_selector.as_str(),
                FormatPickerKind::Subtitle => item.selection.subtitle_selector.as_str(),
                FormatPickerKind::Section => item.selection.download_sections.as_str(),
            })
            .unwrap_or_default()
            .to_owned();

        self.format_picker.open = true;
        self.format_picker.target_item_id = Some(target_item_id);
        self.format_picker.kind = Some(kind);
        self.format_picker.filter_text.clear();
        self.format_picker.filters.clear();

        if kind == FormatPickerKind::Section {
            let options = self.download_section_picker_options();
            self.format_picker.selected_row = options
                .iter()
                .position(|(value, _label)| value == &selected_id)
                .or(Some(0));
            return;
        }

        if kind == FormatPickerKind::Subtitle {
            let subtitle_source = self
                .queue_items
                .get(target_item_id)
                .map(|item| item.selection.subtitle_source)
                .unwrap_or(SubtitleSource::None);
            self.format_picker.subtitle_tab = match subtitle_source {
                SubtitleSource::None => SubtitlePickerTab::None,
                SubtitleSource::Original => SubtitlePickerTab::Original,
                SubtitleSource::Automatic => SubtitlePickerTab::Automatic,
            };
            self.format_picker.subtitle_source_key = match subtitle_source {
                SubtitleSource::None => SubtitleSource::None.key().to_owned(),
                _ => self
                    .queue_items
                    .get(target_item_id)
                    .and_then(|item| {
                        self.subtitle_track_by_id(
                            &item.selection.subtitle_selector,
                            self.item_metadata(target_item_id),
                        )
                    })
                    .map(|track| track.source_key())
                    .unwrap_or_default(),
            };
            self.format_picker.selected_row = if subtitle_source == SubtitleSource::None {
                Some(0)
            } else {
                let options = if subtitle_source == SubtitleSource::Original {
                    self.subtitle_source_options()
                        .into_iter()
                        .filter(|track| track.source == SubtitleSource::Original)
                        .collect::<Vec<_>>()
                } else {
                    self.subtitle_translation_options()
                };
                options.iter().position(|option| option.id == selected_id)
            };
            return;
        }

        let options = self.format_picker_options(kind);
        let selected_row = options.iter().position(|option| option.id == selected_id);
        let selected_option = selected_row.and_then(|index| options.get(index));
        self.format_picker.selected_row = selected_row;

        if let Some(option) = selected_option {
            match kind {
                FormatPickerKind::Video => {
                    if !option.resolution.is_empty() {
                        self.format_picker.filters.resolution = Some(option.resolution.clone());
                    }
                    if !option.dynamic_range.is_empty() {
                        self.format_picker.filters.dynamic_range =
                            Some(option.dynamic_range.clone());
                    }
                    if !option.fps.is_empty() {
                        self.format_picker.filters.fps = Some(option.fps.clone());
                    }
                    if !option.codec.is_empty() {
                        self.format_picker.filters.codec = Some(option.codec.clone());
                    }
                }
                FormatPickerKind::Audio => {
                    if !option.sample_rate.is_empty() {
                        self.format_picker.filters.sample_rate = Some(option.sample_rate.clone());
                    }
                    if !option.codec.is_empty() {
                        self.format_picker.filters.codec = Some(option.codec.clone());
                    }
                }
                FormatPickerKind::Subtitle | FormatPickerKind::Section => {}
            }
        }
    }

    pub fn cancel_format_picker(&mut self) {
        self.format_picker.open = false;
        self.format_picker.target_item_id = None;
        self.format_picker.kind = None;
        self.format_picker.selected_row = None;
        self.format_picker.filter_text.clear();
        self.format_picker.filters.clear();
        self.format_picker.subtitle_source_key.clear();
        self.format_picker.subtitle_tab = SubtitlePickerTab::None;
    }

    pub fn confirm_format_picker_selection(&mut self, selected_format_id: &str) {
        let Some(target_item_id) = self.format_picker.target_item_id else {
            return;
        };
        let Some(kind) = self.format_picker.kind else {
            return;
        };
        let item_metadata = self.item_metadata(target_item_id);
        let is_muxed_selection = self.is_muxed_format(selected_format_id, item_metadata);
        let item_uses_muxed_video = self.item_uses_muxed_video(target_item_id);
        let replacement_audio_selector = if kind == FormatPickerKind::Video && !is_muxed_selection {
            self.replacement_audio_selector_for_video_change(
                target_item_id,
                selected_format_id,
                item_metadata,
            )
        } else {
            None
        };
        let selected_subtitle_source = self
            .subtitle_track_by_id(selected_format_id, item_metadata)
            .map(|track| track.source);
        let Some(item) = self.queue_items.get_mut(target_item_id) else {
            self.cancel_format_picker();
            return;
        };

        match kind {
            FormatPickerKind::Video => {
                item.selection.video_selector = selected_format_id.to_owned();
                if is_muxed_selection {
                    item.selection.audio_selector = selected_format_id.to_owned();
                } else if let Some(audio_selector) = replacement_audio_selector {
                    item.selection.audio_selector = audio_selector;
                }
            }
            FormatPickerKind::Audio => {
                if item_uses_muxed_video {
                    self.cancel_format_picker();
                    return;
                }
                item.selection.audio_selector = selected_format_id.to_owned();
            }
            FormatPickerKind::Subtitle => {
                if selected_format_id.is_empty() {
                    item.selection.subtitle_selector.clear();
                    item.selection.subtitle_source = SubtitleSource::None;
                } else {
                    item.selection.subtitle_selector = selected_format_id.to_owned();
                    item.selection.subtitle_source =
                        selected_subtitle_source.unwrap_or(item.selection.subtitle_source);
                }
            }
            FormatPickerKind::Section => {
                item.selection.download_sections = selected_format_id.trim().to_owned();
                item.completed_selection = None;
            }
        }

        self.last_action = match kind {
            FormatPickerKind::Section if selected_format_id.trim().is_empty() => self.trf(
                "state.range_set_item_full",
                &[("{index}", &(target_item_id + 1).to_string())],
            ),
            FormatPickerKind::Section => self.trf(
                "state.range_set_item_value",
                &[
                    ("{index}", &(target_item_id + 1).to_string()),
                    ("{value}", selected_format_id),
                ],
            ),
            _ => self.trf(
                "state.format_selection_updated",
                &[
                    ("{index}", &(target_item_id + 1).to_string()),
                    ("{kind}", kind.label()),
                    ("{value}", selected_format_id),
                ],
            ),
        };
        self.cancel_format_picker();
    }

    pub fn format_picker_options(&self, kind: FormatPickerKind) -> Vec<FormatOption> {
        let mut options: Vec<FormatOption> = match kind {
            FormatPickerKind::Video => self.video_formats().cloned().collect(),
            FormatPickerKind::Audio => self.audio_formats().cloned().collect(),
            FormatPickerKind::Subtitle | FormatPickerKind::Section => Vec::new(),
        };

        match kind {
            FormatPickerKind::Video => {
                options.sort_by(|left, right| {
                    video_resolution_area(right)
                        .cmp(&video_resolution_area(left))
                        .then_with(|| {
                            human_size_bytes(&right.filesize).cmp(&human_size_bytes(&left.filesize))
                        })
                        .then_with(|| left.id.cmp(&right.id))
                });
            }
            FormatPickerKind::Audio | FormatPickerKind::Subtitle | FormatPickerKind::Section => {}
        }

        options
    }

    pub fn selected_format_summary(&self, item_index: usize, kind: FormatPickerKind) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        if !item.metadata_loaded() {
            return self.tr("picker.waiting_analysis").to_owned();
        }

        if kind == FormatPickerKind::Audio && self.item_uses_muxed_video(item_index) {
            return self.tr("picker.audio_from_video").to_owned();
        }

        if kind == FormatPickerKind::Subtitle {
            return self.selected_subtitle_summary(item_index);
        }
        if kind == FormatPickerKind::Section {
            return self.selected_download_section_summary(item_index);
        }

        let selected_id = match kind {
            FormatPickerKind::Video => &item.selection.video_selector,
            FormatPickerKind::Audio => &item.selection.audio_selector,
            FormatPickerKind::Subtitle => &item.selection.subtitle_selector,
            FormatPickerKind::Section => &item.selection.download_sections,
        };

        self.format_label_by_id(selected_id, item.metadata())
            .unwrap_or_default()
            .to_owned()
    }

    pub fn format_picker_target_title(&self) -> Option<&str> {
        self.format_picker
            .target_item_id
            .and_then(|index| self.queue_items.get(index))
            .map(|item| item.title.as_str())
    }

    pub fn selected_subtitle_summary(&self, item_index: usize) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        if item.selection.subtitle_source == SubtitleSource::None
            || item.selection.subtitle_selector.is_empty()
        {
            return self.tr("tools.subtitle_source.none").to_owned();
        }

        self.subtitle_track_by_id(&item.selection.subtitle_selector, item.metadata())
            .map(|track| {
                format!(
                    "{} / {}",
                    self.subtitle_source_label(track.source),
                    self.localized_subtitle_target_label(track)
                )
            })
            .unwrap_or_else(|| self.tr("picker.not_selected").to_owned())
    }

    pub fn subtitle_source_label(&self, source: SubtitleSource) -> &'static str {
        match source {
            SubtitleSource::None => self.tr("tools.subtitle_source.none"),
            SubtitleSource::Original => self.tr("tools.subtitle_source.original"),
            SubtitleSource::Automatic => self.tr("tools.subtitle_source.automatic"),
        }
    }

    pub fn localized_subtitle_target_label(&self, option: &SubtitleOption) -> String {
        match (&option.target_language_label, &option.target_language_code) {
            (Some(label), Some(code)) => format!("{label} ({code})"),
            (Some(label), None) => label.clone(),
            (None, Some(code)) => code.clone(),
            (None, None) => self.tr("picker.no_translation").to_owned(),
        }
    }

    pub fn item_shows_subtitle_row(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        item.metadata()
            .is_some_and(|metadata| !metadata.subtitle_tracks.is_empty())
    }

    pub fn item_shows_download_section_row(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        item.metadata()
            .is_some_and(|metadata| !metadata.chapters.is_empty())
            || !item.selection.download_sections.trim().is_empty()
    }

    pub fn item_download_section_options(&self, item_index: usize) -> Vec<(String, String)> {
        self.queue_items
            .get(item_index)
            .and_then(QueueItem::metadata)
            .map(|metadata| {
                metadata
                    .chapters
                    .iter()
                    .map(|chapter| {
                        (
                            chapter.download_sections.clone(),
                            self.localized_chapter_label(chapter),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn download_section_picker_options(&self) -> Vec<(String, String)> {
        let Some(item_index) = self.format_picker.target_item_id else {
            return vec![(String::new(), self.tr("picker.full_video").to_owned())];
        };

        let mut options = Vec::with_capacity(1);
        options.push((String::new(), self.tr("picker.full_video").to_owned()));
        options.extend(self.item_download_section_options(item_index));
        options
    }

    pub fn selected_download_section_summary(&self, item_index: usize) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        let selected = item.selection.download_sections.trim();
        if selected.is_empty() {
            return self.tr("picker.full_video").to_owned();
        }

        item.metadata()
            .into_iter()
            .flat_map(|metadata| metadata.chapters.iter())
            .find(|chapter| chapter.download_sections.as_str() == selected)
            .map(|chapter| self.localized_chapter_label(chapter))
            .unwrap_or_else(|| selected.to_owned())
    }

    fn localized_chapter_label(&self, chapter: &crate::domain::ChapterOption) -> String {
        let range = match chapter.end_text.as_deref() {
            Some(end) if !end.is_empty() => format!("{}–{}", chapter.start_text, end),
            _ => format!("{}–{}", chapter.start_text, self.tr("picker.until_end")),
        };

        if chapter.title.trim().is_empty() {
            range
        } else {
            format!("{}  {}", range, chapter.title)
        }
    }

    pub fn set_item_download_sections(&mut self, item_index: usize, value: impl Into<String>) {
        let Some(item) = self.queue_items.get_mut(item_index) else {
            return;
        };
        let title = item.title.clone();
        let value = value.into().trim().to_owned();
        item.selection.download_sections = value.clone();
        item.completed_selection = None;
        self.last_action = if value.is_empty() {
            self.trf("state.range_set_title_full", &[("{title}", title.as_str())])
        } else {
            self.trf(
                "state.range_set_title_value",
                &[("{title}", title.as_str()), ("{value}", value.as_str())],
            )
        };
    }

    pub fn item_uses_seed_compact_layout(&self, item_index: usize) -> bool {
        self.queue_items
            .get(item_index)
            .is_some_and(|item| matches!(item.metadata_state, MetadataState::Idle))
    }

    pub fn subtitle_track_by_id<'a>(
        &'a self,
        id: &str,
        metadata: Option<&'a VideoMetadata>,
    ) -> Option<&'a SubtitleOption> {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.subtitle_tracks.iter())
            .find(|track| track.id == id)
    }

    pub fn current_subtitle_source_key(&self) -> String {
        if !self.format_picker.subtitle_source_key.is_empty() {
            return self.format_picker.subtitle_source_key.clone();
        }

        self.subtitle_source_options()
            .first()
            .map(|track| track.source_key())
            .unwrap_or_else(|| SubtitleSource::None.key().to_owned())
    }

    pub fn item_uses_muxed_video(&self, item_index: usize) -> bool {
        self.queue_items
            .get(item_index)
            .map(|item| self.is_muxed_format(&item.selection.video_selector, item.metadata()))
            .unwrap_or(false)
    }

    fn replacement_audio_selector_for_video_change(
        &self,
        item_index: usize,
        video_selector: &str,
        metadata: Option<&VideoMetadata>,
    ) -> Option<String> {
        let current_audio = self
            .queue_items
            .get(item_index)?
            .selection
            .audio_selector
            .trim();
        if !current_audio.is_empty()
            && current_audio != video_selector
            && !self.is_muxed_format(current_audio, metadata)
        {
            return None;
        }

        first_audio_format_id(metadata)
    }

    pub fn is_muxed_format(&self, format_id: &str, metadata: Option<&VideoMetadata>) -> bool {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == format_id)
            .map(|format| format.kind == MediaKind::Muxed)
            .unwrap_or(false)
    }

    pub fn format_label_by_id<'a>(
        &'a self,
        id: &str,
        metadata: Option<&'a VideoMetadata>,
    ) -> Option<&'a str> {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == id)
            .map(|format| format.label.as_str())
    }

    pub fn format_extension_by_id(&self, id: &str, metadata: Option<&VideoMetadata>) -> String {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == id)
            .map(|format| format.ext.clone())
            .unwrap_or_default()
    }

    pub fn format_codec_by_id(&self, id: &str, metadata: Option<&VideoMetadata>) -> String {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == id)
            .map(|format| format.codec.clone())
            .unwrap_or_default()
    }

    fn apply_analysis_json(
        &mut self,
        json: Value,
        analyzed_source: Option<String>,
        target_item_id: Option<QueueItemId>,
        workflow_id: Option<WorkflowRunId>,
    ) {
        if json.get("entries").and_then(Value::as_array).is_some() {
            let target = analyzed_source.unwrap_or_else(|| "playlist".to_owned());
            self.last_action = self.trf(
                "state.playlist_ignored_for_now",
                &[("{target}", target.as_str())],
            );
            return;
        }

        let title = json
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(self.tr("state.untitled_video"))
            .to_owned();
        let webpage_url = json
            .get("webpage_url")
            .or_else(|| json.get("original_url"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let uploader = json
            .get("uploader")
            .or_else(|| json.get("channel"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let duration_text = json
            .get("duration_string")
            .and_then(Value::as_str)
            .map(normalize_duration_badge_text)
            .unwrap_or_default();
        let description = json
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let upload_date_text = json
            .get("upload_date")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let thumbnail_hint = json
            .get("thumbnail")
            .and_then(Value::as_str)
            .map(|_| "item.thumbnail_preview".to_owned())
            .unwrap_or_else(|| "item.thumbnail".to_owned());
        let thumbnail_url = select_best_thumbnail_url(&json).unwrap_or_default();

        let formats = extract_formats(&json);
        let requested_ids = extract_requested_ids(&json);
        let subtitle_tracks = extract_subtitle_tracks(&json);
        let chapters = extract_chapters(&json, |index| {
            let number = (index + 1).to_string();
            self.trf("state.chapter_fallback", &[("{index}", number.as_str())])
        });

        let metadata = VideoMetadata {
            title: title.clone(),
            uploader,
            duration_text,
            webpage_url,
            description,
            view_count_text: json
                .get("view_count")
                .and_then(Value::as_i64)
                .map(|value| value.to_string())
                .unwrap_or_default(),
            upload_date_text,
            thumbnail_hint,
            thumbnail_url,
            formats: formats.clone(),
            subtitle_tracks: subtitle_tracks.clone(),
            chapters: chapters.clone(),
        };
        let default_video = requested_or_default_format_id(
            &formats,
            &requested_ids,
            &[MediaKind::Video, MediaKind::Muxed],
        );
        let default_audio = if formats
            .iter()
            .find(|format| format.id == default_video)
            .is_some_and(|format| format.kind == MediaKind::Muxed)
        {
            default_video.clone()
        } else {
            requested_or_default_format_id(&formats, &requested_ids, &[MediaKind::Audio])
        };

        let default_subtitle_source = SubtitleSource::None;
        let default_subtitle = String::new();
        let default_file_name = extract_requested_filename(&json)
            .or_else(|| {
                json.get("_filename")
                    .or_else(|| json.get("filename"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .map(|filename| sanitize_file_name_for_windows(&display_file_stem(&filename)))
            .unwrap_or_default();

        if let Some(item_id) = target_item_id {
            if let Some(item) = self.queue_item_mut_by_id(item_id) {
                item.title = title.clone();
                item.thumbnail_hint = "item.thumbnail".to_owned();
                item.thumbnail_url = metadata.thumbnail_url.clone();
                item.duration_text = metadata.duration_text.clone();
                item.metadata_state = MetadataState::Ready(metadata.clone());
                item.selection.video_selector = default_video.clone();
                item.selection.audio_selector = default_audio.clone();
                item.selection.subtitle_selector = default_subtitle.clone();
                item.selection.subtitle_source = default_subtitle_source;
                if item.selection.file_name.trim().is_empty() {
                    item.selection.file_name = default_file_name.clone();
                }
                let run_index = workflow_id
                    .and_then(|workflow_id| {
                        item.workflows.iter().position(|run| run.id == workflow_id)
                    })
                    .or_else(|| {
                        item.workflows
                            .iter()
                            .rposition(|run| run.kind == WorkflowKind::AnalyzeMetadata)
                    });
                if let Some(run) = run_index.and_then(|index| item.workflows.get_mut(index)) {
                    run.state = WorkflowState::Finished;
                    run.detail = analyzed_source
                        .clone()
                        .unwrap_or_else(|| metadata.webpage_url.clone());
                }
            }
            if let Some(workflow_id) = workflow_id {
                self.unregister_active_workflow(workflow_id);
            }
        } else {
            let item_source_url = analyzed_source
                .clone()
                .filter(|source| !source.trim().is_empty())
                .or_else(|| {
                    (!metadata.webpage_url.trim().is_empty()).then(|| metadata.webpage_url.clone())
                })
                .unwrap_or_else(|| self.url_input.trim().to_owned());
            let mut item =
                QueueItem::new(self.alloc_queue_item_id(), item_source_url, title.clone());
            item.selection.quality = self.item_defaults.quality;
            item.selection.write_thumbnail = self.item_defaults.write_thumbnail;
            item.selection.embed_thumbnail = self.item_defaults.embed_thumbnail;
            item.selection.write_subtitles = self.item_defaults.write_subtitles;
            item.selection.embed_subtitles = self.item_defaults.embed_subtitles;
            item.selection.write_chapters = self.item_defaults.write_chapters;
            item.selection.embed_chapters = self.item_defaults.embed_chapters;
            item.selection.use_cookies = self.item_defaults.use_cookies;
            item.selection.use_aria2 = self.item_defaults.use_aria2;
            item.selection.output_dir = self.item_defaults.output_dir.clone();
            item.metadata_state = MetadataState::Ready(metadata.clone());
            item.thumbnail_url = metadata.thumbnail_url.clone();
            item.duration_text = metadata.duration_text.clone();
            item.selection.video_selector = default_video.clone();
            item.selection.audio_selector = default_audio.clone();
            item.selection.subtitle_selector = default_subtitle.clone();
            item.selection.subtitle_source = default_subtitle_source;
            item.selection.file_name = default_file_name.clone();
            self.queue_items = vec![item];
        }

        let analyzed_target = analyzed_source
            .or_else(|| (!metadata.webpage_url.is_empty()).then(|| metadata.webpage_url.clone()))
            .unwrap_or_else(|| title.clone());
        self.last_action = self.trf(
            "state.analysis_complete",
            &[("{title}", analyzed_target.as_str())],
        );
    }
}

fn read_clipboard_text() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    clipboard.get_text().ok()
}

fn extract_monitored_youtube_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|candidate| {
            candidate
                .trim_matches(|ch: char| {
                    matches!(
                        ch,
                        '"' | '\''
                            | '`'
                            | '<'
                            | '>'
                            | '('
                            | ')'
                            | '['
                            | ']'
                            | '{'
                            | '}'
                            | '\u{ff0c}'
                            | '\u{3002}'
                            | '\u{3001}'
                            | '\u{ff1b}'
                            | ';'
                            | '\u{ff1a}'
                            | ':'
                            | ','
                    )
                })
                .to_owned()
        })
        .filter(|candidate| !candidate.is_empty())
        .find_map(|candidate| normalize_monitored_youtube_url(&candidate))
}

fn normalize_monitored_youtube_url(candidate: &str) -> Option<String> {
    let lowered = candidate.to_ascii_lowercase();
    if !(lowered.contains("youtube.com") || lowered.contains("youtu.be")) {
        return None;
    }

    if lowered.starts_with("http://") || lowered.starts_with("https://") {
        Some(candidate.to_owned())
    } else if lowered.starts_with("www.youtube.com")
        || lowered.starts_with("m.youtube.com")
        || lowered.starts_with("youtube.com")
        || lowered.starts_with("youtu.be")
    {
        Some(format!("https://{candidate}"))
    } else {
        None
    }
}

fn canonical_queue_source_key(url: &str) -> String {
    let trimmed = url.trim();
    if let Some(video_id) = youtube_video_id(trimmed) {
        return format!("youtube:video:{video_id}");
    }
    trimmed.to_ascii_lowercase()
}

fn youtube_video_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lowered = trimmed.to_ascii_lowercase();

    if lowered.contains("youtu.be/") {
        let (_, tail) = trimmed.split_once("youtu.be/")?;
        let id = tail
            .split(['?', '&', '#', '/'])
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        return Some(id.to_owned());
    }

    if lowered.contains("youtube.com/watch") || lowered.contains("m.youtube.com/watch") {
        let (_, tail) = trimmed.split_once("v=")?;
        let id = tail
            .split(['&', '#', '?', '/'])
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        return Some(id.to_owned());
    }

    None
}

fn should_retry_analyze_with_cookies(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("sign in to confirm you're not a bot")
        || normalized.contains("sign in to confirm you")
        || normalized.contains("use --cookies-from-browser")
        || normalized.contains("use --cookies for the authentication")
}

fn normalize_export_target_path(path: &str, default_extension: Option<&str>) -> String {
    let trimmed = path.trim();
    let mut target = PathBuf::from(trimmed);
    let has_extension = target
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| !value.trim().is_empty());
    if !has_extension {
        if let Some(extension) = default_extension.filter(|value| !value.trim().is_empty()) {
            target.set_extension(extension);
        }
    }
    target.display().to_string()
}

fn normalized_export_extension(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_start_matches('.').to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn validate_export_extension(
    language: Language,
    kind: DownloadTargetKind,
    extension: &str,
) -> Result<(), String> {
    let valid = match kind {
        DownloadTargetKind::Video => matches!(extension, "mkv" | "mp4" | "webm" | "mov" | "flv"),
        DownloadTargetKind::Audio => {
            matches!(
                extension,
                "opus" | "aac" | "m4a" | "mp3" | "vorbis" | "alac" | "flac" | "wav"
            )
        }
        DownloadTargetKind::Subtitle => {
            matches!(
                extension,
                "srt"
                    | "vtt"
                    | "ass"
                    | "ssa"
                    | "lrc"
                    | "ttml"
                    | "dfxp"
                    | "srv1"
                    | "srv2"
                    | "srv3"
                    | "json3"
            )
        }
        DownloadTargetKind::Normal => true,
    };
    if valid {
        Ok(())
    } else {
        Err(match kind {
            DownloadTargetKind::Video => {
                i18n::text(language, "state.video_extension_error").to_owned()
            }
            DownloadTargetKind::Audio => {
                i18n::text(language, "state.audio_extension_error").to_owned()
            }
            DownloadTargetKind::Subtitle => {
                i18n::text(language, "state.subtitle_extension_error").to_owned()
            }
            DownloadTargetKind::Normal => String::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::metadata::default_format_id;

    #[test]
    fn default_video_format_prefers_highest_resolution() {
        let formats = vec![
            FormatOption::video(
                "low",
                "low",
                MediaKind::Video,
                "640x360",
                "",
                "",
                "mp4",
                "h264",
                "10.00 MB",
            ),
            FormatOption::video(
                "high",
                "high",
                MediaKind::Video,
                "1920x1080",
                "",
                "",
                "mp4",
                "h264",
                "20.00 MB",
            ),
            FormatOption::video(
                "mid",
                "mid",
                MediaKind::Video,
                "1280x720",
                "",
                "",
                "mp4",
                "h264",
                "30.00 MB",
            ),
        ];

        assert_eq!(default_format_id(&formats, &[MediaKind::Video]), "high");
    }

    #[test]
    fn requested_format_still_wins_over_quality_guess() {
        let formats = vec![
            FormatOption::video(
                "requested",
                "requested",
                MediaKind::Video,
                "640x360",
                "",
                "",
                "mp4",
                "h264",
                "10.00 MB",
            ),
            FormatOption::video(
                "high",
                "high",
                MediaKind::Video,
                "1920x1080",
                "",
                "",
                "mp4",
                "h264",
                "20.00 MB",
            ),
        ];

        assert_eq!(
            requested_or_default_format_id(
                &formats,
                &[String::from("requested")],
                &[MediaKind::Video],
            ),
            "requested"
        );
    }

    #[test]
    fn display_file_stem_drops_extension_from_auto_name() {
        assert_eq!(
            display_file_stem(r"download\sample title [abc123].webm"),
            "sample title [abc123]"
        );
    }

    #[test]
    fn first_audio_format_ignores_muxed_formats() {
        let metadata = VideoMetadata {
            formats: vec![
                FormatOption::video(
                    "muxed",
                    "muxed",
                    MediaKind::Muxed,
                    "1280x720",
                    "",
                    "30",
                    "mp4",
                    "h264",
                    "10.00 MB",
                ),
                FormatOption::audio(
                    "audio",
                    "audio",
                    MediaKind::Audio,
                    "48000",
                    "m4a",
                    "aac",
                    "3.00 MB",
                ),
            ],
            ..VideoMetadata::empty_preview()
        };

        assert_eq!(
            first_audio_format_id(Some(&metadata)).as_deref(),
            Some("audio")
        );
    }
}
