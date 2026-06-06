use eframe::egui;
use egui_taffy::Tui;

use crate::app::state::AppState;
use crate::infrastructure::CacheLocationMode;

use super::common::{settings_taffy_form_row, settings_taffy_section};

fn cache_location_mode_label(state: &AppState, mode: CacheLocationMode) -> &'static str {
    match mode {
        CacheLocationMode::YtDlpDefault => {
            state.ui_i18n_text_for_key("options.cache_location.default")
        }
        CacheLocationMode::V2Cache => "yt-dlp-gui",
        CacheLocationMode::WindowsTemp => "Windows",
    }
}

pub(super) fn render_cache_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_i18n_text_for_key("options.cache"), |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_i18n_text_for_key("options.cache_location"),
            |ui| {
                egui::ComboBox::from_id_salt("cache-location-mode")
                    .selected_text(cache_location_mode_label(
                        state,
                        state.tool_paths.cache_mode,
                    ))
                    .show_ui(ui, |ui| {
                        for mode in state.available_cache_location_modes() {
                            let ui_text = cache_location_mode_label(state, mode);
                            let response =
                                ui.selectable_label(state.tool_paths.cache_mode == mode, ui_text);
                            if response.clicked() {
                                state.set_cache_location_mode(mode);
                            }
                        }
                    });
            },
        );

        state.refresh_cache_management_summary_if_stale();

        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_i18n_text_for_key("options.cache_usage"),
            |ui| {
                ui.label(state.cache_management_usage_display());
            },
        );

        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_i18n_text_for_key("options.cache_cleanup"),
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(state.ui_i18n_text_for_key("options.cache_refresh"))
                        .clicked()
                    {
                        state.refresh_cache_management_summary();
                    }
                    if ui
                        .button(state.ui_i18n_text_for_key("options.cache_clear_expired"))
                        .clicked()
                    {
                        state.clear_expired_music_cache();
                    }
                    if ui
                        .button(state.ui_i18n_text_for_key("options.cache_clear_audio"))
                        .clicked()
                    {
                        state.clear_music_stream_cache();
                    }
                    if ui
                        .button(state.ui_i18n_text_for_key("options.cache_clear_all"))
                        .clicked()
                    {
                        state.clear_app_cache();
                    }
                });
            },
        );
    });
}
