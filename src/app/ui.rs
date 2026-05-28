mod advance_tab;
mod common;
mod compact_row;
mod format_picker;
mod item_card;
mod main_tab;
mod measure;
mod options_tab;
mod prepare_tab;
mod processing_tab;
mod single_mode;
mod titlebar;

use eframe::egui::{self, CentralPanel, Ui};

use crate::app::state::{AppState, AppTab};

pub fn render_app(root_ui: &mut Ui, state: &mut AppState) {
    let prompt_open = state.youtube_playlist_prompt.is_some() || state.music_download_prompt_open();
    if state.active_tab == AppTab::Prepare && !state.should_show_prepare_tab() {
        state.active_tab = AppTab::Main;
    }

    let panel_fill = root_ui.visuals().panel_fill;

    CentralPanel::default()
        .frame(egui::Frame::NONE.fill(panel_fill))
        .show_inside(root_ui, |ui| {
            titlebar::render_titlebar(ui, state);

            egui::Frame::NONE
                .fill(panel_fill)
                .inner_margin(egui::Margin {
                    left: 8,
                    right: 8,
                    top: 4,
                    bottom: 6,
                })
                .show(ui, |ui| {
                    ui.add_enabled_ui(!prompt_open, |ui| {
                        if state.format_picker.open {
                            format_picker::render_format_picker_screen(ui, state);
                            return;
                        }

                        if state.should_show_prepare_tab() {
                            match state.active_tab {
                                AppTab::Options => options_tab::render_options_tab(ui, state),
                                AppTab::Log if state.config.show_log_tab => {
                                    processing_tab::render_log_tab(ui, state);
                                }
                                _ => {
                                    state.active_tab = AppTab::Prepare;
                                    prepare_tab::render_prepare_tab(ui, state);
                                }
                            }
                        } else {
                            match state.active_tab {
                                AppTab::Prepare => main_tab::render_main_tab(ui, state),
                                AppTab::Main => main_tab::render_main_tab(ui, state),
                                AppTab::Advance => advance_tab::render_advance_tab(ui, state),
                                AppTab::Options => options_tab::render_options_tab(ui, state),
                                AppTab::Log => {
                                    if state.config.show_log_tab {
                                        processing_tab::render_log_tab(ui, state);
                                    } else {
                                        state.active_tab = AppTab::Options;
                                        options_tab::render_options_tab(ui, state);
                                    }
                                }
                            }
                        }

                        if !state.last_action.is_empty() && !state.should_show_prepare_tab() {
                            ui.separator();
                            ui.small(state.localize_message(&state.last_action));
                        }
                    });
                });
        });

    options_tab::render_youtube_playlist_prompt(root_ui.ctx(), state);
    options_tab::render_music_download_prompt(root_ui.ctx(), state);
}
