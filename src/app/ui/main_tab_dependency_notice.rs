use eframe::egui::{self, Color32, RichText, Ui};

use crate::app::state::AppState;
use crate::app::widgets::icon::{AppIcon, icon_image};

use super::semantic_ui_metrics;

const MISSING_YT_DLP_CALLOUT_KEY: &str = "main.missing_yt_dlp_callout";

pub(super) fn missing_tool_icon_text_button(
    ui: &Ui,
    icon: AppIcon,
    label: &str,
) -> egui::Button<'static> {
    let size = semantic_ui_metrics::standard_icon_size_from_current_control_metrics(ui);
    egui::Button::image_and_text(
        icon_image(icon, size, missing_tool_button_text_color(ui)),
        RichText::new(label)
            .size(size)
            .color(missing_tool_button_text_color(ui)),
    )
    .fill(missing_tool_button_fill(ui))
    .stroke(missing_tool_button_stroke())
}

pub(super) fn missing_tool_plain_button(ui: &Ui, label: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(label).color(missing_tool_button_text_color(ui)))
        .fill(missing_tool_button_fill(ui))
        .stroke(missing_tool_button_stroke())
}

pub(super) fn show_missing_yt_dlp_callout(
    ui: &Ui,
    anchor: egui::Rect,
    id_source: &'static str,
    state: &AppState,
) {
    let x = semantic_ui_metrics::main_missing_yt_dlp_callout_left_for_anchor(anchor);
    let pos = egui::pos2(
        x,
        semantic_ui_metrics::main_missing_yt_dlp_callout_top_for_anchor(anchor),
    );

    egui::Area::new(egui::Id::new(("missing-ytdlp-callout", id_source)))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .fill(missing_tool_callout_fill(ui))
                .stroke(missing_tool_callout_stroke())
                .show(ui, |ui| {
                    ui.set_max_width(semantic_ui_metrics::main_missing_yt_dlp_callout_width());
                    ui.label(
                        RichText::new(state.ui_i18n_text_for_key(MISSING_YT_DLP_CALLOUT_KEY))
                            .color(missing_tool_button_text_color(ui)),
                    );
                });
        });
}

fn missing_tool_button_fill(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(96, 24, 24)
    } else {
        Color32::from_rgb(255, 214, 214)
    }
}

fn missing_tool_button_stroke() -> egui::Stroke {
    egui::Stroke::new(
        semantic_ui_metrics::main_missing_tool_button_stroke_width(),
        Color32::from_rgb(220, 72, 72),
    )
}

fn missing_tool_button_text_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(255, 225, 225)
    } else {
        Color32::from_rgb(190, 0, 28)
    }
}

fn missing_tool_callout_fill(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(42, 16, 16)
    } else {
        Color32::from_rgb(255, 226, 226)
    }
}

fn missing_tool_callout_stroke() -> egui::Stroke {
    egui::Stroke::new(
        semantic_ui_metrics::main_missing_tool_callout_stroke_width(),
        Color32::from_rgb(235, 88, 88),
    )
}
