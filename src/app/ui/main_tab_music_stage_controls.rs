use eframe::egui::{self, Button, RichText, Ui};

use crate::app::state::{AppState, MusicMixMode};
use crate::app::widgets::url_input::accent_blue_for_ui;

use super::semantic_ui_metrics;
use super::xaml_rect_template::show_rect_template;
use super::xaml_ui_nodes::{TemplateAxis, block, fill, rows};

pub(super) fn music_stage_bpm_display(state: &mut AppState) -> (String, bool) {
    let Some(manifest) = state.music_current_analysis_manifest() else {
        return ("-- BPM".to_owned(), false);
    };
    state.music_stage_segment_bpm_display_text(
        &manifest,
        state.music_current_playback_seconds(),
        state.music_chorus_current_display_highlight_range(),
    )
}

pub(super) fn render_music_stage_control_at(ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
    let mode = state.music_mix_mode();
    let response = mix_round_button(ui, rect, mode);
    response
        .clone()
        .on_hover_text(format!("Mix: {}", mode.label()));

    egui::Popup::menu(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| render_mix_mode_popup(ui, state));
}

fn mix_round_button(ui: &mut Ui, rect: egui::Rect, mode: MusicMixMode) -> egui::Response {
    let response = ui.interact(
        rect,
        ui.make_persistent_id("music-mix-mode"),
        egui::Sense::click(),
    );
    let radius = semantic_ui_metrics::main_music_round_button_radius_for_rect(rect);
    let accent = accent_blue_for_ui(ui);
    let active = mode.enabled();
    let active_color = active.then_some(accent);
    let fill = semantic_ui_metrics::main_music_button_fill(ui, &response, active_color);
    let stroke = semantic_ui_metrics::main_music_button_stroke(ui, &response, active_color);
    let text_color = semantic_ui_metrics::main_music_button_foreground(ui, &response, active_color);

    ui.painter().circle_filled(rect.center(), radius, fill);
    ui.painter().circle_stroke(rect.center(), radius, stroke);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "M",
        egui::FontId::proportional(semantic_ui_metrics::main_music_stage_label_size_for_rect(
            rect,
        )),
        text_color,
    );
    if active {
        ui.painter().circle_filled(
            rect.center() + egui::vec2(radius * 0.58, -radius * 0.58),
            (radius * 0.18).max(1.5),
            accent,
        );
    }
    response
}

fn render_mix_mode_popup(ui: &mut Ui, state: &mut AppState) {
    const POPUP_WIDTH: f32 = 188.0;
    const ROW_HEIGHT: f32 = 28.0;

    ui.set_width(POPUP_WIDTH);
    let (popup_rect, _) = ui.allocate_exact_size(
        egui::vec2(POPUP_WIDTH, ROW_HEIGHT * MusicMixMode::ALL.len() as f32),
        egui::Sense::hover(),
    );
    let template = rows(MusicMixMode::ALL.map(|mode| fill(block(mode))));
    show_rect_template(
        popup_rect,
        template,
        &mut |_, _: TemplateAxis, _| 0.0,
        &mut |mode, row_rect| render_mix_mode_choice(ui, state, row_rect, mode),
    );
}

fn render_mix_mode_choice(ui: &mut Ui, state: &mut AppState, rect: egui::Rect, mode: MusicMixMode) {
    let selected = state.music_mix_mode() == mode;
    let label = if selected {
        format!("✓  {}", mode.label())
    } else {
        format!("   {}", mode.label())
    };
    let mut button = Button::new(RichText::new(label).size(11.0)).small();
    if selected {
        button = button.fill(ui.visuals().selection.bg_fill);
    }
    if ui.put(rect, button).clicked() {
        state.set_music_mix_mode(mode);
        ui.close();
    }
}

#[cfg(test)]
mod tests {
    use crate::app::state::MusicMixMode;

    #[test]
    fn mix_menu_exposes_one_complete_mode_axis() {
        assert_eq!(MusicMixMode::ALL.len(), 4);
        assert_eq!(MusicMixMode::ALL[0].label(), "Off");
        assert_eq!(MusicMixMode::ALL[3].label(), "Highlight");
    }
}
