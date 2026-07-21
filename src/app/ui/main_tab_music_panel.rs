use eframe::egui::{self, Color32, Ui};
use egui_taffy::Tui;

use super::main_tab_music_aura::{MusicPlayerAuraRenderer, render_music_player_aura_at};
use super::main_tab_music_controls::row_music_player;
use super::main_tab_music_lyrics::render_music_lyrics_at;
use super::xaml_rect_template::{show_rect_template, show_ui_at_rect};
use super::xaml_ui_nodes::{TemplateAxis, TemplateNode, auto, block, gap, rows};
use super::{semantic_ui_metrics, xaml_taffy_styles};
use crate::app::state::{AppState, MusicLyricsDisplayLine, MusicPlayerAuraDisplay};

pub(super) struct MusicPlayerPanel {
    panel: xaml_taffy_styles::XamlTaffyElement,
    height: f32,
    lyrics_line: Option<MusicLyricsDisplayLine>,
    lyrics_row_height: f32,
    player_row_height: f32,
    aura_renderer: Option<MusicPlayerAuraRenderer>,
}

impl MusicPlayerPanel {
    pub(super) fn resolve(
        ui: &mut Ui,
        state: &mut AppState,
        player_row_height: f32,
        aura_renderer: Option<MusicPlayerAuraRenderer>,
    ) -> Option<Self> {
        if !state.music_player_visible() {
            return None;
        }

        let player_control_row_height =
            semantic_ui_metrics::main_music_player_control_row_height(player_row_height);
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
        let player_panel_height = semantic_ui_metrics::main_music_player_height_from_control_row(
            player_control_row_height,
        );
        let music_panel_height = semantic_ui_metrics::main_music_panel_height_for_content(
            player_panel_height,
            (lyrics_row_height > 0.0).then_some(lyrics_row_height),
        );

        Some(Self {
            panel: xaml_taffy_styles::XamlTaffyElement::fixed_height_block(music_panel_height),
            height: music_panel_height,
            lyrics_line,
            lyrics_row_height,
            player_row_height: player_panel_height,
            aura_renderer,
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
            aura_renderer,
        } = self;

        panel.show_ui(tui, |ui| {
            row_music_player_panel(
                ui,
                state,
                lyrics_line,
                lyrics_row_height,
                player_row_height,
                aura_renderer.as_ref(),
            );
        });
    }
}

fn row_music_player_panel(
    ui: &mut Ui,
    state: &mut AppState,
    lyrics_line: Option<MusicLyricsDisplayLine>,
    lyrics_row_height: f32,
    player_row_height: f32,
    aura_renderer: Option<&MusicPlayerAuraRenderer>,
) {
    let panel_frame = MusicPlayerPanelFrame::allocate(ui);
    let aura_display = state.music_player_aura_display();
    panel_frame.paint(ui, aura_renderer, aura_display);

    ui.scope_builder(
        egui::UiBuilder::new().max_rect(panel_frame.content_rect),
        |ui| {
            show_music_panel_template(
                ui,
                state,
                music_panel_template(
                    lyrics_line,
                    lyrics_row_height,
                    player_row_height,
                    aura_display,
                ),
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

    fn paint(
        &self,
        ui: &Ui,
        aura_renderer: Option<&MusicPlayerAuraRenderer>,
        aura_display: MusicPlayerAuraDisplay,
    ) {
        let rounding = semantic_ui_metrics::main_music_panel_corner_radius();
        ui.painter()
            .rect_filled(self.panel_rect, rounding, music_player_panel_fill(ui));
        render_music_player_aura_at(ui, self.panel_rect, aura_renderer, aura_display, rounding);
        ui.painter().rect_stroke(
            self.panel_rect,
            rounding,
            ui.visuals().widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Inside,
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
    if ui.visuals().dark_mode {
        // True-black glass: the color stays neutral black while a restrained
        // amount of queue content remains visible through the floating panel.
        Color32::from_rgba_unmultiplied(0, 0, 0, 196)
    } else {
        let base = ui.visuals().panel_fill;
        Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 208)
    }
}

type MusicPanelTemplate = TemplateNode<MusicPanelNode>;

enum MusicPanelNode {
    Lyrics {
        line: MusicLyricsDisplayLine,
        height: f32,
        aura_display: MusicPlayerAuraDisplay,
    },
    Player {
        height: f32,
        aura_display: MusicPlayerAuraDisplay,
    },
}

impl MusicPanelNode {
    fn height(&self) -> f32 {
        match self {
            Self::Lyrics { height, .. } | Self::Player { height, .. } => height.max(1.0),
        }
    }

    fn show_at(self, ui: &mut Ui, state: &mut AppState, rect: egui::Rect) {
        show_ui_at_rect(ui, rect, |ui| match self {
            Self::Lyrics {
                line, aura_display, ..
            } => {
                render_music_lyrics_at(ui, music_player_cell_rect(ui), &line, aura_display);
            }
            Self::Player { aura_display, .. } => row_music_player(ui, state, aura_display),
        });
    }
}

fn music_panel_template(
    lyrics_line: Option<MusicLyricsDisplayLine>,
    lyrics_row_height: f32,
    player_row_height: f32,
    aura_display: MusicPlayerAuraDisplay,
) -> MusicPanelTemplate {
    let mut children = Vec::new();

    if let Some(line) = lyrics_line {
        children.push(gap(
            semantic_ui_metrics::main_music_control_to_lyrics_vertical_spacing(),
        ));
        children.push(auto(block(MusicPanelNode::Lyrics {
            line,
            height: lyrics_row_height,
            aura_display,
        })));
    }

    children.push(gap(
        semantic_ui_metrics::main_music_control_to_lyrics_vertical_spacing(),
    ));
    children.push(auto(block(MusicPanelNode::Player {
        height: player_row_height,
        aura_display,
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
