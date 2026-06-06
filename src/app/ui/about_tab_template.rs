use eframe::egui::Ui;
use egui_taffy::Tui;

use crate::app::state::AppState;
use crate::infrastructure::ManagedComponentId;

use super::about_tab_controls::{AboutComponentRow, AboutHeaderRow, AboutRowMetrics};
use super::xaml_template_renderer::{
    TemplateBlockSlot, show_full_height_page_template, show_template_tui_block,
};
use super::xaml_ui_nodes::{self, TemplateChild, block, fill, fixed_px, gap, rows};

// ABOUT TEMPLATE RULES — this file owns the TemplateTree shape only.
//
// Existing project pattern to follow before editing:
// - `main_tab_template.rs`: resolve frame/context/role objects, then dispatch blocks.
// - `options_tab_template.rs`: root page describes section slots; section modules render content.
// - `xaml_ui_nodes.rs`: rows/cols/auto/fill/fixed_px/gap/block are the layout vocabulary.
//
// Rules for this file:
// 1. Describe page structure with TemplateTree nodes only.
// 2. Do not hand-build egui alignment here. No `ui.with_layout`, no spacer math.
// 3. Row/cell UiElement details belong to `about_tab_controls.rs`.
// 4. Slot painting belongs to `about_tab.rs`.
// 5. Do not create a second taffy/template root inside any block.
//
// 每次處理中，如有發現不符合地方應強力修正；不要只修表面症狀。

const ABOUT_HEADER_TO_LIST_GAP_PX: f32 = 6.0;
const ABOUT_ROW_GAP_PX: f32 = 3.0;
const ABOUT_LIST_TO_NOTES_GAP_PX: f32 = 8.0;

type AboutTabTemplateTree = xaml_ui_nodes::TemplateNode<AboutTabNode>;
type AboutTabTemplateChild = TemplateChild<AboutTabNode>;

pub(super) fn render_about_tab_template(ui: &mut Ui, state: &mut AppState) {
    let template = AboutTabTemplate::resolve(ui, state);
    template.show(ui, state);
}

struct AboutTabTemplate {
    root: AboutTabTemplateTree,
}

#[derive(Clone, Copy)]
enum AboutTabNode {
    HeaderRow(AboutHeaderRow),
    ComponentRow(AboutComponentRow),
    ReleaseNotes,
}

impl AboutTabTemplate {
    fn resolve(ui: &mut Ui, state: &mut AppState) -> Self {
        let component_ids = super::about_tab::about_component_ids(state);
        let row_metrics = AboutRowMetrics::resolve(ui, state, &component_ids);
        let root = about_tab_root_template(row_metrics, &component_ids);

        Self { root }
    }

    fn show(self, ui: &mut Ui, state: &mut AppState) {
        let mut show_block = |slot, node, tui: &mut Tui| {
            show_about_tab_block(slot, node, tui, state);
        };
        show_full_height_page_template(ui, "about-tab-template", self.root, &mut show_block);
    }
}

fn about_tab_root_template(
    row_metrics: AboutRowMetrics,
    component_ids: &[ManagedComponentId],
) -> AboutTabTemplateTree {
    let mut root_rows: Vec<AboutTabTemplateChild> = vec![
        fixed_px(
            row_metrics.row_height(),
            block(AboutTabNode::HeaderRow(row_metrics.header_row())),
        ),
        gap(ABOUT_HEADER_TO_LIST_GAP_PX),
    ];

    for id in component_ids.iter().copied() {
        root_rows.push(fixed_px(
            row_metrics.row_height(),
            block(AboutTabNode::ComponentRow(row_metrics.component_row(id))),
        ));
        root_rows.push(gap(ABOUT_ROW_GAP_PX));
    }

    root_rows.push(gap(ABOUT_LIST_TO_NOTES_GAP_PX));
    root_rows.push(fill(block(AboutTabNode::ReleaseNotes)));

    rows(root_rows)
}

fn show_about_tab_block(
    slot: TemplateBlockSlot,
    node: AboutTabNode,
    tui: &mut Tui,
    state: &mut AppState,
) {
    match node {
        AboutTabNode::HeaderRow(row) => show_template_tui_block(slot, tui, |tui| {
            row.show(tui, state);
        }),
        AboutTabNode::ComponentRow(row) => show_template_tui_block(slot, tui, |tui| {
            row.show(tui, state);
        }),
        AboutTabNode::ReleaseNotes => {
            super::xaml_template_renderer::show_template_ui_block(slot, tui, |ui| {
                super::about_tab::render_release_notes(ui, state);
            });
        }
    }
}
