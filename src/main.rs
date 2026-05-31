#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![allow(
    dead_code,
    reason = "staged Rust rewrite keeps planned UI/domain/integration surfaces compiled before they are fully wired"
)]

mod app;
mod domain;
mod i18n;
mod infrastructure;

use std::path::PathBuf;

use app::{NgDlpApp, app_icon::app_window_icon};
use eframe::egui::{self, ViewportBuilder};
use infrastructure::{
    AppConfig, ThemeMode, ToolPaths, WindowPosition, WindowSize, collect_prepare_report,
};

fn main() -> eframe::Result<()> {
    #[cfg(target_os = "windows")]
    if let Err(error) = infrastructure::app_identity::ensure_windows_app_identity() {
        eprintln!("[app-identity] Windows app identity unavailable: {error}");
    }

    let startup = StartupRuntime::load();
    let native_options = eframe::NativeOptions {
        viewport: startup.window_options.viewport_builder(),
        centered: startup.window_options.centered,
        persist_window: false,
        ..Default::default()
    };
    let config = startup.config;
    let tool_paths = startup.tool_paths;

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

struct StartupRuntime {
    config: AppConfig,
    tool_paths: ToolPaths,
    window_options: StartupWindowOptions,
}

impl StartupRuntime {
    fn load() -> Self {
        let config_path = config_file_path();
        let is_first_launch = !config_path.exists();
        let (config, tool_paths) = AppConfig::load_runtime();
        let window_options =
            startup_window_options_from_config(&config, &tool_paths, is_first_launch);
        Self {
            config,
            tool_paths,
            window_options,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct StartupWindowOptions {
    centered: bool,
    keep_window_on_top: bool,
    window_position: Option<WindowPosition>,
    window_size: Option<WindowSize>,
    theme_mode: ThemeMode,
    ui_scale_percent: u16,
}

impl StartupWindowOptions {
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

fn startup_window_options_from_config(
    config: &AppConfig,
    tool_paths: &ToolPaths,
    is_first_launch: bool,
) -> StartupWindowOptions {
    let prepare_report = collect_prepare_report(tool_paths, &config.download_dir);
    let will_show_prepare = !config.prepare_skipped && prepare_report.should_show_tab();

    let restore_window_state = !is_first_launch && !will_show_prepare;
    StartupWindowOptions {
        centered: !restore_window_state || !config.remember_window_position,
        keep_window_on_top: config.keep_window_on_top,
        window_position: restore_window_state
            .then_some(config.window_position)
            .flatten()
            .filter(|_| config.remember_window_position),
        window_size: restore_window_state
            .then_some(config.window_size)
            .flatten()
            .filter(|_| config.remember_window_size),
        theme_mode: config.theme_mode,
        ui_scale_percent: config.ui_scale_percent,
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

fn config_file_path() -> PathBuf {
    let app_dir = portable_root_dir();
    let file_name = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
        })
        .filter(|stem| !stem.is_empty())
        .map(|stem| format!("{stem}.yaml"))
        .unwrap_or_else(|| "yt-dlp-gui.yaml".to_owned());
    app_dir.join(file_name)
}

fn portable_root_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    }

    #[cfg(not(debug_assertions))]
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}
