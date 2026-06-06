#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![allow(
    dead_code,
    reason = "staged Rust rewrite keeps planned UI/domain/integration surfaces compiled before they are fully wired"
)]

mod app;
mod domain;
mod i18n;
mod infrastructure;

use app::{NgDlpApp, app_icon::app_window_icon};
use eframe::egui::{self, ViewportBuilder};
use infrastructure::{AppConfig, ThemeMode, WindowPosition, WindowSize};

fn main() -> eframe::Result<()> {
    if infrastructure::apply_update_args_requested() {
        let Some(args) = infrastructure::parse_apply_update_args() else {
            eprintln!("[self-update] invalid --apply-update arguments");
            std::process::exit(1);
        };
        if let Err(error) = infrastructure::run_apply_update(args) {
            eprintln!("[self-update] {error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    match infrastructure::resume_pending_app_update_on_launch() {
        Ok(true) => return Ok(()),
        Ok(false) => {}
        Err(error) => eprintln!("[self-update] Could not resume pending update: {error}"),
    }

    #[cfg(target_os = "windows")]
    if let Err(error) = infrastructure::app_identity::set_windows_process_app_identity() {
        eprintln!("[app-identity] Windows process identity unavailable: {error}");
    }

    let (config, tool_paths) = AppConfig::load_runtime();
    let window_options = StartupWindowOptions::from_config(&config);
    let native_options = eframe::NativeOptions {
        viewport: window_options.viewport_builder(),
        centered: window_options.window_position.is_none(),
        persist_window: false,
        ..Default::default()
    };

    eframe::run_native(
        "yt-dlp-gui",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(NgDlpApp::new_with_runtime(
                cc,
                config.clone(),
                tool_paths.clone(),
            )))
        }),
    )
}

#[derive(Clone, Copy, Debug)]
struct StartupWindowOptions {
    keep_window_on_top: bool,
    window_position: Option<WindowPosition>,
    window_size: Option<WindowSize>,
    theme_mode: ThemeMode,
    ui_scale_percent: u16,
}

impl StartupWindowOptions {
    fn from_config(config: &AppConfig) -> Self {
        Self {
            keep_window_on_top: config.keep_window_on_top,
            window_position: config
                .window_position
                .filter(|_| config.remember_window_position),
            window_size: config.window_size.filter(|_| config.remember_window_size),
            theme_mode: config.theme_mode,
            ui_scale_percent: config.ui_scale_percent,
        }
    }

    fn viewport_builder(self) -> ViewportBuilder {
        let size = self
            .window_size
            .map(|size| egui::vec2(size.width, size.height))
            .unwrap_or_else(|| default_window_size(ui_scale_factor(self.ui_scale_percent)));

        let mut viewport = ViewportBuilder::default()
            .with_inner_size(size)
            .with_title("yt-dlp-gui")
            .with_icon(app_window_icon(startup_icon_theme(self.theme_mode)));

        if let Some(position) = self.window_position {
            viewport = viewport.with_position(egui::pos2(position.x, position.y));
        }
        if self.keep_window_on_top {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnTop);
        }

        viewport
    }
}

fn startup_icon_theme(theme_mode: ThemeMode) -> egui::Theme {
    match theme_mode {
        ThemeMode::Light => egui::Theme::Light,
        ThemeMode::System | ThemeMode::Dark => egui::Theme::Dark,
    }
}

fn default_window_size(ui_scale_factor: f32) -> egui::Vec2 {
    egui::vec2(480.0, 280.0) * ui_scale_factor
}

fn ui_scale_factor(percent: u16) -> f32 {
    (percent as f32 / 100.0).clamp(0.8, 2.0)
}
