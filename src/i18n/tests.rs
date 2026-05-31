use super::{Language, LanguageSelection, catalog};

fn locale_keys(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let rest = trimmed.strip_prefix('"')?;
            let (key, after_key) = rest.split_once('"')?;
            if after_key.trim_start().starts_with("=>") {
                Some(key)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn i18n_picker_exposes_all_compiled_locales() {
    let picker_languages = catalog::PICKER_ORDER
        .iter()
        .filter_map(|selection| match selection {
            LanguageSelection::Auto => None,
            selection => Some(catalog::resolve_selection(*selection)),
        })
        .collect::<Vec<_>>();

    assert_eq!(
        picker_languages.as_slice(),
        catalog::ALL_LANGUAGES.as_slice()
    );
}

#[test]
fn i18n_manual_selections_resolve_to_selected_language() {
    let selections = [
        (LanguageSelection::ArMa, Language::ArMa),
        (LanguageSelection::DeDe, Language::DeDe),
        (LanguageSelection::ElGr, Language::ElGr),
        (LanguageSelection::EnUs, Language::EnUs),
        (LanguageSelection::EsEs, Language::EsEs),
        (LanguageSelection::FrFr, Language::FrFr),
        (LanguageSelection::ItIt, Language::ItIt),
        (LanguageSelection::JaJp, Language::JaJp),
        (LanguageSelection::KoKr, Language::KoKr),
        (LanguageSelection::PlPl, Language::PlPl),
        (LanguageSelection::PtBr, Language::PtBr),
        (LanguageSelection::RuRu, Language::RuRu),
        (LanguageSelection::UkUa, Language::UkUa),
        (LanguageSelection::ZhCn, Language::ZhCn),
        (LanguageSelection::ZhTw, Language::ZhTw),
    ];

    for (selection, language) in selections {
        assert_eq!(
            catalog::resolve_selection(selection),
            language,
            "manual selection {:?} should resolve to its matching language",
            selection
        );
    }
}

#[test]
fn i18n_auto_detection_accepts_compiled_locales() {
    assert_eq!(catalog::language_from_locale("ar-MA"), Some(Language::ArMa));
    assert_eq!(catalog::language_from_locale("de-DE"), Some(Language::DeDe));
    assert_eq!(catalog::language_from_locale("el-GR"), Some(Language::ElGr));
    assert_eq!(catalog::language_from_locale("en-US"), Some(Language::EnUs));
    assert_eq!(catalog::language_from_locale("es-ES"), Some(Language::EsEs));
    assert_eq!(catalog::language_from_locale("fr-FR"), Some(Language::FrFr));
    assert_eq!(catalog::language_from_locale("it-IT"), Some(Language::ItIt));
    assert_eq!(catalog::language_from_locale("ja-JP"), Some(Language::JaJp));
    assert_eq!(catalog::language_from_locale("ko-KR"), Some(Language::KoKr));
    assert_eq!(catalog::language_from_locale("pl-PL"), Some(Language::PlPl));
    assert_eq!(catalog::language_from_locale("pt-BR"), Some(Language::PtBr));
    assert_eq!(catalog::language_from_locale("ru-RU"), Some(Language::RuRu));
    assert_eq!(catalog::language_from_locale("uk-UA"), Some(Language::UkUa));
    assert_eq!(
        catalog::language_from_locale("zh-Hant-TW"),
        Some(Language::ZhTw)
    );
    assert_eq!(
        catalog::language_from_locale("zh-Hans-CN"),
        Some(Language::ZhCn)
    );

    assert_eq!(catalog::language_from_locale("th-TH"), None);
}

// Locale key coverage check.
//
// This checks structure only: every compiled locale must contain exactly
// the same keys, in the same order, as canonical en-US. It does not judge
// translation wording and it does not scan raw tool output.
#[test]
fn i18n_keys() {
    let canonical = locale_keys(include_str!("en_us.rs"));
    assert!(!canonical.is_empty(), "en_us.rs should define i18n keys");

    let release_locales = [
        ("ar_ma.rs", locale_keys(include_str!("ar_ma.rs"))),
        ("de_de.rs", locale_keys(include_str!("de_de.rs"))),
        ("el_gr.rs", locale_keys(include_str!("el_gr.rs"))),
        ("es_es.rs", locale_keys(include_str!("es_es.rs"))),
        ("fr_fr.rs", locale_keys(include_str!("fr_fr.rs"))),
        ("it_it.rs", locale_keys(include_str!("it_it.rs"))),
        ("ja_jp.rs", locale_keys(include_str!("ja_jp.rs"))),
        ("ko_kr.rs", locale_keys(include_str!("ko_kr.rs"))),
        ("pl_pl.rs", locale_keys(include_str!("pl_pl.rs"))),
        ("pt_br.rs", locale_keys(include_str!("pt_br.rs"))),
        ("ru_ru.rs", locale_keys(include_str!("ru_ru.rs"))),
        ("uk_ua.rs", locale_keys(include_str!("uk_ua.rs"))),
        ("zh_cn.rs", locale_keys(include_str!("zh_cn.rs"))),
        ("zh_tw.rs", locale_keys(include_str!("zh_tw.rs"))),
    ];

    for (locale_name, keys) in release_locales {
        assert_eq!(
            keys, canonical,
            "{locale_name} must have the same keys, in the same order, as en_us.rs"
        );
    }
}

fn collect_rs_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) == Some("i18n") {
                continue;
            }
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn collect_literal_i18n_calls(
    source: &str,
    call_prefix: &str,
    keys: &[&str],
    relative_path: &str,
    missing: &mut Vec<String>,
) {
    let mut search_start = 0;
    while let Some(offset) = source[search_start..].find(call_prefix) {
        let literal_start = search_start + offset + call_prefix.len();
        let Some(literal_end_offset) = source[literal_start..].find('"') else {
            break;
        };
        let key = &source[literal_start..literal_start + literal_end_offset];
        if !keys.contains(&key) {
            let line = source[..literal_start].lines().count();
            missing.push(format!("{relative_path}:{line}: {key}"));
        }
        search_start = literal_start + literal_end_offset + 1;
    }
}

// Direct literal i18n calls must reference keys that exist in en_us.rs.
// This prevents mistakes such as `state.tr("Add")`, which would show the raw
// English text in non-English UI instead of translating through `action.add`.
//
// During the zone-by-zone migration, both legacy .tr/.trf and new .ui_tr/.ui_trf
// calls are accepted, but they all must point to canonical locale keys.
#[test]
fn i18n_used_keys() {
    let keys = locale_keys(include_str!("en_us.rs"));
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = root.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    let mut missing = Vec::new();
    for path in files {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        collect_literal_i18n_calls(&source, ".ui_tr(\"", &keys, &relative_path, &mut missing);
        collect_literal_i18n_calls(&source, ".ui_trf(\"", &keys, &relative_path, &mut missing);
        collect_literal_i18n_calls(&source, ".tr(\"", &keys, &relative_path, &mut missing);
        collect_literal_i18n_calls(&source, ".trf(\"", &keys, &relative_path, &mut missing);
    }

    assert!(
        missing.is_empty(),
        "direct literal i18n calls use undefined keys:\n{}",
        missing.join("\n")
    );
}
