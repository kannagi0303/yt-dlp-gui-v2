use eframe::egui;
use egui_taffy::Tui;

use crate::app::state::{AdvanceDetailPage, AppState};
use crate::app::widgets::icon::AppIcon;
use crate::infrastructure::PostProcessMode;

use super::common::{settings_taffy_form_row, settings_taffy_section, text_trailing_icon_button};

pub(super) fn render_post_processing_section(
    tui: &mut Tui,
    state: &mut AppState,
    label_width: f32,
) {
    settings_taffy_section(
        tui,
        state.ui_i18n_text_for_key("advance.post_processing"),
        |tui| {
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("advance.thumbnail"),
                |ui| {
                    post_process_mode_selector(
                        ui,
                        state,
                        state.config.thumbnail_mode,
                        AppState::set_thumbnail_post_process_mode,
                    );
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("advance.subtitles"),
                |ui| {
                    post_process_mode_selector(
                        ui,
                        state,
                        state.config.subtitle_mode,
                        AppState::set_subtitle_post_process_mode,
                    );
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("advance.chapters"),
                |ui| {
                    post_process_mode_selector(
                        ui,
                        state,
                        state.config.chapter_mode,
                        AppState::set_chapter_post_process_mode,
                    );
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("advance.download_conversion"),
                |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut enabled = state.config.post_download_conversion_enabled;
                        if ui
                            .checkbox(&mut enabled, state.ui_i18n_text_for_key("advance.enable"))
                            .changed()
                        {
                            state.set_enable_builtin_transcode_after_download(enabled);
                        }

                        if ui
                            .add(text_trailing_icon_button(
                                ui,
                                state.ui_i18n_text_for_key("advance.settings"),
                                AppIcon::MenuRight,
                            ))
                            .clicked()
                        {
                            state.open_advance_detail_page(AdvanceDetailPage::Transcode);
                        }
                    });
                },
            );
        },
    );
}

fn post_process_mode_selector(
    ui: &mut egui::Ui,
    state: &mut AppState,
    current: PostProcessMode,
    set_mode: fn(&mut AppState, PostProcessMode),
) {
    ui.horizontal_wrapped(|ui| {
        for mode in PostProcessMode::variants() {
            let response = ui.selectable_label(
                current == mode,
                state.ui_i18n_text_for_key(mode.label_key()),
            );
            if response.clicked() && current != mode {
                set_mode(state, mode);
            }
        }
    });
}
