use eframe::egui::{self, Align, Color32, RichText, ScrollArea, Ui};
use egui_extras::{Size, StripBuilder};

use crate::app::state::{AppState, PrepareDetailPage};
use crate::app::widgets::icon::AppIcon;
use crate::i18n::LanguageSelection;
use crate::infrastructure::{
    DependencyTool, PrepareRequirement, PrepareSeverity, PrepareStatus, ToolInstallStage,
};

use super::common::{
    icon_text_button, measure_label_width, natural_icon_button_width,
    scroll_content_with_right_gap, text_trailing_icon_button,
};

const TITLE_SIZE: f32 = 18.0;
const BODY_SIZE: f32 = 15.0;
const SMALL_SIZE: f32 = 13.0;
const CONTENT_LEFT_INDENT: f32 = 28.0;
const ROW_GAP_EXTRA: f32 = 6.0;

struct ToolRowMetrics {
    row_width: f32,
    row_height: f32,
    icon_width: f32,
    name_width: f32,
    severity_width: f32,
    action_width: f32,
}

pub(super) fn render_prepare_tab(ui: &mut Ui, state: &mut AppState) {
    if matches!(state.prepare_detail_page, Some(PrepareDetailPage::Language)) {
        render_language_detail_page(ui, state);
        return;
    }

    let action_height = ui.spacing().interact_size.y + 12.0;

    StripBuilder::new(ui)
        .size(Size::remainder())
        .size(Size::exact(action_height))
        .vertical(|mut strip| {
            strip.cell(|ui| {
                render_prepare_root_page(ui, state);
            });

            strip.cell(|ui| {
                render_bottom_actions(ui, state);
            });
        });
}

fn render_prepare_root_page(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical()
        .id_salt("prepare-mode-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            scroll_content_with_right_gap(ui, |ui| {
                render_language_selector(ui, state);
                ui.add_space(16.0);
                render_header(ui, state);
                ui.add_space(24.0);
                render_tool_rows(ui, state);
                render_environment_issues(ui, state);
                render_prepare_status(ui, state);
            });
        });
}

fn render_language_selector(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(state.tr("prepare.language"))
                .size(SMALL_SIZE)
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
                    if ui
                        .button(format!("← {}", state.tr("prepare.back")))
                        .clicked()
                    {
                        state.close_prepare_detail_page();
                    }
                    ui.label(RichText::new(state.tr("prepare.language")).strong());
                });
                ui.add_space(10.0);
                for language in LanguageSelection::PICKER_ORDER {
                    render_language_choice_row(ui, state, language);
                }
            });
        });
}

fn render_language_choice_row(ui: &mut Ui, state: &mut AppState, language: LanguageSelection) {
    const CHECK_WIDTH: f32 = 18.0;
    let selected = state.language_selection() == language;
    let label = language_choice_label(state, language);
    ui.horizontal(|ui| {
        ui.add_sized(
            [CHECK_WIDTH, ui.spacing().interact_size.y],
            egui::Label::new(if selected { "✓" } else { "" }),
        );
        if ui.selectable_label(selected, label).clicked() {
            state.set_language_selection(language);
        }
    });
}

fn language_choice_label(state: &AppState, language: LanguageSelection) -> String {
    match language {
        LanguageSelection::Auto => format!(
            "{} ({})",
            state.tr("prepare.auto_detect"),
            language.resolve().native_name()
        ),
        _ => language.native_name().to_owned(),
    }
}

fn render_header(ui: &mut Ui, state: &AppState) {
    ui.label(
        RichText::new(state.tr("prepare.install_the_required_tools_now_or_skip_and_h"))
            .size(TITLE_SIZE)
            .color(detail_color(ui)),
    );
}

fn render_tool_rows(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.add_space(CONTENT_LEFT_INDENT);
        ui.vertical(|ui| {
            let metrics = tool_row_metrics(ui, state);

            for tool in [
                DependencyTool::YtDlp,
                DependencyTool::Deno,
                DependencyTool::Ffmpeg,
            ] {
                if let Some(item) = tool_requirement(state, tool).cloned() {
                    render_tool_row(ui, state, &item, tool, &metrics);
                    ui.add_space(ui.spacing().item_spacing.y + ROW_GAP_EXTRA);
                }
            }
        });
    });
}

fn tool_row_metrics(ui: &Ui, state: &AppState) -> ToolRowMetrics {
    let spacing = ui.spacing();
    let row_height = standard_action_height(ui);
    let icon_width = row_height * 0.65;
    let name_width = measure_label_width(ui, &["yt-dlp", "Deno", "FFmpeg"]);
    let severity_width = measure_label_width(
        ui,
        &[
            state.tr("prepare.required"),
            state.tr("prepare.recommended"),
            state.tr("prepare.optional"),
        ],
    );
    let action_width = standard_action_width(ui, state);
    let status_width = measure_label_width(
        ui,
        &[
            state.tr("prepare.missing"),
            state.tr("prepare.install_later"),
            state.tr("prepare.downloading_100"),
            state.tr("prepare.extracting_100"),
            state.tr("prepare.install_failed"),
        ],
    );
    let gap = spacing.item_spacing.x;
    let desired_width = icon_width
        + gap * 0.8
        + name_width
        + gap * 2.8
        + severity_width
        + gap * 1.4
        + action_width
        + gap
        + status_width;
    let row_width = desired_width.min(ui.available_width());

    ToolRowMetrics {
        row_width,
        row_height,
        icon_width,
        name_width: name_width + gap * 2.8,
        severity_width,
        action_width,
    }
}

fn standard_action_height(ui: &Ui) -> f32 {
    ui.spacing().interact_size.y + 8.0
}

fn standard_action_width(ui: &Ui, state: &AppState) -> f32 {
    [
        state.tr("prepare.install_all"),
        state.tr("prepare.reinstall"),
        state.tr("prepare.installing"),
        state.tr("prepare.skip"),
        state.tr("prepare.install"),
    ]
    .into_iter()
    .map(|label| natural_icon_button_width(ui, label))
    .fold(0.0, f32::max)
        + ui.spacing().button_padding.x
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

fn render_tool_row(
    ui: &mut Ui,
    state: &mut AppState,
    item: &PrepareRequirement,
    tool: DependencyTool,
    metrics: &ToolRowMetrics,
) {
    let installing_tool = state.installing_dependency_tool();
    let is_installing = installing_tool == Some(tool);
    let any_installing = installing_tool.is_some();
    let install_block_reason = state.prepare_dependency_install_block_reason();
    let installed = item.status == PrepareStatus::Ok;

    ui.allocate_ui(egui::vec2(metrics.row_width, metrics.row_height), |ui| {
        StripBuilder::new(ui)
            .size(Size::exact(metrics.icon_width))
            .size(Size::exact(metrics.name_width))
            .size(Size::exact(metrics.severity_width))
            .size(Size::exact(metrics.action_width))
            .size(Size::remainder())
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    centered_label(
                        ui,
                        RichText::new(tool_status_symbol(item, is_installing))
                            .size(TITLE_SIZE)
                            .strong()
                            .color(tool_status_color(ui, item, is_installing)),
                    );
                });
                strip.cell(|ui| {
                    left_label(ui, RichText::new(tool.label()).size(BODY_SIZE).strong());
                });
                strip.cell(|ui| {
                    left_label(
                        ui,
                        RichText::new(state.tr(severity_short_label(item.severity)))
                            .size(BODY_SIZE)
                            .color(severity_color(ui, item.severity)),
                    );
                });
                strip.cell(|ui| {
                    let button_label = if is_installing {
                        state.tr("prepare.installing")
                    } else if installed {
                        state.tr("prepare.reinstall")
                    } else {
                        state.tr("prepare.install")
                    };
                    let button_icon = if is_installing {
                        AppIcon::Loading
                    } else {
                        AppIcon::Download
                    };
                    let response = ui.add_enabled(
                        !any_installing && install_block_reason.is_none(),
                        icon_text_button(ui, button_icon, button_label)
                            .min_size(egui::vec2(metrics.action_width, metrics.row_height)),
                    );
                    if response.clicked() {
                        state.install_dependency_tool(tool);
                    }
                    if let Some(reason) = install_block_reason.as_deref() {
                        response.on_hover_text(state.localize_message(reason));
                    } else if any_installing && !is_installing {
                        response.on_hover_text(
                            state.tr("prepare.another_tool_is_already_being_installed"),
                        );
                    }
                });
                strip.cell(|ui| {
                    left_label(
                        ui,
                        RichText::new(state.localize_message(&tool_status_text(state, item, tool)))
                            .size(BODY_SIZE)
                            .color(detail_color(ui)),
                    );
                });
            });
    });
}

fn centered_label(ui: &mut Ui, text: RichText) {
    ui.with_layout(
        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        |ui| {
            ui.label(text);
        },
    );
}

fn left_label(ui: &mut Ui, text: RichText) {
    ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
        ui.label(text);
    });
}

fn render_prepare_status(ui: &mut Ui, state: &AppState) {
    if state.last_action.trim().is_empty() {
        return;
    }

    ui.add_space(10.0);
    ui.label(
        RichText::new(state.localize_message(&state.last_action))
            .size(SMALL_SIZE)
            .color(detail_color(ui)),
    );
}

fn render_environment_issues(ui: &mut Ui, state: &AppState) {
    let issues = state
        .prepare_requirements()
        .iter()
        .filter(|item| item.action.is_none() && item.needs_attention())
        .collect::<Vec<_>>();

    if issues.is_empty() {
        return;
    }

    ui.add_space(12.0);
    ui.label(
        RichText::new(state.tr("prepare.needs_attention"))
            .size(17.0)
            .strong(),
    );
    ui.add_space(4.0);
    for item in issues {
        render_issue_row(ui, state, item);
        ui.add_space(4.0);
    }
}

fn render_issue_row(ui: &mut Ui, state: &AppState, item: &PrepareRequirement) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_top(|ui| {
            ui.add_sized(
                [
                    ui.spacing().interact_size.y * 0.8,
                    ui.spacing().interact_size.y,
                ],
                egui::Label::new(
                    RichText::new(status_symbol(item.status))
                        .size(TITLE_SIZE)
                        .strong()
                        .color(status_color(ui, item.status)),
                ),
            );
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(state.localize_message(&item.title))
                        .size(BODY_SIZE)
                        .strong(),
                );
                if !item.recommendation.trim().is_empty() {
                    ui.label(
                        RichText::new(state.localize_message(&item.recommendation))
                            .size(SMALL_SIZE)
                            .color(detail_color(ui)),
                    );
                } else {
                    ui.label(
                        RichText::new(state.localize_message(&item.description))
                            .size(SMALL_SIZE)
                            .color(detail_color(ui)),
                    );
                }
                if !item.detail.trim().is_empty() {
                    ui.label(
                        RichText::new(state.localize_message(&item.detail))
                            .size(SMALL_SIZE)
                            .color(warn_color(ui)),
                    );
                }
            });
        });
    });
}

fn render_bottom_actions(ui: &mut Ui, state: &mut AppState) {
    let install_running = state.installing_dependency_tool().is_some();
    let can_install_all = state.prepare_installable_tool_count() > 0;
    let install_block_reason = state.prepare_dependency_install_block_reason();
    let row_height = standard_action_height(ui);
    let install_width = standard_action_width(ui, state);
    let skip_width = standard_action_width(ui, state);

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), row_height),
        egui::Layout::right_to_left(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            let install_all = ui.add_enabled(
                !install_running && can_install_all && install_block_reason.is_none(),
                icon_text_button(ui, AppIcon::Download, state.tr("prepare.install_all"))
                    .min_size(egui::vec2(install_width, row_height)),
            );
            if install_all.clicked() {
                state.install_all_prepare_tools();
            }
            if let Some(reason) = install_block_reason.as_deref() {
                install_all.on_hover_text(state.localize_message(reason));
            }

            let skip = ui.add_enabled(
                !install_running,
                icon_text_button(ui, AppIcon::WindowClose, state.tr("prepare.skip"))
                    .min_size(egui::vec2(skip_width, row_height)),
            );
            if skip.clicked() {
                state.snooze_prepare_tab();
            }
        },
    );
}

fn tool_status_symbol(item: &PrepareRequirement, is_installing: bool) -> &'static str {
    if is_installing {
        "…"
    } else if item.status == PrepareStatus::Ok {
        "✓"
    } else if item.severity == PrepareSeverity::Optional {
        "○"
    } else {
        "×"
    }
}

fn status_symbol(status: PrepareStatus) -> &'static str {
    match status {
        PrepareStatus::Ok => "✓",
        PrepareStatus::Warning => "!",
        PrepareStatus::Missing | PrepareStatus::Failed => "×",
    }
}

fn tool_status_color(ui: &Ui, item: &PrepareRequirement, is_installing: bool) -> Color32 {
    if is_installing {
        warn_color(ui)
    } else if item.status == PrepareStatus::Ok {
        ok_color(ui)
    } else if item.severity == PrepareSeverity::Optional {
        detail_color(ui)
    } else {
        error_color(ui)
    }
}

fn tool_status_text(state: &AppState, item: &PrepareRequirement, tool: DependencyTool) -> String {
    if let Some(progress) = state.dependency_tool_install_progress(tool) {
        if state.installing_dependency_tool() == Some(tool)
            || matches!(progress.stage, ToolInstallStage::Failed)
        {
            return match progress.percent {
                Some(percent)
                    if matches!(
                        progress.stage,
                        ToolInstallStage::Downloading
                            | ToolInstallStage::Extracting
                            | ToolInstallStage::Installing
                    ) =>
                {
                    format!(
                        "{} {percent}%",
                        tool_install_stage_text(state, progress.stage)
                    )
                }
                _ if matches!(progress.stage, ToolInstallStage::Failed)
                    && !progress.message.trim().is_empty() =>
                {
                    state.tr("prepare.install_failed").to_owned()
                }
                _ => tool_install_stage_text(state, progress.stage).to_owned(),
            };
        }
    }

    match item.status {
        PrepareStatus::Ok => state.tr("prepare.status.ready").to_owned(),
        PrepareStatus::Missing if item.severity == PrepareSeverity::Optional => {
            state.tr("prepare.install_later").to_owned()
        }
        PrepareStatus::Missing => state.tr("prepare.status.missing").to_owned(),
        PrepareStatus::Warning => state.tr("prepare.status.warning").to_owned(),
        PrepareStatus::Failed => state.tr("prepare.status.failed").to_owned(),
    }
}

fn tool_install_stage_text(state: &AppState, stage: ToolInstallStage) -> &'static str {
    match stage {
        ToolInstallStage::Preparing => state.tr("tool_install.stage.preparing"),
        ToolInstallStage::Downloading => state.tr("tool_install.stage.downloading"),
        ToolInstallStage::Extracting => state.tr("tool_install.stage.extracting"),
        ToolInstallStage::Installing => state.tr("tool_install.stage.installing"),
        ToolInstallStage::Completed => state.tr("tool_install.stage.completed"),
        ToolInstallStage::Failed => state.tr("tool_install.stage.failed"),
    }
}

fn severity_short_label(severity: PrepareSeverity) -> &'static str {
    match severity {
        PrepareSeverity::Required => "prepare.severity.short.required",
        PrepareSeverity::Recommended => "prepare.severity.short.recommended",
        PrepareSeverity::Optional => "prepare.severity.short.optional",
    }
}

fn severity_color(ui: &Ui, severity: PrepareSeverity) -> Color32 {
    match severity {
        PrepareSeverity::Required => error_color(ui),
        PrepareSeverity::Recommended => warn_color(ui),
        PrepareSeverity::Optional => detail_color(ui),
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
