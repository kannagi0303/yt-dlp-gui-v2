use super::*;

impl AppState {
    pub(super) fn open_download_range_picker(&mut self, target_item_id: usize) {
        let Some(item) = self.queue_items.get(target_item_id) else {
            return;
        };
        let chapter_count = item
            .metadata()
            .map(|metadata| metadata.chapters.len())
            .unwrap_or(0);
        let mut selection = item.selection.download_range.clone();
        selection.retain_available_chapters(chapter_count);
        let playhead_millis = selection
            .custom_time_ranges()
            .first()
            .map(DownloadTimeRange::start_millis)
            .unwrap_or_default();

        self.format_picker.open = true;
        self.format_picker.target_item_id = Some(target_item_id);
        self.format_picker.kind = Some(FormatPickerKind::Section);
        self.format_picker.selected_row = None;
        self.format_picker.filter_text.clear();
        self.format_picker.filters.clear();
        self.format_picker.section_tab = if chapter_count == 0 {
            SectionPickerTab::TimeRange
        } else {
            SectionPickerTab::Chapters
        };
        self.format_picker.download_range_draft = DownloadRangePickerDraft {
            selection,
            playhead_millis,
            start_marker_millis: None,
            end_marker_millis: None,
        };
    }

    pub fn confirm_download_range_picker_selection(&mut self) {
        let Some(item_index) = self.format_picker.target_item_id else {
            return;
        };
        if self.format_picker.kind != Some(FormatPickerKind::Section) {
            return;
        }

        let chapter_count = self
            .queue_items
            .get(item_index)
            .and_then(QueueItem::metadata)
            .map(|metadata| metadata.chapters.len())
            .unwrap_or(0);
        let mut selection = self.format_picker.download_range_draft.selection.clone();
        selection.retain_available_chapters(chapter_count);
        let output_count = resolved_download_output_count(
            &selection,
            self.queue_items
                .get(item_index)
                .and_then(QueueItem::metadata),
        );
        let output_count_text = output_count.to_string();

        let Some(item) = self.queue_items.get_mut(item_index) else {
            self.cancel_format_picker();
            return;
        };
        let title = item.title.clone();
        item.selection.download_range = selection;
        item.completed_selection = None;
        self.last_action = if item.selection.download_range.is_full_video() {
            i18n::format_fixed_english(
                "Download range set: {title} / Full video",
                &[("{title}", title.as_str())],
            )
        } else {
            i18n::format_fixed_english(
                "Download range set: {title} / {count} outputs",
                &[
                    ("{title}", title.as_str()),
                    ("{count}", output_count_text.as_str()),
                ],
            )
        };
        self.cancel_format_picker();
    }

    pub fn item_shows_download_section_row(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        !item.selection.download_range.is_full_video()
            || item.metadata().is_some_and(|metadata| {
                !metadata.chapters.is_empty() || self.config.always_show_download_range
            })
    }

    pub fn selected_download_section_summary(&self, item_index: usize) -> String {
        let Some(item) = self.queue_items.get(item_index) else {
            return String::new();
        };
        let selection = &item.selection.download_range;
        if selection.is_full_video() {
            return self.ui_i18n_text_for_key("picker.full_video").to_owned();
        }

        self.download_range_selection_summary(selection, item.metadata())
    }

    pub fn pending_download_range_summary(&self) -> String {
        let metadata = self.current_download_range_metadata();
        self.download_range_selection_summary(
            &self.format_picker.download_range_draft.selection,
            metadata,
        )
    }

    pub fn pending_download_range_is_empty(&self) -> bool {
        self.format_picker
            .download_range_draft
            .selection
            .is_full_video()
    }

    pub fn pending_download_range_output_count(&self) -> usize {
        resolved_download_output_count(
            &self.format_picker.download_range_draft.selection,
            self.current_download_range_metadata(),
        )
    }

    fn download_range_selection_summary(
        &self,
        selection: &DownloadRangeSelection,
        metadata: Option<&VideoMetadata>,
    ) -> String {
        if selection.is_full_video() {
            return self.ui_i18n_text_for_key("picker.full_video").to_owned();
        }

        let chapter_count = metadata
            .map(|metadata| metadata.chapters.len())
            .unwrap_or(0);
        let selected_chapter_count = selection
            .selected_chapter_indices()
            .iter()
            .filter(|index| **index < chapter_count)
            .count();
        let custom_range_count = selection.custom_time_ranges().len();
        if selected_chapter_count == 0 && custom_range_count == 0 {
            return self.ui_i18n_text_for_key("picker.full_video").to_owned();
        }
        let output_count = resolved_download_output_count(selection, metadata);
        let chapter_count_text = selected_chapter_count.to_string();
        let custom_count_text = custom_range_count.to_string();
        let output_count_text = output_count.to_string();

        let (key, replacements): (&'static str, Vec<(&str, &str)>) =
            match (selected_chapter_count, custom_range_count) {
                (0, _) => (
                    "picker.section_summary.custom",
                    vec![
                        ("{ranges}", custom_count_text.as_str()),
                        ("{outputs}", output_count_text.as_str()),
                    ],
                ),
                (_, 0) => (
                    "picker.section_summary.chapters",
                    vec![
                        ("{chapters}", chapter_count_text.as_str()),
                        ("{outputs}", output_count_text.as_str()),
                    ],
                ),
                _ => (
                    "picker.section_summary.combined",
                    vec![
                        ("{chapters}", chapter_count_text.as_str()),
                        ("{ranges}", custom_count_text.as_str()),
                        ("{outputs}", output_count_text.as_str()),
                    ],
                ),
            };
        self.ui_i18n_text_with_replacements(key, &replacements)
    }

    pub fn current_download_range_chapters(&self) -> Vec<crate::domain::ChapterOption> {
        self.current_download_range_metadata()
            .map(|metadata| metadata.chapters.clone())
            .unwrap_or_default()
    }

    pub fn current_download_range_duration_millis(&self) -> Option<u64> {
        let metadata = self.current_download_range_metadata()?;
        metadata.duration_millis.or_else(|| {
            metadata
                .chapters
                .iter()
                .filter_map(|chapter| chapter.end_millis)
                .max()
        })
    }

    fn current_download_range_metadata(&self) -> Option<&VideoMetadata> {
        self.format_picker
            .target_item_id
            .and_then(|index| self.queue_items.get(index))
            .and_then(QueueItem::metadata)
    }

    pub fn localized_chapter_label(&self, chapter: &crate::domain::ChapterOption) -> String {
        let range = match chapter.end_text.as_deref() {
            Some(end) if !end.is_empty() => format!("{}–{}", chapter.start_text, end),
            _ => format!(
                "{}–{}",
                chapter.start_text,
                self.ui_i18n_text_for_key("picker.until_end")
            ),
        };

        if chapter.title.trim().is_empty() {
            range
        } else {
            format!("{}  {}", range, chapter.title)
        }
    }

    pub fn clear_pending_download_range_selection(&mut self) {
        self.format_picker.download_range_draft.selection.clear();
        self.format_picker.download_range_draft.start_marker_millis = None;
        self.format_picker.download_range_draft.end_marker_millis = None;
    }

    pub fn set_pending_download_range_chapter_selected(
        &mut self,
        chapter_index: usize,
        selected: bool,
    ) {
        let chapter_count = self.current_download_range_chapters().len();
        if chapter_index < chapter_count {
            self.format_picker
                .download_range_draft
                .selection
                .set_chapter_selected(chapter_index, selected);
        }
    }

    pub fn select_all_pending_download_range_chapters(&mut self) {
        let chapter_count = self.current_download_range_chapters().len();
        self.format_picker
            .download_range_draft
            .selection
            .select_all_chapters(chapter_count);
    }

    pub fn clear_pending_download_range_chapters(&mut self) {
        self.format_picker
            .download_range_draft
            .selection
            .clear_chapters();
    }

    pub fn select_pending_download_range_chapters_to_end(&mut self) {
        let chapter_count = self.current_download_range_chapters().len();
        self.format_picker
            .download_range_draft
            .selection
            .select_chapters_from_first_selected_to_end(chapter_count);
    }

    pub fn set_pending_download_range_playhead(&mut self, millis: u64) {
        let maximum = self
            .current_download_range_duration_millis()
            .unwrap_or(millis);
        self.format_picker.download_range_draft.playhead_millis = millis.min(maximum);
    }

    pub fn set_pending_download_range_start_marker(&mut self) {
        let playhead = self.format_picker.download_range_draft.playhead_millis;
        self.format_picker.download_range_draft.start_marker_millis = Some(playhead);
        if self
            .format_picker
            .download_range_draft
            .end_marker_millis
            .is_some_and(|end| end <= playhead)
        {
            self.format_picker.download_range_draft.end_marker_millis = None;
        }
    }

    pub fn set_pending_download_range_end_marker(&mut self) {
        let draft = &mut self.format_picker.download_range_draft;
        if draft
            .start_marker_millis
            .is_some_and(|start| draft.playhead_millis > start)
        {
            draft.end_marker_millis = Some(draft.playhead_millis);
        }
    }

    pub fn add_pending_custom_download_range(&mut self) {
        let draft = &mut self.format_picker.download_range_draft;
        let Some(start) = draft.start_marker_millis else {
            return;
        };
        let Some(end) = draft.end_marker_millis else {
            return;
        };
        let Some(range) = DownloadTimeRange::new(start, end) else {
            return;
        };
        draft.selection.add_custom_time_range(range);
        draft.playhead_millis = end;
        draft.start_marker_millis = None;
        draft.end_marker_millis = None;
    }

    pub fn remove_pending_custom_download_range(&mut self, range_index: usize) {
        self.format_picker
            .download_range_draft
            .selection
            .remove_custom_time_range(range_index);
    }

    pub(super) fn item_download_section_arguments(&self, item_index: usize) -> Vec<String> {
        let Some(item) = self.queue_items.get(item_index) else {
            return Vec::new();
        };
        resolved_download_section_arguments(&item.selection.download_range, item.metadata())
    }
}

fn resolved_download_section_arguments(
    selection: &DownloadRangeSelection,
    metadata: Option<&VideoMetadata>,
) -> Vec<String> {
    let Some(metadata) = metadata else {
        return selection
            .custom_time_ranges()
            .iter()
            .map(DownloadTimeRange::yt_dlp_argument)
            .collect();
    };

    let mut ranges = Vec::new();
    for (start_index, end_index) in
        selection.grouped_selected_chapter_spans(metadata.chapters.len())
    {
        let start = &metadata.chapters[start_index];
        let end = &metadata.chapters[end_index];
        let end_text = end
            .end_text
            .clone()
            .or_else(|| {
                metadata
                    .duration_millis
                    .map(format_download_range_timestamp)
            })
            .unwrap_or_default();
        ranges.push((
            start.start_millis,
            format!("*{}-{end_text}", start.start_text),
        ));
    }
    ranges.extend(
        selection
            .custom_time_ranges()
            .iter()
            .map(|range| (range.start_millis(), range.yt_dlp_argument())),
    );
    ranges.sort_by_key(|(start, _)| *start);
    ranges.dedup_by(|left, right| left.1 == right.1);
    ranges.into_iter().map(|(_, argument)| argument).collect()
}

fn resolved_download_output_count(
    selection: &DownloadRangeSelection,
    metadata: Option<&VideoMetadata>,
) -> usize {
    resolved_download_section_arguments(selection, metadata)
        .len()
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ChapterOption;

    fn chapter(index: usize, start: u64, end: u64) -> ChapterOption {
        ChapterOption::new(
            format!("chapter:{index}"),
            format!("Chapter {}", index + 1),
            format_download_range_timestamp(start),
            Some(format_download_range_timestamp(end)),
            start,
            Some(end),
        )
    }

    #[test]
    fn adjacent_chapters_become_one_download_section_and_gaps_remain_separate() {
        let mut metadata = VideoMetadata::empty_preview();
        metadata.duration_millis = Some(50_000);
        metadata.chapters = vec![
            chapter(0, 0, 10_000),
            chapter(1, 10_000, 20_000),
            chapter(2, 20_000, 30_000),
            chapter(3, 30_000, 40_000),
            chapter(4, 40_000, 50_000),
        ];
        let mut selection = DownloadRangeSelection::default();
        selection.set_chapter_selected(1, true);
        selection.set_chapter_selected(3, true);
        selection.set_chapter_selected(4, true);

        assert_eq!(
            resolved_download_section_arguments(&selection, Some(&metadata)),
            vec!["*00:00:10-00:00:20", "*00:00:30-00:00:50"]
        );
    }

    #[test]
    fn chapter_and_custom_ranges_are_emitted_in_timeline_order() {
        let mut metadata = VideoMetadata::empty_preview();
        metadata.chapters = vec![chapter(0, 20_000, 30_000)];
        let mut selection = DownloadRangeSelection::default();
        selection.set_chapter_selected(0, true);
        selection.add_custom_time_range(
            DownloadTimeRange::new(5_000, 10_000).expect("valid custom range"),
        );

        assert_eq!(
            resolved_download_section_arguments(&selection, Some(&metadata)),
            vec!["*00:00:05-00:00:10", "*00:00:20-00:00:30"]
        );
    }
}
