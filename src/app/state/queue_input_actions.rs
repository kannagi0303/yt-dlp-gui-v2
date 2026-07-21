use super::*;

impl AppState {
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
        if !self.clipboard_monitor_baseline_ready {
            self.last_clipboard_text = text;
            self.clipboard_monitor_baseline_ready = true;
            return;
        }
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
            self.last_action = "Detected a YouTube URL from the clipboard.".to_owned();
        }
    }

    pub(super) fn clipboard_monitor_input_blocked(&self) -> bool {
        self.url_input_locked() || self.format_picker.open
    }

    pub fn clipboard_monitor_enabled(&self) -> bool {
        self.monitor_clipboard
    }

    pub fn analyze_current_input(&mut self) {
        let Some(source) = self.primary_candidate_url() else {
            self.last_action = "There is no URL to analyze.".to_owned();
            return;
        };

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        self.last_action =
            i18n::format_fixed_english("Analyzing: {source}", &[("{source}", source.as_str())]);
        self.spawn_analyze_worker(source, None, None, false);
    }

    pub fn add_current_url_to_batch(&mut self) {
        if self.is_adding_batch {
            self.last_action = "Batch add is still running.".to_owned();
            return;
        }

        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        let source = self.url_input.trim();
        if source.is_empty() {
            self.last_action = "There is no URL to add.".to_owned();
            return;
        }

        let source = source.to_owned();
        if self.app_mode == AppMode::Origin {
            if youtube_url_has_video_and_playlist(&source) {
                let single_source =
                    youtube_url_force_single_video(&source).unwrap_or_else(|| source.clone());
                self.add_single_url_to_batch(single_source);
                return;
            }
            if looks_like_playlist_url(&source) {
                self.last_action = "Origin Mode does not support playlist URLs. Switch to Standard Mode to import a playlist."
                    .to_owned();
                return;
            }
            self.add_single_url_to_batch(source);
            return;
        }

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
                        music_compact: false,
                    });
                    self.last_action =
                        "Detected a video URL that also contains a playlist.".to_owned();
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
                    music_compact: false,
                });
                self.last_action = i18n::format_fixed_english(
                    "Detected high-risk YouTube playlist: {kind}",
                    &[("{kind}", risk.kind.label())],
                );
                return;
            }
        }

        self.begin_batch_add(source);
    }

    pub fn run_primary_url_action(&mut self) {
        if self.app_mode == AppMode::Origin {
            self.add_current_url_to_batch();
        } else if self.queue_display_mode == QueueDisplayMode::Audio {
            self.add_current_url_to_music_compact_batch();
        } else if self.config.direct_download_on_add {
            self.immediate_download_current_url();
        } else {
            self.add_current_url_to_batch();
        }
    }

    pub fn primary_url_action_label_key(&self) -> &'static str {
        if self.is_adding_batch {
            if self.is_cancelling_batch_add {
                "action.stopping"
            } else {
                "action.stop"
            }
        } else if self.app_mode == AppMode::Origin {
            "action.analyze"
        } else if self.queue_display_mode == QueueDisplayMode::Audio {
            "action.add"
        } else if self.config.direct_download_on_add {
            "action.download"
        } else {
            "action.add"
        }
    }

    pub fn immediate_download_current_url(&mut self) {
        if self.is_adding_batch {
            self.last_action = "Batch add is still running.".to_owned();
            return;
        }
        let source = self.url_input.trim();
        if source.is_empty() {
            self.last_action = "There is no URL to download immediately.".to_owned();
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
            self.last_action = "Download now currently only handles one video URL.".to_owned();
            return;
        }

        let item_id = if self.app_mode == AppMode::Origin {
            if !self.active_workflows.is_empty() {
                self.last_action =
                    "Wait for the current Origin Mode item to finish first.".to_owned();
                return;
            }
            self.stop_music_playback();
            self.queue_items.clear();
            self.batch_input.clear();
            let item = self.build_queue_item_from_url(&source);
            let item_id = item.id;
            self.queue_items.push(item);
            item_id
        } else {
            self.ensure_queue_item_for_url(&source)
        };
        if self.app_mode != AppMode::Origin {
            self.url_input.clear();
        }
        let fallback_title = infer_title(&source, "Untitled task", "Imported {tail}");
        self.last_action = i18n::format_fixed_english(
            "Added and ready to download now: {title}",
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
        if prompt.music_compact {
            self.begin_music_batch_add(prompt.source);
        } else {
            self.begin_batch_add(prompt.source);
        }
    }

    pub fn confirm_youtube_playlist_prompt_as_video(&mut self) {
        let Some(prompt) = self.youtube_playlist_prompt.take() else {
            return;
        };
        let source = youtube_url_force_single_video(&prompt.source).unwrap_or(prompt.source);
        if prompt.music_compact {
            self.add_single_music_compact_url(source);
        } else {
            self.add_single_url_to_batch(source);
        }
    }

    pub fn cancel_youtube_playlist_prompt(&mut self) {
        self.youtube_playlist_prompt = None;
        self.last_action = "Current action cancelled.".to_owned();
    }

    pub fn cancel_batch_add(&mut self) {
        self.is_cancelling_batch_add = true;
        if let Some(cancel_flag) = &self.batch_add_cancel_requested {
            cancel_flag.store(true, Ordering::Relaxed);
        }
        if let Some(child_handle) = &self.batch_add_child {
            request_batch_add_stop(child_handle);
        }
        self.last_action = "Stopping batch add...".to_owned();
    }
}
