use crate::app::state::{AppState, FormatPickerKind, FormatPickerViewMode, SubtitlePickerTab};
use eframe::egui::{self, Ui};
use egui_taffy::{Tui, TuiBuilderLogic as _, taffy};

use super::common::UiText;
use super::xaml_template_renderer::TemplateBlockSlot;
use super::{
    format_picker_filters, format_picker_selection, semantic_ui_metrics, xaml_layout_contracts,
    xaml_taffy_styles,
};

pub(super) struct FormatPickerHeaderContext {
    row_contract: xaml_layout_contracts::SingleLineControlRowContract,
    gap: f32,
    back_text: &'static str,
    confirm_text: &'static str,
    pending_selection: Option<String>,
    selection_summary: Option<String>,
    back_cell_style: taffy::Style,
    center_cell_style: taffy::Style,
    summary_cell_style: Option<taffy::Style>,
    confirm_cell_style: taffy::Style,
}

impl FormatPickerHeaderContext {
    pub(super) fn resolve(ui: &Ui, state: &AppState) -> Self {
        let pending_selection = format_picker_selection::pending_selection_id(state);
        let selection_summary = format_picker_selection::pending_selection_summary(state);
        let row_contract =
            semantic_ui_metrics::xaml_format_picker_header_row_contract_from_current_control_metrics(
                ui,
            );
        let gap = ui.spacing().item_spacing.x;
        let back_text = state.ui_i18n_text_for_key(UiText::BACK_TO_MAIN);
        let confirm_text = state.ui_i18n_text_for_key(UiText::CONFIRM);
        let back_element =
            semantic_ui_metrics::xaml_button_ui_element_for_visible_text(ui, back_text);
        let confirm_element =
            semantic_ui_metrics::xaml_button_ui_element_for_visible_text(ui, confirm_text);
        let center_element = format_picker_header_center_ui_element(ui, state, row_contract, gap);
        let summary_element = selection_summary.as_deref().map(|summary| {
            xaml_layout_contracts::UiElement::label(xaml_layout_contracts::LayoutSize::new(
                semantic_ui_metrics::format_picker_header_summary_width_for_visible_text(
                    ui, summary,
                ),
                row_contract.height,
            ))
        });
        let (_, back_cell_style) =
            xaml_taffy_styles::xaml_auto_width_cell_style(row_contract, back_element);
        let (_, center_cell_style) =
            xaml_taffy_styles::xaml_auto_width_cell_style(row_contract, center_element);
        let summary_cell_style = summary_element.map(|element| {
            xaml_taffy_styles::xaml_shrinkable_auto_width_cell_style(row_contract, element).1
        });
        let (_, confirm_cell_style) =
            xaml_taffy_styles::xaml_auto_width_cell_style(row_contract, confirm_element);

        Self {
            row_contract,
            gap,
            back_text,
            confirm_text,
            pending_selection,
            selection_summary,
            back_cell_style,
            center_cell_style,
            summary_cell_style,
            confirm_cell_style,
        }
    }

    pub(super) fn height(&self) -> f32 {
        self.row_contract.height
    }
}

pub(super) fn show_header_block(
    _slot: TemplateBlockSlot,
    tui: &mut Tui,
    state: &mut AppState,
    header: &FormatPickerHeaderContext,
) {
    tui.style(xaml_taffy_styles::xaml_fixed_height_row_style(
        header.row_contract,
        header.gap,
    ))
    .add(|tui| show_header_row(tui, state, header));
}

fn show_header_row(tui: &mut Tui, state: &mut AppState, header: &FormatPickerHeaderContext) {
    tui.style(header.back_cell_style.clone()).ui(|ui| {
        ui.centered_and_justified(|ui| {
            if ui.button(header.back_text).clicked() {
                state.cancel_format_picker();
            }
        });
    });
    tui.style(header.center_cell_style.clone()).ui(|ui| {
        ui.centered_and_justified(|ui| {
            render_header_center(ui, state);
        });
    });
    tui.style(xaml_taffy_styles::xaml_flex_spacer_cell_style(
        header.row_contract,
    ))
    .ui(|_| {});
    if let Some(summary_cell_style) = header.summary_cell_style.clone() {
        tui.style(summary_cell_style).ui(|ui| {
            if let Some(summary) = header.selection_summary.as_deref() {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(summary).color(ui.visuals().weak_text_color()));
                });
            }
        });
    }
    tui.style(header.confirm_cell_style.clone()).ui(|ui| {
        ui.centered_and_justified(|ui| {
            if ui
                .add_enabled(
                    header.pending_selection.is_some(),
                    egui::Button::new(header.confirm_text),
                )
                .clicked()
            {
                if let Some(format_id) = header.pending_selection.as_deref() {
                    state.confirm_format_picker_selection(format_id);
                }
            }
        });
    });
}

fn format_picker_header_center_ui_element(
    ui: &Ui,
    state: &AppState,
    row_contract: xaml_layout_contracts::SingleLineControlRowContract,
    gap: f32,
) -> xaml_layout_contracts::UiElement {
    let Some(kind) = state.format_picker.kind else {
        return xaml_layout_contracts::UiElement::spacer(0.0, row_contract);
    };

    match kind {
        FormatPickerKind::Video | FormatPickerKind::Audio => {
            let filter_text = state.ui_i18n_text_for_key(UiText::PICKER_MODE_FILTER);
            let table_text = state.ui_i18n_text_for_key(UiText::PICKER_MODE_TABLE);
            let measured_group = row_contract.measure_auto_width_ui_element_sequence(
                [
                    semantic_ui_metrics::xaml_selectable_button_ui_element_for_visible_text(
                        ui,
                        filter_text,
                    ),
                    semantic_ui_metrics::xaml_selectable_button_ui_element_for_visible_text(
                        ui, table_text,
                    ),
                ],
                gap,
            );
            xaml_layout_contracts::UiElement::fixed_width_stretch_height(
                measured_group.width,
                row_contract,
            )
        }
        FormatPickerKind::Subtitle => {
            let none_text = state.ui_i18n_text_for_key(SubtitlePickerTab::None.label_key());
            let original_text = state.ui_i18n_text_for_key(SubtitlePickerTab::Original.label_key());
            let automatic_text =
                state.ui_i18n_text_for_key(SubtitlePickerTab::Automatic.label_key());
            let measured_group = row_contract.measure_auto_width_ui_element_sequence(
                [
                    semantic_ui_metrics::xaml_selectable_button_ui_element_for_visible_text(
                        ui, none_text,
                    ),
                    semantic_ui_metrics::xaml_selectable_button_ui_element_for_visible_text(
                        ui,
                        original_text,
                    ),
                    semantic_ui_metrics::xaml_selectable_button_ui_element_for_visible_text(
                        ui,
                        automatic_text,
                    ),
                ],
                gap,
            );
            xaml_layout_contracts::UiElement::fixed_width_stretch_height(
                measured_group.width,
                row_contract,
            )
        }
        FormatPickerKind::Section => {
            let title = state.ui_i18n_text_for_key(UiText::SELECT_SECTION_TITLE);
            xaml_layout_contracts::UiElement::label(xaml_layout_contracts::LayoutSize::new(
                semantic_ui_metrics::format_picker_header_center_title_width_for_visible_text(
                    ui, title,
                ),
                row_contract.height,
            ))
        }
    }
}

fn render_header_center(ui: &mut Ui, state: &mut AppState) {
    let Some(kind) = state.format_picker.kind else {
        return;
    };

    if matches!(kind, FormatPickerKind::Video | FormatPickerKind::Audio) {
        let previous_mode = state.format_picker.view_mode;
        let picker_mode_filter = state.ui_i18n_text_for_key(UiText::PICKER_MODE_FILTER);
        let picker_mode_table = state.ui_i18n_text_for_key(UiText::PICKER_MODE_TABLE);
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
            format_picker_filters::sync_picker_mode(state);
        }
        return;
    }

    if kind == FormatPickerKind::Subtitle {
        render_subtitle_header_tabs(ui, state);
        return;
    }

    let title = match kind {
        FormatPickerKind::Video => UiText::SELECT_VIDEO_TITLE,
        FormatPickerKind::Audio => UiText::SELECT_AUDIO_TITLE,
        FormatPickerKind::Subtitle => UiText::SELECT_SUBTITLE_TITLE,
        FormatPickerKind::Section => UiText::SELECT_SECTION_TITLE,
    };
    ui.label(state.ui_i18n_text_for_key(title));
}

fn render_subtitle_header_tabs(ui: &mut Ui, state: &mut AppState) {
    let none_label = state.ui_i18n_text_for_key(SubtitlePickerTab::None.label_key());
    let original_label = state.ui_i18n_text_for_key(SubtitlePickerTab::Original.label_key());
    let automatic_label = state.ui_i18n_text_for_key(SubtitlePickerTab::Automatic.label_key());

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
