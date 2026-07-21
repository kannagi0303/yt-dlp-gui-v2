use super::*;

impl AppState {
    pub(super) fn is_current_music_session(&self, item_id: QueueItemId, session_id: u64) -> bool {
        self.music.music_player_current_item_id == Some(item_id)
            && self.music.music_playback_session_id == session_id
    }

    pub fn add_music_compact_from_current_url(&mut self) {
        self.add_current_url_to_music_compact_batch();
    }

    pub(super) fn add_current_url_to_music_compact_batch(&mut self) {
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
                        music_compact: true,
                    });
                    self.last_action =
                        "Detected a video URL that also contains a playlist.".to_owned();
                    return;
                }
                YoutubeVideoPlaylistMode::Video => {
                    let single_source =
                        youtube_url_force_single_video(&source).unwrap_or_else(|| source.clone());
                    self.add_single_music_compact_url(single_source);
                    return;
                }
                YoutubeVideoPlaylistMode::Ignore => {}
            }
        }

        self.music.music_player_error = None;
        if !looks_like_playlist_url(&source) {
            // Align with Add: single items enter the list immediately.
            // Music compact then uses a fast flat metadata update because it does not expose format choices.
            self.add_single_music_compact_url(source);
            return;
        }

        if self.config.youtube_high_risk_playlist_prompt {
            if let Some(risk) = classify_youtube_playlist(&source) {
                self.youtube_playlist_prompt = Some(YoutubePlaylistPrompt {
                    source,
                    kind: YoutubePlaylistPromptKind::HighRiskPlaylist,
                    risk: Some(risk),
                    music_compact: true,
                });
                self.last_action = i18n::format_fixed_english(
                    "Detected high-risk YouTube playlist: {kind}",
                    &[("{kind}", risk.kind.label())],
                );
                return;
            }
        }

        // Align with Add: playlist import locks the URL row and streams items in one-by-one.
        // The only difference is the inserted item view kind.
        self.begin_music_batch_add(source);
    }

    pub(super) fn add_single_music_compact_url(&mut self, source: String) {
        let source_key = canonical_queue_source_key(&source);
        if let Some(existing_id) = self
            .queue_items
            .iter()
            .find(|item| canonical_queue_source_key(&item.source_url) == source_key)
            .map(|item| item.id)
        {
            let cache_hit = self
                .queue_item_by_id(existing_id)
                .and_then(|item| self.complete_music_cache_hit_for_item(item));
            if let Some(hit) = cache_hit {
                if let Some(item) = self.queue_item_mut_by_id(existing_id) {
                    restore_music_compact_item_from_cache_hit(item, &hit);
                }
                self.last_action =
                    "Music item is already in the list; local cache was used.".to_owned();
                self.save_active_audio_playlist_if_needed();
            } else {
                self.last_action = "Music item is already in the list.".to_owned();
                let use_cookies = self
                    .queue_item_by_id(existing_id)
                    .map(|item| item.selection.use_cookies)
                    .unwrap_or(self.item_defaults.use_cookies);
                self.spawn_music_flat_update_worker(existing_id, source.clone(), use_cookies);
            }
            self.url_input.clear();
            return;
        }

        let mut item = self.build_queue_item_from_url(&source);
        item.view_kind = QueueItemViewKind::MusicCompact;
        item.compact_music_state = Some(CompactMusicState::Ready);
        item.metadata_state = MetadataState::Idle;
        item.duration_text.clear();
        item.music_duration_seconds = None;
        item.music_stream_url.clear();
        item.music_stream_headers.clear();
        item.music_stream_ext.clear();
        item.music_stream_format_id.clear();
        item.music_stream_acodec.clear();
        item.music_stream_expected_bytes = None;
        item.music_cache_key = music_cache_key(&item.source_url, "flat", "", "");
        item.last_error = None;
        let cache_hit = self.complete_music_cache_hit_for_item(&item);
        if let Some(hit) = cache_hit.as_ref() {
            restore_music_compact_item_from_cache_hit(&mut item, hit);
        }
        let item_id = item.id;
        let title = item.title.clone();
        self.queue_items.push(item);
        self.batch_input_push_unique(&source);
        self.url_input.clear();
        self.last_action =
            i18n::format_fixed_english("Added to batch: {title}", &[("{title}", title.as_str())]);
        self.save_active_audio_playlist_if_needed();
        if cache_hit.is_none() {
            let use_cookies = self
                .queue_item_by_id(item_id)
                .map(|item| item.selection.use_cookies)
                .unwrap_or(self.item_defaults.use_cookies);
            self.spawn_music_flat_update_worker(item_id, source, use_cookies);
        } else {
            self.last_action = i18n::format_fixed_english(
                "Added music from local cache: {title}",
                &[("{title}", title.as_str())],
            );
        }
    }

    pub(super) fn spawn_music_flat_update_worker(
        &mut self,
        item_id: QueueItemId,
        source: String,
        use_cookies: bool,
    ) {
        let tx = self.music_stream_result_tx.clone();
        let tool_paths = self.tool_paths.clone();
        let untitled_task = "Untitled task".to_owned();
        let imported_template = "Imported {tail}".to_owned();
        let tool_log_action_id = self.push_tool_log_action("audio", "flat update");
        thread::spawn(move || {
            let mut command_line = String::new();
            let result = (|| -> Result<PlaylistEntrySeed, String> {
                let mut command =
                    tool_paths.prepare_music_flat_update_command(&source, use_cookies)?;
                command_line = format_process_command_line(&command);
                command
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .stdin(Stdio::null());
                let mut child = command.spawn().map_err(|error| {
                    format!("Could not start yt-dlp music flat import: {error}")
                })?;
                let _process_guard = track_child_process(&child, "yt-dlp music flat update");

                let stdout = match child.stdout.take() {
                    Some(stdout) => stdout,
                    None => {
                        terminate_child_process(&mut child);
                        let _ = child.wait();
                        return Err("Could not read yt-dlp music flat output.".to_owned());
                    }
                };
                let stderr_handle = child.stderr.take().map(|mut stderr| {
                    thread::spawn(move || {
                        let mut stderr_text = String::new();
                        let _ = stderr.read_to_string(&mut stderr_text);
                        stderr_text
                    })
                });
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                let mut first_seed = None;
                loop {
                    line.clear();
                    let read = match reader.read_line(&mut line) {
                        Ok(read) => read,
                        Err(error) => {
                            terminate_child_process(&mut child);
                            let _ = child.wait();
                            return Err(format!(
                                "Could not read yt-dlp music flat output: {error}"
                            ));
                        }
                    };
                    if read == 0 {
                        break;
                    }
                    let raw = line.trim();
                    if raw.is_empty() {
                        continue;
                    }
                    let Ok(entry) = serde_json::from_str::<Value>(raw) else {
                        continue;
                    };
                    if let Some(mut seed) =
                        playlist_entry_seed_from_json(&entry, &untitled_task, &imported_template)
                    {
                        if let Some(thumbnail_url) = select_largest_thumbnail_url(&entry) {
                            seed.thumbnail_url = thumbnail_url;
                            seed.thumbnail_hint = "Thumbnail preview".to_owned();
                        }
                        first_seed = Some(seed);
                        terminate_child_process(&mut child);
                        break;
                    }
                }
                let status = child.wait().map_err(|error| {
                    format!("Could not wait for yt-dlp music flat import: {error}")
                })?;
                let stderr_text = stderr_handle
                    .and_then(|handle| handle.join().ok())
                    .unwrap_or_default();
                if let Some(seed) = first_seed {
                    return Ok(seed);
                }
                let detail = stderr_text.trim();
                if detail.is_empty() {
                    if status.success() {
                        Err("yt-dlp did not return a music entry.".to_owned())
                    } else {
                        Err(format!(
                            "yt-dlp music flat import failed: exit code {:?}",
                            status.code()
                        ))
                    }
                } else {
                    Err(format!("yt-dlp music flat import failed: {detail}"))
                }
            })();
            if command_line.trim().is_empty() && result.is_err() {
                command_line = "yt-dlp".to_owned();
            }
            if !command_line.trim().is_empty() {
                let _ = tx.send(MusicStreamResolveEvent::ToolCommandFinished {
                    action_id: tool_log_action_id,
                    tool: "yt-dlp".to_owned(),
                    action: "flat update".to_owned(),
                    command_line,
                    success: result.is_ok(),
                });
            }
            let _ = tx.send(MusicStreamResolveEvent::FlatUpdate {
                item_id,
                source,
                result,
            });
        });
    }

    pub(super) fn begin_music_batch_add(&mut self, source: String) {
        self.begin_batch_add_with_kind(source, true);
    }

    pub(super) fn update_music_compact_item_from_seed(
        &mut self,
        item_id: QueueItemId,
        seed: PlaylistEntrySeed,
    ) {
        let cache_key = music_cache_key(&seed.source_url, "flat", "", "");
        let mut changed = false;
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            item.source_url = seed.source_url;
            if !seed.title.trim().is_empty() {
                item.title = seed.title;
            }
            item.music_album_title = seed.album_title;
            if !seed.thumbnail_url.trim().is_empty() {
                item.thumbnail_url = seed.thumbnail_url;
            }
            if !seed.thumbnail_hint.trim().is_empty() {
                item.thumbnail_hint = seed.thumbnail_hint;
            }
            if !seed.duration_text.trim().is_empty() {
                item.duration_text = seed.duration_text;
            }
            item.metadata_state = MetadataState::Idle;
            item.compact_music_state = Some(CompactMusicState::Ready);
            item.music_duration_seconds = duration_text_to_seconds(&item.duration_text);
            item.music_stream_url.clear();
            item.music_stream_headers.clear();
            item.music_stream_ext.clear();
            item.music_stream_format_id.clear();
            item.music_stream_acodec.clear();
            item.music_stream_expected_bytes = None;
            item.music_cache_key = cache_key;
            item.last_error = None;
            if item.selection.file_name.trim().is_empty() {
                item.selection.file_name = sanitize_file_name_for_windows(item.title.trim());
            }
            changed = true;
        }
        if changed {
            self.mark_font_content_changed();
        }
        self.restore_music_compact_cache_hit_if_available(item_id);
        if let Some(item) = self.queue_item_by_id(item_id) {
            self.cache_music_cover_for_item(item);
        }
    }

    pub(super) fn append_music_compact_seed(&mut self, seed: PlaylistEntrySeed) -> bool {
        let source_key = canonical_queue_source_key(&seed.source_url);
        if self
            .queue_items
            .iter()
            .any(|item| canonical_queue_source_key(&item.source_url) == source_key)
        {
            return false;
        }

        let mut item = self.build_queue_item_from_seed(seed);
        item.view_kind = QueueItemViewKind::MusicCompact;
        item.compact_music_state = Some(CompactMusicState::Ready);
        item.metadata_state = MetadataState::Idle;
        item.music_duration_seconds = duration_text_to_seconds(&item.duration_text);
        item.music_stream_url.clear();
        item.music_stream_headers.clear();
        item.music_stream_ext.clear();
        item.music_stream_format_id.clear();
        item.music_stream_acodec.clear();
        item.music_stream_expected_bytes = None;
        item.music_cache_key = music_cache_key(&item.source_url, "flat", "", "");
        item.last_error = None;
        let item_id = item.id;
        let source_url = item.source_url.clone();
        self.queue_items.push(item);
        self.mark_font_content_changed();
        self.restore_music_compact_cache_hit_if_available(item_id);
        if let Some(item) = self.queue_item_by_id(item_id) {
            self.cache_music_cover_for_item(item);
        }
        self.batch_input_push_unique(&source_url);
        true
    }

    pub(super) fn cache_music_cover_for_item(&self, item: &QueueItem) {
        let url = item.thumbnail_url.trim();
        if url.is_empty() {
            return;
        }
        let key = if item.music_cache_key.trim().is_empty() {
            music_cache_key(&item.source_url, "flat", "", "")
        } else {
            item.music_cache_key.clone()
        };
        let dir = self
            .music_stream_cache_root()
            .join("covers")
            .join(sanitize_music_cache_key(&key));
        if self.music_cache_cover_dirs(item).into_iter().any(|dir| {
            first_music_cover_file_in_dir(&dir).is_some()
                && cached_music_cover_source_matches(&dir, url)
        }) {
            return;
        }
        let url = url.to_owned();
        thread::spawn(move || {
            if let Err(error) = download_music_cover_to_dir(&url, &dir) {
                eprintln!("[music-stream] flat cover cache skipped: {error}");
            }
        });
    }

    pub(super) fn cache_music_lyrics_for_item(
        &self,
        item: &QueueItem,
        track: Option<&SubtitleOption>,
    ) {
        let Some(track) = track.filter(|track| track.source == SubtitleSource::Original) else {
            return;
        };
        let language_code = track.download_language_code.trim();
        if language_code.is_empty() {
            return;
        }
        let cache_key = if item.music_cache_key.trim().is_empty() {
            music_cache_key(&item.source_url, "lyrics", "", "")
        } else {
            item.music_cache_key.clone()
        };
        let lyrics_path = music_lrc_cache_path(&self.music_stream_cache_root(), &cache_key);
        if lyrics_path.is_file() {
            return;
        }
        let job = MusicLyricsCacheJob {
            source_url: item.source_url.clone(),
            cache_key,
            language_code: language_code.to_owned(),
            use_cookies: item.selection.use_cookies,
        };
        let tool_paths = self.tool_paths.clone();
        let cache_root = self.music_stream_cache_root();
        thread::spawn(move || {
            if let Err(error) = cache_music_lyrics_with_yt_dlp(&tool_paths, &cache_root, job) {
                eprintln!("[music-lyrics] cache skipped: {error}");
            }
        });
    }

    pub fn music_current_lyrics_display(&mut self) -> Option<MusicLyricsDisplayLine> {
        let current = self.current_music_lyrics_line_with_lead();
        self.update_music_lyrics_display_state(current)
    }
}
