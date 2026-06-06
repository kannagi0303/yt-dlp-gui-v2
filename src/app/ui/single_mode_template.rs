use eframe::egui::Ui;
use egui_taffy::{Tui, TuiBuilderLogic as _, taffy};

use crate::app::state::AppState;

use super::single_mode::{
    SingleModeLayoutMetrics, SingleModeView, render_description_field_at, render_title_field_at,
};
use super::single_mode_format_rows::render_format_rows;
use super::single_mode_preview::{
    render_download_thumbnail_checkbox_at, render_right_info_at, render_thumbnail_at,
};
use super::xaml_taffy_styles;
use super::xaml_template_renderer::{TemplateBlockSlot, show_template};
use super::xaml_ui_nodes::{
    self, TemplateAxis, TemplateSizing, block, cols, fill, fixed_px, gap, rows, star,
};

const LEFT_COLUMN_STAR: f32 = 5.0;
const RIGHT_COLUMN_STAR: f32 = 3.0;
const COLUMN_GAP_PX: f32 = 6.0;
const DESCRIPTION_TO_FORMAT_GAP_PX: f32 = 4.0;
const FORMAT_BOTTOM_GAP_PX: f32 = 5.0;
const THUMBNAIL_TO_CHECKBOX_GAP_PX: f32 = 3.0;

pub(super) type SingleModeTemplate = xaml_ui_nodes::TemplateNode<SingleModeNode>;

pub(super) fn right_column_width_for_layout_width(layout_width: f32) -> f32 {
    let content_width = (layout_width - COLUMN_GAP_PX).max(0.0);
    let total_star = LEFT_COLUMN_STAR + RIGHT_COLUMN_STAR;
    if total_star <= 0.0 {
        return 0.0;
    }
    content_width * RIGHT_COLUMN_STAR / total_star
}

#[derive(Clone, Copy)]
pub(super) enum SingleModeNode {
    TitleField,
    DescriptionField,
    FormatRows,
    Thumbnail,
    DownloadThumbnailCheckbox,
    RightInfo,
    Empty,
}

pub(super) fn build_single_mode_template(metrics: &SingleModeLayoutMetrics) -> SingleModeTemplate {
    use SingleModeNode::*;

    // This template intentionally describes only coarse layout slots.  Thumbnail
    // badges, text padding, row internals, and hover details stay in their own
    // painters instead of becoming tiny template leaves.
    cols([
        star(
            LEFT_COLUMN_STAR,
            rows([
                fixed_px(metrics.title_height, block(TitleField)),
                fill(block(DescriptionField)),
                gap(DESCRIPTION_TO_FORMAT_GAP_PX),
                fixed_px(metrics.format_area_height, block(FormatRows)),
                gap(FORMAT_BOTTOM_GAP_PX),
            ]),
        ),
        gap(COLUMN_GAP_PX),
        star(
            RIGHT_COLUMN_STAR,
            rows([
                fixed_px(metrics.right_thumbnail_height, block(Thumbnail)),
                gap(THUMBNAIL_TO_CHECKBOX_GAP_PX),
                fixed_px(
                    metrics.right_checkbox_height,
                    block(DownloadThumbnailCheckbox),
                ),
                fill(block(Empty)),
                fixed_px(metrics.right_info_height, block(RightInfo)),
            ]),
        ),
    ])
}

pub(super) fn show_single_mode_template(
    template: SingleModeTemplate,
    tui: &mut Tui,
    state: &mut AppState,
    view: &SingleModeView,
) {
    let mut show_block = |slot, node, tui: &mut Tui| {
        show_single_mode_block(slot, node, tui, state, view);
    };
    show_template(template, tui, &mut show_block);
}

fn show_single_mode_block(
    slot: TemplateBlockSlot,
    node: SingleModeNode,
    tui: &mut Tui,
    state: &mut AppState,
    view: &SingleModeView,
) {
    match node {
        SingleModeNode::FormatRows => match single_mode_block_style(slot, node) {
            None => render_format_rows(tui, state),
            Some(style) => {
                // FormatRows is a coarse block. Its internal video/audio/subtitle rows
                // remain inside single_mode_format_rows.rs and need the same Tui tree.
                tui.style(style).add(|tui| render_format_rows(tui, state));
            }
        },
        node => {
            let style = single_mode_block_style(slot, node)
                .unwrap_or_else(xaml_taffy_styles::xaml_grow_block_style);
            tui.style(style)
                .ui(|ui| show_single_mode_leaf(node, ui, state, view));
        }
    }
}

fn show_single_mode_leaf(
    node: SingleModeNode,
    ui: &mut Ui,
    state: &mut AppState,
    view: &SingleModeView,
) {
    match node {
        SingleModeNode::TitleField => render_title_field_at(ui, ui.max_rect(), view),
        SingleModeNode::DescriptionField => render_description_field_at(ui, ui.max_rect(), view),
        SingleModeNode::FormatRows => {}
        SingleModeNode::Thumbnail => render_thumbnail_at(ui, ui.max_rect(), state, view),
        SingleModeNode::DownloadThumbnailCheckbox => {
            render_download_thumbnail_checkbox_at(ui, ui.max_rect(), state)
        }
        SingleModeNode::RightInfo => render_right_info_at(ui, ui.max_rect(), state, view),
        SingleModeNode::Empty => {}
    }
}

fn single_mode_block_style(slot: TemplateBlockSlot, node: SingleModeNode) -> Option<taffy::Style> {
    match slot {
        TemplateBlockSlot::Root => None,
        TemplateBlockSlot::Child {
            axis: TemplateAxis::Rows,
            sizing: TemplateSizing::FixedPx(px),
            style: _,
        } if matches!(node, SingleModeNode::FormatRows) => {
            Some(xaml_taffy_styles::xaml_fixed_height_vertical_stack_style(
                px,
                super::item_card::item_detail_row_gap(),
            ))
        }
        TemplateBlockSlot::Child {
            axis: TemplateAxis::Rows,
            sizing: TemplateSizing::FixedPx(px),
            style: _,
        } if matches!(node, SingleModeNode::RightInfo) => Some(
            xaml_taffy_styles::xaml_shrinkable_fixed_height_block_style(px),
        ),
        TemplateBlockSlot::Child { style, .. } => Some(style),
    }
}
