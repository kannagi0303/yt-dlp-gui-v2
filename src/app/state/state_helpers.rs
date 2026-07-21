use super::*;

pub(super) fn analyze_output_parts(
    result: Result<AnalyzeOutput, AnalyzeError>,
) -> (Result<Value, String>, Option<String>) {
    match result {
        Ok(output) => (Ok(output.json), Some(output.command_line)),
        Err(error) => (Err(error.message), error.command_line),
    }
}

pub(super) fn download_target_log_action(target_kind: DownloadTargetKind) -> &'static str {
    match target_kind {
        DownloadTargetKind::Normal => "download",
        DownloadTargetKind::Video => "download video",
        DownloadTargetKind::Audio => "download audio",
        DownloadTargetKind::Subtitle => "download subtitle",
    }
}

pub(super) fn music_source_kind_log_action(source_kind: MusicDownloadSourceKind) -> &'static str {
    match source_kind {
        MusicDownloadSourceKind::CacheCopy => "music cache copy",
        MusicDownloadSourceKind::CacheConvert => "music cache convert",
        MusicDownloadSourceKind::YtDlpOnlineTarget => "music download",
        MusicDownloadSourceKind::YtDlpDownload => "music download",
    }
}

pub(super) fn format_process_command_line(command: &Command) -> String {
    let program = quote_command_arg(&command.get_program().to_string_lossy());
    let args = command
        .get_args()
        .map(|arg| quote_command_arg(&arg.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ");
    if args.is_empty() {
        program
    } else {
        format!("{program} {args}")
    }
}

pub(super) fn quote_command_arg(value: &str) -> String {
    if value.contains([' ', '\t', '"']) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_owned()
    }
}

pub(super) fn restore_music_compact_item_from_cache_hit(
    item: &mut QueueItem,
    hit: &CompleteMusicCacheHit,
) {
    if !hit.source_url.trim().is_empty() {
        item.source_url = hit.source_url.clone();
    }
    if !hit.title.trim().is_empty() {
        item.title = hit.title.clone();
    }
    if !hit.album_title.trim().is_empty() {
        item.music_album_title = hit.album_title.clone();
    }
    if !hit.thumbnail_url.trim().is_empty() {
        item.thumbnail_url = hit.thumbnail_url.clone();
        item.thumbnail_hint = hit.thumbnail_url.clone();
    }
    if let Some(duration) = hit.duration_seconds {
        item.music_duration_seconds = Some(duration);
        item.duration_text = format_duration_seconds(duration);
    }
    item.music_stream_url.clear();
    item.music_stream_headers.clear();
    item.music_stream_ext = hit.ext.clone();
    item.music_stream_format_id = hit.format_id.clone();
    item.music_stream_acodec = hit.acodec.clone();
    item.music_stream_expected_bytes = hit.expected_bytes;
    item.music_cache_key = hit.cache_key.clone();
    item.compact_music_state = Some(CompactMusicState::Ready);
    item.last_error = None;
}

pub(super) fn audio_playlist_item_snapshot(item: &QueueItem) -> AudioPlaylistItemSnapshot {
    AudioPlaylistItemSnapshot {
        source_url: item.source_url.clone(),
        title: item.title.clone(),
        album_title: item.music_album_title.clone(),
        thumbnail_hint: item.thumbnail_hint.clone(),
        thumbnail_url: item.thumbnail_url.clone(),
        duration_text: item.duration_text.clone(),
        duration_seconds: item.music_duration_seconds,
        stream_ext: item.music_stream_ext.clone(),
        stream_format_id: item.music_stream_format_id.clone(),
        stream_acodec: item.music_stream_acodec.clone(),
        expected_bytes: item.music_stream_expected_bytes,
        cache_key: item.music_cache_key.clone(),
        use_cookies: item.selection.use_cookies,
    }
}

pub(super) const BACKGROUND_DOWNLOAD_EVENT_BUDGET_PER_POLL: usize = 96;
pub(super) const BACKGROUND_MUSIC_DOWNLOAD_EVENT_BUDGET_PER_POLL: usize = 24;

pub(super) fn monotonic_progress(current: f32, next: f32) -> f32 {
    if next.is_finite() {
        current.max(next.clamp(0.0, 100.0))
    } else {
        current
    }
}

pub(super) fn format_download_progress_detail(
    _language: Language,
    detail: &DownloadProgressDetail,
) -> String {
    let mut lines = Vec::new();

    push_detail_line(&mut lines, "Downloaded", detail.downloaded.as_deref());
    push_detail_line(&mut lines, "Total", detail.total.as_deref());
    push_detail_line(&mut lines, "Speed", detail.speed.as_deref());
    push_detail_line(&mut lines, "Elapsed", detail.elapsed.as_deref());
    push_detail_line(&mut lines, "Frame", detail.frame.as_deref());
    push_detail_line(&mut lines, "FPS", detail.fps.as_deref());
    push_detail_line(&mut lines, "Time", detail.time.as_deref());

    if lines.is_empty() {
        "Running".to_owned()
    } else {
        lines.join("\n")
    }
}

pub(super) fn push_detail_line(lines: &mut Vec<String>, label: &str, value: Option<&str>) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    lines.push(format!("{label}\t{value}"));
}

pub(super) fn single_mode_status_workflow_visible(run: &WorkflowRun, item: &QueueItem) -> bool {
    if !matches!(
        run.kind,
        WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
    ) {
        return false;
    }

    matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
        || !run.detail.trim().is_empty()
        || run.output_path.is_some()
        || run.error.is_some()
        || item.last_error.is_some()
}

pub(super) fn workflow_tool_label(tool: &ToolKind) -> String {
    match tool {
        ToolKind::YtDlp => "yt-dlp".to_owned(),
        ToolKind::Ffmpeg => "FFMPEG".to_owned(),
        ToolKind::Aria2c => "aria2c".to_owned(),
        ToolKind::Other(label) => label.clone(),
    }
}

pub(super) fn status_lines_contain(lines: &[(String, String)], label: &str) -> bool {
    lines.iter().any(|(candidate, _)| candidate == label)
}

pub(super) fn queue_item_status_key(item: &QueueItem) -> &'static str {
    if let Some(run) = item.workflows.iter().rev().find(|run| {
        matches!(
            run.kind,
            WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
        ) && matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
    }) {
        return match run.state {
            WorkflowState::Queued => "Queued",
            WorkflowState::Running => "Running",
            _ => "Queued",
        };
    }

    if let Some(run) = item
        .workflows
        .iter()
        .rev()
        .find(|run| run.kind == WorkflowKind::DownloadMedia)
    {
        return match run.state {
            WorkflowState::Queued => "Queued",
            WorkflowState::Running => "Running",
            WorkflowState::Finished if item.last_error.is_some() => "Failed",
            WorkflowState::Finished => "Done",
            WorkflowState::Failed => "Failed",
            WorkflowState::Cancelled => "Cancelled",
        };
    }

    match &item.metadata_state {
        MetadataState::Idle => "Not started",
        MetadataState::Queued => "Waiting for analysis",
        MetadataState::Running => "Analyzing",
        MetadataState::Ready(_) => "Queued",
        MetadataState::Failed(_) => "Analysis failed",
    }
}

pub(super) fn thumbnail_cache_key(
    url: &str,
    proxy_url: &str,
    no_check_certificates: bool,
) -> String {
    format!(
        "{}\n{}\n{}",
        proxy_url.trim(),
        no_check_certificates,
        url.trim()
    )
}

pub(super) fn thumbnail_texture_id(key: &str) -> String {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    format!("proxied-thumbnail-{:016x}", hasher.finish())
}

pub(super) fn thumbnail_needs_memory_loader(url: &str) -> bool {
    let url = url.trim();
    url.starts_with("http://") || url.starts_with("https://")
}

pub(super) fn reset_item_for_new_work(item: &mut QueueItem, target_kind: DownloadTargetKind) {
    match target_kind {
        DownloadTargetKind::Normal => {
            item.progress.video = 0.0;
            item.progress.audio = 0.0;
            item.progress.subtitle = 0.0;
            item.progress.post_process = 0.0;
        }
        DownloadTargetKind::Video => item.progress.video = 0.0,
        DownloadTargetKind::Audio => item.progress.audio = 0.0,
        DownloadTargetKind::Subtitle => item.progress.subtitle = 0.0,
    }
    item.last_error = None;
}
