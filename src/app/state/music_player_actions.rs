use super::*;

impl AppState {
    pub(super) fn music_item_active_download_progress_ratio(
        &self,
        item_id: QueueItemId,
    ) -> Option<f32> {
        let item = self.queue_item_by_id(item_id)?;
        let has_active_download = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item_id && workflow.kind == WorkflowKind::DownloadMedia
        });
        if !has_active_download {
            return None;
        }
        let progress = item
            .progress
            .audio
            .max(item.progress.video)
            .max(item.progress.post_process);
        Some(progress.clamp(0.0, 100.0) / 100.0)
    }

    pub fn music_item_compact_progress_status_text(&self, item_id: QueueItemId) -> Option<String> {
        self.music_item_active_download_progress_ratio(item_id)
            .map(|ratio| format!("{}%", (ratio * 100.0).round().clamp(0.0, 99.0) as u32))
    }

    pub fn music_item_playback_progress_ratio(&self, item_id: QueueItemId) -> f32 {
        self.music
            .music_playback
            .as_ref()
            .filter(|control| control.item_id == item_id)
            .map(MusicPlaybackControl::progress_ratio)
            .unwrap_or(0.0)
    }

    pub fn music_player_is_playing(&self) -> bool {
        self.music
            .music_playback
            .as_ref()
            .is_some_and(|control| !control.is_paused())
    }

    pub fn music_player_error_text(&self) -> Option<&str> {
        self.music
            .music_player_error
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub fn has_music_playback_activity(&self) -> bool {
        self.music.music_playback.is_some()
            || self.queue_items.iter().any(|item| {
                matches!(
                    item.compact_music_state,
                    Some(
                        CompactMusicState::Resolving
                            | CompactMusicState::Buffering
                            | CompactMusicState::Playing
                            | CompactMusicState::Paused
                    )
                )
            })
    }

    pub fn music_playback_progress_ratio(&self) -> f32 {
        let Some(control) = self.music.music_playback.as_ref() else {
            return 0.0;
        };
        let Some(duration) = control
            .duration_seconds()
            .filter(|duration| *duration > 0.0)
        else {
            return control.progress_ratio();
        };
        let seconds = self
            .music_current_playback_seconds()
            .unwrap_or_else(|| control.playback_seconds());
        (seconds / duration).clamp(0.0, 1.0) as f32
    }

    pub fn music_playback_cache_progress_ratio(&self) -> f32 {
        self.music
            .music_playback
            .as_ref()
            .map(MusicPlaybackControl::cache_progress_ratio)
            .unwrap_or_else(|| {
                self.music
                    .music_player_current_item_id
                    .map(|id| self.music_cached_progress_for_item(id))
                    .unwrap_or(0.0)
            })
    }

    pub fn music_seek_drag_ratio(&self) -> Option<f32> {
        self.music.music_seek_drag_ratio
    }

    pub fn music_seek_display_ratio(&mut self) -> f32 {
        if let Some(value) = self.music.music_seek_drag_ratio {
            return value.clamp(0.0, 1.0);
        }
        if let (Some(value), Some(deadline)) = (
            self.music.music_seek_snap_ratio,
            self.music.music_seek_snap_deadline,
        ) {
            if Instant::now() <= deadline {
                return value.clamp(0.0, 1.0);
            }
            self.music.music_seek_snap_ratio = None;
            self.music.music_seek_snap_deadline = None;
        }
        self.music_playback_progress_ratio()
    }

    pub fn set_music_seek_drag_ratio(&mut self, ratio: Option<f32>) {
        self.music.music_seek_drag_ratio = ratio.map(|value| value.clamp(0.0, 1.0));
        if ratio.is_some() {
            self.music.music_seek_snap_ratio = None;
            self.music.music_seek_snap_deadline = None;
        }
    }

    pub fn finish_music_seek_drag(&mut self, ratio: f32) {
        self.music.music_seek_drag_ratio = None;
        self.seek_music_playback_ratio(ratio);
    }

    pub fn seek_music_playback_ratio(&mut self, ratio: f32) {
        let Some(control) = self.music.music_playback.clone() else {
            return;
        };
        let requested = ratio.clamp(0.0, 1.0);
        let cache_ratio = control.cache_progress_ratio().clamp(0.0, 1.0);
        let safe_cache_ratio = if control.cache_is_complete() {
            1.0
        } else {
            // Avoid seeking exactly at the growing-file edge; keep a tiny guard
            // so UI over-drag snaps to a point that is actually buffered.
            (cache_ratio - 0.01).max(0.0)
        };
        let allowed = requested.min(safe_cache_ratio);
        if requested > allowed + f32::EPSILON {
            self.music.music_seek_snap_ratio = Some(allowed);
            self.music.music_seek_snap_deadline = Some(Instant::now() + Duration::from_millis(700));
            self.last_action =
                "Outside the cached range; moved back to a playable position.".to_owned();
        } else {
            self.music.music_seek_snap_ratio = None;
            self.music.music_seek_snap_deadline = None;
        }
        let target_seconds = control
            .duration_seconds()
            .map(|duration| duration * f64::from(allowed));
        let promoted_crossfade_main = control.has_promoted_crossfade_main();
        self.music.music_manual_seek_grace_until = Some(Instant::now() + Duration::from_secs(6));
        self.music.music_chorus_fade_out = None;
        self.music.music_chorus_fade_in = None;
        self.music.music_chorus_pending_mix_target = None;
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;
        self.music.music_chorus_pending_fade_in = None;
        self.music.music_chorus_pending_start = None;
        self.music.music_smooth_seek = None;

        if self.music_auto_transition_enabled() {
            control.clear_crossfade_preview();
            control.set_volume(self.music.music_volume);
        }

        if promoted_crossfade_main {
            if let Some(target_seconds) = target_seconds {
                // A promoted crossfade deck is a temporary in-stream preview deck.
                // Decoder-level seek targets apply to the original decoder, not to
                // that promoted preview buffer, so user seek must materialize the
                // current item as a normal playback stream at the requested time.
                self.prepare_music_chorus_start_for_item(control.item_id, target_seconds);
                self.start_music_stream_playback_recorded(control.item_id, false);
                self.last_action = "Seeked current cue as normal playback.".to_owned();
                return;
            }
        }

        control.seek_to_ratio(allowed);
        if let Some(target_seconds) = target_seconds {
            self.reanchor_music_chorus_flow_after_manual_seek(&control, target_seconds);
        }
    }

    pub fn music_playback_time_text(&mut self) -> String {
        let Some(control) = self.music.music_playback.clone() else {
            return "00:00 / --:--".to_owned();
        };
        let duration = control.duration_seconds();
        let preview_ratio = (self.music.music_seek_drag_ratio.is_some()
            || self.music.music_seek_snap_ratio.is_some())
        .then(|| self.music_seek_display_ratio().clamp(0.0, 1.0));
        let current_seconds = match (preview_ratio, duration) {
            (Some(ratio), Some(duration)) => duration * f64::from(ratio),
            _ => control.playback_seconds(),
        };
        let current = format_duration_seconds(current_seconds);
        let total = duration
            .map(format_duration_seconds)
            .unwrap_or_else(|| "--:--".to_owned());
        format!("{current} / {total}")
    }

    pub fn music_volume(&self) -> f32 {
        self.music.music_volume
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        let next = volume.clamp(0.0, 1.0);
        if (self.music.music_volume - next).abs() < 0.001 {
            return;
        }
        self.music.music_volume = next;
        self.config.music_volume = next;
        let _ = self.config.save();
        if let Some(fade) = self.music.music_chorus_fade_out.as_mut() {
            fade.target_volume = self.music.music_volume;
        }
        if let Some(fade) = self.music.music_chorus_fade_in.as_mut() {
            fade.target_volume = self.music.music_volume;
        }
        if let Some(pending) = self.music.music_chorus_pending_fade_in.as_mut() {
            pending.target_volume = self.music.music_volume;
        }
        if let Some(control) = &self.music.music_playback {
            if self.music.music_chorus_fade_out.is_none()
                && self.music.music_chorus_fade_in.is_none()
            {
                control.set_volume(self.music.music_volume);
            }
        }
    }

    pub fn toggle_music_playback(&mut self) {
        let preparing_handoff_target = self
            .music
            .music_playback_ready_handoff
            .as_ref()
            .filter(|bridge| bridge.stop_output_frame.is_none())
            .map(|bridge| bridge.target_item_id);
        if let Some(target_item_id) = preparing_handoff_target {
            // Pause remains authoritative while a replacement stream is still
            // resolving/buffering: abandon B, restore audible A, then pause A.
            if self.recover_music_ready_handoff_after_target_failure(target_item_id) {
                if let Some(control) = self.music.music_playback.clone() {
                    control.pause();
                    self.mark_music_playback_state(control.item_id, CompactMusicState::Paused);
                }
                self.last_action = "Pending track switch cancelled by pause.".to_owned();
                return;
            }
        }

        if let Some(control) = self.music.music_playback.clone() {
            if control.is_paused() {
                // A pending cue must not survive a paused state and then fire
                // immediately after resume. Playback control remains primary.
                self.cancel_music_radio_cue_pending_and_reanchor();
                if self.music.music_automix_enabled {
                    control.set_volume(0.0);
                    control.resume();
                    control.fade_volume_to(self.music.music_volume, Duration::from_millis(420));
                } else {
                    control.resume();
                }
                self.mark_music_playback_state(control.item_id, CompactMusicState::Playing);
            } else {
                self.cancel_music_radio_cue_pending_with_message("Mix next cancelled by pause.");
                if self.music.music_automix_enabled {
                    control.fade_volume_to(0.0, Duration::from_millis(260));
                }
                control.pause();
                self.mark_music_playback_state(control.item_id, CompactMusicState::Paused);
            }
            return;
        }

        let item_id = self
            .music
            .music_player_current_item_id
            .filter(|id| self.music_item_can_play(*id))
            .or_else(|| {
                self.queue_items
                    .iter()
                    .find(|item| self.music_item_can_play(item.id))
                    .map(|item| item.id)
            });

        if let Some(item_id) = item_id {
            self.start_music_stream_playback(item_id);
        } else {
            self.last_action = "There are no playable music items.".to_owned();
        }
    }

    pub fn stop_music_playback(&mut self) {
        let current_item_id = self.music.music_player_current_item_id;
        if let Some(control) = self.music.music_playback.take() {
            control.stop();
            self.mark_music_playback_state(control.item_id, CompactMusicState::Ready);
        }
        if let Some(item_id) = current_item_id {
            // Resolving has no playback control to stop. Reset the row here so
            // an abandoned resolver cannot leave Music Mode permanently busy.
            self.mark_music_playback_state(item_id, CompactMusicState::Ready);
        }
        if let Some(bridge) = self.music.music_chorus_handoff_bridge.take() {
            bridge.control.stop();
        }
        if let Some(handoff) = self.music.music_playback_ready_handoff.take() {
            handoff.control.stop();
        }
        self.next_music_playback_session_id();
        self.music.music_player_current_item_id = None;
        self.music.music_player_error = None;
        self.music.music_seek_drag_ratio = None;
        self.music.music_seek_snap_ratio = None;
        self.music.music_seek_snap_deadline = None;
        self.music.music_manual_seek_grace_until = None;
        self.music.music_smooth_seek = None;
        self.music.music_chorus_flow_segment = None;
        self.clear_music_chorus_transition();
        self.cancel_music_prefetch();
        self.music.media_session.clear();
    }

    fn clear_music_playback_session_advisory_state(&mut self) {
        // Keep pending start/fade-in: Stage Chain prepares those for the new
        // real stream before assigning its session. Everything below belongs
        // to the old playback session and must not survive a manual restart.
        self.music.music_chorus_flow_segment = None;
        self.music.music_chorus_mix_plan = None;
        self.music.music_chorus_fade_out = None;
        self.music.music_chorus_fade_in = None;
        self.music.music_chorus_pending_mix_target = None;
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;
        self.music.music_smooth_seek = None;
    }

    pub(super) fn cancel_music_prefetch(&mut self) {
        if let Some(control) = self.music.music_prefetch_control.take() {
            control.cancel();
        }
        self.music.music_prefetch_active_item_id = None;
        self.music.music_prefetch_pending_item_id = None;
        self.music.music_prefetch_for_current_item_id = None;
        self.music.music_prefetch_started_at = None;
        self.music.music_prefetch_session_id =
            self.music.music_prefetch_session_id.wrapping_add(1).max(1);
    }

    fn prepare_music_ready_handoff_bridge(&mut self, target_item_id: QueueItemId) {
        if self.music.music_chorus_handoff_bridge.is_some() {
            // Stage Mix already owns A -> B timing and fade execution.
            return;
        }
        if let Some(mut handoff) = self.music.music_playback_ready_handoff.take() {
            if handoff.stop_output_frame.is_none() {
                // A second manual choice while B is still preparing should
                // keep the same audible A source and retarget the pending handoff.
                handoff.target_item_id = target_item_id;
                self.music.music_playback_ready_handoff = Some(handoff);
                return;
            }
            handoff.control.stop();
        }

        let Some(control) = self.music.music_playback.clone() else {
            return;
        };
        if !music_ready_handoff_should_keep_source_playing(
            control.item_id,
            target_item_id,
            self.music.music_player_current_item_id,
            control.is_paused(),
        ) {
            return;
        }

        self.music.music_playback_ready_handoff = Some(MusicPlaybackReadyHandoff {
            control,
            target_item_id,
            // None means A remains audible until B emits Started. Stage Mix
            // timed handoffs use an explicit frame and are not changed here.
            stop_output_frame: None,
        });
    }

    pub(super) fn recover_music_ready_handoff_after_target_failure(
        &mut self,
        target_item_id: QueueItemId,
    ) -> bool {
        let recoverable = self
            .music
            .music_playback_ready_handoff
            .as_ref()
            .is_some_and(|bridge| {
                bridge.target_item_id == target_item_id && bridge.stop_output_frame.is_none()
            });
        if !recoverable {
            return false;
        }
        let Some(bridge) = self.music.music_playback_ready_handoff.take() else {
            return false;
        };
        let source = bridge.control;
        let source_has_remaining_audio = music_ready_handoff_source_has_remaining_audio(
            source.playback_seconds(),
            source.duration_seconds(),
        );
        if !source_has_remaining_audio {
            source.stop();
            return false;
        }

        if let Some(target_control) = self
            .music
            .music_playback
            .take()
            .filter(|control| control.item_id == target_item_id)
        {
            target_control.stop();
        }
        source.set_volume(self.music.music_volume);
        let source_state = if source.is_paused() {
            CompactMusicState::Paused
        } else {
            CompactMusicState::Playing
        };
        self.music.music_playback = Some(source.clone());
        self.music.music_player_current_item_id = Some(source.item_id);
        self.music.music_playback_session_id = source.session_id;
        self.mark_music_playback_state(source.item_id, source_state);
        self.music.music_chorus_flow_segment = None;
        self.music.music_chorus_mix_plan = None;
        self.music.music_chorus_fade_out = None;
        self.music.music_chorus_fade_in = None;
        self.music.music_chorus_pending_mix_target = None;
        self.music.music_chorus_preview_job = None;
        self.music.music_chorus_ready_preview = None;
        self.music.music_chorus_pending_fade_in = None;
        self.music.music_chorus_pending_start = None;
        true
    }

    pub(super) fn clear_music_ready_handoff_source_if_matches(
        &mut self,
        item_id: QueueItemId,
        session_id: u64,
    ) -> bool {
        let matches = self
            .music
            .music_playback_ready_handoff
            .as_ref()
            .is_some_and(|handoff| {
                music_ready_handoff_source_identity_matches(
                    handoff.control.item_id,
                    handoff.control.session_id,
                    item_id,
                    session_id,
                )
            });
        if !matches {
            return false;
        }
        self.music.music_playback_ready_handoff = None;
        self.mark_music_playback_state(item_id, CompactMusicState::Ready);
        true
    }

    pub(super) fn next_music_playback_session_id(&mut self) -> u64 {
        self.music.music_playback_session_id =
            self.music.music_playback_session_id.wrapping_add(1).max(1);
        self.music.music_playback_session_id
    }

    pub(super) fn start_music_stream_playback(&mut self, item_id: QueueItemId) {
        self.start_music_stream_playback_recorded(item_id, true);
    }

    pub(super) fn start_music_stream_playback_recorded(
        &mut self,
        item_id: QueueItemId,
        record_history: bool,
    ) {
        if record_history {
            self.record_music_navigation_target(item_id);
        }
        self.prepare_music_mode_start_for_item(item_id);
        self.clear_music_playback_session_advisory_state();
        let session_id = self.next_music_playback_session_id();
        self.start_music_stream_playback_with_session(item_id, session_id);
    }

    pub(super) fn start_music_stream_playback_with_session(
        &mut self,
        item_id: QueueItemId,
        session_id: u64,
    ) {
        if self
            .music
            .music_playback
            .as_ref()
            .is_some_and(|control| control.session_id != session_id)
        {
            self.clear_music_playback_session_advisory_state();
        }
        if self
            .queue_item_by_id(item_id)
            .is_some_and(|item| item.compact_music_state == Some(CompactMusicState::Failed))
        {
            // A failed direct URL may have expired. Force the next attempt
            // through the normal complete-cache restore / fresh resolve path
            // instead of replaying the same poisoned transport information.
            if let Some(item) = self.queue_item_mut_by_id(item_id) {
                item.music_stream_url.clear();
                item.music_stream_headers.clear();
                item.compact_music_state = Some(CompactMusicState::Ready);
                item.last_error = None;
            }
        }
        self.prepare_music_ready_handoff_bridge(item_id);
        if self.music.music_player_current_item_id != Some(item_id) {
            // Manual-seek grace belongs to the source playback session. If it
            // leaks into a newly selected track, that track can play normally
            // for the remaining grace period and then be re-cued from zero.
            self.music.music_manual_seek_grace_until = None;
            self.music.music_reserved_next_item_id = None;
            self.music.music_chorus_flow_segment = None;
            self.music.music_chorus_fade_out = None;
            self.music.music_chorus_fade_in = None;
            self.cancel_music_radio_cue_pending();
            self.music.music_smooth_seek = None;
            if self
                .music
                .music_chorus_pending_fade_in
                .as_ref()
                .is_some_and(|pending| pending.item_id != item_id)
            {
                self.music.music_chorus_pending_fade_in = None;
            }
            if self
                .music
                .music_chorus_pending_start
                .as_ref()
                .is_some_and(|pending| pending.item_id != item_id)
            {
                self.music.music_chorus_pending_start = None;
            }
            self.cancel_music_prefetch();
        }
        if self
            .music
            .music_chorus_handoff_bridge
            .as_ref()
            .is_some_and(|bridge| bridge.target_item_id != item_id)
        {
            if let Some(bridge) = self.music.music_chorus_handoff_bridge.take() {
                bridge.control.stop();
            }
        }
        let keep_previous_as_chorus_bridge = self
            .music
            .music_chorus_handoff_bridge
            .as_ref()
            .is_some_and(|bridge| {
                bridge.target_item_id == item_id
                    && self.music.music_playback.as_ref().is_some_and(|control| {
                        control.item_id == bridge.control.item_id
                            && control.session_id == bridge.control.session_id
                    })
            });
        let keep_previous_as_ready_handoff = self
            .music
            .music_playback_ready_handoff
            .as_ref()
            .is_some_and(|handoff| {
                handoff.target_item_id == item_id
                    && self.music.music_playback.as_ref().is_some_and(|control| {
                        control.item_id == handoff.control.item_id
                            && control.session_id == handoff.control.session_id
                    })
            });
        let keep_previous_playing =
            keep_previous_as_chorus_bridge || keep_previous_as_ready_handoff;
        if !keep_previous_playing {
            if let Some(previous_id) = self
                .music
                .music_player_current_item_id
                .filter(|id| *id != item_id)
            {
                if self
                    .queue_item_by_id(previous_id)
                    .is_some_and(|item| item.compact_music_state != Some(CompactMusicState::Failed))
                {
                    self.mark_music_playback_state(previous_id, CompactMusicState::Ready);
                }
            }
        }
        if !keep_previous_playing {
            if let Some(control) = self.music.music_playback.take() {
                control.stop();
                self.mark_music_playback_state(control.item_id, CompactMusicState::Ready);
            }
        }

        let Some(mut item) = self.queue_item_by_id(item_id).cloned() else {
            self.recover_music_ready_handoff_after_target_failure(item_id);
            return;
        };
        self.music.music_playback_session_id = session_id;
        if item.music_stream_url.trim().is_empty() {
            if let Some(hit) = self.complete_music_cache_hit_for_item(&item) {
                if let Some(target) = self.queue_item_mut_by_id(item_id) {
                    restore_music_compact_item_from_cache_hit(target, &hit);
                    item = target.clone();
                }
                eprintln!(
                    "[music-stream] restored complete cache for item={} key={}",
                    item_id, hit.cache_key
                );
            } else {
                self.resolve_music_item_for_playback_with_session(item_id, session_id);
                return;
            }
        }

        self.music.music_player_error = None;
        let cache_root = self.music_stream_cache_root();
        let cache_dir = cache_root.join(sanitize_music_cache_key(&item.music_cache_key));
        let cache_media_path = cache_dir.join(format!(
            "audio.{}",
            sanitize_music_cache_ext(&item.music_stream_ext)
        ));
        let cache_command = if self.complete_music_cache_media_path(&item).is_some() {
            None
        } else {
            match self.tool_paths.prepare_music_stream_cache_command(
                &item.source_url,
                &cache_media_path,
                &item.music_stream_format_id,
                item.selection.use_cookies,
            ) {
                Ok(command) => Some(command),
                Err(error) => {
                    let message = i18n::format_fixed_english(
                        "Music cache preparation failed: {error}",
                        &[("{error}", error.as_str())],
                    );
                    self.mark_music_playback_state(item_id, CompactMusicState::Failed);
                    if let Some(item) = self.queue_item_mut_by_id(item_id) {
                        item.last_error = Some(error.to_string());
                    }
                    self.music.music_player_error = Some(message.clone());
                    self.push_runtime_log(message.clone());
                    self.last_action = message;
                    self.recover_music_ready_handoff_after_target_failure(item_id);
                    return;
                }
            }
        };

        let lyrics_track = item
            .metadata()
            .and_then(primary_original_subtitle_track_from_metadata)
            .cloned();
        self.cache_music_lyrics_for_item(&item, lyrics_track.as_ref());

        let stream = ResolvedMusicStream {
            item_id,
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
            volume: self.music_chorus_initial_volume_for_item(item_id),
        };
        let control =
            music_stream::spawn_music_stream_playback(stream, self.music_playback_event_tx.clone());
        self.music.music_playback = Some(control);
        self.music.music_player_current_item_id = Some(item_id);
        self.mark_music_playback_state(item_id, CompactMusicState::Buffering);
        self.last_action = i18n::format_fixed_english(
            "Preparing playback: {title}",
            &[("{title}", item.title.as_str())],
        );
    }
}

fn music_ready_handoff_should_keep_source_playing(
    source_item_id: QueueItemId,
    target_item_id: QueueItemId,
    current_item_id: Option<QueueItemId>,
    source_is_paused: bool,
) -> bool {
    source_item_id != target_item_id && !source_is_paused && current_item_id == Some(source_item_id)
}

fn music_ready_handoff_source_has_remaining_audio(
    playback_seconds: f64,
    duration_seconds: Option<f64>,
) -> bool {
    duration_seconds.map_or(true, |duration| playback_seconds + 0.05 < duration)
}

fn music_ready_handoff_source_identity_matches(
    source_item_id: QueueItemId,
    source_session_id: u64,
    event_item_id: QueueItemId,
    event_session_id: u64,
) -> bool {
    source_item_id == event_item_id && source_session_id == event_session_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_handoff_keeps_active_source_for_a_different_target() {
        assert!(music_ready_handoff_should_keep_source_playing(
            1,
            2,
            Some(1),
            false,
        ));
        assert!(!music_ready_handoff_should_keep_source_playing(
            1,
            1,
            Some(1),
            false,
        ));
        assert!(!music_ready_handoff_should_keep_source_playing(
            1,
            2,
            Some(1),
            true,
        ));
    }

    #[test]
    fn ready_handoff_does_not_recover_a_source_that_reached_eof() {
        assert!(music_ready_handoff_source_has_remaining_audio(
            40.0,
            Some(60.0)
        ));
        assert!(!music_ready_handoff_source_has_remaining_audio(
            59.96,
            Some(60.0)
        ));
        assert!(music_ready_handoff_source_has_remaining_audio(40.0, None));
    }

    #[test]
    fn ready_handoff_source_events_require_item_and_session_identity() {
        assert!(music_ready_handoff_source_identity_matches(1, 10, 1, 10));
        assert!(!music_ready_handoff_source_identity_matches(1, 10, 2, 10));
        assert!(!music_ready_handoff_source_identity_matches(1, 10, 1, 11));
    }
}
