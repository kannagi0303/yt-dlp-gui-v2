use crate::infrastructure::{
    AudioPolicy, CompatibilityTarget, ContainerPolicy, FrameRatePolicy, ResolutionPolicy,
    TranscodeIntentSettings, TranscodeSettingKey, VideoCodecPolicy,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompatibilityScope {
    AppleTv,
    AppleMobile,
    AndroidTv,
    AndroidMobile,
    Computer,
    Browser,
    OldTv,
}

impl CompatibilityScope {
    pub(crate) fn variants() -> [Self; 7] {
        [
            Self::AppleTv,
            Self::AppleMobile,
            Self::AndroidTv,
            Self::AndroidMobile,
            Self::Computer,
            Self::Browser,
            Self::OldTv,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CompatibilityProfile {
    pub target: CompatibilityTarget,
    pub scope: CompatibilityScope,
    pub label_key: &'static str,
    pub video: VideoCodecPolicy,
    pub container: ContainerPolicy,
    pub resolution: ResolutionPolicy,
    pub frame_rate: FrameRatePolicy,
    pub audio: AudioPolicy,
    pub max_summary_zh: &'static str,
    pub max_summary_en: &'static str,
}

pub(crate) const COMPATIBILITY_PROFILES: &[CompatibilityProfile] = &[
    CompatibilityProfile {
        target: CompatibilityTarget::AppleTvLegacy,
        scope: CompatibilityScope::AppleTv,
        label_key: "transcode.compat.apple_tv_legacy",
        video: VideoCodecPolicy::H264,
        container: ContainerPolicy::Mp4,
        resolution: ResolutionPolicy::Max1080p,
        frame_rate: FrameRatePolicy::Fps30,
        audio: AudioPolicy::Aac,
        max_summary_zh: "H.264 MP4 · 最高 1080p30 · AAC stereo",
        max_summary_en: "H.264 MP4 · up to 1080p30 · AAC stereo",
    },
    CompatibilityProfile {
        target: CompatibilityTarget::AppleTvModern,
        scope: CompatibilityScope::AppleTv,
        label_key: "transcode.compat.apple_tv_modern",
        video: VideoCodecPolicy::Hevc,
        container: ContainerPolicy::Mp4,
        resolution: ResolutionPolicy::AutoBalance,
        frame_rate: FrameRatePolicy::Fps60,
        audio: AudioPolicy::Aac,
        max_summary_zh: "HEVC MP4 · 最高 4K60 · AAC stereo",
        max_summary_en: "HEVC MP4 · up to 4K60 · AAC stereo",
    },
    CompatibilityProfile {
        target: CompatibilityTarget::IphoneIpad,
        scope: CompatibilityScope::AppleMobile,
        label_key: "transcode.compat.iphone_ipad",
        video: VideoCodecPolicy::Hevc,
        container: ContainerPolicy::Mp4,
        resolution: ResolutionPolicy::AutoBalance,
        frame_rate: FrameRatePolicy::Source,
        audio: AudioPolicy::Aac,
        max_summary_zh: "Apple 行動裝置友善 MP4 · HEVC/AAC",
        max_summary_en: "Apple mobile-friendly MP4 · HEVC/AAC",
    },
    CompatibilityProfile {
        target: CompatibilityTarget::AndroidTv,
        scope: CompatibilityScope::AndroidTv,
        label_key: "transcode.compat.android_tv",
        video: VideoCodecPolicy::Hevc,
        container: ContainerPolicy::Mp4,
        resolution: ResolutionPolicy::AutoBalance,
        frame_rate: FrameRatePolicy::Fps60,
        audio: AudioPolicy::Aac,
        max_summary_zh: "Android TV / Chromecast 友善 MP4 · HEVC 最高 4K60",
        max_summary_en: "Android TV / Chromecast-friendly MP4 · HEVC up to 4K60",
    },
    CompatibilityProfile {
        target: CompatibilityTarget::AndroidPhoneTablet,
        scope: CompatibilityScope::AndroidMobile,
        label_key: "transcode.compat.android_phone_tablet",
        video: VideoCodecPolicy::Hevc,
        container: ContainerPolicy::Mp4,
        resolution: ResolutionPolicy::AutoBalance,
        frame_rate: FrameRatePolicy::Source,
        audio: AudioPolicy::Aac,
        max_summary_zh: "Android 手機/平板友善 MP4 · HEVC/AAC",
        max_summary_en: "Android phone/tablet-friendly MP4 · HEVC/AAC",
    },
    CompatibilityProfile {
        target: CompatibilityTarget::Windows,
        scope: CompatibilityScope::Computer,
        label_key: "transcode.compat.windows",
        video: VideoCodecPolicy::H264,
        container: ContainerPolicy::Mp4,
        resolution: ResolutionPolicy::KeepOriginal,
        frame_rate: FrameRatePolicy::Source,
        audio: AudioPolicy::Aac,
        max_summary_zh: "Windows / 一般電腦友善 MP4 · H.264/AAC",
        max_summary_en: "Windows / general PC-friendly MP4 · H.264/AAC",
    },
    CompatibilityProfile {
        target: CompatibilityTarget::Mac,
        scope: CompatibilityScope::Computer,
        label_key: "transcode.compat.mac",
        video: VideoCodecPolicy::Hevc,
        container: ContainerPolicy::Mp4,
        resolution: ResolutionPolicy::KeepOriginal,
        frame_rate: FrameRatePolicy::Source,
        audio: AudioPolicy::Aac,
        max_summary_zh: "Mac 友善 MP4 · HEVC/AAC",
        max_summary_en: "Mac-friendly MP4 · HEVC/AAC",
    },
    CompatibilityProfile {
        target: CompatibilityTarget::BrowserMp4,
        scope: CompatibilityScope::Browser,
        label_key: "transcode.compat.browser_mp4",
        video: VideoCodecPolicy::H264,
        container: ContainerPolicy::Mp4,
        resolution: ResolutionPolicy::Max1080p,
        frame_rate: FrameRatePolicy::Fps60,
        audio: AudioPolicy::Aac,
        max_summary_zh: "瀏覽器安全 MP4 · H.264/AAC · 最高 1080p60",
        max_summary_en: "Browser-safe MP4 · H.264/AAC · up to 1080p60",
    },
    CompatibilityProfile {
        target: CompatibilityTarget::OldDevice,
        scope: CompatibilityScope::OldTv,
        label_key: "transcode.compat.old_device",
        video: VideoCodecPolicy::H264,
        container: ContainerPolicy::Mp4,
        resolution: ResolutionPolicy::Max720p,
        frame_rate: FrameRatePolicy::Fps30,
        audio: AudioPolicy::Aac,
        max_summary_zh: "舊款電視/USB 播放友善 MP4 · H.264 720p30",
        max_summary_en: "Old TV / USB playback-friendly MP4 · H.264 720p30",
    },
    CompatibilityProfile {
        target: CompatibilityTarget::TvNas,
        scope: CompatibilityScope::OldTv,
        label_key: "transcode.compat.tv_nas",
        video: VideoCodecPolicy::H264,
        container: ContainerPolicy::Mp4,
        resolution: ResolutionPolicy::Max1080p,
        frame_rate: FrameRatePolicy::Fps30,
        audio: AudioPolicy::Aac,
        max_summary_zh: "一般電視/NAS 友善 MP4 · H.264 1080p30",
        max_summary_en: "Generic TV/NAS-friendly MP4 · H.264 1080p30",
    },
];

pub(crate) fn profile_for(target: CompatibilityTarget) -> Option<&'static CompatibilityProfile> {
    COMPATIBILITY_PROFILES
        .iter()
        .find(|profile| profile.target == target)
}

pub(crate) fn profiles_for_scope(
    scope: CompatibilityScope,
) -> impl Iterator<Item = &'static CompatibilityProfile> {
    COMPATIBILITY_PROFILES
        .iter()
        .filter(move |profile| profile.scope == scope)
}

pub(crate) fn scope_for_target(target: CompatibilityTarget) -> Option<CompatibilityScope> {
    profile_for(target).map(|profile| profile.scope)
}

pub(crate) fn apply_profile_to_settings(
    settings: &mut TranscodeIntentSettings,
    profile: &CompatibilityProfile,
) {
    settings.compatibility_target = profile.target;

    if !settings.is_locked(TranscodeSettingKey::VideoCodecPolicy) {
        settings.video_codec_policy = profile.video;
    }
    if !settings.is_locked(TranscodeSettingKey::ContainerPolicy) {
        settings.container_policy = profile.container;
    }
    if !settings.is_locked(TranscodeSettingKey::ResolutionPolicy) {
        settings.resolution_policy = profile.resolution;
    }
    if !settings.is_locked(TranscodeSettingKey::FrameRatePolicy) {
        settings.frame_rate_policy = profile.frame_rate;
    }
    if !settings.is_locked(TranscodeSettingKey::AudioPolicy) {
        settings.audio_policy = profile.audio;
    }
}

pub(crate) fn scope_label(scope: CompatibilityScope, cjk: bool) -> &'static str {
    match scope {
        CompatibilityScope::AppleTv => {
            if cjk {
                "Apple TV"
            } else {
                "Apple TV"
            }
        }
        CompatibilityScope::AppleMobile => {
            if cjk {
                "iPhone/iPad"
            } else {
                "iPhone/iPad"
            }
        }
        CompatibilityScope::AndroidTv => {
            if cjk {
                "Android TV"
            } else {
                "Android TV"
            }
        }
        CompatibilityScope::AndroidMobile => {
            if cjk {
                "Android 手機"
            } else {
                "Android mobile"
            }
        }
        CompatibilityScope::Computer => {
            if cjk {
                "電腦"
            } else {
                "Computer"
            }
        }
        CompatibilityScope::Browser => {
            if cjk {
                "瀏覽器"
            } else {
                "Browser"
            }
        }
        CompatibilityScope::OldTv => {
            if cjk {
                "舊款電視"
            } else {
                "Old TV"
            }
        }
    }
}
