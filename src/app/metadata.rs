use std::path::Path;

use serde_json::Value;

use crate::domain::{
    ChapterOption, FormatOption, MediaKind, SubtitleOption, SubtitleSource, VideoMetadata,
};
use crate::infrastructure::playlist_entry_url;

#[derive(Clone)]
pub(super) struct PlaylistEntrySeed {
    pub source_url: String,
    pub title: String,
    pub thumbnail_url: String,
    pub thumbnail_hint: String,
    pub duration_text: String,
}

pub(super) fn infer_title(url: &str, untitled: &str, imported_template: &str) -> String {
    let tail = url.rsplit('/').next().unwrap_or(url);
    if tail.is_empty() {
        untitled.to_owned()
    } else {
        imported_template.replace("{tail}", tail)
    }
}

pub(super) fn extract_chapters(
    json: &Value,
    fallback_title: impl Fn(usize) -> String,
) -> Vec<ChapterOption> {
    let Some(chapters) = json.get("chapters").and_then(Value::as_array) else {
        return Vec::new();
    };

    chapters
        .iter()
        .enumerate()
        .filter_map(|(index, chapter)| {
            let start = chapter.get("start_time").and_then(Value::as_f64)?;
            if !start.is_finite() || start < 0.0 {
                return None;
            }

            let end = chapter
                .get("end_time")
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite() && *value > start);
            let start_text = format_section_timestamp(start);
            let end_text = end.map(format_section_timestamp);
            let download_sections = match end_text.as_deref() {
                Some(end) => format!("*{start_text}-{end}"),
                None => format!("*{start_text}-"),
            };
            let title = chapter
                .get("title")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| fallback_title(index));

            Some(ChapterOption::new(
                format!("chapter:{index}"),
                title,
                start_text,
                end_text,
                download_sections,
            ))
        })
        .collect()
}

fn format_section_timestamp(seconds: f64) -> String {
    let total_millis = (seconds.max(0.0) * 1000.0).round() as u64;
    let millis = total_millis % 1000;
    let total_seconds = total_millis / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if millis == 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
    }
}

pub(super) fn extract_formats(json: &Value) -> Vec<FormatOption> {
    let mut items = Vec::new();

    if let Some(formats) = json.get("formats").and_then(Value::as_array) {
        for format in formats {
            let Some(format_id) = format.get("format_id").and_then(Value::as_str) else {
                continue;
            };
            let kind = detect_media_kind(format);
            if kind == MediaKind::Other {
                continue;
            }

            items.push(build_format_option(format_id, format, kind));
        }
    }

    if let Some(subtitles) = json.get("subtitles").and_then(Value::as_object) {
        items.push(FormatOption::new("ignore", "[Ignore]", MediaKind::Subtitle));
        for key in subtitles.keys() {
            items.push(FormatOption::new(key, key, MediaKind::Subtitle));
        }
    }

    items
}

fn detect_media_kind(format: &Value) -> MediaKind {
    let vcodec = format
        .get("vcodec")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let acodec = format
        .get("acodec")
        .and_then(Value::as_str)
        .unwrap_or_default();

    match (vcodec == "none", acodec == "none") {
        (false, false) => MediaKind::Muxed,
        (false, true) => MediaKind::Video,
        (true, false) => MediaKind::Audio,
        (true, true) => MediaKind::Other,
    }
}

fn build_format_label(format: &Value, kind: MediaKind) -> String {
    match kind {
        MediaKind::Video | MediaKind::Muxed => build_video_label(format),
        MediaKind::Audio => build_audio_label(format),
        MediaKind::Subtitle => format
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        MediaKind::Other => format
            .get("format")
            .or_else(|| format.get("format_note"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    }
}

fn build_format_option(format_id: &str, format: &Value, kind: MediaKind) -> FormatOption {
    match kind {
        MediaKind::Video | MediaKind::Muxed => {
            let resolution = video_resolution_text(format);
            let dynamic_range = format
                .get("dynamic_range")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let fps = format
                .get("fps")
                .and_then(value_as_number_text)
                .map(|value| format!("{value}fps"))
                .unwrap_or_default();
            let ext = format
                .get("video_ext")
                .or_else(|| format.get("ext"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let codec = normalize_video_codec(
                format
                    .get("vcodec")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            );
            let filesize = format_filesize(format);
            let label =
                join_label_parts(&[&resolution, &dynamic_range, &fps, &ext, &codec, &filesize]);

            FormatOption::video(
                format_id,
                &label,
                kind,
                &resolution,
                &dynamic_range,
                &fps,
                &ext,
                &codec,
                &filesize,
            )
        }
        MediaKind::Audio => {
            let sample_rate = format
                .get("asr")
                .and_then(Value::as_i64)
                .map(|value| format!("{value}Hz"))
                .unwrap_or_else(|| "Unknow".to_owned());
            let ext = format
                .get("audio_ext")
                .or_else(|| format.get("ext"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let codec = normalize_audio_codec(
                format
                    .get("acodec")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            );
            let filesize = format_filesize(format);
            let label = join_label_parts(&[&sample_rate, &ext, &codec, &filesize]);

            FormatOption::audio(
                format_id,
                &label,
                kind,
                &sample_rate,
                &ext,
                &codec,
                &filesize,
            )
        }
        MediaKind::Subtitle | MediaKind::Other => {
            let label = build_format_label(format, kind);
            FormatOption::new(format_id, &label, kind)
        }
    }
}

fn build_video_label(format: &Value) -> String {
    let resolution = video_resolution_text(format);
    let dynamic_range = format
        .get("dynamic_range")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let fps = format
        .get("fps")
        .and_then(value_as_number_text)
        .map(|value| format!("{value}fps"))
        .unwrap_or_default();
    let video_ext = format
        .get("video_ext")
        .or_else(|| format.get("ext"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let vcodec = normalize_video_codec(
        format
            .get("vcodec")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    );
    let filesize = format_filesize(format);

    join_label_parts(&[
        &resolution,
        dynamic_range,
        &fps,
        video_ext,
        &vcodec,
        &filesize,
    ])
}

fn video_resolution_text(format: &Value) -> String {
    if let (Some(width), Some(height)) = (
        format.get("width").and_then(Value::as_i64),
        format.get("height").and_then(Value::as_i64),
    ) {
        format!("{width}x{height}")
    } else {
        format
            .get("resolution")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    }
}

fn build_audio_label(format: &Value) -> String {
    let asr = format
        .get("asr")
        .and_then(Value::as_i64)
        .map(|value| format!("{value}Hz"))
        .unwrap_or_else(|| "Unknow".to_owned());
    let audio_ext = format
        .get("audio_ext")
        .or_else(|| format.get("ext"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let acodec = normalize_audio_codec(
        format
            .get("acodec")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    );
    let filesize = format_filesize(format);

    join_label_parts(&[&asr, audio_ext, &acodec, &filesize])
}

fn value_as_number_text(value: &Value) -> Option<String> {
    match value {
        Value::Number(number) => {
            if let Some(int) = number.as_i64() {
                Some(int.to_string())
            } else {
                let text = number.to_string();
                Some(text.trim_end_matches('0').trim_end_matches('.').to_owned())
            }
        }
        _ => None,
    }
}

fn normalize_video_codec(value: &str) -> String {
    if value.starts_with("vp9") {
        "VP9".to_owned()
    } else if value.starts_with("av01") {
        "AV1".to_owned()
    } else if value.starts_with("avc") {
        "H.264".to_owned()
    } else {
        value.to_owned()
    }
}

fn normalize_audio_codec(value: &str) -> String {
    if value.starts_with("mp4a") {
        "AAC".to_owned()
    } else if value.starts_with("opus") {
        "OPUS".to_owned()
    } else {
        value.to_owned()
    }
}

fn format_filesize(format: &Value) -> String {
    let bytes = format
        .get("filesize")
        .and_then(Value::as_u64)
        .or_else(|| format.get("filesize_approx").and_then(Value::as_u64));

    bytes.map(human_size).unwrap_or_default()
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;

    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{size:.2} {}", UNITS[unit])
    }
}

fn join_label_parts(parts: &[&str]) -> String {
    parts
        .iter()
        .copied()
        .filter(|part| !part.is_empty() && *part != "none")
        .collect::<Vec<_>>()
        .join("  ")
}

pub(super) fn default_format_id(formats: &[FormatOption], preferred_kinds: &[MediaKind]) -> String {
    for kind in preferred_kinds {
        if let Some(format) = formats
            .iter()
            .filter(|item| item.kind == *kind)
            .max_by(|left, right| compare_default_format_quality(left, right, *kind))
        {
            return format.id.clone();
        }
    }
    String::new()
}

fn compare_default_format_quality(
    left: &FormatOption,
    right: &FormatOption,
    kind: MediaKind,
) -> std::cmp::Ordering {
    match kind {
        MediaKind::Video | MediaKind::Muxed => video_resolution_area(left)
            .cmp(&video_resolution_area(right))
            .then_with(|| human_size_bytes(&left.filesize).cmp(&human_size_bytes(&right.filesize))),
        MediaKind::Audio => {
            human_size_bytes(&left.filesize).cmp(&human_size_bytes(&right.filesize))
        }
        _ => std::cmp::Ordering::Equal,
    }
}

pub(super) fn extract_requested_ids(json: &Value) -> Vec<String> {
    json.get("requested_downloads")
        .and_then(Value::as_array)
        .or_else(|| json.get("requested_formats").and_then(Value::as_array))
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("format_id").and_then(Value::as_str))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn extract_requested_filename(json: &Value) -> Option<String> {
    json.get("requested_downloads")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| {
            item.get("filename")
                .or_else(|| item.get("_filename"))
                .and_then(Value::as_str)
        })
        .map(ToOwned::to_owned)
}

pub(super) fn display_file_stem(path_or_name: &str) -> String {
    Path::new(path_or_name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path_or_name)
        .to_owned()
}

pub fn sanitize_file_name_for_windows(file_name: &str) -> String {
    file_name
        .chars()
        .map(|ch| {
            if is_forbidden_file_name_char(ch) || ch.is_control() {
                '_'
            } else {
                ch
            }
        })
        .collect()
}

fn is_forbidden_file_name_char(ch: char) -> bool {
    matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
}

pub(super) fn extract_subtitle_tracks(json: &Value) -> Vec<SubtitleOption> {
    let mut items = Vec::new();
    items.extend(extract_subtitle_group(
        json.get("subtitles"),
        SubtitleSource::Original,
    ));
    items.extend(extract_subtitle_group(
        json.get("automatic_captions"),
        SubtitleSource::Automatic,
    ));
    items
}

fn extract_subtitle_group(value: Option<&Value>, source: SubtitleSource) -> Vec<SubtitleOption> {
    let Some(object) = value.and_then(Value::as_object) else {
        return Vec::new();
    };

    let mut tracks = Vec::new();
    for (language_code, entries) in object {
        let Some(entry) = preferred_subtitle_entry(entries) else {
            continue;
        };
        let ext = entry
            .get("ext")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let url = entry
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        if url.is_empty() {
            continue;
        }

        let fallback_label = entry
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(language_code);
        let source_language_code =
            extract_url_query_param(&url, "lang").unwrap_or_else(|| language_code.to_owned());
        let target_language_code = extract_url_query_param(&url, "tlang");
        let source_language_label = if source == SubtitleSource::Original {
            fallback_label.to_owned()
        } else if target_language_code.is_some() {
            language_name_from_tracks(object, &source_language_code)
                .unwrap_or_else(|| source_language_code.clone())
        } else {
            fallback_label.to_owned()
        };
        let target_language_label = match &target_language_code {
            Some(_) => Some(fallback_label.to_owned()),
            None => None,
        };

        tracks.push(SubtitleOption::new(
            format!(
                "{}:{}:{}",
                source.key(),
                source_language_code,
                target_language_code.as_deref().unwrap_or("none")
            ),
            source,
            language_code.clone(),
            source_language_code,
            source_language_label,
            target_language_code,
            target_language_label,
            ext,
            url,
        ));
    }

    tracks.sort_by(|left, right| {
        left.source_label()
            .cmp(&right.source_label())
            .then_with(|| left.target_label().cmp(&right.target_label()))
    });
    tracks
}

fn preferred_subtitle_entry(entries: &Value) -> Option<&Value> {
    let values = entries.as_array()?;
    for preferred_ext in ["vtt", "srt", "ttml", "srv3", "srv2", "srv1", "json3"] {
        if let Some(entry) = values.iter().find(|item| {
            item.get("ext")
                .and_then(Value::as_str)
                .map(|ext| ext == preferred_ext)
                .unwrap_or(false)
        }) {
            return Some(entry);
        }
    }
    values.first()
}

fn extract_url_query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split('?').nth(1)?;
    for pair in query.split('&') {
        let (current_key, current_value) = pair.split_once('=')?;
        if current_key == key && !current_value.is_empty() {
            return Some(current_value.to_owned());
        }
    }
    None
}

fn language_name_from_tracks(
    object: &serde_json::Map<String, Value>,
    language_code: &str,
) -> Option<String> {
    let entries = object.get(language_code)?.as_array()?;
    let cloned_entries = Value::Array(entries.clone());
    let entry = preferred_subtitle_entry(&cloned_entries)?;
    entry
        .get("name")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

pub(super) fn select_best_thumbnail_url(json: &Value) -> Option<String> {
    let best_from_list = json
        .get("thumbnails")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let url = item.get("url").and_then(Value::as_str)?.trim();
                    if url.is_empty() {
                        return None;
                    }

                    let width = item.get("width").and_then(Value::as_u64).unwrap_or(0);
                    let height = item.get("height").and_then(Value::as_u64).unwrap_or(0);
                    let preference = item
                        .get("preference")
                        .and_then(Value::as_i64)
                        .unwrap_or(i64::MIN);
                    let area = width.saturating_mul(height);

                    Some((preference, area, url.to_owned()))
                })
                .max_by(|left, right| {
                    left.0
                        .cmp(&right.0)
                        .then_with(|| left.1.cmp(&right.1))
                        .then_with(|| left.2.cmp(&right.2))
                })
                .map(|(_, _, url)| url)
        });

    let raw_thumbnail = json
        .get("thumbnail")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(ToOwned::to_owned);

    best_from_list
        .or_else(|| youtube_thumbnail_fallbacks(json).into_iter().next())
        .or(raw_thumbnail)
}

fn youtube_thumbnail_fallbacks(json: &Value) -> Vec<String> {
    let Some(video_id) = json
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Vec::new();
    };

    let Some(extractor) = json
        .get("extractor_key")
        .or_else(|| json.get("extractor"))
        .and_then(Value::as_str)
        .map(|value| value.to_ascii_lowercase())
    else {
        return Vec::new();
    };

    if !extractor.contains("youtube") {
        return Vec::new();
    }

    [
        "maxresdefault.jpg",
        "sddefault.jpg",
        "hqdefault.jpg",
        "mqdefault.jpg",
        "default.jpg",
    ]
    .into_iter()
    .map(|file_name| format!("https://i.ytimg.com/vi/{video_id}/{file_name}"))
    .collect()
}

pub(super) fn requested_or_default_format_id(
    formats: &[FormatOption],
    requested_ids: &[String],
    preferred_kinds: &[MediaKind],
) -> String {
    for kind in preferred_kinds {
        if let Some(format) = formats
            .iter()
            .find(|item| item.kind == *kind && requested_ids.iter().any(|id| id == &item.id))
        {
            return format.id.clone();
        }
    }

    default_format_id(formats, preferred_kinds)
}

pub(super) fn first_audio_format_id(metadata: Option<&VideoMetadata>) -> Option<String> {
    metadata
        .into_iter()
        .flat_map(|metadata| metadata.formats.iter())
        .find(|format| format.kind == MediaKind::Audio)
        .map(|format| format.id.clone())
}

pub(super) fn video_resolution_area(option: &FormatOption) -> u64 {
    let Some((width, height)) = option.resolution.split_once('x') else {
        return 0;
    };
    let Ok(width) = width.parse::<u64>() else {
        return 0;
    };
    let Ok(height) = height.parse::<u64>() else {
        return 0;
    };
    width.saturating_mul(height)
}

pub(super) fn human_size_bytes(text: &str) -> u64 {
    let mut parts = text.split_whitespace();
    let Some(value_text) = parts.next() else {
        return 0;
    };
    let Some(unit) = parts.next() else {
        return 0;
    };
    let Ok(value) = value_text.parse::<f64>() else {
        return 0;
    };
    let multiplier = match unit {
        "B" => 1.0,
        "KB" => 1024.0,
        "MB" => 1024.0 * 1024.0,
        "GB" => 1024.0 * 1024.0 * 1024.0,
        "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 0.0,
    };
    (value * multiplier) as u64
}

pub(super) fn playlist_entry_seed_from_json(
    entry: &Value,
    untitled: &str,
    imported_template: &str,
) -> Option<PlaylistEntrySeed> {
    let source_url = playlist_entry_url(entry)?;
    let inferred_title = infer_title(&source_url, untitled, imported_template);
    let title = entry
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(inferred_title.as_str())
        .to_owned();
    let thumbnail_url = select_best_thumbnail_url(entry).unwrap_or_default();
    let thumbnail_hint = if thumbnail_url.is_empty() {
        "item.thumbnail".to_owned()
    } else {
        "item.thumbnail_preview".to_owned()
    };
    let duration_text = entry
        .get("duration_string")
        .and_then(Value::as_str)
        .map(normalize_duration_badge_text)
        .or_else(|| {
            entry
                .get("duration")
                .and_then(Value::as_f64)
                .map(|value| format_duration_badge(value.round() as u64))
        })
        .unwrap_or_default();

    Some(PlaylistEntrySeed {
        source_url,
        title,
        thumbnail_url,
        thumbnail_hint,
        duration_text,
    })
}

fn format_duration_badge(total_seconds: u64) -> String {
    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{days}:{hours:02}:{minutes:02}:{seconds:02}")
    } else if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

pub(super) fn normalize_duration_badge_text(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let parts = trimmed.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        [seconds] => seconds
            .parse::<u64>()
            .map(format_duration_badge)
            .unwrap_or_else(|_| trimmed.to_owned()),
        [minutes, seconds] => {
            if minutes.trim().is_empty() {
                format!("0:{}", seconds.trim())
            } else {
                trimmed.to_owned()
            }
        }
        _ => trimmed.to_owned(),
    }
}
