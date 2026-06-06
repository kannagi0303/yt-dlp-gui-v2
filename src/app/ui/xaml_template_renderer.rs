//! Shared taffy-backed renderer for the lightweight TemplateTree DSL.
//!
//! Performance notes:
//! - This renderer should stay a lowering layer, not a second UI framework.
//! - Keep page-specific measurement and drawing inside block painters.
//! - Avoid adding per-page callback hooks that force lifetime-heavy generic
//!   closures or repeated full-tree style recomputation.
//! - Prefer adding small shared helpers only when they remove repeated work or
//!   repeated mounting boilerplate across multiple pages.
//!

use eframe::egui::Ui;
use egui_taffy::{Tui, TuiBuilderLogic as _, taffy, tui};

use super::xaml_taffy_styles;
use super::xaml_ui_nodes::{TemplateAxis, TemplateChild, TemplateNode, TemplateSizing};

#[derive(Clone)]
pub(super) enum TemplateBlockSlot {
    Root,
    Child {
        axis: TemplateAxis,
        sizing: TemplateSizing,
        style: taffy::Style,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TemplatePageFrame {
    available_width: f32,
    available_height: f32,
}

impl TemplatePageFrame {
    pub(super) fn resolve(ui: &Ui) -> Self {
        Self {
            available_width: ui.available_width(),
            available_height: ui.available_height(),
        }
    }

    pub(super) fn show_vertical(
        self,
        ui: &mut Ui,
        id_salt: &'static str,
        add_contents: impl FnOnce(&mut Tui),
    ) {
        tui(ui, ui.id().with(id_salt))
            .reserve_width(self.available_width)
            .reserve_height(self.available_height)
            .style(
                xaml_taffy_styles::XamlTaffyElement::vertical_root(self.available_height).style(),
            )
            .show(add_contents);
    }
}

pub(super) fn show_full_height_page_template<Content>(
    ui: &mut Ui,
    id_salt: &'static str,
    template: TemplateNode<Content>,
    show_block: &mut impl FnMut(TemplateBlockSlot, Content, &mut Tui),
) {
    TemplatePageFrame::resolve(ui).show_vertical(ui, id_salt, |tui| {
        show_template(template, tui, show_block);
    });
}

pub(super) fn show_template<Content>(
    template: TemplateNode<Content>,
    tui: &mut Tui,
    show_block: &mut impl FnMut(TemplateBlockSlot, Content, &mut Tui),
) {
    show_template_with_root_style(
        template,
        tui,
        show_block,
        xaml_taffy_styles::xaml_template_root_style,
    );
}

pub(super) fn show_auto_height_template<Content>(
    template: TemplateNode<Content>,
    tui: &mut Tui,
    show_block: &mut impl FnMut(TemplateBlockSlot, Content, &mut Tui),
) {
    show_template_with_root_style(
        template,
        tui,
        show_block,
        xaml_taffy_styles::xaml_template_auto_root_style,
    );
}

pub(super) fn show_auto_height_tui_template<Content>(
    template: TemplateNode<Content>,
    tui: &mut Tui,
    show_content: &mut impl FnMut(Content, &mut Tui),
) {
    let mut show_block = |slot, content, tui: &mut Tui| {
        show_template_tui_block(slot, tui, |tui| show_content(content, tui));
    };
    show_auto_height_template(template, tui, &mut show_block);
}

pub(super) fn show_template_ui_block(
    slot: TemplateBlockSlot,
    tui: &mut Tui,
    add_contents: impl FnOnce(&mut Ui),
) {
    match slot {
        TemplateBlockSlot::Root => tui
            .style(xaml_taffy_styles::xaml_template_auto_root_style(
                TemplateAxis::Rows,
            ))
            .ui(add_contents),
        TemplateBlockSlot::Child { style, .. } => tui.style(style).ui(add_contents),
    }
}

pub(super) fn show_template_tui_block(
    slot: TemplateBlockSlot,
    tui: &mut Tui,
    add_contents: impl FnOnce(&mut Tui),
) {
    match slot {
        TemplateBlockSlot::Root => add_contents(tui),
        TemplateBlockSlot::Child { style, .. } => tui.style(style).add(add_contents),
    }
}

fn show_template_with_root_style<Content>(
    template: TemplateNode<Content>,
    tui: &mut Tui,
    show_block: &mut impl FnMut(TemplateBlockSlot, Content, &mut Tui),
    root_style: impl Fn(TemplateAxis) -> taffy::Style,
) {
    match template {
        TemplateNode::Rows(children) => {
            tui.style(root_style(TemplateAxis::Rows))
                .add(|tui| show_template_children(TemplateAxis::Rows, children, tui, show_block));
        }
        TemplateNode::Cols(children) => {
            tui.style(root_style(TemplateAxis::Cols))
                .add(|tui| show_template_children(TemplateAxis::Cols, children, tui, show_block));
        }
        TemplateNode::Block(content) => show_block(TemplateBlockSlot::Root, content, tui),
    }
}

fn show_template_children<Content>(
    axis: TemplateAxis,
    children: Vec<TemplateChild<Content>>,
    tui: &mut Tui,
    show_block: &mut impl FnMut(TemplateBlockSlot, Content, &mut Tui),
) {
    for child in children {
        match child {
            TemplateChild::Auto(node) => {
                show_template_child(axis, TemplateSizing::Auto, node, tui, show_block);
            }
            TemplateChild::Star { weight, child } => {
                show_template_child(axis, TemplateSizing::Star(weight), child, tui, show_block);
            }
            TemplateChild::FixedPx { px, child } => {
                show_template_child(axis, TemplateSizing::FixedPx(px), child, tui, show_block);
            }
            TemplateChild::ClampPx {
                min,
                ideal,
                max,
                child,
            } => {
                show_template_child(
                    axis,
                    TemplateSizing::ClampPx { min, ideal, max },
                    child,
                    tui,
                    show_block,
                );
            }
            TemplateChild::Gap(size) => {
                tui.style(xaml_taffy_styles::xaml_template_gap_style(axis, size))
                    .ui(|_| {});
            }
        }
    }
}

fn show_template_child<Content>(
    axis: TemplateAxis,
    sizing: TemplateSizing,
    child: TemplateNode<Content>,
    tui: &mut Tui,
    show_block: &mut impl FnMut(TemplateBlockSlot, Content, &mut Tui),
) {
    let style = xaml_taffy_styles::xaml_template_child_style(axis, sizing, child.axis());
    match child {
        TemplateNode::Rows(children) => {
            tui.style(style)
                .add(|tui| show_template_children(TemplateAxis::Rows, children, tui, show_block));
        }
        TemplateNode::Cols(children) => {
            tui.style(style)
                .add(|tui| show_template_children(TemplateAxis::Cols, children, tui, show_block));
        }
        TemplateNode::Block(content) => show_block(
            TemplateBlockSlot::Child {
                axis,
                sizing,
                style,
            },
            content,
            tui,
        ),
    }
}
