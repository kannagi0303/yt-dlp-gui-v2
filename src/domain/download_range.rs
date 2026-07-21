#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DownloadRangeSelection {
    selected_chapter_indices: Vec<usize>,
    custom_time_ranges: Vec<DownloadTimeRange>,
}

impl DownloadRangeSelection {
    pub fn is_full_video(&self) -> bool {
        self.selected_chapter_indices.is_empty() && self.custom_time_ranges.is_empty()
    }

    pub fn selected_chapter_indices(&self) -> &[usize] {
        &self.selected_chapter_indices
    }

    pub fn custom_time_ranges(&self) -> &[DownloadTimeRange] {
        &self.custom_time_ranges
    }

    pub fn clear(&mut self) {
        self.selected_chapter_indices.clear();
        self.custom_time_ranges.clear();
    }

    pub fn clear_chapters(&mut self) {
        self.selected_chapter_indices.clear();
    }

    pub fn retain_available_chapters(&mut self, chapter_count: usize) {
        self.selected_chapter_indices
            .retain(|index| *index < chapter_count);
    }

    pub fn select_all_chapters(&mut self, chapter_count: usize) {
        self.selected_chapter_indices = (0..chapter_count).collect();
    }

    pub fn select_chapters_from_first_selected_to_end(&mut self, chapter_count: usize) {
        let Some(first) = self.selected_chapter_indices.first().copied() else {
            return;
        };
        self.selected_chapter_indices = (first..chapter_count).collect();
    }

    pub fn chapter_is_selected(&self, chapter_index: usize) -> bool {
        self.selected_chapter_indices
            .binary_search(&chapter_index)
            .is_ok()
    }

    pub fn set_chapter_selected(&mut self, chapter_index: usize, selected: bool) {
        match self.selected_chapter_indices.binary_search(&chapter_index) {
            Ok(position) if !selected => {
                self.selected_chapter_indices.remove(position);
            }
            Err(position) if selected => {
                self.selected_chapter_indices
                    .insert(position, chapter_index);
            }
            _ => {}
        }
    }

    pub fn grouped_selected_chapter_spans(&self, chapter_count: usize) -> Vec<(usize, usize)> {
        let mut spans = Vec::new();
        let mut current: Option<(usize, usize)> = None;

        for index in self
            .selected_chapter_indices
            .iter()
            .copied()
            .filter(|index| *index < chapter_count)
        {
            match current {
                Some((start, end)) if index == end + 1 => {
                    current = Some((start, index));
                }
                Some(span) => {
                    spans.push(span);
                    current = Some((index, index));
                }
                None => current = Some((index, index)),
            }
        }

        if let Some(span) = current {
            spans.push(span);
        }
        spans
    }

    pub fn output_count(&self, chapter_count: usize) -> usize {
        let selected_range_count = self.grouped_selected_chapter_spans(chapter_count).len()
            + self.custom_time_ranges.len();
        selected_range_count.max(1)
    }

    pub fn add_custom_time_range(&mut self, range: DownloadTimeRange) {
        if !self.custom_time_ranges.contains(&range) {
            self.custom_time_ranges.push(range);
            self.custom_time_ranges
                .sort_by_key(|range| range.start_millis());
        }
    }

    pub fn remove_custom_time_range(&mut self, range_index: usize) {
        if range_index < self.custom_time_ranges.len() {
            self.custom_time_ranges.remove(range_index);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DownloadTimeRange {
    start_millis: u64,
    end_millis: u64,
}

impl DownloadTimeRange {
    pub fn new(start_millis: u64, end_millis: u64) -> Option<Self> {
        (end_millis > start_millis).then_some(Self {
            start_millis,
            end_millis,
        })
    }

    pub fn start_millis(&self) -> u64 {
        self.start_millis
    }

    pub fn end_millis(&self) -> u64 {
        self.end_millis
    }

    pub fn duration_millis(&self) -> u64 {
        self.end_millis - self.start_millis
    }

    pub fn yt_dlp_argument(&self) -> String {
        format!(
            "*{}-{}",
            format_download_range_timestamp(self.start_millis),
            format_download_range_timestamp(self.end_millis)
        )
    }
}

pub fn format_download_range_timestamp(total_millis: u64) -> String {
    let millis = total_millis % 1000;
    let total_seconds = total_millis / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if millis == 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_chapters_are_sorted_and_grouped_into_contiguous_spans() {
        let mut selection = DownloadRangeSelection::default();
        selection.set_chapter_selected(5, true);
        selection.set_chapter_selected(1, true);
        selection.set_chapter_selected(3, true);
        selection.set_chapter_selected(4, true);

        assert_eq!(selection.selected_chapter_indices(), &[1, 3, 4, 5]);
        assert_eq!(
            selection.grouped_selected_chapter_spans(8),
            vec![(1, 1), (3, 5)]
        );
    }

    #[test]
    fn select_from_first_selected_to_end_keeps_one_contiguous_span() {
        let mut selection = DownloadRangeSelection::default();
        selection.set_chapter_selected(2, true);
        selection.select_chapters_from_first_selected_to_end(6);

        assert_eq!(selection.selected_chapter_indices(), &[2, 3, 4, 5]);
        assert_eq!(selection.grouped_selected_chapter_spans(6), vec![(2, 5)]);
    }

    #[test]
    fn time_range_rejects_empty_or_reversed_values_and_formats_milliseconds() {
        assert!(DownloadTimeRange::new(2_000, 2_000).is_none());
        assert!(DownloadTimeRange::new(3_000, 2_000).is_none());

        let range = DownloadTimeRange::new(75_250, 3_661_000).expect("valid range");
        assert_eq!(range.yt_dlp_argument(), "*00:01:15.250-01:01:01");
    }

    #[test]
    fn duplicate_custom_ranges_are_not_added_twice() {
        let mut selection = DownloadRangeSelection::default();
        let range = DownloadTimeRange::new(10_000, 20_000).expect("valid range");

        selection.add_custom_time_range(range);
        selection.add_custom_time_range(range);

        assert_eq!(selection.custom_time_ranges(), &[range]);
    }

    #[test]
    fn output_count_treats_full_video_as_one_output() {
        let mut selection = DownloadRangeSelection::default();
        assert_eq!(selection.output_count(5), 1);

        selection.set_chapter_selected(1, true);
        selection.set_chapter_selected(2, true);
        selection.set_chapter_selected(4, true);
        assert_eq!(selection.output_count(5), 2);
    }
}
