use eframe::egui::Ui;

use crate::app::state::AppState;

pub(super) fn render_cookie_manager_detail_page(ui: &mut Ui, state: &mut AppState) {
    super::advance_cookie_manager_template::render_cookie_manager_detail_page(ui, state);
}
