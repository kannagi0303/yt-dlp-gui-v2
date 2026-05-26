use eframe::egui::{
    Color32, FontId, Response, TextBuffer, TextEdit, TextStyle, Ui, Widget,
    epaint::text::cursor::CCursor,
    text::{LayoutJob, TextFormat},
};

use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};
use crate::i18n::{self, Language};
use crate::infrastructure::is_windows_known_folder_segment;

pub struct UrlInput<'a> {
    text: &'a mut String,
    hint_text: &'a str,
    enabled: bool,
    language: Language,
}

pub struct DisplayPathInput<'a> {
    text: &'a mut String,
    error: bool,
}

impl<'a> UrlInput<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint_text: "",
            enabled: true,
            language: Language::ZhTw,
        }
    }

    pub fn hint_text(mut self, hint_text: &'a str) -> Self {
        self.hint_text = hint_text;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    pub fn ui(self, ui: &mut Ui) -> eframe::egui::Response {
        ui.add(self)
    }
}

impl<'a> DisplayPathInput<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self { text, error: false }
    }

    pub fn error(mut self, enabled: bool) -> Self {
        self.error = enabled;
        self
    }
}

impl Widget for UrlInput<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            text,
            hint_text,
            enabled,
            language,
        } = self;

        let mut editor = TextEdit::singleline(text).hint_text(hint_text);

        if !enabled {
            editor = editor.interactive(false);
        }

        let mut layouter = move |ui: &Ui, text: &dyn TextBuffer, wrap_width: f32| {
            let mut job = layout_url_text(ui, text.as_str());
            let _ = wrap_width;
            job.wrap.max_width = f32::INFINITY;
            job.wrap.max_rows = 1;
            job.wrap.break_anywhere = false;
            job.wrap.overflow_character = None;
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };

        let output = editor.layouter(&mut layouter).show(ui);
        let response = output.response.response.clone();
        attach_text_edit_context_menu(ui, &response, text, enabled, language, &output);
        response
    }
}

impl Widget for DisplayPathInput<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { text, error } = self;
        let editor = TextEdit::singleline(text).interactive(false);

        let mut layouter = move |ui: &Ui, text: &dyn TextBuffer, wrap_width: f32| {
            let mut job = layout_path_text(ui, text.as_str());
            if error {
                apply_error_color(ui, &mut job);
            }
            let _ = wrap_width;
            job.wrap.max_width = f32::INFINITY;
            job.wrap.max_rows = 1;
            job.wrap.break_anywhere = false;
            job.wrap.overflow_character = None;
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };

        editor.layouter(&mut layouter).ui(ui)
    }
}

fn layout_url_text(ui: &Ui, text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let theme = url_theme(ui);

    if text.is_empty() {
        return job;
    }

    let segments = collect_segments(text);
    if segments.is_empty() {
        append_text(&mut job, text, theme.normal);
    } else {
        let mut cursor = 0;
        for segment in segments {
            if cursor < segment.start {
                append_text(&mut job, &text[cursor..segment.start], theme.normal);
            }
            append_text(
                &mut job,
                &text[segment.start..segment.end],
                theme.color(segment.kind),
            );
            cursor = segment.end;
        }
        if cursor < text.len() {
            append_text(&mut job, &text[cursor..], theme.normal);
        }
    }

    let font_id = FontId::new(
        ui.style().text_styles[&TextStyle::Body].size,
        eframe::egui::FontFamily::Proportional,
    );
    for section in &mut job.sections {
        section.format.font_id = font_id.clone();
    }

    job
}

fn layout_path_text(ui: &Ui, text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let theme = accent_theme(ui);

    if text.is_empty() {
        return job;
    }

    let mut segment_start = 0;
    for (index, ch) in text.char_indices() {
        if matches!(ch, '\\' | '/') {
            if segment_start < index {
                let segment = &text[segment_start..index];
                append_text(
                    &mut job,
                    segment,
                    if is_special_path_segment(segment) {
                        theme.domain
                    } else {
                        theme.normal
                    },
                );
            }
            append_text(&mut job, &text[index..index + ch.len_utf8()], theme.slash);
            segment_start = index + ch.len_utf8();
        }
    }

    if segment_start < text.len() {
        let segment = &text[segment_start..];
        append_text(
            &mut job,
            segment,
            if is_special_path_segment(segment) {
                theme.domain
            } else {
                theme.normal
            },
        );
    }

    let font_id = FontId::new(
        ui.style().text_styles[&TextStyle::Body].size,
        eframe::egui::FontFamily::Proportional,
    );
    for section in &mut job.sections {
        section.format.font_id = font_id.clone();
    }

    job
}

fn apply_error_color(ui: &Ui, job: &mut LayoutJob) {
    for section in &mut job.sections {
        section.format.color = accent_red_for_ui(ui);
    }
}

fn append_text(job: &mut LayoutJob, text: &str, color: Color32) {
    if text.is_empty() {
        return;
    }

    job.append(
        text,
        0.0,
        TextFormat {
            color,
            ..Default::default()
        },
    );
}

fn attach_text_edit_context_menu(
    _ui: &mut Ui,
    response: &Response,
    text: &mut String,
    enabled: bool,
    language: Language,
    output: &eframe::egui::text_edit::TextEditOutput,
) {
    let selected_range = output.cursor_range.filter(|range| !range.is_empty());
    let has_selection = selected_range.is_some();
    let has_text = !text.is_empty();

    response.context_menu(|ui| {
        let copied = selected_range.map(|range| range.slice_str(text.as_str()).to_owned());
        let can_edit = enabled;
        let can_paste = can_edit && clipboard_text().is_some_and(|value| !value.is_empty());

        if menu_action_button(
            ui,
            AppIcon::ContentCut,
            i18n::text(language, "action.cut"),
            can_edit && has_selection,
        )
        .clicked()
        {
            if let Some(range) = selected_range {
                ui.ctx()
                    .copy_text(range.slice_str(text.as_str()).to_owned());
                let cursor = text.delete_selected(&range);
                store_text_cursor(
                    output,
                    response,
                    eframe::egui::text::CCursorRange::one(cursor),
                );
                ui.close();
            }
        }

        if menu_action_button(
            ui,
            AppIcon::ContentCopy,
            i18n::text(language, "action.copy"),
            has_selection,
        )
        .clicked()
        {
            if let Some(copied) = copied {
                ui.ctx().copy_text(copied);
                ui.close();
            }
        }

        if menu_action_button(
            ui,
            AppIcon::ContentPaste,
            i18n::text(language, "action.paste"),
            can_paste,
        )
        .clicked()
        {
            if let Some(paste_text) = clipboard_text() {
                insert_text_at_cursor(text, output, response, &paste_text, false);
                ui.close();
            }
        }

        if menu_action_button(
            ui,
            AppIcon::Eraser,
            i18n::text(language, "action.clear"),
            can_edit && has_text,
        )
        .clicked()
        {
            text.clear();
            let end = eframe::egui::text::CCursorRange::one(CCursor::new(0));
            store_text_cursor(output, response, end);
            ui.close();
        }
    });
}

fn menu_action_button(ui: &mut Ui, icon: AppIcon, label: &str, enabled: bool) -> Response {
    let size = ui.spacing().interact_size.y * 0.72;
    ui.add_enabled(
        enabled,
        eframe::egui::Button::image_and_text(
            icon_image(icon, size, standard_icon_color(ui)),
            label,
        ),
    )
}

fn insert_text_at_cursor(
    text: &mut String,
    output: &eframe::egui::text_edit::TextEditOutput,
    response: &Response,
    paste_text: &str,
    multiline: bool,
) {
    if paste_text.is_empty() {
        return;
    }

    let mut cursor = output.cursor_range.unwrap_or_else(|| {
        eframe::egui::text::CCursorRange::one(CCursor::new(text.chars().count()))
    });
    let mut ccursor = text.delete_selected(&cursor);
    let text_to_insert = if multiline {
        paste_text.to_owned()
    } else {
        paste_text.replace(['\r', '\n'], " ")
    };
    text.insert_text_at(&mut ccursor, &text_to_insert, usize::MAX);
    cursor = eframe::egui::text::CCursorRange::one(ccursor);
    store_text_cursor(output, response, cursor);
}

fn store_text_cursor(
    output: &eframe::egui::text_edit::TextEditOutput,
    response: &Response,
    cursor_range: eframe::egui::text::CCursorRange,
) {
    let mut state = output.state.clone();
    state.cursor.set_char_range(Some(cursor_range));
    state.store(&response.ctx, response.id);
}

fn clipboard_text() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    clipboard.get_text().ok()
}

pub fn accent_blue() -> Color32 {
    Color32::from_rgb(46, 171, 254)
}

pub fn accent_green() -> Color32 {
    Color32::from_rgb(154, 205, 50)
}

pub fn accent_red() -> Color32 {
    Color32::from_rgb(196, 64, 64)
}

pub fn accent_blue_for_ui(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        accent_blue()
    } else {
        Color32::from_rgb(0, 92, 230)
    }
}

pub fn accent_green_for_ui(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        accent_green()
    } else {
        Color32::from_rgb(0, 128, 32)
    }
}

pub fn accent_red_for_ui(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        accent_red()
    } else {
        Color32::from_rgb(210, 0, 32)
    }
}

#[derive(Clone, Copy)]
struct UrlTheme {
    normal: Color32,
    slash: Color32,
    file_name: Color32,
    domain: Color32,
}

fn url_theme(ui: &Ui) -> UrlTheme {
    accent_theme(ui)
}

fn accent_theme(ui: &Ui) -> UrlTheme {
    UrlTheme {
        normal: ui.visuals().text_color(),
        slash: accent_blue_for_ui(ui),
        file_name: neutral_secondary_text_color(ui),
        domain: accent_green_for_ui(ui),
    }
}

fn neutral_secondary_text_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        ui.visuals().weak_text_color()
    } else {
        Color32::from_rgb(82, 88, 96)
    }
}

impl UrlTheme {
    fn color(self, kind: SegmentKind) -> Color32 {
        match kind {
            SegmentKind::Slash => self.slash,
            SegmentKind::FileName => self.file_name,
            SegmentKind::Domain => self.domain,
        }
    }
}

#[derive(Clone, Copy)]
enum SegmentKind {
    Slash,
    FileName,
    Domain,
}

#[derive(Clone, Copy)]
struct Segment {
    start: usize,
    end: usize,
    kind: SegmentKind,
}

#[derive(Clone, Copy)]
struct ParsedUrl {
    scheme_end: usize,
    domain_start: usize,
    domain_end: usize,
    span_end: usize,
}

fn collect_segments(text: &str) -> Vec<Segment> {
    let Some(parsed) = parse_outer_span(text) else {
        return Vec::new();
    };

    let mut segments = Vec::new();

    if parsed.scheme_end > 0 {
        segments.push(Segment {
            start: 0,
            end: parsed.scheme_end,
            kind: SegmentKind::Slash,
        });
    }

    segments.push(Segment {
        start: parsed.scheme_end,
        end: parsed.domain_start,
        kind: SegmentKind::Slash,
    });

    segments.push(Segment {
        start: parsed.domain_start,
        end: parsed.domain_end,
        kind: SegmentKind::Domain,
    });

    let mut index = parsed.domain_end;
    while index < parsed.span_end {
        let next = next_char_boundary(text, index);
        let ch = &text[index..next];

        if matches!(ch, "/" | "?" | "=" | "&") {
            segments.push(Segment {
                start: index,
                end: next,
                kind: SegmentKind::Slash,
            });

            if ch == "/" {
                if let Some(file_end) = match_file_name_end(text, next, parsed.span_end) {
                    segments.push(Segment {
                        start: next,
                        end: file_end,
                        kind: SegmentKind::FileName,
                    });
                    index = file_end;
                    continue;
                }
            }
        }

        index = next;
    }

    segments
}

fn parse_outer_span(text: &str) -> Option<ParsedUrl> {
    let scheme_end = scan_scheme(text)?;
    let domain_start = scheme_end.checked_add(3)?;
    if !text[scheme_end..].starts_with("://") {
        return None;
    }

    let domain_end = scan_domain(text, domain_start)?;
    let span_end = scan_outer_span_end(text, domain_end);

    Some(ParsedUrl {
        scheme_end,
        domain_start,
        domain_end,
        span_end,
    })
}

fn scan_scheme(text: &str) -> Option<usize> {
    let mut end = 0;
    for (idx, ch) in text.char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    if end > 0 && text[end..].starts_with("://") {
        Some(end)
    } else {
        None
    }
}

fn scan_domain(text: &str, start: usize) -> Option<usize> {
    let mut index = start;
    let mut label_count = 0;

    loop {
        let label_end = scan_host_label(text, index)?;
        label_count += 1;
        index = label_end;

        if text.get(index..index + 1) == Some(".") {
            index += 1;
            continue;
        }

        break;
    }

    if label_count >= 2 { Some(index) } else { None }
}

fn scan_host_label(text: &str, start: usize) -> Option<usize> {
    let mut end = start;
    for (offset, ch) in text[start..].char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            end = start + offset + ch.len_utf8();
        } else {
            break;
        }
    }

    if end > start { Some(end) } else { None }
}

fn scan_outer_span_end(text: &str, start: usize) -> usize {
    let mut end = start;
    for (offset, ch) in text[start..].char_indices() {
        if !is_outer_span_char(ch) {
            break;
        }
        end = start + offset + ch.len_utf8();
    }
    end
}

fn is_outer_span_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            '_' | '.' | ',' | '@' | '?' | '^' | '=' | '%' | '&' | ':' | '/' | '~' | '+' | '#' | '-'
        )
}

fn match_file_name_end(text: &str, start: usize, span_end: usize) -> Option<usize> {
    if start >= span_end {
        return None;
    }

    let mut end = start;
    for (offset, ch) in text[start..span_end].char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            end = start + offset + ch.len_utf8();
        } else {
            break;
        }
    }

    if end == start {
        return None;
    }

    match text.get(end..end + 1) {
        Some("?") | Some("#") => Some(end),
        None if end == span_end => Some(end),
        _ => None,
    }
}

fn next_char_boundary(text: &str, index: usize) -> usize {
    text[index..]
        .chars()
        .next()
        .map(|ch| index + ch.len_utf8())
        .unwrap_or(text.len())
}

fn is_special_path_segment(segment: &str) -> bool {
    is_windows_known_folder_segment(segment)
}
