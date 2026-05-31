use std::path::PathBuf;

use eframe::egui::{self, Color32, RichText, ScrollArea, Ui};
use egui_taffy::Tui;

use crate::app::state::{
    AppState, MusicDownloadMode, MusicOriginalPreference, OptionsDetailPage,
    YoutubePlaylistPromptKind,
};
use crate::app::widgets::icon::icon_image;
use crate::app::widgets::url_input::{
    AppTextBox, AppTextBoxSyntax, accent_green_for_ui, accent_red_for_ui,
};
use crate::i18n::LanguageSelection;
use crate::infrastructure::{
    CacheLocationMode, DependencyTool, OutputFileActionMode, ThemeAccentColor, ThemeMode,
    YoutubeVideoPlaylistMode, dependency_tool_exists,
};

use crate::app::widgets::icon::AppIcon;

use super::common::{
    icon_button_text_size, icon_text_button, measure_label_width, natural_icon_button_width,
    settings_scroll_content, settings_section, settings_taffy_form_row,
    settings_taffy_scroll_content, settings_taffy_section, text_trailing_icon_button,
};
use super::measure::{WidthRange, measured_text_width};

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
            let metrics = OptionsLayoutMetrics::new(ui, state);
            settings_taffy_scroll_content(ui, "options-root-settings-taffy", |tui| {
                render_language_group(tui, state, metrics.label_width);
                render_tool_paths_group(tui, state, &metrics);
                render_behavior_group(tui, state, metrics.label_width);
                render_tabs_group(tui, state, metrics.label_width);
                render_playlist_group(tui, state, metrics.label_width);
                render_file_action_group(tui, state, metrics.label_width);
                render_cache_group(tui, state, metrics.label_width);
                render_window_group(tui, state, metrics.label_width);
            });
        });
}

pub(super) fn render_music_download_prompt(ctx: &egui::Context, state: &mut AppState) {
    if !state.music_download_prompt_open() {
        return;
    }

    egui::Window::new(state.ui_tr("options.music_download_format"))
        .id(egui::Id::new("music-download-format-prompt-window-v4"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
            let prompt_width = music_download_prompt_content_width(ctx, ui, state);
            ui.set_width(prompt_width);
            ui.set_max_width(prompt_width);

            render_music_download_preference_panel(ui, state, prompt_width);

            ui.add_space(8.0);
            render_music_download_prompt_actions(ui, state, prompt_width);
        });
}

fn render_music_download_preference_panel(ui: &mut Ui, state: &mut AppState, prompt_width: f32) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 5.0;
        ui.label(
            RichText::new(state.ui_tr("options.music_download_audio_label"))
                .strong()
                .size(PLAYLIST_PROMPT_BODY_SIZE),
        );
        render_music_download_preference_chips(ui, state, prompt_width);
    });
}

fn render_music_download_preference_chips(ui: &mut Ui, state: &mut AppState, prompt_width: f32) {
    let choice = state.music_download_prompt_choice();
    let chip_items = MusicOriginalPreference::ALL
        .into_iter()
        .map(|preference| {
            let label = music_download_preference_label(state, preference).to_owned();
            let width = music_download_preference_chip_width(ui, &label);
            (preference, label, width)
        })
        .collect::<Vec<_>>();
    let total_width = chip_items.iter().map(|(_, _, width)| *width).sum::<f32>()
        + MUSIC_DOWNLOAD_PROMPT_CHIP_GAP * chip_items.len().saturating_sub(1) as f32;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = MUSIC_DOWNLOAD_PROMPT_CHIP_GAP;
        ui.add_space(((prompt_width - total_width) * 0.5).max(0.0));
        for (preference, label, width) in chip_items {
            if music_prompt_choice_chip(
                ui,
                &label,
                choice.mode == MusicDownloadMode::Original
                    && choice.original_preference == preference,
                width,
            )
            .clicked()
            {
                state.set_music_download_prompt_mode(MusicDownloadMode::Original);
                state.set_music_download_original_preference(preference);
                state.set_music_download_embed_cover(true);
                state.set_music_download_write_tags(true);
            }
        }
    });
}

fn render_music_download_prompt_actions(ui: &mut Ui, state: &mut AppState, prompt_width: f32) {
    let cancel_label = state.ui_tr("options.cancel").to_owned();
    let download_label = state.ui_tr("action.download").to_owned();
    let cancel_width = prompt_action_width(ui, &cancel_label);
    let download_width = prompt_action_width(ui, &download_label);
    let total_width = cancel_width + download_width + MUSIC_DOWNLOAD_PROMPT_ACTION_GAP;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = MUSIC_DOWNLOAD_PROMPT_ACTION_GAP;
        ui.add_space((prompt_width - total_width).max(0.0));

        let cancel_button = prompt_action_button(
            ui,
            AppIcon::WindowClose,
            accent_red_for_ui(ui),
            &cancel_label,
            cancel_width,
            prompt_action_height(ui),
        );
        if ui.add(cancel_button).clicked() {
            state.cancel_music_download_prompt();
        }

        let download_button = prompt_action_button(
            ui,
            AppIcon::Download,
            accent_green_for_ui(ui),
            &download_label,
            download_width,
            prompt_action_height(ui),
        );
        if ui.add(download_button).clicked() {
            state.confirm_music_download_choice();
        }
    });
}

fn music_download_prompt_content_width(ctx: &egui::Context, ui: &Ui, state: &AppState) -> f32 {
    let audio_label_text = state.ui_tr("options.music_download_audio_label");
    let title_width = measured_text_width(
        ui,
        std::iter::once(audio_label_text),
        egui::TextStyle::Button,
        0.0,
        WidthRange::new(48.0, 180.0),
    );
    let preference_labels = MusicOriginalPreference::ALL
        .into_iter()
        .map(|preference| music_download_preference_label(state, preference).to_owned())
        .collect::<Vec<_>>();
    let chips_width = preference_labels
        .iter()
        .map(|label| music_download_preference_chip_width(ui, label))
        .sum::<f32>()
        + MUSIC_DOWNLOAD_PROMPT_CHIP_GAP * preference_labels.len().saturating_sub(1) as f32;
    let cancel_text = state.ui_tr("options.cancel");
    let download_text = state.ui_tr("action.download");
    let action_width = prompt_action_width(ui, cancel_text)
        + prompt_action_width(ui, download_text)
        + MUSIC_DOWNLOAD_PROMPT_ACTION_GAP;
    let content_width = title_width.max(chips_width).max(action_width);
    let max_prompt_width = (ctx.content_rect().width() - 24.0).max(MUSIC_DOWNLOAD_PROMPT_MIN_WIDTH);
    content_width.clamp(MUSIC_DOWNLOAD_PROMPT_MIN_WIDTH, max_prompt_width)
}

fn music_download_preference_label(state: &AppState, preference: MusicOriginalPreference) -> &str {
    match preference {
        MusicOriginalPreference::Auto => state.ui_tr("options.music_download_preference_best"),
        MusicOriginalPreference::PreferOpus => "Opus",
        MusicOriginalPreference::PreferAac => "AAC",
        MusicOriginalPreference::PreferMp3 => "MP3",
    }
}

fn music_download_preference_chip_width(ui: &Ui, label: &str) -> f32 {
    measured_text_width(
        ui,
        std::iter::once(label),
        egui::TextStyle::Button,
        MUSIC_DOWNLOAD_PROMPT_CHIP_HORIZONTAL_PADDING * 2.0,
        WidthRange::new(36.0, 96.0),
    )
}

fn music_prompt_choice_chip(
    ui: &mut Ui,
    label: &str,
    selected: bool,
    width: f32,
) -> egui::Response {
    let desired_size = egui::vec2(width, MUSIC_DOWNLOAD_PROMPT_CHIP_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.visuals();
        let fill = if selected {
            visuals.selection.bg_fill
        } else if response.hovered() {
            visuals.widgets.hovered.weak_bg_fill
        } else {
            Color32::TRANSPARENT
        };
        let stroke = if selected {
            egui::Stroke::new(1.0, visuals.selection.stroke.color)
        } else if response.hovered() {
            visuals.widgets.hovered.bg_stroke
        } else {
            visuals.widgets.noninteractive.bg_stroke
        };
        let text_color = if selected {
            visuals.selection.stroke.color
        } else {
            visuals.text_color()
        };
        ui.painter().rect_filled(rect, 6.0, fill);
        ui.painter()
            .rect_stroke(rect, 6.0, stroke, egui::StrokeKind::Outside);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::TextStyle::Button.resolve(ui.style()),
            text_color,
        );
    }

    response
}

const MUSIC_DOWNLOAD_PROMPT_MIN_WIDTH: f32 = 210.0;
const MUSIC_DOWNLOAD_PROMPT_CHIP_HEIGHT: f32 = 24.0;
const MUSIC_DOWNLOAD_PROMPT_CHIP_HORIZONTAL_PADDING: f32 = 8.0;
const MUSIC_DOWNLOAD_PROMPT_CHIP_GAP: f32 = 6.0;
const MUSIC_DOWNLOAD_PROMPT_ACTION_GAP: f32 = 8.0;

pub(super) fn render_youtube_playlist_prompt(ctx: &egui::Context, state: &mut AppState) {
    let Some(prompt) = state.youtube_playlist_prompt.as_ref() else {
        return;
    };
    let prompt_kind = prompt.kind;
    let prompt_risk = prompt.risk;

    match prompt_kind {
        YoutubePlaylistPromptKind::VideoAndPlaylist => {
            let title = state
                .ui_tr("options.this_url_contains_both_a_video_and_a_playlis")
                .to_owned();
            render_playlist_prompt_window(
                ctx,
                state,
                title,
                None,
                YoutubePlaylistPromptKind::VideoAndPlaylist,
            );
        }
        YoutubePlaylistPromptKind::HighRiskPlaylist => {
            let risk = prompt_risk.expect("high risk prompt should include risk");
            let title = format!(
                "{}{}",
                state.ui_tr("options.detected"),
                state.ui_tr(risk.kind.label_key())
            );
            render_playlist_prompt_window(
                ctx,
                state,
                title,
                Some(risk.kind.note_key()),
                YoutubePlaylistPromptKind::HighRiskPlaylist,
            );
        }
    }
}

const PLAYLIST_PROMPT_WIDTH: f32 = 320.0;
const PLAYLIST_PROMPT_TITLE_SIZE: f32 = 16.0;
const PLAYLIST_PROMPT_BODY_SIZE: f32 = 13.0;
const PLAYLIST_PROMPT_ACTION_HEIGHT: f32 = 30.0;

fn render_playlist_prompt_window(
    ctx: &egui::Context,
    state: &mut AppState,
    title: String,
    risk_note_key: Option<&'static str>,
    prompt_kind: YoutubePlaylistPromptKind,
) {
    egui::Window::new(state.ui_tr("options.playlist_prompt"))
        .id(egui::Id::new("youtube-playlist-prompt-window-fit-v5"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(PLAYLIST_PROMPT_WIDTH);
            ui.set_max_width(PLAYLIST_PROMPT_WIDTH);
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);

            render_playlist_prompt_body(ui, state, &title, risk_note_key, prompt_kind);
            ui.add_space(8.0);
            render_playlist_prompt_actions(ui, state, prompt_kind);
        });
}

fn render_playlist_prompt_body(
    ui: &mut Ui,
    state: &AppState,
    title: &str,
    risk_note_key: Option<&'static str>,
    prompt_kind: YoutubePlaylistPromptKind,
) {
    let (heading, body) = match prompt_kind {
        YoutubePlaylistPromptKind::VideoAndPlaylist => (
            state.ui_tr("options.which_one_should_be_loaded").to_owned(),
            state
                .ui_tr("options.both_video_and_playlist_were_detected")
                .to_owned(),
        ),
        YoutubePlaylistPromptKind::HighRiskPlaylist => (
            title.to_owned(),
            risk_note_key
                .map(|key| state.ui_tr(key))
                .unwrap_or(state.ui_tr("options.this_playlist_may_contain_many_items"))
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
            let video_text = state.ui_tr("options.video").to_owned();
            let playlist_text = state.ui_tr("options.playlist").to_owned();
            let cancel_text = state.ui_tr("options.cancel").to_owned();
            let video_width = prompt_action_width(ui, &video_text);
            let playlist_width = prompt_action_width(ui, &playlist_text);
            let cancel_width = prompt_action_width(ui, &cancel_text);
            let total_width = video_width + playlist_width + cancel_width + spacing * 2.0;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                ui.add_space((PLAYLIST_PROMPT_WIDTH - total_width).max(0.0));

                let video_button = prompt_action_button(
                    ui,
                    AppIcon::Video,
                    accent_green_for_ui(ui),
                    &video_text,
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
                    &playlist_text,
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
                    &cancel_text,
                    cancel_width,
                    button_height,
                );
                if ui.add(cancel_button).clicked() {
                    state.cancel_youtube_playlist_prompt();
                }
            });
        }
        YoutubePlaylistPromptKind::HighRiskPlaylist => {
            let load_text = state.ui_tr("options.load").to_owned();
            let cancel_text = state.ui_tr("options.cancel").to_owned();
            let confirm_width = prompt_action_width(ui, &load_text);
            let cancel_width = prompt_action_width(ui, &cancel_text);
            let total_width = confirm_width + cancel_width + spacing;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                ui.add_space((PLAYLIST_PROMPT_WIDTH - total_width).max(0.0));

                let confirm_button = prompt_action_button(
                    ui,
                    AppIcon::Import,
                    accent_green_for_ui(ui),
                    &load_text,
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
                    &cancel_text,
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

fn prompt_action_button<'a>(
    ui: &Ui,
    icon: AppIcon,
    icon_color: Color32,
    label: &'a str,
    width: f32,
    height: f32,
) -> egui::Button<'a> {
    let size = icon_button_text_size(ui);
    egui::Button::image_and_text(
        icon_image(icon, size, icon_color),
        RichText::new(label).size(size),
    )
    .min_size(egui::vec2(width, height))
}

fn render_behavior_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("options.behavior"), |tui| {
        settings_taffy_form_row(tui, label_width, state.ui_tr("options.add_action"), |ui| {
            let mut enabled = state.config.direct_download_on_add;
            if ui
                .checkbox(&mut enabled, state.ui_tr("options.download_directly"))
                .changed()
            {
                state.set_direct_download_on_add(enabled);
            }
        });
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("options.clipboard_change"),
            |ui| {
                let mut enabled = state.config.clipboard_auto_add;
                if ui
                    .checkbox(&mut enabled, state.ui_tr("options.run_immediately"))
                    .changed()
                {
                    state.set_clipboard_auto_add(enabled);
                }
            },
        );
    });
}

fn render_tabs_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("options.tabs"), |tui| {
        settings_taffy_form_row(tui, label_width, state.ui_tr("options.log_tab"), |ui| {
            let mut enabled = state.config.show_log_tab;
            if ui
                .checkbox(&mut enabled, state.ui_tr("options.show_log_tab"))
                .changed()
            {
                state.set_show_log_tab(enabled);
            }
        });
    });
}

fn render_playlist_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("options.playlist_2"), |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("options.with_playlist"),
            |ui| {
                egui::ComboBox::from_id_salt("youtube-video-playlist-mode")
                    .selected_text(match state.config.youtube_video_playlist_mode {
                        YoutubeVideoPlaylistMode::Ask => state.ui_tr("options.ask"),
                        YoutubeVideoPlaylistMode::Video => state.ui_tr("options.single_video"),
                        YoutubeVideoPlaylistMode::Ignore => state.ui_tr("options.full_playlist"),
                    })
                    .show_ui(ui, |ui| {
                        for (mode, label) in [
                            (YoutubeVideoPlaylistMode::Ask, state.ui_tr("options.ask")),
                            (
                                YoutubeVideoPlaylistMode::Video,
                                state.ui_tr("options.single_video"),
                            ),
                            (
                                YoutubeVideoPlaylistMode::Ignore,
                                state.ui_tr("options.full_playlist"),
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
            state.ui_tr("options.high_risk_prompt"),
            |ui| {
                let mut enabled = state.config.youtube_high_risk_playlist_prompt;
                if ui
                    .checkbox(&mut enabled, state.ui_tr("options.on"))
                    .changed()
                {
                    state.set_youtube_high_risk_playlist_prompt(enabled);
                }
            },
        );
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("options.playlist_count"),
            |ui| {
                let mut enabled = state.config.batch_limit_enabled;
                if ui
                    .checkbox(&mut enabled, state.ui_tr("options.limit"))
                    .changed()
                {
                    state.set_batch_limit_enabled(enabled);
                }
                let mut count = state.config.batch_limit_count;
                if ui
                    .add(
                        egui::DragValue::new(&mut count)
                            .range(1..=9999)
                            .prefix(state.ui_tr("options.max"))
                            .suffix(state.ui_tr("options.items")),
                    )
                    .changed()
                {
                    state.set_batch_limit_count(count);
                }
            },
        );
    });
}

fn render_language_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("options.language"), |tui| {
        settings_taffy_form_row(tui, label_width, state.ui_tr("options.language"), |ui| {
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
                        .button(format!("← {}", state.ui_tr("options.back")))
                        .clicked()
                    {
                        state.close_options_detail_page();
                    }
                    ui.label(RichText::new(state.ui_tr("options.language")).strong());
                });
                ui.add_space(10.0);
                settings_section(ui, state.ui_tr("options.language"), |ui| {
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
            state.ui_tr("options.auto_detect"),
            language.resolve().native_name()
        ),
        _ => language.native_name().to_owned(),
    }
}

fn render_tool_paths_group(tui: &mut Tui, state: &mut AppState, metrics: &OptionsLayoutMetrics) {
    settings_taffy_section(tui, state.ui_tr("options.tool_paths"), |tui| {
        render_tool_auto_detect_row(tui, state, metrics);
        tool_path_row(tui, metrics, state, DependencyTool::YtDlp);
        tool_path_row(tui, metrics, state, DependencyTool::Deno);
        tool_path_row(tui, metrics, state, DependencyTool::Ffmpeg);
        tool_path_row(tui, metrics, state, DependencyTool::Aria2c);
    });
}

fn render_tool_auto_detect_row(
    tui: &mut Tui,
    state: &mut AppState,
    metrics: &OptionsLayoutMetrics,
) {
    settings_taffy_form_row(tui, metrics.label_width, "", |ui| {
        let row_height = ui.spacing().interact_size.y;
        let response = ui.add_sized(
            [metrics.auto_detect_width, row_height],
            icon_text_button(ui, AppIcon::Magnify, metrics.auto_detect_text),
        );
        if response.clicked() {
            state.auto_detect_dependency_tool_paths();
        }
    });
}

fn render_file_action_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("options.file_actions"), |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("options.action_button"),
            |ui| {
                egui::ComboBox::from_id_salt("output-file-action-mode")
                    .selected_text(state.ui_tr(output_file_action_mode_label_key(
                        state.config.output_file_action_mode,
                    )))
                    .show_ui(ui, |ui| {
                        for mode in OutputFileActionMode::variants() {
                            if ui
                                .selectable_label(
                                    state.config.output_file_action_mode == mode,
                                    state.ui_tr(output_file_action_mode_label_key(mode)),
                                )
                                .clicked()
                            {
                                state.set_output_file_action_mode(mode);
                            }
                        }
                    });
            },
        );
    });
}

fn output_file_action_mode_label_key(mode: OutputFileActionMode) -> &'static str {
    match mode {
        OutputFileActionMode::Menu => "options.file_action.show_menu",
        OutputFileActionMode::OpenFolder => "item.open_folder",
        OutputFileActionMode::OpenFile => "item.open_file",
    }
}

fn cache_location_mode_label(state: &AppState, mode: CacheLocationMode) -> &'static str {
    match mode {
        CacheLocationMode::YtDlpDefault => state.ui_tr("options.cache_location.default"),
        CacheLocationMode::V2Cache => "yt-dlp-gui",
        CacheLocationMode::WindowsTemp => "Windows",
    }
}

fn theme_mode_label_key(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::System => "options.theme_mode.system",
        ThemeMode::Light => "options.theme_mode.light",
        ThemeMode::Dark => "options.theme_mode.dark",
    }
}

fn theme_accent_color_label_key(color: ThemeAccentColor) -> &'static str {
    match color {
        ThemeAccentColor::Off => "options.theme_color.off",
        ThemeAccentColor::System => "options.theme_color.blue",
        ThemeAccentColor::Blue => "options.theme_color.soft_blue",
        ThemeAccentColor::Purple => "options.theme_color.purple",
        ThemeAccentColor::Pink => "options.theme_color.pink",
        ThemeAccentColor::Green => "options.theme_color.green",
        ThemeAccentColor::Orange => "options.theme_color.orange",
        ThemeAccentColor::Slate => "options.theme_color.slate",
    }
}

fn render_cache_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("options.cache"), |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("options.cache_location"),
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

        settings_taffy_form_row(tui, label_width, state.ui_tr("options.cache_usage"), |ui| {
            ui.label(state.cache_management_usage_display());
        });

        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("options.cache_cleanup"),
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button(state.ui_tr("options.cache_refresh")).clicked() {
                        state.refresh_cache_management_summary();
                    }
                    if ui
                        .button(state.ui_tr("options.cache_clear_expired"))
                        .clicked()
                    {
                        state.clear_expired_music_cache();
                    }
                    if ui
                        .button(state.ui_tr("options.cache_clear_audio"))
                        .clicked()
                    {
                        state.clear_music_stream_cache();
                    }
                    if ui.button(state.ui_tr("options.cache_clear_all")).clicked() {
                        state.clear_app_cache();
                    }
                });
            },
        );
    });
}

fn render_window_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("options.appearance_window"), |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("options.notifications"),
            |ui| {
                let mut enabled = state.config.windows_toast_enabled;
                if ui
                    .checkbox(&mut enabled, state.ui_tr("options.enable"))
                    .changed()
                {
                    state.set_windows_toast_enabled(enabled);
                }
            },
        );
        settings_taffy_form_row(tui, label_width, state.ui_tr("options.theme"), |ui| {
            egui::ComboBox::from_id_salt("theme-mode")
                .selected_text(state.ui_tr(theme_mode_label_key(state.config.theme_mode)))
                .show_ui(ui, |ui| {
                    for mode in ThemeMode::variants() {
                        if ui
                            .selectable_label(
                                state.config.theme_mode == mode,
                                state.ui_tr(theme_mode_label_key(mode)),
                            )
                            .clicked()
                        {
                            state.set_theme_mode(mode);
                        }
                    }
                });
        });
        settings_taffy_form_row(tui, label_width, state.ui_tr("options.theme_color"), |ui| {
            egui::ComboBox::from_id_salt("theme-accent-color")
                .selected_text(state.ui_tr(theme_accent_color_label_key(
                    state.config.theme_accent_color,
                )))
                .show_ui(ui, |ui| {
                    for color in ThemeAccentColor::variants() {
                        if ui
                            .selectable_label(
                                state.config.theme_accent_color == color,
                                state.ui_tr(theme_accent_color_label_key(color)),
                            )
                            .clicked()
                        {
                            state.set_theme_accent_color(color);
                        }
                    }
                });
        });
        settings_taffy_form_row(tui, label_width, state.ui_tr("options.ui_scale"), |ui| {
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
                if ui.button(state.ui_tr("options.apply")).clicked() {
                    state.apply_pending_ui_scale_percent();
                }
            });
            ui.label(format!(
                "{} {}%",
                state.ui_tr("options.current"),
                state.config.ui_scale_percent
            ));
        });
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("options.always_on_top"),
            |ui| {
                let mut enabled = state.config.keep_window_on_top;
                if ui
                    .checkbox(&mut enabled, state.ui_tr("options.enable"))
                    .changed()
                {
                    state.set_keep_window_on_top(enabled);
                }
            },
        );
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("options.window_position"),
            |ui| {
                let mut enabled = state.config.remember_window_position;
                if ui
                    .checkbox(&mut enabled, state.ui_tr("options.remember"))
                    .changed()
                {
                    state.set_remember_window_position(enabled);
                }
            },
        );
        settings_taffy_form_row(tui, label_width, state.ui_tr("options.window_size"), |ui| {
            let mut enabled = state.config.remember_window_size;
            if ui
                .checkbox(&mut enabled, state.ui_tr("options.remember"))
                .changed()
            {
                state.set_remember_window_size(enabled);
            }
        });
    });
}

struct OptionsLayoutMetrics {
    label_width: f32,
    auto_detect_width: f32,
    install_button_width: f32,
    pick_button_width: f32,
    auto_detect_text: &'static str,
    install_text: &'static str,
    reinstall_text: &'static str,
    installing_text: &'static str,
    browse_text: &'static str,
}

impl OptionsLayoutMetrics {
    fn new(ui: &Ui, state: &AppState) -> Self {
        let auto_detect_text = state.ui_tr("options.auto_detect");
        let install_text = state.ui_tr("options.install");
        let reinstall_text = state.ui_tr("options.reinstall");
        let installing_text = state.ui_tr("options.installing");
        let browse_text = state.ui_tr("advance.browse");

        Self {
            label_width: options_label_width(ui, state),
            auto_detect_width: natural_icon_button_width(ui, auto_detect_text),
            install_button_width: natural_icon_button_width(ui, reinstall_text)
                .max(natural_icon_button_width(ui, installing_text))
                .max(natural_icon_button_width(ui, install_text)),
            pick_button_width: natural_icon_button_width(ui, browse_text),
            auto_detect_text,
            install_text,
            reinstall_text,
            installing_text,
            browse_text,
        }
    }
}

fn options_label_width(ui: &Ui, state: &AppState) -> f32 {
    let add_action_text = state.ui_tr("options.add_action");
    let clipboard_change_text = state.ui_tr("options.clipboard_change");
    let log_tab_text = state.ui_tr("options.log_tab");
    let with_playlist_text = state.ui_tr("options.with_playlist");
    let high_risk_prompt_text = state.ui_tr("options.high_risk_prompt");
    let playlist_count_text = state.ui_tr("options.playlist_count");
    let action_button_text = state.ui_tr("options.action_button");
    let language_text = state.ui_tr("options.language");
    let current_language_text = state.ui_tr("options.current_language");
    let cache_usage_text = state.ui_tr("options.cache_usage");
    let cache_cleanup_text = state.ui_tr("options.cache_cleanup");
    let notifications_text = state.ui_tr("options.notifications");
    let theme_text = state.ui_tr("options.theme");
    let theme_color_text = state.ui_tr("options.theme_color");
    let ui_scale_text = state.ui_tr("options.ui_scale");
    let cache_location_text = state.ui_tr("options.cache_location");
    let always_on_top_text = state.ui_tr("options.always_on_top");
    let window_position_text = state.ui_tr("options.window_position");
    let window_size_text = state.ui_tr("options.window_size");

    measure_label_width(
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

fn tool_path_row(
    tui: &mut Tui,
    metrics: &OptionsLayoutMetrics,
    state: &mut AppState,
    tool: DependencyTool,
) {
    let label = tool.label();
    let expected_file_name = tool.executable_name();
    let current_value = state.dependency_tool_path(tool).to_owned();
    let trimmed = current_value.trim().to_owned();
    let missing_file = !trimmed.is_empty() && !dependency_tool_exists(&trimmed);
    let is_active = state.installing_dependency_tool() == Some(tool);
    let install_running = state.installing_dependency_tool().is_some();
    let installed = state.dependency_tool_is_installed(tool);
    let button_label = if is_active {
        metrics.installing_text
    } else if installed {
        metrics.reinstall_text
    } else {
        metrics.install_text
    };
    settings_taffy_form_row(tui, metrics.label_width, label, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            let row_height = ui.spacing().interact_size.y;
            let buttons_width = metrics.install_button_width + metrics.pick_button_width;
            let gap_width = ui.spacing().item_spacing.x * 2.0;
            let path_width = (ui.available_width() - buttons_width - gap_width).max(120.0);

            let mut value = current_value.clone();
            let response = AppTextBox::new(&mut value)
                .language(state.language())
                .syntax(AppTextBoxSyntax::Path)
                .error(missing_file)
                .desired_width(path_width)
                .editable(false)
                .selectable(false)
                .ui(ui);
            drop(response);

            let response = ui.add_enabled(
                !install_running,
                icon_text_button(ui, AppIcon::Download, button_label)
                    .min_size(egui::vec2(metrics.install_button_width, row_height)),
            );
            if response.clicked() {
                state.install_dependency_tool(tool);
            }
            drop(response);

            if ui
                .add_sized(
                    [metrics.pick_button_width, row_height],
                    icon_text_button(ui, AppIcon::FolderSettings, metrics.browse_text),
                )
                .clicked()
            {
                let mut dialog = rfd::FileDialog::new()
                    .add_filter(state.ui_tr("options.filter_executable"), &["exe"])
                    .set_title(format!(
                        "{} {label} {}",
                        state.ui_tr("options.choose"),
                        state.ui_tr("options.executable")
                    ));
                if !trimmed.is_empty() {
                    let current_path = PathBuf::from(&trimmed);
                    if let Some(parent) = current_path.parent().filter(|path| path.is_dir()) {
                        dialog = dialog.set_directory(parent);
                    }
                }
                if let Some(path) = dialog.set_file_name(expected_file_name).pick_file() {
                    set_dependency_tool_path(state, tool, path.display().to_string());
                }
            }
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
