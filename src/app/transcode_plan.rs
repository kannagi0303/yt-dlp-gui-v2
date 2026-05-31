use crate::app::compatibility_profiles::{apply_profile_to_settings, profile_for};
use crate::app::post_process_worker::BuiltInTranscodeProfile;
use crate::infrastructure::{
    AudioPolicy, CompatibilityTarget, ContainerPolicy, ResolutionPolicy, SubtitlePolicy,
    TranscodeIntentMode, TranscodeIntentSettings, TranscodeSettingKey, VideoCodecPolicy,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TranscodeSupportLevel {
    Executable,
    Partial,
    PreviewOnly,
}

impl TranscodeSupportLevel {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Executable => "Executable",
            Self::Partial => "Partially supported",
            Self::PreviewOnly => "Preview only",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TranscodeAdjustmentControl {
    CompatibilityTarget,
    ResolutionPolicy,
    EncodeEffort,
    AudioPolicy,
    EncoderPolicy,
    PassPolicy,
    QualityTarget,
}

impl TranscodeAdjustmentControl {
    pub(super) fn key(self) -> TranscodeSettingKey {
        match self {
            Self::CompatibilityTarget => TranscodeSettingKey::CompatibilityTarget,
            Self::ResolutionPolicy => TranscodeSettingKey::ResolutionPolicy,
            Self::EncodeEffort => TranscodeSettingKey::EncodeEffort,
            Self::AudioPolicy => TranscodeSettingKey::AudioPolicy,
            Self::EncoderPolicy => TranscodeSettingKey::EncoderPolicy,
            Self::PassPolicy => TranscodeSettingKey::PassPolicy,
            Self::QualityTarget => TranscodeSettingKey::QualityTarget,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct TranscodePlan {
    pub summary: String,
    pub output: String,
    pub compatibility: String,
    pub speed: String,
    pub video: String,
    pub audio: String,
    pub container: String,
    pub encoder_status: String,
    pub command_preview: String,
    pub support_level: TranscodeSupportLevel,
    pub backend_profile: Option<BuiltInTranscodeProfile>,
    pub current_route: String,
    pub warnings: Vec<String>,
    pub conflicts: Vec<String>,
    pub current_adjustments: Vec<TranscodeAdjustmentControl>,
    pub lockable_keys: Vec<TranscodeSettingKey>,
    pub command_setting_keys: Vec<TranscodeSettingKey>,
    pub preview_only_setting_keys: Vec<TranscodeSettingKey>,
    pub disconnected_setting_keys: Vec<TranscodeSettingKey>,
}

impl TranscodePlan {
    pub(super) fn is_executable(&self) -> bool {
        self.support_level == TranscodeSupportLevel::Executable && self.backend_profile.is_some()
    }
}

pub(super) fn resolve_transcode_plan(settings: &TranscodeIntentSettings) -> TranscodePlan {
    let requires_processing = output_conversion_required(settings);
    let backend_profile = requires_processing.then_some(BuiltInTranscodeProfile::OutputConversion);
    let support_level = if requires_processing {
        TranscodeSupportLevel::Executable
    } else {
        TranscodeSupportLevel::PreviewOnly
    };
    let video = resolved_codec(settings).to_owned();
    let audio = resolved_audio(settings).to_owned();
    let container = resolved_container(settings).to_owned();

    TranscodePlan {
        summary: result_summary(settings),
        output: format!("{container} · {video} · {audio}"),
        compatibility: "Output conversion follows the selected tail-end format choices.".to_owned(),
        speed: speed_summary(settings),
        video,
        audio,
        container,
        encoder_status: speed_summary(settings),
        command_preview: command_preview_for(settings, backend_profile),
        support_level,
        backend_profile,
        current_route: current_route_summary(settings),
        warnings: warnings_for(settings),
        conflicts: conflicts_for(settings),
        current_adjustments: Vec::new(),
        lockable_keys: Vec::new(),
        command_setting_keys: command_setting_keys_for(backend_profile),
        preview_only_setting_keys: Vec::new(),
        disconnected_setting_keys: Vec::new(),
    }
}

pub(super) fn output_conversion_required(settings: &TranscodeIntentSettings) -> bool {
    settings.video_codec_policy != VideoCodecPolicy::Auto
        || settings.audio_policy != AudioPolicy::Auto
        || settings.container_policy != ContainerPolicy::Auto
        || subtitle_policy_requires_output(settings.subtitle_policy)
}

fn subtitle_policy_requires_output(policy: SubtitlePolicy) -> bool {
    !matches!(policy, SubtitlePolicy::Preserve)
}

pub(super) fn apply_intent_patch(
    settings: &mut TranscodeIntentSettings,
    intent: TranscodeIntentMode,
) {
    settings.intent_mode = intent;
}

pub(super) fn apply_compatibility_patch(
    settings: &mut TranscodeIntentSettings,
    target: CompatibilityTarget,
) {
    settings.compatibility_target = target;

    if let Some(profile) = profile_for(target) {
        apply_profile_to_settings(settings, profile);
        return;
    }

    match target {
        CompatibilityTarget::MostDevices | CompatibilityTarget::OldDevice => {
            settings.video_codec_policy = VideoCodecPolicy::H264;
            settings.container_policy = ContainerPolicy::Mp4;
            settings.audio_policy = AudioPolicy::Aac;
            settings.resolution_policy = ResolutionPolicy::Max1080p;
        }
        CompatibilityTarget::Apple | CompatibilityTarget::Mac => {
            settings.video_codec_policy = VideoCodecPolicy::Hevc;
            settings.container_policy = ContainerPolicy::Mp4;
            settings.audio_policy = AudioPolicy::Aac;
        }
        CompatibilityTarget::TvNas | CompatibilityTarget::Windows => {
            settings.video_codec_policy = VideoCodecPolicy::H264;
            settings.container_policy = ContainerPolicy::Mp4;
            settings.audio_policy = AudioPolicy::Aac;
        }
        CompatibilityTarget::AppleTvLegacy
        | CompatibilityTarget::AppleTvModern
        | CompatibilityTarget::IphoneIpad
        | CompatibilityTarget::AndroidTv
        | CompatibilityTarget::AndroidPhoneTablet
        | CompatibilityTarget::BrowserMp4 => {}
    }
}

fn command_preview_for(
    settings: &TranscodeIntentSettings,
    backend_profile: Option<BuiltInTranscodeProfile>,
) -> String {
    if backend_profile.is_none() {
        return "No post-process command will run because all conversion choices are set to source.".to_owned();
    }

    let video = match settings.video_codec_policy {
        VideoCodecPolicy::Auto => "-c:v copy".to_owned(),
        VideoCodecPolicy::H264 => "-c:v h264_nvenc|h264_qsv|h264_amf|libx264".to_owned(),
        VideoCodecPolicy::Hevc => "-c:v hevc_nvenc|hevc_qsv|hevc_amf|libx265".to_owned(),
        VideoCodecPolicy::Av1 => "-c:v av1_nvenc|av1_qsv|av1_amf|libsvtav1|libaom-av1".to_owned(),
    };
    let audio = match settings.audio_policy {
        AudioPolicy::Auto => "-c:a copy".to_owned(),
        AudioPolicy::Aac => "-c:a aac -b:a 192k".to_owned(),
        AudioPolicy::Opus => "-c:a libopus -b:a 160k".to_owned(),
        AudioPolicy::Flac => "-c:a flac".to_owned(),
    };
    let container = match settings.container_policy {
        ContainerPolicy::Auto => "same container".to_owned(),
        ContainerPolicy::Mp4 => "OUTPUT.mp4".to_owned(),
        ContainerPolicy::Mkv => "OUTPUT.mkv".to_owned(),
        ContainerPolicy::Mov => "OUTPUT.mov".to_owned(),
    };

    format!("ffmpeg -hide_banner -y -i INPUT -map 0:v:0? -map 0:a:0? {video} {audio} {container}")
}

fn resolved_codec(settings: &TranscodeIntentSettings) -> &'static str {
    match settings.video_codec_policy {
        VideoCodecPolicy::Auto => "Source",
        VideoCodecPolicy::H264 => "H.264",
        VideoCodecPolicy::Hevc => "HEVC",
        VideoCodecPolicy::Av1 => "AV1",
    }
}

fn resolved_audio(settings: &TranscodeIntentSettings) -> &'static str {
    match settings.audio_policy {
        AudioPolicy::Auto => "Source",
        AudioPolicy::Aac => "AAC",
        AudioPolicy::Opus => "Opus",
        AudioPolicy::Flac => "FLAC",
    }
}

fn resolved_container(settings: &TranscodeIntentSettings) -> &'static str {
    match settings.container_policy {
        ContainerPolicy::Auto => "Source",
        ContainerPolicy::Mp4 => "MP4",
        ContainerPolicy::Mkv => "MKV",
        ContainerPolicy::Mov => "MOV",
    }
}

fn result_summary(settings: &TranscodeIntentSettings) -> String {
    if output_conversion_required(settings) {
        "Convert after download; keep the picture visually close to the source.".to_owned()
    } else {
        "Keep downloaded output as-is.".to_owned()
    }
}

fn speed_summary(_settings: &TranscodeIntentSettings) -> String {
    "Auto hardware encoder when video is re-encoded; software fallback if needed".to_owned()
}

fn warnings_for(settings: &TranscodeIntentSettings) -> Vec<String> {
    let mut warnings = Vec::new();
    if settings.video_codec_policy == VideoCodecPolicy::Av1 {
        warnings.push(
            "AV1 depends on available FFmpeg encoders and may be slow without hardware support."
                .to_owned(),
        );
    }
    if settings.container_policy == ContainerPolicy::Mp4
        && settings.audio_policy == AudioPolicy::Flac
    {
        warnings.push(
            "MP4 + FLAC is not a safe compatibility pair; MKV or MOV is usually safer.".to_owned(),
        );
    }
    warnings
}

fn conflicts_for(settings: &TranscodeIntentSettings) -> Vec<String> {
    let mut conflicts = Vec::new();
    if settings.container_policy == ContainerPolicy::Mp4
        && settings.audio_policy == AudioPolicy::Opus
    {
        conflicts.push("MP4 + Opus may not play everywhere. MKV is safer for Opus.".to_owned());
    }
    conflicts
}

fn command_setting_keys_for(
    backend_profile: Option<BuiltInTranscodeProfile>,
) -> Vec<TranscodeSettingKey> {
    if backend_profile.is_some() {
        vec![
            TranscodeSettingKey::VideoCodecPolicy,
            TranscodeSettingKey::AudioPolicy,
            TranscodeSettingKey::ContainerPolicy,
        ]
    } else {
        Vec::new()
    }
}

fn current_route_summary(settings: &TranscodeIntentSettings) -> String {
    if output_conversion_required(settings) {
        "Download -> Post-process output conversion".to_owned()
    } else {
        "Download -> Keep source output".to_owned()
    }
}
