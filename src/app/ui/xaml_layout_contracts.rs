/// XAML-like layout contracts used by the egui UI layer.
///
/// This module intentionally stays independent from egui, taffy, AppState, and
/// product logic. It only describes sizing and alignment contracts so the same
/// model can be moved to another egui project later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HorizontalAlignment {
    Left,
    Center,
    Right,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VerticalAlignment {
    Top,
    Center,
    Bottom,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum LayoutLength {
    Auto,
    Pixel(f32),
    Star(f32),
}

impl LayoutLength {
    pub(super) fn pixel_non_negative(value: f32) -> Self {
        Self::Pixel(value.max(0.0))
    }

    pub(super) fn star_or_one(value: f32) -> Self {
        Self::Star(value.max(1.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct LayoutSize {
    pub(super) width: f32,
    pub(super) height: f32,
}

impl LayoutSize {
    pub(super) const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    pub(super) fn new(width: f32, height: f32) -> Self {
        Self {
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }

    pub(super) fn max(self, other: Self) -> Self {
        Self::new(self.width.max(other.width), self.height.max(other.height))
    }

    pub(super) fn to_array(self) -> [f32; 2] {
        [self.width, self.height]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ParentLayoutSlot {
    pub(super) width: f32,
    pub(super) height: f32,
}

impl ParentLayoutSlot {
    pub(super) fn new(width: f32, height: f32) -> Self {
        Self {
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ElementLayoutContract {
    pub(super) width: LayoutLength,
    pub(super) height: LayoutLength,
    pub(super) min_width: f32,
    pub(super) min_height: f32,
    pub(super) max_width: f32,
    pub(super) max_height: f32,
    pub(super) horizontal_alignment: HorizontalAlignment,
    pub(super) vertical_alignment: VerticalAlignment,
}

impl Default for ElementLayoutContract {
    fn default() -> Self {
        Self {
            width: LayoutLength::Auto,
            height: LayoutLength::Auto,
            min_width: 0.0,
            min_height: 0.0,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            horizontal_alignment: HorizontalAlignment::Stretch,
            vertical_alignment: VerticalAlignment::Stretch,
        }
    }
}

impl ElementLayoutContract {
    pub(super) fn new(width: LayoutLength, height: LayoutLength) -> Self {
        Self {
            width,
            height,
            ..Self::default()
        }
    }

    pub(super) fn with_min_size(mut self, min_width: f32, min_height: f32) -> Self {
        self.min_width = min_width.max(0.0);
        self.min_height = min_height.max(0.0);
        self
    }

    pub(super) fn with_max_size(mut self, max_width: f32, max_height: f32) -> Self {
        self.max_width = max_width.max(self.min_width);
        self.max_height = max_height.max(self.min_height);
        self
    }

    pub(super) fn with_alignment(
        mut self,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> Self {
        self.horizontal_alignment = horizontal_alignment;
        self.vertical_alignment = vertical_alignment;
        self
    }

    pub(super) fn resolve_size_for_parent_slot(
        self,
        parent_slot: ParentLayoutSlot,
        intrinsic_size: LayoutSize,
    ) -> LayoutSize {
        LayoutSize::new(
            resolve_axis_size(
                self.width,
                self.horizontal_alignment == HorizontalAlignment::Stretch,
                parent_slot.width,
                intrinsic_size.width,
                self.min_width,
                self.max_width,
            ),
            resolve_axis_size(
                self.height,
                self.vertical_alignment == VerticalAlignment::Stretch,
                parent_slot.height,
                intrinsic_size.height,
                self.min_height,
                self.max_height,
            ),
        )
    }

    pub(super) fn fill_parent_slot() -> Self {
        Self::new(LayoutLength::Star(1.0), LayoutLength::Star(1.0))
            .with_alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
    }

    pub(super) fn auto_width_fill_height(minimum_width: f32, minimum_height: f32) -> Self {
        Self::new(LayoutLength::Auto, LayoutLength::Star(1.0))
            .with_min_size(minimum_width, minimum_height)
            .with_alignment(HorizontalAlignment::Left, VerticalAlignment::Stretch)
    }

    pub(super) fn star_width_fill_height(minimum_width: f32, minimum_height: f32) -> Self {
        Self::new(LayoutLength::Star(1.0), LayoutLength::Star(1.0))
            .with_min_size(minimum_width, minimum_height)
            .with_alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UiElementKind {
    Generic,
    Button,
    IconTextButton,
    SelectableButton,
    TextBox,
    SingleLineTextInput,
    IconButton,
    Label,
    Spinner,
    Spacer,
    Root,
    Row,
    Panel,
    ContentPresenter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct UiElement {
    pub(super) kind: UiElementKind,
    pub(super) layout: ElementLayoutContract,
    pub(super) intrinsic_size: LayoutSize,
    pub(super) horizontal_content_alignment: HorizontalAlignment,
    pub(super) vertical_content_alignment: VerticalAlignment,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct MeasuredUiElement {
    pub(super) kind: UiElementKind,
    pub(super) size: LayoutSize,
    pub(super) horizontal_alignment: HorizontalAlignment,
    pub(super) vertical_alignment: VerticalAlignment,
    pub(super) horizontal_content_alignment: HorizontalAlignment,
    pub(super) vertical_content_alignment: VerticalAlignment,
}

impl MeasuredUiElement {
    pub(super) fn to_array(self) -> [f32; 2] {
        self.size.to_array()
    }
}

impl UiElement {
    pub(super) fn new(kind: UiElementKind, intrinsic_size: LayoutSize) -> Self {
        Self {
            kind,
            layout: ElementLayoutContract::default()
                .with_min_size(intrinsic_size.width, intrinsic_size.height),
            intrinsic_size,
            horizontal_content_alignment: HorizontalAlignment::Center,
            vertical_content_alignment: VerticalAlignment::Center,
        }
    }

    pub(super) fn from_intrinsic_size(intrinsic_size: LayoutSize) -> Self {
        Self::new(UiElementKind::Generic, intrinsic_size)
    }

    pub(super) fn kind(mut self, kind: UiElementKind) -> Self {
        self.kind = kind;
        self
    }

    pub(super) fn with_kind(self, kind: UiElementKind) -> Self {
        self.kind(kind)
    }

    pub(super) fn width(mut self, width: LayoutLength) -> Self {
        self.layout.width = width;
        self
    }

    pub(super) fn with_width(self, width: LayoutLength) -> Self {
        self.width(width)
    }

    pub(super) fn height(mut self, height: LayoutLength) -> Self {
        self.layout.height = height;
        self
    }

    pub(super) fn with_height(self, height: LayoutLength) -> Self {
        self.height(height)
    }

    pub(super) fn min_width(mut self, min_width: f32) -> Self {
        self.layout.min_width = min_width.max(0.0);
        self
    }

    pub(super) fn min_height(mut self, min_height: f32) -> Self {
        self.layout.min_height = min_height.max(0.0);
        self
    }

    pub(super) fn min_size(self, min_width: f32, min_height: f32) -> Self {
        self.min_width(min_width).min_height(min_height)
    }

    pub(super) fn with_min_size(self, min_width: f32, min_height: f32) -> Self {
        self.min_size(min_width, min_height)
    }

    pub(super) fn max_width(mut self, max_width: f32) -> Self {
        self.layout.max_width = max_width.max(self.layout.min_width);
        self
    }

    pub(super) fn max_height(mut self, max_height: f32) -> Self {
        self.layout.max_height = max_height.max(self.layout.min_height);
        self
    }

    pub(super) fn max_size(self, max_width: f32, max_height: f32) -> Self {
        self.max_width(max_width).max_height(max_height)
    }

    pub(super) fn with_max_size(self, max_width: f32, max_height: f32) -> Self {
        self.max_size(max_width, max_height)
    }

    pub(super) fn horizontal_alignment(
        mut self,
        horizontal_alignment: HorizontalAlignment,
    ) -> Self {
        self.layout.horizontal_alignment = horizontal_alignment;
        self
    }

    pub(super) fn vertical_alignment(mut self, vertical_alignment: VerticalAlignment) -> Self {
        self.layout.vertical_alignment = vertical_alignment;
        self
    }

    pub(super) fn alignment(
        self,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> Self {
        self.horizontal_alignment(horizontal_alignment)
            .vertical_alignment(vertical_alignment)
    }

    pub(super) fn with_alignment(
        self,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> Self {
        self.alignment(horizontal_alignment, vertical_alignment)
    }

    pub(super) fn horizontal_content_alignment(
        mut self,
        horizontal_content_alignment: HorizontalAlignment,
    ) -> Self {
        self.horizontal_content_alignment = horizontal_content_alignment;
        self
    }

    pub(super) fn vertical_content_alignment(
        mut self,
        vertical_content_alignment: VerticalAlignment,
    ) -> Self {
        self.vertical_content_alignment = vertical_content_alignment;
        self
    }

    pub(super) fn content_alignment(
        self,
        horizontal_content_alignment: HorizontalAlignment,
        vertical_content_alignment: VerticalAlignment,
    ) -> Self {
        self.horizontal_content_alignment(horizontal_content_alignment)
            .vertical_content_alignment(vertical_content_alignment)
    }

    pub(super) fn with_content_alignment(
        self,
        horizontal_content_alignment: HorizontalAlignment,
        vertical_content_alignment: VerticalAlignment,
    ) -> Self {
        self.content_alignment(horizontal_content_alignment, vertical_content_alignment)
    }

    pub(super) fn auto_width_stretch_height(intrinsic_size: LayoutSize) -> Self {
        Self::from_intrinsic_size(intrinsic_size)
            .width(LayoutLength::Auto)
            .height(LayoutLength::Star(1.0))
            .alignment(HorizontalAlignment::Left, VerticalAlignment::Stretch)
            .content_alignment(HorizontalAlignment::Center, VerticalAlignment::Center)
    }

    pub(super) fn stretch_width_stretch_height(minimum_size: LayoutSize) -> Self {
        Self::from_intrinsic_size(minimum_size)
            .width(LayoutLength::Star(1.0))
            .height(LayoutLength::Star(1.0))
            .alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
            .content_alignment(HorizontalAlignment::Center, VerticalAlignment::Center)
    }

    pub(super) fn button(intrinsic_size: LayoutSize) -> Self {
        Self::auto_width_stretch_height(intrinsic_size).kind(UiElementKind::Button)
    }

    pub(super) fn icon_text_button(intrinsic_size: LayoutSize) -> Self {
        Self::auto_width_stretch_height(intrinsic_size).kind(UiElementKind::IconTextButton)
    }

    pub(super) fn selectable_button(intrinsic_size: LayoutSize) -> Self {
        Self::auto_width_stretch_height(intrinsic_size).kind(UiElementKind::SelectableButton)
    }

    pub(super) fn text_box(row: SingleLineControlRowContract) -> Self {
        Self::stretch_width_stretch_height(LayoutSize::new(0.0, row.height))
            .kind(UiElementKind::TextBox)
    }

    pub(super) fn single_line_text_input(row: SingleLineControlRowContract) -> Self {
        Self::text_box(row).kind(UiElementKind::SingleLineTextInput)
    }

    pub(super) fn fixed_width_stretch_height(
        width: f32,
        row: SingleLineControlRowContract,
    ) -> Self {
        Self::from_intrinsic_size(LayoutSize::new(width, row.height))
            .width(LayoutLength::Pixel(width.max(0.0)))
            .height(LayoutLength::Star(1.0))
            .alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
            .content_alignment(HorizontalAlignment::Center, VerticalAlignment::Center)
    }

    pub(super) fn fixed_size(width: f32, height: f32) -> Self {
        let size = LayoutSize::new(width, height);
        Self::from_intrinsic_size(size)
            .width(LayoutLength::Pixel(size.width))
            .height(LayoutLength::Pixel(size.height))
            .alignment(HorizontalAlignment::Center, VerticalAlignment::Center)
            .content_alignment(HorizontalAlignment::Center, VerticalAlignment::Center)
    }

    pub(super) fn square(size: f32) -> Self {
        Self::fixed_size(size, size)
    }

    pub(super) fn spinner_square(size: f32) -> Self {
        Self::square(size).kind(UiElementKind::Spinner)
    }

    pub(super) fn spacer(width: f32, row: SingleLineControlRowContract) -> Self {
        Self::fixed_width_stretch_height(width, row).kind(UiElementKind::Spacer)
    }

    pub(super) fn fixed_height_spacer(height: f32) -> Self {
        Self::fixed_size(0.0, height).kind(UiElementKind::Spacer)
    }

    pub(super) fn fixed_height_row(height: f32) -> Self {
        let height = height.max(0.0);
        Self::from_intrinsic_size(LayoutSize::new(0.0, height))
            .width(LayoutLength::Star(1.0))
            .height(LayoutLength::Pixel(height))
            .alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
            .content_alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
            .kind(UiElementKind::Row)
    }

    pub(super) fn vertical_root(height: f32) -> Self {
        let height = height.max(0.0);
        Self::from_intrinsic_size(LayoutSize::new(0.0, height))
            .width(LayoutLength::Star(1.0))
            .height(LayoutLength::Pixel(height))
            .alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
            .content_alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
            .kind(UiElementKind::Root)
    }

    pub(super) fn fill_content_presenter() -> Self {
        Self::from_intrinsic_size(LayoutSize::ZERO)
            .width(LayoutLength::Star(1.0))
            .height(LayoutLength::Star(1.0))
            .alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
            .content_alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
            .kind(UiElementKind::ContentPresenter)
    }

    pub(super) fn auto_content_panel() -> Self {
        Self::from_intrinsic_size(LayoutSize::ZERO)
            .width(LayoutLength::Star(1.0))
            .height(LayoutLength::Auto)
            .alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
            .content_alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
            .kind(UiElementKind::Panel)
    }

    pub(super) fn icon_button_square(row: SingleLineControlRowContract) -> Self {
        Self::auto_width_stretch_height(LayoutSize::new(row.height, row.height))
            .kind(UiElementKind::IconButton)
    }

    pub(super) fn label(intrinsic_size: LayoutSize) -> Self {
        Self::auto_width_stretch_height(intrinsic_size).kind(UiElementKind::Label)
    }

    pub(super) fn measure(self, parent_slot: ParentLayoutSlot) -> LayoutSize {
        self.layout
            .resolve_size_for_parent_slot(parent_slot, self.intrinsic_size)
    }

    pub(super) fn measure_element(self, parent_slot: ParentLayoutSlot) -> MeasuredUiElement {
        MeasuredUiElement {
            kind: self.kind,
            size: self.measure(parent_slot),
            horizontal_alignment: self.layout.horizontal_alignment,
            vertical_alignment: self.layout.vertical_alignment,
            horizontal_content_alignment: self.horizontal_content_alignment,
            vertical_content_alignment: self.vertical_content_alignment,
        }
    }

    pub(super) fn measure_for_parent_slot(self, parent_slot: ParentLayoutSlot) -> LayoutSize {
        self.measure(parent_slot)
    }

    pub(super) fn measure_element_for_parent_slot(
        self,
        parent_slot: ParentLayoutSlot,
    ) -> MeasuredUiElement {
        self.measure_element(parent_slot)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SingleLineControlRowContract {
    pub(super) height: f32,
}

impl SingleLineControlRowContract {
    pub(super) fn new(height: f32) -> Self {
        Self {
            height: height.max(1.0),
        }
    }

    pub(super) fn parent_slot_for_width(self, width: f32) -> ParentLayoutSlot {
        ParentLayoutSlot::new(width, self.height)
    }

    pub(super) fn measure_ui_element(self, element: UiElement, parent_width: f32) -> LayoutSize {
        self.measure_element(element, parent_width).size
    }

    pub(super) fn measure_element(
        self,
        element: UiElement,
        parent_width: f32,
    ) -> MeasuredUiElement {
        element.measure_element(self.parent_slot_for_width(parent_width))
    }

    pub(super) fn measure_child(self, child: UiElement, parent_width: f32) -> LayoutSize {
        self.measure_ui_element(child, parent_width)
    }

    pub(super) fn measure_auto_width_ui_element(self, element: UiElement) -> LayoutSize {
        self.measure_auto_width_element(element).size
    }

    pub(super) fn measure_auto_width_element(self, element: UiElement) -> MeasuredUiElement {
        self.measure_element(element, element.intrinsic_size.width)
    }

    pub(super) fn measure_auto_width_child(self, child: UiElement) -> LayoutSize {
        self.measure_auto_width_ui_element(child)
    }

    pub(super) fn measure_auto_width_ui_element_sequence(
        self,
        elements: impl IntoIterator<Item = UiElement>,
        gap: f32,
    ) -> LayoutSize {
        let mut count = 0usize;
        let mut total_width: f32 = 0.0;
        let mut max_height: f32 = self.height;

        for element in elements {
            let measured = self.measure_auto_width_ui_element(element);
            total_width += measured.width;
            max_height = max_height.max(measured.height);
            count += 1;
        }

        if count > 1 {
            total_width += gap.max(0.0) * (count - 1) as f32;
        }

        LayoutSize::new(total_width, max_height)
    }

    pub(super) fn measure_stretch_width_ui_element(
        self,
        element: UiElement,
        available_width: f32,
    ) -> LayoutSize {
        self.measure_stretch_width_element(element, available_width)
            .size
    }

    pub(super) fn measure_stretch_width_element(
        self,
        element: UiElement,
        available_width: f32,
    ) -> MeasuredUiElement {
        self.measure_element(element, available_width)
    }

    pub(super) fn measure_fixed_width_element(
        self,
        element: UiElement,
        width: f32,
    ) -> MeasuredUiElement {
        self.measure_element(element.width(LayoutLength::Pixel(width.max(0.0))), width)
    }

    pub(super) fn measure_spacer(self, width: f32) -> LayoutSize {
        self.measure_auto_width_ui_element(UiElement::spacer(width, self))
    }

    pub(super) fn measure_stretch_width_child(
        self,
        child: UiElement,
        available_width: f32,
    ) -> LayoutSize {
        self.measure_stretch_width_ui_element(child, available_width)
    }

    pub(super) fn fill_width_child_size(self, width: f32) -> LayoutSize {
        self.measure_stretch_width_ui_element(
            UiElement::stretch_width_stretch_height(LayoutSize::new(width, self.height)),
            width,
        )
    }

    pub(super) fn intrinsic_width_fill_height_child_size(self, intrinsic_width: f32) -> LayoutSize {
        self.measure_auto_width_ui_element(UiElement::auto_width_stretch_height(LayoutSize::new(
            intrinsic_width,
            self.height,
        )))
    }

    pub(super) fn square_fill_height_child_size(self) -> LayoutSize {
        self.measure_auto_width_ui_element(UiElement::icon_button_square(self))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SharedSizeGroupContract {
    pub(super) width: f32,
    pub(super) height: f32,
}

impl SharedSizeGroupContract {
    pub(super) fn from_intrinsic_sizes(sizes: &[LayoutSize]) -> Self {
        let size = sizes
            .iter()
            .copied()
            .fold(LayoutSize::ZERO, |current, next| current.max(next));
        Self {
            width: size.width,
            height: size.height,
        }
    }

    pub(super) fn equal_width_fill_height_child_size(
        self,
        row: SingleLineControlRowContract,
    ) -> LayoutSize {
        row.measure_auto_width_ui_element(UiElement::auto_width_stretch_height(LayoutSize::new(
            self.width,
            self.height.max(row.height),
        )))
    }

    pub(super) fn equal_width_button_size_for_row(
        self,
        row: SingleLineControlRowContract,
    ) -> LayoutSize {
        row.measure_auto_width_ui_element(UiElement::button(LayoutSize::new(
            self.width,
            self.height.max(row.height),
        )))
    }
}

fn resolve_axis_size(
    length: LayoutLength,
    stretch_when_auto: bool,
    parent_size: f32,
    intrinsic_size: f32,
    min_size: f32,
    max_size: f32,
) -> f32 {
    let raw = match length {
        LayoutLength::Auto if stretch_when_auto => parent_size,
        LayoutLength::Auto => intrinsic_size,
        LayoutLength::Pixel(value) => value,
        LayoutLength::Star(_) => parent_size,
    };

    raw.clamp(min_size.max(0.0), max_size.max(min_size).max(0.0))
}
