use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::infrastructure::{configure_background_command, resolve_tool_path};

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub(super) struct MediaProbeInfo {
    pub container: Option<String>,
    pub duration_sec: Option<f64>,
    pub file_size_bytes: Option<u64>,
    pub total_bitrate_bps: Option<u64>,
    pub video: Option<VideoProbeInfo>,
    pub audio: Option<AudioProbeInfo>,
    pub has_subtitle: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub(super) struct VideoProbeInfo {
    pub codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<(u32, u32)>,
    pub bitrate_bps: Option<u64>,
    pub pix_fmt: Option<String>,
    pub profile: Option<String>,
    pub color_transfer: Option<String>,
    pub color_primaries: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub(super) struct AudioProbeInfo {
    pub codec: Option<String>,
    pub bitrate_bps: Option<u64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
}

pub(super) fn ffprobe_companion_path_for_ffmpeg(ffmpeg_path: &Path) -> PathBuf {
    ffmpeg_path
        .parent()
        .map(|parent| parent.join("ffprobe.exe"))
        .unwrap_or_else(|| resolve_tool_path(".\\tools\\ffmpeg\\ffprobe.exe"))
}

pub(super) fn probe_media_with_ffprobe(
    ffprobe_path: &Path,
    input_path: &Path,
) -> Result<MediaProbeInfo, String> {
    if !ffprobe_path.is_file() {
        return Err(format!("ffprobe.exe was not found: {}", ffprobe_path.display()));
    }

    let mut command = Command::new(ffprobe_path);
    configure_background_command(&mut command);
    let output = command
        .arg("-v")
        .arg("error")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(input_path)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("Could not start ffprobe: {error}"))?;

    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if detail.is_empty() {
            format!("ffprobe failed with exit code {:?}", output.status.code())
        } else {
            format!("ffprobe failed: {detail}")
        });
    }

    let root: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Could not parse ffprobe JSON: {error}"))?;
    Ok(media_probe_info_from_json(&root))
}

fn media_probe_info_from_json(root: &Value) -> MediaProbeInfo {
    let format = root.get("format");
    let mut info = MediaProbeInfo {
        container: format
            .and_then(|value| value.get("format_name"))
            .and_then(json_string),
        duration_sec: format
            .and_then(|value| value.get("duration"))
            .and_then(json_f64),
        file_size_bytes: format
            .and_then(|value| value.get("size"))
            .and_then(json_u64),
        total_bitrate_bps: format
            .and_then(|value| value.get("bit_rate"))
            .and_then(json_u64),
        video: None,
        audio: None,
        has_subtitle: false,
    };

    if let Some(streams) = root.get("streams").and_then(Value::as_array) {
        for stream in streams {
            match stream.get("codec_type").and_then(Value::as_str) {
                Some("video") if info.video.is_none() => {
                    info.video = Some(video_probe_info_from_stream(stream));
                }
                Some("audio") if info.audio.is_none() => {
                    info.audio = Some(audio_probe_info_from_stream(stream));
                }
                Some("subtitle") => {
                    info.has_subtitle = true;
                }
                _ => {}
            }
        }
    }

    info
}

fn video_probe_info_from_stream(stream: &Value) -> VideoProbeInfo {
    VideoProbeInfo {
        codec: stream.get("codec_name").and_then(json_string),
        width: stream.get("width").and_then(json_u64).and_then(|value| u32::try_from(value).ok()),
        height: stream
            .get("height")
            .and_then(json_u64)
            .and_then(|value| u32::try_from(value).ok()),
        fps: stream
            .get("avg_frame_rate")
            .or_else(|| stream.get("r_frame_rate"))
            .and_then(json_string)
            .and_then(|value| parse_rational(&value)),
        bitrate_bps: stream.get("bit_rate").and_then(json_u64),
        pix_fmt: stream.get("pix_fmt").and_then(json_string),
        profile: stream.get("profile").and_then(json_string),
        color_transfer: stream.get("color_transfer").and_then(json_string),
        color_primaries: stream.get("color_primaries").and_then(json_string),
    }
}

fn audio_probe_info_from_stream(stream: &Value) -> AudioProbeInfo {
    AudioProbeInfo {
        codec: stream.get("codec_name").and_then(json_string),
        bitrate_bps: stream.get("bit_rate").and_then(json_u64),
        sample_rate: stream
            .get("sample_rate")
            .and_then(json_u64)
            .and_then(|value| u32::try_from(value).ok()),
        channels: stream
            .get("channels")
            .and_then(json_u64)
            .and_then(|value| u32::try_from(value).ok()),
    }
}

fn json_string(value: &Value) -> Option<String> {
    value.as_str().map(ToOwned::to_owned)
}

fn json_u64(value: &Value) -> Option<u64> {
    if let Some(value) = value.as_u64() {
        return Some(value);
    }
    value.as_str()?.trim().parse::<u64>().ok()
}

fn json_f64(value: &Value) -> Option<f64> {
    if let Some(value) = value.as_f64() {
        return Some(value);
    }
    value.as_str()?.trim().parse::<f64>().ok()
}

fn parse_rational(value: &str) -> Option<(u32, u32)> {
    let (num, den) = value.split_once('/')?;
    let num = num.trim().parse::<u32>().ok()?;
    let den = den.trim().parse::<u32>().ok()?;
    if num == 0 || den == 0 {
        return None;
    }
    Some((num, den))
}
