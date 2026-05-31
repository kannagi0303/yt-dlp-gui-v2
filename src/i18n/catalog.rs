use super::{Language, LanguageSelection};

// User-facing language picker order.
//
// All compiled locale tables are shown in the language picker. Each locale
// table keeps full canonical key coverage; English fallback remains only as
// a final safety net for unknown keys.
pub(super) const PICKER_ORDER: [LanguageSelection; 16] = [
    LanguageSelection::Auto,
    LanguageSelection::ArMa,
    LanguageSelection::DeDe,
    LanguageSelection::ElGr,
    LanguageSelection::EnUs,
    LanguageSelection::EsEs,
    LanguageSelection::FrFr,
    LanguageSelection::ItIt,
    LanguageSelection::JaJp,
    LanguageSelection::KoKr,
    LanguageSelection::PlPl,
    LanguageSelection::PtBr,
    LanguageSelection::RuRu,
    LanguageSelection::UkUa,
    LanguageSelection::ZhCn,
    LanguageSelection::ZhTw,
];

// Locales with strict key-order coverage in `cargo test i18n_keys`.
// All compiled locale tables must keep the canonical en-US key order.
pub(super) const RELEASE_LANGUAGES: [Language; 15] = ALL_LANGUAGES;

pub(super) const ALL_LANGUAGES: [Language; 15] = [
    Language::ArMa,
    Language::DeDe,
    Language::ElGr,
    Language::EnUs,
    Language::EsEs,
    Language::FrFr,
    Language::ItIt,
    Language::JaJp,
    Language::KoKr,
    Language::PlPl,
    Language::PtBr,
    Language::RuRu,
    Language::UkUa,
    Language::ZhCn,
    Language::ZhTw,
];

pub(super) fn resolve_selection(selection: LanguageSelection) -> Language {
    match selection {
        LanguageSelection::Auto => Language::EnUs,
        LanguageSelection::ArMa => Language::ArMa,
        LanguageSelection::DeDe => Language::DeDe,
        LanguageSelection::ElGr => Language::ElGr,
        LanguageSelection::EnUs => Language::EnUs,
        LanguageSelection::EsEs => Language::EsEs,
        LanguageSelection::FrFr => Language::FrFr,
        LanguageSelection::ItIt => Language::ItIt,
        LanguageSelection::JaJp => Language::JaJp,
        LanguageSelection::KoKr => Language::KoKr,
        LanguageSelection::PlPl => Language::PlPl,
        LanguageSelection::PtBr => Language::PtBr,
        LanguageSelection::RuRu => Language::RuRu,
        LanguageSelection::UkUa => Language::UkUa,
        LanguageSelection::ZhCn => Language::ZhCn,
        LanguageSelection::ZhTw => Language::ZhTw,
    }
}

pub(super) fn language_from_locale(locale: &str) -> Option<Language> {
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
    if locale.starts_with("el") {
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
