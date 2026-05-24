use eframe::egui::{self, Color32, RichText, Spinner, Ui};
use egui_extras::{Size, StripBuilder};

use crate::app::state::AppState;
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::app::widgets::url_input::{DisplayPathInput, UrlInput};

use super::common::{
    UiText, icon_button_text_size, icon_text_button, natural_button_width,
    natural_icon_button_width,
};
use super::item_card::render_batch_list;

const MAIN_SECTION_GAP: f32 = 6.0;
const MISSING_YT_DLP_TOOLTIP_KEY: &str = "main.tooltip.missing_yt_dlp";
const MISSING_YT_DLP_CALLOUT_WIDTH: f32 = 320.0;

pub(super) fn render_main_tab(ui: &mut Ui, state: &mut AppState) {
    let row_height = ui.spacing().interact_size.y;

    StripBuilder::new(ui)
        .size(Size::exact(row_height))
        .size(Size::exact(MAIN_SECTION_GAP))
        .size(Size::remainder().at_least(0.0))
        .size(Size::exact(MAIN_SECTION_GAP))
        .size(Size::exact(row_height))
        .vertical(|mut strip| {
            strip.cell(|ui| {
                row_url_input(ui, state);
            });
            strip.empty();

            strip.cell(|ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                render_batch_list(ui, state);
            });
            strip.empty();
            strip.cell(|ui| {
                row_output_and_download(ui, state);
            });
        });
}

fn row_url_input(ui: &mut Ui, state: &mut AppState) {
    let row_height = ui.spacing().interact_size.y;
    let show_spinner = state.is_adding_batch && !state.is_cancelling_batch_add;
    let url_input_locked = state.url_input_locked();
    let spinner_size = row_height * 0.75;
    let spinner_gap = 4.0;
    let clipboard_toggle_width = row_height;
    let action_width = if state.config.direct_download_on_add {
        natural_icon_button_width(ui, state.primary_url_action_label())
    } else {
        natural_button_width(ui, state.primary_url_action_label())
    } + if show_spinner {
        spinner_size + spinner_gap
    } else {
        0.0
    };
    let row_width = ui.available_width();

    let url_hint = state.tr(UiText::URL_HINT);
    let language = state.language();

    ui.allocate_ui(egui::vec2(row_width, row_height), |ui| {
        StripBuilder::new(ui)
            .size(Size::remainder())
            .size(Size::exact(clipboard_toggle_width))
            .size(Size::exact(action_width))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    let response = ui.add_sized(
                        [ui.available_width(), row_height],
                        UrlInput::new(&mut state.url_input)
                            .hint_text(url_hint)
                            .language(language)
                            .enabled(!url_input_locked),
                    );
                    if !state.url_input.trim().is_empty() {
                        response.on_hover_text(state.url_input.as_str());
                    }
                });

                strip.cell(|ui| {
                    let response = ui.add_sized(
                        [ui.available_width(), row_height],
                        clipboard_monitor_button(ui, state),
                    );
                    let hover_text = clipboard_monitor_hover_text(state);
                    response.clone().on_hover_text(hover_text);
                    if response.clicked() {
                        state.set_monitor_clipboard(!state.clipboard_monitor_enabled());
                    }
                });

                strip.cell(|ui| {
                    if show_spinner {
                        let original_spacing_x = ui.spacing().item_spacing.x;
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.horizontal(|ui| {
                            ui.allocate_ui(
                                egui::vec2(spinner_size + spinner_gap, row_height),
                                |ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.add(Spinner::new().size(spinner_size));
                                    });
                                },
                            );
                            let response = ui.add_sized(
                                [ui.available_width(), row_height],
                                primary_url_action_button(ui, state),
                            );
                            if response.clicked() {
                                state.cancel_batch_add();
                            }
                        });
                        ui.spacing_mut().item_spacing.x = original_spacing_x;
                        return;
                    }

                    if state.is_adding_batch && state.is_cancelling_batch_add {
                        ui.add_enabled(
                            false,
                            primary_url_action_button(ui, state)
                                .min_size(egui::vec2(ui.available_width(), row_height)),
                        );
                        return;
                    }

                    let missing_yt_dlp = state.required_dependency_notice().is_some();
                    let button = primary_url_action_button_for_state(ui, state, missing_yt_dlp)
                        .min_size(egui::vec2(ui.available_width(), row_height));
                    let response = if missing_yt_dlp {
                        ui.add(button)
                    } else {
                        ui.add_enabled(!url_input_locked, button)
                    };
                    if missing_yt_dlp {
                        show_missing_yt_dlp_callout(ui, response.rect, "url-action", state);
                    } else if response.clicked() {
                        state.run_primary_url_action();
                    }
                });
            });
    });
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
    let mut button = egui::Button::image(icon_image(icon, size, icon_color)).small();
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

fn row_output_and_download(ui: &mut Ui, state: &mut AppState) {
    let row_height = ui.spacing().interact_size.y;
    let download_width = natural_icon_button_width(ui, state.tr(UiText::DOWNLOAD));
    let target_button_width = natural_icon_button_width(ui, state.tr(UiText::TARGET_DIR));
    let row_width = ui.available_width();
    let mut output_dir_display = state.output_dir_display();
    let output_locked_by_config = state.output_dir_locked_by_config();
    let output_config_source = state.output_dir_config_source_display();
    let has_pending_downloads = state.has_pending_download_items();

    ui.allocate_ui(egui::vec2(row_width, row_height), |ui| {
        StripBuilder::new(ui)
            .size(Size::exact(target_button_width))
            .size(Size::remainder())
            .size(Size::exact(download_width))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
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

                strip.cell(|ui| {
                    ui.add_sized(
                        [ui.available_width(), row_height],
                        DisplayPathInput::new(&mut output_dir_display),
                    );
                });

                strip.cell(|ui| {
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
                        state.start_single_download();
                    }
                });
            });
    });
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

fn primary_url_action_icon() -> AppIcon {
    AppIcon::Download
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
    if state.config.direct_download_on_add {
        if muted {
            missing_tool_icon_text_button(
                ui,
                primary_url_action_icon(),
                state.primary_url_action_label(),
            )
        } else {
            icon_text_button(
                ui,
                primary_url_action_icon(),
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
