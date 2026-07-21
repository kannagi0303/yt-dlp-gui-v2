use super::*;

impl AppState {
    pub(super) fn current_music_lyrics_line_with_lead(&mut self) -> Option<String> {
        let control = self.music.music_playback.clone()?;
        let item = self.queue_item_by_id(control.item_id)?.clone();
        let cache_key = item.music_cache_key.trim();
        if cache_key.is_empty() {
            return None;
        }
        let seconds = if self.music.music_seek_drag_ratio.is_some()
            || self.music.music_seek_snap_ratio.is_some()
        {
            control
                .duration_seconds()
                .map(|duration| {
                    duration * f64::from(self.music_seek_display_ratio().clamp(0.0, 1.0))
                })
                .unwrap_or_else(|| control.playback_seconds())
        } else {
            control.playback_seconds()
        } + MUSIC_LYRICS_DISPLAY_LEAD_SECONDS;
        let path = music_lrc_cache_path(&self.music_stream_cache_root(), cache_key);
        if !path.is_file() {
            return None;
        }
        let lines = self.cached_music_lrc_lines(cache_key, path)?;
        current_lrc_line_text(&lines, seconds)
    }

    pub(super) fn update_music_lyrics_display_state(
        &mut self,
        current: Option<String>,
    ) -> Option<MusicLyricsDisplayLine> {
        let now = Instant::now();
        match current {
            Some(current) => {
                if self.music.music_lyrics_display_line.as_deref() != Some(current.as_str()) {
                    self.music.music_lyrics_previous_line =
                        self.music.music_lyrics_display_line.take();
                    self.music.music_lyrics_display_line = Some(current);
                    self.music.music_lyrics_transition_started_at = Some(now);
                }
            }
            None => {
                self.music.music_lyrics_display_line = None;
                self.music.music_lyrics_previous_line = None;
                self.music.music_lyrics_transition_started_at = None;
                return None;
            }
        }

        let fade = self
            .music
            .music_lyrics_transition_started_at
            .map(|started| {
                (now.duration_since(started).as_secs_f64() / MUSIC_LYRICS_FADE_SECONDS)
                    .clamp(0.0, 1.0) as f32
            })
            .unwrap_or(1.0);
        if fade >= 1.0 {
            self.music.music_lyrics_previous_line = None;
            self.music.music_lyrics_transition_started_at = None;
        }

        self.music
            .music_lyrics_display_line
            .as_ref()
            .map(|current| MusicLyricsDisplayLine {
                current: current.clone(),
                previous: self.music.music_lyrics_previous_line.clone(),
                fade,
            })
    }

    pub(super) fn cached_music_lrc_lines(
        &mut self,
        cache_key: &str,
        path: PathBuf,
    ) -> Option<Vec<LrcLine>> {
        let metadata = fs::metadata(&path).ok();
        let modified = metadata.as_ref().and_then(|meta| meta.modified().ok());
        let now = Instant::now();
        if metadata.is_none() {
            let entry = self
                .music
                .music_lyrics_cache
                .entry(cache_key.to_owned())
                .or_insert_with(|| CachedLrcTrack {
                    path: path.clone(),
                    modified: None,
                    lines: Vec::new(),
                    missing_checked_at: Some(now),
                });
            let recently_checked = entry
                .missing_checked_at
                .is_some_and(|checked| now.duration_since(checked) < Duration::from_secs(1));
            if !recently_checked {
                entry.path = path;
                entry.modified = None;
                entry.lines.clear();
                entry.missing_checked_at = Some(now);
            }
            return None;
        }

        let reload = self
            .music
            .music_lyrics_cache
            .get(cache_key)
            .map_or(true, |entry| {
                entry.path != path || entry.modified != modified || entry.lines.is_empty()
            });
        if reload {
            let lines = parse_lrc_file(&path).unwrap_or_default();
            self.music.music_lyrics_cache.insert(
                cache_key.to_owned(),
                CachedLrcTrack {
                    path,
                    modified,
                    lines,
                    missing_checked_at: None,
                },
            );
        }
        self.music
            .music_lyrics_cache
            .get(cache_key)
            .map(|entry| entry.lines.clone())
            .filter(|lines| !lines.is_empty())
    }

    pub fn has_music_compact_items(&self) -> bool {
        self.queue_display_mode == QueueDisplayMode::Audio && !self.queue_items.is_empty()
    }

    pub fn music_item_cache_progress_ratio(&self, item_id: QueueItemId) -> f32 {
        if let Some(control) = self
            .music
            .music_playback
            .as_ref()
            .filter(|control| control.item_id == item_id)
        {
            return control.cache_progress_ratio();
        }
        self.music_cached_progress_for_item(item_id)
    }

    pub fn music_item_cache_activity(
        &self,
        item_id: QueueItemId,
    ) -> Option<MusicItemCacheActivity> {
        let current_state = self
            .queue_item_by_id(item_id)
            .and_then(|item| item.compact_music_state);
        let playback = self
            .music
            .music_playback
            .as_ref()
            .map(|control| (control.item_id, control.cache_is_complete()));

        music_item_cache_activity_from_runtime(
            item_id,
            self.music.music_player_current_item_id,
            current_state,
            playback,
            self.music.music_prefetch_pending_item_id,
            self.music.music_prefetch_active_item_id,
        )
    }

    pub fn music_item_compact_progress_ratio(&self, item_id: QueueItemId) -> f32 {
        if let Some(progress) = self.music_item_active_download_progress_ratio(item_id) {
            return progress;
        }
        self.music_item_cache_progress_ratio(item_id)
    }

    pub fn music_item_compact_progress_visible(&self, item_id: QueueItemId) -> bool {
        if self
            .music_item_active_download_progress_ratio(item_id)
            .is_some()
        {
            return true;
        }
        let playback_cache_is_growing = self
            .music
            .music_playback
            .as_ref()
            .filter(|control| control.item_id == item_id)
            .is_some_and(|control| {
                let progress = control.cache_progress_ratio();
                !control.cache_is_complete() && progress > 0.0 && progress < 0.999
            });
        if playback_cache_is_growing {
            return true;
        }

        self.music.music_prefetch_active_item_id == Some(item_id) && {
            let progress = self.music_item_cache_progress_ratio(item_id);
            progress > 0.0 && progress < 0.999
        }
    }
}

fn music_item_cache_activity_from_runtime(
    item_id: QueueItemId,
    current_item_id: Option<QueueItemId>,
    current_state: Option<CompactMusicState>,
    playback: Option<(QueueItemId, bool)>,
    prefetch_pending_item_id: Option<QueueItemId>,
    prefetch_active_item_id: Option<QueueItemId>,
) -> Option<MusicItemCacheActivity> {
    if prefetch_pending_item_id == Some(item_id) {
        return Some(MusicItemCacheActivity::Preparing);
    }
    if prefetch_active_item_id == Some(item_id) {
        return Some(MusicItemCacheActivity::Caching);
    }
    if current_item_id != Some(item_id) {
        return None;
    }
    if current_state == Some(CompactMusicState::Resolving) {
        return Some(MusicItemCacheActivity::Preparing);
    }
    playback
        .filter(|(playback_item_id, cache_is_complete)| {
            *playback_item_id == item_id && !cache_is_complete
        })
        .map(|_| MusicItemCacheActivity::Caching)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_activity_tracks_current_resolve_before_playback_control_exists() {
        assert_eq!(
            music_item_cache_activity_from_runtime(
                7,
                Some(7),
                Some(CompactMusicState::Resolving),
                None,
                None,
                None,
            ),
            Some(MusicItemCacheActivity::Preparing)
        );
    }

    #[test]
    fn cache_activity_tracks_background_prefetch_without_changing_current_item() {
        assert_eq!(
            music_item_cache_activity_from_runtime(
                8,
                Some(7),
                Some(CompactMusicState::Ready),
                Some((7, false)),
                None,
                Some(8),
            ),
            Some(MusicItemCacheActivity::Caching)
        );
    }

    #[test]
    fn cache_activity_stops_when_current_playback_cache_is_complete() {
        assert_eq!(
            music_item_cache_activity_from_runtime(
                7,
                Some(7),
                Some(CompactMusicState::Playing),
                Some((7, true)),
                None,
                None,
            ),
            None
        );
    }

    #[test]
    fn cache_activity_ignores_stale_non_current_row_state() {
        assert_eq!(
            music_item_cache_activity_from_runtime(
                8,
                Some(7),
                Some(CompactMusicState::Resolving),
                Some((7, false)),
                None,
                None,
            ),
            None
        );
    }
}
