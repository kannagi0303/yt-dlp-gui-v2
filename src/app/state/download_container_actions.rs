use super::*;

impl AppState {
    pub fn item_supports_webm_download_container(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };
        let Some(metadata) = item.metadata() else {
            return false;
        };
        if self.item_uses_muxed_video(item_index) {
            return false;
        }

        let video_codec = metadata
            .formats
            .iter()
            .find(|format| format.id == item.selection.video_selector)
            .map(|format| format.codec.as_str())
            .unwrap_or_default();
        let audio_codec = metadata
            .formats
            .iter()
            .find(|format| format.id == item.selection.audio_selector)
            .map(|format| format.codec.as_str())
            .unwrap_or_default();

        codecs_support_webm_container(video_codec, audio_codec)
    }

    pub fn resolved_item_download_container(
        &self,
        item_index: usize,
    ) -> Option<DownloadContainerPreference> {
        let item = self.queue_items.get(item_index)?;
        match item.selection.container_preference {
            DownloadContainerPreference::Mkv => Some(DownloadContainerPreference::Mkv),
            DownloadContainerPreference::Webm
                if self.item_supports_webm_download_container(item_index) =>
            {
                Some(DownloadContainerPreference::Webm)
            }
            DownloadContainerPreference::Webm => None,
            DownloadContainerPreference::Auto => {
                if !self.item_supports_webm_download_container(item_index) {
                    return None;
                }
                if item.selection.embed_thumbnail {
                    return Some(DownloadContainerPreference::Mkv);
                }

                let metadata = item.metadata()?;
                let video_ext = metadata
                    .formats
                    .iter()
                    .find(|format| format.id == item.selection.video_selector)
                    .map(|format| format.ext.as_str())
                    .unwrap_or_default();
                let audio_ext = metadata
                    .formats
                    .iter()
                    .find(|format| format.id == item.selection.audio_selector)
                    .map(|format| format.ext.as_str())
                    .unwrap_or_default();
                Some(
                    if video_ext.eq_ignore_ascii_case("webm")
                        && audio_ext.eq_ignore_ascii_case("webm")
                    {
                        DownloadContainerPreference::Webm
                    } else {
                        DownloadContainerPreference::Mkv
                    },
                )
            }
        }
    }

    pub fn item_download_container_override(&self, item_index: usize) -> Option<&'static str> {
        let item = self.queue_items.get(item_index)?;
        if let Some(extension) = item.selection.container_preference.extension() {
            return Some(extension);
        }
        (item.selection.embed_thumbnail && self.item_supports_webm_download_container(item_index))
            .then_some("mkv")
    }

    pub fn set_item_download_container_preference(
        &mut self,
        item_index: usize,
        preference: DownloadContainerPreference,
    ) {
        if preference == DownloadContainerPreference::Webm
            && !self.item_supports_webm_download_container(item_index)
        {
            return;
        }
        let Some(item) = self.queue_items.get_mut(item_index) else {
            return;
        };
        apply_download_container_preference(&mut item.selection, preference);
    }

    pub(super) fn reconcile_item_download_container(&mut self, item_index: usize) {
        if self.queue_items.get(item_index).is_some_and(|item| {
            item.selection.container_preference == DownloadContainerPreference::Webm
        }) && !self.item_supports_webm_download_container(item_index)
        {
            if let Some(item) = self.queue_items.get_mut(item_index) {
                item.selection.container_preference = DownloadContainerPreference::Auto;
            }
        }
    }
}

fn apply_download_container_preference(
    selection: &mut crate::domain::DownloadSelection,
    preference: DownloadContainerPreference,
) {
    if preference == DownloadContainerPreference::Webm && selection.embed_thumbnail {
        selection.write_thumbnail = true;
        selection.embed_thumbnail = false;
    }
    selection.container_preference = preference;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_container_extensions_are_stable() {
        assert_eq!(DownloadContainerPreference::Mkv.extension(), Some("mkv"));
        assert_eq!(DownloadContainerPreference::Webm.extension(), Some("webm"));
        assert_eq!(DownloadContainerPreference::Auto.extension(), None);
    }

    #[test]
    fn choosing_webm_keeps_thumbnail_as_sidecar_instead_of_embedding() {
        let mut selection = crate::domain::DownloadSelection {
            write_thumbnail: true,
            embed_thumbnail: true,
            ..Default::default()
        };

        apply_download_container_preference(&mut selection, DownloadContainerPreference::Webm);

        assert_eq!(
            selection.container_preference,
            DownloadContainerPreference::Webm
        );
        assert!(selection.write_thumbnail);
        assert!(!selection.embed_thumbnail);
    }
}
