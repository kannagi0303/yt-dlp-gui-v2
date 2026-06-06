use egui_taffy::Tui;

use crate::app::state::{AppState, FormatPickerKind};
use crate::infrastructure::DownloadTargetKind;

use super::common::UiText;
use super::item_card;

pub(super) fn single_mode_format_row_count(state: &AppState) -> usize {
    let Some(_) = state.queue_items.first() else {
        // Match normal mode empty-state behavior: show only video/audio rows
        // until a loaded item actually exposes subtitle choices.
        return 2;
    };

    let index = 0;
    let mut rows = 2usize; // video + audio, matching the normal item baseline.
    if state.item_shows_subtitle_row(index) {
        rows += 1;
    }
    if state.item_shows_download_section_row(index) {
        rows += 1;
    }
    rows
}

pub(super) fn render_format_rows(tui: &mut Tui, state: &mut AppState) {
    if state.queue_items.is_empty() {
        render_empty_format_rows(tui, state);
        return;
    }

    let index = 0usize;
    let item_id = state.queue_items[index].id;
    let item_locked = state.item_is_busy(index);
    let audio_locked = state.item_uses_muxed_video(index);
    let show_subtitle_row = state.item_shows_subtitle_row(index);
    let show_section_row = state.item_shows_download_section_row(index);
    let label_width = {
        let ui = tui.egui_ui();
        item_card::visible_item_label_width(ui, state, false, show_subtitle_row, show_section_row)
    };

    let video_label = state.ui_i18n_text_for_key(UiText::VIDEO).to_owned();
    let audio_label = state.ui_i18n_text_for_key(UiText::AUDIO).to_owned();
    let subtitle_label = state.ui_i18n_text_for_key(UiText::SUBTITLE).to_owned();
    let section_label = state.ui_i18n_text_for_key(UiText::SECTION).to_owned();
    let video_summary = state
        .localize_message(&state.selected_format_summary(index, FormatPickerKind::Video))
        .to_string();
    let audio_summary = state
        .localize_message(&state.selected_format_summary(index, FormatPickerKind::Audio))
        .to_string();
    let subtitle_summary = state
        .localize_message(&state.selected_format_summary(index, FormatPickerKind::Subtitle))
        .to_string();
    let section_summary = state
        .localize_message(&state.selected_download_section_summary(index))
        .to_string();
    let video_progress = state.item_progress(index, FormatPickerKind::Video);
    let audio_progress = state.item_progress(index, FormatPickerKind::Audio);
    let subtitle_progress = state.item_progress(index, FormatPickerKind::Subtitle);
    let show_av_progress = state.item_av_progress_visible(index);
    let show_subtitle_progress = state.item_subtitle_progress_visible(index);
    let video_export_enabled = state.item_can_export(index, DownloadTargetKind::Video);
    let audio_export_enabled = state.item_can_export(index, DownloadTargetKind::Audio);
    let subtitle_export_enabled = state.item_can_export(index, DownloadTargetKind::Subtitle);
    let mut pending_export = None;

    item_card::item_format_summary_row(
        tui,
        label_width,
        &video_label,
        &video_summary,
        video_progress,
        show_av_progress,
        !item_locked,
        video_export_enabled,
        || state.open_format_picker(index, FormatPickerKind::Video),
        || pending_export = Some((item_id, DownloadTargetKind::Video)),
    );
    item_card::item_format_summary_row(
        tui,
        label_width,
        &audio_label,
        &audio_summary,
        audio_progress,
        show_av_progress,
        !audio_locked && !item_locked,
        audio_export_enabled,
        || state.open_format_picker(index, FormatPickerKind::Audio),
        || pending_export = Some((item_id, DownloadTargetKind::Audio)),
    );
    if show_subtitle_row {
        item_card::item_format_summary_row(
            tui,
            label_width,
            &subtitle_label,
            &subtitle_summary,
            subtitle_progress,
            show_subtitle_progress,
            !item_locked,
            subtitle_export_enabled,
            || state.open_format_picker(index, FormatPickerKind::Subtitle),
            || pending_export = Some((item_id, DownloadTargetKind::Subtitle)),
        );
    }
    if show_section_row {
        item_card::item_download_section_summary_row(
            tui,
            label_width,
            &section_label,
            &section_summary,
            !item_locked,
            || state.open_format_picker(index, FormatPickerKind::Section),
        );
    }

    if let Some((item_id, kind)) = pending_export {
        item_card::open_export_dialog(state, item_id, kind);
    }
}

fn render_empty_format_rows(tui: &mut Tui, state: &mut AppState) {
    let label_width = {
        let ui = tui.egui_ui();
        item_card::visible_item_label_width(ui, state, false, false, false)
    };
    let video_waiting = state
        .ui_i18n_text_for_key("item.after_adding_choose_the_video_format_here")
        .to_owned();
    let audio_waiting = state
        .ui_i18n_text_for_key("item.after_adding_choose_the_audio_format_here")
        .to_owned();
    let video_label = state.ui_i18n_text_for_key(UiText::VIDEO).to_owned();
    let audio_label = state.ui_i18n_text_for_key(UiText::AUDIO).to_owned();

    item_card::item_format_summary_row(
        tui,
        label_width,
        &video_label,
        &video_waiting,
        0.0,
        false,
        false,
        false,
        || {},
        || {},
    );
    item_card::item_format_summary_row(
        tui,
        label_width,
        &audio_label,
        &audio_waiting,
        0.0,
        false,
        false,
        false,
        || {},
        || {},
    );
}
