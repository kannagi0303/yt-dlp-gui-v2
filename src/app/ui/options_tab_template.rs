use eframe::egui::{ScrollArea, Ui};
use egui_taffy::Tui;

use crate::app::state::AppState;

use super::common::settings_taffy_scroll_content;
use super::options_layout::OptionsLayoutMetrics;
use super::xaml_template_renderer::show_auto_height_tui_template;
use super::xaml_ui_nodes::{TemplateNode, auto_block_rows};

#[derive(Clone, Copy)]
enum OptionsRootNode {
    Language,
    ToolPaths,
    Behavior,
    Tabs,
    Playlist,
    FileAction,
    Cache,
    Window,
}

type OptionsRootTemplate = TemplateNode<OptionsRootNode>;

pub(super) fn render_options_root_page(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical()
        .id_salt("options-tab-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let metrics = OptionsLayoutMetrics::new(ui, state);
            settings_taffy_scroll_content(ui, "options-root-settings-taffy", |tui| {
                show_options_root_template(
                    options_root_template(),
                    tui,
                    state,
                    metrics.label_width,
                );
            });
        });
}

fn options_root_template() -> OptionsRootTemplate {
    auto_block_rows([
        OptionsRootNode::Language,
        OptionsRootNode::ToolPaths,
        OptionsRootNode::Behavior,
        OptionsRootNode::Tabs,
        OptionsRootNode::Playlist,
        OptionsRootNode::FileAction,
        OptionsRootNode::Cache,
        OptionsRootNode::Window,
    ])
}

fn show_options_root_template(
    template: OptionsRootTemplate,
    tui: &mut Tui,
    state: &mut AppState,
    label_width: f32,
) {
    let mut show_block = |node, tui: &mut Tui| {
        show_options_root_block(node, tui, state, label_width);
    };
    show_auto_height_tui_template(template, tui, &mut show_block);
}

fn show_options_root_block(
    node: OptionsRootNode,
    tui: &mut Tui,
    state: &mut AppState,
    label_width: f32,
) {
    match node {
        OptionsRootNode::Language => {
            super::options_language::render_language_group(tui, state, label_width)
        }
        OptionsRootNode::ToolPaths => {
            super::options_tool_paths::render_tool_paths_group(tui, state, label_width)
        }
        OptionsRootNode::Behavior => {
            super::options_behavior::render_behavior_group(tui, state, label_width)
        }
        OptionsRootNode::Tabs => {
            super::options_behavior::render_tabs_group(tui, state, label_width)
        }
        OptionsRootNode::Playlist => {
            super::options_behavior::render_playlist_group(tui, state, label_width)
        }
        OptionsRootNode::FileAction => {
            super::options_behavior::render_file_action_group(tui, state, label_width)
        }
        OptionsRootNode::Cache => super::options_cache::render_cache_group(tui, state, label_width),
        OptionsRootNode::Window => {
            super::options_window::render_window_group(tui, state, label_width)
        }
    }
}
