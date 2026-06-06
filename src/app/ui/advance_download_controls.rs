use eframe::egui;
use egui_taffy::Tui;

use crate::app::state::AppState;
use crate::app::widgets::url_input::{AppTextBox, AppTextBoxSyntax};
use crate::infrastructure::FileTimeMode;

use super::common::{settings_taffy_form_row, settings_taffy_section};
use super::semantic_ui_metrics;

pub(super) fn render_aria2_section(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, "Aria2", |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_i18n_text_for_key("advance.external_downloader"),
            |ui| {
                let mut use_aria2 = state.item_defaults.use_aria2;
                if ui
                    .checkbox(
                        &mut use_aria2,
                        state.ui_i18n_text_for_key("advance.use_aria2_for_faster_downloads"),
                    )
                    .changed()
                {
                    state.set_use_aria2(use_aria2);
                }
            },
        );
    });
}
pub(super) fn render_download_processing_section(
    tui: &mut Tui,
    state: &mut AppState,
    label_width: f32,
) {
    settings_taffy_section(
        tui,
        state.ui_i18n_text_for_key("advance.download_control"),
        |tui| {
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("advance.concurrent_fragments"),
                |ui| {
                    let selected = state.tool_paths.concurrent_fragments;
                    egui::ComboBox::from_id_salt("concurrent-fragments")
                        .selected_text(format!("{}", selected.max(1)))
                        .show_ui(ui, |ui| {
                            for value in state.available_concurrent_fragment_values() {
                                let label = if value == 1 {
                                    state.ui_i18n_text_for_key("advance.1_default").to_owned()
                                } else {
                                    value.to_string()
                                };
                                if ui.selectable_label(selected == value, label).clicked() {
                                    state.set_concurrent_fragments(value);
                                }
                            }
                        })
                        .response;
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("advance.rate_limit"),
                |ui| {
                    let mut limit_rate = state.tool_paths.limit_rate.clone();
                    if AppTextBox::new(&mut limit_rate)
                .hint_text(state.ui_i18n_text_for_key("advance.e_g_2m_800k_leave_empty_for_unlimited"))
                .language(state.language())
                .syntax(AppTextBoxSyntax::Plain)
                .desired_width(semantic_ui_metrics::advance_form_standard_text_field_width())
                .ui(ui)
                .changed()
            {
                state.set_limit_rate(limit_rate);
            }
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("advance.chapters"),
                |ui| {
                    let mut compatibility_mode = state.tool_paths.chapter_compatibility_mode;
                    if ui
                        .checkbox(
                            &mut compatibility_mode,
                            state.ui_i18n_text_for_key(
                                "advance.chapter_download_compatibility_mode",
                            ),
                        )
                        .changed()
                    {
                        state.set_chapter_compatibility_mode(compatibility_mode);
                    }
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("advance.file_time"),
                |ui| {
                    let selected = state.tool_paths.file_time_mode;
                    egui::ComboBox::from_id_salt("file-time-mode")
                        .selected_text(state.ui_i18n_text_for_key(selected.label_key()))
                        .show_ui(ui, |ui| {
                            for mode in [
                                FileTimeMode::None,
                                FileTimeMode::UseUploadDate,
                                FileTimeMode::UseDownloadTime,
                            ] {
                                if ui
                                    .selectable_label(
                                        selected == mode,
                                        state.ui_i18n_text_for_key(mode.label_key()),
                                    )
                                    .clicked()
                                {
                                    state.set_file_time_mode(mode);
                                }
                            }
                        })
                        .response;
                },
            );
        },
    );
}
