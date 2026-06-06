use eframe::egui::{self, RichText, Ui};

use crate::app::state::{AppState, MusicPlaybackMode};
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::app::widgets::url_input::accent_red_for_ui;

use super::semantic_ui_metrics;
use super::xaml_rect_template::{show_rect_template, show_ui_at_rect};
use super::xaml_ui_nodes::{TemplateAxis, TemplateNode, auto, block, cols, gap, star};

type MusicPlayerTemplate = TemplateNode<MusicPlayerNode>;

#[derive(Debug, Clone, Copy)]
enum MusicPlayerNode {
    Button(MusicButtonRole),
    ContentPresenter(MusicContentRole),
}

#[derive(Debug, Clone, Copy)]
enum MusicButtonRole {
    Previous,
    PlayToggle,
    Next,
    PlaybackMode,
}

#[derive(Debug, Clone, Copy)]
enum MusicContentRole {
    Seek,
    Time,
    Volume,
}

pub(super) fn row_music_player(ui: &mut Ui, state: &mut AppState) {
    let row_height = ui.spacing().interact_size.y.max(1.0);
    let row_width = ui.available_width().max(1.0);
    ui.set_width(row_width);
    let (row_rect, _) =
        ui.allocate_exact_size(egui::vec2(row_width, row_height), egui::Sense::hover());

    let template = music_player_template(ui);
    show_music_player_template(ui, state, template, row_rect);
}

impl MusicPlayerNode {
    fn button(role: MusicButtonRole) -> Self {
        Self::Button(role)
    }

    fn content_presenter(role: MusicContentRole) -> Self {
        Self::ContentPresenter(role)
    }

    fn auto_width(self, row_height: f32) -> f32 {
        match self {
            Self::Button(_) => row_height.max(1.0),
            Self::ContentPresenter(role) => role.auto_width(),
        }
    }

    fn show_at(self, ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
        match self {
            Self::Button(role) => role.show_at(ui, state, rect),
            Self::ContentPresenter(role) => role.show_at(ui, state, rect),
        }
    }
}

impl MusicButtonRole {
    fn icon(self, state: &AppState) -> AppIcon {
        match self {
            Self::Previous => AppIcon::SkipPrevious,
            Self::PlayToggle => {
                if state.music_player_is_playing() {
                    AppIcon::Pause
                } else {
                    AppIcon::Play
                }
            }
            Self::Next => AppIcon::SkipNext,
            Self::PlaybackMode => music_playback_mode_icon(state.music_playback_mode_kind()),
        }
    }

    fn id_salt(self) -> &'static str {
        match self {
            Self::Previous => "music-previous",
            Self::PlayToggle => "music-play-toggle",
            Self::Next => "music-next",
            Self::PlaybackMode => "music-playback-mode",
        }
    }

    fn activate(self, state: &mut AppState) {
        match self {
            Self::Previous => state.previous_music_item(),
            Self::PlayToggle => state.toggle_music_playback(),
            Self::Next => state.next_music_item(),
            Self::PlaybackMode => state.cycle_music_playback_mode(),
        }
    }

    fn show_at(self, ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
        if music_icon_button_at(ui, rect, self.icon(state), self.id_salt()).clicked() {
            self.activate(state);
        }
    }
}

impl MusicContentRole {
    fn auto_width(self) -> f32 {
        match self {
            Self::Seek => 0.0,
            Self::Time => semantic_ui_metrics::main_music_player_time_text_width(),
            Self::Volume => semantic_ui_metrics::main_music_player_volume_control_width(),
        }
    }

    fn show_at(self, ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
        match self {
            Self::Seek => render_music_seek_or_error(ui, state, rect),
            Self::Time => render_music_time_at(ui, rect, &state.music_playback_time_text()),
            Self::Volume => render_music_volume_control(ui, state, rect),
        }
    }
}

fn music_player_template(ui: &Ui) -> MusicPlayerTemplate {
    let control_gap =
        semantic_ui_metrics::main_music_player_control_spacing_from_current_spacing(ui);
    cols([
        auto(block(MusicPlayerNode::button(MusicButtonRole::Previous))),
        gap(control_gap),
        auto(block(MusicPlayerNode::button(MusicButtonRole::PlayToggle))),
        gap(control_gap),
        auto(block(MusicPlayerNode::button(MusicButtonRole::Next))),
        gap(control_gap),
        star(
            1.0,
            block(MusicPlayerNode::content_presenter(MusicContentRole::Seek)),
        ),
        gap(control_gap),
        auto(block(MusicPlayerNode::content_presenter(
            MusicContentRole::Time,
        ))),
        gap(control_gap),
        auto(block(MusicPlayerNode::button(
            MusicButtonRole::PlaybackMode,
        ))),
        gap(control_gap),
        auto(block(MusicPlayerNode::content_presenter(
            MusicContentRole::Volume,
        ))),
    ])
}

fn show_music_player_template(
    ui: &mut Ui,
    state: &mut AppState,
    template: MusicPlayerTemplate,
    row_rect: egui::Rect,
) {
    let mut auto_main_size = |node: &MusicPlayerNode, axis: TemplateAxis, cross_size: f32| {
        debug_assert_eq!(axis, TemplateAxis::Cols);
        node.auto_width(cross_size)
    };
    let mut show_block = |node: MusicPlayerNode, rect: egui::Rect| {
        node.show_at(ui, state, rect);
    };

    show_rect_template(row_rect, template, &mut auto_main_size, &mut show_block);
}

fn render_music_seek_or_error(ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
    if rect.width() <= 1.0 {
        return;
    }

    if let Some(error) = state.music_player_error_text() {
        render_music_player_error_at(ui, rect, error);
    } else {
        render_music_seek_bar_at(ui, state, rect);
    }
}

fn render_music_volume_control(ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
    let mut volume = state.music_volume();
    let icon_size = rect.height().max(1.0);
    let volume_icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.top()),
        egui::vec2(icon_size, rect.height().max(1.0)),
    );
    music_round_icon_at(ui, volume_icon_rect, AppIcon::VolumeHigh);

    let slider_left = volume_icon_rect.right() + ui.spacing().icon_spacing;
    let volume_slider_right = rect.right().max(slider_left);
    let volume_slider_rect = egui::Rect::from_min_max(
        egui::pos2(slider_left, rect.top()),
        egui::pos2(volume_slider_right, rect.bottom()),
    );
    if volume_slider_rect.width() > 1.0 && volume_slider_rect.height() > 1.0 {
        let volume_response = slider_at(ui, volume_slider_rect, &mut volume);
        if volume_response.changed() {
            state.set_music_volume(volume);
        }
    }
}

fn slider_at(ui: &mut Ui, rect: egui::Rect, value: &mut f32) -> egui::Response {
    let slider = egui::Slider::new(value, 0.0..=1.0).show_value(false);
    show_ui_at_rect(ui, rect, |ui| {
        ui.spacing_mut().slider_width = rect.width().max(1.0);
        ui.add_sized(rect.size(), slider)
    })
}

fn render_music_time_at(ui: &Ui, rect: egui::Rect, text: &str) {
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::TextStyle::Body.resolve(ui.style()),
        ui.visuals().text_color(),
    );
}

fn render_music_player_error_at(ui: &Ui, rect: egui::Rect, error: &str) {
    let color = accent_red_for_ui(ui);
    let galley = egui::WidgetText::from(RichText::new(error).color(color)).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        rect.width(),
        egui::TextStyle::Body,
    );
    let pos = egui::pos2(rect.left(), rect.center().y - galley.size().y * 0.5);
    ui.painter().galley(pos, galley, color);
}

fn music_icon_button_at(
    ui: &mut Ui,
    rect: egui::Rect,
    icon: AppIcon,
    id_salt: &str,
) -> egui::Response {
    let id = ui.make_persistent_id(("music-icon-button", id_salt));
    let response = ui.interact(rect, id, egui::Sense::click());
    let visuals = ui.style().interact(&response);
    let radius = semantic_ui_metrics::main_music_round_button_radius_for_rect(rect);
    ui.painter()
        .circle_filled(rect.center(), radius, visuals.bg_fill);
    ui.painter()
        .circle_stroke(rect.center(), radius, visuals.bg_stroke);

    let icon_size =
        semantic_ui_metrics::main_music_playback_icon_size_from_current_control_metrics(ui);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(icon, icon_size, visuals.fg_stroke.color).paint_at(ui, icon_rect);
    response
}

fn music_round_icon_at(ui: &Ui, rect: egui::Rect, icon: AppIcon) {
    let radius = semantic_ui_metrics::main_music_round_button_radius_for_rect(rect);
    let fill = ui.visuals().faint_bg_color;
    let stroke = ui.visuals().widgets.inactive.bg_stroke;
    ui.painter().circle_filled(rect.center(), radius, fill);
    ui.painter().circle_stroke(rect.center(), radius, stroke);

    let icon_size =
        semantic_ui_metrics::main_music_volume_icon_size_from_current_control_metrics(ui);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(icon, icon_size, standard_icon_color(ui)).paint_at(ui, icon_rect);
}

fn music_playback_mode_icon(mode: MusicPlaybackMode) -> AppIcon {
    match mode {
        MusicPlaybackMode::Sequential => AppIcon::ArrowRight,
        MusicPlaybackMode::RepeatAll => AppIcon::Repeat,
        MusicPlaybackMode::Shuffle => AppIcon::Shuffle,
        MusicPlaybackMode::RepeatOne => AppIcon::RepeatOnce,
    }
}

fn render_music_seek_bar_at(ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
    let rect = semantic_ui_metrics::main_music_seek_bar_inner_rect(rect);
    if rect.width() <= 1.0 {
        return;
    }

    let mut value = state.music_seek_display_ratio().clamp(0.0, 1.0);
    let response = slider_at(ui, rect, &mut value);

    if response.changed() || response.dragged() {
        state.set_music_seek_drag_ratio(Some(value));
        ui.ctx().request_repaint();
    }

    let pointer_down = ui.input(|input| input.pointer.primary_down());
    if state.music_seek_drag_ratio().is_some() && !pointer_down {
        let final_ratio = state
            .music_seek_drag_ratio()
            .unwrap_or(value)
            .clamp(0.0, 1.0);
        state.finish_music_seek_drag(final_ratio);
    }
}
