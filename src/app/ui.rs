mod advance_tab;
mod common;
mod format_picker;
mod item_card;
mod main_tab;
mod options_tab;
mod prepare_tab;
mod processing_tab;

use eframe::egui::{self, CentralPanel};

use crate::app::state::{AppState, AppTab};

use self::common::UiText;

pub fn render_app(ctx: &egui::Context, state: &mut AppState) {
    let prompt_open = state.youtube_playlist_prompt.is_some();
    if state.active_tab == AppTab::Prepare && !state.should_show_prepare_tab() {
        state.active_tab = AppTab::Main;
    }

    CentralPanel::default().show(ctx, |ui| {
        ui.add_enabled_ui(!prompt_open, |ui| {
            if state.format_picker.open {
                format_picker::render_format_picker_screen(ui, state);
                return;
            }

            if state.should_show_prepare_tab() {
                state.active_tab = AppTab::Prepare;
                prepare_tab::render_prepare_tab(ui, state);
            } else {
                let tab_main = state.tr(UiText::TAB_MAIN);
                let tab_advance = state.tr(UiText::TAB_ADVANCE);
                let tab_options = state.tr(UiText::TAB_OPTIONS);
                let tab_log = state.tr(UiText::TAB_LOG);
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut state.active_tab, AppTab::Main, tab_main);
                    ui.selectable_value(&mut state.active_tab, AppTab::Advance, tab_advance);
                    ui.selectable_value(&mut state.active_tab, AppTab::Options, tab_options);
                    if state.config.show_log_tab {
                        ui.selectable_value(&mut state.active_tab, AppTab::Log, tab_log);
                    }
                });
                ui.separator();

                match state.active_tab {
                    AppTab::Prepare => main_tab::render_main_tab(ui, state),
                    AppTab::Main => main_tab::render_main_tab(ui, state),
                    AppTab::Advance => advance_tab::render_advance_tab(ui, state),
                    AppTab::Options => options_tab::render_options_tab(ui, state),
                    AppTab::Processing => {
                        state.active_tab = AppTab::Advance;
                        advance_tab::render_advance_tab(ui, state);
                    }
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

    options_tab::render_youtube_playlist_prompt(ctx, state);
}
