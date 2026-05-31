#[derive(Clone)]
pub struct VideoMetadata {
    pub title: String,
    pub channel: String,
    pub channel_url: String,
    pub uploader: String,
    pub uploader_url: String,
    pub creator: String,
    pub creator_url: String,
    pub duration_text: String,
    pub webpage_url: String,
    pub description: String,
    pub view_count_text: String,
    pub upload_date_text: String,
    pub thumbnail_hint: String,
    pub thumbnail_url: String,
    pub formats: Vec<FormatOption>,
    pub subtitle_tracks: Vec<SubtitleOption>,
    pub chapters: Vec<ChapterOption>,
}

impl VideoMetadata {
    pub fn empty_preview() -> Self {
        Self {
            title: String::new(),
            channel: String::new(),
            channel_url: String::new(),
            uploader: String::new(),
            uploader_url: String::new(),
            creator: String::new(),
            creator_url: String::new(),
            duration_text: String::new(),
            webpage_url: String::new(),
            description: String::new(),
            view_count_text: String::new(),
            upload_date_text: String::new(),
            thumbnail_hint: "item.thumbnail".to_owned(),
            thumbnail_url: String::new(),
            formats: Vec::new(),
            subtitle_tracks: Vec::new(),
            chapters: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct ChapterOption {
    pub id: String,
    pub title: String,
    pub start_text: String,
    pub end_text: Option<String>,
    pub download_sections: String,
}

impl ChapterOption {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        start_text: impl Into<String>,
        end_text: Option<String>,
        download_sections: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            start_text: start_text.into(),
            end_text,
            download_sections: download_sections.into(),
        }
    }

    pub fn label(&self) -> String {
        let range = match self.end_text.as_deref() {
            Some(end) if !end.is_empty() => format!("{}–{}", self.start_text, end),
            _ => format!("{}–end", self.start_text),
        };

        if self.title.trim().is_empty() {
            range
        } else {
            format!("{}  {}", range, self.title)
        }
    }
}

#[derive(Clone)]
pub struct SubtitleOption {
    pub id: String,
    pub source: SubtitleSource,
    pub download_language_code: String,
    pub source_language_code: String,
    pub source_language_label: String,
    pub target_language_code: Option<String>,
    pub target_language_label: Option<String>,
    pub ext: String,
    pub url: String,
}

impl SubtitleOption {
    pub fn new(
        id: impl Into<String>,
        source: SubtitleSource,
        download_language_code: impl Into<String>,
        source_language_code: impl Into<String>,
        source_language_label: impl Into<String>,
        target_language_code: Option<String>,
        target_language_label: Option<String>,
        ext: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source,
            download_language_code: download_language_code.into(),
            source_language_code: source_language_code.into(),
            source_language_label: source_language_label.into(),
            target_language_code,
            target_language_label,
            ext: ext.into(),
            url: url.into(),
        }
    }

    pub fn source_key(&self) -> String {
        format!("{}:{}", self.source.key(), self.source_language_code)
    }

    pub fn source_label(&self) -> String {
        format!(
            "{} / {} ({})",
            self.source.label(),
            self.source_language_label,
            self.source_language_code
        )
    }

    pub fn target_label(&self) -> String {
        match (&self.target_language_label, &self.target_language_code) {
            (Some(label), Some(code)) => format!("{label} ({code})"),
            (Some(label), None) => label.clone(),
            (None, Some(code)) => code.clone(),
            (None, None) => "No translation".to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubtitleSource {
    None,
    Original,
    Automatic,
}

impl SubtitleSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "No subtitles",
            Self::Original => "Original subtitles",
            Self::Automatic => "Automatic subtitles",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Original => "orig",
            Self::Automatic => "auto",
        }
    }
}

#[derive(Clone)]
pub struct FormatOption {
    pub id: String,
    pub label: String,
    pub kind: MediaKind,
    pub is_muxed: bool,
    pub resolution: String,
    pub dynamic_range: String,
    pub fps: String,
    pub ext: String,
    pub codec: String,
    pub sample_rate: String,
    pub filesize: String,
}

impl FormatOption {
    pub fn new(id: &str, label: &str, kind: MediaKind) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
            kind,
            is_muxed: kind == MediaKind::Muxed,
            resolution: String::new(),
            dynamic_range: String::new(),
            fps: String::new(),
            ext: String::new(),
            codec: String::new(),
            sample_rate: String::new(),
            filesize: String::new(),
        }
    }

    pub fn video(
        id: &str,
        label: &str,
        kind: MediaKind,
        resolution: &str,
        dynamic_range: &str,
        fps: &str,
        ext: &str,
        codec: &str,
        filesize: &str,
    ) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
            kind,
            is_muxed: kind == MediaKind::Muxed,
            resolution: resolution.to_owned(),
            dynamic_range: dynamic_range.to_owned(),
            fps: fps.to_owned(),
            ext: ext.to_owned(),
            codec: codec.to_owned(),
            sample_rate: String::new(),
            filesize: filesize.to_owned(),
        }
    }

    pub fn audio(
        id: &str,
        label: &str,
        kind: MediaKind,
        sample_rate: &str,
        ext: &str,
        codec: &str,
        filesize: &str,
    ) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
            kind,
            is_muxed: kind == MediaKind::Muxed,
            resolution: String::new(),
            dynamic_range: String::new(),
            fps: String::new(),
            ext: ext.to_owned(),
            codec: codec.to_owned(),
            sample_rate: sample_rate.to_owned(),
            filesize: filesize.to_owned(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Video,
    Audio,
    Muxed,
    Subtitle,
    Other,
}

impl MediaKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Muxed => "muxed",
            Self::Subtitle => "subtitle",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QualityPreset {
    Best,
    P1080,
    P720,
    AudioOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueItemViewKind {
    VideoCard,
    MusicCompact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactMusicState {
    Resolving,
    Ready,
    Buffering,
    Playing,
    Paused,
    Failed,
}

impl QualityPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Best => "Best",
            Self::P1080 => "1080p",
            Self::P720 => "720p",
            Self::AudioOnly => "Audio only",
        }
    }
}

pub struct DownloadOptions {
    pub quality: QualityPreset,
    pub write_thumbnail: bool,
    pub embed_thumbnail: bool,
    pub write_subtitles: bool,
    pub embed_subtitles: bool,
    pub write_chapters: bool,
    pub embed_chapters: bool,
    pub use_cookies: bool,
    pub use_aria2: bool,
    pub output_dir: String,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            quality: QualityPreset::Best,
            write_thumbnail: false,
            embed_thumbnail: false,
            write_subtitles: false,
            embed_subtitles: false,
            write_chapters: false,
            embed_chapters: false,
            use_cookies: false,
            use_aria2: false,
            output_dir: "Desktop".to_owned(),
        }
    }
}

pub type QueueItemId = u64;
pub type WorkflowRunId = u64;

#[derive(Clone)]
pub struct QueueItem {
    pub id: QueueItemId,
    pub source_url: String,
    pub title: String,
    pub music_album_title: String,
    pub thumbnail_hint: String,
    pub thumbnail_url: String,
    pub duration_text: String,
    pub cookie_policy: CookiePolicy,
    pub metadata_state: MetadataState,
    pub selection: DownloadSelection,
    pub progress: ItemProgress,
    pub workflows: Vec<WorkflowRun>,
    pub view_kind: QueueItemViewKind,
    pub compact_music_state: Option<CompactMusicState>,
    pub music_stream_url: String,
    pub music_stream_headers: Vec<(String, String)>,
    pub music_stream_ext: String,
    pub music_stream_format_id: String,
    pub music_stream_acodec: String,
    pub music_stream_expected_bytes: Option<u64>,
    pub music_cache_key: String,
    pub music_duration_seconds: Option<f64>,
    pub completed_selection: Option<CompletedSelection>,
    pub last_output_path: Option<String>,
    pub last_error: Option<String>,
}

impl QueueItem {
    pub fn new(id: QueueItemId, source_url: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id,
            source_url: source_url.into(),
            title: title.into(),
            music_album_title: String::new(),
            thumbnail_hint: "item.thumbnail".to_owned(),
            thumbnail_url: String::new(),
            duration_text: String::new(),
            cookie_policy: CookiePolicy::Unknown,
            metadata_state: MetadataState::Queued,
            selection: DownloadSelection::default(),
            progress: ItemProgress::default(),
            workflows: Vec::new(),
            view_kind: QueueItemViewKind::VideoCard,
            compact_music_state: None,
            music_stream_url: String::new(),
            music_stream_headers: Vec::new(),
            music_stream_ext: String::new(),
            music_stream_format_id: String::new(),
            music_stream_acodec: String::new(),
            music_stream_expected_bytes: None,
            music_cache_key: String::new(),
            music_duration_seconds: None,
            completed_selection: None,
            last_output_path: None,
            last_error: None,
        }
    }

    pub fn metadata(&self) -> Option<&VideoMetadata> {
        match &self.metadata_state {
            MetadataState::Ready(metadata) => Some(metadata),
            _ => None,
        }
    }

    pub fn metadata_mut(&mut self) -> Option<&mut VideoMetadata> {
        match &mut self.metadata_state {
            MetadataState::Ready(metadata) => Some(metadata),
            _ => None,
        }
    }

    pub fn metadata_loaded(&self) -> bool {
        matches!(self.metadata_state, MetadataState::Ready(_))
    }

    pub fn status_text(&self) -> &'static str {
        if let Some(run) = self.workflows.iter().rev().find(|run| {
            matches!(
                run.kind,
                WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
            ) && matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
        }) {
            return match run.state {
                WorkflowState::Queued => "Queued",
                WorkflowState::Running => "Running",
                _ => "Queued",
            };
        }

        if let Some(run) = self
            .workflows
            .iter()
            .rev()
            .find(|run| run.kind == WorkflowKind::DownloadMedia)
        {
            match run.state {
                WorkflowState::Queued => return "Queued",
                WorkflowState::Running => return "Running",
                WorkflowState::Finished if self.last_error.is_some() => return "Failed",
                WorkflowState::Finished => return "Done",
                WorkflowState::Failed => return "Failed",
                WorkflowState::Cancelled => return "Cancelled",
            }
        }

        match &self.metadata_state {
            MetadataState::Idle => "Not started",
            MetadataState::Queued => "Waiting for analysis",
            MetadataState::Running => "Analyzing",
            MetadataState::Ready(_) => "Queued",
            MetadataState::Failed(_) => "Analysis failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CookiePolicy {
    Unknown,
    NotNeeded,
    Required,
}

#[derive(Clone)]
pub enum MetadataState {
    Idle,
    Queued,
    Running,
    Ready(VideoMetadata),
    Failed(String),
}

#[derive(Clone, PartialEq, Eq)]
pub struct DownloadSelection {
    pub quality: QualityPreset,
    pub video_selector: String,
    pub audio_selector: String,
    pub subtitle_selector: String,
    pub subtitle_source: SubtitleSource,
    pub write_thumbnail: bool,
    pub embed_thumbnail: bool,
    pub write_subtitles: bool,
    pub embed_subtitles: bool,
    pub write_chapters: bool,
    pub embed_chapters: bool,
    pub use_cookies: bool,
    pub use_aria2: bool,
    pub output_dir: String,
    pub file_name: String,
    pub download_sections: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct CompletedSelection {
    pub video_selector: String,
    pub audio_selector: String,
    pub subtitle_selector: String,
    pub file_name: String,
    pub download_sections: String,
    pub write_thumbnail: bool,
    pub embed_thumbnail: bool,
    pub write_subtitles: bool,
    pub embed_subtitles: bool,
    pub write_chapters: bool,
    pub embed_chapters: bool,
}

impl CompletedSelection {
    pub fn from_selection(selection: &DownloadSelection) -> Self {
        Self {
            video_selector: selection.video_selector.clone(),
            audio_selector: selection.audio_selector.clone(),
            subtitle_selector: selection.subtitle_selector.clone(),
            file_name: selection.file_name.clone(),
            download_sections: selection.download_sections.clone(),
            write_thumbnail: selection.write_thumbnail,
            embed_thumbnail: selection.embed_thumbnail,
            write_subtitles: selection.write_subtitles,
            embed_subtitles: selection.embed_subtitles,
            write_chapters: selection.write_chapters,
            embed_chapters: selection.embed_chapters,
        }
    }
}

impl Default for DownloadSelection {
    fn default() -> Self {
        Self {
            quality: QualityPreset::Best,
            video_selector: String::new(),
            audio_selector: String::new(),
            subtitle_selector: String::new(),
            subtitle_source: SubtitleSource::None,
            write_thumbnail: false,
            embed_thumbnail: false,
            write_subtitles: false,
            embed_subtitles: false,
            write_chapters: false,
            embed_chapters: false,
            use_cookies: false,
            use_aria2: false,
            output_dir: "Desktop".to_owned(),
            file_name: String::new(),
            download_sections: String::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct ItemProgress {
    pub video: f32,
    pub audio: f32,
    pub subtitle: f32,
    pub post_process: f32,
}

#[derive(Clone)]
pub struct WorkflowRun {
    pub id: WorkflowRunId,
    pub kind: WorkflowKind,
    pub tool: ToolKind,
    pub state: WorkflowState,
    pub progress: f32,
    pub detail: String,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

impl WorkflowRun {
    pub fn new(
        id: WorkflowRunId,
        kind: WorkflowKind,
        tool: ToolKind,
        state: WorkflowState,
    ) -> Self {
        Self {
            id,
            kind,
            tool,
            state,
            progress: 0.0,
            detail: String::new(),
            output_path: None,
            error: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkflowKind {
    AnalyzeMetadata,
    DownloadMedia,
    ExportMedia,
    PostProcess,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolKind {
    YtDlp,
    Ffmpeg,
    Aria2c,
    Other(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkflowState {
    Queued,
    Running,
    Finished,
    Failed,
    Cancelled,
}
