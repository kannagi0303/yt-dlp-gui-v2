use eframe::egui::{self, Color32, RichText, Spinner, TextStyle, Ui, WidgetText};
use egui_taffy::taffy::prelude::{length, percent};
use egui_taffy::{Tui, TuiBuilderLogic as _, taffy, tui};

use crate::app::state::{AppMode, AppState, MusicLyricsDisplayLine, MusicPlaybackMode};
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::app::widgets::url_input::{
    AppTextBox, AppTextBoxSyntax, accent_red_for_ui, app_textbox_single_line_height,
};

use super::common::{
    UiText, icon_button_text_size, icon_text_button, natural_button_width,
    natural_icon_button_width,
};
use super::item_card::render_batch_list;
use super::single_mode::build_single_mode_item;

const MAIN_SECTION_GAP: f32 = 6.0;
const MAIN_BOTTOM_PADDING: f32 = 2.0;
const MUSIC_PANEL_PADDING_X: f32 = 8.0;
const MUSIC_PANEL_PADDING_Y: f32 = 5.0;
const MUSIC_PANEL_GAP: f32 = 4.0;
const MUSIC_LYRICS_FONT_DELTA: f32 = 4.5;
const MISSING_YT_DLP_TOOLTIP_KEY: &str = "main.tooltip.missing_yt_dlp";
const MISSING_YT_DLP_CALLOUT_WIDTH: f32 = 320.0;
const MAIN_INLINE_CONTROL_GAP_SCALE: f32 = 0.5;
const SINGLE_CONTENT_OUTPUT_GAP_REDUCTION: f32 = 4.0;

pub(super) fn render_main_tab(ui: &mut Ui, state: &mut AppState) {
    let row_height = ui.spacing().interact_size.y;
    let show_music_player = state.music_player_visible();
    let music_lyrics_line = if show_music_player {
        state.music_current_lyrics_display()
    } else {
        None
    };
    let music_lyrics_row_height = if music_lyrics_line.is_some() {
        (egui::TextStyle::Body.resolve(ui.style()).size + MUSIC_LYRICS_FONT_DELTA + 8.0)
            .max(row_height)
    } else {
        0.0
    };
    let music_panel_height = MUSIC_PANEL_PADDING_Y * 2.0
        + row_height
        + if music_lyrics_line.is_some() {
            music_lyrics_row_height + MUSIC_PANEL_GAP
        } else {
            0.0
        };
    let available_width = ui.available_width();
    let available_height = ui.available_height();
    let url_row_metrics = UrlRowMetrics::new(ui, state, row_height);
    let output_row_metrics = OutputRowMetrics::new(ui, state, row_height);
    let content_output_gap = if state.app_mode() == AppMode::Origin {
        (MAIN_SECTION_GAP - SINGLE_CONTENT_OUTPUT_GAP_REDUCTION).max(0.0)
    } else {
        MAIN_SECTION_GAP
    };

    tui(ui, ui.id().with("main-tab-vertical-layout"))
        .reserve_width(available_width)
        .reserve_height(available_height)
        .style(taffy::Style {
            display: taffy::Display::Flex,
            flex_direction: taffy::FlexDirection::Column,
            size: taffy::Size {
                width: percent(1.0),
                height: length(available_height),
            },
            min_size: taffy::Size {
                width: percent(1.0),
                height: length(0.0),
            },
            padding: length(0.0),
            margin: length(0.0),
            gap: length(0.0),
            ..Default::default()
        })
        .show(|tui| {
            tui.style(main_url_row_style(&url_row_metrics)).add(|tui| {
                row_url_input(tui, state, &url_row_metrics);
            });
            tui.style(main_spacer_style(MAIN_SECTION_GAP)).ui(|_| {});

            if state.app_mode() == AppMode::Origin {
                tui.style(main_flex_content_style()).add(|tui| {
                    build_single_mode_item(tui, state, row_height);
                });
            } else {
                tui.style(main_flex_content_style()).ui(|ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height());
                    render_batch_list(ui, state);
                });
            }
            tui.style(main_spacer_style(content_output_gap)).ui(|_| {});

            if show_music_player {
                tui.style(main_fixed_row_style(music_panel_height))
                    .ui(|ui| {
                        row_music_player_panel(
                            ui,
                            state,
                            music_lyrics_line.as_ref(),
                            music_lyrics_row_height,
                            row_height,
                        );
                    });
                tui.style(main_spacer_style(MAIN_SECTION_GAP)).ui(|_| {});
            }

            tui.style(main_fixed_row_style(output_row_metrics.row_height))
                .add(|tui| {
                    row_output_and_download(tui, state, &output_row_metrics);
                });
            if MAIN_BOTTOM_PADDING > 0.0 {
                tui.style(main_spacer_style(MAIN_BOTTOM_PADDING)).ui(|_| {});
            }
        });
}

fn main_fixed_row_style(height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        min_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn main_spacer_style(height: f32) -> taffy::Style {
    main_fixed_row_style(height)
}

fn main_flex_content_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0),
            height: length(0.0),
        },
        min_size: taffy::Size {
            width: percent(1.0),
            height: length(0.0),
        },
        flex_basis: length(0.0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

struct UrlRowMetrics {
    is_single_mode: bool,
    show_spinner: bool,
    analysis_running: bool,
    url_input_locked: bool,
    row_height: f32,
    control_height: f32,
    spinner_size: f32,
    spinner_gap: f32,
    clipboard_width: f32,
    action_width: f32,
    gap: f32,
}

impl UrlRowMetrics {
    fn new(ui: &Ui, state: &AppState, _base_control_height: f32) -> Self {
        let is_single_mode = state.app_mode() == AppMode::Origin;
        let analysis_running = state.single_mode_analysis_running();
        let show_spinner = state.is_adding_batch && !state.is_cancelling_batch_add;
        let url_input_locked = state.url_input_locked();
        let control_height = app_textbox_single_line_height(ui);
        let spinner_size = control_height * 0.75;
        let spinner_gap = 4.0;
        let action_width = if primary_url_action_uses_icon(state) {
            natural_icon_button_width(ui, state.primary_url_action_label())
        } else {
            natural_button_width(ui, state.primary_url_action_label())
        } + if show_spinner {
            spinner_size + spinner_gap
        } else {
            0.0
        };
        let clipboard_width = control_height;
        let gap = main_inline_control_gap(ui);

        Self {
            is_single_mode,
            show_spinner,
            analysis_running,
            url_input_locked,
            row_height: control_height,
            control_height,
            spinner_size,
            spinner_gap,
            clipboard_width,
            action_width,
            gap,
        }
    }
}

struct OutputRowMetrics {
    row_height: f32,
    target_button_width: f32,
    download_width: f32,
    gap: f32,
}

impl OutputRowMetrics {
    fn new(ui: &Ui, state: &AppState, _base_control_height: f32) -> Self {
        let row_height = app_textbox_single_line_height(ui);

        Self {
            row_height,
            target_button_width: natural_icon_button_width(ui, state.tr(UiText::TARGET_DIR)),
            download_width: natural_icon_button_width(ui, state.tr(UiText::DOWNLOAD)),
            gap: main_inline_control_gap(ui),
        }
    }
}

fn main_inline_control_gap(ui: &Ui) -> f32 {
    ui.spacing().item_spacing.x * MAIN_INLINE_CONTROL_GAP_SCALE
}

fn main_url_row_style(metrics: &UrlRowMetrics) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0),
            height: length(metrics.row_height),
        },
        min_size: taffy::Size {
            width: percent(1.0),
            height: length(metrics.row_height),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(metrics.row_height),
        },
        gap: length(metrics.gap),
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn main_url_flex_cell_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(0.0),
            height: percent(1.0),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        flex_basis: length(0.0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn main_url_fixed_cell_style(width: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(width),
            height: percent(1.0),
        },
        min_size: taffy::Size {
            width: length(width),
            height: length(0.0),
        },
        max_size: taffy::Size {
            width: length(width),
            height: percent(1.0),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn row_url_input(tui: &mut Tui, state: &mut AppState, metrics: &UrlRowMetrics) {
    let url_hint = state.tr(UiText::URL_HINT).to_owned();
    let language = state.language();
    tui.style(main_url_flex_cell_style()).ui(|ui| {
        let response = AppTextBox::new(&mut state.url_input)
            .hint_text(&url_hint)
            .language(language)
            .enabled(!metrics.url_input_locked)
            .syntax(AppTextBoxSyntax::Url)
            .desired_width(ui.available_width())
            .min_rows(1)
            .max_rows(Some(1))
            .allow_newline(false)
            .ctrl_click_links(false)
            .ui(ui);
        let submit_requested = metrics.is_single_mode
            && response.has_focus()
            && ui.input(|input| input.key_pressed(egui::Key::Enter));
        if !state.url_input.trim().is_empty() {
            response.on_hover_text(state.url_input.as_str());
        }
        if submit_requested && !metrics.url_input_locked {
            state.run_primary_url_action();
        }
    });

    tui.style(main_url_fixed_cell_style(metrics.clipboard_width))
        .ui(|ui| {
            let response = ui.add_sized(
                [ui.available_width(), metrics.control_height],
                clipboard_monitor_button(ui, state),
            );
            let hover_text = clipboard_monitor_hover_text(state);
            response.clone().on_hover_text(hover_text);
            if response.clicked() {
                state.set_monitor_clipboard(!state.clipboard_monitor_enabled());
            }
        });

    tui.style(main_url_fixed_cell_style(metrics.action_width))
        .ui(|ui| {
            if metrics.analysis_running {
                render_single_analysis_spinner_button(ui, state, metrics);
                return;
            }

            if metrics.show_spinner {
                render_url_spinner_action_cell(ui, state, metrics);
                return;
            }

            if state.is_adding_batch && state.is_cancelling_batch_add {
                let button = primary_url_action_button(ui, state);
                ui.add_enabled(
                    false,
                    button
                        .min_size(egui::vec2(ui.available_width(), metrics.control_height))
                        .wrap_mode(egui::TextWrapMode::Extend),
                );
                return;
            }

            let missing_yt_dlp = state.required_dependency_notice().is_some();
            let button = primary_url_action_button_for_state(ui, state, missing_yt_dlp)
                .min_size(egui::vec2(ui.available_width(), metrics.control_height))
                .wrap_mode(egui::TextWrapMode::Extend);
            let response = if missing_yt_dlp {
                ui.add(button)
            } else {
                ui.add_enabled(!metrics.url_input_locked, button)
            };
            if missing_yt_dlp {
                show_missing_yt_dlp_callout(ui, response.rect, "url-action", state);
            } else if response.clicked() {
                state.run_primary_url_action();
            }
        });
}

fn render_url_spinner_action_cell(ui: &mut Ui, state: &mut AppState, metrics: &UrlRowMetrics) {
    let rect = ui.max_rect();
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        let original_spacing_x = ui.spacing().item_spacing.x;
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.horizontal(|ui| {
            ui.allocate_ui(
                egui::vec2(metrics.spinner_size + metrics.spinner_gap, rect.height()),
                |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.add(Spinner::new().size(metrics.spinner_size));
                    });
                },
            );
            let response = ui.add_sized(
                [ui.available_width(), rect.height()],
                primary_url_action_button(ui, state).wrap_mode(egui::TextWrapMode::Extend),
            );
            if response.clicked() {
                state.cancel_batch_add();
            }
        });
        ui.spacing_mut().item_spacing.x = original_spacing_x;
    });
}

fn render_single_analysis_spinner_button(
    ui: &mut Ui,
    state: &AppState,
    metrics: &UrlRowMetrics,
) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), metrics.control_height);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let (button_bg_fill, button_bg_stroke, button_fg_color) = {
        let visuals = &ui.visuals().widgets.inactive;
        (visuals.bg_fill, visuals.bg_stroke, visuals.fg_stroke.color)
    };
    ui.painter().rect(
        rect,
        2.0,
        button_bg_fill,
        button_bg_stroke,
        egui::StrokeKind::Outside,
    );

    let icon_size = icon_button_text_size(ui);
    let label = state.primary_url_action_label();
    let galley = WidgetText::from(RichText::new(label).size(icon_size)).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        TextStyle::Button,
    );
    let icon_spacing = ui.spacing().icon_spacing;
    let content_width = icon_size + icon_spacing + galley.size().x;
    let icon_left = rect.center().x - content_width * 0.5;
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(icon_left, rect.center().y - icon_size * 0.5),
        egui::vec2(icon_size, icon_size),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.add(Spinner::new().size(icon_size));
        });
    });
    let text_pos = egui::pos2(
        icon_rect.right() + icon_spacing,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, button_fg_color);
    response
}

fn clipboard_monitor_button(ui: &Ui, state: &AppState) -> egui::Button<'static> {
    let enabled = state.clipboard_monitor_enabled();
    let icon = if enabled {
        AppIcon::MonitorEye
    } else {
        AppIcon::MonitorOff
    };
    let size = ui.spacing().interact_size.y * 0.72;
    let icon_color = if enabled {
        Color32::WHITE
    } else {
        standard_icon_color(ui)
    };
    let mut button = egui::Button::image(icon_image(icon, size, icon_color));
    if enabled {
        button = button.fill(ui.visuals().selection.bg_fill);
    }
    button
}

fn clipboard_monitor_hover_text(state: &AppState) -> &'static str {
    if state.clipboard_monitor_enabled() {
        if state.config.clipboard_auto_add {
            state.tr("main.clipboard_monitor_on_the_next_youtube_url_ch")
        } else {
            state.tr("main.clipboard_monitor_on_the_next_youtube_url_ch_2")
        }
    } else {
        state.tr("main.clipboard_monitor_off_turning_it_on_only_mem")
    }
}

fn row_music_player_panel(
    ui: &mut Ui,
    state: &mut AppState,
    lyrics_line: Option<&MusicLyricsDisplayLine>,
    lyrics_row_height: f32,
    player_row_height: f32,
) {
    let panel_height = ui.available_height();
    let (panel_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), panel_height),
        egui::Sense::hover(),
    );

    ui.painter().rect(
        panel_rect,
        7.0,
        music_player_panel_fill(ui),
        ui.visuals().widgets.noninteractive.bg_stroke,
        egui::StrokeKind::Outside,
    );

    let content_rect = panel_rect.shrink2(egui::vec2(MUSIC_PANEL_PADDING_X, MUSIC_PANEL_PADDING_Y));
    let mut player_top = content_rect.top();

    if let Some(line) = lyrics_line {
        let lyrics_rect = egui::Rect::from_min_size(
            content_rect.min,
            egui::vec2(content_rect.width(), lyrics_row_height),
        );
        render_music_lyrics_at(ui, lyrics_rect, line);
        player_top = lyrics_rect.bottom() + MUSIC_PANEL_GAP;
    }

    let player_rect = egui::Rect::from_min_size(
        egui::pos2(content_rect.left(), player_top),
        egui::vec2(content_rect.width(), player_row_height),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(player_rect), |ui| {
        row_music_player(ui, state);
    });
}

fn render_music_lyrics_at(ui: &mut Ui, rect: egui::Rect, line: &MusicLyricsDisplayLine) {
    let fade = line.fade.clamp(0.0, 1.0);
    if fade < 1.0 {
        ui.ctx().request_repaint();
    }
    if let Some(previous) = line.previous.as_deref().filter(|_| fade < 1.0) {
        render_music_lyrics_text_at(ui, rect, previous, 1.0 - fade);
    }
    render_music_lyrics_text_at(ui, rect, &line.current, fade.max(0.001));
}

fn render_music_lyrics_text_at(ui: &mut Ui, rect: egui::Rect, line: &str, alpha: f32) {
    let font_size = egui::TextStyle::Body.resolve(ui.style()).size + MUSIC_LYRICS_FONT_DELTA;
    let text = RichText::new(line).size(font_size);
    let galley = egui::WidgetText::from(text).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        rect.width(),
        egui::TextStyle::Body,
    );
    let pos = egui::pos2(
        rect.center().x - galley.size().x * 0.5,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(
        pos,
        galley,
        color_with_alpha(ui.visuals().text_color(), alpha),
    );
}

fn color_with_alpha(color: Color32, alpha: f32) -> Color32 {
    let alpha = (f32::from(color.a()) * alpha.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn music_player_panel_fill(ui: &Ui) -> Color32 {
    let base = if ui.visuals().dark_mode {
        ui.visuals().extreme_bg_color
    } else {
        ui.visuals().panel_fill
    };
    let alpha = if ui.visuals().dark_mode { 196 } else { 224 };
    Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha)
}

fn row_music_player(ui: &mut Ui, state: &mut AppState) {
    let row_height = ui.spacing().interact_size.y;
    let spacing = ui.spacing().item_spacing.x.max(8.0);
    let button_width = row_height;
    let time_width = 108.0;
    let volume_width = 128.0;
    let mut volume = state.music_volume();

    ui.set_width(ui.available_width());
    let (row_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::hover(),
    );

    let previous_rect = egui::Rect::from_min_size(
        egui::pos2(row_rect.left(), row_rect.top()),
        egui::vec2(button_width, row_height),
    );
    let play_rect = egui::Rect::from_min_size(
        egui::pos2(previous_rect.right() + spacing, row_rect.top()),
        egui::vec2(button_width, row_height),
    );
    let next_rect = egui::Rect::from_min_size(
        egui::pos2(play_rect.right() + spacing, row_rect.top()),
        egui::vec2(button_width, row_height),
    );

    // Build the player as fixed left/right groups with a single flexible middle.
    // The seek and volume controls both stay native egui Sliders; only their
    // scoped slider_width is changed so their visual style remains identical.
    let volume_rect = egui::Rect::from_min_max(
        egui::pos2(
            (row_rect.right() - volume_width).max(row_rect.left()),
            row_rect.top(),
        ),
        egui::pos2(row_rect.right(), row_rect.bottom()),
    );
    let mode_rect = egui::Rect::from_min_size(
        egui::pos2(
            (volume_rect.left() - spacing - button_width).max(row_rect.left()),
            row_rect.top(),
        ),
        egui::vec2(button_width, row_height),
    );
    let time_rect = egui::Rect::from_min_size(
        egui::pos2(
            (mode_rect.left() - spacing - time_width).max(row_rect.left()),
            row_rect.top(),
        ),
        egui::vec2(time_width, row_height),
    );
    let seek_left = next_rect.right() + spacing;
    let seek_right = time_rect.left() - spacing;
    let seek_rect = egui::Rect::from_min_max(
        egui::pos2(seek_left, row_rect.top()),
        egui::pos2(seek_right.max(seek_left), row_rect.bottom()),
    );

    if music_icon_button_at(
        ui,
        previous_rect,
        AppIcon::SkipPrevious,
        state.tr("music.previous"),
    )
    .clicked()
    {
        state.previous_music_item();
    }
    let (icon, label) = if state.music_player_is_playing() {
        (AppIcon::Pause, state.tr("music.pause"))
    } else {
        (AppIcon::Play, state.tr("music.play"))
    };
    if music_icon_button_at(ui, play_rect, icon, label).clicked() {
        state.toggle_music_playback();
    }
    if music_icon_button_at(ui, next_rect, AppIcon::SkipNext, state.tr("music.next")).clicked() {
        state.next_music_item();
    }

    if seek_rect.width() > 1.0 {
        if let Some(error) = state.music_player_error_text() {
            render_music_player_error_at(ui, seek_rect, error);
        } else {
            render_music_seek_bar_at(ui, state, seek_rect);
        }
    }

    render_music_time_at(ui, time_rect, &state.music_playback_time_text());

    let icon = music_playback_mode_icon(state.music_playback_mode_kind());
    if music_icon_button_at(ui, mode_rect, icon, state.music_playback_mode_tooltip()).clicked() {
        state.cycle_music_playback_mode();
    }

    let icon_size = row_height;
    let volume_icon_rect = egui::Rect::from_min_size(
        egui::pos2(volume_rect.left(), volume_rect.top()),
        egui::vec2(icon_size, row_height),
    );
    music_round_icon_at(ui, volume_icon_rect, AppIcon::VolumeHigh);
    let slider_left = volume_icon_rect.right() + ui.spacing().icon_spacing;
    let volume_slider_rect = egui::Rect::from_min_max(
        egui::pos2(slider_left, volume_rect.top()),
        egui::pos2(volume_rect.right(), volume_rect.bottom()),
    );
    let slider = egui::Slider::new(&mut volume, 0.0..=1.0).show_value(false);
    let volume_response = ui
        .scope_builder(egui::UiBuilder::new().max_rect(volume_slider_rect), |ui| {
            ui.spacing_mut().slider_width = volume_slider_rect.width().max(1.0);
            ui.add_sized(volume_slider_rect.size(), slider)
        })
        .inner;
    if volume_response.changed() {
        state.set_music_volume(volume);
    }
}

fn render_music_time_at(ui: &Ui, rect: egui::Rect, text: &str) {
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::TextStyle::Body.resolve(ui.style()),
        ui.visuals().text_color(),
    );
}

fn render_music_player_error_at(ui: &Ui, rect: egui::Rect, error: &str) {
    let color = accent_red_for_ui(ui);
    let galley = egui::WidgetText::from(RichText::new(error).color(color)).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        rect.width(),
        egui::TextStyle::Body,
    );
    let pos = egui::pos2(rect.left(), rect.center().y - galley.size().y * 0.5);
    ui.painter().galley(pos, galley, color);
}

fn music_icon_button_at(
    ui: &mut Ui,
    rect: egui::Rect,
    icon: AppIcon,
    tooltip: &str,
) -> egui::Response {
    let id = ui.make_persistent_id(("music-icon-button", tooltip));
    let response = ui
        .interact(rect, id, egui::Sense::click())
        .on_hover_text(tooltip);
    let visuals = ui.style().interact(&response);
    let radius = (rect.width().min(rect.height()) * 0.5 - 1.0).max(1.0);
    ui.painter()
        .circle_filled(rect.center(), radius, visuals.bg_fill);
    ui.painter()
        .circle_stroke(rect.center(), radius, visuals.bg_stroke);

    let icon_size = icon_button_text_size(ui) * 0.92;
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(icon, icon_size, visuals.fg_stroke.color).paint_at(ui, icon_rect);
    response
}

fn music_round_icon_at(ui: &Ui, rect: egui::Rect, icon: AppIcon) {
    let radius = (rect.width().min(rect.height()) * 0.5 - 1.0).max(1.0);
    let fill = ui.visuals().faint_bg_color;
    let stroke = ui.visuals().widgets.inactive.bg_stroke;
    ui.painter().circle_filled(rect.center(), radius, fill);
    ui.painter().circle_stroke(rect.center(), radius, stroke);

    let icon_size = icon_button_text_size(ui) * 0.86;
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(icon, icon_size, standard_icon_color(ui)).paint_at(ui, icon_rect);
}

fn music_playback_mode_icon(mode: MusicPlaybackMode) -> AppIcon {
    match mode {
        MusicPlaybackMode::Sequential => AppIcon::ArrowRight,
        MusicPlaybackMode::RepeatAll => AppIcon::Repeat,
        MusicPlaybackMode::Shuffle => AppIcon::Shuffle,
        MusicPlaybackMode::RepeatOne => AppIcon::RepeatOnce,
    }
}

fn render_music_seek_bar_at(ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
    let rect = rect.shrink2(egui::vec2(2.0, 0.0));
    if rect.width() <= 1.0 {
        return;
    }

    let mut value = state.music_seek_display_ratio().clamp(0.0, 1.0);
    let slider = egui::Slider::new(&mut value, 0.0..=1.0).show_value(false);
    let response = ui
        .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.spacing_mut().slider_width = rect.width().max(1.0);
            ui.add_sized(rect.size(), slider)
        })
        .inner;

    if response.changed() || response.dragged() {
        state.set_music_seek_drag_ratio(Some(value));
        ui.ctx().request_repaint();
    }

    let pointer_down = ui.input(|input| input.pointer.primary_down());
    if state.music_seek_drag_ratio().is_some() && !pointer_down {
        let final_ratio = state
            .music_seek_drag_ratio()
            .unwrap_or(value)
            .clamp(0.0, 1.0);
        state.finish_music_seek_drag(final_ratio);
    }

    let hover = if state.music_playback_cache_progress_ratio() < 0.999 {
        state.tr("music.seek_cached_range_hint")
    } else {
        state.tr("music.seek_hint")
    };
    response.on_hover_text(hover);
}

fn row_output_and_download(tui: &mut Tui, state: &mut AppState, metrics: &OutputRowMetrics) {
    let row_height = metrics.row_height;
    let download_width = metrics.download_width;
    let target_button_width = metrics.target_button_width;
    let mut output_dir_display = state.output_dir_display();
    let output_locked_by_config = state.output_dir_locked_by_config();
    let output_config_source = state.output_dir_config_source_display();
    let has_pending_downloads = state.has_pending_download_items();

    tui.style(main_output_row_style(row_height, metrics.gap))
        .add(|tui| {
            tui.style(main_url_fixed_cell_style(target_button_width))
                .ui(|ui| {
                    let response = ui.add_enabled(
                        !output_locked_by_config,
                        icon_text_button(
                            ui,
                            AppIcon::FolderMoveOutline,
                            state.tr(UiText::TARGET_DIR),
                        )
                        .min_size(egui::vec2(ui.available_width(), row_height)),
                    );
                    if response.clicked() {
                        let mut dialog = rfd::FileDialog::new();
                        if let Ok(current_dir) = crate::infrastructure::resolve_output_dir(
                            &state.item_defaults.output_dir,
                        ) {
                            dialog = dialog.set_directory(current_dir);
                        }
                        if let Some(folder) = dialog.pick_folder() {
                            state.set_output_dir(folder.display().to_string());
                        }
                    }
                    if output_locked_by_config {
                        response.on_hover_text(
                            output_config_source
                                .as_deref()
                                .map(|path| {
                                    format!("{}{}", state.tr("main.controlled_by_config_2"), path)
                                })
                                .unwrap_or_else(|| {
                                    state.tr("main.controlled_by_config").to_owned()
                                }),
                        );
                    }
                });

            tui.style(main_url_flex_cell_style()).ui(|ui| {
                AppTextBox::new(&mut output_dir_display)
                    .editable(false)
                    .selectable(true)
                    .syntax(AppTextBoxSyntax::Path)
                    .desired_width(ui.available_width())
                    .min_rows(1)
                    .max_rows(Some(1))
                    .allow_newline(false)
                    .ctrl_click_links(false)
                    .ui(ui);
            });

            tui.style(main_url_fixed_cell_style(download_width))
                .ui(|ui| {
                    let missing_yt_dlp =
                        has_pending_downloads && state.required_dependency_notice().is_some();
                    let button = download_button_for_state(ui, state, missing_yt_dlp)
                        .min_size(egui::vec2(ui.available_width(), row_height));
                    let response = if has_pending_downloads {
                        ui.add(button)
                    } else {
                        ui.add_enabled(false, button)
                    };
                    if missing_yt_dlp {
                        // The missing-tool notice is always shown near the URL action button.
                    }
                    if response.clicked() && has_pending_downloads && !missing_yt_dlp {
                        state.request_main_download();
                    }
                });
        });
}

fn main_output_row_style(height: f32, gap: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        min_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        gap: length(gap),
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn missing_tool_button_fill(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(96, 24, 24)
    } else {
        Color32::from_rgb(255, 214, 214)
    }
}

fn missing_tool_button_stroke() -> egui::Stroke {
    egui::Stroke::new(1.0, Color32::from_rgb(220, 72, 72))
}

fn missing_tool_button_text_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(255, 225, 225)
    } else {
        Color32::from_rgb(190, 0, 28)
    }
}

fn missing_tool_callout_fill(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(42, 16, 16)
    } else {
        Color32::from_rgb(255, 226, 226)
    }
}

fn missing_tool_callout_stroke() -> egui::Stroke {
    egui::Stroke::new(1.0, Color32::from_rgb(235, 88, 88))
}

fn show_missing_yt_dlp_callout(
    ui: &Ui,
    anchor: egui::Rect,
    id_source: &'static str,
    state: &AppState,
) {
    let x = (anchor.right() - MISSING_YT_DLP_CALLOUT_WIDTH).max(8.0);
    let pos = egui::pos2(x, anchor.bottom() + 6.0);

    egui::Area::new(egui::Id::new(("missing-ytdlp-callout", id_source)))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .fill(missing_tool_callout_fill(ui))
                .stroke(missing_tool_callout_stroke())
                .show(ui, |ui| {
                    ui.set_max_width(MISSING_YT_DLP_CALLOUT_WIDTH);
                    ui.label(
                        RichText::new(state.tr(MISSING_YT_DLP_TOOLTIP_KEY))
                            .color(missing_tool_button_text_color(ui)),
                    );
                });
        });
}

fn primary_url_action_icon(state: &AppState) -> AppIcon {
    if state.app_mode() == AppMode::Origin {
        AppIcon::Magnify
    } else {
        AppIcon::Download
    }
}

fn primary_url_action_uses_icon(state: &AppState) -> bool {
    state.app_mode() == AppMode::Origin
        || (state.config.direct_download_on_add && !state.queue_display_mode_is_audio())
}

fn missing_tool_icon_text_button(ui: &Ui, icon: AppIcon, label: &str) -> egui::Button<'static> {
    let size = icon_button_text_size(ui);
    egui::Button::image_and_text(
        icon_image(icon, size, missing_tool_button_text_color(ui)),
        RichText::new(label)
            .size(size)
            .color(missing_tool_button_text_color(ui)),
    )
    .fill(missing_tool_button_fill(ui))
    .stroke(missing_tool_button_stroke())
}

fn primary_url_action_button_for_state(
    ui: &Ui,
    state: &AppState,
    muted: bool,
) -> egui::Button<'static> {
    if primary_url_action_uses_icon(state) {
        if muted {
            missing_tool_icon_text_button(
                ui,
                primary_url_action_icon(state),
                state.primary_url_action_label(),
            )
        } else {
            icon_text_button(
                ui,
                primary_url_action_icon(state),
                state.primary_url_action_label(),
            )
        }
    } else if muted {
        egui::Button::new(
            RichText::new(state.primary_url_action_label())
                .color(missing_tool_button_text_color(ui)),
        )
        .fill(missing_tool_button_fill(ui))
        .stroke(missing_tool_button_stroke())
    } else {
        egui::Button::new(state.primary_url_action_label())
    }
}

fn primary_url_action_button(ui: &Ui, state: &AppState) -> egui::Button<'static> {
    primary_url_action_button_for_state(ui, state, false)
}

fn download_button_for_state(ui: &Ui, state: &AppState, muted: bool) -> egui::Button<'static> {
    if muted {
        missing_tool_icon_text_button(ui, AppIcon::Download, state.tr(UiText::DOWNLOAD))
    } else {
        icon_text_button(ui, AppIcon::Download, state.tr(UiText::DOWNLOAD))
    }
}
