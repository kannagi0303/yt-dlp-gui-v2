use crate::app::state::AppState;
use eframe::egui::{self, Align, Layout, ScrollArea, Sense, Ui};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

use super::common::{UiText, cell_label_center};
use super::semantic_ui_metrics;

pub(super) fn render_section_picker_contents(ui: &mut Ui, state: &mut AppState) {
    let options = state.download_section_picker_options();
    if options.is_empty() {
        state.format_picker.selected_row = None;
        ui.add_space(semantic_ui_metrics::format_picker_empty_message_top_vertical_spacing());
        ui.label(state.ui_i18n_text_for_key("picker.no_chapters_available"));
        return;
    }

    if state
        .format_picker
        .selected_row
        .map_or(true, |row| row >= options.len())
    {
        state.format_picker.selected_row = Some(0);
    }

    let description_height =
        semantic_ui_metrics::format_picker_section_description_height_for_option_count_and_compatibility_mode(
            options.len(),
            state.tool_paths.chapter_compatibility_mode,
        );
    let row_height = semantic_ui_metrics::format_picker_section_row_height();

    StripBuilder::new(ui)
        .size(Size::exact(description_height))
        .size(Size::remainder().at_least(
            semantic_ui_metrics::format_picker_section_table_minimum_body_height(),
        ))
        .vertical(|mut strip| {
            strip.cell(|ui| {
                ui.add(
                    egui::Label::new(
                        state.ui_i18n_text_for_key("picker.choose_the_range_to_download_for_this_item_d"),
                    )
                    .selectable(false),
                );
                if state.tool_paths.chapter_compatibility_mode && options.len() > 1 {
                    ui.add(
                        egui::Label::new(
                            state.ui_i18n_text_for_key("picker.chapter_compatibility_mode_is_on_chapter_dow"),
                        )
                        .selectable(false),
                    );
                }
            });
            strip.cell(|ui| {
                ui.separator();
                let max_height = ui.available_height().max(
                    semantic_ui_metrics::format_picker_section_table_minimum_body_height(),
                );
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(max_height)
                    .show(ui, |ui| {
                        TableBuilder::new(ui)
                            .id_salt("section-picker-table")
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
                            .header(row_height, |mut header| {
                                header.col(|ui| cell_label_center(ui, ""));
                                header.col(|ui| {
                                    cell_label_center(ui, state.ui_i18n_text_for_key(UiText::HEADER_RANGE))
                                });
                            })
                            .body(|body| {
                                body.rows(row_height, options.len(), |mut row| {
                                    let option_index = row.index();
                                    let (value, label) = &options[option_index];
                                    let is_selected =
                                        state.format_picker.selected_row == Some(option_index);
                                    row.set_selected(is_selected);

                                    row.col(|ui| {
                                        cell_label_center(ui, if is_selected { "✓" } else { "" })
                                    });
                                    row.col(|ui| {
                                        ui.add(
                                            egui::Label::new(label.as_str())
                                                .truncate()
                                                .selectable(false)
                                                .sense(Sense::empty()),
                                        );
                                    });

                                    let response = row.response();
                                    if response.clicked() {
                                        state.format_picker.selected_row = Some(option_index);
                                    }
                                    if response.double_clicked() {
                                        state.confirm_format_picker_selection(value);
                                    }
                                });
                            });
                    });
            });
        });
}
