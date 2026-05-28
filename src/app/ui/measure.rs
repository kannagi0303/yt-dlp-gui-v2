use eframe::egui::{self, FontId, TextStyle, TextWrapMode, Ui, WidgetText};

#[derive(Clone, Copy, Debug)]
pub(super) struct WidthRange {
    pub min: f32,
    pub max: f32,
}

impl WidthRange {
    pub(super) fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub(super) fn clamp(self, width: f32) -> f32 {
        width.clamp(self.min, self.max)
    }
}

pub(super) fn text_width(ui: &Ui, text: &str, text_style: TextStyle) -> f32 {
    WidgetText::from(text)
        .into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, text_style)
        .size()
        .x
}

pub(super) fn max_text_width<'a>(
    ui: &Ui,
    texts: impl IntoIterator<Item = &'a str>,
    text_style: TextStyle,
) -> f32 {
    texts
        .into_iter()
        .map(|text| text_width(ui, text, text_style.clone()))
        .fold(0.0, f32::max)
}

pub(super) fn measured_text_width<'a>(
    ui: &Ui,
    texts: impl IntoIterator<Item = &'a str>,
    text_style: TextStyle,
    extra_width: f32,
    width_range: WidthRange,
) -> f32 {
    width_range.clamp(max_text_width(ui, texts, text_style) + extra_width)
}

pub(super) fn measured_column_width<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
    text_style: TextStyle,
    extra_width: f32,
    width_range: WidthRange,
) -> f32 {
    let header_width = text_width(ui, header, text_style.clone());
    width_range.clamp(header_width.max(max_text_width(ui, values, text_style)) + extra_width)
}

pub(super) fn wrapped_text_height(
    ui: &Ui,
    text: &str,
    max_width: f32,
    font_id: FontId,
    text_style: TextStyle,
    max_rows: Option<usize>,
    break_anywhere: bool,
    overflow_character: Option<char>,
) -> f32 {
    let width = max_width.max(0.0);
    let mut job =
        egui::text::LayoutJob::simple(text.to_owned(), font_id, ui.visuals().text_color(), width);
    if let Some(max_rows) = max_rows {
        job.wrap.max_rows = max_rows;
    }
    job.wrap.break_anywhere = break_anywhere;
    job.wrap.overflow_character = overflow_character;

    WidgetText::from(job)
        .into_galley(ui, Some(TextWrapMode::Wrap), width, text_style)
        .size()
        .y
}

pub(super) fn text_line_height(ui: &Ui, font_id: FontId, text_style: TextStyle) -> f32 {
    let job = egui::text::LayoutJob::simple(
        "Hg".to_owned(),
        font_id,
        ui.visuals().text_color(),
        f32::INFINITY,
    );
    WidgetText::from(job)
        .into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, text_style)
        .size()
        .y
}

pub(super) fn max_text_height_for_lines(
    ui: &Ui,
    font_id: FontId,
    text_style: TextStyle,
    max_lines: usize,
    extra_height: f32,
) -> f32 {
    text_line_height(ui, font_id, text_style) * max_lines as f32 + extra_height
}
