use eframe::egui;
use egui_taffy::Tui;

use crate::app::state::AppState;
use crate::infrastructure::{ThemeAccentColor, ThemeMode};

use super::common::{settings_taffy_form_row, settings_taffy_section};

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

pub(super) fn render_window_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(
        tui,
        state.ui_i18n_text_for_key("options.appearance_window"),
        |tui| {
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.notifications"),
                |ui| {
                    let mut enabled = state.config.windows_toast_enabled;
                    if ui
                        .checkbox(&mut enabled, state.ui_i18n_text_for_key("options.enable"))
                        .changed()
                    {
                        state.set_windows_toast_enabled(enabled);
                    }
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.theme"),
                |ui| {
                    egui::ComboBox::from_id_salt("theme-mode")
                        .selected_text(
                            state.ui_i18n_text_for_key(theme_mode_label_key(
                                state.config.theme_mode,
                            )),
                        )
                        .show_ui(ui, |ui| {
                            for mode in ThemeMode::variants() {
                                if ui
                                    .selectable_label(
                                        state.config.theme_mode == mode,
                                        state.ui_i18n_text_for_key(theme_mode_label_key(mode)),
                                    )
                                    .clicked()
                                {
                                    state.set_theme_mode(mode);
                                }
                            }
                        });
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.theme_color"),
                |ui| {
                    egui::ComboBox::from_id_salt("theme-accent-color")
                        .selected_text(state.ui_i18n_text_for_key(theme_accent_color_label_key(
                            state.config.theme_accent_color,
                        )))
                        .show_ui(ui, |ui| {
                            for color in ThemeAccentColor::variants() {
                                if ui
                                    .selectable_label(
                                        state.config.theme_accent_color == color,
                                        state.ui_i18n_text_for_key(theme_accent_color_label_key(
                                            color,
                                        )),
                                    )
                                    .clicked()
                                {
                                    state.set_theme_accent_color(color);
                                }
                            }
                        });
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.ui_scale"),
                |ui| {
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
                        if ui
                            .button(state.ui_i18n_text_for_key("options.apply"))
                            .clicked()
                        {
                            state.apply_pending_ui_scale_percent();
                        }
                    });
                    ui.label(format!(
                        "{} {}%",
                        state.ui_i18n_text_for_key("options.current"),
                        state.config.ui_scale_percent
                    ));
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.always_on_top"),
                |ui| {
                    let mut enabled = state.config.keep_window_on_top;
                    if ui
                        .checkbox(&mut enabled, state.ui_i18n_text_for_key("options.enable"))
                        .changed()
                    {
                        state.set_keep_window_on_top(enabled);
                    }
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.window_position"),
                |ui| {
                    let mut enabled = state.config.remember_window_position;
                    if ui
                        .checkbox(&mut enabled, state.ui_i18n_text_for_key("options.remember"))
                        .changed()
                    {
                        state.set_remember_window_position(enabled);
                    }
                },
            );
            settings_taffy_form_row(
                tui,
                label_width,
                state.ui_i18n_text_for_key("options.window_size"),
                |ui| {
                    let mut enabled = state.config.remember_window_size;
                    if ui
                        .checkbox(&mut enabled, state.ui_i18n_text_for_key("options.remember"))
                        .changed()
                    {
                        state.set_remember_window_size(enabled);
                    }
                },
            );
        },
    );
}
