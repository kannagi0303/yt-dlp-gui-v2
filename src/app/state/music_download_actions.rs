use super::*;

impl AppState {
    pub(super) fn start_download_with_music_choice(&mut self, choice: MusicDownloadChoice) {
        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }
        if self.has_running_download_workflow() {
            self.last_action =
                "A download is already running. Please wait for it to finish.".to_owned();
            return;
        }

        self.music.active_music_download_choice = Some(choice);
        self.enqueue_download_ready_items();

        let Some(item_id) = self
            .queue_items
            .iter()
            .find(|item| item_latest_download_state(item).is_some_and(is_pending_download_state))
            .map(|item| item.id)
        else {
            self.last_action = "There are no runnable batch items.".to_owned();
            return;
        };

        if self.queue_mode_downloads_as_audio() {
            let _ = self.start_music_download_task_at(item_id, choice);
        } else {
            let emit_json = self
                .queue_item_by_id(item_id)
                .is_some_and(|item| !item.metadata_loaded());
            let _ = self.start_download_task_at(item_id, emit_json);
        }
    }

    pub(super) fn start_music_download_task_at(
        &mut self,
        item_id: QueueItemId,
        choice: MusicDownloadChoice,
    ) -> Result<(), String> {
        let Some(task_index) = self.queue_item_index_by_id(item_id) else {
            let error = "Target download item was not found.".to_owned();
            self.last_action = error.clone();
            return Err(error);
        };
        if self.has_running_download_workflow() {
            let error = "A download is already running. Please wait for it to finish.".to_owned();
            self.last_action = error.clone();
            return Err(error);
        }
        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.mark_download_preflight_failed(item_id, &error);
            self.last_action = error.clone();
            return Err(error);
        }

        self.prepare_queue_item_for_audio_mode(item_id);

        let Some(mut item) = self.queue_items.get(task_index).cloned() else {
            let error = "Analyze the video before starting download.".to_owned();
            self.last_action = error.clone();
            return Err(error);
        };

        if self.complete_music_cache_media_path(&item).is_none() {
            if let Some(hit) = self.complete_music_cache_hit_for_item(&item) {
                if let Some(target) = self.queue_items.get_mut(task_index) {
                    restore_music_compact_item_from_cache_hit(target, &hit);
                    item = target.clone();
                }
            }
        }

        let output_dir = resolve_output_dir(&item.selection.output_dir)
            .or_else(|_| resolve_output_dir(&self.item_defaults.output_dir))?;
        let cache_media_path = self.complete_music_cache_media_path(&item);
        let cover_path = self
            .music_cache_cover_path(&item)
            .filter(|path| path.is_file());
        let cover_cache_dir = self.music_cache_cover_write_dir(&item);
        let has_cover_source =
            choice.embed_cover && (cover_path.is_some() || !item.thumbnail_url.trim().is_empty());
        let source_kind = match cache_media_path.as_ref() {
            Some(path) if music_cache_can_be_copied_for_choice(choice, path, has_cover_source) => {
                MusicDownloadSourceKind::CacheCopy
            }
            Some(_) => MusicDownloadSourceKind::CacheConvert,
            None => MusicDownloadSourceKind::YtDlpDownload,
        };

        if source_kind == MusicDownloadSourceKind::CacheConvert {
            let ffmpeg = resolve_tool_path(&self.tool_paths.ffmpeg);
            if !ffmpeg.is_file() {
                let error = format!(
                    "ffmpeg.exe was not found: {}. Install FFmpeg from Options first.",
                    ffmpeg.display()
                );
                self.mark_download_preflight_failed(item_id, &error);
                self.last_action = error.clone();
                return Err(error);
            }
        }

        let workflow_id = self.alloc_workflow_run_id();
        let tool = music_download_tool_kind(source_kind);
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::DownloadMedia,
            tool.clone(),
        );
        if let Some(item) = self.queue_items.get_mut(task_index) {
            reset_item_for_new_work(item, DownloadTargetKind::Normal);
            item.completed_selection = None;
            item.selection.file_name = music_output_stem_template_for_title(&item.title);
            item.selection.audio_selector = choice.selection_token().to_owned();
            if let Some(run) = item.workflows.iter_mut().rev().find(|run| {
                run.kind == WorkflowKind::DownloadMedia
                    && matches!(run.state, WorkflowState::Queued | WorkflowState::Failed)
            }) {
                run.id = workflow_id;
                run.tool = tool;
                run.state = WorkflowState::Running;
                run.progress = 0.0;
                run.detail = item.source_url.clone();
                run.output_path = None;
                run.error = None;
            } else {
                let mut run = WorkflowRun::new(
                    workflow_id,
                    WorkflowKind::DownloadMedia,
                    tool,
                    WorkflowState::Running,
                );
                run.detail = item.source_url.clone();
                item.workflows.push(run);
            }
        }

        self.last_action = i18n::format_fixed_english(
            "Downloading music: {title}",
            &[("{title}", item.title.as_str())],
        );

        let job = MusicDownloadJob {
            item_id,
            workflow_id,
            source_url: item.source_url.clone(),
            title: item.title.clone(),
            album_title: item.music_album_title.clone(),
            output_dir,
            choice,
            source_acodec: item.music_stream_acodec.clone(),
            cache_media_path,
            cover_path,
            cover_cache_dir,
            thumbnail_url: item.thumbnail_url.clone(),
            use_cookies: self.should_use_cookies_for_item(item_id),
        };

        let tool_paths = self.tool_paths.clone();
        let tx = self.music_download_event_tx.clone();
        let child_handle = Arc::new(Mutex::new(None));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.attach_active_download_process(
            workflow_id,
            child_handle.clone(),
            cancel_requested.clone(),
        );

        thread::spawn(move || {
            run_music_download_worker(tool_paths, job, tx, child_handle, cancel_requested);
        });

        Ok(())
    }

    pub(super) fn complete_music_cache_media_path(&self, item: &QueueItem) -> Option<PathBuf> {
        complete_music_cache_media_path_in_root(item, &self.music_stream_cache_root())
    }

    pub(super) fn music_cache_cover_path(&self, item: &QueueItem) -> Option<PathBuf> {
        self.music_cache_cover_dirs(item)
            .into_iter()
            .find_map(|dir| first_music_cover_file_in_dir(&dir))
    }

    pub(super) fn music_cache_cover_write_dir(&self, item: &QueueItem) -> Option<PathBuf> {
        let key = if item.music_cache_key.trim().is_empty() {
            music_cache_key(&item.source_url, "flat", "", "")
        } else {
            item.music_cache_key.clone()
        };
        (!key.trim().is_empty()).then(|| {
            self.music_stream_cache_root()
                .join("covers")
                .join(sanitize_music_cache_key(&key))
        })
    }

    pub(super) fn music_cache_cover_dirs(&self, item: &QueueItem) -> Vec<PathBuf> {
        let cache_root = self.music_stream_cache_root();
        let mut dirs = Vec::new();
        if !item.music_cache_key.trim().is_empty() {
            let key = sanitize_music_cache_key(&item.music_cache_key);
            dirs.push(cache_root.join(&key));
            dirs.push(cache_root.join("covers").join(&key));
        }
        let flat_key = sanitize_music_cache_key(&music_cache_key(&item.source_url, "flat", "", ""));
        dirs.push(cache_root.join("covers").join(flat_key));
        dirs
    }
}
