mod batch_add_worker;
mod download_worker;
mod format_picker_state;
mod metadata;
mod queue_status;
pub mod state;
mod thumbnail_worker;
mod tool_install_worker;
pub mod ui;
pub mod widgets;

use std::{fs, path::PathBuf, time::Duration};

use eframe::{
    CreationContext,
    egui::{self, FontData, FontDefinitions, FontFamily},
};
use egui_extras::install_image_loaders;

use self::state::AppState;

pub struct NgDlpApp {
    state: AppState,
    applied_keep_window_on_top: Option<bool>,
    applied_ui_scale_percent: Option<u16>,
    startup_focus_requested: bool,
}

impl NgDlpApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        configure_fonts(&cc.egui_ctx);
        install_image_loaders(&cc.egui_ctx);
        let state = AppState::new();
        let initial_ui_scale_percent = state.config.ui_scale_percent;
        cc.egui_ctx
            .set_zoom_factor(ui_scale_factor(initial_ui_scale_percent));

        Self {
            state,
            applied_keep_window_on_top: None,
            applied_ui_scale_percent: Some(initial_ui_scale_percent),
            startup_focus_requested: false,
        }
    }
}

impl Drop for NgDlpApp {
    fn drop(&mut self) {
        self.state.cleanup_active_tool_install();
        self.state.cleanup_active_download_processes();
    }
}

impl NgDlpApp {
    fn request_startup_focus(&mut self, ctx: &egui::Context) {
        if self.startup_focus_requested {
            return;
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        self.startup_focus_requested = true;
    }

    fn apply_window_options(&mut self, ctx: &egui::Context) {
        let enabled = self.state.config.keep_window_on_top;
        if self.applied_keep_window_on_top == Some(enabled) {
            return;
        }

        let level = if enabled {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
        self.applied_keep_window_on_top = Some(enabled);
    }

    fn apply_ui_scale(&mut self, ctx: &egui::Context) {
        let percent = self.state.config.ui_scale_percent;
        let Some(previous_percent) = self.applied_ui_scale_percent else {
            ctx.set_zoom_factor(ui_scale_factor(percent));
            self.applied_ui_scale_percent = Some(percent);
            return;
        };
        if previous_percent == percent {
            return;
        }

        ctx.set_zoom_factor(ui_scale_factor(percent));
        self.resize_window_for_ui_scale(ctx, previous_percent, percent);
        self.applied_ui_scale_percent = Some(percent);
    }

    fn resize_window_for_ui_scale(&self, ctx: &egui::Context, previous_percent: u16, percent: u16) {
        let previous = ui_scale_factor(previous_percent);
        if previous <= f32::EPSILON {
            return;
        }

        let viewport = ctx.input(|input| input.viewport().clone());
        if viewport.minimized.unwrap_or(false) || viewport.maximized.unwrap_or(false) {
            return;
        }

        let Some(inner_rect) = viewport.inner_rect else {
            return;
        };

        let ratio = ui_scale_factor(percent) / previous;
        let size = inner_rect.size() * ratio;
        if size.x.is_finite() && size.y.is_finite() && size.x > 0.0 && size.y > 0.0 {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
        }
    }
}

fn ui_scale_factor(percent: u16) -> f32 {
    (percent as f32 / 100.0).clamp(0.8, 2.0)
}

impl eframe::App for NgDlpApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.apply_window_options(ctx);
        self.apply_ui_scale(ctx);
        self.request_startup_focus(ctx);
        self.state.sync_window_state(ctx);
        self.state.poll_background_work();
        self.state.poll_thumbnail_work(ctx);
        self.state.poll_clipboard_monitor();
        if self.state.has_active_work() {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else if self.state.has_loading_thumbnails() {
            ctx.request_repaint_after(Duration::from_millis(250));
        } else if self.state.clipboard_monitor_enabled() {
            ctx.request_repaint_after(Duration::from_millis(800));
        }
        ui::render_app(ctx, &mut self.state);
    }
}

fn configure_fonts(ctx: &eframe::egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut loaded_font_names = Vec::new();

    for (font_name, font_bytes) in load_windows_ui_fonts() {
        fonts
            .font_data
            .insert(font_name.clone(), FontData::from_owned(font_bytes).into());
        loaded_font_names.push(font_name);
    }

    prepend_ui_font_order(&mut fonts, FontFamily::Proportional, &loaded_font_names);
    prepend_ui_font_order(&mut fonts, FontFamily::Monospace, &loaded_font_names);

    ctx.set_fonts(fonts);
}

fn prepend_ui_font_order(
    fonts: &mut FontDefinitions,
    family: FontFamily,
    loaded_font_names: &[String],
) {
    let preferred = [
        "windows-segoeui",
        "windows-jhenghei",
        "windows-yahei",
        "windows-malgungothic",
        "windows-yugothic",
        "windows-meiryo",
        "windows-msgothic",
        "windows-mingliu",
        "windows-simsun",
        "windows-gulim",
        "windows-batang",
        "windows-segoeuiemoji",
        "windows-segoeuisymbol",
    ];

    let mut family_fonts = preferred
        .into_iter()
        .filter(|font_name| loaded_font_names.iter().any(|loaded| loaded == font_name))
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if let Some(existing_fonts) = fonts.families.get(&family) {
        let existing_fonts = existing_fonts.clone();
        for font_name in existing_fonts {
            if !family_fonts.iter().any(|loaded| loaded == &font_name) {
                family_fonts.push(font_name);
            }
        }
    }

    if !family_fonts.is_empty() {
        fonts.families.insert(family, family_fonts);
    }
}

#[cfg(target_os = "windows")]
fn load_windows_ui_fonts() -> Vec<(String, Vec<u8>)> {
    let font_dir = PathBuf::from(r"C:\Windows\Fonts");
    let candidates = [
        ("windows-segoeui", "segoeui.ttf"),
        ("windows-jhenghei", "msjh.ttc"),
        ("windows-yahei", "msyh.ttc"),
        ("windows-malgungothic", "malgun.ttf"),
        ("windows-yugothic", "YuGothM.ttc"),
        ("windows-meiryo", "meiryo.ttc"),
        ("windows-msgothic", "msgothic.ttc"),
        ("windows-mingliu", "mingliu.ttc"),
        ("windows-simsun", "simsun.ttc"),
        ("windows-gulim", "gulim.ttc"),
        ("windows-batang", "batang.ttc"),
        ("windows-segoeuiemoji", "seguiemj.ttf"),
        ("windows-segoeuisymbol", "seguisym.ttf"),
    ];

    candidates
        .into_iter()
        .filter_map(|(font_name, file_name)| {
            let path = font_dir.join(file_name);
            fs::read(path)
                .ok()
                .map(|bytes| (font_name.to_owned(), bytes))
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn load_windows_ui_fonts() -> Vec<(String, Vec<u8>)> {
    Vec::new()
}
