use crate::app::state::AppState;
use egui_taffy::Tui;

use super::common::settings_taffy_form_row;
use super::processing_conversion::{
    ConversionNode, conversion_label, conversion_label_width, conversion_nodes,
    render_conversion_choices,
};
use super::xaml_template_renderer::show_auto_height_tui_template;
use super::xaml_ui_nodes::{TemplateNode, auto_block_rows};

pub(super) fn render_processing_settings_content_tui(tui: &mut Tui, state: &mut AppState) {
    // The enable switch lives in Advance > Post-processing; this page only edits conversion details.
    let mut settings = state.config.transcode_intent.clone();
    let before = settings.clone();
    let label_width = conversion_label_width(tui.egui_ui(), state);
    {
        let mut show_block = |node: ConversionNode, tui: &mut Tui| {
            let label = conversion_label(state, node);
            settings_taffy_form_row(tui, label_width, label.as_str(), |ui| {
                render_conversion_choices(ui, state, &mut settings, node);
            });
        };

        show_auto_height_tui_template(conversion_template(), tui, &mut show_block);
    }

    if before != settings {
        state.set_transcode_intent(settings);
    }
}

fn conversion_template() -> TemplateNode<ConversionNode> {
    auto_block_rows(conversion_nodes())
}
