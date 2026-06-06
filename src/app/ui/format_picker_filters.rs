use crate::app::state::{AppState, FormatPickerFilters, FormatPickerKind, FormatPickerViewMode};
use crate::domain::FormatOption;
use eframe::egui::{self, Align, Layout, ScrollArea, Ui};

use super::common::UiText;
use super::semantic_ui_metrics;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterField {
    Resolution,
    DynamicRange,
    Fps,
    Codec,
    SampleRate,
}

struct FilterChainStage {
    label: String,
    field: FilterField,
    values: Vec<String>,
    compatible_values: Vec<String>,
    selected: Option<String>,
}

struct FilterNodeRect {
    value: String,
    rect: egui::Rect,
    selected: bool,
}

pub(super) fn sync_picker_mode(state: &mut AppState) {
    match state.format_picker.view_mode {
        FormatPickerViewMode::Filter => {
            if let Some(selected_row) = state.format_picker.selected_row {
                let kind = match state.format_picker.kind {
                    Some(kind) => kind,
                    None => return,
                };
                let options = state.format_picker_options(kind);
                if let Some(option) = options.get(selected_row) {
                    apply_filters_from_option(&mut state.format_picker.filters, kind, option);
                }
            }
        }
        FormatPickerViewMode::Table => {
            let kind = match state.format_picker.kind {
                Some(kind) => kind,
                None => return,
            };
            let options = state.format_picker_options(kind);
            let filtered = filtered_rows(&options, &state.format_picker.filters);
            if filtered.len() == 1 {
                state.format_picker.selected_row = filtered.first().copied();
            }
        }
    }
}

pub(super) fn apply_filters_from_option(
    filters: &mut FormatPickerFilters,
    kind: FormatPickerKind,
    option: &FormatOption,
) {
    filters.clear();
    match kind {
        FormatPickerKind::Video => {
            if !option.resolution.is_empty() {
                filters.resolution = Some(option.resolution.clone());
            }
            if !option.dynamic_range.is_empty() {
                filters.dynamic_range = Some(option.dynamic_range.clone());
            }
            if !option.fps.is_empty() {
                filters.fps = Some(option.fps.clone());
            }
            if !option.codec.is_empty() {
                filters.codec = Some(option.codec.clone());
            }
        }
        FormatPickerKind::Audio => {
            if !option.sample_rate.is_empty() {
                filters.sample_rate = Some(option.sample_rate.clone());
            }
            if !option.codec.is_empty() {
                filters.codec = Some(option.codec.clone());
            }
        }
        FormatPickerKind::Subtitle | FormatPickerKind::Section => {}
    }
}

pub(super) fn filtered_rows(options: &[FormatOption], filters: &FormatPickerFilters) -> Vec<usize> {
    options
        .iter()
        .enumerate()
        .filter_map(|(index, option)| option_matches_filters(option, filters).then_some(index))
        .collect()
}

fn option_matches_filters(option: &FormatOption, filters: &FormatPickerFilters) -> bool {
    if let Some(resolution) = &filters.resolution {
        if &option.resolution != resolution {
            return false;
        }
    }
    if let Some(dynamic_range) = &filters.dynamic_range {
        if &option.dynamic_range != dynamic_range {
            return false;
        }
    }
    if let Some(fps) = &filters.fps {
        if &option.fps != fps {
            return false;
        }
    }
    if let Some(codec) = &filters.codec {
        if &option.codec != codec {
            return false;
        }
    }
    if let Some(sample_rate) = &filters.sample_rate {
        if &option.sample_rate != sample_rate {
            return false;
        }
    }

    true
}

pub(super) fn render_format_picker_filters(
    ui: &mut Ui,
    state: &mut AppState,
    kind: FormatPickerKind,
    options: &[FormatOption],
) {
    let filters_snapshot = state.format_picker.filters.clone();
    let stages = match kind {
        FormatPickerKind::Video => vec![
            FilterChainStage {
                label: state
                    .ui_i18n_text_for_key(UiText::FILTER_RESOLUTION)
                    .to_owned(),
                field: FilterField::Resolution,
                values: all_resolution_values(options),
                compatible_values: compatible_resolution_values(options, &filters_snapshot),
                selected: filters_snapshot.resolution.clone(),
            },
            FilterChainStage {
                label: state.ui_i18n_text_for_key(UiText::FILTER_FPS).to_owned(),
                field: FilterField::Fps,
                values: all_fps_values(options),
                compatible_values: compatible_fps_values(options, &filters_snapshot),
                selected: filters_snapshot.fps.clone(),
            },
            FilterChainStage {
                label: state.ui_i18n_text_for_key(UiText::FILTER_CODEC).to_owned(),
                field: FilterField::Codec,
                values: all_codec_values(options),
                compatible_values: compatible_codec_values(options, &filters_snapshot),
                selected: filters_snapshot.codec.clone(),
            },
            FilterChainStage {
                label: state.ui_i18n_text_for_key(UiText::FILTER_RANGE).to_owned(),
                field: FilterField::DynamicRange,
                values: all_range_values(options),
                compatible_values: compatible_range_values(options, &filters_snapshot),
                selected: filters_snapshot.dynamic_range.clone(),
            },
        ],
        FormatPickerKind::Audio => vec![
            FilterChainStage {
                label: state
                    .ui_i18n_text_for_key(UiText::FILTER_SAMPLE_RATE)
                    .to_owned(),
                field: FilterField::SampleRate,
                values: all_sample_rate_values(options),
                compatible_values: compatible_sample_rate_values(options, &filters_snapshot),
                selected: filters_snapshot.sample_rate.clone(),
            },
            FilterChainStage {
                label: state.ui_i18n_text_for_key(UiText::FILTER_CODEC).to_owned(),
                field: FilterField::Codec,
                values: all_codec_values(options),
                compatible_values: compatible_codec_values(options, &filters_snapshot),
                selected: filters_snapshot.codec.clone(),
            },
        ],
        FormatPickerKind::Subtitle | FormatPickerKind::Section => Vec::new(),
    };

    render_filter_chain(
        ui,
        stages,
        kind,
        options,
        &mut state.format_picker.filters,
        &mut state.format_picker.selected_row,
    );
}

fn available_filter_values(
    options: &[FormatOption],
    filters: &FormatPickerFilters,
    value_fn: impl Fn(&FormatOption) -> String,
) -> Vec<String> {
    let mut values = Vec::new();
    for option in options {
        if !option_matches_filters(option, filters) {
            continue;
        }
        let value = value_fn(option);
        if value.is_empty() || values.iter().any(|existing| existing == &value) {
            continue;
        }
        values.push(value);
    }
    values
}

fn distinct_values(
    options: &[FormatOption],
    value_fn: impl Fn(&FormatOption) -> String,
) -> Vec<String> {
    let mut values = Vec::new();
    for option in options {
        let value = value_fn(option);
        if value.is_empty() || values.iter().any(|existing| existing == &value) {
            continue;
        }
        values.push(value);
    }
    values
}

fn all_resolution_values(options: &[FormatOption]) -> Vec<String> {
    distinct_values(options, |option| option.resolution.clone())
}

fn compatible_resolution_values(
    options: &[FormatOption],
    filters: &FormatPickerFilters,
) -> Vec<String> {
    let mut filters = filters.clone();
    filters.resolution = None;
    available_filter_values(options, &filters, |option| option.resolution.clone())
}

fn all_range_values(options: &[FormatOption]) -> Vec<String> {
    distinct_values(options, |option| option.dynamic_range.clone())
}

fn compatible_range_values(options: &[FormatOption], filters: &FormatPickerFilters) -> Vec<String> {
    let mut filters = filters.clone();
    filters.dynamic_range = None;
    available_filter_values(options, &filters, |option| option.dynamic_range.clone())
}

fn all_fps_values(options: &[FormatOption]) -> Vec<String> {
    distinct_values(options, |option| option.fps.clone())
}

fn compatible_fps_values(options: &[FormatOption], filters: &FormatPickerFilters) -> Vec<String> {
    let mut filters = filters.clone();
    filters.fps = None;
    available_filter_values(options, &filters, |option| option.fps.clone())
}

fn all_codec_values(options: &[FormatOption]) -> Vec<String> {
    distinct_values(options, |option| option.codec.clone())
}

fn compatible_codec_values(options: &[FormatOption], filters: &FormatPickerFilters) -> Vec<String> {
    let mut filters = filters.clone();
    filters.codec = None;
    available_filter_values(options, &filters, |option| option.codec.clone())
}

fn all_sample_rate_values(options: &[FormatOption]) -> Vec<String> {
    distinct_values(options, |option| option.sample_rate.clone())
}

fn compatible_sample_rate_values(
    options: &[FormatOption],
    filters: &FormatPickerFilters,
) -> Vec<String> {
    let mut filters = filters.clone();
    filters.sample_rate = None;
    available_filter_values(options, &filters, |option| option.sample_rate.clone())
}

fn render_filter_chain(
    ui: &mut Ui,
    stages: Vec<FilterChainStage>,
    kind: FormatPickerKind,
    options: &[FormatOption],
    filters: &mut FormatPickerFilters,
    selected_row: &mut Option<usize>,
) {
    let stages: Vec<FilterChainStage> = stages
        .into_iter()
        .filter(|stage| !stage.values.is_empty())
        .collect();
    if stages.is_empty() {
        return;
    }

    let button_padding = semantic_ui_metrics::format_picker_filter_button_padding();
    let node_height =
        semantic_ui_metrics::format_picker_filter_node_height_from_current_control_metrics(ui);
    let stage_widths = measure_filter_stage_widths(ui, &stages, button_padding.x);
    let stage_count = stages.len();
    let viewport_width =
        semantic_ui_metrics::format_picker_filter_viewport_width_from_available_width(
            ui.available_width(),
        );
    let slot_width =
        semantic_ui_metrics::format_picker_filter_slot_width_for_viewport_width_and_stage_count(
            viewport_width,
            stage_count,
        );
    let content_width = viewport_width;

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(content_width);
            ui.set_min_height(ui.available_height());

            let mut columns: Vec<Vec<FilterNodeRect>> = Vec::with_capacity(stages.len());

            ui.horizontal_top(|ui| {
                for (stage_index, stage) in stages.iter().enumerate() {
                    let stage_width =
                        semantic_ui_metrics::format_picker_filter_stage_width_for_slot_width(
                            stage_widths[stage_index],
                            slot_width,
                        );
                    let mut node_rects = Vec::with_capacity(stage.values.len());

                    ui.allocate_ui_with_layout(
                        egui::vec2(slot_width, ui.available_height()),
                        Layout::top_down(Align::Center),
                        |ui| {
                            ui.set_width(slot_width);
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(stage.label.as_str()).strong(),
                                )
                                .selectable(false),
                            );
                            ui.add_space(
                                semantic_ui_metrics::format_picker_filter_stage_title_to_nodes_vertical_spacing(),
                            );

                            ui.scope(|ui| {
                                ui.spacing_mut().button_padding = button_padding;
                                for value in &stage.values {
                                    let is_selected =
                                        stage.selected.as_deref() == Some(value.as_str());
                                    let is_enabled = is_selected
                                        || stage
                                            .compatible_values
                                            .iter()
                                            .any(|compatible| compatible == value);
                                    let mut button = egui::Button::new(value.as_str())
                                        .frame(true)
                                        .min_size(egui::vec2(stage_width, node_height));
                                    if !is_enabled && !is_selected {
                                        button = button.fill(incompatible_filter_button_fill(ui));
                                    }

                                    let response = ui.add_sized(
                                        egui::vec2(stage_width, node_height),
                                        button.selected(is_selected),
                                    );

                                    if response.clicked() {
                                        if is_selected {
                                            set_filter_value(filters, stage.field, None);
                                            *selected_row =
                                                selected_row_for_filters(options, filters);
                                        } else {
                                            *selected_row = force_pick_filter_value(
                                                filters,
                                                stage.field,
                                                value.clone(),
                                                kind,
                                                options,
                                            );
                                        }
                                    }

                                    node_rects.push(FilterNodeRect {
                                        value: value.clone(),
                                        rect: response.rect,
                                        selected: is_selected,
                                    });

                                    ui.add_space(semantic_ui_metrics::format_picker_filter_node_vertical_spacing());
                                }
                            });
                        },
                    );
                    columns.push(node_rects);
                }
            });

            draw_filter_chain_connections(ui, &stages, &columns, options, *selected_row);
        });
}

fn draw_filter_chain_connections(
    ui: &Ui,
    stages: &[FilterChainStage],
    columns: &[Vec<FilterNodeRect>],
    options: &[FormatOption],
    selected_row: Option<usize>,
) {
    if stages.len() < 2 || columns.len() < 2 {
        return;
    }

    let active_color = filter_flow_active_line_color(ui);
    let painter = ui.painter();
    let selected_option = selected_row.and_then(|row| options.get(row));

    for pair_index in 0..(stages.len() - 1) {
        let left_stage = &stages[pair_index];
        let right_stage = &stages[pair_index + 1];
        let left_nodes = &columns[pair_index];
        let right_nodes = &columns[pair_index + 1];

        let Some(left_value) = selected_filter_value(selected_option, left_stage) else {
            continue;
        };
        let Some(right_value) = selected_filter_value(selected_option, right_stage) else {
            continue;
        };

        let Some(left_node) = left_nodes.iter().find(|node| node.value == left_value) else {
            continue;
        };
        let Some(right_node) = right_nodes.iter().find(|node| node.value == right_value) else {
            continue;
        };
        if !left_node.selected || !right_node.selected {
            continue;
        }

        let stroke = egui::Stroke::new(
            semantic_ui_metrics::format_picker_filter_connection_stroke_width(),
            active_color,
        );
        let start = egui::pos2(left_node.rect.right(), left_node.rect.center().y);
        let end = egui::pos2(right_node.rect.left(), right_node.rect.center().y);
        draw_curved_filter_connection(painter, start, end, stroke);
    }
}

fn measure_filter_stage_widths(
    ui: &Ui,
    stages: &[FilterChainStage],
    horizontal_padding: f32,
) -> Vec<f32> {
    stages
        .iter()
        .map(|stage| {
            semantic_ui_metrics::format_picker_filter_stage_node_width_for_visible_values(
                ui,
                stage.values.iter().map(|value| value.as_str()),
                horizontal_padding,
            )
        })
        .collect()
}

fn selected_filter_value(
    selected_option: Option<&FormatOption>,
    stage: &FilterChainStage,
) -> Option<String> {
    if let Some(option) = selected_option {
        let value = value_for_field(option, stage.field);
        if !value.is_empty() {
            return Some(value.to_owned());
        }
    }
    stage.selected.clone()
}

fn draw_curved_filter_connection(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    stroke: egui::Stroke,
) {
    let mid_x = (start.x + end.x) * 0.5;
    let dy = end.y - start.y;
    if dy.abs() < 1.0 {
        painter.line_segment([start, end], stroke);
        return;
    }

    let direction = dy.signum();
    let radius = dy.abs().mul_add(0.25, 0.0).clamp(8.0, 14.0);
    let left_corner_start = egui::pos2(mid_x - radius, start.y);
    let left_corner_end = egui::pos2(mid_x, start.y + direction * radius);
    let right_corner_start = egui::pos2(mid_x, end.y - direction * radius);
    let right_corner_end = egui::pos2(mid_x + radius, end.y);

    painter.line_segment([start, left_corner_start], stroke);
    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        [
            left_corner_start,
            egui::pos2(mid_x - radius * 0.45, start.y),
            egui::pos2(mid_x, start.y + direction * radius * 0.55),
            left_corner_end,
        ],
        false,
        egui::Color32::TRANSPARENT,
        stroke,
    ));
    painter.line_segment([left_corner_end, right_corner_start], stroke);
    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        [
            right_corner_start,
            egui::pos2(mid_x, end.y - direction * radius * 0.55),
            egui::pos2(mid_x + radius * 0.45, end.y),
            right_corner_end,
        ],
        false,
        egui::Color32::TRANSPARENT,
        stroke,
    ));
    painter.line_segment([right_corner_end, end], stroke);
}

fn filter_flow_active_line_color(ui: &Ui) -> egui::Color32 {
    ui.visuals().widgets.active.bg_fill
}

fn incompatible_filter_button_fill(ui: &Ui) -> egui::Color32 {
    if ui.visuals().dark_mode {
        egui::Color32::BLACK
    } else {
        ui.visuals().widgets.noninteractive.bg_fill
    }
}

fn set_filter_value(filters: &mut FormatPickerFilters, field: FilterField, value: Option<String>) {
    match field {
        FilterField::Resolution => filters.resolution = value,
        FilterField::DynamicRange => filters.dynamic_range = value,
        FilterField::Fps => filters.fps = value,
        FilterField::Codec => filters.codec = value,
        FilterField::SampleRate => filters.sample_rate = value,
    }
}

fn force_pick_filter_value(
    filters: &mut FormatPickerFilters,
    field: FilterField,
    value: String,
    kind: FormatPickerKind,
    options: &[FormatOption],
) -> Option<usize> {
    let previous_filters = filters.clone();
    set_filter_value(filters, field, Some(value.clone()));

    if !filtered_rows(options, filters).is_empty() {
        maybe_snap_filters_to_single(filters, kind, options);
        return selected_row_for_filters(options, filters);
    }

    if let Some((index, option)) =
        closest_option_for_forced_value(options, kind, field, &value, &previous_filters)
    {
        apply_filters_from_option(filters, kind, option);
        return Some(index);
    }

    filters.clear();
    set_filter_value(filters, field, Some(value));
    selected_row_for_filters(options, filters)
}

fn current_for_field(filters: &FormatPickerFilters, field: FilterField) -> Option<String> {
    match field {
        FilterField::Resolution => filters.resolution.clone(),
        FilterField::DynamicRange => filters.dynamic_range.clone(),
        FilterField::Fps => filters.fps.clone(),
        FilterField::Codec => filters.codec.clone(),
        FilterField::SampleRate => filters.sample_rate.clone(),
    }
}

fn selected_row_for_filters(
    options: &[FormatOption],
    filters: &FormatPickerFilters,
) -> Option<usize> {
    filtered_rows(options, filters).into_iter().next()
}

fn closest_option_for_forced_value<'a>(
    options: &'a [FormatOption],
    kind: FormatPickerKind,
    field: FilterField,
    value: &str,
    previous_filters: &FormatPickerFilters,
) -> Option<(usize, &'a FormatOption)> {
    let fields = filter_fields_for_kind(kind);
    options
        .iter()
        .enumerate()
        .filter(|(_, option)| value_for_field(option, field) == value)
        .max_by_key(|(_, option)| filter_proximity_score(option, field, fields, previous_filters))
}

fn filter_fields_for_kind(kind: FormatPickerKind) -> &'static [FilterField] {
    match kind {
        FormatPickerKind::Video => &[
            FilterField::Resolution,
            FilterField::Fps,
            FilterField::Codec,
            FilterField::DynamicRange,
        ],
        FormatPickerKind::Audio => &[FilterField::SampleRate, FilterField::Codec],
        FormatPickerKind::Subtitle | FormatPickerKind::Section => &[],
    }
}

fn filter_proximity_score(
    option: &FormatOption,
    forced_field: FilterField,
    fields: &[FilterField],
    previous_filters: &FormatPickerFilters,
) -> i64 {
    fields
        .iter()
        .filter(|&&field| field != forced_field)
        .map(|&field| {
            let Some(previous_value) = current_for_field(previous_filters, field) else {
                return 0;
            };
            score_field_value(field, &previous_value, value_for_field(option, field))
        })
        .sum()
}

fn score_field_value(field: FilterField, previous_value: &str, candidate_value: &str) -> i64 {
    if previous_value == candidate_value {
        return 10_000;
    }

    match field {
        FilterField::Resolution => numeric_closeness_score(
            resolution_area_text(previous_value),
            resolution_area_text(candidate_value),
        ),
        FilterField::Fps | FilterField::SampleRate => numeric_closeness_score(
            number_from_text(previous_value),
            number_from_text(candidate_value),
        ),
        FilterField::DynamicRange | FilterField::Codec => 0,
    }
}

fn numeric_closeness_score(previous: Option<i64>, candidate: Option<i64>) -> i64 {
    let (Some(previous), Some(candidate)) = (previous, candidate) else {
        return 0;
    };
    5_000 - previous.abs_diff(candidate) as i64
}

fn resolution_area_text(value: &str) -> Option<i64> {
    let (width, height) = value.split_once('x')?;
    let width = width.trim().parse::<i64>().ok()?;
    let height = height.trim().parse::<i64>().ok()?;
    Some(width.saturating_mul(height))
}

fn number_from_text(value: &str) -> Option<i64> {
    let digits: String = value.chars().filter(|ch| ch.is_ascii_digit()).collect();
    digits.parse::<i64>().ok()
}

fn value_for_field(option: &FormatOption, field: FilterField) -> &str {
    match field {
        FilterField::Resolution => &option.resolution,
        FilterField::DynamicRange => &option.dynamic_range,
        FilterField::Fps => &option.fps,
        FilterField::Codec => &option.codec,
        FilterField::SampleRate => &option.sample_rate,
    }
}

fn maybe_snap_filters_to_single(
    filters: &mut FormatPickerFilters,
    kind: FormatPickerKind,
    options: &[FormatOption],
) {
    let rows = filtered_rows(options, filters);
    if rows.len() == 1 {
        if let Some(option) = rows.first().and_then(|&index| options.get(index)) {
            apply_filters_from_option(filters, kind, option);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::MediaKind;

    #[test]
    fn forced_video_filter_picks_closest_existing_combination() {
        let options = vec![
            FormatOption::video(
                "low-av1",
                "low-av1",
                MediaKind::Video,
                "1280x720",
                "SDR",
                "30",
                "webm",
                "av1",
                "10.00 MB",
            ),
            FormatOption::video(
                "high-av1",
                "high-av1",
                MediaKind::Video,
                "1920x1080",
                "SDR",
                "60",
                "webm",
                "av1",
                "20.00 MB",
            ),
            FormatOption::video(
                "high-h264",
                "high-h264",
                MediaKind::Video,
                "1920x1080",
                "SDR",
                "60",
                "mp4",
                "h264",
                "30.00 MB",
            ),
        ];
        let mut filters = FormatPickerFilters {
            resolution: Some("1920x1080".to_owned()),
            dynamic_range: Some("SDR".to_owned()),
            fps: Some("60".to_owned()),
            ext: Some("mp4".to_owned()),
            codec: Some("h264".to_owned()),
            sample_rate: None,
        };

        let selected_row = force_pick_filter_value(
            &mut filters,
            FilterField::Codec,
            "av1".to_owned(),
            FormatPickerKind::Video,
            &options,
        );

        assert_eq!(selected_row, Some(1));
        assert_eq!(filters.codec.as_deref(), Some("av1"));
        assert_eq!(filters.resolution.as_deref(), Some("1920x1080"));
        assert_eq!(filters.fps.as_deref(), Some("60"));
        assert_eq!(filters.ext.as_deref(), None);
        assert_eq!(filtered_rows(&options, &filters), vec![1]);
    }

    #[test]
    fn forced_audio_filter_picks_closest_sample_rate() {
        let options = vec![
            FormatOption::audio(
                "opus-low",
                "opus-low",
                MediaKind::Audio,
                "44100",
                "webm",
                "opus",
                "3.00 MB",
            ),
            FormatOption::audio(
                "opus-high",
                "opus-high",
                MediaKind::Audio,
                "48000",
                "webm",
                "opus",
                "4.00 MB",
            ),
            FormatOption::audio(
                "aac",
                "aac",
                MediaKind::Audio,
                "48000",
                "m4a",
                "aac",
                "5.00 MB",
            ),
        ];
        let mut filters = FormatPickerFilters {
            resolution: None,
            dynamic_range: None,
            fps: None,
            ext: Some("m4a".to_owned()),
            codec: Some("aac".to_owned()),
            sample_rate: Some("48000".to_owned()),
        };

        let selected_row = force_pick_filter_value(
            &mut filters,
            FilterField::Codec,
            "opus".to_owned(),
            FormatPickerKind::Audio,
            &options,
        );

        assert_eq!(selected_row, Some(1));
        assert_eq!(filters.codec.as_deref(), Some("opus"));
        assert_eq!(filters.sample_rate.as_deref(), Some("48000"));
        assert_eq!(filters.ext.as_deref(), None);
        assert_eq!(filtered_rows(&options, &filters), vec![1]);
    }
}
