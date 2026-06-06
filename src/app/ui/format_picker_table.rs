use crate::app::state::{AppState, FormatPickerKind};
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::domain::{FormatOption, MediaKind};
use eframe::egui::{Align, Layout, ScrollArea, Sense, Ui};
use egui_extras::{Column, TableBuilder};

use super::common::{UiText, cell_label, cell_label_center, cell_label_right};
use super::format_picker_filters::apply_filters_from_option;
use super::semantic_ui_metrics;

pub(super) fn render_format_picker_table(
    ui: &mut Ui,
    state: &mut AppState,
    kind: FormatPickerKind,
    options: &[FormatOption],
) {
    let row_height = semantic_ui_metrics::format_picker_section_row_height();
    let max_height = ui
        .available_height()
        .max(semantic_ui_metrics::format_picker_section_table_minimum_body_height());
    let table_widths = measure_format_picker_table_widths(ui, state, kind, options);
    let exact_width = table_widths.exact_width_for(kind);
    let available_width = ui.available_width();
    let needs_horizontal_scroll = exact_width > available_width + 2.0;

    ui.set_height(max_height);
    if needs_horizontal_scroll {
        ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(exact_width);
                render_format_picker_table_inner(
                    ui,
                    state,
                    kind,
                    options,
                    table_widths,
                    row_height,
                    max_height,
                );
            });
    } else {
        render_format_picker_table_inner(
            ui,
            state,
            kind,
            options,
            table_widths,
            row_height,
            max_height,
        );
    }
}

fn render_format_picker_table_inner(
    ui: &mut Ui,
    state: &mut AppState,
    kind: FormatPickerKind,
    options: &[FormatOption],
    table_widths: FormatPickerTableWidths,
    row_height: f32,
    max_height: f32,
) {
    let table = match kind {
        FormatPickerKind::Video => TableBuilder::new(ui)
            .id_salt("video-format-picker-table")
            .striped(true)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::exact(table_widths.marker))
            .column(Column::exact(table_widths.resolution))
            .column(Column::exact(table_widths.dynamic_range))
            .column(Column::exact(table_widths.fps))
            .column(Column::exact(table_widths.codec))
            .column(Column::exact(table_widths.filesize))
            .column(Column::remainder().at_least(
                semantic_ui_metrics::format_picker_table_remainder_column_minimum_width(),
            ))
            .min_scrolled_height(semantic_ui_metrics::format_picker_table_minimum_scrolled_height())
            .max_scroll_height(max_height)
            .header(row_height, |mut header| {
                let resolution_text = state.ui_i18n_text_for_key(UiText::HEADER_RESOLUTION);
                let range_text = state.ui_i18n_text_for_key(UiText::HEADER_RANGE);
                let fps_text = state.ui_i18n_text_for_key(UiText::HEADER_FPS);
                let codec_text = state.ui_i18n_text_for_key(UiText::HEADER_CODEC);
                let filesize_text = state.ui_i18n_text_for_key(UiText::HEADER_FILESIZE);
                header.col(|ui| cell_label_center(ui, ""));
                header.col(|ui| cell_label_center(ui, resolution_text));
                header.col(|ui| cell_label_center(ui, range_text));
                header.col(|ui| cell_label_center(ui, fps_text));
                header.col(|ui| cell_label_center(ui, codec_text));
                header.col(|ui| cell_label_center(ui, filesize_text));
                header.col(|_| {});
            }),
        FormatPickerKind::Audio => TableBuilder::new(ui)
            .id_salt("audio-format-picker-table")
            .striped(true)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::exact(table_widths.marker))
            .column(Column::exact(table_widths.sample_rate))
            .column(Column::exact(table_widths.codec))
            .column(Column::exact(table_widths.filesize))
            .column(Column::remainder().at_least(
                semantic_ui_metrics::format_picker_table_remainder_column_minimum_width(),
            ))
            .min_scrolled_height(semantic_ui_metrics::format_picker_table_minimum_scrolled_height())
            .max_scroll_height(max_height)
            .header(row_height, |mut header| {
                let sample_rate_text = state.ui_i18n_text_for_key(UiText::HEADER_SAMPLE_RATE);
                let codec_text = state.ui_i18n_text_for_key(UiText::HEADER_CODEC);
                let filesize_text = state.ui_i18n_text_for_key(UiText::HEADER_FILESIZE);
                header.col(|ui| cell_label_center(ui, ""));
                header.col(|ui| cell_label_center(ui, sample_rate_text));
                header.col(|ui| cell_label_center(ui, codec_text));
                header.col(|ui| cell_label_center(ui, filesize_text));
                header.col(|_| {});
            }),
        FormatPickerKind::Subtitle | FormatPickerKind::Section => unreachable!(),
    };

    table.body(|body| {
        body.rows(row_height, options.len(), |mut row| {
            let option_index = row.index();
            let option = &options[option_index];
            let is_selected = state.format_picker.selected_row == Some(option_index);
            row.set_selected(is_selected);

            match kind {
                FormatPickerKind::Video => {
                    row.col(|ui| render_muxed_marker(ui, option.kind == MediaKind::Muxed));
                    row.col(|ui| cell_label_center(ui, &option.resolution));
                    row.col(|ui| cell_label_center(ui, &option.dynamic_range));
                    row.col(|ui| cell_label_center(ui, &option.fps));
                    row.col(|ui| cell_label_center(ui, &option.codec));
                    row.col(|ui| cell_label_right(ui, &option.filesize));
                    row.col(|_| {});
                }
                FormatPickerKind::Audio => {
                    row.col(|ui| render_muxed_marker(ui, option.kind == MediaKind::Muxed));
                    row.col(|ui| cell_label(ui, &option.sample_rate));
                    row.col(|ui| cell_label(ui, &option.codec));
                    row.col(|ui| cell_label(ui, &option.filesize));
                    row.col(|_| {});
                }
                FormatPickerKind::Subtitle | FormatPickerKind::Section => unreachable!(),
            }

            let response = row.response();
            if response.clicked() {
                state.format_picker.selected_row = Some(option_index);
                apply_filters_from_option(&mut state.format_picker.filters, kind, option);
            }
            if response.double_clicked() {
                state.confirm_format_picker_selection(&option.id);
            }
        });
    });
}

fn render_muxed_marker(ui: &mut Ui, visible: bool) {
    if !visible {
        return;
    }
    let icon_size = semantic_ui_metrics::format_picker_muxed_marker_icon_size();
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.add(icon_image(
            AppIcon::LinkVariant,
            icon_size,
            standard_icon_color(ui).linear_multiply(0.72),
        ));
    });
}

#[derive(Clone, Copy)]
struct FormatPickerTableWidths {
    marker: f32,
    resolution: f32,
    dynamic_range: f32,
    fps: f32,
    codec: f32,
    filesize: f32,
    sample_rate: f32,
}

impl FormatPickerTableWidths {
    fn exact_width_for(self, kind: FormatPickerKind) -> f32 {
        match kind {
            FormatPickerKind::Video => {
                self.marker
                    + self.resolution
                    + self.dynamic_range
                    + self.fps
                    + self.codec
                    + self.filesize
            }
            FormatPickerKind::Audio => self.marker + self.sample_rate + self.codec + self.filesize,
            FormatPickerKind::Subtitle | FormatPickerKind::Section => 0.0,
        }
    }
}

fn measure_format_picker_table_widths(
    ui: &Ui,
    state: &AppState,
    kind: FormatPickerKind,
    options: &[FormatOption],
) -> FormatPickerTableWidths {
    let marker = semantic_ui_metrics::format_picker_table_marker_column_width();
    let resolution_text = state.ui_i18n_text_for_key(UiText::HEADER_RESOLUTION);
    let range_text = state.ui_i18n_text_for_key(UiText::HEADER_RANGE);
    let fps_text = state.ui_i18n_text_for_key(UiText::HEADER_FPS);
    let sample_rate_text = state.ui_i18n_text_for_key(UiText::HEADER_SAMPLE_RATE);
    let codec_text = state.ui_i18n_text_for_key(UiText::HEADER_CODEC);
    let filesize_text = state.ui_i18n_text_for_key(UiText::HEADER_FILESIZE);
    let resolution =
        semantic_ui_metrics::format_picker_table_resolution_column_width_for_header_and_values(
            ui,
            resolution_text,
            options.iter().map(|option| option.resolution.as_str()),
        );
    let dynamic_range =
        semantic_ui_metrics::format_picker_table_dynamic_range_column_width_for_header_and_values(
            ui,
            range_text,
            options.iter().map(|option| option.dynamic_range.as_str()),
        );
    let fps = semantic_ui_metrics::format_picker_table_fps_column_width_for_header_and_values(
        ui,
        fps_text,
        options.iter().map(|option| option.fps.as_str()),
    );
    let sample_rate =
        semantic_ui_metrics::format_picker_table_sample_rate_column_width_for_header_and_values(
            ui,
            sample_rate_text,
            options.iter().map(|option| option.sample_rate.as_str()),
        );
    let codec = match kind {
        FormatPickerKind::Audio => {
            semantic_ui_metrics::format_picker_table_audio_codec_column_width_for_header_and_values(
                ui,
                codec_text,
                options.iter().map(|option| option.codec.as_str()),
            )
        }
        _ => {
            semantic_ui_metrics::format_picker_table_video_codec_column_width_for_header_and_values(
                ui,
                codec_text,
                options.iter().map(|option| option.codec.as_str()),
            )
        }
    };
    let filesize =
        semantic_ui_metrics::format_picker_table_filesize_column_width_for_header_and_values(
            ui,
            filesize_text,
            options.iter().map(|option| option.filesize.as_str()),
        );

    FormatPickerTableWidths {
        marker,
        resolution,
        dynamic_range,
        fps,
        codec,
        filesize,
        sample_rate,
    }
}
