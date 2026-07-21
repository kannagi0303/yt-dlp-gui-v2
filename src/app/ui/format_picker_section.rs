use crate::app::state::{AppState, SectionPickerTab};
use eframe::egui::{Align, Layout, ScrollArea, Sense, Ui};
use egui_extras::{Column, TableBuilder};

use super::{format_picker_time_range, semantic_ui_metrics};

pub(super) fn render_section_picker_contents(ui: &mut Ui, state: &mut AppState) {
    match state.format_picker.section_tab {
        SectionPickerTab::Chapters => render_chapter_picker_contents(ui, state),
        SectionPickerTab::TimeRange => {
            format_picker_time_range::render_time_range_picker_contents(ui, state)
        }
    }
}

fn render_chapter_picker_contents(ui: &mut Ui, state: &mut AppState) {
    let chapters = state.current_download_range_chapters();
    if chapters.is_empty() {
        ui.add_space(semantic_ui_metrics::format_picker_empty_message_top_vertical_spacing());
        ui.label(state.ui_i18n_text_for_key("picker.no_chapters_available"));
        return;
    }

    let row_height = semantic_ui_metrics::format_picker_section_row_height();
    let max_height = ui
        .available_height()
        .max(semantic_ui_metrics::format_picker_section_table_minimum_body_height());
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(max_height)
        .show(ui, |ui| {
            TableBuilder::new(ui)
                .id_salt("download-range-chapter-picker-table")
                .striped(true)
                .sense(Sense::click())
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::exact(
                    semantic_ui_metrics::format_picker_section_table_marker_column_width(),
                ))
                .column(Column::remainder().at_least(
                    semantic_ui_metrics::format_picker_section_table_range_column_minimum_width(),
                ))
                .min_scrolled_height(
                    semantic_ui_metrics::format_picker_section_table_minimum_body_height(),
                )
                .max_scroll_height(max_height)
                .body(|body| {
                    body.rows(row_height, chapters.len(), |mut row| {
                        let chapter_index = row.index();
                        let chapter = &chapters[chapter_index];
                        let mut is_selected = state
                            .format_picker
                            .download_range_draft
                            .selection
                            .chapter_is_selected(chapter_index);
                        let mut checkbox_changed = false;
                        row.set_selected(is_selected);

                        row.col(|ui| {
                            if ui.checkbox(&mut is_selected, "").changed() {
                                checkbox_changed = true;
                                state.set_pending_download_range_chapter_selected(
                                    chapter_index,
                                    is_selected,
                                );
                            }
                        });
                        row.col(|ui| {
                            ui.add(
                                eframe::egui::Label::new(state.localized_chapter_label(chapter))
                                    .truncate()
                                    .selectable(false)
                                    .sense(Sense::empty()),
                            );
                        });

                        if row.response().clicked() && !checkbox_changed {
                            state.set_pending_download_range_chapter_selected(
                                chapter_index,
                                !is_selected,
                            );
                        }
                    });
                });
        });
}
