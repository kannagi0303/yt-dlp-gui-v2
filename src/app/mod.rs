pub(crate) mod app_icon;
mod batch_add_worker;
mod compatibility_profiles;
mod download_worker;
mod format_picker_state;
mod media_probe;
mod metadata;
mod music_stream;
mod native_titlebar;
mod post_process_worker;
mod queue_status;
pub mod state;
mod thumbnail_worker;
mod tool_install_worker;
mod transcode_graph;
mod transcode_plan;
pub mod ui;
pub mod widgets;

use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use eframe::{
    CreationContext,
    egui::{self, FontData, FontDefinitions, FontFamily},
};
use egui_extras::install_image_loaders;

use self::{native_titlebar::NativeTitlebarAccentResult, state::AppState};
use crate::infrastructure::{ThemeAccentColor, ThemeMode};

const WINDOW_ICON_FADE_DURATION: Duration = Duration::from_millis(180);

pub struct NgDlpApp {
    state: AppState,
    applied_keep_window_on_top: Option<bool>,
    applied_theme_mode: Option<ThemeMode>,
    applied_theme_accent: Option<(ThemeAccentColor, bool)>,
    applied_native_titlebar_accent: Option<(ThemeAccentColor, bool)>,
    applied_window_icon_theme: Option<egui::Theme>,
    window_icon_transition: Option<WindowIconTransition>,
    applied_ui_scale_percent: Option<u16>,
    startup_focus_requested: bool,
}

struct WindowIconTransition {
    from: egui::Theme,
    to: egui::Theme,
    started_at: Instant,
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
            applied_theme_mode: None,
            applied_theme_accent: None,
            applied_native_titlebar_accent: None,
            applied_window_icon_theme: None,
            window_icon_transition: None,
            applied_ui_scale_percent: Some(initial_ui_scale_percent),
            startup_focus_requested: false,
        }
    }
}

impl Drop for NgDlpApp {
    fn drop(&mut self) {
        self.state.stop_music_playback();
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

    fn apply_theme_mode(&mut self, ctx: &egui::Context) {
        let mode = self.state.config.theme_mode;
        if self.applied_theme_mode == Some(mode) {
            return;
        }

        match mode {
            ThemeMode::System => {
                ctx.set_theme(egui::ThemePreference::System);
                ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(
                    egui::SystemTheme::SystemDefault,
                ));
            }
            ThemeMode::Light => {
                ctx.set_theme(egui::Theme::Light);
                ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Light));
            }
            ThemeMode::Dark => {
                ctx.set_theme(egui::Theme::Dark);
                ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));
            }
        }
        self.applied_theme_mode = Some(mode);
        self.applied_theme_accent = None;
        self.applied_native_titlebar_accent = None;
    }

    fn apply_theme_accent(&mut self, ctx: &egui::Context) {
        let accent = self.state.config.theme_accent_color;
        let dark_mode = ctx.global_style().visuals.dark_mode;
        let key = (accent, dark_mode);
        if self.applied_theme_accent == Some(key) {
            return;
        }

        let mut style = (*ctx.global_style()).clone();
        style.visuals = if dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        if !matches!(accent, ThemeAccentColor::Off) {
            tint_visuals(&mut style.visuals, accent, dark_mode);
        }
        ctx.set_global_style(style);
        self.applied_theme_accent = Some(key);
        self.applied_native_titlebar_accent = None;
    }

    fn apply_native_titlebar_accent(&mut self, ctx: &egui::Context) {
        let accent = self.state.config.theme_accent_color;
        let dark_mode = ctx.global_style().visuals.dark_mode;
        let key = (accent, dark_mode);
        if self.applied_native_titlebar_accent == Some(key) {
            return;
        }

        match native_titlebar::apply_titlebar_accent(accent, dark_mode) {
            NativeTitlebarAccentResult::Applied | NativeTitlebarAccentResult::Unsupported => {
                self.applied_native_titlebar_accent = Some(key);
            }
            NativeTitlebarAccentResult::NotReady => {}
        }
    }

    fn apply_window_icon_theme(&mut self, ctx: &egui::Context) {
        let theme = ctx.theme();
        if self.applied_window_icon_theme == Some(theme) && self.window_icon_transition.is_none() {
            return;
        }

        if self.applied_window_icon_theme.is_none() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Icon(Some(Arc::new(
                app_icon::app_window_icon(theme),
            ))));
            self.applied_window_icon_theme = Some(theme);
            return;
        }

        let transition_target = self
            .window_icon_transition
            .as_ref()
            .map(|transition| transition.to);
        if transition_target != Some(theme) && self.applied_window_icon_theme != Some(theme) {
            self.window_icon_transition =
                self.applied_window_icon_theme
                    .map(|from| WindowIconTransition {
                        from,
                        to: theme,
                        started_at: Instant::now(),
                    });
        }

        let Some(transition) = self.window_icon_transition.as_ref() else {
            return;
        };
        let from = transition.from;
        let to = transition.to;
        let progress =
            transition.started_at.elapsed().as_secs_f32() / WINDOW_ICON_FADE_DURATION.as_secs_f32();
        if progress >= 1.0 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Icon(Some(Arc::new(
                app_icon::app_window_icon(to),
            ))));
            self.applied_window_icon_theme = Some(to);
            self.window_icon_transition = None;
            return;
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Icon(Some(Arc::new(
            app_icon::app_window_icon_crossfade(from, to, progress),
        ))));
        ctx.request_repaint_after(Duration::from_millis(16));
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

fn tint_visuals(visuals: &mut egui::Visuals, accent: ThemeAccentColor, dark_mode: bool) {
    let (r, g, b) = accent.rgb();
    let accent = egui::Color32::from_rgb(r, g, b);
    let panel_tint = if dark_mode { 0.024 } else { 0.060 };
    let window_tint = if dark_mode { 0.020 } else { 0.052 };
    let widget_tint = if dark_mode { 0.060 } else { 0.110 };
    let hover_tint = if dark_mode { 0.095 } else { 0.165 };

    visuals.panel_fill = mix_color(visuals.panel_fill, accent, panel_tint);
    visuals.window_fill = mix_color(visuals.window_fill, accent, window_tint);
    visuals.extreme_bg_color = mix_color(visuals.extreme_bg_color, accent, panel_tint * 0.75);
    visuals.selection.bg_fill = mix_color(visuals.selection.bg_fill, accent, 0.55);

    visuals.widgets.inactive.bg_fill =
        mix_color(visuals.widgets.inactive.bg_fill, accent, widget_tint);
    visuals.widgets.hovered.bg_fill =
        mix_color(visuals.widgets.hovered.bg_fill, accent, hover_tint);
    visuals.widgets.active.bg_fill =
        mix_color(visuals.widgets.active.bg_fill, accent, hover_tint + 0.08);
    visuals.widgets.open.bg_fill = mix_color(visuals.widgets.open.bg_fill, accent, hover_tint);
}

fn mix_color(base: egui::Color32, accent: egui::Color32, amount: f32) -> egui::Color32 {
    let amount = amount.clamp(0.0, 1.0);
    let inv = 1.0 - amount;
    egui::Color32::from_rgba_premultiplied(
        ((base.r() as f32 * inv) + (accent.r() as f32 * amount)).round() as u8,
        ((base.g() as f32 * inv) + (accent.g() as f32 * amount)).round() as u8,
        ((base.b() as f32 * inv) + (accent.b() as f32 * amount)).round() as u8,
        base.a(),
    )
}

impl eframe::App for NgDlpApp {
    fn logic(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.apply_window_options(ctx);
        self.apply_theme_mode(ctx);
        self.apply_theme_accent(ctx);
        self.apply_native_titlebar_accent(ctx);
        self.apply_window_icon_theme(ctx);
        self.apply_ui_scale(ctx);
        self.request_startup_focus(ctx);
        self.state.sync_window_state(ctx);
        self.state.poll_background_work();
        self.state.poll_thumbnail_work(ctx);
        self.state.poll_clipboard_monitor();
        if self.state.has_active_work() || self.state.has_music_playback_activity() {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else if self.state.has_loading_thumbnails() {
            ctx.request_repaint_after(Duration::from_millis(250));
        } else if self.state.clipboard_monitor_enabled() {
            ctx.request_repaint_after(Duration::from_millis(800));
        }
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        ui::render_app(ui, &mut self.state);
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
