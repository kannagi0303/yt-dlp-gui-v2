use eframe::egui::{
    self, Align2, Color32, Response, RichText, Sense, Stroke, TextStyle, TextWrapMode, Ui,
    WidgetText,
};

use crate::app::state::ThumbnailRenderSource;
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::app::widgets::url_input::{accent_blue_for_ui, accent_green_for_ui, accent_red_for_ui};

pub(super) const COMPACT_ROW_HEIGHT: f32 = 40.0;
const COMPACT_ROW_SIDE_INSET: f32 = 1.0;
const COMPACT_COVER_SIZE: f32 = 32.0;
const COMPACT_ROW_RADIUS: f32 = 6.0;
const COMPACT_ROW_PADDING_X: f32 = 6.0;
const COMPACT_GAP: f32 = 6.0;
const COMPACT_STATUS_WIDTH: f32 = 48.0;

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
    pub play_enabled: bool,
    pub remove_enabled: bool,
}

pub(super) struct CompactRowOutput {
    pub response: Response,
    pub play_clicked: bool,
    pub remove_clicked: bool,
}

pub(super) fn render_music_compact_row(ui: &mut Ui, spec: CompactRowSpec<'_>) -> CompactRowOutput {
    let row_width = ui.available_width().max(1.0);
    let desired_size = egui::vec2(row_width, COMPACT_ROW_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
    let row_rect = rect.shrink2(egui::vec2(COMPACT_ROW_SIDE_INSET, 0.0));
    let fill = compact_row_fill(ui, spec.visual_state);
    let stroke = Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);

    ui.painter().rect(
        row_rect,
        COMPACT_ROW_RADIUS,
        fill,
        stroke,
        egui::StrokeKind::Outside,
    );

    let progress = spec.progress.clamp(0.0, 1.0);
    if spec.show_progress && progress > 0.0 && progress < 0.999 {
        let fill_rect = egui::Rect::from_min_max(
            row_rect.min,
            egui::pos2(
                row_rect.left() + row_rect.width() * progress,
                row_rect.bottom(),
            ),
        );
        ui.painter()
            .rect_filled(fill_rect, COMPACT_ROW_RADIUS, compact_progress_fill(ui));
    }

    let center_y = row_rect.center().y;
    let cover_rect = egui::Rect::from_min_size(
        egui::pos2(
            row_rect.left() + COMPACT_ROW_PADDING_X,
            center_y - COMPACT_COVER_SIZE * 0.5,
        ),
        egui::vec2(COMPACT_COVER_SIZE, COMPACT_COVER_SIZE),
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
    render_compact_cover(ui, cover_rect, spec.thumbnail_url, spec.thumbnail_source);
    render_cover_play_overlay(
        ui,
        cover_rect,
        &play_response,
        spec.play_enabled && (response.hovered() || spec.is_current),
        spec.is_playing,
    );

    if spec.is_current {
        let marker_rect = egui::Rect::from_min_max(
            egui::pos2(row_rect.left(), row_rect.top() + 4.0),
            egui::pos2(row_rect.left() + 2.0, row_rect.bottom() - 4.0),
        );
        ui.painter()
            .rect_filled(marker_rect, 1.0, accent_blue_for_ui(ui));
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
        egui::pos2(action_rect.left() - COMPACT_STATUS_WIDTH, row_rect.top()),
        egui::vec2(COMPACT_STATUS_WIDTH, row_rect.height()),
    );
    render_right_status(ui, status_rect, spec.status_text, spec.visual_state);

    let title_left = cover_rect.right() + COMPACT_GAP;
    let title_right = status_rect.left() - COMPACT_GAP;
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
                5.0,
                Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
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
                5.0,
                Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
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
    ui.painter().rect_filled(rect, 5.0, fill);
    ui.painter().rect_stroke(
        rect,
        5.0,
        Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
        egui::StrokeKind::Outside,
    );
    let icon_size = rect.width() * 0.50;
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(AppIcon::VolumeHigh, icon_size, standard_icon_color(ui)).paint_at(ui, icon_rect);
}

fn render_cover_play_overlay(
    ui: &Ui,
    rect: egui::Rect,
    response: &Response,
    visible: bool,
    is_playing: bool,
) {
    if !visible && !response.hovered() {
        return;
    }
    let radius = rect.width() * 0.32;
    let center = rect.center();
    let fill = if ui.visuals().dark_mode {
        Color32::from_rgba_unmultiplied(0, 0, 0, 150)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 190)
    };
    ui.painter().circle_filled(center, radius, fill);
    let icon = if is_playing {
        AppIcon::Pause
    } else {
        AppIcon::Play
    };
    let icon_size = radius * 1.15;
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
    let body_size = TextStyle::Body.resolve(ui.style()).size;
    let galley = WidgetText::from(RichText::new(title).size(body_size + 1.0)).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        max_width,
        TextStyle::Body,
    );
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
        2.0,
        visuals.bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Outside,
    );

    let icon_size = 14.0;
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(AppIcon::WindowClose, icon_size, icon_color).paint_at(ui, icon_rect);
}
