//! Rect-only TemplateTree resolver for local manual layouts.
//!
//! Use this when a region only needs deterministic slot slicing and does not
//! need taffy. This is the cheaper path for compact controls, fixed panels, and
//! other small hot layouts. Keep it block-only; nested containers belong in the
//! taffy renderer unless profiling says otherwise.
//!

use eframe::egui::{self, Ui};

use super::xaml_ui_nodes::{TemplateAxis, TemplateChild, TemplateNode};

enum RectSlot<Content> {
    Block(Content, MainSize),
    Gap(f32),
}

#[derive(Clone, Copy)]
enum MainSize {
    Fixed(f32),
    Star(f32),
}

pub(super) fn show_rect_template<Content>(
    rect: egui::Rect,
    template: TemplateNode<Content>,
    auto_main_size: &mut impl FnMut(&Content, TemplateAxis, f32) -> f32,
    show_block: &mut impl FnMut(Content, egui::Rect),
) {
    let rect = normalized_rect(rect);
    let (axis, children) = match template {
        TemplateNode::Rows(children) => (TemplateAxis::Rows, children),
        TemplateNode::Cols(children) => (TemplateAxis::Cols, children),
        TemplateNode::Block(content) => return show_block(content, rect),
    };

    let (main_start, main_available, cross) = match axis {
        TemplateAxis::Rows => (rect.top(), rect.height(), rect.width()),
        TemplateAxis::Cols => (rect.left(), rect.width(), rect.height()),
    };
    let slots = children
        .into_iter()
        .filter_map(|child| rect_slot(child, axis, cross, auto_main_size))
        .collect::<Vec<_>>();
    let fixed_total = slots.iter().map(RectSlot::fixed_main_size).sum::<f32>();
    let star_total = slots.iter().map(RectSlot::star_weight).sum::<f32>();
    let star_unit = if star_total > 0.0 {
        ((main_available - fixed_total).max(0.0)) / star_total
    } else {
        0.0
    };

    let mut cursor = main_start;
    for slot in slots {
        let size = slot.main_px(star_unit).max(0.0);
        let next = cursor + size;
        if let RectSlot::Block(content, _) = slot {
            let slot_rect = rect_from_main_range(rect, axis, cursor, next);
            if slot_rect.width() > 1.0 && slot_rect.height() > 1.0 {
                show_block(content, slot_rect);
            }
        }
        cursor = next;
    }
}

pub(super) fn show_ui_at_rect<R>(
    ui: &mut Ui,
    rect: egui::Rect,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_width(rect.width());
        ui.set_height(rect.height());
        add_contents(ui)
    })
    .inner
}

fn rect_slot<Content>(
    child: TemplateChild<Content>,
    axis: TemplateAxis,
    cross: f32,
    auto_main_size: &mut impl FnMut(&Content, TemplateAxis, f32) -> f32,
) -> Option<RectSlot<Content>> {
    match child {
        TemplateChild::Auto(TemplateNode::Block(content)) => {
            let size = auto_main_size(&content, axis, cross).max(0.0);
            Some(RectSlot::Block(content, MainSize::Fixed(size)))
        }
        TemplateChild::Star {
            weight,
            child: TemplateNode::Block(content),
        } => Some(RectSlot::Block(content, MainSize::Star(weight.max(0.0)))),
        TemplateChild::FixedPx {
            px,
            child: TemplateNode::Block(content),
        } => Some(RectSlot::Block(content, MainSize::Fixed(px.max(0.0)))),
        TemplateChild::ClampPx {
            ideal,
            child: TemplateNode::Block(content),
            ..
        } => Some(RectSlot::Block(content, MainSize::Fixed(ideal.max(0.0)))),
        TemplateChild::Gap(size) => Some(RectSlot::Gap(size.max(0.0))),
        _ => None,
    }
}

impl<Content> RectSlot<Content> {
    fn fixed_main_size(&self) -> f32 {
        match self {
            Self::Block(_, MainSize::Fixed(size)) | Self::Gap(size) => *size,
            Self::Block(_, MainSize::Star(_)) => 0.0,
        }
    }

    fn star_weight(&self) -> f32 {
        match self {
            Self::Block(_, MainSize::Star(weight)) => *weight,
            Self::Block(_, MainSize::Fixed(_)) | Self::Gap(_) => 0.0,
        }
    }

    fn main_px(&self, star_unit: f32) -> f32 {
        match self {
            Self::Block(_, MainSize::Fixed(size)) | Self::Gap(size) => *size,
            Self::Block(_, MainSize::Star(weight)) => star_unit * *weight,
        }
    }
}

fn normalized_rect(rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        rect.min,
        egui::pos2(
            rect.right().max(rect.left() + 1.0),
            rect.bottom().max(rect.top() + 1.0),
        ),
    )
}

fn rect_from_main_range(rect: egui::Rect, axis: TemplateAxis, start: f32, end: f32) -> egui::Rect {
    match axis {
        TemplateAxis::Rows => egui::Rect::from_min_max(
            egui::pos2(rect.left(), start),
            egui::pos2(rect.right(), end.min(rect.bottom().max(start))),
        ),
        TemplateAxis::Cols => egui::Rect::from_min_max(
            egui::pos2(start, rect.top()),
            egui::pos2(end.min(rect.right().max(start)), rect.bottom()),
        ),
    }
}
