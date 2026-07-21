use super::*;
use crate::app::metadata::default_format_id;

#[test]
fn prepare_update_status_hides_stale_failed_for_installed_tool() {
    assert!(!prepare_dependency_update_status_is_visible(
        ComponentUpdateStatus::Failed,
        true,
        false,
        true,
    ));
}

#[test]
fn prepare_update_status_keeps_failed_for_missing_tool() {
    assert!(prepare_dependency_update_status_is_visible(
        ComponentUpdateStatus::Failed,
        true,
        false,
        false,
    ));
}

#[test]
fn prepare_update_status_keeps_running_states_visible() {
    assert!(prepare_dependency_update_status_is_visible(
        ComponentUpdateStatus::Downloading,
        true,
        true,
        true,
    ));
}

#[test]
fn component_update_signal_tracks_update_attention_statuses() {
    assert!(component_update_status_needs_attention_signal(
        ComponentUpdateStatus::UpdateAvailable
    ));
    assert!(component_update_status_needs_attention_signal(
        ComponentUpdateStatus::PendingRestart
    ));
    assert!(!component_update_status_needs_attention_signal(
        ComponentUpdateStatus::UpToDate
    ));
    assert!(!component_update_status_needs_attention_signal(
        ComponentUpdateStatus::Missing
    ));
}

#[test]
fn default_video_format_prefers_highest_resolution() {
    let formats = vec![
        FormatOption::video(
            "low",
            "low",
            MediaKind::Video,
            "640x360",
            "",
            "",
            "mp4",
            "h264",
            "10.00 MB",
        ),
        FormatOption::video(
            "high",
            "high",
            MediaKind::Video,
            "1920x1080",
            "",
            "",
            "mp4",
            "h264",
            "20.00 MB",
        ),
        FormatOption::video(
            "mid",
            "mid",
            MediaKind::Video,
            "1280x720",
            "",
            "",
            "mp4",
            "h264",
            "30.00 MB",
        ),
    ];

    assert_eq!(default_format_id(&formats, &[MediaKind::Video]), "high");
}

#[test]
fn requested_format_still_wins_over_quality_guess() {
    let formats = vec![
        FormatOption::video(
            "requested",
            "requested",
            MediaKind::Video,
            "640x360",
            "",
            "",
            "mp4",
            "h264",
            "10.00 MB",
        ),
        FormatOption::video(
            "high",
            "high",
            MediaKind::Video,
            "1920x1080",
            "",
            "",
            "mp4",
            "h264",
            "20.00 MB",
        ),
    ];

    assert_eq!(
        requested_or_default_format_id(&formats, &[String::from("requested")], &[MediaKind::Video],),
        "requested"
    );
}

#[test]
fn display_file_stem_drops_extension_from_auto_name() {
    assert_eq!(
        display_file_stem(r"download\sample title [abc123].webm"),
        "sample title [abc123]"
    );
}

#[test]
fn first_audio_format_ignores_muxed_formats() {
    let metadata = VideoMetadata {
        formats: vec![
            FormatOption::video(
                "muxed",
                "muxed",
                MediaKind::Muxed,
                "1280x720",
                "",
                "30",
                "mp4",
                "h264",
                "10.00 MB",
            ),
            FormatOption::audio(
                "audio",
                "audio",
                MediaKind::Audio,
                "48000",
                "m4a",
                "aac",
                "3.00 MB",
            ),
        ],
        ..VideoMetadata::empty_preview()
    };

    assert_eq!(
        first_audio_format_id(Some(&metadata)).as_deref(),
        Some("audio")
    );
}

#[test]
fn music_audio_export_plan_copies_matching_opus_source() {
    let plan = resolve_music_audio_export_plan(
        MusicDownloadFormat::Opus,
        &MusicAudioSourceProfile::from_codec("opus"),
    );
    assert_eq!(plan.ffmpeg_args, vec!["-c:a".to_owned(), "copy".to_owned()]);
}

#[test]
fn music_audio_export_plan_encodes_when_source_codec_differs() {
    let plan = resolve_music_audio_export_plan(
        MusicDownloadFormat::Opus,
        &MusicAudioSourceProfile::from_codec("aac"),
    );
    assert!(plan.ffmpeg_args.iter().any(|arg| arg == "libopus"));
}

#[test]
fn music_audio_export_plan_treats_mp4a_as_aac_for_m4a() {
    let plan = resolve_music_audio_export_plan(
        MusicDownloadFormat::M4aAac,
        &MusicAudioSourceProfile::from_codec("mp4a.40.2"),
    );
    assert_eq!(plan.ffmpeg_args, vec!["-c:a".to_owned(), "copy".to_owned()]);
}

#[test]
fn music_audio_export_plan_uses_source_bitrate_for_opus() {
    let source = MusicAudioSourceProfile {
        acodec: "aac".to_owned(),
        bitrate_kbps: Some(128),
        sample_rate: Some(48_000),
        channels: Some(2),
    };
    let plan = resolve_music_audio_export_plan(MusicDownloadFormat::Opus, &source);
    assert_eq!(
        plan.ffmpeg_args,
        vec![
            "-c:a".to_owned(),
            "libopus".to_owned(),
            "-b:a".to_owned(),
            "128k".to_owned(),
        ]
    );
}

#[test]
fn music_audio_export_plan_reduces_mono_or_narrowband_bitrate() {
    let source = MusicAudioSourceProfile {
        acodec: "aac".to_owned(),
        bitrate_kbps: Some(160),
        sample_rate: Some(22_050),
        channels: Some(1),
    };
    let plan = resolve_music_audio_export_plan(MusicDownloadFormat::Opus, &source);
    assert_eq!(
        plan.ffmpeg_args,
        vec![
            "-c:a".to_owned(),
            "libopus".to_owned(),
            "-b:a".to_owned(),
            "64k".to_owned(),
        ]
    );
}

#[test]
fn music_online_target_selector_prefers_requested_codec_before_fallback() {
    assert!(
        music_online_target_format_selector(MusicDownloadFormat::Opus)
            .starts_with("bestaudio[acodec^=opus]")
    );
    assert!(
        music_online_target_format_selector(MusicDownloadFormat::M4aAac)
            .starts_with("bestaudio[ext=m4a]")
    );
}

#[test]
fn recovered_tool_log_step_does_not_keep_parent_failed() {
    let steps = vec![
        tool_log_step_for_test(ToolLogStatus::Recovered),
        tool_log_step_for_test(ToolLogStatus::Skipped),
        tool_log_step_for_test(ToolLogStatus::Success),
    ];

    assert_eq!(aggregate_tool_log_status(&steps), ToolLogStatus::Success);
}

#[test]
fn unrecovered_failed_tool_log_step_keeps_parent_failed() {
    let steps = vec![
        tool_log_step_for_test(ToolLogStatus::Failed),
        tool_log_step_for_test(ToolLogStatus::Skipped),
        tool_log_step_for_test(ToolLogStatus::Success),
    ];

    assert_eq!(aggregate_tool_log_status(&steps), ToolLogStatus::Failed);
}

#[test]
fn recovered_tool_log_without_later_success_is_recovered_not_failed() {
    let steps = vec![
        tool_log_step_for_test(ToolLogStatus::Recovered),
        tool_log_step_for_test(ToolLogStatus::Skipped),
    ];

    assert_eq!(aggregate_tool_log_status(&steps), ToolLogStatus::Recovered);
}

#[test]
fn cache_summary_counts_only_flat_audio_namespace() {
    let root = std::env::temp_dir().join(format!(
        "yt-dlp-gui-v2-audio-cache-summary-test-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let current = root.join("audio").join("current");
    let legacy = root.join("music-stream").join("legacy");
    fs::create_dir_all(&current).expect("create current audio cache");
    fs::create_dir_all(&legacy).expect("create legacy audio cache");
    fs::write(current.join("audio.bin"), [1u8; 3]).expect("write current cache");
    fs::write(legacy.join("audio.bin"), [1u8; 5]).expect("write legacy cache");

    let summary = calculate_cache_management_summary(&root);

    assert_eq!(summary.music_bytes, 3);

    let _ = fs::remove_dir_all(root);
}

fn tool_log_step_for_test(status: ToolLogStatus) -> ToolLogStep {
    ToolLogStep {
        id: 0,
        status,
        tool: String::new(),
        action: String::new(),
        command: String::new(),
        detail: None,
    }
}
