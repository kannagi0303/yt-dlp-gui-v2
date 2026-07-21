// Shared shell and layout helpers.
mod common;
mod measure;
mod semantic_ui_metrics;
mod settings_detail_template;
mod titlebar;

// Generic views and reusable page widgets.
mod about_tab;
mod about_tab_controls;
mod about_tab_template;
mod compact_row;
mod format_picker;
mod format_picker_content;
mod format_picker_filters;
mod format_picker_header;
mod format_picker_section;
mod format_picker_selection;
mod format_picker_subtitle;
mod format_picker_table;
mod format_picker_template;
mod format_picker_time_range;
mod item_card;
mod item_card_compact;
mod item_card_output_actions;
mod item_card_template;
mod prepare_tab;
mod prepare_tab_template;
mod single_mode;
mod single_mode_format_rows;
mod single_mode_preview;
mod single_mode_template;

// Processing / log viewer.
mod processing_command_viewer;
mod processing_conversion;
mod processing_conversion_template;
mod processing_log_table;
mod processing_log_viewer;
mod processing_tab;
mod processing_tab_template;

// Main tab template and role controls.
mod main_tab;
mod main_tab_controls;
mod main_tab_dependency_notice;
mod main_tab_monitor_control;
mod main_tab_music_aura;
mod main_tab_music_aura_dynamics;
mod main_tab_music_controls;
mod main_tab_music_lyrics;
mod main_tab_music_panel;
mod main_tab_music_stage_controls;
mod main_tab_output_actions;
mod main_tab_template;
mod main_tab_text_fields;
mod main_tab_url_action;

// Options tab sections.
mod options_behavior;
mod options_cache;
mod options_language;
mod options_layout;
mod options_prompts;
mod options_tab;
mod options_tab_template;
mod options_tool_paths;
mod options_window;

// Advance tab sections.
mod advance_command_previews;
mod advance_conversion;
mod advance_conversion_template;
mod advance_cookie_manager;
mod advance_cookie_manager_template;
mod advance_cookie_rescue;
mod advance_download_controls;
mod advance_network;
mod advance_post_processing;
mod advance_source;
mod advance_tab;
mod advance_tab_template;

// XAML-like template layer.
mod xaml_layout_contracts;
mod xaml_rect_template;
mod xaml_taffy_styles;
mod xaml_template_renderer;
mod xaml_ui_nodes;

use eframe::egui::{self, CentralPanel, Ui};

use crate::app::state::{AppState, AppTab};

pub(crate) struct UiRenderResources {
    music_player_aura: Option<main_tab_music_aura::MusicPlayerAuraRenderer>,
}

impl UiRenderResources {
    pub(crate) fn new(gl: Option<&eframe::glow::Context>) -> Self {
        let music_player_aura = gl.and_then(|gl| {
            main_tab_music_aura::MusicPlayerAuraRenderer::new(gl)
                .map_err(|error| {
                    eprintln!("[music-aura] GPU renderer unavailable: {error}");
                })
                .ok()
        });
        Self { music_player_aura }
    }

    fn music_player_aura(&self) -> Option<main_tab_music_aura::MusicPlayerAuraRenderer> {
        self.music_player_aura.clone()
    }

    pub(crate) fn destroy(&mut self, gl: Option<&eframe::glow::Context>) {
        if let (Some(renderer), Some(gl)) = (self.music_player_aura.take(), gl) {
            renderer.destroy(gl);
        }
    }
}

pub fn render_app(root_ui: &mut Ui, state: &mut AppState, render_resources: &UiRenderResources) {
    let prompt_open = state.youtube_playlist_prompt.is_some()
        || state.music_download_prompt_open()
        || state.youtube_login_rescue_dialog_visible();
    if state.active_tab == AppTab::Prepare && !state.should_show_prepare_tab() {
        state.active_tab = AppTab::Main;
    }

    let panel_fill = root_ui.visuals().panel_fill;

    CentralPanel::default()
        .frame(egui::Frame::NONE.fill(panel_fill))
        .show(root_ui, |ui| {
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
                                AppTab::About => about_tab::render_about_tab(ui, state),
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
                                AppTab::Prepare => {
                                    main_tab::render_main_tab(ui, state, render_resources)
                                }
                                AppTab::Main => {
                                    main_tab::render_main_tab(ui, state, render_resources)
                                }
                                AppTab::Advance => advance_tab::render_advance_tab(ui, state),
                                AppTab::Options => options_tab::render_options_tab(ui, state),
                                AppTab::About => about_tab::render_about_tab(ui, state),
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

    options_prompts::render_youtube_playlist_prompt(root_ui.ctx(), state);
    options_prompts::render_music_download_prompt(root_ui.ctx(), state);
    advance_cookie_rescue::render_youtube_login_rescue_dialog(root_ui.ctx(), state);
}
