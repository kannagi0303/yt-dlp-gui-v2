use eframe::egui;
use egui_taffy::Tui;

use crate::app::state::AppState;

use super::common::{settings_taffy_form_row, settings_taffy_section};

pub(super) fn render_config_source_section(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_i18n_text_for_key("advance.source"), |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_i18n_text_for_key("advance.config"),
            |ui| {
                let config_files = state.available_yt_dlp_config_files();
                let selected_label = if state.tool_paths.yt_dlp_config.trim().is_empty() {
                    state.ui_i18n_text_for_key("advance.none").to_owned()
                } else {
                    config_files
                        .iter()
                        .find(|option| option.path == state.tool_paths.yt_dlp_config)
                        .map(|option| option.name.clone())
                        .unwrap_or_else(|| state.tool_paths.yt_dlp_config.clone())
                };
                egui::ComboBox::from_id_salt("yt-dlp-config-file")
                    .selected_text(selected_label)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                state.tool_paths.yt_dlp_config.trim().is_empty(),
                                state.ui_i18n_text_for_key("advance.none"),
                            )
                            .clicked()
                        {
                            state.set_yt_dlp_config_path(String::new());
                        }
                        for option in config_files {
                            if ui
                                .selectable_label(
                                    state.tool_paths.yt_dlp_config == option.path,
                                    option.name.as_str(),
                                )
                                .clicked()
                            {
                                state.set_yt_dlp_config_path(option.path);
                            }
                        }
                    })
                    .response;
            },
        );
    });
}
