use egui_taffy::taffy;

use super::{semantic_ui_metrics, xaml_taffy_styles};

pub(super) fn item_card_root_style() -> taffy::Style {
    xaml_taffy_styles::xaml_horizontal_auto_stack_style(
        semantic_ui_metrics::item_card_column_gap(),
        taffy::AlignItems::FlexStart,
    )
}

pub(super) fn item_thumbnail_column_style() -> taffy::Style {
    xaml_taffy_styles::xaml_fixed_size_flex_cell_style(
        semantic_ui_metrics::item_card_thumbnail_width(),
        semantic_ui_metrics::item_card_thumbnail_height(),
    )
}

pub(super) fn item_detail_column_style() -> taffy::Style {
    xaml_taffy_styles::xaml_weighted_width_auto_height_vertical_stack_style(
        semantic_ui_metrics::item_card_detail_row_gap(),
    )
}
