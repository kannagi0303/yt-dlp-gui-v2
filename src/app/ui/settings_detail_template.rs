use eframe::egui::{RichText, ScrollArea, Ui};
use egui_taffy::{Tui, TuiBuilderLogic as _, tui};

use crate::app::state::AppState;

use super::common::{UiText, settings_taffy_scroll_content};
use super::xaml_template_renderer::{TemplateBlockSlot, show_auto_height_template};
use super::xaml_ui_nodes::{TemplateNode, auto, block, gap, rows};
use super::{semantic_ui_metrics, xaml_layout_contracts, xaml_taffy_styles};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SettingsDetailNode {
    Header,
    Body,
}

type SettingsDetailTemplate = TemplateNode<SettingsDetailNode>;

pub(super) fn render_settings_detail_page(
    ui: &mut Ui,
    scroll_id: &'static str,
    taffy_id: &'static str,
    header_to_body_gap: f32,
    show_block: &mut impl FnMut(TemplateBlockSlot, SettingsDetailNode, &mut Tui),
) {
    ScrollArea::vertical()
        .id_salt(scroll_id)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            settings_taffy_scroll_content(ui, taffy_id, |tui| {
                show_auto_height_template(
                    settings_detail_template(header_to_body_gap),
                    tui,
                    show_block,
                );
            });
        });
}

fn settings_detail_template(header_to_body_gap: f32) -> SettingsDetailTemplate {
    rows([
        auto(block(SettingsDetailNode::Header)),
        gap(header_to_body_gap),
        auto(block(SettingsDetailNode::Body)),
    ])
}

pub(super) fn render_settings_detail_header(
    ui: &mut Ui,
    state: &mut AppState,
    title_key: &'static str,
    close_detail_page: fn(&mut AppState),
) {
    let row_contract =
        semantic_ui_metrics::xaml_format_picker_header_row_contract_from_current_control_metrics(
            ui,
        );
    let gap = ui.spacing().item_spacing.x;
    let back_text = state.ui_i18n_text_for_key(UiText::BACK_TO_MAIN);
    let confirm_text = state.ui_i18n_text_for_key(UiText::CONFIRM);
    let title_text = state.ui_i18n_text_for_key(title_key);

    let back_element = semantic_ui_metrics::xaml_button_ui_element_for_visible_text(ui, back_text);
    let title_element =
        xaml_layout_contracts::UiElement::label(xaml_layout_contracts::LayoutSize::new(
            semantic_ui_metrics::format_picker_header_center_title_width_for_visible_text(
                ui, title_text,
            ),
            row_contract.height,
        ));
    let confirm_element =
        semantic_ui_metrics::xaml_button_ui_element_for_visible_text(ui, confirm_text);

    let (_, back_cell_style) =
        xaml_taffy_styles::xaml_auto_width_cell_style(row_contract, back_element);
    let (_, title_cell_style) =
        xaml_taffy_styles::xaml_auto_width_cell_style(row_contract, title_element);
    let (_, confirm_cell_style) =
        xaml_taffy_styles::xaml_auto_width_cell_style(row_contract, confirm_element);

    let mut back_clicked = false;
    let mut confirm_clicked = false;
    let available_width = ui.available_width();
    tui(ui, ui.id().with(("settings-detail-header", title_key)))
        .reserve_width(available_width)
        .reserve_height(row_contract.height)
        .style(xaml_taffy_styles::xaml_fixed_height_row_style(
            row_contract,
            gap,
        ))
        .show(|tui| {
            tui.style(back_cell_style).ui(|ui| {
                ui.centered_and_justified(|ui| {
                    back_clicked = ui.button(back_text).clicked();
                });
            });
            tui.style(title_cell_style).ui(|ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new(title_text).strong());
                });
            });
            tui.style(xaml_taffy_styles::xaml_flex_spacer_cell_style(row_contract))
                .ui(|_| {});
            tui.style(confirm_cell_style).ui(|ui| {
                ui.centered_and_justified(|ui| {
                    confirm_clicked = ui.button(confirm_text).clicked();
                });
            });
        });

    if back_clicked || confirm_clicked {
        close_detail_page(state);
    }
}
