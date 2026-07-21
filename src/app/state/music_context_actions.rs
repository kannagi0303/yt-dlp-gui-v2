use super::*;

impl AppState {
    pub fn queue_display_mode_is_audio(&self) -> bool {
        self.queue_display_mode == QueueDisplayMode::Audio
    }

    pub(super) fn enter_audio_queue_context(&mut self) {
        if self.queue_display_mode == QueueDisplayMode::Audio {
            return;
        }
        self.music.non_audio_queue_items = std::mem::take(&mut self.queue_items);
        self.queue_items = std::mem::take(&mut self.music.audio_queue_items);
        for item in &mut self.queue_items {
            item.view_kind = QueueItemViewKind::MusicCompact;
            item.compact_music_state
                .get_or_insert(CompactMusicState::Ready);
        }
        self.prune_music_navigation_state();
        self.rebuild_batch_input_from_queue();
    }

    pub(super) fn leave_audio_queue_context(&mut self) {
        if self.queue_display_mode != QueueDisplayMode::Audio {
            return;
        }
        self.stop_music_playback_for_audio_context();
        self.music.audio_queue_items = std::mem::take(&mut self.queue_items);
        self.save_audio_playlist_items(&self.music.audio_queue_items);
        self.queue_items = std::mem::take(&mut self.music.non_audio_queue_items);
        for item in &mut self.queue_items {
            item.view_kind = QueueItemViewKind::VideoCard;
            item.compact_music_state = None;
        }
        self.rebuild_batch_input_from_queue();
    }

    pub(super) fn stop_music_playback_for_audio_context(&mut self) {
        self.stop_music_playback();
        self.music.music_reserved_next_item_id = None;
        self.music.music_scroll_to_item_id = None;
        self.music.music_history_back.clear();
        self.music.music_history_forward.clear();
    }

    pub fn music_player_visible(&self) -> bool {
        self.queue_display_mode == QueueDisplayMode::Audio && !self.queue_items.is_empty()
    }

    pub(super) fn poll_media_session_commands(&mut self) {
        while let Some(command) = self.music.media_session.poll_command() {
            if self.queue_display_mode != QueueDisplayMode::Audio {
                continue;
            }
            match command {
                MediaSessionCommand::Play => {
                    if !self.music_player_is_playing() {
                        self.toggle_music_playback();
                    }
                }
                MediaSessionCommand::Pause => {
                    if self.music_player_is_playing() {
                        self.toggle_music_playback();
                    }
                }
                MediaSessionCommand::Previous => self.previous_music_item(),
                MediaSessionCommand::Next => self.next_music_item(),
                MediaSessionCommand::Stop => self.stop_music_playback(),
            }
        }
    }

    pub(super) fn sync_media_session(&mut self) {
        if self.queue_display_mode != QueueDisplayMode::Audio {
            self.music.media_session.clear();
            return;
        }

        let Some(item_id) = self.music.music_player_current_item_id else {
            self.music.media_session.clear();
            return;
        };
        let Some(item) = self.queue_item_by_id(item_id) else {
            self.music.media_session.clear();
            return;
        };

        let duration_seconds = self
            .music
            .music_playback
            .as_ref()
            .and_then(MusicPlaybackControl::duration_seconds)
            .or(item.music_duration_seconds)
            .or_else(|| duration_text_to_seconds(&item.duration_text));
        let position_seconds = self
            .music
            .music_playback
            .as_ref()
            .map(MusicPlaybackControl::playback_seconds)
            .unwrap_or(0.0);
        let status = match item.compact_music_state.unwrap_or(CompactMusicState::Ready) {
            CompactMusicState::Resolving | CompactMusicState::Buffering => {
                MediaSessionPlaybackStatus::Changing
            }
            CompactMusicState::Playing => MediaSessionPlaybackStatus::Playing,
            CompactMusicState::Paused => MediaSessionPlaybackStatus::Paused,
            CompactMusicState::Failed => MediaSessionPlaybackStatus::Stopped,
            CompactMusicState::Ready => {
                if self.music.music_playback.is_some() {
                    if self.music_player_is_playing() {
                        MediaSessionPlaybackStatus::Playing
                    } else {
                        MediaSessionPlaybackStatus::Paused
                    }
                } else {
                    MediaSessionPlaybackStatus::Paused
                }
            }
        };

        let display_title = stable_media_session_title(&item.title, &item.source_url);
        let (artist, title) = split_artist_title_for_media_session(&display_title);
        let track = MediaSessionTrack {
            key: format!("{}:{}", item.id, item.source_url),
            title,
            artist,
            thumbnail_url: item.thumbnail_url.clone(),
            duration_seconds,
        };
        let timeline = MediaSessionTimeline {
            position_seconds,
            duration_seconds: track.duration_seconds,
        };
        self.music.media_session.update(&track, status, timeline);
    }

    pub fn queue_mode_downloads_as_audio(&self) -> bool {
        self.queue_display_mode == QueueDisplayMode::Audio
    }

    pub fn music_download_prompt_open(&self) -> bool {
        self.music.music_download_prompt_open
    }

    pub fn request_main_download(&mut self) {
        if self.queue_mode_downloads_as_audio() {
            self.prepare_queue_items_for_audio_mode();
            self.music.music_download_prompt_choice.mode = MusicDownloadMode::Original;
            self.music.music_download_prompt_choice.embed_cover = true;
            self.music.music_download_prompt_choice.write_tags = true;
            self.music.music_download_prompt_open = true;
        } else {
            self.start_single_download();
        }
    }

    pub fn music_download_prompt_choice(&self) -> MusicDownloadChoice {
        self.music.music_download_prompt_choice
    }

    pub fn set_music_download_prompt_mode(&mut self, mode: MusicDownloadMode) {
        self.music.music_download_prompt_choice.mode = mode;
    }

    pub fn set_music_download_original_preference(&mut self, preference: MusicOriginalPreference) {
        self.music.music_download_prompt_choice.original_preference = preference;
    }

    pub fn set_music_download_unified_format(&mut self, format: MusicDownloadFormat) {
        self.music.music_download_prompt_choice.unified_format = format;
    }

    pub fn set_music_download_embed_cover(&mut self, enabled: bool) {
        self.music.music_download_prompt_choice.embed_cover = enabled;
    }

    pub fn set_music_download_write_tags(&mut self, enabled: bool) {
        self.music.music_download_prompt_choice.write_tags = enabled;
    }

    pub fn cancel_music_download_prompt(&mut self) {
        self.music.music_download_prompt_open = false;
    }

    pub fn confirm_music_download_choice(&mut self) {
        let mut choice = self.music.music_download_prompt_choice;
        choice.mode = MusicDownloadMode::Original;
        choice.embed_cover = true;
        choice.write_tags = true;
        self.music.music_download_prompt_choice = choice;
        self.music.music_download_prompt_open = false;
        self.start_download_with_music_choice(choice);
    }
}
