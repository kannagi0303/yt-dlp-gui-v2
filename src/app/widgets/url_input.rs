use eframe::egui::{
    self, Color32, FontId, Response, ScrollArea, Stroke, TextBuffer, TextEdit, TextStyle, Ui,
    Widget,
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

pub struct UrlSyntaxInput<'a> {
    text: &'a mut String,
    hint_text: &'a str,
    enabled: bool,
    language: Language,
    desired_width: Option<f32>,
    min_rows: usize,
    max_rows: Option<usize>,
    allow_newline: bool,
    ctrl_click_links: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppTextBoxSyntax {
    Plain,
    Url,
    Path,
    Description,
}

pub struct AppTextBox<'a> {
    text: &'a mut String,
    hint_text: &'a str,
    enabled: bool,
    editable: bool,
    selectable: bool,
    language: Language,
    syntax: AppTextBoxSyntax,
    error: bool,
    desired_width: Option<f32>,
    desired_height: Option<f32>,
    min_rows: usize,
    max_rows: Option<usize>,
    allow_newline: bool,
    ctrl_click_links: bool,
    tag_link_base_url: Option<String>,
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

impl<'a> UrlSyntaxInput<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint_text: "",
            enabled: true,
            language: Language::ZhTw,
            desired_width: None,
            min_rows: 1,
            max_rows: None,
            allow_newline: false,
            ctrl_click_links: true,
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

    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    pub fn min_rows(mut self, rows: usize) -> Self {
        self.min_rows = rows.max(1);
        self
    }

    pub fn max_rows(mut self, rows: Option<usize>) -> Self {
        self.max_rows = rows.map(|rows| rows.max(1));
        self
    }

    pub fn allow_newline(mut self, enabled: bool) -> Self {
        self.allow_newline = enabled;
        self
    }

    pub fn ctrl_click_links(mut self, enabled: bool) -> Self {
        self.ctrl_click_links = enabled;
        self
    }

    pub fn ui(self, ui: &mut Ui) -> eframe::egui::Response {
        ui.add(self)
    }
}

impl<'a> AppTextBox<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint_text: "",
            enabled: true,
            editable: true,
            selectable: true,
            language: Language::ZhTw,
            syntax: AppTextBoxSyntax::Plain,
            error: false,
            desired_width: None,
            desired_height: None,
            min_rows: 1,
            max_rows: Some(1),
            allow_newline: false,
            ctrl_click_links: false,
            tag_link_base_url: None,
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

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    pub fn syntax(mut self, syntax: AppTextBoxSyntax) -> Self {
        self.syntax = syntax;
        self
    }

    pub fn error(mut self, enabled: bool) -> Self {
        self.error = enabled;
        self
    }

    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    pub fn desired_height(mut self, height: f32) -> Self {
        self.desired_height = Some(height);
        self
    }

    pub fn min_rows(mut self, rows: usize) -> Self {
        self.min_rows = rows.max(1);
        self
    }

    pub fn max_rows(mut self, rows: Option<usize>) -> Self {
        self.max_rows = rows.map(|rows| rows.max(1));
        self
    }

    pub fn allow_newline(mut self, enabled: bool) -> Self {
        self.allow_newline = enabled;
        self
    }

    pub fn ctrl_click_links(mut self, enabled: bool) -> Self {
        self.ctrl_click_links = enabled;
        self
    }

    pub fn tag_link_base_url(mut self, base_url: Option<String>) -> Self {
        self.tag_link_base_url = base_url;
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

        let mut editor = TextEdit::singleline(text)
            .hint_text(hint_text)
            .frame(text_field_frame(ui))
            .background_color(text_field_bg_color(ui));

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

impl Widget for UrlSyntaxInput<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            text,
            hint_text,
            enabled,
            language,
            desired_width,
            min_rows,
            max_rows,
            allow_newline,
            ctrl_click_links,
        } = self;

        let mut textbox = AppTextBox::new(text)
            .hint_text(hint_text)
            .enabled(enabled)
            .editable(true)
            .selectable(true)
            .language(language)
            .syntax(AppTextBoxSyntax::Url)
            .min_rows(min_rows)
            .max_rows(max_rows)
            .allow_newline(allow_newline)
            .ctrl_click_links(ctrl_click_links);
        if let Some(width) = desired_width {
            textbox = textbox.desired_width(width);
        }
        textbox.ui(ui)
    }
}

impl Widget for AppTextBox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            text,
            hint_text,
            enabled,
            editable,
            selectable,
            language,
            syntax,
            error,
            desired_width,
            desired_height,
            min_rows,
            max_rows,
            allow_newline,
            ctrl_click_links,
            tag_link_base_url,
        } = self;

        let frame = app_textbox_frame(ui);
        let frame_margin = frame.total_margin().sum();
        let desired_width = desired_width
            .unwrap_or_else(|| ui.available_width())
            .min(ui.available_width())
            .max(frame_margin.x + 1.0);
        let inner_width = (desired_width - frame_margin.x).max(1.0);
        let content_rows =
            app_textbox_content_rows(ui, text.as_str(), desired_width, min_rows, syntax);
        let line_height = ui.text_style_height(&TextStyle::Body).max(1.0);
        let desired_inner_height =
            desired_height.map(|height| (height - frame_margin.y).max(line_height));
        let visible_rows = max_rows.map_or(content_rows, |max_rows| {
            content_rows.min(max_rows.max(1)).max(min_rows.max(1))
        });
        let visible_inner_height = desired_inner_height
            .unwrap_or_else(|| app_textbox_inner_height_for_rows(ui, visible_rows));
        let content_inner_height = content_rows.max(1) as f32 * line_height;
        let needs_inner_scroll = if desired_inner_height.is_some() {
            content_inner_height > visible_inner_height + 0.5
        } else {
            max_rows.is_some_and(|max_rows| {
                let max_rows = max_rows.max(1);
                max_rows > 1 && content_rows > max_rows
            })
        };

        let mut readonly_text;
        let editor_text = if editable {
            text
        } else {
            readonly_text = text.clone();
            &mut readonly_text
        };

        let ctrl_link_visual = ctrl_click_links && ui.input(|input| input.modifiers.ctrl);
        let tag_link_visual = ctrl_link_visual && tag_link_base_url.is_some();
        let output = frame
            .show(ui, |ui| {
                ui.set_min_width(inner_width);
                ui.set_max_width(inner_width);
                if let Some(height) = desired_inner_height {
                    ui.set_min_height(height);
                }

                let mut show_editor =
                    |ui: &mut Ui,
                     rows: usize,
                     wrap_max_rows: Option<usize>,
                     min_height: Option<f32>| {
                        let mut editor = TextEdit::multiline(&mut *editor_text)
                            .desired_width(inner_width)
                            .desired_rows(rows)
                            .hint_text(hint_text)
                            .frame(egui::Frame::NONE)
                            .margin(egui::Margin::same(0))
                            .background_color(Color32::TRANSPARENT);

                        if let Some(min_height) = min_height {
                            editor = editor.min_size(egui::vec2(inner_width, min_height));
                        }

                        if !enabled || (!editable && !selectable) {
                            editor = editor.interactive(false);
                        }

                        let mut layouter =
                            move |ui: &Ui, text: &dyn TextBuffer, wrap_width: f32| {
                                let mut job = layout_app_textbox_text(
                                    ui,
                                    text.as_str(),
                                    syntax,
                                    ctrl_link_visual,
                                    tag_link_visual,
                                );
                                if error {
                                    apply_error_color(ui, &mut job);
                                }
                                job.wrap.max_width = wrap_width;
                                job.wrap.max_rows = wrap_max_rows.unwrap_or(usize::MAX);
                                job.wrap.break_anywhere = true;
                                job.wrap.overflow_character = wrap_max_rows.map(|_| '…');
                                ui.fonts_mut(|fonts| fonts.layout_job(job))
                            };

                        editor.layouter(&mut layouter).show(ui)
                    };

                if needs_inner_scroll {
                    ScrollArea::vertical()
                        .id_salt(("app-textbox-scroll", hint_text, syntax))
                        .max_height(visible_inner_height)
                        .min_scrolled_height(visible_inner_height)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(inner_width);
                            show_editor(ui, content_rows, None, None)
                        })
                        .inner
                } else {
                    show_editor(ui, visible_rows, max_rows, desired_inner_height)
                }
            })
            .inner;
        let response = output.response.response.clone();
        let enter_pressed =
            response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

        if editable && !allow_newline && output.response.changed() && contains_newline(editor_text)
        {
            editor_text.retain(|ch| ch != '\r' && ch != '\n');
        }

        if ctrl_click_links
            && response.clicked()
            && ui.input(|input| input.modifiers.ctrl)
            && let Some(link) = output.cursor_range.and_then(|range| {
                app_textbox_link_at_cursor(editor_text.as_str(), range.primary.index.into(), syntax)
            })
        {
            match link {
                AppTextBoxLink::Url(url) => ui.ctx().open_url(egui::OpenUrl::new_tab(url)),
                AppTextBoxLink::Tag(tag) => {
                    if let Some(url) = tag_link_base_url
                        .as_deref()
                        .and_then(|base_url| hashtag_url(base_url, &tag))
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                    }
                }
            }
        }

        if enter_pressed && !allow_newline {
            // Keep Enter observable to callers through the normal egui input
            // state, but never leave a newline in this textbox mode.
        }

        attach_text_edit_context_menu(
            ui,
            &response,
            editor_text,
            editable && enabled,
            language,
            &output,
        );
        response
    }
}

impl Widget for DisplayPathInput<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { text, error } = self;
        let editor = TextEdit::singleline(text)
            .interactive(false)
            .frame(text_field_frame(ui))
            .background_color(text_field_bg_color(ui));

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

fn layout_app_textbox_text(
    ui: &Ui,
    text: &str,
    syntax: AppTextBoxSyntax,
    link_visual: bool,
    tag_link_visual: bool,
) -> LayoutJob {
    match syntax {
        AppTextBoxSyntax::Plain => layout_plain_text(ui, text),
        AppTextBoxSyntax::Url => layout_url_text_with_link_visual(ui, text, link_visual),
        AppTextBoxSyntax::Path => layout_path_text(ui, text),
        AppTextBoxSyntax::Description => {
            layout_description_text(ui, text, link_visual, tag_link_visual)
        }
    }
}

fn layout_plain_text(ui: &Ui, text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    if text.is_empty() {
        return job;
    }

    append_text(&mut job, text, text_field_text_color(ui));
    apply_body_font(ui, &mut job);
    job
}

fn layout_url_text(ui: &Ui, text: &str) -> LayoutJob {
    layout_url_text_with_link_visual(ui, text, false)
}

fn layout_url_text_with_link_visual(ui: &Ui, text: &str, link_visual: bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    let theme = url_theme(ui);

    if text.is_empty() {
        return job;
    }

    let segments = collect_segments(text);
    let underline = link_visual && parse_outer_span(text).is_some();
    if !segments.is_empty() {
        let mut cursor = 0;
        for segment in segments {
            if cursor < segment.start {
                append_text_with_options(
                    &mut job,
                    &text[cursor..segment.start],
                    theme.normal,
                    underline,
                );
            }
            append_text_with_options(
                &mut job,
                &text[segment.start..segment.end],
                theme.color(segment.kind),
                underline,
            );
            cursor = segment.end;
        }
        if cursor < text.len() {
            append_text_with_options(&mut job, &text[cursor..], theme.normal, underline);
        }
    } else {
        append_text_with_tag_tokens(&mut job, text, theme.normal, theme.domain, link_visual);
    }

    apply_body_font(ui, &mut job);
    job
}

fn layout_description_text(
    ui: &Ui,
    text: &str,
    url_link_visual: bool,
    tag_link_visual: bool,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    if text.is_empty() {
        return job;
    }

    let theme = url_theme(ui);
    let mut cursor = 0usize;
    for token in collect_description_tokens(text) {
        if cursor < token.start {
            append_text_with_options(&mut job, &text[cursor..token.start], theme.normal, false);
        }
        let color = match token.kind {
            AppTextBoxTokenKind::Url => accent_blue_for_ui(ui),
            AppTextBoxTokenKind::Tag => theme.domain,
        };
        let underline = match token.kind {
            AppTextBoxTokenKind::Url => url_link_visual,
            AppTextBoxTokenKind::Tag => tag_link_visual,
        };
        append_text_with_options(&mut job, &text[token.start..token.end], color, underline);
        cursor = token.end;
    }
    if cursor < text.len() {
        append_text_with_options(&mut job, &text[cursor..], theme.normal, false);
    }

    apply_body_font(ui, &mut job);
    job
}

fn apply_body_font(ui: &Ui, job: &mut LayoutJob) {
    let font_id = FontId::new(
        ui.style().text_styles[&TextStyle::Body].size,
        eframe::egui::FontFamily::Proportional,
    );
    for section in &mut job.sections {
        section.format.font_id = font_id.clone();
    }
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

    apply_body_font(ui, &mut job);
    job
}

fn apply_error_color(ui: &Ui, job: &mut LayoutJob) {
    for section in &mut job.sections {
        section.format.color = accent_red_for_ui(ui);
    }
}

fn append_text_with_tag_tokens(
    job: &mut LayoutJob,
    text: &str,
    normal_color: Color32,
    tag_color: Color32,
    link_visual: bool,
) {
    let mut cursor = 0;
    let mut found = false;
    for (start, end) in collect_tag_segments(text) {
        found = true;
        if cursor < start {
            append_text_with_options(job, &text[cursor..start], normal_color, false);
        }
        append_text_with_options(job, &text[start..end], tag_color, link_visual);
        cursor = end;
    }
    if cursor < text.len() {
        append_text_with_options(job, &text[cursor..], normal_color, false);
    }
    if !found && text.is_empty() {
        append_text_with_options(job, text, normal_color, false);
    }
}

fn append_text(job: &mut LayoutJob, text: &str, color: Color32) {
    append_text_with_options(job, text, color, false);
}

fn append_text_with_options(job: &mut LayoutJob, text: &str, color: Color32, underline: bool) {
    if text.is_empty() {
        return;
    }

    job.append(
        text,
        0.0,
        TextFormat {
            color,
            underline: if underline {
                Stroke::new(1.0, color)
            } else {
                Stroke::NONE
            },
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

fn hashtag_url(base_url: &str, tag: &str) -> Option<String> {
    let base = base_url.trim();
    if base.is_empty() {
        return None;
    }
    let tag = tag
        .trim()
        .trim_start_matches('#')
        .trim_start_matches('＃')
        .trim();
    if tag.is_empty() || tag.chars().any(char::is_whitespace) {
        return None;
    }
    Some(format!("{}{}", base, percent_encode_path_segment(tag)))
}

fn percent_encode_path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(*byte as char);
            }
            _ => {
                use std::fmt::Write as _;
                let _ = write!(&mut encoded, "%{:02X}", byte);
            }
        }
    }
    encoded
}

fn contains_newline(text: &str) -> bool {
    text.contains('\r') || text.contains('\n')
}

const APP_TEXTBOX_MARGIN_X: f32 = 6.0;
const APP_TEXTBOX_MARGIN_Y: f32 = 4.0;

pub fn text_field_bg_color(ui: &Ui) -> Color32 {
    ui.visuals().extreme_bg_color.linear_multiply(0.55)
}

pub fn text_field_text_color(ui: &Ui) -> Color32 {
    ui.visuals().widgets.inactive.fg_stroke.color
}

pub fn text_field_frame(ui: &Ui) -> egui::Frame {
    egui::Frame::NONE
        .fill(text_field_bg_color(ui))
        .stroke(Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .corner_radius(2.0)
        .inner_margin(egui::Margin::symmetric(4, 2))
}

pub fn url_syntax_textbox_single_line_height(ui: &Ui) -> f32 {
    app_textbox_single_line_height(ui)
}

pub fn url_syntax_textbox_height(ui: &Ui, text: &str, width: f32) -> f32 {
    app_textbox_height(ui, text, width, 1, None, AppTextBoxSyntax::Url)
}

pub fn app_textbox_single_line_height(ui: &Ui) -> f32 {
    app_textbox_height_for_rows(ui, 1)
}

pub fn app_textbox_height(
    ui: &Ui,
    text: &str,
    width: f32,
    min_rows: usize,
    max_rows: Option<usize>,
    syntax: AppTextBoxSyntax,
) -> f32 {
    let rows = app_textbox_visible_rows(ui, text, width, min_rows, max_rows, syntax);
    app_textbox_single_line_height(ui)
        + (rows.saturating_sub(1) as f32 * ui.text_style_height(&TextStyle::Body))
}

fn app_textbox_visible_rows(
    ui: &Ui,
    text: &str,
    width: f32,
    min_rows: usize,
    max_rows: Option<usize>,
    syntax: AppTextBoxSyntax,
) -> usize {
    let content_rows = app_textbox_content_rows(ui, text, width, min_rows, syntax);
    max_rows.map_or(content_rows, |max_rows| {
        content_rows.min(max_rows.max(1)).max(min_rows.max(1))
    })
}

fn app_textbox_content_rows(
    ui: &Ui,
    text: &str,
    width: f32,
    min_rows: usize,
    syntax: AppTextBoxSyntax,
) -> usize {
    let min_rows = min_rows.max(1);
    if text.trim().is_empty() || width <= 0.0 {
        return min_rows;
    }

    let content_width = app_textbox_inner_width(ui, width).max(1.0);
    let mut job = layout_app_textbox_text(ui, text, syntax, false, false);
    job.wrap.max_width = content_width;
    job.wrap.max_rows = usize::MAX;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = None;

    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let line_height = ui.text_style_height(&TextStyle::Body).max(1.0);
    ((galley.size().y / line_height).ceil() as usize).max(min_rows)
}

fn app_textbox_height_for_rows(ui: &Ui, rows: usize) -> f32 {
    app_textbox_inner_height_for_rows(ui, rows) + app_textbox_frame(ui).total_margin().sum().y
}

fn app_textbox_inner_height_for_rows(ui: &Ui, rows: usize) -> f32 {
    rows.max(1) as f32 * ui.text_style_height(&TextStyle::Body)
}

pub fn url_syntax_textbox_frame(ui: &Ui) -> egui::Frame {
    app_textbox_frame(ui)
}

pub fn app_textbox_frame(ui: &Ui) -> egui::Frame {
    egui::Frame::NONE
        .fill(text_field_bg_color(ui))
        .stroke(Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .corner_radius(2.0)
        .inner_margin(egui::Margin::symmetric(APP_TEXTBOX_MARGIN_X as i8, 4))
}

fn app_textbox_inner_width(ui: &Ui, width: f32) -> f32 {
    (width - app_textbox_frame(ui).total_margin().sum().x).max(1.0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppTextBoxLink {
    Url(String),
    Tag(String),
}

fn app_textbox_link_at_cursor(
    text: &str,
    char_index: usize,
    syntax: AppTextBoxSyntax,
) -> Option<AppTextBoxLink> {
    let byte_index = char_to_byte_index(text, char_index);
    match syntax {
        AppTextBoxSyntax::Plain | AppTextBoxSyntax::Path => None,
        AppTextBoxSyntax::Url => {
            if let Some(parsed) = parse_outer_span(text) {
                if byte_index <= parsed.span_end {
                    return Some(AppTextBoxLink::Url(text[..parsed.span_end].to_owned()));
                }
            }
            tag_at_byte_index(text, byte_index).map(AppTextBoxLink::Tag)
        }
        AppTextBoxSyntax::Description => link_token_at_byte_index(text, byte_index),
    }
}

fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

#[derive(Clone, Copy)]
enum AppTextBoxTokenKind {
    Url,
    Tag,
}

#[derive(Clone, Copy)]
struct AppTextBoxToken {
    start: usize,
    end: usize,
    kind: AppTextBoxTokenKind,
}

fn collect_description_tokens(text: &str) -> Vec<AppTextBoxToken> {
    let mut tokens = Vec::new();
    let mut index = 0usize;

    while index < text.len() {
        let rest = &text[index..];
        if rest.starts_with("http://") || rest.starts_with("https://") {
            if let Some(parsed) = parse_outer_span(rest) {
                let end = index + parsed.span_end;
                tokens.push(AppTextBoxToken {
                    start: index,
                    end,
                    kind: AppTextBoxTokenKind::Url,
                });
                index = end;
                continue;
            }
        }

        let Some(ch) = rest.chars().next() else {
            break;
        };
        if ch == '#' || ch == '＃' {
            let start = index;
            let mut end = index + ch.len_utf8();
            for next in text[end..].chars() {
                if next == '_' || next.is_alphanumeric() {
                    end += next.len_utf8();
                } else {
                    break;
                }
            }
            if end > start + ch.len_utf8() {
                tokens.push(AppTextBoxToken {
                    start,
                    end,
                    kind: AppTextBoxTokenKind::Tag,
                });
                index = end;
                continue;
            }
        }

        index += ch.len_utf8();
    }

    tokens
}

fn link_token_at_byte_index(text: &str, byte_index: usize) -> Option<AppTextBoxLink> {
    collect_description_tokens(text)
        .into_iter()
        .find(|token| byte_index >= token.start && byte_index <= token.end)
        .map(|token| match token.kind {
            AppTextBoxTokenKind::Url => {
                AppTextBoxLink::Url(text[token.start..token.end].to_owned())
            }
            AppTextBoxTokenKind::Tag => {
                AppTextBoxLink::Tag(text[token.start..token.end].to_owned())
            }
        })
}

fn collect_tag_segments(text: &str) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let mut index = 0;
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if ch == '#' || ch == '＃' {
            let start = index;
            let mut end = index + ch.len_utf8();
            for next in text[end..].chars() {
                if next == '_' || next.is_alphanumeric() {
                    end += next.len_utf8();
                } else {
                    break;
                }
            }
            if end > start + ch.len_utf8() {
                segments.push((start, end));
                index = end;
                continue;
            }
        }
        index += ch.len_utf8();
    }
    segments
}

fn tag_at_byte_index(text: &str, byte_index: usize) -> Option<String> {
    if text.is_empty() || byte_index > text.len() {
        return None;
    }
    let mut start = byte_index;
    while start > 0 {
        let previous = text[..start].char_indices().last()?.0;
        let ch = text[previous..start].chars().next()?;
        if ch == '#' || ch == '＃' {
            start = previous;
            break;
        }
        if !(ch == '_' || ch.is_alphanumeric()) {
            return None;
        }
        start = previous;
    }
    if !matches!(text[start..].chars().next(), Some('#' | '＃')) {
        return None;
    }
    let mut end = start + text[start..].chars().next()?.len_utf8();
    for ch in text[end..].chars() {
        if ch == '_' || ch.is_alphanumeric() {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    if end > start + text[start..].chars().next()?.len_utf8() {
        Some(text[start..end].to_owned())
    } else {
        None
    }
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
        normal: text_field_text_color(ui),
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
