use eframe::egui::{Grid, Ui};
use egui_taffy::{Tui, TuiBuilderLogic as _};

use crate::app::state::{AppState, SavedCookieFile};

use super::semantic_ui_metrics;
use super::settings_detail_template::{
    SettingsDetailNode, render_settings_detail_header, render_settings_detail_page,
};
use super::xaml_taffy_styles::xaml_auto_height_block_style;
use super::xaml_template_renderer::{
    TemplateBlockSlot, show_template_tui_block, show_template_ui_block,
};

pub(super) fn render_cookie_manager_detail_page(ui: &mut Ui, state: &mut AppState) {
    let mut show_block = |slot, node, tui: &mut Tui| {
        show_cookie_manager_detail_block(slot, node, tui, state);
    };
    render_settings_detail_page(
        ui,
        "advance-cookie-manager-page-scroll",
        "advance-cookie-manager-detail-taffy",
        semantic_ui_metrics::download_conversion_detail_header_to_body_vertical_spacing(),
        &mut show_block,
    );
}

fn show_cookie_manager_detail_block(
    slot: TemplateBlockSlot,
    node: SettingsDetailNode,
    tui: &mut Tui,
    state: &mut AppState,
) {
    match node {
        SettingsDetailNode::Header => {
            show_template_ui_block(slot, tui, |ui| render_header(ui, state));
        }
        SettingsDetailNode::Body => {
            show_template_tui_block(slot, tui, |tui| render_body(tui, state));
        }
    }
}

fn render_header(ui: &mut Ui, state: &mut AppState) {
    render_settings_detail_header(
        ui,
        state,
        "advance.cookie_manager_title",
        AppState::close_advance_detail_page,
    );
}

fn render_body(tui: &mut Tui, state: &mut AppState) {
    tui.style(xaml_auto_height_block_style()).ui(|ui| {
        if ui
            .button(state.ui_i18n_text_for_key("advance.add_cookie"))
            .clicked()
        {
            state.open_youtube_login_rescue_prompt();
        }
    });
    tui.style(xaml_auto_height_block_style()).ui(|ui| {
        ui.separator();
    });
    tui.style(xaml_auto_height_block_style()).ui(|ui| {
        render_cookie_file_table(ui, state);
    });
}

fn render_cookie_file_table(ui: &mut Ui, state: &mut AppState) {
    let entries = state.saved_cookie_files();
    if entries.is_empty() {
        ui.label(state.ui_i18n_text_for_key("advance.cookie_manager_empty"));
        return;
    }

    Grid::new("advance-cookie-manager-files-grid")
        .num_columns(3)
        .striped(true)
        .show(ui, |ui| {
            ui.strong(state.ui_i18n_text_for_key("advance.cookie_manager_name"));
            ui.strong(state.ui_i18n_text_for_key("advance.cookie_manager_updated"));
            ui.strong(state.ui_i18n_text_for_key("advance.cookie_manager_actions"));
            ui.end_row();

            for entry in entries {
                render_cookie_file_row(ui, state, &entry);
            }
        });
}

fn render_cookie_file_row(ui: &mut Ui, state: &mut AppState, entry: &SavedCookieFile) {
    ui.label(entry.display_name.as_str());
    ui.label(format_cookie_updated_time(entry.updated_unix));
    ui.horizontal_wrapped(|ui| {
        if ui
            .button(state.ui_i18n_text_for_key("advance.cookie_manager_refresh"))
            .clicked()
        {
            state.refresh_saved_cookie_file(&entry.id);
        }
        if ui
            .button(state.ui_i18n_text_for_key("advance.cookie_manager_delete"))
            .clicked()
        {
            state.delete_saved_cookie_file(&entry.id);
        }
    });
    ui.end_row();
}

fn format_cookie_updated_time(unix_seconds: u64) -> String {
    if unix_seconds == 0 {
        return "-".to_owned();
    }

    let days = (unix_seconds / 86_400) as i64;
    let seconds_of_day = unix_seconds % 86_400;
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let (year, month, day) = civil_from_days_utc(days);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02} UTC")
}

fn civil_from_days_utc(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    // Howard Hinnant civil calendar conversion. Kept local to avoid adding a
    // date/time dependency for one compact Cookie Manager timestamp.
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as i32, m as u32, d as u32)
}
