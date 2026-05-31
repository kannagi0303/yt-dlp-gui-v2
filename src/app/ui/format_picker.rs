use crate::app::state::{
    AppState, FormatPickerFilters, FormatPickerKind, FormatPickerViewMode, SubtitlePickerTab,
};
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::domain::{FormatOption, MediaKind, SubtitleSource};
use eframe::egui::{self, Align, Layout, ScrollArea, Sense, Ui};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use egui_taffy::taffy::prelude::{length, percent};
use egui_taffy::{TuiBuilderLogic as _, taffy, tui};

use super::common::{
    UiText, cell_label, cell_label_center, cell_label_right, natural_button_width,
};
use super::measure::{WidthRange, measured_column_width, measured_text_width, text_width};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterField {
    Resolution,
    DynamicRange,
    Fps,
    Codec,
    SampleRate,
}

struct FilterChainStage {
    label: String,
    field: FilterField,
    values: Vec<String>,
    compatible_values: Vec<String>,
    selected: Option<String>,
}

struct FilterNodeRect {
    value: String,
    rect: egui::Rect,
    selected: bool,
}

pub(super) fn render_format_picker_screen(ui: &mut Ui, state: &mut AppState) {
    let pending_selection = pending_picker_selection_id(state);
    let selection_summary = pending_picker_selection_summary(state);
    let available_width = ui.available_width();
    let available_height = ui.available_height();
    let header_height = ui.spacing().interact_size.y + 12.0;
    let gap = ui.spacing().item_spacing.x;
    let back_text = state.ui_tr(UiText::BACK_TO_MAIN);
    let confirm_text = state.ui_tr(UiText::CONFIRM);
    let back_width = natural_button_width(ui, back_text);
    let confirm_width = natural_button_width(ui, confirm_text);
    let center_width = format_picker_header_center_width(ui, state);
    let wanted_summary_width = selection_summary
        .as_deref()
        .map(|summary| text_width(ui, summary, egui::TextStyle::Body) + gap * 1.5)
        .unwrap_or(0.0);
    let max_summary_width =
        (available_width - back_width - center_width - confirm_width - gap * 4.0).max(0.0);
    let summary_width = wanted_summary_width.min(max_summary_width);

    tui(ui, ui.id().with("format-picker-root"))
        .reserve_width(available_width)
        .reserve_height(available_height)
        .style(format_picker_root_style(available_height))
        .show(|tui| {
            tui.style(format_picker_fixed_row_style(header_height))
                .add(|tui| {
                    tui.style(format_picker_header_row_style(header_height, gap))
                        .add(|tui| {
                            tui.style(format_picker_fixed_cell_style(back_width))
                                .ui(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        if ui.button(back_text).clicked() {
                                            state.cancel_format_picker();
                                        }
                                    });
                                });
                            tui.style(format_picker_fixed_cell_style(center_width))
                                .ui(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        render_format_picker_header_center(ui, state);
                                    });
                                });
                            tui.style(format_picker_flex_spacer_style()).ui(|_| {});
                            if summary_width > 0.0 {
                                tui.style(format_picker_fixed_cell_style(summary_width))
                                    .ui(|ui| {
                                        if let Some(summary) = selection_summary.as_deref() {
                                            ui.centered_and_justified(|ui| {
                                                ui.label(
                                                    egui::RichText::new(summary)
                                                        .color(ui.visuals().weak_text_color()),
                                                );
                                            });
                                        }
                                    });
                            }
                            tui.style(format_picker_fixed_cell_style(confirm_width))
                                .ui(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        if ui
                                            .add_enabled(
                                                pending_selection.is_some(),
                                                egui::Button::new(confirm_text),
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
                });
            tui.style(format_picker_flex_content_style()).ui(|ui| {
                ui.separator();
                render_format_picker_contents(ui, state);
            });
        });
}

fn format_picker_root_style(height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        min_size: taffy::Size {
            width: percent(1.0),
            height: length(0.0),
        },
        gap: length(0.0),
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn format_picker_fixed_row_style(height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        min_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn format_picker_flex_content_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0),
            height: length(0.0),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        flex_basis: length(0.0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn format_picker_header_row_style(height: f32, gap: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        min_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        gap: length(gap),
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn format_picker_fixed_cell_style(width: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(width),
            height: percent(1.0),
        },
        min_size: taffy::Size {
            width: length(width),
            height: length(0.0),
        },
        max_size: taffy::Size {
            width: length(width),
            height: percent(1.0),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn format_picker_flex_spacer_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(0.0),
            height: percent(1.0),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        flex_basis: length(0.0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn format_picker_header_center_width(ui: &Ui, state: &AppState) -> f32 {
    let Some(kind) = state.format_picker.kind else {
        return 0.0;
    };

    let gap = ui.spacing().item_spacing.x;
    match kind {
        FormatPickerKind::Video | FormatPickerKind::Audio => {
            let filter_text = state.ui_tr(UiText::PICKER_MODE_FILTER);
            let table_text = state.ui_tr(UiText::PICKER_MODE_TABLE);
            natural_button_width(ui, filter_text) + natural_button_width(ui, table_text) + gap
        }
        FormatPickerKind::Subtitle => {
            let none_text = state.ui_tr(SubtitlePickerTab::None.label_key());
            let original_text = state.ui_tr(SubtitlePickerTab::Original.label_key());
            let automatic_text = state.ui_tr(SubtitlePickerTab::Automatic.label_key());
            natural_button_width(ui, none_text)
                + natural_button_width(ui, original_text)
                + natural_button_width(ui, automatic_text)
                + gap * 2.0
        }
        FormatPickerKind::Section => {
            let title = state.ui_tr(UiText::SELECT_SECTION_TITLE);
            text_width(ui, title, egui::TextStyle::Body) + gap * 2.0
        }
    }
}

fn render_format_picker_header_center(ui: &mut Ui, state: &mut AppState) {
    let Some(kind) = state.format_picker.kind else {
        return;
    };

    if matches!(kind, FormatPickerKind::Video | FormatPickerKind::Audio) {
        let previous_mode = state.format_picker.view_mode;
        let picker_mode_filter = state.ui_tr(UiText::PICKER_MODE_FILTER);
        let picker_mode_table = state.ui_tr(UiText::PICKER_MODE_TABLE);
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
        });
        if state.format_picker.view_mode != previous_mode {
            sync_picker_mode(state);
        }
        return;
    }

    if kind == FormatPickerKind::Subtitle {
        render_subtitle_picker_header_tabs(ui, state);
        return;
    }

    let title = match kind {
        FormatPickerKind::Video => UiText::SELECT_VIDEO_TITLE,
        FormatPickerKind::Audio => UiText::SELECT_AUDIO_TITLE,
        FormatPickerKind::Subtitle => UiText::SELECT_SUBTITLE_TITLE,
        FormatPickerKind::Section => UiText::SELECT_SECTION_TITLE,
    };
    ui.label(state.ui_tr(title));
}

fn render_subtitle_picker_header_tabs(ui: &mut Ui, state: &mut AppState) {
    let none_label = state.ui_tr(SubtitlePickerTab::None.label_key());
    let original_label = state.ui_tr(SubtitlePickerTab::Original.label_key());
    let automatic_label = state.ui_tr(SubtitlePickerTab::Automatic.label_key());

    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut state.format_picker.subtitle_tab,
            SubtitlePickerTab::None,
            none_label,
        );
        ui.selectable_value(
            &mut state.format_picker.subtitle_tab,
            SubtitlePickerTab::Original,
            original_label,
        );
        ui.selectable_value(
            &mut state.format_picker.subtitle_tab,
            SubtitlePickerTab::Automatic,
            automatic_label,
        );
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

    if let Some(selected_row) = state.format_picker.selected_row {
        if selected_row >= options.len() {
            state.format_picker.selected_row = None;
        }
    }

    if state.format_picker.view_mode == FormatPickerViewMode::Filter && filtered_rows.len() == 1 {
        state.format_picker.selected_row = filtered_rows.first().copied();
    }

    match state.format_picker.view_mode {
        FormatPickerViewMode::Filter => {
            if filtered_rows.is_empty() {
                ui.add_space(12.0);
                ui.label(state.ui_tr(UiText::EMPTY_TABLE));
            }
            render_format_picker_filters(ui, state, kind, &options);
        }
        FormatPickerViewMode::Table => {
            if options.is_empty() {
                ui.add_space(12.0);
                ui.label(state.ui_tr(UiText::EMPTY_TABLE));
            } else {
                render_format_picker_table(ui, state, kind, &options);
            }
        }
    }
}

fn pending_picker_selection_summary(state: &AppState) -> Option<String> {
    let kind = state.format_picker.kind?;
    if !matches!(kind, FormatPickerKind::Video | FormatPickerKind::Audio) {
        return None;
    }

    let size_label = state.ui_tr(UiText::HEADER_FILESIZE);
    let filesize = pending_picker_selected_format(state)
        .map(|option| option.filesize.trim().to_owned())
        .filter(|filesize| !filesize.is_empty())
        .unwrap_or_else(|| "—".to_owned());
    Some(format!("{size_label} {filesize}"))
}

fn pending_picker_selected_format(state: &AppState) -> Option<FormatOption> {
    let kind = state.format_picker.kind?;
    if !matches!(kind, FormatPickerKind::Video | FormatPickerKind::Audio) {
        return None;
    }

    let options = state.format_picker_options(kind);
    match state.format_picker.view_mode {
        FormatPickerViewMode::Filter => {
            let visible_rows = filtered_rows(&options, &state.format_picker.filters);
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
        FormatPickerViewMode::Table => state
            .format_picker
            .selected_row
            .and_then(|row| options.get(row))
            .cloned(),
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
        ui.label(state.ui_tr("picker.no_chapters_available"));
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
                        state.ui_tr("picker.choose_the_range_to_download_for_this_item_d"),
                    )
                    .selectable(false),
                );
                if state.tool_paths.chapter_compatibility_mode && options.len() > 1 {
                    ui.add(
                        egui::Label::new(
                            state.ui_tr("picker.chapter_compatibility_mode_is_on_chapter_dow"),
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
                                    cell_label_center(ui, state.ui_tr(UiText::HEADER_RANGE))
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

fn render_subtitle_picker_contents(ui: &mut Ui, state: &mut AppState) {
    let source_options = state.subtitle_source_options();
    if state.format_picker.subtitle_source_key.is_empty() {
        state.format_picker.subtitle_source_key = SubtitleSource::None.key().to_owned();
    }

    let active_page = state.format_picker.subtitle_tab;

    if active_page == SubtitlePickerTab::None {
        state.format_picker.subtitle_source_key = SubtitleSource::None.key().to_owned();
        state.format_picker.selected_row = Some(0);
        ui.label(state.ui_tr("picker.subtitles_will_not_be_downloaded"));
        return;
    }

    if source_options.is_empty() {
        ui.label(state.ui_tr("picker.no_subtitles_are_available_for_this_video"));
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
        ui.label(state.ui_tr("picker.no_subtitles_are_available_in_this_tab"));
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

    ui.label(state.ui_tr("picker.source_language"));
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
    ui.label(state.ui_tr("picker.translation_target"));
    ui.label(state.ui_tr("picker.tip_youtube_auto_translated_subtitles_are_mo"));

    let options = state.subtitle_translation_options();
    if options.is_empty() {
        ui.label(state.ui_tr("picker.no_subtitles_are_available_for_this_source"));
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
                    cell_label_center(ui, state.ui_tr("picker.target"));
                });
                header.col(|ui| {
                    cell_label_center(ui, state.ui_tr(UiText::HEADER_EXT));
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
    ui.label(state.ui_tr("picker.available_subtitles"));

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
                    cell_label_center(ui, state.ui_tr("picker.language"));
                });
                header.col(|ui| {
                    cell_label_center(ui, state.ui_tr(UiText::HEADER_EXT));
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
    let filters_snapshot = state.format_picker.filters.clone();
    let stages = match kind {
        FormatPickerKind::Video => vec![
            FilterChainStage {
                label: state.ui_tr(UiText::FILTER_RESOLUTION).to_owned(),
                field: FilterField::Resolution,
                values: all_resolution_values(options),
                compatible_values: compatible_resolution_values(options, &filters_snapshot),
                selected: filters_snapshot.resolution.clone(),
            },
            FilterChainStage {
                label: state.ui_tr(UiText::FILTER_FPS).to_owned(),
                field: FilterField::Fps,
                values: all_fps_values(options),
                compatible_values: compatible_fps_values(options, &filters_snapshot),
                selected: filters_snapshot.fps.clone(),
            },
            FilterChainStage {
                label: state.ui_tr(UiText::FILTER_CODEC).to_owned(),
                field: FilterField::Codec,
                values: all_codec_values(options),
                compatible_values: compatible_codec_values(options, &filters_snapshot),
                selected: filters_snapshot.codec.clone(),
            },
            FilterChainStage {
                label: state.ui_tr(UiText::FILTER_RANGE).to_owned(),
                field: FilterField::DynamicRange,
                values: all_range_values(options),
                compatible_values: compatible_range_values(options, &filters_snapshot),
                selected: filters_snapshot.dynamic_range.clone(),
            },
        ],
        FormatPickerKind::Audio => vec![
            FilterChainStage {
                label: state.ui_tr(UiText::FILTER_SAMPLE_RATE).to_owned(),
                field: FilterField::SampleRate,
                values: all_sample_rate_values(options),
                compatible_values: compatible_sample_rate_values(options, &filters_snapshot),
                selected: filters_snapshot.sample_rate.clone(),
            },
            FilterChainStage {
                label: state.ui_tr(UiText::FILTER_CODEC).to_owned(),
                field: FilterField::Codec,
                values: all_codec_values(options),
                compatible_values: compatible_codec_values(options, &filters_snapshot),
                selected: filters_snapshot.codec.clone(),
            },
        ],
        FormatPickerKind::Subtitle | FormatPickerKind::Section => Vec::new(),
    };

    render_filter_chain(
        ui,
        stages,
        kind,
        options,
        &mut state.format_picker.filters,
        &mut state.format_picker.selected_row,
    );
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

fn render_filter_chain(
    ui: &mut Ui,
    stages: Vec<FilterChainStage>,
    kind: FormatPickerKind,
    options: &[FormatOption],
    filters: &mut FormatPickerFilters,
    selected_row: &mut Option<usize>,
) {
    let stages: Vec<FilterChainStage> = stages
        .into_iter()
        .filter(|stage| !stage.values.is_empty())
        .collect();
    if stages.is_empty() {
        return;
    }

    let button_padding = egui::vec2(5.0, 1.0);
    let node_height = (ui.spacing().interact_size.y - 10.0).clamp(20.0, 24.0);
    let stage_widths = measure_filter_stage_widths(ui, &stages, button_padding.x);
    let stage_count = stages.len();
    let viewport_width = (ui.available_width() - 18.0).max(1.0);
    let slot_width = (viewport_width / stage_count as f32).max(1.0);
    let content_width = viewport_width;

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(content_width);
            ui.set_min_height(ui.available_height());

            let mut columns: Vec<Vec<FilterNodeRect>> = Vec::with_capacity(stages.len());

            ui.horizontal_top(|ui| {
                for (stage_index, stage) in stages.iter().enumerate() {
                    let stage_width = stage_widths[stage_index].min((slot_width - 8.0).max(48.0));
                    let mut node_rects = Vec::with_capacity(stage.values.len());

                    ui.allocate_ui_with_layout(
                        egui::vec2(slot_width, ui.available_height()),
                        Layout::top_down(Align::Center),
                        |ui| {
                            ui.set_width(slot_width);
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(stage.label.as_str()).strong(),
                                )
                                .selectable(false),
                            );
                            ui.add_space(5.0);

                            ui.scope(|ui| {
                                ui.spacing_mut().button_padding = button_padding;
                                for value in &stage.values {
                                    let is_selected =
                                        stage.selected.as_deref() == Some(value.as_str());
                                    let is_enabled = is_selected
                                        || stage
                                            .compatible_values
                                            .iter()
                                            .any(|compatible| compatible == value);
                                    let mut button = egui::Button::new(value.as_str())
                                        .frame(true)
                                        .min_size(egui::vec2(stage_width, node_height));
                                    if !is_enabled && !is_selected {
                                        button = button.fill(incompatible_filter_button_fill(ui));
                                    }

                                    let response = ui.add_sized(
                                        egui::vec2(stage_width, node_height),
                                        button.selected(is_selected),
                                    );

                                    if response.clicked() {
                                        if is_selected {
                                            set_filter_value(filters, stage.field, None);
                                            *selected_row =
                                                selected_row_for_filters(options, filters);
                                        } else {
                                            *selected_row = force_pick_filter_value(
                                                filters,
                                                stage.field,
                                                value.clone(),
                                                kind,
                                                options,
                                            );
                                        }
                                    }

                                    node_rects.push(FilterNodeRect {
                                        value: value.clone(),
                                        rect: response.rect,
                                        selected: is_selected,
                                    });

                                    ui.add_space(3.0);
                                }
                            });
                        },
                    );
                    columns.push(node_rects);
                }
            });

            draw_filter_chain_connections(ui, &stages, &columns, options, *selected_row);
        });
}

fn draw_filter_chain_connections(
    ui: &Ui,
    stages: &[FilterChainStage],
    columns: &[Vec<FilterNodeRect>],
    options: &[FormatOption],
    selected_row: Option<usize>,
) {
    if stages.len() < 2 || columns.len() < 2 {
        return;
    }

    let active_color = filter_flow_active_line_color(ui);
    let painter = ui.painter();
    let selected_option = selected_row.and_then(|row| options.get(row));

    for pair_index in 0..(stages.len() - 1) {
        let left_stage = &stages[pair_index];
        let right_stage = &stages[pair_index + 1];
        let left_nodes = &columns[pair_index];
        let right_nodes = &columns[pair_index + 1];

        let Some(left_value) = selected_filter_value(selected_option, left_stage) else {
            continue;
        };
        let Some(right_value) = selected_filter_value(selected_option, right_stage) else {
            continue;
        };

        let Some(left_node) = left_nodes.iter().find(|node| node.value == left_value) else {
            continue;
        };
        let Some(right_node) = right_nodes.iter().find(|node| node.value == right_value) else {
            continue;
        };
        if !left_node.selected || !right_node.selected {
            continue;
        }

        let stroke = egui::Stroke::new(2.0, active_color);
        let start = egui::pos2(left_node.rect.right(), left_node.rect.center().y);
        let end = egui::pos2(right_node.rect.left(), right_node.rect.center().y);
        draw_curved_filter_connection(painter, start, end, stroke);
    }
}

fn measure_filter_stage_widths(
    ui: &Ui,
    stages: &[FilterChainStage],
    horizontal_padding: f32,
) -> Vec<f32> {
    stages
        .iter()
        .map(|stage| {
            measured_text_width(
                ui,
                stage.values.iter().map(|value| value.as_str()),
                egui::TextStyle::Button,
                horizontal_padding * 2.0 + 8.0,
                WidthRange::new(64.0, 220.0),
            )
        })
        .collect()
}

fn selected_filter_value(
    selected_option: Option<&FormatOption>,
    stage: &FilterChainStage,
) -> Option<String> {
    if let Some(option) = selected_option {
        let value = value_for_field(option, stage.field);
        if !value.is_empty() {
            return Some(value.to_owned());
        }
    }
    stage.selected.clone()
}

fn draw_curved_filter_connection(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    stroke: egui::Stroke,
) {
    let mid_x = (start.x + end.x) * 0.5;
    let dy = end.y - start.y;
    if dy.abs() < 1.0 {
        painter.line_segment([start, end], stroke);
        return;
    }

    let direction = dy.signum();
    let radius = dy.abs().mul_add(0.25, 0.0).clamp(8.0, 14.0);
    let left_corner_start = egui::pos2(mid_x - radius, start.y);
    let left_corner_end = egui::pos2(mid_x, start.y + direction * radius);
    let right_corner_start = egui::pos2(mid_x, end.y - direction * radius);
    let right_corner_end = egui::pos2(mid_x + radius, end.y);

    painter.line_segment([start, left_corner_start], stroke);
    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        [
            left_corner_start,
            egui::pos2(mid_x - radius * 0.45, start.y),
            egui::pos2(mid_x, start.y + direction * radius * 0.55),
            left_corner_end,
        ],
        false,
        egui::Color32::TRANSPARENT,
        stroke,
    ));
    painter.line_segment([left_corner_end, right_corner_start], stroke);
    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        [
            right_corner_start,
            egui::pos2(mid_x, end.y - direction * radius * 0.55),
            egui::pos2(mid_x + radius * 0.45, end.y),
            right_corner_end,
        ],
        false,
        egui::Color32::TRANSPARENT,
        stroke,
    ));
    painter.line_segment([right_corner_end, end], stroke);
}

fn filter_flow_active_line_color(ui: &Ui) -> egui::Color32 {
    ui.visuals().widgets.active.bg_fill
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
            .column(Column::remainder().at_least(0.0))
            .min_scrolled_height(180.0)
            .max_scroll_height(max_height)
            .header(row_height, |mut header| {
                let resolution_text = state.ui_tr(UiText::HEADER_RESOLUTION);
                let range_text = state.ui_tr(UiText::HEADER_RANGE);
                let fps_text = state.ui_tr(UiText::HEADER_FPS);
                let codec_text = state.ui_tr(UiText::HEADER_CODEC);
                let filesize_text = state.ui_tr(UiText::HEADER_FILESIZE);
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
            .column(Column::remainder().at_least(0.0))
            .min_scrolled_height(180.0)
            .max_scroll_height(max_height)
            .header(row_height, |mut header| {
                let sample_rate_text = state.ui_tr(UiText::HEADER_SAMPLE_RATE);
                let codec_text = state.ui_tr(UiText::HEADER_CODEC);
                let filesize_text = state.ui_tr(UiText::HEADER_FILESIZE);
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
    let icon_size = 14.0;
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
    let marker = 18.0;
    let resolution_text = state.ui_tr(UiText::HEADER_RESOLUTION);
    let range_text = state.ui_tr(UiText::HEADER_RANGE);
    let fps_text = state.ui_tr(UiText::HEADER_FPS);
    let sample_rate_text = state.ui_tr(UiText::HEADER_SAMPLE_RATE);
    let codec_text = state.ui_tr(UiText::HEADER_CODEC);
    let filesize_text = state.ui_tr(UiText::HEADER_FILESIZE);
    let resolution = measure_format_table_column(
        ui,
        resolution_text,
        options.iter().map(|option| option.resolution.as_str()),
        72.0,
        128.0,
    );
    let dynamic_range = measure_format_table_column(
        ui,
        range_text,
        options.iter().map(|option| option.dynamic_range.as_str()),
        44.0,
        88.0,
    );
    let fps = measure_format_table_column(
        ui,
        fps_text,
        options.iter().map(|option| option.fps.as_str()),
        36.0,
        72.0,
    );
    let sample_rate = measure_format_table_column(
        ui,
        sample_rate_text,
        options.iter().map(|option| option.sample_rate.as_str()),
        76.0,
        124.0,
    );
    let codec_min = match kind {
        FormatPickerKind::Audio => 96.0,
        _ => 84.0,
    };
    let codec = measure_format_table_column(
        ui,
        codec_text,
        options.iter().map(|option| option.codec.as_str()),
        codec_min,
        220.0,
    );
    let filesize = measure_format_table_column(
        ui,
        filesize_text,
        options.iter().map(|option| option.filesize.as_str()),
        70.0,
        112.0,
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

fn measure_format_table_column<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
    min_width: f32,
    max_width: f32,
) -> f32 {
    measured_column_width(
        ui,
        header,
        values,
        egui::TextStyle::Body,
        ui.spacing().item_spacing.x * 2.0 + 14.0,
        WidthRange::new(min_width, max_width),
    )
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
