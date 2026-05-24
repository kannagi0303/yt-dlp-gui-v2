use eframe::egui::{self, Align, Layout, ScrollArea, Sense, Ui};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

use crate::app::state::{
    AppState, FormatPickerFilters, FormatPickerKind, FormatPickerViewMode, SubtitlePickerTab,
};
use crate::domain::{FormatOption, MediaKind, SubtitleSource};

use super::common::{
    UiText, cell_label, cell_label_center, cell_label_right, natural_button_width,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterField {
    Resolution,
    DynamicRange,
    Fps,
    Codec,
    SampleRate,
}

pub(super) fn render_format_picker_screen(ui: &mut Ui, state: &mut AppState) {
    let pending_selection = pending_picker_selection_id(state);
    let header_height = ui.spacing().interact_size.y + ui.spacing().item_spacing.y + 20.0;
    let back_width = natural_button_width(ui, state.tr(UiText::BACK_TO_MAIN));
    let confirm_width = natural_button_width(ui, state.tr(UiText::CONFIRM));

    StripBuilder::new(ui)
        .size(Size::exact(header_height))
        .size(Size::remainder().at_least(0.0))
        .vertical(|mut strip| {
            strip.cell(|ui| {
                StripBuilder::new(ui)
                    .size(Size::initial(back_width))
                    .size(Size::remainder().at_least(0.0))
                    .size(Size::initial(confirm_width))
                    .horizontal(|mut strip| {
                        strip.cell(|ui| {
                            if ui.button(state.tr(UiText::BACK_TO_MAIN)).clicked() {
                                state.cancel_format_picker();
                            }
                        });
                        strip.cell(|ui| {
                            ui.vertical(|ui| {
                                let title = match state.format_picker.kind {
                                    Some(FormatPickerKind::Video) => UiText::SELECT_VIDEO_TITLE,
                                    Some(FormatPickerKind::Audio) => UiText::SELECT_AUDIO_TITLE,
                                    Some(FormatPickerKind::Subtitle) => {
                                        UiText::SELECT_SUBTITLE_TITLE
                                    }
                                    Some(FormatPickerKind::Section) => UiText::SELECT_SECTION_TITLE,
                                    None => "",
                                };
                                ui.label(state.tr(title));

                                let item_title =
                                    state.format_picker_target_title().unwrap_or_default();
                                let response = ui.add(egui::Label::new(item_title).truncate());
                                if !item_title.is_empty() {
                                    response.on_hover_text(item_title);
                                }
                            });
                        });
                        strip.cell(|ui| {
                            if ui
                                .add_enabled(
                                    pending_selection.is_some(),
                                    egui::Button::new(state.tr(UiText::CONFIRM)),
                                )
                                .clicked()
                            {
                                if let Some(format_id) = pending_selection.as_deref() {
                                    state.confirm_format_picker_selection(format_id);
                                }
                            }
                        });
                    });
            });
            strip.cell(|ui| {
                ui.separator();
                render_format_picker_contents(ui, state);
            });
        });
}

fn render_format_picker_contents(ui: &mut Ui, state: &mut AppState) {
    let Some(kind) = state.format_picker.kind else {
        state.cancel_format_picker();
        return;
    };

    if kind == FormatPickerKind::Subtitle {
        render_subtitle_picker_contents(ui, state);
        return;
    }

    if kind == FormatPickerKind::Section {
        render_section_picker_contents(ui, state);
        return;
    }

    let options = state.format_picker_options(kind);
    let filtered_rows = filtered_rows(&options, &state.format_picker.filters);
    let previous_mode = state.format_picker.view_mode;

    if let Some(selected_row) = state.format_picker.selected_row {
        if selected_row >= options.len() {
            state.format_picker.selected_row = None;
        }
    }

    if state.format_picker.view_mode == FormatPickerViewMode::Filter && filtered_rows.len() == 1 {
        state.format_picker.selected_row = filtered_rows.first().copied();
    }

    let tab_height = ui.spacing().interact_size.y;
    let picker_mode_filter = state.tr(UiText::PICKER_MODE_FILTER);
    let picker_mode_table = state.tr(UiText::PICKER_MODE_TABLE);

    StripBuilder::new(ui)
        .size(Size::exact(tab_height))
        .size(Size::remainder().at_least(0.0))
        .vertical(|mut strip| {
            strip.cell(|ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut state.format_picker.view_mode,
                        FormatPickerViewMode::Filter,
                        picker_mode_filter,
                    );
                    ui.selectable_value(
                        &mut state.format_picker.view_mode,
                        FormatPickerViewMode::Table,
                        picker_mode_table,
                    );
                    let _sort_state = state.format_picker.sort_state;
                });
            });
            strip.cell(|ui| {
                ui.separator();
                match state.format_picker.view_mode {
                    FormatPickerViewMode::Filter => {
                        if filtered_rows.is_empty() {
                            ui.add_space(12.0);
                            ui.label(state.tr(UiText::EMPTY_TABLE));
                        }
                        render_format_picker_filters(ui, state, kind, &options);
                    }
                    FormatPickerViewMode::Table => {
                        if options.is_empty() {
                            ui.add_space(12.0);
                            ui.label(state.tr(UiText::EMPTY_TABLE));
                        } else {
                            render_format_picker_table(ui, state, kind, &options);
                        }
                    }
                }
            });
        });

    if state.format_picker.view_mode != previous_mode {
        sync_picker_mode(state);
    }
}

fn pending_picker_selection_id(state: &AppState) -> Option<String> {
    let kind = state.format_picker.kind?;
    if kind == FormatPickerKind::Subtitle {
        if state.format_picker.subtitle_source_key == SubtitleSource::None.key() {
            return Some(String::new());
        }
        let options = subtitle_pending_options(state);
        return state
            .format_picker
            .selected_row
            .and_then(|row| options.get(row))
            .map(|option| option.id.clone());
    }
    if kind == FormatPickerKind::Section {
        let options = state.download_section_picker_options();
        return state
            .format_picker
            .selected_row
            .and_then(|row| options.get(row))
            .map(|(value, _label)| value.clone());
    }
    let options = state.format_picker_options(kind);
    match state.format_picker.view_mode {
        FormatPickerViewMode::Filter if kind == FormatPickerKind::Subtitle => None,
        FormatPickerViewMode::Filter => {
            let visible_rows = filtered_rows(&options, &state.format_picker.filters);
            if visible_rows.len() == 1 {
                return visible_rows
                    .first()
                    .and_then(|&index| options.get(index))
                    .map(|option| option.id.clone());
            }

            state
                .format_picker
                .selected_row
                .filter(|row| visible_rows.iter().any(|index| index == row))
                .and_then(|row| options.get(row))
                .map(|option| option.id.clone())
        }
        FormatPickerViewMode::Table => state
            .format_picker
            .selected_row
            .and_then(|row| options.get(row))
            .map(|option| option.id.clone()),
    }
}

fn render_section_picker_contents(ui: &mut Ui, state: &mut AppState) {
    let options = state.download_section_picker_options();
    if options.is_empty() {
        state.format_picker.selected_row = None;
        ui.add_space(12.0);
        ui.label(state.tr("picker.no_chapters_available"));
        return;
    }

    if state
        .format_picker
        .selected_row
        .map_or(true, |row| row >= options.len())
    {
        state.format_picker.selected_row = Some(0);
    }

    let description_height = if state.tool_paths.chapter_compatibility_mode && options.len() > 1 {
        52.0
    } else {
        30.0
    };
    let row_height = 24.0;

    StripBuilder::new(ui)
        .size(Size::exact(description_height))
        .size(Size::remainder().at_least(160.0))
        .vertical(|mut strip| {
            strip.cell(|ui| {
                ui.add(
                    egui::Label::new(
                        state.tr("picker.choose_the_range_to_download_for_this_item_d"),
                    )
                    .selectable(false),
                );
                if state.tool_paths.chapter_compatibility_mode && options.len() > 1 {
                    ui.add(
                        egui::Label::new(
                            state.tr("picker.chapter_compatibility_mode_is_on_chapter_dow"),
                        )
                        .selectable(false),
                    );
                }
            });
            strip.cell(|ui| {
                ui.separator();
                let max_height = ui.available_height().max(160.0);
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(max_height)
                    .show(ui, |ui| {
                        TableBuilder::new(ui)
                            .id_salt("section-picker-table")
                            .striped(true)
                            .sense(Sense::click())
                            .cell_layout(Layout::left_to_right(Align::Center))
                            .column(Column::exact(20.0))
                            .column(Column::remainder().at_least(160.0))
                            .min_scrolled_height(160.0)
                            .max_scroll_height(max_height)
                            .header(row_height, |mut header| {
                                header.col(|ui| cell_label_center(ui, ""));
                                header.col(|ui| {
                                    cell_label_center(ui, state.tr(UiText::HEADER_RANGE))
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
                                        let response = ui.add(
                                            egui::Label::new(label.as_str())
                                                .truncate()
                                                .selectable(false)
                                                .sense(Sense::empty()),
                                        );
                                        if !label.is_empty() {
                                            response.on_hover_text(label.as_str());
                                        }
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

fn render_subtitle_picker_contents(ui: &mut Ui, state: &mut AppState) {
    let source_options = state.subtitle_source_options();
    if state.format_picker.subtitle_source_key.is_empty() {
        state.format_picker.subtitle_source_key = SubtitleSource::None.key().to_owned();
    }

    let none_label = state.tr(SubtitlePickerTab::None.label());
    let original_label = state.tr(SubtitlePickerTab::Original.label());
    let automatic_label = state.tr(SubtitlePickerTab::Automatic.label());
    let active_page = &mut state.format_picker.subtitle_tab;

    ui.horizontal(|ui| {
        ui.selectable_value(active_page, SubtitlePickerTab::None, none_label);
        ui.selectable_value(active_page, SubtitlePickerTab::Original, original_label);
        ui.selectable_value(active_page, SubtitlePickerTab::Automatic, automatic_label);
    });
    ui.separator();

    if *active_page == SubtitlePickerTab::None {
        state.format_picker.subtitle_source_key = SubtitleSource::None.key().to_owned();
        state.format_picker.selected_row = Some(0);
        ui.label(state.tr("picker.subtitles_will_not_be_downloaded"));
        return;
    }

    if source_options.is_empty() {
        ui.label(state.tr("picker.no_subtitles_are_available_for_this_video"));
        state.format_picker.selected_row = None;
        return;
    }

    let page_sources: Vec<_> = source_options
        .iter()
        .filter(|option| {
            matches!(
                (*active_page, option.source),
                (SubtitlePickerTab::Original, SubtitleSource::Original)
                    | (SubtitlePickerTab::Automatic, SubtitleSource::Automatic)
            )
        })
        .cloned()
        .collect();

    if page_sources.is_empty() {
        ui.label(state.tr("picker.no_subtitles_are_available_in_this_tab"));
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

    if *active_page == SubtitlePickerTab::Original {
        render_original_subtitle_picker(ui, state, &page_sources);
        return;
    }

    ui.label(state.tr("picker.source_language"));
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
    ui.label(state.tr("picker.translation_target"));
    ui.label(state.tr("picker.tip_youtube_auto_translated_subtitles_are_mo"));

    let options = state.subtitle_translation_options();
    if options.is_empty() {
        ui.label(state.tr("picker.no_subtitles_are_available_for_this_source"));
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
            .column(Column::remainder().at_least(180.0))
            .column(Column::auto().at_least(48.0))
            .header(24.0, |mut header| {
                header.col(|ui| {
                    cell_label_center(ui, state.tr("picker.target"));
                });
                header.col(|ui| {
                    cell_label_center(ui, state.tr(UiText::HEADER_EXT));
                });
            });

        table.body(|body| {
            body.rows(24.0, options.len(), |mut row| {
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
            });
        });
    });
}

fn subtitle_pending_options(state: &AppState) -> Vec<crate::domain::SubtitleOption> {
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
    ui.label(state.tr("picker.available_subtitles"));

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
            .column(Column::remainder().at_least(180.0))
            .column(Column::auto().at_least(48.0))
            .header(24.0, |mut header| {
                header.col(|ui| {
                    cell_label_center(ui, state.tr("picker.language"));
                });
                header.col(|ui| {
                    cell_label_center(ui, state.tr(UiText::HEADER_EXT));
                });
            });

        table.body(|body| {
            body.rows(24.0, page_sources.len(), |mut row| {
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
            });
        });
    });
}

fn sync_picker_mode(state: &mut AppState) {
    match state.format_picker.view_mode {
        FormatPickerViewMode::Filter => {
            if let Some(selected_row) = state.format_picker.selected_row {
                let kind = match state.format_picker.kind {
                    Some(kind) => kind,
                    None => return,
                };
                let options = state.format_picker_options(kind);
                if let Some(option) = options.get(selected_row) {
                    apply_filters_from_option(&mut state.format_picker.filters, kind, option);
                }
            }
        }
        FormatPickerViewMode::Table => {
            let kind = match state.format_picker.kind {
                Some(kind) => kind,
                None => return,
            };
            let options = state.format_picker_options(kind);
            let filtered = filtered_rows(&options, &state.format_picker.filters);
            if filtered.len() == 1 {
                state.format_picker.selected_row = filtered.first().copied();
            }
        }
    }
}

fn apply_filters_from_option(
    filters: &mut FormatPickerFilters,
    kind: FormatPickerKind,
    option: &FormatOption,
) {
    filters.clear();
    match kind {
        FormatPickerKind::Video => {
            if !option.resolution.is_empty() {
                filters.resolution = Some(option.resolution.clone());
            }
            if !option.dynamic_range.is_empty() {
                filters.dynamic_range = Some(option.dynamic_range.clone());
            }
            if !option.fps.is_empty() {
                filters.fps = Some(option.fps.clone());
            }
            if !option.codec.is_empty() {
                filters.codec = Some(option.codec.clone());
            }
        }
        FormatPickerKind::Audio => {
            if !option.sample_rate.is_empty() {
                filters.sample_rate = Some(option.sample_rate.clone());
            }
            if !option.codec.is_empty() {
                filters.codec = Some(option.codec.clone());
            }
        }
        FormatPickerKind::Subtitle | FormatPickerKind::Section => {}
    }
}

fn filtered_rows(options: &[FormatOption], filters: &FormatPickerFilters) -> Vec<usize> {
    options
        .iter()
        .enumerate()
        .filter_map(|(index, option)| option_matches_filters(option, filters).then_some(index))
        .collect()
}

fn option_matches_filters(option: &FormatOption, filters: &FormatPickerFilters) -> bool {
    if let Some(resolution) = &filters.resolution {
        if &option.resolution != resolution {
            return false;
        }
    }
    if let Some(dynamic_range) = &filters.dynamic_range {
        if &option.dynamic_range != dynamic_range {
            return false;
        }
    }
    if let Some(fps) = &filters.fps {
        if &option.fps != fps {
            return false;
        }
    }
    if let Some(codec) = &filters.codec {
        if &option.codec != codec {
            return false;
        }
    }
    if let Some(sample_rate) = &filters.sample_rate {
        if &option.sample_rate != sample_rate {
            return false;
        }
    }

    true
}

fn render_format_picker_filters(
    ui: &mut Ui,
    state: &mut AppState,
    kind: FormatPickerKind,
    options: &[FormatOption],
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());

        match kind {
            FormatPickerKind::Video => {
                let resolution_label = state.tr(UiText::FILTER_RESOLUTION);
                let fps_label = state.tr(UiText::FILTER_FPS);
                let codec_label = state.tr(UiText::FILTER_CODEC);
                let range_label = state.tr(UiText::FILTER_RANGE);
                let filters = &mut state.format_picker.filters;
                let selected_row = &mut state.format_picker.selected_row;
                render_filter_row(
                    ui,
                    resolution_label,
                    FilterField::Resolution,
                    all_resolution_values(options),
                    compatible_resolution_values(options, filters),
                    filters.resolution.clone(),
                    kind,
                    options,
                    filters,
                    selected_row,
                );
                render_filter_row(
                    ui,
                    fps_label,
                    FilterField::Fps,
                    all_fps_values(options),
                    compatible_fps_values(options, filters),
                    filters.fps.clone(),
                    kind,
                    options,
                    filters,
                    selected_row,
                );
                render_filter_row(
                    ui,
                    codec_label,
                    FilterField::Codec,
                    all_codec_values(options),
                    compatible_codec_values(options, filters),
                    filters.codec.clone(),
                    kind,
                    options,
                    filters,
                    selected_row,
                );
                render_filter_row(
                    ui,
                    range_label,
                    FilterField::DynamicRange,
                    all_range_values(options),
                    compatible_range_values(options, filters),
                    filters.dynamic_range.clone(),
                    kind,
                    options,
                    filters,
                    selected_row,
                );
            }
            FormatPickerKind::Audio => {
                let sample_rate_label = state.tr(UiText::FILTER_SAMPLE_RATE);
                let codec_label = state.tr(UiText::FILTER_CODEC);
                let filters = &mut state.format_picker.filters;
                let selected_row = &mut state.format_picker.selected_row;
                render_filter_row(
                    ui,
                    sample_rate_label,
                    FilterField::SampleRate,
                    all_sample_rate_values(options),
                    compatible_sample_rate_values(options, filters),
                    filters.sample_rate.clone(),
                    kind,
                    options,
                    filters,
                    selected_row,
                );
                render_filter_row(
                    ui,
                    codec_label,
                    FilterField::Codec,
                    all_codec_values(options),
                    compatible_codec_values(options, filters),
                    filters.codec.clone(),
                    kind,
                    options,
                    filters,
                    selected_row,
                );
            }
            FormatPickerKind::Subtitle | FormatPickerKind::Section => {}
        }
    });
}

fn available_filter_values(
    options: &[FormatOption],
    filters: &FormatPickerFilters,
    value_fn: impl Fn(&FormatOption) -> String,
) -> Vec<String> {
    let mut values = Vec::new();
    for option in options {
        if !option_matches_filters(option, filters) {
            continue;
        }
        let value = value_fn(option);
        if value.is_empty() || values.iter().any(|existing| existing == &value) {
            continue;
        }
        values.push(value);
    }
    values
}

fn distinct_values(
    options: &[FormatOption],
    value_fn: impl Fn(&FormatOption) -> String,
) -> Vec<String> {
    let mut values = Vec::new();
    for option in options {
        let value = value_fn(option);
        if value.is_empty() || values.iter().any(|existing| existing == &value) {
            continue;
        }
        values.push(value);
    }
    values
}

fn all_resolution_values(options: &[FormatOption]) -> Vec<String> {
    distinct_values(options, |option| option.resolution.clone())
}

fn compatible_resolution_values(
    options: &[FormatOption],
    filters: &FormatPickerFilters,
) -> Vec<String> {
    let mut filters = filters.clone();
    filters.resolution = None;
    available_filter_values(options, &filters, |option| option.resolution.clone())
}

fn all_range_values(options: &[FormatOption]) -> Vec<String> {
    distinct_values(options, |option| option.dynamic_range.clone())
}

fn compatible_range_values(options: &[FormatOption], filters: &FormatPickerFilters) -> Vec<String> {
    let mut filters = filters.clone();
    filters.dynamic_range = None;
    available_filter_values(options, &filters, |option| option.dynamic_range.clone())
}

fn all_fps_values(options: &[FormatOption]) -> Vec<String> {
    distinct_values(options, |option| option.fps.clone())
}

fn compatible_fps_values(options: &[FormatOption], filters: &FormatPickerFilters) -> Vec<String> {
    let mut filters = filters.clone();
    filters.fps = None;
    available_filter_values(options, &filters, |option| option.fps.clone())
}

fn all_codec_values(options: &[FormatOption]) -> Vec<String> {
    distinct_values(options, |option| option.codec.clone())
}

fn compatible_codec_values(options: &[FormatOption], filters: &FormatPickerFilters) -> Vec<String> {
    let mut filters = filters.clone();
    filters.codec = None;
    available_filter_values(options, &filters, |option| option.codec.clone())
}

fn all_sample_rate_values(options: &[FormatOption]) -> Vec<String> {
    distinct_values(options, |option| option.sample_rate.clone())
}

fn compatible_sample_rate_values(
    options: &[FormatOption],
    filters: &FormatPickerFilters,
) -> Vec<String> {
    let mut filters = filters.clone();
    filters.sample_rate = None;
    available_filter_values(options, &filters, |option| option.sample_rate.clone())
}

fn render_filter_row(
    ui: &mut Ui,
    label: &str,
    field: FilterField,
    values: Vec<String>,
    compatible_values: Vec<String>,
    selected: Option<String>,
    kind: FormatPickerKind,
    options: &[FormatOption],
    filters: &mut FormatPickerFilters,
    selected_row: &mut Option<usize>,
) {
    if values.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        ui.set_width(ui.available_width());
        ui.label(label);
        ui.horizontal_wrapped(|ui| {
            ui.set_width(ui.available_width());
            for value in values {
                let is_selected = selected.as_deref() == Some(value.as_str());
                let is_enabled = is_selected || compatible_values.iter().any(|item| item == &value);
                let mut button = egui::Button::new(&value)
                    .frame(true)
                    .min_size(egui::vec2(0.0, ui.spacing().interact_size.y + 8.0));
                if !is_enabled && !is_selected {
                    button = button.fill(incompatible_filter_button_fill(ui));
                }

                if ui.add(button.selected(is_selected)).clicked() {
                    if is_selected {
                        set_filter_value(filters, field, None);
                        *selected_row = selected_row_for_filters(options, filters);
                    } else {
                        *selected_row =
                            force_pick_filter_value(filters, field, value, kind, options);
                    }
                }
            }
        });
    });
}

fn incompatible_filter_button_fill(ui: &Ui) -> egui::Color32 {
    if ui.visuals().dark_mode {
        egui::Color32::BLACK
    } else {
        ui.visuals().widgets.noninteractive.bg_fill
    }
}

fn set_filter_value(filters: &mut FormatPickerFilters, field: FilterField, value: Option<String>) {
    match field {
        FilterField::Resolution => filters.resolution = value,
        FilterField::DynamicRange => filters.dynamic_range = value,
        FilterField::Fps => filters.fps = value,
        FilterField::Codec => filters.codec = value,
        FilterField::SampleRate => filters.sample_rate = value,
    }
}

fn force_pick_filter_value(
    filters: &mut FormatPickerFilters,
    field: FilterField,
    value: String,
    kind: FormatPickerKind,
    options: &[FormatOption],
) -> Option<usize> {
    let previous_filters = filters.clone();
    set_filter_value(filters, field, Some(value.clone()));

    if !filtered_rows(options, filters).is_empty() {
        maybe_snap_filters_to_single(filters, kind, options);
        return selected_row_for_filters(options, filters);
    }

    if let Some((index, option)) =
        closest_option_for_forced_value(options, kind, field, &value, &previous_filters)
    {
        apply_filters_from_option(filters, kind, option);
        return Some(index);
    }

    filters.clear();
    set_filter_value(filters, field, Some(value));
    selected_row_for_filters(options, filters)
}

fn current_for_field(filters: &FormatPickerFilters, field: FilterField) -> Option<String> {
    match field {
        FilterField::Resolution => filters.resolution.clone(),
        FilterField::DynamicRange => filters.dynamic_range.clone(),
        FilterField::Fps => filters.fps.clone(),
        FilterField::Codec => filters.codec.clone(),
        FilterField::SampleRate => filters.sample_rate.clone(),
    }
}

fn selected_row_for_filters(
    options: &[FormatOption],
    filters: &FormatPickerFilters,
) -> Option<usize> {
    filtered_rows(options, filters).into_iter().next()
}

fn closest_option_for_forced_value<'a>(
    options: &'a [FormatOption],
    kind: FormatPickerKind,
    field: FilterField,
    value: &str,
    previous_filters: &FormatPickerFilters,
) -> Option<(usize, &'a FormatOption)> {
    let fields = filter_fields_for_kind(kind);
    options
        .iter()
        .enumerate()
        .filter(|(_, option)| value_for_field(option, field) == value)
        .max_by_key(|(_, option)| filter_proximity_score(option, field, fields, previous_filters))
}

fn filter_fields_for_kind(kind: FormatPickerKind) -> &'static [FilterField] {
    match kind {
        FormatPickerKind::Video => &[
            FilterField::Resolution,
            FilterField::Fps,
            FilterField::Codec,
            FilterField::DynamicRange,
        ],
        FormatPickerKind::Audio => &[FilterField::SampleRate, FilterField::Codec],
        FormatPickerKind::Subtitle | FormatPickerKind::Section => &[],
    }
}

fn filter_proximity_score(
    option: &FormatOption,
    forced_field: FilterField,
    fields: &[FilterField],
    previous_filters: &FormatPickerFilters,
) -> i64 {
    fields
        .iter()
        .filter(|&&field| field != forced_field)
        .map(|&field| {
            let Some(previous_value) = current_for_field(previous_filters, field) else {
                return 0;
            };
            score_field_value(field, &previous_value, value_for_field(option, field))
        })
        .sum()
}

fn score_field_value(field: FilterField, previous_value: &str, candidate_value: &str) -> i64 {
    if previous_value == candidate_value {
        return 10_000;
    }

    match field {
        FilterField::Resolution => numeric_closeness_score(
            resolution_area_text(previous_value),
            resolution_area_text(candidate_value),
        ),
        FilterField::Fps | FilterField::SampleRate => numeric_closeness_score(
            number_from_text(previous_value),
            number_from_text(candidate_value),
        ),
        FilterField::DynamicRange | FilterField::Codec => 0,
    }
}

fn numeric_closeness_score(previous: Option<i64>, candidate: Option<i64>) -> i64 {
    let (Some(previous), Some(candidate)) = (previous, candidate) else {
        return 0;
    };
    5_000 - previous.abs_diff(candidate) as i64
}

fn resolution_area_text(value: &str) -> Option<i64> {
    let (width, height) = value.split_once('x')?;
    let width = width.trim().parse::<i64>().ok()?;
    let height = height.trim().parse::<i64>().ok()?;
    Some(width.saturating_mul(height))
}

fn number_from_text(value: &str) -> Option<i64> {
    let digits: String = value.chars().filter(|ch| ch.is_ascii_digit()).collect();
    digits.parse::<i64>().ok()
}

fn value_for_field(option: &FormatOption, field: FilterField) -> &str {
    match field {
        FilterField::Resolution => &option.resolution,
        FilterField::DynamicRange => &option.dynamic_range,
        FilterField::Fps => &option.fps,
        FilterField::Codec => &option.codec,
        FilterField::SampleRate => &option.sample_rate,
    }
}

fn maybe_snap_filters_to_single(
    filters: &mut FormatPickerFilters,
    kind: FormatPickerKind,
    options: &[FormatOption],
) {
    let rows = filtered_rows(options, filters);
    if rows.len() == 1 {
        if let Some(option) = rows.first().and_then(|&index| options.get(index)) {
            apply_filters_from_option(filters, kind, option);
        }
    }
}

fn render_format_picker_table(
    ui: &mut Ui,
    state: &mut AppState,
    kind: FormatPickerKind,
    options: &[FormatOption],
) {
    let row_height = 24.0;
    let max_height = ui.available_height().max(160.0);

    ui.set_height(max_height);
    ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let table = match kind {
                FormatPickerKind::Video => TableBuilder::new(ui)
                    .id_salt("video-format-picker-table")
                    .striped(true)
                    .sense(Sense::click())
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(Column::exact(20.0))
                    .column(Column::remainder().at_least(96.0))
                    .column(Column::auto().at_least(32.0))
                    .column(Column::auto().at_least(24.0))
                    .column(Column::auto().at_least(96.0))
                    .column(Column::auto().at_least(72.0))
                    .min_scrolled_height(180.0)
                    .max_scroll_height(max_height)
                    .header(row_height, |mut header| {
                        header.col(|ui| cell_label_center(ui, ""));
                        header.col(|ui| cell_label_center(ui, state.tr(UiText::HEADER_RESOLUTION)));
                        header.col(|ui| cell_label_center(ui, state.tr(UiText::HEADER_RANGE)));
                        header.col(|ui| cell_label_center(ui, state.tr(UiText::HEADER_FPS)));
                        header.col(|ui| cell_label_center(ui, state.tr(UiText::HEADER_CODEC)));
                        header.col(|ui| cell_label_center(ui, state.tr(UiText::HEADER_FILESIZE)));
                    }),
                FormatPickerKind::Audio => TableBuilder::new(ui)
                    .id_salt("audio-format-picker-table")
                    .striped(true)
                    .sense(Sense::click())
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(Column::exact(20.0))
                    .column(Column::auto().at_least(84.0))
                    .column(Column::remainder().at_least(96.0))
                    .column(Column::auto().at_least(72.0))
                    .min_scrolled_height(180.0)
                    .max_scroll_height(max_height)
                    .header(row_height, |mut header| {
                        header.col(|ui| cell_label_center(ui, ""));
                        header
                            .col(|ui| cell_label_center(ui, state.tr(UiText::HEADER_SAMPLE_RATE)));
                        header.col(|ui| cell_label_center(ui, state.tr(UiText::HEADER_CODEC)));
                        header.col(|ui| cell_label_center(ui, state.tr(UiText::HEADER_FILESIZE)));
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
                            row.col(|ui| {
                                cell_label(
                                    ui,
                                    if option.kind == MediaKind::Muxed {
                                        "⛓"
                                    } else {
                                        ""
                                    },
                                )
                            });
                            row.col(|ui| cell_label_center(ui, &option.resolution));
                            row.col(|ui| cell_label_center(ui, &option.dynamic_range));
                            row.col(|ui| cell_label_right(ui, &option.fps));
                            row.col(|ui| cell_label_center(ui, &option.codec));
                            row.col(|ui| cell_label_right(ui, &option.filesize));
                        }
                        FormatPickerKind::Audio => {
                            row.col(|ui| {
                                cell_label(
                                    ui,
                                    if option.kind == MediaKind::Muxed {
                                        "⛓"
                                    } else {
                                        ""
                                    },
                                )
                            });
                            row.col(|ui| cell_label(ui, &option.sample_rate));
                            row.col(|ui| cell_label(ui, &option.codec));
                            row.col(|ui| cell_label(ui, &option.filesize));
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
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forced_video_filter_picks_closest_existing_combination() {
        let options = vec![
            FormatOption::video(
                "low-av1",
                "low-av1",
                MediaKind::Video,
                "1280x720",
                "SDR",
                "30",
                "webm",
                "av1",
                "10.00 MB",
            ),
            FormatOption::video(
                "high-av1",
                "high-av1",
                MediaKind::Video,
                "1920x1080",
                "SDR",
                "60",
                "webm",
                "av1",
                "20.00 MB",
            ),
            FormatOption::video(
                "high-h264",
                "high-h264",
                MediaKind::Video,
                "1920x1080",
                "SDR",
                "60",
                "mp4",
                "h264",
                "30.00 MB",
            ),
        ];
        let mut filters = FormatPickerFilters {
            resolution: Some("1920x1080".to_owned()),
            dynamic_range: Some("SDR".to_owned()),
            fps: Some("60".to_owned()),
            ext: Some("mp4".to_owned()),
            codec: Some("h264".to_owned()),
            sample_rate: None,
        };

        let selected_row = force_pick_filter_value(
            &mut filters,
            FilterField::Codec,
            "av1".to_owned(),
            FormatPickerKind::Video,
            &options,
        );

        assert_eq!(selected_row, Some(1));
        assert_eq!(filters.codec.as_deref(), Some("av1"));
        assert_eq!(filters.resolution.as_deref(), Some("1920x1080"));
        assert_eq!(filters.fps.as_deref(), Some("60"));
        assert_eq!(filters.ext.as_deref(), None);
        assert_eq!(filtered_rows(&options, &filters), vec![1]);
    }

    #[test]
    fn forced_audio_filter_picks_closest_sample_rate() {
        let options = vec![
            FormatOption::audio(
                "opus-low",
                "opus-low",
                MediaKind::Audio,
                "44100",
                "webm",
                "opus",
                "3.00 MB",
            ),
            FormatOption::audio(
                "opus-high",
                "opus-high",
                MediaKind::Audio,
                "48000",
                "webm",
                "opus",
                "4.00 MB",
            ),
            FormatOption::audio(
                "aac",
                "aac",
                MediaKind::Audio,
                "48000",
                "m4a",
                "aac",
                "5.00 MB",
            ),
        ];
        let mut filters = FormatPickerFilters {
            resolution: None,
            dynamic_range: None,
            fps: None,
            ext: Some("m4a".to_owned()),
            codec: Some("aac".to_owned()),
            sample_rate: Some("48000".to_owned()),
        };

        let selected_row = force_pick_filter_value(
            &mut filters,
            FilterField::Codec,
            "opus".to_owned(),
            FormatPickerKind::Audio,
            &options,
        );

        assert_eq!(selected_row, Some(1));
        assert_eq!(filters.codec.as_deref(), Some("opus"));
        assert_eq!(filters.sample_rate.as_deref(), Some("48000"));
        assert_eq!(filters.ext.as_deref(), None);
        assert_eq!(filtered_rows(&options, &filters), vec![1]);
    }
}
