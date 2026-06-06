use eframe::egui::{self, Align, RichText, Sense, TextStyle, TextWrapMode, Ui, WidgetText};
use egui_taffy::{Tui, TuiBuilderLogic as _, taffy, tui};

use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};

use super::{semantic_ui_metrics, xaml_layout_contracts, xaml_taffy_styles};

pub(super) struct UiText;
impl UiText {
    pub(super) const TAB_ADVANCE: &'static str = "tab.advanced";
    pub(super) const TAB_ABOUT: &'static str = "tab.about";
    pub(super) const TAB_OPTIONS: &'static str = "tab.options";
    pub(super) const TAB_LOG: &'static str = "tab.log";
    pub(super) const URL_HINT: &'static str = "main.url_hint";
    pub(super) const DOWNLOAD: &'static str = "action.download";
    pub(super) const VIDEO: &'static str = "media.video";
    pub(super) const AUDIO: &'static str = "media.audio";
    pub(super) const SUBTITLE: &'static str = "media.subtitle";
    pub(super) const SECTION: &'static str = "media.section";
    pub(super) const FILE_NAME: &'static str = "item.file_name";
    pub(super) const TARGET_DIR: &'static str = "main.target_folder";
    pub(super) const SELECT_VIDEO_TITLE: &'static str = "picker.title.video";
    pub(super) const SELECT_AUDIO_TITLE: &'static str = "picker.title.audio";
    pub(super) const SELECT_SUBTITLE_TITLE: &'static str = "picker.title.subtitle";
    pub(super) const SELECT_SECTION_TITLE: &'static str = "picker.title.section";
    pub(super) const BACK_TO_MAIN: &'static str = "action.back";
    pub(super) const PICKER_MODE_FILTER: &'static str = "picker.mode.filter";
    pub(super) const PICKER_MODE_TABLE: &'static str = "picker.mode.table";
    pub(super) const CONFIRM: &'static str = "action.confirm";
    pub(super) const EMPTY_TABLE: &'static str = "picker.empty_table";
    pub(super) const HEADER_RESOLUTION: &'static str = "picker.header.resolution";
    pub(super) const HEADER_RANGE: &'static str = "picker.header.range";
    pub(super) const HEADER_FPS: &'static str = "picker.header.fps";
    pub(super) const HEADER_EXT: &'static str = "picker.header.format";
    pub(super) const HEADER_CODEC: &'static str = "picker.header.codec";
    pub(super) const HEADER_FILESIZE: &'static str = "picker.header.size";
    pub(super) const HEADER_SAMPLE_RATE: &'static str = "picker.header.sample_rate";
    pub(super) const FILTER_RESOLUTION: &'static str = "picker.filter.resolution";
    pub(super) const FILTER_RANGE: &'static str = "picker.filter.range";
    pub(super) const FILTER_FPS: &'static str = "picker.filter.fps";
    pub(super) const FILTER_CODEC: &'static str = "picker.filter.codec";
    pub(super) const FILTER_SAMPLE_RATE: &'static str = "picker.filter.sample_rate";
}

pub(super) fn icon_text_button(ui: &Ui, icon: AppIcon, label: &str) -> egui::Button<'static> {
    let size = semantic_ui_metrics::standard_icon_size_from_current_control_metrics(ui);
    egui::Button::image_and_text(
        icon_image(icon, size, standard_icon_color(ui)),
        RichText::new(label).size(size),
    )
}

pub(super) fn text_trailing_icon_button(
    ui: &Ui,
    label: &str,
    icon: AppIcon,
) -> impl egui::Widget + 'static {
    let label = label.to_owned();
    let icon_size = semantic_ui_metrics::standard_icon_size_from_current_control_metrics(ui);
    move |ui: &mut Ui| {
        let galley = WidgetText::from(label.clone()).into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Button,
        );
        let padding = ui.spacing().button_padding;
        let icon_spacing = ui.spacing().icon_spacing;
        let content_width = galley.size().x + icon_spacing + icon_size;
        let desired_size = egui::vec2(
            (content_width + padding.x * 2.0).max(ui.spacing().interact_size.x),
            ui.spacing().interact_size.y,
        );
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
        let visuals = ui.style().interact(&response);

        ui.painter().rect(
            rect,
            2.0,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Outside,
        );

        let content_left = rect.center().x - content_width / 2.0;
        let text_pos = egui::pos2(content_left, rect.center().y - galley.size().y / 2.0);
        ui.painter()
            .galley(text_pos, galley.clone(), visuals.text_color());

        let icon_min = egui::pos2(
            content_left + galley.size().x + icon_spacing,
            rect.center().y - icon_size / 2.0,
        );
        let icon_rect = egui::Rect::from_min_size(icon_min, egui::vec2(icon_size, icon_size));
        icon_image(icon, icon_size, standard_icon_color(ui)).paint_at(ui, icon_rect);

        response
    }
}

pub(super) fn shared_item_label_width(ui: &Ui) -> f32 {
    semantic_ui_metrics::settings_form_label_column_width_for_visible_texts(
        ui,
        &[
            UiText::VIDEO,
            UiText::AUDIO,
            UiText::SUBTITLE,
            UiText::SECTION,
            UiText::FILE_NAME,
        ],
    )
}

pub(super) fn cell_label(ui: &mut Ui, text: &str) {
    ui.add(
        egui::Label::new(text)
            .selectable(false)
            .sense(Sense::empty()),
    );
}

pub(super) fn cell_label_center(ui: &mut Ui, text: &str) {
    ui.with_layout(
        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        |ui| {
            cell_label(ui, text);
        },
    );
}

pub(super) fn cell_label_right(ui: &mut Ui, text: &str) {
    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
        cell_label(ui, text);
    });
}

pub(super) fn form_row_label(
    ui: &mut Ui,
    label_width: f32,
    label: &str,
    add_contents: impl FnOnce(&mut Ui),
) {
    let row = semantic_ui_metrics::xaml_settings_form_single_line_row_contract_from_current_control_metrics(ui);
    form_row_label_with_xaml_contract(ui, row, label_width, label, add_contents);
}

fn form_row_label_with_xaml_contract(
    ui: &mut Ui,
    row: xaml_layout_contracts::SingleLineControlRowContract,
    label_width: f32,
    label: &str,
    add_contents: impl FnOnce(&mut Ui),
) {
    ui.horizontal(|ui| {
        let label_element = semantic_ui_metrics::xaml_label_ui_element_from_row_contract_and_width(
            row,
            label_width,
        );
        let label_size = row.measure_auto_width_ui_element(label_element);
        ui.allocate_ui_with_layout(
            egui::vec2(label_size.width, label_size.height),
            egui::Layout::right_to_left(Align::Center),
            |ui| {
                ui.label(label);
            },
        );
        ui.vertical(|ui| {
            let value_element =
                semantic_ui_metrics::xaml_single_line_text_input_ui_element_from_row_contract(row);
            let value_size =
                row.measure_stretch_width_ui_element(value_element, ui.available_width());
            ui.set_width(value_size.width);
            ui.set_min_height(value_size.height);
            add_contents(ui);
        });
    });
}

pub(super) fn settings_scroll_content(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    scroll_content_with_right_gap(ui, add_contents);
}

pub(super) fn scroll_content_with_right_gap(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    let trailing_safe_gap = semantic_ui_metrics::settings_scroll_content_trailing_safe_gap(ui);
    let content_width = (ui.available_width() - trailing_safe_gap).max(0.0);
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(content_width, 0.0),
            egui::Layout::top_down(Align::Min),
            |ui| {
                ui.set_width(content_width);
                add_contents(ui);
            },
        );
        ui.add_space(trailing_safe_gap);
    });
}

pub(super) fn settings_taffy_scroll_content(
    ui: &mut Ui,
    id_salt: &'static str,
    add_contents: impl FnOnce(&mut Tui),
) {
    scroll_content_with_right_gap(ui, |ui| {
        let content_width = ui.available_width();
        tui(ui, ui.id().with(id_salt))
            .reserve_width(content_width)
            .style(settings_taffy_root_style())
            .show(|tui| add_contents(tui));
    });
}

pub(super) fn settings_taffy_section(
    tui: &mut Tui,
    title: &str,
    add_contents: impl FnOnce(&mut Tui),
) {
    let title = title.to_owned();
    settings_taffy_spacer(
        tui,
        semantic_ui_metrics::settings_form_section_before_title_vertical_spacing(),
    );
    tui.style(settings_taffy_auto_row_style()).ui(move |ui| {
        ui.label(RichText::new(title.as_str()).strong());
    });
    tui.style(settings_taffy_auto_row_style()).ui(|ui| {
        ui.separator();
    });
    settings_taffy_spacer(
        tui,
        semantic_ui_metrics::settings_form_section_after_separator_vertical_spacing(),
    );
    tui.style(settings_taffy_section_body_style())
        .add(add_contents);
    settings_taffy_spacer(
        tui,
        semantic_ui_metrics::settings_form_section_after_body_vertical_spacing(),
    );
}

pub(super) fn settings_taffy_form_row(
    tui: &mut Tui,
    label_width: f32,
    label: &str,
    add_contents: impl FnOnce(&mut Ui),
) {
    let label = label.to_owned();
    tui.style(settings_taffy_auto_row_style()).ui(move |ui| {
        let row = semantic_ui_metrics::xaml_settings_form_single_line_row_contract_from_current_control_metrics(ui);
        let vertical_gap = semantic_ui_metrics::settings_form_row_vertical_spacing_from_current_text_metrics(ui);
        ui.set_min_height(
            semantic_ui_metrics::xaml_settings_form_row_minimum_height_from_row_contract_and_vertical_spacing(
                row,
                vertical_gap,
            ),
        );
        ui.add_space(vertical_gap);
        form_row_label_with_xaml_contract(ui, row, label_width, label.as_str(), |ui| {
            add_contents(ui);
        });
        ui.add_space(vertical_gap);
    });
}

fn settings_taffy_root_style() -> taffy::Style {
    xaml_taffy_styles::xaml_vertical_auto_root_style()
}

fn settings_taffy_section_body_style() -> taffy::Style {
    xaml_taffy_styles::xaml_vertical_auto_section_style()
}

fn settings_taffy_auto_row_style() -> taffy::Style {
    xaml_taffy_styles::xaml_auto_height_block_style()
}

fn settings_taffy_spacer(tui: &mut Tui, height: f32) {
    if height <= 0.0 {
        return;
    }
    tui.style(xaml_taffy_styles::xaml_fixed_height_block_style(height))
        .ui(|_| {});
}

pub(super) fn settings_section(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    ui.add_space(semantic_ui_metrics::settings_form_section_before_title_vertical_spacing());
    ui.label(RichText::new(title).strong());
    ui.separator();
    ui.add_space(semantic_ui_metrics::settings_form_section_after_separator_vertical_spacing());
    add_contents(ui);
    ui.add_space(semantic_ui_metrics::settings_form_section_after_body_vertical_spacing());
}
