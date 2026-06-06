//! Shared lightweight TemplateTree helper DSL for egui/taffy screens.
//!
//! The tree is a **layout template**, not a component system and not a VDOM.
//! Page files provide their own content roles and painters. This module only
//! owns the reusable template words and the small IR needed by the taffy and
//! rect-template renderers.
//!
//! DSL dictionary:
//! - `cols([...])`: split the available area into horizontal column slots,
//!   visually `[a] [b] [c]`.
//! - `rows([...])`: split the available area into vertical row slots,
//!   visually `[a] / [b] / [c]`.
//! - `block(role)`: a coarse leaf slot painted by the owning page/section.
//! - `auto(node)`: size this child by its intrinsic/content size on the
//!   parent axis.
//! - `star(weight, node)`: give this child a weighted share of the remaining
//!   parent-axis space.
//! - `fill(node)`: shorthand for `star(1.0, node)` when a child simply fills
//!   remaining space.
//! - `fixed_px(px, node)`: reserve an exact parent-axis size.
//! - `clamp_px(min, ideal, max, node)`: use an ideal parent-axis size with
//!   min/max constraints.
//! - `gap(px)`: insert fixed spacing along the parent container's main axis.
//! - `auto_block_rows([...])`: shorthand for a vertical list of auto-sized
//!   blocks, mainly for settings/section pages.
//!
//! Rule of thumb: containers (`cols`/`rows`) describe shape; child wrappers
//! (`auto`/`star`/`fill`/`fixed_px`/`clamp_px`/`gap`) describe how the parent allocates
//! space; `block` marks the coarse painter boundary. Fine widget details should
//! stay inside the page's painter/block implementation.
//!
//! Performance rules:
//! 1. Keep the tree coarse. Add page/section/panel slots, not badges, padding,
//!    text fragments, icon offsets, or hover details. Node count is layout cost.
//! 2. Prefer stable template shape. Avoid rebuilding a different tree for every
//!    tiny state change. Only add/remove branches for real structural changes.
//! 3. Pick the cheapest backend that fits the layout: taffy template for page
//!    structure, rect template for local manual slot slicing, direct egui code
//!    for tiny hot paths.
//! 4. Do not add more DSL words for convenience unless they reduce repeated
//!    allocation or repeated layout work. Readability sugar alone is no longer
//!    the priority.
//! 5. Keep expensive measurements outside the tree when possible. Text widths,
//!    translated label sizes, and image/texture readiness should be cached or
//!    resolved by the owning painter/block.
//! 6. Treat first-frame instability as a rendering concern. If a block depends
//!    on late texture/text measurement, prefer a narrow one-shot repaint/discard
//!    near that block instead of changing the template shape.
//!
//! TODO(perf):
//! - Audit repeated per-frame template allocation in hot paths. If needed, try a
//!   small array-backed representation before considering broader caching.
//! - Add optional debug logging/overlay for `TemplateTree path -> rect -> block`
//!   to catch zero-size first-frame slots and unexpectedly deep trees.
//! - Consider measurement caches for i18n label widths and header/button text
//!   sizes keyed by locale/text/style/pixels-per-point.
//! - Keep `block(...)` explicit so GPT/Codex keeps the coarse painter boundary;
//!   do not replace it with implicit component-style leaves unless profiling
//!   proves a clear benefit.

/// Container axis used by `rows(...)` and `cols(...)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TemplateAxis {
    /// Vertical row slots: `[a] / [b] / [c]`.
    Rows,
    /// Horizontal column slots: `[a] [b] [c]`.
    Cols,
}

/// Reusable layout template node.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum TemplateNode<Content> {
    /// Splits children into vertical row slots.
    Rows(Vec<TemplateChild<Content>>),
    /// Splits children into horizontal column slots.
    Cols(Vec<TemplateChild<Content>>),
    /// Coarse leaf role. The owning page decides how to paint it.
    Block(Content),
}

impl<Content> TemplateNode<Content> {
    pub(super) fn axis(&self) -> Option<TemplateAxis> {
        match self {
            Self::Rows(_) => Some(TemplateAxis::Rows),
            Self::Cols(_) => Some(TemplateAxis::Cols),
            Self::Block(_) => None,
        }
    }
}

/// Parent-axis sizing semantics for a child slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum TemplateSizing {
    /// Content/intrinsic size on the parent axis.
    Auto,
    /// Weighted share of remaining parent-axis space.
    Star(f32),
    /// Exact parent-axis size in logical pixels.
    FixedPx(f32),
    /// Ideal parent-axis size in logical pixels, bounded by min/max.
    ClampPx { min: f32, ideal: f32, max: f32 },
}

/// A child entry inside `rows(...)` or `cols(...)`.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum TemplateChild<Content> {
    Auto(TemplateNode<Content>),
    Star {
        weight: f32,
        child: TemplateNode<Content>,
    },
    FixedPx {
        px: f32,
        child: TemplateNode<Content>,
    },
    ClampPx {
        min: f32,
        ideal: f32,
        max: f32,
        child: TemplateNode<Content>,
    },
    /// Fixed spacing along the parent container's main axis.
    Gap(f32),
}

/// Creates vertical row slots: `[a] / [b] / [c]`.
pub(super) fn rows<Content>(
    children: impl IntoIterator<Item = TemplateChild<Content>>,
) -> TemplateNode<Content> {
    TemplateNode::Rows(children.into_iter().collect())
}

/// Creates horizontal column slots: `[a] [b] [c]`.
pub(super) fn cols<Content>(
    children: impl IntoIterator<Item = TemplateChild<Content>>,
) -> TemplateNode<Content> {
    TemplateNode::Cols(children.into_iter().collect())
}

/// Marks a coarse paint boundary owned by the current page/section.
pub(super) fn block<Content>(content: Content) -> TemplateNode<Content> {
    TemplateNode::Block(content)
}

/// Builds a vertical list of auto-sized block rows.
///
/// This keeps page templates focused on layout intent instead of repeating
/// `rows([auto(block(...)), ...])` for simple settings/section lists.
pub(super) fn auto_block_rows<Content>(
    contents: impl IntoIterator<Item = Content>,
) -> TemplateNode<Content> {
    rows(contents.into_iter().map(|content| auto(block(content))))
}

/// Sizes a child by its intrinsic/content size on the parent axis.
pub(super) fn auto<Content>(child: TemplateNode<Content>) -> TemplateChild<Content> {
    TemplateChild::Auto(child)
}

/// Gives a child a weighted share of remaining parent-axis space.
pub(super) fn star<Content>(weight: f32, child: TemplateNode<Content>) -> TemplateChild<Content> {
    TemplateChild::Star {
        weight: weight.max(0.0),
        child,
    }
}

/// Fills remaining parent-axis space with the default star weight.
///
/// Use `star(weight, node)` when the relative weight matters; use `fill(node)`
/// when the child simply consumes the remaining space.
pub(super) fn fill<Content>(child: TemplateNode<Content>) -> TemplateChild<Content> {
    star(1.0, child)
}

/// Reserves an exact parent-axis size in logical pixels.
pub(super) fn fixed_px<Content>(px: f32, child: TemplateNode<Content>) -> TemplateChild<Content> {
    TemplateChild::FixedPx {
        px: px.max(0.0),
        child,
    }
}

/// Reserves an ideal parent-axis size bounded by min/max logical pixels.
pub(super) fn clamp_px<Content>(
    min: f32,
    ideal: f32,
    max: f32,
    child: TemplateNode<Content>,
) -> TemplateChild<Content> {
    let min = min.max(0.0);
    let max = max.max(min);
    TemplateChild::ClampPx {
        min,
        ideal: ideal.clamp(min, max),
        max,
        child,
    }
}

/// Inserts fixed spacing along the parent container's main axis.
///
/// In `cols([...])`, this becomes horizontal space; in `rows([...])`,
/// this becomes vertical space.
pub(super) fn gap<Content>(size: f32) -> TemplateChild<Content> {
    TemplateChild::Gap(size.max(0.0))
}
