pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.advanced" => "Advanced",
        "tab.about" => "About",
        "about.tools" => "Tool versions",
        "about.current_version" => "Current",
        "about.latest_version" => "Latest",
        "about.author" => "Author",
        "about.source" => "Source",
        "about.status" => "Status",
        "about.message" => "Message",
        "about.check_updates" => "Check updates",
        "about.update_all" => "Update all",
        "about.restart" => "Restart",
        "about.open_release" => "Open Release",
        "about.install" => "Install",
        "about.update" => "Update",
        "about.running" => "Update check is running...",
        "about.last_check" => "Last check:",
        "about.relative.minutes" => "{count} min",
        "about.relative.hour" => "1 hour",
        "about.relative.hours" => "{count} hours",
        "about.relative.day" => "1 day",
        "about.relative.days" => "{count} days",
        "about.never_checked" => "Updates have not been checked yet",
        "about.no_release_notes_loaded" => "No release notes loaded. Press Check Updates first.",
        "about.ownership.managed_portable" => "v2 managed",
        "about.ownership.external" => "External",
        "about.ownership.missing" => "Missing",
        "about.ownership.unknown" => "Unknown",
        "about.status.unknown" => "Not checked",
        "about.status.checking" => "Checking",
        "about.status.up_to_date" => "Latest ✓",
        "about.status.update_available" => "Update available ↑",
        "about.status.missing" => "Missing +",
        "about.status.downloading" => "Downloading",
        "about.status.downloading_percent" => "Downloading {percent}%",
        "about.status.staged" => "Staged",
        "about.status.pending_restart" => "Pending restart",
        "about.status.applying" => "Applying",
        "about.status.installed" => "Installed",
        "about.status.skipped" => "Skipped",
        "about.status.failed" => "Failed !",
        "tab.options" => "Options",
        "tab.log" => "Log",
        "main.url_hint" => "URL",
        "action.download" => "Download",
        "action.add" => "Add",
        "action.analyze" => "Analyze",
        "action.stop" => "Stop",
        "action.stopping" => "Stopping",
        "action.cut" => "Cut",
        "action.copy" => "Copy",
        "action.paste" => "Paste",
        "action.clear" => "Clear",
        "item.thumbnail" => "Thumbnail",
        "item.thumbnail_preview" => "Thumbnail preview",
        "single.title" => "Title",
        "single.description" => "Description",
        "single.info.channel" => "Channel",
        "single.info.date" => "Date",
        "single.info.views" => "Views",
        "item.download_thumbnail" => "Download thumbnail",
        "media.video" => "Video",
        "media.audio" => "Audio",
        "media.subtitle" => "Subtitles",
        "media.section" => "Range",
        "item.file_name" => "File name",
        "main.target_folder" => "Output folder",
        "picker.title.video" => "Select video format",
        "picker.title.audio" => "Select audio format",
        "picker.title.subtitle" => "Select subtitles",
        "picker.title.section" => "Select range",
        "action.back" => "Back",
        "picker.mode.filter" => "Filters",
        "picker.mode.table" => "Table",
        "action.confirm" => "Confirm",
        "picker.empty_table" => "No format items to display",
        "picker.header.resolution" => "Resolution",
        "picker.header.range" => "Range",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "Format",
        "picker.header.codec" => "Codec",
        "picker.header.size" => "Size",
        "picker.header.sample_rate" => "Sample rate",
        "picker.filter.resolution" => "Resolution",
        "picker.filter.range" => "Range",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "Codec",
        "picker.filter.sample_rate" => "Sample rate",
        "main.missing_yt_dlp_callout" => {
            "yt-dlp is missing. Install it or choose yt-dlp.exe in Options."
        }
        "advance.source" => "Source",
        "advance.config" => "Config",
        "advance.none" => "None",
        "advance.network_access" => "Network & Access",
        "advance.proxy" => "Proxy",
        "advance.enable_proxy" => "Enable proxy",
        "advance.certificate" => "Certificate",
        "advance.skip_certificate_verification" => "Skip certificate verification",
        "advance.use_cookies" => "Use cookies",
        "advance.enable_cookies" => "Enable cookies",
        "advance.cookie_source" => "Cookie source",
        "advance.cookie_source.auto" => "Auto by website",
        "advance.cookie_source.file" => "Use file (cookies.txt)",
        "advance.cookie_auto" => "Auto",
        "advance.cookie_auto_note" => "Downloads use the saved Cookie that matches the URL.",
        "advance.cookie_rescue" => "Cookie",
        "advance.cookie_file" => "Cookie file",
        "advance.get_cookie" => "Get Cookie",
        "advance.cookie" => "Cookie",
        "advance.cookie.off" => "Do not use",
        "advance.cookie.browser" => "Browser",
        "advance.cookie.file" => "File",
        "advance.cookie_file_source" => "File source",
        "advance.cookie_file_custom" => "Custom cookies.txt",
        "advance.cookie_file_auto_select" => "Auto select",
        "advance.cookie_manager_row" => "Management",
        "advance.manage_cookie" => "Manage Cookie",
        "advance.cookie_manager_title" => "Cookie Manager",
        "advance.add_cookie" => "Add Cookie",
        "advance.cookie_manager_empty" => "No Cookie files have been added.",
        "advance.cookie_manager_name" => "Name",
        "advance.cookie_manager_updated" => "Updated",
        "advance.cookie_manager_actions" => "Actions",
        "advance.cookie_manager_refresh" => "Reacquire",
        "advance.cookie_manager_delete" => "Delete",
        "advance.file" => "File",
        "youtube_login_rescue.short_note" => "Open a dedicated browser window to get cookies.",
        "youtube_login_rescue.title" => "Get Cookie",
        "youtube_login_rescue.confirm_heading" => "Open a dedicated browser login window",
        "youtube_login_rescue.confirm_body" => {
            "An independent {browser} window will open the URL without using your personal browser data."
        }
        "youtube_login_rescue.target_url_label" => "Website URL",
        "youtube_login_rescue.target_url_hint" => "https://www.youtube.com/",
        "youtube_login_rescue.clipboard_prefilled" => "URL was filled from clipboard.",
        "youtube_login_rescue.drop_url_note" => "Paste a URL, or drop a .url / text file.",
        "youtube_login_rescue.paste_clipboard" => "Paste clipboard",
        "youtube_login_rescue.cookie_note" => {
            "Sign in there, then click the top-page “I have finished signing in, save cookies” button."
        }
        "youtube_login_rescue.no_browser_title" => "No supported browser found",
        "youtube_login_rescue.no_browser_body" => {
            "Getting cookies currently needs Chrome, Brave, or Microsoft Edge. You can still choose cookies.txt manually."
        }
        "youtube_login_rescue.start" => "Start",
        "youtube_login_rescue.opening" => "Opening {browser}...",
        "youtube_login_rescue.waiting_for_cdp" => {
            "Waiting for the {browser} login window to connect..."
        }
        "youtube_login_rescue.waiting_for_cookie" => {
            "Login window is connected. Sign in, then click the save button at the top of the page."
        }
        "youtube_login_rescue.cookie_exported" => "Cookie has been saved.",
        "youtube_login_rescue.cookie_exported_note" => {
            "Saved {site} Cookie. Downloads from that website will use it automatically."
        }
        "youtube_login_rescue.do_not_close_note" => {
            "Please keep the login browser open until you click the in-page save button."
        }
        "youtube_login_rescue.cdp_ready" => "Login window is connected.",
        "youtube_login_rescue.ready_next_step_note" => {
            "Please finish signing in to YouTube in the browser. Cookie export will be added in the next step."
        }
        "youtube_login_rescue.close_login_window" => "Close login window",
        "youtube_login_rescue.failed" => "Get Cookie failed",
        "youtube_login_rescue.retry" => "Retry",
        "advance.no_cookies_txt_selected" => "No cookies.txt selected",
        "advance.browse" => "Browse",
        "advance.select_netscape_cookies_txt" => "Select Netscape cookies.txt",
        "advance.clear" => "Clear",
        "advance.browser" => "Browser",
        "advance.default" => "Default",
        "advance.external_downloader" => "External downloader",
        "advance.use_aria2_for_faster_downloads" => "Use Aria2 for faster downloads",
        "advance.download_control" => "Download control",
        "advance.concurrent_fragments" => "Concurrent fragments",
        "advance.live_streams" => "Live streams",
        "advance.download_live_streams_from_start_experimental" => {
            "Download live streams from the start (experimental)"
        }
        "advance.1_default" => "1 (default)",
        "advance.rate_limit" => "Rate limit",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => {
            "e.g. 2M, 800K; leave empty for unlimited"
        }
        "advance.chapters" => "Chapters",
        "advance.download_range" => "Download range",
        "advance.always_show_download_range" => "Always show range selection",
        "advance.chapter_download_compatibility_mode" => "Chapter download compatibility mode",
        "advance.file_time" => "File time",
        "advance.file_time.none" => "Do not change",
        "advance.file_time.upload_date" => "Use video upload date",
        "advance.file_time.download_time" => "Use download time",
        "advance.post_processing" => "Post-processing",
        "advance.thumbnail" => "Thumbnail",
        "advance.download" => "Download",
        "advance.embed" => "Embed",
        "advance.subtitles" => "Subtitles",
        "advance.download_conversion" => "Convert after download",
        "advance.enable" => "Enable",
        "advance.settings" => "Settings",
        "item.save_as" => "Save as",
        "item.error" => "Error",
        "item.all" => "All",
        "item.queued" => "Queued",
        "item.done" => "Done",
        "item.failed" => "Failed",
        "item.clear_all" => "Clear all",
        "item.add_a_video_url" => "Add a video URL",
        "item.add_an_audio_url" => "Add an audio URL",
        "item.after_adding_choose_the_video_format_here" => "Choose video format",
        "item.after_adding_choose_the_audio_format_here" => "Choose audio format",
        "item.loading_thumbnail" => "Loading thumbnail",
        "item.file_actions" => "File actions",
        "item.open_file" => "Open file",
        "item.open_folder" => "Open folder",
        "item.copy_path" => "Copy path",
        "item.file_not_found_opened_the_output_location" => {
            "File not found; opened the output location."
        }
        "item.opened_output_location" => "Opened output location.",
        "item.copied_output_path" => "Copied output path.",
        "prepare.language" => "Language",
        "prepare.back" => "Back",
        "prepare.auto_detect" => "Auto detect",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Install the required tools now, or skip and handle them later in Options."
        }
        "prepare.optional" => "Optional",
        "prepare.missing" => "Missing",
        "prepare.install_later" => "Install later",
        "prepare.downloading_100" => "Downloading 100%",
        "prepare.extracting_100" => "Extracting 100%",
        "prepare.install_failed" => "Install failed",
        "prepare.install_all" => "Install all",
        "prepare.reinstall" => "Reinstall",
        "prepare.installing" => "Installing",
        "prepare.skip" => "Skip",
        "prepare.install" => "Install",
        "prepare.another_tool_is_already_being_installed" => {
            "Another tool is already being installed."
        }
        "prepare.needs_attention" => "Needs attention",
        "prepare.req.app_folder.title" => "App folder",
        "prepare.req.app_folder.description" => {
            "The portable folder must be writable for settings and support folders."
        }
        "prepare.req.tools_folder.title" => "Tools folder",
        "prepare.req.tools_folder.description" => {
            "Dependency deployment stores yt-dlp, FFmpeg, and Deno here."
        }
        "prepare.req.deployment_temp.title" => "Deployment temp",
        "prepare.req.deployment_temp.description" => {
            "FFmpeg and Deno extraction uses this temp folder."
        }
        "prepare.req.download_cache.title" => "Download cache",
        "prepare.req.download_cache.description" => {
            "yt-dlp-gui cache mode stores yt-dlp cache here."
        }
        "prepare.req.output_folder.title" => "Output folder",
        "prepare.req.output_folder.description" => "Videos, audio, and subtitles are saved here.",
        "prepare.req.output_folder.recommendation" => {
            "Choose a valid output folder from Main or Options."
        }
        "prepare.req.config_file.title" => "Config file",
        "prepare.req.config_file.description" => {
            "The app must be able to save prepare-page skip and tool path settings."
        }
        "prepare.req.generic_writable_recommendation" => {
            "Choose a writable folder and check permissions."
        }
        "prepare.req.config_not_folder" => {
            "The config path points to a folder. Choose a file path instead."
        }
        "prepare.req.config_readonly" => "The config file is read-only.",
        "prepare.req.config_readonly_recommendation" => {
            "Allow writing to the config file or choose another app folder."
        }
        "prepare.req.use_folder_path" => "Choose a folder path instead of a file path.",
        "prepare.req.move_portable_folder" => "Move the app to a writable portable folder.",
        "prepare.req.avoid_protected_folder" => {
            "Do not place the portable app under Program Files or the Windows directory. Move it to D:\\Portable or a user folder."
        }
        "prepare.req.move_non_synced_folder" => {
            "Move it to a non-synced folder, for example D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.drive_parent_exists" => "Make sure the drive and parent folder exist.",
        "prepare.req.permission_denied" => {
            "Move the app to a writable portable folder. If Desktop/Documents/Downloads still fail, Defender Controlled Folder Access may be blocking it."
        }
        "prepare.req.file_in_use" => {
            "Close the program that may be using this folder, or choose another folder."
        }
        "prepare.req.free_disk_space" => "Free disk space or choose another disk.",
        "prepare.req.path_too_long" => {
            "Move the app to a shorter path, for example D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.choose_writable_portable_folder" => {
            "Choose a clearly writable portable folder and check again."
        }
        "prepare.req.clear_write_test" => "Remove the leftover write-test file and check again.",
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "This URL contains both a video and a playlist"
        }
        "options.detected" => "Detected ",
        "options.playlist_prompt" => "Playlist prompt",
        "options.which_one_should_be_loaded" => "Which one should be loaded?",
        "options.both_video_and_playlist_were_detected" => "Both video and playlist were detected",
        "options.this_playlist_may_contain_many_items" => "This playlist may contain many items.",
        "options.playlist_risk.kind.channel_generated" => "YouTube generated channel playlist",
        "options.playlist_risk.kind.youtube_mix_radio" => "YouTube Mix / Radio",
        "options.playlist_risk.kind.youtube_music_album" => "YouTube Music album/collection",
        "options.playlist_risk.kind.liked_videos" => "Liked videos",
        "options.playlist_risk.kind.favorites_legacy" => "Legacy favorites playlist",
        "options.playlist_risk.note.channel_generated" => {
            "Treat this YouTube-generated channel playlist conservatively."
        }
        "options.playlist_risk.note.youtube_mix_radio" => {
            "This Mix / Radio playlist may contain many items and can change over time."
        }
        "options.playlist_risk.note.youtube_music_album" => {
            "This is usually a YouTube Music album or collection."
        }
        "options.playlist_risk.note.liked_videos" => {
            "Liked videos usually require login or cookies."
        }
        "options.playlist_risk.note.favorites_legacy" => {
            "This is a legacy favorites playlist style and may not be stable now."
        }
        "options.video" => "Video",
        "options.playlist" => "Playlist",
        "options.cancel" => "Cancel",
        "options.load" => "Load",
        "options.behavior" => "Behavior",
        "options.add_action" => "Add action",
        "options.download_directly" => "Download directly",
        "options.clipboard_change" => "Clipboard change",
        "options.run_immediately" => "Run immediately",
        "options.tabs" => "Tabs",
        "options.log_tab" => "Log tab",
        "options.show_log_tab" => "Show log",
        "options.playlist_2" => "Playlist",
        "options.with_playlist" => "With playlist",
        "options.ask" => "Ask",
        "options.single_video" => "Single video",
        "options.full_playlist" => "Full playlist",
        "options.high_risk_prompt" => "High-risk prompt",
        "options.on" => "On",
        "options.playlist_count" => "Playlist count",
        "options.limit" => "Limit",
        "options.max" => "Max:",
        "options.items" => " items",
        "options.language" => "Language",
        "options.current_language" => "Current language",
        "options.back" => "Back",
        "options.choose" => "Choose",
        "options.auto_detect" => "Auto detect",
        "options.tool_paths" => "Tool paths",
        "options.file_actions" => "File actions",
        "options.action_button" => "Action button",
        "options.file_action.show_menu" => "Show menu",
        "options.cache" => "Cache",
        "options.cache_location" => "Cache location",
        "options.cache_location.default" => "Default",
        "options.cache_usage" => "Usage",
        "options.cache_usage_detail" => "Total: {total} · Audio: {audio} · Expired: {expired}",
        "options.cache_cleanup" => "Cleanup",
        "options.cache_refresh" => "Refresh",
        "options.cache_clear_expired" => "Clear expired",
        "options.cache_clear_audio" => "Clear audio",
        "options.cache_clear_all" => "Clear all",
        "options.appearance_window" => "Appearance & Window",
        "options.notifications" => "Notifications",
        "options.enable" => "Enable",
        "options.theme" => "Theme",
        "options.theme_mode.system" => "Follow system",
        "options.theme_mode.light" => "Light",
        "options.theme_mode.dark" => "Dark",
        "options.theme_color" => "Theme color",
        "options.theme_color.off" => "Off",
        "options.theme_color.blue" => "Blue",
        "options.theme_color.soft_blue" => "Soft blue",
        "options.theme_color.purple" => "Purple",
        "options.theme_color.pink" => "Pink",
        "options.theme_color.green" => "Green",
        "options.theme_color.orange" => "Orange",
        "options.theme_color.slate" => "Slate",
        "options.ui_scale" => "UI scale",
        "options.apply" => "Apply",
        "options.current" => "Current",
        "options.always_on_top" => "Always on top",
        "options.window_position" => "Window position",
        "options.remember" => "Remember",
        "options.window_size" => "Window size",
        "options.reinstall" => "Reinstall",
        "options.installing" => "Installing",
        "options.install" => "Install",
        "options.executable" => "executable",
        "main.controlled_by_config" => "Controlled by config: ",
        "main.controlled_by_config_2" => "Controlled by config",
        "picker.section_tab.chapters" => "Chapters",
        "picker.section_tab.time_range" => "Time range",
        "picker.section_chapter_instructions" => {
            "Select one or more chapters. Adjacent chapters become one output."
        }
        "picker.section_time_instructions" => {
            "Move the playhead, set the start and end, then add the range."
        }
        "picker.section_time_unavailable" => {
            "The video duration is unavailable, so a custom time range cannot be created."
        }
        "picker.section_select_all" => "Select all",
        "picker.section_from_selected_to_end" => "From first selected to end",
        "picker.section_set_start" => "Set start",
        "picker.section_set_end" => "Set end",
        "picker.section_add_range" => "Add range",
        "picker.section_no_custom_ranges" => "No custom time ranges added.",
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
        "picker.language" => "Language",
        "picker.subtitle_tab.none" => "No subtitles",
        "picker.subtitle_tab.original" => "Original subtitles",
        "picker.subtitle_tab.automatic" => "Automatic subtitles",
        "picker.waiting_analysis" => "Waiting for analysis",
        "picker.audio_from_video" => "Decided by video format",
        "picker.not_selected" => "Not selected",
        "picker.full_video" => "Full video",
        "picker.section_summary.chapters" => "{chapters} chapters selected · {outputs} outputs",
        "picker.section_summary.custom" => "{ranges} time ranges · {outputs} outputs",
        "picker.section_summary.combined" => {
            "{chapters} chapters + {ranges} time ranges · {outputs} outputs"
        }
        "picker.no_translation" => "No translation",
        "picker.until_end" => "end",
        "prepare.status.ready" => "Ready",
        "prepare.status.missing" => "Missing",
        "prepare.status.warning" => "Needs attention",
        "prepare.status.failed" => "Failed",
        "tool_install.stage.preparing" => "Preparing",
        "tool_install.stage.downloading" => "Downloading",
        "tool_install.stage.extracting" => "Extracting",
        "tool_install.stage.installing" => "Installing",
        "tool_install.stage.completed" => "Completed",
        "tool_install.stage.failed" => "Failed",
        "item.status.queued" => "Queued",
        "item.status.running" => "Running",
        "item.status.finished" => "Done",
        "item.status.failed" => "Failed",
        "item.status.cancelled" => "Cancelled",
        "processing.transcode" => "Transcode",
        "transcode.graph.axis.compatibility" => "Compatibility",
        "transcode.graph.axis.capacity" => "Capacity",
        "transcode.graph.axis.resolution" => "Resolution",
        "transcode.graph.axis.format" => "Format",
        "transcode.graph.compatibility_scope" => "Compatibility scope",
        "transcode.graph.capacity_target" => "Size target",
        "transcode.graph.resolution_limit" => "Resolution limit",
        "transcode.graph.format_goal" => "Format goal",
        "processing.video" => "Video",
        "processing.audio" => "Audio",
        "processing.container" => "Container",
        "processing.subtitle" => "Subtitles",
        "processing.choice.source" => "Original",
        "processing.subtitle.preserve" => "Original",
        "processing.subtitle.embed" => "Embed",
        "processing.subtitle.burn" => "Burn in",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "All files",
        "options.filter_executable" => "Executable",
        "app_mode.origin" => "Origin Mode",
        "app_mode.standard" => "Standard Mode",
        "app_mode.audio" => "Audio Mode",
        "music.status.completed" => "Done",
        "music.status.resolving" => "Resolving",
        "music.status.buffering" => "Buffering",
        "music.status.ready" => "Ready",
        "music.status.caching" => "Caching",
        "music.status.playing" => "Playing",
        "music.status.paused" => "Paused",
        "music.status.failed" => "Failed",
        "notification.download_complete" => "Download complete",
        "notification.download_failed" => "Download failed",
        "notification.completed_file" => "Completed: {file}",
        "notification.download_completed" => "Download completed.",
        "options.music_download_format" => "Music audio",
        "options.music_download_audio_label" => "Audio output",
        "options.music_download_preference_best" => "Best",
        _ => key,
    }
}
