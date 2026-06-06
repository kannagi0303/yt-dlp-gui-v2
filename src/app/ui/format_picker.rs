use crate::app::state::AppState;
use eframe::egui::Ui;

use super::format_picker_template;

pub(super) fn render_format_picker_screen(ui: &mut Ui, state: &mut AppState) {
    format_picker_template::render_format_picker_screen(ui, state);
}
