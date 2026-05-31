// i18n boundary:
//
// The app UI should speak the selected language: labels, buttons, tabs,
// dialogs, and direct visible UI labels.
//
// i18n-exempt by design:
// - raw yt-dlp / ffmpeg / ffprobe / deno output
// - raw command lines, CLI option names, and CLI argument values
// - codec, container, extension, and media format tokens
// - config keys, internal enum names, and developer/debug logs
// - runtime/internal messages, process errors, and generated diagnostics
//
// Keep external tool text and runtime diagnostics fixed English so users can
// copy, search, and compare them with upstream tool behavior. Localize the app
// chrome and intentional UI copy around them instead.

use serde::{Deserialize, Serialize};

mod catalog;

#[cfg(test)]
mod tests;

// Locale modules may include draft/future translations.
// All compiled locales are selectable; missing keys fall back to English.
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
    // User-facing locale picker order.
    //
    // The picker order lives in catalog.rs so future i18n work can adjust
    // language scope without touching translation tables.
    pub const PICKER_ORDER: [Self; 16] = catalog::PICKER_ORDER;

    pub fn resolve(self) -> Language {
        match self {
            Self::Auto => detect_system_language(),
            selection => catalog::resolve_selection(selection),
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
    // All compiled locale tables.
    pub const ALL: [Self; 15] = catalog::ALL_LANGUAGES;

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

pub fn format_fixed_english(template: &str, args: &[(&str, &str)]) -> String {
    let mut output = template.to_owned();
    for (placeholder, value) in args {
        output = output.replace(placeholder, value);
    }
    output
}

pub fn localize_message(language: Language, value: &str) -> String {
    // Internal/runtime messages are intentionally fixed English.
    //
    // This function only treats literal i18n keys as UI text. Any other value is
    // returned unchanged so external tool output, diagnostic details, file paths,
    // process errors, and internal status messages do not expand the locale tables.
    let direct = text(language, value);
    if direct != value {
        direct.to_owned()
    } else {
        value.to_owned()
    }
}

fn detect_system_language() -> Language {
    system_locale_name()
        .as_deref()
        .and_then(catalog::language_from_locale)
        .unwrap_or(Language::EnUs)
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
