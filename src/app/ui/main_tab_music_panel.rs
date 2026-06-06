use eframe::egui::{self, Color32, Ui};
use egui_taffy::Tui;

use super::main_tab_music_controls::row_music_player;
use super::main_tab_music_lyrics::render_music_lyrics_at;
use super::xaml_rect_template::{show_rect_template, show_ui_at_rect};
use super::xaml_ui_nodes::{TemplateAxis, TemplateNode, auto, block, gap, rows};
use super::{semantic_ui_metrics, xaml_taffy_styles};
use crate::app::state::{AppState, MusicLyricsDisplayLine};

pub(super) struct MusicPlayerPanel {
    panel: xaml_taffy_styles::XamlTaffyElement,
    height: f32,
    lyrics_line: Option<MusicLyricsDisplayLine>,
    lyrics_row_height: f32,
    player_row_height: f32,
}

impl MusicPlayerPanel {
    pub(super) fn resolve(
        ui: &mut Ui,
        state: &mut AppState,
        player_row_height: f32,
    ) -> Option<Self> {
        if !state.music_player_visible() {
            return None;
        }

        let lyrics_line = state.music_current_lyrics_display();
        let lyrics_row_height = lyrics_line
            .as_ref()
            .map(|_| {
                semantic_ui_metrics::main_music_lyrics_row_height_from_current_text_metrics(
                    ui,
                    player_row_height,
                )
            })
            .unwrap_or(0.0);
        let music_panel_height = semantic_ui_metrics::main_music_panel_height_for_content(
            player_row_height,
            (lyrics_row_height > 0.0).then_some(lyrics_row_height),
        );

        Some(Self {
            panel: xaml_taffy_styles::XamlTaffyElement::fixed_height_block(music_panel_height),
            height: music_panel_height,
            lyrics_line,
            lyrics_row_height,
            player_row_height,
        })
    }

    pub(super) fn height(&self) -> f32 {
        self.height
    }

    pub(super) fn show(self, tui: &mut Tui, state: &mut AppState) {
        let Self {
            panel,
            height: _,
            lyrics_line,
            lyrics_row_height,
            player_row_height,
        } = self;

        panel.show_ui(tui, |ui| {
            row_music_player_panel(ui, state, lyrics_line, lyrics_row_height, player_row_height);
        });
    }
}

fn row_music_player_panel(
    ui: &mut Ui,
    state: &mut AppState,
    lyrics_line: Option<MusicLyricsDisplayLine>,
    lyrics_row_height: f32,
    player_row_height: f32,
) {
    let panel_frame = MusicPlayerPanelFrame::allocate(ui);
    panel_frame.paint(ui);

    ui.scope_builder(
        egui::UiBuilder::new().max_rect(panel_frame.content_rect),
        |ui| {
            show_music_panel_template(
                ui,
                state,
                music_panel_template(lyrics_line, lyrics_row_height, player_row_height),
            );
        },
    );
}

#[derive(Debug, Clone, Copy)]
struct MusicPlayerPanelFrame {
    panel_rect: egui::Rect,
    content_rect: egui::Rect,
}

impl MusicPlayerPanelFrame {
    fn allocate(ui: &mut Ui) -> Self {
        // The audio player may be shown immediately after restoring the audio-mode
        // playlist on startup. During those early/narrow layout frames, egui can
        // report very small available sizes. Keep every manually allocated rect
        // non-negative so restored audio state cannot panic the app.
        let panel_width = ui.available_width().max(1.0);
        let panel_height = ui.available_height().max(1.0);
        let (panel_rect, _) =
            ui.allocate_exact_size(egui::vec2(panel_width, panel_height), egui::Sense::hover());
        let content_rect = semantic_ui_metrics::main_music_panel_content_rect(panel_rect);

        Self {
            panel_rect,
            content_rect,
        }
    }

    fn paint(&self, ui: &Ui) {
        ui.painter().rect(
            self.panel_rect,
            7.0,
            music_player_panel_fill(ui),
            ui.visuals().widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Outside,
        );
    }
}

fn music_player_cell_rect(ui: &Ui) -> egui::Rect {
    let rect = ui.max_rect();
    egui::Rect::from_min_max(
        rect.min,
        egui::pos2(
            rect.right().max(rect.left() + 1.0),
            rect.bottom().max(rect.top() + 1.0),
        ),
    )
}

fn music_player_panel_fill(ui: &Ui) -> Color32 {
    let base = if ui.visuals().dark_mode {
        ui.visuals().extreme_bg_color
    } else {
        ui.visuals().panel_fill
    };
    let alpha = if ui.visuals().dark_mode { 196 } else { 224 };
    Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha)
}

type MusicPanelTemplate = TemplateNode<MusicPanelNode>;

enum MusicPanelNode {
    Lyrics {
        line: MusicLyricsDisplayLine,
        height: f32,
    },
    Player {
        height: f32,
    },
}

impl MusicPanelNode {
    fn height(&self) -> f32 {
        match self {
            Self::Lyrics { height, .. } | Self::Player { height } => height.max(1.0),
        }
    }

    fn show_at(self, ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
        show_ui_at_rect(ui, rect, |ui| match self {
            Self::Lyrics { line, .. } => {
                render_music_lyrics_at(ui, music_player_cell_rect(ui), &line);
            }
            Self::Player { .. } => row_music_player(ui, state),
        });
    }
}

fn music_panel_template(
    lyrics_line: Option<MusicLyricsDisplayLine>,
    lyrics_row_height: f32,
    player_row_height: f32,
) -> MusicPanelTemplate {
    let mut children = Vec::new();

    if let Some(line) = lyrics_line {
        children.push(auto(block(MusicPanelNode::Lyrics {
            line,
            height: lyrics_row_height,
        })));
        children.push(gap(
            semantic_ui_metrics::main_music_control_to_lyrics_vertical_spacing(),
        ));
    }

    children.push(auto(block(MusicPanelNode::Player {
        height: player_row_height,
    })));
    rows(children)
}

fn show_music_panel_template(ui: &mut Ui, state: &mut AppState, template: MusicPanelTemplate) {
    let panel_rect = music_player_cell_rect(ui);
    let mut auto_main_size = |node: &MusicPanelNode, axis: TemplateAxis, _cross_size: f32| {
        debug_assert_eq!(axis, TemplateAxis::Rows);
        node.height()
    };
    let mut show_block = |node: MusicPanelNode, rect: egui::Rect| {
        node.show_at(ui, state, rect);
    };

    show_rect_template(panel_rect, template, &mut auto_main_size, &mut show_block);
}
