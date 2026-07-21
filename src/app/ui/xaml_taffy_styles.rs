//! Thin bridge from XAML-like UI element contracts to taffy styles.
//!
//! This module is intentionally small: xaml_layout_contracts stays independent
//! from taffy, while UI files can ask this bridge for taffy styles instead of
//! hand-calculating fixed/flex cells.

use eframe::egui::Ui;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{Tui, TuiBuilderLogic as _, taffy};

use super::xaml_layout_contracts::{
    LayoutLength, LayoutSize, MeasuredUiElement, SingleLineControlRowContract, UiElement,
};
use super::xaml_ui_nodes::{TemplateAxis, TemplateSizing};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum XamlTaffyElementRole {
    VerticalRoot,
    FixedHeightBlock,
    GrowBlock,
    FixedHeightRow { column_gap: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct XamlTaffyElement {
    element: UiElement,
    role: XamlTaffyElementRole,
}

impl XamlTaffyElement {
    pub(super) fn vertical_root(height: f32) -> Self {
        Self {
            element: UiElement::vertical_root(height),
            role: XamlTaffyElementRole::VerticalRoot,
        }
    }

    pub(super) fn fixed_height_block(height: f32) -> Self {
        Self {
            element: UiElement::fixed_height_row(height),
            role: XamlTaffyElementRole::FixedHeightBlock,
        }
    }

    pub(super) fn fixed_height_spacer(height: f32) -> Self {
        Self {
            element: UiElement::fixed_height_spacer(height),
            role: XamlTaffyElementRole::FixedHeightBlock,
        }
    }

    pub(super) fn grow_block() -> Self {
        Self {
            element: UiElement::fill_content_presenter(),
            role: XamlTaffyElementRole::GrowBlock,
        }
    }

    pub(super) fn fixed_height_row(row: SingleLineControlRowContract, column_gap: f32) -> Self {
        Self {
            element: UiElement::fixed_height_row(row.height),
            role: XamlTaffyElementRole::FixedHeightRow {
                column_gap: column_gap.max(0.0),
            },
        }
    }

    pub(super) fn style(self) -> taffy::Style {
        match self.role {
            XamlTaffyElementRole::VerticalRoot => self.element.taffy_vertical_root_style(),
            XamlTaffyElementRole::FixedHeightBlock => self.element.taffy_fixed_height_block_style(),
            XamlTaffyElementRole::GrowBlock => self.element.taffy_grow_block_style(),
            XamlTaffyElementRole::FixedHeightRow { column_gap } => {
                self.element.taffy_fixed_height_row_style(column_gap)
            }
        }
    }

    pub(super) fn show_empty(self, tui: &mut Tui) {
        tui.style(self.style()).ui(|_| {});
    }

    pub(super) fn show_tui(self, tui: &mut Tui, add_contents: impl FnOnce(&mut Tui)) {
        tui.style(self.style()).add(add_contents);
    }

    pub(super) fn show_ui(self, tui: &mut Tui, add_contents: impl FnOnce(&mut Ui)) {
        tui.style(self.style()).ui(add_contents);
    }

    pub(super) fn show_fill_ui(self, tui: &mut Tui, add_contents: impl FnOnce(&mut Ui)) {
        self.show_ui(tui, |ui| show_fill_content_presenter(ui, add_contents));
    }
}

pub(super) trait UiElementTaffyExt {
    fn taffy_vertical_root_style(self) -> taffy::Style;
    fn taffy_fixed_height_block_style(self) -> taffy::Style;
    fn taffy_grow_block_style(self) -> taffy::Style;
    fn taffy_fixed_height_row_style(self, gap: f32) -> taffy::Style;
}

impl UiElementTaffyExt for UiElement {
    fn taffy_vertical_root_style(self) -> taffy::Style {
        xaml_vertical_root_style(ui_element_pixel_height(self))
    }

    fn taffy_fixed_height_block_style(self) -> taffy::Style {
        xaml_fixed_height_block_style(ui_element_pixel_height(self))
    }

    fn taffy_grow_block_style(self) -> taffy::Style {
        xaml_grow_block_style()
    }

    fn taffy_fixed_height_row_style(self, gap: f32) -> taffy::Style {
        xaml_fixed_height_row_style(
            SingleLineControlRowContract::new(ui_element_pixel_height(self)),
            gap,
        )
    }
}

fn ui_element_pixel_height(element: UiElement) -> f32 {
    match element.layout.height {
        LayoutLength::Pixel(height) => height.max(0.0),
        _ => element.intrinsic_size.height.max(0.0),
    }
}

pub(super) fn show_fill_content_presenter(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    ui.set_width(ui.available_width());
    ui.set_height(ui.available_height());
    add_contents(ui);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum XamlSingleLineRowCellWidth {
    Auto,
    Fixed(f32),
    Star { min_width: f32, weight: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct XamlSingleLineRowCell {
    row_layout: XamlSingleLineRowLayout,
    element: UiElement,
    width: XamlSingleLineRowCellWidth,
}

impl XamlSingleLineRowCell {
    pub(super) fn auto(row_layout: XamlSingleLineRowLayout, element: UiElement) -> Self {
        Self {
            row_layout,
            element,
            width: XamlSingleLineRowCellWidth::Auto,
        }
    }

    pub(super) fn fixed_width(
        row_layout: XamlSingleLineRowLayout,
        element: UiElement,
        width: f32,
    ) -> Self {
        Self {
            row_layout,
            element,
            width: XamlSingleLineRowCellWidth::Fixed(width.max(0.0)),
        }
    }

    pub(super) fn star(
        row_layout: XamlSingleLineRowLayout,
        element: UiElement,
        min_width: f32,
        weight: f32,
    ) -> Self {
        Self {
            row_layout,
            element,
            width: XamlSingleLineRowCellWidth::Star {
                min_width: min_width.max(0.0),
                weight: weight.max(0.0),
            },
        }
    }

    pub(super) fn style(self) -> taffy::Style {
        match self.width {
            XamlSingleLineRowCellWidth::Auto => self.row_layout.auto_width_cell(self.element).1,
            XamlSingleLineRowCellWidth::Fixed(width) => {
                self.row_layout.fixed_width_cell_style(width)
            }
            XamlSingleLineRowCellWidth::Star { min_width, weight } => {
                self.row_layout.star_width_cell_style(min_width, weight)
            }
        }
    }

    pub(super) fn measured_size_for_available_width(self, available_width: f32) -> LayoutSize {
        match self.width {
            XamlSingleLineRowCellWidth::Auto => {
                self.row_layout
                    .measure_auto_width_element(self.element)
                    .size
            }
            XamlSingleLineRowCellWidth::Fixed(width) => {
                self.row_layout
                    .row()
                    .measure_fixed_width_element(self.element, width)
                    .size
            }
            XamlSingleLineRowCellWidth::Star { .. } => {
                self.row_layout
                    .measure_stretch_width_element(self.element, available_width)
                    .size
            }
        }
    }

    pub(super) fn measured_size_for_ui(self, ui: &Ui) -> LayoutSize {
        self.measured_size_for_available_width(ui.available_width())
    }

    pub(super) fn show(self, tui: &mut Tui, add_contents: impl FnOnce(&mut Ui)) {
        tui.style(self.style()).ui(add_contents);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct XamlSingleLineRowLayout {
    row: SingleLineControlRowContract,
    column_gap: f32,
}

impl XamlSingleLineRowLayout {
    pub(super) fn new(row: SingleLineControlRowContract) -> Self {
        Self {
            row,
            column_gap: 0.0,
        }
    }

    pub(super) fn with_column_gap(mut self, column_gap: f32) -> Self {
        self.column_gap = column_gap.max(0.0);
        self
    }

    pub(super) fn row(self) -> SingleLineControlRowContract {
        self.row
    }

    pub(super) fn height(self) -> f32 {
        self.row.height
    }

    pub(super) fn element(self) -> XamlTaffyElement {
        XamlTaffyElement::fixed_height_row(self.row, self.column_gap)
    }

    pub(super) fn style(self) -> taffy::Style {
        self.element().style()
    }

    pub(super) fn show(self, tui: &mut Tui, add_contents: impl FnOnce(&mut Tui)) {
        self.element().show_tui(tui, add_contents);
    }

    pub(super) fn auto_width_cell(self, element: UiElement) -> (MeasuredUiElement, taffy::Style) {
        xaml_auto_width_cell_style(self.row, element)
    }

    pub(super) fn shrinkable_auto_width_cell(
        self,
        element: UiElement,
    ) -> (MeasuredUiElement, taffy::Style) {
        xaml_shrinkable_auto_width_cell_style(self.row, element)
    }

    pub(super) fn fixed_width_cell_style(self, width: f32) -> taffy::Style {
        xaml_fixed_width_cell_style(self.row, width)
    }

    pub(super) fn star_width_cell_style(self, min_width: f32, star: f32) -> taffy::Style {
        xaml_star_width_cell_style(self.row, min_width, star)
    }

    pub(super) fn star_width_text_box_cell_style(self) -> taffy::Style {
        xaml_star_width_text_box_cell_style(self.row)
    }

    pub(super) fn flex_spacer_cell_style(self) -> taffy::Style {
        xaml_flex_spacer_cell_style(self.row)
    }

    pub(super) fn auto_width_element_cell(self, element: UiElement) -> XamlSingleLineRowCell {
        XamlSingleLineRowCell::auto(self, element)
    }

    pub(super) fn fixed_width_element_cell(
        self,
        element: UiElement,
        width: f32,
    ) -> XamlSingleLineRowCell {
        XamlSingleLineRowCell::fixed_width(self, element, width)
    }

    pub(super) fn star_width_element_cell(
        self,
        element: UiElement,
        min_width: f32,
        weight: f32,
    ) -> XamlSingleLineRowCell {
        XamlSingleLineRowCell::star(self, element, min_width, weight)
    }

    pub(super) fn star_width_text_box_cell(self) -> XamlSingleLineRowCell {
        self.star_width_element_cell(UiElement::single_line_text_input(self.row), 0.0, 1.0)
    }

    pub(super) fn star_width_stretch_cell(self) -> XamlSingleLineRowCell {
        self.star_width_element_cell(
            UiElement::stretch_width_stretch_height(LayoutSize::new(0.0, self.row.height)),
            0.0,
            1.0,
        )
    }

    pub(super) fn fixed_width_stretch_cell(self, width: f32) -> XamlSingleLineRowCell {
        self.fixed_width_element_cell(
            UiElement::fixed_width_stretch_height(width, self.row),
            width,
        )
    }

    pub(super) fn show_fixed_width_cell(
        self,
        tui: &mut Tui,
        width: f32,
        add_contents: impl FnOnce(&mut Ui),
    ) {
        tui.style(self.fixed_width_cell_style(width))
            .ui(add_contents);
    }

    pub(super) fn show_star_width_text_box_cell(
        self,
        tui: &mut Tui,
        add_contents: impl FnOnce(&mut Ui),
    ) {
        tui.style(self.star_width_text_box_cell_style())
            .ui(add_contents);
    }

    pub(super) fn measure_auto_width_element(self, element: UiElement) -> MeasuredUiElement {
        self.row.measure_auto_width_element(element)
    }

    pub(super) fn measure_stretch_width_element(
        self,
        element: UiElement,
        available_width: f32,
    ) -> MeasuredUiElement {
        self.row
            .measure_stretch_width_element(element, available_width)
    }

    pub(super) fn measure_spacer(self, width: f32) -> super::xaml_layout_contracts::LayoutSize {
        self.row.measure_spacer(width)
    }
}

pub(super) fn xaml_horizontal_auto_stack_style(
    gap: f32,
    align_items: taffy::AlignItems,
) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(align_items),
        size: taffy::Size {
            width: percent(1.0_f32),
            height: auto(),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        gap: length(gap.max(0.0)),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_weighted_width_auto_height_vertical_stack_style(gap: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: length(0.0_f32),
            height: auto(),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        flex_basis: length(0.0_f32),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        gap: length(gap.max(0.0)),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_horizontal_grow_root_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0_f32),
            height: percent(1.0_f32),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: percent(1.0_f32),
        },
        flex_basis: length(0.0_f32),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        gap: length(0.0_f32),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_vertical_grow_root_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0_f32),
            height: percent(1.0_f32),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: percent(1.0_f32),
        },
        flex_basis: length(0.0_f32),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        gap: length(0.0_f32),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_template_root_style(axis: TemplateAxis) -> taffy::Style {
    match axis {
        TemplateAxis::Rows => xaml_vertical_grow_root_style(),
        TemplateAxis::Cols => xaml_horizontal_grow_root_style(),
    }
}

pub(super) fn xaml_template_auto_root_style(axis: TemplateAxis) -> taffy::Style {
    match axis {
        TemplateAxis::Rows => xaml_vertical_auto_root_style(),
        TemplateAxis::Cols => xaml_horizontal_auto_stack_style(0.0, taffy::AlignItems::Stretch),
    }
}

pub(super) fn xaml_template_child_style(
    parent_axis: TemplateAxis,
    sizing: TemplateSizing,
    child_axis: Option<TemplateAxis>,
) -> taffy::Style {
    let mut style = xaml_template_base_child_style(parent_axis, child_axis);
    apply_template_sizing(&mut style, parent_axis, sizing);
    style
}

pub(super) fn xaml_template_gap_style(parent_axis: TemplateAxis, size: f32) -> taffy::Style {
    xaml_template_child_style(parent_axis, TemplateSizing::FixedPx(size), None)
}

/// Relative layer host for XAML-style content with layout-owned overlays.
///
/// Children remain responsible for declaring either full-size flow content or
/// an absolute edge attachment. Page code must not derive overlay coordinates
/// from sibling rects.
pub(super) fn xaml_overlay_host_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        position: taffy::Position::Relative,
        size: taffy::Size {
            width: percent(1.0_f32),
            height: percent(1.0_f32),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: percent(1.0_f32),
        },
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_overlay_content_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        position: taffy::Position::Absolute,
        inset: taffy::Rect {
            left: length(0.0_f32),
            right: length(0.0_f32),
            top: length(0.0_f32),
            bottom: length(0.0_f32),
        },
        size: taffy::Size {
            width: percent(1.0_f32),
            height: percent(1.0_f32),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: percent(1.0_f32),
        },
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_bottom_overlay_style(height: f32) -> taffy::Style {
    let height = height.max(0.0);
    taffy::Style {
        position: taffy::Position::Absolute,
        inset: taffy::Rect {
            left: length(0.0_f32),
            right: length(0.0_f32),
            top: auto(),
            bottom: length(0.0_f32),
        },
        size: taffy::Size {
            width: percent(1.0_f32),
            height: length(height),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(height),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: length(height),
        },
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

fn xaml_template_base_child_style(
    parent_axis: TemplateAxis,
    child_axis: Option<TemplateAxis>,
) -> taffy::Style {
    let mut style = taffy::Style {
        display: match child_axis {
            Some(_) => taffy::Display::Flex,
            None => taffy::Display::Block,
        },
        align_items: child_axis.map(|_| taffy::AlignItems::Stretch),
        gap: length(0.0_f32),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    };

    if let Some(axis) = child_axis {
        style.flex_direction = match axis {
            TemplateAxis::Rows => taffy::FlexDirection::Column,
            TemplateAxis::Cols => taffy::FlexDirection::Row,
        };
    }

    match parent_axis {
        TemplateAxis::Rows => {
            style.size.width = percent(1.0_f32);
            style.max_size.width = percent(1.0_f32);
        }
        TemplateAxis::Cols => {
            style.size.height = percent(1.0_f32);
            style.max_size.height = percent(1.0_f32);
        }
    }

    style.min_size = taffy::Size {
        width: length(0.0_f32),
        height: length(0.0_f32),
    };
    style
}

fn apply_template_sizing(
    style: &mut taffy::Style,
    parent_axis: TemplateAxis,
    sizing: TemplateSizing,
) {
    match (parent_axis, sizing) {
        (TemplateAxis::Rows, TemplateSizing::Auto) => {
            style.size.height = auto();
            style.flex_grow = 0.0;
            style.flex_shrink = 1.0;
        }
        (TemplateAxis::Cols, TemplateSizing::Auto) => {
            style.size.width = auto();
            style.flex_grow = 0.0;
            style.flex_shrink = 1.0;
        }
        (TemplateAxis::Rows, TemplateSizing::Star(weight)) => {
            style.size.height = length(0.0_f32);
            style.max_size.height = percent(1.0_f32);
            style.flex_basis = length(0.0_f32);
            style.flex_grow = weight.max(0.0);
            style.flex_shrink = 1.0;
        }
        (TemplateAxis::Cols, TemplateSizing::Star(weight)) => {
            style.size.width = length(0.0_f32);
            style.max_size.width = percent(1.0_f32);
            style.flex_basis = length(0.0_f32);
            style.flex_grow = weight.max(0.0);
            style.flex_shrink = 1.0;
        }
        (TemplateAxis::Rows, TemplateSizing::FixedPx(px)) => {
            let px = px.max(0.0);
            style.size.height = length(px);
            style.min_size.height = length(px);
            style.max_size.height = length(px);
            style.flex_grow = 0.0;
            style.flex_shrink = 0.0;
        }
        (TemplateAxis::Cols, TemplateSizing::FixedPx(px)) => {
            let px = px.max(0.0);
            style.size.width = length(px);
            style.min_size.width = length(px);
            style.max_size.width = length(px);
            style.flex_grow = 0.0;
            style.flex_shrink = 0.0;
        }
        (TemplateAxis::Rows, TemplateSizing::ClampPx { min, ideal, max }) => {
            let min = min.max(0.0);
            let max = max.max(min);
            style.size.height = length(ideal.clamp(min, max));
            style.min_size.height = length(min);
            style.max_size.height = length(max);
            style.flex_grow = 0.0;
            style.flex_shrink = 1.0;
        }
        (TemplateAxis::Cols, TemplateSizing::ClampPx { min, ideal, max }) => {
            let min = min.max(0.0);
            let max = max.max(min);
            style.size.width = length(ideal.clamp(min, max));
            style.min_size.width = length(min);
            style.max_size.width = length(max);
            style.flex_grow = 0.0;
            style.flex_shrink = 1.0;
        }
    }
}

pub(super) fn xaml_fixed_height_vertical_stack_style(height: f32, gap: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0_f32),
            height: length(height.max(0.0)),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: length(height.max(0.0)),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        gap: length(gap.max(0.0)),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_shrinkable_fixed_height_block_style(height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0_f32),
            height: length(height.max(0.0)),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: length(height.max(0.0)),
        },
        flex_grow: 0.0,
        flex_shrink: 1.0,
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

#[cfg(test)]
mod overlay_style_tests {
    use super::*;

    #[test]
    fn overlay_host_owns_relative_full_size_layout() {
        let style = xaml_overlay_host_style();

        assert_eq!(style.position, taffy::Position::Relative);
        assert_eq!(style.size.width, percent(1.0_f32));
        assert_eq!(style.size.height, percent(1.0_f32));
    }

    #[test]
    fn bottom_overlay_is_absolute_and_height_bounded() {
        let style = xaml_bottom_overlay_style(72.0);

        assert_eq!(style.position, taffy::Position::Absolute);
        assert_eq!(style.inset.left, length(0.0_f32));
        assert_eq!(style.inset.right, length(0.0_f32));
        assert_eq!(style.inset.bottom, length(0.0_f32));
        assert_eq!(style.size.height, length(72.0_f32));
        assert_eq!(style.min_size.height, length(72.0_f32));
        assert_eq!(style.max_size.height, length(72.0_f32));
    }

    #[test]
    fn overlay_content_is_an_absolute_full_size_layer() {
        let style = xaml_overlay_content_style();

        assert_eq!(style.display, taffy::Display::Flex);
        assert_eq!(style.flex_direction, taffy::FlexDirection::Column);
        assert_eq!(style.align_items, Some(taffy::AlignItems::Stretch));
        assert_eq!(style.position, taffy::Position::Absolute);
        assert_eq!(style.inset.left, length(0.0_f32));
        assert_eq!(style.inset.right, length(0.0_f32));
        assert_eq!(style.inset.top, length(0.0_f32));
        assert_eq!(style.inset.bottom, length(0.0_f32));
        assert_eq!(style.size.width, percent(1.0_f32));
        assert_eq!(style.size.height, percent(1.0_f32));
    }
}

pub(super) fn xaml_vertical_root_style(height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0_f32),
            height: length(height.max(0.0)),
        },
        min_size: taffy::Size {
            width: percent(1.0_f32),
            height: length(0.0_f32),
        },
        gap: length(0.0_f32),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_vertical_auto_root_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0_f32),
            height: auto(),
        },
        min_size: taffy::Size {
            width: percent(1.0_f32),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: auto(),
        },
        gap: length(0.0_f32),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_vertical_auto_section_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0_f32),
            height: auto(),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        gap: length(0.0_f32),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_auto_height_block_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0_f32),
            height: auto(),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_fixed_height_block_style(height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0_f32),
            height: length(height.max(0.0)),
        },
        min_size: taffy::Size {
            width: percent(1.0_f32),
            height: length(height.max(0.0)),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: length(height.max(0.0)),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_grow_block_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: percent(1.0_f32),
            height: length(0.0_f32),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        flex_basis: length(0.0_f32),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_fixed_size_flex_cell_style(width: f32, height: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        size: taffy::Size {
            width: length(width.max(0.0)),
            height: length(height.max(0.0)),
        },
        min_size: taffy::Size {
            width: length(width.max(0.0)),
            height: length(height.max(0.0)),
        },
        max_size: taffy::Size {
            width: length(width.max(0.0)),
            height: length(height.max(0.0)),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        align_items: Some(taffy::AlignItems::Stretch),
        justify_content: Some(taffy::JustifyContent::Center),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_fixed_width_stretch_height_flex_cell_style(width: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        size: taffy::Size {
            width: length(width.max(0.0)),
            height: percent(1.0_f32),
        },
        min_size: taffy::Size {
            width: length(width.max(0.0)),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: length(width.max(0.0)),
            height: percent(1.0_f32),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        align_items: Some(taffy::AlignItems::Stretch),
        justify_content: Some(taffy::JustifyContent::Center),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_weighted_width_stretch_height_cell_style(weight: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(0.0_f32),
            height: percent(1.0_f32),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: percent(1.0_f32),
        },
        flex_basis: length(0.0_f32),
        flex_grow: weight.max(0.0),
        flex_shrink: 1.0,
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_fixed_width_stretch_height_gap_style(width: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(width.max(0.0)),
            height: percent(1.0_f32),
        },
        min_size: taffy::Size {
            width: length(width.max(0.0)),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: length(width.max(0.0)),
            height: percent(1.0_f32),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_fixed_height_row_style(
    row: SingleLineControlRowContract,
    gap: f32,
) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::Stretch),
        size: taffy::Size {
            width: percent(1.0_f32),
            height: length(row.height),
        },
        min_size: taffy::Size {
            width: percent(1.0_f32),
            height: length(row.height),
        },
        max_size: taffy::Size {
            width: percent(1.0_f32),
            height: length(row.height),
        },
        gap: length(gap.max(0.0)),
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_auto_width_cell_style(
    row: SingleLineControlRowContract,
    element: UiElement,
) -> (MeasuredUiElement, taffy::Style) {
    let measured = row.measure_auto_width_element(element);
    (measured, xaml_measured_fixed_width_cell_style(measured))
}

pub(super) fn xaml_shrinkable_auto_width_cell_style(
    row: SingleLineControlRowContract,
    element: UiElement,
) -> (MeasuredUiElement, taffy::Style) {
    let measured = row.measure_auto_width_element(element);
    (
        measured,
        xaml_measured_shrinkable_width_cell_style(measured),
    )
}

pub(super) fn xaml_fixed_width_cell_style(
    row: SingleLineControlRowContract,
    width: f32,
) -> taffy::Style {
    let fixed_element = UiElement::fixed_width_stretch_height(width, row);
    let measured = row.measure_fixed_width_element(fixed_element, width);
    xaml_measured_fixed_width_cell_style(measured)
}

pub(super) fn xaml_star_width_text_box_cell_style(
    row: SingleLineControlRowContract,
) -> taffy::Style {
    xaml_star_width_cell_style(row, 0.0, 1.0)
}

pub(super) fn xaml_measured_fixed_width_cell_style(measured: MeasuredUiElement) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(measured.size.width),
            height: percent(1.0_f32),
        },
        min_size: taffy::Size {
            width: length(measured.size.width),
            height: length(0.0_f32),
        },
        max_size: taffy::Size {
            width: length(measured.size.width),
            height: percent(1.0_f32),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_measured_shrinkable_width_cell_style(
    measured: MeasuredUiElement,
) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(measured.size.width),
            height: percent(1.0_f32),
        },
        min_size: taffy::Size {
            width: length(0.0_f32),
            height: length(0.0_f32),
        },
        flex_basis: length(measured.size.width),
        flex_grow: 0.0,
        flex_shrink: 1.0,
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_star_width_cell_style(
    row: SingleLineControlRowContract,
    min_width: f32,
    star: f32,
) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        size: taffy::Size {
            width: length(0.0_f32),
            height: percent(1.0_f32),
        },
        min_size: taffy::Size {
            width: length(min_width.max(0.0)),
            height: length(row.height),
        },
        flex_basis: length(0.0_f32),
        flex_grow: star.max(0.0),
        flex_shrink: 1.0,
        padding: length(0.0_f32),
        margin: length(0.0_f32),
        ..Default::default()
    }
}

pub(super) fn xaml_flex_spacer_cell_style(row: SingleLineControlRowContract) -> taffy::Style {
    xaml_star_width_cell_style(row, 0.0, 1.0)
}
