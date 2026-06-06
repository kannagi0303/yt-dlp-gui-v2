use crate::app::state::{AppState, ToolLogAction, ToolLogStatus, ToolLogStep};
use eframe::egui::{self, Color32, ScrollArea, Ui};

use super::semantic_ui_metrics;

#[derive(Clone, Copy, Debug)]
pub(super) struct LogTableWidths {
    time: f32,
    status: f32,
    action: f32,
    mode: f32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LogTableLayout {
    time: f32,
    status: f32,
    mode: f32,
    action: f32,
}

pub(super) fn render_log_table(ui: &mut Ui, state: &mut AppState, layout: &LogTableLayout) {
    ScrollArea::vertical()
        .id_salt("log-action-table-scroll")
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            let actions = state.tool_logs.iter().cloned().collect::<Vec<_>>();
            if actions.is_empty() {
                render_empty_log_row(ui, "No command logs yet.");
                return;
            }

            for action in actions.iter() {
                render_tool_action_row(ui, state, action, layout);
                if state.log_viewer_expanded_action == Some(action.id) {
                    render_tool_action_steps(ui, state, action, layout);
                }
            }
        });
}

pub(super) fn render_log_table_header(ui: &mut Ui, layout: &LogTableLayout) {
    let row_height = semantic_ui_metrics::processing_log_table_header_row_height();
    let desired = egui::vec2(ui.available_width(), row_height);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let visuals = ui.visuals();

    ui.painter().rect_filled(
        rect,
        semantic_ui_metrics::processing_log_table_background_corner_radius(),
        visuals.extreme_bg_color,
    );
    paint_row_bottom_line(ui, rect, subtle_line_color(ui));

    let mut x = rect.left();
    paint_cell_text_center(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.time, row_height),
        ),
        "Time",
        visuals.weak_text_color(),
        true,
        false,
    );
    x += layout.time;
    paint_cell_text_center(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.status, row_height),
        ),
        "Status",
        visuals.weak_text_color(),
        false,
        false,
    );
    x += layout.status;
    paint_cell_text_center(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.mode, row_height),
        ),
        "Mode",
        visuals.weak_text_color(),
        false,
        false,
    );
    x += layout.mode;
    paint_cell_text(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.action, row_height),
        ),
        "Action",
        visuals.weak_text_color(),
        false,
        false,
    );
}

fn render_tool_action_row(
    ui: &mut Ui,
    state: &mut AppState,
    action: &ToolLogAction,
    layout: &LogTableLayout,
) {
    let expanded = state.log_viewer_expanded_action == Some(action.id);
    let selected = expanded;
    let status = action.status;
    let row_height = semantic_ui_metrics::processing_log_table_action_row_height();
    let desired = egui::vec2(ui.available_width(), row_height);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());

    if response.clicked() {
        state.log_viewer_selected_step = None;
        if expanded {
            state.log_viewer_expanded_action = None;
        } else {
            state.log_viewer_expanded_action = Some(action.id);
        }
    }

    paint_table_row_background(ui, rect, selected, response.hovered());
    paint_row_bottom_line(ui, rect, subtle_line_color(ui));

    let visuals = ui.visuals();
    let mut x = rect.left();
    paint_cell_text_center(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.time, row_height),
        ),
        action.timestamp.as_str(),
        visuals.weak_text_color(),
        true,
        false,
    );
    x += layout.time;
    paint_status_cell(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.status, row_height),
        ),
        status,
    );
    x += layout.status;
    paint_cell_text_center(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.mode, row_height),
        ),
        action.mode.as_str(),
        log_parent_mode_color(ui),
        false,
        true,
    );
    x += layout.mode;
    let action_title = format!("{} {}", if expanded { "▾" } else { "▸" }, action.action);
    paint_cell_text(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.action, row_height),
        ),
        &action_title,
        visuals.text_color(),
        false,
        false,
    );
}

fn render_tool_action_steps(
    ui: &mut Ui,
    state: &mut AppState,
    action: &ToolLogAction,
    layout: &LogTableLayout,
) {
    if action.steps.is_empty() {
        return;
    }

    let row_height = semantic_ui_metrics::processing_log_table_step_row_height();
    let bottom_gap = row_height;
    let chain_height = row_height * action.steps.len() as f32 + bottom_gap;
    let desired = egui::vec2(ui.available_width(), chain_height);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    paint_row_bottom_line(ui, rect, subtle_line_color(ui));

    for (index, step) in action.steps.iter().enumerate() {
        let top = rect.top() + row_height * index as f32;
        let row_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left(), top),
            egui::pos2(rect.right(), top + row_height),
        );
        render_chain_text_step(ui, state, action, step, row_rect, layout);
    }
}

fn render_chain_text_step(
    ui: &mut Ui,
    state: &mut AppState,
    action: &ToolLogAction,
    step: &ToolLogStep,
    rect: egui::Rect,
    layout: &LogTableLayout,
) {
    let selected = state.log_viewer_selected_step == Some(step.id);
    let response = ui.interact(
        rect,
        ui.id().with(("log-chain-text-step", action.id, step.id)),
        egui::Sense::click(),
    );

    if response.clicked() {
        state.log_viewer_selected_step = Some(step.id);
    }

    paint_table_row_background(ui, rect, selected, response.hovered());

    let mut x = rect.left();
    x += layout.time;

    paint_status_cell(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.status, rect.height()),
        ),
        step.status,
    );
    x += layout.status;

    paint_cell_text_center(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.mode, rect.height()),
        ),
        step.tool.as_str(),
        ui.visuals().weak_text_color(),
        true,
        false,
    );
    x += layout.mode;

    let step_label = format!("› {}", step.action);
    paint_cell_text(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(layout.action, rect.height()),
        ),
        &step_label,
        ui.visuals().text_color(),
        false,
        false,
    );
}

fn render_empty_log_row(ui: &mut Ui, entry: &str) {
    let row_height = semantic_ui_metrics::processing_log_table_parent_row_height();
    let desired = egui::vec2(ui.available_width(), row_height);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::hover());
    paint_table_row_background(ui, rect, false, response.hovered());
    paint_row_bottom_line(ui, rect, subtle_line_color(ui));
    paint_cell_text(
        ui,
        rect,
        entry,
        ui.visuals().weak_text_color(),
        false,
        false,
    );
}

fn paint_status_cell(ui: &Ui, rect: egui::Rect, status: ToolLogStatus) {
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        status_symbol(status),
        egui::FontId::proportional(
            semantic_ui_metrics::processing_log_table_status_icon_font_size(),
        ),
        status_color(ui, status),
    );
}

fn paint_cell_text(
    ui: &Ui,
    rect: egui::Rect,
    text: &str,
    color: Color32,
    monospace: bool,
    strong: bool,
) {
    paint_cell_text_with_align(
        ui,
        rect,
        text,
        color,
        monospace,
        strong,
        egui::Align2::LEFT_CENTER,
    );
}

fn paint_cell_text_center(
    ui: &Ui,
    rect: egui::Rect,
    text: &str,
    color: Color32,
    monospace: bool,
    strong: bool,
) {
    paint_cell_text_with_align(
        ui,
        rect,
        text,
        color,
        monospace,
        strong,
        egui::Align2::CENTER_CENTER,
    );
}

fn paint_cell_text_with_align(
    ui: &Ui,
    rect: egui::Rect,
    text: &str,
    color: Color32,
    monospace: bool,
    strong: bool,
    align: egui::Align2,
) {
    let pos = if align == egui::Align2::CENTER_CENTER {
        rect.center()
    } else {
        let x = semantic_ui_metrics::processing_log_table_text_x_for_alignment(rect);
        egui::pos2(x, rect.center().y)
    };
    let size = semantic_ui_metrics::processing_log_table_text_font_size(strong);
    let font = if monospace {
        egui::FontId::monospace(size)
    } else {
        egui::FontId::proportional(size)
    };
    ui.painter()
        .with_clip_rect(semantic_ui_metrics::processing_log_table_text_clip_rect(
            rect,
        ))
        .text(pos, align, text, font, color);
}

fn paint_table_row_background(ui: &Ui, rect: egui::Rect, selected: bool, hovered: bool) {
    let fill = if selected {
        selection_row_fill(ui)
    } else if hovered {
        hover_row_fill(ui)
    } else {
        Color32::TRANSPARENT
    };

    if fill != Color32::TRANSPARENT {
        ui.painter().rect_filled(
            rect,
            semantic_ui_metrics::processing_log_table_background_corner_radius(),
            fill,
        );
    }
}

fn paint_row_bottom_line(ui: &Ui, rect: egui::Rect, color: Color32) {
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(
            semantic_ui_metrics::processing_log_table_row_separator_stroke_width(),
            color,
        ),
    );
}

fn selection_row_fill(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(28, 45, 64)
    } else {
        Color32::from_rgb(218, 235, 255)
    }
}

fn hover_row_fill(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(32, 34, 36)
    } else {
        Color32::from_rgb(244, 247, 250)
    }
}

fn subtle_line_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_gray(42)
    } else {
        Color32::from_gray(218)
    }
}

fn log_parent_mode_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(156, 220, 254)
    } else {
        Color32::from_rgb(0, 102, 180)
    }
}

fn status_symbol(status: ToolLogStatus) -> &'static str {
    match status {
        ToolLogStatus::Running => "…",
        ToolLogStatus::Success => "✓",
        ToolLogStatus::Recovered => "↺",
        ToolLogStatus::Failed => "✕",
        ToolLogStatus::Skipped => "·",
    }
}

fn status_color(ui: &Ui, status: ToolLogStatus) -> egui::Color32 {
    match status {
        ToolLogStatus::Running => {
            if ui.visuals().dark_mode {
                egui::Color32::from_rgb(156, 220, 254)
            } else {
                egui::Color32::from_rgb(0, 102, 180)
            }
        }
        ToolLogStatus::Success => egui::Color32::from_rgb(70, 160, 110),
        ToolLogStatus::Recovered => egui::Color32::from_rgb(190, 145, 70),
        ToolLogStatus::Failed => egui::Color32::from_rgb(210, 80, 80),
        ToolLogStatus::Skipped => {
            if ui.visuals().dark_mode {
                egui::Color32::from_gray(160)
            } else {
                egui::Color32::from_gray(90)
            }
        }
    }
}

pub(super) fn measure_log_table_widths(ui: &Ui, state: &AppState) -> LogTableWidths {
    LogTableWidths {
        time: semantic_ui_metrics::processing_log_table_time_column_width_for_visible_timestamps(
            ui,
            state
                .tool_logs
                .iter()
                .map(|action| action.timestamp.as_str()),
        ),
        status: semantic_ui_metrics::processing_log_table_status_column_width(ui),
        action: semantic_ui_metrics::processing_log_table_action_column_width_for_visible_actions(
            ui,
            state.tool_logs.iter().flat_map(|action| {
                std::iter::once(action.action.as_str())
                    .chain(action.steps.iter().map(|step| step.action.as_str()))
            }),
        ),
        mode: semantic_ui_metrics::processing_log_table_mode_column_width(ui),
    }
}

pub(super) fn log_table_layout(widths: &LogTableWidths, available_width: f32) -> LogTableLayout {
    let fixed = widths.time + widths.status + widths.mode;
    let action = (available_width - fixed).max(widths.action);

    LogTableLayout {
        time: widths.time,
        status: widths.status,
        mode: widths.mode,
        action,
    }
}
