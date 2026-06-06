use eframe::egui::Ui;

use crate::app::state::{AdvanceDetailPage, AppState};

pub(super) fn render_advance_tab(ui: &mut Ui, state: &mut AppState) {
    match state.advance_detail_page {
        Some(AdvanceDetailPage::Transcode) => {
            super::advance_conversion::render_download_conversion_detail_page(ui, state);
        }
        Some(AdvanceDetailPage::CookieManager) => {
            super::advance_cookie_manager::render_cookie_manager_detail_page(ui, state);
        }
        None => {
            super::advance_tab_template::render_advance_root_page(ui, state);
        }
    }
}
