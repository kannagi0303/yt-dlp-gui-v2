use super::*;

pub struct AppState {
    pub active_tab: AppTab,
    pub url_input: String,
    pub batch_input: String,
    pub batch_enabled: bool,
    pub monitor_clipboard: bool,
    pub(super) last_clipboard_text: String,
    pub(super) last_clipboard_check: Option<Instant>,
    pub(super) clipboard_monitor_baseline_ready: bool,
    pub empty_item_preview: VideoMetadata,
    pub queue_items: Vec<QueueItem>,
    pub(super) queue_display_mode: QueueDisplayMode,
    pub(super) app_mode: AppMode,
    pub item_defaults: DownloadOptions,
    pub config: AppConfig,
    pub(super) pending_ui_scale_percent: u16,
    pub options_detail_page: Option<OptionsDetailPage>,
    pub prepare_detail_page: Option<PrepareDetailPage>,
    pub advance_detail_page: Option<AdvanceDetailPage>,
    pub about_detail_target: AboutDetailTarget,
    pub tool_paths: ToolPaths,
    pub prepare_report: PrepareReport,
    pub(super) prepare_tab_snoozed: bool,
    pub last_action: String,
    pub(super) logs: LogState,
    pub format_picker: FormatPickerState,
    pub is_adding_batch: bool,
    pub is_cancelling_batch_add: bool,
    pub youtube_playlist_prompt: Option<YoutubePlaylistPrompt>,
    pub(super) cookie_rescue: CookieRescueState,
    pub(super) analyze_result_rx: Receiver<AnalyzeResult>,
    pub(super) analyze_result_tx: Sender<AnalyzeResult>,
    pub(super) batch_add_result_rx: Option<Receiver<BatchAddEvent>>,
    pub(super) batch_add_child: Option<Arc<Mutex<Option<Child>>>>,
    pub(super) batch_add_cancel_requested: Option<Arc<AtomicBool>>,
    pub(super) batch_add_music_compact: bool,
    pub(super) download_result_rx: Receiver<DownloadEvent>,
    pub(super) download_result_tx: Sender<DownloadEvent>,
    pub(super) post_process_result_rx: Receiver<PostProcessEvent>,
    pub(super) post_process_result_tx: Sender<PostProcessEvent>,
    pub(super) component_update_result_rx: Receiver<ComponentUpdateEvent>,
    pub(super) component_update_result_tx: Sender<ComponentUpdateEvent>,
    pub(super) thumbnail_result_rx: Receiver<ThumbnailFetchEvent>,
    pub(super) thumbnail_result_tx: Sender<ThumbnailFetchEvent>,
    pub(super) music_stream_result_rx: Receiver<MusicStreamResolveEvent>,
    pub(super) music_stream_result_tx: Sender<MusicStreamResolveEvent>,
    pub(super) music_playback_event_rx: Receiver<MusicPlaybackEvent>,
    pub(super) music_playback_event_tx: Sender<MusicPlaybackEvent>,
    pub(super) music_download_event_rx: Receiver<MusicDownloadEvent>,
    pub(super) music_download_event_tx: Sender<MusicDownloadEvent>,
    pub(super) music: MusicState,
    pub(super) thumbnail_cache: HashMap<String, ThumbnailCacheEntry>,
    pub component_update_snapshot: ComponentUpdateSnapshot,
    pub about_markdown_cache: CommonMarkCache,
    pub(super) app_instance_guard: Option<AppInstanceGuard>,
    pub(super) font_content_revision: u64,
    pub(super) active_workflows: HashMap<WorkflowRunId, ActiveWorkflow>,
    pub(super) next_queue_item_id: QueueItemId,
    pub(super) next_workflow_run_id: WorkflowRunId,
}

impl AppState {
    pub fn new() -> Self {
        let (config, tool_paths) = AppConfig::load_runtime();
        Self::from_runtime(config, tool_paths)
    }

    pub fn from_runtime(mut config: AppConfig, tool_paths: ToolPaths) -> Self {
        let (analyze_result_tx, analyze_result_rx) = mpsc::channel();
        let (download_result_tx, download_result_rx) = mpsc::channel();
        let (post_process_result_tx, post_process_result_rx) = mpsc::channel();
        let (component_update_result_tx, component_update_result_rx) = mpsc::channel();
        let (thumbnail_result_tx, thumbnail_result_rx) = mpsc::channel();
        let (music_stream_result_tx, music_stream_result_rx) = mpsc::channel();
        let (music_playback_event_tx, music_playback_event_rx) = mpsc::channel();
        let (music_download_event_tx, music_download_event_rx) = mpsc::channel();
        let pending_ui_scale_percent = config.ui_scale_percent;
        let music_volume = config.music_volume.clamp(0.0, 1.0);
        let music_playback_mode = MusicPlaybackMode::from_config_value(&config.music_playback_mode);
        let app_mode = AppMode::from_config_value(&config.app_mode);
        let queue_display_mode = QueueDisplayMode::from_app_mode(app_mode);
        config.app_mode = app_mode.config_value().to_owned();
        config.queue_display_mode = queue_display_mode.config_value().to_owned();
        let mut state = Self {
            active_tab: AppTab::Main,
            url_input: String::new(),
            batch_input: String::new(),
            batch_enabled: true,
            monitor_clipboard: config.auto_paste_clipboard,
            last_clipboard_text: String::new(),
            last_clipboard_check: config.auto_paste_clipboard.then(Instant::now),
            clipboard_monitor_baseline_ready: false,
            empty_item_preview: VideoMetadata::empty_preview(),
            queue_items: Vec::new(),
            queue_display_mode,
            app_mode,
            item_defaults: {
                let mut defaults = DownloadOptions::default();
                defaults.output_dir = config.download_dir.clone();
                defaults.use_cookies = config.use_browser_cookies;
                defaults.use_aria2 = config.use_aria2;
                defaults.write_thumbnail = config.thumbnail_mode.writes();
                defaults.embed_thumbnail = config.thumbnail_mode.embeds();
                defaults.write_subtitles = config.subtitle_mode.writes();
                defaults.embed_subtitles = config.subtitle_mode.embeds();
                defaults.write_chapters = config.chapter_mode.writes();
                defaults.embed_chapters = config.chapter_mode.embeds();
                defaults
            },
            config,
            pending_ui_scale_percent,
            options_detail_page: None,
            prepare_detail_page: None,
            advance_detail_page: None,
            about_detail_target: AboutDetailTarget::App,
            tool_paths,
            prepare_report: PrepareReport::default(),
            prepare_tab_snoozed: false,
            last_action: String::new(),
            logs: LogState::default(),
            format_picker: FormatPickerState::default(),
            is_adding_batch: false,
            is_cancelling_batch_add: false,
            youtube_playlist_prompt: None,
            cookie_rescue: CookieRescueState::default(),
            analyze_result_rx,
            analyze_result_tx,
            batch_add_result_rx: None,
            batch_add_child: None,
            batch_add_cancel_requested: None,
            batch_add_music_compact: false,
            download_result_rx,
            download_result_tx,
            post_process_result_rx,
            post_process_result_tx,
            component_update_result_rx,
            component_update_result_tx,
            thumbnail_result_rx,
            thumbnail_result_tx,
            music_stream_result_rx,
            music_stream_result_tx,
            music_playback_event_rx,
            music_playback_event_tx,
            music_download_event_rx,
            music_download_event_tx,
            music: MusicState::new(music_volume, music_playback_mode),
            thumbnail_cache: HashMap::new(),
            component_update_snapshot: component_update_startup_snapshot(),
            about_markdown_cache: CommonMarkCache::default(),
            app_instance_guard: register_app_instance(),
            font_content_revision: 1,
            active_workflows: HashMap::new(),
            next_queue_item_id: 1,
            next_workflow_run_id: 1,
        };

        cleanup_applied_update();
        state.restore_saved_audio_playlist();
        state.prepare_report = collect_dependency_presence_report(&state.tool_paths);
        state.sanitize_startup_prepare_component_update_snapshot();
        schedule_startup_transient_cleanup();
        if state.should_show_prepare_tab() {
            state.active_tab = AppTab::Prepare;
        }
        state
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        self.force_stop_owned_external_tools();
    }
}

impl AppState {
    fn force_stop_owned_external_tools(&mut self) {
        if let Some(cancel_requested) = self.batch_add_cancel_requested.take() {
            cancel_requested.store(true, Ordering::Relaxed);
        }
        if let Some(child_handle) = self.batch_add_child.take() {
            Self::terminate_child_handle(&child_handle);
        }

        for workflow in self.active_workflows.values_mut() {
            if let Some(cancel_requested) = workflow.cancel_requested.as_ref() {
                cancel_requested.store(true, Ordering::Relaxed);
            }
            if let Some(child_handle) = workflow.download_child.as_ref() {
                Self::terminate_child_handle(child_handle);
            }
        }
        self.active_workflows.clear();

        crate::infrastructure::force_cleanup_tracked_processes();
    }

    fn terminate_child_handle(child_handle: &Arc<Mutex<Option<Child>>>) {
        let Ok(mut guard) = child_handle.lock() else {
            return;
        };
        let Some(mut child) = guard.take() else {
            return;
        };
        terminate_child_process(&mut child);
        let _ = child.wait();
    }
}
