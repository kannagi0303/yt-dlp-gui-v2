use super::*;

impl AppState {
    pub(super) fn parsed_batch_urls(&self) -> Vec<String> {
        self.batch_input
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    pub(super) fn append_batch_seed(&mut self, _source: &str, seed: PlaylistEntrySeed) {
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
        self.mark_font_content_changed();

        if !self
            .batch_input
            .lines()
            .map(str::trim)
            .any(|line| canonical_queue_source_key(line) == source_key)
        {
            self.batch_input_push_unique(&source_url);
        }
    }

    pub(super) fn build_queue_item_from_url(&mut self, url: &str) -> QueueItem {
        let title = infer_title(url, "Untitled task", "Imported {tail}");
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
        item.selection.download_range.clear();
        item
    }

    pub(super) fn build_queue_item_from_seed(&mut self, seed: PlaylistEntrySeed) -> QueueItem {
        let mut item = self.build_queue_item_from_url(&seed.source_url);
        if !seed.title.trim().is_empty() {
            item.title = seed.title;
        }
        item.music_album_title = seed.album_title;
        item.thumbnail_hint = seed.thumbnail_hint;
        item.thumbnail_url = seed.thumbnail_url;
        item.duration_text = seed.duration_text;
        item.metadata_state = MetadataState::Idle;
        if item.selection.file_name.trim().is_empty() {
            item.selection.file_name = sanitize_file_name_for_windows(item.title.trim());
        }
        item
    }

    pub(super) fn prefetch_event_is_current(&self, item_id: QueueItemId, session_id: u64) -> bool {
        self.music.music_prefetch_active_item_id == Some(item_id)
            && self.music.music_prefetch_session_id == session_id
    }

    pub(super) fn transcode_temp_root_path(&self) -> PathBuf {
        self.app_cache_root_path().join("transcode-temp")
    }

    pub(super) fn rebuild_batch_input_from_queue(&mut self) {
        self.batch_input = self
            .queue_items
            .iter()
            .filter(|item| !item.source_url.trim().is_empty())
            .map(|item| item.source_url.clone())
            .collect::<Vec<_>>()
            .join("\n");
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

    pub(super) fn ensure_yt_dlp_ready(&self) -> Result<(), String> {
        self.tool_paths.validate_yt_dlp_available().map(|_| ())
    }

    pub(super) fn begin_batch_add(&mut self, source: String) {
        self.begin_batch_add_with_kind(source, false);
    }

    pub(super) fn begin_batch_add_with_kind(&mut self, source: String, music_compact: bool) {
        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.youtube_playlist_prompt = None;
            self.last_action = error;
            return;
        }

        self.youtube_playlist_prompt = None;
        self.is_adding_batch = true;
        self.is_cancelling_batch_add = false;
        self.batch_add_music_compact = music_compact;
        self.last_action =
            i18n::format_fixed_english("Adding: {source}", &[("{source}", source.as_str())]);

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
        let untitled_task = "Untitled task".to_owned();
        let imported_template = "Imported {tail}".to_owned();
        let log_mode = if music_compact {
            "audio".to_owned()
        } else {
            self.app_mode.config_value().to_owned()
        };
        let tool_log_action_id = self.push_tool_log_action(log_mode, "batch import");

        thread::spawn(move || {
            run_batch_add_worker(
                tool_paths,
                source_for_worker,
                limit,
                untitled_task,
                imported_template,
                music_compact,
                tool_log_action_id,
                tx,
                child_handle,
                cancel_requested,
            );
        });
    }
}
