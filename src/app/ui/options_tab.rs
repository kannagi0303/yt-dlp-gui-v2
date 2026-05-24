use std::path::PathBuf;

use eframe::egui::{self, Color32, RichText, ScrollArea, Ui};
use egui_extras::{Size, StripBuilder};

use crate::app::state::{AppState, OptionsDetailPage, YoutubePlaylistPromptKind};
use crate::app::widgets::icon::icon_image;
use crate::app::widgets::url_input::{
    DisplayPathInput, accent_green_for_ui, accent_red_for_ui,
};
use crate::i18n::LanguageSelection;
use crate::infrastructure::{
    DependencyTool, OutputFileActionMode, ThemeAccentColor, ThemeMode, YoutubeVideoPlaylistMode,
    dependency_tool_exists,
};

use crate::app::widgets::icon::AppIcon;

use super::common::{
    form_row_label, icon_button_text_size, icon_text_button, measure_label_width,
    natural_icon_button_width, settings_scroll_content, settings_section,
    text_trailing_icon_button,
};

pub(super) fn render_options_tab(ui: &mut Ui, state: &mut AppState) {
    match state.options_detail_page {
        Some(OptionsDetailPage::Language) => render_language_detail_page(ui, state),
        None => render_options_root_page(ui, state),
    }
}

fn render_options_root_page(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical()
        .id_salt("options-tab-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            settings_scroll_content(ui, |ui| {
                let label_width = options_label_width(ui, state);
                ui.vertical(|ui| {
                    render_language_group(ui, state, label_width);
                    render_tool_paths_group(ui, state, label_width);
                    render_behavior_group(ui, state, label_width);
                    render_tabs_group(ui, state, label_width);
                    render_playlist_group(ui, state, label_width);
                    render_file_action_group(ui, state, label_width);
                    render_cache_group(ui, state, label_width);
                    render_window_group(ui, state, label_width);
                });
            });
        });
}

pub(super) fn render_youtube_playlist_prompt(ctx: &egui::Context, state: &mut AppState) {
    let Some(prompt) = state.youtube_playlist_prompt.as_ref() else {
        return;
    };
    let prompt_kind = prompt.kind;
    let prompt_risk = prompt.risk;
    let prompt_source = prompt.source.clone();

    let title = match prompt_kind {
        YoutubePlaylistPromptKind::VideoAndPlaylist => {
            state.tr("options.this_url_contains_both_a_video_and_a_playlis")
        }
        YoutubePlaylistPromptKind::HighRiskPlaylist => {
            let risk = prompt_risk.expect("high risk prompt should include risk");
            return render_playlist_prompt_window(
                ctx,
                state,
                &format!(
                    "{}{}",
                    state.tr("options.detected"),
                    state.tr(risk.kind.label())
                ),
                Some(risk.note),
                YoutubePlaylistPromptKind::HighRiskPlaylist,
            );
        }
    };
    let note = prompt_risk.map(|risk| risk.note);
    let _ = prompt_source;
    render_playlist_prompt_window(
        ctx,
        state,
        title,
        note,
        YoutubePlaylistPromptKind::VideoAndPlaylist,
    );
}

const PLAYLIST_PROMPT_WIDTH: f32 = 320.0;
const PLAYLIST_PROMPT_TITLE_SIZE: f32 = 16.0;
const PLAYLIST_PROMPT_BODY_SIZE: f32 = 13.0;
const PLAYLIST_PROMPT_ACTION_HEIGHT: f32 = 30.0;

fn render_playlist_prompt_window(
    ctx: &egui::Context,
    state: &mut AppState,
    title: &str,
    risk_note: Option<&str>,
    prompt_kind: YoutubePlaylistPromptKind,
) {
    egui::Window::new(state.tr("options.playlist_prompt"))
        .id(egui::Id::new("youtube-playlist-prompt-window-fit-v5"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(PLAYLIST_PROMPT_WIDTH);
            ui.set_max_width(PLAYLIST_PROMPT_WIDTH);
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);

            render_playlist_prompt_body(ui, state, title, risk_note, prompt_kind);
            ui.add_space(8.0);
            render_playlist_prompt_actions(ui, state, prompt_kind);
        });
}

fn render_playlist_prompt_body(
    ui: &mut Ui,
    state: &AppState,
    title: &str,
    risk_note: Option<&str>,
    prompt_kind: YoutubePlaylistPromptKind,
) {
    let (heading, body) = match prompt_kind {
        YoutubePlaylistPromptKind::VideoAndPlaylist => (
            state.tr("options.which_one_should_be_loaded").to_owned(),
            state
                .tr("options.both_video_and_playlist_were_detected")
                .to_owned(),
        ),
        YoutubePlaylistPromptKind::HighRiskPlaylist => (
            title.to_owned(),
            risk_note
                .unwrap_or(state.tr("options.this_playlist_may_contain_many_items"))
                .to_owned(),
        ),
    };

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 5.0;
        ui.label(
            RichText::new(heading)
                .strong()
                .size(PLAYLIST_PROMPT_TITLE_SIZE),
        );
        ui.label(RichText::new(body).size(PLAYLIST_PROMPT_BODY_SIZE));
    });
}

fn render_playlist_prompt_actions(
    ui: &mut Ui,
    state: &mut AppState,
    prompt_kind: YoutubePlaylistPromptKind,
) {
    let button_height = prompt_action_height(ui);
    let spacing = 8.0;

    match prompt_kind {
        YoutubePlaylistPromptKind::VideoAndPlaylist => {
            let video_width = prompt_action_width(ui, state.tr("options.video"));
            let playlist_width = prompt_action_width(ui, state.tr("options.playlist"));
            let cancel_width = prompt_action_width(ui, state.tr("options.cancel"));
            let total_width = video_width + playlist_width + cancel_width + spacing * 2.0;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                ui.add_space((PLAYLIST_PROMPT_WIDTH - total_width).max(0.0));

                let video_button = prompt_action_button(
                    ui,
                    AppIcon::Video,
                    accent_green_for_ui(ui),
                    state.tr("options.video"),
                    video_width,
                    button_height,
                );
                if ui.add(video_button).clicked() {
                    state.confirm_youtube_playlist_prompt_as_video();
                }

                let playlist_button = prompt_action_button(
                    ui,
                    AppIcon::Import,
                    accent_green_for_ui(ui),
                    state.tr("options.playlist"),
                    playlist_width,
                    button_height,
                );
                if ui.add(playlist_button).clicked() {
                    state.confirm_youtube_playlist_prompt();
                }

                let cancel_button = prompt_action_button(
                    ui,
                    AppIcon::WindowClose,
                    accent_red_for_ui(ui),
                    state.tr("options.cancel"),
                    cancel_width,
                    button_height,
                );
                if ui.add(cancel_button).clicked() {
                    state.cancel_youtube_playlist_prompt();
                }
            });
        }
        YoutubePlaylistPromptKind::HighRiskPlaylist => {
            let confirm_width = prompt_action_width(ui, state.tr("options.load"));
            let cancel_width = prompt_action_width(ui, state.tr("options.cancel"));
            let total_width = confirm_width + cancel_width + spacing;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                ui.add_space((PLAYLIST_PROMPT_WIDTH - total_width).max(0.0));

                let confirm_button = prompt_action_button(
                    ui,
                    AppIcon::Import,
                    accent_green_for_ui(ui),
                    state.tr("options.load"),
                    confirm_width,
                    button_height,
                );
                if ui.add(confirm_button).clicked() {
                    state.confirm_youtube_playlist_prompt();
                }

                let cancel_button = prompt_action_button(
                    ui,
                    AppIcon::WindowClose,
                    accent_red_for_ui(ui),
                    state.tr("options.cancel"),
                    cancel_width,
                    button_height,
                );
                if ui.add(cancel_button).clicked() {
                    state.cancel_youtube_playlist_prompt();
                }
            });
        }
    }
}

fn prompt_action_height(_ui: &Ui) -> f32 {
    PLAYLIST_PROMPT_ACTION_HEIGHT
}

fn prompt_action_width(ui: &Ui, label: &str) -> f32 {
    (natural_icon_button_width(ui, label) + ui.spacing().button_padding.x).max(64.0)
}

fn prompt_action_button(
    ui: &Ui,
    icon: AppIcon,
    icon_color: Color32,
    label: &str,
    width: f32,
    height: f32,
) -> egui::Button<'static> {
    let size = icon_button_text_size(ui);
    egui::Button::image_and_text(
        icon_image(icon, size, icon_color),
        RichText::new(label).size(size),
    )
    .min_size(egui::vec2(width, height))
}

fn render_behavior_group(ui: &mut Ui, state: &mut AppState, label_width: f32) {
    settings_section(ui, state.tr("options.behavior"), |ui| {
        form_row_label(ui, label_width, state.tr("options.add_action"), |ui| {
            let mut enabled = state.config.direct_download_on_add;
            if ui
                .checkbox(&mut enabled, state.tr("options.download_directly"))
                .changed()
            {
                state.set_direct_download_on_add(enabled);
            }
        });
        form_row_label(
            ui,
            label_width,
            state.tr("options.clipboard_change"),
            |ui| {
                let mut enabled = state.config.clipboard_auto_add;
                if ui
                    .checkbox(&mut enabled, state.tr("options.run_immediately"))
                    .changed()
                {
                    state.set_clipboard_auto_add(enabled);
                }
            },
        );
    });
}

fn render_tabs_group(ui: &mut Ui, state: &mut AppState, label_width: f32) {
    settings_section(ui, state.tr("options.tabs"), |ui| {
        form_row_label(ui, label_width, state.tr("options.log_tab"), |ui| {
            let mut enabled = state.config.show_log_tab;
            if ui
                .checkbox(&mut enabled, state.tr("options.show_log_tab"))
                .changed()
            {
                state.set_show_log_tab(enabled);
            }
        });
    });
}

fn render_playlist_group(ui: &mut Ui, state: &mut AppState, label_width: f32) {
    settings_section(ui, state.tr("options.playlist_2"), |ui| {
        form_row_label(ui, label_width, state.tr("options.with_playlist"), |ui| {
            egui::ComboBox::from_id_salt("youtube-video-playlist-mode")
                .selected_text(match state.config.youtube_video_playlist_mode {
                    YoutubeVideoPlaylistMode::Ask => state.tr("options.ask"),
                    YoutubeVideoPlaylistMode::Video => state.tr("options.single_video"),
                    YoutubeVideoPlaylistMode::Ignore => state.tr("options.full_playlist"),
                })
                .show_ui(ui, |ui| {
                    for (mode, label) in [
                        (YoutubeVideoPlaylistMode::Ask, state.tr("options.ask")),
                        (
                            YoutubeVideoPlaylistMode::Video,
                            state.tr("options.single_video"),
                        ),
                        (
                            YoutubeVideoPlaylistMode::Ignore,
                            state.tr("options.full_playlist"),
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
        });
        form_row_label(
            ui,
            label_width,
            state.tr("options.high_risk_prompt"),
            |ui| {
                let mut enabled = state.config.youtube_high_risk_playlist_prompt;
                if ui.checkbox(&mut enabled, state.tr("options.on")).changed() {
                    state.set_youtube_high_risk_playlist_prompt(enabled);
                }
            },
        );
        form_row_label(ui, label_width, state.tr("options.playlist_count"), |ui| {
            let mut enabled = state.config.batch_limit_enabled;
            if ui
                .checkbox(&mut enabled, state.tr("options.limit"))
                .changed()
            {
                state.set_batch_limit_enabled(enabled);
            }
            let mut count = state.config.batch_limit_count;
            if ui
                .add(
                    egui::DragValue::new(&mut count)
                        .range(1..=9999)
                        .prefix(state.tr("options.max"))
                        .suffix(state.tr("options.items")),
                )
                .changed()
            {
                state.set_batch_limit_count(count);
            }
        });
    });
}

fn render_language_group(ui: &mut Ui, state: &mut AppState, label_width: f32) {
    settings_section(ui, state.tr("options.language"), |ui| {
        form_row_label(ui, label_width, state.tr("options.language"), |ui| {
            let label = language_choice_label(state, state.language_selection());
            if ui
                .add(text_trailing_icon_button(ui, &label, AppIcon::MenuRight))
                .clicked()
            {
                state.open_options_detail_page(OptionsDetailPage::Language);
            }
        });
    });
}

fn render_language_detail_page(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical()
        .id_salt("options-language-page-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            settings_scroll_content(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .button(format!("← {}", state.tr("options.back")))
                        .clicked()
                    {
                        state.close_options_detail_page();
                    }
                    ui.label(RichText::new(state.tr("options.language")).strong());
                });
                ui.add_space(10.0);
                settings_section(ui, state.tr("options.language"), |ui| {
                    for language in LanguageSelection::PICKER_ORDER {
                        render_language_choice_row(ui, state, language);
                    }
                });
            });
        });
}

fn render_language_choice_row(ui: &mut Ui, state: &mut AppState, language: LanguageSelection) {
    const CHECK_WIDTH: f32 = 18.0;
    let selected = state.language_selection() == language;
    let label = language_choice_label(state, language);
    ui.horizontal(|ui| {
        ui.add_sized(
            [CHECK_WIDTH, ui.spacing().interact_size.y],
            egui::Label::new(if selected { "✓" } else { "" }),
        );
        if ui.selectable_label(selected, label).clicked() {
            state.set_language_selection(language);
        }
    });
}

fn language_choice_label(state: &AppState, language: LanguageSelection) -> String {
    match language {
        LanguageSelection::Auto => format!(
            "{} ({})",
            state.tr("options.auto_detect"),
            language.resolve().native_name()
        ),
        _ => language.native_name().to_owned(),
    }
}

fn render_tool_paths_group(ui: &mut Ui, state: &mut AppState, label_width: f32) {
    settings_section(ui, state.tr("options.tool_paths"), |ui| {
        ui.vertical(|ui| {
            tool_path_row(ui, label_width, state, DependencyTool::YtDlp);
            tool_path_row(ui, label_width, state, DependencyTool::Deno);
            tool_path_row(ui, label_width, state, DependencyTool::Ffmpeg);
            tool_path_row(ui, label_width, state, DependencyTool::Aria2c);
        });
    });
}

fn render_file_action_group(ui: &mut Ui, state: &mut AppState, label_width: f32) {
    settings_section(ui, state.tr("options.file_actions"), |ui| {
        form_row_label(ui, label_width, state.tr("options.action_button"), |ui| {
            egui::ComboBox::from_id_salt("output-file-action-mode")
                .selected_text(state.tr(state.config.output_file_action_mode.label()))
                .show_ui(ui, |ui| {
                    for mode in OutputFileActionMode::variants() {
                        if ui
                            .selectable_label(
                                state.config.output_file_action_mode == mode,
                                state.tr(mode.label()),
                            )
                            .clicked()
                        {
                            state.set_output_file_action_mode(mode);
                        }
                    }
                });
        });
    });
}

fn render_cache_group(ui: &mut Ui, state: &mut AppState, label_width: f32) {
    settings_section(ui, state.tr("options.cache"), |ui| {
        form_row_label(ui, label_width, state.tr("options.cache_location"), |ui| {
            egui::ComboBox::from_id_salt("cache-location-mode")
                .selected_text(state.tr(state.tool_paths.cache_mode.label()))
                .show_ui(ui, |ui| {
                    for mode in state.available_cache_location_modes() {
                        let response = ui.selectable_label(
                            state.tool_paths.cache_mode == mode,
                            state.tr(mode.label()),
                        );
                        if response.clicked() {
                            state.set_cache_location_mode(mode);
                        }
                    }
                });
        });
    });
}

fn render_window_group(ui: &mut Ui, state: &mut AppState, label_width: f32) {
    settings_section(ui, state.tr("options.appearance_window"), |ui| {
        form_row_label(ui, label_width, state.tr("options.notifications"), |ui| {
            let mut enabled = state.config.windows_toast_enabled;
            if ui
                .checkbox(&mut enabled, state.tr("options.enable"))
                .changed()
            {
                state.set_windows_toast_enabled(enabled);
            }
        });
        form_row_label(ui, label_width, state.tr("options.theme"), |ui| {
            egui::ComboBox::from_id_salt("theme-mode")
                .selected_text(state.tr(state.config.theme_mode.label()))
                .show_ui(ui, |ui| {
                    for mode in ThemeMode::variants() {
                        if ui
                            .selectable_label(
                                state.config.theme_mode == mode,
                                state.tr(mode.label()),
                            )
                            .clicked()
                        {
                            state.set_theme_mode(mode);
                        }
                    }
                });
        });
        form_row_label(ui, label_width, state.tr("options.theme_color"), |ui| {
            egui::ComboBox::from_id_salt("theme-accent-color")
                .selected_text(state.tr(state.config.theme_accent_color.label()))
                .show_ui(ui, |ui| {
                    for color in ThemeAccentColor::variants() {
                        if ui
                            .selectable_label(
                                state.config.theme_accent_color == color,
                                state.tr(color.label()),
                            )
                            .clicked()
                        {
                            state.set_theme_accent_color(color);
                        }
                    }
                });
        });
        form_row_label(ui, label_width, state.tr("options.ui_scale"), |ui| {
            let mut pending = state.pending_ui_scale_percent();
            if ui
                .add(
                    egui::DragValue::new(&mut pending)
                        .range(80..=200)
                        .speed(1.0)
                        .suffix("%"),
                )
                .changed()
            {
                state.set_pending_ui_scale_percent(pending);
            }

            let has_pending_change = state.ui_scale_has_pending_change();
            ui.add_enabled_ui(has_pending_change, |ui| {
                if ui.button(state.tr("options.apply")).clicked() {
                    state.apply_pending_ui_scale_percent();
                }
            });
            ui.label(format!(
                "{} {}%",
                state.tr("options.current"),
                state.config.ui_scale_percent
            ));
        });
        form_row_label(ui, label_width, state.tr("options.always_on_top"), |ui| {
            let mut enabled = state.config.keep_window_on_top;
            if ui
                .checkbox(&mut enabled, state.tr("options.enable"))
                .changed()
            {
                state.set_keep_window_on_top(enabled);
            }
        });
        form_row_label(ui, label_width, state.tr("options.window_position"), |ui| {
            let mut enabled = state.config.remember_window_position;
            if ui
                .checkbox(&mut enabled, state.tr("options.remember"))
                .changed()
            {
                state.set_remember_window_position(enabled);
            }
        });
        form_row_label(ui, label_width, state.tr("options.window_size"), |ui| {
            let mut enabled = state.config.remember_window_size;
            if ui
                .checkbox(&mut enabled, state.tr("options.remember"))
                .changed()
            {
                state.set_remember_window_size(enabled);
            }
        });
    });
}

fn options_label_width(ui: &Ui, state: &AppState) -> f32 {
    measure_label_width(
        ui,
        &[
            "yt-dlp",
            "Deno",
            "FFmpeg",
            "Aria2",
            state.tr("options.add_action"),
            state.tr("options.clipboard_change"),
            state.tr("options.log_tab"),
            state.tr("options.with_playlist"),
            state.tr("options.high_risk_prompt"),
            state.tr("options.playlist_count"),
            state.tr("options.action_button"),
            state.tr("options.language"),
            state.tr("options.current_language"),
            state.tr("options.notifications"),
            state.tr("options.theme"),
            state.tr("options.theme_color"),
            state.tr("options.ui_scale"),
            state.tr("options.cache_location"),
            state.tr("options.always_on_top"),
            state.tr("options.window_position"),
            state.tr("options.window_size"),
        ],
    )
}

fn tool_path_row(ui: &mut Ui, label_width: f32, state: &mut AppState, tool: DependencyTool) {
    let row_height = ui.spacing().interact_size.y;
    let install_button_width = natural_icon_button_width(ui, state.tr("options.reinstall")).max(
        natural_icon_button_width(ui, state.tr("options.installing")),
    );
    let pick_button_width = natural_icon_button_width(ui, state.tr("advance.browse"));
    let row_width = ui.available_width();
    let label = tool.label();
    let expected_file_name = tool.executable_name();
    let current_value = state.dependency_tool_path(tool).to_owned();
    let trimmed = current_value.trim().to_owned();
    let missing_file = !trimmed.is_empty() && !dependency_tool_exists(&trimmed);
    let is_active = state.installing_dependency_tool() == Some(tool);
    let install_running = state.installing_dependency_tool().is_some();
    let installed = state.dependency_tool_is_installed(tool);
    let button_label = if is_active {
        state.tr("options.installing")
    } else if installed {
        state.tr("options.reinstall")
    } else {
        state.tr("options.install")
    };
    let status = state.dependency_tool_status_text(tool);

    ui.allocate_ui(egui::vec2(row_width, row_height), |ui| {
        StripBuilder::new(ui)
            .size(Size::exact(label_width))
            .size(Size::remainder().at_least(120.0))
            .size(Size::exact(install_button_width))
            .size(Size::exact(pick_button_width))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), row_height),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.label(label);
                        },
                    );
                });
                strip.cell(|ui| {
                    let mut value = current_value.clone();
                    let response = ui.add_sized(
                        [ui.available_width(), row_height],
                        DisplayPathInput::new(&mut value).error(missing_file),
                    );
                    if missing_file {
                        response.on_hover_text(format!(
                            "{}{}",
                            state.tr("options.file_not_found"),
                            current_value
                        ));
                    } else if !trimmed.is_empty() {
                        response.on_hover_text(current_value.clone());
                    }
                });
                strip.cell(|ui| {
                    let response = ui.add_enabled(
                        !install_running,
                        icon_text_button(ui, AppIcon::Download, button_label)
                            .min_size(egui::vec2(ui.available_width(), row_height)),
                    );
                    if response.clicked() {
                        state.install_dependency_tool(tool);
                    }
                    let localized_status = state.localize_message(&status);
                    if is_active {
                        response.on_hover_text(format!(
                            "{}\n{}{}",
                            localized_status,
                            state.tr("options.will_install_to"),
                            tool.default_portable_path()
                        ));
                    } else if install_running {
                        response.on_hover_text(
                            state.tr("options.another_tool_is_being_installed_please_wait"),
                        );
                    } else {
                        response.on_hover_text(format!(
                            "{}\n{}{}",
                            localized_status,
                            state.tr("options.install_to"),
                            tool.default_portable_path()
                        ));
                    }
                });
                strip.cell(|ui| {
                    if ui
                        .add_sized(
                            [ui.available_width(), row_height],
                            icon_text_button(
                                ui,
                                AppIcon::FolderSettings,
                                state.tr("advance.browse"),
                            ),
                        )
                        .clicked()
                    {
                        let mut dialog = rfd::FileDialog::new()
                            .add_filter(state.tr("options.filter_executable"), &["exe"])
                            .set_title(format!(
                                "{} {label} {}",
                                state.tr("options.choose"),
                                state.tr("options.executable")
                            ));
                        if !trimmed.is_empty() {
                            let current_path = PathBuf::from(&trimmed);
                            if let Some(parent) = current_path.parent().filter(|path| path.is_dir())
                            {
                                dialog = dialog.set_directory(parent);
                            }
                        }
                        if let Some(path) = dialog.set_file_name(expected_file_name).pick_file() {
                            set_dependency_tool_path(state, tool, path.display().to_string());
                        }
                    }
                });
            });
    });
}

fn set_dependency_tool_path(state: &mut AppState, tool: DependencyTool, path: String) {
    match tool {
        DependencyTool::YtDlp => state.set_yt_dlp_path(path),
        DependencyTool::Ffmpeg => state.set_ffmpeg_path(path),
        DependencyTool::Aria2c => state.set_aria2c_path(path),
        DependencyTool::Deno => state.set_deno_path(path),
    }
}
