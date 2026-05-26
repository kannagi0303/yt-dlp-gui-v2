#[cfg(target_os = "windows")]
pub fn send_download_finished_windows_toast(
    language: crate::i18n::Language,
    title: &str,
    output_path: Option<&str>,
) -> Result<(), String> {
    windows_toast::send_download_finished_windows_toast(language, title, output_path)
}

#[cfg(not(target_os = "windows"))]
pub fn send_download_finished_windows_toast(
    language: crate::i18n::Language,
    _title: &str,
    _output_path: Option<&str>,
) -> Result<(), String> {
    Err(crate::i18n::text(language, "notification.windows_toast_windows_only").to_owned())
}

#[cfg(target_os = "windows")]
pub fn send_download_failed_windows_toast(
    language: crate::i18n::Language,
    title: &str,
    error: &str,
) -> Result<(), String> {
    windows_toast::send_download_failed_windows_toast(language, title, error)
}

#[cfg(not(target_os = "windows"))]
pub fn send_download_failed_windows_toast(
    language: crate::i18n::Language,
    _title: &str,
    _error: &str,
) -> Result<(), String> {
    Err(crate::i18n::text(language, "notification.windows_toast_windows_only").to_owned())
}

#[cfg(target_os = "windows")]
mod windows_toast {
    use std::path::Path;

    use crate::i18n::{self, Language};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::infrastructure::app_identity;

    use win32_notif::{
        NotificationBuilder, ToastsNotifier,
        notification::visual::{Text, text::HintStyle},
    };

    const TOAST_GROUP: &str = "yt-dlp-gui";
    const DOWNLOAD_FINISHED_TAG: &str = "download-finished";
    const DOWNLOAD_FAILED_TAG: &str = "download-failed";

    pub fn send_download_finished_windows_toast(
        language: Language,
        title: &str,
        output_path: Option<&str>,
    ) -> Result<(), String> {
        let detail = output_path
            .and_then(file_name_from_path)
            .map(|file_name| {
                format!(
                    "{}{}",
                    i18n::text(language, "notification.download_finished_detail_prefix"),
                    file_name
                )
            })
            .unwrap_or_else(|| {
                i18n::text(language, "notification.download_finished_detail").to_owned()
            });

        show_notification(
            DOWNLOAD_FINISHED_TAG,
            i18n::text(language, "notification.download_finished"),
            title,
            &detail,
        )
    }

    pub fn send_download_failed_windows_toast(
        language: Language,
        title: &str,
        error: &str,
    ) -> Result<(), String> {
        let detail = i18n::localize_message(language, error);
        show_notification(
            DOWNLOAD_FAILED_TAG,
            i18n::text(language, "notification.download_failed"),
            title,
            &truncate_to_chars(detail.trim(), 180),
        )
    }

    fn show_notification(tag: &str, title: &str, line_1: &str, line_2: &str) -> Result<(), String> {
        app_identity::ensure_windows_app_identity()?;

        let notifier = ToastsNotifier::new(Some(app_identity::APP_AUMID))
            .map_err(|error| format!("Could not create Windows Toast notifier: {error}"))?;
        let notification = NotificationBuilder::new()
            .visual(Text::create(0, title).with_style(HintStyle::Title))
            .visual(Text::create(1, line_1))
            .visual(Text::create(2, line_2))
            .build(next_sequence_number(), &notifier, tag, TOAST_GROUP)
            .map_err(|error| format!("Could not create Windows Toast content: {error}"))?;

        notification
            .show()
            .map_err(|error| format!("Could not send Windows Toast: {error}"))
    }

    fn next_sequence_number() -> u32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() as u32)
            .unwrap_or(1)
    }

    fn file_name_from_path(path: &str) -> Option<String> {
        Path::new(path)
            .file_name()
            .and_then(|value| value.to_str())
            .map(ToOwned::to_owned)
    }

    fn truncate_to_chars(value: &str, max_chars: usize) -> String {
        let mut chars = value.chars();
        let mut output = String::new();
        for _ in 0..max_chars {
            let Some(ch) = chars.next() else {
                return output;
            };
            output.push(ch);
        }
        if chars.next().is_some() {
            output.push('…');
        }
        output
    }
}
