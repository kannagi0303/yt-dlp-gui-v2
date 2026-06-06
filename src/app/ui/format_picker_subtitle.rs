use crate::app::state::{AppState, SubtitlePickerTab};
use crate::domain::SubtitleSource;
use eframe::egui::{Align, Layout, ScrollArea, Sense, Ui};
use egui_extras::{Column, TableBuilder};

use super::common::{UiText, cell_label, cell_label_center};
use super::semantic_ui_metrics;

pub(super) fn render_subtitle_picker_contents(ui: &mut Ui, state: &mut AppState) {
    let source_options = state.subtitle_source_options();
    if state.format_picker.subtitle_source_key.is_empty() {
        state.format_picker.subtitle_source_key = SubtitleSource::None.key().to_owned();
    }

    let active_page = state.format_picker.subtitle_tab;

    if active_page == SubtitlePickerTab::None {
        state.format_picker.subtitle_source_key = SubtitleSource::None.key().to_owned();
        state.format_picker.selected_row = Some(0);
        ui.label(state.ui_i18n_text_for_key("picker.subtitles_will_not_be_downloaded"));
        return;
    }

    if source_options.is_empty() {
        ui.label(state.ui_i18n_text_for_key("picker.no_subtitles_are_available_for_this_video"));
        state.format_picker.selected_row = None;
        return;
    }

    let page_sources: Vec<_> = source_options
        .iter()
        .filter(|option| {
            matches!(
                (active_page, option.source),
                (SubtitlePickerTab::Original, SubtitleSource::Original)
                    | (SubtitlePickerTab::Automatic, SubtitleSource::Automatic)
            )
        })
        .cloned()
        .collect();

    if page_sources.is_empty() {
        ui.label(state.ui_i18n_text_for_key("picker.no_subtitles_are_available_in_this_tab"));
        state.format_picker.selected_row = None;
        return;
    }

    if state.format_picker.subtitle_source_key == SubtitleSource::None.key() {
        if let Some(first) = page_sources.first() {
            state.format_picker.subtitle_source_key = first.source_key();
            state.format_picker.selected_row = None;
        }
    }

    if !page_sources
        .iter()
        .any(|option| option.source_key() == state.format_picker.subtitle_source_key)
    {
        if let Some(first) = page_sources.first() {
            state.format_picker.subtitle_source_key = first.source_key();
            state.format_picker.selected_row = None;
        }
    }

    if active_page == SubtitlePickerTab::Original {
        render_original_subtitle_picker(ui, state, &page_sources);
        return;
    }

    ui.label(state.ui_i18n_text_for_key("picker.source_language"));
    ui.horizontal_wrapped(|ui| {
        for option in &page_sources {
            let source_key = option.source_key();
            let selected = state.format_picker.subtitle_source_key == source_key;
            let source_lang_label = format!(
                "{} ({})",
                option.source_language_label, option.source_language_code
            );
            if ui.selectable_label(selected, source_lang_label).clicked() {
                state.format_picker.subtitle_source_key = source_key;
                state.format_picker.selected_row = None;
            }
        }
    });
    ui.separator();
    ui.label(state.ui_i18n_text_for_key("picker.translation_target"));
    ui.label(state.ui_i18n_text_for_key("picker.tip_youtube_auto_translated_subtitles_are_mo"));

    let options = state.subtitle_translation_options();
    if options.is_empty() {
        ui.label(state.ui_i18n_text_for_key("picker.no_subtitles_are_available_for_this_source"));
        state.format_picker.selected_row = None;
        return;
    }

    if let Some(row) = state.format_picker.selected_row {
        if row >= options.len() {
            state.format_picker.selected_row = None;
        }
    }

    ScrollArea::vertical().show(ui, |ui| {
        let table = TableBuilder::new(ui)
            .id_salt("subtitle-picker-table")
            .striped(true)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::remainder().at_least(
                semantic_ui_metrics::format_picker_subtitle_target_column_minimum_width(),
            ))
            .column(Column::auto().at_least(
                semantic_ui_metrics::format_picker_subtitle_extension_column_minimum_width(),
            ))
            .header(
                semantic_ui_metrics::format_picker_subtitle_table_row_height(),
                |mut header| {
                    header.col(|ui| {
                        cell_label_center(ui, state.ui_i18n_text_for_key("picker.target"));
                    });
                    header.col(|ui| {
                        cell_label_center(ui, state.ui_i18n_text_for_key(UiText::HEADER_EXT));
                    });
                },
            );

        table.body(|body| {
            body.rows(
                semantic_ui_metrics::format_picker_subtitle_table_row_height(),
                options.len(),
                |mut row| {
                    let option_index = row.index();
                    let option = &options[option_index];
                    let is_selected = state.format_picker.selected_row == Some(option_index);
                    row.set_selected(is_selected);

                    row.col(|ui| {
                        cell_label(ui, &state.localized_subtitle_target_label(option));
                    });
                    row.col(|ui| {
                        cell_label_center(ui, &option.ext);
                    });

                    let response = row.response();
                    if response.clicked() {
                        state.format_picker.selected_row = Some(option_index);
                    }
                    if response.double_clicked() {
                        state.confirm_format_picker_selection(&option.id);
                    }
                },
            );
        });
    });
}

pub(super) fn subtitle_pending_options(state: &AppState) -> Vec<crate::domain::SubtitleOption> {
    match state.format_picker.subtitle_tab {
        SubtitlePickerTab::None => Vec::new(),
        SubtitlePickerTab::Original => state
            .subtitle_source_options()
            .into_iter()
            .filter(|track| track.source == SubtitleSource::Original)
            .collect(),
        SubtitlePickerTab::Automatic => state.subtitle_translation_options(),
    }
}

fn render_original_subtitle_picker(
    ui: &mut Ui,
    state: &mut AppState,
    page_sources: &[crate::domain::SubtitleOption],
) {
    ui.label(state.ui_i18n_text_for_key("picker.available_subtitles"));

    if let Some(row) = state.format_picker.selected_row {
        if row >= page_sources.len() {
            state.format_picker.selected_row = None;
        }
    }

    ScrollArea::vertical().show(ui, |ui| {
        let table = TableBuilder::new(ui)
            .id_salt("original-subtitle-picker-table")
            .striped(true)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::remainder().at_least(
                semantic_ui_metrics::format_picker_subtitle_target_column_minimum_width(),
            ))
            .column(Column::auto().at_least(
                semantic_ui_metrics::format_picker_subtitle_extension_column_minimum_width(),
            ))
            .header(
                semantic_ui_metrics::format_picker_subtitle_table_row_height(),
                |mut header| {
                    header.col(|ui| {
                        cell_label_center(ui, state.ui_i18n_text_for_key("picker.language"));
                    });
                    header.col(|ui| {
                        cell_label_center(ui, state.ui_i18n_text_for_key(UiText::HEADER_EXT));
                    });
                },
            );

        table.body(|body| {
            body.rows(
                semantic_ui_metrics::format_picker_subtitle_table_row_height(),
                page_sources.len(),
                |mut row| {
                    let option_index = row.index();
                    let option = &page_sources[option_index];
                    let is_selected = state.format_picker.selected_row == Some(option_index);
                    row.set_selected(is_selected);

                    row.col(|ui| {
                        let label = format!(
                            "{} ({})",
                            option.source_language_label, option.source_language_code
                        );
                        cell_label(ui, &label);
                    });
                    row.col(|ui| {
                        cell_label_center(ui, &option.ext);
                    });

                    let response = row.response();
                    if response.clicked() {
                        state.format_picker.selected_row = Some(option_index);
                    }
                    if response.double_clicked() {
                        state.confirm_format_picker_selection(&option.id);
                    }
                },
            );
        });
    });
}
