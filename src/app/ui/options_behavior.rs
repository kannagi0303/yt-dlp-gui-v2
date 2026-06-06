use eframe::egui;
use egui_taffy::Tui;

use crate::app::state::AppState;
use crate::infrastructure::{OutputFileActionMode, YoutubeVideoPlaylistMode};

use super::common::{settings_taffy_form_row, settings_taffy_section};

pub(super) fn render_behavior_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_i18n_text_for_key("options.behavior"), |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_i18n_text_for_key("options.add_action"),
            |ui| {
                let mut enabled = state.config.direct_download_on_add;
                if ui
                    .checkbox(
                        &mut enabled,
                        state.ui_i18n_text_for_key("options.download_directly"),
                    )
                    .changed()
                {
                    state.set_direct_download_on_add(enabled);
                }
            },
        );
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_i18n_text_for_key("options.clipboard_change"),
            |ui| {
                let mut enabled = state.config.clipboard_auto_add;
                if ui
                    .checkbox(
                        &mut enabled,
                        state.ui_i18n_text_for_key("options.run_immediately"),
                    )
                    .changed()
                {
                    state.set_clipboard_auto_add(enabled);
                }
            },
        );
    });
}

pub(super) fn render_tabs_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_i18n_text_for_key("options.tabs"), |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_i18n_text_for_key("options.log_tab"),
            |ui| {
                let mut enabled = state.config.show_log_tab;
                if ui
                    .checkbox(
                        &mut enabled,
                        state.ui_i18n_text_for_key("options.show_log_tab"),
                    )
                    .changed()
                {
                    state.set_show_log_tab(enabled);
                }
            },
        );
    });
}

pub(super) fn render_playlist_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(
        tui,
        state.ui_i18n_text_for_key("options.playlist_2"),
        |tui| {
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.with_playlist"),
                |ui| {
                    egui::ComboBox::from_id_salt("youtube-video-playlist-mode")
                        .selected_text(match state.config.youtube_video_playlist_mode {
                            YoutubeVideoPlaylistMode::Ask => {
                                state.ui_i18n_text_for_key("options.ask")
                            }
                            YoutubeVideoPlaylistMode::Video => {
                                state.ui_i18n_text_for_key("options.single_video")
                            }
                            YoutubeVideoPlaylistMode::Ignore => {
                                state.ui_i18n_text_for_key("options.full_playlist")
                            }
                        })
                        .show_ui(ui, |ui| {
                            for (mode, label) in [
                                (
                                    YoutubeVideoPlaylistMode::Ask,
                                    state.ui_i18n_text_for_key("options.ask"),
                                ),
                                (
                                    YoutubeVideoPlaylistMode::Video,
                                    state.ui_i18n_text_for_key("options.single_video"),
                                ),
                                (
                                    YoutubeVideoPlaylistMode::Ignore,
                                    state.ui_i18n_text_for_key("options.full_playlist"),
                                ),
                            ] {
                                if ui
                                    .selectable_label(
                                        state.config.youtube_video_playlist_mode == mode,
                                        label,
                                    )
                                    .clicked()
                                {
                                    state.set_youtube_video_playlist_mode(mode);
                                }
                            }
                        });
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.high_risk_prompt"),
                |ui| {
                    let mut enabled = state.config.youtube_high_risk_playlist_prompt;
                    if ui
                        .checkbox(&mut enabled, state.ui_i18n_text_for_key("options.on"))
                        .changed()
                    {
                        state.set_youtube_high_risk_playlist_prompt(enabled);
                    }
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.playlist_count"),
                |ui| {
                    let mut enabled = state.config.batch_limit_enabled;
                    if ui
                        .checkbox(&mut enabled, state.ui_i18n_text_for_key("options.limit"))
                        .changed()
                    {
                        state.set_batch_limit_enabled(enabled);
                    }
                    let mut count = state.config.batch_limit_count;
                    if ui
                        .add(
                            egui::DragValue::new(&mut count)
                                .range(1..=9999)
                                .prefix(state.ui_i18n_text_for_key("options.max"))
                                .suffix(state.ui_i18n_text_for_key("options.items")),
                        )
                        .changed()
                    {
                        state.set_batch_limit_count(count);
                    }
                },
            );
        },
    );
}

pub(super) fn render_file_action_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(
        tui,
        state.ui_i18n_text_for_key("options.file_actions"),
        |tui| {
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.action_button"),
                |ui| {
                    egui::ComboBox::from_id_salt("output-file-action-mode")
                        .selected_text(state.ui_i18n_text_for_key(
                            output_file_action_mode_label_key(state.config.output_file_action_mode),
                        ))
                        .show_ui(ui, |ui| {
                            for mode in OutputFileActionMode::variants() {
                                if ui
                                    .selectable_label(
                                        state.config.output_file_action_mode == mode,
                                        state.ui_i18n_text_for_key(
                                            output_file_action_mode_label_key(mode),
                                        ),
                                    )
                                    .clicked()
                                {
                                    state.set_output_file_action_mode(mode);
                                }
                            }
                        });
                },
            );
        },
    );
}

fn output_file_action_mode_label_key(mode: OutputFileActionMode) -> &'static str {
    match mode {
        OutputFileActionMode::Menu => "options.file_action.show_menu",
        OutputFileActionMode::OpenFolder => "item.open_folder",
        OutputFileActionMode::OpenFile => "item.open_file",
    }
}
