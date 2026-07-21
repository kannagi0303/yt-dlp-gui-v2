use crate::app::state::AppState;
use crate::domain::{ChapterOption, DownloadTimeRange, format_download_range_timestamp};
use eframe::egui::{self, Align2, Color32, FontId, Sense, Stroke, Ui};
use egui_extras::{Column, TableBuilder};

use super::semantic_ui_metrics;

pub(super) fn render_time_range_picker_contents(ui: &mut Ui, state: &mut AppState) {
    let Some(duration_millis) = state.current_download_range_duration_millis() else {
        ui.add_space(semantic_ui_metrics::format_picker_empty_message_top_vertical_spacing());
        ui.label(state.ui_i18n_text_for_key("picker.section_time_unavailable"));
        return;
    };

    render_time_range_timeline(ui, state, duration_millis);
    render_time_range_actions(ui, state);
    render_saved_time_ranges(ui, state);
}

fn render_time_range_timeline(ui: &mut Ui, state: &mut AppState, duration_millis: u64) {
    let desired_size = egui::vec2(
        ui.available_width(),
        semantic_ui_metrics::format_picker_time_range_timeline_height(),
    );
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
    let track_rect = rect.shrink2(egui::vec2(
        semantic_ui_metrics::format_picker_time_range_timeline_horizontal_inset(),
        semantic_ui_metrics::format_picker_time_range_timeline_vertical_inset(),
    ));
    let chapters = state.current_download_range_chapters();
    let chapter_boundaries = chapter_boundary_millis(&chapters, duration_millis);

    if let Some(pointer) = response
        .interact_pointer_pos()
        .filter(|_| response.clicked() || response.dragged())
    {
        let ratio = ((pointer.x - track_rect.left()) / track_rect.width().max(1.0)).clamp(0.0, 1.0);
        let raw_millis = (duration_millis as f64 * ratio as f64).round() as u64;
        state.set_pending_download_range_playhead(snapped_timeline_millis(
            raw_millis,
            duration_millis,
            track_rect.width(),
            &chapter_boundaries,
        ));
    }

    let track_y = track_rect.center().y;
    let visuals = ui.visuals().clone();
    let accent = visuals.selection.bg_fill;
    let painter = ui.painter().clone();
    painter.line_segment(
        [
            egui::pos2(track_rect.left(), track_y),
            egui::pos2(track_rect.right(), track_y),
        ],
        Stroke::new(
            semantic_ui_metrics::format_picker_time_range_track_stroke_width(),
            visuals.widgets.inactive.bg_fill,
        ),
    );

    paint_selected_ranges(
        &painter,
        state,
        track_rect,
        duration_millis,
        &chapters,
        accent,
    );
    paint_pending_range(&painter, state, track_rect, duration_millis, accent);
    paint_chapter_markers(ui, state, track_rect, duration_millis, &chapters);
    paint_playhead(&painter, state, track_rect, duration_millis, accent);
    paint_timeline_timestamps(&painter, state, track_rect, duration_millis, &visuals);
}

fn paint_selected_ranges(
    painter: &egui::Painter,
    state: &AppState,
    track_rect: egui::Rect,
    duration_millis: u64,
    chapters: &[ChapterOption],
    accent: Color32,
) {
    for (start_index, end_index) in state
        .format_picker
        .download_range_draft
        .selection
        .grouped_selected_chapter_spans(chapters.len())
    {
        let start = chapters[start_index].start_millis;
        let end = chapters[end_index].end_millis.unwrap_or(duration_millis);
        if let Some(range) = DownloadTimeRange::new(start, end) {
            paint_range_segment(
                painter,
                track_rect,
                duration_millis,
                &range,
                accent.gamma_multiply(0.28),
            );
        }
    }

    for range in state
        .format_picker
        .download_range_draft
        .selection
        .custom_time_ranges()
    {
        paint_range_segment(
            painter,
            track_rect,
            duration_millis,
            range,
            accent.gamma_multiply(0.45),
        );
    }
}

fn paint_pending_range(
    painter: &egui::Painter,
    state: &AppState,
    track_rect: egui::Rect,
    duration_millis: u64,
    accent: Color32,
) {
    let draft = &state.format_picker.download_range_draft;
    if let (Some(start), Some(end)) = (draft.start_marker_millis, draft.end_marker_millis) {
        if let Some(range) = DownloadTimeRange::new(start, end) {
            paint_range_segment(painter, track_rect, duration_millis, &range, accent);
        }
    }
    if let Some(start) = draft.start_marker_millis {
        paint_boundary_notch(painter, track_rect, duration_millis, start, accent);
    }
    if let Some(end) = draft.end_marker_millis {
        paint_boundary_notch(painter, track_rect, duration_millis, end, accent);
    }
}

fn paint_chapter_markers(
    ui: &mut Ui,
    state: &mut AppState,
    track_rect: egui::Rect,
    duration_millis: u64,
    chapters: &[ChapterOption],
) {
    let track_y = track_rect.center().y;
    let visuals = ui.visuals();
    let marker_radius = semantic_ui_metrics::format_picker_time_range_chapter_marker_radius();
    for (index, chapter) in chapters.iter().enumerate() {
        let marker_x = timeline_x(track_rect, duration_millis, chapter.start_millis);
        let marker_center = egui::pos2(marker_x, track_y);
        ui.painter()
            .circle_filled(marker_center, marker_radius, visuals.weak_text_color());
        let marker_rect = egui::Rect::from_center_size(
            marker_center,
            egui::Vec2::splat(
                marker_radius
                    * semantic_ui_metrics::format_picker_time_range_marker_hit_radius_scale(),
            ),
        );
        let marker_response = ui.interact(
            marker_rect,
            ui.id().with(("download-range-chapter-marker", index)),
            Sense::click(),
        );
        if marker_response.clicked() {
            state.set_pending_download_range_playhead(chapter.start_millis);
        }
        marker_response.on_hover_text(state.localized_chapter_label(chapter));

        if chapters.len() <= 24 {
            ui.painter().text(
                egui::pos2(marker_x, track_rect.top()),
                Align2::CENTER_TOP,
                (index + 1).to_string(),
                FontId::proportional(
                    semantic_ui_metrics::format_picker_time_range_marker_label_font_size(),
                ),
                visuals.weak_text_color(),
            );
        }
    }
}

fn paint_playhead(
    painter: &egui::Painter,
    state: &AppState,
    track_rect: egui::Rect,
    duration_millis: u64,
    accent: Color32,
) {
    let playhead_x = timeline_x(
        track_rect,
        duration_millis,
        state.format_picker.download_range_draft.playhead_millis,
    );
    painter.circle_filled(
        egui::pos2(playhead_x, track_rect.center().y),
        semantic_ui_metrics::format_picker_time_range_playhead_radius(),
        accent,
    );
}

fn paint_timeline_timestamps(
    painter: &egui::Painter,
    state: &AppState,
    track_rect: egui::Rect,
    duration_millis: u64,
    visuals: &egui::Visuals,
) {
    let font =
        FontId::proportional(semantic_ui_metrics::format_picker_time_range_timestamp_font_size());
    painter.text(
        egui::pos2(track_rect.left(), track_rect.bottom()),
        Align2::LEFT_BOTTOM,
        format_download_range_timestamp(0),
        font.clone(),
        visuals.weak_text_color(),
    );
    painter.text(
        egui::pos2(track_rect.center().x, track_rect.bottom()),
        Align2::CENTER_BOTTOM,
        format_download_range_timestamp(state.format_picker.download_range_draft.playhead_millis),
        font.clone(),
        visuals.text_color(),
    );
    painter.text(
        egui::pos2(track_rect.right(), track_rect.bottom()),
        Align2::RIGHT_BOTTOM,
        format_download_range_timestamp(duration_millis),
        font,
        visuals.weak_text_color(),
    );
}

fn paint_range_segment(
    painter: &egui::Painter,
    track_rect: egui::Rect,
    duration_millis: u64,
    range: &DownloadTimeRange,
    color: Color32,
) {
    let left = timeline_x(track_rect, duration_millis, range.start_millis());
    let right = timeline_x(track_rect, duration_millis, range.end_millis());
    let half_height = semantic_ui_metrics::format_picker_time_range_segment_half_height();
    let range_rect = egui::Rect::from_min_max(
        egui::pos2(left, track_rect.center().y - half_height),
        egui::pos2(right, track_rect.center().y + half_height),
    );
    painter.rect_filled(
        range_rect,
        semantic_ui_metrics::format_picker_time_range_segment_corner_radius(),
        color,
    );
}

fn paint_boundary_notch(
    painter: &egui::Painter,
    track_rect: egui::Rect,
    duration_millis: u64,
    millis: u64,
    color: Color32,
) {
    let x = timeline_x(track_rect, duration_millis, millis);
    let half_height = semantic_ui_metrics::format_picker_time_range_boundary_notch_half_height();
    painter.line_segment(
        [
            egui::pos2(x, track_rect.center().y - half_height),
            egui::pos2(x, track_rect.center().y + half_height),
        ],
        Stroke::new(
            semantic_ui_metrics::format_picker_time_range_playhead_stroke_width(),
            color,
        ),
    );
}

fn timeline_x(rect: egui::Rect, duration_millis: u64, millis: u64) -> f32 {
    let ratio = millis.min(duration_millis) as f64 / duration_millis.max(1) as f64;
    rect.left() + rect.width() * ratio as f32
}

fn chapter_boundary_millis(chapters: &[ChapterOption], duration_millis: u64) -> Vec<u64> {
    let mut boundaries = vec![0, duration_millis];
    for chapter in chapters {
        boundaries.push(chapter.start_millis.min(duration_millis));
        if let Some(end) = chapter.end_millis {
            boundaries.push(end.min(duration_millis));
        }
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    boundaries
}

fn snapped_timeline_millis(
    raw_millis: u64,
    duration_millis: u64,
    track_width: f32,
    boundaries: &[u64],
) -> u64 {
    if duration_millis == 0 || track_width <= 0.0 {
        return raw_millis.min(duration_millis);
    }
    let snap_window_millis = (duration_millis as f64
        * semantic_ui_metrics::format_picker_time_range_snap_distance_pixels() as f64
        / track_width as f64)
        .round() as u64;

    boundaries
        .iter()
        .copied()
        .min_by_key(|boundary| raw_millis.abs_diff(*boundary))
        .filter(|boundary| raw_millis.abs_diff(*boundary) <= snap_window_millis)
        .unwrap_or(raw_millis)
        .min(duration_millis)
}

fn render_time_range_actions(ui: &mut Ui, state: &mut AppState) {
    let draft = &state.format_picker.download_range_draft;
    let can_set_end = draft
        .start_marker_millis
        .is_some_and(|start| draft.playhead_millis > start);
    let can_add = matches!(
        (draft.start_marker_millis, draft.end_marker_millis),
        (Some(start), Some(end)) if end > start
    );

    ui.horizontal_wrapped(|ui| {
        if ui
            .button(state.ui_i18n_text_for_key("picker.section_set_start"))
            .clicked()
        {
            state.set_pending_download_range_start_marker();
        }
        if ui
            .add_enabled(
                can_set_end,
                egui::Button::new(state.ui_i18n_text_for_key("picker.section_set_end")),
            )
            .clicked()
        {
            state.set_pending_download_range_end_marker();
        }
        if ui
            .add_enabled(
                can_add,
                egui::Button::new(state.ui_i18n_text_for_key("picker.section_add_range")),
            )
            .clicked()
        {
            state.add_pending_custom_download_range();
        }
    });
}

fn render_saved_time_ranges(ui: &mut Ui, state: &mut AppState) {
    let ranges = state
        .format_picker
        .download_range_draft
        .selection
        .custom_time_ranges()
        .to_vec();
    if ranges.is_empty() {
        return;
    }

    ui.separator();
    let row_height = semantic_ui_metrics::format_picker_section_row_height();
    TableBuilder::new(ui)
        .id_salt("download-range-custom-time-table")
        .striped(true)
        .column(Column::exact(
            semantic_ui_metrics::format_picker_section_table_marker_column_width(),
        ))
        .column(Column::remainder())
        .body(|body| {
            body.rows(row_height, ranges.len(), |mut row| {
                let index = row.index();
                let range = ranges[index];
                row.col(|ui| {
                    if ui.small_button("×").clicked() {
                        state.remove_pending_custom_download_range(index);
                    }
                });
                row.col(|ui| {
                    ui.label(format!(
                        "{}–{}",
                        format_download_range_timestamp(range.start_millis()),
                        format_download_range_timestamp(range.end_millis())
                    ));
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playhead_snaps_to_nearby_chapter_boundary() {
        assert_eq!(
            snapped_timeline_millis(19_600, 100_000, 800.0, &[0, 20_000, 50_000, 100_000]),
            20_000
        );
    }

    #[test]
    fn playhead_keeps_free_position_outside_snap_window() {
        assert_eq!(
            snapped_timeline_millis(17_000, 100_000, 800.0, &[0, 20_000, 50_000, 100_000]),
            17_000
        );
    }
}
