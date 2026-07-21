use eframe::egui::{self, Ui};
use egui_taffy::Tui;

use crate::app::state::{AppMode, AppState};
use crate::app::widgets::icon::AppIcon;
use crate::domain::{DownloadContainerPreference, QueueItemId};

use super::common::{UiText, icon_text_button};
use super::item_card::draw_item_output_container_picker;
use super::main_tab_dependency_notice::missing_tool_icon_text_button;
use super::{semantic_ui_metrics, xaml_layout_contracts, xaml_taffy_styles};

#[derive(Debug, Clone, Copy)]
pub(super) struct TargetSelectButton {
    cell: xaml_taffy_styles::XamlSingleLineRowCell,
    button_size: xaml_layout_contracts::LayoutSize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DownloadButton {
    cell: xaml_taffy_styles::XamlSingleLineRowCell,
    button_size: xaml_layout_contracts::LayoutSize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DownloadContainerPicker {
    cell: xaml_taffy_styles::XamlSingleLineRowCell,
    item_id: QueueItemId,
    selected: DownloadContainerPreference,
}

impl DownloadContainerPicker {
    pub(super) fn resolve(
        row: xaml_taffy_styles::XamlSingleLineRowLayout,
        state: &AppState,
    ) -> Option<Self> {
        if state.app_mode() != AppMode::Origin
            || state.queue_items.is_empty()
            || state.item_is_busy(0)
            || !state.item_supports_webm_download_container(0)
        {
            return None;
        }
        Some(Self {
            cell: row.fixed_width_stretch_cell(
                semantic_ui_metrics::item_card_output_container_picker_width(),
            ),
            item_id: state.queue_items[0].id,
            selected: state.resolved_item_download_container(0)?,
        })
    }

    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        self.cell.show(tui, |ui| {
            if let Some(container) =
                draw_item_output_container_picker(ui, self.item_id, self.selected)
            {
                state.set_item_download_container_preference(0, container);
            }
        });
    }
}

impl TargetSelectButton {
    pub(super) fn resolve(
        row: xaml_taffy_styles::XamlSingleLineRowLayout,
        ui: &Ui,
        state: &AppState,
    ) -> Self {
        let target_text = state.ui_i18n_text_for_key(UiText::TARGET_DIR);
        let element =
            semantic_ui_metrics::xaml_icon_text_button_ui_element_for_visible_text(ui, target_text);
        let button_size = row.measure_auto_width_element(element).size;
        Self {
            cell: row.fixed_width_element_cell(element, button_size.width),
            button_size,
        }
    }

    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        self.cell.show(tui, |ui| {
            let target_text = state.ui_i18n_text_for_key(UiText::TARGET_DIR);
            let output_locked_by_config = state.output_dir_locked_by_config();
            let response = ui.add_enabled(
                !output_locked_by_config,
                icon_text_button(ui, AppIcon::FolderMoveOutline, target_text)
                    .min_size(egui::vec2(self.button_size.width, self.button_size.height)),
            );
            if response.clicked() {
                let mut dialog = rfd::FileDialog::new();
                if let Ok(current_dir) =
                    crate::infrastructure::resolve_output_dir(&state.item_defaults.output_dir)
                {
                    dialog = dialog.set_directory(current_dir);
                }
                if let Some(folder) = dialog.pick_folder() {
                    state.set_output_dir(folder.display().to_string());
                }
            }
        });
    }
}

impl DownloadButton {
    pub(super) fn resolve(
        row: xaml_taffy_styles::XamlSingleLineRowLayout,
        ui: &Ui,
        state: &AppState,
    ) -> Self {
        let download_text = state.ui_i18n_text_for_key(UiText::DOWNLOAD);
        let element = semantic_ui_metrics::xaml_icon_text_button_ui_element_for_visible_text(
            ui,
            download_text,
        );
        let button_size = row.measure_auto_width_element(element).size;
        Self {
            cell: row.fixed_width_element_cell(element, button_size.width),
            button_size,
        }
    }

    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        self.cell.show(tui, |ui| {
            let has_pending_downloads = state.has_pending_download_items();
            let missing_yt_dlp =
                has_pending_downloads && state.required_dependency_notice().is_some();
            let button = download_button_for_state(ui, state, missing_yt_dlp)
                .min_size(egui::vec2(self.button_size.width, self.button_size.height));
            let response = if has_pending_downloads {
                ui.add(button)
            } else {
                ui.add_enabled(false, button)
            };
            if response.clicked() && has_pending_downloads && !missing_yt_dlp {
                state.request_main_download();
            }
        });
    }
}

fn download_button_for_state(ui: &Ui, state: &AppState, muted: bool) -> egui::Button<'static> {
    let download_text = state.ui_i18n_text_for_key(UiText::DOWNLOAD);
    if muted {
        missing_tool_icon_text_button(ui, AppIcon::Download, download_text)
    } else {
        icon_text_button(ui, AppIcon::Download, download_text)
    }
}
