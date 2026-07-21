use std::time::Duration;

use eframe::egui::{
    self, Align2, Color32, Response, RichText, Sense, Stroke, TextStyle, TextWrapMode, Ui,
    WidgetText,
};

use crate::app::state::ThumbnailRenderSource;
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::app::widgets::url_input::{accent_blue_for_ui, accent_green_for_ui, accent_red_for_ui};

use super::semantic_ui_metrics;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum CompactRowVisualState {
    Idle,
    Resolving,
    Playing { progress: f32 },
    Paused { progress: f32 },
    Finished,
    Downloaded,
    Failed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum CompactRowActivityPulse {
    #[default]
    None,
    CachePreparing,
    MixNextStandby,
}

impl CompactRowActivityPulse {
    fn is_active(self) -> bool {
        self != Self::None
    }
}

pub(super) struct CompactRowSpec<'a> {
    pub id_salt: u64,
    pub title: &'a str,
    pub thumbnail_url: &'a str,
    pub thumbnail_source: ThumbnailRenderSource,
    pub status_text: &'a str,
    pub visual_state: CompactRowVisualState,
    pub progress: f32,
    pub show_progress: bool,
    pub is_current: bool,
    pub is_playing: bool,
    pub activity_pulse: CompactRowActivityPulse,
    pub play_enabled: bool,
    pub cover_uses_mix_next: bool,
    pub remove_enabled: bool,
}

pub(super) struct CompactRowOutput {
    pub response: Response,
    pub play_clicked: bool,
    pub remove_clicked: bool,
}

pub(super) fn render_music_compact_row(ui: &mut Ui, spec: CompactRowSpec<'_>) -> CompactRowOutput {
    let row_width = ui.available_width().max(1.0);
    let desired_size = egui::vec2(row_width, semantic_ui_metrics::music_compact_row_height());
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
    let row_rect = rect.shrink2(egui::vec2(
        semantic_ui_metrics::music_compact_row_side_inset(),
        0.0,
    ));
    let fill = compact_row_fill(ui, spec.visual_state);
    let stroke = Stroke::new(
        semantic_ui_metrics::music_compact_row_border_stroke_width(),
        ui.visuals().widgets.noninteractive.bg_stroke.color,
    );

    ui.painter().rect(
        row_rect,
        semantic_ui_metrics::music_compact_row_corner_radius(),
        fill,
        stroke,
        egui::StrokeKind::Outside,
    );

    if spec.activity_pulse.is_active() {
        render_activity_breathing_row(ui, row_rect, spec.activity_pulse);
    }

    let progress = spec.progress.clamp(0.0, 1.0);
    if spec.show_progress && progress > 0.0 && progress < 0.999 {
        let fill_rect = egui::Rect::from_min_max(
            row_rect.min,
            egui::pos2(
                row_rect.left() + row_rect.width() * progress,
                row_rect.bottom(),
            ),
        );
        ui.painter().rect_filled(
            fill_rect,
            semantic_ui_metrics::music_compact_row_corner_radius(),
            compact_progress_fill(ui),
        );
    }

    let center_y = row_rect.center().y;
    let cover_rect = egui::Rect::from_min_size(
        egui::pos2(
            row_rect.left() + semantic_ui_metrics::music_compact_row_horizontal_padding(),
            center_y - semantic_ui_metrics::music_compact_cover_size() * 0.5,
        ),
        egui::vec2(
            semantic_ui_metrics::music_compact_cover_size(),
            semantic_ui_metrics::music_compact_cover_size(),
        ),
    );
    let play_response = ui.interact(
        cover_rect,
        ui.id().with(("compact-row-play", spec.id_salt)),
        if spec.play_enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    play_response.clone().on_hover_text(if spec.is_playing {
        "Pause"
    } else if spec.cover_uses_mix_next {
        "Mix next"
    } else {
        "Play"
    });
    render_compact_cover(ui, cover_rect, spec.thumbnail_url, spec.thumbnail_source);
    render_cover_play_overlay(
        ui,
        cover_rect,
        &play_response,
        spec.play_enabled
            && (response.hovered() || spec.is_current || spec.activity_pulse.is_active()),
        spec.is_playing,
        spec.cover_uses_mix_next,
    );
    if spec.activity_pulse.is_active() {
        render_activity_breathing_cover(ui, cover_rect, spec.activity_pulse);
    }

    if spec.is_current {
        let marker_rect = semantic_ui_metrics::music_compact_current_marker_rect_for_row(row_rect);
        ui.painter().rect_filled(
            marker_rect,
            semantic_ui_metrics::music_compact_current_marker_corner_radius(),
            accent_blue_for_ui(ui),
        );
    }

    let action_width = normal_item_delete_button_width(ui);
    let action_right_inset = normal_item_delete_right_inset(ui);
    let action_rect = egui::Rect::from_min_size(
        egui::pos2(
            row_rect.right() - action_right_inset - action_width,
            center_y - action_width * 0.5,
        ),
        egui::vec2(action_width, action_width),
    );
    let action_response = ui.interact(
        action_rect,
        ui.id().with(("compact-row-remove", spec.id_salt)),
        if spec.remove_enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if spec.remove_enabled {
        render_remove_action(
            ui,
            action_rect,
            &action_response,
            response.hovered() || action_response.hovered(),
        );
    }

    let status_rect = egui::Rect::from_min_size(
        egui::pos2(
            action_rect.left() - semantic_ui_metrics::music_compact_status_column_width(),
            row_rect.top(),
        ),
        egui::vec2(
            semantic_ui_metrics::music_compact_status_column_width(),
            row_rect.height(),
        ),
    );
    render_right_status(ui, status_rect, spec.status_text, spec.visual_state);

    let title_left =
        cover_rect.right() + semantic_ui_metrics::music_compact_title_to_neighbor_horizontal_gap();
    let title_right =
        status_rect.left() - semantic_ui_metrics::music_compact_title_to_neighbor_horizontal_gap();
    let title_width = (title_right - title_left).max(0.0);
    render_title(
        ui,
        title_left,
        center_y,
        title_width,
        spec.title,
        spec.visual_state,
    );

    let response = response;

    CompactRowOutput {
        response,
        play_clicked: spec.play_enabled && play_response.clicked(),
        remove_clicked: spec.remove_enabled && action_response.clicked(),
    }
}

fn compact_row_fill(ui: &Ui, state: CompactRowVisualState) -> Color32 {
    match state {
        CompactRowVisualState::Failed => subtle_tint(ui, Color32::from_rgb(170, 54, 54), 0.12),
        CompactRowVisualState::Finished | CompactRowVisualState::Downloaded => {
            ui.visuals().widgets.noninteractive.bg_fill
        }
        CompactRowVisualState::Playing { .. } => subtle_tint(ui, accent_blue_for_ui(ui), 0.06),
        CompactRowVisualState::Paused { .. } => subtle_tint(ui, accent_blue_for_ui(ui), 0.04),
        _ => ui.visuals().widgets.noninteractive.bg_fill,
    }
}

fn compact_progress_fill(ui: &Ui) -> Color32 {
    let color = accent_blue_for_ui(ui);
    if ui.visuals().dark_mode {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 110)
    } else {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 90)
    }
}

fn render_activity_breathing_row(ui: &Ui, rect: egui::Rect, activity: CompactRowActivityPulse) {
    ui.ctx().request_repaint_after(Duration::from_millis(80));
    let phase = ui.input(|input| input.time) * std::f64::consts::TAU / 1.35;
    let pulse = ((phase.sin() as f32) * 0.5 + 0.5).clamp(0.0, 1.0);
    let color = accent_blue_for_ui(ui);
    let (tint_base, tint_span, glow_base, glow_span, stroke_base, stroke_span) = match activity {
        CompactRowActivityPulse::MixNextStandby => (20.0, 30.0, 80.0, 95.0, 1.15, 0.75),
        CompactRowActivityPulse::CachePreparing => (10.0, 18.0, 52.0, 68.0, 0.95, 0.55),
        CompactRowActivityPulse::None => return,
    };
    let corner_radius = semantic_ui_metrics::music_compact_row_corner_radius();
    let tint = Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (tint_base + tint_span * pulse).round().clamp(0.0, 72.0) as u8,
    );
    let glow = Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (glow_base + glow_span * pulse).round().clamp(0.0, 210.0) as u8,
    );
    ui.painter().rect_filled(rect, corner_radius, tint);
    ui.painter().rect_stroke(
        rect.expand(0.75 + 1.25 * pulse),
        corner_radius,
        Stroke::new(stroke_base + stroke_span * pulse, glow),
        egui::StrokeKind::Outside,
    );
}

fn render_activity_breathing_cover(ui: &Ui, rect: egui::Rect, activity: CompactRowActivityPulse) {
    let phase = ui.input(|input| input.time) * std::f64::consts::TAU / 1.18;
    let pulse = ((phase.sin() as f32) * 0.5 + 0.5).clamp(0.0, 1.0);
    let color = accent_blue_for_ui(ui);
    let (glow_base, glow_span, stroke_base, stroke_span) = match activity {
        CompactRowActivityPulse::MixNextStandby => (96.0, 96.0, 1.2, 0.8),
        CompactRowActivityPulse::CachePreparing => (62.0, 72.0, 1.0, 0.6),
        CompactRowActivityPulse::None => return,
    };
    let glow = Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (glow_base + glow_span * pulse).round().clamp(0.0, 220.0) as u8,
    );
    ui.painter().rect_stroke(
        rect.expand(1.0 + 1.2 * pulse),
        semantic_ui_metrics::music_compact_cover_corner_radius(),
        Stroke::new(stroke_base + stroke_span * pulse, glow),
        egui::StrokeKind::Outside,
    );
}

fn subtle_tint(ui: &Ui, color: Color32, strength: f32) -> Color32 {
    let base = ui.visuals().widgets.noninteractive.bg_fill;
    let mix = |a: u8, b: u8| -> u8 {
        ((a as f32) * (1.0 - strength) + (b as f32) * strength)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color32::from_rgba_unmultiplied(
        mix(base.r(), color.r()),
        mix(base.g(), color.g()),
        mix(base.b(), color.b()),
        base.a(),
    )
}

fn render_compact_cover(
    ui: &mut Ui,
    rect: egui::Rect,
    thumbnail_url: &str,
    thumbnail_source: ThumbnailRenderSource,
) {
    match thumbnail_source {
        ThumbnailRenderSource::Texture(texture) => {
            ui.painter().image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().rect_stroke(
                rect,
                semantic_ui_metrics::music_compact_cover_corner_radius(),
                Stroke::new(
                    semantic_ui_metrics::music_compact_cover_border_stroke_width(),
                    ui.visuals().widgets.noninteractive.bg_stroke.color,
                ),
                egui::StrokeKind::Outside,
            );
            return;
        }
        ThumbnailRenderSource::DirectUrl if !thumbnail_url.trim().is_empty() => {
            egui::Image::new(thumbnail_url)
                .fit_to_exact_size(rect.size())
                .show_loading_spinner(false)
                .paint_at(ui, rect);
            ui.painter().rect_stroke(
                rect,
                semantic_ui_metrics::music_compact_cover_corner_radius(),
                Stroke::new(
                    semantic_ui_metrics::music_compact_cover_border_stroke_width(),
                    ui.visuals().widgets.noninteractive.bg_stroke.color,
                ),
                egui::StrokeKind::Outside,
            );
            return;
        }
        ThumbnailRenderSource::Loading
        | ThumbnailRenderSource::Failed(_)
        | ThumbnailRenderSource::None
        | ThumbnailRenderSource::DirectUrl => {}
    }

    let fill = if ui.visuals().dark_mode {
        Color32::from_rgb(48, 50, 56)
    } else {
        Color32::from_rgb(232, 235, 242)
    };
    ui.painter().rect_filled(
        rect,
        semantic_ui_metrics::music_compact_cover_corner_radius(),
        fill,
    );
    ui.painter().rect_stroke(
        rect,
        semantic_ui_metrics::music_compact_cover_corner_radius(),
        Stroke::new(
            semantic_ui_metrics::music_compact_cover_border_stroke_width(),
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ),
        egui::StrokeKind::Outside,
    );
    let icon_size = semantic_ui_metrics::music_compact_placeholder_icon_size_for_cover_rect(rect);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(AppIcon::VolumeHigh, icon_size, standard_icon_color(ui)).paint_at(ui, icon_rect);
}

fn render_cover_play_overlay(
    ui: &Ui,
    rect: egui::Rect,
    response: &Response,
    visible: bool,
    is_playing: bool,
    uses_mix_next: bool,
) {
    if !visible && !response.hovered() {
        return;
    }
    let radius = semantic_ui_metrics::music_compact_play_overlay_radius_for_cover_rect(rect);
    let center = rect.center();
    let fill = if ui.visuals().dark_mode {
        Color32::from_rgba_unmultiplied(0, 0, 0, 150)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 190)
    };
    ui.painter().circle_filled(center, radius, fill);
    let icon = if is_playing {
        AppIcon::Pause
    } else if uses_mix_next {
        AppIcon::SkipNext
    } else {
        AppIcon::Play
    };
    let icon_size = semantic_ui_metrics::music_compact_play_overlay_icon_size_for_radius(radius);
    let icon_rect = egui::Rect::from_center_size(center, egui::vec2(icon_size, icon_size));
    icon_image(icon, icon_size, standard_icon_color(ui)).paint_at(ui, icon_rect);
}

fn render_title(
    ui: &Ui,
    left: f32,
    center_y: f32,
    max_width: f32,
    title: &str,
    state: CompactRowVisualState,
) {
    let color = match state {
        CompactRowVisualState::Failed => Color32::from_rgb(210, 74, 74),
        CompactRowVisualState::Finished => accent_blue_for_ui(ui),
        CompactRowVisualState::Downloaded => accent_green_for_ui(ui),
        _ => ui.visuals().text_color(),
    };
    let galley = WidgetText::from(
        RichText::new(title).size(semantic_ui_metrics::music_compact_title_font_size_from_body(ui)),
    )
    .into_galley(ui, Some(TextWrapMode::Truncate), max_width, TextStyle::Body);
    let pos = egui::pos2(left, center_y - galley.size().y * 0.5);
    ui.painter().galley(pos, galley, color);
}

fn render_right_status(ui: &Ui, rect: egui::Rect, text: &str, state: CompactRowVisualState) {
    let color = match state {
        CompactRowVisualState::Failed => Color32::from_rgb(210, 74, 74),
        CompactRowVisualState::Finished | CompactRowVisualState::Playing { .. } => {
            accent_blue_for_ui(ui)
        }
        CompactRowVisualState::Downloaded => accent_green_for_ui(ui),
        _ => ui.visuals().weak_text_color(),
    };
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        egui::TextStyle::Body.resolve(ui.style()),
        color,
    );
}

fn normal_item_delete_button_width(ui: &Ui) -> f32 {
    ui.spacing().interact_size.y
}

fn normal_item_delete_right_inset(ui: &Ui) -> f32 {
    // Match the normal item card header, where the delete button sits inside
    // an egui group frame and is aligned to the frame's inner right edge.
    egui::Frame::group(ui.style()).inner_margin.right as f32
}

fn render_remove_action(ui: &Ui, rect: egui::Rect, response: &Response, item_hovered: bool) {
    let visuals = ui.style().interact(response);
    let icon_color = if response.hovered() || item_hovered {
        accent_red_for_ui(ui)
    } else {
        ui.visuals().weak_text_color()
    };

    ui.painter().rect(
        rect,
        semantic_ui_metrics::music_compact_remove_button_corner_radius(),
        visuals.bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Outside,
    );

    let icon_size = semantic_ui_metrics::music_compact_remove_icon_size();
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(AppIcon::WindowClose, icon_size, icon_color).paint_at(ui, icon_rect);
}
