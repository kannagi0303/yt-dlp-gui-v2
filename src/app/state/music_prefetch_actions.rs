use super::*;

impl AppState {
    pub(super) fn music_prefetch_is_preparing_item(&self, item_id: QueueItemId) -> bool {
        self.music.music_prefetch_pending_item_id == Some(item_id)
            || self.music.music_prefetch_active_item_id == Some(item_id)
    }

    pub(super) fn resolve_music_item_for_playback(&mut self, item_id: QueueItemId) {
        let session_id = self.next_music_playback_session_id();
        self.resolve_music_item_for_playback_with_session(item_id, session_id);
    }

    pub(super) fn resolve_music_item_for_playback_with_session(
        &mut self,
        item_id: QueueItemId,
        session_id: u64,
    ) {
        let Some(item) = self.queue_item_by_id(item_id).cloned() else {
            return;
        };
        if item.source_url.trim().is_empty() {
            self.last_action = "Music item is missing a source URL.".to_owned();
            return;
        }
        self.mark_music_playback_state(item_id, CompactMusicState::Resolving);
        self.music.music_player_current_item_id = Some(item_id);
        self.music.music_playback_session_id = session_id;
        self.music.music_player_error = None;
        self.last_action = i18n::format_fixed_english(
            "Resolving music stream: {title}",
            &[("{title}", item.title.as_str())],
        );

        let tx = self.music_stream_result_tx.clone();
        let tool_paths = self.tool_paths.clone();
        let source = item.source_url.clone();
        let use_cookies = item.selection.use_cookies;
        let tool_log_action_id = self.push_tool_log_action("audio", "resolve stream");
        thread::spawn(move || {
            let (result, command_line, success) =
                match tool_paths.analyze_music_stream_url_detailed(&source, use_cookies) {
                    Ok(output) => {
                        let command_line = output.command_line.clone();
                        let result = music_stream_seed_from_json(&output.json, &source);
                        (result, Some(command_line), true)
                    }
                    Err(error) => (Err(error.message), error.command_line, false),
                };
            let command_line = command_line.or_else(|| (!success).then(|| "yt-dlp".to_owned()));
            if let Some(command_line) = command_line {
                let _ = tx.send(MusicStreamResolveEvent::ToolCommandFinished {
                    action_id: tool_log_action_id,
                    tool: "yt-dlp".to_owned(),
                    action: "resolve stream".to_owned(),
                    command_line,
                    success,
                });
            }
            let _ = tx.send(MusicStreamResolveEvent::Resolve {
                item_id,
                session_id,
                source,
                play_after_resolve: true,
                result,
            });
        });
    }

    pub(super) fn maybe_prefetch_next_music_item(&mut self) {
        if self.queue_display_mode != QueueDisplayMode::Audio {
            return;
        }
        if self.music.music_prefetch_active_item_id.is_some()
            || self.music.music_prefetch_pending_item_id.is_some()
        {
            return;
        }
        let Some(control) = self.music.music_playback.clone() else {
            return;
        };
        if control.is_paused() {
            return;
        }
        let current_item_id = control.item_id;
        if self.music.music_player_current_item_id != Some(current_item_id) {
            return;
        }
        if self.music.music_prefetch_for_current_item_id == Some(current_item_id) {
            return;
        }
        let played = control.playback_seconds();
        if played < MUSIC_PREFETCH_MIN_PLAY_SECONDS {
            return;
        }
        let Some(duration) = control.duration_seconds().or_else(|| {
            self.queue_item_by_id(current_item_id)
                .and_then(|item| item.music_duration_seconds)
        }) else {
            return;
        };
        if duration <= 0.0 || !duration.is_finite() {
            return;
        }
        let lead = self.music.music_prefetch_lead_seconds.clamp(
            MUSIC_PREFETCH_MIN_LEAD_SECONDS,
            MUSIC_PREFETCH_MAX_LEAD_SECONDS,
        );
        let remaining = (duration - played).max(0.0);
        if remaining > lead {
            return;
        }

        let allow_wrap = matches!(
            self.music.music_playback_mode,
            MusicPlaybackMode::RepeatAll | MusicPlaybackMode::Shuffle
        );
        let Some(next_item_id) = self.peek_next_music_item_id_for_prefetch(allow_wrap) else {
            return;
        };
        if next_item_id == current_item_id {
            return;
        }
        self.music.music_prefetch_for_current_item_id = Some(current_item_id);
        self.start_music_prefetch_for_item(next_item_id);
    }

    pub(super) fn start_music_prefetch_for_item(&mut self, item_id: QueueItemId) {
        let Some(item) = self.queue_item_by_id(item_id).cloned() else {
            return;
        };
        if self.music.music_prefetch_active_item_id == Some(item_id)
            || self.music.music_prefetch_pending_item_id == Some(item_id)
        {
            return;
        }
        if self.music.music_prefetch_active_item_id.is_some()
            || self.music.music_prefetch_pending_item_id.is_some()
        {
            // A newly selected Stage Mix target supersedes advisory prefetch
            // work for the old target. Invalidate the old prefetch session so
            // its late result cannot occupy the new target's slot.
            self.cancel_music_prefetch();
        }
        if self.complete_music_cache_media_path(&item).is_some() {
            return;
        }
        if let Some(hit) = self.complete_music_cache_hit_for_item(&item) {
            if let Some(target) = self.queue_item_mut_by_id(item_id) {
                restore_music_compact_item_from_cache_hit(target, &hit);
            }
            self.save_active_audio_playlist_if_needed();
            return;
        }
        // Measure the complete prepare cost, including a possible yt-dlp
        // resolve. The old timer started only after resolve and consistently
        // underestimated how early the following track needed to be prefetched.
        self.music.music_prefetch_started_at = Some(Instant::now());
        if item.music_stream_url.trim().is_empty() || item.music_stream_format_id.trim().is_empty()
        {
            self.resolve_music_item_for_prefetch(item_id);
            return;
        }
        self.start_resolved_music_prefetch(item);
    }

    pub(super) fn resolve_music_item_for_prefetch(&mut self, item_id: QueueItemId) {
        let Some(item) = self.queue_item_by_id(item_id).cloned() else {
            return;
        };
        if item.source_url.trim().is_empty() {
            return;
        }
        self.music.music_prefetch_pending_item_id = Some(item_id);
        let session_id = self.next_music_prefetch_session_id();
        let tx = self.music_stream_result_tx.clone();
        let tool_paths = self.tool_paths.clone();
        let source = item.source_url.clone();
        let use_cookies = item.selection.use_cookies;
        let tool_log_action_id = self.push_tool_log_action("audio", "prefetch resolve");
        thread::spawn(move || {
            let (result, command_line, success) =
                match tool_paths.analyze_music_stream_url_detailed(&source, use_cookies) {
                    Ok(output) => {
                        let command_line = output.command_line.clone();
                        let result = music_stream_seed_from_json(&output.json, &source);
                        (result, Some(command_line), true)
                    }
                    Err(error) => (Err(error.message), error.command_line, false),
                };
            let command_line = command_line.or_else(|| (!success).then(|| "yt-dlp".to_owned()));
            if let Some(command_line) = command_line {
                let _ = tx.send(MusicStreamResolveEvent::ToolCommandFinished {
                    action_id: tool_log_action_id,
                    tool: "yt-dlp".to_owned(),
                    action: "prefetch resolve".to_owned(),
                    command_line,
                    success,
                });
            }
            let _ = tx.send(MusicStreamResolveEvent::Resolve {
                item_id,
                session_id,
                source,
                play_after_resolve: false,
                result,
            });
        });
    }

    pub(super) fn start_resolved_music_prefetch(&mut self, item: QueueItem) {
        if self.complete_music_cache_media_path(&item).is_some() {
            self.music.music_prefetch_started_at = None;
            return;
        }
        let cache_root = self.music_stream_cache_root();
        let cache_dir = cache_root.join(sanitize_music_cache_key(&item.music_cache_key));
        let cache_media_path = cache_dir.join(format!(
            "audio.{}",
            sanitize_music_cache_ext(&item.music_stream_ext)
        ));
        let cache_command = match self.tool_paths.prepare_music_stream_cache_command(
            &item.source_url,
            &cache_media_path,
            &item.music_stream_format_id,
            item.selection.use_cookies,
        ) {
            Ok(command) => Some(command),
            Err(error) => {
                self.music.music_prefetch_started_at = None;
                eprintln!(
                    "[music-prefetch] prepare skipped for item={}: {error}",
                    item.id
                );
                return;
            }
        };

        let session_id = self.next_music_prefetch_session_id();
        self.music.music_prefetch_active_item_id = Some(item.id);
        self.music
            .music_prefetch_started_at
            .get_or_insert_with(Instant::now);
        let stream = ResolvedMusicStream {
            item_id: item.id,
            session_id,
            source_url: item.source_url.clone(),
            direct_url: item.music_stream_url.clone(),
            headers: item.music_stream_headers.clone(),
            title: item.title.clone(),
            album_title: item.music_album_title.clone(),
            thumbnail_url: item.thumbnail_url.clone(),
            duration_seconds: item.music_duration_seconds,
            ext: item.music_stream_ext.clone(),
            format_id: item.music_stream_format_id.clone(),
            acodec: item.music_stream_acodec.clone(),
            cache_key: item.music_cache_key.clone(),
            expected_bytes: item.music_stream_expected_bytes,
            cache_root,
            cache_command,
            volume: 0.0,
        };
        let control =
            music_stream::spawn_music_stream_prefetch(stream, self.music_playback_event_tx.clone());
        self.music.music_prefetch_control = Some(control);
    }
}
