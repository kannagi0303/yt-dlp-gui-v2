use eframe::egui::{
    self, Align2, Color32, FontId, Image, Rect, Sense, Stroke, TextStyle, TextWrapMode, Ui,
};
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic as _, taffy, tui};

use crate::app::custom_chrome;
use crate::app::state::{AppMode, AppState, AppTab};
use crate::app::widgets::icon::{AppIcon, icon_image, standard_icon_color};

use super::common::UiText;
use super::measure::{measured_text_width, WidthRange};

const APP_LOGO_BYTES: &[u8] = include_bytes!("../../../assets/logo.ico");

const TITLEBAR_HEIGHT: f32 = 26.0;
const APP_ICON_CELL_WIDTH: f32 = 25.0;
const APP_ICON_LEFT_MARGIN: f32 = 8.0;
const APP_ICON_SIZE: f32 = 16.0;
const TITLE_LEFT_PADDING: f32 = 5.0;
const WINDOW_BUTTON_WIDTH: f32 = 40.0;
const WINDOW_BUTTON_ICON_SIZE: f32 = 13.0;
const HOME_BUTTON_WIDTH: f32 = ESCAPE_BUTTON_WIDTH;
const HOME_ICON_SIZE: f32 = 17.0;
const ESCAPE_BUTTON_WIDTH: f32 = 40.0;
const ESCAPE_ICON_SIZE: f32 = 28.0;
const ESCAPE_MENU_WIDTH_GUARD: f32 = 2.0;

pub(super) fn render_titlebar(ui: &mut Ui, state: &mut AppState) {
    let width = ui.available_width();
    let root_rect = Rect::from_min_size(
        ui.available_rect_before_wrap().min,
        egui::vec2(width, TITLEBAR_HEIGHT),
    );

    let show_home_button = should_show_home_button(state);
    let right_client_area_width = if show_home_button {
        HOME_BUTTON_WIDTH + ESCAPE_BUTTON_WIDTH + WINDOW_BUTTON_WIDTH * 3.0
    } else {
        ESCAPE_BUTTON_WIDTH + WINDOW_BUTTON_WIDTH * 3.0
    };
    custom_chrome::set_titlebar_hit_test_metrics(
        TITLEBAR_HEIGHT,
        right_client_area_width,
        ui.ctx().pixels_per_point(),
    );

    ui.painter().rect_filled(root_rect, 0.0, titlebar_fill(ui));
    ui.painter().line_segment(
        [root_rect.left_bottom(), root_rect.right_bottom()],
        Stroke::new(1.0, subtle_separator_color(ui)),
    );

    tui(ui, ui.id().with("custom_chrome_titlebar"))
        .reserve_width(width)
        .reserve_height(TITLEBAR_HEIGHT)
        .style(taffy::Style {
            display: taffy::Display::Flex,
            flex_direction: taffy::FlexDirection::Row,
            align_items: Some(taffy::AlignItems::Stretch),
            size: taffy::Size {
                width: percent(1.0),
                height: length(TITLEBAR_HEIGHT),
            },
            min_size: taffy::Size {
                width: percent(1.0),
                height: length(TITLEBAR_HEIGHT),
            },
            max_size: taffy::Size {
                width: percent(1.0),
                height: length(TITLEBAR_HEIGHT),
            },
            gap: length(0.0),
            padding: length(0.0),
            ..Default::default()
        })
        .show(|tui| {
            tui.style(fixed_cell(APP_ICON_CELL_WIDTH))
                .ui(render_app_icon);
            tui.style(title_cell()).ui(render_title_region);
            if show_home_button {
                tui.style(fixed_cell(HOME_BUTTON_WIDTH))
                    .ui(|ui| render_home_button(ui, state));
            }
            tui.style(fixed_cell(ESCAPE_BUTTON_WIDTH))
                .ui(|ui| render_escape_menu(ui, state, root_rect));
            tui.style(fixed_cell(WINDOW_BUTTON_WIDTH)).ui(|ui| {
                render_caption_control(
                    ui,
                    AppIcon::WindowMinimize,
                    "Minimize",
                    CaptionButtonKind::Normal,
                )
            });
            tui.style(fixed_cell(WINDOW_BUTTON_WIDTH)).ui(|ui| {
                let is_maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
                let icon = if is_maximized {
                    AppIcon::WindowRestore
                } else {
                    AppIcon::WindowMaximize
                };
                let hint = if is_maximized { "Restore" } else { "Maximize" };
                render_caption_control(ui, icon, hint, CaptionButtonKind::Normal);
            });
            tui.style(fixed_cell(WINDOW_BUTTON_WIDTH)).ui(|ui| {
                render_caption_control(ui, AppIcon::WindowClose, "Close", CaptionButtonKind::Close)
            });
        });
}

fn fixed_cell(width: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        size: taffy::Size {
            width: length(width),
            height: length(TITLEBAR_HEIGHT),
        },
        min_size: taffy::Size {
            width: length(width),
            height: length(TITLEBAR_HEIGHT),
        },
        max_size: taffy::Size {
            width: length(width),
            height: length(TITLEBAR_HEIGHT),
        },
        flex_grow: 0.0,
        flex_shrink: 0.0,
        align_items: Some(taffy::AlignItems::Stretch),
        justify_content: Some(taffy::JustifyContent::Center),
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn title_cell() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        size: taffy::Size {
            width: auto(),
            height: length(TITLEBAR_HEIGHT),
        },
        min_size: taffy::Size {
            width: length(0.0),
            height: length(TITLEBAR_HEIGHT),
        },
        max_size: taffy::Size {
            width: percent(1.0),
            height: length(TITLEBAR_HEIGHT),
        },
        flex_basis: length(0.0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        align_items: Some(taffy::AlignItems::Stretch),
        padding: length(0.0),
        margin: length(0.0),
        ..Default::default()
    }
}

fn cell_rect(ui: &Ui) -> Rect {
    let mut rect = ui.max_rect();
    rect.max.y = rect.min.y + TITLEBAR_HEIGHT;
    rect
}

fn render_app_icon(ui: &mut Ui) {
    let rect = cell_rect(ui);
    let icon_rect = Rect::from_min_size(
        egui::pos2(
            rect.left() + APP_ICON_LEFT_MARGIN,
            rect.center().y - APP_ICON_SIZE / 2.0,
        ),
        egui::vec2(APP_ICON_SIZE, APP_ICON_SIZE),
    );
    ui.put(
        icon_rect,
        Image::from_bytes("bytes://app-title-logo.ico", APP_LOGO_BYTES)
            .fit_to_exact_size(egui::vec2(APP_ICON_SIZE, APP_ICON_SIZE)),
    );
}

fn render_title_region(ui: &mut Ui) {
    let rect = cell_rect(ui);
    let response = ui.interact(
        rect,
        ui.id().with("title_drag_region"),
        Sense::click_and_drag(),
    );

    let title_pos = egui::pos2(rect.left() + TITLE_LEFT_PADDING, rect.center().y);
    ui.painter().text(
        title_pos,
        Align2::LEFT_CENTER,
        "yt-dlp-gui",
        FontId::proportional(12.0),
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

fn should_show_home_button(state: &AppState) -> bool {
    !state.should_show_prepare_tab() && state.active_tab != AppTab::Main
}

fn render_home_button(ui: &mut Ui, state: &mut AppState) {
    let response = caption_icon_button(
        ui,
        AppIcon::Home,
        "Home",
        CaptionButtonKind::Normal,
        HOME_ICON_SIZE,
    )
    .on_hover_text(state.tr(UiText::TAB_MAIN));

    if response.clicked() {
        egui::Popup::close_all(ui.ctx());
        state.active_tab = AppTab::Main;
    }
}

fn render_escape_menu(ui: &mut Ui, state: &mut AppState, titlebar_rect: Rect) {
    let response = caption_icon_button(
        ui,
        AppIcon::MenuDown,
        "Options",
        CaptionButtonKind::Normal,
        ESCAPE_ICON_SIZE,
    );

    let popup_id = egui::Popup::default_response_id(&response);
    let popup_open = egui::Popup::is_id_open(ui.ctx(), popup_id);
    custom_chrome::set_titlebar_client_hit_test_for_popup(popup_open);

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

            if state.should_show_prepare_tab() {
                titlebar_menu_item(ui, state, AppTab::Prepare, UiText::TAB_PREPARE);
            } else {
                titlebar_menu_item(ui, state, AppTab::Main, UiText::TAB_MAIN);
                titlebar_menu_item(ui, state, AppTab::Advance, UiText::TAB_ADVANCE);
            }
            titlebar_menu_item(ui, state, AppTab::Options, UiText::TAB_OPTIONS);
            if state.config.show_log_tab {
                titlebar_menu_item(ui, state, AppTab::Log, UiText::TAB_LOG);
            }
        });
}

fn escape_menu_width(ui: &Ui, state: &AppState) -> f32 {
    let mut labels: Vec<&'static str> = Vec::new();

    if !state.should_show_prepare_tab() {
        labels.extend(AppMode::ALL.iter().map(|mode| state.tr(mode.label_key())));
    }

    if state.should_show_prepare_tab() {
        labels.push(state.tr(UiText::TAB_PREPARE));
    } else {
        labels.push(state.tr(UiText::TAB_MAIN));
        labels.push(state.tr(UiText::TAB_ADVANCE));
    }

    labels.push(state.tr(UiText::TAB_OPTIONS));
    if state.config.show_log_tab {
        labels.push(state.tr(UiText::TAB_LOG));
    }

    let horizontal_padding = ui.spacing().button_padding.x * 2.0 + ESCAPE_MENU_WIDTH_GUARD;
    measured_text_width(
        ui,
        labels,
        TextStyle::Button,
        horizontal_padding,
        WidthRange::new(ui.spacing().interact_size.x, f32::INFINITY),
    )
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
            egui::Button::selectable(state.app_mode() == mode, state.tr(mode.label_key()))
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
    if ui
        .add(
            egui::Button::selectable(state.active_tab == tab, state.tr(label_key))
                .wrap_mode(TextWrapMode::Extend)
                .min_size(egui::vec2(item_width, ui.spacing().interact_size.y)),
        )
        .clicked()
    {
        state.active_tab = tab;
        ui.close();
    }
}

fn escape_menu_item_width(ui: &Ui) -> f32 {
    ui.available_width().max(ui.spacing().interact_size.x)
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
) {
    let response = caption_icon_button(ui, icon, hover_text, kind, WINDOW_BUTTON_ICON_SIZE)
        .on_hover_text(hover_text);
    if response.clicked() {
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
) -> egui::Response {
    let rect = cell_rect(ui);
    let response = ui.interact(rect, ui.id().with(hover_text), Sense::click());

    if response.is_pointer_button_down_on() {
        ui.painter()
            .rect_filled(rect, 0.0, caption_pressed_fill_for(ui, kind));
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, 0.0, caption_hover_fill_for(ui, kind));
    }

    let icon_rect = Rect::from_center_size(rect.center(), egui::vec2(icon_size, icon_size));
    ui.put(
        icon_rect,
        icon_image(
            icon,
            icon_size,
            caption_icon_color(ui, kind, response.hovered()),
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

fn caption_icon_color(ui: &Ui, kind: CaptionButtonKind, hovered: bool) -> Color32 {
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
