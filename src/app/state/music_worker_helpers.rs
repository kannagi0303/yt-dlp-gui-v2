use super::*;

pub(super) fn run_music_download_worker(
    tool_paths: ToolPaths,
    mut job: MusicDownloadJob,
    tx: Sender<MusicDownloadEvent>,
    child_handle: Arc<Mutex<Option<Child>>>,
    cancel_requested: Arc<AtomicBool>,
) {
    if job.choice.embed_cover && !job.cover_path.as_ref().is_some_and(|path| path.is_file()) {
        job.cover_path = ensure_music_download_cover_path(&job);
    }
    let has_cover =
        job.choice.embed_cover && job.cover_path.as_ref().is_some_and(|path| path.is_file());
    let online_target_available = job.choice.target_format().is_some_and(|format| {
        job.cache_media_path.as_ref().is_some_and(|path| {
            !music_cache_source_matches_target(format, path, &job.source_acodec)
        }) && online_music_target_source_available(&tool_paths, &job, format)
    });
    let source_kind = match job.cache_media_path.as_ref() {
        Some(path) if music_cache_can_be_copied_for_choice(job.choice, path, has_cover) => {
            MusicDownloadSourceKind::CacheCopy
        }
        Some(_) if online_target_available => MusicDownloadSourceKind::YtDlpOnlineTarget,
        Some(_) => MusicDownloadSourceKind::CacheConvert,
        None => MusicDownloadSourceKind::YtDlpDownload,
    };

    let result = match source_kind {
        MusicDownloadSourceKind::CacheCopy => {
            copy_music_cache_output(&job, tx.clone(), &cancel_requested)
        }
        MusicDownloadSourceKind::CacheConvert => convert_music_cache_output(
            &tool_paths,
            &job,
            tx.clone(),
            &child_handle,
            &cancel_requested,
        ),
        MusicDownloadSourceKind::YtDlpOnlineTarget | MusicDownloadSourceKind::YtDlpDownload => {
            download_music_output_with_yt_dlp(
                &tool_paths,
                &job,
                source_kind,
                tx.clone(),
                &child_handle,
                &cancel_requested,
            )
        }
    };

    let result = result.map(|path_text| {
        let output_path = PathBuf::from(&path_text);
        match ensure_music_download_album_metadata_written(
            &tool_paths,
            &job,
            output_path,
            source_kind,
            &tx,
        ) {
            Ok(path) => path.display().to_string(),
            Err(error) => {
                eprintln!("[music-download] album metadata pass skipped: {error}");
                path_text
            }
        }
    });

    let _ = tx.send(MusicDownloadEvent::Finished {
        item_id: job.item_id,
        workflow_id: job.workflow_id,
        source_kind,
        result,
    });
}

pub(super) fn ensure_music_download_cover_path(job: &MusicDownloadJob) -> Option<PathBuf> {
    if let Some(path) = job.cover_path.as_ref().filter(|path| path.is_file()) {
        return Some(path.clone());
    }
    let dir = job.cover_cache_dir.as_ref()?;
    let url = job.thumbnail_url.trim();
    let cached = first_music_cover_file_in_dir(dir);
    if url.is_empty() {
        return cached;
    }
    if cached.is_some() && cached_music_cover_source_matches(dir, url) {
        return cached;
    }
    download_music_cover_to_dir(url, dir).ok().or(cached)
}

pub(super) fn first_music_cover_file_in_dir(dir: &Path) -> Option<PathBuf> {
    [
        "cover.jpg",
        "cover.jpeg",
        "cover.png",
        "cover.webp",
        "cover.img",
    ]
    .into_iter()
    .map(|name| dir.join(name))
    .find(|path| path.is_file())
}

pub(super) fn music_cover_source_url_path(dir: &Path) -> PathBuf {
    dir.join("source_url.txt")
}

pub(super) fn cached_music_cover_source_matches(dir: &Path, url: &str) -> bool {
    fs::read_to_string(music_cover_source_url_path(dir))
        .map(|value| value.trim() == url.trim())
        .unwrap_or(false)
}

pub(super) fn remove_cached_music_cover_files(dir: &Path) {
    for name in [
        "cover.jpg",
        "cover.jpeg",
        "cover.png",
        "cover.webp",
        "cover.img",
    ] {
        let _ = fs::remove_file(dir.join(name));
    }
}

pub(super) fn download_music_cover_to_dir(url: &str, dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(dir)
        .map_err(|error| format!("Could not create music cover cache: {error}"))?;
    let mut response = ureq::get(url)
        .call()
        .map_err(|error| format!("Could not download music cover cache: {error}"))?;
    let status = response.status().as_u16();
    if status >= 400 {
        return Err(format!(
            "Could not download music cover cache: HTTP {status}"
        ));
    }
    let mut reader = response.body_mut().as_reader();
    let mut data = Vec::new();
    reader
        .read_to_end(&mut data)
        .map_err(|error| format!("Could not read music cover cache: {error}"))?;
    if data.is_empty() {
        return Err("Downloaded music cover cache is empty.".to_owned());
    }
    let extension = music_cover_extension_from_bytes(&data);
    let path = dir.join(format!("cover.{extension}"));
    remove_cached_music_cover_files(dir);
    fs::write(&path, data)
        .map_err(|error| format!("Could not write music cover cache: {error}"))?;
    let _ = fs::write(music_cover_source_url_path(dir), url);
    Ok(path)
}

pub(super) fn music_cover_extension_from_bytes(data: &[u8]) -> &'static str {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if data.starts_with(b"\x89PNG\r\n\x1A\n") {
        "png"
    } else if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        "webp"
    } else {
        "img"
    }
}

pub(super) fn send_music_tool_command_finished(
    tx: &Sender<MusicDownloadEvent>,
    job: &MusicDownloadJob,
    source_kind: MusicDownloadSourceKind,
    tool: &str,
    action: &str,
    command_line: String,
    success: bool,
) {
    let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
        item_id: job.item_id,
        workflow_id: job.workflow_id,
        source_kind,
        tool: tool.to_owned(),
        action: action.to_owned(),
        command_line,
        success,
    });
}

pub(super) fn ensure_music_download_requested_extension(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    output_path: PathBuf,
    source_kind: MusicDownloadSourceKind,
    tx: &Sender<MusicDownloadEvent>,
) -> Result<PathBuf, String> {
    let Some(target_format) = job.choice.target_format() else {
        return Ok(output_path);
    };
    let current_ext = output_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if current_ext.eq_ignore_ascii_case(target_format.extension()) {
        return Ok(output_path);
    }
    // yt-dlp should normally return the requested extension, but some extract-audio
    // paths can leave a nearby container behind. Keep this pass generic so cached
    // and direct music downloads still honor the user-facing format choice.
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    if !ffmpeg.is_file() || !output_path.is_file() {
        return Ok(output_path);
    }
    let target_path =
        unique_music_output_path(&job.output_dir, &job.title, target_format.extension());
    let mut command = Command::new(&ffmpeg);
    configure_background_command(&mut command);
    command.arg("-y").arg("-i").arg(&output_path).args(
        resolve_music_audio_export_plan(
            target_format,
            &probe_music_audio_source_profile(tool_paths, &output_path, &job.source_acodec),
        )
        .ffmpeg_args,
    );
    if job.choice.write_tags {
        append_music_metadata_args(&mut command, job);
    }
    command.arg(&target_path);
    let command_line = format_process_command_line(&command);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    let output = match run_tracked_command_output(&mut command, "ffmpeg music post pass") {
        Ok(output) => output,
        Err(error) => {
            send_music_tool_command_finished(
                tx,
                job,
                source_kind,
                "ffmpeg",
                "extension pass",
                command_line,
                false,
            );
            return Err(format!("Could not start FFmpeg music output pass: {error}"));
        }
    };
    if !output.status.success() {
        send_music_tool_command_finished(
            tx,
            job,
            source_kind,
            "ffmpeg",
            "extension pass",
            command_line.clone(),
            false,
        );
        let _ = fs::remove_file(&target_path);
        let detail = String::from_utf8_lossy(&output.stderr)
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("unknown FFmpeg error")
            .to_owned();
        return Err(format!("FFmpeg music output pass failed: {detail}"));
    }
    send_music_tool_command_finished(
        tx,
        job,
        source_kind,
        "ffmpeg",
        "extension pass",
        command_line,
        true,
    );
    let _ = fs::remove_file(&output_path);
    Ok(target_path)
}

pub(super) fn ensure_music_download_cover_embedded(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    output_path: PathBuf,
    source_kind: MusicDownloadSourceKind,
    tx: &Sender<MusicDownloadEvent>,
) -> Result<PathBuf, String> {
    if !job.choice.embed_cover || !music_output_path_supports_embedded_cover(&output_path) {
        return Ok(output_path);
    }
    let Some(cover_path) = job.cover_path.as_ref().filter(|path| path.is_file()) else {
        return Ok(output_path);
    };
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    if !ffmpeg.is_file() || !output_path.is_file() {
        return Ok(output_path);
    }
    let extension = output_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("m4a");
    let temp_path = output_path.with_extension(format!("cover-pass.{extension}"));
    let mut command = Command::new(&ffmpeg);
    configure_background_command(&mut command);
    command
        .arg("-y")
        .arg("-i")
        .arg(&output_path)
        .arg("-i")
        .arg(cover_path)
        .args(["-map", "0:a:0", "-map", "1:v:0", "-c:a", "copy"])
        .args([
            "-c:v",
            "mjpeg",
            "-disposition:v:0",
            "attached_pic",
            "-metadata:s:v",
            "title=Album cover",
            "-metadata:s:v",
            "comment=Cover (front)",
        ]);
    if job.choice.write_tags {
        append_music_metadata_args(&mut command, job);
    }
    command.arg(&temp_path);
    let command_line = format_process_command_line(&command);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    let output = match run_tracked_command_output(&mut command, "ffmpeg music post pass") {
        Ok(output) => output,
        Err(error) => {
            send_music_tool_command_finished(
                tx,
                job,
                source_kind,
                "ffmpeg",
                "embed cover",
                command_line,
                false,
            );
            return Err(format!("Could not start FFmpeg cover embed pass: {error}"));
        }
    };
    if !output.status.success() {
        send_music_tool_command_finished(
            tx,
            job,
            source_kind,
            "ffmpeg",
            "embed cover",
            command_line.clone(),
            false,
        );
        let _ = fs::remove_file(&temp_path);
        let detail = String::from_utf8_lossy(&output.stderr)
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("unknown FFmpeg error")
            .to_owned();
        return Err(format!("FFmpeg cover embed pass failed: {detail}"));
    }
    fs::remove_file(&output_path).map_err(|error| {
        format!("Could not replace music output after cover embed pass: {error}")
    })?;
    fs::rename(&temp_path, &output_path)
        .map_err(|error| format!("Could not move music output after cover embed pass: {error}"))?;
    send_music_tool_command_finished(
        tx,
        job,
        source_kind,
        "ffmpeg",
        "embed cover",
        command_line,
        true,
    );
    Ok(output_path)
}

pub(super) fn ensure_music_download_album_metadata_written(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    output_path: PathBuf,
    source_kind: MusicDownloadSourceKind,
    tx: &Sender<MusicDownloadEvent>,
) -> Result<PathBuf, String> {
    if !job.choice.write_tags {
        return Ok(output_path);
    }
    if matches!(
        source_kind,
        MusicDownloadSourceKind::YtDlpDownload | MusicDownloadSourceKind::YtDlpOnlineTarget
    ) && job.album_title.trim().is_empty()
    {
        return Ok(output_path);
    }
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    if !ffmpeg.is_file() || !output_path.is_file() {
        return Ok(output_path);
    }
    let extension = output_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("m4a");
    let temp_path = output_path.with_extension(format!("metadata-pass.{extension}"));
    let mut command = Command::new(&ffmpeg);
    configure_background_command(&mut command);
    command
        .arg("-y")
        .arg("-i")
        .arg(&output_path)
        .args(["-map", "0", "-c", "copy"]);
    append_music_metadata_args(&mut command, job);
    command.arg(&temp_path);
    let command_line = format_process_command_line(&command);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    let output = match run_tracked_command_output(&mut command, "ffmpeg music post pass") {
        Ok(output) => output,
        Err(error) => {
            send_music_tool_command_finished(
                tx,
                job,
                source_kind,
                "ffmpeg",
                "write metadata",
                command_line,
                false,
            );
            return Err(format!(
                "Could not start FFmpeg album metadata pass: {error}"
            ));
        }
    };
    if !output.status.success() {
        send_music_tool_command_finished(
            tx,
            job,
            source_kind,
            "ffmpeg",
            "write metadata",
            command_line.clone(),
            false,
        );
        let _ = fs::remove_file(&temp_path);
        let detail = String::from_utf8_lossy(&output.stderr)
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("unknown FFmpeg error")
            .to_owned();
        return Err(format!("FFmpeg album metadata pass failed: {detail}"));
    }
    fs::remove_file(&output_path)
        .map_err(|error| format!("Could not replace music output after metadata pass: {error}"))?;
    fs::rename(&temp_path, &output_path)
        .map_err(|error| format!("Could not move music output after metadata pass: {error}"))?;
    send_music_tool_command_finished(
        tx,
        job,
        source_kind,
        "ffmpeg",
        "write metadata",
        command_line,
        true,
    );
    Ok(output_path)
}

pub(super) fn copy_music_cache_output(
    job: &MusicDownloadJob,
    tx: Sender<MusicDownloadEvent>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<String, String> {
    let Some(source) = job.cache_media_path.as_ref() else {
        return Err("Music cache file is missing.".to_owned());
    };
    fs::create_dir_all(&job.output_dir)
        .map_err(|error| format!("Could not create music download folder: {error}"))?;
    let output_extension = music_output_extension_for_choice(job.choice, source);
    let output_path = unique_music_output_path(&job.output_dir, &job.title, &output_extension);
    let _ = tx.send(MusicDownloadEvent::Progress {
        item_id: job.item_id,
        workflow_id: job.workflow_id,
        percent: 5.0,
    });
    if cancel_requested.load(Ordering::Relaxed) {
        return Err(DOWNLOAD_CANCELLED_MESSAGE.to_owned());
    }
    fs::copy(source, &output_path)
        .map_err(|error| format!("Could not copy music cache: {error}"))?;
    let _ = tx.send(MusicDownloadEvent::Progress {
        item_id: job.item_id,
        workflow_id: job.workflow_id,
        percent: 100.0,
    });
    Ok(output_path.display().to_string())
}

pub(super) fn convert_music_cache_output(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    tx: Sender<MusicDownloadEvent>,
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<String, String> {
    let Some(source) = job.cache_media_path.as_ref() else {
        return Err("Music cache file is missing.".to_owned());
    };
    fs::create_dir_all(&job.output_dir)
        .map_err(|error| format!("Could not create music download folder: {error}"))?;
    let output_extension = music_output_extension_for_choice(job.choice, source);
    let output_path = unique_music_output_path(&job.output_dir, &job.title, &output_extension);
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    if !ffmpeg.is_file() {
        return Err(format!(
            "ffmpeg.exe was not found: {}. Install FFmpeg from Options first.",
            ffmpeg.display()
        ));
    }

    let mut command = Command::new(&ffmpeg);
    configure_background_command(&mut command);
    command.arg("-y").arg("-i").arg(source);
    let has_cover = job.choice.embed_cover
        && job.cover_path.as_ref().is_some_and(|path| path.is_file())
        && music_extension_supports_embedded_cover(
            output_path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default(),
        );
    if has_cover {
        if let Some(cover) = job.cover_path.as_ref() {
            command.arg("-i").arg(cover);
        }
    }
    command.args(["-map", "0:a:0"]);
    if has_cover {
        command.args(["-map", "1:v:0"]);
    }
    let source_profile = probe_music_audio_source_profile(tool_paths, source, &job.source_acodec);
    let audio_args = if let Some(target_format) = job.choice.target_format() {
        resolve_music_audio_export_plan(target_format, &source_profile).ffmpeg_args
    } else {
        vec!["-c:a".to_owned(), "copy".to_owned()]
    };
    command.args(audio_args);
    if has_cover {
        command.args([
            "-c:v",
            "mjpeg",
            "-disposition:v:0",
            "attached_pic",
            "-metadata:s:v",
            "title=Album cover",
            "-metadata:s:v",
            "comment=Cover (front)",
        ]);
    }
    if job.choice.write_tags {
        append_music_metadata_args(&mut command, job);
    }
    command.arg(&output_path);
    let command_line = format_process_command_line(&command);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let _ = tx.send(MusicDownloadEvent::Progress {
        item_id: job.item_id,
        workflow_id: job.workflow_id,
        percent: 10.0,
    });
    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start FFmpeg music conversion: {error}"))?;
    let stderr = child.stderr.take();
    if let Ok(mut guard) = child_handle.lock() {
        *guard = Some(child);
    }
    let stderr_handle = stderr.map(|stderr| {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            reader.lines().map_while(Result::ok).collect::<Vec<_>>()
        })
    });
    let status = wait_music_child(child_handle, cancel_requested);
    let stderr_lines = stderr_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    if cancel_requested.load(Ordering::Relaxed) {
        let _ = fs::remove_file(&output_path);
        let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
            item_id: job.item_id,
            workflow_id: job.workflow_id,
            source_kind: MusicDownloadSourceKind::CacheConvert,
            tool: "ffmpeg".to_owned(),
            action: "convert".to_owned(),
            command_line,
            success: false,
        });
        return Err(DOWNLOAD_CANCELLED_MESSAGE.to_owned());
    }
    match status {
        Some(Ok(status)) if status.success() => {
            let _ = tx.send(MusicDownloadEvent::Progress {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                percent: 100.0,
            });
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind: MusicDownloadSourceKind::CacheConvert,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line,
                success: true,
            });
            Ok(output_path.display().to_string())
        }
        Some(Ok(status)) => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind: MusicDownloadSourceKind::CacheConvert,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line: command_line.clone(),
                success: false,
            });
            let detail = stderr_lines
                .iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| format!("exit code {:?}", status.code()));
            Err(format!("FFmpeg music conversion failed: {detail}"))
        }
        Some(Err(error)) => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind: MusicDownloadSourceKind::CacheConvert,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line: command_line.clone(),
                success: false,
            });
            Err(format!(
                "Could not wait for FFmpeg music conversion: {error}"
            ))
        }
        None => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind: MusicDownloadSourceKind::CacheConvert,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line,
                success: false,
            });
            Err("Could not wait for FFmpeg music conversion: child process missing".to_owned())
        }
    }
}

pub(super) fn download_music_output_with_yt_dlp(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    source_kind: MusicDownloadSourceKind,
    tx: Sender<MusicDownloadEvent>,
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<String, String> {
    let prepared = tool_paths.prepare_music_audio_download_command(
        &job.source_url,
        &job.output_dir,
        job.choice
            .target_format()
            .map(MusicDownloadFormat::yt_dlp_audio_format),
        job.choice.format_selector(),
        job.choice.embed_cover,
        job.choice.write_tags,
        job.use_cookies,
    )?;
    println!(
        "[music-download] output: {}",
        prepared.output_path.display()
    );
    println!("[music-download] command: {}", prepared.command_line);
    let PreparedDownload {
        mut command,
        output_path,
        command_line,
    } = prepared;

    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start yt-dlp music download: {error}"))?;
    let _process_guard = track_child_process(&child, "yt-dlp music cache/download");

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    if let Ok(mut guard) = child_handle.lock() {
        *guard = Some(child);
    }

    let item_id = job.item_id;
    let workflow_id = job.workflow_id;
    let stdout_handle = stdout.map(|stdout| {
        let tx = tx.clone();
        thread::spawn(move || read_music_yt_dlp_stream(stdout, item_id, workflow_id, tx))
    });
    let stderr_handle = stderr.map(|stderr| {
        let tx = tx.clone();
        thread::spawn(move || read_music_yt_dlp_stream(stderr, item_id, workflow_id, tx))
    });

    let status = wait_music_child(child_handle, cancel_requested);
    let mut lines = stdout_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    lines.extend(
        stderr_handle
            .and_then(|handle| handle.join().ok())
            .unwrap_or_default(),
    );

    if cancel_requested.load(Ordering::Relaxed) {
        let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
            item_id: job.item_id,
            workflow_id: job.workflow_id,
            source_kind,
            tool: "yt-dlp".to_owned(),
            action: "download".to_owned(),
            command_line: command_line.clone(),
            success: false,
        });
        return Err(DOWNLOAD_CANCELLED_MESSAGE.to_owned());
    }

    match status {
        Some(Ok(status)) if status.success() => {
            let output_path = reported_music_final_output_path(&lines)
                .or_else(|| {
                    find_latest_music_download_output_for_choice(&job.output_dir, job.choice)
                })
                .unwrap_or(output_path);
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind,
                tool: "yt-dlp".to_owned(),
                action: "download".to_owned(),
                command_line: command_line.clone(),
                success: true,
            });
            let output_path = match ensure_music_download_requested_extension(
                &tool_paths,
                job,
                output_path.clone(),
                source_kind,
                &tx,
            ) {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("[music-download] requested extension pass skipped: {error}");
                    output_path
                }
            };
            let output_path = match ensure_music_download_cover_embedded(
                &tool_paths,
                job,
                output_path.clone(),
                source_kind,
                &tx,
            ) {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("[music-download] cover embed pass skipped: {error}");
                    output_path
                }
            };
            let _ = tx.send(MusicDownloadEvent::Progress {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                percent: 100.0,
            });
            Ok(output_path.display().to_string())
        }
        Some(Ok(status)) => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind,
                tool: "yt-dlp".to_owned(),
                action: "download".to_owned(),
                command_line: command_line.clone(),
                success: false,
            });
            let detail = lines
                .iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| format!("exit code {:?}", status.code()));
            Err(format!("yt-dlp music download failed: {detail}"))
        }
        Some(Err(error)) => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind,
                tool: "yt-dlp".to_owned(),
                action: "download".to_owned(),
                command_line: command_line.clone(),
                success: false,
            });
            Err(format!("Could not wait for yt-dlp music download: {error}"))
        }
        None => {
            let _ = tx.send(MusicDownloadEvent::ToolCommandFinished {
                item_id: job.item_id,
                workflow_id: job.workflow_id,
                source_kind,
                tool: "yt-dlp".to_owned(),
                action: "download".to_owned(),
                command_line,
                success: false,
            });
            Err("Could not wait for yt-dlp music download: child process missing".to_owned())
        }
    }
}

pub(super) fn read_music_yt_dlp_stream<R: Read>(
    reader: R,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: Sender<MusicDownloadEvent>,
) -> Vec<String> {
    let mut reader = BufReader::new(reader);
    let mut lines = Vec::new();
    let mut pending = Vec::new();
    let mut chunk = [0_u8; 4096];

    loop {
        let bytes_read = match std::io::Read::read(&mut reader, &mut chunk) {
            Ok(size) => size,
            Err(_) => break,
        };
        if bytes_read == 0 {
            break;
        }

        for &byte in &chunk[..bytes_read] {
            if matches!(byte, b'\n' | b'\r') {
                process_music_yt_dlp_line(&pending, item_id, workflow_id, &tx, &mut lines);
                pending.clear();
            } else {
                pending.push(byte);
            }
        }
    }

    if !pending.is_empty() {
        process_music_yt_dlp_line(&pending, item_id, workflow_id, &tx, &mut lines);
    }

    lines
}

pub(super) fn process_music_yt_dlp_line(
    bytes: &[u8],
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: &Sender<MusicDownloadEvent>,
    lines: &mut Vec<String>,
) {
    let line = String::from_utf8_lossy(bytes).trim().to_owned();
    if line.is_empty() {
        return;
    }

    if let Some(percent) = parse_music_yt_dlp_progress_percent(&line) {
        let _ = tx.send(MusicDownloadEvent::Progress {
            item_id,
            workflow_id,
            percent,
        });
    }
    lines.push(line);
}

pub(super) fn parse_music_yt_dlp_progress_percent(line: &str) -> Option<f32> {
    parse_music_progress_template_percent(line).or_else(|| parse_default_download_percent(line))
}

pub(super) fn parse_music_progress_template_percent(line: &str) -> Option<f32> {
    let value = line
        .trim()
        .strip_prefix("[yt-dlp],")?
        .split(',')
        .next()?
        .trim();
    parse_percent_text(value)
}

pub(super) fn parse_default_download_percent(line: &str) -> Option<f32> {
    let body = line.trim().strip_prefix("[download]")?.trim_start();
    if body.starts_with("Destination:") {
        return None;
    }

    body.split_whitespace()
        .find_map(|part| parse_percent_text(part.trim()))
}

pub(super) fn parse_percent_text(value: &str) -> Option<f32> {
    value
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .ok()
        .map(|value| value.clamp(0.0, 100.0))
}

pub(super) fn reported_music_final_output_path(lines: &[String]) -> Option<PathBuf> {
    lines.iter().rev().find_map(|line| {
        let payload = line.trim().strip_prefix(FINAL_OUTPUT_PATH_PREFIX)?.trim();
        let parsed = serde_json::from_str::<String>(payload)
            .unwrap_or_else(|_| payload.trim_matches('"').to_owned());
        let trimmed = parsed.trim();
        (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
    })
}

pub(super) fn wait_music_child(
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Option<std::io::Result<std::process::ExitStatus>> {
    loop {
        if cancel_requested.load(Ordering::Relaxed) {
            if let Ok(mut guard) = child_handle.lock() {
                if let Some(child) = guard.as_mut() {
                    let _ = child.kill();
                }
            }
        }
        if let Ok(mut guard) = child_handle.lock() {
            let Some(child) = guard.as_mut() else {
                return None;
            };
            match child.try_wait() {
                Ok(Some(status)) => {
                    *guard = None;
                    return Some(Ok(status));
                }
                Ok(None) => {}
                Err(error) => {
                    *guard = None;
                    return Some(Err(error));
                }
            }
        } else {
            return None;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

pub(super) fn unique_music_output_path(output_dir: &Path, title: &str, extension: &str) -> PathBuf {
    let stem = sanitize_file_name_for_windows(&music_output_stem_for_title(title));
    let base = if stem.trim().is_empty() {
        "music"
    } else {
        stem.trim()
    };
    let extension = extension.trim().trim_start_matches('.');
    let mut path = output_dir.join(format!("{base}.{extension}"));
    if !path.exists() {
        return path;
    }
    for index in 2..10_000 {
        path = output_dir.join(format!("{base} ({index}).{extension}"));
        if !path.exists() {
            return path;
        }
    }
    output_dir.join(format!(
        "{base}.{}.{}",
        unique_timestamp_suffix(),
        extension
    ))
}

pub(super) fn find_latest_music_download_output_for_choice(
    dir: &Path,
    choice: MusicDownloadChoice,
) -> Option<PathBuf> {
    if let Some(format) = choice.target_format() {
        return find_latest_file_in_dir(dir, format.extension());
    }
    find_latest_music_original_output(dir)
}

pub(super) fn find_latest_music_original_output(dir: &Path) -> Option<PathBuf> {
    const AUDIO_EXTENSIONS: [&str; 10] = [
        "m4a", "webm", "opus", "mp3", "aac", "flac", "wav", "ogg", "oga", "mka",
    ];
    AUDIO_EXTENSIONS
        .into_iter()
        .filter_map(|extension| find_latest_file_in_dir(dir, extension))
        .max_by_key(|path| {
            path.metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
        })
}

pub(super) fn find_latest_file_in_dir(dir: &Path, extension: &str) -> Option<PathBuf> {
    let extension = extension.trim().trim_start_matches('.');
    fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(extension))
        })
        .max_by_key(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok())
}

pub(super) fn music_output_stem_template_for_title(title: &str) -> String {
    let (artist, title) = split_artist_title(title);
    match artist {
        Some(artist) if !artist.trim().is_empty() => format!("{artist} - {title}"),
        _ => title.to_owned(),
    }
}

pub(super) fn music_output_stem_for_title(title: &str) -> String {
    music_output_stem_template_for_title(title)
}

pub(super) fn split_artist_title(title: &str) -> (Option<String>, String) {
    let trimmed = title.trim();
    if let Some((artist, title)) = trimmed.split_once(" - ") {
        let artist = artist.trim();
        let title = title.trim();
        if !artist.is_empty() && !title.is_empty() {
            return (Some(artist.to_owned()), title.to_owned());
        }
    }
    (None, trimmed.to_owned())
}

pub(super) fn append_music_metadata_args(command: &mut Command, job: &MusicDownloadJob) {
    let (artist, title) = split_artist_title(&job.title);
    if let Some(artist) = artist
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        command.arg("-metadata").arg(format!("artist={artist}"));
    }
    let title = title.trim();
    if !title.is_empty() {
        command.arg("-metadata").arg(format!("title={title}"));
    }
    let album = job.album_title.trim();
    if !album.is_empty() {
        command.arg("-metadata").arg(format!("album={album}"));
    }
}

pub(super) fn music_download_tool_kind(source_kind: MusicDownloadSourceKind) -> ToolKind {
    match source_kind {
        MusicDownloadSourceKind::CacheCopy => ToolKind::Other("cache".to_owned()),
        MusicDownloadSourceKind::CacheConvert => ToolKind::Ffmpeg,
        MusicDownloadSourceKind::YtDlpOnlineTarget | MusicDownloadSourceKind::YtDlpDownload => {
            ToolKind::YtDlp
        }
    }
}

pub(super) fn music_cache_can_be_copied_for_choice(
    choice: MusicDownloadChoice,
    path: &Path,
    has_cover: bool,
) -> bool {
    if has_cover && music_output_path_supports_embedded_cover(path) {
        return false;
    }
    match choice.target_format() {
        Some(format) => music_download_format_matches_cache(format, path),
        None => true,
    }
}

pub(super) fn music_download_format_matches_cache(
    format: MusicDownloadFormat,
    path: &Path,
) -> bool {
    let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    let ext = ext.trim().trim_start_matches('.');
    ext.eq_ignore_ascii_case(format.extension())
}

pub(super) fn music_output_extension_for_choice(
    choice: MusicDownloadChoice,
    source_path: &Path,
) -> String {
    choice
        .target_format()
        .map(MusicDownloadFormat::extension)
        .or_else(|| {
            source_path
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.trim().trim_start_matches('.'))
                .filter(|value| !value.is_empty())
        })
        .unwrap_or("audio")
        .to_owned()
}

pub(super) fn music_output_path_supports_embedded_cover(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(music_extension_supports_embedded_cover)
}

pub(super) fn music_extension_supports_embedded_cover(extension: &str) -> bool {
    matches!(
        extension
            .trim()
            .trim_start_matches('.')
            .to_ascii_lowercase()
            .as_str(),
        "mp3" | "m4a" | "flac"
    )
}

pub(super) fn resolve_music_audio_export_plan(
    format: MusicDownloadFormat,
    source: &MusicAudioSourceProfile,
) -> MusicAudioExportPlan {
    // Audio export follows a conservative source-aware model:
    // 1. never re-encode when the source codec already matches the target,
    // 2. prefer the online source selected by yt-dlp when it already exists,
    // 3. only then encode with a preserve-perceived-quality heuristic.
    //
    // The heuristic intentionally avoids exposing bitrate/sample-rate knobs in the UI.
    // It is based on public codec recommendations and listening-test consensus, not on
    // a promise of mathematically lossless output for lossy transcodes.
    if music_source_codec_matches_target_format(format, &source.acodec) {
        return MusicAudioExportPlan {
            ffmpeg_args: vec!["-c:a".to_owned(), "copy".to_owned()],
        };
    }

    MusicAudioExportPlan {
        ffmpeg_args: encode_music_audio_args_for_intent(
            format,
            MusicAudioQualityIntent::PreservePerceivedQuality,
            source,
        ),
    }
}

pub(super) fn encode_music_audio_args_for_intent(
    format: MusicDownloadFormat,
    intent: MusicAudioQualityIntent,
    source: &MusicAudioSourceProfile,
) -> Vec<String> {
    match intent {
        MusicAudioQualityIntent::PreservePerceivedQuality => match format {
            MusicDownloadFormat::Mp3 => vec![
                "-c:a".to_owned(),
                "libmp3lame".to_owned(),
                "-q:a".to_owned(),
                mp3_quality_for_source(source).to_string(),
            ],
            MusicDownloadFormat::M4aAac => vec![
                "-c:a".to_owned(),
                "aac".to_owned(),
                "-b:a".to_owned(),
                format!("{}k", lossy_bitrate_for_source(source, 160, 192, 256, 320)),
            ],
            MusicDownloadFormat::Opus => vec![
                "-c:a".to_owned(),
                "libopus".to_owned(),
                "-b:a".to_owned(),
                format!("{}k", lossy_bitrate_for_source(source, 96, 128, 160, 192)),
            ],
            MusicDownloadFormat::Flac => vec![
                "-c:a".to_owned(),
                "flac".to_owned(),
                "-compression_level".to_owned(),
                "8".to_owned(),
            ],
            MusicDownloadFormat::Wav => {
                vec!["-c:a".to_owned(), "pcm_s16le".to_owned(), "-vn".to_owned()]
            }
        },
    }
}

pub(super) fn mp3_quality_for_source(source: &MusicAudioSourceProfile) -> u8 {
    match source.bitrate_kbps {
        Some(value) if value <= 96 => 5,
        Some(value) if value <= 160 => 3,
        Some(value) if value <= 224 => 2,
        _ => 0,
    }
}

pub(super) fn lossy_bitrate_for_source(
    source: &MusicAudioSourceProfile,
    low: u32,
    mid: u32,
    high: u32,
    max: u32,
) -> u32 {
    let Some(source_bitrate) = source.bitrate_kbps else {
        return high;
    };
    let mono_or_narrowband = source.channels == Some(1)
        || source
            .sample_rate
            .is_some_and(|sample_rate| sample_rate <= 24_000);
    let selected = if source_bitrate <= 96 {
        low
    } else if source_bitrate <= 160 {
        mid
    } else if source_bitrate <= 256 {
        high
    } else {
        max
    };
    if mono_or_narrowband {
        (selected / 2).max(64)
    } else {
        selected
    }
}

pub(super) fn probe_music_audio_source_profile(
    tool_paths: &ToolPaths,
    input_path: &Path,
    fallback_acodec: &str,
) -> MusicAudioSourceProfile {
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    let ffprobe = ffprobe_companion_path_for_ffmpeg(&ffmpeg);
    let mut profile = MusicAudioSourceProfile::from_codec(fallback_acodec);
    let Ok(info) = probe_media_with_ffprobe(&ffprobe, input_path) else {
        return profile;
    };
    let Some(audio) = info.audio else {
        return profile;
    };
    if let Some(codec) = audio.codec.filter(|value| !value.trim().is_empty()) {
        profile.acodec = codec;
    }
    profile.bitrate_kbps = audio
        .bitrate_bps
        .map(|value| ((value as f64) / 1000.0).round().max(1.0) as u32);
    profile.sample_rate = audio.sample_rate;
    profile.channels = audio.channels;
    profile
}

pub(super) fn music_cache_source_matches_target(
    format: MusicDownloadFormat,
    path: &Path,
    source_acodec: &str,
) -> bool {
    music_download_format_matches_cache(format, path)
        || music_source_codec_matches_target_format(format, source_acodec)
}

pub(super) fn online_music_target_source_available(
    tool_paths: &ToolPaths,
    job: &MusicDownloadJob,
    format: MusicDownloadFormat,
) -> bool {
    let selector = music_online_target_format_selector(format);
    let Ok(output) = tool_paths.analyze_music_stream_url_with_selector(
        &job.source_url,
        job.use_cookies,
        selector,
    ) else {
        return false;
    };
    let Ok(seed) = music_stream_seed_from_json(&output.json, &job.source_url) else {
        return false;
    };
    music_source_codec_matches_target_format(format, &seed.acodec)
}

pub(super) fn music_source_codec_matches_target_format(
    format: MusicDownloadFormat,
    source_acodec: &str,
) -> bool {
    let normalized = normalize_music_source_codec(source_acodec);
    if normalized.is_empty() || normalized == "none" {
        return false;
    }
    match format {
        MusicDownloadFormat::Mp3 => normalized == "mp3",
        MusicDownloadFormat::M4aAac => normalized == "aac",
        MusicDownloadFormat::Opus => normalized == "opus",
        MusicDownloadFormat::Flac => normalized == "flac",
        MusicDownloadFormat::Wav => normalized.starts_with("pcm_"),
    }
}

pub(super) fn normalize_music_source_codec(source_acodec: &str) -> String {
    let codec = source_acodec.trim().to_ascii_lowercase();
    if codec.starts_with("mp4a") || codec == "aac_latm" {
        "aac".to_owned()
    } else if codec.starts_with("opus") {
        "opus".to_owned()
    } else if codec.starts_with("mp3") || codec == "libmp3lame" {
        "mp3".to_owned()
    } else if codec.starts_with("flac") {
        "flac".to_owned()
    } else {
        codec
    }
}

pub(super) fn unique_timestamp_suffix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(super) fn music_cache_updated_is_fresh(updated_unix_seconds: u64) -> bool {
    updated_unix_seconds > 0
        && unique_timestamp_suffix().saturating_sub(updated_unix_seconds)
            <= MUSIC_STREAM_CACHE_TTL_SECONDS
}

pub(super) fn audio_cache_manifest_is_fresh(manifest: &AudioCacheManifestSnapshot) -> bool {
    music_cache_updated_is_fresh(manifest.updated_unix_seconds)
}

pub(super) fn sanitize_music_cache_key(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "unknown".to_owned();
    }
    trimmed
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub(super) fn music_cache_manifest_progress_ratio(
    manifest_path: &Path,
    fallback_expected_bytes: Option<u64>,
) -> Option<f32> {
    let manifest = read_yaml_file::<AudioCacheManifestSnapshot>(manifest_path)?;
    if !audio_cache_manifest_is_fresh(&manifest) {
        return None;
    }
    if manifest.complete {
        return Some(1.0);
    }
    let expected = manifest
        .expected_bytes
        .or(fallback_expected_bytes)
        .filter(|value| *value > 0)?;
    let range_bytes = manifest
        .ranges
        .iter()
        .map(|range| range.end.saturating_sub(range.start))
        .sum::<u64>();
    let downloaded = (range_bytes > 0)
        .then_some(range_bytes)
        .or(manifest.downloaded_bytes)?;
    Some((downloaded as f32 / expected as f32).clamp(0.0, 1.0))
}

pub(super) fn sanitize_music_cache_ext(value: &str) -> String {
    let cleaned = value.trim().trim_start_matches('.');
    if cleaned.is_empty() {
        "bin".to_owned()
    } else {
        cleaned
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase()
    }
}

pub(super) fn calculate_cache_management_summary(root: &Path) -> CacheManagementSummary {
    let total_bytes = dir_size_bytes(root);
    let music_root = root.join("audio");
    let music_bytes = dir_size_bytes(&music_root);
    let expired_music_bytes = expired_music_cache_size_bytes(&music_root);
    CacheManagementSummary {
        total_bytes,
        music_bytes,
        expired_music_bytes,
    }
}

pub(super) fn dir_size_bytes(path: &Path) -> u64 {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return 0;
    };
    if metadata.is_file() {
        return metadata.len();
    }
    if !metadata.is_dir() {
        return 0;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| dir_size_bytes(&entry.path()))
        .sum()
}

pub(super) fn expired_music_cache_size_bytes(root: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(root) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| music_cache_dir_is_expired(path))
        .map(|path| dir_size_bytes(&path))
        .sum()
}

pub(super) fn remove_expired_music_cache_dirs(root: &Path) -> std::io::Result<CacheRemovalSummary> {
    let mut summary = CacheRemovalSummary::default();
    let Ok(entries) = fs::read_dir(root) else {
        return Ok(summary);
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !music_cache_dir_is_expired(&path) {
            continue;
        }
        let bytes = dir_size_bytes(&path);
        remove_path(&path)?;
        summary.bytes = summary.bytes.saturating_add(bytes);
        summary.entries = summary.entries.saturating_add(1);
    }
    Ok(summary)
}

pub(super) fn music_cache_dir_is_expired(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    if path.file_name().and_then(|value| value.to_str()) == Some("covers") {
        return false;
    }

    let manifest_path = path.join("manifest.yaml");
    if let Some(manifest) = read_yaml_file::<AudioCacheManifestSnapshot>(&manifest_path) {
        return !audio_cache_manifest_is_fresh(&manifest);
    }

    path_modified_age_seconds(path).is_some_and(|age| age > MUSIC_STREAM_CACHE_TTL_SECONDS)
}

pub(super) fn path_modified_age_seconds(path: &Path) -> Option<u64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    SystemTime::now()
        .duration_since(modified)
        .ok()
        .map(|duration| duration.as_secs())
}

pub(super) fn remove_path_contents_or_dir(path: &Path) -> std::io::Result<CacheRemovalSummary> {
    let summary = CacheRemovalSummary {
        bytes: dir_size_bytes(path),
        entries: if path.exists() { 1 } else { 0 },
    };
    if path.exists() {
        remove_path(path)?;
    }
    Ok(summary)
}

pub(super) fn remove_safe_app_cache_contents(path: &Path) -> std::io::Result<CacheRemovalSummary> {
    ensure_safe_app_cache_root(path)?;
    remove_dir_contents_collecting_summary(path)
}

pub(super) fn ensure_safe_app_cache_root(path: &Path) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};

    let path = normalized_path_for_safety(path);
    if path.parent().is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Refusing to clear a filesystem root as cache.",
        ));
    }

    let mut protected_roots = Vec::new();
    if let Ok(current_dir) = std::env::current_dir() {
        protected_roots.push(normalized_path_for_safety(&current_dir));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            protected_roots.push(normalized_path_for_safety(parent));
        }
    }
    for var in ["USERPROFILE", "HOME"] {
        if let Ok(home) = std::env::var(var) {
            let home = normalized_path_for_safety(Path::new(&home));
            protected_roots.push(home.clone());
            for child in [
                "Desktop",
                "Downloads",
                "Documents",
                "Pictures",
                "Videos",
                "Music",
            ] {
                protected_roots.push(normalized_path_for_safety(&home.join(child)));
            }
        }
    }

    if protected_roots.iter().any(|protected| protected == &path) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Refusing to clear a protected folder as cache. Choose an app-owned cache folder.",
        ));
    }

    Ok(())
}

pub(super) fn app_portable_root_path() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    }

    #[cfg(not(debug_assertions))]
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub(super) fn normalized_path_for_safety(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    absolute.canonicalize().unwrap_or(absolute)
}

pub(super) fn remove_dir_contents_collecting_summary(
    path: &Path,
) -> std::io::Result<CacheRemovalSummary> {
    let mut summary = CacheRemovalSummary::default();
    let Ok(entries) = fs::read_dir(path) else {
        return Ok(summary);
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let bytes = dir_size_bytes(&path);
        remove_path(&path)?;
        summary.bytes = summary.bytes.saturating_add(bytes);
        summary.entries = summary.entries.saturating_add(1);
    }
    Ok(summary)
}

pub(super) fn remove_path(path: &Path) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

pub(super) fn format_byte_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0_usize;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
