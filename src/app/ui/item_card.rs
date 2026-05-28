use eframe::egui::{
    self, Align, Layout, RichText, ScrollArea, Sense, Spinner, TextEdit, TextStyle, TextWrapMode,
    Ui, WidgetText,
};
use egui_extras::{Size, StripBuilder};
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{Tui, TuiBuilderLogic as _, taffy, tui};

use crate::app::state::{
    AppState, FormatPickerKind, ItemTitleVisualState, ThumbnailRenderSource,
    sanitize_file_name_for_windows,
};
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::app::widgets::url_input::{accent_blue_for_ui, accent_green_for_ui, accent_red_for_ui};
use crate::domain::{CompactMusicState, QueueItemId};
use crate::infrastructure::{
    DownloadTargetKind, OutputFileActionMode, open_output_file, open_output_folder,
    output_file_exists, output_parent_folder_exists,
};

use super::common::{ITEM_TITLE_FONT_SIZE, UiText, cell_label_right};
use super::compact_row::{CompactRowSpec, CompactRowVisualState, render_music_compact_row};
use super::measure::{max_text_height_for_lines, text_width, wrapped_text_height};

const TITLE_ROW_HEIGHT: f32 = 16.0;
const TITLE_SPINNER_TOP_PADDING: f32 = 2.0;
const TITLE_TEXT_TOP_PADDING: f32 = -3.0;
const TITLE_DELETE_TOP_PADDING: f32 = -2.0;
const TITLE_SPINNER_SIZE: f32 = 9.0;
const ITEM_THUMBNAIL_WIDTH: f32 = 128.0;
const ITEM_THUMBNAIL_HEIGHT: f32 = ITEM_THUMBNAIL_WIDTH * 9.0 / 16.0;
const ITEM_FIELD_ROW_HEIGHT: f32 = 18.0;
const ITEM_FIELD_ROW_PADDING_Y: f32 = 0.0;
const ITEM_DETAIL_COLUMN_GAP: f32 = 3.0;
const ITEM_DETAIL_ROW_GAP: f32 = 3.0;
const ITEM_CARD_COLUMN_GAP: f32 = 8.0;

pub(super) fn render_batch_list(ui: &mut Ui, state: &mut AppState) {
    render_queue_toolbar(ui, state);
    ui.add_space(ui.spacing().item_spacing.y);
    let mut pending_remove_item_id = None;
    let mut pending_cancel_item_id = None;
    let mut pending_export = None;

    ScrollArea::vertical()
        .id_salt("batch-item-list")
        .show(ui, |ui| {
            if state.queue_items.is_empty() {
                if state.queue_display_mode_is_audio() {
                    render_empty_music_compact_item(ui, state);
                } else {
                    let empty_item_preview = state.empty_item_preview.clone();
                    render_empty_batch_item_card(ui, state, &empty_item_preview);
                }
                return;
            }

            for index in 0..state.queue_items.len() {
                let item_id = state.queue_items[index].id;
                if state.queue_display_mode_is_audio() {
                    let item = &state.queue_items[index];
                    let title = item.title.clone();
                    let id_salt = item.id;
                    let duration_text = item.duration_text.clone();
                    let music_state = item.compact_music_state.unwrap_or(CompactMusicState::Ready);
                    let thumbnail_url = state.item_thumbnail_url(index).to_owned();
                    let thumbnail_source =
                        state.thumbnail_render_source_for_url(ui.ctx(), &thumbnail_url);
                    let cache_progress = state.music_item_cache_progress_ratio(item_id);
                    let row_progress = state.music_item_compact_progress_ratio(item_id);
                    let show_row_progress = state.music_item_compact_progress_visible(item_id);
                    let playback_progress = state.music_item_playback_progress_ratio(item_id);
                    let is_current = state.music_current_item_id() == Some(item_id);
                    let is_playing = state.music_item_is_playing(item_id);
                    let (mut visual_state, mut status_text) = compact_music_visual_state(
                        state,
                        music_state,
                        &duration_text,
                        playback_progress,
                        cache_progress,
                    );
                    if let Some(progress_text) =
                        state.music_item_compact_progress_status_text(item_id)
                    {
                        status_text = progress_text;
                    }
                    if state.item_title_visual_state(index) == ItemTitleVisualState::Completed {
                        visual_state = CompactRowVisualState::Downloaded;
                        status_text = state.tr("music.status.completed").to_owned();
                    } else if state.music_item_has_complete_cache(item_id) {
                        visual_state = CompactRowVisualState::Finished;
                    }
                    let output = render_music_compact_row(
                        ui,
                        CompactRowSpec {
                            id_salt,
                            title: &title,
                            thumbnail_url: &thumbnail_url,
                            thumbnail_source,
                            status_text: &status_text,
                            visual_state,
                            progress: row_progress,
                            show_progress: show_row_progress,
                            is_current,
                            is_playing,
                            play_enabled: true,
                            remove_enabled: true,
                            play_label: if is_playing {
                                state.tr("music.pause")
                            } else {
                                state.tr("music.play")
                            },
                            remove_label: state.tr("item.remove"),
                        },
                    );
                    if output.play_clicked {
                        state.play_music_item(item_id);
                    }
                    if output.remove_clicked {
                        pending_remove_item_id = Some(item_id);
                    }
                    ui.add_space(ui.spacing().item_spacing.y);
                    continue;
                }
                let title = state.item_title_text(index);
                let title_hover = title.clone();
                let title_state = state.item_title_visual_state(index);
                let title_loading = state.item_title_is_loading(index);
                let item_locked = state.item_is_busy(index);
                let item_cancellable = state.item_has_cancellable_download_workflow(item_id);
                let video_summary = state.selected_format_summary(index, FormatPickerKind::Video);
                let audio_summary = state.selected_format_summary(index, FormatPickerKind::Audio);
                let subtitle_summary =
                    state.selected_format_summary(index, FormatPickerKind::Subtitle);
                let audio_locked = state.item_uses_muxed_video(index);
                let use_seed_compact_layout = state.item_uses_seed_compact_layout(index);
                let show_subtitle_row = state.item_shows_subtitle_row(index);
                let video_progress = state.item_progress(index, FormatPickerKind::Video);
                let audio_progress = state.item_progress(index, FormatPickerKind::Audio);
                let subtitle_progress = state.item_progress(index, FormatPickerKind::Subtitle);
                let show_av_progress = state.item_av_progress_visible(index);
                let show_subtitle_progress = state.item_subtitle_progress_visible(index);
                let show_section_row = state.item_shows_download_section_row(index);
                let section_summary = state.selected_download_section_summary(index);
                let item_error_text = state.item_error_text(index);
                let item_label_width = visible_item_label_width(
                    ui,
                    state,
                    use_seed_compact_layout,
                    show_subtitle_row,
                    show_section_row,
                );

                ui.set_width(ui.available_width());
                let hover_memory_id = ui
                    .id()
                    .with(("queue-item-hover", state.queue_items[index].id));
                let item_hovered = ui
                    .ctx()
                    .data(|data| data.get_temp::<bool>(hover_memory_id).unwrap_or(false));
                let group_response = ui.group(|ui| {
                    let card_width = ui.available_width();
                    let detail_width =
                        (card_width - ITEM_THUMBNAIL_WIDTH - ITEM_CARD_COLUMN_GAP).max(0.0);
                    let header_height = item_header_height(ui, &title, title_loading, detail_width);
                    let visible_body_rows = visible_item_body_rows(
                        use_seed_compact_layout,
                        show_subtitle_row,
                        show_section_row,
                        item_error_text.is_some(),
                    );
                    let body_target_height = (ITEM_THUMBNAIL_HEIGHT - header_height).max(0.0);
                    let body_rows_gap =
                        visible_body_rows.saturating_sub(1) as f32 * ITEM_DETAIL_ROW_GAP;
                    let body_content_height =
                        visible_body_rows as f32 * item_row_block_height() + body_rows_gap;
                    let body_spacer = (body_target_height - body_content_height).max(0.0);
                    let error_color = accent_red_for_ui(ui);

                    ui.set_width(card_width);
                    tui(ui, ui.id().with(("normal-item-card", item_id)))
                        .reserve_width(card_width)
                        .style(item_card_root_style())
                        .show(|tui| {
                            tui.style(item_thumbnail_column_style()).ui(|ui| {
                                let thumbnail_url = state.item_thumbnail_url(index).to_owned();
                                let thumbnail_hint = state
                                    .localized_thumbnail_hint(state.item_thumbnail_hint(index))
                                    .into_owned();
                                let duration_text = state.item_duration_text(index).to_owned();
                                let thumbnail_source =
                                    state.thumbnail_render_source_for_url(ui.ctx(), &thumbnail_url);
                                row_thumbnail(
                                    ui,
                                    state,
                                    &thumbnail_url,
                                    &thumbnail_hint,
                                    &duration_text,
                                    thumbnail_source,
                                );
                            });

                            tui.style(item_detail_column_style()).add(|tui| {
                                if item_header_row(
                                    tui,
                                    &title,
                                    &title_hover,
                                    title_state,
                                    title_loading,
                                    header_height,
                                    !item_locked || item_cancellable,
                                    item_hovered,
                                    if item_cancellable {
                                        state.tr("item.stop_download")
                                    } else {
                                        state.tr("item.remove")
                                    },
                                ) {
                                    if item_cancellable {
                                        pending_cancel_item_id = Some(item_id);
                                    } else {
                                        pending_remove_item_id = Some(item_id);
                                    }
                                }
                                if !use_seed_compact_layout {
                                    item_format_summary_row(
                                        tui,
                                        item_label_width,
                                        state.tr(UiText::VIDEO),
                                        &state.localize_message(&video_summary),
                                        video_progress,
                                        show_av_progress,
                                        !item_locked,
                                        state.item_can_export(index, DownloadTargetKind::Video),
                                        state.tr("item.save_as"),
                                        || state.open_format_picker(index, FormatPickerKind::Video),
                                        || {
                                            pending_export =
                                                Some((item_id, DownloadTargetKind::Video))
                                        },
                                    );
                                    item_format_summary_row(
                                        tui,
                                        item_label_width,
                                        state.tr(UiText::AUDIO),
                                        &state.localize_message(&audio_summary),
                                        audio_progress,
                                        show_av_progress,
                                        !audio_locked && !item_locked,
                                        state.item_can_export(index, DownloadTargetKind::Audio),
                                        state.tr("item.save_as"),
                                        || state.open_format_picker(index, FormatPickerKind::Audio),
                                        || {
                                            pending_export =
                                                Some((item_id, DownloadTargetKind::Audio))
                                        },
                                    );
                                    if show_subtitle_row {
                                        item_format_summary_row(
                                            tui,
                                            item_label_width,
                                            state.tr(UiText::SUBTITLE),
                                            &state.localize_message(&subtitle_summary),
                                            subtitle_progress,
                                            show_subtitle_progress,
                                            !item_locked,
                                            state.item_can_export(
                                                index,
                                                DownloadTargetKind::Subtitle,
                                            ),
                                            state.tr("item.save_as"),
                                            || {
                                                state.open_format_picker(
                                                    index,
                                                    FormatPickerKind::Subtitle,
                                                )
                                            },
                                            || {
                                                pending_export =
                                                    Some((item_id, DownloadTargetKind::Subtitle))
                                            },
                                        );
                                    }
                                    if show_section_row {
                                        item_download_section_summary_row(
                                            tui,
                                            item_label_width,
                                            state.tr(UiText::SECTION),
                                            &state.localize_message(&section_summary),
                                            !item_locked,
                                            || {
                                                state.open_format_picker(
                                                    index,
                                                    FormatPickerKind::Section,
                                                )
                                            },
                                        );
                                    }
                                }
                                if let Some(error_text) = item_error_text.as_deref() {
                                    item_status_message_row(
                                        tui,
                                        item_label_width,
                                        state.tr("item.error"),
                                        &state.localize_message(error_text),
                                        error_color,
                                    );
                                }
                                item_taffy_spacer(tui, body_spacer);
                                item_file_name_input_row(
                                    tui,
                                    state,
                                    index,
                                    !item_locked,
                                    item_label_width,
                                );
                            });
                        });
                });
                ui.ctx().data_mut(|data| {
                    data.insert_temp(hover_memory_id, group_response.response.hovered());
                });
            }
        });

    if let Some(item_id) = pending_cancel_item_id {
        state.cancel_item_download(item_id);
    }
    if let Some(item_id) = pending_remove_item_id {
        state.remove_queue_item(item_id);
    }
    if let Some((item_id, kind)) = pending_export {
        open_export_dialog(state, item_id, kind);
    }
}

fn compact_music_visual_state(
    app: &AppState,
    state: CompactMusicState,
    duration_text: &str,
    playback_progress: f32,
    cache_progress: f32,
) -> (CompactRowVisualState, String) {
    match state {
        CompactMusicState::Resolving => (
            CompactRowVisualState::Resolving,
            app.tr("music.status.resolving").to_owned(),
        ),
        CompactMusicState::Buffering => {
            let label = if cache_progress > 0.0 {
                format!(
                    "{}%",
                    (cache_progress * 100.0).round().clamp(1.0, 99.0) as u32
                )
            } else {
                app.tr("music.status.buffering").to_owned()
            };
            (CompactRowVisualState::Resolving, label)
        }
        CompactMusicState::Ready => (
            if cache_progress >= 0.999 {
                CompactRowVisualState::Finished
            } else {
                CompactRowVisualState::Idle
            },
            if duration_text.trim().is_empty() {
                app.tr("music.status.ready").to_owned()
            } else {
                duration_text.to_owned()
            },
        ),
        CompactMusicState::Playing => (
            if cache_progress >= 0.999 {
                CompactRowVisualState::Finished
            } else {
                CompactRowVisualState::Playing {
                    progress: playback_progress,
                }
            },
            if cache_progress < 1.0 {
                app.tr("music.status.caching").to_owned()
            } else {
                app.tr("music.status.playing").to_owned()
            },
        ),
        CompactMusicState::Paused => (
            if cache_progress >= 0.999 {
                CompactRowVisualState::Finished
            } else {
                CompactRowVisualState::Paused {
                    progress: playback_progress,
                }
            },
            app.tr("music.status.paused").to_owned(),
        ),
        CompactMusicState::Failed => (
            CompactRowVisualState::Failed,
            app.tr("music.status.failed").to_owned(),
        ),
    }
}

fn render_queue_toolbar(ui: &mut Ui, state: &mut AppState) {
    let summary = state.queue_summary();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} {}  {} {}  {} {}  {} {}",
                state.tr("item.all"),
                summary.total,
                state.tr("item.queued"),
                summary.queued,
                state.tr("item.done"),
                summary.completed,
                state.tr("item.failed"),
                summary.failed,
            ))
            .strong(),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let response = ui.add_enabled(
                summary.total > 0,
                super::common::icon_text_button(ui, AppIcon::Eraser, state.tr("item.clear_all")),
            );
            if response.clicked() {
                state.clear_queue();
            }
        });
    });
}

fn render_empty_music_compact_item(ui: &mut Ui, state: &AppState) {
    let title = state.tr("item.add_an_audio_url");
    let output = render_music_compact_row(
        ui,
        CompactRowSpec {
            id_salt: 0,
            title,
            thumbnail_url: "",
            thumbnail_source: ThumbnailRenderSource::None,
            status_text: state.tr(UiText::AUDIO),
            visual_state: CompactRowVisualState::Idle,
            progress: 0.0,
            show_progress: false,
            is_current: false,
            is_playing: false,
            play_enabled: false,
            remove_enabled: false,
            play_label: "",
            remove_label: "",
        },
    );
    let _ = output;
}

fn render_empty_batch_item_card(
    ui: &mut Ui,
    state: &AppState,
    metadata: &crate::domain::VideoMetadata,
) {
    let item_label_width = visible_item_label_width(ui, state, false, false, false);
    ui.set_width(ui.available_width());
    ui.group(|ui| {
        let card_width = ui.available_width();
        let detail_width = (card_width - ITEM_THUMBNAIL_WIDTH - ITEM_CARD_COLUMN_GAP).max(0.0);
        let header_title = state.tr("item.add_a_video_url");
        let header_height = item_header_height(ui, header_title, false, detail_width);
        let visible_body_rows = 3usize;
        let body_target_height = (ITEM_THUMBNAIL_HEIGHT - header_height).max(0.0);
        let body_rows_gap = visible_body_rows.saturating_sub(1) as f32 * ITEM_DETAIL_ROW_GAP;
        let body_content_height =
            visible_body_rows as f32 * item_row_block_height() + body_rows_gap;
        let body_spacer = (body_target_height - body_content_height).max(0.0);

        ui.set_width(card_width);
        ui.horizontal(|ui| {
            row_thumbnail(
                ui,
                state,
                &metadata.thumbnail_url,
                state
                    .localized_thumbnail_hint(&metadata.thumbnail_hint)
                    .as_ref(),
                &metadata.duration_text,
                ThumbnailRenderSource::DirectUrl,
            );
            ui.add_space(ITEM_CARD_COLUMN_GAP);
            ui.vertical(|ui| {
                ui.set_width(detail_width);
                let _ = row_item_header(
                    ui,
                    header_title,
                    "",
                    ItemTitleVisualState::Pending,
                    false,
                    header_height,
                    false,
                    false,
                    state.tr("item.remove"),
                );
                row_empty_format_summary(
                    ui,
                    item_label_width,
                    state.tr(UiText::VIDEO),
                    state.tr("item.after_adding_choose_the_video_format_here"),
                    state.tr("item.save_as"),
                );
                row_empty_format_summary(
                    ui,
                    item_label_width,
                    state.tr(UiText::AUDIO),
                    state.tr("item.after_adding_choose_the_audio_format_here"),
                    state.tr("item.save_as"),
                );
                ui.add_space(body_spacer);
                row_empty_file_name_placeholder(ui, state, "", item_label_width);
            });
        });
    });
}

fn row_item_header(
    ui: &mut Ui,
    title: &str,
    hover_url: &str,
    state: ItemTitleVisualState,
    loading: bool,
    row_height: f32,
    delete_enabled: bool,
    item_hovered: bool,
    action_hover_text: &str,
) -> bool {
    let delete_button_width = ui.spacing().interact_size.y;
    let mut delete_clicked = false;

    ui.allocate_ui(egui::vec2(ui.available_width(), row_height), |ui| {
        StripBuilder::new(ui)
            .size(Size::remainder().at_least(0.0))
            .size(Size::exact(delete_button_width))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    row_item_title(ui, title, hover_url, state, loading);
                });
                strip.cell(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(TITLE_DELETE_TOP_PADDING);
                        let response = ui.add_enabled(
                            delete_enabled,
                            draw_delete_icon_button(delete_button_width, item_hovered),
                        );
                        if response.clicked() {
                            delete_clicked = true;
                        }
                        response.on_hover_text(action_hover_text);
                    });
                });
            });
    });

    delete_clicked
}

fn row_empty_format_summary(
    ui: &mut Ui,
    label_width: f32,
    label: &str,
    summary: &str,
    download_hover_text: &str,
) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let action_width = row_height;

    ui.allocate_ui(egui::vec2(ui.available_width(), row_height), |ui| {
        let original_spacing_x = ui.spacing().item_spacing.x;
        ui.spacing_mut().item_spacing.x = ITEM_DETAIL_COLUMN_GAP;
        StripBuilder::new(ui)
            .size(Size::exact(label_width))
            .size(Size::remainder().at_least(120.0).at_most(10_000.0))
            .size(Size::exact(action_width))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    cell_label_right(ui, label);
                });
                strip.cell(|ui| {
                    let _ = draw_picker_summary(ui, summary, 0.0, false, row_height, false);
                });
                strip.cell(|ui| {
                    ui.set_max_width(action_width);
                    draw_download_icon_button(ui, row_height, false, download_hover_text);
                });
            });
        ui.spacing_mut().item_spacing.x = original_spacing_x;
    });
}

fn row_empty_file_name_placeholder(ui: &mut Ui, state: &AppState, value: &str, label_width: f32) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let action_width = row_height;
    let placeholder = value.to_owned();

    ui.allocate_ui(egui::vec2(ui.available_width(), row_height), |ui| {
        let original_spacing_x = ui.spacing().item_spacing.x;
        ui.spacing_mut().item_spacing.x = ITEM_DETAIL_COLUMN_GAP;
        StripBuilder::new(ui)
            .size(Size::exact(label_width))
            .size(Size::remainder().at_least(120.0).at_most(10_000.0))
            .size(Size::exact(action_width))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    cell_label_right(ui, state.tr(UiText::FILE_NAME));
                });
                strip.cell(|ui| {
                    let _ = draw_file_name_display(ui, &placeholder, row_height, 0.0, false);
                });
                strip.cell(|ui| {
                    ui.set_max_width(action_width);
                    draw_output_action_arrow_button(ui, row_height, false).on_hover_text(
                        state.tr("item.file_actions_are_available_after_download_co"),
                    );
                });
            });
        ui.spacing_mut().item_spacing.x = original_spacing_x;
    });
}

fn item_header_row(
    tui: &mut Tui,
    title: &str,
    hover_url: &str,
    state: ItemTitleVisualState,
    loading: bool,
    row_height: f32,
    delete_enabled: bool,
    item_hovered: bool,
    action_hover_text: &str,
) -> bool {
    let delete_button_width = ITEM_FIELD_ROW_HEIGHT;
    let mut delete_clicked = false;

    tui.style(item_header_row_style(row_height, delete_button_width))
        .add(|tui| {
            tui.style(item_flex_cell_style()).ui(|ui| {
                row_item_title(ui, title, hover_url, state, loading);
            });
            tui.style(item_fixed_cell_style(delete_button_width))
                .ui(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(TITLE_DELETE_TOP_PADDING);
                        let response = ui.add_enabled(
                            delete_enabled,
                            draw_delete_icon_button(delete_button_width, item_hovered),
                        );
                        if response.clicked() {
                            delete_clicked = true;
                        }
                        response.on_hover_text(action_hover_text);
                    });
                });
        });

    delete_clicked
}

fn item_card_root_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::FlexStart),
        size: taffy::Size {
            width: percent(1.0),
            height: auto(),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        gap: length(ITEM_CARD_COLUMN_GAP),
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn item_thumbnail_column_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(ITEM_THUMBNAIL_WIDTH),
            height: length(ITEM_THUMBNAIL_HEIGHT),
        },
        min_size: taffy::Size {
            width: length(ITEM_THUMBNAIL_WIDTH),
            height: length(ITEM_THUMBNAIL_HEIGHT),
        },
        max_size: taffy::Size {
            width: length(ITEM_THUMBNAIL_WIDTH),
            height: length(ITEM_THUMBNAIL_HEIGHT),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn item_detail_column_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: length(0.0),
            height: auto(),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(0.0),
        },
        flex_basis: length(0.0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        gap: length(ITEM_DETAIL_ROW_GAP),
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn item_header_row_style(height: f32, action_width: f32) -> taffy::Style {
    item_row_style(height, action_width)
}

fn item_format_row_style(row_height: f32, action_width: f32) -> taffy::Style {
    item_row_style(row_height, action_width)
}

fn item_row_style(height: f32, _action_width: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(height),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        gap: length(ITEM_DETAIL_COLUMN_GAP),
        padding: length(0.0),
        margin: length(0.0),
        flex_grow: 0.0,
        flex_shrink: 0.0,
        ..Default::default()
    }
}

fn item_fixed_cell_style(width: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(width),
            height: percent(1.0),
        },
        min_size: taffy::Size {
            width: length(width),
            height: length(0.0),
        },
        max_size: taffy::Size {
            width: length(width),
            height: percent(1.0),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn item_flex_cell_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(0.0),
            height: percent(1.0),
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

fn item_taffy_spacer(tui: &mut Tui, height: f32) {
    if height <= 0.0 {
        return;
    }
    tui.style(taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0),
            height: length(height),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(height),
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
    })
    .ui(|_| {});
}

fn draw_delete_icon_button(width: f32, item_hovered: bool) -> impl egui::Widget {
    move |ui: &mut Ui| {
        let row_height = ui.spacing().interact_size.y;
        let desired_size = egui::vec2(width, row_height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
        let visuals = ui.style().interact(&response);
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
        let icon_rect =
            egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
        icon_image(AppIcon::WindowClose, icon_size, icon_color).paint_at(ui, icon_rect);

        response
    }
}

fn visible_item_body_rows(
    use_seed_compact_layout: bool,
    show_subtitle_row: bool,
    show_section_row: bool,
    show_error_row: bool,
) -> usize {
    let mut rows = 1usize;
    if !use_seed_compact_layout {
        rows += 2;
        if show_subtitle_row {
            rows += 1;
        }
        if show_section_row {
            rows += 1;
        }
    }
    if show_error_row {
        rows += 1;
    }
    rows
}

pub(super) fn item_row_block_height() -> f32 {
    ITEM_FIELD_ROW_HEIGHT + ITEM_FIELD_ROW_PADDING_Y * 2.0
}

pub(super) fn item_detail_row_gap() -> f32 {
    ITEM_DETAIL_ROW_GAP
}

fn item_header_height(ui: &Ui, title: &str, loading: bool, available_width: f32) -> f32 {
    let delete_button_width = ui.spacing().interact_size.y;
    let spinner_width = if loading {
        TITLE_SPINNER_SIZE + ui.spacing().item_spacing.x
    } else {
        0.0
    };
    let title_width = (available_width - delete_button_width - spinner_width).max(0.0);
    let title_height =
        measure_two_line_title_height(ui, title, title_width).min(max_two_line_title_height(ui));
    let spinner_height = if loading {
        (TITLE_SPINNER_TOP_PADDING.max(0.0) + TITLE_SPINNER_SIZE).max(TITLE_ROW_HEIGHT)
    } else {
        0.0
    };

    TITLE_ROW_HEIGHT.max(title_height.max(spinner_height))
}

pub(super) fn visible_item_label_width(
    ui: &Ui,
    state: &AppState,
    use_seed_compact_layout: bool,
    show_subtitle_row: bool,
    show_section_row: bool,
) -> f32 {
    let mut labels = vec![UiText::FILE_NAME];
    if !use_seed_compact_layout {
        labels.push(UiText::VIDEO);
        labels.push(UiText::AUDIO);
        if show_subtitle_row {
            labels.push(UiText::SUBTITLE);
        }
        if show_section_row {
            labels.push(UiText::SECTION);
        }
    }

    let max_width = labels
        .into_iter()
        .map(|label| text_width(ui, state.tr(label), TextStyle::Body))
        .fold(0.0, f32::max);

    max_width
}

fn row_item_title(
    ui: &mut Ui,
    title: &str,
    hover_url: &str,
    state: ItemTitleVisualState,
    loading: bool,
) {
    let color = match state {
        ItemTitleVisualState::Default => ui.visuals().text_color(),
        ItemTitleVisualState::Pending => ui.visuals().weak_text_color(),
        ItemTitleVisualState::Ready => accent_blue_for_ui(ui),
        ItemTitleVisualState::Completed => accent_green_for_ui(ui),
        ItemTitleVisualState::Failed => accent_red_for_ui(ui),
    };

    if loading {
        let spinner_width = TITLE_SPINNER_SIZE + ui.spacing().item_spacing.x;

        StripBuilder::new(ui)
            .size(Size::exact(spinner_width))
            .size(Size::remainder().at_least(0.0))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(TITLE_SPINNER_TOP_PADDING);
                        ui.add(Spinner::new().size(TITLE_SPINNER_SIZE));
                    });
                });
                strip.cell(|ui| {
                    row_item_title_text(ui, title, hover_url, color);
                });
            });
    } else {
        row_item_title_text(ui, title, hover_url, color);
    }
}

fn row_item_title_text(ui: &mut Ui, title: &str, hover_url: &str, color: egui::Color32) {
    ui.vertical(|ui| {
        ui.add_space(TITLE_TEXT_TOP_PADDING);
        let available_width = ui.available_width();
        let job = two_line_title_job(title, available_width, ITEM_TITLE_FONT_SIZE, color);
        let response = ui.add(
            egui::Label::new(job)
                .wrap_mode(TextWrapMode::Wrap)
                .selectable(false)
                .sense(Sense::hover()),
        );
        if !hover_url.is_empty() {
            response.on_hover_text(hover_url);
        }
    });
}

fn item_title_font_id() -> egui::FontId {
    egui::FontId::new(ITEM_TITLE_FONT_SIZE, egui::FontFamily::Proportional)
}

fn two_line_title_job(
    text: &str,
    max_width: f32,
    size: f32,
    color: egui::Color32,
) -> egui::text::LayoutJob {
    let font_id = egui::FontId::new(size, egui::FontFamily::Proportional);
    let mut job = egui::text::LayoutJob::simple(text.to_owned(), font_id, color, max_width);
    job.wrap.max_rows = 2;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    job
}

fn measure_two_line_title_height(ui: &Ui, text: &str, max_width: f32) -> f32 {
    wrapped_text_height(
        ui,
        text,
        max_width,
        item_title_font_id(),
        TextStyle::Body,
        Some(2),
        true,
        Some('…'),
    )
}

fn max_two_line_title_height(ui: &Ui) -> f32 {
    max_text_height_for_lines(ui, item_title_font_id(), TextStyle::Body, 2, 2.0)
        .max(TITLE_ROW_HEIGHT)
}

fn row_thumbnail(
    ui: &mut Ui,
    state: &AppState,
    thumbnail_url: &str,
    thumbnail_hint: &str,
    duration_text: &str,
    thumbnail_source: ThumbnailRenderSource,
) {
    let size = egui::vec2(ITEM_THUMBNAIL_WIDTH, ITEM_THUMBNAIL_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let visuals = &ui.style().visuals.widgets.noninteractive;

    ui.painter()
        .rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Outside);

    match thumbnail_source {
        ThumbnailRenderSource::Texture(texture) => {
            ui.painter().image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            if response.hovered() {
                response.on_hover_text(thumbnail_url);
            }
            paint_duration_badge(ui, rect, duration_text);
            return;
        }
        ThumbnailRenderSource::DirectUrl if !thumbnail_url.is_empty() => {
            let image = egui::Image::new(thumbnail_url)
                .fit_to_exact_size(size)
                .show_loading_spinner(false);
            image.paint_at(ui, rect);
            if response.hovered() {
                response.on_hover_text(thumbnail_url);
            }
            paint_duration_badge(ui, rect, duration_text);
            return;
        }
        ThumbnailRenderSource::Loading => {
            paint_thumbnail_placeholder(
                ui,
                rect,
                state.tr("item.loading_thumbnail"),
                visuals.fg_stroke.color,
            );
        }
        ThumbnailRenderSource::Failed(error) => {
            paint_thumbnail_placeholder(ui, rect, thumbnail_hint, visuals.fg_stroke.color);
            if response.hovered() {
                response.on_hover_text(error);
            }
        }
        ThumbnailRenderSource::None | ThumbnailRenderSource::DirectUrl => {
            paint_thumbnail_placeholder(ui, rect, thumbnail_hint, visuals.fg_stroke.color);
        }
    }
    paint_duration_badge(ui, rect, duration_text);
}

fn paint_thumbnail_placeholder(ui: &Ui, rect: egui::Rect, text: &str, color: egui::Color32) {
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        TextStyle::Body.resolve(ui.style()),
        color,
    );
}

fn paint_duration_badge(ui: &Ui, rect: egui::Rect, duration_text: &str) {
    let text = duration_text.trim();
    if text.is_empty() {
        return;
    }

    let galley = WidgetText::from(text).into_galley(
        ui,
        Some(TextWrapMode::Extend),
        f32::INFINITY,
        TextStyle::Small,
    );
    let padding = egui::vec2(2.0, 2.0);
    let badge_size = galley.size() + padding * 2.0;
    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(
            rect.right() - badge_size.x - 4.0,
            rect.bottom() - badge_size.y - 4.0,
        ),
        badge_size,
    );

    ui.painter().rect_filled(
        badge_rect,
        2.0,
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220),
    );
    ui.painter()
        .galley(badge_rect.min + padding, galley, egui::Color32::WHITE);
}

pub(super) fn item_download_section_summary_row(
    tui: &mut Tui,
    label_width: f32,
    label: &str,
    summary: &str,
    enabled: bool,
    on_choose: impl FnOnce(),
) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let action_width = row_height;
    let mut choose_clicked = false;

    tui.style(item_format_row_style(row_height, action_width))
        .add(|tui| {
            tui.style(item_fixed_cell_style(label_width)).ui(|ui| {
                cell_label_right(ui, label);
            });
            tui.style(item_flex_cell_style()).ui(|ui| {
                if draw_picker_summary(ui, summary, 0.0, false, row_height, enabled).clicked() {
                    choose_clicked = true;
                }
            });
            tui.style(item_fixed_cell_style(action_width)).ui(|ui| {
                ui.set_max_width(action_width);
            });
        });

    if choose_clicked {
        on_choose();
    }
}

pub(super) fn item_format_summary_row(
    tui: &mut Tui,
    label_width: f32,
    label: &str,
    summary: &str,
    progress: f32,
    show_progress: bool,
    picker_enabled: bool,
    download_enabled: bool,
    download_hover_text: &str,
    on_choose: impl FnOnce(),
    on_download: impl FnOnce(),
) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let action_width = row_height;
    let mut choose_clicked = false;
    let mut download_clicked = false;

    tui.style(item_format_row_style(row_height, action_width))
        .add(|tui| {
            tui.style(item_fixed_cell_style(label_width)).ui(|ui| {
                cell_label_right(ui, label);
            });
            tui.style(item_flex_cell_style()).ui(|ui| {
                if draw_picker_summary(
                    ui,
                    summary,
                    progress,
                    show_progress,
                    row_height,
                    picker_enabled,
                )
                .clicked()
                {
                    choose_clicked = true;
                }
            });
            tui.style(item_fixed_cell_style(action_width)).ui(|ui| {
                ui.set_max_width(action_width);
                if draw_download_icon_button(ui, row_height, download_enabled, download_hover_text)
                    .clicked()
                {
                    download_clicked = true;
                }
            });
        });

    if choose_clicked {
        on_choose();
    }
    if download_clicked {
        on_download();
    }
}

fn item_status_message_row(
    tui: &mut Tui,
    label_width: f32,
    label: &str,
    message: &str,
    color: egui::Color32,
) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let action_width = row_height;

    tui.style(item_format_row_style(row_height, action_width))
        .add(|tui| {
            tui.style(item_fixed_cell_style(label_width)).ui(|ui| {
                cell_label_right(ui, label);
            });
            tui.style(item_flex_cell_style()).ui(|ui| {
                let _ = draw_status_message(ui, message, row_height, color);
            });
            tui.style(item_fixed_cell_style(action_width)).ui(|ui| {
                ui.set_max_width(action_width);
            });
        });
}

fn item_file_name_input_row(
    tui: &mut Tui,
    state: &mut AppState,
    index: usize,
    enabled: bool,
    label_width: f32,
) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let action_width = row_height;
    let output_path = state.item_output_file_path(index);
    let output_action_mode = state.config.output_file_action_mode;
    let file_name_progress = state.item_file_name_progress(index);
    let show_file_name_progress = state.item_file_name_progress_visible(index);

    tui.style(item_format_row_style(row_height, action_width))
        .add(|tui| {
            tui.style(item_fixed_cell_style(label_width)).ui(|ui| {
                cell_label_right(ui, state.tr(UiText::FILE_NAME));
            });
            tui.style(item_flex_cell_style()).ui(|ui| {
                if enabled {
                    let response = file_name_text_edit(
                        ui,
                        &mut state.queue_items[index].selection.file_name,
                        row_height,
                        true,
                    );
                    if response.changed() {
                        let sanitized = sanitize_file_name_for_windows(
                            &state.queue_items[index].selection.file_name,
                        );
                        if state.queue_items[index].selection.file_name != sanitized {
                            state.queue_items[index].selection.file_name = sanitized;
                        }
                    }
                } else {
                    let _ = draw_file_name_display(
                        ui,
                        &state.queue_items[index].selection.file_name,
                        row_height,
                        file_name_progress,
                        show_file_name_progress,
                    );
                }
            });
            tui.style(item_fixed_cell_style(action_width)).ui(|ui| {
                ui.set_max_width(action_width);
                if enabled {
                    if let Some(output_path) = output_path.as_deref() {
                        row_output_action_button(
                            ui,
                            state,
                            output_path,
                            output_action_mode,
                            row_height,
                        );
                    }
                } else {
                    draw_output_action_arrow_button(ui, row_height, false).on_hover_text(
                        state.tr("item.file_actions_are_available_after_download_co"),
                    );
                }
            });
        });
}

pub(super) fn row_download_section_summary(
    ui: &mut Ui,
    label_width: f32,
    label: &str,
    summary: &str,
    enabled: bool,
    on_choose: impl FnOnce(),
) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let label_gap = 4.0;
    let action_width = row_height;
    let row_padding_y = ITEM_FIELD_ROW_PADDING_Y;

    ui.allocate_ui(
        egui::vec2(ui.available_width(), row_height + row_padding_y * 2.0),
        |ui| {
            let original_spacing_x = ui.spacing().item_spacing.x;
            ui.spacing_mut().item_spacing.x = 3.0;
            StripBuilder::new(ui)
                .size(Size::exact(label_width))
                .size(Size::remainder().at_least(120.0).at_most(10_000.0))
                .size(Size::exact(action_width))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let label_inner_width = (ui.available_width() - label_gap).max(0.0);
                        ui.allocate_ui(egui::vec2(label_inner_width, row_height), |ui| {
                            cell_label_right(ui, label);
                        });
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        if draw_picker_summary(ui, summary, 0.0, false, row_height, enabled)
                            .clicked()
                        {
                            on_choose();
                        }
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        ui.set_max_width(action_width);
                    });
                });
            ui.spacing_mut().item_spacing.x = original_spacing_x;
        },
    );
}

pub(super) fn row_format_summary(
    ui: &mut Ui,
    label_width: f32,
    label: &str,
    summary: &str,
    progress: f32,
    show_progress: bool,
    picker_enabled: bool,
    download_enabled: bool,
    download_hover_text: &str,
    on_choose: impl FnOnce(),
    on_download: impl FnOnce(),
) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let label_gap = 4.0;
    let action_width = row_height;
    let row_padding_y = ITEM_FIELD_ROW_PADDING_Y;

    ui.allocate_ui(
        egui::vec2(ui.available_width(), row_height + row_padding_y * 2.0),
        |ui| {
            let original_spacing_x = ui.spacing().item_spacing.x;
            ui.spacing_mut().item_spacing.x = 3.0;
            StripBuilder::new(ui)
                .size(Size::exact(label_width))
                .size(Size::remainder().at_least(120.0).at_most(10_000.0))
                .size(Size::exact(action_width))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let label_inner_width = (ui.available_width() - label_gap).max(0.0);
                        ui.allocate_ui(egui::vec2(label_inner_width, row_height), |ui| {
                            cell_label_right(ui, label);
                        });
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        if draw_picker_summary(
                            ui,
                            summary,
                            progress,
                            show_progress,
                            row_height,
                            picker_enabled,
                        )
                        .clicked()
                        {
                            on_choose();
                        }
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        ui.set_max_width(action_width);
                        if draw_download_icon_button(
                            ui,
                            row_height,
                            download_enabled,
                            download_hover_text,
                        )
                        .clicked()
                        {
                            on_download();
                        }
                    });
                });
            ui.spacing_mut().item_spacing.x = original_spacing_x;
        },
    );
}

pub(super) fn open_export_dialog(
    state: &mut AppState,
    item_id: QueueItemId,
    kind: DownloadTargetKind,
) {
    let Some(item_index) = state.queue_items.iter().position(|item| item.id == item_id) else {
        return;
    };
    if !state.item_can_export(item_index, kind) {
        return;
    }

    let mut dialog = rfd::FileDialog::new();
    if let Some(directory) = state.item_export_initial_directory(item_index) {
        dialog = dialog.set_directory(directory);
    }
    if let Some(file_name) = state.item_export_default_name(item_index, kind) {
        dialog = dialog.set_file_name(&file_name);
    }
    dialog = match kind {
        DownloadTargetKind::Video => dialog
            .add_filter(".mp4", &["mp4"])
            .add_filter(".mkv", &["mkv"])
            .add_filter(".webm", &["webm"])
            .add_filter(".mov", &["mov"])
            .add_filter(".flv", &["flv"]),
        DownloadTargetKind::Audio => dialog
            .add_filter(".mp3", &["mp3"])
            .add_filter(".m4a", &["m4a"])
            .add_filter(".flac", &["flac"])
            .add_filter(".wav", &["wav"])
            .add_filter(".opus", &["opus"])
            .add_filter(".aac", &["aac"])
            .add_filter(".vorbis", &["vorbis"])
            .add_filter(".alac", &["alac"]),
        DownloadTargetKind::Subtitle => dialog
            .add_filter(".srt", &["srt"])
            .add_filter(".vtt", &["vtt"])
            .add_filter(".ass", &["ass"])
            .add_filter(".ssa", &["ssa"])
            .add_filter(".lrc", &["lrc"])
            .add_filter(".ttml", &["ttml"])
            .add_filter(".dfxp", &["dfxp"])
            .add_filter(".json3", &["json3"])
            .add_filter(".srv3", &["srv3"])
            .add_filter(".srv2", &["srv2"])
            .add_filter(".srv1", &["srv1"]),
        DownloadTargetKind::Normal => dialog,
    };

    if let Some(path) = dialog.save_file() {
        if let Err(error) = state.start_item_export(item_id, kind, path.display().to_string()) {
            state.set_last_action_message(error);
        }
    }
}

fn row_status_message(
    ui: &mut Ui,
    label_width: f32,
    label: &str,
    message: &str,
    color: egui::Color32,
) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let action_width = row_height;
    let row_padding_y = ITEM_FIELD_ROW_PADDING_Y;
    let label_gap = 4.0;

    ui.allocate_ui(
        egui::vec2(ui.available_width(), row_height + row_padding_y * 2.0),
        |ui| {
            let original_spacing_x = ui.spacing().item_spacing.x;
            ui.spacing_mut().item_spacing.x = 3.0;
            StripBuilder::new(ui)
                .size(Size::exact(label_width))
                .size(Size::remainder().at_least(120.0).at_most(10_000.0))
                .size(Size::exact(action_width))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let label_inner_width = (ui.available_width() - label_gap).max(0.0);
                        ui.allocate_ui(egui::vec2(label_inner_width, row_height), |ui| {
                            cell_label_right(ui, label);
                        });
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let _ = draw_status_message(ui, message, row_height, color);
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        ui.set_max_width(action_width);
                    });
                });
            ui.spacing_mut().item_spacing.x = original_spacing_x;
        },
    );
}

fn draw_download_icon_button(
    ui: &mut Ui,
    row_height: f32,
    enabled: bool,
    hover_text: &str,
) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), row_height);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(desired_size, sense);
    let visuals = ui.style().interact(&response);
    let stroke_color = if enabled {
        standard_icon_color(ui)
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
    let icon_size = (rect.height() - 8.0).max(10.0);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(AppIcon::Download, icon_size, stroke_color).paint_at(ui, icon_rect);

    response.on_hover_text(hover_text)
}

fn draw_picker_summary(
    ui: &mut Ui,
    summary: &str,
    progress: f32,
    show_progress: bool,
    row_height: f32,
    enabled: bool,
) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), row_height);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(desired_size, sense);
    let visuals = ui.style().interact(&response);
    let fill_ratio = if show_progress {
        (progress / 100.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let fill_width = rect.width() * fill_ratio;
    let fill_rect =
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + fill_width, rect.max.y));
    let fill_color = egui::Color32::from_rgb(90, 168, 108);
    let normal_text = visuals.text_color();
    let inverted_text = egui::Color32::from_rgb(15, 28, 18);
    let bg_fill = if enabled {
        ui.visuals().text_edit_bg_color()
    } else {
        item_surface_bg_color(ui)
    };

    ui.painter().rect(
        rect,
        2.0,
        bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Outside,
    );
    if fill_width > 0.0 {
        ui.painter().rect_filled(fill_rect, 2.0, fill_color);
    }

    let galley = WidgetText::from(summary).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        rect.width() - 8.0,
        TextStyle::Body,
    );
    let text_pos = egui::pos2(rect.min.x + 4.0, rect.center().y - galley.size().y * 0.5);
    ui.painter().galley(text_pos, galley.clone(), normal_text);
    if fill_width > 0.0 {
        ui.painter()
            .with_clip_rect(fill_rect)
            .galley(text_pos, galley, inverted_text);
    }
    response
}

fn draw_status_message(
    ui: &mut Ui,
    message: &str,
    row_height: f32,
    color: egui::Color32,
) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), row_height);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
    let bg_fill = item_surface_bg_color(ui);

    ui.painter().rect(
        rect,
        2.0,
        bg_fill,
        egui::Stroke::new(1.0, color),
        egui::StrokeKind::Outside,
    );

    let galley = WidgetText::from(message).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        rect.width() - 8.0,
        TextStyle::Body,
    );
    let text_pos = egui::pos2(rect.min.x + 4.0, rect.center().y - galley.size().y * 0.5);
    ui.painter().galley(text_pos, galley, color);
    response.on_hover_text(message)
}

fn item_surface_bg_color(ui: &Ui) -> egui::Color32 {
    ui.visuals().panel_fill
}

fn file_name_text_edit(
    ui: &mut Ui,
    value: &mut String,
    row_height: f32,
    enabled: bool,
) -> egui::Response {
    let bg_fill = if enabled {
        ui.visuals().text_edit_bg_color()
    } else {
        item_surface_bg_color(ui)
    };

    let width = ui.available_width();
    ui.add_enabled_ui(enabled, |ui| {
        ui.add_sized(
            [width, row_height],
            TextEdit::singleline(value)
                .desired_width(width)
                .background_color(bg_fill)
                .margin(egui::Margin::symmetric(4, 2)),
        )
    })
    .inner
}

fn draw_file_name_display(
    ui: &mut Ui,
    value: &str,
    row_height: f32,
    progress: f32,
    show_progress: bool,
) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), row_height);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
    let visuals = ui.style().interact(&response);
    let fill_ratio = if show_progress {
        (progress / 100.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let fill_width = rect.width() * fill_ratio;
    let fill_rect =
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + fill_width, rect.max.y));

    ui.painter().rect(
        rect,
        2.0,
        item_surface_bg_color(ui),
        visuals.bg_stroke,
        egui::StrokeKind::Outside,
    );
    if fill_width > 0.0 {
        ui.painter()
            .rect_filled(fill_rect, 2.0, accent_green_for_ui(ui));
    }

    if !value.is_empty() {
        let galley = WidgetText::from(value).into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            rect.width() - 8.0,
            TextStyle::Body,
        );
        let text_pos = egui::pos2(rect.min.x + 4.0, rect.center().y - galley.size().y * 0.5);
        ui.painter()
            .galley(text_pos, galley.clone(), visuals.text_color());
        if fill_width > 0.0 {
            ui.painter().with_clip_rect(fill_rect).galley(
                text_pos,
                galley,
                egui::Color32::from_rgb(15, 28, 18),
            );
        }
    }

    response
}

fn row_file_name_input(
    ui: &mut Ui,
    state: &mut AppState,
    index: usize,
    enabled: bool,
    label_width: f32,
) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let action_width = row_height;
    let row_padding_y = ITEM_FIELD_ROW_PADDING_Y;
    let label_gap = 4.0;
    let output_path = state.item_output_file_path(index);
    let output_action_mode = state.config.output_file_action_mode;
    let file_name_progress = state.item_file_name_progress(index);
    let show_file_name_progress = state.item_file_name_progress_visible(index);

    ui.allocate_ui(
        egui::vec2(ui.available_width(), row_height + row_padding_y * 2.0),
        |ui| {
            let original_spacing_x = ui.spacing().item_spacing.x;
            ui.spacing_mut().item_spacing.x = 3.0;
            StripBuilder::new(ui)
                .size(Size::exact(label_width))
                .size(Size::remainder().at_least(120.0).at_most(10_000.0))
                .size(Size::exact(action_width))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let label_inner_width = (ui.available_width() - label_gap).max(0.0);
                        ui.allocate_ui(egui::vec2(label_inner_width, row_height), |ui| {
                            cell_label_right(ui, state.tr(UiText::FILE_NAME));
                        });
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        if enabled {
                            let response = file_name_text_edit(
                                ui,
                                &mut state.queue_items[index].selection.file_name,
                                row_height,
                                true,
                            );
                            if response.changed() {
                                let sanitized = sanitize_file_name_for_windows(
                                    &state.queue_items[index].selection.file_name,
                                );
                                if state.queue_items[index].selection.file_name != sanitized {
                                    state.queue_items[index].selection.file_name = sanitized;
                                }
                            }
                        } else {
                            let _ = draw_file_name_display(
                                ui,
                                &state.queue_items[index].selection.file_name,
                                row_height,
                                file_name_progress,
                                show_file_name_progress,
                            );
                        }
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        ui.set_max_width(action_width);
                        if enabled {
                            if let Some(output_path) = output_path.as_deref() {
                                row_output_action_button(
                                    ui,
                                    state,
                                    output_path,
                                    output_action_mode,
                                    row_height,
                                );
                            }
                        } else {
                            draw_output_action_arrow_button(ui, row_height, false).on_hover_text(
                                state.tr("item.file_actions_are_available_after_download_co"),
                            );
                        }
                    });
                });
            ui.spacing_mut().item_spacing.x = original_spacing_x;
        },
    );
}

fn row_output_action_button(
    ui: &mut Ui,
    state: &mut AppState,
    output_path: &str,
    mode: OutputFileActionMode,
    row_height: f32,
) {
    match mode {
        OutputFileActionMode::Menu => {
            let response = draw_output_action_arrow_button(ui, row_height, true)
                .on_hover_text(state.tr("item.file_actions"));
            egui::Popup::menu(&response).show(|ui| {
                let file_exists = output_file_exists(output_path);
                let folder_exists = output_parent_folder_exists(output_path);

                if ui
                    .add_enabled(file_exists, egui::Button::new(state.tr("item.open_file")))
                    .clicked()
                {
                    perform_output_action(ui, state, output_path, OutputAction::OpenFile);
                    ui.close();
                }
                if ui
                    .add_enabled(
                        folder_exists,
                        egui::Button::new(state.tr("item.open_folder")),
                    )
                    .clicked()
                {
                    perform_output_action(ui, state, output_path, OutputAction::OpenFolder);
                    ui.close();
                }
                if ui.button(state.tr("item.copy_path")).clicked() {
                    perform_output_action(ui, state, output_path, OutputAction::CopyPath);
                    ui.close();
                }
            });
        }
        OutputFileActionMode::OpenFolder => {
            if draw_output_action_arrow_button(ui, row_height, true)
                .on_hover_text(state.tr("item.open_folder"))
                .clicked()
            {
                perform_output_action(ui, state, output_path, OutputAction::OpenFolder);
            }
        }
        OutputFileActionMode::OpenFile => {
            if draw_output_action_arrow_button(ui, row_height, true)
                .on_hover_text(state.tr("item.open_file"))
                .clicked()
            {
                perform_output_action(ui, state, output_path, OutputAction::OpenFile);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum OutputAction {
    OpenFile,
    OpenFolder,
    CopyPath,
}

fn perform_output_action(
    ui: &mut Ui,
    state: &mut AppState,
    output_path: &str,
    action: OutputAction,
) {
    match action {
        OutputAction::OpenFile => match open_output_file(output_path) {
            Ok(()) => state.set_last_action_message(state.tr("item.opened_output_file")),
            Err(file_error) => match open_output_folder(output_path) {
                Ok(()) => state.set_last_action_message(
                    state.tr("item.file_not_found_opened_the_output_location"),
                ),
                Err(folder_error) => {
                    state.set_last_action_message(format!("{file_error}; {folder_error}"));
                }
            },
        },
        OutputAction::OpenFolder => match open_output_folder(output_path) {
            Ok(()) => state.set_last_action_message(state.tr("item.opened_output_location")),
            Err(error) => state.set_last_action_message(error),
        },
        OutputAction::CopyPath => {
            ui.ctx().copy_text(output_path.to_owned());
            state.set_last_action_message(state.tr("item.copied_output_path"));
        }
    }
}

fn draw_output_action_arrow_button(ui: &mut Ui, row_height: f32, enabled: bool) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), row_height);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(desired_size, sense);
    let visuals = ui.style().interact(&response);
    let text_color = if enabled {
        standard_icon_color(ui)
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
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "→",
        TextStyle::Body.resolve(ui.style()),
        text_color,
    );

    response
}

fn row_file_name_placeholder(ui: &mut Ui, state: &AppState, value: &str, label_width: f32) {
    let row_height = ITEM_FIELD_ROW_HEIGHT;
    let action_width = row_height;
    let row_padding_y = ITEM_FIELD_ROW_PADDING_Y;
    let label_gap = 4.0;
    let placeholder = value.to_owned();

    ui.allocate_ui(
        egui::vec2(ui.available_width(), row_height + row_padding_y * 2.0),
        |ui| {
            let original_spacing_x = ui.spacing().item_spacing.x;
            ui.spacing_mut().item_spacing.x = 3.0;
            StripBuilder::new(ui)
                .size(Size::exact(label_width))
                .size(Size::remainder().at_least(120.0).at_most(10_000.0))
                .size(Size::exact(action_width))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let label_inner_width = (ui.available_width() - label_gap).max(0.0);
                        ui.allocate_ui(egui::vec2(label_inner_width, row_height), |ui| {
                            cell_label_right(ui, state.tr(UiText::FILE_NAME));
                        });
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let _ = draw_file_name_display(ui, &placeholder, row_height, 0.0, false);
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        ui.set_max_width(action_width);
                        draw_output_action_arrow_button(ui, row_height, false).on_hover_text(
                            state.tr("item.file_actions_are_available_after_download_co"),
                        );
                    });
                });
            ui.spacing_mut().item_spacing.x = original_spacing_x;
        },
    );
}
