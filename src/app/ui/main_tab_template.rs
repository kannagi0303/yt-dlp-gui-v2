use eframe::egui::Ui;
use egui_taffy::{Tui, TuiBuilderLogic as _};

use crate::app::state::{AppMode, AppState};
use crate::app::widgets::url_input::app_textbox_single_line_height;

use super::item_card::render_batch_list;
use super::main_tab_controls::{MainTabButtonRole, MainTabControls, MainTabTextBoxRole};
use super::main_tab_music_panel::MusicPlayerPanel;
use super::main_tab_output_actions::DownloadContainerPicker;
use super::single_mode::build_single_mode_item;
use super::xaml_template_renderer::{TemplateBlockSlot, show_full_height_page_template};
use super::xaml_ui_nodes::{auto, block, fill, gap, rows};
use super::{UiRenderResources, semantic_ui_metrics, xaml_taffy_styles, xaml_ui_nodes};

type MainTabTemplateTree = xaml_ui_nodes::TemplateNode<MainTabNode>;

pub(super) fn render_main_tab(
    ui: &mut Ui,
    state: &mut AppState,
    render_resources: &UiRenderResources,
) {
    let template = MainTabTemplate::resolve(ui, state, render_resources);
    template.show(ui, state);
}

struct MainTabTemplate {
    context: MainTabTemplateContext,
    root: MainTabTemplateTree,
}

enum MainTabNode {
    UrlInputRow {
        url: MainTabTextBoxRole,
        monitor_toggle: MainTabButtonRole,
        start: MainTabButtonRole,
    },
    MainContent {
        music_player_panel: Option<MusicPlayerPanel>,
    },
    OutputRow {
        target_select: MainTabButtonRole,
        path: MainTabTextBoxRole,
        download_container: Option<DownloadContainerPicker>,
        download: MainTabButtonRole,
    },
}

#[derive(Debug, Clone, Copy)]
struct MainTabTemplateContext {
    row: xaml_taffy_styles::XamlSingleLineRowLayout,
    row_height: f32,
    section_spacing: f32,
    content_output_gap: f32,
    bottom_trailing_spacing: f32,
}

impl MainTabTemplate {
    fn resolve(ui: &mut Ui, state: &mut AppState, render_resources: &UiRenderResources) -> Self {
        let context = MainTabTemplateContext::resolve(ui, state);
        let controls = MainTabControls::resolve(ui, state, context.row);
        let music_player_panel = MusicPlayerPanel::resolve(
            ui,
            state,
            context.row_height,
            render_resources.music_player_aura(),
        );
        let root = main_tab_root_template(context, controls, music_player_panel);

        Self { context, root }
    }

    fn show(self, ui: &mut Ui, state: &mut AppState) {
        let context = self.context;
        let mut show_block = |slot, node, tui: &mut Tui| {
            show_main_tab_block(slot, node, tui, state, context);
        };
        show_full_height_page_template(ui, "main-tab-template", self.root, &mut show_block);
    }
}

impl MainTabTemplateContext {
    fn resolve(ui: &Ui, state: &AppState) -> Self {
        let row_height = ui.spacing().interact_size.y;
        let section_spacing = semantic_ui_metrics::main_section_vertical_spacing();
        let content_output_gap = if state.app_mode() == AppMode::Origin {
            semantic_ui_metrics::main_content_to_output_vertical_spacing_for_origin_mode()
        } else {
            section_spacing
        };
        let bottom_trailing_spacing = semantic_ui_metrics::main_bottom_trailing_vertical_spacing();

        Self {
            row: main_single_line_row_layout(ui, row_height),
            row_height,
            section_spacing,
            content_output_gap,
            bottom_trailing_spacing,
        }
    }
}

fn main_tab_root_template(
    context: MainTabTemplateContext,
    controls: MainTabControls,
    music_player_panel: Option<MusicPlayerPanel>,
) -> MainTabTemplateTree {
    let MainTabControls {
        url,
        monitor_toggle,
        start,
        target_select,
        path,
        download_container,
        download,
    } = controls;

    let mut root_rows = vec![
        auto(block(MainTabNode::UrlInputRow {
            url,
            monitor_toggle,
            start,
        })),
        gap(context.section_spacing),
        fill(block(MainTabNode::MainContent { music_player_panel })),
        gap(context.content_output_gap),
    ];

    root_rows.push(auto(block(MainTabNode::OutputRow {
        target_select,
        path,
        download_container,
        download,
    })));

    if context.bottom_trailing_spacing > 0.0 {
        root_rows.push(gap(context.bottom_trailing_spacing));
    }

    rows(root_rows)
}

fn show_main_tab_block(
    slot: TemplateBlockSlot,
    node: MainTabNode,
    tui: &mut Tui,
    state: &mut AppState,
    context: MainTabTemplateContext,
) {
    match node {
        MainTabNode::UrlInputRow {
            url,
            monitor_toggle,
            start,
        } => show_main_tab_single_line_row_block(slot, tui, context, |tui| {
            show_main_tab_url_input_row(url, monitor_toggle, start, tui, state);
        }),
        MainTabNode::MainContent { music_player_panel } => {
            show_main_tab_content(slot, tui, state, context, music_player_panel)
        }
        MainTabNode::OutputRow {
            target_select,
            path,
            download_container,
            download,
        } => show_main_tab_single_line_row_block(slot, tui, context, |tui| {
            show_main_tab_output_row(
                target_select,
                path,
                download_container,
                download,
                tui,
                state,
            );
        }),
    }
}

fn show_main_tab_single_line_row_block(
    slot: TemplateBlockSlot,
    tui: &mut Tui,
    context: MainTabTemplateContext,
    add_contents: impl FnOnce(&mut Tui),
) {
    match slot {
        TemplateBlockSlot::Root => add_contents(tui),
        TemplateBlockSlot::Child { .. } => tui.style(context.row.style()).add(add_contents),
    }
}

fn show_main_tab_url_input_row(
    url: MainTabTextBoxRole,
    monitor_toggle: MainTabButtonRole,
    start: MainTabButtonRole,
    tui: &mut Tui,
    state: &mut AppState,
) {
    url.show(tui, state);
    monitor_toggle.show(tui, state);
    start.show(tui, state);
}

fn show_main_tab_output_row(
    target_select: MainTabButtonRole,
    path: MainTabTextBoxRole,
    download_container: Option<DownloadContainerPicker>,
    download: MainTabButtonRole,
    tui: &mut Tui,
    state: &mut AppState,
) {
    target_select.show(tui, state);
    path.show(tui, state);
    if let Some(download_container) = download_container {
        download_container.show(tui, state);
    }
    download.show(tui, state);
}

fn show_main_tab_content(
    slot: TemplateBlockSlot,
    tui: &mut Tui,
    state: &mut AppState,
    context: MainTabTemplateContext,
    music_player_panel: Option<MusicPlayerPanel>,
) {
    match slot {
        TemplateBlockSlot::Root => {
            show_main_tab_content_layers(tui, state, context, music_player_panel);
        }
        TemplateBlockSlot::Child { style, .. } => {
            tui.style(style).add(|tui| {
                show_main_tab_content_layers(tui, state, context, music_player_panel);
            });
        }
    }
}

fn show_main_tab_content_layers(
    tui: &mut Tui,
    state: &mut AppState,
    context: MainTabTemplateContext,
    music_player_panel: Option<MusicPlayerPanel>,
) {
    let overlay_height = music_player_panel
        .as_ref()
        .map(MusicPlayerPanel::height)
        .unwrap_or(0.0);
    let bottom_safe_area =
        overlay_height + f32::from(overlay_height > 0.0) * context.section_spacing;

    tui.style(xaml_taffy_styles::xaml_overlay_host_style())
        .add(|tui| {
            tui.style(xaml_taffy_styles::xaml_overlay_content_style())
                .add(|tui| {
                    show_main_tab_content_body(tui, state, context, bottom_safe_area);
                });
            if let Some(music_player_panel) = music_player_panel {
                tui.style(xaml_taffy_styles::xaml_bottom_overlay_style(overlay_height))
                    .add(|tui| music_player_panel.show(tui, state));
            }
        });
}

fn show_main_tab_content_body(
    tui: &mut Tui,
    state: &mut AppState,
    context: MainTabTemplateContext,
    bottom_safe_area: f32,
) {
    match state.app_mode() {
        AppMode::Origin => build_single_mode_item(tui, state, context.row_height),
        _ => {
            xaml_taffy_styles::XamlTaffyElement::grow_block()
                .show_fill_ui(tui, |ui| render_batch_list(ui, state, bottom_safe_area));
        }
    }
}

fn main_single_line_row_layout(
    ui: &Ui,
    row_height: f32,
) -> xaml_taffy_styles::XamlSingleLineRowLayout {
    let control_height = app_textbox_single_line_height(ui).max(row_height);
    let row_contract =
        semantic_ui_metrics::xaml_single_line_control_row_contract_from_height(control_height);
    xaml_taffy_styles::XamlSingleLineRowLayout::new(row_contract)
        .with_column_gap(main_inline_control_gap(ui))
}

fn main_inline_control_gap(ui: &Ui) -> f32 {
    semantic_ui_metrics::main_inline_control_gap_from_current_spacing(ui)
}
