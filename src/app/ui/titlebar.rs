use eframe::egui::{self, Align2, Color32, FontId, Image, Rect, Sense, Stroke, TextWrapMode, Ui};
use egui_taffy::{TuiBuilderLogic as _, taffy, tui};

use crate::app::custom_chrome;
use crate::app::state::{AboutDetailTarget, AppMode, AppState, AppTab};
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};

use super::common::UiText;
use super::{semantic_ui_metrics, xaml_taffy_styles};

const APP_LOGO_BYTES: &[u8] = include_bytes!("../../../assets/logo.ico");

pub(super) fn render_titlebar(ui: &mut Ui, state: &mut AppState) {
    let width = ui.available_width();
    let titlebar_height = semantic_ui_metrics::titlebar_height();
    let root_rect = Rect::from_min_size(
        ui.available_rect_before_wrap().min,
        egui::vec2(width, titlebar_height),
    );

    let context_action = titlebar_context_action(state);
    let show_escape_menu = should_show_escape_menu(state);
    let navigation_enabled = !state.youtube_login_rescue_dialog_visible();
    let right_client_area_width = semantic_ui_metrics::titlebar_right_client_area_width(
        context_action.is_some(),
        show_escape_menu,
    );
    custom_chrome::set_titlebar_hit_test_metrics(
        titlebar_height,
        right_client_area_width,
        ui.ctx().pixels_per_point(),
    );

    ui.painter().rect_filled(
        root_rect,
        semantic_ui_metrics::titlebar_background_corner_radius(),
        titlebar_fill(ui),
    );
    ui.painter().line_segment(
        [root_rect.left_bottom(), root_rect.right_bottom()],
        Stroke::new(
            semantic_ui_metrics::titlebar_separator_stroke_width(),
            subtle_separator_color(ui),
        ),
    );

    tui(ui, ui.id().with("custom_chrome_titlebar"))
        .reserve_width(width)
        .reserve_height(titlebar_height)
        .style(xaml_taffy_styles::xaml_fixed_height_row_style(
            super::xaml_layout_contracts::SingleLineControlRowContract::new(titlebar_height),
            0.0,
        ))
        .show(|tui| {
            tui.style(fixed_cell(
                semantic_ui_metrics::titlebar_app_icon_cell_width(),
            ))
            .ui(render_app_icon);
            tui.style(title_cell()).ui(render_title_region);
            if let Some(action) = context_action {
                tui.style(fixed_cell(
                    semantic_ui_metrics::titlebar_context_action_button_width(),
                ))
                .ui(|ui| render_titlebar_context_action(ui, state, action, navigation_enabled));
            }
            if show_escape_menu {
                tui.style(fixed_cell(
                    semantic_ui_metrics::titlebar_escape_button_width(),
                ))
                .ui(|ui| render_escape_menu(ui, state, root_rect, navigation_enabled));
            }
            tui.style(fixed_cell(
                semantic_ui_metrics::titlebar_window_button_width(),
            ))
            .ui(|ui| {
                render_caption_control(
                    ui,
                    AppIcon::WindowMinimize,
                    "Minimize",
                    CaptionButtonKind::Normal,
                    true,
                )
            });
            tui.style(fixed_cell(
                semantic_ui_metrics::titlebar_window_button_width(),
            ))
            .ui(|ui| {
                let is_maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
                let icon = if is_maximized {
                    AppIcon::WindowRestore
                } else {
                    AppIcon::WindowMaximize
                };
                let hint = if is_maximized { "Restore" } else { "Maximize" };
                render_caption_control(ui, icon, hint, CaptionButtonKind::Normal, true);
            });
            tui.style(fixed_cell(
                semantic_ui_metrics::titlebar_window_button_width(),
            ))
            .ui(|ui| {
                render_caption_control(
                    ui,
                    AppIcon::WindowClose,
                    "Close",
                    CaptionButtonKind::Close,
                    true,
                )
            });
        });
}

fn fixed_cell(width: f32) -> taffy::Style {
    xaml_taffy_styles::xaml_fixed_size_flex_cell_style(
        width,
        semantic_ui_metrics::titlebar_height(),
    )
}

fn title_cell() -> taffy::Style {
    xaml_taffy_styles::xaml_weighted_width_stretch_height_cell_style(1.0)
}

fn cell_rect(ui: &Ui) -> Rect {
    let mut rect = ui.max_rect();
    rect.max.y = rect.min.y + semantic_ui_metrics::titlebar_height();
    rect
}

fn render_app_icon(ui: &mut Ui) {
    let rect = cell_rect(ui);
    let icon_rect = Rect::from_min_size(
        egui::pos2(
            rect.left() + semantic_ui_metrics::titlebar_app_icon_left_margin(),
            rect.center().y - semantic_ui_metrics::titlebar_app_icon_size() / 2.0,
        ),
        egui::vec2(
            semantic_ui_metrics::titlebar_app_icon_size(),
            semantic_ui_metrics::titlebar_app_icon_size(),
        ),
    );
    ui.put(
        icon_rect,
        Image::from_bytes("bytes://app-title-logo.ico", APP_LOGO_BYTES).fit_to_exact_size(
            egui::vec2(
                semantic_ui_metrics::titlebar_app_icon_size(),
                semantic_ui_metrics::titlebar_app_icon_size(),
            ),
        ),
    );
}

fn render_title_region(ui: &mut Ui) {
    let rect = cell_rect(ui);
    let response = ui.interact(
        rect,
        ui.id().with("title_drag_region"),
        Sense::click_and_drag(),
    );

    let title_pos = egui::pos2(
        rect.left() + semantic_ui_metrics::titlebar_title_left_padding(),
        rect.center().y,
    );
    ui.painter().text(
        title_pos,
        Align2::LEFT_CENTER,
        "yt-dlp-gui",
        FontId::proportional(semantic_ui_metrics::titlebar_app_title_font_size()),
        title_text_color(ui),
    );

    if response.clicked() || response.double_clicked() || response.drag_started() {
        egui::Popup::close_all(ui.ctx());
    }
    if response.double_clicked() {
        toggle_maximized(ui);
    }
    if response.drag_started() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
}

fn should_show_escape_menu(state: &AppState) -> bool {
    !state.should_show_prepare_tab()
}

#[derive(Clone, Copy)]
enum TitlebarContextAction {
    Home,
    UpdateSignal,
}

fn titlebar_context_action(state: &AppState) -> Option<TitlebarContextAction> {
    if state.should_show_prepare_tab() {
        None
    } else if state.active_tab != AppTab::Main {
        Some(TitlebarContextAction::Home)
    } else if state.component_update_attention_signal_visible() {
        Some(TitlebarContextAction::UpdateSignal)
    } else {
        None
    }
}

fn render_titlebar_context_action(
    ui: &mut Ui,
    state: &mut AppState,
    action: TitlebarContextAction,
    enabled: bool,
) {
    let (icon, hint, icon_size) = match action {
        TitlebarContextAction::Home => (
            AppIcon::Home,
            "Home",
            semantic_ui_metrics::titlebar_home_icon_size(),
        ),
        TitlebarContextAction::UpdateSignal => (
            AppIcon::NewBox,
            "Updates",
            semantic_ui_metrics::titlebar_update_signal_icon_size(),
        ),
    };

    let response = caption_icon_button(
        ui,
        icon,
        hint,
        CaptionButtonKind::Normal,
        icon_size,
        enabled,
    );

    if enabled && response.clicked() {
        egui::Popup::close_all(ui.ctx());
        match action {
            TitlebarContextAction::Home => state.active_tab = AppTab::Main,
            TitlebarContextAction::UpdateSignal => {
                state.select_about_detail(AboutDetailTarget::App);
                state.active_tab = AppTab::About;
            }
        }
    }
}

fn render_escape_menu(ui: &mut Ui, state: &mut AppState, titlebar_rect: Rect, enabled: bool) {
    let response = caption_icon_button(
        ui,
        AppIcon::MenuDown,
        "Options",
        CaptionButtonKind::Normal,
        semantic_ui_metrics::titlebar_escape_icon_size(),
        enabled,
    );

    let popup_id = egui::Popup::default_response_id(&response);
    let popup_open = egui::Popup::is_id_open(ui.ctx(), popup_id);
    custom_chrome::set_titlebar_client_hit_test_for_popup(popup_open);

    if !enabled {
        egui::Popup::close_id(ui.ctx(), popup_id);
        custom_chrome::set_titlebar_client_hit_test_for_popup(false);
        return;
    }

    if !ui.input(|input| input.focused) {
        egui::Popup::close_id(ui.ctx(), popup_id);
        custom_chrome::set_titlebar_client_hit_test_for_popup(false);
    } else if popup_open && clicked_titlebar_outside_escape_button(ui, titlebar_rect, response.rect)
    {
        egui::Popup::close_id(ui.ctx(), popup_id);
        custom_chrome::set_titlebar_client_hit_test_for_popup(false);
    }

    egui::Popup::menu(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            let menu_width = escape_menu_width(ui, state);
            ui.set_width(menu_width);

            if !state.should_show_prepare_tab() {
                for mode in AppMode::ALL {
                    titlebar_app_mode_item(ui, state, mode);
                }
                ui.separator();
            }

            if !state.should_show_prepare_tab() {
                titlebar_menu_item(ui, state, AppTab::Advance, UiText::TAB_ADVANCE);
            }
            titlebar_menu_item(ui, state, AppTab::Options, UiText::TAB_OPTIONS);
            titlebar_menu_item(ui, state, AppTab::About, UiText::TAB_ABOUT);
            if state.config.show_log_tab {
                titlebar_menu_item(ui, state, AppTab::Log, UiText::TAB_LOG);
            }
        });
}

fn escape_menu_width(ui: &Ui, state: &AppState) -> f32 {
    let mut labels: Vec<&'static str> = Vec::new();

    if !state.should_show_prepare_tab() {
        labels.extend(
            AppMode::ALL
                .iter()
                .map(|mode| state.ui_i18n_text_for_key(mode.label_key())),
        );
    }

    if !state.should_show_prepare_tab() {
        labels.push(state.ui_i18n_text_for_key(UiText::TAB_ADVANCE));
    }

    if !state.should_show_prepare_tab() {
        labels.push(state.ui_i18n_text_for_key(UiText::TAB_ADVANCE));
    }
    labels.push(state.ui_i18n_text_for_key(UiText::TAB_OPTIONS));
    labels.push(state.ui_i18n_text_for_key(UiText::TAB_ABOUT));
    if state.config.show_log_tab {
        labels.push(state.ui_i18n_text_for_key(UiText::TAB_LOG));
    }

    semantic_ui_metrics::titlebar_escape_menu_width_for_visible_labels(ui, labels)
}

fn clicked_titlebar_outside_escape_button(ui: &Ui, titlebar_rect: Rect, escape_rect: Rect) -> bool {
    ui.input(|input| {
        if !input.pointer.any_pressed() {
            return false;
        }
        let Some(pos) = input.pointer.interact_pos() else {
            return false;
        };
        titlebar_rect.contains(pos) && !escape_rect.contains(pos)
    })
}

fn titlebar_app_mode_item(ui: &mut Ui, state: &mut AppState, mode: AppMode) {
    let item_width = escape_menu_item_width(ui);
    if ui
        .add(
            egui::Button::selectable(
                state.app_mode() == mode,
                state.ui_i18n_text_for_key(mode.label_key()),
            )
            .wrap_mode(TextWrapMode::Extend)
            .min_size(egui::vec2(item_width, ui.spacing().interact_size.y)),
        )
        .clicked()
    {
        state.set_app_mode(mode);
        state.active_tab = AppTab::Main;
        ui.close();
    }
}

fn titlebar_menu_item(ui: &mut Ui, state: &mut AppState, tab: AppTab, label_key: &'static str) {
    let item_width = escape_menu_item_width(ui);
    let response = ui.add(
        egui::Button::selectable(
            state.active_tab == tab,
            state.ui_i18n_text_for_key(label_key),
        )
        .wrap_mode(TextWrapMode::Extend)
        .min_size(egui::vec2(item_width, ui.spacing().interact_size.y)),
    );

    if response.clicked() {
        if tab == AppTab::Log {
            state.enter_log_tab();
        } else {
            state.active_tab = tab;
        }
        ui.close();
    }
}

fn escape_menu_item_width(ui: &Ui) -> f32 {
    semantic_ui_metrics::titlebar_menu_item_minimum_width_from_current_control_metrics(ui)
}

#[derive(Clone, Copy)]
enum CaptionButtonKind {
    Normal,
    Close,
}

fn render_caption_control(
    ui: &mut Ui,
    icon: AppIcon,
    hover_text: &'static str,
    kind: CaptionButtonKind,
    enabled: bool,
) {
    let response = caption_icon_button(
        ui,
        icon,
        hover_text,
        kind,
        semantic_ui_metrics::titlebar_window_button_icon_size(),
        enabled,
    );
    if enabled && response.clicked() {
        match icon {
            AppIcon::WindowMinimize => ui
                .ctx()
                .send_viewport_cmd(egui::ViewportCommand::Minimized(true)),
            AppIcon::WindowMaximize | AppIcon::WindowRestore => toggle_maximized(ui),
            AppIcon::WindowClose => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close),
            _ => {}
        }
    }
}

fn caption_icon_button(
    ui: &mut Ui,
    icon: AppIcon,
    hover_text: &'static str,
    kind: CaptionButtonKind,
    icon_size: f32,
    enabled: bool,
) -> egui::Response {
    let rect = cell_rect(ui);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let response = ui.interact(rect, ui.id().with(hover_text), sense);

    if enabled && response.is_pointer_button_down_on() {
        ui.painter().rect_filled(
            rect,
            semantic_ui_metrics::titlebar_background_corner_radius(),
            caption_pressed_fill_for(ui, kind),
        );
    } else if enabled && response.hovered() {
        ui.painter().rect_filled(
            rect,
            semantic_ui_metrics::titlebar_background_corner_radius(),
            caption_hover_fill_for(ui, kind),
        );
    }

    let icon_rect = Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    ui.put(
        icon_rect,
        icon_image(
            icon,
            icon_size,
            caption_icon_color(ui, kind, enabled, response.hovered()),
        ),
    );

    response
}

fn toggle_maximized(ui: &Ui) {
    let is_maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
    ui.ctx()
        .send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
}

fn titlebar_fill(ui: &Ui) -> Color32 {
    ui.visuals().panel_fill
}

fn title_text_color(ui: &Ui) -> Color32 {
    ui.visuals().text_color().linear_multiply(0.84)
}

fn caption_icon_color(ui: &Ui, kind: CaptionButtonKind, enabled: bool, hovered: bool) -> Color32 {
    if !enabled {
        return standard_icon_color(ui).linear_multiply(0.38);
    }

    match kind {
        CaptionButtonKind::Close if hovered => Color32::WHITE,
        _ => standard_icon_color(ui).linear_multiply(0.86),
    }
}

fn caption_hover_fill(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_white_alpha(28)
    } else {
        Color32::from_black_alpha(18)
    }
}

fn caption_pressed_fill(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_white_alpha(42)
    } else {
        Color32::from_black_alpha(30)
    }
}

fn caption_hover_fill_for(ui: &Ui, kind: CaptionButtonKind) -> Color32 {
    match kind {
        CaptionButtonKind::Close => Color32::from_rgb(196, 43, 28),
        CaptionButtonKind::Normal => caption_hover_fill(ui),
    }
}

fn caption_pressed_fill_for(ui: &Ui, kind: CaptionButtonKind) -> Color32 {
    match kind {
        CaptionButtonKind::Close => Color32::from_rgb(153, 32, 22),
        CaptionButtonKind::Normal => caption_pressed_fill(ui),
    }
}

fn subtle_separator_color(ui: &Ui) -> Color32 {
    ui.visuals()
        .widgets
        .noninteractive
        .bg_stroke
        .color
        .linear_multiply(0.55)
}
