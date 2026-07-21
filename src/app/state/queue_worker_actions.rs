use super::*;

impl AppState {
    pub(super) fn enqueue_item_analysis(&mut self, item_id: QueueItemId, source: String) {
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

        self.last_action =
            i18n::format_fixed_english("Analyzing: {source}", &[("{source}", source.as_str())]);
        self.spawn_analyze_worker(
            source,
            Some(item_id),
            Some(workflow_id),
            self.should_use_cookies_for_item(item_id),
        );
    }

    pub(super) fn spawn_analyze_worker(
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
                tool_log_action_id: None,
                command_line: None,
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
                tool_log_action_id: None,
                command_line: None,
                result: Err(error),
            });
            return;
        }

        let tool_log_action_id =
            Some(self.push_tool_log_action(self.app_mode.config_value(), "analyze"));

        let tool_paths = self.tool_paths.clone();
        let tx = self.analyze_result_tx.clone();
        let source_for_worker = source.clone();

        thread::spawn(move || {
            let (result, command_line) = analyze_output_parts(
                tool_paths.analyze_url_detailed(&source_for_worker, use_cookies),
            );
            let _ = tx.send(AnalyzeResult {
                source: source_for_worker,
                target_item_id,
                workflow_id,
                used_cookies: use_cookies,
                tool_log_action_id,
                command_line,
                result,
            });
        });
    }

    pub(super) fn disable_missing_aria2_for_request(&self, request: &mut DownloadRequest) -> bool {
        if !request.use_aria2 || request.target_kind == DownloadTargetKind::Subtitle {
            return false;
        }

        if dependency_tool_exists(&self.tool_paths.aria2c) {
            return false;
        }

        request.use_aria2 = false;
        true
    }

    pub(super) fn start_download_task_at(
        &mut self,
        item_id: QueueItemId,
        emit_json: bool,
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

        let Some(item) = self.queue_items.get(task_index) else {
            let error = "Analyze the video before starting download.".to_owned();
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
            merge_output_container: self
                .item_download_container_override(task_index)
                .map(str::to_owned),
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
            download_section_args: self.item_download_section_arguments(task_index),
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
            i18n::format_fixed_english(
                "Downloading: {title} (Aria2 not found; using yt-dlp native download)",
                &[("{title}", title.as_str())],
            )
        } else {
            i18n::format_fixed_english("Downloading: {title}", &[("{title}", title.as_str())])
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
            return Err("Target export item was not found.".to_owned());
        };
        if !self.item_can_export(item_index, kind) {
            return Err("This item cannot be exported right now.".to_owned());
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            if let Some(item) = self.queue_items.get_mut(item_index) {
                item.last_error = Some(error.clone());
            }
            self.last_action = error.clone();
            return Err(error);
        }
        if self.has_running_download_workflow() {
            let error =
                "A download or export is already running. Please wait for it to finish.".to_owned();
            self.last_action = error.clone();
            return Err(error);
        }

        let Some(item) = self.queue_items.get(item_index) else {
            return Err("Target export item was not found.".to_owned());
        };
        let Some(metadata) = item.metadata() else {
            return Err("Analyze the video before exporting.".to_owned());
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
                return Err("Choose subtitles before exporting.".to_owned());
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
            .ok_or_else(|| "Specify a file extension.".to_owned())?;
        validate_export_extension(kind, &export_ext)?;

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
            merge_output_container: None,
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
            download_section_args: if kind == DownloadTargetKind::Subtitle {
                Vec::new()
            } else {
                self.item_download_section_arguments(item_index)
            },
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
            DownloadTargetKind::Video => i18n::format_fixed_english(
                "Exporting video: {title}",
                &[("{title}", item_title.as_str())],
            ),
            DownloadTargetKind::Audio => i18n::format_fixed_english(
                "Exporting audio: {title}",
                &[("{title}", item_title.as_str())],
            ),
            DownloadTargetKind::Normal => i18n::format_fixed_english(
                "Downloading: {title}",
                &[("{title}", item_title.as_str())],
            ),
            DownloadTargetKind::Subtitle => i18n::format_fixed_english(
                "Exporting subtitles: {title}",
                &[("{title}", item_title.as_str())],
            ),
        };
        self.last_action = if aria2_fallback {
            i18n::format_fixed_english(
                "{action} (Aria2 not found; using yt-dlp native download)",
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
        if self.queue_structure_mutation_blocked("clearing the queue") {
            return;
        }

        self.stop_music_playback();
        self.queue_items.clear();
        self.music.music_history_back.clear();
        self.music.music_history_forward.clear();
        self.music.music_reserved_next_item_id = None;
        self.last_action = "Queue cleared.".to_owned();
        self.save_active_audio_playlist_if_needed();
    }

    pub fn remove_queue_item(&mut self, item_id: QueueItemId) {
        let Some(index) = self.queue_item_index_by_id(item_id) else {
            return;
        };

        if self.item_is_busy(index) {
            self.last_action = "Running items cannot be removed.".to_owned();
            return;
        }

        if self.music.music_player_current_item_id == Some(item_id) {
            self.stop_music_playback();
        }

        let removed = self.queue_items.remove(index);
        if self
            .music
            .music_chorus_pending_mix_target
            .as_ref()
            .is_some_and(|target| target.target_item_id == item_id)
        {
            self.cancel_music_radio_cue_pending();
        }
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
        self.last_action =
            i18n::format_fixed_english("Removed: {title}", &[("{title}", removed.title.as_str())]);
        self.prune_music_navigation_state();
        self.save_active_audio_playlist_if_needed();
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
}
