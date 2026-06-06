#![allow(dead_code)]

use crate::app::state::AppState;

fn config_location_preview(state: &AppState) -> String {
    let trimmed = state.tool_paths.yt_dlp_config.trim();
    if trimmed.is_empty() {
        "--ignore-config".to_owned()
    } else {
        format!("--config-location {trimmed}")
    }
}
fn proxy_preview(state: &AppState) -> String {
    let trimmed = state.tool_paths.proxy_url.trim();
    if trimmed.is_empty() {
        "--proxy <proxy-url>".to_owned()
    } else {
        format!("--proxy {trimmed}")
    }
}
fn certificate_preview() -> String {
    "--no-check-certificates".to_owned()
}
fn cookie_preview(state: &AppState) -> String {
    if state.cookie_source_uses_auto() {
        return "--cookies data/cookies/<matched-site>.cookies.txt".to_owned();
    }

    if state.cookie_source_uses_file() {
        let trimmed = state.tool_paths.browser_cookie_file.trim();
        if trimmed.is_empty() {
            return "--cookies <cookies.txt-path>".to_owned();
        }
        return format!("--cookies {trimmed}");
    }

    let source = state.tool_paths.browser_cookie_source.trim();
    let source = if source.is_empty() {
        "<browser>"
    } else {
        source
    };

    let profile = state.tool_paths.browser_cookie_profile.trim();
    let cookie_arg = if profile.is_empty() {
        source.to_owned()
    } else {
        format!("{source}:{profile}")
    };
    format!("--cookies-from-browser {cookie_arg}")
}
fn aria2_preview(state: &AppState) -> String {
    let aria2_path = state.tool_paths.aria2c.trim();
    let mut lines = Vec::new();
    if aria2_path.is_empty() {
        lines.push("--external-downloader <aria2c-path>".to_owned());
    } else {
        lines.push(format!("--external-downloader {aria2_path}"));
    }

    if state.tool_paths.effective_proxy_url().is_some() || state.tool_paths.no_check_certificates {
        let mut args = Vec::new();
        if let Some(proxy_url) = state.tool_paths.effective_proxy_url() {
            args.push(format!("--all-proxy={proxy_url}"));
        }
        if state.tool_paths.no_check_certificates {
            args.push("--check-certificate=false".to_owned());
        }
        lines.push(format!(
            "--external-downloader-args aria2c:{}",
            args.join(" ")
        ));
    }

    lines.join("\n")
}
fn concurrent_fragments_preview(state: &AppState) -> String {
    let fragments = state.tool_paths.concurrent_fragments.max(1);
    format!("--concurrent-fragments {fragments}")
}
fn limit_rate_preview(state: &AppState) -> String {
    let trimmed = state.tool_paths.limit_rate.trim();
    if trimmed.is_empty() {
        "--limit-rate <rate>".to_owned()
    } else {
        format!("--limit-rate {trimmed}")
    }
}
fn chapter_compatibility_preview(_state: &AppState) -> String {
    "For range downloads\n--compat-options no-direct-merge\n--format best[...][vcodec!=none][acodec!=none]/best".to_owned()
}
fn thumbnail_download_preview(_state: &AppState) -> String {
    "--write-thumbnail\n--convert-thumbnails jpg".to_owned()
}
fn thumbnail_embed_preview(_state: &AppState) -> String {
    "--embed-thumbnail\n--convert-thumbnails jpg".to_owned()
}
fn subtitle_download_preview(_state: &AppState) -> String {
    "When subtitles are selected\n--write-subs / --write-auto-subs\n--convert-subs srt".to_owned()
}
fn subtitle_embed_preview(_state: &AppState) -> String {
    "--embed-subs".to_owned()
}
fn chapter_download_preview(_state: &AppState) -> String {
    "--split-chapters".to_owned()
}
fn chapter_embed_preview(_state: &AppState) -> String {
    "--embed-chapters".to_owned()
}
