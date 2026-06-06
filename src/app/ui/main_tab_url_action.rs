use eframe::egui::{self, RichText, Spinner, TextStyle, Ui, WidgetText};
use egui_taffy::Tui;

use crate::app::state::{AppMode, AppState};
use crate::app::widgets::icon::AppIcon;

use super::common::icon_text_button;
use super::main_tab_dependency_notice::{
    missing_tool_icon_text_button, missing_tool_plain_button, show_missing_yt_dlp_callout,
};
use super::{semantic_ui_metrics, xaml_taffy_styles};

type MainUrlActionButtonTemplate = StartButton;

#[derive(Debug, Clone, Copy)]
pub(super) struct StartButton {
    cell: xaml_taffy_styles::XamlSingleLineRowCell,
    show_spinner: bool,
    analysis_running: bool,
    url_input_locked: bool,
    spinner_size: f32,
    spinner_gap: f32,
}

impl StartButton {
    pub(super) fn resolve(
        row: xaml_taffy_styles::XamlSingleLineRowLayout,
        ui: &Ui,
        state: &AppState,
    ) -> Self {
        let show_spinner = state.is_adding_batch && !state.is_cancelling_batch_add;
        let analysis_running = state.single_mode_analysis_running();
        let url_input_locked = state.url_input_locked();
        let spinner_size =
            semantic_ui_metrics::main_url_action_spinner_size_for_control_height(row.height());
        let spinner_gap = semantic_ui_metrics::main_url_action_spinner_to_text_horizontal_spacing();
        let action_text = state.ui_i18n_text_for_key(state.primary_url_action_label_key());
        let action_element = if primary_url_action_uses_icon(state) {
            semantic_ui_metrics::xaml_icon_text_button_ui_element_for_visible_text(ui, action_text)
        } else {
            semantic_ui_metrics::xaml_button_ui_element_for_visible_text(ui, action_text)
        };
        let action_button_size = row.measure_auto_width_element(action_element).size;
        let spinner_size_for_cell = row
            .measure_auto_width_element(
                semantic_ui_metrics::xaml_spinner_ui_element_for_square_size(spinner_size),
            )
            .size;
        let spinner_gap_size = row.measure_spacer(spinner_gap);
        let cell_width = action_button_size.width
            + if show_spinner {
                spinner_size_for_cell.width + spinner_gap_size.width
            } else {
                0.0
            };

        Self {
            cell: row.fixed_width_stretch_cell(cell_width),
            show_spinner,
            analysis_running,
            url_input_locked,
            spinner_size,
            spinner_gap,
        }
    }

    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        self.cell.show(tui, |ui| {
            if self.analysis_running {
                render_single_analysis_spinner_button(ui, state, &self);
                return;
            }

            if self.show_spinner {
                render_url_spinner_action_cell(ui, state, &self);
                return;
            }

            let action_button_size = self.cell.measured_size_for_ui(ui);
            let action_button_vec2 =
                egui::vec2(action_button_size.width, action_button_size.height);

            if state.is_adding_batch && state.is_cancelling_batch_add {
                let button = primary_url_action_button(ui, state);
                ui.add_enabled(
                    false,
                    button
                        .min_size(action_button_vec2)
                        .wrap_mode(egui::TextWrapMode::Extend),
                );
                return;
            }

            let missing_yt_dlp = state.required_dependency_notice().is_some();
            let button = primary_url_action_button_for_state(ui, state, missing_yt_dlp)
                .min_size(action_button_vec2)
                .wrap_mode(egui::TextWrapMode::Extend);
            let response = if missing_yt_dlp {
                ui.add(button)
            } else {
                ui.add_enabled(!self.url_input_locked, button)
            };
            if missing_yt_dlp {
                show_missing_yt_dlp_callout(ui, response.rect, "url-action", state);
            } else if response.clicked() {
                state.run_primary_url_action();
            }
        });
    }
}

fn render_url_spinner_action_cell(
    ui: &mut Ui,
    state: &mut AppState,
    action_button: &MainUrlActionButtonTemplate,
) {
    let rect = ui.max_rect();
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        let original_spacing_x = ui.spacing().item_spacing.x;
        ui.spacing_mut().item_spacing.x =
            semantic_ui_metrics::url_row_spinner_action_cell_horizontal_spacing();
        ui.horizontal(|ui| {
            ui.allocate_ui(
                egui::vec2(
                    action_button.spinner_size + action_button.spinner_gap,
                    rect.height(),
                ),
                |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.add(Spinner::new().size(action_button.spinner_size));
                    });
                },
            );
            let response = ui.add_sized(
                [ui.available_width(), rect.height()],
                primary_url_action_button(ui, state).wrap_mode(egui::TextWrapMode::Extend),
            );
            if response.clicked() {
                state.cancel_batch_add();
            }
        });
        ui.spacing_mut().item_spacing.x = original_spacing_x;
    });
}

fn render_single_analysis_spinner_button(
    ui: &mut Ui,
    state: &AppState,
    action_button: &MainUrlActionButtonTemplate,
) -> egui::Response {
    let analysis_button_size = action_button.cell.measured_size_for_ui(ui);
    let desired_size = egui::vec2(analysis_button_size.width, analysis_button_size.height);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let (button_bg_fill, button_bg_stroke, button_fg_color) = {
        let visuals = &ui.visuals().widgets.inactive;
        (visuals.bg_fill, visuals.bg_stroke, visuals.fg_stroke.color)
    };
    ui.painter().rect(
        rect,
        2.0,
        button_bg_fill,
        button_bg_stroke,
        egui::StrokeKind::Outside,
    );

    let icon_size = semantic_ui_metrics::standard_icon_size_from_current_control_metrics(ui);
    let label = state.ui_i18n_text_for_key(state.primary_url_action_label_key());
    let galley = WidgetText::from(RichText::new(label).size(icon_size)).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        TextStyle::Button,
    );
    let icon_spacing = ui.spacing().icon_spacing;
    let content_width = icon_size + icon_spacing + galley.size().x;
    let icon_left = rect.center().x - content_width * 0.5;
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(icon_left, rect.center().y - icon_size * 0.5),
        egui::vec2(icon_size, icon_size),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.add(Spinner::new().size(icon_size));
        });
    });
    let text_pos = egui::pos2(
        icon_rect.right() + icon_spacing,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, button_fg_color);
    response
}

fn primary_url_action_icon(state: &AppState) -> AppIcon {
    if state.app_mode() == AppMode::Origin {
        AppIcon::Magnify
    } else if state.config.direct_download_on_add && !state.queue_display_mode_is_audio() {
        AppIcon::Download
    } else {
        AppIcon::Import
    }
}

fn primary_url_action_uses_icon(state: &AppState) -> bool {
    if state.is_adding_batch {
        return state.app_mode() == AppMode::Origin
            || (state.config.direct_download_on_add && !state.queue_display_mode_is_audio());
    }

    state.app_mode() == AppMode::Origin
        || state.queue_display_mode_is_audio()
        || state.config.direct_download_on_add
        || state.app_mode() == AppMode::Standard
}

fn primary_url_action_button_for_state(
    ui: &Ui,
    state: &AppState,
    muted: bool,
) -> egui::Button<'static> {
    if primary_url_action_uses_icon(state) {
        if muted {
            missing_tool_icon_text_button(
                ui,
                primary_url_action_icon(state),
                state.ui_i18n_text_for_key(state.primary_url_action_label_key()),
            )
        } else {
            icon_text_button(
                ui,
                primary_url_action_icon(state),
                state.ui_i18n_text_for_key(state.primary_url_action_label_key()),
            )
        }
    } else if muted {
        missing_tool_plain_button(
            ui,
            state.ui_i18n_text_for_key(state.primary_url_action_label_key()),
        )
    } else {
        egui::Button::new(state.ui_i18n_text_for_key(state.primary_url_action_label_key()))
    }
}

fn primary_url_action_button(ui: &Ui, state: &AppState) -> egui::Button<'static> {
    primary_url_action_button_for_state(ui, state, false)
}
