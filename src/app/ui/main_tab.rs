use eframe::egui::Ui;

use crate::app::state::AppState;

use super::main_tab_template;

pub(super) fn render_main_tab(ui: &mut Ui, state: &mut AppState) {
    main_tab_template::render_main_tab(ui, state);
}
