use crate::app::state::ToolLogStep;
use eframe::egui::{self, Color32, FontId, RichText, TextStyle, TextWrapMode, Ui, WidgetText};

use super::semantic_ui_metrics;

pub(super) fn command_viewer_height(
    ui: &Ui,
    selected_step: Option<&ToolLogStep>,
    available_width: f32,
    available_height: f32,
) -> f32 {
    let Some(step) = selected_step else {
        return 0.0;
    };

    let content_width =
        semantic_ui_metrics::processing_command_viewer_content_width_for_available_width(
            available_width,
        );
    let command_tokens = display_command_tokens(step.command.as_str());
    let mut content_height = measured_wrapped_command_height(ui, &command_tokens, content_width);
    if let Some(detail) = step_detail_text(step) {
        content_height += semantic_ui_metrics::processing_command_viewer_token_spacing_y();
        content_height += measured_wrapped_plain_text_height(ui, detail, content_width);
    }
    let natural_height =
        semantic_ui_metrics::processing_command_viewer_natural_height_for_content_height(
            content_height,
        );
    let max_height = available_height.max(0.0);
    natural_height.min(max_height)
}

fn step_detail_text(step: &ToolLogStep) -> Option<&str> {
    step.detail
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
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
            line_width
                + semantic_ui_metrics::processing_command_viewer_token_spacing_x()
                + token_width
        };

        if line_width > 0.0 && next_width > max_width {
            line_count += 1;
            line_width = token_width;
        } else {
            line_width = next_width;
        }
    }

    semantic_ui_metrics::processing_command_viewer_line_stack_height(line_height, line_count)
}

fn measured_wrapped_plain_text_height(ui: &Ui, text: &str, max_width: f32) -> f32 {
    let tokens = text
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    measured_wrapped_command_height(ui, &tokens, max_width)
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
    FontId::monospace(semantic_ui_metrics::processing_command_viewer_font_size())
}

pub(super) fn render_command_viewer(ui: &mut Ui, step: &ToolLogStep) {
    let desired = ui.available_size();
    if desired.x <= 0.0 || desired.y <= 0.0 {
        return;
    }
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_width(rect.width());
        ui.set_height(rect.height());
        command_line_frame(ui, step);
    });
}

fn command_line_frame(ui: &mut Ui, step: &ToolLogStep) {
    egui::Frame::NONE
        .fill(command_viewer_panel_fill(ui))
        .stroke(egui::Stroke::new(
            semantic_ui_metrics::processing_command_viewer_frame_stroke_width(),
            command_viewer_frame_line_color(ui),
        ))
        .corner_radius(semantic_ui_metrics::processing_command_viewer_frame_corner_radius())
        .inner_margin(semantic_ui_metrics::processing_command_viewer_frame_inner_margin())
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(ui.available_height());
            render_highlighted_command(ui, step.command.as_str());
            if let Some(detail) = step_detail_text(step) {
                ui.add_space(semantic_ui_metrics::processing_command_viewer_token_spacing_y());
                ui.label(RichText::new("Error").strong().font(command_font_id()));
                render_plain_monospace_text(ui, detail);
            }
        });
}

fn command_viewer_frame_line_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_gray(42)
    } else {
        Color32::from_gray(218)
    }
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
        let command_token_spacing = semantic_ui_metrics::processing_command_viewer_token_spacing();
        ui.spacing_mut().item_spacing.x = command_token_spacing.x;
        ui.spacing_mut().item_spacing.y = command_token_spacing.y;
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

fn render_plain_monospace_text(ui: &mut Ui, text: &str) {
    ui.horizontal_wrapped(|ui| {
        let command_token_spacing = semantic_ui_metrics::processing_command_viewer_token_spacing();
        ui.spacing_mut().item_spacing.x = command_token_spacing.x;
        ui.spacing_mut().item_spacing.y = command_token_spacing.y;
        for token in text.split_whitespace() {
            ui.add(
                egui::Label::new(RichText::new(token).font(command_font_id()))
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
