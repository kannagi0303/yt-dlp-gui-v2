pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.prepare" => "준비",
        "tab.main" => "메인",
        "tab.advanced" => "고급",
        "tab.options" => "옵션",
        "main.url_hint" => "URL 붙여넣기",
        "action.download" => "다운로드",
        "action.add" => "＋ 추가",
        "action.stop" => "중지",
        "action.stopping" => "중지 중",
        "action.cut" => "잘라내기",
        "action.copy" => "복사",
        "action.paste" => "붙여넣기",
        "action.clear" => "지우기",
        "item.thumbnail" => "썸네일",
        "item.thumbnail_preview" => "썸네일 미리보기",
        "notification.download_finished" => "다운로드 완료",
        "notification.download_failed" => "다운로드 실패",
        "notification.download_finished_detail_prefix" => "완료: ",
        "notification.download_finished_detail" => "다운로드가 완료되었습니다.",
        "notification.windows_toast_windows_only" => "Windows Toast는 Windows에서만 지원됩니다.",
        "media.video" => "동영상",
        "media.audio" => "오디오",
        "media.subtitle" => "자막",
        "media.section" => "범위",
        "item.file_name" => "파일 이름",
        "main.target_folder" => "출력 폴더",
        "picker.title.video" => "동영상 형식 선택",
        "picker.title.audio" => "오디오 형식 선택",
        "picker.title.subtitle" => "자막 선택",
        "picker.title.section" => "범위 선택",
        "action.back" => "뒤로",
        "picker.mode.filter" => "필터",
        "picker.mode.table" => "표",
        "action.confirm" => "확인",
        "picker.empty_table" => "No format items to display",
        "picker.header.resolution" => "해상도",
        "picker.header.range" => "범위",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "형식",
        "picker.header.codec" => "코덱",
        "picker.header.size" => "크기",
        "picker.header.sample_rate" => "샘플 레이트",
        "picker.filter.resolution" => "해상도",
        "picker.filter.range" => "범위",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "코덱",
        "picker.filter.sample_rate" => "샘플 레이트",
        "main.tooltip.missing_yt_dlp" => {
            "yt-dlp is missing. Install it or choose yt-dlp.exe in Options."
        }
        "advance.source" => "소스",
        "advance.config" => "설정",
        "advance.none" => "없음",
        "advance.network_access" => "네트워크 및 접근",
        "advance.proxy" => "프록시",
        "advance.enable_proxy" => "프록시 사용",
        "advance.certificate" => "인증서",
        "advance.skip_certificate_verification" => "인증서 확인 건너뛰기",
        "advance.use_cookies" => "쿠키 사용",
        "advance.enable_cookies" => "쿠키 사용",
        "advance.cookie_source" => "쿠키 소스",
        "advance.cookie_file" => "쿠키 파일",
        "advance.no_cookies_txt_selected" => "No cookies.txt selected",
        "advance.browse" => "찾아보기",
        "advance.select_netscape_cookies_txt" => "Select Netscape cookies.txt",
        "advance.clear" => "지우기",
        "advance.browser" => "브라우저",
        "advance.default" => "기본값",
        "advance.external_downloader" => "외부 다운로더",
        "advance.use_aria2_for_faster_downloads" => "Use Aria2 for faster downloads",
        "advance.download_control" => "다운로드 제어",
        "advance.concurrent_fragments" => "Concurrent fragments",
        "advance.1_default" => "1 (default)",
        "advance.rate_limit" => "Rate limit",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => {
            "e.g. 2M, 800K; leave empty for unlimited"
        }
        "advance.chapters" => "챕터",
        "advance.chapter_download_compatibility_mode" => "Chapter download compatibility mode",
        "advance.file_time" => "파일 시간",
        "advance.post_processing" => "Post-processing",
        "advance.thumbnail" => "썸네일",
        "advance.download" => "다운로드",
        "advance.embed" => "삽입",
        "advance.subtitles" => "자막",
        "item.stop_download" => "다운로드 중지",
        "item.remove" => "삭제",
        "item.save_as" => "다른 이름으로 저장",
        "item.error" => "오류",
        "item.all" => "All",
        "item.queued" => "대기 중",
        "item.done" => "완료",
        "item.failed" => "실패",
        "item.clear_all" => "모두 지우기",
        "item.add_a_video_url" => "동영상 URL 추가",
        "item.after_adding_choose_the_video_format_here" => {
            "After adding, choose the video format here."
        }
        "item.after_adding_choose_the_audio_format_here" => {
            "After adding, choose the audio format here."
        }
        "item.loading_thumbnail" => "Loading thumbnail",
        "item.file_actions" => "File actions",
        "item.open_file" => "파일 열기",
        "item.open_folder" => "폴더 열기",
        "item.copy_path" => "경로 복사",
        "item.opened_output_file" => "Opened output file.",
        "item.file_not_found_opened_the_output_location" => {
            "File not found; opened the output location."
        }
        "item.opened_output_location" => "Opened output location.",
        "item.copied_output_path" => "Copied output path.",
        "item.file_actions_are_available_after_download_co" => {
            "File actions are available after download completes"
        }
        "prepare.language" => "언어",
        "prepare.back" => "뒤로",
        "prepare.choose" => "선택",
        "prepare.auto_detect" => "자동 감지",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Install the required tools now, or skip and handle them later in Options."
        }
        "prepare.required" => "필수",
        "prepare.recommended" => "권장",
        "prepare.optional" => "선택",
        "prepare.missing" => "없음",
        "prepare.install_later" => "나중에 설치",
        "prepare.downloading_100" => "Downloading 100%",
        "prepare.extracting_100" => "Extracting 100%",
        "prepare.install_failed" => "설치 실패",
        "prepare.install_all" => "모두 설치",
        "prepare.reinstall" => "다시 설치",
        "prepare.installing" => "설치 중",
        "prepare.skip" => "건너뛰기",
        "prepare.install" => "설치",
        "prepare.another_tool_is_already_being_installed" => {
            "Another tool is already being installed."
        }
        "prepare.needs_attention" => "확인 필요",
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "This URL contains both a video and a playlist"
        }
        "options.detected" => "Detected ",
        "options.playlist_prompt" => "Playlist prompt",
        "options.which_one_should_be_loaded" => "Which one should be loaded?",
        "options.both_video_and_playlist_were_detected" => "Both video and playlist were detected",
        "options.this_playlist_may_contain_many_items" => "This playlist may contain many items.",
        "options.video" => "동영상",
        "options.playlist" => "재생목록",
        "options.cancel" => "취소",
        "options.load" => "불러오기",
        "options.behavior" => "동작",
        "options.add_action" => "Add action",
        "options.download_directly" => "바로 다운로드",
        "options.clipboard_change" => "클립보드 변경",
        "options.run_immediately" => "즉시 실행",
        "options.playlist_2" => "재생목록",
        "options.with_playlist" => "With playlist",
        "options.ask" => "묻기",
        "options.single_video" => "단일 동영상",
        "options.full_playlist" => "전체 재생목록",
        "options.high_risk_prompt" => "High-risk prompt",
        "options.on" => "켜짐",
        "options.playlist_count" => "Playlist count",
        "options.limit" => "Limit",
        "options.max" => "Max:",
        "options.items" => " items",
        "options.language" => "언어",
        "options.current_language" => "현재 언어",
        "options.back" => "뒤로",
        "options.choose" => "선택",
        "options.auto_detect" => "자동 감지",
        "options.tool_paths" => "도구 경로",
        "options.file_actions" => "File actions",
        "options.action_button" => "Action button",
        "options.cache" => "캐시",
        "options.cache_location" => "Cache location",
        "options.appearance_window" => "모양 및 창",
        "options.notifications" => "알림",
        "options.enable" => "사용",
        "options.ui_scale" => "UI 배율",
        "options.apply" => "적용",
        "options.current" => "현재",
        "options.always_on_top" => "항상 위",
        "options.window_position" => "창 위치",
        "options.remember" => "기억",
        "options.window_size" => "창 크기",
        "options.reinstall" => "다시 설치",
        "options.installing" => "설치 중",
        "options.browse" => "찾아보기",
        "options.install" => "설치",
        "options.file_not_found" => "File not found: ",
        "options.will_install_to" => "Will install to: ",
        "options.another_tool_is_being_installed_please_wait" => {
            "Another tool is being installed. Please wait for it to finish."
        }
        "options.install_to" => "Install to: ",
        "options.executable" => "executable",
        "main.clipboard_monitor_on_the_next_youtube_url_ch" => {
            "Clipboard monitor: on. The next YouTube URL change will be added immediately."
        }
        "main.clipboard_monitor_on_the_next_youtube_url_ch_2" => {
            "Clipboard monitor: on. The next YouTube URL change will fill the URL field."
        }
        "main.clipboard_monitor_off_turning_it_on_only_mem" => {
            "Clipboard monitor: off. Turning it on only memorizes the current clipboard; the next change will be handled."
        }
        "main.controlled_by_config" => "Controlled by config: ",
        "main.controlled_by_config_2" => "Controlled by config",
        "main.actual_path" => "Actual path: ",
        "picker.no_chapters_available" => "No chapters available.",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "Choose the range to download for this item. Default is the full video."
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "Chapter compatibility mode is on: chapter downloads will use a more stable single-file format."
        }
        "picker.subtitles_will_not_be_downloaded" => "Subtitles will not be downloaded.",
        "picker.no_subtitles_are_available_for_this_video" => {
            "No subtitles are available for this video."
        }
        "picker.no_subtitles_are_available_in_this_tab" => {
            "No subtitles are available in this tab."
        }
        "picker.source_language" => "Source language",
        "picker.translation_target" => "Translation target",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "Tip: YouTube auto-translated subtitles are more likely to be rate-limited than original subtitles. Choose “No translation” if you only need the source text."
        }
        "picker.no_subtitles_are_available_for_this_source" => {
            "No subtitles are available for this source."
        }
        "picker.target" => "Target",
        "picker.available_subtitles" => "Available subtitles",
        "picker.language" => "언어",
        "picker.subtitle_tab.none" => "자막 없음",
        "picker.subtitle_tab.original" => "원본 자막",
        "picker.subtitle_tab.automatic" => "자동 자막",
        "config.youtube_playlist_mode.ask" => "묻기",
        "config.youtube_playlist_mode.video" => "동영상",
        "config.youtube_playlist_mode.ignore" => "Ignore",
        "config.output_action.menu" => "Show menu",
        "config.output_action.open_folder" => "폴더 열기",
        "config.output_action.open_file" => "파일 열기",
        "tools.file_time.none" => "변경하지 않음",
        "tools.file_time.use_upload_date" => "동영상 게시일 사용",
        "tools.file_time.use_download_time" => "다운로드 시간 사용",
        "tools.file_time.none_hint" => {
            "--mtime/--no-mtime을 전달하지 않고 최종 파일 시간도 수정하지 않습니다."
        }
        "tools.file_time.use_upload_date_hint" => {
            "yt-dlp가 최종 파일 경로를 보고한 뒤 파일 수정 시간을 동영상 게시일로 설정합니다."
        }
        "tools.file_time.use_download_time_hint" => "--no-mtime",
        "tools.cache_mode.default" => "기본값",
        "tools.cache_mode.v2_cache" => "yt-dlp-gui",
        "tools.cache_mode.windows_temp" => "Windows",
        "tools.subtitle_source.none" => "자막 없음",
        "tools.subtitle_source.original" => "원본 자막",
        "tools.subtitle_source.automatic" => "자동 자막",
        "tools.quality.best" => "최고",
        "tools.quality.audio_only" => "오디오만",
        "tools.youtube_playlist.channel_generated" => "YouTube generated channel playlist",
        "tools.youtube_playlist.mix_radio" => "YouTube Mix / Radio",
        "tools.youtube_playlist.music_album" => "YouTube Music album/collection",
        "tools.youtube_playlist.liked_videos" => "Liked videos",
        "tools.youtube_playlist.favorites_legacy" => "Legacy favorites playlist",
        "prepare.severity.required" => "Required item",
        "prepare.severity.recommended" => "Recommended item",
        "prepare.severity.optional" => "Optional item",
        "prepare.status.ready" => "준비됨",
        "prepare.status.missing" => "없음",
        "prepare.status.warning" => "확인 필요",
        "prepare.status.failed" => "실패",
        "tool_install.stage.preparing" => "Preparing",
        "tool_install.stage.downloading" => "Downloading",
        "tool_install.stage.extracting" => "Extracting",
        "tool_install.stage.installing" => "Installing",
        "tool_install.stage.completed" => "Completed",
        "tool_install.stage.failed" => "Failed",
        "domain.media.video" => "video",
        "domain.media.audio" => "audio",
        "domain.media.muxed" => "muxed",
        "domain.media.subtitle" => "subtitle",
        "domain.media.other" => "other",
        "domain.quality.best" => "최고",
        "domain.quality.audio_only" => "오디오만",
        "prepare.severity.short.required" => "Required",
        "prepare.severity.short.recommended" => "Recommended",
        "prepare.severity.short.optional" => "Optional",
        "item.status.idle" => "Not started",
        "item.status.queued" => "Queued",
        "item.status.running" => "Running",
        "item.status.finished" => "Done",
        "item.status.failed" => "Failed",
        "item.status.cancelled" => "Cancelled",
        "item.status.waiting_analysis" => "Waiting for analysis",
        "item.status.analyzing" => "Analyzing",
        "item.status.analysis_failed" => "Analysis failed",
        "picker.waiting_analysis" => "Waiting for analysis",
        "picker.audio_from_video" => "Decided by Video format",
        "picker.not_selected" => "Not selected",
        "picker.full_video" => "Full video",
        "picker.no_translation" => "No translation",
        "picker.until_end" => "end",
        "state.clipboard_detected_url" => "Detected a YouTube URL from the clipboard.",
        "state.no_url_to_analyze" => "There is no URL to analyze.",
        "state.analyzing_source" => "Analyzing: {source}",
        "state.batch_add_running" => "Batch add is still running.",
        "state.no_url_to_add" => "There is no URL to add.",
        "state.video_url_contains_playlist" => {
            "Detected a video URL that also contains a playlist."
        }
        "state.detected_high_risk_playlist" => "Detected high-risk YouTube playlist: {kind}",
        "state.no_url_to_download_now" => "There is no URL to download immediately.",
        "state.download_now_single_video_only" => {
            "Download now currently only handles one video URL."
        }
        "state.added_ready_download_now" => "Added and ready to download now: {title}",
        "state.current_action_cancelled" => "Current action cancelled.",
        "state.stopping_batch_add" => "Stopping batch add...",
        "state.retrying_analysis_cookie" => "Retrying analysis with cookies: {source}",
        "state.batch_no_new_items" => "No new items were found in the batch.",
        "state.playlist_added_limited" => {
            "Added {count} batch items from the playlist (limit applied)."
        }
        "state.batch_added_title" => "Added to batch: {title}",
        "state.playlist_added" => "Added {count} batch items from the playlist.",
        "state.batch_add_cancelled" => "Batch add cancelled.",
        "state.batch_add_cancelled_with_count" => "Batch add cancelled; {count} items were added.",
        "state.batch_add_interrupted" => "Batch add was interrupted.",
        "state.deployment_complete" => "Deployment complete",
        "state.tool_deployed" => "{tool} downloaded and deployed.",
        "state.tool_deploy_failed" => "{tool} deployment failed: {error}",
        "state.download_item_fallback" => "Download item",
        "state.download_stopped" => "Download stopped.",
        "state.no_url_to_add_batch" => "There is no URL to add to the batch.",
        "state.batch_input_added" => "Added {count} queued items from batch input.",
        "state.no_url_to_download" => "There is no URL to download.",
        "state.download_already_running" => {
            "A download is already running. Please wait for it to finish."
        }
        "state.no_runnable_batch_items" => "There are no runnable batch items.",
        "state.no_download_to_stop" => "There is no download to stop.",
        "state.stopping_download" => "Stopping download...",
        "state.target_download_not_found" => "Target download item was not found.",
        "state.analyze_before_download" => "Analyze the video before starting download.",
        "state.downloading_title" => "Downloading: {title}",
        "state.downloading_title_aria2_fallback" => {
            "Downloading: {title} (Aria2 not found; using yt-dlp native download)"
        }
        "state.target_export_not_found" => "Target export item was not found.",
        "state.cannot_export_item" => "This item cannot be exported right now.",
        "state.analyze_before_export" => "Analyze the video before exporting.",
        "state.choose_subtitles_before_export" => "Choose subtitles before exporting.",
        "state.specify_file_extension" => "Specify a file extension.",
        "state.exporting_video" => "Exporting video: {title}",
        "state.exporting_audio" => "Exporting audio: {title}",
        "state.exporting_subtitles" => "Exporting subtitles: {title}",
        "state.cleared_queue" => "Queue cleared.",
        "state.cannot_remove_running_item" => "Running items cannot be removed.",
        "state.removed_item" => "Removed: {title}",
        "state.controlled_by_config" => "Controlled by config",
        "state.install_blocked_by_prepare" => "Handle {items} before installing dependency tools.",
        "state.tool_deployment_running" => "{tool} deployment is still running.",
        "state.no_tools_to_install" => "There are no tools to install.",
        "state.no_selected_tools_to_install" => "There are no selected deployable items.",
        "state.prepare_skipped" => {
            "Prepare page skipped. You can handle dependency deployment later in Options."
        }
        "state.skip_failed" => "Skip failed: {error}",
        "state.preparing_deployment" => "Preparing deployment",
        "state.tool_downloading_deploying" => "{tool} downloading and deploying...",
        "state.found" => "Found",
        "state.not_found" => "Not found",
        "state.clipboard_monitor_enabled_auto_add" => {
            "Clipboard monitor enabled; the next YouTube URL change will be added immediately."
        }
        "state.clipboard_monitor_enabled_fill" => {
            "Clipboard monitor enabled; the next YouTube URL change will fill the URL field."
        }
        "state.clipboard_monitor_disabled" => "Clipboard monitor disabled.",
        "state.clipboard_will_auto_add" => {
            "YouTube URLs will be added immediately after the clipboard changes."
        }
        "state.clipboard_will_fill_only" => "Clipboard changes will only fill the URL field.",
        "state.adding_source" => "Adding: {source}",
        "state.added_to_list" => "Added to list: {title}",
        "state.range_set_item_full" => "Download range set: Item {index} / Full video",
        "state.range_set_item_value" => "Download range set: Item {index} / {value}",
        "state.format_selection_updated" => {
            "Format selection updated: Item {index} / {kind} / {value}"
        }
        "state.range_set_title_full" => "Download range set: {title} / Full video",
        "state.range_set_title_value" => "Download range set: {title} / {value}",
        "state.playlist_ignored_for_now" => "Playlist is ignored for now: {target}",
        "state.untitled_video" => "Untitled video",
        "state.analysis_complete" => "Analysis complete: {title}",
        "state.video_extension_error" => "Video export only supports mkv / mp4 / webm / mov / flv.",
        "state.audio_extension_error" => {
            "Audio export only supports opus / aac / m4a / mp3 / vorbis / alac / flac / wav."
        }
        "state.subtitle_extension_error" => {
            "Subtitle extension must be srt, vtt, ass, ssa, lrc, ttml, dfxp, json3, srv3, srv2, or srv1."
        }
        "state.action_aria2_fallback" => "{action} (Aria2 not found; using yt-dlp native download)",
        "state.cache_yt_dlp_default" => "yt-dlp default",
        "playlist.note.mix_radio" => {
            "This Mix / Radio playlist may contain many items and can change over time."
        }
        "playlist.note.channel_generated" => {
            "Treat this YouTube-generated channel playlist conservatively."
        }
        "playlist.note.liked_videos" => "Liked videos usually require login or cookies.",
        "playlist.note.favorites_legacy" => {
            "This is a legacy favorites playlist style and may not be stable now."
        }
        "playlist.note.music_album" => "This is usually a YouTube Music album or collection.",
        "prepare.tool.ytdlp.description" => "Core video analysis and downloading.",
        "prepare.tool.deno.description" => "Improves YouTube analysis stability.",
        "prepare.tool.ffmpeg.description" => {
            "Merges video/audio, converts formats, and handles thumbnails/subtitles."
        }
        "prepare.req.app_root.title" => "App folder",
        "prepare.req.app_root.description" => {
            "The portable folder must be writable for settings and support folders."
        }
        "prepare.req.tools_dir.title" => "tools folder",
        "prepare.req.tools_dir.description" => {
            "Dependency deployment stores yt-dlp, FFmpeg, and Deno here."
        }
        "prepare.req.tool_install_cache.title" => "Deployment temp",
        "prepare.req.tool_install_cache.description" => {
            "FFmpeg and Deno extraction uses this temp folder."
        }
        "prepare.req.cache.title" => "Download cache",
        "prepare.req.cache.description" => "yt-dlp-gui cache mode stores yt-dlp cache here.",
        "prepare.req.output.title" => "Output folder",
        "prepare.req.output.description" => "Videos, audio, and subtitles are saved here.",
        "prepare.req.output.recommendation" => "Choose a valid output folder from Main or Options.",
        "prepare.req.config.title" => "Config file",
        "prepare.req.config.description" => {
            "The app must be able to save prepare-page skip and tool path settings."
        }
        "prepare.req.move_to_writable" => "Move the app to a writable portable folder.",
        "prepare.req.move_to_writable_example" => {
            "Move the app to a writable portable folder, for example D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.generic_writable_recommendation" => {
            "If deployment or config saving fails later, move the app to a writable non-synced portable folder."
        }
        "prepare.req.use_folder_path" => "Use a folder path instead.",
        "prepare.req.path_not_folder" => "{path} is not a folder",
        "prepare.req.config_not_folder" => "Make sure the config path is not a folder.",
        "prepare.req.config_readonly" => "Config file is read-only",
        "prepare.req.config_readonly_recommendation" => {
            "Clear the config file read-only attribute, or move it to a writable portable folder."
        }
        "prepare.req.clear_write_test" => {
            "Retry later, or remove the leftover .yt-dlp-gui-write-test file in the folder."
        }
        "runtime.download_cancelled" => "Download cancelled.",
        "runtime.yt_dlp_not_found" => {
            "yt-dlp was not found: {path}. Install yt-dlp first, or handle dependency deployment in Options."
        }
        "runtime.cookie_file_source_missing" => {
            "Cookies are enabled and the cookie source is file, but no valid Netscape cookies.txt is selected."
        }
        "runtime.cookie_source_missing" => {
            "Cookies are enabled, but no browser or cookies.txt source is selected."
        }
        "runtime.cookie_file_not_found" => {
            "Cookie file was not found: {path}. Choose a Netscape cookies.txt again, or change the cookie source back to browser."
        }
        "runtime.download_folder_empty" => "Download folder cannot be empty.",
        "runtime.could_not_start_yt_dlp" => "Could not start yt-dlp: {error}",
        "runtime.yt_dlp_analysis_failed" => "yt-dlp analysis failed: {error}",
        "runtime.could_not_parse_yt_dlp_json" => "Could not parse yt-dlp JSON: {error}",
        "runtime.yt_dlp_download_failed" => "yt-dlp download failed: {error}",
        "runtime.could_not_wait_yt_dlp" => "Could not wait for yt-dlp to finish: {error}",
        "runtime.could_not_wait_yt_dlp_missing" => {
            "Could not wait for yt-dlp to finish: child process missing"
        }
        "runtime.could_not_determine_subtitle_output" => {
            "Could not determine subtitle output file name: {error}"
        }
        "runtime.converted_subtitle_missing" => {
            "yt-dlp finished, but the converted subtitle file was not found: {error}"
        }
        "runtime.could_not_overwrite_subtitle" => {
            "Could not overwrite existing subtitle file: {error}"
        }
        "runtime.could_not_copy_subtitle" => {
            "Could not copy subtitle file to target location: {error}"
        }
        "runtime.could_not_remove_temp_subtitle" => {
            "Could not remove temporary subtitle file: {error}"
        }
        "runtime.could_not_create_download_folder" => "Could not create download folder: {error}",
        "runtime.file_does_not_exist" => "File does not exist: {error}",
        "runtime.file_location_does_not_exist" => "File location does not exist: {error}",
        "runtime.could_not_open_file" => "Could not open file: {error}",
        "runtime.could_not_open_containing_folder" => "Could not open containing folder: {error}",
        "runtime.could_not_open_folder" => "Could not open folder: {error}",
        "runtime.thumbnail_empty_url" => "Thumbnail load failed: empty URL",
        "runtime.thumbnail_no_data" => "Thumbnail load failed: no data received",
        "runtime.thumbnail_too_large" => "Thumbnail load failed: image too large",
        "runtime.thumbnail_decode_failed" => "Thumbnail decode failed: {error}",
        "runtime.invalid_thumbnail_proxy" => "Invalid thumbnail proxy setting: {error}",
        "runtime.thumbnail_http" => "Thumbnail load failed: HTTP {error}",
        "runtime.thumbnail_load_failed" => "Thumbnail load failed: {error}",
        "runtime.config_create_folder" => "Could not create config folder: {error}",
        "runtime.config_serialize" => "Could not serialize config file: {error}",
        "runtime.config_write" => "Could not write config file: {error}",
        "runtime.toast_create_notifier" => "Could not create Windows Toast notifier: {error}",
        "runtime.toast_create_content" => "Could not create Windows Toast content: {error}",
        "runtime.toast_send" => "Could not send Windows Toast: {error}",
        "runtime.toast_create_registration" => {
            "Could not create Windows Toast registration data: {error}"
        }
        "runtime.toast_register_aumid" => "Could not register Windows Toast AUMID: {error}",
        "runtime.dependency_windows_only" => {
            "Dependency deployment currently only supports Windows."
        }
        "runtime.could_not_create_tools_folder" => "Could not create tools folder {path}: {error}",
        "runtime.install_finished_missing" => {
            "{tool} installation finished, but {path} was not found."
        }
        "runtime.could_not_start_powershell" => "Could not start PowerShell: {error}",
        "runtime.could_not_read_powershell_stdout" => "Could not read PowerShell stdout.",
        "runtime.could_not_read_powershell_stderr" => "Could not read PowerShell stderr.",
        "runtime.could_not_read_powershell_output" => "Could not read PowerShell output: {error}",
        "runtime.could_not_wait_powershell" => "Could not wait for PowerShell to finish: {error}",
        "runtime.powershell_failed_exit" => "PowerShell failed: exit code {error}",
        "runtime.could_not_read_playlist_output" => {
            "Could not read yt-dlp playlist output: {error}"
        }
        "runtime.batch_import_failed" => "yt-dlp batch import failed: {error}",
        "runtime.current_path" => "Current path: {path}",
        "runtime.default_path" => "Default path: {path}",
        "runtime.not_found_path" => "Not found: {path}",
        "runtime.can_install_to" => "Can install to {path}.",
        "runtime.can_save_path" => "Can save: {path}",
        "runtime.system_check" => "System check: {detail}",
        "runtime.save_test" => "Save test: {detail}",
        "runtime.write_test" => "Write test: {detail}",
        "runtime.path_is_folder" => "{path} is a folder",
        "runtime.path_is_not_folder" => "{path} is not a folder",
        "runtime.writable_path" => "Writable: {path}",
        "runtime.missing_parent_directory" => "missing parent directory",
        "runtime.could_not_create_config_folder" => "Could not create config folder",
        "runtime.could_not_read_config_file_status" => "Could not read config file status",
        "runtime.could_not_open_config_file_for_writing" => {
            "Could not open config file for writing"
        }
        "runtime.could_not_create_folder" => "Could not create folder",
        "runtime.could_not_create_rename_delete_test_file" => {
            "Could not create, rename, or delete the test file"
        }
        "runtime.reason_path_inaccessible" => {
            "Path does not exist or the parent path is inaccessible"
        }
        "runtime.recommend_parent_exists" => "Make sure the drive and parent folder exist.",
        "runtime.reason_permission_denied_windows" => {
            "Permission denied or blocked by Windows security settings"
        }
        "runtime.recommend_move_portable_defender" => {
            "Move the app to a writable portable folder; if Desktop/Documents/Downloads still fail, Defender Controlled Folder Access may be blocking it."
        }
        "runtime.reason_in_use" => "File or folder is being used by another program",
        "runtime.recommend_close_program" => {
            "Close the program that may be using this folder, or choose another folder."
        }
        "runtime.reason_name_conflict" => "Test file already exists or name conflict",
        "runtime.reason_disk_space" => "Not enough disk space",
        "runtime.recommend_free_space" => "Free disk space or choose another disk.",
        "runtime.reason_path_too_long" => "Path is too long",
        "runtime.recommend_shorter_path" => {
            "Move the app to a shorter path, for example D:\\Portable\\yt-dlp-gui-v2."
        }
        "runtime.reason_windows_error_code" => "Windows error code {code}",
        "runtime.recommend_writable_portable_folder" => {
            "Choose a clearly writable portable folder and check again."
        }
        "runtime.reason_permission_denied" => "Permission denied or blocked by security settings",
        "runtime.reason_path_not_exist" => "Path does not exist",
        "runtime.reason_file_already_exists" => "File already exists",
        "runtime.reason_write_failed" => "Write failed",
        "runtime.recommend_not_system_folder" => {
            "Do not place the portable app under Program Files or the Windows directory; move it to D:\\Portable or a user folder."
        }
        "runtime.recommend_non_synced_folder" => {
            "Move it to a non-synced folder, for example D:\\Portable\\yt-dlp-gui-v2."
        }
        "runtime.could_not_read_playlist_output_empty" => "Could not read yt-dlp playlist output.",
        "runtime.chromium_cookie_locked" => {
            "Could not read the Chromium/Chrome cookie database directly. The browser may have locked the Network\\Cookies database. Close the browser and retry, or change Cookie source to Use file (cookies.txt) in Advanced. Original message: {error}"
        }
        "advance.cookie_source_file" => "Use file (cookies.txt)",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "All files",
        "state.untitled_task" => "Untitled task",
        "state.imported_source" => "Imported {tail}",
        "state.chapter_fallback" => "Chapter {index}",
        "runtime.config_path_unresolved" => "Config file path could not be resolved",
        "runtime.folder_readonly" => "Folder is marked read-only",
        "runtime.network_path_warning" => {
            "Located on a network path; permissions or file locks may affect it"
        }
        "runtime.protected_directory_warning" => "Located in a Windows protected directory",
        "runtime.onedrive_warning" => {
            "Located in a OneDrive sync path; sync locks or security blocking may occur"
        }
        "runtime.youtube_auto_translated_subtitle_429" => {
            "YouTube temporarily rejected the auto-translated subtitle request (HTTP 429 Too Many Requests). This is rate limiting on YouTube timedtext auto-translation. The GUI keeps the native yt-dlp flow and diagnostic output instead of switching to a custom downloader. Try enabling Cookie/cookies.txt for this item, or choose original automatic subtitles/original subtitles and retry. Original message: {error}"
        }
        "runtime.youtube_subtitle_429_conversion" => {
            "YouTube temporarily rejected the subtitle request (HTTP 429 Too Many Requests). The source subtitle file was not downloaded, so SRT conversion will not run. Retry later, or enable browser cookies before exporting. Original message: {error}"
        }
        "runtime.youtube_subtitle_429_analysis" => {
            "YouTube rejected the subtitle request (HTTP 429 Too Many Requests). This often happens on the YouTube auto-translation timedtext endpoint. cookies.txt can provide login state, but may not satisfy PO Token / rate-limit requirements for that endpoint. The GUI keeps the native yt-dlp flow and diagnostic logs instead of switching to a custom downloader. Original message: {error}"
        }
        "options.filter_executable" => "Executable",
        _ => super::en_us::text(key),
    }
}
