use eframe::egui::{self, Align2, Color32, FontId, Rect, RichText, Sense, Spinner, Ui};
use egui_taffy::taffy::prelude::{length, percent};
use egui_taffy::{Tui, TuiBuilderLogic as _, taffy};

use crate::app::state::{AppState, FormatPickerKind, ThumbnailRenderSource};
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::app::widgets::url_input::{AppTextBox, AppTextBoxSyntax};
use crate::infrastructure::DownloadTargetKind;

use super::common::UiText;

const XAML_COLUMN_GAP: f32 = 6.0;
const XAML_LEFT_COLUMN_WEIGHT: f32 = 5.0;
const XAML_RIGHT_COLUMN_WEIGHT: f32 = 3.0;
const APP_TEXTBOX_SINGLE_LINE_EXTRA_Y: f32 = 8.0;
const XAML_DESCRIPTION_FORMAT_GAP: f32 = 4.0;
const XAML_FORMAT_BOTTOM_GAP: f32 = 5.0;
const XAML_FIELD_MARGIN_Y: f32 = 2.0;
const INFO_LABEL_WIDTH: f32 = 66.0;
const INFO_LINE_HEIGHT: f32 = 14.0;
const INFO_BOTTOM_MARGIN: f32 = 4.0;
const SINGLE_THUMBNAIL_LOADING_SPINNER_SIZE: f32 = 48.0;
const RIGHT_INFO_SLOT_LINES: usize = 8;
const RIGHT_THUMBNAIL_CHECKBOX_GAP: f32 = 3.0;
const THUMBNAIL_ASPECT_RATIO: f32 = 16.0 / 9.0;

pub(super) fn build_single_mode_item(tui: &mut Tui, state: &mut AppState, row_height: f32) {
    let view = SingleModeView::from_state(state);
    let layout_width = tui.egui_ui().available_rect_before_wrap().width();
    let metrics = SingleModeLayoutMetrics::new(row_height, state, layout_width);

    tui.style(single_item_root_style()).add(|tui| {
        tui.style(xaml_weighted_column_style(XAML_LEFT_COLUMN_WEIGHT))
            .add(|tui| {
                build_left_xaml_column(tui, state, &view, &metrics);
            });
        tui.style(xaml_column_gap_style()).ui(|ui| {
            paint_xaml_column_gap(ui);
        });
        tui.style(xaml_weighted_column_style(XAML_RIGHT_COLUMN_WEIGHT))
            .add(|tui| {
                build_right_xaml_column(tui, state, &view, &metrics);
            });
    });
}

struct SingleModeLayoutMetrics {
    title_height: f32,
    format_area_height: f32,
    right_checkbox_height: f32,
    right_thumbnail_height: f32,
    right_info_height: f32,
}

impl SingleModeLayoutMetrics {
    fn new(row_height: f32, state: &AppState, layout_width: f32) -> Self {
        let row_height = row_height.max(24.0);
        let title_height = row_height + APP_TEXTBOX_SINGLE_LINE_EXTRA_Y;
        let format_row_height = super::item_card::item_row_block_height();
        let format_row_count = single_mode_format_row_count(state);
        let format_row_gaps =
            format_row_count.saturating_sub(1) as f32 * super::item_card::item_detail_row_gap();
        let format_area_height = format_row_height * format_row_count as f32 + format_row_gaps;
        let right_checkbox_height = row_height;
        let right_column_width = right_column_width_from_layout(layout_width);
        let right_thumbnail_height = right_thumbnail_row_height(right_column_width);
        let right_info_height = right_info_slot_height();

        Self {
            title_height,
            format_area_height,
            right_checkbox_height,
            right_thumbnail_height,
            right_info_height,
        }
    }
}

fn right_column_width_from_layout(layout_width: f32) -> f32 {
    let content_width = (layout_width - XAML_COLUMN_GAP).max(0.0);
    let total_weight = XAML_LEFT_COLUMN_WEIGHT + XAML_RIGHT_COLUMN_WEIGHT;
    if total_weight <= 0.0 {
        return 0.0;
    }
    content_width * XAML_RIGHT_COLUMN_WEIGHT / total_weight
}

fn right_thumbnail_row_height(right_column_width: f32) -> f32 {
    if right_column_width <= 0.0 || THUMBNAIL_ASPECT_RATIO <= 0.0 {
        return 0.0;
    }

    // The thumbnail should be width-first: use the right column width to reserve
    // the 16:9 row height up front. Add the top field margin used by the painter
    // so the inner thumbnail box is not forced to aspect-fit against a shorter
    // row and shrink horizontally.
    (right_column_width / THUMBNAIL_ASPECT_RATIO) + XAML_FIELD_MARGIN_Y + 1.0
}

fn single_item_root_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0),
            height: percent(1.0),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: percent(1.0),
        },
        flex_basis: length(0.0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        gap: length(0.0),
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn xaml_weighted_column_style(weight: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: length(0.0),
            height: percent(1.0),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: percent(1.0),
        },
        flex_basis: length(0.0),
        flex_grow: weight,
        flex_shrink: 1.0,
        padding: length(0.0),
        margin: length(0.0),
        gap: length(0.0),
        ..Default::default()
    }
}

fn xaml_column_gap_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(XAML_COLUMN_GAP),
            height: percent(1.0),
        },
        min_size: taffy::Size {
            width: length(XAML_COLUMN_GAP),
            height: length(0.0),
        },
        max_size: taffy::Size {
            width: length(XAML_COLUMN_GAP),
            height: percent(1.0),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn paint_xaml_column_gap(_ui: &mut Ui) {
    // XAML used a 2px structural column gap, not a painted divider.
    // Keep this cell empty so the right preview area has breathing room without a gray bar.
}

fn build_left_xaml_column(
    tui: &mut Tui,
    state: &mut AppState,
    view: &SingleModeView,
    metrics: &SingleModeLayoutMetrics,
) {
    tui.style(xaml_fixed_row_style(metrics.title_height))
        .ui(|ui| {
            render_title_field_at(ui, ui.max_rect(), view);
        });
    tui.style(xaml_grow_row_style()).ui(|ui| {
        render_description_field_at(ui, ui.max_rect(), view);
    });
    tui.style(xaml_fixed_row_style(XAML_DESCRIPTION_FORMAT_GAP))
        .ui(|_| {});
    tui.style(xaml_format_area_style(metrics.format_area_height))
        .add(|tui| {
            render_format_rows(tui, state);
        });
    tui.style(xaml_fixed_row_style(XAML_FORMAT_BOTTOM_GAP))
        .ui(|_| {});
}

fn xaml_fixed_row_style(height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn xaml_grow_row_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0),
            height: length(0.0),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        flex_basis: length(0.0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn xaml_format_area_style(height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        gap: length(super::item_card::item_detail_row_gap()),
        ..Default::default()
    }
}

fn single_mode_format_row_count(state: &AppState) -> usize {
    let Some(_) = state.queue_items.first() else {
        // Match normal mode empty-state behavior: show only video/audio rows
        // until a loaded item actually exposes subtitle choices.
        return 2;
    };

    let index = 0;
    let mut rows = 2usize; // video + audio, matching the normal item baseline.
    if state.item_shows_subtitle_row(index) {
        rows += 1;
    }
    if state.item_shows_download_section_row(index) {
        rows += 1;
    }
    rows
}

fn render_format_rows(tui: &mut Tui, state: &mut AppState) {
    if state.queue_items.is_empty() {
        render_empty_format_rows(tui, state);
        return;
    }

    let index = 0usize;
    let item_id = state.queue_items[index].id;
    let item_locked = state.item_is_busy(index);
    let audio_locked = state.item_uses_muxed_video(index);
    let show_subtitle_row = state.item_shows_subtitle_row(index);
    let show_section_row = state.item_shows_download_section_row(index);
    let label_width = {
        let ui = tui.egui_ui();
        super::item_card::visible_item_label_width(
            ui,
            state,
            false,
            show_subtitle_row,
            show_section_row,
        )
    };

    let video_label = state.ui_tr(UiText::VIDEO).to_owned();
    let audio_label = state.ui_tr(UiText::AUDIO).to_owned();
    let subtitle_label = state.ui_tr(UiText::SUBTITLE).to_owned();
    let section_label = state.ui_tr(UiText::SECTION).to_owned();
    let video_summary = state
        .localize_message(&state.selected_format_summary(index, FormatPickerKind::Video))
        .to_string();
    let audio_summary = state
        .localize_message(&state.selected_format_summary(index, FormatPickerKind::Audio))
        .to_string();
    let subtitle_summary = state
        .localize_message(&state.selected_format_summary(index, FormatPickerKind::Subtitle))
        .to_string();
    let section_summary = state
        .localize_message(&state.selected_download_section_summary(index))
        .to_string();
    let video_progress = state.item_progress(index, FormatPickerKind::Video);
    let audio_progress = state.item_progress(index, FormatPickerKind::Audio);
    let subtitle_progress = state.item_progress(index, FormatPickerKind::Subtitle);
    let show_av_progress = state.item_av_progress_visible(index);
    let show_subtitle_progress = state.item_subtitle_progress_visible(index);
    let video_export_enabled = state.item_can_export(index, DownloadTargetKind::Video);
    let audio_export_enabled = state.item_can_export(index, DownloadTargetKind::Audio);
    let subtitle_export_enabled = state.item_can_export(index, DownloadTargetKind::Subtitle);
    let mut pending_export = None;

    super::item_card::item_format_summary_row(
        tui,
        label_width,
        &video_label,
        &video_summary,
        video_progress,
        show_av_progress,
        !item_locked,
        video_export_enabled,
        || state.open_format_picker(index, FormatPickerKind::Video),
        || pending_export = Some((item_id, DownloadTargetKind::Video)),
    );
    super::item_card::item_format_summary_row(
        tui,
        label_width,
        &audio_label,
        &audio_summary,
        audio_progress,
        show_av_progress,
        !audio_locked && !item_locked,
        audio_export_enabled,
        || state.open_format_picker(index, FormatPickerKind::Audio),
        || pending_export = Some((item_id, DownloadTargetKind::Audio)),
    );
    if show_subtitle_row {
        super::item_card::item_format_summary_row(
            tui,
            label_width,
            &subtitle_label,
            &subtitle_summary,
            subtitle_progress,
            show_subtitle_progress,
            !item_locked,
            subtitle_export_enabled,
            || state.open_format_picker(index, FormatPickerKind::Subtitle),
            || pending_export = Some((item_id, DownloadTargetKind::Subtitle)),
        );
    }
    if show_section_row {
        super::item_card::item_download_section_summary_row(
            tui,
            label_width,
            &section_label,
            &section_summary,
            !item_locked,
            || state.open_format_picker(index, FormatPickerKind::Section),
        );
    }

    if let Some((item_id, kind)) = pending_export {
        super::item_card::open_export_dialog(state, item_id, kind);
    }
}

fn render_empty_format_rows(tui: &mut Tui, state: &mut AppState) {
    let label_width = {
        let ui = tui.egui_ui();
        super::item_card::visible_item_label_width(ui, state, false, false, false)
    };
    let video_waiting = state
        .ui_tr("item.after_adding_choose_the_video_format_here")
        .to_owned();
    let audio_waiting = state
        .ui_tr("item.after_adding_choose_the_audio_format_here")
        .to_owned();
    let video_label = state.ui_tr(UiText::VIDEO).to_owned();
    let audio_label = state.ui_tr(UiText::AUDIO).to_owned();

    super::item_card::item_format_summary_row(
        tui,
        label_width,
        &video_label,
        &video_waiting,
        0.0,
        false,
        false,
        false,
        || {},
        || {},
    );
    super::item_card::item_format_summary_row(
        tui,
        label_width,
        &audio_label,
        &audio_waiting,
        0.0,
        false,
        false,
        false,
        || {},
        || {},
    );
}

fn build_right_xaml_column(
    tui: &mut Tui,
    state: &mut AppState,
    view: &SingleModeView,
    metrics: &SingleModeLayoutMetrics,
) {
    // Keep the right column as one flat taffy column. Nesting the thumbnail and
    // checkbox into a separate grow container makes the thumbnail resolve
    // against the remaining vertical space first, so a wide right column can
    // still produce a narrow 16:9 thumbnail. The bottom info area is already a
    // fixed slot, so a flat column is enough and keeps the thumbnail's width as
    // the primary constraint.
    tui.style(xaml_thumbnail_row_style(metrics.right_thumbnail_height))
        .ui(|ui| {
            render_thumbnail_at(ui, ui.max_rect(), state, view);
        });
    tui.style(xaml_fixed_row_style(RIGHT_THUMBNAIL_CHECKBOX_GAP))
        .ui(|_| {});
    tui.style(xaml_fixed_row_style(metrics.right_checkbox_height))
        .ui(|ui| {
            render_download_thumbnail_checkbox_at(ui, ui.max_rect(), state);
        });
    tui.style(xaml_grow_row_style()).ui(|_| {});
    tui.style(xaml_info_slot_style(metrics.right_info_height))
        .ui(|ui| {
            render_right_info_at(ui, ui.max_rect(), state, view);
        });
}

fn xaml_thumbnail_row_style(height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0),
            height: length(height.max(0.0)),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(height.max(0.0)),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn xaml_info_slot_style(height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0),
            height: length(height.max(0.0)),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(height.max(0.0)),
        },
        flex_grow: 0.0,
        // Keep the thumbnail width-first. When vertical room is tight, this
        // bottom slot may shrink before the thumbnail row does.
        flex_shrink: 1.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

struct SingleModeView {
    title: String,
    description: String,
    title_hint: String,
    description_hint: String,
    thumbnail_hint: String,
    thumbnail_url: String,
    duration_text: String,
    webpage_url: String,
    creator_name: String,
    creator_url: String,
    upload_date: String,
    view_count: String,
    status_lines: Vec<(String, String)>,
}

impl SingleModeView {
    fn from_state(state: &AppState) -> Self {
        let status_lines = state.single_mode_status_lines();
        let Some(item) = state.queue_items.first() else {
            return Self::empty(state);
        };

        if let Some(metadata) = item.metadata() {
            return Self {
                title: metadata.title.clone(),
                description: metadata.description.clone(),
                title_hint: state.ui_tr("single.title").to_owned(),
                description_hint: state.ui_tr("single.description").to_owned(),
                thumbnail_hint: state
                    .localized_thumbnail_hint(&metadata.thumbnail_hint)
                    .into_owned(),
                thumbnail_url: metadata.thumbnail_url.clone(),
                duration_text: metadata.duration_text.clone(),
                webpage_url: metadata.webpage_url.clone(),
                creator_name: single_mode_creator_name(metadata),
                creator_url: single_mode_creator_url(metadata),
                upload_date: metadata.upload_date_text.clone(),
                view_count: metadata.view_count_text.clone(),
                status_lines,
            };
        }

        Self {
            title: item.title.clone(),
            description: String::new(),
            title_hint: state.ui_tr("single.title").to_owned(),
            description_hint: state.ui_tr("single.description").to_owned(),
            thumbnail_hint: state
                .localized_thumbnail_hint(&item.thumbnail_hint)
                .into_owned(),
            thumbnail_url: item.thumbnail_url.clone(),
            duration_text: item.duration_text.clone(),
            webpage_url: String::new(),
            creator_name: String::new(),
            creator_url: String::new(),
            upload_date: String::new(),
            view_count: String::new(),
            status_lines,
        }
    }

    fn empty(state: &AppState) -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            title_hint: state.ui_tr("single.title").to_owned(),
            description_hint: state.ui_tr("single.description").to_owned(),
            thumbnail_hint: state.ui_tr("item.thumbnail").to_owned(),
            thumbnail_url: String::new(),
            duration_text: String::new(),
            webpage_url: String::new(),
            creator_name: String::new(),
            creator_url: String::new(),
            upload_date: String::new(),
            view_count: String::new(),
            status_lines: Vec::new(),
        }
    }
}

fn single_mode_creator_name(metadata: &crate::domain::VideoMetadata) -> String {
    first_non_empty(&[
        metadata.channel.as_str(),
        metadata.uploader.as_str(),
        metadata.creator.as_str(),
    ])
}

fn single_mode_creator_url(metadata: &crate::domain::VideoMetadata) -> String {
    first_non_empty(&[
        metadata.channel_url.as_str(),
        metadata.uploader_url.as_str(),
        metadata.creator_url.as_str(),
    ])
}

fn first_non_empty(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .unwrap_or_default()
        .to_owned()
}

fn render_title_field_at(ui: &mut Ui, rect: Rect, view: &SingleModeView) {
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }

    let mut text = view.title.clone();
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_clip_rect(rect);
        ui.set_min_size(rect.size());
        AppTextBox::new(&mut text)
            .hint_text(view.title_hint.as_str())
            .editable(false)
            .selectable(true)
            .syntax(AppTextBoxSyntax::Plain)
            .desired_width(rect.width())
            .min_rows(1)
            .max_rows(Some(1))
            .allow_newline(false)
            .ctrl_click_links(false)
            .ui(ui);
    });
}

fn render_description_field_at(ui: &mut Ui, rect: Rect, view: &SingleModeView) {
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_clip_rect(rect);
        ui.set_min_size(rect.size());
        let mut text = view.description.clone();
        AppTextBox::new(&mut text)
            .hint_text(view.description_hint.as_str())
            .editable(false)
            .selectable(true)
            .syntax(AppTextBoxSyntax::Description)
            .desired_width(rect.width())
            .desired_height(rect.height())
            .min_rows(1)
            .max_rows(None)
            .allow_newline(true)
            .ctrl_click_links(true)
            .tag_link_base_url(youtube_hashtag_base_url(&view.webpage_url))
            .ui(ui);
    });
}

fn youtube_hashtag_base_url(webpage_url: &str) -> Option<String> {
    let host = url_host(webpage_url)?;
    let host = host.trim_start_matches("www.");
    if host == "youtube.com" || host.ends_with(".youtube.com") || host == "youtu.be" {
        Some("https://www.youtube.com/hashtag/".to_owned())
    } else {
        None
    }
}

fn url_host(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let after_scheme = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed);
    let host_port = after_scheme
        .split(|ch| matches!(ch, '/' | '?' | '#'))
        .next()?
        .trim()
        .trim_end_matches('.');
    if host_port.is_empty() || host_port.chars().any(char::is_whitespace) {
        return None;
    }
    let host = host_port
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(host_port)
        .split(':')
        .next()
        .unwrap_or(host_port)
        .to_ascii_lowercase();
    (!host.is_empty()).then_some(host)
}

fn render_thumbnail_at(ui: &mut Ui, rect: Rect, state: &mut AppState, view: &SingleModeView) {
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }

    // The taffy row already owns the 16:9 aspect ratio. Do not shrink only
    // vertically before fitting; that creates a narrower aspect-fit rect and
    // leaves a visible empty strip on the right side. Keep the thumbnail frame
    // on the full row rect so its left edge still lines up with the checkbox
    // below while its right edge reaches the right column boundary.
    let thumbnail_area =
        Rect::from_min_max(rect.min + egui::vec2(0.0, XAML_FIELD_MARGIN_Y), rect.max);
    let thumbnail_rect = fit_aspect_rect_left_aligned(thumbnail_area, THUMBNAIL_ASPECT_RATIO);
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
            if ui.button(state.ui_tr("item.save_as")).clicked() {
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

fn render_download_thumbnail_checkbox_at(ui: &mut Ui, rect: Rect, state: &mut AppState) {
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_clip_rect(rect);
        ui.set_min_size(rect.size());
        let mut checked = state.item_defaults.write_thumbnail;
        let response = ui.checkbox(
            &mut checked,
            RichText::new(state.ui_tr("item.download_thumbnail"))
                .color(single_mode_picker_text_color(ui)),
        );
        if response.changed() {
            state.set_write_thumbnail(checked);
        }
    });
}

fn right_info_slot_height() -> f32 {
    RIGHT_INFO_SLOT_LINES as f32 * INFO_LINE_HEIGHT + INFO_BOTTOM_MARGIN
}

struct SingleInfoLine {
    label: String,
    value: String,
    link_url: Option<String>,
}

fn render_right_info_at(ui: &mut Ui, rect: Rect, state: &AppState, view: &SingleModeView) {
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
                label: state.ui_tr("single.info.channel").to_owned(),
                value: creator_name.to_owned(),
                link_url: (youtube_source && !creator_url.is_empty())
                    .then(|| creator_url.to_owned()),
            });
        }

        let upload_date = format_single_info_date(&view.upload_date);
        if !upload_date.is_empty() {
            lines.push(SingleInfoLine {
                label: state.ui_tr("single.info.date").to_owned(),
                value: upload_date,
                link_url: None,
            });
        }

        let view_count = compact_count_text(&view.view_count);
        if !view_count.is_empty() {
            lines.push(SingleInfoLine {
                label: state.ui_tr("single.info.views").to_owned(),
                value: view_count,
                link_url: None,
            });
        }
    }

    if lines.is_empty() {
        return;
    }

    let max_visible_lines = ((rect.height() - INFO_BOTTOM_MARGIN).max(0.0) / INFO_LINE_HEIGHT)
        .floor()
        .max(0.0) as usize;
    if max_visible_lines == 0 {
        return;
    }

    let hidden_count = lines.len().saturating_sub(max_visible_lines);
    let visible_lines = &lines[hidden_count..];
    let total_height = visible_lines.len() as f32 * INFO_LINE_HEIGHT;
    let mut line_top = (rect.bottom() - INFO_BOTTOM_MARGIN - total_height).max(rect.top());
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
        2.0,
        ui.visuals().faint_bg_color,
        stroke,
        egui::StrokeKind::Outside,
    );

    let inner = rect.shrink(1.0);
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
            let spinner_size = SINGLE_THUMBNAIL_LOADING_SPINNER_SIZE
                .min(inner.width().min(inner.height()).max(1.0));
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
    if duration_text.trim().is_empty() || inner.width() <= 58.0 || inner.height() <= 20.0 {
        return;
    }
    let badge_rect = Rect::from_min_size(
        egui::pos2(inner.right() - 56.0, inner.bottom() - 18.0),
        egui::vec2(54.0, 16.0),
    );
    ui.painter()
        .rect_filled(badge_rect, 2.0, Color32::from_black_alpha(150));
    ui.painter().text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        duration_text,
        FontId::proportional(10.0),
        Color32::WHITE,
    );
}

fn paint_single_thumbnail_placeholder(ui: &Ui, inner: Rect, hint: &str) {
    let icon_size = 28.0_f32.min((inner.width().min(inner.height()) * 0.45).max(8.0));
    let icon_rect = Rect::from_center_size(
        inner.center() - egui::vec2(0.0, 10.0_f32.min(inner.height() * 0.12)),
        egui::vec2(icon_size, icon_size),
    );
    icon_image(
        AppIcon::Video,
        icon_size,
        standard_icon_color(ui).linear_multiply(0.72),
    )
    .paint_at(ui, icon_rect);
    if inner.height() > 34.0 {
        ui.painter().text(
            inner.center() + egui::vec2(0.0, 18.0_f32.min(inner.height() * 0.2)),
            Align2::CENTER_CENTER,
            hint,
            FontId::proportional(11.0),
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
    if *line_top + INFO_LINE_HEIGHT > rect.bottom() + 0.5 {
        return;
    }

    let line_rect = Rect::from_min_size(
        egui::pos2(rect.left(), *line_top),
        egui::vec2(rect.width(), INFO_LINE_HEIGHT),
    );
    *line_top += INFO_LINE_HEIGHT;

    let label_rect = Rect::from_min_size(
        line_rect.min,
        egui::vec2(INFO_LABEL_WIDTH.min(line_rect.width()), line_rect.height()),
    );
    let value_rect = Rect::from_min_max(
        egui::pos2(
            (label_rect.right() + 2.0).min(line_rect.right()),
            line_rect.top(),
        ),
        line_rect.max,
    );

    paint_truncated_text(
        ui,
        label_rect,
        RichText::new(line.label.as_str())
            .size(10.0)
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
    let rich_text = RichText::new(text).size(10.0).color(color);
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
        let underline_y = (y + galley.size().y - 1.0).min(rect.bottom() - 1.0);
        ui.painter().with_clip_rect(rect).line_segment(
            [
                egui::pos2(x, underline_y),
                egui::pos2((x + galley.size().x).min(rect.right()), underline_y),
            ],
            egui::Stroke::new(1.0, color),
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
