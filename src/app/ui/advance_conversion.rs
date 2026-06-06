use eframe::egui::Ui;

use crate::app::state::AppState;

pub(super) fn render_download_conversion_detail_page(ui: &mut Ui, state: &mut AppState) {
    super::advance_conversion_template::render_download_conversion_detail_page(ui, state);
}
