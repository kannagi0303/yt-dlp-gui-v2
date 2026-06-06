use eframe::egui::{self, Align2, Color32, FontId, Rect, RichText, Sense, Spinner, Ui};

use crate::app::state::{AppState, ThumbnailRenderSource};
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};

use super::semantic_ui_metrics;
use super::single_mode::{SingleModeView, youtube_hashtag_base_url};

pub(super) fn render_thumbnail_at(
    ui: &mut Ui,
    rect: Rect,
    state: &mut AppState,
    view: &SingleModeView,
) {
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }

    // The taffy row already owns the 16:9 aspect ratio. Do not shrink only
    // vertically before fitting; that creates a narrower aspect-fit rect and
    // leaves a visible empty strip on the right side. Keep the thumbnail frame
    // on the full row rect so its left edge still lines up with the checkbox
    // below while its right edge reaches the right column boundary.
    let thumbnail_area = semantic_ui_metrics::single_mode_thumbnail_area_for_row_rect(rect);
    let thumbnail_rect = fit_aspect_rect_left_aligned(
        thumbnail_area,
        semantic_ui_metrics::single_mode_thumbnail_aspect_ratio(),
    );
    let response = ui.interact(
        thumbnail_rect,
        ui.make_persistent_id("single-mode-thumbnail"),
        Sense::click(),
    );
    let thumbnail_source =
        state.single_thumbnail_render_source_for_url(ui.ctx(), &view.thumbnail_url);
    paint_thumbnail_box(ui, thumbnail_rect, state, view, thumbnail_source);
    if !view.thumbnail_url.is_empty() {
        response.context_menu(|ui| {
            if ui
                .button(state.ui_i18n_text_for_key("item.save_as"))
                .clicked()
            {
                save_single_mode_thumbnail_as(state, view);
                ui.close();
            }
        });
    }
}

fn fit_aspect_rect_left_aligned(available: Rect, aspect_ratio: f32) -> Rect {
    if available.width() <= 0.0 || available.height() <= 0.0 || aspect_ratio <= 0.0 {
        return available;
    }

    let mut size = egui::vec2(available.width(), available.width() / aspect_ratio);
    if size.y > available.height() {
        size.y = available.height();
        size.x = size.y * aspect_ratio;
    }

    Rect::from_min_size(available.min, size)
}

pub(super) fn render_download_thumbnail_checkbox_at(ui: &mut Ui, rect: Rect, state: &mut AppState) {
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_clip_rect(rect);
        ui.set_min_size(rect.size());
        let mut checked = state.item_defaults.write_thumbnail;
        let response = ui.checkbox(
            &mut checked,
            RichText::new(state.ui_i18n_text_for_key("item.download_thumbnail"))
                .color(single_mode_picker_text_color(ui)),
        );
        if response.changed() {
            state.set_write_thumbnail(checked);
        }
    });
}

struct SingleInfoLine {
    label: String,
    value: String,
    link_url: Option<String>,
}

pub(super) fn render_right_info_at(
    ui: &mut Ui,
    rect: Rect,
    state: &AppState,
    view: &SingleModeView,
) {
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }

    let mut lines: Vec<SingleInfoLine> = Vec::new();
    if !view.status_lines.is_empty() {
        lines.extend(
            view.status_lines
                .iter()
                .map(|(label, value)| SingleInfoLine {
                    label: label.clone(),
                    value: value.clone(),
                    link_url: None,
                }),
        );
    } else {
        let youtube_source = is_youtube_source(&view.webpage_url);
        let creator_name = view.creator_name.trim();
        if !creator_name.is_empty() {
            let creator_url = view.creator_url.trim();
            lines.push(SingleInfoLine {
                label: state.ui_i18n_text_for_key("single.info.channel").to_owned(),
                value: creator_name.to_owned(),
                link_url: (youtube_source && !creator_url.is_empty())
                    .then(|| creator_url.to_owned()),
            });
        }

        let upload_date = format_single_info_date(&view.upload_date);
        if !upload_date.is_empty() {
            lines.push(SingleInfoLine {
                label: state.ui_i18n_text_for_key("single.info.date").to_owned(),
                value: upload_date,
                link_url: None,
            });
        }

        let view_count = compact_count_text(&view.view_count);
        if !view_count.is_empty() {
            lines.push(SingleInfoLine {
                label: state.ui_i18n_text_for_key("single.info.views").to_owned(),
                value: view_count,
                link_url: None,
            });
        }
    }

    if lines.is_empty() {
        return;
    }

    let max_visible_lines =
        ((rect.height() - semantic_ui_metrics::single_mode_info_bottom_margin()).max(0.0)
            / semantic_ui_metrics::single_mode_info_line_height())
        .floor()
        .max(0.0) as usize;
    if max_visible_lines == 0 {
        return;
    }

    let hidden_count = lines.len().saturating_sub(max_visible_lines);
    let visible_lines = &lines[hidden_count..];
    let total_height =
        visible_lines.len() as f32 * semantic_ui_metrics::single_mode_info_line_height();
    let mut line_top =
        (rect.bottom() - semantic_ui_metrics::single_mode_info_bottom_margin() - total_height)
            .max(rect.top());
    for line in visible_lines {
        render_info_line_at(ui, rect, &mut line_top, line);
    }
}

fn is_youtube_source(webpage_url: &str) -> bool {
    youtube_hashtag_base_url(webpage_url).is_some()
}

fn format_single_info_date(value: &str) -> String {
    let value = value.trim();
    if value.len() == 8 && value.chars().all(|ch| ch.is_ascii_digit()) {
        return format!("{}-{}-{}", &value[0..4], &value[4..6], &value[6..8]);
    }
    if let Some(head) = value.get(..10) {
        if head.as_bytes().get(4) == Some(&b'-')
            && head.as_bytes().get(7) == Some(&b'-')
            && head
                .chars()
                .enumerate()
                .all(|(index, ch)| matches!(index, 4 | 7) || ch.is_ascii_digit())
        {
            return head.to_owned();
        }
    }
    value.to_owned()
}

fn compact_count_text(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    let parseable = value
        .chars()
        .all(|ch| ch.is_ascii_digit() || matches!(ch, ',' | '_' | ' '));
    if !parseable {
        return value.to_owned();
    }
    let digits = value
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>();
    let Ok(count) = digits.parse::<u64>() else {
        return value.to_owned();
    };
    format_compact_count(count)
}

fn format_compact_count(count: u64) -> String {
    match count {
        1_000_000_000.. => compact_unit(count, 1_000_000_000, "B"),
        1_000_000.. => compact_unit(count, 1_000_000, "M"),
        1_000.. => compact_unit(count, 1_000, "K"),
        _ => count.to_string(),
    }
}

fn compact_unit(count: u64, unit: u64, suffix: &str) -> String {
    let value = count as f64 / unit as f64;
    if value >= 100.0 || (value.fract() * 10.0).round() == 0.0 {
        format!("{:.0}{suffix}", value)
    } else {
        format!("{:.1}{suffix}", value)
    }
}

fn paint_thumbnail_box(
    ui: &mut Ui,
    rect: Rect,
    state: &AppState,
    view: &SingleModeView,
    thumbnail_source: ThumbnailRenderSource,
) {
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.painter().rect(
        rect,
        semantic_ui_metrics::single_mode_thumbnail_frame_corner_radius(),
        ui.visuals().faint_bg_color,
        stroke,
        egui::StrokeKind::Outside,
    );

    let inner = semantic_ui_metrics::single_mode_thumbnail_inner_rect(rect);
    match thumbnail_source {
        ThumbnailRenderSource::Texture(texture) => {
            ui.painter().image(
                texture.id(),
                inner,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            paint_single_duration_badge(ui, inner, &view.duration_text);
            return;
        }
        ThumbnailRenderSource::DirectUrl if !view.thumbnail_url.trim().is_empty() => {
            egui::Image::new(view.thumbnail_url.as_str())
                .fit_to_exact_size(inner.size())
                .show_loading_spinner(false)
                .paint_at(ui, inner);
            paint_single_duration_badge(ui, inner, &view.duration_text);
            return;
        }
        ThumbnailRenderSource::Loading => {
            let spinner_size =
                semantic_ui_metrics::single_mode_thumbnail_loading_spinner_size_for_inner_rect(
                    inner,
                );
            let spinner_rect =
                Rect::from_center_size(inner.center(), egui::vec2(spinner_size, spinner_size));
            ui.scope_builder(egui::UiBuilder::new().max_rect(spinner_rect), |ui| {
                ui.centered_and_justified(|ui| {
                    ui.add(Spinner::new().size(spinner_size));
                });
            });
            return;
        }
        ThumbnailRenderSource::Failed(_error) => {
            paint_single_thumbnail_placeholder(
                ui,
                inner,
                state.localize_message(&view.thumbnail_hint).as_str(),
            );
            drop(ui.interact(
                rect,
                ui.make_persistent_id("single-mode-thumbnail-error"),
                Sense::hover(),
            ));
            return;
        }
        ThumbnailRenderSource::None | ThumbnailRenderSource::DirectUrl => {}
    }

    paint_single_thumbnail_placeholder(
        ui,
        inner,
        state.localize_message(&view.thumbnail_hint).as_str(),
    );
}

fn paint_single_duration_badge(ui: &Ui, inner: Rect, duration_text: &str) {
    if duration_text.trim().is_empty()
        || !semantic_ui_metrics::single_mode_duration_badge_should_be_visible(inner)
    {
        return;
    }
    let badge_rect = semantic_ui_metrics::single_mode_duration_badge_rect(inner);
    ui.painter().rect_filled(
        badge_rect,
        semantic_ui_metrics::single_mode_duration_badge_corner_radius(),
        Color32::from_black_alpha(150),
    );
    ui.painter().text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        duration_text,
        FontId::proportional(semantic_ui_metrics::single_mode_duration_badge_font_size()),
        Color32::WHITE,
    );
}

fn paint_single_thumbnail_placeholder(ui: &Ui, inner: Rect, hint: &str) {
    let icon_size = semantic_ui_metrics::single_mode_placeholder_icon_size_for_inner_rect(inner);
    let icon_rect = Rect::from_center_size(
        semantic_ui_metrics::single_mode_placeholder_icon_center_for_inner_rect(inner),
        egui::vec2(icon_size, icon_size),
    );
    icon_image(
        AppIcon::Video,
        icon_size,
        standard_icon_color(ui).linear_multiply(0.72),
    )
    .paint_at(ui, icon_rect);
    if semantic_ui_metrics::single_mode_placeholder_text_should_be_visible(inner) {
        ui.painter().text(
            semantic_ui_metrics::single_mode_placeholder_text_center_for_inner_rect(inner),
            Align2::CENTER_CENTER,
            hint,
            FontId::proportional(semantic_ui_metrics::single_mode_placeholder_text_font_size()),
            ui.visuals().weak_text_color(),
        );
    }
}

fn single_mode_picker_text_color(ui: &Ui) -> Color32 {
    ui.visuals().widgets.inactive.fg_stroke.color
}

fn save_single_mode_thumbnail_as(state: &mut AppState, view: &SingleModeView) {
    let url = view.thumbnail_url.trim();
    if url.is_empty() {
        return;
    }

    let file_name = format!("{}.jpg", sanitize_thumbnail_file_stem(&view.title));
    let dialog = rfd::FileDialog::new()
        .add_filter("JPEG image", &["jpg", "jpeg"])
        .add_filter("PNG image", &["png"])
        .add_filter("WebP image", &["webp"])
        .add_filter("Original image", &["jpg", "jpeg", "png", "webp", "img"])
        .set_file_name(&file_name);

    if let Some(path) = dialog.save_file() {
        if let Err(error) = state.save_thumbnail_url_to_path(url, &path) {
            state.set_last_action_message(error);
        }
    }
}

fn sanitize_thumbnail_file_stem(title: &str) -> String {
    let mut value = title
        .trim()
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            ch if ch.is_control() => '_',
            ch => ch,
        })
        .collect::<String>();
    value = value.trim_matches(|ch| ch == ' ' || ch == '.').to_owned();
    if value.is_empty() {
        "thumbnail".to_owned()
    } else {
        value
    }
}

fn render_info_line_at(ui: &mut Ui, rect: Rect, line_top: &mut f32, line: &SingleInfoLine) {
    if *line_top + semantic_ui_metrics::single_mode_info_line_height() > rect.bottom() + 0.5 {
        return;
    }

    let line_rect = Rect::from_min_size(
        egui::pos2(rect.left(), *line_top),
        egui::vec2(
            rect.width(),
            semantic_ui_metrics::single_mode_info_line_height(),
        ),
    );
    *line_top += semantic_ui_metrics::single_mode_info_line_height();

    let label_rect = Rect::from_min_size(
        line_rect.min,
        egui::vec2(
            semantic_ui_metrics::single_mode_info_label_width_for_line_width(line_rect.width()),
            line_rect.height(),
        ),
    );
    let value_rect = Rect::from_min_max(
        egui::pos2(
            (label_rect.right()
                + semantic_ui_metrics::single_mode_info_label_to_value_horizontal_gap())
            .min(line_rect.right()),
            line_rect.top(),
        ),
        line_rect.max,
    );

    paint_truncated_text(
        ui,
        label_rect,
        RichText::new(line.label.as_str())
            .size(semantic_ui_metrics::single_mode_info_text_font_size())
            .color(single_mode_picker_text_color(ui)),
        single_mode_picker_text_color(ui),
        egui::TextStyle::Small,
        egui::Align::Min,
    );

    let ctrl_down = ui.input(|input| input.modifiers.ctrl);
    if let Some(url) = line.link_url.as_deref() {
        let response = ui.interact(
            value_rect,
            ui.id()
                .with(("single-info-link", line.label.as_str(), line.value.as_str())),
            Sense::click(),
        );
        if ctrl_down && response.hovered() {
            ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::PointingHand);
        }
        if ctrl_down && response.clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(url.to_owned()));
        }
    }

    paint_truncated_text_with_optional_underline(
        ui,
        value_rect,
        line.value.as_str(),
        ctrl_down && line.link_url.is_some(),
    );
}

fn paint_truncated_text_with_optional_underline(ui: &Ui, rect: Rect, text: &str, underline: bool) {
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }
    let color = single_mode_picker_text_color(ui);
    let rich_text = RichText::new(text)
        .size(semantic_ui_metrics::single_mode_info_text_font_size())
        .color(color);
    let galley = egui::WidgetText::from(rich_text).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        rect.width(),
        egui::TextStyle::Small,
    );
    let x = rect.right() - galley.size().x;
    let y = rect.center().y - galley.size().y * 0.5;
    ui.painter()
        .with_clip_rect(rect)
        .galley(egui::pos2(x, y), galley.clone(), color);
    if underline {
        let underline_inset = semantic_ui_metrics::single_mode_info_text_underline_vertical_inset();
        let underline_y =
            (y + galley.size().y - underline_inset).min(rect.bottom() - underline_inset);
        ui.painter().with_clip_rect(rect).line_segment(
            [
                egui::pos2(x, underline_y),
                egui::pos2((x + galley.size().x).min(rect.right()), underline_y),
            ],
            egui::Stroke::new(
                semantic_ui_metrics::single_mode_info_text_underline_stroke_width(),
                color,
            ),
        );
    }
}

fn paint_truncated_text(
    ui: &Ui,
    rect: Rect,
    text: RichText,
    fallback_color: Color32,
    style: egui::TextStyle,
    align_x: egui::Align,
) {
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }

    let galley = egui::WidgetText::from(text).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        rect.width(),
        style,
    );
    let x = match align_x {
        egui::Align::Min => rect.left(),
        egui::Align::Center => rect.center().x - galley.size().x * 0.5,
        egui::Align::Max => rect.right() - galley.size().x,
    };
    let y = rect.center().y - galley.size().y * 0.5;
    ui.painter()
        .with_clip_rect(rect)
        .galley(egui::pos2(x, y), galley, fallback_color);
}
