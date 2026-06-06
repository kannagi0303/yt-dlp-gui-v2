use crate::app::state::AppState;
use eframe::egui::Ui;

pub(super) fn render_log_tab(ui: &mut Ui, state: &mut AppState) {
    super::processing_tab_template::render_log_tab(ui, state);
}

pub(super) fn render_processing_settings_content(ui: &mut Ui, state: &mut AppState) {
    super::processing_conversion::render_processing_settings_content(ui, state);
}
