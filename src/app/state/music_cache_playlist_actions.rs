use super::*;

impl AppState {
    pub(super) fn music_playable_item_ids(&self) -> Vec<QueueItemId> {
        self.queue_items
            .iter()
            .filter(|item| self.music_item_can_play(item.id))
            .map(|item| item.id)
            .collect()
    }

    pub(super) fn complete_music_cache_hit_for_item(
        &self,
        item: &QueueItem,
    ) -> Option<CompleteMusicCacheHit> {
        let source_key = canonical_queue_source_key(&item.source_url);
        let mut best: Option<(u64, CompleteMusicCacheHit)> = None;
        let root = self.music_stream_cache_root();
        let entries = fs::read_dir(&root).ok()?;
        for entry in entries.filter_map(Result::ok) {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let manifest_path = dir.join("manifest.yaml");
            let Some(manifest) = read_yaml_file::<AudioCacheManifestSnapshot>(&manifest_path)
            else {
                continue;
            };
            if !audio_cache_manifest_is_fresh(&manifest) {
                let _ = fs::remove_dir_all(&dir);
                continue;
            }
            if !manifest.complete {
                continue;
            }
            let manifest_source = manifest.source_url.trim().to_owned();
            if manifest_source.is_empty() {
                continue;
            };
            if canonical_queue_source_key(&manifest_source) != source_key {
                continue;
            }
            let ext = manifest.ext.trim().to_owned();
            if ext.trim().is_empty() {
                continue;
            }
            let media_path = dir.join(format!("audio.{}", sanitize_music_cache_ext(&ext)));
            let media_len = fs::metadata(&media_path)
                .map(|meta| meta.len())
                .unwrap_or(0);
            if media_len == 0 {
                continue;
            }
            let expected_bytes = manifest.expected_bytes;
            if expected_bytes.is_some_and(|expected| expected > media_len) {
                continue;
            }
            let cache_key = dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_owned();
            if cache_key.trim().is_empty() {
                continue;
            }
            let updated = manifest.updated_unix_seconds;
            let hit = CompleteMusicCacheHit {
                cache_key,
                source_url: manifest_source,
                title: manifest.title,
                album_title: manifest.album_title,
                thumbnail_url: manifest.thumbnail_url,
                duration_seconds: manifest.duration_seconds,
                ext,
                format_id: manifest.format_id,
                acodec: manifest.acodec,
                expected_bytes,
            };
            let replace_best = match best.as_ref() {
                Some((best_updated, _)) => updated >= *best_updated,
                None => true,
            };
            if replace_best {
                best = Some((updated, hit));
            }
        }
        best.map(|(_, hit)| hit)
    }

    pub(super) fn prepare_queue_items_for_audio_mode(&mut self) {
        let ids = self
            .queue_items
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>();
        for item_id in ids {
            self.prepare_queue_item_for_audio_mode(item_id);
        }
    }

    pub(super) fn prepare_queue_item_for_audio_mode(&mut self, item_id: QueueItemId) {
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            if item.compact_music_state.is_none() {
                item.compact_music_state = Some(CompactMusicState::Ready);
            }
            if item.music_cache_key.trim().is_empty() && !item.source_url.trim().is_empty() {
                item.music_cache_key = music_cache_key(&item.source_url, "flat", "", "");
            }
            if item.music_duration_seconds.is_none() && !item.duration_text.trim().is_empty() {
                item.music_duration_seconds = duration_text_to_seconds(&item.duration_text);
            }
        }

        self.restore_music_compact_cache_hit_if_available(item_id);
        if let Some(item) = self.queue_item_by_id(item_id) {
            self.cache_music_cover_for_item(item);
        }
    }

    pub(super) fn restore_music_compact_cache_hit_if_available(
        &mut self,
        item_id: QueueItemId,
    ) -> bool {
        let hit = {
            self.queue_item_by_id(item_id)
                .and_then(|item| self.complete_music_cache_hit_for_item(item))
        };
        let Some(hit) = hit else {
            return false;
        };
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            restore_music_compact_item_from_cache_hit(item, &hit);
            self.mark_font_content_changed();
            return true;
        }
        false
    }

    pub fn music_item_has_complete_cache(&self, item_id: QueueItemId) -> bool {
        let Some(item) = self.queue_item_by_id(item_id) else {
            return false;
        };
        self.complete_music_cache_media_path(item).is_some()
    }

    pub(super) fn music_cached_progress_for_item(&self, item_id: QueueItemId) -> f32 {
        let Some(item) = self.queue_item_by_id(item_id) else {
            return 0.0;
        };
        music_cached_progress_for_item_in_root(item, &self.music_stream_cache_root())
    }

    pub(super) fn music_stream_cache_root(&self) -> PathBuf {
        self.audio_cache_root_path()
    }

    pub(super) fn audio_cache_root_path(&self) -> PathBuf {
        self.app_cache_root_path().join("audio")
    }

    pub(super) fn audio_playlist_snapshot_path(&self) -> PathBuf {
        self.app_cache_root_path().join("audio-playlist.yaml")
    }

    pub(super) fn restore_saved_audio_playlist(&mut self) {
        let items = self.load_audio_playlist_snapshot_items();
        if self.queue_display_mode == QueueDisplayMode::Audio {
            self.queue_items = items;
            self.rebuild_batch_input_from_queue();
        } else {
            self.music.audio_queue_items = items;
        }
    }

    pub(super) fn load_audio_playlist_snapshot_items(&mut self) -> Vec<QueueItem> {
        let path = self.audio_playlist_snapshot_path();
        let Some(snapshot) = read_yaml_file::<AudioPlaylistSnapshot>(&path) else {
            return Vec::new();
        };
        snapshot
            .items
            .into_iter()
            .filter_map(|entry| self.queue_item_from_audio_playlist_snapshot(entry))
            .collect()
    }

    pub(super) fn queue_item_from_audio_playlist_snapshot(
        &mut self,
        entry: AudioPlaylistItemSnapshot,
    ) -> Option<QueueItem> {
        let source = entry.source_url.trim().to_owned();
        if source.is_empty() {
            return None;
        }
        let mut item = self.build_queue_item_from_url(&source);
        item.view_kind = QueueItemViewKind::MusicCompact;
        item.compact_music_state = Some(CompactMusicState::Ready);
        item.metadata_state = MetadataState::Idle;
        if !entry.title.trim().is_empty() {
            item.title = entry.title;
        }
        item.music_album_title = entry.album_title;
        item.thumbnail_hint = if entry.thumbnail_hint.trim().is_empty() {
            "item.thumbnail".to_owned()
        } else {
            entry.thumbnail_hint
        };
        item.thumbnail_url = entry.thumbnail_url;
        item.duration_text = entry.duration_text;
        item.music_duration_seconds = entry
            .duration_seconds
            .or_else(|| duration_text_to_seconds(&item.duration_text));
        item.music_stream_url.clear();
        item.music_stream_headers.clear();
        item.music_stream_ext = entry.stream_ext;
        item.music_stream_format_id = entry.stream_format_id;
        item.music_stream_acodec = entry.stream_acodec;
        item.music_stream_expected_bytes = entry.expected_bytes;
        item.music_cache_key = if entry.cache_key.trim().is_empty() {
            music_cache_key(&item.source_url, "flat", "", "")
        } else {
            entry.cache_key
        };
        item.selection.use_cookies = entry.use_cookies;
        item.last_error = None;
        self.restore_music_compact_cache_hit_for_item(&mut item);
        Some(item)
    }

    pub(super) fn save_active_audio_playlist_if_needed(&self) {
        if self.queue_display_mode == QueueDisplayMode::Audio {
            self.save_audio_playlist_items(&self.queue_items);
        } else {
            self.save_audio_playlist_items(&self.music.audio_queue_items);
        }
    }

    pub(super) fn save_audio_playlist_items(&self, items: &[QueueItem]) {
        let snapshot = AudioPlaylistSnapshot {
            version: 1,
            items: items
                .iter()
                .filter(|item| item.view_kind == QueueItemViewKind::MusicCompact)
                .filter(|item| !item.source_url.trim().is_empty())
                .map(audio_playlist_item_snapshot)
                .collect(),
        };
        let path = self.audio_playlist_snapshot_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(error) = write_yaml_file(&path, &snapshot) {
            eprintln!("[music-playlist] save skipped: {error}");
        }
    }

    pub(super) fn restore_music_compact_cache_hit_for_item(&self, item: &mut QueueItem) {
        if let Some(hit) = self.complete_music_cache_hit_for_item(item) {
            restore_music_compact_item_from_cache_hit(item, &hit);
        }
    }
}
