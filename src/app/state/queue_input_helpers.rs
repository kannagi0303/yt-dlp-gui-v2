use super::*;

pub(super) fn read_clipboard_text() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    clipboard.get_text().ok()
}

pub(super) fn extract_monitored_youtube_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|candidate| {
            candidate
                .trim_matches(|ch: char| {
                    matches!(
                        ch,
                        '"' | '\''
                            | '`'
                            | '<'
                            | '>'
                            | '('
                            | ')'
                            | '['
                            | ']'
                            | '{'
                            | '}'
                            | '\u{ff0c}'
                            | '\u{3002}'
                            | '\u{3001}'
                            | '\u{ff1b}'
                            | ';'
                            | '\u{ff1a}'
                            | ':'
                            | ','
                    )
                })
                .to_owned()
        })
        .filter(|candidate| !candidate.is_empty())
        .find_map(|candidate| normalize_monitored_youtube_url(&candidate))
}

pub(super) fn normalize_monitored_youtube_url(candidate: &str) -> Option<String> {
    let lowered = candidate.to_ascii_lowercase();
    if !(lowered.contains("youtube.com") || lowered.contains("youtu.be")) {
        return None;
    }

    if lowered.starts_with("http://") || lowered.starts_with("https://") {
        Some(candidate.to_owned())
    } else if lowered.starts_with("www.youtube.com")
        || lowered.starts_with("m.youtube.com")
        || lowered.starts_with("youtube.com")
        || lowered.starts_with("youtu.be")
    {
        Some(format!("https://{candidate}"))
    } else {
        None
    }
}

pub(super) fn canonical_queue_source_key(url: &str) -> String {
    let trimmed = url.trim();
    if let Some(video_id) = youtube_video_id(trimmed) {
        return format!("youtube:video:{video_id}");
    }
    trimmed.to_ascii_lowercase()
}

pub(super) fn youtube_video_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lowered = trimmed.to_ascii_lowercase();

    if lowered.contains("youtu.be/") {
        let (_, tail) = trimmed.split_once("youtu.be/")?;
        let id = tail
            .split(['?', '&', '#', '/'])
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        return Some(id.to_owned());
    }

    if lowered.contains("youtube.com/watch") || lowered.contains("m.youtube.com/watch") {
        let (_, tail) = trimmed.split_once("v=")?;
        let id = tail
            .split(['&', '#', '?', '/'])
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        return Some(id.to_owned());
    }

    None
}

pub(super) fn should_retry_analyze_with_cookies(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("sign in to confirm you're not a bot")
        || normalized.contains("sign in to confirm you")
        || normalized.contains("use --cookies-from-browser")
        || normalized.contains("use --cookies for the authentication")
}

pub(super) fn normalize_export_target_path(path: &str, default_extension: Option<&str>) -> String {
    let trimmed = path.trim();
    let mut target = PathBuf::from(trimmed);
    let has_extension = target
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| !value.trim().is_empty());
    if !has_extension {
        if let Some(extension) = default_extension.filter(|value| !value.trim().is_empty()) {
            target.set_extension(extension);
        }
    }
    target.display().to_string()
}

pub(super) fn normalized_export_extension(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_start_matches('.').to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub(super) fn validate_export_extension(
    kind: DownloadTargetKind,
    extension: &str,
) -> Result<(), String> {
    let valid = match kind {
        DownloadTargetKind::Video => matches!(extension, "mkv" | "mp4" | "webm" | "mov" | "flv"),
        DownloadTargetKind::Audio => {
            matches!(
                extension,
                "opus" | "aac" | "m4a" | "mp3" | "vorbis" | "alac" | "flac" | "wav"
            )
        }
        DownloadTargetKind::Subtitle => {
            matches!(
                extension,
                "srt"
                    | "vtt"
                    | "ass"
                    | "ssa"
                    | "lrc"
                    | "ttml"
                    | "dfxp"
                    | "srv1"
                    | "srv2"
                    | "srv3"
                    | "json3"
            )
        }
        DownloadTargetKind::Normal => true,
    };
    if valid {
        Ok(())
    } else {
        Err(match kind {
            DownloadTargetKind::Video => "Could not determine the video file extension.".to_owned(),
            DownloadTargetKind::Audio => "Could not determine the audio file extension.".to_owned(),
            DownloadTargetKind::Subtitle => {
                "Could not determine the subtitle file extension.".to_owned()
            }
            DownloadTargetKind::Normal => String::new(),
        })
    }
}

pub(super) fn flat_music_entries_from_url(
    tool_paths: ToolPaths,
    source: &str,
    untitled_task: &str,
    imported_template: &str,
) -> Result<Vec<PlaylistEntrySeed>, String> {
    let mut command = tool_paths.prepare_batch_add_command(source)?;
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start yt-dlp music flat import: {error}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Could not read yt-dlp music flat output.".to_owned())?;
    let mut stderr = child.stderr.take();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    let mut seeds = Vec::new();

    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|error| format!("Could not read yt-dlp music flat output: {error}"))?;
        if read == 0 {
            break;
        }
        let raw = line.trim();
        if raw.is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_str::<Value>(raw) else {
            continue;
        };
        if let Some(mut seed) =
            playlist_entry_seed_from_json(&entry, untitled_task, imported_template)
        {
            if let Some(thumbnail_url) = select_largest_thumbnail_url(&entry) {
                seed.thumbnail_url = thumbnail_url;
                seed.thumbnail_hint = "Thumbnail preview".to_owned();
            }
            seeds.push(seed);
        }
    }

    let status = child
        .wait()
        .map_err(|error| format!("Could not wait for yt-dlp music flat import: {error}"))?;
    let mut stderr_text = String::new();
    if let Some(mut reader) = stderr.take() {
        let _ = reader.read_to_string(&mut stderr_text);
    }
    if !status.success() && seeds.is_empty() {
        let detail = stderr_text.trim();
        return Err(if detail.is_empty() {
            format!(
                "yt-dlp music flat import failed: exit code {:?}",
                status.code()
            )
        } else {
            format!("yt-dlp music flat import failed: {detail}")
        });
    }
    if seeds.is_empty() {
        return Err("yt-dlp did not return any music list entries.".to_owned());
    }
    Ok(seeds)
}
