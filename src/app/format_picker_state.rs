#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatPickerKind {
    Video,
    Audio,
    Subtitle,
    Section,
}

impl FormatPickerKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Subtitle => "subtitle",
            Self::Section => "section",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatPickerViewMode {
    Filter,
    Table,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatPickerSortColumn {
    Kind,
    Resolution,
    DynamicRange,
    Fps,
    Ext,
    Codec,
    Filesize,
    SampleRate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormatPickerSortState {
    pub column: Option<FormatPickerSortColumn>,
    pub ascending: bool,
}

impl Default for FormatPickerSortState {
    fn default() -> Self {
        Self {
            column: None,
            ascending: true,
        }
    }
}

pub struct FormatPickerState {
    pub open: bool,
    pub target_item_id: Option<usize>,
    pub kind: Option<FormatPickerKind>,
    pub view_mode: FormatPickerViewMode,
    pub selected_row: Option<usize>,
    pub filter_text: String,
    pub sort_state: FormatPickerSortState,
    pub filters: FormatPickerFilters,
    pub subtitle_source_key: String,
    pub subtitle_tab: SubtitlePickerTab,
}

impl Default for FormatPickerState {
    fn default() -> Self {
        Self {
            open: false,
            target_item_id: None,
            kind: None,
            view_mode: FormatPickerViewMode::Filter,
            selected_row: None,
            filter_text: String::new(),
            sort_state: FormatPickerSortState::default(),
            filters: FormatPickerFilters::default(),
            subtitle_source_key: String::new(),
            subtitle_tab: SubtitlePickerTab::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubtitlePickerTab {
    None,
    Original,
    Automatic,
}

impl SubtitlePickerTab {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "picker.subtitle_tab.none",
            Self::Original => "picker.subtitle_tab.original",
            Self::Automatic => "picker.subtitle_tab.automatic",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FormatPickerFilters {
    pub resolution: Option<String>,
    pub dynamic_range: Option<String>,
    pub fps: Option<String>,
    pub ext: Option<String>,
    pub codec: Option<String>,
    pub sample_rate: Option<String>,
}

impl FormatPickerFilters {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
