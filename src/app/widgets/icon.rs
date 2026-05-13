use eframe::egui::{self, Color32, Image};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppIcon {
    CheckboxBlankOutline,
    CheckCircle,
    ContentCopy,
    ContentCut,
    ContentPaste,
    ContentSave,
    Download,
    Eraser,
    FolderMoveOutline,
    FolderSettings,
    Import,
    Information,
    LinkVariant,
    Loading,
    Magnify,
    MenuDown,
    MenuRight,
    Monitor,
    MonitorEye,
    MonitorOff,
    Multimedia,
    NewBox,
    Package,
    Play,
    Video,
    VolumeHigh,
    WindowClose,
    WindowMaximize,
    WindowMinimize,
    WindowRestore,
}

impl AppIcon {
    fn name(self) -> &'static str {
        match self {
            Self::CheckboxBlankOutline => "CheckboxBlankOutline",
            Self::CheckCircle => "CheckCircle",
            Self::ContentCopy => "ContentCopy",
            Self::ContentCut => "ContentCut",
            Self::ContentPaste => "ContentPaste",
            Self::ContentSave => "ContentSave",
            Self::Download => "Download",
            Self::Eraser => "Eraser",
            Self::FolderMoveOutline => "FolderMoveOutline",
            Self::FolderSettings => "FolderSettings",
            Self::Import => "Import",
            Self::Information => "Information",
            Self::LinkVariant => "LinkVariant",
            Self::Loading => "Loading",
            Self::Magnify => "Magnify",
            Self::MenuDown => "MenuDown",
            Self::MenuRight => "MenuRight",
            Self::Monitor => "Monitor",
            Self::MonitorEye => "MonitorEye",
            Self::MonitorOff => "MonitorOff",
            Self::Multimedia => "Multimedia",
            Self::NewBox => "NewBox",
            Self::Package => "Package",
            Self::Play => "Play",
            Self::Video => "Video",
            Self::VolumeHigh => "VolumeHigh",
            Self::WindowClose => "WindowClose",
            Self::WindowMaximize => "WindowMaximize",
            Self::WindowMinimize => "WindowMinimize",
            Self::WindowRestore => "WindowRestore",
        }
    }

    fn svg_bytes(self) -> &'static [u8] {
        match self {
            Self::CheckboxBlankOutline => {
                include_bytes!("../../../assets/icons/CheckboxBlankOutline.svg")
            }
            Self::CheckCircle => include_bytes!("../../../assets/icons/CheckCircle.svg"),
            Self::ContentCopy => include_bytes!("../../../assets/icons/ContentCopy.svg"),
            Self::ContentCut => include_bytes!("../../../assets/icons/ContentCut.svg"),
            Self::ContentPaste => include_bytes!("../../../assets/icons/ContentPaste.svg"),
            Self::ContentSave => include_bytes!("../../../assets/icons/ContentSave.svg"),
            Self::Download => include_bytes!("../../../assets/icons/Download.svg"),
            Self::Eraser => include_bytes!("../../../assets/icons/Eraser.svg"),
            Self::FolderMoveOutline => {
                include_bytes!("../../../assets/icons/FolderMoveOutline.svg")
            }
            Self::FolderSettings => include_bytes!("../../../assets/icons/FolderSettings.svg"),
            Self::Import => include_bytes!("../../../assets/icons/Import.svg"),
            Self::Information => include_bytes!("../../../assets/icons/Information.svg"),
            Self::LinkVariant => include_bytes!("../../../assets/icons/LinkVariant.svg"),
            Self::Loading => include_bytes!("../../../assets/icons/Loading.svg"),
            Self::Magnify => include_bytes!("../../../assets/icons/Magnify.svg"),
            Self::MenuDown => include_bytes!("../../../assets/icons/MenuDown.svg"),
            Self::MenuRight => include_bytes!("../../../assets/icons/MenuRight.svg"),
            Self::Monitor => include_bytes!("../../../assets/icons/Monitor.svg"),
            Self::MonitorEye => include_bytes!("../../../assets/icons/MonitorEye.svg"),
            Self::MonitorOff => include_bytes!("../../../assets/icons/MonitorOff.svg"),
            Self::Multimedia => include_bytes!("../../../assets/icons/Multimedia.svg"),
            Self::NewBox => include_bytes!("../../../assets/icons/NewBox.svg"),
            Self::Package => include_bytes!("../../../assets/icons/Package.svg"),
            Self::Play => include_bytes!("../../../assets/icons/Play.svg"),
            Self::Video => include_bytes!("../../../assets/icons/Video.svg"),
            Self::VolumeHigh => include_bytes!("../../../assets/icons/VolumeHigh.svg"),
            Self::WindowClose => include_bytes!("../../../assets/icons/WindowClose.svg"),
            Self::WindowMaximize => include_bytes!("../../../assets/icons/WindowMaximize.svg"),
            Self::WindowMinimize => include_bytes!("../../../assets/icons/WindowMinimize.svg"),
            Self::WindowRestore => include_bytes!("../../../assets/icons/WindowRestore.svg"),
        }
    }

    fn uri(self) -> String {
        format!("bytes://icons/{}-mask.svg", self.name())
    }

    fn mask_svg_bytes(self) -> Vec<u8> {
        String::from_utf8_lossy(self.svg_bytes())
            .replace("currentColor", "#FFFFFF")
            .into_bytes()
    }
}

pub fn icon_image(icon: AppIcon, size: f32, color: Color32) -> Image<'static> {
    Image::from_bytes(icon.uri(), icon.mask_svg_bytes())
        .fit_to_exact_size(egui::vec2(size, size))
        .tint(color)
}
