use crate::app::state::AppState;
use eframe::egui::Ui;
use egui_taffy::Tui;

use super::xaml_template_renderer::{TemplateBlockSlot, show_full_height_page_template};
use super::xaml_ui_nodes::{self, block, fill, fixed_px, rows};
use super::{format_picker_content, format_picker_header};

type FormatPickerTemplateTree = xaml_ui_nodes::TemplateNode<FormatPickerNode>;

pub(super) fn render_format_picker_screen(ui: &mut Ui, state: &mut AppState) {
    let template = FormatPickerTemplate::resolve(ui, state);
    template.show(ui, state);
}

struct FormatPickerTemplate {
    context: FormatPickerTemplateContext,
    root: FormatPickerTemplateTree,
}

#[derive(Clone, Copy)]
enum FormatPickerNode {
    Header,
    Content,
}

struct FormatPickerTemplateContext {
    header: format_picker_header::FormatPickerHeaderContext,
}

impl FormatPickerTemplate {
    fn resolve(ui: &mut Ui, state: &mut AppState) -> Self {
        let context = FormatPickerTemplateContext::resolve(ui, state);
        let root = format_picker_root_template(&context);

        Self { context, root }
    }

    fn show(self, ui: &mut Ui, state: &mut AppState) {
        let context = self.context;
        let mut show_block = |slot, node, tui: &mut Tui| {
            show_format_picker_block(slot, node, tui, state, &context);
        };
        show_full_height_page_template(ui, "format-picker-template", self.root, &mut show_block);
    }
}

impl FormatPickerTemplateContext {
    fn resolve(ui: &Ui, state: &AppState) -> Self {
        Self {
            header: format_picker_header::FormatPickerHeaderContext::resolve(ui, state),
        }
    }
}

fn format_picker_root_template(context: &FormatPickerTemplateContext) -> FormatPickerTemplateTree {
    use FormatPickerNode::*;

    rows([
        fixed_px(context.header.height(), block(Header)),
        fill(block(Content)),
    ])
}

fn show_format_picker_block(
    slot: TemplateBlockSlot,
    node: FormatPickerNode,
    tui: &mut Tui,
    state: &mut AppState,
    context: &FormatPickerTemplateContext,
) {
    match node {
        FormatPickerNode::Header => {
            format_picker_header::show_header_block(slot, tui, state, &context.header)
        }
        FormatPickerNode::Content => format_picker_content::show_content_block(slot, tui, state),
    }
}
