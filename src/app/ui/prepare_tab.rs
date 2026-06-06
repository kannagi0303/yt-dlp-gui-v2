use eframe::egui::{self, Align, Color32, RichText, ScrollArea, Ui};
use egui_extras::{Size, StripBuilder};

use crate::app::state::{AppState, PrepareDetailPage};
use crate::app::widgets::icon::{AppIcon, icon_image};
use crate::i18n::LanguageSelection;
use crate::infrastructure::{
    ComponentUpdateStatus, DependencyTool, PrepareRequirement, PrepareStatus,
};

use super::common::{icon_text_button, scroll_content_with_right_gap, text_trailing_icon_button};
use super::semantic_ui_metrics;

#[derive(Clone, Copy)]
pub(super) struct ToolRowMetrics {
    pub(super) row_width: f32,
    pub(super) row_height: f32,
    pub(super) icon_width: f32,
    pub(super) name_width: f32,
}

pub(super) fn render_prepare_tab(ui: &mut Ui, state: &mut AppState) {
    if matches!(state.prepare_detail_page, Some(PrepareDetailPage::Language)) {
        render_language_detail_page(ui, state);
        return;
    }

    super::prepare_tab_template::render_prepare_tab_template(ui, state);
}

pub(super) fn render_language_selector(ui: &mut Ui, state: &mut AppState) {
    let language_text = state.ui_i18n_text_for_key("prepare.language");
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(language_text)
                .size(semantic_ui_metrics::prepare_small_text_size())
                .color(detail_color(ui)),
        );
        let label = language_choice_label(state, state.language_selection());
        if ui
            .add(text_trailing_icon_button(ui, &label, AppIcon::MenuRight))
            .clicked()
        {
            state.open_prepare_detail_page(PrepareDetailPage::Language);
        }
    });
}

fn render_language_detail_page(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical()
        .id_salt("prepare-language-page-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            scroll_content_with_right_gap(ui, |ui| {
                ui.horizontal(|ui| {
                    let back_text = state.ui_i18n_text_for_key("prepare.back");
                    let language_text = state.ui_i18n_text_for_key("prepare.language");
                    if ui.button(format!("← {back_text}")).clicked() {
                        state.close_prepare_detail_page();
                    }
                    ui.label(RichText::new(language_text).strong());
                });
                ui.add_space(
                    semantic_ui_metrics::prepare_language_detail_header_to_choices_vertical_spacing(
                    ),
                );
                for language in LanguageSelection::PICKER_ORDER {
                    render_language_choice_row(ui, state, language);
                }
            });
        });
}

fn render_language_choice_row(ui: &mut Ui, state: &mut AppState, language: LanguageSelection) {
    let selected = state.language_selection() == language;
    let label = language_choice_label(state, language);
    ui.horizontal(|ui| {
        ui.add_sized(
            [
                semantic_ui_metrics::prepare_language_choice_checkmark_column_width(),
                ui.spacing().interact_size.y,
            ],
            egui::Label::new(if selected { "✓" } else { "" }),
        );
        if ui.selectable_label(selected, label).clicked() {
            state.set_language_selection(language);
        }
    });
}

fn language_choice_label(state: &AppState, language: LanguageSelection) -> String {
    match language {
        LanguageSelection::Auto => {
            let auto_detect_text = state.ui_i18n_text_for_key("prepare.auto_detect");
            format!(
                "{} ({})",
                auto_detect_text,
                language.resolve().native_name()
            )
        }
        _ => language.native_name().to_owned(),
    }
}

pub(super) fn render_header(ui: &mut Ui, state: &AppState) {
    let header_text =
        state.ui_i18n_text_for_key("prepare.install_the_required_tools_now_or_skip_and_h");
    ui.label(
        RichText::new(header_text)
            .size(semantic_ui_metrics::prepare_title_text_size())
            .color(detail_color(ui)),
    );
}

pub(super) fn tool_row_metrics(ui: &Ui, state: &AppState) -> ToolRowMetrics {
    let row_height = semantic_ui_metrics::prepare_tool_row_height_from_current_control_metrics(ui);
    let icon_width = semantic_ui_metrics::prepare_tool_row_icon_width_from_row_height(row_height);
    let base_name_width =
        semantic_ui_metrics::prepare_tool_row_name_width_for_visible_tool_names(ui);
    let missing_text = state.ui_i18n_text_for_key("prepare.missing");
    let install_later_text = state.ui_i18n_text_for_key("prepare.install_later");
    let downloading_text = state.ui_i18n_text_for_key("prepare.downloading_100");
    let extracting_text = state.ui_i18n_text_for_key("prepare.extracting_100");
    let install_failed_text = state.ui_i18n_text_for_key("prepare.install_failed");
    let status_width = semantic_ui_metrics::prepare_tool_row_status_width_for_visible_labels(
        ui,
        &[
            missing_text,
            install_later_text,
            downloading_text,
            extracting_text,
            install_failed_text,
        ],
    );
    let desired_width = semantic_ui_metrics::prepare_tool_row_width_for_columns(
        ui,
        icon_width,
        base_name_width,
        0.0,
        0.0,
        status_width,
    );
    let row_width = desired_width.min(ui.available_width());

    ToolRowMetrics {
        row_width,
        row_height,
        icon_width,
        name_width: semantic_ui_metrics::prepare_tool_row_name_width_with_following_column_spacing(
            ui,
            base_name_width,
        ),
    }
}

pub(super) fn has_tool_requirement(state: &AppState, tool: DependencyTool) -> bool {
    tool_requirement(state, tool).is_some()
}

fn tool_requirement<'a>(
    state: &'a AppState,
    tool: DependencyTool,
) -> Option<&'a PrepareRequirement> {
    state
        .prepare_requirements()
        .iter()
        .find(|item| item.has_install_action(tool))
}

pub(super) fn render_tool_row_slot(
    ui: &mut Ui,
    state: &mut AppState,
    tool: DependencyTool,
    metrics: &ToolRowMetrics,
) {
    if let Some(item) = tool_requirement(state, tool).cloned() {
        render_tool_row(ui, state, &item, tool, metrics);
    }
}

fn render_tool_row(
    ui: &mut Ui,
    state: &mut AppState,
    item: &PrepareRequirement,
    tool: DependencyTool,
    metrics: &ToolRowMetrics,
) {
    let is_installing = state.dependency_tool_update_is_running(tool);
    let update_status = state.dependency_tool_update_status(tool);
    let update_running = state.component_update_running();
    let indicator = tool_status_indicator(item, update_status, is_installing, update_running);
    let indicator_color = tool_status_color(ui, item, update_status, is_installing, update_running);

    ui.allocate_ui(egui::vec2(metrics.row_width, metrics.row_height), |ui| {
        StripBuilder::new(ui)
            .size(Size::exact(metrics.icon_width))
            .size(Size::exact(metrics.name_width))
            .size(Size::remainder())
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    centered_status_indicator(ui, indicator, indicator_color);
                });
                strip.cell(|ui| {
                    left_label(
                        ui,
                        RichText::new(tool.label())
                            .size(semantic_ui_metrics::prepare_body_text_size())
                            .strong(),
                    );
                });
                strip.cell(|ui| {
                    left_label(
                        ui,
                        RichText::new(state.localize_message(&tool_status_text(state, item, tool)))
                            .size(semantic_ui_metrics::prepare_body_text_size())
                            .color(detail_color(ui)),
                    );
                });
            });
    });
}

fn left_label(ui: &mut Ui, text: RichText) {
    ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
        ui.label(text);
    });
}

pub(super) fn render_prepare_status(ui: &mut Ui, state: &AppState) {
    let Some(status_text) = state.prepare_footer_status_text() else {
        return;
    };

    ui.label(
        RichText::new(status_text)
            .size(semantic_ui_metrics::prepare_small_text_size())
            .color(detail_color(ui)),
    );
}

pub(super) fn has_environment_issues(state: &AppState) -> bool {
    state
        .prepare_requirements()
        .iter()
        .any(|item| item.action.is_none() && item.needs_attention())
}

pub(super) fn render_environment_issues(ui: &mut Ui, state: &AppState) {
    let issues = state
        .prepare_requirements()
        .iter()
        .filter(|item| item.action.is_none() && item.needs_attention())
        .collect::<Vec<_>>();

    if issues.is_empty() {
        return;
    }

    let needs_attention_text = state.ui_i18n_text_for_key("prepare.needs_attention");
    ui.label(RichText::new(needs_attention_text).size(17.0).strong());
    ui.add_space(semantic_ui_metrics::prepare_environment_issues_header_to_rows_vertical_spacing());
    for item in issues {
        render_issue_row(ui, state, item);
        ui.add_space(
            semantic_ui_metrics::prepare_environment_issues_between_rows_vertical_spacing(),
        );
    }
}

fn render_issue_row(ui: &mut Ui, state: &AppState, item: &PrepareRequirement) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_top(|ui| {
            ui.add_sized(
                [
                    semantic_ui_metrics::prepare_environment_issue_status_column_width_from_current_control_metrics(ui),
                    ui.spacing().interact_size.y,
                ],
                egui::Label::new(
                    RichText::new(status_symbol(item.status))
                        .size(semantic_ui_metrics::prepare_title_text_size())
                        .strong()
                        .color(status_color(ui, item.status)),
                ),
            );
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(state.localize_message(&item.title))
                        .size(semantic_ui_metrics::prepare_body_text_size())
                        .strong(),
                );
                if !item.recommendation.trim().is_empty() {
                    ui.label(
                        RichText::new(state.localize_message(&item.recommendation))
                            .size(semantic_ui_metrics::prepare_small_text_size())
                            .color(detail_color(ui)),
                    );
                } else {
                    ui.label(
                        RichText::new(state.localize_message(&item.description))
                            .size(semantic_ui_metrics::prepare_small_text_size())
                            .color(detail_color(ui)),
                    );
                }
                if !item.detail.trim().is_empty() {
                    ui.label(
                        RichText::new(state.localize_message(&item.detail))
                            .size(semantic_ui_metrics::prepare_small_text_size())
                            .color(warn_color(ui)),
                    );
                }
            });
        });
    });
}

pub(super) fn render_bottom_actions(ui: &mut Ui, state: &mut AppState) {
    let install_running = state.component_update_running();
    let can_install_all = state.prepare_installable_tool_count() > 0;
    let install_block_reason = state.prepare_dependency_install_block_reason();
    let row_height = semantic_ui_metrics::prepare_tool_row_height_from_current_control_metrics(ui);
    let install_all_text = state.ui_i18n_text_for_key("prepare.install_all");
    let reinstall_text = state.ui_i18n_text_for_key("prepare.reinstall");
    let installing_text = state.ui_i18n_text_for_key("prepare.installing");
    let skip_text = state.ui_i18n_text_for_key("prepare.skip");
    let install_text = state.ui_i18n_text_for_key("prepare.install");
    let action_width = semantic_ui_metrics::prepare_tool_row_action_width_for_visible_labels(
        ui,
        &[
            install_all_text,
            reinstall_text,
            installing_text,
            skip_text,
            install_text,
        ],
    );

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), row_height),
        egui::Layout::right_to_left(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x =
                semantic_ui_metrics::prepare_bottom_action_button_horizontal_spacing();
            let install_all = ui.add_enabled(
                !install_running && can_install_all && install_block_reason.is_none(),
                icon_text_button(ui, AppIcon::Download, install_all_text)
                    .min_size(egui::vec2(action_width, row_height)),
            );
            if install_all.clicked() {
                state.install_all_prepare_tools();
            }
            drop(install_all);

            let skip = ui.add_enabled(
                !install_running,
                icon_text_button(ui, AppIcon::WindowClose, skip_text)
                    .min_size(egui::vec2(action_width, row_height)),
            );
            if skip.clicked() {
                state.snooze_prepare_tab();
            }
        },
    );
}

#[derive(Clone, Copy)]
enum ToolStatusIndicator {
    Check,
    Cross,
    Pending,
}

fn tool_status_indicator(
    item: &PrepareRequirement,
    update_status: Option<ComponentUpdateStatus>,
    is_installing: bool,
    update_running: bool,
) -> ToolStatusIndicator {
    match update_status {
        Some(
            ComponentUpdateStatus::Checking
            | ComponentUpdateStatus::Downloading
            | ComponentUpdateStatus::Staged
            | ComponentUpdateStatus::Applying,
        ) if update_running || is_installing => ToolStatusIndicator::Pending,
        Some(ComponentUpdateStatus::Missing | ComponentUpdateStatus::UpdateAvailable)
            if update_running =>
        {
            ToolStatusIndicator::Pending
        }
        Some(ComponentUpdateStatus::Installed | ComponentUpdateStatus::UpToDate)
            if item.status == PrepareStatus::Ok =>
        {
            ToolStatusIndicator::Check
        }
        Some(ComponentUpdateStatus::Failed) => ToolStatusIndicator::Cross,
        _ if update_running && item.needs_attention() => ToolStatusIndicator::Pending,
        _ if item.status == PrepareStatus::Ok => ToolStatusIndicator::Check,
        _ => ToolStatusIndicator::Cross,
    }
}

fn centered_status_indicator(ui: &mut Ui, indicator: ToolStatusIndicator, color: Color32) {
    let icon = match indicator {
        ToolStatusIndicator::Check => AppIcon::Check,
        ToolStatusIndicator::Cross => AppIcon::Close,
        ToolStatusIndicator::Pending => AppIcon::DownloadCircle,
    };
    let icon_size = semantic_ui_metrics::standard_icon_size_from_current_control_metrics(ui);
    ui.with_layout(
        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        |ui| {
            ui.add(icon_image(icon, icon_size, color));
        },
    );
}

fn status_symbol(status: PrepareStatus) -> &'static str {
    match status {
        PrepareStatus::Ok => "✓",
        PrepareStatus::Warning => "!",
        PrepareStatus::Missing | PrepareStatus::Failed => "×",
    }
}

fn tool_status_color(
    ui: &Ui,
    item: &PrepareRequirement,
    update_status: Option<ComponentUpdateStatus>,
    is_installing: bool,
    update_running: bool,
) -> Color32 {
    match update_status {
        Some(
            ComponentUpdateStatus::Checking
            | ComponentUpdateStatus::Downloading
            | ComponentUpdateStatus::Staged
            | ComponentUpdateStatus::Applying,
        ) if update_running || is_installing => warn_color(ui),
        Some(ComponentUpdateStatus::Missing | ComponentUpdateStatus::UpdateAvailable)
            if update_running =>
        {
            warn_color(ui)
        }
        Some(ComponentUpdateStatus::Installed | ComponentUpdateStatus::UpToDate)
            if item.status == PrepareStatus::Ok =>
        {
            ok_color(ui)
        }
        Some(ComponentUpdateStatus::Failed) => error_color(ui),
        _ if update_running && item.needs_attention() => warn_color(ui),
        _ if item.status == PrepareStatus::Ok => ok_color(ui),
        _ => error_color(ui),
    }
}

fn tool_status_text(state: &AppState, item: &PrepareRequirement, tool: DependencyTool) -> String {
    if let Some(status) = state.dependency_tool_update_status_text(tool) {
        return status;
    }

    match item.status {
        PrepareStatus::Ok => state
            .ui_i18n_text_for_key("prepare.status.ready")
            .to_owned(),
        PrepareStatus::Missing => state
            .ui_i18n_text_for_key("prepare.status.missing")
            .to_owned(),
        PrepareStatus::Warning => state
            .ui_i18n_text_for_key("prepare.status.warning")
            .to_owned(),
        PrepareStatus::Failed => state
            .ui_i18n_text_for_key("prepare.status.failed")
            .to_owned(),
    }
}

fn status_color(ui: &Ui, status: PrepareStatus) -> Color32 {
    match status {
        PrepareStatus::Ok => ok_color(ui),
        PrepareStatus::Missing | PrepareStatus::Failed => error_color(ui),
        PrepareStatus::Warning => warn_color(ui),
    }
}

fn ok_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(120, 210, 150)
    } else {
        Color32::from_rgb(0, 132, 54)
    }
}

fn warn_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(230, 185, 85)
    } else {
        Color32::from_rgb(200, 116, 0)
    }
}

fn error_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(240, 110, 110)
    } else {
        Color32::from_rgb(214, 0, 32)
    }
}

fn detail_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(170, 180, 190)
    } else {
        Color32::from_rgb(82, 88, 96)
    }
}
