use eframe::egui::{self, RichText, Ui};
use egui_taffy::Tui;

use crate::app::state::{AppState, OptionsDetailPage};
use crate::app::widgets::icon::AppIcon;
use crate::i18n::LanguageSelection;

use super::common::{
    settings_section, settings_taffy_form_row, settings_taffy_section, text_trailing_icon_button,
};
use super::semantic_ui_metrics;
use super::settings_detail_template::{SettingsDetailNode, render_settings_detail_page};
use super::xaml_template_renderer::{TemplateBlockSlot, show_template_ui_block};

pub(super) fn render_language_group(tui: &mut Tui, state: &mut AppState, label_width: f32) {
    settings_taffy_section(tui, state.ui_i18n_text_for_key("options.language"), |tui| {
        settings_taffy_form_row(
            tui,
            label_width,
            state.ui_i18n_text_for_key("options.language"),
            |ui| {
                let label = language_choice_label(state, state.language_selection());
                if ui
                    .add(text_trailing_icon_button(ui, &label, AppIcon::MenuRight))
                    .clicked()
                {
                    state.open_options_detail_page(OptionsDetailPage::Language);
                }
            },
        );
    });
}

pub(super) fn render_language_detail_page(ui: &mut Ui, state: &mut AppState) {
    let mut show_block = |slot, node, tui: &mut Tui| {
        show_language_detail_block(slot, node, tui, state);
    };
    render_settings_detail_page(
        ui,
        "options-language-page-scroll",
        "options-language-detail-template",
        semantic_ui_metrics::options_language_detail_header_to_body_vertical_spacing(),
        &mut show_block,
    );
}

fn show_language_detail_block(
    slot: TemplateBlockSlot,
    node: SettingsDetailNode,
    tui: &mut Tui,
    state: &mut AppState,
) {
    match node {
        SettingsDetailNode::Header => show_template_ui_block(slot, tui, |ui| {
            render_language_detail_header(ui, state);
        }),
        SettingsDetailNode::Body => show_template_ui_block(slot, tui, |ui| {
            render_language_detail_body(ui, state);
        }),
    }
}

fn render_language_detail_header(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if ui
            .button(format!("← {}", state.ui_i18n_text_for_key("options.back")))
            .clicked()
        {
            state.close_options_detail_page();
        }
        ui.label(RichText::new(state.ui_i18n_text_for_key("options.language")).strong());
    });
}

fn render_language_detail_body(ui: &mut Ui, state: &mut AppState) {
    settings_section(ui, state.ui_i18n_text_for_key("options.language"), |ui| {
        for language in LanguageSelection::PICKER_ORDER {
            render_language_choice_row(ui, state, language);
        }
    });
}

fn render_language_choice_row(ui: &mut Ui, state: &mut AppState, language: LanguageSelection) {
    let check_width = semantic_ui_metrics::options_language_checkmark_column_width();
    let selected = state.language_selection() == language;
    let label = language_choice_label(state, language);
    ui.horizontal(|ui| {
        ui.add_sized(
            [check_width, ui.spacing().interact_size.y],
            egui::Label::new(if selected { "✓" } else { "" }),
        );
        if ui.selectable_label(selected, label).clicked() {
            state.set_language_selection(language);
        }
    });
}

fn language_choice_label(state: &AppState, language: LanguageSelection) -> String {
    match language {
        LanguageSelection::Auto => format!(
            "{} ({})",
            state.ui_i18n_text_for_key("options.auto_detect"),
            language.resolve().native_name()
        ),
        _ => language.native_name().to_owned(),
    }
}
