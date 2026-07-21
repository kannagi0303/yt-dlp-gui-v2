use eframe::egui::Ui;
use egui_taffy::Tui;

use crate::app::state::AppState;

use super::main_tab_monitor_control::MonitorToggleButton;
use super::main_tab_output_actions::{DownloadButton, DownloadContainerPicker, TargetSelectButton};
use super::main_tab_text_fields::{PathTextBox, UrlTextBox};
use super::main_tab_url_action::StartButton;
use super::xaml_taffy_styles;

pub(super) enum MainTabTextBoxRole {
    Url(UrlTextBox),
    Path(PathTextBox),
}

pub(super) enum MainTabButtonRole {
    MonitorToggle(MonitorToggleButton),
    Start(StartButton),
    TargetSelect(TargetSelectButton),
    Download(DownloadButton),
}

pub(super) struct MainTabControls {
    pub(super) url: MainTabTextBoxRole,
    pub(super) monitor_toggle: MainTabButtonRole,
    pub(super) start: MainTabButtonRole,
    pub(super) target_select: MainTabButtonRole,
    pub(super) path: MainTabTextBoxRole,
    pub(super) download_container: Option<DownloadContainerPicker>,
    pub(super) download: MainTabButtonRole,
}

impl MainTabTextBoxRole {
    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        match self {
            Self::Url(url_text_box) => url_text_box.show(tui, state),
            Self::Path(path_text_box) => path_text_box.show(tui, state),
        }
    }
}

impl MainTabButtonRole {
    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        match self {
            Self::MonitorToggle(monitor_toggle_button) => monitor_toggle_button.show(tui, state),
            Self::Start(start_button) => start_button.show(tui, state),
            Self::TargetSelect(target_select_button) => target_select_button.show(tui, state),
            Self::Download(download_button) => download_button.show(tui, state),
        }
    }
}

impl MainTabControls {
    pub(super) fn resolve(
        ui: &Ui,
        state: &AppState,
        row: xaml_taffy_styles::XamlSingleLineRowLayout,
    ) -> Self {
        Self {
            url: MainTabTextBoxRole::Url(UrlTextBox::resolve(row, state)),
            monitor_toggle: MainTabButtonRole::MonitorToggle(MonitorToggleButton::resolve(row)),
            start: MainTabButtonRole::Start(StartButton::resolve(row, ui, state)),
            target_select: MainTabButtonRole::TargetSelect(TargetSelectButton::resolve(
                row, ui, state,
            )),
            path: MainTabTextBoxRole::Path(PathTextBox::resolve(row)),
            download_container: DownloadContainerPicker::resolve(row, state),
            download: MainTabButtonRole::Download(DownloadButton::resolve(row, ui, state)),
        }
    }
}
