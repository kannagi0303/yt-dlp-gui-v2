use serde::{Deserialize, Serialize};

mod ar_ma;
mod de_de;
mod el_gr;
mod en_us;
mod es_es;
mod fr_fr;
mod it_it;
mod ja_jp;
mod ko_kr;
mod pl_pl;
mod pt_br;
mod ru_ru;
mod uk_ua;
mod zh_cn;
mod zh_tw;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LanguageSelection {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "ar-MA")]
    ArMa,
    #[serde(rename = "de-DE")]
    DeDe,
    #[serde(rename = "el-GR")]
    ElGr,
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "es-ES")]
    EsEs,
    #[serde(rename = "fr-FR")]
    FrFr,
    #[serde(rename = "it-IT")]
    ItIt,
    #[serde(rename = "ja-JP")]
    JaJp,
    #[serde(rename = "ko-KR")]
    KoKr,
    #[serde(rename = "pl-PL")]
    PlPl,
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "ru-RU")]
    RuRu,
    #[serde(rename = "uk-UA")]
    UkUa,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "zh-TW")]
    ZhTw,
}

impl Default for LanguageSelection {
    fn default() -> Self {
        Self::Auto
    }
}

impl LanguageSelection {
    // Stable master picker order. Keep it independent from OS language order.
    pub const PICKER_ORDER: [Self; 16] = [
        Self::Auto,
        Self::EnUs,
        Self::ArMa,
        Self::DeDe,
        Self::ElGr,
        Self::EsEs,
        Self::FrFr,
        Self::ItIt,
        Self::JaJp,
        Self::KoKr,
        Self::PlPl,
        Self::PtBr,
        Self::RuRu,
        Self::UkUa,
        Self::ZhCn,
        Self::ZhTw,
    ];

    pub fn resolve(self) -> Language {
        match self {
            Self::Auto => detect_system_language(),
            Self::ArMa => Language::ArMa,
            Self::DeDe => Language::DeDe,
            Self::ElGr => Language::ElGr,
            Self::EnUs => Language::EnUs,
            Self::EsEs => Language::EsEs,
            Self::FrFr => Language::FrFr,
            Self::ItIt => Language::ItIt,
            Self::JaJp => Language::JaJp,
            Self::KoKr => Language::KoKr,
            Self::PlPl => Language::PlPl,
            Self::PtBr => Language::PtBr,
            Self::RuRu => Language::RuRu,
            Self::UkUa => Language::UkUa,
            Self::ZhCn => Language::ZhCn,
            Self::ZhTw => Language::ZhTw,
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Self::Auto => "Auto detect",
            Self::ArMa => Language::ArMa.native_name(),
            Self::DeDe => Language::DeDe.native_name(),
            Self::ElGr => Language::ElGr.native_name(),
            Self::EnUs => Language::EnUs.native_name(),
            Self::EsEs => Language::EsEs.native_name(),
            Self::FrFr => Language::FrFr.native_name(),
            Self::ItIt => Language::ItIt.native_name(),
            Self::JaJp => Language::JaJp.native_name(),
            Self::KoKr => Language::KoKr.native_name(),
            Self::PlPl => Language::PlPl.native_name(),
            Self::PtBr => Language::PtBr.native_name(),
            Self::RuRu => Language::RuRu.native_name(),
            Self::UkUa => Language::UkUa.native_name(),
            Self::ZhCn => Language::ZhCn.native_name(),
            Self::ZhTw => Language::ZhTw.native_name(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    #[serde(rename = "ar-MA")]
    ArMa,
    #[serde(rename = "de-DE")]
    DeDe,
    #[serde(rename = "el-GR")]
    ElGr,
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "es-ES")]
    EsEs,
    #[serde(rename = "fr-FR")]
    FrFr,
    #[serde(rename = "it-IT")]
    ItIt,
    #[serde(rename = "ja-JP")]
    JaJp,
    #[serde(rename = "ko-KR")]
    KoKr,
    #[serde(rename = "pl-PL")]
    PlPl,
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "ru-RU")]
    RuRu,
    #[serde(rename = "uk-UA")]
    UkUa,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "zh-TW")]
    ZhTw,
}

impl Default for Language {
    fn default() -> Self {
        Self::EnUs
    }
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::ArMa => "ar-MA",
            Self::DeDe => "de-DE",
            Self::ElGr => "el-GR",
            Self::EnUs => "en-US",
            Self::EsEs => "es-ES",
            Self::FrFr => "fr-FR",
            Self::ItIt => "it-IT",
            Self::JaJp => "ja-JP",
            Self::KoKr => "ko-KR",
            Self::PlPl => "pl-PL",
            Self::PtBr => "pt-BR",
            Self::RuRu => "ru-RU",
            Self::UkUa => "uk-UA",
            Self::ZhCn => "zh-CN",
            Self::ZhTw => "zh-TW",
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Self::ArMa => "العربية",
            Self::DeDe => "Deutsch",
            Self::ElGr => "Ελληνικά",
            Self::EnUs => "English",
            Self::EsEs => "Español",
            Self::FrFr => "Français",
            Self::ItIt => "Italiano",
            Self::JaJp => "日本語",
            Self::KoKr => "한국어",
            Self::PlPl => "Polski",
            Self::PtBr => "Português do Brasil",
            Self::RuRu => "Русский",
            Self::UkUa => "Українська",
            Self::ZhCn => "简体中文",
            Self::ZhTw => "繁體中文",
        }
    }
}

pub fn text<'a>(language: Language, key: &'a str) -> &'a str {
    let translated = match language {
        Language::ArMa => ar_ma::text(key),
        Language::DeDe => de_de::text(key),
        Language::ElGr => el_gr::text(key),
        Language::EnUs => en_us::text(key),
        Language::EsEs => es_es::text(key),
        Language::FrFr => fr_fr::text(key),
        Language::ItIt => it_it::text(key),
        Language::JaJp => ja_jp::text(key),
        Language::KoKr => ko_kr::text(key),
        Language::PlPl => pl_pl::text(key),
        Language::PtBr => pt_br::text(key),
        Language::RuRu => ru_ru::text(key),
        Language::UkUa => uk_ua::text(key),
        Language::ZhCn => zh_cn::text(key),
        Language::ZhTw => zh_tw::text(key),
    };

    if language != Language::EnUs && translated == key {
        en_us::text(key)
    } else {
        translated
    }
}

pub fn format_text(language: Language, key: &'static str, args: &[(&str, &str)]) -> String {
    let mut output = text(language, key).to_owned();
    for (placeholder, value) in args {
        output = output.replace(placeholder, value);
    }
    output
}

pub fn localize_message(language: Language, value: &str) -> String {
    let direct = text(language, value);
    if direct != value {
        return direct.to_owned();
    }

    localize_runtime_message(language, value.trim())
}

fn localize_runtime_message(language: Language, value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    if let Some(mapped) = localize_runtime_exact(language, value) {
        return mapped;
    }

    if value.contains('\u{ff1b}') {
        return value
            .split('\u{ff1b}')
            .map(|part| localize_runtime_message(language, part.trim()))
            .collect::<Vec<_>>()
            .join("\u{ff1b}");
    }

    if value.contains("; ") {
        return value
            .split("; ")
            .map(|part| localize_runtime_message(language, part.trim()))
            .collect::<Vec<_>>()
            .join("; ");
    }

    for &(prefix, key) in RUNTIME_PREFIX_MESSAGES {
        if let Some(rest) = value.strip_prefix(prefix) {
            let detail = localize_runtime_message(language, rest.trim());
            return format_text(language, key, &[("{error}", detail.as_str())]);
        }
    }

    if let Some(rest) = value.strip_prefix("yt-dlp was not found: ") {
        if let Some(path) =
            rest.strip_suffix(". Install yt-dlp first, or handle dependency deployment in Options.")
        {
            return format_text(
                language,
                "runtime.yt_dlp_not_found",
                &[("{path}", path.trim())],
            );
        }
    }

    if let Some(rest) = value.strip_prefix("Cookie file was not found: ") {
        if let Some(path) = rest.strip_suffix(
            ". Choose a Netscape cookies.txt again, or change the cookie source back to browser.",
        ) {
            return format_text(
                language,
                "runtime.cookie_file_not_found",
                &[("{path}", path.trim())],
            );
        }
    }

    if let Some(detail) = value
        .strip_prefix("Could not read the Chromium/Chrome cookie database directly. This is usually because the browser locked the Network\\Cookies database, not because the checkbox state is wrong. Close the browser and retry, or change Cookie source to Use file (cookies.txt) in Advanced. Original message: ")
    {
        return format_text(language, "runtime.chromium_cookie_locked", &[("{error}", detail.trim())]);
    }

    if let Some(detail) = value
        .strip_prefix("YouTube temporarily rejected the auto-translated subtitle request (HTTP 429 Too Many Requests). This is rate limiting on YouTube timedtext auto-translation. The GUI keeps the native yt-dlp flow and diagnostic output instead of switching to a custom downloader. Try enabling Cookie/cookies.txt for this item, or choose original automatic subtitles/original subtitles and retry. Original message: ")
    {
        return format_text(language, "runtime.youtube_auto_translated_subtitle_429", &[("{error}", detail.trim())]);
    }

    if let Some(detail) = value
        .strip_prefix("YouTube temporarily rejected the subtitle request (HTTP 429 Too Many Requests). The source subtitle file was not downloaded, so SRT conversion will not run. Retry later, or enable browser cookies before exporting. Original message: ")
    {
        return format_text(language, "runtime.youtube_subtitle_429_conversion", &[("{error}", detail.trim())]);
    }

    if let Some(detail) = value
        .strip_prefix("YouTube rejected the subtitle request (HTTP 429 Too Many Requests). This often happens on the YouTube auto-translation timedtext endpoint. cookies.txt can provide login state, but may not satisfy PO Token / rate-limit requirements for that endpoint. The GUI keeps the native yt-dlp flow and diagnostic logs instead of switching to a custom downloader. Original message: ")
    {
        return format_text(language, "runtime.youtube_subtitle_429_analysis", &[("{error}", detail.trim())]);
    }

    if let Some(rest) = value.strip_prefix("Could not create tools folder ") {
        if let Some((path, error)) = rest.rsplit_once(": ") {
            return format_text(
                language,
                "runtime.could_not_create_tools_folder",
                &[("{path}", path.trim()), ("{error}", error.trim())],
            );
        }
    }

    if let Some((tool, rest)) = value.split_once(" installation finished, but ") {
        if let Some(path) = rest.strip_suffix(" was not found.") {
            return format_text(
                language,
                "runtime.install_finished_missing",
                &[("{tool}", tool.trim()), ("{path}", path.trim())],
            );
        }
    }

    if let Some(rest) = value.strip_prefix("Can install to ") {
        if let Some(path) = rest.strip_suffix('.') {
            return format_text(
                language,
                "runtime.can_install_to",
                &[("{path}", path.trim())],
            );
        }
    }

    if let Some(path) = value.strip_prefix("Current path: ") {
        return format_text(language, "runtime.current_path", &[("{path}", path.trim())]);
    }
    if let Some(path) = value.strip_prefix("Default path: ") {
        return format_text(language, "runtime.default_path", &[("{path}", path.trim())]);
    }
    if let Some(path) = value.strip_prefix("Not found: ") {
        return format_text(
            language,
            "runtime.not_found_path",
            &[("{path}", path.trim())],
        );
    }
    if let Some(path) = value.strip_prefix("Writable: ") {
        return format_text(
            language,
            "runtime.writable_path",
            &[("{path}", path.trim())],
        );
    }
    if let Some(path) = value.strip_prefix("Can save: ") {
        return format_text(
            language,
            "runtime.can_save_path",
            &[("{path}", path.trim())],
        );
    }
    if let Some(path) = value.strip_suffix(" is a folder") {
        return format_text(
            language,
            "runtime.path_is_folder",
            &[("{path}", path.trim())],
        );
    }
    if let Some(path) = value.strip_suffix(" is not a folder") {
        return format_text(
            language,
            "runtime.path_is_not_folder",
            &[("{path}", path.trim())],
        );
    }

    if let Some(detail) = value.strip_prefix("System check: ") {
        let detail = localize_runtime_message(language, detail.trim());
        return format_text(
            language,
            "runtime.system_check",
            &[("{detail}", detail.as_str())],
        );
    }
    if let Some(detail) = value.strip_prefix("Save test: ") {
        let detail = localize_runtime_message(language, detail.trim());
        return format_text(
            language,
            "runtime.save_test",
            &[("{detail}", detail.as_str())],
        );
    }
    if let Some(detail) = value.strip_prefix("Write test: ") {
        let detail = localize_runtime_message(language, detail.trim());
        return format_text(
            language,
            "runtime.write_test",
            &[("{detail}", detail.as_str())],
        );
    }

    if let Some(code) = value.strip_prefix("Windows error code ") {
        return format_text(
            language,
            "runtime.reason_windows_error_code",
            &[("{code}", code.trim())],
        );
    }

    if let Some((action, detail)) = value.split_once(": ") {
        if let Some(localized_action) = localize_runtime_exact(language, action.trim()) {
            let localized_detail = localize_parenthesized_detail(language, detail.trim());
            return format!("{localized_action}: {localized_detail}");
        }
    }

    value.to_owned()
}

fn localize_parenthesized_detail(language: Language, value: &str) -> String {
    if let Some((reason, rest)) = value.split_once(" (") {
        if let Some(raw_error) = rest.strip_suffix(')') {
            let reason = localize_runtime_message(language, reason.trim());
            return format!("{reason} ({raw_error})");
        }
    }

    localize_runtime_message(language, value)
}

fn localize_runtime_exact(language: Language, value: &str) -> Option<String> {
    let key = match value {
        "Download cancelled." => "runtime.download_cancelled",
        "Cookies are enabled and the cookie source is file, but no valid Netscape cookies.txt is selected." => {
            "runtime.cookie_file_source_missing"
        }
        "Cookies are enabled, but no browser or cookies.txt source is selected." => {
            "runtime.cookie_source_missing"
        }
        "Download folder cannot be empty." => "runtime.download_folder_empty",
        "Could not wait for yt-dlp to finish: child process missing" => {
            "runtime.could_not_wait_yt_dlp_missing"
        }
        "Thumbnail load failed: empty URL" => "runtime.thumbnail_empty_url",
        "Thumbnail load failed: no data received" => "runtime.thumbnail_no_data",
        "Thumbnail load failed: image too large" => "runtime.thumbnail_too_large",
        "Dependency deployment currently only supports Windows." => {
            "runtime.dependency_windows_only"
        }
        "Could not read PowerShell stdout." => "runtime.could_not_read_powershell_stdout",
        "Could not read PowerShell stderr." => "runtime.could_not_read_powershell_stderr",
        "Could not read yt-dlp playlist output." => "runtime.could_not_read_playlist_output_empty",
        "missing parent directory" => "runtime.missing_parent_directory",
        "Config file path could not be resolved" => "runtime.config_path_unresolved",
        "Folder is marked read-only" => "runtime.folder_readonly",
        "Located on a network path; permissions or file locks may affect it" => {
            "runtime.network_path_warning"
        }
        "Located in a Windows protected directory" => "runtime.protected_directory_warning",
        "Located in a OneDrive sync path; sync locks or security blocking may occur" => {
            "runtime.onedrive_warning"
        }
        "Could not create config folder" => "runtime.could_not_create_config_folder",
        "Could not read config file status" => "runtime.could_not_read_config_file_status",
        "Could not open config file for writing" => {
            "runtime.could_not_open_config_file_for_writing"
        }
        "Could not create folder" => "runtime.could_not_create_folder",
        "Could not create, rename, or delete the test file" => {
            "runtime.could_not_create_rename_delete_test_file"
        }
        "Path does not exist or the parent path is inaccessible" => {
            "runtime.reason_path_inaccessible"
        }
        "Make sure the drive and parent folder exist." => "runtime.recommend_parent_exists",
        "Permission denied or blocked by Windows security settings" => {
            "runtime.reason_permission_denied_windows"
        }
        "Move the app to a writable portable folder; if Desktop/Documents/Downloads still fail, Defender Controlled Folder Access may be blocking it." => {
            "runtime.recommend_move_portable_defender"
        }
        "File or folder is being used by another program" => "runtime.reason_in_use",
        "Close the program that may be using this folder, or choose another folder." => {
            "runtime.recommend_close_program"
        }
        "Test file already exists or name conflict" => "runtime.reason_name_conflict",
        "Not enough disk space" => "runtime.reason_disk_space",
        "Free disk space or choose another disk." => "runtime.recommend_free_space",
        "Path is too long" => "runtime.reason_path_too_long",
        "Move the app to a shorter path, for example D:\\Portable\\yt-dlp-gui-v2." => {
            "runtime.recommend_shorter_path"
        }
        "Choose a clearly writable portable folder and check again." => {
            "runtime.recommend_writable_portable_folder"
        }
        "Permission denied or blocked by security settings" => "runtime.reason_permission_denied",
        "Path does not exist" => "runtime.reason_path_not_exist",
        "File already exists" => "runtime.reason_file_already_exists",
        "Write failed" => "runtime.reason_write_failed",
        "Do not place the portable app under Program Files or the Windows directory; move it to D:\\Portable or a user folder." => {
            "runtime.recommend_not_system_folder"
        }
        "Move it to a non-synced folder, for example D:\\Portable\\yt-dlp-gui-v2." => {
            "runtime.recommend_non_synced_folder"
        }
        _ => return None,
    };

    Some(text(language, key).to_owned())
}

const RUNTIME_PREFIX_MESSAGES: &[(&str, &str)] = &[
    ("Could not start yt-dlp: ", "runtime.could_not_start_yt_dlp"),
    ("yt-dlp analysis failed: ", "runtime.yt_dlp_analysis_failed"),
    (
        "Could not parse yt-dlp JSON: ",
        "runtime.could_not_parse_yt_dlp_json",
    ),
    ("yt-dlp download failed: ", "runtime.yt_dlp_download_failed"),
    (
        "Could not wait for yt-dlp to finish: ",
        "runtime.could_not_wait_yt_dlp",
    ),
    (
        "Could not determine subtitle output file name: ",
        "runtime.could_not_determine_subtitle_output",
    ),
    (
        "yt-dlp finished, but the converted subtitle file was not found: ",
        "runtime.converted_subtitle_missing",
    ),
    (
        "Could not overwrite existing subtitle file: ",
        "runtime.could_not_overwrite_subtitle",
    ),
    (
        "Could not copy subtitle file to target location: ",
        "runtime.could_not_copy_subtitle",
    ),
    (
        "Could not remove temporary subtitle file: ",
        "runtime.could_not_remove_temp_subtitle",
    ),
    (
        "Could not create download folder: ",
        "runtime.could_not_create_download_folder",
    ),
    ("File does not exist: ", "runtime.file_does_not_exist"),
    (
        "File location does not exist: ",
        "runtime.file_location_does_not_exist",
    ),
    ("Could not open file: ", "runtime.could_not_open_file"),
    (
        "Could not open containing folder: ",
        "runtime.could_not_open_containing_folder",
    ),
    ("Could not open folder: ", "runtime.could_not_open_folder"),
    (
        "Thumbnail decode failed: ",
        "runtime.thumbnail_decode_failed",
    ),
    (
        "Invalid thumbnail proxy setting: ",
        "runtime.invalid_thumbnail_proxy",
    ),
    ("Thumbnail load failed: HTTP ", "runtime.thumbnail_http"),
    ("Thumbnail load failed: ", "runtime.thumbnail_load_failed"),
    (
        "Could not create config folder: ",
        "runtime.config_create_folder",
    ),
    (
        "Could not serialize config file: ",
        "runtime.config_serialize",
    ),
    ("Could not write config file: ", "runtime.config_write"),
    (
        "Could not create Windows Toast notifier: ",
        "runtime.toast_create_notifier",
    ),
    (
        "Could not create Windows Toast content: ",
        "runtime.toast_create_content",
    ),
    ("Could not send Windows Toast: ", "runtime.toast_send"),
    (
        "Could not create Windows Toast registration data: ",
        "runtime.toast_create_registration",
    ),
    (
        "Could not register Windows Toast AUMID: ",
        "runtime.toast_register_aumid",
    ),
    (
        "Could not start PowerShell: ",
        "runtime.could_not_start_powershell",
    ),
    (
        "Could not read PowerShell output: ",
        "runtime.could_not_read_powershell_output",
    ),
    (
        "Could not wait for PowerShell to finish: ",
        "runtime.could_not_wait_powershell",
    ),
    (
        "PowerShell failed: exit code ",
        "runtime.powershell_failed_exit",
    ),
    (
        "Could not read yt-dlp playlist output: ",
        "runtime.could_not_read_playlist_output",
    ),
    (
        "yt-dlp batch import failed: ",
        "runtime.batch_import_failed",
    ),
];

fn detect_system_language() -> Language {
    system_locale_name()
        .as_deref()
        .and_then(language_from_locale)
        .unwrap_or(Language::EnUs)
}

fn language_from_locale(locale: &str) -> Option<Language> {
    let locale = locale.trim().replace('_', "-").to_ascii_lowercase();
    if locale.is_empty() {
        return None;
    }

    if locale.starts_with("ar") {
        return Some(Language::ArMa);
    }
    if locale.starts_with("de") {
        return Some(Language::DeDe);
    }
    if locale.starts_with("el") || locale.starts_with("gr") {
        return Some(Language::ElGr);
    }
    if locale.starts_with("en") {
        return Some(Language::EnUs);
    }
    if locale.starts_with("es") {
        return Some(Language::EsEs);
    }
    if locale.starts_with("fr") {
        return Some(Language::FrFr);
    }
    if locale.starts_with("it") {
        return Some(Language::ItIt);
    }
    if locale.starts_with("ja") {
        return Some(Language::JaJp);
    }
    if locale.starts_with("ko") {
        return Some(Language::KoKr);
    }
    if locale.starts_with("pl") {
        return Some(Language::PlPl);
    }
    if locale.starts_with("pt") {
        return Some(Language::PtBr);
    }
    if locale.starts_with("ru") {
        return Some(Language::RuRu);
    }
    if locale.starts_with("uk") {
        return Some(Language::UkUa);
    }
    if locale.starts_with("zh") {
        if locale.contains("cn") || locale.contains("sg") || locale.contains("hans") {
            return Some(Language::ZhCn);
        }
        return Some(Language::ZhTw);
    }

    None
}

#[cfg(windows)]
fn system_locale_name() -> Option<String> {
    const LOCALE_NAME_MAX_LENGTH: usize = 85;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserDefaultLocaleName(lp_locale_name: *mut u16, cch_locale_name: i32) -> i32;
    }

    let mut buffer = [0u16; LOCALE_NAME_MAX_LENGTH];
    let len = unsafe { GetUserDefaultLocaleName(buffer.as_mut_ptr(), buffer.len() as i32) };
    if len <= 1 {
        return None;
    }

    let end = (len as usize).saturating_sub(1).min(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..end]))
}

#[cfg(not(windows))]
fn system_locale_name() -> Option<String> {
    ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(|key| std::env::var(key).ok())
        .map(|value| value.split('.').next().unwrap_or(value.as_str()).to_owned())
}
