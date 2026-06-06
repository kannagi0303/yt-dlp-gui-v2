use eframe::egui::{self, Rect, Ui};
use egui_taffy::Tui;

use crate::app::state::AppState;
use crate::app::widgets::url_input::{AppTextBox, AppTextBoxSyntax};

use super::semantic_ui_metrics;
use super::single_mode_format_rows::single_mode_format_row_count;
use super::single_mode_template::{
    build_single_mode_template, right_column_width_for_layout_width, show_single_mode_template,
};

pub(super) fn build_single_mode_item(tui: &mut Tui, state: &mut AppState, row_height: f32) {
    let view = SingleModeView::from_state(state);
    let layout_width = tui.egui_ui().available_rect_before_wrap().width();
    let metrics = SingleModeLayoutMetrics::new(row_height, state, layout_width);

    let template = build_single_mode_template(&metrics);
    show_single_mode_template(template, tui, state, &view);
}

pub(super) struct SingleModeLayoutMetrics {
    pub(super) title_height: f32,
    pub(super) format_area_height: f32,
    pub(super) right_checkbox_height: f32,
    pub(super) right_thumbnail_height: f32,
    pub(super) right_info_height: f32,
}

impl SingleModeLayoutMetrics {
    fn new(row_height: f32, state: &AppState, layout_width: f32) -> Self {
        let row_height = row_height.max(24.0);
        let title_height =
            semantic_ui_metrics::single_mode_title_field_height_for_control_height(row_height);
        let format_row_height = super::item_card::item_row_block_height();
        let format_row_count = single_mode_format_row_count(state);
        let format_row_gaps =
            format_row_count.saturating_sub(1) as f32 * super::item_card::item_detail_row_gap();
        let format_area_height = format_row_height * format_row_count as f32 + format_row_gaps;
        let right_checkbox_height = row_height;
        let right_column_width = right_column_width_for_layout_width(layout_width);
        let right_thumbnail_height =
            semantic_ui_metrics::single_mode_thumbnail_row_height_for_right_column_width(
                right_column_width,
            );
        let right_info_height = semantic_ui_metrics::single_mode_right_info_slot_height();

        Self {
            title_height,
            format_area_height,
            right_checkbox_height,
            right_thumbnail_height,
            right_info_height,
        }
    }
}

pub(super) struct SingleModeView {
    pub(super) title: String,
    pub(super) description: String,
    pub(super) title_hint: String,
    pub(super) description_hint: String,
    pub(super) thumbnail_hint: String,
    pub(super) thumbnail_url: String,
    pub(super) duration_text: String,
    pub(super) webpage_url: String,
    pub(super) creator_name: String,
    pub(super) creator_url: String,
    pub(super) upload_date: String,
    pub(super) view_count: String,
    pub(super) status_lines: Vec<(String, String)>,
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
                title_hint: state.ui_i18n_text_for_key("single.title").to_owned(),
                description_hint: state.ui_i18n_text_for_key("single.description").to_owned(),
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
            title_hint: state.ui_i18n_text_for_key("single.title").to_owned(),
            description_hint: state.ui_i18n_text_for_key("single.description").to_owned(),
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
            title_hint: state.ui_i18n_text_for_key("single.title").to_owned(),
            description_hint: state.ui_i18n_text_for_key("single.description").to_owned(),
            thumbnail_hint: state.ui_i18n_text_for_key("item.thumbnail").to_owned(),
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

pub(super) fn render_title_field_at(ui: &mut Ui, rect: Rect, view: &SingleModeView) {
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

pub(super) fn render_description_field_at(ui: &mut Ui, rect: Rect, view: &SingleModeView) {
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

pub(super) fn youtube_hashtag_base_url(webpage_url: &str) -> Option<String> {
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
