use crate::app::state::{AppState, ToolLogAction, ToolLogStatus, ToolLogStep};
use crate::infrastructure::{
    AudioPolicy, ContainerPolicy, SubtitlePolicy, TranscodeIntentSettings, VideoCodecPolicy,
};
use eframe::egui::{
    self, Color32, FontId, RichText, ScrollArea, TextStyle, TextWrapMode, Ui, WidgetText,
};

use super::common::{form_row_label, measure_label_width};
use super::measure::{WidthRange, measured_column_width};

#[derive(Clone, Copy, Debug)]
struct LogTableWidths {
    time: f32,
    status: f32,
    action: f32,
    mode: f32,
}

#[derive(Clone, Copy, Debug)]
struct LogTableLayout {
    time: f32,
    status: f32,
    mode: f32,
    action: f32,
}

pub(super) fn render_log_tab(ui: &mut Ui, state: &mut AppState) {
    ensure_valid_log_selection(state);

    let available = ui.available_size();
    if available.x <= 0.0 || available.y <= 0.0 {
        return;
    }

    let command_height = command_viewer_height(ui, state, available.x, available.y);
    let header_height = 20.0;
    let command_gap = if command_height > 0.0 { 8.0 } else { 0.0 };
    let command_height = if command_height > 0.0 {
        command_height.min((available.y - header_height - command_gap).max(0.0))
    } else {
        0.0
    };

    let widths = measure_log_table_widths(ui, state);
    let layout = log_table_layout(&widths, available.x);
    let (root_rect, _) = ui.allocate_exact_size(available, egui::Sense::hover());

    let header_rect = egui::Rect::from_min_size(
        root_rect.min,
        egui::vec2(root_rect.width(), header_height.min(root_rect.height())),
    );
    let command_rect = if command_height > 0.0 {
        Some(egui::Rect::from_min_max(
            egui::pos2(root_rect.left(), root_rect.bottom() - command_height),
            root_rect.right_bottom(),
        ))
    } else {
        None
    };
    let table_bottom = command_rect
        .map(|rect| (rect.top() - command_gap).max(header_rect.bottom()))
        .unwrap_or(root_rect.bottom());
    let table_rect = egui::Rect::from_min_max(
        egui::pos2(root_rect.left(), header_rect.bottom()),
        egui::pos2(root_rect.right(), table_bottom),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(header_rect), |ui| {
        ui.set_width(header_rect.width());
        ui.set_height(header_rect.height());
        render_log_table_header(ui, state, &layout);
    });
    ui.scope_builder(egui::UiBuilder::new().max_rect(table_rect), |ui| {
        ui.set_width(table_rect.width());
        ui.set_height(table_rect.height());
        render_log_table(ui, state, &layout);
    });
    if let Some(command_rect) = command_rect {
        ui.scope_builder(egui::UiBuilder::new().max_rect(command_rect), |ui| {
            ui.set_width(command_rect.width());
            ui.set_height(command_rect.height());
            render_command_viewer(ui, state);
        });
    }
}

fn render_log_table(ui: &mut Ui, state: &mut AppState, layout: &LogTableLayout) {
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

fn render_log_table_header(ui: &mut Ui, _state: &AppState, layout: &LogTableLayout) {
    let row_height = 20.0;
    let desired = egui::vec2(ui.available_width(), row_height);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let visuals = ui.visuals();

    ui.painter()
        .rect_filled(rect, 0.0, visuals.extreme_bg_color);
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
    let row_height = 19.0;
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

    let row_height = 18.0;
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
    let row_height = 22.0;
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

fn render_log_section_separator(ui: &mut Ui, title: &str) {
    let row_height = 24.0;
    let desired = egui::vec2(ui.available_width(), row_height);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    paint_row_bottom_line(ui, rect, subtle_line_color(ui));
    paint_cell_text(ui, rect, title, ui.visuals().weak_text_color(), false, true);
}

fn paint_status_cell(ui: &Ui, rect: egui::Rect, status: ToolLogStatus) {
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        status_symbol(status),
        egui::FontId::proportional(14.0),
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
        let x = if rect.width() <= 12.0 {
            rect.left()
        } else {
            rect.left() + 6.0
        };
        egui::pos2(x, rect.center().y)
    };
    let size = if strong { 13.0 } else { 12.5 };
    let font = if monospace {
        egui::FontId::monospace(size)
    } else {
        egui::FontId::proportional(size)
    };
    ui.painter()
        .with_clip_rect(rect.shrink2(egui::vec2(3.0, 0.0)))
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
        ui.painter().rect_filled(rect, 0.0, fill);
    }
}

fn paint_row_bottom_line(ui: &Ui, rect: egui::Rect, color: Color32) {
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(1.0, color),
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

const COMMAND_FONT_SIZE: f32 = 13.0;
const COMMAND_TOKEN_SPACING_X: f32 = 5.0;
const COMMAND_TOKEN_SPACING_Y: f32 = 5.0;
const COMMAND_FRAME_MARGIN_X: f32 = 10.0;
const COMMAND_FRAME_MARGIN_Y: f32 = 7.0;

fn command_viewer_height(
    ui: &Ui,
    state: &AppState,
    available_width: f32,
    available_height: f32,
) -> f32 {
    let Some(step) = selected_tool_log_step(state) else {
        return 0.0;
    };

    let content_width = (available_width - COMMAND_FRAME_MARGIN_X * 2.0 - 2.0).max(1.0);
    let tokens = display_command_tokens(step.command.as_str());
    let content_height = measured_wrapped_command_height(ui, &tokens, content_width);
    let natural_height = content_height + COMMAND_FRAME_MARGIN_Y * 2.0 + 2.0;
    let max_height = available_height.max(0.0);
    natural_height.min(max_height)
}

fn measured_wrapped_command_height(ui: &Ui, tokens: &[String], max_width: f32) -> f32 {
    let line_height = command_line_height(ui);
    if tokens.is_empty() {
        return line_height;
    }

    let mut line_count = 1_usize;
    let mut line_width = 0.0_f32;

    for token in tokens {
        let token_width = command_token_width(ui, token);
        let next_width = if line_width <= 0.0 {
            token_width
        } else {
            line_width + COMMAND_TOKEN_SPACING_X + token_width
        };

        if line_width > 0.0 && next_width > max_width {
            line_count += 1;
            line_width = token_width;
        } else {
            line_width = next_width;
        }
    }

    line_height * line_count as f32 + COMMAND_TOKEN_SPACING_Y * line_count.saturating_sub(1) as f32
}

fn command_token_width(ui: &Ui, token: &str) -> f32 {
    WidgetText::from(command_layout_job(ui, token, f32::INFINITY))
        .into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Monospace,
        )
        .size()
        .x
}

fn command_line_height(ui: &Ui) -> f32 {
    WidgetText::from(command_layout_job(ui, "Hg", f32::INFINITY))
        .into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Monospace,
        )
        .size()
        .y
}

fn command_layout_job(ui: &Ui, text: &str, wrap_width: f32) -> egui::text::LayoutJob {
    egui::text::LayoutJob::simple(
        text.to_owned(),
        command_font_id(),
        ui.visuals().text_color(),
        wrap_width,
    )
}

fn command_font_id() -> FontId {
    FontId::monospace(COMMAND_FONT_SIZE)
}

fn render_command_viewer(ui: &mut Ui, state: &mut AppState) {
    let Some(step) = selected_tool_log_step(state) else {
        return;
    };

    let desired = ui.available_size();
    if desired.x <= 0.0 || desired.y <= 0.0 {
        return;
    }
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_width(rect.width());
        ui.set_height(rect.height());
        command_line_frame(ui, step.command.as_str());
    });
}

fn command_line_frame(ui: &mut Ui, text: &str) {
    egui::Frame::NONE
        .fill(command_viewer_panel_fill(ui))
        .stroke(egui::Stroke::new(1.0, subtle_line_color(ui)))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(
            COMMAND_FRAME_MARGIN_X as i8,
            COMMAND_FRAME_MARGIN_Y as i8,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(ui.available_height());
            render_highlighted_command(ui, text);
        });
}

fn command_viewer_panel_fill(ui: &Ui) -> Color32 {
    let base = if ui.visuals().dark_mode {
        ui.visuals().extreme_bg_color
    } else {
        ui.visuals().panel_fill
    };
    let alpha = if ui.visuals().dark_mode { 204 } else { 226 };
    Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha)
}

fn render_highlighted_command(ui: &mut Ui, command: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = COMMAND_TOKEN_SPACING_X;
        ui.spacing_mut().item_spacing.y = COMMAND_TOKEN_SPACING_Y;
        let tokens = display_command_tokens(command);
        for (index, token) in tokens.iter().enumerate() {
            let color = command_token_color(ui, index, token);
            ui.add(
                egui::Label::new(
                    RichText::new(token.as_str())
                        .font(command_font_id())
                        .color(color),
                )
                .sense(egui::Sense::hover()),
            );
        }
    });
}

fn display_command_tokens(command: &str) -> Vec<String> {
    let tokens = command_tokens(command);
    let mut visible = Vec::with_capacity(tokens.len());
    let mut index = 0;

    while index < tokens.len() {
        let token = &tokens[index];

        if is_hidden_command_option_with_inline_value(token)
            || is_hidden_command_standalone_token(token)
            || is_hidden_command_plumbing_value(token)
        {
            index += 1;
            continue;
        }

        if is_hidden_command_option_with_next_value(token) {
            index += 1;
            if index < tokens.len() {
                index += 1;
            }
            continue;
        }

        let display_token = if visible.is_empty() {
            display_tool_token(token)
        } else {
            token.clone()
        };
        visible.push(display_token);
        index += 1;
    }

    visible
}

fn display_tool_token(token: &str) -> String {
    let trimmed = token.trim_matches('"');
    let name = trimmed
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches(".exe");
    match name.to_ascii_lowercase().as_str() {
        "yt-dlp" | "yt_dlp" => "yt-dlp".to_owned(),
        "ffmpeg" => "ffmpeg".to_owned(),
        "ffprobe" => "ffprobe".to_owned(),
        _ => name.to_owned(),
    }
}

fn is_hidden_command_option_with_next_value(token: &str) -> bool {
    matches!(
        token,
        // App-managed infrastructure arguments. They remain in the raw command
        // for debug/export, but the default viewer hides them so the command
        // reads like the user's actual operation.
        "--ffmpeg-location"
            | "--js-runtimes"
            | "--cache-dir"
            | "--paths"
            | "--config-location"
            | "--config-locations"
            | "--progress-template"
            | "--print"
            | "-P"
    )
}

fn is_hidden_command_option_with_inline_value(token: &str) -> bool {
    token.starts_with("--ffmpeg-location=")
        || token.starts_with("--js-runtimes=")
        || token.starts_with("--cache-dir=")
        || token.starts_with("--paths=")
        || token.starts_with("--config-location=")
        || token.starts_with("--config-locations=")
        || token.starts_with("--progress-template=")
        || token.starts_with("--print=")
        || token.starts_with("-P=")
        || (token.starts_with("-P") && token.contains("temp:"))
}

fn is_hidden_command_standalone_token(token: &str) -> bool {
    matches!(token, "--progress" | "--newline")
}

fn is_hidden_command_plumbing_value(token: &str) -> bool {
    token.starts_with("deno:")
        || token.starts_with("temp:")
        || token == "<ffmpeg>"
        || token == "<deno>"
        || token.starts_with("[yt-dlp],")
        || token.starts_with("after_move:")
}

fn command_tokens(command: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in command.chars() {
        match quote {
            Some(active_quote) => {
                current.push(ch);
                if ch == active_quote {
                    quote = None;
                }
            }
            None if ch == '\'' || ch == '"' => {
                current.push(ch);
                quote = Some(ch);
            }
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            None => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn command_token_color(ui: &Ui, index: usize, token: &str) -> Color32 {
    // Shell-like palette: command names in blue, options in cyan, strings/targets in warm
    // tones, and placeholders muted.  This keeps the command readable without turning the
    // log viewer into a full terminal emulator.
    if index == 0 || matches!(token, "yt-dlp" | "ffmpeg" | "ffprobe") {
        return if ui.visuals().dark_mode {
            Color32::from_rgb(86, 156, 214)
        } else {
            Color32::from_rgb(0, 92, 160)
        };
    }

    if token.starts_with('-') {
        return if ui.visuals().dark_mode {
            Color32::from_rgb(156, 220, 254)
        } else {
            Color32::from_rgb(0, 102, 180)
        };
    }

    if token.starts_with('<') && token.ends_with('>') {
        return ui.visuals().weak_text_color();
    }

    if token.starts_with("http://") || token.starts_with("https://") {
        return if ui.visuals().dark_mode {
            Color32::from_rgb(206, 145, 120)
        } else {
            Color32::from_rgb(165, 85, 35)
        };
    }

    if token.contains('=') || token.contains(':') || token.contains('.') {
        return if ui.visuals().dark_mode {
            Color32::from_rgb(181, 206, 168)
        } else {
            Color32::from_rgb(67, 126, 55)
        };
    }

    ui.visuals().text_color()
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

fn status_symbol(status: ToolLogStatus) -> &'static str {
    match status {
        ToolLogStatus::Running => "…",
        ToolLogStatus::Success => "✓",
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

fn measure_log_table_widths(ui: &Ui, state: &AppState) -> LogTableWidths {
    let extra = ui.spacing().item_spacing.x * 2.0 + 12.0;
    LogTableWidths {
        time: measured_column_width(
            ui,
            "Time",
            state
                .tool_logs
                .iter()
                .map(|action| action.timestamp.as_str()),
            TextStyle::Body,
            extra,
            WidthRange::new(74.0, 96.0),
        ),
        status: measured_column_width(
            ui,
            "Status",
            ["✓", "✕", "·", "…"],
            TextStyle::Body,
            extra,
            WidthRange::new(36.0, 58.0),
        ),
        action: measured_column_width(
            ui,
            "Action",
            state.tool_logs.iter().flat_map(|action| {
                std::iter::once(action.action.as_str())
                    .chain(action.steps.iter().map(|step| step.action.as_str()))
            }),
            TextStyle::Body,
            extra,
            WidthRange::new(132.0, 320.0),
        ),
        mode: measured_column_width(
            ui,
            "Mode",
            [
                "origin", "audio", "normal", "yt-dlp", "ffmpeg", "ffprobe", "app",
            ],
            TextStyle::Body,
            extra,
            WidthRange::new(76.0, 124.0),
        ),
    }
}

fn log_table_layout(widths: &LogTableWidths, available_width: f32) -> LogTableLayout {
    let fixed = widths.time + widths.status + widths.mode;
    let action = (available_width - fixed).max(widths.action);

    LogTableLayout {
        time: widths.time,
        status: widths.status,
        mode: widths.mode,
        action,
    }
}

pub(super) fn render_processing_settings_content(ui: &mut Ui, state: &mut AppState) {
    // The enable switch lives in Advance > Post-processing; this page only edits conversion details.
    let mut settings = state.config.transcode_intent.clone();
    let before = settings.clone();

    render_post_download_conversion(ui, state, &mut settings);

    if before != settings {
        state.set_transcode_intent(settings);
    }
}

fn render_post_download_conversion(
    ui: &mut Ui,
    state: &mut AppState,
    settings: &mut TranscodeIntentSettings,
) {
    let video_text = state.ui_tr("processing.video");
    let audio_text = state.ui_tr("processing.audio");
    let container_text = state.ui_tr("processing.container");
    let subtitle_text = state.ui_tr("processing.subtitle");
    let labels = [video_text, audio_text, container_text, subtitle_text];
    let label_width = measure_label_width(ui, &labels);

    form_row_label(ui, label_width, video_text, |ui| {
        render_video_codec_choices(ui, state, settings);
    });
    form_row_label(ui, label_width, audio_text, |ui| {
        render_audio_choices(ui, state, settings);
    });
    form_row_label(ui, label_width, container_text, |ui| {
        render_container_choices(ui, state, settings);
    });
    form_row_label(ui, label_width, subtitle_text, |ui| {
        render_subtitle_choices(ui, state, settings);
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConversionField {
    Video,
    Audio,
    Container,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConversionCombination {
    video: VideoCodecPolicy,
    audio: AudioPolicy,
    container: ContainerPolicy,
}

impl ConversionCombination {
    fn from_settings(settings: &TranscodeIntentSettings) -> Self {
        Self {
            video: settings.video_codec_policy,
            audio: settings.audio_policy,
            container: settings.container_policy,
        }
    }

    fn apply_to(self, settings: &mut TranscodeIntentSettings) {
        settings.video_codec_policy = self.video;
        settings.audio_policy = self.audio;
        settings.container_policy = self.container;
    }

    fn is_allowed(self) -> bool {
        container_allowed_for_codecs(self.container, self.video, self.audio)
    }
}

fn render_video_codec_choices(
    ui: &mut Ui,
    state: &AppState,
    settings: &mut TranscodeIntentSettings,
) {
    let options = [
        (
            VideoCodecPolicy::Auto,
            state.ui_tr("processing.choice.source"),
        ),
        (VideoCodecPolicy::H264, "H.264"),
        (VideoCodecPolicy::Hevc, "HEVC"),
        (VideoCodecPolicy::Av1, "AV1"),
    ];
    ui.horizontal_wrapped(|ui| {
        for (value, label) in options {
            let selected = settings.video_codec_policy == value;
            let compatible = value_is_currently_compatible(settings, ConversionField::Video, value);
            if choice_button(ui, selected, compatible, label).clicked() {
                let forced = if selected && value != VideoCodecPolicy::Auto {
                    VideoCodecPolicy::Auto
                } else {
                    value
                };
                force_video_choice(settings, forced);
            }
        }
    });
}

fn render_audio_choices(ui: &mut Ui, state: &AppState, settings: &mut TranscodeIntentSettings) {
    let options = [
        (AudioPolicy::Auto, state.ui_tr("processing.choice.source")),
        (AudioPolicy::Aac, "AAC"),
        (AudioPolicy::Opus, "Opus"),
        (AudioPolicy::Flac, "FLAC"),
    ];
    ui.horizontal_wrapped(|ui| {
        for (value, label) in options {
            let selected = settings.audio_policy == value;
            let compatible = value_is_currently_compatible(settings, ConversionField::Audio, value);
            if choice_button(ui, selected, compatible, label).clicked() {
                let forced = if selected && value != AudioPolicy::Auto {
                    AudioPolicy::Auto
                } else {
                    value
                };
                force_audio_choice(settings, forced);
            }
        }
    });
}

fn render_container_choices(ui: &mut Ui, state: &AppState, settings: &mut TranscodeIntentSettings) {
    let options = [
        (
            ContainerPolicy::Auto,
            state.ui_tr("processing.choice.source"),
        ),
        (ContainerPolicy::Mp4, "MP4"),
        (ContainerPolicy::Mkv, "MKV"),
        (ContainerPolicy::Mov, "MOV"),
    ];
    ui.horizontal_wrapped(|ui| {
        for (value, label) in options {
            let selected = settings.container_policy == value;
            let compatible =
                value_is_currently_compatible(settings, ConversionField::Container, value);
            if choice_button(ui, selected, compatible, label).clicked() {
                let forced = if selected && value != ContainerPolicy::Auto {
                    ContainerPolicy::Auto
                } else {
                    value
                };
                force_container_choice(settings, forced);
            }
        }
    });
}

fn render_subtitle_choices(ui: &mut Ui, state: &AppState, settings: &mut TranscodeIntentSettings) {
    let preserve_text = state.ui_tr("processing.subtitle.preserve");
    let embed_text = state.ui_tr("processing.subtitle.embed");
    let burn_text = state.ui_tr("processing.subtitle.burn");
    let options = [
        (SubtitlePolicy::Preserve, preserve_text),
        (SubtitlePolicy::Embed, embed_text),
        (SubtitlePolicy::Burn, burn_text),
    ];
    ui.horizontal_wrapped(|ui| {
        for (value, label) in options {
            let selected = settings.subtitle_policy == value;
            if choice_button(ui, selected, true, label).clicked() {
                settings.subtitle_policy = if selected && value != SubtitlePolicy::Preserve {
                    SubtitlePolicy::Preserve
                } else {
                    value
                };
            }
        }
    });
}

fn choice_button(ui: &mut Ui, selected: bool, compatible: bool, label: &str) -> egui::Response {
    let mut button = egui::Button::new(label)
        .frame(true)
        .min_size(egui::vec2(0.0, ui.spacing().interact_size.y + 6.0));
    if !compatible && !selected {
        button = button.fill(incompatible_choice_fill(ui));
    }
    ui.add(button.selected(selected))
}

fn incompatible_choice_fill(ui: &Ui) -> egui::Color32 {
    if ui.visuals().dark_mode {
        egui::Color32::BLACK
    } else {
        ui.visuals().widgets.noninteractive.bg_fill
    }
}

trait ConversionChoiceValue: Copy + Eq {
    fn apply_to(self, combination: &mut ConversionCombination, field: ConversionField);
    fn matches_field(self, combination: ConversionCombination, field: ConversionField) -> bool;
}

impl ConversionChoiceValue for VideoCodecPolicy {
    fn apply_to(self, combination: &mut ConversionCombination, field: ConversionField) {
        if field == ConversionField::Video {
            combination.video = self;
        }
    }

    fn matches_field(self, combination: ConversionCombination, field: ConversionField) -> bool {
        field == ConversionField::Video && combination.video == self
    }
}

impl ConversionChoiceValue for AudioPolicy {
    fn apply_to(self, combination: &mut ConversionCombination, field: ConversionField) {
        if field == ConversionField::Audio {
            combination.audio = self;
        }
    }

    fn matches_field(self, combination: ConversionCombination, field: ConversionField) -> bool {
        field == ConversionField::Audio && combination.audio == self
    }
}

impl ConversionChoiceValue for ContainerPolicy {
    fn apply_to(self, combination: &mut ConversionCombination, field: ConversionField) {
        if field == ConversionField::Container {
            combination.container = self;
        }
    }

    fn matches_field(self, combination: ConversionCombination, field: ConversionField) -> bool {
        field == ConversionField::Container && combination.container == self
    }
}

fn value_is_currently_compatible<T>(
    settings: &TranscodeIntentSettings,
    field: ConversionField,
    value: T,
) -> bool
where
    T: ConversionChoiceValue,
{
    let mut combination = ConversionCombination::from_settings(settings);
    value.apply_to(&mut combination, field);
    combination.is_allowed()
}

fn force_video_choice(settings: &mut TranscodeIntentSettings, value: VideoCodecPolicy) {
    force_conversion_choice(settings, ConversionField::Video, value);
}

fn force_audio_choice(settings: &mut TranscodeIntentSettings, value: AudioPolicy) {
    force_conversion_choice(settings, ConversionField::Audio, value);
}

fn force_container_choice(settings: &mut TranscodeIntentSettings, value: ContainerPolicy) {
    force_conversion_choice(settings, ConversionField::Container, value);
}

fn force_conversion_choice<T>(
    settings: &mut TranscodeIntentSettings,
    field: ConversionField,
    value: T,
) where
    T: ConversionChoiceValue,
{
    let previous = ConversionCombination::from_settings(settings);
    let mut direct = previous;
    value.apply_to(&mut direct, field);
    if direct.is_allowed() {
        direct.apply_to(settings);
        return;
    }

    let Some(best) = conversion_combinations()
        .into_iter()
        .filter(|combination| value.matches_field(*combination, field))
        .max_by_key(|combination| conversion_choice_score(previous, *combination, field))
    else {
        direct.apply_to(settings);
        return;
    };

    best.apply_to(settings);
}

fn conversion_combinations() -> Vec<ConversionCombination> {
    const VIDEOS: [VideoCodecPolicy; 4] = [
        VideoCodecPolicy::Auto,
        VideoCodecPolicy::H264,
        VideoCodecPolicy::Hevc,
        VideoCodecPolicy::Av1,
    ];
    const AUDIOS: [AudioPolicy; 4] = [
        AudioPolicy::Auto,
        AudioPolicy::Aac,
        AudioPolicy::Opus,
        AudioPolicy::Flac,
    ];
    const CONTAINERS: [ContainerPolicy; 4] = [
        ContainerPolicy::Auto,
        ContainerPolicy::Mp4,
        ContainerPolicy::Mkv,
        ContainerPolicy::Mov,
    ];

    let mut combinations = Vec::new();
    for video in VIDEOS {
        for audio in AUDIOS {
            for container in CONTAINERS {
                let combination = ConversionCombination {
                    video,
                    audio,
                    container,
                };
                if combination.is_allowed() {
                    combinations.push(combination);
                }
            }
        }
    }
    combinations
}

fn conversion_choice_score(
    previous: ConversionCombination,
    candidate: ConversionCombination,
    forced_field: ConversionField,
) -> i64 {
    let mut score = 0;
    if forced_field != ConversionField::Video {
        score += score_video_choice(previous.video, candidate.video);
    }
    if forced_field != ConversionField::Audio {
        score += score_audio_choice(previous.audio, candidate.audio);
    }
    if forced_field != ConversionField::Container {
        score += score_container_choice(previous.container, candidate.container);
    }
    score
}

fn score_video_choice(previous: VideoCodecPolicy, candidate: VideoCodecPolicy) -> i64 {
    if previous == candidate {
        return 10_000;
    }
    match candidate {
        VideoCodecPolicy::H264 => 700,
        VideoCodecPolicy::Hevc => 650,
        VideoCodecPolicy::Av1 => 550,
        VideoCodecPolicy::Auto => 200,
    }
}

fn score_audio_choice(previous: AudioPolicy, candidate: AudioPolicy) -> i64 {
    if previous == candidate {
        return 10_000;
    }
    match candidate {
        AudioPolicy::Aac => 700,
        AudioPolicy::Opus => 600,
        AudioPolicy::Flac => 500,
        AudioPolicy::Auto => 200,
    }
}

fn score_container_choice(previous: ContainerPolicy, candidate: ContainerPolicy) -> i64 {
    if previous == candidate {
        return 10_000;
    }
    match candidate {
        ContainerPolicy::Mkv => 700,
        ContainerPolicy::Mp4 => 650,
        ContainerPolicy::Mov => 600,
        ContainerPolicy::Auto => 200,
    }
}

fn container_allowed_for_codecs(
    container: ContainerPolicy,
    video: VideoCodecPolicy,
    audio: AudioPolicy,
) -> bool {
    video_allowed_for_container(video, container) && audio_allowed_for_container(audio, container)
}

fn video_allowed_for_container(video: VideoCodecPolicy, container: ContainerPolicy) -> bool {
    match container {
        ContainerPolicy::Auto | ContainerPolicy::Mkv => true,
        ContainerPolicy::Mp4 => matches!(
            video,
            VideoCodecPolicy::Auto
                | VideoCodecPolicy::H264
                | VideoCodecPolicy::Hevc
                | VideoCodecPolicy::Av1
        ),
        ContainerPolicy::Mov => matches!(
            video,
            VideoCodecPolicy::Auto | VideoCodecPolicy::H264 | VideoCodecPolicy::Hevc
        ),
    }
}

fn audio_allowed_for_container(audio: AudioPolicy, container: ContainerPolicy) -> bool {
    match container {
        ContainerPolicy::Auto | ContainerPolicy::Mkv => true,
        ContainerPolicy::Mp4 | ContainerPolicy::Mov => {
            matches!(audio, AudioPolicy::Auto | AudioPolicy::Aac)
        }
    }
}
