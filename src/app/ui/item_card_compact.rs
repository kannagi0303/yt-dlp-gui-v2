use eframe::egui::{Align, Ui};

use crate::app::state::{AppState, ItemTitleVisualState, ThumbnailRenderSource};
use crate::domain::{CompactMusicState, QueueItemId};

use super::common::UiText;
use super::compact_row::{CompactRowSpec, CompactRowVisualState, render_music_compact_row};

pub(super) fn render_music_queue_item_row(
    ui: &mut Ui,
    state: &mut AppState,
    index: usize,
) -> Option<QueueItemId> {
    let item = &state.queue_items[index];
    let item_id = item.id;
    let title = item.title.clone();
    let id_salt = item.id;
    let duration_text = item.duration_text.clone();
    let music_state = item.compact_music_state.unwrap_or(CompactMusicState::Ready);
    let thumbnail_url = state.item_thumbnail_url(index).to_owned();
    let thumbnail_source = state.thumbnail_render_source_for_url(ui.ctx(), &thumbnail_url);
    let cache_progress = state.music_item_cache_progress_ratio(item_id);
    let row_progress = state.music_item_compact_progress_ratio(item_id);
    let show_row_progress = state.music_item_compact_progress_visible(item_id);
    let playback_progress = state.music_item_playback_progress_ratio(item_id);
    let is_current = state.music_current_item_id() == Some(item_id);
    let is_playing = state.music_item_is_playing(item_id);
    let (mut visual_state, mut status_text) = compact_music_visual_state(
        state,
        music_state,
        &duration_text,
        playback_progress,
        cache_progress,
    );

    if let Some(progress_text) = state.music_item_compact_progress_status_text(item_id) {
        status_text = progress_text;
    }
    if state.item_title_visual_state(index) == ItemTitleVisualState::Completed {
        visual_state = CompactRowVisualState::Downloaded;
        status_text = state
            .ui_i18n_text_for_key("music.status.completed")
            .to_owned();
    } else if state.music_item_has_complete_cache(item_id) {
        visual_state = CompactRowVisualState::Finished;
    }

    let output = render_music_compact_row(
        ui,
        CompactRowSpec {
            id_salt,
            title: &title,
            thumbnail_url: &thumbnail_url,
            thumbnail_source,
            status_text: &status_text,
            visual_state,
            progress: row_progress,
            show_progress: show_row_progress,
            is_current,
            is_playing,
            play_enabled: true,
            remove_enabled: true,
        },
    );

    if state.take_music_scroll_to_item_request(item_id) {
        output.response.scroll_to_me(Some(Align::Center));
    }
    if output.play_clicked {
        state.play_music_item(item_id);
    }
    ui.add_space(ui.spacing().item_spacing.y);

    output.remove_clicked.then_some(item_id)
}

fn compact_music_visual_state(
    app: &AppState,
    state: CompactMusicState,
    duration_text: &str,
    playback_progress: f32,
    cache_progress: f32,
) -> (CompactRowVisualState, String) {
    match state {
        CompactMusicState::Resolving => (
            CompactRowVisualState::Resolving,
            app.ui_i18n_text_for_key("music.status.resolving")
                .to_owned(),
        ),
        CompactMusicState::Buffering => {
            let label = if cache_progress > 0.0 {
                format!(
                    "{}%",
                    (cache_progress * 100.0).round().clamp(1.0, 99.0) as u32
                )
            } else {
                app.ui_i18n_text_for_key("music.status.buffering")
                    .to_owned()
            };
            (CompactRowVisualState::Resolving, label)
        }
        CompactMusicState::Ready => (
            if cache_progress >= 0.999 {
                CompactRowVisualState::Finished
            } else {
                CompactRowVisualState::Idle
            },
            if duration_text.trim().is_empty() {
                app.ui_i18n_text_for_key("music.status.ready").to_owned()
            } else {
                duration_text.to_owned()
            },
        ),
        CompactMusicState::Playing => (
            if cache_progress >= 0.999 {
                CompactRowVisualState::Finished
            } else {
                CompactRowVisualState::Playing {
                    progress: playback_progress,
                }
            },
            if cache_progress < 1.0 {
                app.ui_i18n_text_for_key("music.status.caching").to_owned()
            } else {
                app.ui_i18n_text_for_key("music.status.playing").to_owned()
            },
        ),
        CompactMusicState::Paused => (
            if cache_progress >= 0.999 {
                CompactRowVisualState::Finished
            } else {
                CompactRowVisualState::Paused {
                    progress: playback_progress,
                }
            },
            app.ui_i18n_text_for_key("music.status.paused").to_owned(),
        ),
        CompactMusicState::Failed => (
            CompactRowVisualState::Failed,
            app.ui_i18n_text_for_key("music.status.failed").to_owned(),
        ),
    }
}

pub(super) fn render_empty_music_compact_item(ui: &mut Ui, state: &AppState) {
    let title = state.ui_i18n_text_for_key("item.add_an_audio_url");
    let output = render_music_compact_row(
        ui,
        CompactRowSpec {
            id_salt: 0,
            title,
            thumbnail_url: "",
            thumbnail_source: ThumbnailRenderSource::None,
            status_text: state.ui_i18n_text_for_key(UiText::AUDIO),
            visual_state: CompactRowVisualState::Idle,
            progress: 0.0,
            show_progress: false,
            is_current: false,
            is_playing: false,
            play_enabled: false,
            remove_enabled: false,
        },
    );
    let _ = output;
}
