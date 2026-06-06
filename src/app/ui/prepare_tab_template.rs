use eframe::egui::{ScrollArea, Ui};
use egui_taffy::Tui;

use crate::app::state::AppState;
use crate::infrastructure::DependencyTool;

use super::common::settings_taffy_scroll_content;
use super::semantic_ui_metrics;
use super::xaml_template_renderer::{
    TemplateBlockSlot, show_auto_height_template, show_full_height_page_template,
    show_template_ui_block,
};
use super::xaml_ui_nodes::{self, TemplateChild, auto, block, cols, fill, fixed_px, gap, rows};

type PrepareTabTemplateTree = xaml_ui_nodes::TemplateNode<PrepareTabNode>;
type PrepareContentTemplateTree = xaml_ui_nodes::TemplateNode<PrepareContentNode>;
type PrepareContentTemplateChild = TemplateChild<PrepareContentNode>;

pub(super) fn render_prepare_tab_template(ui: &mut Ui, state: &mut AppState) {
    let template = PrepareTabTemplate::resolve(ui, state);
    template.show(ui, state);
}

struct PrepareTabTemplate {
    context: PrepareTabTemplateContext,
    root: PrepareTabTemplateTree,
}

#[derive(Clone, Copy)]
enum PrepareTabNode {
    ScrollContent,
    BottomActions,
}

#[derive(Clone, Copy)]
enum PrepareContentNode {
    LanguageSelector,
    Header,
    ToolRow(DependencyTool),
    EnvironmentIssues,
    Status,
    Empty,
}

struct PrepareTabTemplateContext {
    tool_row_metrics: super::prepare_tab::ToolRowMetrics,
    tool_row_gap: f32,
    bottom_action_height: f32,
}

impl PrepareTabTemplate {
    fn resolve(ui: &Ui, state: &AppState) -> Self {
        let context = PrepareTabTemplateContext::resolve(ui, state);
        Self {
            root: prepare_tab_root_template(&context),
            context,
        }
    }

    fn show(self, ui: &mut Ui, state: &mut AppState) {
        let context = self.context;
        let mut show_block = |slot, node, tui: &mut Tui| {
            show_prepare_tab_block(slot, node, tui, state, &context);
        };
        show_full_height_page_template(ui, "prepare-tab-template", self.root, &mut show_block);
    }
}

impl PrepareTabTemplateContext {
    fn resolve(ui: &Ui, state: &AppState) -> Self {
        Self {
            tool_row_metrics: super::prepare_tab::tool_row_metrics(ui, state),
            tool_row_gap: semantic_ui_metrics::prepare_tool_row_vertical_spacing_after_each_row(ui),
            bottom_action_height:
                semantic_ui_metrics::prepare_bottom_action_row_height_from_current_control_metrics(
                    ui,
                ),
        }
    }
}

fn prepare_tab_root_template(context: &PrepareTabTemplateContext) -> PrepareTabTemplateTree {
    rows([
        fill(block(PrepareTabNode::ScrollContent)),
        fixed_px(
            context.bottom_action_height,
            block(PrepareTabNode::BottomActions),
        ),
    ])
}

fn show_prepare_tab_block(
    slot: TemplateBlockSlot,
    node: PrepareTabNode,
    tui: &mut Tui,
    state: &mut AppState,
    context: &PrepareTabTemplateContext,
) {
    match node {
        PrepareTabNode::ScrollContent => show_template_ui_block(slot, tui, |ui| {
            render_prepare_scroll_content(ui, state, context);
        }),
        PrepareTabNode::BottomActions => show_template_ui_block(slot, tui, |ui| {
            super::prepare_tab::render_bottom_actions(ui, state);
        }),
    }
}

fn render_prepare_scroll_content(
    ui: &mut Ui,
    state: &mut AppState,
    context: &PrepareTabTemplateContext,
) {
    ScrollArea::vertical()
        .id_salt("prepare-mode-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            settings_taffy_scroll_content(ui, "prepare-root-content-taffy", |tui| {
                let template = prepare_content_template(state, context);
                let mut show_block = |slot, node, tui: &mut Tui| {
                    show_prepare_content_block(slot, node, tui, state, context);
                };
                show_auto_height_template(template, tui, &mut show_block);
            });
        });
}

fn prepare_content_template(
    state: &AppState,
    context: &PrepareTabTemplateContext,
) -> PrepareContentTemplateTree {
    let mut rows_children: Vec<PrepareContentTemplateChild> = vec![
        auto(block(PrepareContentNode::LanguageSelector)),
        gap(semantic_ui_metrics::prepare_root_language_to_header_vertical_spacing()),
        auto(block(PrepareContentNode::Header)),
        gap(semantic_ui_metrics::prepare_primary_section_vertical_spacing()),
    ];

    let visible_tools = [
        DependencyTool::YtDlp,
        DependencyTool::Deno,
        DependencyTool::Ffmpeg,
    ]
    .into_iter()
    .filter(|tool| super::prepare_tab::has_tool_requirement(state, *tool))
    .collect::<Vec<_>>();

    for (index, tool) in visible_tools.iter().copied().enumerate() {
        rows_children.push(fixed_px(
            context.tool_row_metrics.row_height,
            prepare_tool_row_template(tool, context),
        ));
        if index + 1 < visible_tools.len() {
            rows_children.push(gap(context.tool_row_gap));
        }
    }

    if super::prepare_tab::has_environment_issues(state) {
        rows_children.push(gap(
            semantic_ui_metrics::prepare_primary_section_vertical_spacing(),
        ));
        rows_children.push(auto(block(PrepareContentNode::EnvironmentIssues)));
    }

    if state.prepare_footer_status_text().is_some() {
        rows_children.push(gap(
            semantic_ui_metrics::prepare_primary_section_vertical_spacing(),
        ));
        rows_children.push(auto(block(PrepareContentNode::Status)));
    }

    rows(rows_children)
}

fn prepare_tool_row_template(
    tool: DependencyTool,
    context: &PrepareTabTemplateContext,
) -> PrepareContentTemplateTree {
    cols([
        gap(semantic_ui_metrics::prepare_tool_rows_content_left_indent()),
        fixed_px(
            context.tool_row_metrics.row_width,
            block(PrepareContentNode::ToolRow(tool)),
        ),
        fill(block(PrepareContentNode::Empty)),
    ])
}

fn show_prepare_content_block(
    slot: TemplateBlockSlot,
    node: PrepareContentNode,
    tui: &mut Tui,
    state: &mut AppState,
    context: &PrepareTabTemplateContext,
) {
    match node {
        PrepareContentNode::LanguageSelector => show_template_ui_block(slot, tui, |ui| {
            super::prepare_tab::render_language_selector(ui, state);
        }),
        PrepareContentNode::Header => show_template_ui_block(slot, tui, |ui| {
            super::prepare_tab::render_header(ui, state);
        }),
        PrepareContentNode::ToolRow(tool) => show_template_ui_block(slot, tui, |ui| {
            super::prepare_tab::render_tool_row_slot(ui, state, tool, &context.tool_row_metrics);
        }),
        PrepareContentNode::EnvironmentIssues => show_template_ui_block(slot, tui, |ui| {
            super::prepare_tab::render_environment_issues(ui, state);
        }),
        PrepareContentNode::Status => show_template_ui_block(slot, tui, |ui| {
            super::prepare_tab::render_prepare_status(ui, state);
        }),
        PrepareContentNode::Empty => show_template_ui_block(slot, tui, |_| {}),
    }
}
