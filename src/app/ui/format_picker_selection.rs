use crate::app::state::{AppState, FormatPickerKind, FormatPickerViewMode};
use crate::domain::{FormatOption, SubtitleSource};

use super::common::UiText;
use super::{format_picker_filters, format_picker_subtitle};

pub(super) fn pending_selection_summary(state: &AppState) -> Option<String> {
    let kind = state.format_picker.kind?;
    if kind == FormatPickerKind::Section {
        return (!state.pending_download_range_is_empty())
            .then(|| state.pending_download_range_summary());
    }
    if !matches!(kind, FormatPickerKind::Video | FormatPickerKind::Audio) {
        return None;
    }

    let size_label = state.ui_i18n_text_for_key(UiText::HEADER_FILESIZE);
    let filesize = pending_selected_format(state)
        .map(|option| option.filesize.trim().to_owned())
        .filter(|filesize| !filesize.is_empty())
        .unwrap_or_else(|| "—".to_owned());
    Some(format!("{size_label} {filesize}"))
}

pub(super) fn pending_selection_id(state: &AppState) -> Option<String> {
    let kind = state.format_picker.kind?;
    if kind == FormatPickerKind::Subtitle {
        if state.format_picker.subtitle_source_key == SubtitleSource::None.key() {
            return Some(String::new());
        }
        let options = format_picker_subtitle::subtitle_pending_options(state);
        return state
            .format_picker
            .selected_row
            .and_then(|row| options.get(row))
            .map(|option| option.id.clone());
    }
    if kind == FormatPickerKind::Section {
        return Some(String::new());
    }

    pending_selected_format(state).map(|option| option.id.clone())
}

fn pending_selected_format(state: &AppState) -> Option<FormatOption> {
    let kind = state.format_picker.kind?;
    if !matches!(kind, FormatPickerKind::Video | FormatPickerKind::Audio) {
        return None;
    }

    let options = state.format_picker_options(kind);
    match state.format_picker.view_mode {
        FormatPickerViewMode::Filter => pending_filtered_format(state, &options),
        FormatPickerViewMode::Table => state
            .format_picker
            .selected_row
            .and_then(|row| options.get(row))
            .cloned(),
    }
}

fn pending_filtered_format(state: &AppState, options: &[FormatOption]) -> Option<FormatOption> {
    let visible_rows = format_picker_filters::filtered_rows(options, &state.format_picker.filters);
    if visible_rows.len() == 1 {
        return visible_rows
            .first()
            .and_then(|&index| options.get(index))
            .cloned();
    }

    state
        .format_picker
        .selected_row
        .filter(|row| visible_rows.iter().any(|index| index == row))
        .and_then(|row| options.get(row))
        .cloned()
}
