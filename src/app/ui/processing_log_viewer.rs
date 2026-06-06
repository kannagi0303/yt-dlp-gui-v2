use crate::app::state::{AppState, ToolLogStep};
use eframe::egui::{self, Ui};

use super::processing_command_viewer;
use super::processing_log_table;
use super::semantic_ui_metrics;
use super::xaml_rect_template::{show_rect_template, show_ui_at_rect};
use super::xaml_ui_nodes::{TemplateAxis, block, fill, fixed_px, gap, rows};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProcessingLogNode {
    Header,
    LogTable,
    CommandViewer,
}

pub(super) fn render_log_tab(ui: &mut Ui, state: &mut AppState) {
    ensure_valid_log_selection(state);

    let available = ui.available_size();
    if available.x <= 0.0 || available.y <= 0.0 {
        return;
    }

    let command_height = command_viewer_height(ui, state, available.x, available.y);
    let command_gap = command_viewer_gap(command_height);
    let header_height = semantic_ui_metrics::processing_log_table_header_row_height();
    let widths = processing_log_table::measure_log_table_widths(ui, state);
    let layout = processing_log_table::log_table_layout(&widths, available.x);
    let (root_rect, _) = ui.allocate_exact_size(available, egui::Sense::hover());
    let template = processing_log_template(header_height, command_gap, command_height);
    let mut auto_main_size = |_: &ProcessingLogNode, _: TemplateAxis, _: f32| 0.0;
    let mut show_block = |node: ProcessingLogNode, rect: egui::Rect| {
        show_processing_log_block(ui, state, node, rect, &layout);
    };

    show_rect_template(root_rect, template, &mut auto_main_size, &mut show_block);
}

fn processing_log_template(
    header_height: f32,
    command_gap: f32,
    command_height: f32,
) -> super::xaml_ui_nodes::TemplateNode<ProcessingLogNode> {
    let mut children = vec![
        fixed_px(header_height, block(ProcessingLogNode::Header)),
        fill(block(ProcessingLogNode::LogTable)),
    ];
    if command_height > 0.0 {
        children.push(gap(command_gap));
        children.push(fixed_px(
            command_height,
            block(ProcessingLogNode::CommandViewer),
        ));
    }
    rows(children)
}

fn show_processing_log_block(
    ui: &mut Ui,
    state: &mut AppState,
    node: ProcessingLogNode,
    rect: egui::Rect,
    layout: &processing_log_table::LogTableLayout,
) {
    show_ui_at_rect(ui, rect, |ui| match node {
        ProcessingLogNode::Header => processing_log_table::render_log_table_header(ui, layout),
        ProcessingLogNode::LogTable => processing_log_table::render_log_table(ui, state, layout),
        ProcessingLogNode::CommandViewer => {
            if let Some(step) = selected_tool_log_step(state) {
                processing_command_viewer::render_command_viewer(ui, step);
            }
        }
    });
}

fn command_viewer_height(
    ui: &Ui,
    state: &AppState,
    available_width: f32,
    available_height: f32,
) -> f32 {
    let header_height = semantic_ui_metrics::processing_log_table_header_row_height();
    let raw_height = processing_command_viewer::command_viewer_height(
        ui,
        selected_tool_log_step(state),
        available_width,
        available_height,
    );
    if raw_height <= 0.0 {
        return 0.0;
    }
    let command_gap =
        semantic_ui_metrics::processing_command_viewer_to_log_table_vertical_spacing();
    raw_height.min((available_height - header_height - command_gap).max(0.0))
}

fn command_viewer_gap(command_height: f32) -> f32 {
    if command_height > 0.0 {
        semantic_ui_metrics::processing_command_viewer_to_log_table_vertical_spacing()
    } else {
        0.0
    }
}

fn ensure_valid_log_selection(state: &mut AppState) {
    if let Some(selected_step) = state.log_viewer_selected_step {
        if selected_tool_log_step(state).is_none() {
            state.log_viewer_selected_step = None;
        }
        if !state
            .tool_logs
            .iter()
            .any(|action| action.steps.iter().any(|step| step.id == selected_step))
        {
            state.log_viewer_selected_step = None;
        }
    }

    if let Some(action_id) = state.log_viewer_expanded_action {
        if !state.tool_logs.iter().any(|action| action.id == action_id) {
            state.log_viewer_expanded_action = None;
        }
    }
}

fn selected_tool_log_step(state: &AppState) -> Option<&ToolLogStep> {
    let selected_step = state.log_viewer_selected_step?;
    state
        .tool_logs
        .iter()
        .flat_map(|action| action.steps.iter())
        .find(|step| step.id == selected_step)
}
