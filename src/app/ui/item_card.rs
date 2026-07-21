use eframe::egui::{
    self, Align, Layout, RichText, ScrollArea, Sense, Spinner, TextEdit, TextStyle, TextWrapMode,
    Ui, WidgetText,
};
use egui_extras::{Size, StripBuilder};
use egui_taffy::{Tui, TuiBuilderLogic as _, taffy, tui};

use crate::app::state::{
    AppState, FormatPickerKind, ItemTitleVisualState, ThumbnailRenderSource,
    sanitize_file_name_for_windows,
};
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::app::widgets::url_input::{accent_blue_for_ui, accent_green_for_ui, accent_red_for_ui};
use crate::domain::DownloadContainerPreference;
use crate::infrastructure::DownloadTargetKind;

use super::common::{UiText, cell_label_right};
use super::item_card_compact::{render_empty_music_compact_item, render_music_queue_item_row};
pub(super) use super::item_card_output_actions::open_export_dialog;
use super::item_card_output_actions::{draw_output_action_arrow_button, row_output_action_button};
use super::item_card_template::{
    item_card_root_style, item_detail_column_style, item_thumbnail_column_style,
};
use super::xaml_layout_contracts::{LayoutLength, LayoutSize, SingleLineControlRowContract};
use super::{semantic_ui_metrics, xaml_taffy_styles};

fn item_card_field_row_contract() -> SingleLineControlRowContract {
    semantic_ui_metrics::xaml_single_line_control_row_contract_from_height(
        semantic_ui_metrics::item_card_field_row_height(),
    )
}

fn item_card_action_icon_button_size_for_row(row: SingleLineControlRowContract) -> LayoutSize {
    row.measure_auto_width_ui_element(
        semantic_ui_metrics::xaml_icon_button_ui_element_from_row_contract(row),
    )
}

fn item_card_action_column_width_for_row(row: SingleLineControlRowContract) -> f32 {
    item_card_action_icon_button_size_for_row(row).width
}

fn item_card_stretch_width_action_icon_button_size_for_available_width(
    row: SingleLineControlRowContract,
    available_width: f32,
) -> LayoutSize {
    row.measure_stretch_width_ui_element(
        semantic_ui_metrics::xaml_icon_button_ui_element_from_row_contract(row)
            .width(LayoutLength::Star(1.0)),
        available_width,
    )
}

fn item_card_field_label_size_for_available_width(
    row: SingleLineControlRowContract,
    available_width: f32,
) -> LayoutSize {
    let label_width =
        semantic_ui_metrics::item_card_label_inner_width_for_available_width(available_width);
    row.measure_stretch_width_ui_element(
        semantic_ui_metrics::xaml_label_ui_element_from_row_contract_and_width(row, 0.0),
        label_width,
    )
}

fn item_card_stretch_width_field_size_for_available_width(
    row: SingleLineControlRowContract,
    available_width: f32,
) -> LayoutSize {
    row.measure_stretch_width_ui_element(
        semantic_ui_metrics::xaml_stretch_width_ui_element_from_row_contract(row),
        available_width,
    )
}

fn item_card_single_line_text_input_size_for_available_width(
    row: SingleLineControlRowContract,
    available_width: f32,
) -> LayoutSize {
    row.measure_stretch_width_ui_element(
        semantic_ui_metrics::xaml_single_line_text_input_ui_element_from_row_contract(row),
        available_width,
    )
}

fn item_card_layout_size_to_vec2(size: LayoutSize) -> egui::Vec2 {
    egui::vec2(size.width, size.height)
}

pub(super) fn render_batch_list(ui: &mut Ui, state: &mut AppState, bottom_safe_area: f32) {
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
                    if let Some(item_id) = render_music_queue_item_row(ui, state, index) {
                        pending_remove_item_id = Some(item_id);
                    }
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
                        semantic_ui_metrics::item_card_detail_width_for_card_width(card_width);
                    let header_height = item_header_height(ui, &title, title_loading, detail_width);
                    let visible_body_rows = visible_item_body_rows(
                        use_seed_compact_layout,
                        show_subtitle_row,
                        show_section_row,
                        item_error_text.is_some(),
                    );
                    let body_target_height =
                        semantic_ui_metrics::item_card_body_target_height_for_header_height(
                            header_height,
                        );
                    let body_content_height =
                        semantic_ui_metrics::item_card_body_content_height_for_visible_row_count(
                            visible_body_rows,
                        );
                    let body_spacer =
                        semantic_ui_metrics::item_card_body_spacer_height_for_target_and_content(
                            body_target_height,
                            body_content_height,
                        );
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
                                        state.ui_i18n_text_for_key(UiText::VIDEO),
                                        &state.localize_message(&video_summary),
                                        video_progress,
                                        show_av_progress,
                                        !item_locked,
                                        state.item_can_export(index, DownloadTargetKind::Video),
                                        || state.open_format_picker(index, FormatPickerKind::Video),
                                        || {
                                            pending_export =
                                                Some((item_id, DownloadTargetKind::Video))
                                        },
                                    );
                                    item_format_summary_row(
                                        tui,
                                        item_label_width,
                                        state.ui_i18n_text_for_key(UiText::AUDIO),
                                        &state.localize_message(&audio_summary),
                                        audio_progress,
                                        show_av_progress,
                                        !audio_locked && !item_locked,
                                        state.item_can_export(index, DownloadTargetKind::Audio),
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
                                            state.ui_i18n_text_for_key(UiText::SUBTITLE),
                                            &state.localize_message(&subtitle_summary),
                                            subtitle_progress,
                                            show_subtitle_progress,
                                            !item_locked,
                                            state.item_can_export(
                                                index,
                                                DownloadTargetKind::Subtitle,
                                            ),
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
                                            state.ui_i18n_text_for_key(UiText::SECTION),
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
                                        state.ui_i18n_text_for_key("item.error"),
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
            ui.add_space(bottom_safe_area.max(0.0));
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

fn render_queue_toolbar(ui: &mut Ui, state: &mut AppState) {
    let summary = state.queue_summary();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} {}  {} {}  {} {}  {} {}",
                state.ui_i18n_text_for_key("item.all"),
                summary.total,
                state.ui_i18n_text_for_key("item.queued"),
                summary.queued,
                state.ui_i18n_text_for_key("item.done"),
                summary.completed,
                state.ui_i18n_text_for_key("item.failed"),
                summary.failed,
            ))
            .strong(),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let response = ui.add_enabled(
                summary.total > 0,
                super::common::icon_text_button(
                    ui,
                    AppIcon::Eraser,
                    state.ui_i18n_text_for_key("item.clear_all"),
                ),
            );
            if response.clicked() {
                state.clear_queue();
            }
        });
    });
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
        let detail_width = semantic_ui_metrics::item_card_detail_width_for_card_width(card_width);
        let header_title = state.ui_i18n_text_for_key("item.add_a_video_url");
        let header_height = item_header_height(ui, header_title, false, detail_width);
        let visible_body_rows = 3usize;
        let body_target_height =
            semantic_ui_metrics::item_card_body_target_height_for_header_height(header_height);
        let body_content_height =
            semantic_ui_metrics::item_card_body_content_height_for_visible_row_count(
                visible_body_rows,
            );
        let body_spacer = semantic_ui_metrics::item_card_body_spacer_height_for_target_and_content(
            body_target_height,
            body_content_height,
        );

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
            ui.add_space(semantic_ui_metrics::item_card_column_gap());
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
                );
                row_empty_format_summary(
                    ui,
                    item_label_width,
                    state.ui_i18n_text_for_key(UiText::VIDEO),
                    state.ui_i18n_text_for_key("item.after_adding_choose_the_video_format_here"),
                );
                row_empty_format_summary(
                    ui,
                    item_label_width,
                    state.ui_i18n_text_for_key(UiText::AUDIO),
                    state.ui_i18n_text_for_key("item.after_adding_choose_the_audio_format_here"),
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
) -> bool {
    let delete_button_width =
        semantic_ui_metrics::item_card_title_delete_button_width_from_current_control_metrics(ui);
    let mut delete_clicked = false;

    ui.allocate_ui(egui::vec2(ui.available_width(), row_height), |ui| {
        StripBuilder::new(ui)
            .size(
                Size::remainder()
                    .at_least(semantic_ui_metrics::item_card_zero_remainder_column_minimum_width()),
            )
            .size(Size::exact(delete_button_width))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    row_item_title(ui, title, hover_url, state, loading);
                });
                strip.cell(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(semantic_ui_metrics::item_card_title_delete_top_padding());
                        let response = ui.add_enabled(
                            delete_enabled,
                            draw_delete_icon_button(delete_button_width, item_hovered),
                        );
                        if response.clicked() {
                            delete_clicked = true;
                        }
                    });
                });
            });
    });

    delete_clicked
}

fn row_empty_format_summary(ui: &mut Ui, label_width: f32, label: &str, summary: &str) {
    let row_contract = item_card_field_row_contract();
    let row_height = row_contract.height;
    let action_width = item_card_action_column_width_for_row(row_contract);

    ui.allocate_ui(egui::vec2(ui.available_width(), row_height), |ui| {
        let original_spacing_x = ui.spacing().item_spacing.x;
        ui.spacing_mut().item_spacing.x = semantic_ui_metrics::item_card_detail_column_gap();
        StripBuilder::new(ui)
            .size(Size::exact(label_width))
            .size(
                Size::remainder()
                    .at_least(semantic_ui_metrics::item_card_remainder_column_minimum_width())
                    .at_most(semantic_ui_metrics::item_card_remainder_column_maximum_width()),
            )
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
                    draw_download_icon_button(ui, row_height, false);
                });
            });
        ui.spacing_mut().item_spacing.x = original_spacing_x;
    });
}

fn row_empty_file_name_placeholder(ui: &mut Ui, state: &AppState, value: &str, label_width: f32) {
    let row_contract = item_card_field_row_contract();
    let row_height = row_contract.height;
    let action_width = item_card_action_column_width_for_row(row_contract);
    let placeholder = value.to_owned();

    ui.allocate_ui(egui::vec2(ui.available_width(), row_height), |ui| {
        let original_spacing_x = ui.spacing().item_spacing.x;
        ui.spacing_mut().item_spacing.x = semantic_ui_metrics::item_card_detail_column_gap();
        StripBuilder::new(ui)
            .size(Size::exact(label_width))
            .size(
                Size::remainder()
                    .at_least(semantic_ui_metrics::item_card_remainder_column_minimum_width())
                    .at_most(semantic_ui_metrics::item_card_remainder_column_maximum_width()),
            )
            .size(Size::exact(action_width))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    cell_label_right(ui, state.ui_i18n_text_for_key(UiText::FILE_NAME));
                });
                strip.cell(|ui| {
                    let _ = draw_file_name_display(ui, &placeholder, row_height, 0.0, false);
                });
                strip.cell(|ui| {
                    ui.set_max_width(action_width);
                    draw_output_action_arrow_button(ui, row_height, false);
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
) -> bool {
    let delete_button_width = item_card_action_column_width_for_row(item_card_field_row_contract());
    let mut delete_clicked = false;

    tui.style(item_header_row_style(row_height, delete_button_width))
        .add(|tui| {
            tui.style(item_flex_cell_style()).ui(|ui| {
                row_item_title(ui, title, hover_url, state, loading);
            });
            tui.style(item_fixed_cell_style(delete_button_width))
                .ui(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(semantic_ui_metrics::item_card_title_delete_top_padding());
                        let response = ui.add_enabled(
                            delete_enabled,
                            draw_delete_icon_button(delete_button_width, item_hovered),
                        );
                        if response.clicked() {
                            delete_clicked = true;
                        }
                    });
                });
        });

    delete_clicked
}

fn item_header_row_style(height: f32, action_width: f32) -> taffy::Style {
    item_row_style(height, action_width)
}

fn item_format_row_style(row_height: f32, action_width: f32) -> taffy::Style {
    item_row_style(row_height, action_width)
}

fn item_row_style(height: f32, _action_width: f32) -> taffy::Style {
    xaml_taffy_styles::xaml_fixed_height_row_style(
        SingleLineControlRowContract::new(height),
        semantic_ui_metrics::item_card_detail_column_gap(),
    )
}

fn item_fixed_cell_style(width: f32) -> taffy::Style {
    xaml_taffy_styles::xaml_fixed_width_stretch_height_gap_style(width)
}

fn item_flex_cell_style() -> taffy::Style {
    xaml_taffy_styles::xaml_flex_spacer_cell_style(SingleLineControlRowContract::new(1.0))
}

fn item_taffy_spacer(tui: &mut Tui, height: f32) {
    if height <= 0.0 {
        return;
    }
    tui.style(xaml_taffy_styles::xaml_fixed_height_block_style(height))
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
            semantic_ui_metrics::item_card_icon_button_corner_radius(),
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Outside,
        );

        let icon_size = semantic_ui_metrics::item_card_delete_icon_size();
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
    semantic_ui_metrics::item_card_row_block_height()
}

pub(super) fn item_detail_row_gap() -> f32 {
    semantic_ui_metrics::item_card_detail_row_gap()
}

fn item_header_height(ui: &Ui, title: &str, loading: bool, available_width: f32) -> f32 {
    let delete_button_width =
        semantic_ui_metrics::item_card_title_delete_button_width_from_current_control_metrics(ui);
    let spinner_width = if loading {
        semantic_ui_metrics::item_card_title_spinner_width_from_current_spacing(ui)
    } else {
        0.0
    };
    let title_width = (available_width - delete_button_width - spinner_width).max(0.0);
    let title_height =
        measure_two_line_title_height(ui, title, title_width).min(max_two_line_title_height(ui));
    let spinner_height = if loading {
        semantic_ui_metrics::item_card_loading_spinner_height()
    } else {
        0.0
    };

    semantic_ui_metrics::item_card_title_height_for_measured_parts(title_height, spinner_height)
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

    semantic_ui_metrics::item_card_visible_label_width_for_translated_label_keys(ui, state, &labels)
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
        let spinner_width =
            semantic_ui_metrics::item_card_title_spinner_width_from_current_spacing(ui);

        StripBuilder::new(ui)
            .size(Size::exact(spinner_width))
            .size(
                Size::remainder()
                    .at_least(semantic_ui_metrics::item_card_zero_remainder_column_minimum_width()),
            )
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(semantic_ui_metrics::item_card_title_spinner_top_padding());
                        ui.add(
                            Spinner::new()
                                .size(semantic_ui_metrics::item_card_title_spinner_size()),
                        );
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

fn row_item_title_text(ui: &mut Ui, title: &str, _hover_url: &str, color: egui::Color32) {
    ui.vertical(|ui| {
        ui.add_space(semantic_ui_metrics::item_card_title_text_top_padding());
        let available_width = ui.available_width();
        let job = two_line_title_job(
            title,
            available_width,
            semantic_ui_metrics::item_card_title_font_size(),
            color,
        );
        ui.add(
            egui::Label::new(job)
                .wrap_mode(TextWrapMode::Wrap)
                .selectable(false)
                .sense(Sense::hover()),
        );
    });
}

fn item_title_font_id() -> egui::FontId {
    egui::FontId::new(
        semantic_ui_metrics::item_card_title_font_size(),
        egui::FontFamily::Proportional,
    )
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
    semantic_ui_metrics::item_card_two_line_title_height_for_visible_text_and_width(
        ui,
        text,
        max_width,
        item_title_font_id(),
    )
}

fn max_two_line_title_height(ui: &Ui) -> f32 {
    semantic_ui_metrics::item_card_maximum_two_line_title_height_for_font(ui, item_title_font_id())
}

fn row_thumbnail(
    ui: &mut Ui,
    state: &AppState,
    thumbnail_url: &str,
    thumbnail_hint: &str,
    duration_text: &str,
    thumbnail_source: ThumbnailRenderSource,
) {
    let size = semantic_ui_metrics::item_card_thumbnail_size();
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
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
            paint_duration_badge(ui, rect, duration_text);
            return;
        }
        ThumbnailRenderSource::DirectUrl if !thumbnail_url.is_empty() => {
            let image = egui::Image::new(thumbnail_url)
                .fit_to_exact_size(size)
                .show_loading_spinner(false);
            image.paint_at(ui, rect);
            paint_duration_badge(ui, rect, duration_text);
            return;
        }
        ThumbnailRenderSource::Loading => {
            paint_thumbnail_placeholder(
                ui,
                rect,
                state.ui_i18n_text_for_key("item.loading_thumbnail"),
                visuals.fg_stroke.color,
            );
        }
        ThumbnailRenderSource::Failed(_error) => {
            paint_thumbnail_placeholder(ui, rect, thumbnail_hint, visuals.fg_stroke.color);
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
    let padding = semantic_ui_metrics::item_card_duration_badge_padding();
    let badge_size = galley.size() + padding * 2.0;
    let badge_rect =
        semantic_ui_metrics::item_card_duration_badge_rect_for_container_and_content_size(
            rect, badge_size,
        );

    ui.painter().rect_filled(
        badge_rect,
        semantic_ui_metrics::item_card_duration_badge_corner_radius(),
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220),
    );
    ui.painter()
        .galley(badge_rect.min + padding, galley, egui::Color32::WHITE);
}

fn draw_download_icon_button(ui: &mut Ui, row_height: f32, enabled: bool) -> egui::Response {
    let row_contract =
        semantic_ui_metrics::xaml_single_line_control_row_contract_from_height(row_height);
    let desired_size = item_card_layout_size_to_vec2(
        item_card_stretch_width_action_icon_button_size_for_available_width(
            row_contract,
            ui.available_width(),
        ),
    );
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
        semantic_ui_metrics::item_card_icon_button_corner_radius(),
        visuals.bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Outside,
    );
    let icon_size = semantic_ui_metrics::item_card_download_icon_size_for_row_height(rect.height());
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    icon_image(AppIcon::Download, icon_size, stroke_color).paint_at(ui, icon_rect);

    response
}

fn draw_picker_summary(
    ui: &mut Ui,
    summary: &str,
    progress: f32,
    show_progress: bool,
    row_height: f32,
    enabled: bool,
) -> egui::Response {
    let row_contract =
        semantic_ui_metrics::xaml_single_line_control_row_contract_from_height(row_height);
    let desired_size = item_card_layout_size_to_vec2(
        item_card_stretch_width_field_size_for_available_width(row_contract, ui.available_width()),
    );
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

    let corner_radius = semantic_ui_metrics::item_card_field_corner_radius();
    ui.painter().rect_filled(rect, corner_radius, bg_fill);
    if fill_width > 0.0 {
        ui.painter()
            .rect_filled(fill_rect, corner_radius, fill_color);
    }
    ui.painter().rect(
        rect,
        corner_radius,
        egui::Color32::TRANSPARENT,
        visuals.bg_stroke,
        egui::StrokeKind::Outside,
    );

    let galley = WidgetText::from(summary).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        semantic_ui_metrics::item_card_field_text_available_width(rect),
        TextStyle::Body,
    );
    let text_pos =
        semantic_ui_metrics::item_card_field_text_position_for_galley(rect, galley.size());
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
    let row_contract =
        semantic_ui_metrics::xaml_single_line_control_row_contract_from_height(row_height);
    let desired_size = item_card_layout_size_to_vec2(
        item_card_stretch_width_field_size_for_available_width(row_contract, ui.available_width()),
    );
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
    let bg_fill = item_surface_bg_color(ui);

    ui.painter().rect(
        rect,
        semantic_ui_metrics::item_card_field_corner_radius(),
        bg_fill,
        egui::Stroke::new(
            semantic_ui_metrics::item_card_status_message_stroke_width(),
            color,
        ),
        egui::StrokeKind::Outside,
    );

    let galley = WidgetText::from(message).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        semantic_ui_metrics::item_card_field_text_available_width(rect),
        TextStyle::Body,
    );
    let text_pos =
        semantic_ui_metrics::item_card_field_text_position_for_galley(rect, galley.size());
    ui.painter().galley(text_pos, galley, color);
    response
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

    let row_contract =
        semantic_ui_metrics::xaml_single_line_control_row_contract_from_height(row_height);
    let size = item_card_single_line_text_input_size_for_available_width(
        row_contract,
        ui.available_width(),
    );
    ui.add_enabled_ui(enabled, |ui| {
        ui.add_sized(
            size.to_array(),
            TextEdit::singleline(value)
                .desired_width(size.width)
                .background_color(bg_fill)
                .margin(semantic_ui_metrics::item_card_field_text_edit_margin()),
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
    let row_contract =
        semantic_ui_metrics::xaml_single_line_control_row_contract_from_height(row_height);
    let desired_size = item_card_layout_size_to_vec2(
        item_card_stretch_width_field_size_for_available_width(row_contract, ui.available_width()),
    );
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

    let corner_radius = semantic_ui_metrics::item_card_field_corner_radius();
    ui.painter()
        .rect_filled(rect, corner_radius, item_surface_bg_color(ui));
    if fill_width > 0.0 {
        ui.painter()
            .rect_filled(fill_rect, corner_radius, accent_green_for_ui(ui));
    }
    ui.painter().rect(
        rect,
        corner_radius,
        egui::Color32::TRANSPARENT,
        visuals.bg_stroke,
        egui::StrokeKind::Outside,
    );

    if !value.is_empty() {
        let galley = WidgetText::from(value).into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            semantic_ui_metrics::item_card_field_text_available_width(rect),
            TextStyle::Body,
        );
        let text_pos =
            semantic_ui_metrics::item_card_field_text_position_for_galley(rect, galley.size());
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

pub(super) fn item_download_section_summary_row(
    tui: &mut Tui,
    label_width: f32,
    label: &str,
    summary: &str,
    enabled: bool,
    on_choose: impl FnOnce(),
) {
    let row_contract = item_card_field_row_contract();
    let row_height = row_contract.height;
    let action_width = item_card_action_column_width_for_row(row_contract);
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
    on_choose: impl FnOnce(),
    on_download: impl FnOnce(),
) {
    let row_contract = item_card_field_row_contract();
    let row_height = row_contract.height;
    let action_width = item_card_action_column_width_for_row(row_contract);
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
                if draw_download_icon_button(ui, row_height, download_enabled).clicked() {
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
    let row_contract = item_card_field_row_contract();
    let row_height = row_contract.height;
    let action_width = item_card_action_column_width_for_row(row_contract);

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
    let row_contract = item_card_field_row_contract();
    let row_height = row_contract.height;
    let action_width = item_card_action_column_width_for_row(row_contract);
    let output_path = state.item_output_file_path(index);
    let output_action_mode = state.config.output_file_action_mode;
    let file_name_progress = state.item_file_name_progress(index);
    let show_file_name_progress = state.item_file_name_progress_visible(index);
    let show_container_picker = enabled && state.item_supports_webm_download_container(index);
    let selected_container = state.resolved_item_download_container(index);
    let item_id = state.queue_items[index].id;
    let mut pending_container = None;

    tui.style(item_format_row_style(row_height, action_width))
        .add(|tui| {
            tui.style(item_fixed_cell_style(label_width)).ui(|ui| {
                cell_label_right(ui, state.ui_i18n_text_for_key(UiText::FILE_NAME));
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
            if show_container_picker {
                tui.style(item_fixed_cell_style(
                    semantic_ui_metrics::item_card_output_container_picker_width(),
                ))
                .ui(|ui| {
                    let selected = selected_container.unwrap_or(DownloadContainerPreference::Mkv);
                    pending_container = draw_item_output_container_picker(ui, item_id, selected);
                });
            }
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
                    draw_output_action_arrow_button(ui, row_height, false);
                }
            });
        });

    if let Some(container) = pending_container {
        state.set_item_download_container_preference(index, container);
    }
}

pub(super) fn draw_item_output_container_picker(
    ui: &mut Ui,
    item_id: crate::domain::QueueItemId,
    selected_container: DownloadContainerPreference,
) -> Option<DownloadContainerPreference> {
    ui.push_id(("item-output-container", item_id), |ui| {
        let desired_size = ui.available_size_before_wrap();
        let label = format!(".{}", selected_container.extension().unwrap_or("mkv"));
        let response = ui.add_sized(desired_size, egui::Button::new(label));
        let mut pending = None;

        egui::Popup::menu(&response)
            .width(response.rect.width())
            .show(|ui| {
                ui.set_min_width(response.rect.width());
                for (container, label) in [
                    (DownloadContainerPreference::Mkv, ".mkv"),
                    (DownloadContainerPreference::Webm, ".webm"),
                ] {
                    if ui
                        .selectable_label(selected_container == container, label)
                        .clicked()
                    {
                        pending = (container != selected_container).then_some(container);
                        ui.close();
                    }
                }
            });
        pending
    })
    .inner
}

pub(super) fn row_download_section_summary(
    ui: &mut Ui,
    label_width: f32,
    label: &str,
    summary: &str,
    enabled: bool,
    on_choose: impl FnOnce(),
) {
    let row_contract = item_card_field_row_contract();
    let row_height = row_contract.height;
    let action_width = item_card_action_column_width_for_row(row_contract);
    let row_padding_y = semantic_ui_metrics::item_card_field_row_vertical_padding();

    ui.allocate_ui(
        semantic_ui_metrics::item_card_field_row_total_size_for_available_width(
            ui.available_width(),
            row_height,
        ),
        |ui| {
            let original_spacing_x = ui.spacing().item_spacing.x;
            ui.spacing_mut().item_spacing.x = semantic_ui_metrics::item_card_detail_column_gap();
            StripBuilder::new(ui)
                .size(Size::exact(label_width))
                .size(
                    Size::remainder()
                        .at_least(semantic_ui_metrics::item_card_remainder_column_minimum_width())
                        .at_most(semantic_ui_metrics::item_card_remainder_column_maximum_width()),
                )
                .size(Size::exact(action_width))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let label_size = item_card_field_label_size_for_available_width(
                            row_contract,
                            ui.available_width(),
                        );
                        ui.allocate_ui(item_card_layout_size_to_vec2(label_size), |ui| {
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
    on_choose: impl FnOnce(),
    on_download: impl FnOnce(),
) {
    let row_contract = item_card_field_row_contract();
    let row_height = row_contract.height;
    let action_width = item_card_action_column_width_for_row(row_contract);
    let row_padding_y = semantic_ui_metrics::item_card_field_row_vertical_padding();

    ui.allocate_ui(
        semantic_ui_metrics::item_card_field_row_total_size_for_available_width(
            ui.available_width(),
            row_height,
        ),
        |ui| {
            let original_spacing_x = ui.spacing().item_spacing.x;
            ui.spacing_mut().item_spacing.x = semantic_ui_metrics::item_card_detail_column_gap();
            StripBuilder::new(ui)
                .size(Size::exact(label_width))
                .size(
                    Size::remainder()
                        .at_least(semantic_ui_metrics::item_card_remainder_column_minimum_width())
                        .at_most(semantic_ui_metrics::item_card_remainder_column_maximum_width()),
                )
                .size(Size::exact(action_width))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let label_size = item_card_field_label_size_for_available_width(
                            row_contract,
                            ui.available_width(),
                        );
                        ui.allocate_ui(item_card_layout_size_to_vec2(label_size), |ui| {
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
                        if draw_download_icon_button(ui, row_height, download_enabled).clicked() {
                            on_download();
                        }
                    });
                });
            ui.spacing_mut().item_spacing.x = original_spacing_x;
        },
    );
}

fn row_file_name_placeholder(ui: &mut Ui, state: &AppState, value: &str, label_width: f32) {
    let row_contract = item_card_field_row_contract();
    let row_height = row_contract.height;
    let action_width = item_card_action_column_width_for_row(row_contract);
    let row_padding_y = semantic_ui_metrics::item_card_field_row_vertical_padding();
    let placeholder = value.to_owned();

    ui.allocate_ui(
        semantic_ui_metrics::item_card_field_row_total_size_for_available_width(
            ui.available_width(),
            row_height,
        ),
        |ui| {
            let original_spacing_x = ui.spacing().item_spacing.x;
            ui.spacing_mut().item_spacing.x = semantic_ui_metrics::item_card_detail_column_gap();
            StripBuilder::new(ui)
                .size(Size::exact(label_width))
                .size(
                    Size::remainder()
                        .at_least(semantic_ui_metrics::item_card_remainder_column_minimum_width())
                        .at_most(semantic_ui_metrics::item_card_remainder_column_maximum_width()),
                )
                .size(Size::exact(action_width))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let label_size = item_card_field_label_size_for_available_width(
                            row_contract,
                            ui.available_width(),
                        );
                        ui.allocate_ui(item_card_layout_size_to_vec2(label_size), |ui| {
                            cell_label_right(ui, state.ui_i18n_text_for_key(UiText::FILE_NAME));
                        });
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        let _ = draw_file_name_display(ui, &placeholder, row_height, 0.0, false);
                    });
                    strip.cell(|ui| {
                        ui.add_space(row_padding_y);
                        ui.set_max_width(action_width);
                        draw_output_action_arrow_button(ui, row_height, false);
                    });
                });
            ui.spacing_mut().item_spacing.x = original_spacing_x;
        },
    );
}
