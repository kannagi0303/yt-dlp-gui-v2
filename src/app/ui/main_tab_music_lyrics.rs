use eframe::egui::{self, Color32, RichText, Ui};

use crate::app::state::MusicLyricsDisplayLine;

use super::semantic_ui_metrics;

pub(super) fn render_music_lyrics_at(ui: &mut Ui, rect: egui::Rect, line: &MusicLyricsDisplayLine) {
    let fade = line.fade.clamp(0.0, 1.0);
    if fade < 1.0 {
        ui.ctx().request_repaint();
    }
    if let Some(previous) = line.previous.as_deref().filter(|_| fade < 1.0) {
        render_music_lyrics_text_at(ui, rect, previous, 1.0 - fade);
    }
    render_music_lyrics_text_at(ui, rect, &line.current, fade.max(0.001));
}

fn render_music_lyrics_text_at(ui: &mut Ui, rect: egui::Rect, line: &str, alpha: f32) {
    let font_size = semantic_ui_metrics::main_music_lyrics_font_size_from_body(ui);
    let text = RichText::new(line).size(font_size);
    let galley = egui::WidgetText::from(text).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        rect.width(),
        egui::TextStyle::Body,
    );
    let pos = egui::pos2(
        rect.center().x - galley.size().x * 0.5,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(
        pos,
        galley,
        color_with_alpha(ui.visuals().text_color(), alpha),
    );
}

fn color_with_alpha(color: Color32, alpha: f32) -> Color32 {
    let alpha = (f32::from(color.a()) * alpha.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}
