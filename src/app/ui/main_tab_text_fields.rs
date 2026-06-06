use eframe::egui;
use egui_taffy::Tui;

use crate::app::state::{AppMode, AppState};
use crate::app::widgets::url_input::{AppTextBox, AppTextBoxSyntax};

use super::common::UiText;
use super::xaml_taffy_styles;

#[derive(Debug, Clone, Copy)]
pub(super) struct UrlTextBox {
    cell: xaml_taffy_styles::XamlSingleLineRowCell,
    is_single_mode: bool,
    url_input_locked: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PathTextBox {
    cell: xaml_taffy_styles::XamlSingleLineRowCell,
}

impl UrlTextBox {
    pub(super) fn resolve(
        row: xaml_taffy_styles::XamlSingleLineRowLayout,
        state: &AppState,
    ) -> Self {
        Self {
            cell: row.star_width_text_box_cell(),
            is_single_mode: state.app_mode() == AppMode::Origin,
            url_input_locked: state.url_input_locked(),
        }
    }

    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        self.cell.show(tui, |ui| {
            let url_hint = state.ui_i18n_text_for_key(UiText::URL_HINT).to_owned();
            let language = state.language();
            let url_input_size = self.cell.measured_size_for_ui(ui);
            let response = AppTextBox::new(&mut state.url_input)
                .hint_text(&url_hint)
                .language(language)
                .enabled(!self.url_input_locked)
                .syntax(AppTextBoxSyntax::Url)
                .desired_width(url_input_size.width)
                .desired_height(url_input_size.height)
                .min_rows(1)
                .max_rows(Some(1))
                .allow_newline(false)
                .ctrl_click_links(false)
                .ui(ui);
            let submit_requested = self.is_single_mode
                && response.has_focus()
                && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if submit_requested && !self.url_input_locked {
                state.run_primary_url_action();
            }
        });
    }
}

impl PathTextBox {
    pub(super) fn resolve(row: xaml_taffy_styles::XamlSingleLineRowLayout) -> Self {
        Self {
            cell: row.star_width_text_box_cell(),
        }
    }

    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        self.cell.show(tui, |ui| {
            let mut output_dir_display = state.output_dir_display();
            let output_text_box_size = self.cell.measured_size_for_ui(ui);
            AppTextBox::new(&mut output_dir_display)
                .editable(false)
                .selectable(true)
                .syntax(AppTextBoxSyntax::Path)
                .desired_width(output_text_box_size.width)
                .desired_height(output_text_box_size.height)
                .min_rows(1)
                .max_rows(Some(1))
                .allow_newline(false)
                .ctrl_click_links(false)
                .ui(ui);
        });
    }
}
