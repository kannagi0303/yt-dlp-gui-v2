use eframe::egui::{Align, Layout, Ui};
use egui_taffy::taffy::prelude::{length, percent};
use egui_taffy::{Tui, TuiBuilderLogic as _, taffy};

use crate::app::state::AppState;
use crate::infrastructure::ManagedComponentId;

use super::xaml_layout_contracts::{HorizontalAlignment, LayoutSize, UiElement, VerticalAlignment};
use super::{semantic_ui_metrics, xaml_taffy_styles};

// ABOUT ROW CONTROL RULES
//
// This file is the adapter between About's TemplateTree slots and concrete
// UiElement cells. It is deliberately separated from `about_tab_template.rs` so
// the template file can stay as a tree description, like the existing tab
// templates. It is also separated from `about_tab.rs` so painter code cannot
// own row/column alignment.
//
// Allowed here:
// - XamlSingleLineRowLayout / UiElement cell construction;
// - fixed/star cell widths;
// - content alignment inside a cell.
//
// Forbidden here:
// - product state mutation except through painter callbacks in about_tab.rs;
// - creating another taffy root/template tree;
// - local i18n fallback dictionaries.
//
// 每次處理中，如有發現不符合地方應強力修正。

const ABOUT_CELL_GAP_PX: f32 = 8.0;
const ABOUT_VERSION_MIN_WIDTH_PX: f32 = 44.0;
const ABOUT_ARROW_MIN_WIDTH_PX: f32 = 18.0;

// Row-wide interaction belongs to the row control layer, not to per-cell
// painters.  Keep this as a layout-owned overlay so hover/selection is one
// continuous row instead of several disconnected cell rectangles.

#[derive(Debug, Clone, Copy)]
pub(super) struct AboutRowMetrics {
    row: xaml_taffy_styles::XamlSingleLineRowLayout,
    row_height: f32,
    current_width: f32,
    arrow_width: f32,
    latest_width: f32,
    action_width: f32,
    check_button_width: f32,
    last_check_width: f32,
    update_all_button_width: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AboutHeaderRow {
    metrics: AboutRowMetrics,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AboutComponentRow {
    metrics: AboutRowMetrics,
    id: ManagedComponentId,
}

impl AboutRowMetrics {
    pub(super) fn resolve(ui: &Ui, state: &AppState, component_ids: &[ManagedComponentId]) -> Self {
        let row_contract =
            semantic_ui_metrics::xaml_settings_form_single_line_row_contract_from_current_control_metrics(ui);
        let row = xaml_taffy_styles::XamlSingleLineRowLayout::new(row_contract)
            .with_column_gap(ABOUT_CELL_GAP_PX);
        let row_height = row.height();

        Self {
            row,
            row_height,
            current_width: super::about_tab::about_current_version_width(ui, state, component_ids)
                .max(ABOUT_VERSION_MIN_WIDTH_PX),
            arrow_width: super::about_tab::about_arrow_width(ui).max(ABOUT_ARROW_MIN_WIDTH_PX),
            latest_width: super::about_tab::about_latest_version_width(ui, state, component_ids)
                .max(ABOUT_VERSION_MIN_WIDTH_PX),
            action_width: super::about_tab::about_action_slot_width(ui, state, component_ids),
            last_check_width: super::about_tab::about_last_check_width(ui, state),
            check_button_width: super::about_tab::about_check_button_width(ui, state),
            update_all_button_width: super::about_tab::about_update_all_button_width(ui, state),
        }
    }

    pub(super) fn row_height(self) -> f32 {
        self.row_height
    }

    pub(super) fn header_row(self) -> AboutHeaderRow {
        AboutHeaderRow { metrics: self }
    }

    pub(super) fn component_row(self, id: ManagedComponentId) -> AboutComponentRow {
        AboutComponentRow { metrics: self, id }
    }
}

impl AboutHeaderRow {
    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        let metrics = self.metrics;
        metrics.row.show(tui, |tui| {
            show_star_cell(tui, metrics, HorizontalAlignment::Left, |ui| {
                super::about_tab::render_header_lead(ui, state)
            });
            show_fixed_cell(
                tui,
                metrics,
                metrics.last_check_width,
                UiElement::label,
                HorizontalAlignment::Right,
                |ui| super::about_tab::render_last_check_inline(ui, state),
            );
            show_fixed_cell(
                tui,
                metrics,
                metrics.check_button_width,
                UiElement::icon_text_button,
                HorizontalAlignment::Center,
                |ui| super::about_tab::render_check_updates_button(ui, state),
            );
            show_fixed_cell(
                tui,
                metrics,
                metrics.update_all_button_width,
                UiElement::icon_text_button,
                HorizontalAlignment::Center,
                |ui| super::about_tab::render_update_all_button(ui, state),
            );
        });
    }
}

impl AboutComponentRow {
    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        let metrics = self.metrics;
        let id = self.id;
        metrics.row.show(tui, |tui| {
            show_component_row_overlay(tui, state, id);
            show_star_cell(tui, metrics, HorizontalAlignment::Left, |ui| {
                super::about_tab::render_component_name_cell(ui, state, id)
            });
            show_fixed_cell(
                tui,
                metrics,
                metrics.current_width,
                UiElement::label,
                HorizontalAlignment::Right,
                |ui| super::about_tab::render_component_current_version_cell(ui, state, id),
            );
            show_fixed_cell(
                tui,
                metrics,
                metrics.arrow_width,
                UiElement::label,
                HorizontalAlignment::Center,
                |ui| super::about_tab::render_component_arrow_cell(ui, state, id),
            );
            show_fixed_cell(
                tui,
                metrics,
                metrics.latest_width,
                UiElement::label,
                HorizontalAlignment::Left,
                |ui| super::about_tab::render_component_latest_version_cell(ui, state, id),
            );
            show_fixed_cell(
                tui,
                metrics,
                metrics.action_width,
                UiElement::button,
                HorizontalAlignment::Right,
                |ui| super::about_tab::render_component_action_cell(ui, state, id),
            );
        });
    }
}

fn show_component_row_overlay(tui: &mut Tui, state: &mut AppState, id: ManagedComponentId) {
    tui.style(component_row_overlay_style()).ui(|ui| {
        super::about_tab::render_component_row_overlay(ui, state, id);
    });
}

fn component_row_overlay_style() -> taffy::Style {
    taffy::Style {
        position: taffy::Position::Absolute,
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
        margin: length(0.0),
        padding: length(0.0),
        ..Default::default()
    }
}

fn show_star_cell(
    tui: &mut Tui,
    metrics: AboutRowMetrics,
    content_alignment: HorizontalAlignment,
    add_contents: impl FnOnce(&mut Ui),
) {
    let element = UiElement::stretch_width_stretch_height(LayoutSize::new(0.0, metrics.row_height))
        .with_content_alignment(content_alignment, VerticalAlignment::Center);
    metrics
        .row
        .star_width_element_cell(element, 0.0, 1.0)
        .show(tui, |ui| show_cell_content(ui, element, add_contents));
}

fn show_fixed_cell(
    tui: &mut Tui,
    metrics: AboutRowMetrics,
    width: f32,
    make_element: impl FnOnce(LayoutSize) -> UiElement,
    content_alignment: HorizontalAlignment,
    add_contents: impl FnOnce(&mut Ui),
) {
    let width = width.max(0.0);
    let element = make_element(LayoutSize::new(width, metrics.row_height))
        .with_content_alignment(content_alignment, VerticalAlignment::Center);
    metrics
        .row
        .fixed_width_element_cell(element, width)
        .show(tui, |ui| show_cell_content(ui, element, add_contents));
}

fn show_cell_content(ui: &mut Ui, element: UiElement, add_contents: impl FnOnce(&mut Ui)) {
    ui.set_width(ui.available_width().max(0.0));
    ui.set_height(ui.available_height().max(0.0));

    match element.horizontal_content_alignment {
        HorizontalAlignment::Left | HorizontalAlignment::Stretch => {
            ui.with_layout(Layout::left_to_right(Align::Center), add_contents);
        }
        HorizontalAlignment::Center => {
            ui.centered_and_justified(add_contents);
        }
        HorizontalAlignment::Right => {
            ui.with_layout(Layout::right_to_left(Align::Center), add_contents);
        }
    }
}
