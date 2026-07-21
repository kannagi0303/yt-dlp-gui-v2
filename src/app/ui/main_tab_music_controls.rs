use std::time::Duration;

use eframe::egui::{self, RichText, Stroke, Ui};

use crate::app::state::{
    AppState, MusicPlaybackMode, MusicPlayerAuraDisplay, MusicPlayerAuraTrackField,
};
use crate::app::widgets::icon::{AppIcon, icon_image};
use crate::app::widgets::url_input::{accent_blue_for_ui, accent_red_for_ui};

use super::main_tab_music_stage_controls::{
    music_stage_bpm_display, render_music_stage_control_at,
};
use super::semantic_ui_metrics;
use super::xaml_rect_template::{show_rect_template, show_ui_at_rect};
use super::xaml_ui_nodes::{
    TemplateAxis, TemplateNode, auto, block, cols, fill, fixed_px, gap, rows, star,
};

#[derive(Debug, Clone, Copy)]
enum MusicPlayerRowNode {
    Seek,
    Controls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MusicSeekRangeKind {
    Playable,
    Mix,
}

#[derive(Debug, Clone, Copy)]
struct MusicSeekRange {
    kind: MusicSeekRangeKind,
    start_seconds: f64,
    end_seconds: f64,
}

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
    Bpm,
    AnalysisPearls,
    Spacer,
    Time,
    StageControls,
    Volume,
}

pub(super) fn row_music_player(
    ui: &mut Ui,
    state: &mut AppState,
    aura_display: MusicPlayerAuraDisplay,
) {
    let control_row_height =
        semantic_ui_metrics::main_music_player_control_row_height_from_current_metrics(ui);
    let row_width = ui.available_width().max(1.0);
    let player_height = ui.available_height().max(1.0);
    ui.set_width(row_width);
    let (player_rect, _) =
        ui.allocate_exact_size(egui::vec2(row_width, player_height), egui::Sense::hover());
    let template = rows([
        fixed_px(
            semantic_ui_metrics::main_music_player_seek_row_height(),
            block(MusicPlayerRowNode::Seek),
        ),
        gap(semantic_ui_metrics::main_music_player_seek_to_controls_spacing()),
        fill(block(MusicPlayerRowNode::Controls)),
    ]);
    show_rect_template(
        player_rect,
        template,
        &mut |_, _: TemplateAxis, _| 0.0,
        &mut |node, rect| match node {
            MusicPlayerRowNode::Seek => render_music_seek_or_error(ui, state, rect),
            MusicPlayerRowNode::Controls => show_music_player_template(
                ui,
                state,
                aura_display,
                music_player_template(ui),
                rect,
                control_row_height,
            ),
        },
    );
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
            Self::ContentPresenter(role) => role.auto_width(row_height),
        }
    }

    fn show_at(
        self,
        ui: &mut Ui,
        state: &mut AppState,
        aura_display: MusicPlayerAuraDisplay,
        rect: egui::Rect,
    ) {
        match self {
            Self::Button(role) => role.show_at(ui, state, rect),
            Self::ContentPresenter(role) => role.show_at(ui, state, aura_display, rect),
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
            Self::PlaybackMode => {}
        }
    }

    fn show_at(self, ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
        if matches!(self, Self::PlaybackMode) {
            render_music_playback_mode_control(ui, state, rect);
            return;
        }
        if music_icon_button_at(ui, rect, self.icon(state), self.id_salt(), None).clicked() {
            self.activate(state);
        }
    }
}

fn render_music_playback_mode_control(ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
    let current = state.music_playback_mode_kind();
    let active_color = (current != MusicPlaybackMode::Sequential).then(|| accent_blue_for_ui(ui));
    let response = music_icon_button_at(
        ui,
        rect,
        music_playback_mode_icon(current),
        "music-playback-mode",
        active_color,
    );
    response.clone().on_hover_text(format!(
        "Playback order: {}",
        state.music_playback_mode_text()
    ));

    egui::Popup::menu(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| render_music_playback_mode_popup(ui, state));
}

fn render_music_playback_mode_popup(ui: &mut Ui, state: &mut AppState) {
    const POPUP_WIDTH: f32 = 172.0;
    const ROW_HEIGHT: f32 = 28.0;

    ui.set_width(POPUP_WIDTH);
    let (popup_rect, _) = ui.allocate_exact_size(
        egui::vec2(
            POPUP_WIDTH,
            ROW_HEIGHT * MusicPlaybackMode::ALL.len() as f32,
        ),
        egui::Sense::hover(),
    );
    let template = rows(MusicPlaybackMode::ALL.map(|mode| fill(block(mode))));
    show_rect_template(
        popup_rect,
        template,
        &mut |_, _: TemplateAxis, _| 0.0,
        &mut |mode, row_rect| render_music_playback_mode_choice(ui, state, row_rect, mode),
    );
}

fn render_music_playback_mode_choice(
    ui: &mut Ui,
    state: &mut AppState,
    rect: egui::Rect,
    mode: MusicPlaybackMode,
) {
    let selected = state.music_playback_mode_kind() == mode;
    let visible = state.ui_i18n_text_for_key(mode.label_key());
    let label = if selected {
        format!("✓  {visible}")
    } else {
        format!("   {visible}")
    };
    let mut button = egui::Button::new(RichText::new(label).size(11.0)).small();
    if selected {
        button = button.fill(ui.visuals().selection.bg_fill);
    }
    if ui.put(rect, button).clicked() {
        state.set_music_playback_mode(mode);
        ui.close();
    }
}

impl MusicContentRole {
    fn auto_width(self, row_height: f32) -> f32 {
        match self {
            Self::Bpm => semantic_ui_metrics::main_music_player_bpm_width(),
            Self::AnalysisPearls => semantic_ui_metrics::main_music_player_analysis_pearls_width(),
            Self::Spacer => 0.0,
            Self::Time => semantic_ui_metrics::main_music_player_time_text_width(),
            Self::StageControls => row_height.max(1.0),
            Self::Volume => row_height.max(1.0),
        }
    }

    fn show_at(
        self,
        ui: &mut Ui,
        state: &mut AppState,
        aura_display: MusicPlayerAuraDisplay,
        rect: egui::Rect,
    ) {
        match self {
            Self::Bpm => render_music_bpm_at(ui, state, rect),
            Self::AnalysisPearls => render_music_analysis_pearls_at(ui, aura_display, rect),
            Self::Spacer => {}
            Self::Time => render_music_time_at(ui, rect, &state.music_playback_time_text()),
            Self::StageControls => render_music_stage_control_at(ui, state, rect),
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
        auto(block(MusicPlayerNode::content_presenter(
            MusicContentRole::Volume,
        ))),
        gap(control_gap),
        auto(block(MusicPlayerNode::content_presenter(
            MusicContentRole::Time,
        ))),
        gap(control_gap),
        auto(block(MusicPlayerNode::content_presenter(
            MusicContentRole::Bpm,
        ))),
        gap(control_gap),
        auto(block(MusicPlayerNode::content_presenter(
            MusicContentRole::AnalysisPearls,
        ))),
        gap(control_gap),
        star(
            1.0,
            block(MusicPlayerNode::content_presenter(MusicContentRole::Spacer)),
        ),
        gap(control_gap),
        auto(block(MusicPlayerNode::content_presenter(
            MusicContentRole::StageControls,
        ))),
        gap(control_gap),
        auto(block(MusicPlayerNode::button(
            MusicButtonRole::PlaybackMode,
        ))),
    ])
}

fn render_music_bpm_at(ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
    let (text, animating) = music_stage_bpm_display(state);
    let color = if animating {
        accent_blue_for_ui(ui)
    } else {
        ui.visuals().weak_text_color()
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::TextStyle::Body.resolve(ui.style()),
        color,
    );
    if animating {
        ui.ctx().request_repaint_after(Duration::from_millis(16));
    }
}

fn show_music_player_template(
    ui: &mut Ui,
    state: &mut AppState,
    aura_display: MusicPlayerAuraDisplay,
    template: MusicPlayerTemplate,
    row_rect: egui::Rect,
    control_row_height: f32,
) {
    let mut auto_main_size = |node: &MusicPlayerNode, axis: TemplateAxis, cross_size: f32| {
        debug_assert_eq!(axis, TemplateAxis::Cols);
        node.auto_width(control_row_height.min(cross_size).max(1.0))
    };
    let mut show_block = |node: MusicPlayerNode, rect: egui::Rect| {
        node.show_at(ui, state, aura_display, rect);
    };

    show_rect_template(row_rect, template, &mut auto_main_size, &mut show_block);
}

#[derive(Clone, Copy, Debug, Default)]
struct MusicAnalysisPearlEnvelope {
    item_id: Option<u64>,
    fast: [f32; 4],
    slow: [f32; 4],
}

impl MusicAnalysisPearlEnvelope {
    fn advance(&mut self, targets: [f32; 4], dt: f32) {
        for index in 0..4 {
            let target = targets[index];
            let fast_time = if target > self.fast[index] {
                0.05
            } else {
                0.18
            };
            let slow_time = if target > self.slow[index] {
                0.24
            } else {
                0.32
            };
            self.fast[index] += (target - self.fast[index]) * envelope_alpha(dt, fast_time);
            self.slow[index] += (target - self.slow[index]) * envelope_alpha(dt, slow_time);
        }
    }

    fn signed_motion(&self, index: usize) -> f32 {
        ((self.fast[index] - self.slow[index]) * 3.0).clamp(-1.0, 1.0)
    }
}

fn envelope_alpha(dt: f32, time_constant: f32) -> f32 {
    if dt > 0.0 {
        1.0 - (-dt / time_constant).exp()
    } else {
        0.0
    }
}

fn render_music_analysis_pearls_at(ui: &mut Ui, display: MusicPlayerAuraDisplay, rect: egui::Rect) {
    let targets = music_analysis_pearl_targets(display);
    let item_id = display.primary_item_id;
    let dt = ui.input(|input| input.stable_dt).clamp(0.0, 0.05);
    let memory_id = ui.make_persistent_id("music-analysis-pearl-envelope");
    let mut envelope = ui.ctx().data_mut(|data| {
        data.get_temp::<MusicAnalysisPearlEnvelope>(memory_id)
            .unwrap_or_default()
    });
    if envelope.item_id != item_id {
        envelope = MusicAnalysisPearlEnvelope {
            item_id,
            fast: [0.0; 4],
            slow: [0.0; 4],
        };
    }
    envelope.advance(targets, dt);
    ui.ctx()
        .data_mut(|data| data.insert_temp(memory_id, envelope));

    let beat_phase = display
        .primary
        .map(|field| field.beat_phase)
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    let beat_glint = (1.0 - beat_phase).powf(7.0) * 0.14;
    let max_lift = semantic_ui_metrics::main_music_player_analysis_pearl_max_lift();
    let baseline_y = rect.center().y;
    let accent = accent_blue_for_ui(ui);
    let template = cols([
        star(1.0, block(0_usize)),
        gap(2.0),
        star(1.0, block(1_usize)),
        gap(2.0),
        star(1.0, block(2_usize)),
        gap(2.0),
        star(1.0, block(3_usize)),
    ]);
    show_rect_template(
        rect,
        template,
        &mut |_, _: TemplateAxis, _| 0.0,
        &mut |index, dot_rect| {
            let value = envelope.fast[index].clamp(0.0, 1.0);
            let center = egui::pos2(
                dot_rect.center().x,
                baseline_y - max_lift * envelope.signed_motion(index),
            );
            let radius =
                semantic_ui_metrics::main_music_player_analysis_pearl_radius(value + beat_glint);
            ui.painter().circle_filled(
                center,
                radius * 2.15,
                accent.gamma_multiply(0.10 + value * 0.10),
            );
            ui.painter()
                .circle_filled(center, radius, accent.gamma_multiply(0.48 + value * 0.52));
        },
    );

    if display.animating {
        ui.ctx().request_repaint_after(Duration::from_millis(16));
    }
}

fn music_analysis_pearl_targets(display: MusicPlayerAuraDisplay) -> [f32; 4] {
    fn bands(field: MusicPlayerAuraTrackField) -> [f32; 4] {
        [
            field.spectrum_bands[0].max(field.spectrum_bands[1]),
            field.spectrum_bands[2].max(field.spectrum_bands[3]),
            field.spectrum_bands[4].max(field.spectrum_bands[5]),
            field.spectrum_bands[6].max(field.spectrum_bands[7]),
        ]
    }

    let Some(primary) = display.primary else {
        return [0.0; 4];
    };
    let primary = bands(primary);
    let combined = if let Some(secondary) = display.secondary {
        let secondary = bands(secondary);
        let progress = display.mix_progress.clamp(0.0, 1.0);
        let weight_a = (progress * std::f32::consts::FRAC_PI_2).cos();
        let weight_b = (progress * std::f32::consts::FRAC_PI_2).sin();
        std::array::from_fn(|index| {
            ((primary[index] * weight_a).powi(2) + (secondary[index] * weight_b).powi(2)).sqrt()
        })
    } else {
        primary
    };
    combined.map(|value| {
        ((value.clamp(0.0, 1.0) - 0.035) / 0.965)
            .clamp(0.0, 1.0)
            .powf(0.62)
    })
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
    let response = music_icon_button_at(ui, rect, AppIcon::VolumeHigh, "music-volume-popup", None);
    response.clone().on_hover_text("Volume");
    egui::Popup::menu(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            let width = semantic_ui_metrics::main_music_player_volume_popup_width();
            let height = ui.spacing().interact_size.y.max(1.0);
            ui.set_width(width);
            let (slider_rect, _) =
                ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
            let mut volume = state.music_volume();
            let volume_response = slider_at(ui, slider_rect, &mut volume);
            if volume_response.changed() {
                state.set_music_volume(volume);
            }
        });
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
    active_color: Option<egui::Color32>,
) -> egui::Response {
    let id = ui.make_persistent_id(("music-icon-button", id_salt));
    let response = ui.interact(rect, id, egui::Sense::click());
    let radius = semantic_ui_metrics::main_music_round_button_radius_for_rect(rect);
    ui.painter().circle_filled(
        rect.center(),
        radius,
        semantic_ui_metrics::main_music_button_fill(ui, &response, active_color),
    );
    ui.painter().circle_stroke(
        rect.center(),
        radius,
        semantic_ui_metrics::main_music_button_stroke(ui, &response, active_color),
    );

    let icon_size = semantic_ui_metrics::main_music_playback_icon_size_for_rect(rect);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(
        icon,
        icon_size,
        semantic_ui_metrics::main_music_button_foreground(ui, &response, active_color),
    )
    .paint_at(ui, icon_rect);
    response
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

    let duration_seconds = state.music_current_duration_seconds();
    let seek_ranges = current_music_seek_ranges(state);

    let mut value = state.music_seek_display_ratio().clamp(0.0, 1.0);
    let response = ui.interact(
        rect,
        ui.make_persistent_id("music-painted-seek-bar"),
        egui::Sense::click_and_drag(),
    );
    if response.hovered() && !seek_ranges.is_empty() {
        response.clone().on_hover_text(seek_range_hover_text(
            &seek_ranges,
            duration_seconds.unwrap_or_default(),
        ));
    }

    if (response.clicked() || response.dragged())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        value = music_seek_ratio_for_pointer(rect, pointer.x);
        state.set_music_seek_drag_ratio(Some(value));
        ui.ctx().request_repaint();
    }

    if response.has_focus() {
        let keyboard_delta = ui.input(|input| {
            if input.key_pressed(egui::Key::ArrowLeft) {
                -0.01
            } else if input.key_pressed(egui::Key::ArrowRight) {
                0.01
            } else {
                0.0
            }
        });
        if keyboard_delta != 0.0 {
            value = (value + keyboard_delta).clamp(0.0, 1.0);
            state.set_music_seek_drag_ratio(Some(value));
            state.finish_music_seek_drag(value);
        }
    }

    let pointer_down = ui.input(|input| input.pointer.primary_down());
    if state.music_seek_drag_ratio().is_some() && !pointer_down {
        let final_ratio = state
            .music_seek_drag_ratio()
            .unwrap_or(value)
            .clamp(0.0, 1.0);
        state.finish_music_seek_drag(final_ratio);
    }

    let track_rect = egui::Rect::from_center_size(
        rect.center(),
        egui::vec2(rect.width(), if response.hovered() { 5.0 } else { 4.0 }),
    );
    let playable_range_ratios = seek_range_ratios(
        &seek_ranges,
        duration_seconds.unwrap_or_default(),
        MusicSeekRangeKind::Playable,
    );
    let played_fill_ratios = played_seek_fill_ratios(value, playable_range_ratios);
    let accent = accent_blue_for_ui(ui);
    let track_color = ui.visuals().widgets.inactive.bg_fill;
    let knob_radius = if response.dragged() {
        6.0
    } else if response.hovered() {
        5.5
    } else {
        4.5
    };

    ui.painter()
        .rect_filled(track_rect, track_rect.height() * 0.5, track_color);
    render_music_seek_range(
        ui,
        track_rect,
        &seek_ranges,
        duration_seconds.unwrap_or_default(),
        MusicSeekRangeKind::Playable,
    );
    if let Some((fill_start_ratio, fill_end_ratio)) = played_fill_ratios {
        let fill_rect = egui::Rect::from_min_max(
            egui::pos2(
                track_rect.left() + track_rect.width() * fill_start_ratio,
                track_rect.top(),
            ),
            egui::pos2(
                track_rect.left() + track_rect.width() * fill_end_ratio,
                track_rect.bottom(),
            ),
        );
        ui.painter()
            .rect_filled(fill_rect, fill_rect.height() * 0.5, accent);
    }
    render_music_seek_range(
        ui,
        track_rect,
        &seek_ranges,
        duration_seconds.unwrap_or_default(),
        MusicSeekRangeKind::Mix,
    );
    let knob_x = track_rect.left() + track_rect.width() * value;
    ui.painter().circle_filled(
        egui::pos2(knob_x, track_rect.center().y),
        knob_radius,
        accent,
    );
}

fn current_music_seek_ranges(state: &AppState) -> Vec<MusicSeekRange> {
    let mut ranges = Vec::with_capacity(2);
    if (state.music_automix_enabled()
        || state.music_chorus_flow_enabled()
        || state.music_trim_enabled())
        && let Some((start_seconds, end_seconds)) =
            state.music_chorus_current_display_highlight_range()
    {
        ranges.push(MusicSeekRange {
            kind: MusicSeekRangeKind::Playable,
            start_seconds,
            end_seconds,
        });
    }
    if let Some((start_seconds, end_seconds)) = state.music_current_mix_window_seconds() {
        ranges.push(MusicSeekRange {
            kind: MusicSeekRangeKind::Mix,
            start_seconds,
            end_seconds,
        });
    }
    ranges
}

fn render_music_seek_range(
    ui: &Ui,
    track_rect: egui::Rect,
    ranges: &[MusicSeekRange],
    duration_seconds: f64,
    kind: MusicSeekRangeKind,
) {
    let Some((start_ratio, end_ratio)) = seek_range_ratios(ranges, duration_seconds, kind) else {
        return;
    };
    let left = track_rect.left() + track_rect.width() * start_ratio;
    let right = track_rect.left() + track_rect.width() * end_ratio;
    let color = music_seek_range_color(ui, kind);
    let band = egui::Rect::from_min_max(
        egui::pos2(left, track_rect.top()),
        egui::pos2(right.max(left + 1.0), track_rect.bottom()),
    );
    ui.painter()
        .rect_filled(band, track_rect.height() * 0.5, color);
    for x in [left, right] {
        ui.painter().line_segment(
            [
                egui::pos2(x, track_rect.top() - 2.0),
                egui::pos2(x, track_rect.bottom() + 2.0),
            ],
            Stroke::new(1.0, color),
        );
    }
}

fn seek_range_ratios(
    ranges: &[MusicSeekRange],
    duration_seconds: f64,
    kind: MusicSeekRangeKind,
) -> Option<(f32, f32)> {
    let range = ranges.iter().find(|range| range.kind == kind)?;
    normalized_seek_range_ratios(range.start_seconds, range.end_seconds, duration_seconds)
}

fn played_seek_fill_ratios(
    playback_ratio: f32,
    playable_range: Option<(f32, f32)>,
) -> Option<(f32, f32)> {
    let (start, limit) = playable_range.unwrap_or((0.0, 1.0));
    let start = start.clamp(0.0, 1.0);
    let end = playback_ratio.clamp(start, limit.clamp(start, 1.0));
    (end > start).then_some((start, end))
}

fn normalized_seek_range_ratios(
    start_seconds: f64,
    end_seconds: f64,
    duration_seconds: f64,
) -> Option<(f32, f32)> {
    if !duration_seconds.is_finite()
        || duration_seconds <= 0.0
        || !start_seconds.is_finite()
        || !end_seconds.is_finite()
        || end_seconds <= start_seconds
    {
        return None;
    }
    let start = (start_seconds / duration_seconds).clamp(0.0, 1.0) as f32;
    let end = (end_seconds / duration_seconds).clamp(0.0, 1.0) as f32;
    (end > start).then_some((start, end))
}

fn music_seek_range_color(ui: &Ui, kind: MusicSeekRangeKind) -> egui::Color32 {
    match kind {
        MusicSeekRangeKind::Playable => {
            let accent = accent_blue_for_ui(ui);
            egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 84)
        }
        MusicSeekRangeKind::Mix => egui::Color32::from_rgb(242, 168, 58),
    }
}

fn seek_range_hover_text(ranges: &[MusicSeekRange], duration_seconds: f64) -> String {
    ranges
        .iter()
        .filter(|range| {
            normalized_seek_range_ratios(range.start_seconds, range.end_seconds, duration_seconds)
                .is_some()
        })
        .map(|range| {
            format!(
                "{}  {} – {}",
                music_seek_range_label(range.kind),
                format_marker_time(range.start_seconds),
                format_marker_time(range.end_seconds)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn music_seek_range_label(kind: MusicSeekRangeKind) -> &'static str {
    match kind {
        MusicSeekRangeKind::Playable => "Playback range",
        MusicSeekRangeKind::Mix => "Mix range",
    }
}

fn format_marker_time(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

fn music_seek_ratio_for_pointer(rect: egui::Rect, pointer_x: f32) -> f32 {
    if rect.width() <= 0.0 {
        return 0.0;
    }
    ((pointer_x - rect.left()) / rect.width()).clamp(0.0, 1.0)
}

#[cfg(test)]
mod painted_seek_tests {
    use super::{
        MusicAnalysisPearlEnvelope, music_analysis_pearl_targets, music_seek_ratio_for_pointer,
        normalized_seek_range_ratios, played_seek_fill_ratios,
    };
    use crate::app::state::{MusicPlayerAuraDisplay, MusicPlayerAuraTrackField};
    use eframe::egui;

    #[test]
    fn painted_seek_pointer_ratio_clamps_to_track() {
        let rect = egui::Rect::from_min_max(egui::pos2(10.0, 0.0), egui::pos2(110.0, 20.0));

        assert_eq!(music_seek_ratio_for_pointer(rect, -5.0), 0.0);
        assert_eq!(music_seek_ratio_for_pointer(rect, 60.0), 0.5);
        assert_eq!(music_seek_ratio_for_pointer(rect, 140.0), 1.0);
    }

    #[test]
    fn seek_range_ratios_use_full_song_duration_and_reject_invalid_ranges() {
        assert_eq!(
            normalized_seek_range_ratios(30.0, 90.0, 120.0),
            Some((0.25, 0.75))
        );
        assert_eq!(normalized_seek_range_ratios(90.0, 30.0, 120.0), None);
        assert_eq!(normalized_seek_range_ratios(30.0, 90.0, 0.0), None);
    }

    #[test]
    fn played_fill_starts_at_local_range_start_or_song_start() {
        assert_eq!(played_seek_fill_ratios(0.40, None), Some((0.0, 0.40)));
        assert_eq!(
            played_seek_fill_ratios(0.40, Some((0.25, 0.75))),
            Some((0.25, 0.40))
        );
        assert_eq!(played_seek_fill_ratios(0.20, Some((0.25, 0.75))), None);
        assert_eq!(
            played_seek_fill_ratios(0.90, Some((0.25, 0.75))),
            Some((0.25, 0.75))
        );
    }

    #[test]
    fn analysis_pearl_targets_keep_four_frequency_pairs_distinct() {
        let mut field = MusicPlayerAuraTrackField::default();
        field.spectrum_bands = [1.0, 0.2, 0.7, 0.1, 0.45, 0.05, 0.18, 0.02];
        let targets = music_analysis_pearl_targets(MusicPlayerAuraDisplay {
            primary: Some(field),
            ..Default::default()
        });

        assert!(targets[0] > targets[1]);
        assert!(targets[1] > targets[2]);
        assert!(targets[2] > targets[3]);
        assert!(targets.iter().all(|value| (0.0..=1.0).contains(value)));
    }

    #[test]
    fn analysis_pearl_targets_use_equal_power_mix_ownership() {
        let mut primary = MusicPlayerAuraTrackField::default();
        primary.spectrum_bands[0] = 1.0;
        let mut secondary = MusicPlayerAuraTrackField::default();
        secondary.spectrum_bands[0] = 1.0;
        let targets = music_analysis_pearl_targets(MusicPlayerAuraDisplay {
            primary: Some(primary),
            secondary: Some(secondary),
            mix_progress: 0.5,
            ..Default::default()
        });

        assert!((targets[0] - 1.0).abs() < 0.000_001);
    }

    #[test]
    fn analysis_pearl_motion_returns_to_center_and_reacts_in_both_directions() {
        let mut envelope = MusicAnalysisPearlEnvelope::default();
        assert_eq!(envelope.signed_motion(0), 0.0);

        envelope.advance([1.0, 0.0, 0.0, 0.0], 1.0 / 60.0);
        assert!(envelope.signed_motion(0) > 0.0);

        for _ in 0..300 {
            envelope.advance([1.0, 0.0, 0.0, 0.0], 1.0 / 60.0);
        }
        envelope.advance([0.0; 4], 1.0 / 60.0);
        assert!(envelope.signed_motion(0) < 0.0);

        for _ in 0..300 {
            envelope.advance([0.0; 4], 1.0 / 60.0);
        }
        assert!(envelope.signed_motion(0).abs() < 0.001);
    }

    #[test]
    fn music_control_metrics_keep_a_compact_twenty_eight_pixel_minimum() {
        assert_eq!(
            super::semantic_ui_metrics::main_music_player_control_row_height(22.0),
            28.0
        );
        assert_eq!(
            super::semantic_ui_metrics::main_music_player_control_row_height(32.0),
            32.0
        );
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(28.0, 28.0));
        assert!(
            (super::semantic_ui_metrics::main_music_playback_icon_size_for_rect(rect) - 17.92)
                .abs()
                < 0.001
        );
    }
}
