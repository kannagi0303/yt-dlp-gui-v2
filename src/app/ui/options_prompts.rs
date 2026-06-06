use eframe::egui::{self, Color32, RichText, Ui};

use crate::app::state::{
    AppState, MusicDownloadMode, MusicOriginalPreference, YoutubePlaylistPromptKind,
};
use crate::app::widgets::icon::{AppIcon, icon_image};
use crate::app::widgets::url_input::{accent_green_for_ui, accent_red_for_ui};

use super::semantic_ui_metrics;

pub(super) fn render_music_download_prompt(ctx: &egui::Context, state: &mut AppState) {
    if !state.music_download_prompt_open() {
        return;
    }

    egui::Window::new(state.ui_i18n_text_for_key("options.music_download_format"))
        .id(egui::Id::new("music-download-format-prompt-window-v4"))
        .collapsible(false)
        .resizable(false)
        .anchor(
            egui::Align2::CENTER_CENTER,
            semantic_ui_metrics::modal_prompt_center_anchor_vector(),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing =
                semantic_ui_metrics::music_download_prompt_item_spacing();
            let prompt_width = music_download_prompt_content_width(ctx, ui, state);
            ui.set_width(prompt_width);
            ui.set_max_width(prompt_width);

            render_music_download_preference_panel(ui, state, prompt_width);

            ui.add_space(
                semantic_ui_metrics::music_download_prompt_action_to_panel_vertical_spacing(),
            );
            render_music_download_prompt_actions(ui, state, prompt_width);
        });
}

fn render_music_download_preference_panel(ui: &mut Ui, state: &mut AppState, prompt_width: f32) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y =
            semantic_ui_metrics::music_download_prompt_preference_panel_vertical_spacing();
        ui.label(
            RichText::new(state.ui_i18n_text_for_key("options.music_download_audio_label"))
                .strong()
                .size(semantic_ui_metrics::playlist_prompt_body_text_size()),
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
            let width =
                semantic_ui_metrics::music_download_prompt_choice_chip_width_for_visible_label(
                    ui, &label,
                );
            (preference, label, width)
        })
        .collect::<Vec<_>>();
    let chip_spacing = semantic_ui_metrics::music_download_prompt_choice_chip_horizontal_spacing();
    let total_width = chip_items.iter().map(|(_, _, width)| *width).sum::<f32>()
        + chip_spacing * chip_items.len().saturating_sub(1) as f32;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = chip_spacing;
        ui.add_space(
            semantic_ui_metrics::remaining_width_before_centered_prompt_content(
                prompt_width,
                total_width,
            ),
        );
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
    let cancel_label = state.ui_i18n_text_for_key("options.cancel").to_owned();
    let download_label = state.ui_i18n_text_for_key("action.download").to_owned();
    let cancel_width = semantic_ui_metrics::prompt_action_button_width_for_icon_and_visible_text(
        ui,
        &cancel_label,
    );
    let download_width = semantic_ui_metrics::prompt_action_button_width_for_icon_and_visible_text(
        ui,
        &download_label,
    );
    let action_spacing =
        semantic_ui_metrics::music_download_prompt_action_button_horizontal_spacing();
    let total_width = cancel_width + download_width + action_spacing;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = action_spacing;
        ui.add_space(
            semantic_ui_metrics::remaining_width_before_right_aligned_prompt_actions(
                prompt_width,
                total_width,
            ),
        );

        let cancel_button = prompt_action_button(
            ui,
            AppIcon::WindowClose,
            accent_red_for_ui(ui),
            &cancel_label,
            cancel_width,
            semantic_ui_metrics::prompt_action_button_height_for_playlist_decision(),
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
            semantic_ui_metrics::prompt_action_button_height_for_playlist_decision(),
        );
        if ui.add(download_button).clicked() {
            state.confirm_music_download_choice();
        }
    });
}

fn music_download_prompt_content_width(ctx: &egui::Context, ui: &Ui, state: &AppState) -> f32 {
    let audio_label_text = state.ui_i18n_text_for_key("options.music_download_audio_label");
    let title_width = semantic_ui_metrics::music_download_prompt_title_width_for_audio_label(
        ui,
        audio_label_text,
    );
    let preference_labels = MusicOriginalPreference::ALL
        .into_iter()
        .map(|preference| music_download_preference_label(state, preference).to_owned())
        .collect::<Vec<_>>();
    let chip_spacing = semantic_ui_metrics::music_download_prompt_choice_chip_horizontal_spacing();
    let chips_width = preference_labels
        .iter()
        .map(|label| {
            semantic_ui_metrics::music_download_prompt_choice_chip_width_for_visible_label(
                ui, label,
            )
        })
        .sum::<f32>()
        + chip_spacing * preference_labels.len().saturating_sub(1) as f32;
    let cancel_text = state.ui_i18n_text_for_key("options.cancel");
    let download_text = state.ui_i18n_text_for_key("action.download");
    let action_width =
        semantic_ui_metrics::prompt_action_button_width_for_icon_and_visible_text(ui, cancel_text)
            + semantic_ui_metrics::prompt_action_button_width_for_icon_and_visible_text(
                ui,
                download_text,
            )
            + semantic_ui_metrics::music_download_prompt_action_button_horizontal_spacing();
    let content_width = title_width.max(chips_width).max(action_width);
    let max_prompt_width = semantic_ui_metrics::music_download_prompt_maximum_width_for_viewport(
        ctx.content_rect().width(),
    );
    semantic_ui_metrics::music_download_prompt_width_from_content_width(
        content_width,
        max_prompt_width,
    )
}

fn music_download_preference_label(state: &AppState, preference: MusicOriginalPreference) -> &str {
    match preference {
        MusicOriginalPreference::Auto => {
            state.ui_i18n_text_for_key("options.music_download_preference_best")
        }
        MusicOriginalPreference::PreferOpus => "Opus",
        MusicOriginalPreference::PreferAac => "AAC",
        MusicOriginalPreference::PreferMp3 => "MP3",
    }
}

fn music_prompt_choice_chip(
    ui: &mut Ui,
    label: &str,
    selected: bool,
    width: f32,
) -> egui::Response {
    let desired_size = egui::vec2(
        width,
        semantic_ui_metrics::music_download_prompt_choice_chip_height(),
    );
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
            egui::Stroke::new(
                semantic_ui_metrics::playlist_prompt_choice_selected_stroke_width(),
                visuals.selection.stroke.color,
            )
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
        ui.painter().rect_filled(
            rect,
            semantic_ui_metrics::playlist_prompt_choice_corner_radius(),
            fill,
        );
        ui.painter().rect_stroke(
            rect,
            semantic_ui_metrics::playlist_prompt_choice_corner_radius(),
            stroke,
            egui::StrokeKind::Outside,
        );
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

pub(super) fn render_youtube_playlist_prompt(ctx: &egui::Context, state: &mut AppState) {
    let Some(prompt) = state.youtube_playlist_prompt.as_ref() else {
        return;
    };
    let prompt_kind = prompt.kind;
    let prompt_risk = prompt.risk;

    match prompt_kind {
        YoutubePlaylistPromptKind::VideoAndPlaylist => {
            let title = state
                .ui_i18n_text_for_key("options.this_url_contains_both_a_video_and_a_playlis")
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
                state.ui_i18n_text_for_key("options.detected"),
                state.ui_i18n_text_for_key(risk.kind.label_key())
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

fn render_playlist_prompt_window(
    ctx: &egui::Context,
    state: &mut AppState,
    title: String,
    risk_note_key: Option<&'static str>,
    prompt_kind: YoutubePlaylistPromptKind,
) {
    egui::Window::new(state.ui_i18n_text_for_key("options.playlist_prompt"))
        .id(egui::Id::new("youtube-playlist-prompt-window-fit-v5"))
        .collapsible(false)
        .resizable(false)
        .anchor(
            egui::Align2::CENTER_CENTER,
            semantic_ui_metrics::modal_prompt_center_anchor_vector(),
        )
        .show(ctx, |ui| {
            let prompt_width = semantic_ui_metrics::playlist_prompt_content_width();
            ui.set_width(prompt_width);
            ui.set_max_width(prompt_width);
            ui.spacing_mut().item_spacing = semantic_ui_metrics::playlist_prompt_item_spacing();
            ui.spacing_mut().button_padding = semantic_ui_metrics::playlist_prompt_button_padding();

            render_playlist_prompt_body(ui, state, &title, risk_note_key, prompt_kind);
            ui.add_space(semantic_ui_metrics::playlist_prompt_actions_to_body_vertical_spacing());
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
            state
                .ui_i18n_text_for_key("options.which_one_should_be_loaded")
                .to_owned(),
            state
                .ui_i18n_text_for_key("options.both_video_and_playlist_were_detected")
                .to_owned(),
        ),
        YoutubePlaylistPromptKind::HighRiskPlaylist => (
            title.to_owned(),
            risk_note_key
                .map(|key| state.ui_i18n_text_for_key(key))
                .unwrap_or(
                    state.ui_i18n_text_for_key("options.this_playlist_may_contain_many_items"),
                )
                .to_owned(),
        ),
    };

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y =
            semantic_ui_metrics::playlist_prompt_body_vertical_spacing();
        ui.label(
            RichText::new(heading)
                .strong()
                .size(semantic_ui_metrics::playlist_prompt_title_text_size()),
        );
        ui.label(RichText::new(body).size(semantic_ui_metrics::playlist_prompt_body_text_size()));
    });
}

fn render_playlist_prompt_actions(
    ui: &mut Ui,
    state: &mut AppState,
    prompt_kind: YoutubePlaylistPromptKind,
) {
    let button_height = semantic_ui_metrics::prompt_action_button_height_for_playlist_decision();
    let spacing = semantic_ui_metrics::playlist_prompt_action_button_horizontal_spacing();

    match prompt_kind {
        YoutubePlaylistPromptKind::VideoAndPlaylist => {
            let video_text = state.ui_i18n_text_for_key("options.video").to_owned();
            let playlist_text = state.ui_i18n_text_for_key("options.playlist").to_owned();
            let cancel_text = state.ui_i18n_text_for_key("options.cancel").to_owned();
            let video_width =
                semantic_ui_metrics::prompt_action_button_width_for_icon_and_visible_text(
                    ui,
                    &video_text,
                );
            let playlist_width =
                semantic_ui_metrics::prompt_action_button_width_for_icon_and_visible_text(
                    ui,
                    &playlist_text,
                );
            let cancel_width =
                semantic_ui_metrics::prompt_action_button_width_for_icon_and_visible_text(
                    ui,
                    &cancel_text,
                );
            let total_width = video_width + playlist_width + cancel_width + spacing * 2.0;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                ui.add_space(
                    semantic_ui_metrics::remaining_width_before_right_aligned_prompt_actions(
                        semantic_ui_metrics::playlist_prompt_content_width(),
                        total_width,
                    ),
                );

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
            let load_text = state.ui_i18n_text_for_key("options.load").to_owned();
            let cancel_text = state.ui_i18n_text_for_key("options.cancel").to_owned();
            let confirm_width =
                semantic_ui_metrics::prompt_action_button_width_for_icon_and_visible_text(
                    ui, &load_text,
                );
            let cancel_width =
                semantic_ui_metrics::prompt_action_button_width_for_icon_and_visible_text(
                    ui,
                    &cancel_text,
                );
            let total_width = confirm_width + cancel_width + spacing;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                ui.add_space(
                    semantic_ui_metrics::remaining_width_before_right_aligned_prompt_actions(
                        semantic_ui_metrics::playlist_prompt_content_width(),
                        total_width,
                    ),
                );

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

fn prompt_action_button<'a>(
    ui: &Ui,
    icon: AppIcon,
    icon_color: Color32,
    label: &'a str,
    width: f32,
    height: f32,
) -> egui::Button<'a> {
    let size = semantic_ui_metrics::standard_icon_size_from_current_control_metrics(ui);
    egui::Button::image_and_text(
        icon_image(icon, size, icon_color),
        RichText::new(label).size(size),
    )
    .min_size(egui::vec2(width, height))
}
