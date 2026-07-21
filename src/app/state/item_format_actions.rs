use super::*;

impl AppState {
    pub fn latest_download_status(&self) -> Option<String> {
        let item = self.queue_items.last()?;
        Some(format!(
            "{}: {} | {}",
            self.ui_i18n_text_for_key(queue_item_status_key(item)),
            item.title,
            item.last_output_path
                .as_deref()
                .or(item.last_error.as_deref())
                .unwrap_or(item.source_url.as_str())
        ))
    }
    pub fn item_status_text(&self, item_index: usize) -> &'static str {
        let Some(item) = self.queue_items.get(item_index) else {
            return "";
        };
        self.ui_i18n_text_for_key(queue_item_status_key(item))
    }
    pub fn single_mode_status_lines(&self) -> Vec<(String, String)> {
        let Some(item) = self.queue_items.first() else {
            return Vec::new();
        };

        let Some(run) = item
            .workflows
            .iter()
            .rev()
            .find(|run| single_mode_status_workflow_visible(run, item))
        else {
            return Vec::new();
        };

        let mut lines = Vec::new();
        lines.push(("Downloader".to_owned(), workflow_tool_label(&run.tool)));

        for line in run.detail.lines() {
            let Some((label, value)) = line.split_once('\t') else {
                continue;
            };
            let label = label.trim();
            let value = value.trim();
            if !label.is_empty() && !value.is_empty() {
                lines.push((label.to_owned(), value.to_owned()));
            }
        }

        if run.progress > 0.0 && run.progress < 100.0 && !status_lines_contain(&lines, "Progress") {
            lines.push((
                "Progress".to_owned(),
                format!("{:.0}%", run.progress.round()),
            ));
        }

        let status = match run.state {
            WorkflowState::Queued => self.ui_i18n_text_for_key("item.status.queued"),
            WorkflowState::Running => self.ui_i18n_text_for_key("item.status.running"),
            WorkflowState::Finished if item.last_error.is_some() => {
                self.ui_i18n_text_for_key("item.status.failed")
            }
            WorkflowState::Finished => self.ui_i18n_text_for_key("item.status.finished"),
            WorkflowState::Failed => self.ui_i18n_text_for_key("item.status.failed"),
            WorkflowState::Cancelled => self.ui_i18n_text_for_key("item.status.cancelled"),
        };
        lines.push(("Status".to_owned(), status.to_owned()));

        if let Some(error) = run
            .error
            .as_deref()
            .or(item.last_error.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            lines.push(("Error".to_owned(), error.to_owned()));
        }

        lines
    }
    pub fn item_title_text(&self, item_index: usize) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        if !item.title.trim().is_empty() {
            item.title.clone()
        } else {
            item.source_url.clone()
        }
    }
    pub fn item_title_is_loading(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };
        matches!(item.metadata_state, MetadataState::Running)
            || item
                .workflows
                .iter()
                .any(|run| run.state == WorkflowState::Running)
    }
    pub fn single_mode_analysis_running(&self) -> bool {
        if self.app_mode != AppMode::Origin {
            return false;
        }
        let Some(item) = self.queue_items.first() else {
            return false;
        };
        matches!(
            item.metadata_state,
            MetadataState::Queued | MetadataState::Running
        ) || item.workflows.iter().any(|run| {
            run.kind == WorkflowKind::AnalyzeMetadata
                && matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
        })
    }
    pub fn url_input_locked(&self) -> bool {
        self.is_adding_batch || self.youtube_playlist_prompt.is_some()
    }
    pub(super) fn add_single_url_to_batch(&mut self, source: String) {
        if let Err(error) = self.ensure_yt_dlp_ready() {
            self.last_action = error;
            return;
        }

        let item_id = if self.app_mode == AppMode::Origin {
            if !self.active_workflows.is_empty() {
                self.last_action =
                    "Wait for the current Origin Mode item to finish first.".to_owned();
                return;
            }
            self.stop_music_playback();
            self.queue_items.clear();
            self.batch_input.clear();
            let item = self.build_queue_item_from_url(&source);
            let item_id = item.id;
            self.queue_items.push(item);
            item_id
        } else {
            self.ensure_queue_item_for_url(&source)
        };
        if self.app_mode != AppMode::Origin {
            self.url_input.clear();
        }
        let fallback_title = infer_title(&source, "Untitled task", "Imported {tail}");
        self.last_action = i18n::format_fixed_english(
            "Added to list: {title}",
            &[("{title}", fallback_title.as_str())],
        );
        self.enqueue_item_analysis(item_id, source);
    }
    pub fn item_title_visual_state(&self, item_index: usize) -> ItemTitleVisualState {
        let Some(item) = self.queue_items.get(item_index) else {
            return ItemTitleVisualState::Default;
        };

        if item.workflows.iter().any(|run| {
            matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
                && matches!(
                    run.kind,
                    WorkflowKind::DownloadMedia
                        | WorkflowKind::ExportMedia
                        | WorkflowKind::PostProcess
                )
        }) {
            return ItemTitleVisualState::Pending;
        }

        if item
            .completed_selection
            .as_ref()
            .is_some_and(|completed| selection_matches_completed(&item.selection, completed))
        {
            return ItemTitleVisualState::Completed;
        }

        if item.last_error.is_some() || matches!(item.metadata_state, MetadataState::Failed(_)) {
            return ItemTitleVisualState::Failed;
        }

        if matches!(
            item.metadata_state,
            MetadataState::Queued | MetadataState::Running
        ) {
            return ItemTitleVisualState::Pending;
        }

        if item.metadata_loaded() {
            return ItemTitleVisualState::Ready;
        }

        ItemTitleVisualState::Pending
    }
    pub fn item_error_text(&self, item_index: usize) -> Option<String> {
        let item = self.queue_items.get(item_index)?;
        if let Some(error) = item.last_error.as_ref() {
            return Some(error.clone());
        }

        if item_latest_download_state(item).is_some_and(|state| {
            matches!(
                state,
                WorkflowState::Queued | WorkflowState::Running | WorkflowState::Finished
            )
        }) {
            return None;
        }

        match &item.metadata_state {
            MetadataState::Failed(error) => Some(error.clone()),
            _ => None,
        }
    }
    pub fn item_output_file_path(&self, item_index: usize) -> Option<String> {
        let item = self.queue_items.get(item_index)?;
        if item.last_error.is_some() {
            return None;
        }
        if !item_latest_download_state(item).is_some_and(|state| state == WorkflowState::Finished) {
            return None;
        }
        item.last_output_path
            .clone()
            .filter(|path| !path.trim().is_empty())
    }
    pub fn item_progress(&self, item_index: usize, kind: FormatPickerKind) -> f32 {
        let Some(item) = self.queue_items.get(item_index) else {
            return 0.0;
        };
        let raw = if self.item_uses_muxed_video(item_index) {
            let shared = item.progress.video.max(item.progress.audio);
            match kind {
                FormatPickerKind::Video | FormatPickerKind::Audio => shared,
                FormatPickerKind::Subtitle => item.progress.subtitle,
                FormatPickerKind::Section => 0.0,
            }
        } else {
            match kind {
                FormatPickerKind::Video => item.progress.video,
                FormatPickerKind::Audio => item.progress.audio,
                FormatPickerKind::Subtitle => item.progress.subtitle,
                FormatPickerKind::Section => 0.0,
            }
        };
        raw.clamp(0.0, 100.0)
    }
    pub fn item_av_progress_visible(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        let has_active_download = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id && workflow.kind == WorkflowKind::DownloadMedia
        });
        if has_active_download {
            let video = item.progress.video;
            let audio = item.progress.audio;
            return video > 0.0 || audio > 0.0;
        }

        let has_active_export = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id && workflow.kind == WorkflowKind::ExportMedia
        });
        if has_active_export {
            let video = item.progress.video;
            let audio = item.progress.audio;
            let active_sides = [video, audio]
                .into_iter()
                .filter(|value| *value > 0.0)
                .collect::<Vec<_>>();
            if active_sides.is_empty() {
                return false;
            }
            return active_sides.iter().any(|value| *value < 100.0);
        }

        false
    }
    pub fn item_subtitle_progress_visible(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        let has_active_work = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id
                && matches!(
                    workflow.kind,
                    WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia
                )
        });
        if !has_active_work {
            return false;
        }

        item.progress.subtitle > 0.0 && item.progress.subtitle < 100.0
    }
    pub fn item_file_name_progress(&self, item_index: usize) -> f32 {
        self.queue_items
            .get(item_index)
            .map(|item| item.progress.post_process)
            .unwrap_or(0.0)
    }
    pub fn item_file_name_progress_visible(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        let has_active_post_process = self.active_workflows.values().any(|workflow| {
            workflow.item_id == item.id && workflow.kind == WorkflowKind::PostProcess
        });
        has_active_post_process
            && item.progress.post_process > 0.0
            && item.progress.post_process < 100.0
    }
    pub(super) fn resolve_download_format_selection(
        &self,
        video_selector: &str,
        audio_selector: &str,
        metadata: Option<&VideoMetadata>,
    ) -> (String, String) {
        if self.is_muxed_format(video_selector, metadata) {
            return (video_selector.to_owned(), video_selector.to_owned());
        }

        let resolved_audio = if audio_selector.trim().is_empty()
            || audio_selector == video_selector
            || self.is_muxed_format(audio_selector, metadata)
        {
            metadata
                .into_iter()
                .flat_map(|metadata| metadata.formats.iter())
                .find(|format| format.kind == MediaKind::Audio)
                .map(|format| format.id.clone())
                .unwrap_or_default()
        } else {
            audio_selector.to_owned()
        };

        if resolved_audio.is_empty() {
            (resolved_audio, video_selector.to_owned())
        } else {
            (
                resolved_audio.clone(),
                format!("{}+{}", video_selector.trim(), resolved_audio.trim()),
            )
        }
    }
    pub fn video_formats(&self) -> impl Iterator<Item = &FormatOption> {
        self.current_picker_metadata()
            .formats
            .iter()
            .filter(|format| matches!(format.kind, MediaKind::Video | MediaKind::Muxed))
    }
    pub fn audio_formats(&self) -> impl Iterator<Item = &FormatOption> {
        self.current_picker_metadata()
            .formats
            .iter()
            .filter(|format| format.kind == MediaKind::Audio)
    }
    pub fn subtitle_source_options(&self) -> Vec<SubtitleOption> {
        let mut items: Vec<SubtitleOption> = self
            .current_picker_metadata()
            .subtitle_tracks
            .iter()
            .cloned()
            .fold(Vec::new(), |mut acc, track| {
                if !acc
                    .iter()
                    .any(|item| item.source_key() == track.source_key())
                {
                    acc.push(track);
                }
                acc
            });

        items.sort_by(|left, right| {
            left.source
                .label()
                .cmp(right.source.label())
                .then_with(|| left.source_language_label.cmp(&right.source_language_label))
                .then_with(|| left.source_language_code.cmp(&right.source_language_code))
        });
        items
    }
    pub fn subtitle_translation_options(&self) -> Vec<SubtitleOption> {
        let source_key = self.current_subtitle_source_key();
        let mut items: Vec<SubtitleOption> = self
            .current_picker_metadata()
            .subtitle_tracks
            .iter()
            .filter(|track| track.source_key() == source_key)
            .cloned()
            .collect();

        items.sort_by(|left, right| {
            left.target_language_code
                .is_some()
                .cmp(&right.target_language_code.is_some())
                .then_with(|| left.target_label().cmp(&right.target_label()))
        });
        items
    }
    pub fn open_format_picker(&mut self, target_item_id: usize, kind: FormatPickerKind) {
        if kind == FormatPickerKind::Section {
            self.open_download_range_picker(target_item_id);
            return;
        }

        let selected_id = self
            .queue_items
            .get(target_item_id)
            .map(|item| match kind {
                FormatPickerKind::Video => item.selection.video_selector.as_str(),
                FormatPickerKind::Audio => item.selection.audio_selector.as_str(),
                FormatPickerKind::Subtitle => item.selection.subtitle_selector.as_str(),
                FormatPickerKind::Section => "",
            })
            .unwrap_or_default()
            .to_owned();

        self.format_picker.open = true;
        self.format_picker.target_item_id = Some(target_item_id);
        self.format_picker.kind = Some(kind);
        self.format_picker.view_mode = if self.app_mode == AppMode::Origin
            && matches!(kind, FormatPickerKind::Video | FormatPickerKind::Audio)
        {
            FormatPickerViewMode::Table
        } else {
            FormatPickerViewMode::Filter
        };
        self.format_picker.filter_text.clear();
        self.format_picker.filters.clear();

        if kind == FormatPickerKind::Subtitle {
            let subtitle_source = self
                .queue_items
                .get(target_item_id)
                .map(|item| item.selection.subtitle_source)
                .unwrap_or(SubtitleSource::None);
            self.format_picker.subtitle_tab = match subtitle_source {
                SubtitleSource::None => SubtitlePickerTab::None,
                SubtitleSource::Original => SubtitlePickerTab::Original,
                SubtitleSource::Automatic => SubtitlePickerTab::Automatic,
            };
            self.format_picker.subtitle_source_key = match subtitle_source {
                SubtitleSource::None => SubtitleSource::None.key().to_owned(),
                _ => self
                    .queue_items
                    .get(target_item_id)
                    .and_then(|item| {
                        self.subtitle_track_by_id(
                            &item.selection.subtitle_selector,
                            self.item_metadata(target_item_id),
                        )
                    })
                    .map(|track| track.source_key())
                    .unwrap_or_default(),
            };
            self.format_picker.selected_row = if subtitle_source == SubtitleSource::None {
                Some(0)
            } else {
                let options = if subtitle_source == SubtitleSource::Original {
                    self.subtitle_source_options()
                        .into_iter()
                        .filter(|track| track.source == SubtitleSource::Original)
                        .collect::<Vec<_>>()
                } else {
                    self.subtitle_translation_options()
                };
                options.iter().position(|option| option.id == selected_id)
            };
            return;
        }

        let options = self.format_picker_options(kind);
        let selected_row = options.iter().position(|option| option.id == selected_id);
        let selected_option = selected_row.and_then(|index| options.get(index));
        self.format_picker.selected_row = selected_row;

        if let Some(option) = selected_option {
            match kind {
                FormatPickerKind::Video => {
                    if !option.resolution.is_empty() {
                        self.format_picker.filters.resolution = Some(option.resolution.clone());
                    }
                    if !option.dynamic_range.is_empty() {
                        self.format_picker.filters.dynamic_range =
                            Some(option.dynamic_range.clone());
                    }
                    if !option.fps.is_empty() {
                        self.format_picker.filters.fps = Some(option.fps.clone());
                    }
                    if !option.codec.is_empty() {
                        self.format_picker.filters.codec = Some(option.codec.clone());
                    }
                }
                FormatPickerKind::Audio => {
                    if !option.sample_rate.is_empty() {
                        self.format_picker.filters.sample_rate = Some(option.sample_rate.clone());
                    }
                    if !option.codec.is_empty() {
                        self.format_picker.filters.codec = Some(option.codec.clone());
                    }
                }
                FormatPickerKind::Subtitle | FormatPickerKind::Section => {}
            }
        }
    }
    pub fn cancel_format_picker(&mut self) {
        self.format_picker.open = false;
        self.format_picker.target_item_id = None;
        self.format_picker.kind = None;
        self.format_picker.selected_row = None;
        self.format_picker.filter_text.clear();
        self.format_picker.filters.clear();
        self.format_picker.subtitle_source_key.clear();
        self.format_picker.subtitle_tab = SubtitlePickerTab::None;
        self.format_picker.section_tab = SectionPickerTab::Chapters;
        self.format_picker.download_range_draft = DownloadRangePickerDraft::default();
    }
    pub fn confirm_format_picker_selection(&mut self, selected_format_id: &str) {
        let Some(target_item_id) = self.format_picker.target_item_id else {
            return;
        };
        let Some(kind) = self.format_picker.kind else {
            return;
        };
        if kind == FormatPickerKind::Section {
            self.confirm_download_range_picker_selection();
            return;
        }
        let item_metadata = self.item_metadata(target_item_id);
        let is_muxed_selection = self.is_muxed_format(selected_format_id, item_metadata);
        let item_uses_muxed_video = self.item_uses_muxed_video(target_item_id);
        let replacement_audio_selector = if kind == FormatPickerKind::Video && !is_muxed_selection {
            self.replacement_audio_selector_for_video_change(
                target_item_id,
                selected_format_id,
                item_metadata,
            )
        } else {
            None
        };
        let selected_subtitle_source = self
            .subtitle_track_by_id(selected_format_id, item_metadata)
            .map(|track| track.source);
        let Some(item) = self.queue_items.get_mut(target_item_id) else {
            self.cancel_format_picker();
            return;
        };

        match kind {
            FormatPickerKind::Video => {
                item.selection.video_selector = selected_format_id.to_owned();
                if is_muxed_selection {
                    item.selection.audio_selector = selected_format_id.to_owned();
                } else if let Some(audio_selector) = replacement_audio_selector {
                    item.selection.audio_selector = audio_selector;
                }
            }
            FormatPickerKind::Audio => {
                if item_uses_muxed_video {
                    self.cancel_format_picker();
                    return;
                }
                item.selection.audio_selector = selected_format_id.to_owned();
            }
            FormatPickerKind::Subtitle => {
                if selected_format_id.is_empty() {
                    item.selection.subtitle_selector.clear();
                    item.selection.subtitle_source = SubtitleSource::None;
                } else {
                    item.selection.subtitle_selector = selected_format_id.to_owned();
                    item.selection.subtitle_source =
                        selected_subtitle_source.unwrap_or(item.selection.subtitle_source);
                }
            }
            FormatPickerKind::Section => {
                unreachable!("section picker has a structured commit path")
            }
        }

        self.reconcile_item_download_container(target_item_id);
        self.last_action = i18n::format_fixed_english(
            "Format selection updated: Item {index} / {kind} / {value}",
            &[
                ("{index}", &(target_item_id + 1).to_string()),
                ("{kind}", kind.label()),
                ("{value}", selected_format_id),
            ],
        );
        self.cancel_format_picker();
    }
    pub fn format_picker_options(&self, kind: FormatPickerKind) -> Vec<FormatOption> {
        let mut options: Vec<FormatOption> = match kind {
            FormatPickerKind::Video => self.video_formats().cloned().collect(),
            FormatPickerKind::Audio => self.audio_formats().cloned().collect(),
            FormatPickerKind::Subtitle | FormatPickerKind::Section => Vec::new(),
        };

        match kind {
            FormatPickerKind::Video => {
                options.sort_by(|left, right| {
                    video_resolution_area(right)
                        .cmp(&video_resolution_area(left))
                        .then_with(|| {
                            human_size_bytes(&right.filesize).cmp(&human_size_bytes(&left.filesize))
                        })
                        .then_with(|| left.id.cmp(&right.id))
                });
            }
            FormatPickerKind::Audio | FormatPickerKind::Subtitle | FormatPickerKind::Section => {}
        }

        options
    }
    pub fn selected_format_summary(&self, item_index: usize, kind: FormatPickerKind) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        if !item.metadata_loaded() {
            return self
                .ui_i18n_text_for_key("picker.waiting_analysis")
                .to_owned();
        }

        if kind == FormatPickerKind::Audio && self.item_uses_muxed_video(item_index) {
            return self
                .ui_i18n_text_for_key("picker.audio_from_video")
                .to_owned();
        }

        if kind == FormatPickerKind::Subtitle {
            return self.selected_subtitle_summary(item_index);
        }
        if kind == FormatPickerKind::Section {
            return self.selected_download_section_summary(item_index);
        }

        let selected_id = match kind {
            FormatPickerKind::Video => &item.selection.video_selector,
            FormatPickerKind::Audio => &item.selection.audio_selector,
            FormatPickerKind::Subtitle => &item.selection.subtitle_selector,
            FormatPickerKind::Section => "",
        };

        self.format_label_by_id(selected_id, item.metadata())
            .unwrap_or_default()
            .to_owned()
    }
    pub fn format_picker_target_title(&self) -> Option<&str> {
        self.format_picker
            .target_item_id
            .and_then(|index| self.queue_items.get(index))
            .map(|item| item.title.as_str())
    }
    pub fn selected_subtitle_summary(&self, item_index: usize) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };

        if item.selection.subtitle_source == SubtitleSource::None
            || item.selection.subtitle_selector.is_empty()
        {
            return self
                .ui_i18n_text_for_key("picker.subtitle_tab.none")
                .to_owned();
        }

        self.subtitle_track_by_id(&item.selection.subtitle_selector, item.metadata())
            .map(|track| {
                format!(
                    "{} / {}",
                    self.subtitle_source_label(track.source),
                    self.localized_subtitle_target_label(track)
                )
            })
            .unwrap_or_else(|| self.ui_i18n_text_for_key("picker.not_selected").to_owned())
    }
    pub fn subtitle_source_label(&self, source: SubtitleSource) -> &'static str {
        match source {
            SubtitleSource::None => self.ui_i18n_text_for_key("picker.subtitle_tab.none"),
            SubtitleSource::Original => self.ui_i18n_text_for_key("picker.subtitle_tab.original"),
            SubtitleSource::Automatic => self.ui_i18n_text_for_key("picker.subtitle_tab.automatic"),
        }
    }
    pub fn localized_subtitle_target_label(&self, option: &SubtitleOption) -> String {
        match (&option.target_language_label, &option.target_language_code) {
            (Some(label), Some(code)) => format!("{label} ({code})"),
            (Some(label), None) => label.clone(),
            (None, Some(code)) => code.clone(),
            (None, None) => self
                .ui_i18n_text_for_key("picker.no_translation")
                .to_owned(),
        }
    }
    pub fn item_shows_subtitle_row(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        item.metadata()
            .is_some_and(|metadata| !metadata.subtitle_tracks.is_empty())
    }
    pub fn item_uses_seed_compact_layout(&self, item_index: usize) -> bool {
        self.queue_items
            .get(item_index)
            .is_some_and(|item| matches!(item.metadata_state, MetadataState::Idle))
    }
    pub fn subtitle_track_by_id<'a>(
        &'a self,
        id: &str,
        metadata: Option<&'a VideoMetadata>,
    ) -> Option<&'a SubtitleOption> {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.subtitle_tracks.iter())
            .find(|track| track.id == id)
    }
    pub fn current_subtitle_source_key(&self) -> String {
        if !self.format_picker.subtitle_source_key.is_empty() {
            return self.format_picker.subtitle_source_key.clone();
        }

        self.subtitle_source_options()
            .first()
            .map(|track| track.source_key())
            .unwrap_or_else(|| SubtitleSource::None.key().to_owned())
    }
    pub fn item_uses_muxed_video(&self, item_index: usize) -> bool {
        self.queue_items
            .get(item_index)
            .map(|item| self.is_muxed_format(&item.selection.video_selector, item.metadata()))
            .unwrap_or(false)
    }
    fn replacement_audio_selector_for_video_change(
        &self,
        item_index: usize,
        video_selector: &str,
        metadata: Option<&VideoMetadata>,
    ) -> Option<String> {
        let current_audio = self
            .queue_items
            .get(item_index)?
            .selection
            .audio_selector
            .trim();
        if !current_audio.is_empty()
            && current_audio != video_selector
            && !self.is_muxed_format(current_audio, metadata)
        {
            return None;
        }

        first_audio_format_id(metadata)
    }
    pub fn is_muxed_format(&self, format_id: &str, metadata: Option<&VideoMetadata>) -> bool {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == format_id)
            .map(|format| format.kind == MediaKind::Muxed)
            .unwrap_or(false)
    }
    pub fn format_label_by_id<'a>(
        &'a self,
        id: &str,
        metadata: Option<&'a VideoMetadata>,
    ) -> Option<&'a str> {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == id)
            .map(|format| format.label.as_str())
    }
    pub fn format_extension_by_id(&self, id: &str, metadata: Option<&VideoMetadata>) -> String {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == id)
            .map(|format| format.ext.clone())
            .unwrap_or_default()
    }
    pub fn format_codec_by_id(&self, id: &str, metadata: Option<&VideoMetadata>) -> String {
        metadata
            .into_iter()
            .flat_map(|metadata| metadata.formats.iter())
            .find(|format| format.id == id)
            .map(|format| format.codec.clone())
            .unwrap_or_default()
    }
    pub(super) fn apply_analysis_json(
        &mut self,
        json: Value,
        analyzed_source: Option<String>,
        target_item_id: Option<QueueItemId>,
        workflow_id: Option<WorkflowRunId>,
    ) {
        if json.get("entries").and_then(Value::as_array).is_some() {
            let target = analyzed_source.unwrap_or_else(|| "playlist".to_owned());
            self.last_action = i18n::format_fixed_english(
                "Playlist is ignored for now: {target}",
                &[("{target}", target.as_str())],
            );
            return;
        }

        let title = json
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Untitled video")
            .to_owned();
        let webpage_url = json
            .get("webpage_url")
            .or_else(|| json.get("original_url"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let channel = json_str_field(&json, "channel").unwrap_or_default();
        let channel_url = json_str_field(&json, "channel_url").unwrap_or_default();
        let uploader = json_str_field(&json, "uploader").unwrap_or_default();
        let uploader_url = json_str_field(&json, "uploader_url").unwrap_or_default();
        let creator = json_str_field(&json, "creator").unwrap_or_default();
        let creator_url = json_str_field(&json, "creator_url").unwrap_or_default();
        let duration_text = json
            .get("duration_string")
            .and_then(Value::as_str)
            .map(normalize_duration_badge_text)
            .unwrap_or_default();
        let duration_millis = json
            .get("duration")
            .and_then(Value::as_f64)
            .filter(|duration| duration.is_finite() && *duration > 0.0)
            .map(|duration| (duration * 1000.0).round() as u64);
        let description = json
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let upload_date_text = json
            .get("upload_date")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let thumbnail_hint = json
            .get("thumbnail")
            .and_then(Value::as_str)
            .map(|_| "Thumbnail preview".to_owned())
            .unwrap_or_else(|| "item.thumbnail".to_owned());
        let thumbnail_url = select_best_thumbnail_url(&json).unwrap_or_default();

        let formats = extract_formats(&json);
        let requested_ids = extract_requested_ids(&json);
        let subtitle_tracks = extract_subtitle_tracks(&json);
        let chapters = extract_chapters(&json, |index| {
            let number = (index + 1).to_string();
            i18n::format_fixed_english("Chapter {index}", &[("{index}", number.as_str())])
        });

        let metadata = VideoMetadata {
            title: title.clone(),
            channel,
            channel_url,
            uploader,
            uploader_url,
            creator,
            creator_url,
            duration_text,
            duration_millis,
            webpage_url,
            description,
            view_count_text: json_number_or_str_field(&json, "view_count").unwrap_or_default(),
            upload_date_text,
            thumbnail_hint,
            thumbnail_url,
            formats: formats.clone(),
            subtitle_tracks: subtitle_tracks.clone(),
            chapters: chapters.clone(),
        };
        let default_video = requested_or_default_format_id(
            &formats,
            &requested_ids,
            &[MediaKind::Video, MediaKind::Muxed],
        );
        let default_audio = if formats
            .iter()
            .find(|format| format.id == default_video)
            .is_some_and(|format| format.kind == MediaKind::Muxed)
        {
            default_video.clone()
        } else {
            requested_or_default_format_id(&formats, &requested_ids, &[MediaKind::Audio])
        };

        let default_subtitle_source = SubtitleSource::None;
        let default_subtitle = String::new();
        let default_file_name = extract_requested_filename(&json)
            .or_else(|| {
                json.get("_filename")
                    .or_else(|| json.get("filename"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .map(|filename| sanitize_file_name_for_windows(&display_file_stem(&filename)))
            .unwrap_or_default();

        if let Some(item_id) = target_item_id {
            if let Some(item) = self.queue_item_mut_by_id(item_id) {
                item.title = title.clone();
                item.thumbnail_hint = "item.thumbnail".to_owned();
                item.thumbnail_url = metadata.thumbnail_url.clone();
                item.duration_text = metadata.duration_text.clone();
                item.metadata_state = MetadataState::Ready(metadata.clone());
                item.selection
                    .download_range
                    .retain_available_chapters(metadata.chapters.len());
                item.selection.video_selector = default_video.clone();
                item.selection.audio_selector = default_audio.clone();
                item.selection.subtitle_selector = default_subtitle.clone();
                item.selection.subtitle_source = default_subtitle_source;
                if item.selection.file_name.trim().is_empty() {
                    item.selection.file_name = default_file_name.clone();
                }
                let run_index = workflow_id
                    .and_then(|workflow_id| {
                        item.workflows.iter().position(|run| run.id == workflow_id)
                    })
                    .or_else(|| {
                        item.workflows
                            .iter()
                            .rposition(|run| run.kind == WorkflowKind::AnalyzeMetadata)
                    });
                if let Some(run) = run_index.and_then(|index| item.workflows.get_mut(index)) {
                    run.state = WorkflowState::Finished;
                    run.detail = analyzed_source
                        .clone()
                        .unwrap_or_else(|| metadata.webpage_url.clone());
                }
            }
            if let Some(item_index) = self.queue_item_index_by_id(item_id) {
                self.reconcile_item_download_container(item_index);
            }
            if let Some(workflow_id) = workflow_id {
                self.unregister_active_workflow(workflow_id);
            }
        } else {
            let item_source_url = analyzed_source
                .clone()
                .filter(|source| !source.trim().is_empty())
                .or_else(|| {
                    (!metadata.webpage_url.trim().is_empty()).then(|| metadata.webpage_url.clone())
                })
                .unwrap_or_else(|| self.url_input.trim().to_owned());
            let mut item =
                QueueItem::new(self.alloc_queue_item_id(), item_source_url, title.clone());
            item.selection.quality = self.item_defaults.quality;
            item.selection.write_thumbnail = self.item_defaults.write_thumbnail;
            item.selection.embed_thumbnail = self.item_defaults.embed_thumbnail;
            item.selection.write_subtitles = self.item_defaults.write_subtitles;
            item.selection.embed_subtitles = self.item_defaults.embed_subtitles;
            item.selection.write_chapters = self.item_defaults.write_chapters;
            item.selection.embed_chapters = self.item_defaults.embed_chapters;
            item.selection.use_cookies = self.item_defaults.use_cookies;
            item.selection.use_aria2 = self.item_defaults.use_aria2;
            item.selection.output_dir = self.item_defaults.output_dir.clone();
            item.metadata_state = MetadataState::Ready(metadata.clone());
            item.thumbnail_url = metadata.thumbnail_url.clone();
            item.duration_text = metadata.duration_text.clone();
            item.selection.video_selector = default_video.clone();
            item.selection.audio_selector = default_audio.clone();
            item.selection.subtitle_selector = default_subtitle.clone();
            item.selection.subtitle_source = default_subtitle_source;
            item.selection.file_name = default_file_name.clone();
            self.queue_items = vec![item];
        }

        let analyzed_target = analyzed_source
            .or_else(|| (!metadata.webpage_url.is_empty()).then(|| metadata.webpage_url.clone()))
            .unwrap_or_else(|| title.clone());
        self.last_action = i18n::format_fixed_english(
            "Analysis complete: {title}",
            &[("{title}", analyzed_target.as_str())],
        );
        self.mark_font_content_changed();
    }
}
