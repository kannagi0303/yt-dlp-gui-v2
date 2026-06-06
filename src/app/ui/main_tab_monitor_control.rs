use eframe::egui::{self, Color32, Ui};
use egui_taffy::Tui;

use crate::app::state::AppState;
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};

use super::{semantic_ui_metrics, xaml_layout_contracts, xaml_taffy_styles};

#[derive(Debug, Clone, Copy)]
pub(super) struct MonitorToggleButton {
    cell: xaml_taffy_styles::XamlSingleLineRowCell,
    button_size: xaml_layout_contracts::LayoutSize,
}

impl MonitorToggleButton {
    pub(super) fn resolve(row: xaml_taffy_styles::XamlSingleLineRowLayout) -> Self {
        let element = semantic_ui_metrics::xaml_icon_button_ui_element_from_row_contract(row.row());
        let button_size = row.measure_auto_width_element(element).size;
        Self {
            cell: row.fixed_width_element_cell(element, button_size.width),
            button_size,
        }
    }

    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        self.cell.show(tui, |ui| {
            let response = ui.add_sized(
                self.button_size.to_array(),
                clipboard_monitor_button(ui, state),
            );
            if response.clicked() {
                state.set_monitor_clipboard(!state.clipboard_monitor_enabled());
            }
        });
    }
}

fn clipboard_monitor_button(ui: &Ui, state: &AppState) -> egui::Button<'static> {
    let enabled = state.clipboard_monitor_enabled();
    let icon = if enabled {
        AppIcon::MonitorEye
    } else {
        AppIcon::MonitorOff
    };
    let size = ui.spacing().interact_size.y * 0.72;
    let icon_color = if enabled {
        Color32::WHITE
    } else {
        standard_icon_color(ui)
    };
    let mut button = egui::Button::image(icon_image(icon, size, icon_color));
    if enabled {
        button = button.fill(ui.visuals().selection.bg_fill);
    }
    button
}
