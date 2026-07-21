use super::*;

pub(super) fn stable_media_session_title(title: &str, source_url: &str) -> String {
    let trimmed = title.trim();
    if !trimmed.is_empty() && !is_transient_media_session_title(trimmed) {
        return trimmed.to_owned();
    }

    let source = source_url.trim();
    if source.is_empty() {
        "Audio".to_owned()
    } else {
        source.to_owned()
    }
}

pub(super) fn is_transient_media_session_title(value: &str) -> bool {
    const STATUS_KEYS: &[&str] = &[
        "music.status.resolving",
        "music.status.buffering",
        "music.status.ready",
        "music.status.caching",
        "music.status.playing",
    ];

    let trimmed = value.trim();
    STATUS_KEYS.iter().any(|key| {
        Language::ALL
            .iter()
            .any(|language| i18n::text(*language, key) == trimmed)
    }) || matches!(trimmed, "Buffering...")
}

pub(super) fn split_artist_title_for_media_session(value: &str) -> (String, String) {
    let trimmed = value.trim();
    for separator in [" - ", " – ", " — "] {
        if let Some((artist, title)) = trimmed.split_once(separator) {
            let artist = artist.trim();
            let title = title.trim();
            if !artist.is_empty() && !title.is_empty() {
                return (artist.to_owned(), title.to_owned());
            }
        }
    }
    (String::new(), trimmed.to_owned())
}

pub(super) fn duration_text_to_seconds(text: &str) -> Option<f64> {
    let mut total = 0_u64;
    let mut saw_part = false;
    for part in text.trim().split(':') {
        let value = part.trim().parse::<u64>().ok()?;
        total = total.saturating_mul(60).saturating_add(value);
        saw_part = true;
    }
    saw_part
        .then_some(total as f64)
        .filter(|value| *value > 0.0)
}

pub(super) fn music_stream_seed_from_json(
    json: &Value,
    source: &str,
) -> Result<MusicStreamSeed, String> {
    let requested = json
        .get("requested_downloads")
        .and_then(Value::as_array)
        .and_then(|items| items.first());

    let direct_url = requested
        .and_then(|value| json_str_field(value, "url"))
        .or_else(|| json_str_field(json, "url"))
        .ok_or_else(|| "yt-dlp did not return a playable audio stream URL.".to_owned())?;

    let title = json_str_field(json, "title")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| source.to_owned());
    let album_title = music_album_title_from_json(json);
    let duration_seconds = json_f64_field(json, "duration");
    let duration_text = json_str_field(json, "duration_string")
        .as_deref()
        .map(normalize_duration_badge_text)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| duration_seconds.map(format_duration_seconds))
        .unwrap_or_default();
    let thumbnail_url = select_largest_thumbnail_url(json).unwrap_or_default();
    let ext = requested
        .and_then(|value| json_str_field(value, "ext"))
        .or_else(|| json_str_field(json, "ext"))
        .unwrap_or_default();
    let format_id = requested
        .and_then(|value| json_str_field(value, "format_id"))
        .or_else(|| json_str_field(json, "format_id"))
        .unwrap_or_default();
    let acodec = requested
        .and_then(|value| json_str_field(value, "acodec"))
        .or_else(|| json_str_field(json, "acodec"))
        .unwrap_or_default();
    let expected_bytes = requested
        .and_then(|value| json_u64_field(value, "filesize"))
        .or_else(|| requested.and_then(|value| json_u64_field(value, "filesize_approx")))
        .or_else(|| json_u64_field(json, "filesize"))
        .or_else(|| json_u64_field(json, "filesize_approx"));
    let cache_key = music_cache_key(source, &format_id, &ext, &acodec);
    let headers = requested
        .and_then(|value| value.get("http_headers"))
        .and_then(headers_from_json)
        .or_else(|| json.get("http_headers").and_then(headers_from_json))
        .unwrap_or_default();

    Ok(MusicStreamSeed {
        source_url: source.to_owned(),
        title,
        album_title,
        thumbnail_url,
        thumbnail_hint: "item.thumbnail".to_owned(),
        duration_text,
        duration_seconds,
        direct_url,
        headers,
        ext,
        format_id,
        acodec,
        expected_bytes,
        cache_key,
        lyrics_track: primary_original_subtitle_track_from_json(json),
    })
}

pub(super) fn primary_original_subtitle_track_from_json(json: &Value) -> Option<SubtitleOption> {
    let tracks = extract_subtitle_tracks(json);
    primary_original_subtitle_track_from_tracks(json, tracks.into_iter())
}

pub(super) fn primary_original_subtitle_track_from_metadata(
    metadata: &VideoMetadata,
) -> Option<&SubtitleOption> {
    let original_tracks = metadata
        .subtitle_tracks
        .iter()
        .filter(|track| is_direct_original_subtitle_track(track))
        .collect::<Vec<_>>();
    if original_tracks.is_empty() {
        return None;
    }

    let preferred_languages = metadata_language_candidates_from_metadata(metadata);
    for language in preferred_languages {
        if let Some(track) = original_tracks.iter().find(|track| {
            subtitle_language_matches(&track.download_language_code, &language)
                || subtitle_language_matches(&track.source_language_code, &language)
        }) {
            return Some(*track);
        }
    }

    original_tracks.into_iter().next()
}

pub(super) fn primary_original_subtitle_track_from_tracks(
    json: &Value,
    tracks: impl Iterator<Item = SubtitleOption>,
) -> Option<SubtitleOption> {
    let original_tracks = tracks
        .filter(is_direct_original_subtitle_track)
        .collect::<Vec<_>>();
    if original_tracks.is_empty() {
        return None;
    }

    let preferred_languages = metadata_language_candidates(json);
    for language in preferred_languages {
        if let Some(track) = original_tracks.iter().find(|track| {
            subtitle_language_matches(&track.download_language_code, &language)
                || subtitle_language_matches(&track.source_language_code, &language)
        }) {
            return Some(track.clone());
        }
    }

    original_tracks.into_iter().next()
}

pub(super) fn is_direct_original_subtitle_track(track: &SubtitleOption) -> bool {
    track.source == SubtitleSource::Original && track.target_language_code.is_none()
}

pub(super) fn metadata_language_candidates(json: &Value) -> Vec<String> {
    let mut languages = Vec::new();
    for key in ["language", "original_language", "lang"] {
        if let Some(value) = json_str_field(json, key) {
            push_unique_language(&mut languages, value);
        }
    }
    push_text_inferred_language_candidates(
        &mut languages,
        [
            "track",
            "title",
            "fulltitle",
            "alt_title",
            "artist",
            "artists",
            "creator",
            "channel",
            "uploader",
        ]
        .into_iter()
        .filter_map(|key| json_str_field(json, key)),
    );
    languages
}

pub(super) fn metadata_language_candidates_from_metadata(metadata: &VideoMetadata) -> Vec<String> {
    let mut languages = Vec::new();
    push_text_inferred_language_candidates(
        &mut languages,
        [
            metadata.title.as_str(),
            metadata.creator.as_str(),
            metadata.channel.as_str(),
            metadata.uploader.as_str(),
        ],
    );
    languages
}

pub(super) fn push_text_inferred_language_candidates<T: AsRef<str>>(
    languages: &mut Vec<String>,
    texts: impl IntoIterator<Item = T>,
) {
    let mut saw_japanese_kana = false;
    let mut saw_hangul = false;
    let mut saw_thai = false;

    for text in texts {
        for ch in text.as_ref().chars() {
            let code = ch as u32;
            saw_japanese_kana |= (0x3040..=0x309f).contains(&code)
                || (0x30a0..=0x30ff).contains(&code)
                || (0xff66..=0xff9d).contains(&code);
            saw_hangul |= (0xac00..=0xd7af).contains(&code)
                || (0x1100..=0x11ff).contains(&code)
                || (0x3130..=0x318f).contains(&code);
            saw_thai |= (0x0e00..=0x0e7f).contains(&code);
        }
    }

    if saw_japanese_kana {
        push_unique_language(languages, "ja".to_owned());
    }
    if saw_hangul {
        push_unique_language(languages, "ko".to_owned());
    }
    if saw_thai {
        push_unique_language(languages, "th".to_owned());
    }
}

pub(super) fn push_unique_language(languages: &mut Vec<String>, value: String) {
    let normalized = normalize_subtitle_language_code(&value);
    if normalized.is_empty() {
        return;
    }
    if !languages.iter().any(|item| item == &normalized) {
        languages.push(normalized);
    }
}

pub(super) fn subtitle_language_matches(left: &str, right: &str) -> bool {
    let left = normalize_subtitle_language_code(left);
    let right = normalize_subtitle_language_code(right);
    !left.is_empty()
        && !right.is_empty()
        && (left == right
            || left.starts_with(&format!("{right}-"))
            || right.starts_with(&format!("{left}-")))
}

pub(super) fn normalize_subtitle_language_code(value: &str) -> String {
    let normalized = value.trim().replace('_', "-").to_ascii_lowercase();
    match normalized.as_str() {
        "jp" | "jpn" | "japanese" => "ja".to_owned(),
        "kr" | "kor" | "korean" => "ko".to_owned(),
        "cn" | "chi" | "zho" | "chinese" => "zh".to_owned(),
        "tw" => "zh-tw".to_owned(),
        _ => normalized,
    }
}

pub(super) fn complete_music_cache_media_path_in_root(
    item: &QueueItem,
    cache_root: &Path,
) -> Option<PathBuf> {
    if item.music_cache_key.trim().is_empty() || item.music_stream_ext.trim().is_empty() {
        return None;
    }
    let cache_dir = cache_root.join(sanitize_music_cache_key(&item.music_cache_key));
    let manifest_path = cache_dir.join("manifest.yaml");
    let manifest = read_yaml_file::<AudioCacheManifestSnapshot>(&manifest_path)?;
    if !audio_cache_manifest_is_fresh(&manifest) {
        let _ = fs::remove_dir_all(&cache_dir);
        return None;
    }
    if !manifest.complete {
        return None;
    }
    let path = cache_dir.join(format!(
        "audio.{}",
        sanitize_music_cache_ext(&item.music_stream_ext)
    ));
    let media_len = fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
    if media_len == 0 {
        return None;
    }
    let expected_bytes = manifest.expected_bytes.or(item.music_stream_expected_bytes);
    if expected_bytes.is_some_and(|expected| expected > media_len) {
        return None;
    }
    Some(path)
}

pub(super) fn music_cached_progress_for_item_in_root(item: &QueueItem, cache_root: &Path) -> f32 {
    if item.music_cache_key.trim().is_empty() || item.music_stream_ext.trim().is_empty() {
        return 0.0;
    }
    let cache_dir = cache_root.join(sanitize_music_cache_key(&item.music_cache_key));
    let manifest_path = cache_dir.join("manifest.yaml");
    if let Some(ratio) =
        music_cache_manifest_progress_ratio(&manifest_path, item.music_stream_expected_bytes)
    {
        return ratio;
    }
    let path = cache_dir.join(format!(
        "audio.{}",
        sanitize_music_cache_ext(&item.music_stream_ext)
    ));
    let len = fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
    if let Some(expected) = item.music_stream_expected_bytes.filter(|value| *value > 0) {
        return (len as f32 / expected as f32).clamp(0.0, 1.0);
    }
    0.0
}

pub(super) fn music_lrc_cache_path(cache_root: &Path, cache_key: &str) -> PathBuf {
    cache_root
        .join("lyrics")
        .join(sanitize_music_cache_key(cache_key))
        .join("lyrics.lrc")
}

pub(super) fn cache_music_lyrics_with_yt_dlp(
    tool_paths: &ToolPaths,
    cache_root: &Path,
    job: MusicLyricsCacheJob,
) -> Result<(), String> {
    let lyrics_dir = cache_root
        .join("lyrics")
        .join(sanitize_music_cache_key(&job.cache_key));
    fs::create_dir_all(&lyrics_dir)
        .map_err(|error| format!("Could not create lyrics cache folder: {error}"))?;
    let target_path = lyrics_dir.join("lyrics.lrc");
    if target_path.is_file() {
        return Ok(());
    }
    let mut command = tool_paths.prepare_music_lyrics_cache_command(
        &job.source_url,
        &lyrics_dir,
        &job.language_code,
        job.use_cookies,
    )?;
    let output = run_tracked_command_output(&mut command, "yt-dlp lyrics cache")
        .map_err(|error| format!("Could not start yt-dlp lyrics cache: {error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr)
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("unknown yt-dlp error")
            .to_owned();
        return Err(format!("yt-dlp lyrics cache failed: {detail}"));
    }
    let Some(candidate) = find_latest_file_in_dir(&lyrics_dir, "lrc") else {
        return Err("yt-dlp finished, but no LRC lyrics file was produced.".to_owned());
    };
    if candidate != target_path {
        if target_path.exists() {
            let _ = fs::remove_file(&target_path);
        }
        fs::rename(&candidate, &target_path)
            .or_else(|_| fs::copy(&candidate, &target_path).map(|_| ()))
            .map_err(|error| format!("Could not move LRC lyrics into cache: {error}"))?;
    }
    cleanup_music_lyrics_cache_dir(&lyrics_dir, &target_path);
    Ok(())
}

pub(super) fn cleanup_music_lyrics_cache_dir(lyrics_dir: &Path, keep_path: &Path) {
    let Ok(entries) = fs::read_dir(lyrics_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path != keep_path && path.is_file() {
            let _ = fs::remove_file(path);
        }
    }
}

pub(super) fn parse_lrc_file(path: &Path) -> Result<Vec<LrcLine>, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("Could not read LRC lyrics cache: {error}"))?;
    Ok(parse_lrc_text(&text))
}

pub(super) fn parse_lrc_text(text: &str) -> Vec<LrcLine> {
    let mut lines = Vec::new();
    for raw_line in text.lines() {
        let mut rest = raw_line.trim();
        let mut timestamps = Vec::new();
        while let Some(stripped) = rest.strip_prefix('[') {
            let Some((timestamp, tail)) = stripped.split_once(']') else {
                break;
            };
            let Some(seconds) = parse_lrc_timestamp(timestamp) else {
                break;
            };
            timestamps.push(seconds);
            rest = tail.trim_start();
        }
        let text = rest.trim();
        if text.is_empty() || timestamps.is_empty() {
            continue;
        }
        for seconds in timestamps {
            lines.push(LrcLine {
                seconds,
                text: text.to_owned(),
            });
        }
    }
    lines.sort_by(|left, right| {
        left.seconds
            .partial_cmp(&right.seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    lines
}

pub(super) fn parse_lrc_timestamp(value: &str) -> Option<f64> {
    let mut parts = value.trim().split(':').collect::<Vec<_>>();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    let seconds_text = parts.pop()?;
    let minutes = parts.pop()?.parse::<u64>().ok()?;
    let hours = parts
        .pop()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let seconds = seconds_text.replace(',', ".").parse::<f64>().ok()?;
    if !seconds.is_finite() || seconds < 0.0 {
        return None;
    }
    Some(hours as f64 * 3600.0 + minutes as f64 * 60.0 + seconds)
}

pub(super) fn current_lrc_line_text(lines: &[LrcLine], seconds: f64) -> Option<String> {
    if !seconds.is_finite() || lines.is_empty() {
        return None;
    }
    let index = lines.partition_point(|line| line.seconds <= seconds.max(0.0));
    if index == 0 {
        return None;
    }
    lines
        .get(index - 1)
        .map(|line| line.text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

pub(super) fn music_cache_key(source: &str, format_id: &str, ext: &str, acodec: &str) -> String {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    format_id.hash(&mut hasher);
    ext.hash(&mut hasher);
    acodec.hash(&mut hasher);
    format!("music_{:016x}", hasher.finish())
}

pub(super) fn json_str_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn music_album_title_from_json(value: &Value) -> String {
    json_str_field(value, "album")
        .or_else(|| json_str_field(value, "playlist_title"))
        .or_else(|| json_str_field(value, "playlist"))
        .unwrap_or_default()
}

pub(super) fn json_number_or_str_field(value: &Value, key: &str) -> Option<String> {
    let value = value.get(key)?;
    if let Some(text) = value.as_str() {
        let text = text.trim();
        return (!text.is_empty()).then(|| text.to_owned());
    }
    if let Some(number) = value.as_u64() {
        return Some(number.to_string());
    }
    if let Some(number) = value.as_i64() {
        return (number >= 0).then(|| number.to_string());
    }
    if let Some(number) = value
        .as_f64()
        .filter(|number| number.is_finite() && *number >= 0.0)
    {
        return Some(format!("{number:.0}"));
    }
    None
}

pub(super) fn json_f64_field(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

pub(super) fn json_u64_field(value: &Value, key: &str) -> Option<u64> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
}

pub(super) fn headers_from_json(value: &Value) -> Option<Vec<(String, String)>> {
    let object = value.as_object()?;
    let mut headers = Vec::new();
    for (name, raw_value) in object {
        let Some(value) = raw_value
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        headers.push((name.clone(), value.to_owned()));
    }
    Some(headers)
}

pub(super) fn format_duration_seconds(seconds: f64) -> String {
    if !seconds.is_finite() || seconds <= 0.0 {
        return "--:--".to_owned();
    }
    let total_seconds = seconds.round() as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}
