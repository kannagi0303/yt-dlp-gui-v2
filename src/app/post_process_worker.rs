use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::app::media_probe::{
    MediaProbeInfo, ffprobe_companion_path_for_ffmpeg, probe_media_with_ffprobe,
};
use crate::domain::{QueueItemId, WorkflowRunId};
use crate::infrastructure::{
    AudioPolicy, ContainerPolicy, SubtitlePolicy, ToolPaths, TranscodeIntentSettings,
    VideoCodecPolicy, configure_background_command, resolve_tool_path,
};

pub(super) const POST_PROCESS_CANCELLED_MESSAGE: &str = "Post-processing cancelled.";

pub(super) struct PostProcessResult {
    pub item_id: QueueItemId,
    pub workflow_id: WorkflowRunId,
    pub result: Result<String, String>,
}

pub(super) enum PostProcessEvent {
    Progress {
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        percent: f32,
    },
    ToolCommandFinished {
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        tool: String,
        action: String,
        command_line: String,
        success: bool,
    },
    Finished(PostProcessResult),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BuiltInTranscodeProfile {
    OutputConversion,
}

impl BuiltInTranscodeProfile {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::OutputConversion => "Output conversion",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VideoEncoderKind {
    Copy,
    H264Nvenc,
    H264Qsv,
    H264Amf,
    LibX264,
    HevcNvenc,
    HevcQsv,
    HevcAmf,
    LibX265,
    Av1Nvenc,
    Av1Qsv,
    Av1Amf,
    LibSvtAv1,
    LibAomAv1,
}

impl VideoEncoderKind {
    fn label(self) -> &'static str {
        match self {
            Self::Copy => "copy video",
            Self::H264Nvenc => "NVIDIA NVENC H.264",
            Self::H264Qsv => "Intel Quick Sync H.264",
            Self::H264Amf => "AMD AMF H.264",
            Self::LibX264 => "libx264",
            Self::HevcNvenc => "NVIDIA NVENC HEVC",
            Self::HevcQsv => "Intel Quick Sync HEVC",
            Self::HevcAmf => "AMD AMF HEVC",
            Self::LibX265 => "libx265",
            Self::Av1Nvenc => "NVIDIA NVENC AV1",
            Self::Av1Qsv => "Intel Quick Sync AV1",
            Self::Av1Amf => "AMD AMF AV1",
            Self::LibSvtAv1 => "SVT-AV1",
            Self::LibAomAv1 => "libaom-av1",
        }
    }

    fn ffmpeg_name(self) -> Option<&'static str> {
        match self {
            Self::Copy => None,
            Self::H264Nvenc => Some("h264_nvenc"),
            Self::H264Qsv => Some("h264_qsv"),
            Self::H264Amf => Some("h264_amf"),
            Self::LibX264 => Some("libx264"),
            Self::HevcNvenc => Some("hevc_nvenc"),
            Self::HevcQsv => Some("hevc_qsv"),
            Self::HevcAmf => Some("hevc_amf"),
            Self::LibX265 => Some("libx265"),
            Self::Av1Nvenc => Some("av1_nvenc"),
            Self::Av1Qsv => Some("av1_qsv"),
            Self::Av1Amf => Some("av1_amf"),
            Self::LibSvtAv1 => Some("libsvtav1"),
            Self::LibAomAv1 => Some("libaom-av1"),
        }
    }

    fn is_hardware(self) -> bool {
        matches!(
            self,
            Self::H264Nvenc
                | Self::H264Qsv
                | Self::H264Amf
                | Self::HevcNvenc
                | Self::HevcQsv
                | Self::HevcAmf
                | Self::Av1Nvenc
                | Self::Av1Qsv
                | Self::Av1Amf
        )
    }

    fn fallback_args(self) -> &'static [&'static str] {
        match self {
            Self::Copy => &["-c:v", "copy"],
            Self::H264Nvenc => &[
                "-c:v",
                "h264_nvenc",
                "-preset",
                "medium",
                "-cq",
                "23",
                "-b:v",
                "0",
                "-pix_fmt",
                "yuv420p",
            ],
            Self::H264Qsv => &[
                "-c:v",
                "h264_qsv",
                "-preset",
                "medium",
                "-global_quality",
                "23",
                "-pix_fmt",
                "nv12",
            ],
            Self::H264Amf => &[
                "-c:v", "h264_amf", "-quality", "balanced", "-rc", "cqp", "-qp_i", "21", "-qp_p",
                "23", "-pix_fmt", "yuv420p",
            ],
            Self::LibX264 => &[
                "-c:v", "libx264", "-preset", "medium", "-crf", "20", "-pix_fmt", "yuv420p",
            ],
            Self::HevcNvenc => &[
                "-c:v",
                "hevc_nvenc",
                "-preset",
                "medium",
                "-cq",
                "26",
                "-b:v",
                "0",
                "-pix_fmt",
                "yuv420p",
            ],
            Self::HevcQsv => &[
                "-c:v",
                "hevc_qsv",
                "-preset",
                "medium",
                "-global_quality",
                "26",
            ],
            Self::HevcAmf => &[
                "-c:v", "hevc_amf", "-quality", "balanced", "-rc", "cqp", "-qp_i", "24", "-qp_p",
                "26", "-pix_fmt", "yuv420p",
            ],
            Self::LibX265 => &[
                "-c:v", "libx265", "-preset", "medium", "-crf", "25", "-pix_fmt", "yuv420p",
            ],
            Self::Av1Nvenc => &[
                "-c:v",
                "av1_nvenc",
                "-preset",
                "medium",
                "-cq",
                "28",
                "-b:v",
                "0",
            ],
            Self::Av1Qsv => &[
                "-c:v",
                "av1_qsv",
                "-preset",
                "medium",
                "-global_quality",
                "28",
            ],
            Self::Av1Amf => &[
                "-c:v", "av1_amf", "-quality", "balanced", "-rc", "cqp", "-qp_i", "27", "-qp_p",
                "29",
            ],
            Self::LibSvtAv1 => &["-c:v", "libsvtav1", "-preset", "8", "-crf", "30"],
            Self::LibAomAv1 => &[
                "-c:v",
                "libaom-av1",
                "-cpu-used",
                "6",
                "-crf",
                "30",
                "-b:v",
                "0",
            ],
        }
    }

    fn args(self, media: Option<&MediaProbeInfo>) -> Vec<String> {
        if let Some(kbps) = target_video_bitrate_kbps(media, self.target_codec()) {
            return self.bitrate_args(kbps);
        }

        self.fallback_args()
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect()
    }

    fn target_codec(self) -> Option<TargetVideoCodec> {
        match self {
            Self::H264Nvenc | Self::H264Qsv | Self::H264Amf | Self::LibX264 => {
                Some(TargetVideoCodec::H264)
            }
            Self::HevcNvenc | Self::HevcQsv | Self::HevcAmf | Self::LibX265 => {
                Some(TargetVideoCodec::Hevc)
            }
            Self::Av1Nvenc | Self::Av1Qsv | Self::Av1Amf | Self::LibSvtAv1 | Self::LibAomAv1 => {
                Some(TargetVideoCodec::Av1)
            }
            Self::Copy => None,
        }
    }

    fn bitrate_args(self, kbps: u64) -> Vec<String> {
        let bitrate = format!("{}k", kbps.max(96));
        let maxrate = format!("{}k", ((kbps as f32) * 1.6).round().max(kbps as f32) as u64);
        let bufsize = format!("{}k", (kbps.saturating_mul(3)).max(256));
        let mut args = match self {
            Self::Copy => vec!["-c:v", "copy"],
            Self::H264Nvenc => vec!["-c:v", "h264_nvenc", "-preset", "medium", "-rc", "vbr"],
            Self::H264Qsv => vec!["-c:v", "h264_qsv", "-preset", "medium"],
            Self::H264Amf => vec![
                "-c:v", "h264_amf", "-quality", "balanced", "-rc", "vbr_peak",
            ],
            Self::LibX264 => vec!["-c:v", "libx264", "-preset", "medium"],
            Self::HevcNvenc => vec!["-c:v", "hevc_nvenc", "-preset", "medium", "-rc", "vbr"],
            Self::HevcQsv => vec!["-c:v", "hevc_qsv", "-preset", "medium"],
            Self::HevcAmf => vec![
                "-c:v", "hevc_amf", "-quality", "balanced", "-rc", "vbr_peak",
            ],
            Self::LibX265 => vec!["-c:v", "libx265", "-preset", "medium"],
            Self::Av1Nvenc => vec!["-c:v", "av1_nvenc", "-preset", "medium", "-rc", "vbr"],
            Self::Av1Qsv => vec!["-c:v", "av1_qsv", "-preset", "medium"],
            Self::Av1Amf => vec!["-c:v", "av1_amf", "-quality", "balanced", "-rc", "vbr_peak"],
            Self::LibSvtAv1 => vec!["-c:v", "libsvtav1", "-preset", "8"],
            Self::LibAomAv1 => vec!["-c:v", "libaom-av1", "-cpu-used", "6"],
        }
        .into_iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

        args.extend([
            "-b:v".to_owned(),
            bitrate,
            "-maxrate".to_owned(),
            maxrate,
            "-bufsize".to_owned(),
            bufsize,
        ]);

        if matches!(
            self,
            Self::H264Nvenc
                | Self::H264Amf
                | Self::LibX264
                | Self::HevcNvenc
                | Self::HevcAmf
                | Self::LibX265
        ) {
            args.extend(["-pix_fmt".to_owned(), "yuv420p".to_owned()]);
        }

        args
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TargetVideoCodec {
    H264,
    Hevc,
    Av1,
}

fn target_video_bitrate_kbps(
    media: Option<&MediaProbeInfo>,
    target_codec: Option<TargetVideoCodec>,
) -> Option<u64> {
    let media = media?;
    let target_codec = target_codec?;
    let video = media.video.as_ref()?;
    let source_bitrate = video.bitrate_bps?;
    let source_codec = video.codec.as_deref()?;
    let ratio = codec_bitrate_ratio(source_codec, target_codec, video.height)?;
    Some(((source_bitrate as f32 * ratio) / 1000.0).round().max(96.0) as u64)
}

fn codec_bitrate_ratio(
    source_codec: &str,
    target_codec: TargetVideoCodec,
    height: Option<u32>,
) -> Option<f32> {
    let source = normalized_video_codec_name(source_codec);
    let resolution = match height.unwrap_or(1080) {
        0..=720 => ResolutionClass::P720,
        721..=1080 => ResolutionClass::P1080,
        _ => ResolutionClass::P4k,
    };

    match (source.as_deref(), target_codec, resolution) {
        (Some("h264"), TargetVideoCodec::H264, _) => Some(1.00),
        (Some("h264"), TargetVideoCodec::Hevc, ResolutionClass::P720) => Some(0.70),
        (Some("h264"), TargetVideoCodec::Hevc, ResolutionClass::P1080) => Some(0.55),
        (Some("h264"), TargetVideoCodec::Hevc, ResolutionClass::P4k) => Some(0.50),
        (Some("h264"), TargetVideoCodec::Av1, ResolutionClass::P720) => Some(0.65),
        (Some("h264"), TargetVideoCodec::Av1, ResolutionClass::P1080) => Some(0.50),
        (Some("h264"), TargetVideoCodec::Av1, ResolutionClass::P4k) => Some(0.45),
        (Some("hevc"), TargetVideoCodec::H264, ResolutionClass::P720) => Some(1.45),
        (Some("hevc"), TargetVideoCodec::H264, ResolutionClass::P1080) => Some(1.80),
        (Some("hevc"), TargetVideoCodec::H264, ResolutionClass::P4k) => Some(2.00),
        (Some("hevc"), TargetVideoCodec::Hevc, _) => Some(1.00),
        (Some("hevc"), TargetVideoCodec::Av1, _) => Some(0.85),
        (Some("vp9"), TargetVideoCodec::H264, _) => Some(1.35),
        (Some("vp9"), TargetVideoCodec::Hevc, _) => Some(1.00),
        (Some("vp9"), TargetVideoCodec::Av1, _) => Some(0.85),
        (Some("av1"), TargetVideoCodec::H264, _) => Some(1.80),
        (Some("av1"), TargetVideoCodec::Hevc, _) => Some(1.20),
        (Some("av1"), TargetVideoCodec::Av1, _) => Some(1.00),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResolutionClass {
    P720,
    P1080,
    P4k,
}

fn normalized_video_codec_name(codec: &str) -> Option<&'static str> {
    match codec.trim().to_ascii_lowercase().as_str() {
        "h264" | "avc" => Some("h264"),
        "hevc" | "h265" => Some("hevc"),
        "av1" => Some("av1"),
        "vp9" => Some("vp9"),
        _ => None,
    }
}

pub(super) fn run_builtin_transcode_worker(
    tool_paths: ToolPaths,
    settings: TranscodeIntentSettings,
    input_path: String,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: mpsc::Sender<PostProcessEvent>,
    child_handle: Arc<Mutex<Option<Child>>>,
    cancel_requested: Arc<AtomicBool>,
) {
    let result = run_output_conversion_attempt(
        &tool_paths,
        &settings,
        &input_path,
        item_id,
        workflow_id,
        tx.clone(),
        &child_handle,
        &cancel_requested,
    );

    let _ = tx.send(PostProcessEvent::Finished(PostProcessResult {
        item_id,
        workflow_id,
        result,
    }));
}

pub(super) fn request_post_process_stop(
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) {
    cancel_requested.store(true, Ordering::Relaxed);
    if let Ok(mut guard) = child_handle.lock() {
        if let Some(child) = guard.as_mut() {
            terminate_child_process_tree(child);
        }
    }
}

fn run_output_conversion_attempt(
    tool_paths: &ToolPaths,
    settings: &TranscodeIntentSettings,
    input_path: &str,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: mpsc::Sender<PostProcessEvent>,
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<String, String> {
    let input_path = PathBuf::from(input_path);
    if !input_path.is_file() {
        return Err(format!(
            "Input media file was not found: {}",
            input_path.display()
        ));
    }

    let ffmpeg = validate_ffmpeg_available(tool_paths)?;
    let media_probe = probe_input_media(&ffmpeg, &input_path);
    let sidecar_subtitle = find_sidecar_subtitle(&input_path);
    let mut effective_settings = settings.clone();
    if settings.subtitle_policy == SubtitlePolicy::Burn
        && sidecar_subtitle.is_none()
        && !media_probe.as_ref().is_some_and(|info| info.has_subtitle)
    {
        effective_settings.subtitle_policy = SubtitlePolicy::Preserve;
    }
    normalize_output_settings(&mut effective_settings, &input_path, media_probe.as_ref());

    if !output_conversion_required(&effective_settings, &input_path) {
        let _ = tx.send(PostProcessEvent::Progress {
            item_id,
            workflow_id,
            percent: 100.0,
        });
        return Ok(input_path.display().to_string());
    }

    let final_output = output_path_for(&input_path, effective_settings.container_policy);
    let temp_output = transcode_temp_output_path(&final_output);
    let video_attempts = video_encoder_attempts(&ffmpeg, effective_settings.video_codec_policy);
    if video_attempts.is_empty() {
        return Err("No usable FFmpeg video encoder was found for the selected output.".to_owned());
    }

    let mut failed_attempts = Vec::new();
    for encoder in video_attempts {
        remove_existing_temp_output(&temp_output)?;
        match run_ffmpeg_output_conversion(
            &ffmpeg,
            &effective_settings,
            encoder,
            &input_path,
            &temp_output,
            media_probe.as_ref(),
            item_id,
            workflow_id,
            tx.clone(),
            child_handle,
            cancel_requested,
        ) {
            Ok(()) => {
                replace_with_transcoded_output(&input_path, &temp_output, &final_output)?;
                let _ = tx.send(PostProcessEvent::Progress {
                    item_id,
                    workflow_id,
                    percent: 100.0,
                });
                return Ok(final_output.display().to_string());
            }
            Err(error) if error == POST_PROCESS_CANCELLED_MESSAGE => {
                let _ = fs::remove_file(&temp_output);
                return Err(error);
            }
            Err(error) if encoder.is_hardware() => {
                let _ = fs::remove_file(&temp_output);
                eprintln!(
                    "[post-process] hardware encoder failed; fallback continues: {}: {error}",
                    encoder.label()
                );
                failed_attempts.push(format!("{}: {error}", encoder.label()));
            }
            Err(error) => {
                let _ = fs::remove_file(&temp_output);
                if failed_attempts.is_empty() {
                    return Err(error);
                }
                return Err(format!(
                    "{error}; hardware fallback attempts: {}",
                    failed_attempts.join(" | ")
                ));
            }
        }
    }

    Err("FFmpeg output conversion failed: no encoder attempt succeeded".to_owned())
}

fn output_conversion_required(settings: &TranscodeIntentSettings, input_path: &Path) -> bool {
    if settings.video_codec_policy != VideoCodecPolicy::Auto
        || settings.audio_policy != AudioPolicy::Auto
        || subtitle_policy_requires_output(settings.subtitle_policy)
    {
        return true;
    }
    match settings.container_policy {
        ContainerPolicy::Auto => false,
        ContainerPolicy::Mp4 => !has_extension(input_path, "mp4"),
        ContainerPolicy::Mkv => !has_extension(input_path, "mkv"),
        ContainerPolicy::Mov => !has_extension(input_path, "mov"),
    }
}

fn subtitle_policy_requires_output(policy: SubtitlePolicy) -> bool {
    !matches!(policy, SubtitlePolicy::Preserve)
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

fn probe_input_media(ffmpeg: &Path, input_path: &Path) -> Option<MediaProbeInfo> {
    let ffprobe = ffprobe_companion_path_for_ffmpeg(ffmpeg);
    match probe_media_with_ffprobe(&ffprobe, input_path) {
        Ok(info) => Some(info),
        Err(error) => {
            eprintln!(
                "[post-process] ffprobe unavailable; using conservative encode defaults: {error}"
            );
            None
        }
    }
}

fn normalize_output_settings(
    settings: &mut TranscodeIntentSettings,
    input_path: &Path,
    media: Option<&MediaProbeInfo>,
) {
    if settings.container_policy == ContainerPolicy::Auto
        && (settings.video_codec_policy != VideoCodecPolicy::Auto
            || settings.audio_policy != AudioPolicy::Auto
            || settings.subtitle_policy == SubtitlePolicy::Embed)
    {
        if let Some(inferred) = container_policy_for_extension(input_path).filter(|container| {
            container_allowed_for_codecs(
                *container,
                settings.video_codec_policy,
                settings.audio_policy,
            )
        }) {
            settings.container_policy = inferred;
        } else {
            settings.container_policy =
                best_container_for_codecs(settings.video_codec_policy, settings.audio_policy);
        }
    }

    if settings.subtitle_policy == SubtitlePolicy::Burn
        && settings.video_codec_policy == VideoCodecPolicy::Auto
    {
        settings.video_codec_policy = VideoCodecPolicy::H264;
    }

    match settings.container_policy {
        ContainerPolicy::Mp4 | ContainerPolicy::Mov => {
            if settings.video_codec_policy == VideoCodecPolicy::Auto
                && !source_video_allowed_for_container(media, settings.container_policy)
            {
                settings.video_codec_policy = VideoCodecPolicy::H264;
            }
            if !video_allowed_for_container(settings.video_codec_policy, settings.container_policy)
            {
                settings.video_codec_policy = VideoCodecPolicy::H264;
            }

            if settings.audio_policy == AudioPolicy::Auto
                && !source_audio_allowed_for_container(media, settings.container_policy)
            {
                settings.audio_policy = AudioPolicy::Aac;
            }
            if !audio_allowed_for_container(settings.audio_policy, settings.container_policy) {
                settings.audio_policy = AudioPolicy::Aac;
            }
        }
        ContainerPolicy::Mkv | ContainerPolicy::Auto => {}
    }
}

fn container_policy_for_extension(path: &Path) -> Option<ContainerPolicy> {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("mp4" | "m4v") => Some(ContainerPolicy::Mp4),
        Some("mkv") => Some(ContainerPolicy::Mkv),
        Some("mov") => Some(ContainerPolicy::Mov),
        _ => None,
    }
}

fn best_container_for_codecs(video: VideoCodecPolicy, audio: AudioPolicy) -> ContainerPolicy {
    [
        ContainerPolicy::Mkv,
        ContainerPolicy::Mp4,
        ContainerPolicy::Mov,
    ]
    .into_iter()
    .find(|container| container_allowed_for_codecs(*container, video, audio))
    .unwrap_or(ContainerPolicy::Mkv)
}

fn container_allowed_for_codecs(
    container: ContainerPolicy,
    video: VideoCodecPolicy,
    audio: AudioPolicy,
) -> bool {
    video_allowed_for_container(video, container) && audio_allowed_for_container(audio, container)
}

fn video_allowed_for_container(video: VideoCodecPolicy, container: ContainerPolicy) -> bool {
    match container {
        ContainerPolicy::Auto | ContainerPolicy::Mkv => true,
        ContainerPolicy::Mp4 => matches!(
            video,
            VideoCodecPolicy::Auto
                | VideoCodecPolicy::H264
                | VideoCodecPolicy::Hevc
                | VideoCodecPolicy::Av1
        ),
        ContainerPolicy::Mov => matches!(
            video,
            VideoCodecPolicy::Auto | VideoCodecPolicy::H264 | VideoCodecPolicy::Hevc
        ),
    }
}

fn audio_allowed_for_container(audio: AudioPolicy, container: ContainerPolicy) -> bool {
    match container {
        ContainerPolicy::Auto | ContainerPolicy::Mkv => true,
        ContainerPolicy::Mp4 | ContainerPolicy::Mov => {
            matches!(audio, AudioPolicy::Auto | AudioPolicy::Aac)
        }
    }
}

fn source_video_allowed_for_container(
    media: Option<&MediaProbeInfo>,
    container: ContainerPolicy,
) -> bool {
    let Some(codec) = media
        .and_then(|info| info.video.as_ref())
        .and_then(|video| video.codec.as_deref())
        .and_then(normalized_video_codec_name)
    else {
        return true;
    };

    match container {
        ContainerPolicy::Auto | ContainerPolicy::Mkv => true,
        ContainerPolicy::Mp4 => matches!(codec, "h264" | "hevc" | "av1"),
        ContainerPolicy::Mov => matches!(codec, "h264" | "hevc"),
    }
}

fn source_audio_allowed_for_container(
    media: Option<&MediaProbeInfo>,
    container: ContainerPolicy,
) -> bool {
    let Some(codec) = media
        .and_then(|info| info.audio.as_ref())
        .and_then(|audio| audio.codec.as_deref())
        .map(|value| value.trim().to_ascii_lowercase())
    else {
        return true;
    };

    match container {
        ContainerPolicy::Auto | ContainerPolicy::Mkv => true,
        ContainerPolicy::Mp4 => matches!(codec.as_str(), "aac" | "mp3" | "alac"),
        ContainerPolicy::Mov => {
            matches!(codec.as_str(), "aac" | "alac" | "pcm_s16le" | "pcm_s24le")
        }
    }
}

fn video_encoder_attempts(ffmpeg: &Path, policy: VideoCodecPolicy) -> Vec<VideoEncoderKind> {
    let all = match policy {
        VideoCodecPolicy::Auto => vec![VideoEncoderKind::Copy],
        VideoCodecPolicy::H264 => vec![
            VideoEncoderKind::H264Nvenc,
            VideoEncoderKind::H264Qsv,
            VideoEncoderKind::H264Amf,
            VideoEncoderKind::LibX264,
        ],
        VideoCodecPolicy::Hevc => vec![
            VideoEncoderKind::HevcNvenc,
            VideoEncoderKind::HevcQsv,
            VideoEncoderKind::HevcAmf,
            VideoEncoderKind::LibX265,
        ],
        VideoCodecPolicy::Av1 => vec![
            VideoEncoderKind::Av1Nvenc,
            VideoEncoderKind::Av1Qsv,
            VideoEncoderKind::Av1Amf,
            VideoEncoderKind::LibSvtAv1,
            VideoEncoderKind::LibAomAv1,
        ],
    };

    if policy == VideoCodecPolicy::Auto {
        return all;
    }

    match available_ffmpeg_encoders(ffmpeg) {
        Some(encoders) => all
            .into_iter()
            .filter(|encoder| {
                encoder
                    .ffmpeg_name()
                    .is_some_and(|name| ffmpeg_encoder_is_available(&encoders, name))
            })
            .collect(),
        None => all,
    }
}

fn available_ffmpeg_encoders(ffmpeg: &Path) -> Option<String> {
    let mut command = Command::new(ffmpeg);
    configure_background_command(&mut command);
    let output = command
        .arg("-hide_banner")
        .arg("-encoders")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Some(text)
}

fn ffmpeg_encoder_is_available(encoders: &str, encoder_name: &str) -> bool {
    encoders.lines().any(|line| {
        line.split_whitespace()
            .any(|part| part.eq_ignore_ascii_case(encoder_name))
    })
}

// i18n boundary:
// Output conversion can report localized app-owned status, but ffmpeg command
// lines, progress tokens, codec/container names, paths, and stderr details remain
// raw technical text.
fn run_ffmpeg_output_conversion(
    ffmpeg: &Path,
    settings: &TranscodeIntentSettings,
    encoder: VideoEncoderKind,
    input_path: &Path,
    temp_output: &Path,
    media_probe: Option<&MediaProbeInfo>,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: mpsc::Sender<PostProcessEvent>,
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<(), String> {
    let mut command = build_output_conversion_command(
        ffmpeg,
        settings,
        encoder,
        input_path,
        temp_output,
        media_probe,
    );

    let command_line = output_conversion_command_line(
        ffmpeg,
        settings,
        encoder,
        input_path,
        temp_output,
        media_probe,
    );
    println!("[post-process] output encoder: {}", encoder.label());
    println!("[post-process] output command: {command_line}");

    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start FFmpeg: {error}"))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    if let Ok(mut guard) = child_handle.lock() {
        *guard = Some(child);
    }

    let stdout_handle =
        stdout.map(|stdout| thread::spawn(move || read_plain_process_stream(stdout, false)));
    let stderr_handle = stderr.map(|stderr| {
        let tx = tx.clone();
        thread::spawn(move || read_ffmpeg_progress_stream(stderr, item_id, workflow_id, tx))
    });

    let status = wait_post_process_child(child_handle, cancel_requested);

    let _stdout_lines = stdout_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    let stderr_lines = stderr_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();

    if cancel_requested.load(Ordering::Relaxed) {
        let _ = tx.send(PostProcessEvent::ToolCommandFinished {
            item_id,
            workflow_id,
            tool: "ffmpeg".to_owned(),
            action: "convert".to_owned(),
            command_line: command_line.clone(),
            success: false,
        });
        return Err(POST_PROCESS_CANCELLED_MESSAGE.to_owned());
    }

    match status {
        Some(Ok(status)) if status.success() => {
            let _ = tx.send(PostProcessEvent::ToolCommandFinished {
                item_id,
                workflow_id,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line,
                success: true,
            });
            Ok(())
        }
        Some(Ok(status)) => {
            let detail = stderr_lines
                .iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| format!("exit code {:?}", status.code()));
            let _ = tx.send(PostProcessEvent::ToolCommandFinished {
                item_id,
                workflow_id,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line: command_line.clone(),
                success: false,
            });
            Err(format!(
                "FFmpeg output conversion failed with {}: {detail}",
                encoder.label()
            ))
        }
        Some(Err(error)) => {
            let _ = tx.send(PostProcessEvent::ToolCommandFinished {
                item_id,
                workflow_id,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line: command_line.clone(),
                success: false,
            });
            Err(format!("Could not wait for FFmpeg to finish: {error}"))
        }
        None => {
            let _ = tx.send(PostProcessEvent::ToolCommandFinished {
                item_id,
                workflow_id,
                tool: "ffmpeg".to_owned(),
                action: "convert".to_owned(),
                command_line,
                success: false,
            });
            Err("Could not wait for FFmpeg to finish: child process missing".to_owned())
        }
    }
}

fn build_output_conversion_command(
    ffmpeg: &Path,
    settings: &TranscodeIntentSettings,
    encoder: VideoEncoderKind,
    input_path: &Path,
    temp_output: &Path,
    media_probe: Option<&MediaProbeInfo>,
) -> Command {
    let sidecar_subtitle = find_sidecar_subtitle(input_path);
    let burn_filter = subtitle_burn_filter(
        settings,
        input_path,
        sidecar_subtitle.as_deref(),
        media_probe,
    );

    let mut command = Command::new(ffmpeg);
    configure_background_command(&mut command);
    command.arg("-hide_banner").arg("-y");
    if encoder.is_hardware() && burn_filter.is_none() {
        command.arg("-hwaccel").arg("auto");
    }
    command.arg("-i").arg(input_path);
    if should_add_sidecar_subtitle_input(settings, sidecar_subtitle.as_deref()) {
        if let Some(subtitle) = sidecar_subtitle.as_deref() {
            command.arg("-i").arg(subtitle);
        }
    }

    command.arg("-map").arg("0:v:0?").arg("-map").arg("0:a:0?");
    append_subtitle_maps(&mut command, settings, sidecar_subtitle.as_deref());

    for arg in encoder.args(media_probe) {
        command.arg(arg);
    }
    if let Some(filter) = burn_filter {
        command.arg("-vf").arg(filter);
    }
    append_audio_args(&mut command, settings.audio_policy);
    append_subtitle_args(&mut command, settings, sidecar_subtitle.as_deref());
    append_container_args(&mut command, settings, encoder);

    command
        .arg(temp_output)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    command
}

fn find_sidecar_subtitle(input_path: &Path) -> Option<PathBuf> {
    const EXTENSIONS: [&str; 8] = ["srt", "ass", "ssa", "vtt", "lrc", "ttml", "dfxp", "json3"];

    for extension in EXTENSIONS {
        let candidate = input_path.with_extension(extension);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let parent = input_path.parent()?;
    let stem = input_path.file_stem()?.to_str()?;
    let prefix = format!("{stem}.");
    let mut matches = fs::read_dir(parent)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|extension| {
                    EXTENSIONS
                        .iter()
                        .any(|known| extension.eq_ignore_ascii_case(known))
                })
        })
        .filter(|path| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == stem || value.starts_with(&prefix))
        })
        .collect::<Vec<_>>();
    matches.sort_by_key(|path| path.file_name().map(|value| value.to_os_string()));
    matches.into_iter().next()
}

fn should_add_sidecar_subtitle_input(
    settings: &TranscodeIntentSettings,
    sidecar: Option<&Path>,
) -> bool {
    sidecar.is_some() && settings.subtitle_policy == SubtitlePolicy::Embed
}

fn append_subtitle_maps(
    command: &mut Command,
    settings: &TranscodeIntentSettings,
    sidecar: Option<&Path>,
) {
    match settings.subtitle_policy {
        SubtitlePolicy::Preserve => {
            command.arg("-map").arg("0:s?");
        }
        SubtitlePolicy::Embed => {
            command.arg("-map").arg("0:s?");
            if sidecar.is_some() {
                command.arg("-map").arg("1:0?");
            }
        }
        SubtitlePolicy::Burn => {}
    }
}

fn append_subtitle_args(
    command: &mut Command,
    settings: &TranscodeIntentSettings,
    sidecar: Option<&Path>,
) {
    match settings.subtitle_policy {
        SubtitlePolicy::Preserve | SubtitlePolicy::Embed => {
            if matches!(
                settings.container_policy,
                ContainerPolicy::Mp4 | ContainerPolicy::Mov
            ) {
                command.arg("-c:s").arg("mov_text");
            } else {
                command.arg("-c:s").arg("copy");
            }
        }
        SubtitlePolicy::Burn => {
            let _ = sidecar;
            command.arg("-sn");
        }
    }
}

fn subtitle_burn_filter(
    settings: &TranscodeIntentSettings,
    input_path: &Path,
    sidecar: Option<&Path>,
    media_probe: Option<&MediaProbeInfo>,
) -> Option<String> {
    if settings.subtitle_policy != SubtitlePolicy::Burn {
        return None;
    }

    if let Some(sidecar) = sidecar {
        return Some(format!(
            "subtitles=filename='{}'",
            escape_subtitle_filter_path(sidecar)
        ));
    }

    media_probe.is_some_and(|info| info.has_subtitle).then(|| {
        format!(
            "subtitles=filename='{}':si=0",
            escape_subtitle_filter_path(input_path)
        )
    })
}

fn escape_subtitle_filter_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .replace(':', "\\:")
        .replace('\'', "\\'")
}

fn append_audio_args(command: &mut Command, policy: AudioPolicy) {
    match policy {
        AudioPolicy::Auto => {
            command.arg("-c:a").arg("copy");
        }
        AudioPolicy::Aac => {
            command.arg("-c:a").arg("aac").arg("-b:a").arg("192k");
        }
        AudioPolicy::Opus => {
            command.arg("-c:a").arg("libopus").arg("-b:a").arg("160k");
        }
        AudioPolicy::Flac => {
            command.arg("-c:a").arg("flac");
        }
    }
}

fn append_container_args(
    command: &mut Command,
    settings: &TranscodeIntentSettings,
    encoder: VideoEncoderKind,
) {
    if matches!(
        settings.container_policy,
        ContainerPolicy::Mp4 | ContainerPolicy::Mov
    ) {
        if matches!(
            encoder,
            VideoEncoderKind::HevcNvenc
                | VideoEncoderKind::HevcQsv
                | VideoEncoderKind::HevcAmf
                | VideoEncoderKind::LibX265
        ) {
            command.arg("-tag:v").arg("hvc1");
        }
        command.arg("-movflags").arg("+faststart");
    }
}

fn validate_ffmpeg_available(tool_paths: &ToolPaths) -> Result<PathBuf, String> {
    let ffmpeg = resolve_tool_path(&tool_paths.ffmpeg);
    if ffmpeg.is_file() {
        Ok(ffmpeg)
    } else {
        Err(format!(
            "ffmpeg.exe was not found: {}. Install FFmpeg from Options first.",
            ffmpeg.display()
        ))
    }
}

fn output_path_for(input_path: &Path, container: ContainerPolicy) -> PathBuf {
    let Some(extension) = container_extension(container) else {
        return input_path.to_path_buf();
    };
    if has_extension(input_path, extension) {
        return input_path.to_path_buf();
    }
    let mut output = input_path.to_path_buf();
    output.set_extension(extension);
    output
}

fn container_extension(container: ContainerPolicy) -> Option<&'static str> {
    match container {
        ContainerPolicy::Auto => None,
        ContainerPolicy::Mp4 => Some("mp4"),
        ContainerPolicy::Mkv => Some("mkv"),
        ContainerPolicy::Mov => Some("mov"),
    }
}

fn transcode_temp_output_path(final_output: &Path) -> PathBuf {
    let parent = final_output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stem = final_output
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("post-process");
    let extension = final_output
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("mkv");
    parent.join(format!("{stem}.post-process.tmp.{extension}"))
}

fn remove_existing_temp_output(temp_output: &Path) -> Result<(), String> {
    if temp_output.exists() {
        fs::remove_file(temp_output).map_err(|error| {
            format!("Could not remove existing post-process temp file: {error}")
        })?;
    }
    Ok(())
}

fn replace_with_transcoded_output(
    input_path: &Path,
    temp_output: &Path,
    final_output: &Path,
) -> Result<(), String> {
    if same_path(input_path, final_output) {
        replace_file(temp_output, input_path)?;
        return Ok(());
    }

    replace_file(temp_output, final_output)?;
    fs::remove_file(input_path)
        .map_err(|error| format!("Could not remove original media file: {error}"))
}

fn replace_file(source: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        fs::remove_file(target)
            .map_err(|error| format!("Could not overwrite target media file: {error}"))?;
    }
    match fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(_) => {
            fs::copy(source, target)
                .map_err(|error| format!("Could not copy converted media file: {error}"))?;
            fs::remove_file(source)
                .map_err(|error| format!("Could not remove temporary media file: {error}"))
        }
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }

    #[cfg(not(target_os = "windows"))]
    {
        left == right
    }
}

// i18n-exempt:
// Preserve raw stdout/stderr lines from external tools. The UI may summarize a
// failure in the selected language, but raw details must stay copy/search-safe.
fn read_plain_process_stream(stream: impl std::io::Read, is_stderr: bool) -> Vec<String> {
    let mut lines = Vec::new();
    read_process_stream_lines(stream, |line| {
        if is_stderr {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
        lines.push(line.to_owned());
    });
    lines
}

// i18n-exempt:
// ffmpeg progress keys and diagnostic lines are external tool protocol/text. Parse
// them into app status where needed, but keep the original tokens untranslated.
fn read_ffmpeg_progress_stream(
    stream: impl std::io::Read,
    item_id: QueueItemId,
    workflow_id: WorkflowRunId,
    tx: mpsc::Sender<PostProcessEvent>,
) -> Vec<String> {
    let mut duration_seconds = None;
    let mut lines = Vec::new();
    read_process_stream_lines(stream, |line| {
        eprintln!("{line}");
        lines.push(line.to_owned());

        if duration_seconds.is_none() {
            duration_seconds = parse_duration_seconds(line);
        }

        if let Some(percent) = parse_ffmpeg_transcode_progress_percent(line, duration_seconds)
            .filter(|value| *value < 100.0)
        {
            let _ = tx.send(PostProcessEvent::Progress {
                item_id,
                workflow_id,
                percent,
            });
        }
    });
    lines
}

fn read_process_stream_lines(stream: impl std::io::Read, mut on_line: impl FnMut(&str)) {
    let mut reader = BufReader::new(stream);
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
                process_pending_line(&mut pending, &mut on_line);
            } else {
                pending.push(byte);
            }
        }
    }

    process_pending_line(&mut pending, &mut on_line);
}

fn process_pending_line(pending: &mut Vec<u8>, on_line: &mut impl FnMut(&str)) {
    if pending.is_empty() {
        return;
    }
    let line = String::from_utf8_lossy(pending).trim().to_owned();
    pending.clear();
    if !line.is_empty() {
        on_line(&line);
    }
}

fn parse_duration_seconds(line: &str) -> Option<f32> {
    let (_, tail) = line.split_once("Duration:")?;
    let timestamp = tail.trim_start().split(',').next()?.trim();
    parse_progress_timestamp_seconds(timestamp)
}

fn parse_ffmpeg_transcode_progress_percent(
    line: &str,
    duration_seconds: Option<f32>,
) -> Option<f32> {
    let duration = duration_seconds?;
    if duration <= 0.0 {
        return None;
    }

    let time_text = line
        .split_whitespace()
        .find_map(|part| part.strip_prefix("time="))?;
    let elapsed = parse_progress_timestamp_seconds(time_text)?;
    Some(((elapsed / duration) * 100.0).clamp(1.0, 99.0))
}

fn parse_progress_timestamp_seconds(value: &str) -> Option<f32> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("N/A") {
        return None;
    }
    let mut parts = value.split(':').collect::<Vec<_>>();
    if parts.len() > 3 || parts.is_empty() {
        return None;
    }
    while parts.len() < 3 {
        parts.insert(0, "0");
    }

    let hours = parts[0].parse::<f32>().ok()?;
    let minutes = parts[1].parse::<f32>().ok()?;
    let seconds = parts[2].parse::<f32>().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

fn wait_post_process_child(
    child_handle: &Arc<Mutex<Option<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Option<std::io::Result<std::process::ExitStatus>> {
    let mut stop_attempts = 0usize;

    loop {
        if cancel_requested.load(Ordering::Relaxed) {
            if let Ok(mut guard) = child_handle.lock() {
                if let Some(child) = guard.as_mut() {
                    terminate_child_process_tree(child);
                    stop_attempts += 1;
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

        if stop_attempts > 40 {
            if let Ok(mut guard) = child_handle.lock() {
                if let Some(child) = guard.as_mut() {
                    let _ = child.kill();
                }
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(target_os = "windows")]
fn terminate_child_process_tree(child: &mut Child) {
    let mut command = std::process::Command::new("taskkill");
    configure_background_command(&mut command);
    let _ = command
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
}

#[cfg(not(target_os = "windows"))]
fn terminate_child_process_tree(child: &mut Child) {
    let _ = child.kill();
}

// i18n-exempt:
// This builds the literal ffmpeg command. Codec names, container extensions, CLI
// flags, stream selectors, and file paths must remain raw technical tokens.
fn output_conversion_command_line(
    ffmpeg: &Path,
    settings: &TranscodeIntentSettings,
    encoder: VideoEncoderKind,
    input_path: &Path,
    temp_output: &Path,
    media_probe: Option<&MediaProbeInfo>,
) -> String {
    let sidecar_subtitle = find_sidecar_subtitle(input_path);
    let burn_filter = subtitle_burn_filter(
        settings,
        input_path,
        sidecar_subtitle.as_deref(),
        media_probe,
    );
    let mut parts = vec![
        quote_arg(ffmpeg),
        "-hide_banner".to_owned(),
        "-y".to_owned(),
    ];
    if encoder.is_hardware() && burn_filter.is_none() {
        parts.push("-hwaccel".to_owned());
        parts.push("auto".to_owned());
    }
    parts.push("-i".to_owned());
    parts.push(quote_arg(input_path));
    if should_add_sidecar_subtitle_input(settings, sidecar_subtitle.as_deref()) {
        if let Some(subtitle) = sidecar_subtitle.as_deref() {
            parts.push("-i".to_owned());
            parts.push(quote_arg(subtitle));
        }
    }
    parts.push("-map".to_owned());
    parts.push("0:v:0?".to_owned());
    parts.push("-map".to_owned());
    parts.push("0:a:0?".to_owned());
    match settings.subtitle_policy {
        SubtitlePolicy::Preserve => {
            parts.push("-map".to_owned());
            parts.push("0:s?".to_owned());
        }
        SubtitlePolicy::Embed => {
            parts.push("-map".to_owned());
            parts.push("0:s?".to_owned());
            if sidecar_subtitle.is_some() {
                parts.push("-map".to_owned());
                parts.push("1:0?".to_owned());
            }
        }
        SubtitlePolicy::Burn => {}
    }
    parts.extend(encoder.args(media_probe));
    if let Some(filter) = burn_filter {
        parts.push("-vf".to_owned());
        parts.push(filter);
    }
    match settings.audio_policy {
        AudioPolicy::Auto => parts.extend(["-c:a".to_owned(), "copy".to_owned()]),
        AudioPolicy::Aac => parts.extend([
            "-c:a".to_owned(),
            "aac".to_owned(),
            "-b:a".to_owned(),
            "192k".to_owned(),
        ]),
        AudioPolicy::Opus => parts.extend([
            "-c:a".to_owned(),
            "libopus".to_owned(),
            "-b:a".to_owned(),
            "160k".to_owned(),
        ]),
        AudioPolicy::Flac => parts.extend(["-c:a".to_owned(), "flac".to_owned()]),
    }
    match settings.subtitle_policy {
        SubtitlePolicy::Preserve | SubtitlePolicy::Embed => {
            if matches!(
                settings.container_policy,
                ContainerPolicy::Mp4 | ContainerPolicy::Mov
            ) {
                parts.extend(["-c:s".to_owned(), "mov_text".to_owned()]);
            } else {
                parts.extend(["-c:s".to_owned(), "copy".to_owned()]);
            }
        }
        SubtitlePolicy::Burn => parts.push("-sn".to_owned()),
    }
    if matches!(
        settings.container_policy,
        ContainerPolicy::Mp4 | ContainerPolicy::Mov
    ) {
        if matches!(
            encoder,
            VideoEncoderKind::HevcNvenc
                | VideoEncoderKind::HevcQsv
                | VideoEncoderKind::HevcAmf
                | VideoEncoderKind::LibX265
        ) {
            parts.extend(["-tag:v".to_owned(), "hvc1".to_owned()]);
        }
        parts.extend(["-movflags".to_owned(), "+faststart".to_owned()]);
    }
    parts.push(quote_arg(temp_output));
    parts.join(" ")
}

fn quote_arg(path: &Path) -> String {
    let text = path.display().to_string();
    if text.contains(' ') {
        format!("\"{}\"", text.replace('"', "\\\""))
    } else {
        text
    }
}
