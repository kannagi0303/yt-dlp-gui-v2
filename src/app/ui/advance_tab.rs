use eframe::egui::{self, Align, Layout, RichText, ScrollArea, Ui};
use egui_taffy::Tui;

use crate::app::state::{AdvanceDetailPage, AppState};
use crate::infrastructure::{BrowserCookieSourceOption, FileTimeMode};

use crate::app::widgets::icon::AppIcon;
use crate::app::widgets::url_input::{AppTextBox, AppTextBoxSyntax};

use super::common::{
    measure_label_width, settings_scroll_content, settings_taffy_form_row,
    settings_taffy_scroll_content, settings_taffy_section, text_trailing_icon_button,
};

const ADVANCE_TEXT_WIDTH: f32 = 280.0;

pub(super) fn render_advance_tab(ui: &mut Ui, state: &mut AppState) {
    if matches!(
        state.advance_detail_page,
        Some(AdvanceDetailPage::Transcode)
    ) {
        render_download_conversion_detail_page(ui, state);
        return;
    }

    ScrollArea::vertical()
        .id_salt("advance-tab-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let label_width = advance_label_width(ui, state);
            settings_taffy_scroll_content(ui, "advance-root-settings-taffy", |tui| {
                render_config_source_section(tui, state, label_width);
                render_network_section(tui, state, label_width);
                render_post_processing_section(tui, state, label_width);
                render_download_processing_section(tui, state, label_width);
                render_aria2_section(tui, state, label_width);
            });
        });
}

fn render_download_conversion_detail_page(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical()
        .id_salt("advance-download-conversion-page-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            settings_scroll_content(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .button(format!("← {}", state.ui_tr("options.back")))
                        .clicked()
                    {
                        state.close_advance_detail_page();
                    }
                    ui.label(RichText::new(state.ui_tr("advance.download_conversion")).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(state.ui_tr("action.confirm")).clicked() {
                            state.close_advance_detail_page();
                        }
                    });
                });
                ui.add_space(10.0);
                // Keep this detail page free of a second enable switch.
                super::processing_tab::render_processing_settings_content(ui, state);
            });
        });
}

fn render_config_source_section(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("advance.source"), |tui| {
        settings_taffy_form_row(tui, label_width, state.ui_tr("advance.config"), |ui| {
            let config_files = state.available_yt_dlp_config_files();
            let selected_label = if state.tool_paths.yt_dlp_config.trim().is_empty() {
                state.ui_tr("advance.none").to_owned()
            } else {
                config_files
                    .iter()
                    .find(|option| option.path == state.tool_paths.yt_dlp_config)
                    .map(|option| option.name.clone())
                    .unwrap_or_else(|| state.tool_paths.yt_dlp_config.clone())
            };
            egui::ComboBox::from_id_salt("yt-dlp-config-file")
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            state.tool_paths.yt_dlp_config.trim().is_empty(),
                            state.ui_tr("advance.none"),
                        )
                        .clicked()
                    {
                        state.set_yt_dlp_config_path(String::new());
                    }
                    for option in config_files {
                        if ui
                            .selectable_label(
                                state.tool_paths.yt_dlp_config == option.path,
                                option.name.as_str(),
                            )
                            .clicked()
                        {
                            state.set_yt_dlp_config_path(option.path);
                        }
                    }
                })
                .response;
        });
    });
}

fn render_network_section(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("advance.network_access"), |tui| {
        settings_taffy_form_row(tui, label_width, state.ui_tr("advance.proxy"), |ui| {
            let mut proxy_enabled = state.config.proxy_enabled;
            if ui
                .checkbox(&mut proxy_enabled, state.ui_tr("advance.enable_proxy"))
                .changed()
            {
                state.set_proxy_enabled(proxy_enabled);
            }
            let mut proxy_url = state.tool_paths.proxy_url.clone();
            let response = AppTextBox::new(&mut proxy_url)
                .hint_text("protocol://ip:port")
                .language(state.language())
                .syntax(AppTextBoxSyntax::Url)
                .desired_width(ADVANCE_TEXT_WIDTH)
                .ui(ui);
            if response.changed() {
                state.set_proxy_url(proxy_url);
            }
        });

        settings_taffy_form_row(tui, label_width, state.ui_tr("advance.certificate"), |ui| {
            let mut no_check_certificates = state.tool_paths.no_check_certificates;
            if ui
                .checkbox(
                    &mut no_check_certificates,
                    state.ui_tr("advance.skip_certificate_verification"),
                )
                .changed()
            {
                state.set_no_check_certificates(no_check_certificates);
            }
        });

        settings_taffy_form_row(tui, label_width, state.ui_tr("advance.use_cookies"), |ui| {
            let mut use_cookies = state.item_defaults.use_cookies;
            if ui
                .checkbox(&mut use_cookies, state.ui_tr("advance.enable_cookies"))
                .changed()
            {
                state.set_use_browser_cookies(use_cookies);
            }
        });
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("advance.cookie_source"),
            |ui| {
                let cookie_sources = state.available_browser_cookie_sources();
                let selected_label = cookie_sources
                    .iter()
                    .find(|option| option.value == state.tool_paths.browser_cookie_source)
                    .map(|option| cookie_source_label(state, option))
                    .unwrap_or_else(|| state.tool_paths.browser_cookie_source.clone());
                egui::ComboBox::from_id_salt("browser-cookie-source")
                    .selected_text(selected_label)
                    .show_ui(ui, |ui| {
                        for option in cookie_sources {
                            let option_label = cookie_source_label(state, &option);
                            if ui
                                .selectable_label(
                                    state.tool_paths.browser_cookie_source == option.value,
                                    option_label,
                                )
                                .clicked()
                            {
                                state.set_browser_cookie_source(option.value);
                            }
                        }
                    })
                    .response;
            },
        );
        if state.cookie_source_uses_file() {
            render_cookie_file_row(tui, state, label_width);
        } else {
            render_browser_cookie_profile_row(tui, state, label_width);
        }
    });
}

fn cookie_source_label(state: &AppState, option: &BrowserCookieSourceOption) -> String {
    if option.value == "file" {
        state.ui_tr("advance.cookie_source.file").to_owned()
    } else {
        option.label.to_owned()
    }
}

fn render_cookie_file_row(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_form_row(tui, label_width, state.ui_tr("advance.cookie_file"), |ui| {
        let mut cookie_file_display = if state.tool_paths.browser_cookie_file.trim().is_empty() {
            state.ui_tr("advance.no_cookies_txt_selected").to_owned()
        } else {
            state.tool_paths.browser_cookie_file.clone()
        };
        ui.horizontal(|ui| {
            AppTextBox::new(&mut cookie_file_display)
                .language(state.language())
                .syntax(AppTextBoxSyntax::Path)
                .desired_width(ADVANCE_TEXT_WIDTH)
                .editable(false)
                .selectable(false)
                .enabled(false)
                .ui(ui);
            if ui.button(state.ui_tr("advance.browse")).clicked() {
                let mut dialog = rfd::FileDialog::new()
                    .add_filter(state.ui_tr("advance.filter_netscape_cookies_txt"), &["txt"])
                    .add_filter(state.ui_tr("advance.filter_all_files"), &["*"])
                    .set_title(state.ui_tr("advance.select_netscape_cookies_txt"));
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
            if ui.button(state.ui_tr("advance.clear")).clicked() {
                state.set_browser_cookie_file(String::new());
            }
        });
    });
}

fn render_browser_cookie_profile_row(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_form_row(tui, label_width, state.ui_tr("advance.browser"), |ui| {
        let cookie_profiles = state.available_browser_cookie_profiles();
        let selected_profile_label = if state.tool_paths.browser_cookie_profile.trim().is_empty() {
            state.ui_tr("advance.default").to_owned()
        } else {
            cookie_profiles
                .iter()
                .find(|option| option.value == state.tool_paths.browser_cookie_profile)
                .map(|option| option.label.clone())
                .unwrap_or_else(|| state.tool_paths.browser_cookie_profile.clone())
        };
        ui.horizontal(|ui| {
            ui.label(state.ui_tr("advance.config"));
            egui::ComboBox::from_id_salt("browser-cookie-profile")
                .selected_text(selected_profile_label)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            state.tool_paths.browser_cookie_profile.trim().is_empty(),
                            state.ui_tr("advance.default"),
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
    });
}

fn render_aria2_section(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, "Aria2", |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("advance.external_downloader"),
            |ui| {
                let mut use_aria2 = state.item_defaults.use_aria2;
                if ui
                    .checkbox(
                        &mut use_aria2,
                        state.ui_tr("advance.use_aria2_for_faster_downloads"),
                    )
                    .changed()
                {
                    state.set_use_aria2(use_aria2);
                }
            },
        );
    });
}

fn render_download_processing_section(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("advance.download_control"), |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("advance.concurrent_fragments"),
            |ui| {
                let selected = state.tool_paths.concurrent_fragments;
                egui::ComboBox::from_id_salt("concurrent-fragments")
                    .selected_text(format!("{}", selected.max(1)))
                    .show_ui(ui, |ui| {
                        for value in state.available_concurrent_fragment_values() {
                            let label = if value == 1 {
                                state.ui_tr("advance.1_default").to_owned()
                            } else {
                                value.to_string()
                            };
                            if ui.selectable_label(selected == value, label).clicked() {
                                state.set_concurrent_fragments(value);
                            }
                        }
                    })
                    .response;
            },
        );
        settings_taffy_form_row(tui, label_width, state.ui_tr("advance.rate_limit"), |ui| {
            let mut limit_rate = state.tool_paths.limit_rate.clone();
            if AppTextBox::new(&mut limit_rate)
                .hint_text(state.ui_tr("advance.e_g_2m_800k_leave_empty_for_unlimited"))
                .language(state.language())
                .syntax(AppTextBoxSyntax::Plain)
                .desired_width(ADVANCE_TEXT_WIDTH)
                .ui(ui)
                .changed()
            {
                state.set_limit_rate(limit_rate);
            }
        });
        settings_taffy_form_row(tui, label_width, state.ui_tr("advance.chapters"), |ui| {
            let mut compatibility_mode = state.tool_paths.chapter_compatibility_mode;
            if ui
                .checkbox(
                    &mut compatibility_mode,
                    state.ui_tr("advance.chapter_download_compatibility_mode"),
                )
                .changed()
            {
                state.set_chapter_compatibility_mode(compatibility_mode);
            }
        });
        settings_taffy_form_row(tui, label_width, state.ui_tr("advance.file_time"), |ui| {
            let selected = state.tool_paths.file_time_mode;
            egui::ComboBox::from_id_salt("file-time-mode")
                .selected_text(state.ui_tr(selected.label_key()))
                .show_ui(ui, |ui| {
                    for mode in [
                        FileTimeMode::None,
                        FileTimeMode::UseUploadDate,
                        FileTimeMode::UseDownloadTime,
                    ] {
                        if ui
                            .selectable_label(selected == mode, state.ui_tr(mode.label_key()))
                            .clicked()
                        {
                            state.set_file_time_mode(mode);
                        }
                    }
                })
                .response;
        });
    });
}

fn render_post_processing_section(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_tr("advance.post_processing"), |tui| {
        settings_taffy_form_row(tui, label_width, state.ui_tr("advance.thumbnail"), |ui| {
            ui.horizontal(|ui| {
                let mut write_thumbnail = state.item_defaults.write_thumbnail;
                if ui
                    .checkbox(&mut write_thumbnail, state.ui_tr("advance.download"))
                    .changed()
                {
                    state.set_write_thumbnail(write_thumbnail);
                }

                let mut embed_thumbnail = state.item_defaults.embed_thumbnail;
                if ui
                    .add_enabled(
                        state.item_defaults.write_thumbnail,
                        egui::Checkbox::new(&mut embed_thumbnail, state.ui_tr("advance.embed")),
                    )
                    .changed()
                {
                    state.set_embed_thumbnail(embed_thumbnail);
                }
            });
        });
        settings_taffy_form_row(tui, label_width, state.ui_tr("advance.subtitles"), |ui| {
            ui.horizontal(|ui| {
                let mut write_subtitles = state.item_defaults.write_subtitles;
                if ui
                    .checkbox(&mut write_subtitles, state.ui_tr("advance.download"))
                    .changed()
                {
                    state.set_write_subtitles(write_subtitles);
                }

                let mut embed_subtitles = state.item_defaults.embed_subtitles;
                if ui
                    .add_enabled(
                        state.item_defaults.write_subtitles,
                        egui::Checkbox::new(&mut embed_subtitles, state.ui_tr("advance.embed")),
                    )
                    .changed()
                {
                    state.set_embed_subtitles(embed_subtitles);
                }
            });
        });
        settings_taffy_form_row(tui, label_width, state.ui_tr("advance.chapters"), |ui| {
            ui.horizontal(|ui| {
                let mut write_chapters = state.item_defaults.write_chapters;
                if ui
                    .checkbox(&mut write_chapters, state.ui_tr("advance.download"))
                    .changed()
                {
                    state.set_write_chapters(write_chapters);
                }

                let mut embed_chapters = state.item_defaults.embed_chapters;
                if ui
                    .add_enabled(
                        state.item_defaults.write_chapters,
                        egui::Checkbox::new(&mut embed_chapters, state.ui_tr("advance.embed")),
                    )
                    .changed()
                {
                    state.set_embed_chapters(embed_chapters);
                }
            });
        });
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_tr("advance.download_conversion"),
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    let mut enabled = state.config.post_download_conversion_enabled;
                    if ui
                        .checkbox(&mut enabled, state.ui_tr("advance.enable"))
                        .changed()
                    {
                        state.set_enable_builtin_transcode_after_download(enabled);
                    }

                    if ui
                        .add(text_trailing_icon_button(
                            ui,
                            state.ui_tr("advance.settings"),
                            AppIcon::MenuRight,
                        ))
                        .clicked()
                    {
                        state.open_advance_detail_page(AdvanceDetailPage::Transcode);
                    }
                });
            },
        );
    });
}

fn config_location_preview(state: &AppState) -> String {
    let trimmed = state.tool_paths.yt_dlp_config.trim();
    if trimmed.is_empty() {
        "--ignore-config".to_owned()
    } else {
        format!("--config-location {trimmed}")
    }
}

fn proxy_preview(state: &AppState) -> String {
    let trimmed = state.tool_paths.proxy_url.trim();
    if trimmed.is_empty() {
        "--proxy <proxy-url>".to_owned()
    } else {
        format!("--proxy {trimmed}")
    }
}

fn certificate_preview() -> String {
    "--no-check-certificates".to_owned()
}

fn cookie_preview(state: &AppState) -> String {
    if state.cookie_source_uses_file() {
        let trimmed = state.tool_paths.browser_cookie_file.trim();
        if trimmed.is_empty() {
            return "--cookies <cookies.txt-path>".to_owned();
        }
        return format!("--cookies {trimmed}");
    }

    let source = state.tool_paths.browser_cookie_source.trim();
    let source = if source.is_empty() {
        "<browser>"
    } else {
        source
    };

    let profile = state.tool_paths.browser_cookie_profile.trim();
    let cookie_arg = if profile.is_empty() {
        source.to_owned()
    } else {
        format!("{source}:{profile}")
    };
    format!("--cookies-from-browser {cookie_arg}")
}

fn aria2_preview(state: &AppState) -> String {
    let aria2_path = state.tool_paths.aria2c.trim();
    let mut lines = Vec::new();
    if aria2_path.is_empty() {
        lines.push("--external-downloader <aria2c-path>".to_owned());
    } else {
        lines.push(format!("--external-downloader {aria2_path}"));
    }

    if state.tool_paths.effective_proxy_url().is_some() || state.tool_paths.no_check_certificates {
        let mut args = Vec::new();
        if let Some(proxy_url) = state.tool_paths.effective_proxy_url() {
            args.push(format!("--all-proxy={proxy_url}"));
        }
        if state.tool_paths.no_check_certificates {
            args.push("--check-certificate=false".to_owned());
        }
        lines.push(format!(
            "--external-downloader-args aria2c:{}",
            args.join(" ")
        ));
    }

    lines.join("\n")
}

fn concurrent_fragments_preview(state: &AppState) -> String {
    let fragments = state.tool_paths.concurrent_fragments.max(1);
    format!("--concurrent-fragments {fragments}")
}

fn limit_rate_preview(state: &AppState) -> String {
    let trimmed = state.tool_paths.limit_rate.trim();
    if trimmed.is_empty() {
        "--limit-rate <rate>".to_owned()
    } else {
        format!("--limit-rate {trimmed}")
    }
}

fn chapter_compatibility_preview(_state: &AppState) -> String {
    "For range downloads\n--compat-options no-direct-merge\n--format best[...][vcodec!=none][acodec!=none]/best".to_owned()
}

fn thumbnail_download_preview(_state: &AppState) -> String {
    "--write-thumbnail\n--convert-thumbnails jpg".to_owned()
}

fn thumbnail_embed_preview(_state: &AppState) -> String {
    "--embed-thumbnail\n--convert-thumbnails jpg".to_owned()
}

fn subtitle_download_preview(_state: &AppState) -> String {
    "When subtitles are selected\n--write-subs / --write-auto-subs\n--convert-subs srt".to_owned()
}

fn subtitle_embed_preview(_state: &AppState) -> String {
    "--embed-subs".to_owned()
}

fn chapter_download_preview(_state: &AppState) -> String {
    "--split-chapters".to_owned()
}

fn chapter_embed_preview(_state: &AppState) -> String {
    "--embed-chapters".to_owned()
}

fn advance_label_width(ui: &Ui, state: &AppState) -> f32 {
    let config_text = state.ui_tr("advance.config");
    let proxy_text = state.ui_tr("advance.proxy");
    let certificate_text = state.ui_tr("advance.certificate");
    let cookies_text = state.ui_tr("advance.use_cookies");
    let cookie_source_text = state.ui_tr("advance.cookie_source");
    let cookie_file_text = state.ui_tr("advance.cookie_file");
    let browser_text = state.ui_tr("advance.browser");
    let external_downloader_text = state.ui_tr("advance.external_downloader");
    let concurrent_fragments_text = state.ui_tr("advance.concurrent_fragments");
    let rate_limit_text = state.ui_tr("advance.rate_limit");
    let chapters_text = state.ui_tr("advance.chapters");
    let file_time_text = state.ui_tr("advance.file_time");
    let thumbnail_text = state.ui_tr("advance.thumbnail");
    let subtitles_text = state.ui_tr("advance.subtitles");
    let download_conversion_text = state.ui_tr("advance.download_conversion");

    measure_label_width(
        ui,
        &[
            config_text,
            proxy_text,
            certificate_text,
            cookies_text,
            cookie_source_text,
            cookie_file_text,
            browser_text,
            external_downloader_text,
            concurrent_fragments_text,
            rate_limit_text,
            chapters_text,
            file_time_text,
            thumbnail_text,
            subtitles_text,
            download_conversion_text,
        ],
    )
}
