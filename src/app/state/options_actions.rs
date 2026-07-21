use super::*;

impl AppState {
    pub fn available_quality_presets(&self) -> [QualityPreset; 4] {
        [
            QualityPreset::Best,
            QualityPreset::P1080,
            QualityPreset::P720,
            QualityPreset::AudioOnly,
        ]
    }
    pub fn resolved_output_dir_display(&self) -> String {
        if self.output_dir_locked_by_config() {
            return "Controlled by config".to_owned();
        }
        let path = self.item_defaults.output_dir.as_str();
        resolve_output_dir(path)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| path.to_owned())
    }
    pub fn output_dir_display(&self) -> String {
        if self.output_dir_locked_by_config() {
            return "Controlled by config".to_owned();
        }
        let path = self.item_defaults.output_dir.as_str();
        display_output_dir(path)
    }
    pub fn language(&self) -> Language {
        self.config.language.resolve()
    }
    pub fn language_selection(&self) -> LanguageSelection {
        self.config.language
    }
    pub fn font_content_revision(&self) -> u64 {
        self.font_content_revision
    }
    pub(super) fn mark_font_content_changed(&mut self) {
        self.font_content_revision = self.font_content_revision.wrapping_add(1);
    }
    pub fn language_selection_display_name(&self) -> String {
        match self.language_selection() {
            LanguageSelection::Auto => format!(
                "{} ({})",
                LanguageSelection::Auto.native_name(),
                self.language().native_name()
            ),
            language => language.native_name().to_owned(),
        }
    }
    pub fn ui_i18n_text_for_key(&self, key: &'static str) -> &'static str {
        i18n::text(self.language(), key)
    }
    pub fn ui_i18n_text_with_replacements(
        &self,
        key: &'static str,
        replacements: &[(&str, &str)],
    ) -> String {
        i18n::format_text(self.language(), key, replacements)
    }
    pub fn localize_message(&self, value: &str) -> String {
        i18n::localize_message(self.language(), value)
    }
    pub fn set_language_selection(&mut self, language: LanguageSelection) {
        if self.config.language == language {
            return;
        }
        self.config.language = language;
        let _ = self.config.save();
    }
    pub fn open_options_detail_page(&mut self, page: OptionsDetailPage) {
        self.options_detail_page = Some(page);
    }
    pub fn close_options_detail_page(&mut self) {
        self.options_detail_page = None;
    }
    pub fn open_prepare_detail_page(&mut self, page: PrepareDetailPage) {
        self.prepare_detail_page = Some(page);
    }
    pub fn close_prepare_detail_page(&mut self) {
        self.prepare_detail_page = None;
    }
    pub fn open_advance_detail_page(&mut self, page: AdvanceDetailPage) {
        self.advance_detail_page = Some(page);
    }
    pub fn close_advance_detail_page(&mut self) {
        self.advance_detail_page = None;
    }
    pub fn set_last_action_message(&mut self, message: impl Into<String>) {
        self.last_action = message.into();
    }
    pub fn set_windows_toast_enabled(&mut self, enabled: bool) {
        self.config.windows_toast_enabled = enabled;
        let _ = self.config.save();
    }
    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        if self.config.theme_mode == mode {
            return;
        }
        self.config.theme_mode = mode;
        let _ = self.config.save();
    }
    pub fn set_theme_accent_color(&mut self, color: ThemeAccentColor) {
        if self.config.theme_accent_color == color {
            return;
        }
        self.config.theme_accent_color = color;
        let _ = self.config.save();
    }
    pub fn set_show_log_tab(&mut self, enabled: bool) {
        self.config.show_log_tab = enabled;
        if !enabled && self.active_tab == AppTab::Log {
            self.active_tab = AppTab::Options;
        }
        let _ = self.config.save();
    }
    pub fn set_transcode_intent(
        &mut self,
        settings: crate::infrastructure::TranscodeIntentSettings,
    ) {
        let settings = settings.normalized();
        if self.config.transcode_intent == settings {
            return;
        }
        self.config.transcode_intent = settings;
        let _ = self.config.save();
    }
    pub(super) fn send_download_result_windows_toast(
        &self,
        title: String,
        result: Result<String, String>,
    ) {
        if !self.config.windows_toast_enabled {
            return;
        }
        let language = self.language();

        thread::spawn(move || {
            let result = match result {
                Ok(output_path) => send_download_finished_windows_toast(
                    language,
                    title.as_str(),
                    (!output_path.trim().is_empty()).then_some(output_path.as_str()),
                ),
                Err(error) => {
                    if error == DOWNLOAD_CANCELLED_MESSAGE {
                        Ok(())
                    } else {
                        send_download_failed_windows_toast(language, title.as_str(), error.as_str())
                    }
                }
            };

            if let Err(error) = result {
                eprintln!("[notification] Windows Toast failed: {error}");
            }
        });
    }
    pub fn set_output_dir(&mut self, path: impl Into<String>) {
        if self.output_dir_locked_by_config() {
            return;
        }
        let path = path.into();
        self.item_defaults.output_dir = path.clone();
        for item in &mut self.queue_items {
            item.selection.output_dir = path.clone();
        }
        self.config.set_download_dir(path);
        let _ = self.config.save();
        self.refresh_prepare_report();
    }
    pub fn output_dir_locked_by_config(&self) -> bool {
        self.tool_paths.effective_config_owns_output()
    }
    pub fn output_dir_config_source_display(&self) -> Option<String> {
        self.tool_paths
            .effective_config_path()
            .map(|path| path.display().to_string())
    }
    pub fn set_proxy_enabled(&mut self, enabled: bool) {
        self.config.proxy_enabled = enabled;
        self.tool_paths.proxy_enabled = enabled;
        let _ = self.config.save();
    }
    pub fn set_proxy_url(&mut self, value: impl Into<String>) {
        self.config.set_proxy_url(value);
        self.tool_paths.proxy_url = self.config.proxy_url.clone();
        self.tool_paths.proxy_enabled = self.config.proxy_enabled;
        let _ = self.config.save();
    }
    pub fn set_no_check_certificates(&mut self, enabled: bool) {
        self.config.no_check_certificates = enabled;
        self.tool_paths.no_check_certificates = enabled;
        let _ = self.config.save();
    }
    pub fn set_limit_rate(&mut self, value: impl Into<String>) {
        self.config.set_limit_rate(value);
        self.tool_paths.limit_rate = self.config.limit_rate.clone();
        let _ = self.config.save();
    }

    pub fn set_live_from_start(&mut self, enabled: bool) {
        self.config.live_from_start = enabled;
        self.tool_paths.live_from_start = enabled;
        let _ = self.config.save();
    }
    pub fn set_download_sections(&mut self, value: impl Into<String>) {
        self.config.set_download_sections(value);
        self.tool_paths.download_sections = self.config.download_sections.clone();
        let _ = self.config.save();
    }
    pub fn set_file_time_mode(&mut self, mode: FileTimeMode) {
        self.config.file_time_mode = mode;
        self.tool_paths.file_time_mode = mode;
        let _ = self.config.save();
    }
    pub fn set_auto_analyze(&mut self, enabled: bool) {
        self.config.auto_analyze = enabled;
        let _ = self.config.save();
    }
    pub fn set_keep_window_on_top(&mut self, enabled: bool) {
        self.config.keep_window_on_top = enabled;
        let _ = self.config.save();
    }
    pub fn pending_ui_scale_percent(&self) -> u16 {
        self.pending_ui_scale_percent
    }
    pub fn set_pending_ui_scale_percent(&mut self, value: u16) {
        self.pending_ui_scale_percent = normalize_ui_scale_percent(value);
    }
    pub fn ui_scale_has_pending_change(&self) -> bool {
        self.pending_ui_scale_percent != self.config.ui_scale_percent
    }
    pub fn apply_pending_ui_scale_percent(&mut self) {
        self.config.ui_scale_percent = self.pending_ui_scale_percent;
        let _ = self.config.save();
    }
    pub fn set_ui_scale_percent(&mut self, value: u16) {
        let normalized = normalize_ui_scale_percent(value);
        self.pending_ui_scale_percent = normalized;
        self.config.ui_scale_percent = normalized;
        let _ = self.config.save();
    }
    pub fn set_remember_window_position(&mut self, enabled: bool) {
        self.config.remember_window_position = enabled;
        if !enabled {
            self.config.window_position = None;
        }
        let _ = self.config.save();
    }
    pub fn set_remember_window_size(&mut self, enabled: bool) {
        self.config.remember_window_size = enabled;
        if !enabled {
            self.config.window_size = None;
        }
        let _ = self.config.save();
    }
    pub fn sync_window_state(&mut self, ctx: &eframe::egui::Context) {
        if !self.config.remember_window_position && !self.config.remember_window_size {
            return;
        }

        let viewport = ctx.input(|input| input.viewport().clone());
        if viewport.minimized.unwrap_or(false) || viewport.maximized.unwrap_or(false) {
            return;
        }

        let mut changed = false;
        if self.config.remember_window_position {
            if let Some(outer_rect) = viewport.outer_rect {
                if let Some(position) = WindowPosition::new(outer_rect.min.x, outer_rect.min.y) {
                    if self.config.window_position != Some(position) {
                        self.config.window_position = Some(position);
                        changed = true;
                    }
                }
            }
        }

        if self.config.remember_window_size {
            if let Some(inner_rect) = viewport.inner_rect {
                let size = inner_rect.size();
                if let Some(window_size) = WindowSize::new(size.x, size.y) {
                    if self.config.window_size != Some(window_size) {
                        self.config.window_size = Some(window_size);
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            return;
        }

        let _ = self.config.save();
    }
    pub fn set_batch_limit_enabled(&mut self, enabled: bool) {
        self.config.batch_limit_enabled = enabled;
        let _ = self.config.save();
    }
    pub fn set_direct_download_on_add(&mut self, enabled: bool) {
        self.config.direct_download_on_add = enabled;
        let _ = self.config.save();
    }
    pub fn set_output_file_action_mode(&mut self, mode: OutputFileActionMode) {
        self.config.output_file_action_mode = mode;
        let _ = self.config.save();
    }
    pub fn set_batch_limit_count(&mut self, count: usize) {
        self.config.batch_limit_count = count.max(1);
        let _ = self.config.save();
    }
    pub fn set_monitor_clipboard(&mut self, enabled: bool) {
        self.monitor_clipboard = enabled;
        self.config.auto_paste_clipboard = enabled;
        let _ = self.config.save();
        if enabled {
            self.last_clipboard_text = read_clipboard_text().unwrap_or_default();
            self.last_clipboard_check = Some(Instant::now());
            self.clipboard_monitor_baseline_ready = true;
            self.last_action = if self.config.clipboard_auto_add {
                "Clipboard monitor enabled; the next YouTube URL change will be added immediately."
                    .to_owned()
            } else {
                "Clipboard monitor enabled; the next YouTube URL change will fill the URL field."
                    .to_owned()
            };
        } else {
            self.clipboard_monitor_baseline_ready = false;
            self.last_action = "Clipboard monitor disabled.".to_owned();
        }
    }
    pub fn set_clipboard_auto_add(&mut self, enabled: bool) {
        self.config.clipboard_auto_add = enabled;
        let _ = self.config.save();
        if self.monitor_clipboard {
            self.last_clipboard_text = read_clipboard_text().unwrap_or_default();
            self.last_clipboard_check = Some(Instant::now());
            self.clipboard_monitor_baseline_ready = true;
            self.last_action = if enabled {
                "YouTube URLs will be added immediately after the clipboard changes.".to_owned()
            } else {
                "Clipboard changes will only fill the URL field.".to_owned()
            };
        }
    }
    pub fn set_youtube_high_risk_playlist_prompt(&mut self, enabled: bool) {
        self.config.youtube_high_risk_playlist_prompt = enabled;
        let _ = self.config.save();
    }
    pub fn set_youtube_video_playlist_mode(&mut self, mode: YoutubeVideoPlaylistMode) {
        self.config.youtube_video_playlist_mode = mode;
        let _ = self.config.save();
    }
    pub fn available_concurrent_fragment_values(&self) -> [usize; 4] {
        [1, 2, 4, 8]
    }
    pub fn set_concurrent_fragments(&mut self, value: usize) {
        let value = match value {
            1 | 2 | 4 | 8 => value,
            0 => 1,
            3 => 4,
            5..=7 => 8,
            _ => 8,
        };
        self.tool_paths.concurrent_fragments = value;
        self.config.concurrent_fragments = value;
        let _ = self.config.save();
    }
    pub fn set_youtube_subs_po_token(&mut self, token: impl Into<String>) {
        let token = token.into();
        self.tool_paths.youtube_subs_po_token = token.clone();
        self.config.youtube_subs_po_token = token;
        let _ = self.config.save();
    }
    pub fn set_youtube_extractor_args(&mut self, args: impl Into<String>) {
        let args = args.into();
        self.tool_paths.youtube_extractor_args = args.clone();
        self.config.youtube_extractor_args = args;
        let _ = self.config.save();
    }
    pub fn set_use_browser_cookies(&mut self, enabled: bool) {
        self.item_defaults.use_cookies = enabled;
        for item in &mut self.queue_items {
            item.selection.use_cookies = enabled;
        }
        self.config.use_browser_cookies = enabled;
        let _ = self.config.save();
    }
    pub fn set_use_aria2(&mut self, enabled: bool) {
        self.item_defaults.use_aria2 = enabled;
        for item in &mut self.queue_items {
            item.selection.use_aria2 = enabled;
        }
        self.config.use_aria2 = enabled;
        let _ = self.config.save();
    }
    pub fn set_thumbnail_post_process_mode(&mut self, mode: PostProcessMode) {
        self.item_defaults.write_thumbnail = mode.writes();
        self.item_defaults.embed_thumbnail = mode.embeds();
        for item in &mut self.queue_items {
            item.selection.write_thumbnail = mode.writes();
            item.selection.embed_thumbnail = mode.embeds();
            if mode.embeds()
                && item.selection.container_preference == DownloadContainerPreference::Webm
            {
                item.selection.container_preference = DownloadContainerPreference::Mkv;
            }
        }
        self.config.thumbnail_mode = mode;
        let _ = self.config.save();
    }
    pub fn set_subtitle_post_process_mode(&mut self, mode: PostProcessMode) {
        self.item_defaults.write_subtitles = mode.writes();
        self.item_defaults.embed_subtitles = mode.embeds();
        for item in &mut self.queue_items {
            item.selection.write_subtitles = mode.writes();
            item.selection.embed_subtitles = mode.embeds();
        }
        self.config.subtitle_mode = mode;
        let _ = self.config.save();
    }
    pub fn set_chapter_post_process_mode(&mut self, mode: PostProcessMode) {
        self.item_defaults.write_chapters = mode.writes();
        self.item_defaults.embed_chapters = mode.embeds();
        for item in &mut self.queue_items {
            item.selection.write_chapters = mode.writes();
            item.selection.embed_chapters = mode.embeds();
        }
        self.config.chapter_mode = mode;
        let _ = self.config.save();
    }
    pub fn set_write_thumbnail(&mut self, enabled: bool) {
        let mode = if enabled {
            if self.item_defaults.embed_thumbnail {
                PostProcessMode::Embed
            } else {
                PostProcessMode::Download
            }
        } else {
            PostProcessMode::Off
        };
        self.set_thumbnail_post_process_mode(mode);
    }
    pub fn set_embed_thumbnail(&mut self, enabled: bool) {
        let mode = if enabled {
            PostProcessMode::Embed
        } else if self.item_defaults.write_thumbnail {
            PostProcessMode::Download
        } else {
            PostProcessMode::Off
        };
        self.set_thumbnail_post_process_mode(mode);
    }
    pub fn set_write_subtitles(&mut self, enabled: bool) {
        let mode = if enabled {
            if self.item_defaults.embed_subtitles {
                PostProcessMode::Embed
            } else {
                PostProcessMode::Download
            }
        } else {
            PostProcessMode::Off
        };
        self.set_subtitle_post_process_mode(mode);
    }
    pub fn set_embed_subtitles(&mut self, enabled: bool) {
        let mode = if enabled {
            PostProcessMode::Embed
        } else if self.item_defaults.write_subtitles {
            PostProcessMode::Download
        } else {
            PostProcessMode::Off
        };
        self.set_subtitle_post_process_mode(mode);
    }
    pub fn set_write_chapters(&mut self, enabled: bool) {
        let mode = if enabled {
            if self.item_defaults.embed_chapters {
                PostProcessMode::Embed
            } else {
                PostProcessMode::Download
            }
        } else {
            PostProcessMode::Off
        };
        self.set_chapter_post_process_mode(mode);
    }
    pub fn set_embed_chapters(&mut self, enabled: bool) {
        let mode = if enabled {
            PostProcessMode::Embed
        } else if self.item_defaults.write_chapters {
            PostProcessMode::Download
        } else {
            PostProcessMode::Off
        };
        self.set_chapter_post_process_mode(mode);
    }
}
