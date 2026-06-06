use eframe::egui::Ui;

use crate::app::state::AppState;

use super::semantic_ui_metrics;

pub(super) struct OptionsLayoutMetrics {
    pub(super) label_width: f32,
}

impl OptionsLayoutMetrics {
    pub(super) fn new(ui: &Ui, state: &AppState) -> Self {
        Self {
            label_width: options_label_width(ui, state),
        }
    }
}

fn options_label_width(ui: &Ui, state: &AppState) -> f32 {
    let add_action_text = state.ui_i18n_text_for_key("options.add_action");
    let clipboard_change_text = state.ui_i18n_text_for_key("options.clipboard_change");
    let log_tab_text = state.ui_i18n_text_for_key("options.log_tab");
    let with_playlist_text = state.ui_i18n_text_for_key("options.with_playlist");
    let high_risk_prompt_text = state.ui_i18n_text_for_key("options.high_risk_prompt");
    let playlist_count_text = state.ui_i18n_text_for_key("options.playlist_count");
    let action_button_text = state.ui_i18n_text_for_key("options.action_button");
    let language_text = state.ui_i18n_text_for_key("options.language");
    let current_language_text = state.ui_i18n_text_for_key("options.current_language");
    let cache_usage_text = state.ui_i18n_text_for_key("options.cache_usage");
    let cache_cleanup_text = state.ui_i18n_text_for_key("options.cache_cleanup");
    let notifications_text = state.ui_i18n_text_for_key("options.notifications");
    let theme_text = state.ui_i18n_text_for_key("options.theme");
    let theme_color_text = state.ui_i18n_text_for_key("options.theme_color");
    let ui_scale_text = state.ui_i18n_text_for_key("options.ui_scale");
    let cache_location_text = state.ui_i18n_text_for_key("options.cache_location");
    let always_on_top_text = state.ui_i18n_text_for_key("options.always_on_top");
    let window_position_text = state.ui_i18n_text_for_key("options.window_position");
    let window_size_text = state.ui_i18n_text_for_key("options.window_size");

    semantic_ui_metrics::settings_form_label_column_width_for_visible_texts(
        ui,
        &[
            "yt-dlp",
            "Deno",
            "FFmpeg",
            "Aria2",
            add_action_text,
            clipboard_change_text,
            log_tab_text,
            with_playlist_text,
            high_risk_prompt_text,
            playlist_count_text,
            action_button_text,
            language_text,
            current_language_text,
            cache_usage_text,
            cache_cleanup_text,
            notifications_text,
            theme_text,
            theme_color_text,
            ui_scale_text,
            cache_location_text,
            always_on_top_text,
            window_position_text,
            window_size_text,
        ],
    )
}
