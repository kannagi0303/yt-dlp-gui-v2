use super::*;

fn stale_music_playback_row_should_recover(
    current_item_id: Option<QueueItemId>,
    event_item_id: QueueItemId,
    state: Option<CompactMusicState>,
) -> bool {
    current_item_id != Some(event_item_id)
        && matches!(
            state,
            Some(
                CompactMusicState::Resolving
                    | CompactMusicState::Buffering
                    | CompactMusicState::Playing
                    | CompactMusicState::Paused
            )
        )
}

impl AppState {
    pub(super) fn apply_music_stream_result(&mut self, message: MusicStreamResolveEvent) {
        match message {
            MusicStreamResolveEvent::ToolCommandFinished {
                action_id,
                tool,
                action,
                command_line,
                success,
            } => {
                let status = if success {
                    ToolLogStatus::Success
                } else if action == "prefetch resolve" {
                    ToolLogStatus::Skipped
                } else {
                    ToolLogStatus::Failed
                };
                self.push_tool_log_step(action_id, status, tool, action, command_line);
            }
            MusicStreamResolveEvent::FlatImport { source, result } => match result {
                Ok(seeds) => {
                    let mut added = 0usize;
                    for seed in seeds {
                        if self.append_music_compact_seed(seed) {
                            added += 1;
                        }
                    }
                    self.music.music_player_error = None;
                    self.last_action = if added == 0 {
                        i18n::format_fixed_english(
                            "No music items could be added: {source}",
                            &[("{source}", source.as_str())],
                        )
                    } else {
                        i18n::format_fixed_english(
                            "Added {count} music items.",
                            &[("{count}", &added.to_string())],
                        )
                    };
                    self.save_active_audio_playlist_if_needed();
                }
                Err(error) => {
                    let message = i18n::format_fixed_english(
                        "Music list analysis failed: {error}",
                        &[("{error}", error.as_str())],
                    );
                    self.music.music_player_error = Some(message.clone());
                    self.push_runtime_log(message.clone());
                    self.last_action = message;
                    eprintln!("[music-stream] flat import failed: {error}");
                }
            },
            MusicStreamResolveEvent::FlatUpdate {
                item_id,
                source,
                result,
            } => {
                match result {
                    Ok(seed) => {
                        self.update_music_compact_item_from_seed(item_id, seed);
                        self.music.music_player_error = None;
                        self.save_active_audio_playlist_if_needed();
                    }
                    Err(error) => {
                        // Flat metadata is an enhancement for single-item compact rows.
                        // Keep the row playable by URL even when the fast metadata probe fails.
                        eprintln!(
                            "[music-stream] flat update skipped for item={item_id} source={source}: {error}"
                        );
                    }
                }
            }
            MusicStreamResolveEvent::Resolve {
                item_id,
                session_id,
                source,
                play_after_resolve,
                result,
            } => {
                let event_is_current = if play_after_resolve {
                    self.is_current_music_session(item_id, session_id)
                } else {
                    self.music.music_prefetch_pending_item_id == Some(item_id)
                        && self.music.music_prefetch_session_id == session_id
                };
                if !event_is_current {
                    eprintln!(
                        "[music-stream] ignored stale resolve result for item={item_id} session={session_id} play={play_after_resolve}"
                    );
                    return;
                }

                match result {
                    Ok(seed) => {
                        let lyrics_track = seed.lyrics_track.clone();
                        let mut updated_item = None;
                        if let Some(item) = self.queue_item_mut_by_id(item_id) {
                            item.source_url = seed.source_url;
                            item.title = seed.title;
                            if !seed.album_title.trim().is_empty() {
                                item.music_album_title = seed.album_title;
                            }
                            if !seed.thumbnail_url.trim().is_empty() {
                                item.thumbnail_url = seed.thumbnail_url;
                            }
                            if !seed.thumbnail_hint.trim().is_empty() {
                                item.thumbnail_hint = seed.thumbnail_hint;
                            }
                            if !seed.duration_text.trim().is_empty() {
                                item.duration_text = seed.duration_text;
                            }
                            item.music_duration_seconds = seed.duration_seconds;
                            item.music_stream_url = seed.direct_url;
                            item.music_stream_headers = seed.headers;
                            item.music_stream_ext = seed.ext;
                            item.music_stream_format_id = seed.format_id;
                            item.music_stream_acodec = seed.acodec;
                            item.music_stream_expected_bytes = seed.expected_bytes;
                            item.music_cache_key = seed.cache_key;
                            item.metadata_state = MetadataState::Idle;
                            item.compact_music_state = Some(CompactMusicState::Ready);
                            item.last_error = None;
                            if item.selection.file_name.trim().is_empty() {
                                item.selection.file_name =
                                    sanitize_file_name_for_windows(item.title.trim());
                            }
                            updated_item = Some(item.clone());
                        }
                        if let Some(item) = updated_item.as_ref() {
                            self.mark_font_content_changed();
                            self.cache_music_lyrics_for_item(item, lyrics_track.as_ref());
                        }
                        if !play_after_resolve {
                            let is_current_prefetch_resolve =
                                self.music.music_prefetch_pending_item_id == Some(item_id)
                                    && self.music.music_prefetch_session_id == session_id;
                            if is_current_prefetch_resolve {
                                self.music.music_prefetch_pending_item_id = None;
                                if let Some(item) = updated_item {
                                    self.start_resolved_music_prefetch(item);
                                }
                                self.save_active_audio_playlist_if_needed();
                            }
                            return;
                        }
                        self.music.music_player_error = None;
                        self.last_action = i18n::format_fixed_english(
                            "Music stream ready: {source}",
                            &[("{source}", source.as_str())],
                        );
                        self.save_active_audio_playlist_if_needed();
                        if play_after_resolve {
                            self.start_music_stream_playback_with_session(item_id, session_id);
                        }
                    }
                    Err(error) => {
                        if !play_after_resolve {
                            self.music.music_prefetch_pending_item_id = None;
                            self.music.music_prefetch_started_at = None;
                            eprintln!(
                                "[music-prefetch] resolve skipped for item={item_id}: {error}"
                            );
                            return;
                        }
                        if let Some(item) = self.queue_item_mut_by_id(item_id) {
                            item.metadata_state = MetadataState::Failed(error.clone());
                            item.compact_music_state = Some(CompactMusicState::Failed);
                            item.last_error = Some(error.clone());
                        }
                        let current_track_continues =
                            self.recover_music_ready_handoff_after_target_failure(item_id);
                        let message = i18n::format_fixed_english(
                            if current_track_continues {
                                "Target preparation failed; current track continues: {error}"
                            } else {
                                "Music stream analysis failed: {error}"
                            },
                            &[("{error}", error.as_str())],
                        );
                        self.music.music_player_error = Some(message.clone());
                        self.push_runtime_log(message.clone());
                        eprintln!("[music-stream] resolve failed: {error}");
                        self.last_action = message;
                    }
                }
            }
        }
    }

    pub(super) fn apply_music_download_event(&mut self, event: MusicDownloadEvent) {
        match event {
            MusicDownloadEvent::Progress {
                item_id,
                workflow_id,
                percent,
            } => {
                if let Some(item) = self.queue_item_mut_by_id(item_id) {
                    let display_percent = percent.clamp(0.0, 100.0);
                    item.progress.audio = monotonic_progress(item.progress.audio, display_percent);
                    if let Some(run) = item.workflows.iter_mut().find(|run| run.id == workflow_id) {
                        run.progress = monotonic_progress(run.progress, display_percent);
                    }
                }
            }
            MusicDownloadEvent::ToolCommandFinished {
                item_id: _,
                workflow_id,
                source_kind,
                tool,
                action,
                command_line,
                success,
            } => {
                let action_id = self.workflow_tool_log_action(
                    workflow_id,
                    "audio",
                    music_source_kind_log_action(source_kind),
                );
                self.push_tool_log_step(
                    action_id,
                    self.tool_log_status_for_workflow_step(workflow_id, success),
                    tool,
                    action,
                    command_line,
                );
            }
            MusicDownloadEvent::Finished {
                item_id,
                workflow_id,
                source_kind,
                result,
            } => {
                self.unregister_active_workflow(workflow_id);
                self.finish_workflow_tool_log(workflow_id);
                if let Some(item) = self.queue_item_mut_by_id(item_id) {
                    if let Some(run) = item.workflows.iter_mut().find(|run| run.id == workflow_id) {
                        match &result {
                            Ok(output_path) => {
                                run.state = WorkflowState::Finished;
                                run.progress = 100.0;
                                run.output_path = Some(output_path.clone());
                                item.progress.audio = 100.0;
                                item.progress.video = 100.0;
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
                            Err(error) if error == DOWNLOAD_CANCELLED_MESSAGE => {
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

                match result {
                    Ok(output_path) => {
                        let source_label = match source_kind {
                            MusicDownloadSourceKind::CacheCopy => "cache copy",
                            MusicDownloadSourceKind::CacheConvert => "cache convert",
                            MusicDownloadSourceKind::YtDlpOnlineTarget => "yt-dlp online target",
                            MusicDownloadSourceKind::YtDlpDownload => "yt-dlp",
                        };
                        self.push_runtime_log(format!(
                            "Music download finished ({source_label}): {output_path}"
                        ));
                        self.last_action.clear();
                    }
                    Err(error) if error == DOWNLOAD_CANCELLED_MESSAGE => {
                        self.push_runtime_log("Music download cancelled".to_owned());
                        self.last_action = "Download stopped.".to_owned();
                    }
                    Err(error) => {
                        self.push_runtime_log(format!("Music download failed: {error}"));
                        eprintln!("[music-download] {error}");
                        self.last_action = error;
                    }
                }
                self.start_next_queued_download_after(item_id);
            }
        }
    }

    fn repair_stale_music_playback_row(&mut self, item_id: QueueItemId) {
        let state = self
            .queue_item_by_id(item_id)
            .and_then(|item| item.compact_music_state);
        if stale_music_playback_row_should_recover(
            self.music.music_player_current_item_id,
            item_id,
            state,
        ) {
            self.mark_music_playback_state(item_id, CompactMusicState::Ready);
        }
    }

    pub(super) fn apply_music_playback_event(&mut self, event: MusicPlaybackEvent) {
        match event {
            MusicPlaybackEvent::ToolCommandFinished {
                item_id,
                session_id,
                tool,
                action,
                command_line,
                success,
            } => {
                if !self.is_current_music_session(item_id, session_id) {
                    return;
                }
                let action_id = self.push_tool_log_action("audio", "playback cache");
                self.push_tool_log_step(
                    action_id,
                    if success {
                        ToolLogStatus::Success
                    } else {
                        ToolLogStatus::Failed
                    },
                    tool,
                    action,
                    command_line,
                );
            }
            MusicPlaybackEvent::Started {
                item_id,
                session_id,
            } => {
                if !self.is_current_music_session(item_id, session_id) {
                    self.repair_stale_music_playback_row(item_id);
                    eprintln!(
                        "[music-stream] ignored stale playback start for item={item_id} session={session_id}"
                    );
                    return;
                }
                self.music.music_player_error = None;
                self.music.music_player_current_item_id = Some(item_id);
                self.mark_music_playback_state(item_id, CompactMusicState::Playing);
                self.start_music_chorus_fade_in_if_pending(item_id, session_id);
                self.release_music_playback_ready_handoff_if_ready(item_id);
                self.release_music_chorus_handoff_bridge_if_ready(item_id, session_id);
            }
            MusicPlaybackEvent::Finished {
                item_id,
                session_id,
            } => {
                if !self.is_current_music_session(item_id, session_id) {
                    // A promoted Stage Mix / preview deck can deliver an old finish event after
                    // a newer session has already taken over.  Do not advance the queue, and do
                    // not print this as an alarming stream error.  Just repair stale UI state
                    // when the stale item is no longer the active playback target.
                    self.clear_music_ready_handoff_source_if_matches(item_id, session_id);
                    self.repair_stale_music_playback_row(item_id);
                    return;
                }
                if self.finish_music_chorus_transition_on_stream_finished(item_id, session_id) {
                    return;
                }
                self.mark_music_playback_state(item_id, CompactMusicState::Ready);
                self.music.music_playback = None;
                self.music.music_chorus_flow_segment = None;
                self.clear_music_chorus_transition();
                self.music.music_player_error = None;
                self.last_action = "Playback finished.".to_owned();
                self.advance_music_after_finished(item_id);
            }
            MusicPlaybackEvent::Stopped {
                item_id,
                session_id,
            } => {
                if !self.is_current_music_session(item_id, session_id) {
                    self.clear_music_ready_handoff_source_if_matches(item_id, session_id);
                    self.repair_stale_music_playback_row(item_id);
                    eprintln!(
                        "[music-stream] ignored stale playback stop for item={item_id} session={session_id}"
                    );
                    return;
                }
                self.music.music_playback = None;
                self.music.music_chorus_flow_segment = None;
                self.clear_music_chorus_transition();
                if self
                    .queue_item_by_id(item_id)
                    .is_some_and(|item| item.compact_music_state != Some(CompactMusicState::Failed))
                {
                    self.mark_music_playback_state(item_id, CompactMusicState::Ready);
                }
            }
            MusicPlaybackEvent::Failed {
                item_id,
                session_id,
                error,
            } => {
                if !self.is_current_music_session(item_id, session_id) {
                    // Stale event from a playback session that was replaced by user action.
                    // Do not poison the row/cache health for a deliberate track switch.
                    self.clear_music_ready_handoff_source_if_matches(item_id, session_id);
                    self.repair_stale_music_playback_row(item_id);
                    eprintln!(
                        "[music-stream] ignored stale playback failure for item={item_id} session={session_id}: {error}"
                    );
                    return;
                }
                self.mark_music_playback_state(item_id, CompactMusicState::Failed);
                let current_track_continues =
                    self.recover_music_ready_handoff_after_target_failure(item_id);
                if !current_track_continues {
                    self.music.music_playback = None;
                    self.music.music_chorus_flow_segment = None;
                    self.clear_music_chorus_transition();
                    self.music.music_player_current_item_id = None;
                }
                if let Some(item) = self.queue_item_mut_by_id(item_id) {
                    item.last_error = Some(error.clone());
                }
                let message = i18n::format_fixed_english(
                    if current_track_continues {
                        "Target playback failed; current track continues: {error}"
                    } else {
                        "Playback failed: {error}"
                    },
                    &[("{error}", error.as_str())],
                );
                self.music.music_player_error = Some(message.clone());
                self.push_runtime_log(message.clone());
                eprintln!("[music-stream] playback failed: {error}");
                self.last_action = message;
            }
            MusicPlaybackEvent::PrefetchToolCommandFinished {
                item_id,
                session_id,
                tool,
                action,
                command_line,
                success,
            } => {
                if !self.prefetch_event_is_current(item_id, session_id) {
                    return;
                }
                let action_id = self.push_tool_log_action("audio", "prefetch cache");
                let status = if success {
                    ToolLogStatus::Success
                } else {
                    ToolLogStatus::Skipped
                };
                self.push_tool_log_step(action_id, status, tool, action, command_line);
            }
            MusicPlaybackEvent::PrefetchFinished {
                item_id,
                session_id,
                success,
                error,
            } => {
                if !self.prefetch_event_is_current(item_id, session_id) {
                    return;
                }
                if success {
                    if let Some(started) = self.music.music_prefetch_started_at {
                        let elapsed = Instant::now().duration_since(started).as_secs_f64();
                        if elapsed.is_finite() && elapsed > 0.0 {
                            self.music.music_prefetch_lead_seconds =
                                music_prefetch_lead_seconds_after_success(elapsed);
                        }
                    }
                } else if let Some(error) = error {
                    eprintln!("[music-prefetch] cache skipped for item={item_id}: {error}");
                }
                if self.music.music_prefetch_active_item_id == Some(item_id) {
                    self.music.music_prefetch_control = None;
                }
                self.music.music_prefetch_active_item_id = None;
                self.music.music_prefetch_started_at = None;
            }
        }
    }
}

fn music_prefetch_lead_seconds_after_success(elapsed_seconds: f64) -> f64 {
    (elapsed_seconds * MUSIC_PREFETCH_SPEED_MULTIPLIER + MUSIC_PREFETCH_STARTUP_SAFETY_SECONDS)
        .clamp(
            MUSIC_PREFETCH_MIN_LEAD_SECONDS,
            MUSIC_PREFETCH_MAX_LEAD_SECONDS,
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_event_recovers_only_a_non_current_transient_row() {
        assert!(stale_music_playback_row_should_recover(
            Some(2),
            1,
            Some(CompactMusicState::Buffering),
        ));
        assert!(!stale_music_playback_row_should_recover(
            Some(1),
            1,
            Some(CompactMusicState::Buffering),
        ));
        assert!(!stale_music_playback_row_should_recover(
            Some(2),
            1,
            Some(CompactMusicState::Failed),
        ));
    }

    #[test]
    fn successful_prefetch_lead_includes_resolve_and_startup_safety() {
        assert_eq!(music_prefetch_lead_seconds_after_success(8.0), 25.0);
        assert_eq!(music_prefetch_lead_seconds_after_success(1.0), 20.0);
        assert_eq!(music_prefetch_lead_seconds_after_success(60.0), 90.0);
    }
}
