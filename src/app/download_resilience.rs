#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DownloadErrorKind {
    TransientNetwork,
    RateLimited,
    FragmentFailure,
    AuthRequired,
    CookieInvalidOrExpired,
    FormatUnavailable,
    PostprocessThumbnailFailure,
    PostprocessMetadataFailure,
    ToolMissingOrBroken,
    FatalUnsupported,
    Cancelled,
    Unknown,
}

#[derive(Clone, Debug)]
pub(super) struct DownloadAttemptFailure {
    pub kind: DownloadErrorKind,
    pub message: String,
    pub recovered_output_path: Option<String>,
}

impl DownloadAttemptFailure {
    pub fn new(kind: DownloadErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            recovered_output_path: None,
        }
    }

    pub fn from_tool_setup_error(message: impl Into<String>) -> Self {
        let message = message.into();
        let kind = classify_download_error(&[], &[], &message);
        Self::new(kind, message)
    }

    pub fn from_attempt_output(
        stdout_lines: &[String],
        stderr_lines: &[String],
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let kind = classify_download_error(stdout_lines, stderr_lines, &message);
        Self::new(kind, message)
    }

    pub fn with_recovered_output_path(mut self, output_path: impl Into<String>) -> Self {
        self.recovered_output_path = Some(output_path.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecoveryDecision {
    DoNotRecover,
    LogOnly {
        action: &'static str,
        detail: &'static str,
    },
    KeepMainOutput {
        action: &'static str,
        detail: &'static str,
    },
    RetryWithoutThumbnail {
        action: &'static str,
        detail: &'static str,
    },
    RetryWithFormatFallback {
        action: &'static str,
        detail: &'static str,
    },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct DownloadAttemptContext {
    thumbnail_retry_used: bool,
    format_fallback_used: bool,
}

impl DownloadAttemptContext {
    pub fn new() -> Self {
        Self {
            thumbnail_retry_used: false,
            format_fallback_used: false,
        }
    }

    pub fn mark_thumbnail_retry(&mut self) {
        self.thumbnail_retry_used = true;
    }

    pub fn mark_format_fallback(&mut self) {
        self.format_fallback_used = true;
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct DownloadResiliencePolicy;

impl DownloadResiliencePolicy {
    pub fn decide(
        self,
        kind: DownloadErrorKind,
        context: &DownloadAttemptContext,
        request_has_thumbnail: bool,
        recovered_output_available: bool,
        format_fallback_available: bool,
    ) -> RecoveryDecision {
        match kind {
            DownloadErrorKind::PostprocessThumbnailFailure
                if request_has_thumbnail && !context.thumbnail_retry_used =>
            {
                RecoveryDecision::RetryWithoutThumbnail {
                    action: "retry without thumbnail",
                    detail: "thumbnail postprocess failed; retrying once without thumbnail embed",
                }
            }
            DownloadErrorKind::PostprocessMetadataFailure if recovered_output_available => {
                RecoveryDecision::KeepMainOutput {
                    action: "keep downloaded file",
                    detail: "metadata postprocess failed, but the main media file was kept",
                }
            }
            DownloadErrorKind::AuthRequired => RecoveryDecision::LogOnly {
                action: "cookie required",
                detail: "cookie rescue may be needed; automatic cookie flow was not started",
            },
            DownloadErrorKind::CookieInvalidOrExpired => RecoveryDecision::LogOnly {
                action: "cookie expired",
                detail: "cookie data may be invalid or expired; update cookies before retrying",
            },
            DownloadErrorKind::RateLimited => RecoveryDecision::LogOnly {
                action: "rate limited",
                detail: "server rate limit detected; v2 avoids immediate retry to stay website-friendly",
            },
            DownloadErrorKind::TransientNetwork | DownloadErrorKind::FragmentFailure => {
                RecoveryDecision::LogOnly {
                    action: "yt-dlp retry exhausted",
                    detail: "yt-dlp already handled its own retry policy; v2 will not immediately retry the site",
                }
            }
            DownloadErrorKind::FormatUnavailable
                if format_fallback_available && !context.format_fallback_used =>
            {
                RecoveryDecision::RetryWithFormatFallback {
                    action: "retry with safe format selector",
                    detail: "selected format is unavailable; retrying once with a conservative yt-dlp fallback selector",
                }
            }
            DownloadErrorKind::FormatUnavailable => RecoveryDecision::LogOnly {
                action: "format unavailable",
                detail: "selected format is unavailable; automatic fallback was not applied again",
            },
            _ => RecoveryDecision::DoNotRecover,
        }
    }
}

pub(super) fn classify_download_error(
    stdout_lines: &[String],
    stderr_lines: &[String],
    fallback_message: &str,
) -> DownloadErrorKind {
    let normalized = combined_error_text(stdout_lines, stderr_lines, fallback_message);

    if contains_any(
        &normalized,
        &[
            "download cancelled",
            "cancelled",
            "canceled",
            "operation was canceled",
        ],
    ) {
        return DownloadErrorKind::Cancelled;
    }

    if contains_any(
        &normalized,
        &[
            "yt-dlp was not found",
            "ffmpeg was not found",
            "executable not found",
            "could not start yt-dlp",
            "permission denied",
            "access is denied",
        ],
    ) {
        return DownloadErrorKind::ToolMissingOrBroken;
    }

    if contains_any(
        &normalized,
        &[
            "cookies are no longer valid",
            "unable to extract cookies",
            "could not copy browser cookie database",
            "failed to decrypt",
            "cookie file is invalid",
            "invalid cookie",
        ],
    ) {
        return DownloadErrorKind::CookieInvalidOrExpired;
    }

    if contains_any(
        &normalized,
        &[
            "sign in to confirm",
            "login required",
            "private video",
            "members-only",
            "members only",
            "age-restricted",
            "confirm your age",
            "use --cookies",
            "cookies from browser",
        ],
    ) {
        return DownloadErrorKind::AuthRequired;
    }

    if contains_any(
        &normalized,
        &[
            "http error 429",
            "http 429",
            "too many requests",
            "rate limit",
            "rate-limit",
        ],
    ) {
        return DownloadErrorKind::RateLimited;
    }

    if contains_any(
        &normalized,
        &[
            "requested format is not available",
            "format is not available",
            "no video formats found",
            "no audio formats found",
            "no formats found",
            "unable to extract video data",
        ],
    ) {
        return DownloadErrorKind::FormatUnavailable;
    }

    if contains_all(&normalized, &["thumbnail", "postprocess"])
        || contains_all(&normalized, &["thumbnail", "ffmpeg"])
        || contains_all(&normalized, &["attached pic", "ffmpeg"])
        || contains_all(
            &normalized,
            &[
                "extracted extension",
                "unusual",
                "skipped for safety reasons",
            ],
        )
        || contains_any(
            &normalized,
            &[
                "unable to embed thumbnail",
                "failed to embed thumbnail",
                "error embedding thumbnail",
            ],
        )
    {
        return DownloadErrorKind::PostprocessThumbnailFailure;
    }

    if contains_all(&normalized, &["metadata", "postprocess"])
        || contains_any(
            &normalized,
            &[
                "unable to embed metadata",
                "failed to embed metadata",
                "error embedding metadata",
            ],
        )
    {
        return DownloadErrorKind::PostprocessMetadataFailure;
    }

    if contains_any(
        &normalized,
        &[
            "unable to download fragment",
            "fragment retries",
            "fragment retry",
            "http error 403 on fragment",
        ],
    ) || (normalized.contains("fragment") && normalized.contains("http error 403"))
    {
        return DownloadErrorKind::FragmentFailure;
    }

    if contains_any(
        &normalized,
        &[
            "http error 403",
            "http 403",
            "timed out",
            "timeout",
            "connection reset",
            "connection aborted",
            "temporarily unavailable",
            "unable to download video data",
            "got server http error",
            "remote end closed connection",
            "connection refused",
        ],
    ) {
        return DownloadErrorKind::TransientNetwork;
    }

    if contains_any(
        &normalized,
        &[
            "unsupported url",
            "no suitable extractor",
            "drm",
            "live event will begin",
            "video unavailable",
            "video has been removed",
            "this video is unavailable",
        ],
    ) {
        return DownloadErrorKind::FatalUnsupported;
    }

    DownloadErrorKind::Unknown
}

fn combined_error_text(
    stdout_lines: &[String],
    stderr_lines: &[String],
    fallback_message: &str,
) -> String {
    let mut text = String::new();
    for line in stdout_lines.iter().chain(stderr_lines.iter()) {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str(fallback_message);
    text.to_ascii_lowercase()
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn contains_all(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().all(|needle| haystack.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_rate_limit_separately_from_network() {
        assert_eq!(
            classify_download_error(
                &[],
                &["ERROR: HTTP Error 429: Too Many Requests".to_owned()],
                ""
            ),
            DownloadErrorKind::RateLimited
        );
    }

    #[test]
    fn classifies_auth_before_transient_http_errors() {
        assert_eq!(
            classify_download_error(
                &[],
                &["ERROR: HTTP Error 403. Sign in to confirm you're not a bot".to_owned()],
                ""
            ),
            DownloadErrorKind::AuthRequired
        );
    }

    #[test]
    fn classifies_thumbnail_safety_fallback() {
        assert_eq!(
            classify_download_error(
                &[],
                &[
                    "ERROR: extracted extension is unusual and skipped for safety reasons"
                        .to_owned()
                ],
                ""
            ),
            DownloadErrorKind::PostprocessThumbnailFailure
        );
    }

    #[test]
    fn retries_format_unavailable_once_with_fallback() {
        let policy = DownloadResiliencePolicy::default();
        let mut context = DownloadAttemptContext::new();
        assert!(matches!(
            policy.decide(
                DownloadErrorKind::FormatUnavailable,
                &context,
                false,
                false,
                true
            ),
            RecoveryDecision::RetryWithFormatFallback { .. }
        ));

        context.mark_format_fallback();
        assert!(matches!(
            policy.decide(
                DownloadErrorKind::FormatUnavailable,
                &context,
                false,
                false,
                true
            ),
            RecoveryDecision::LogOnly { .. }
        ));
    }
}
