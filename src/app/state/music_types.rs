const MUSIC_ORIGINAL_AUTO_SELECTOR: &str = "bestaudio/best[acodec!=none]";
const MUSIC_ORIGINAL_MP3_SELECTOR: &str =
    "bestaudio[ext=mp3]/bestaudio[acodec^=mp3]/bestaudio/best[acodec!=none]";
const MUSIC_ORIGINAL_AAC_SELECTOR: &str = "bestaudio[ext=m4a]/bestaudio[acodec^=mp4a]/bestaudio[acodec^=aac]/bestaudio/best[acodec!=none]";
const MUSIC_ORIGINAL_OPUS_SELECTOR: &str = "bestaudio[acodec^=opus]/bestaudio[ext=opus]/bestaudio[ext=webm][acodec^=opus]/bestaudio/best[acodec!=none]";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicPlaybackMode {
    Sequential,
    RepeatAll,
    Shuffle,
    RepeatOne,
}

impl MusicPlaybackMode {
    pub const ALL: [Self; 4] = [
        Self::Sequential,
        Self::RepeatAll,
        Self::Shuffle,
        Self::RepeatOne,
    ];

    pub(crate) fn label_key(self) -> &'static str {
        match self {
            Self::Sequential => "Sequence",
            Self::RepeatAll => "Repeat",
            Self::Shuffle => "Shuffle",
            Self::RepeatOne => "Repeat one",
        }
    }

    pub(crate) fn config_value(self) -> &'static str {
        match self {
            Self::Sequential => "sequential",
            Self::RepeatAll => "repeat_all",
            Self::Shuffle => "shuffle",
            Self::RepeatOne => "repeat_one",
        }
    }

    pub(crate) fn from_config_value(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "repeat_all" | "repeat" | "loop" => Self::RepeatAll,
            "shuffle" | "random" => Self::Shuffle,
            "repeat_one" | "single" | "one" => Self::RepeatOne,
            _ => Self::Sequential,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicMixMode {
    Off,
    FullSong,
    SkipQuietEdges,
    Highlight,
}

impl MusicMixMode {
    pub const ALL: [Self; 4] = [
        Self::Off,
        Self::FullSong,
        Self::SkipQuietEdges,
        Self::Highlight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::FullSong => "Full song",
            Self::SkipQuietEdges => "Skip quiet edges",
            Self::Highlight => "Highlight",
        }
    }

    pub fn enabled(self) -> bool {
        self != Self::Off
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicDownloadFormat {
    Mp3,
    M4aAac,
    Opus,
    Flac,
    Wav,
}

impl MusicDownloadFormat {
    pub const SIMPLE_OUTPUTS: [Self; 3] = [Self::Mp3, Self::M4aAac, Self::Opus];

    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::M4aAac => "m4a",
            Self::Opus => "opus",
            Self::Flac => "flac",
            Self::Wav => "wav",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Mp3 => "MP3",
            Self::M4aAac => "AAC",
            Self::Opus => "Opus",
            Self::Flac => "FLAC",
            Self::Wav => "WAV",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicDownloadMode {
    Original,
    Unified,
}

impl MusicDownloadMode {
    pub const ALL: [Self; 2] = [Self::Original, Self::Unified];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Original => "Original file",
            Self::Unified => "Unified format",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicOriginalPreference {
    Auto,
    PreferMp3,
    PreferAac,
    PreferOpus,
}

impl MusicOriginalPreference {
    pub const ALL: [Self; 4] = [
        Self::Auto,
        Self::PreferOpus,
        Self::PreferAac,
        Self::PreferMp3,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Auto => "Best",
            Self::PreferMp3 => "MP3",
            Self::PreferAac => "AAC",
            Self::PreferOpus => "Opus",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MusicDownloadChoice {
    pub mode: MusicDownloadMode,
    pub original_preference: MusicOriginalPreference,
    pub unified_format: MusicDownloadFormat,
    pub embed_cover: bool,
    pub write_tags: bool,
}

impl Default for MusicDownloadChoice {
    fn default() -> Self {
        Self {
            mode: MusicDownloadMode::Original,
            original_preference: MusicOriginalPreference::Auto,
            unified_format: MusicDownloadFormat::M4aAac,
            embed_cover: true,
            write_tags: true,
        }
    }
}

impl MusicDownloadChoice {
    pub(crate) fn target_format(self) -> Option<MusicDownloadFormat> {
        match self.mode {
            MusicDownloadMode::Original => None,
            MusicDownloadMode::Unified => Some(self.unified_format),
        }
    }

    pub(crate) fn format_selector(self) -> &'static str {
        match self.mode {
            MusicDownloadMode::Original => match self.original_preference {
                MusicOriginalPreference::Auto => MUSIC_ORIGINAL_AUTO_SELECTOR,
                MusicOriginalPreference::PreferMp3 => MUSIC_ORIGINAL_MP3_SELECTOR,
                MusicOriginalPreference::PreferAac => MUSIC_ORIGINAL_AAC_SELECTOR,
                MusicOriginalPreference::PreferOpus => MUSIC_ORIGINAL_OPUS_SELECTOR,
            },
            MusicDownloadMode::Unified => music_online_target_format_selector(self.unified_format),
        }
    }

    pub(crate) fn selection_token(self) -> &'static str {
        match self.mode {
            MusicDownloadMode::Original => match self.original_preference {
                MusicOriginalPreference::Auto => "original:auto",
                MusicOriginalPreference::PreferMp3 => "original:mp3",
                MusicOriginalPreference::PreferAac => "original:aac",
                MusicOriginalPreference::PreferOpus => "original:opus",
            },
            MusicDownloadMode::Unified => self.unified_format.extension(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicDownloadSourceKind {
    CacheCopy,
    CacheConvert,
    /// yt-dlp selected an online source that already matches the requested audio codec.
    YtDlpOnlineTarget,
    YtDlpDownload,
}

impl MusicDownloadFormat {
    pub(crate) fn yt_dlp_audio_format(self) -> &'static str {
        self.extension()
    }
}

pub(crate) fn music_online_target_format_selector(format: MusicDownloadFormat) -> &'static str {
    match format {
        MusicDownloadFormat::Mp3 => {
            "bestaudio[ext=mp3]/bestaudio[acodec^=mp3]/bestaudio/best[acodec!=none]"
        }
        MusicDownloadFormat::M4aAac => {
            "bestaudio[ext=m4a]/bestaudio[acodec^=mp4a]/bestaudio[acodec^=aac]/bestaudio/best[acodec!=none]"
        }
        MusicDownloadFormat::Opus => {
            "bestaudio[acodec^=opus]/bestaudio[ext=opus]/bestaudio[ext=webm][acodec^=opus]/bestaudio/best[acodec!=none]"
        }
        MusicDownloadFormat::Flac => {
            "bestaudio[ext=flac]/bestaudio[acodec^=flac]/bestaudio/best[acodec!=none]"
        }
        MusicDownloadFormat::Wav => {
            "bestaudio[ext=wav]/bestaudio[acodec^=pcm]/bestaudio/best[acodec!=none]"
        }
    }
}
