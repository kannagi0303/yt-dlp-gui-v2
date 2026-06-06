use crate::app::state::{AppState, FormatPickerKind, FormatPickerViewMode};
use crate::domain::FormatOption;
use eframe::egui::Ui;
use egui_taffy::{Tui, TuiBuilderLogic as _};

use super::common::UiText;
use super::xaml_template_renderer::TemplateBlockSlot;
use super::{
    format_picker_filters, format_picker_section, format_picker_subtitle, format_picker_table,
    semantic_ui_metrics, xaml_taffy_styles,
};

pub(super) fn show_content_block(slot: TemplateBlockSlot, tui: &mut Tui, state: &mut AppState) {
    match slot {
        TemplateBlockSlot::Root => {
            xaml_taffy_styles::XamlTaffyElement::grow_block()
                .show_fill_ui(tui, |ui| show_format_picker_contents(ui, state));
        }
        TemplateBlockSlot::Child { style, .. } => {
            tui.style(style)
                .ui(|ui| show_format_picker_contents(ui, state));
        }
    }
}

fn show_format_picker_contents(ui: &mut Ui, state: &mut AppState) {
    ui.separator();
    render_format_picker_contents(ui, state);
}

fn render_format_picker_contents(ui: &mut Ui, state: &mut AppState) {
    let Some(kind) = state.format_picker.kind else {
        state.cancel_format_picker();
        return;
    };

    if render_special_picker_contents(ui, state, kind) {
        return;
    }

    render_format_or_audio_picker_contents(ui, state, kind);
}

fn render_special_picker_contents(
    ui: &mut Ui,
    state: &mut AppState,
    kind: FormatPickerKind,
) -> bool {
    match kind {
        FormatPickerKind::Subtitle => {
            format_picker_subtitle::render_subtitle_picker_contents(ui, state);
            true
        }
        FormatPickerKind::Section => {
            format_picker_section::render_section_picker_contents(ui, state);
            true
        }
        _ => false,
    }
}

fn render_format_or_audio_picker_contents(
    ui: &mut Ui,
    state: &mut AppState,
    kind: FormatPickerKind,
) {
    let options = state.format_picker_options(kind);
    let filtered_rows =
        format_picker_filters::filtered_rows(&options, &state.format_picker.filters);

    sanitize_selected_row(state, options.len());
    auto_select_single_filtered_row(state, &filtered_rows);

    match state.format_picker.view_mode {
        FormatPickerViewMode::Filter => {
            render_filter_view(ui, state, kind, &options, &filtered_rows)
        }
        FormatPickerViewMode::Table => render_table_view(ui, state, kind, &options),
    }
}

fn sanitize_selected_row(state: &mut AppState, options_len: usize) {
    if let Some(selected_row) = state.format_picker.selected_row {
        if selected_row >= options_len {
            state.format_picker.selected_row = None;
        }
    }
}

fn auto_select_single_filtered_row(state: &mut AppState, filtered_rows: &[usize]) {
    if state.format_picker.view_mode == FormatPickerViewMode::Filter && filtered_rows.len() == 1 {
        state.format_picker.selected_row = filtered_rows.first().copied();
    }
}

fn render_filter_view(
    ui: &mut Ui,
    state: &mut AppState,
    kind: FormatPickerKind,
    options: &[FormatOption],
    filtered_rows: &[usize],
) {
    if filtered_rows.is_empty() {
        show_empty_table_message(ui, state);
    }

    format_picker_filters::render_format_picker_filters(ui, state, kind, options);
}

fn render_table_view(
    ui: &mut Ui,
    state: &mut AppState,
    kind: FormatPickerKind,
    options: &[FormatOption],
) {
    if options.is_empty() {
        show_empty_table_message(ui, state);
    } else {
        format_picker_table::render_format_picker_table(ui, state, kind, options);
    }
}

fn show_empty_table_message(ui: &mut Ui, state: &AppState) {
    ui.add_space(semantic_ui_metrics::format_picker_empty_message_top_vertical_spacing());
    ui.label(state.ui_i18n_text_for_key(UiText::EMPTY_TABLE));
}
