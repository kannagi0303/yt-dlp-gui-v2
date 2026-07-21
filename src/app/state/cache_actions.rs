use super::*;

impl AppState {
    pub fn set_post_download_conversion_enabled(&mut self, enabled: bool) {
        self.config.post_download_conversion_enabled = enabled;
        let _ = self.config.save();
    }

    pub fn set_enable_builtin_transcode_after_download(&mut self, enabled: bool) {
        self.set_post_download_conversion_enabled(enabled);
    }

    pub fn set_chapter_compatibility_mode(&mut self, enabled: bool) {
        self.tool_paths.chapter_compatibility_mode = enabled;
        self.config.chapter_compatibility_mode = enabled;
        let _ = self.config.save();
    }

    pub fn set_always_show_download_range(&mut self, enabled: bool) {
        self.config.always_show_download_range = enabled;
        let _ = self.config.save();
    }

    pub fn set_quality_preset(&mut self, preset: QualityPreset) {
        self.item_defaults.quality = preset;
        for item in &mut self.queue_items {
            item.selection.quality = preset;
        }
    }

    pub fn cache_location_display(&self) -> String {
        match self.tool_paths.cache_mode {
            CacheLocationMode::YtDlpDefault => "yt-dlp default".to_owned(),
            CacheLocationMode::V2Cache => {
                crate::infrastructure::resolve_output_dir(&self.tool_paths.cache_dir)
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|_| self.tool_paths.cache_dir.clone())
            }
            CacheLocationMode::WindowsTemp => std::env::temp_dir().display().to_string(),
        }
    }

    pub fn refresh_cache_management_summary_if_stale(&mut self) {
        if self
            .music
            .cache_management_summary_refreshed_at
            .is_some_and(|last| last.elapsed() < Duration::from_secs(2))
        {
            return;
        }
        self.refresh_cache_management_summary();
    }

    pub fn refresh_cache_management_summary(&mut self) {
        let root = self.app_cache_root_path();
        self.music.cache_management_summary = calculate_cache_management_summary(&root);
        self.music.cache_management_summary_refreshed_at = Some(Instant::now());
    }

    pub fn cache_management_usage_display(&self) -> String {
        let total = format_byte_size(self.music.cache_management_summary.total_bytes);
        let audio = format_byte_size(self.music.cache_management_summary.music_bytes);
        let expired = format_byte_size(self.music.cache_management_summary.expired_music_bytes);
        self.ui_i18n_text_with_replacements(
            "options.cache_usage_detail",
            &[
                ("{total}", total.as_str()),
                ("{audio}", audio.as_str()),
                ("{expired}", expired.as_str()),
            ],
        )
    }

    pub fn clear_expired_music_cache(&mut self) {
        let root = self.music_stream_cache_root();
        match remove_expired_music_cache_dirs(&root) {
            Ok(summary) => {
                self.refresh_cache_management_summary();
                let count = summary.entries.to_string();
                let size = format_byte_size(summary.bytes);
                self.last_action = i18n::format_fixed_english(
                    "Cleared {count} expired cache entries ({size}).",
                    &[("{count}", count.as_str()), ("{size}", size.as_str())],
                );
            }
            Err(error) => {
                let error = error.to_string();
                self.last_action = i18n::format_fixed_english(
                    "Cache cleanup failed: {error}",
                    &[("{error}", error.as_str())],
                );
            }
        }
    }

    pub fn clear_music_stream_cache(&mut self) {
        self.stop_music_playback();
        let root = self.music_stream_cache_root();
        match remove_path_contents_or_dir(&root) {
            Ok(summary) => {
                self.refresh_cache_management_summary();
                let count = summary.entries.to_string();
                let size = format_byte_size(summary.bytes);
                self.last_action = i18n::format_fixed_english(
                    "Cleared audio cache: {count} entries ({size}).",
                    &[("{count}", count.as_str()), ("{size}", size.as_str())],
                );
            }
            Err(error) => {
                let error = error.to_string();
                self.last_action = i18n::format_fixed_english(
                    "Cache cleanup failed: {error}",
                    &[("{error}", error.as_str())],
                );
            }
        }
    }

    pub fn clear_app_cache(&mut self) {
        self.stop_music_playback();
        let root = self.app_cache_root_path();
        match remove_safe_app_cache_contents(&root) {
            Ok(summary) => {
                self.refresh_cache_management_summary();
                let count = summary.entries.to_string();
                let size = format_byte_size(summary.bytes);
                self.last_action = i18n::format_fixed_english(
                    "Cleared app cache: {count} entries ({size}).",
                    &[("{count}", count.as_str()), ("{size}", size.as_str())],
                );
            }
            Err(error) => {
                let error = error.to_string();
                self.last_action = i18n::format_fixed_english(
                    "Cache cleanup failed: {error}",
                    &[("{error}", error.as_str())],
                );
            }
        }
    }

    pub(super) fn app_cache_root_path(&self) -> PathBuf {
        crate::infrastructure::resolve_output_dir(&self.tool_paths.cache_dir)
            .unwrap_or_else(|_| PathBuf::from(&self.tool_paths.cache_dir))
    }
}
