pub(crate) mod app_icon;
mod app_mode;
mod batch_add_worker;
mod compatibility_profiles;
mod component_update_worker;
mod custom_chrome;
mod download_resilience;
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
mod transcode_graph;
mod transcode_plan;
pub mod ui;
pub mod widgets;

use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use eframe::{
    CreationContext,
    egui::{self, FontData, FontDefinitions, FontFamily},
};
use egui_extras::install_image_loaders;

use self::{
    custom_chrome::CustomChromeResult,
    native_titlebar::NativeTitlebarAccentResult,
    state::{AppState, OptionsDetailPage, PrepareDetailPage},
};
use crate::i18n::Language;
use crate::infrastructure::{AppConfig, ThemeAccentColor, ThemeMode, ToolPaths};

const WINDOW_ICON_FADE_DURATION: Duration = Duration::from_millis(180);

pub struct NgDlpApp {
    state: AppState,
    applied_keep_window_on_top: Option<bool>,
    applied_theme_mode: Option<ThemeMode>,
    applied_theme_accent: Option<(ThemeAccentColor, bool)>,
    applied_custom_chrome: bool,
    applied_native_titlebar_accent: Option<(ThemeAccentColor, bool)>,
    applied_window_icon_theme: Option<egui::Theme>,
    window_icon_transition: Option<WindowIconTransition>,
    applied_ui_scale_percent: Option<u16>,
    applied_font_profile: Option<FontLoadProfile>,
    dynamic_font_scripts: DynamicFontScripts,
    observed_font_content_revision: u64,
    startup_focus_requested: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FontLoadProfile {
    language: Language,
    language_picker_visible: bool,
    dynamic_scripts: DynamicFontScripts,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DynamicFontScripts {
    han: bool,
    japanese: bool,
    korean: bool,
    emoji: bool,
    indic: bool,
    thai: bool,
}

impl DynamicFontScripts {
    fn observe_text(&mut self, text: &str) {
        for character in text.chars() {
            let code = character as u32;
            self.han |= is_han_codepoint(code);
            self.japanese |= is_japanese_codepoint(code);
            self.korean |= is_korean_codepoint(code);
            self.emoji |= is_emoji_codepoint(code);
            self.indic |= is_indic_codepoint(code);
            self.thai |= is_thai_or_lao_codepoint(code);
            if self.is_complete() {
                return;
            }
        }
    }

    fn merge(&mut self, other: Self) {
        self.han |= other.han;
        self.japanese |= other.japanese;
        self.korean |= other.korean;
        self.emoji |= other.emoji;
        self.indic |= other.indic;
        self.thai |= other.thai;
    }

    fn is_complete(self) -> bool {
        self.han && self.japanese && self.korean && self.emoji && self.indic && self.thai
    }
}

struct WindowIconTransition {
    from: egui::Theme,
    to: egui::Theme,
    started_at: Instant,
}

fn startup_font_profile(config: &AppConfig) -> FontLoadProfile {
    FontLoadProfile {
        language: config.language.resolve(),
        language_picker_visible: false,
        dynamic_scripts: DynamicFontScripts::default(),
    }
}

impl NgDlpApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let (config, tool_paths) = AppConfig::load_runtime();
        Self::new_with_runtime(cc, config, tool_paths)
    }

    pub fn new_with_runtime(
        cc: &CreationContext<'_>,
        config: AppConfig,
        tool_paths: ToolPaths,
    ) -> Self {
        let startup_font_profile = startup_font_profile(&config);
        let font_worker = thread::Builder::new()
            .name("startup-font-read".to_owned())
            .spawn(move || load_windows_ui_fonts(startup_font_profile))
            .ok();
        let state = AppState::from_runtime(config, tool_paths);
        let startup_fonts = font_worker
            .and_then(|worker| worker.join().ok())
            .unwrap_or_else(|| load_windows_ui_fonts(startup_font_profile));
        let ctx = &cc.egui_ctx;
        configure_loaded_fonts(ctx, startup_fonts);
        install_image_loaders(ctx);
        ctx.options_mut(|options| {
            options.max_passes = std::num::NonZeroUsize::new(2).unwrap();
        });
        let initial_ui_scale_percent = state.config.ui_scale_percent;
        ctx.set_zoom_factor(ui_scale_factor(initial_ui_scale_percent));

        Self {
            state,
            applied_keep_window_on_top: None,
            applied_theme_mode: None,
            applied_theme_accent: None,
            applied_custom_chrome: false,
            applied_native_titlebar_accent: None,
            applied_window_icon_theme: None,
            window_icon_transition: None,
            applied_ui_scale_percent: Some(initial_ui_scale_percent),
            applied_font_profile: Some(startup_font_profile),
            dynamic_font_scripts: DynamicFontScripts::default(),
            observed_font_content_revision: 0,
            startup_focus_requested: false,
        }
    }
}

impl Drop for NgDlpApp {
    fn drop(&mut self) {
        self.state.stop_music_playback();
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

    fn apply_custom_chrome(&mut self) {
        if self.applied_custom_chrome {
            return;
        }

        match custom_chrome::install() {
            CustomChromeResult::Applied | CustomChromeResult::Unsupported => {
                self.applied_custom_chrome = true;
                self.applied_native_titlebar_accent = None;
            }
            CustomChromeResult::NotReady => {}
        }
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
        self.apply_custom_chrome();
        self.apply_native_titlebar_accent(ctx);
        self.apply_window_icon_theme(ctx);
        self.apply_ui_scale(ctx);
        self.request_startup_focus(ctx);
        self.state.sync_window_state(ctx);
        self.state.poll_background_work();
        self.refresh_dynamic_font_scripts();
        self.apply_fonts(ctx);
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

impl NgDlpApp {
    fn refresh_dynamic_font_scripts(&mut self) {
        let revision = self.state.font_content_revision();
        if self.observed_font_content_revision == revision {
            return;
        }

        self.dynamic_font_scripts
            .merge(dynamic_font_scripts_for_state(&self.state));
        self.observed_font_content_revision = revision;
    }

    fn apply_fonts(&mut self, ctx: &egui::Context) {
        let profile = FontLoadProfile {
            language: self.state.language(),
            language_picker_visible: matches!(
                self.state.options_detail_page,
                Some(OptionsDetailPage::Language)
            ) || matches!(
                self.state.prepare_detail_page,
                Some(PrepareDetailPage::Language)
            ),
            dynamic_scripts: self.dynamic_font_scripts,
        };
        if self.applied_font_profile == Some(profile) {
            return;
        }

        configure_fonts(ctx, profile);
        self.applied_font_profile = Some(profile);
    }
}

fn dynamic_font_scripts_for_state(state: &AppState) -> DynamicFontScripts {
    let mut scripts = DynamicFontScripts::default();
    for text in [
        state.url_input.as_str(),
        state.batch_input.as_str(),
        state.last_action.as_str(),
    ] {
        scripts.observe_text(text);
    }
    observe_metadata_font_scripts(&mut scripts, &state.empty_item_preview);

    for item in &state.queue_items {
        for text in [
            item.title.as_str(),
            item.music_album_title.as_str(),
            item.selection.file_name.as_str(),
            item.last_error.as_deref().unwrap_or_default(),
        ] {
            scripts.observe_text(text);
        }
        if let Some(metadata) = item.metadata() {
            observe_metadata_font_scripts(&mut scripts, metadata);
        }
        if scripts.is_complete() {
            break;
        }
    }
    scripts
}

fn observe_metadata_font_scripts(
    scripts: &mut DynamicFontScripts,
    metadata: &crate::domain::VideoMetadata,
) {
    for text in [
        metadata.title.as_str(),
        metadata.channel.as_str(),
        metadata.uploader.as_str(),
        metadata.creator.as_str(),
        metadata.description.as_str(),
    ] {
        scripts.observe_text(text);
    }
    for chapter in &metadata.chapters {
        scripts.observe_text(&chapter.title);
    }
    for subtitle in &metadata.subtitle_tracks {
        scripts.observe_text(&subtitle.source_language_label);
        scripts.observe_text(
            subtitle
                .target_language_label
                .as_deref()
                .unwrap_or_default(),
        );
        observe_language_code_font_scripts(scripts, &subtitle.source_language_code);
        if let Some(code) = subtitle.target_language_code.as_deref() {
            observe_language_code_font_scripts(scripts, code);
        }
    }
}

fn observe_language_code_font_scripts(scripts: &mut DynamicFontScripts, language_code: &str) {
    let primary = language_code
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    match primary.as_str() {
        "ja" => scripts.japanese = true,
        "ko" => scripts.korean = true,
        "zh" => scripts.han = true,
        "th" | "lo" => scripts.thai = true,
        "hi" | "bn" | "pa" | "gu" | "or" | "ta" | "te" | "kn" | "ml" | "si" | "my" | "km" => {
            scripts.indic = true
        }
        _ => {}
    }
}

fn is_han_codepoint(code: u32) -> bool {
    matches!(
        code,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2FA1F
    )
}

fn is_japanese_codepoint(code: u32) -> bool {
    matches!(
        code,
        0x3040..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9F
    )
}

fn is_korean_codepoint(code: u32) -> bool {
    matches!(
        code,
        0x1100..=0x11FF
            | 0x3130..=0x318F
            | 0xA960..=0xA97F
            | 0xAC00..=0xD7AF
            | 0xD7B0..=0xD7FF
    )
}

fn is_emoji_codepoint(code: u32) -> bool {
    matches!(code, 0x2600..=0x27BF | 0xFE0F | 0x1F000..=0x1FAFF)
}

fn is_indic_codepoint(code: u32) -> bool {
    matches!(
        code,
        0x0900..=0x0DFF | 0x0F00..=0x109F | 0x1780..=0x17FF
    )
}

fn is_thai_or_lao_codepoint(code: u32) -> bool {
    matches!(code, 0x0E00..=0x0EFF)
}

fn configure_fonts(ctx: &eframe::egui::Context, profile: FontLoadProfile) {
    configure_loaded_fonts(ctx, load_windows_ui_fonts(profile));
}

fn configure_loaded_fonts(ctx: &eframe::egui::Context, loaded_fonts: Vec<(String, Vec<u8>)>) {
    let mut fonts = FontDefinitions::default();
    let mut loaded_font_names = Vec::new();

    for (font_name, font_bytes) in loaded_fonts {
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
    let mut family_fonts = loaded_font_names.to_vec();

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

struct WindowsFontGroup {
    font_name: &'static str,
    file_names: &'static [&'static str],
}

const WINDOWS_SEGOE_UI: WindowsFontGroup = WindowsFontGroup {
    font_name: "windows-segoeui",
    file_names: &["segoeui.ttf", "segoeuil.ttf", "arial.ttf"],
};
const WINDOWS_TRADITIONAL_CHINESE: WindowsFontGroup = WindowsFontGroup {
    font_name: "windows-jhenghei",
    file_names: &[
        "msjh.ttc",
        "mingliu.ttc",
        "msyh.ttc",
        "simsun.ttc",
        "YuGothM.ttc",
        "malgun.ttf",
    ],
};
const WINDOWS_SIMPLIFIED_CHINESE: WindowsFontGroup = WindowsFontGroup {
    font_name: "windows-yahei",
    file_names: &[
        "msyh.ttc",
        "simsun.ttc",
        "msjh.ttc",
        "mingliu.ttc",
        "YuGothM.ttc",
        "malgun.ttf",
    ],
};
const WINDOWS_JAPANESE: WindowsFontGroup = WindowsFontGroup {
    font_name: "windows-yugothic",
    file_names: &[
        "YuGothM.ttc",
        "meiryo.ttc",
        "msgothic.ttc",
        "msjh.ttc",
        "msyh.ttc",
    ],
};
const WINDOWS_KOREAN: WindowsFontGroup = WindowsFontGroup {
    font_name: "windows-malgungothic",
    file_names: &[
        "malgun.ttf",
        "gulim.ttc",
        "batang.ttc",
        "msjh.ttc",
        "msyh.ttc",
    ],
};
const WINDOWS_EMOJI: WindowsFontGroup = WindowsFontGroup {
    font_name: "windows-segoeuiemoji",
    file_names: &["seguiemj.ttf", "seguisym.ttf"],
};
const WINDOWS_INDIC: WindowsFontGroup = WindowsFontGroup {
    font_name: "windows-nirmalaui",
    file_names: &[
        "Nirmala.ttc",
        "Nirmala.ttf",
        "Mangal.ttf",
        "Vrinda.ttf",
        "Raavi.ttf",
        "Shruti.ttf",
        "Kalinga.ttf",
        "Latha.ttf",
        "Gautami.ttf",
        "Tunga.ttf",
        "Kartika.ttf",
    ],
};
const WINDOWS_THAI: WindowsFontGroup = WindowsFontGroup {
    font_name: "windows-leelawadeeui",
    file_names: &["LeelawUI.ttf", "LeelawUIb.ttf", "tahoma.ttf"],
};
const WINDOWS_SEGOE_SYMBOL: WindowsFontGroup = WindowsFontGroup {
    font_name: "windows-segoeuisymbol",
    file_names: &["seguisym.ttf", "seguiemj.ttf", "segoeui.ttf"],
};

fn windows_ui_font_groups(profile: FontLoadProfile) -> Vec<&'static WindowsFontGroup> {
    let mut groups = vec![&WINDOWS_SEGOE_UI];
    push_unique_font_group(
        &mut groups,
        windows_font_group_for_language(profile.language),
    );

    if profile.language_picker_visible {
        for group in [
            &WINDOWS_TRADITIONAL_CHINESE,
            &WINDOWS_SIMPLIFIED_CHINESE,
            &WINDOWS_JAPANESE,
            &WINDOWS_KOREAN,
        ] {
            push_unique_font_group(&mut groups, Some(group));
        }
    }
    if profile.dynamic_scripts.japanese {
        push_unique_font_group(&mut groups, Some(&WINDOWS_JAPANESE));
    }
    if profile.dynamic_scripts.korean {
        push_unique_font_group(&mut groups, Some(&WINDOWS_KOREAN));
    }
    if profile.dynamic_scripts.han {
        push_unique_font_group(&mut groups, Some(&WINDOWS_TRADITIONAL_CHINESE));
        push_unique_font_group(&mut groups, Some(&WINDOWS_SIMPLIFIED_CHINESE));
    }
    if profile.dynamic_scripts.emoji {
        push_unique_font_group(&mut groups, Some(&WINDOWS_EMOJI));
    }
    if profile.dynamic_scripts.indic {
        push_unique_font_group(&mut groups, Some(&WINDOWS_INDIC));
    }
    if profile.dynamic_scripts.thai {
        push_unique_font_group(&mut groups, Some(&WINDOWS_THAI));
    }
    push_unique_font_group(&mut groups, Some(&WINDOWS_SEGOE_SYMBOL));
    groups
}

fn push_unique_font_group(
    groups: &mut Vec<&'static WindowsFontGroup>,
    group: Option<&'static WindowsFontGroup>,
) {
    let Some(group) = group else {
        return;
    };
    if !groups
        .iter()
        .any(|existing| existing.font_name == group.font_name)
    {
        groups.push(group);
    }
}

fn windows_font_group_for_language(language: Language) -> Option<&'static WindowsFontGroup> {
    match language {
        Language::ZhTw => Some(&WINDOWS_TRADITIONAL_CHINESE),
        Language::ZhCn => Some(&WINDOWS_SIMPLIFIED_CHINESE),
        Language::JaJp => Some(&WINDOWS_JAPANESE),
        Language::KoKr => Some(&WINDOWS_KOREAN),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn load_windows_ui_fonts(profile: FontLoadProfile) -> Vec<(String, Vec<u8>)> {
    let font_dir = PathBuf::from(r"C:\Windows\Fonts");
    let mut loaded_paths = HashSet::new();
    windows_ui_font_groups(profile)
        .into_iter()
        .filter_map(|group| {
            group.file_names.iter().find_map(|file_name| {
                let path = font_dir.join(file_name);
                if !loaded_paths.insert(path.clone()) {
                    return None;
                }
                fs::read(path)
                    .ok()
                    .map(|bytes| (group.font_name.to_owned(), bytes))
            })
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn load_windows_ui_fonts(_profile: FontLoadProfile) -> Vec<(String, Vec<u8>)> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font_group_names(profile: FontLoadProfile) -> Vec<&'static str> {
        windows_ui_font_groups(profile)
            .into_iter()
            .map(|group| group.font_name)
            .collect()
    }

    #[test]
    fn startup_font_profile_only_loads_the_current_cjk_family() {
        let names = font_group_names(FontLoadProfile {
            language: Language::ZhTw,
            language_picker_visible: false,
            dynamic_scripts: DynamicFontScripts::default(),
        });

        assert_eq!(
            names,
            vec![
                "windows-segoeui",
                "windows-jhenghei",
                "windows-segoeuisymbol"
            ]
        );
    }

    #[test]
    fn language_picker_profile_loads_each_modern_cjk_family() {
        let names = font_group_names(FontLoadProfile {
            language: Language::EnUs,
            language_picker_visible: true,
            dynamic_scripts: DynamicFontScripts::default(),
        });

        assert_eq!(
            names,
            vec![
                "windows-segoeui",
                "windows-jhenghei",
                "windows-yahei",
                "windows-yugothic",
                "windows-malgungothic",
                "windows-segoeuisymbol"
            ]
        );
    }

    #[test]
    fn mixed_video_text_requests_lazy_script_fallbacks() {
        let mut scripts = DynamicFontScripts::default();
        scripts.observe_text("日本語かな 한국어 中文 🎬 हिन्दी ไทย");

        assert!(scripts.han);
        assert!(scripts.japanese);
        assert!(scripts.korean);
        assert!(scripts.emoji);
        assert!(scripts.indic);
        assert!(scripts.thai);
    }

    #[test]
    fn active_locale_font_precedes_dynamic_han_fallbacks() {
        let names = font_group_names(FontLoadProfile {
            language: Language::ZhCn,
            language_picker_visible: false,
            dynamic_scripts: DynamicFontScripts {
                han: true,
                ..DynamicFontScripts::default()
            },
        });

        assert_eq!(
            names,
            vec![
                "windows-segoeui",
                "windows-yahei",
                "windows-jhenghei",
                "windows-segoeuisymbol"
            ]
        );
    }
}
