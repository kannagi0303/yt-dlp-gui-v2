use eframe::egui::Ui;

use crate::app::state::{AppState, OptionsDetailPage};

pub(super) fn render_options_tab(ui: &mut Ui, state: &mut AppState) {
    match state.options_detail_page {
        Some(OptionsDetailPage::Language) => {
            super::options_language::render_language_detail_page(ui, state)
        }
        None => super::options_tab_template::render_options_root_page(ui, state),
    }
}
