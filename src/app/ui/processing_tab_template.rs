use crate::app::state::AppState;
use eframe::egui::{self, Ui};

use super::xaml_rect_template::{show_rect_template, show_ui_at_rect};
use super::xaml_ui_nodes::{TemplateAxis, TemplateNode, block, fill, rows};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProcessingTabNode {
    LogViewer,
}

type ProcessingTabTemplate = TemplateNode<ProcessingTabNode>;

pub(super) fn render_log_tab(ui: &mut Ui, state: &mut AppState) {
    let available = ui.available_size();
    if available.x <= 0.0 || available.y <= 0.0 {
        return;
    }

    let (root_rect, _) = ui.allocate_exact_size(available, egui::Sense::hover());
    let mut auto_main_size = |_: &ProcessingTabNode, _: TemplateAxis, _: f32| 0.0;
    let mut show_block = |node: ProcessingTabNode, rect: egui::Rect| match node {
        ProcessingTabNode::LogViewer => show_ui_at_rect(ui, rect, |ui| {
            super::processing_log_viewer::render_log_tab(ui, state);
        }),
    };

    show_rect_template(
        root_rect,
        processing_tab_template(),
        &mut auto_main_size,
        &mut show_block,
    );
}

fn processing_tab_template() -> ProcessingTabTemplate {
    use ProcessingTabNode::*;

    rows([fill(block(LogViewer))])
}
