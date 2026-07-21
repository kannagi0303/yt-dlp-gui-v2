use crate::infrastructure::{ManagedComponentId, YoutubePlaylistRisk};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTab {
    Prepare,
    Main,
    Advance,
    Options,
    About,
    Log,
}

#[derive(Clone, Debug)]
pub struct MusicLyricsDisplayLine {
    pub current: String,
    pub previous: Option<String>,
    pub fade: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MusicPlayerAuraPulse {
    pub origin: f32,
    pub age: f32,
    pub strength: f32,
    pub air: f32,
}

impl Default for MusicPlayerAuraPulse {
    fn default() -> Self {
        Self {
            origin: 0.0,
            age: 1.0,
            strength: 0.0,
            air: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MusicPlayerAuraTrackField {
    pub bar_phase: f32,
    pub beat_phase: f32,
    pub downbeat_strength: f32,
    pub energy: f32,
    pub energy_momentum: f32,
    pub boundary: f32,
    pub novelty: f32,
    pub recurrence: f32,
    pub chorusness: f32,
    pub chroma_hue: f32,
    pub chroma_coherence: f32,
    pub section_color_unit: f32,
    pub section_color_strength: f32,
    pub spectrum_bands: [f32; 8],
    pub spectrum_peaks: [f32; 8],
    pub pulses: [MusicPlayerAuraPulse; 4],
}

impl Default for MusicPlayerAuraTrackField {
    fn default() -> Self {
        Self {
            bar_phase: 0.0,
            beat_phase: 0.0,
            downbeat_strength: 0.0,
            energy: 0.0,
            energy_momentum: 0.0,
            boundary: 0.0,
            novelty: 0.0,
            recurrence: 0.0,
            chorusness: 0.0,
            chroma_hue: 0.58,
            chroma_coherence: 0.0,
            section_color_unit: 0.5,
            section_color_strength: 0.0,
            spectrum_bands: [0.0; 8],
            spectrum_peaks: [0.0; 8],
            pulses: [MusicPlayerAuraPulse::default(); 4],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MusicPlayerAuraDisplay {
    pub animating: bool,
    pub primary_item_id: Option<u64>,
    pub primary: Option<MusicPlayerAuraTrackField>,
    pub secondary_item_id: Option<u64>,
    pub secondary: Option<MusicPlayerAuraTrackField>,
    pub mix_progress: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionsDetailPage {
    Language,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrepareDetailPage {
    Language,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdvanceDetailPage {
    Transcode,
    CookieManager,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CookieUsageMode {
    Off,
    Browser,
    File,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CookieFileSourceMode {
    Custom,
    AutoSelect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedCookieFile {
    pub id: String,
    pub display_name: String,
    pub login_url: String,
    pub updated_unix: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AboutDetailTarget {
    App,
    Tool(ManagedComponentId),
}

pub enum ThumbnailRenderSource {
    None,
    DirectUrl,
    Loading,
    Texture(eframe::egui::TextureHandle),
    Failed(String),
}

pub(super) enum ThumbnailCacheEntry {
    Loading,
    Ready(eframe::egui::TextureHandle),
    Failed(String),
}

pub struct YoutubePlaylistPrompt {
    pub source: String,
    pub kind: YoutubePlaylistPromptKind,
    pub risk: Option<YoutubePlaylistRisk>,
    pub music_compact: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YoutubePlaylistPromptKind {
    VideoAndPlaylist,
    HighRiskPlaylist,
}
