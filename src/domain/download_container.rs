#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DownloadContainerPreference {
    #[default]
    Auto,
    Mkv,
    Webm,
}

impl DownloadContainerPreference {
    pub fn extension(self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::Mkv => Some("mkv"),
            Self::Webm => Some("webm"),
        }
    }
}

pub fn codecs_support_webm_container(video_codec: &str, audio_codec: &str) -> bool {
    webm_video_codec_is_supported(video_codec) && webm_audio_codec_is_supported(audio_codec)
}

fn webm_video_codec_is_supported(codec: &str) -> bool {
    let codec = codec.trim().to_ascii_lowercase();
    codec.starts_with("vp8")
        || codec.starts_with("vp9")
        || codec.starts_with("av1")
        || codec.starts_with("av01")
}

fn webm_audio_codec_is_supported(codec: &str) -> bool {
    let codec = codec.trim().to_ascii_lowercase();
    codec.starts_with("opus") || codec.starts_with("vorbis")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webm_accepts_vp9_opus_and_rejects_h264_aac() {
        assert!(codecs_support_webm_container("VP9", "OPUS"));
        assert!(codecs_support_webm_container("AV1", "Vorbis"));
        assert!(!codecs_support_webm_container("H.264", "AAC"));
    }
}
