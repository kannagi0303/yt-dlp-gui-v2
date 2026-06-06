use eframe::egui::{self, RichText, Ui};
use egui_taffy::{Tui, TuiBuilderLogic as _, tui};

use crate::app::state::{AppState, YoutubeLoginRescuePhase};
use crate::app::widgets::url_input::{AppTextBox, AppTextBoxSyntax};

use super::semantic_ui_metrics;
use super::xaml_taffy_styles::XamlSingleLineRowLayout;

pub(super) fn render_youtube_login_rescue_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.youtube_login_rescue_dialog_visible() {
        return;
    }

    egui::Window::new(state.ui_i18n_text_for_key("youtube_login_rescue.title"))
        .id(egui::Id::new((
            "youtube-login-rescue-dialog-fit-v4",
            youtube_login_rescue_phase_window_id(state),
        )))
        .collapsible(false)
        .resizable(false)
        .anchor(
            egui::Align2::CENTER_CENTER,
            semantic_ui_metrics::cookie_acquisition_dialog_center_anchor_vector(),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing =
                semantic_ui_metrics::cookie_acquisition_dialog_item_spacing();
            ui.spacing_mut().button_padding =
                semantic_ui_metrics::cookie_acquisition_dialog_button_padding();

            let dialog_width = youtube_login_rescue_dialog_width(ctx, ui, state);
            ui.set_width(dialog_width);
            ui.set_min_width(dialog_width);
            ui.set_max_width(dialog_width);

            match state.youtube_login_rescue_phase {
                YoutubeLoginRescuePhase::Confirm => {
                    render_youtube_login_rescue_confirm(ui, state, dialog_width)
                }
                YoutubeLoginRescuePhase::NoSupportedBrowser => {
                    render_youtube_login_rescue_no_browser(ui, state)
                }
                YoutubeLoginRescuePhase::Starting
                | YoutubeLoginRescuePhase::WaitingForCdp
                | YoutubeLoginRescuePhase::WaitingForCookie => {
                    render_youtube_login_rescue_running(ui, state)
                }
                YoutubeLoginRescuePhase::CookieExported => {
                    render_youtube_login_rescue_exported(ui, state)
                }
                YoutubeLoginRescuePhase::Failed => render_youtube_login_rescue_failed(ui, state),
                YoutubeLoginRescuePhase::Idle | YoutubeLoginRescuePhase::Closed => {}
            }
        });
}
fn youtube_login_rescue_phase_window_id(state: &AppState) -> &'static str {
    match state.youtube_login_rescue_phase {
        YoutubeLoginRescuePhase::Confirm => "confirm",
        YoutubeLoginRescuePhase::NoSupportedBrowser => "no-browser",
        YoutubeLoginRescuePhase::Starting => "starting",
        YoutubeLoginRescuePhase::WaitingForCdp => "waiting-cdp",
        YoutubeLoginRescuePhase::WaitingForCookie => "waiting-cookie",
        YoutubeLoginRescuePhase::CookieExported => "exported",
        YoutubeLoginRescuePhase::Failed => "failed",
        YoutubeLoginRescuePhase::Idle => "idle",
        YoutubeLoginRescuePhase::Closed => "closed",
    }
}
fn youtube_login_rescue_dialog_width(ctx: &egui::Context, ui: &Ui, state: &AppState) -> f32 {
    let browser_name = state
        .youtube_login_rescue_browser
        .as_ref()
        .map(|browser| browser.display_name.clone())
        .unwrap_or_else(|| "Chrome".to_owned());
    let site_name = state
        .youtube_login_rescue_site_name
        .as_deref()
        .unwrap_or("Cookie");
    let error_text = state
        .youtube_login_rescue_error
        .as_deref()
        .unwrap_or_default();
    let target_error_text = state
        .youtube_login_rescue_target_error
        .as_deref()
        .unwrap_or_default();

    let confirm_body = state.ui_i18n_text_with_replacements(
        "youtube_login_rescue.confirm_body",
        &[("{browser}", browser_name.as_str())],
    );
    let no_browser_body = state
        .ui_i18n_text_for_key("youtube_login_rescue.no_browser_body")
        .to_owned();
    let opening = state.ui_i18n_text_with_replacements(
        "youtube_login_rescue.opening",
        &[("{browser}", browser_name.as_str())],
    );
    let waiting_for_cdp = state.ui_i18n_text_with_replacements(
        "youtube_login_rescue.waiting_for_cdp",
        &[("{browser}", browser_name.as_str())],
    );
    let exported_note = state.ui_i18n_text_with_replacements(
        "youtube_login_rescue.cookie_exported_note",
        &[("{site}", site_name)],
    );

    let confirm_texts = [
        state.ui_i18n_text_for_key("youtube_login_rescue.title"),
        confirm_body.as_str(),
        state.ui_i18n_text_for_key("youtube_login_rescue.cookie_note"),
        target_error_text,
    ];
    let no_browser_texts = [
        state.ui_i18n_text_for_key("youtube_login_rescue.no_browser_title"),
        no_browser_body.as_str(),
    ];
    let running_texts = [
        opening.as_str(),
        waiting_for_cdp.as_str(),
        state.ui_i18n_text_for_key("youtube_login_rescue.waiting_for_cookie"),
        state.ui_i18n_text_for_key("youtube_login_rescue.do_not_close_note"),
    ];
    let exported_texts = [
        state.ui_i18n_text_for_key("youtube_login_rescue.cookie_exported"),
        exported_note.as_str(),
    ];
    let failed_texts = [
        state.ui_i18n_text_for_key("youtube_login_rescue.failed"),
        error_text,
    ];

    let confirm_actions = [
        state.ui_i18n_text_for_key("youtube_login_rescue.paste_clipboard"),
        state.ui_i18n_text_for_key("options.cancel"),
        state.ui_i18n_text_for_key("youtube_login_rescue.start"),
    ];
    let confirm_only_actions = [state.ui_i18n_text_for_key("action.confirm")];
    let failed_actions = [
        state.ui_i18n_text_for_key("youtube_login_rescue.retry"),
        state.ui_i18n_text_for_key("options.cancel"),
    ];
    let no_actions: [&str; 0] = [];

    let (texts, action_texts, min_width): (&[&str], &[&str], f32) =
        match state.youtube_login_rescue_phase {
            YoutubeLoginRescuePhase::Confirm => (
                &confirm_texts,
                &confirm_actions,
                semantic_ui_metrics::cookie_acquisition_dialog_minimum_width_for_start_phase(),
            ),
            YoutubeLoginRescuePhase::NoSupportedBrowser => (
                &no_browser_texts,
                &confirm_only_actions,
                semantic_ui_metrics::cookie_acquisition_dialog_minimum_width_for_message_phase(),
            ),
            YoutubeLoginRescuePhase::Starting
            | YoutubeLoginRescuePhase::WaitingForCdp
            | YoutubeLoginRescuePhase::WaitingForCookie => (
                &running_texts,
                &no_actions,
                semantic_ui_metrics::cookie_acquisition_dialog_minimum_width_for_message_phase(),
            ),
            YoutubeLoginRescuePhase::CookieExported => (
                &exported_texts,
                &confirm_only_actions,
                semantic_ui_metrics::cookie_acquisition_dialog_minimum_width_for_message_phase(),
            ),
            YoutubeLoginRescuePhase::Failed => (
                &failed_texts,
                &failed_actions,
                semantic_ui_metrics::cookie_acquisition_dialog_minimum_width_for_message_phase(),
            ),
            YoutubeLoginRescuePhase::Idle | YoutubeLoginRescuePhase::Closed => (
                &[],
                &no_actions,
                semantic_ui_metrics::cookie_acquisition_dialog_minimum_width_for_message_phase(),
            ),
        };

    let max_width = semantic_ui_metrics::cookie_acquisition_dialog_maximum_width_for_viewport(
        ctx.content_rect().width(),
        min_width,
    );
    let text_width = semantic_ui_metrics::cookie_acquisition_dialog_content_width_for_visible_texts(
        ui,
        texts.iter().copied(),
        min_width,
        max_width,
    );
    let action_width =
        semantic_ui_metrics::cookie_acquisition_dialog_action_row_width_for_visible_labels(
            ui,
            action_texts,
            min_width,
            max_width,
        );

    text_width.max(action_width).clamp(min_width, max_width)
}
fn dialog_action_row_contract(
    ui: &Ui,
) -> super::xaml_layout_contracts::SingleLineControlRowContract {
    semantic_ui_metrics::xaml_dialog_action_row_contract_from_current_control_metrics(ui)
}
fn dialog_action_button_size_for_label(
    ui: &Ui,
    row_contract: super::xaml_layout_contracts::SingleLineControlRowContract,
    label: &str,
) -> [f32; 2] {
    row_contract
        .measure_auto_width_ui_element(
            semantic_ui_metrics::xaml_button_ui_element_for_visible_text(ui, label),
        )
        .to_array()
}
fn dialog_shared_action_button_size_for_keys(
    ui: &Ui,
    state: &AppState,
    row_contract: super::xaml_layout_contracts::SingleLineControlRowContract,
    keys: &[&'static str],
) -> [f32; 2] {
    semantic_ui_metrics::xaml_dialog_action_button_shared_size_group_for_translated_label_keys(
        ui, state, keys,
    )
    .equal_width_button_size_for_row(row_contract)
    .to_array()
}
fn render_dialog_action_row(
    ui: &mut Ui,
    id_salt: &'static str,
    add_contents: impl FnOnce(&mut Tui, XamlSingleLineRowLayout),
) {
    let row_contract = dialog_action_row_contract(ui);
    let row_layout =
        XamlSingleLineRowLayout::new(row_contract).with_column_gap(ui.spacing().item_spacing.x);
    let available_width = ui.available_width();
    tui(ui, ui.id().with(id_salt))
        .reserve_width(available_width)
        .reserve_height(row_layout.height())
        .style(row_layout.style())
        .show(|tui| add_contents(tui, row_layout));
}
fn dialog_action_button_cell(
    tui: &mut Tui,
    row_layout: XamlSingleLineRowLayout,
    size: [f32; 2],
    label: &str,
) -> bool {
    let mut clicked = false;
    tui.style(row_layout.fixed_width_cell_style(size[0]))
        .ui(|ui| {
            clicked = ui.add_sized(size, egui::Button::new(label)).clicked();
        });
    clicked
}
fn dialog_action_spacer_cell(tui: &mut Tui, row_layout: XamlSingleLineRowLayout) {
    tui.style(row_layout.flex_spacer_cell_style()).ui(|_| {});
}
fn render_youtube_login_rescue_confirm(ui: &mut Ui, state: &mut AppState, dialog_width: f32) {
    let dropped_paths = ui.ctx().input(|input| {
        input
            .raw
            .dropped_files
            .iter()
            .filter_map(|file| file.path.clone())
            .collect::<Vec<_>>()
    });
    if !dropped_paths.is_empty() {
        state.apply_youtube_login_rescue_dropped_paths(dropped_paths);
    }

    let browser_name = state
        .youtube_login_rescue_browser
        .as_ref()
        .map(|browser| browser.display_name.clone())
        .unwrap_or_else(|| "Chrome".to_owned());
    let mut target_url = state.youtube_login_rescue_target_url.clone();
    let response = AppTextBox::new(&mut target_url)
        .hint_text(state.ui_i18n_text_for_key("youtube_login_rescue.target_url_hint"))
        .language(state.language())
        .syntax(AppTextBoxSyntax::Url)
        .desired_width(dialog_width)
        .error(state.youtube_login_rescue_target_error.is_some())
        .ui(ui);
    if response.changed() {
        state.set_youtube_login_rescue_target_url(target_url);
    }

    if let Some(error) = state.youtube_login_rescue_target_error.as_ref() {
        ui.colored_label(ui.visuals().error_fg_color, state.localize_message(error));
    }

    ui.label(state.ui_i18n_text_with_replacements(
        "youtube_login_rescue.confirm_body",
        &[("{browser}", browser_name.as_str())],
    ));
    ui.label(state.ui_i18n_text_for_key("youtube_login_rescue.cookie_note"));
    ui.add_space(semantic_ui_metrics::cookie_acquisition_dialog_field_to_body_vertical_spacing());
    let row_contract = dialog_action_row_contract(ui);
    let paste_label = state
        .ui_i18n_text_for_key("youtube_login_rescue.paste_clipboard")
        .to_owned();
    let cancel_label = state.ui_i18n_text_for_key("options.cancel").to_owned();
    let start_label = state
        .ui_i18n_text_for_key("youtube_login_rescue.start")
        .to_owned();
    let paste_button_size = dialog_action_button_size_for_label(ui, row_contract, &paste_label);
    let shared_button_size = dialog_shared_action_button_size_for_keys(
        ui,
        state,
        row_contract,
        &["youtube_login_rescue.start", "options.cancel"],
    );
    let mut paste_clicked = false;
    let mut cancel_clicked = false;
    let mut start_clicked = false;
    render_dialog_action_row(
        ui,
        "youtube-login-rescue-confirm-actions",
        |tui, row_layout| {
            paste_clicked =
                dialog_action_button_cell(tui, row_layout, paste_button_size, &paste_label);
            dialog_action_spacer_cell(tui, row_layout);
            cancel_clicked =
                dialog_action_button_cell(tui, row_layout, shared_button_size, &cancel_label);
            start_clicked =
                dialog_action_button_cell(tui, row_layout, shared_button_size, &start_label);
        },
    );
    if paste_clicked {
        state.paste_clipboard_to_youtube_login_rescue_target();
    }
    if cancel_clicked {
        state.cancel_youtube_login_rescue_prompt();
    }
    if start_clicked {
        state.start_youtube_login_rescue();
    }
}
fn render_youtube_login_rescue_no_browser(ui: &mut Ui, state: &mut AppState) {
    ui.label(
        RichText::new(state.ui_i18n_text_for_key("youtube_login_rescue.no_browser_title")).strong(),
    );
    ui.label(state.ui_i18n_text_for_key("youtube_login_rescue.no_browser_body"));
    ui.add_space(semantic_ui_metrics::cookie_acquisition_dialog_field_to_body_vertical_spacing());
    let row_contract = dialog_action_row_contract(ui);
    let label = state.ui_i18n_text_for_key("action.confirm").to_owned();
    let button_size = dialog_action_button_size_for_label(ui, row_contract, &label);
    let mut clicked = false;
    render_dialog_action_row(
        ui,
        "youtube-login-rescue-no-browser-actions",
        |tui, row_layout| {
            dialog_action_spacer_cell(tui, row_layout);
            clicked = dialog_action_button_cell(tui, row_layout, button_size, &label);
        },
    );
    if clicked {
        state.cancel_youtube_login_rescue_prompt();
    }
}
fn render_youtube_login_rescue_running(ui: &mut Ui, state: &mut AppState) {
    let browser_name = state
        .youtube_login_rescue_browser
        .as_ref()
        .map(|browser| browser.display_name.clone())
        .unwrap_or_else(|| "Chrome".to_owned());
    let key = match state.youtube_login_rescue_phase {
        YoutubeLoginRescuePhase::Starting => "youtube_login_rescue.opening",
        YoutubeLoginRescuePhase::WaitingForCookie => "youtube_login_rescue.waiting_for_cookie",
        _ => "youtube_login_rescue.waiting_for_cdp",
    };
    ui.label(
        RichText::new(
            state.ui_i18n_text_with_replacements(key, &[("{browser}", browser_name.as_str())]),
        )
        .strong(),
    );
    ui.label(state.ui_i18n_text_for_key("youtube_login_rescue.do_not_close_note"));
}
fn render_youtube_login_rescue_exported(ui: &mut Ui, state: &mut AppState) {
    ui.label(
        RichText::new(state.ui_i18n_text_for_key("youtube_login_rescue.cookie_exported")).strong(),
    );
    let site_name = state
        .youtube_login_rescue_site_name
        .as_deref()
        .unwrap_or("Cookie");
    ui.label(state.ui_i18n_text_with_replacements(
        "youtube_login_rescue.cookie_exported_note",
        &[("{site}", site_name)],
    ));
    ui.add_space(semantic_ui_metrics::cookie_acquisition_dialog_field_to_body_vertical_spacing());
    let row_contract = dialog_action_row_contract(ui);
    let label = state.ui_i18n_text_for_key("action.confirm").to_owned();
    let button_size = dialog_action_button_size_for_label(ui, row_contract, &label);
    let mut clicked = false;
    render_dialog_action_row(
        ui,
        "youtube-login-rescue-exported-actions",
        |tui, row_layout| {
            dialog_action_spacer_cell(tui, row_layout);
            clicked = dialog_action_button_cell(tui, row_layout, button_size, &label);
        },
    );
    if clicked {
        state.close_youtube_login_rescue_browser();
    }
}
fn render_youtube_login_rescue_failed(ui: &mut Ui, state: &mut AppState) {
    ui.label(RichText::new(state.ui_i18n_text_for_key("youtube_login_rescue.failed")).strong());
    if let Some(error) = state.youtube_login_rescue_error.as_ref() {
        ui.label(error);
    }
    ui.add_space(semantic_ui_metrics::cookie_acquisition_dialog_field_to_body_vertical_spacing());
    let row_contract = dialog_action_row_contract(ui);
    let cancel_label = state.ui_i18n_text_for_key("options.cancel").to_owned();
    let retry_label = state
        .ui_i18n_text_for_key("youtube_login_rescue.retry")
        .to_owned();
    let shared_button_size = dialog_shared_action_button_size_for_keys(
        ui,
        state,
        row_contract,
        &["youtube_login_rescue.retry", "options.cancel"],
    );
    let mut cancel_clicked = false;
    let mut retry_clicked = false;
    render_dialog_action_row(
        ui,
        "youtube-login-rescue-failed-actions",
        |tui, row_layout| {
            dialog_action_spacer_cell(tui, row_layout);
            cancel_clicked =
                dialog_action_button_cell(tui, row_layout, shared_button_size, &cancel_label);
            retry_clicked =
                dialog_action_button_cell(tui, row_layout, shared_button_size, &retry_label);
        },
    );
    if cancel_clicked {
        state.cancel_youtube_login_rescue_prompt();
    }
    if retry_clicked {
        state.retry_youtube_login_rescue_detection();
    }
}
