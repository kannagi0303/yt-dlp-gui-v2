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
    let startup_diagnostics_enabled = infrastructure::enable_startup_diagnostics_if_requested();
    infrastructure::record_startup_checkpoint("main entered");
    infrastructure::record_startup_event("build", env!("YT_DLP_GUI_BUILD_DATE"));
    if startup_diagnostics_enabled {
        if let Ok(exe_path) = std::env::current_exe() {
            infrastructure::record_startup_event("exe", exe_path.display().to_string());
        }
        if let Ok(current_dir) = std::env::current_dir() {
            infrastructure::record_startup_event("cwd", current_dir.display().to_string());
        }
    }
    infrastructure::install_process_cleanup_panic_hook();
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
    match infrastructure::parse_edition_command() {
        Ok(Some(command)) => {
            if let Err(error) = infrastructure::run_edition_command(command) {
                eprintln!("[edition] {error}");
                std::process::exit(1);
            }
            return Ok(());
        }
        Ok(None) => {}
        Err(error) => {
            eprintln!("[edition] {error}");
            std::process::exit(1);
        }
    }

    let runtime_edition = infrastructure::load_current_runtime_edition();
    infrastructure::set_runtime_edition(runtime_edition.clone());

    if !runtime_edition.is_custom_edition() {
        match infrastructure::resume_pending_app_update_on_launch() {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(error) => eprintln!("[self-update] Could not resume pending update: {error}"),
        }
    }

    #[cfg(target_os = "windows")]
    if let Err(error) = infrastructure::app_identity::set_windows_process_app_identity() {
        eprintln!("[app-identity] Windows process identity unavailable: {error}");
    }

    let (config, tool_paths) = AppConfig::load_runtime();
    infrastructure::record_startup_checkpoint("config loaded");
    let window_title = infrastructure::runtime_window_title();
    let window_options = StartupWindowOptions::from_config(&config, window_title.clone());
    let centered = window_options.window_position.is_none();
    let native_options = eframe::NativeOptions {
        viewport: window_options.viewport_builder(),
        centered,
        persist_window: false,
        ..Default::default()
    };
    infrastructure::record_startup_checkpoint("native options built");
    infrastructure::record_startup_checkpoint("before run_native");

    let result = eframe::run_native(
        &window_title,
        native_options,
        Box::new(move |cc| {
            infrastructure::record_startup_checkpoint("app creator entered");
            let app = NgDlpApp::new_with_runtime(cc, config.clone(), tool_paths.clone());
            infrastructure::record_startup_checkpoint("app creator finished");
            Ok(Box::new(app))
        }),
    );
    if let Err(error) = &result {
        infrastructure::record_startup_error("run_native", format!("{error:?}"));
    }
    result
}

#[derive(Clone, Debug)]
struct StartupWindowOptions {
    keep_window_on_top: bool,
    window_position: Option<WindowPosition>,
    window_size: Option<WindowSize>,
    theme_mode: ThemeMode,
    ui_scale_percent: u16,
    window_title: String,
}

impl StartupWindowOptions {
    fn from_config(config: &AppConfig, window_title: String) -> Self {
        let restored_size = config.window_size.filter(|_| config.remember_window_size);
        let window_size = sanitize_startup_window_size(restored_size);
        let effective_size = window_size
            .map(|size| egui::vec2(size.width, size.height))
            .unwrap_or_else(|| default_window_size(ui_scale_factor(config.ui_scale_percent)));

        let restored_position = config
            .window_position
            .filter(|_| config.remember_window_position);
        let window_position = sanitize_startup_window_position(restored_position, effective_size);

        Self {
            keep_window_on_top: config.keep_window_on_top,
            // Issue #10 guardrail: saved YAML from older monitor layouts can restore the
            // window outside the current visible desktop. Sanitize before NativeOptions are
            // built so eframe can fall back to a centered startup window instead of appearing
            // to flash and disappear.
            window_position,
            window_size,
            theme_mode: config.theme_mode,
            ui_scale_percent: config.ui_scale_percent,
            window_title,
        }
    }

    fn viewport_builder(self) -> ViewportBuilder {
        let size = self
            .window_size
            .map(|size| egui::vec2(size.width, size.height))
            .unwrap_or_else(|| default_window_size(ui_scale_factor(self.ui_scale_percent)));

        let mut viewport = ViewportBuilder::default()
            .with_inner_size(size)
            .with_title(self.window_title)
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

fn sanitize_startup_window_size(size: Option<WindowSize>) -> Option<WindowSize> {
    let Some(size) = size else {
        return None;
    };
    if WindowSize::new(size.width, size.height).is_none() {
        infrastructure::record_startup_event(
            "window-state",
            format!(
                "ignored invalid saved size width={} height={}",
                size.width, size.height
            ),
        );
        return None;
    }

    if let Some(screen) = startup_visible_screen_rect() {
        if size.width > screen.width || size.height > screen.height {
            infrastructure::record_startup_event(
                "window-state",
                format!(
                    "ignored oversized saved size width={} height={} screen={}x{}",
                    size.width, size.height, screen.width, screen.height
                ),
            );
            return None;
        }
    }

    Some(size)
}

fn sanitize_startup_window_position(
    position: Option<WindowPosition>,
    effective_size: egui::Vec2,
) -> Option<WindowPosition> {
    let Some(position) = position else {
        return None;
    };
    if WindowPosition::new(position.x, position.y).is_none() {
        infrastructure::record_startup_event(
            "window-state",
            format!(
                "ignored invalid saved position x={} y={}",
                position.x, position.y
            ),
        );
        return None;
    }

    let window_rect = StartupRect {
        x: position.x,
        y: position.y,
        width: effective_size.x,
        height: effective_size.y,
    };
    if let Some(screen) = startup_visible_screen_rect() {
        if !window_rect.has_safe_visible_area_within(screen) {
            infrastructure::record_startup_event(
                "window-state",
                format!(
                    "ignored off-screen saved position x={} y={} size={}x{} screen=({}, {}) {}x{}",
                    position.x,
                    position.y,
                    effective_size.x,
                    effective_size.y,
                    screen.x,
                    screen.y,
                    screen.width,
                    screen.height
                ),
            );
            return None;
        }
    }

    Some(position)
}

#[derive(Clone, Copy, Debug)]
struct StartupRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl StartupRect {
    fn has_safe_visible_area_within(self, other: Self) -> bool {
        const MIN_VISIBLE_EDGE: f32 = 96.0;
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        (right - left) >= MIN_VISIBLE_EDGE && (bottom - top) >= MIN_VISIBLE_EDGE
    }
}

#[cfg(target_os = "windows")]
fn startup_visible_screen_rect() -> Option<StartupRect> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };

    // Use the virtual desktop because a saved window may legitimately live on a secondary
    // monitor, including monitors placed with negative coordinates.
    let x = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) } as f32;
    let y = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) } as f32;
    let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) } as f32;
    let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) } as f32;
    if width > 0.0 && height > 0.0 {
        Some(StartupRect {
            x,
            y,
            width,
            height,
        })
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
fn startup_visible_screen_rect() -> Option<StartupRect> {
    None
}
