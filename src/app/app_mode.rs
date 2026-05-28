#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppMode {
    #[default]
    Origin,
    Standard,
    Audio,
}

impl AppMode {
    pub const ALL: [Self; 3] = [Self::Origin, Self::Standard, Self::Audio];

    pub fn from_config_value(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "origin" => Self::Origin,
            "standard" => Self::Standard,
            "audio" => Self::Audio,
            _ => Self::Origin,
        }
    }

    pub fn config_value(self) -> &'static str {
        match self {
            Self::Origin => "origin",
            Self::Standard => "standard",
            Self::Audio => "audio",
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Origin => "app_mode.origin",
            Self::Standard => "app_mode.standard",
            Self::Audio => "app_mode.audio",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueDisplayMode {
    Normal,
    Audio,
}

impl QueueDisplayMode {
    pub fn from_app_mode(mode: AppMode) -> Self {
        match mode {
            AppMode::Audio => Self::Audio,
            AppMode::Origin | AppMode::Standard => Self::Normal,
        }
    }

    pub fn config_value(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Audio => "audio",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppMode;

    #[test]
    fn app_mode_accepts_only_canonical_config_values() {
        assert_eq!(AppMode::from_config_value("origin"), AppMode::Origin);
        assert_eq!(AppMode::from_config_value("standard"), AppMode::Standard);
        assert_eq!(AppMode::from_config_value("audio"), AppMode::Audio);

        assert_eq!(AppMode::from_config_value("single"), AppMode::Origin);
        assert_eq!(AppMode::from_config_value("normal"), AppMode::Origin);
        assert_eq!(AppMode::from_config_value("garbage"), AppMode::Origin);
    }

    #[test]
    fn app_mode_saves_only_canonical_config_values() {
        assert_eq!(AppMode::Origin.config_value(), "origin");
        assert_eq!(AppMode::Standard.config_value(), "standard");
        assert_eq!(AppMode::Audio.config_value(), "audio");
    }
}
