use super::*;

fn music_navigation_uses_mix(
    mix_mode: MusicMixMode,
    current_item_id: Option<QueueItemId>,
    target_item_id: QueueItemId,
    playback_is_running: bool,
) -> bool {
    mix_mode.enabled()
        && playback_is_running
        && current_item_id.is_some()
        && current_item_id != Some(target_item_id)
}

impl AppState {
    pub(super) fn next_music_prefetch_session_id(&mut self) -> u64 {
        self.music.music_prefetch_session_id =
            self.music.music_prefetch_session_id.wrapping_add(1).max(1);
        self.music.music_prefetch_session_id
    }

    pub(super) fn music_item_can_play(&self, item_id: QueueItemId) -> bool {
        self.queue_display_mode == QueueDisplayMode::Audio
            && self
                .queue_item_by_id(item_id)
                .is_some_and(|item| !item.source_url.trim().is_empty())
    }

    pub(super) fn mark_music_playback_state(
        &mut self,
        item_id: QueueItemId,
        music_state: CompactMusicState,
    ) {
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            item.compact_music_state = Some(music_state);
        }
    }

    pub fn music_current_item_id(&self) -> Option<QueueItemId> {
        self.music.music_player_current_item_id
    }

    pub fn music_item_is_playing(&self, item_id: QueueItemId) -> bool {
        self.music.music_player_current_item_id == Some(item_id) && self.music_player_is_playing()
    }

    pub fn music_item_navigation_uses_mix(&self, item_id: QueueItemId) -> bool {
        self.music_item_can_play(item_id)
            && music_navigation_uses_mix(
                self.music_mix_mode(),
                self.music.music_player_current_item_id,
                item_id,
                self.music_player_is_playing(),
            )
    }

    pub fn play_music_item(&mut self, item_id: QueueItemId) {
        if self.music.music_player_current_item_id == Some(item_id) {
            self.cancel_music_radio_cue_pending_with_message(
                "Mix next cancelled; staying on the current track.",
            );
            if self.music.music_playback.is_some() {
                self.toggle_music_playback();
            } else if self.music_item_can_play(item_id) {
                // Resolving/Buffering are transient UI states, not playback
                // authority. A missing control means the user may explicitly
                // recover a stale or failed session by starting a fresh one.
                self.start_music_stream_playback_recorded(item_id, false);
            } else {
                self.last_action = "This music item has no playable source URL.".to_owned();
            }
            return;
        }
        if self.request_or_cancel_music_navigation_mix(item_id, true) {
            return;
        }
        if self.music_item_can_play(item_id) {
            self.start_music_stream_playback(item_id);
        } else if let Some(item) = self.queue_item_by_id(item_id) {
            let message = match item.compact_music_state {
                Some(CompactMusicState::Resolving) => "Music stream is still resolving.".to_owned(),
                Some(CompactMusicState::Buffering) => "Music is buffering.".to_owned(),
                Some(CompactMusicState::Failed) => item
                    .last_error
                    .clone()
                    .unwrap_or_else(|| "This music item cannot be played right now.".to_owned()),
                _ => "Music stream is not ready yet.".to_owned(),
            };
            self.last_action = message;
        }
    }

    pub fn previous_music_item(&mut self) {
        let Some(item_id) = self.previous_music_item_id() else {
            self.last_action = "No previous track.".to_owned();
            return;
        };
        if let Some(current) = self
            .music
            .music_player_current_item_id
            .filter(|id| *id != item_id)
        {
            self.music.music_history_forward.push(current);
        }
        self.request_music_scroll_to_item(item_id);
        if !self.request_or_cancel_music_navigation_mix(item_id, false) {
            self.start_music_stream_playback_recorded(item_id, false);
        }
    }

    pub fn next_music_item(&mut self) {
        let Some(item_id) = self.next_music_item_id(false) else {
            self.last_action = "No next track.".to_owned();
            return;
        };
        self.request_music_scroll_to_item(item_id);
        if !self.request_or_cancel_music_navigation_mix(item_id, true) {
            self.start_music_stream_playback_recorded(item_id, true);
        }
    }

    pub(super) fn request_music_scroll_to_item(&mut self, item_id: QueueItemId) {
        self.music.music_scroll_to_item_id = Some(item_id);
    }

    pub fn take_music_scroll_to_item_request(&mut self, item_id: QueueItemId) -> bool {
        if self.music.music_scroll_to_item_id == Some(item_id) {
            self.music.music_scroll_to_item_id = None;
            return true;
        }
        false
    }

    fn request_or_cancel_music_navigation_mix(
        &mut self,
        item_id: QueueItemId,
        record_history: bool,
    ) -> bool {
        if !self.music_item_navigation_uses_mix(item_id) {
            return false;
        }
        if self.music_mix_next_pending_item_indicator(item_id) {
            self.cancel_music_radio_cue_pending_with_message("Mix next cancelled.");
            return true;
        }
        if !self.request_music_mix_next_to_item(item_id, record_history) {
            self.last_action = "This track cannot be prepared as Mix next.".to_owned();
        }
        true
    }

    pub fn set_music_playback_mode(&mut self, mode: MusicPlaybackMode) {
        if self.music.music_playback_mode == mode {
            return;
        }
        self.cancel_music_radio_cue_pending_and_reanchor();
        self.music.music_playback_mode = mode;
        self.config.music_playback_mode = self.music.music_playback_mode.config_value().to_owned();
        let _ = self.config.save();
        let mode_label = self
            .ui_i18n_text_for_key(self.music.music_playback_mode.label_key())
            .to_owned();
        self.last_action =
            i18n::format_fixed_english("Playback mode: {mode}", &[("{mode}", mode_label.as_str())]);
    }

    pub fn music_playback_mode_text(&self) -> &'static str {
        self.ui_i18n_text_for_key(self.music.music_playback_mode.label_key())
    }

    pub fn music_playback_mode_kind(&self) -> MusicPlaybackMode {
        self.music.music_playback_mode
    }

    pub(super) fn advance_music_after_finished(&mut self, finished_item_id: QueueItemId) {
        // EOF closes the source session even for RepeatOne. A seek guard from
        // that finished session must never defer initialization of the next
        // playback session.
        self.music.music_manual_seek_grace_until = None;
        let next = match self.music.music_playback_mode {
            MusicPlaybackMode::RepeatOne => Some(finished_item_id),
            MusicPlaybackMode::Sequential => self.next_music_item_id(false),
            MusicPlaybackMode::RepeatAll | MusicPlaybackMode::Shuffle => {
                self.next_music_item_id(true)
            }
        };
        if let Some(item_id) = next {
            self.start_music_stream_playback(item_id);
        }
    }

    pub(super) fn previous_music_item_id(&mut self) -> Option<QueueItemId> {
        while let Some(item_id) = self.music.music_history_back.pop() {
            if self.music_item_can_play(item_id) {
                return Some(item_id);
            }
        }

        if self.music.music_playback_mode == MusicPlaybackMode::Shuffle {
            return None;
        }

        let items = self.music_playable_item_ids();
        if items.is_empty() {
            return None;
        }
        let current = self.music.music_player_current_item_id?;
        let index = items.iter().position(|id| *id == current)?;
        if index > 0 {
            items.get(index - 1).copied()
        } else {
            items
                .last()
                .copied()
                .filter(|_| self.music.music_playback_mode == MusicPlaybackMode::RepeatAll)
        }
    }

    pub(super) fn next_music_item_id(&mut self, allow_wrap: bool) -> Option<QueueItemId> {
        if self.music.music_playback_mode == MusicPlaybackMode::Shuffle {
            while let Some(item_id) = self.music.music_history_forward.pop() {
                if self.music_item_can_play(item_id) {
                    return Some(item_id);
                }
            }
            if let Some(item_id) = self
                .music
                .music_reserved_next_item_id
                .take()
                .filter(|id| self.music_item_can_play(*id))
            {
                return Some(item_id);
            }
            return self.random_music_next_item_id(allow_wrap);
        }
        self.ordered_next_music_item_id(allow_wrap)
    }

    pub(super) fn peek_next_music_item_id_for_prefetch(
        &mut self,
        allow_wrap: bool,
    ) -> Option<QueueItemId> {
        if self.music.music_playback_mode == MusicPlaybackMode::Shuffle {
            if let Some(item_id) = self
                .music
                .music_reserved_next_item_id
                .filter(|id| self.music_item_can_play(*id))
            {
                return Some(item_id);
            }
            let item_id = self.random_music_next_item_id(allow_wrap)?;
            self.music.music_reserved_next_item_id = Some(item_id);
            return Some(item_id);
        }
        self.ordered_next_music_item_id(allow_wrap)
    }

    pub(super) fn ordered_next_music_item_id(&self, allow_wrap: bool) -> Option<QueueItemId> {
        let items = self.music_playable_item_ids();
        if items.is_empty() {
            return None;
        }
        let Some(current) = self.music.music_player_current_item_id else {
            return items.first().copied();
        };
        let Some(index) = items.iter().position(|id| *id == current) else {
            return items.first().copied();
        };
        if let Some(next) = items.get(index + 1).copied() {
            return Some(next);
        }
        if allow_wrap || self.music.music_playback_mode == MusicPlaybackMode::RepeatAll {
            items.first().copied()
        } else {
            None
        }
    }

    pub(super) fn random_music_next_item_id(&self, allow_wrap: bool) -> Option<QueueItemId> {
        let items = self.music_playable_item_ids();
        if items.is_empty() {
            return None;
        }
        let current = self.music.music_player_current_item_id;
        let candidates = items
            .iter()
            .copied()
            .filter(|id| Some(*id) != current)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return allow_wrap.then(|| items[0]);
        }
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as usize)
            .unwrap_or(0);
        candidates.get(seed % candidates.len()).copied()
    }

    pub(super) fn record_music_navigation_target(&mut self, item_id: QueueItemId) {
        let Some(current) = self.music.music_player_current_item_id else {
            return;
        };
        if current == item_id {
            return;
        }
        if self.music.music_history_back.last().copied() != Some(current) {
            self.music.music_history_back.push(current);
        }
        if self.music.music_history_back.len() > MUSIC_PLAY_HISTORY_LIMIT {
            let excess = self.music.music_history_back.len() - MUSIC_PLAY_HISTORY_LIMIT;
            self.music.music_history_back.drain(0..excess);
        }
        self.music.music_history_forward.clear();
    }

    pub(super) fn prune_music_navigation_state(&mut self) {
        let playable = self.music_playable_item_ids();
        self.music
            .music_history_back
            .retain(|id| playable.contains(id));
        self.music
            .music_history_forward
            .retain(|id| playable.contains(id));
        if self
            .music
            .music_reserved_next_item_id
            .is_some_and(|id| !playable.contains(&id))
        {
            self.music.music_reserved_next_item_id = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_navigation_requires_an_active_other_track() {
        assert!(music_navigation_uses_mix(
            MusicMixMode::FullSong,
            Some(1),
            2,
            true
        ));
        assert!(!music_navigation_uses_mix(
            MusicMixMode::Off,
            Some(1),
            2,
            true
        ));
        assert!(!music_navigation_uses_mix(
            MusicMixMode::Highlight,
            Some(1),
            1,
            true
        ));
        assert!(!music_navigation_uses_mix(
            MusicMixMode::Highlight,
            Some(1),
            2,
            false
        ));
        assert!(!music_navigation_uses_mix(
            MusicMixMode::Highlight,
            None,
            2,
            true
        ));
    }
}
