use eframe::egui;
use egui_taffy::Tui;

use crate::app::state::{AdvanceDetailPage, AppState, CookieFileSourceMode, CookieUsageMode};
use crate::app::widgets::icon::AppIcon;
use crate::app::widgets::url_input::{AppTextBox, AppTextBoxSyntax};

use super::common::{settings_taffy_form_row, settings_taffy_section, text_trailing_icon_button};
use super::semantic_ui_metrics;

pub(super) fn render_network_section(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(
        tui,
        state.ui_i18n_text_for_key("advance.network_access"),
        |tui| {
            render_proxy_row(tui, state, label_width);
            render_certificate_row(tui, state, label_width);
            render_cookie_usage_row(tui, state, label_width);

            match state.cookie_usage_mode() {
                CookieUsageMode::Off => {}
                CookieUsageMode::Browser => {
                    render_browser_cookie_profile_row(tui, state, label_width);
                }
                CookieUsageMode::File => {
                    render_cookie_file_source_row(tui, state, label_width);
                    match state.cookie_file_source_mode() {
                        CookieFileSourceMode::Custom => {
                            render_cookie_file_row(tui, state, label_width);
                        }
                        CookieFileSourceMode::AutoSelect => {
                            render_cookie_manager_row(tui, state, label_width);
                        }
                    }
                }
            }
        },
    );
}

fn render_proxy_row(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_form_row(
        tui,
        label_width,
        state.ui_i18n_text_for_key("advance.proxy"),
        |ui| {
            let mut proxy_enabled = state.config.proxy_enabled;
            if ui
                .checkbox(
                    &mut proxy_enabled,
                    state.ui_i18n_text_for_key("advance.enable_proxy"),
                )
                .changed()
            {
                state.set_proxy_enabled(proxy_enabled);
            }
            let mut proxy_url = state.tool_paths.proxy_url.clone();
            let response = AppTextBox::new(&mut proxy_url)
                .hint_text("protocol://ip:port")
                .language(state.language())
                .syntax(AppTextBoxSyntax::Url)
                .desired_width(semantic_ui_metrics::advance_form_standard_text_field_width())
                .ui(ui);
            if response.changed() {
                state.set_proxy_url(proxy_url);
            }
        },
    );
}

fn render_certificate_row(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_form_row(
        tui,
        label_width,
        state.ui_i18n_text_for_key("advance.certificate"),
        |ui| {
            let mut no_check_certificates = state.tool_paths.no_check_certificates;
            if ui
                .checkbox(
                    &mut no_check_certificates,
                    state.ui_i18n_text_for_key("advance.skip_certificate_verification"),
                )
                .changed()
            {
                state.set_no_check_certificates(no_check_certificates);
            }
        },
    );
}

fn render_cookie_usage_row(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_form_row(
        tui,
        label_width,
        state.ui_i18n_text_for_key("advance.cookie"),
        |ui| {
            ui.horizontal_wrapped(|ui| {
                cookie_usage_button(ui, state, CookieUsageMode::Off, "advance.cookie.off");
                cookie_usage_button(
                    ui,
                    state,
                    CookieUsageMode::Browser,
                    "advance.cookie.browser",
                );
                cookie_usage_button(ui, state, CookieUsageMode::File, "advance.cookie.file");
            });
        },
    );
}

fn cookie_usage_button(
    ui: &mut egui::Ui,
    state: &mut AppState,
    mode: CookieUsageMode,
    label_key: &'static str,
) {
    if ui
        .selectable_label(
            state.cookie_usage_mode() == mode,
            state.ui_i18n_text_for_key(label_key),
        )
        .clicked()
    {
        state.set_cookie_usage_mode(mode);
    }
}

fn render_cookie_file_source_row(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_form_row(
        tui,
        label_width,
        state.ui_i18n_text_for_key("advance.cookie_file_source"),
        |ui| {
            ui.horizontal_wrapped(|ui| {
                cookie_file_source_button(
                    ui,
                    state,
                    CookieFileSourceMode::Custom,
                    "advance.cookie_file_custom",
                );
                cookie_file_source_button(
                    ui,
                    state,
                    CookieFileSourceMode::AutoSelect,
                    "advance.cookie_file_auto_select",
                );
            });
        },
    );
}

fn cookie_file_source_button(
    ui: &mut egui::Ui,
    state: &mut AppState,
    mode: CookieFileSourceMode,
    label_key: &'static str,
) {
    if ui
        .selectable_label(
            state.cookie_file_source_mode() == mode,
            state.ui_i18n_text_for_key(label_key),
        )
        .clicked()
    {
        state.set_cookie_file_source_mode(mode);
    }
}

fn render_cookie_manager_row(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_form_row(
        tui,
        label_width,
        state.ui_i18n_text_for_key("advance.cookie_manager_row"),
        |ui| {
            if ui
                .add(text_trailing_icon_button(
                    ui,
                    state.ui_i18n_text_for_key("advance.manage_cookie"),
                    AppIcon::MenuRight,
                ))
                .clicked()
            {
                state.open_advance_detail_page(AdvanceDetailPage::CookieManager);
            }
        },
    );
}

fn render_cookie_file_row(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_form_row(
        tui,
        label_width,
        state.ui_i18n_text_for_key("advance.file"),
        |ui| {
            let mut cookie_file_display = if state.tool_paths.browser_cookie_file.trim().is_empty()
            {
                state
                    .ui_i18n_text_for_key("advance.no_cookies_txt_selected")
                    .to_owned()
            } else {
                state.tool_paths.browser_cookie_file.clone()
            };
            ui.horizontal(|ui| {
                AppTextBox::new(&mut cookie_file_display)
                    .language(state.language())
                    .syntax(AppTextBoxSyntax::Path)
                    .desired_width(semantic_ui_metrics::advance_form_standard_text_field_width())
                    .editable(false)
                    .selectable(false)
                    .enabled(false)
                    .ui(ui);
                if ui
                    .button(state.ui_i18n_text_for_key("advance.browse"))
                    .clicked()
                {
                    let mut dialog = rfd::FileDialog::new()
                        .add_filter(
                            state.ui_i18n_text_for_key("advance.filter_netscape_cookies_txt"),
                            &["txt"],
                        )
                        .add_filter(
                            state.ui_i18n_text_for_key("advance.filter_all_files"),
                            &["*"],
                        )
                        .set_title(
                            state.ui_i18n_text_for_key("advance.select_netscape_cookies_txt"),
                        );
                    if !state.tool_paths.browser_cookie_file.trim().is_empty() {
                        let current_path =
                            std::path::PathBuf::from(&state.tool_paths.browser_cookie_file);
                        if let Some(parent) = current_path.parent().filter(|path| path.is_dir()) {
                            dialog = dialog.set_directory(parent);
                        }
                    }
                    if let Some(path) = dialog.pick_file() {
                        state.set_browser_cookie_file(path.display().to_string());
                    }
                }
                if ui
                    .button(state.ui_i18n_text_for_key("advance.clear"))
                    .clicked()
                {
                    state.set_browser_cookie_file(String::new());
                }
            });
        },
    );
}

fn render_browser_cookie_profile_row(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_form_row(
        tui,
        label_width,
        state.ui_i18n_text_for_key("advance.browser"),
        |ui| {
            let cookie_sources = state
                .available_browser_cookie_sources()
                .into_iter()
                .filter(|option| option.value != "auto" && option.value != "file")
                .collect::<Vec<_>>();
            let selected_source_label = cookie_sources
                .iter()
                .find(|option| option.value == state.tool_paths.browser_cookie_source)
                .map(|option| option.label.to_owned())
                .unwrap_or_else(|| state.tool_paths.browser_cookie_source.clone());

            let cookie_profiles = state.available_browser_cookie_profiles();
            let selected_profile_label =
                if state.tool_paths.browser_cookie_profile.trim().is_empty() {
                    state.ui_i18n_text_for_key("advance.default").to_owned()
                } else {
                    cookie_profiles
                        .iter()
                        .find(|option| option.value == state.tool_paths.browser_cookie_profile)
                        .map(|option| option.label.clone())
                        .unwrap_or_else(|| state.tool_paths.browser_cookie_profile.clone())
                };

            ui.horizontal_wrapped(|ui| {
                egui::ComboBox::from_id_salt("browser-cookie-source")
                    .selected_text(selected_source_label)
                    .show_ui(ui, |ui| {
                        for option in cookie_sources {
                            let selected = state.tool_paths.browser_cookie_source == option.value;
                            if ui.selectable_label(selected, option.label).clicked() {
                                state.set_browser_cookie_source(option.value);
                            }
                        }
                    })
                    .response;
                ui.label(state.ui_i18n_text_for_key("advance.config"));
                egui::ComboBox::from_id_salt("browser-cookie-profile")
                    .selected_text(selected_profile_label)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                state.tool_paths.browser_cookie_profile.trim().is_empty(),
                                state.ui_i18n_text_for_key("advance.default"),
                            )
                            .clicked()
                        {
                            state.set_browser_cookie_profile(String::new());
                        }
                        for option in cookie_profiles {
                            if ui
                                .selectable_label(
                                    state.tool_paths.browser_cookie_profile == option.value,
                                    option.label.as_str(),
                                )
                                .clicked()
                            {
                                state.set_browser_cookie_profile(option.value);
                            }
                        }
                    })
                    .response;
            });
        },
    );
}
