use std::path::PathBuf;

use eframe::egui::{self, Ui};
use egui_taffy::Tui;

use crate::app::state::AppState;
use crate::app::widgets::icon::AppIcon;
use crate::app::widgets::url_input::{AppTextBox, AppTextBoxSyntax};
use crate::infrastructure::{DependencyTool, dependency_tool_exists};

use super::common::{icon_text_button, settings_taffy_form_row, settings_taffy_section};
use super::semantic_ui_metrics;

pub(super) fn render_tool_paths_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    let section_title = state.ui_i18n_text_for_key("options.tool_paths");
    settings_taffy_section(tui, section_title, |tui| {
        render_tool_auto_detect_row(tui, state, label_width);
        for tool in [
            DependencyTool::YtDlp,
            DependencyTool::Deno,
            DependencyTool::Ffmpeg,
            DependencyTool::Aria2c,
        ] {
            tool_path_row(tui, state, label_width, tool);
        }
    });
}

fn render_tool_auto_detect_row(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    let auto_detect_text = state.ui_i18n_text_for_key("options.auto_detect");
    settings_taffy_form_row(tui, label_width, "", |ui| {
        let row_height = ui.spacing().interact_size.y;
        let width = semantic_ui_metrics::standard_icon_text_button_width_for_visible_text(
            ui,
            auto_detect_text,
        );
        let response = ui.add_sized(
            [width, row_height],
            icon_text_button(ui, AppIcon::Magnify, auto_detect_text),
        );
        if response.clicked() {
            state.auto_detect_dependency_tool_paths();
        }
    });
}

fn tool_path_row(tui: &mut Tui, state: &mut AppState, label_width: f32, tool: DependencyTool) {
    let label = tool.label();
    let expected_file_name = tool.executable_name();
    let current_value = state.dependency_tool_path(tool).to_owned();
    let trimmed = current_value.trim().to_owned();
    let missing_file = !trimmed.is_empty() && !dependency_tool_exists(&trimmed);
    let is_active = state.dependency_tool_update_is_running(tool);
    let install_running = state.component_update_running();
    let installed = state.dependency_tool_is_installed(tool);
    let install_text = state.ui_i18n_text_for_key("options.install");
    let reinstall_text = state.ui_i18n_text_for_key("options.reinstall");
    let installing_text = state.ui_i18n_text_for_key("options.installing");
    let browse_text = state.ui_i18n_text_for_key("advance.browse");
    let button_label = if is_active {
        installing_text
    } else if installed {
        reinstall_text
    } else {
        install_text
    };

    settings_taffy_form_row(tui, label_width, label, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x =
                semantic_ui_metrics::tool_path_row_control_horizontal_spacing();

            let metrics = ToolPathRowMetrics::new(
                ui,
                install_text,
                reinstall_text,
                installing_text,
                browse_text,
            );
            let mut value = current_value.clone();
            let response = AppTextBox::new(&mut value)
                .language(state.language())
                .syntax(AppTextBoxSyntax::Path)
                .error(missing_file)
                .desired_width(metrics.path_width)
                .editable(false)
                .selectable(false)
                .ui(ui);
            drop(response);

            let response = ui.add_enabled(
                !install_running,
                icon_text_button(ui, AppIcon::Download, button_label)
                    .min_size(egui::vec2(metrics.install_button_width, metrics.row_height)),
            );
            if response.clicked() {
                state.install_dependency_tool(tool);
            }
            drop(response);

            if ui
                .add_sized(
                    [metrics.pick_button_width, metrics.row_height],
                    icon_text_button(ui, AppIcon::FolderSettings, browse_text),
                )
                .clicked()
            {
                choose_dependency_tool_path(state, tool, label, expected_file_name, &trimmed);
            }
        });
    });
}

struct ToolPathRowMetrics {
    row_height: f32,
    path_width: f32,
    install_button_width: f32,
    pick_button_width: f32,
}

impl ToolPathRowMetrics {
    fn new(
        ui: &Ui,
        install_text: &str,
        reinstall_text: &str,
        installing_text: &str,
        browse_text: &str,
    ) -> Self {
        let row_height =
            semantic_ui_metrics::tool_path_row_standard_control_height_from_current_ui_metrics(ui);
        let install_button_width =
            semantic_ui_metrics::standard_icon_text_button_width_for_visible_text(
                ui,
                reinstall_text,
            )
            .max(
                semantic_ui_metrics::standard_icon_text_button_width_for_visible_text(
                    ui,
                    installing_text,
                ),
            )
            .max(
                semantic_ui_metrics::standard_icon_text_button_width_for_visible_text(
                    ui,
                    install_text,
                ),
            );
        let pick_button_width =
            semantic_ui_metrics::standard_icon_text_button_width_for_visible_text(ui, browse_text);
        let path_width =
            semantic_ui_metrics::tool_path_row_path_text_field_width_for_available_width_and_buttons(
                ui.available_width(),
                install_button_width,
                pick_button_width,
                ui.spacing().item_spacing.x,
            );

        Self {
            row_height,
            path_width,
            install_button_width,
            pick_button_width,
        }
    }
}

fn choose_dependency_tool_path(
    state: &mut AppState,
    tool: DependencyTool,
    label: &str,
    expected_file_name: &str,
    current_path_text: &str,
) {
    let mut dialog = rfd::FileDialog::new()
        .add_filter(
            state.ui_i18n_text_for_key("options.filter_executable"),
            &["exe"],
        )
        .set_title(format!(
            "{} {label} {}",
            state.ui_i18n_text_for_key("options.choose"),
            state.ui_i18n_text_for_key("options.executable")
        ));
    if !current_path_text.is_empty() {
        let current_path = PathBuf::from(current_path_text);
        if let Some(parent) = current_path.parent().filter(|path| path.is_dir()) {
            dialog = dialog.set_directory(parent);
        }
    }
    if let Some(path) = dialog.set_file_name(expected_file_name).pick_file() {
        set_dependency_tool_path(state, tool, path.display().to_string());
    }
}

fn set_dependency_tool_path(state: &mut AppState, tool: DependencyTool, path: String) {
    match tool {
        DependencyTool::YtDlp => state.set_yt_dlp_path(path),
        DependencyTool::Ffmpeg => state.set_ffmpeg_path(path),
        DependencyTool::Aria2c => state.set_aria2c_path(path),
        DependencyTool::Deno => state.set_deno_path(path),
    }
}
