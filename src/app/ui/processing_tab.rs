use eframe::egui::{self, ScrollArea, Ui};

use crate::app::state::AppState;
use crate::infrastructure::{
    AudioPolicy, ContainerPolicy, SubtitlePolicy, TranscodeIntentSettings, VideoCodecPolicy,
};

use super::common::{form_row_label, measure_label_width, settings_scroll_content};

pub(super) fn render_log_tab(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical()
        .id_salt("log-tab-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            settings_scroll_content(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button(state.tr("log.clear")).clicked() {
                        state.runtime_log.clear();
                    }
                    if ui
                        .add_enabled(
                            !state.runtime_log.is_empty(),
                            egui::Button::new(state.tr("log.copy")),
                        )
                        .clicked()
                    {
                        let text = state
                            .runtime_log
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .join("\n");
                        ui.ctx().copy_text(text);
                    }
                });
                ui.add_space(6.0);

                if state.runtime_log.is_empty() {
                    ui.weak(state.tr("log.empty"));
                } else {
                    for entry in &state.runtime_log {
                        ui.label(entry);
                    }
                }
            });
        });
}

pub(super) fn render_processing_settings_content(ui: &mut Ui, state: &mut AppState) {
    // The enable switch lives in Advance > Post-processing; this page only edits conversion details.
    let mut settings = state.config.transcode_intent.clone();
    let before = settings.clone();

    render_post_download_conversion(ui, state, &mut settings);

    if before != settings {
        state.set_transcode_intent(settings);
    }
}

fn render_post_download_conversion(
    ui: &mut Ui,
    state: &mut AppState,
    settings: &mut TranscodeIntentSettings,
) {
    let labels = [
        state.tr("processing.video"),
        state.tr("processing.audio"),
        state.tr("processing.container"),
        state.tr("processing.subtitle"),
    ];
    let label_width = measure_label_width(ui, &labels);

    form_row_label(ui, label_width, state.tr("processing.video"), |ui| {
        render_video_codec_choices(ui, state, settings);
    });
    form_row_label(ui, label_width, state.tr("processing.audio"), |ui| {
        render_audio_choices(ui, state, settings);
    });
    form_row_label(ui, label_width, state.tr("processing.container"), |ui| {
        render_container_choices(ui, state, settings);
    });
    form_row_label(ui, label_width, state.tr("processing.subtitle"), |ui| {
        render_subtitle_choices(ui, state, settings);
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConversionField {
    Video,
    Audio,
    Container,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConversionCombination {
    video: VideoCodecPolicy,
    audio: AudioPolicy,
    container: ContainerPolicy,
}

impl ConversionCombination {
    fn from_settings(settings: &TranscodeIntentSettings) -> Self {
        Self {
            video: settings.video_codec_policy,
            audio: settings.audio_policy,
            container: settings.container_policy,
        }
    }

    fn apply_to(self, settings: &mut TranscodeIntentSettings) {
        settings.video_codec_policy = self.video;
        settings.audio_policy = self.audio;
        settings.container_policy = self.container;
    }

    fn is_allowed(self) -> bool {
        container_allowed_for_codecs(self.container, self.video, self.audio)
    }
}

fn render_video_codec_choices(
    ui: &mut Ui,
    state: &AppState,
    settings: &mut TranscodeIntentSettings,
) {
    let options = [
        (VideoCodecPolicy::Auto, state.tr("processing.choice.source")),
        (VideoCodecPolicy::H264, state.tr("processing.video.h264")),
        (VideoCodecPolicy::Hevc, state.tr("processing.video.hevc")),
        (VideoCodecPolicy::Av1, state.tr("processing.video.av1")),
    ];
    ui.horizontal_wrapped(|ui| {
        for (value, label) in options {
            let selected = settings.video_codec_policy == value;
            let compatible = value_is_currently_compatible(settings, ConversionField::Video, value);
            if choice_button(ui, selected, compatible, label).clicked() {
                let forced = if selected && value != VideoCodecPolicy::Auto {
                    VideoCodecPolicy::Auto
                } else {
                    value
                };
                force_video_choice(settings, forced);
            }
        }
    });
}

fn render_audio_choices(ui: &mut Ui, state: &AppState, settings: &mut TranscodeIntentSettings) {
    let options = [
        (AudioPolicy::Auto, state.tr("processing.choice.source")),
        (AudioPolicy::Aac, state.tr("processing.audio.aac")),
        (AudioPolicy::Opus, state.tr("processing.audio.opus")),
        (AudioPolicy::Flac, state.tr("processing.audio.flac")),
    ];
    ui.horizontal_wrapped(|ui| {
        for (value, label) in options {
            let selected = settings.audio_policy == value;
            let compatible = value_is_currently_compatible(settings, ConversionField::Audio, value);
            if choice_button(ui, selected, compatible, label).clicked() {
                let forced = if selected && value != AudioPolicy::Auto {
                    AudioPolicy::Auto
                } else {
                    value
                };
                force_audio_choice(settings, forced);
            }
        }
    });
}

fn render_container_choices(ui: &mut Ui, state: &AppState, settings: &mut TranscodeIntentSettings) {
    let options = [
        (ContainerPolicy::Auto, state.tr("processing.choice.source")),
        (ContainerPolicy::Mp4, state.tr("processing.container.mp4")),
        (ContainerPolicy::Mkv, state.tr("processing.container.mkv")),
        (ContainerPolicy::Mov, state.tr("processing.container.mov")),
    ];
    ui.horizontal_wrapped(|ui| {
        for (value, label) in options {
            let selected = settings.container_policy == value;
            let compatible =
                value_is_currently_compatible(settings, ConversionField::Container, value);
            if choice_button(ui, selected, compatible, label).clicked() {
                let forced = if selected && value != ContainerPolicy::Auto {
                    ContainerPolicy::Auto
                } else {
                    value
                };
                force_container_choice(settings, forced);
            }
        }
    });
}

fn render_subtitle_choices(ui: &mut Ui, state: &AppState, settings: &mut TranscodeIntentSettings) {
    let options = [
        (
            SubtitlePolicy::Preserve,
            state.tr("processing.subtitle.preserve"),
        ),
        (SubtitlePolicy::Embed, state.tr("processing.subtitle.embed")),
        (SubtitlePolicy::Burn, state.tr("processing.subtitle.burn")),
    ];
    ui.horizontal_wrapped(|ui| {
        for (value, label) in options {
            let selected = settings.subtitle_policy == value;
            if choice_button(ui, selected, true, label).clicked() {
                settings.subtitle_policy = if selected && value != SubtitlePolicy::Preserve {
                    SubtitlePolicy::Preserve
                } else {
                    value
                };
            }
        }
    });
}

fn choice_button(ui: &mut Ui, selected: bool, compatible: bool, label: &str) -> egui::Response {
    let mut button = egui::Button::new(label)
        .frame(true)
        .min_size(egui::vec2(0.0, ui.spacing().interact_size.y + 6.0));
    if !compatible && !selected {
        button = button.fill(incompatible_choice_fill(ui));
    }
    ui.add(button.selected(selected))
}

fn incompatible_choice_fill(ui: &Ui) -> egui::Color32 {
    if ui.visuals().dark_mode {
        egui::Color32::BLACK
    } else {
        ui.visuals().widgets.noninteractive.bg_fill
    }
}

trait ConversionChoiceValue: Copy + Eq {
    fn apply_to(self, combination: &mut ConversionCombination, field: ConversionField);
    fn matches_field(self, combination: ConversionCombination, field: ConversionField) -> bool;
}

impl ConversionChoiceValue for VideoCodecPolicy {
    fn apply_to(self, combination: &mut ConversionCombination, field: ConversionField) {
        if field == ConversionField::Video {
            combination.video = self;
        }
    }

    fn matches_field(self, combination: ConversionCombination, field: ConversionField) -> bool {
        field == ConversionField::Video && combination.video == self
    }
}

impl ConversionChoiceValue for AudioPolicy {
    fn apply_to(self, combination: &mut ConversionCombination, field: ConversionField) {
        if field == ConversionField::Audio {
            combination.audio = self;
        }
    }

    fn matches_field(self, combination: ConversionCombination, field: ConversionField) -> bool {
        field == ConversionField::Audio && combination.audio == self
    }
}

impl ConversionChoiceValue for ContainerPolicy {
    fn apply_to(self, combination: &mut ConversionCombination, field: ConversionField) {
        if field == ConversionField::Container {
            combination.container = self;
        }
    }

    fn matches_field(self, combination: ConversionCombination, field: ConversionField) -> bool {
        field == ConversionField::Container && combination.container == self
    }
}

fn value_is_currently_compatible<T>(
    settings: &TranscodeIntentSettings,
    field: ConversionField,
    value: T,
) -> bool
where
    T: ConversionChoiceValue,
{
    let mut combination = ConversionCombination::from_settings(settings);
    value.apply_to(&mut combination, field);
    combination.is_allowed()
}

fn force_video_choice(settings: &mut TranscodeIntentSettings, value: VideoCodecPolicy) {
    force_conversion_choice(settings, ConversionField::Video, value);
}

fn force_audio_choice(settings: &mut TranscodeIntentSettings, value: AudioPolicy) {
    force_conversion_choice(settings, ConversionField::Audio, value);
}

fn force_container_choice(settings: &mut TranscodeIntentSettings, value: ContainerPolicy) {
    force_conversion_choice(settings, ConversionField::Container, value);
}

fn force_conversion_choice<T>(
    settings: &mut TranscodeIntentSettings,
    field: ConversionField,
    value: T,
) where
    T: ConversionChoiceValue,
{
    let previous = ConversionCombination::from_settings(settings);
    let mut direct = previous;
    value.apply_to(&mut direct, field);
    if direct.is_allowed() {
        direct.apply_to(settings);
        return;
    }

    let Some(best) = conversion_combinations()
        .into_iter()
        .filter(|combination| value.matches_field(*combination, field))
        .max_by_key(|combination| conversion_choice_score(previous, *combination, field))
    else {
        direct.apply_to(settings);
        return;
    };

    best.apply_to(settings);
}

fn conversion_combinations() -> Vec<ConversionCombination> {
    const VIDEOS: [VideoCodecPolicy; 4] = [
        VideoCodecPolicy::Auto,
        VideoCodecPolicy::H264,
        VideoCodecPolicy::Hevc,
        VideoCodecPolicy::Av1,
    ];
    const AUDIOS: [AudioPolicy; 4] = [
        AudioPolicy::Auto,
        AudioPolicy::Aac,
        AudioPolicy::Opus,
        AudioPolicy::Flac,
    ];
    const CONTAINERS: [ContainerPolicy; 4] = [
        ContainerPolicy::Auto,
        ContainerPolicy::Mp4,
        ContainerPolicy::Mkv,
        ContainerPolicy::Mov,
    ];

    let mut combinations = Vec::new();
    for video in VIDEOS {
        for audio in AUDIOS {
            for container in CONTAINERS {
                let combination = ConversionCombination {
                    video,
                    audio,
                    container,
                };
                if combination.is_allowed() {
                    combinations.push(combination);
                }
            }
        }
    }
    combinations
}

fn conversion_choice_score(
    previous: ConversionCombination,
    candidate: ConversionCombination,
    forced_field: ConversionField,
) -> i64 {
    let mut score = 0;
    if forced_field != ConversionField::Video {
        score += score_video_choice(previous.video, candidate.video);
    }
    if forced_field != ConversionField::Audio {
        score += score_audio_choice(previous.audio, candidate.audio);
    }
    if forced_field != ConversionField::Container {
        score += score_container_choice(previous.container, candidate.container);
    }
    score
}

fn score_video_choice(previous: VideoCodecPolicy, candidate: VideoCodecPolicy) -> i64 {
    if previous == candidate {
        return 10_000;
    }
    match candidate {
        VideoCodecPolicy::H264 => 700,
        VideoCodecPolicy::Hevc => 650,
        VideoCodecPolicy::Av1 => 550,
        VideoCodecPolicy::Auto => 200,
    }
}

fn score_audio_choice(previous: AudioPolicy, candidate: AudioPolicy) -> i64 {
    if previous == candidate {
        return 10_000;
    }
    match candidate {
        AudioPolicy::Aac => 700,
        AudioPolicy::Opus => 600,
        AudioPolicy::Flac => 500,
        AudioPolicy::Auto => 200,
    }
}

fn score_container_choice(previous: ContainerPolicy, candidate: ContainerPolicy) -> i64 {
    if previous == candidate {
        return 10_000;
    }
    match candidate {
        ContainerPolicy::Mkv => 700,
        ContainerPolicy::Mp4 => 650,
        ContainerPolicy::Mov => 600,
        ContainerPolicy::Auto => 200,
    }
}

fn container_allowed_for_codecs(
    container: ContainerPolicy,
    video: VideoCodecPolicy,
    audio: AudioPolicy,
) -> bool {
    video_allowed_for_container(video, container) && audio_allowed_for_container(audio, container)
}

fn video_allowed_for_container(video: VideoCodecPolicy, container: ContainerPolicy) -> bool {
    match container {
        ContainerPolicy::Auto | ContainerPolicy::Mkv => true,
        ContainerPolicy::Mp4 => matches!(
            video,
            VideoCodecPolicy::Auto
                | VideoCodecPolicy::H264
                | VideoCodecPolicy::Hevc
                | VideoCodecPolicy::Av1
        ),
        ContainerPolicy::Mov => matches!(
            video,
            VideoCodecPolicy::Auto | VideoCodecPolicy::H264 | VideoCodecPolicy::Hevc
        ),
    }
}

fn audio_allowed_for_container(audio: AudioPolicy, container: ContainerPolicy) -> bool {
    match container {
        ContainerPolicy::Auto | ContainerPolicy::Mkv => true,
        ContainerPolicy::Mp4 | ContainerPolicy::Mov => {
            matches!(audio, AudioPolicy::Auto | AudioPolicy::Aac)
        }
    }
}
