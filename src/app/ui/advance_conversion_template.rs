use eframe::egui::Ui;
use egui_taffy::Tui;

use crate::app::state::AppState;

use super::semantic_ui_metrics;
use super::settings_detail_template::{
    SettingsDetailNode, render_settings_detail_header, render_settings_detail_page,
};
use super::xaml_template_renderer::{
    TemplateBlockSlot, show_template_tui_block, show_template_ui_block,
};

pub(super) fn render_download_conversion_detail_page(ui: &mut Ui, state: &mut AppState) {
    let mut show_block = |slot, node, tui: &mut Tui| {
        show_conversion_detail_block(slot, node, tui, state);
    };
    render_settings_detail_page(
        ui,
        "advance-download-conversion-page-scroll",
        "advance-download-conversion-detail-taffy",
        semantic_ui_metrics::download_conversion_detail_header_to_body_vertical_spacing(),
        &mut show_block,
    );
}

fn show_conversion_detail_block(
    slot: TemplateBlockSlot,
    node: SettingsDetailNode,
    tui: &mut Tui,
    state: &mut AppState,
) {
    match node {
        SettingsDetailNode::Header => {
            show_template_ui_block(slot, tui, |ui| render_header(ui, state))
        }
        SettingsDetailNode::Body => show_template_tui_block(slot, tui, |tui| {
            super::processing_conversion_template::render_processing_settings_content_tui(
                tui, state,
            );
        }),
    }
}

fn render_header(ui: &mut Ui, state: &mut AppState) {
    render_settings_detail_header(
        ui,
        state,
        "advance.download_conversion",
        AppState::close_advance_detail_page,
    );
}
