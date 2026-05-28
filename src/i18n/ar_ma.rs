pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.prepare" => "التحضير",
        "tab.main" => "الرئيسية",
        "tab.advanced" => "متقدم",
        "tab.options" => "الخيارات",
        "main.url_hint" => "URL",
        "action.download" => "تنزيل",
        "action.add" => "＋ إضافة",
        "action.stop" => "إيقاف",
        "action.stopping" => "جارٍ الإيقاف",
        "action.cut" => "قص",
        "action.copy" => "نسخ",
        "action.paste" => "لصق",
        "action.clear" => "مسح",
        "item.thumbnail" => "صورة مصغّرة",
        "item.thumbnail_preview" => "معاينة الصورة المصغّرة",
        "notification.download_finished" => "اكتمل التنزيل",
        "notification.download_failed" => "فشل التنزيل",
        "notification.download_finished_detail_prefix" => "اكتمل: ",
        "notification.download_finished_detail" => "اكتمل التنزيل.",
        "notification.windows_toast_windows_only" => "Windows Toast مدعوم على Windows فقط.",
        "media.video" => "فيديو",
        "media.audio" => "الصوت",
        "media.subtitle" => "الترجمات",
        "media.section" => "النطاق",
        "item.file_name" => "اسم الملف",
        "main.target_folder" => "مجلد الإخراج",
        "picker.title.video" => "Select video format",
        "picker.title.audio" => "Select audio format",
        "picker.title.subtitle" => "Select subtitles",
        "picker.title.section" => "Select range",
        "action.back" => "رجوع",
        "picker.mode.filter" => "عوامل التصفية",
        "picker.mode.table" => "جدول",
        "action.confirm" => "تأكيد",
        "picker.empty_table" => "No format items to display",
        "picker.header.resolution" => "الدقة",
        "picker.header.range" => "النطاق",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "الصيغة",
        "picker.header.codec" => "الترميز",
        "picker.header.size" => "الحجم",
        "picker.header.sample_rate" => "Sample rate",
        "picker.filter.resolution" => "الدقة",
        "picker.filter.range" => "النطاق",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "الترميز",
        "picker.filter.sample_rate" => "Sample rate",
        "main.tooltip.missing_yt_dlp" => {
            "yt-dlp is missing. Install it or choose yt-dlp.exe in Options."
        }
        "advance.source" => "المصدر",
        "advance.config" => "الإعدادات",
        "advance.none" => "لا شيء",
        "advance.network_access" => "Network & Access",
        "advance.proxy" => "الوكيل",
        "advance.enable_proxy" => "Enable proxy",
        "advance.certificate" => "Certificate",
        "advance.skip_certificate_verification" => "Skip certificate verification",
        "advance.use_cookies" => "Use cookies",
        "advance.enable_cookies" => "Enable cookies",
        "advance.cookie_source" => "Cookie source",
        "advance.cookie_file" => "Cookie file",
        "advance.no_cookies_txt_selected" => "No cookies.txt selected",
        "advance.browse" => "Browse",
        "advance.select_netscape_cookies_txt" => "Select Netscape cookies.txt",
        "advance.clear" => "Clear",
        "advance.browser" => "المتصفح",
        "advance.default" => "افتراضي",
        "advance.external_downloader" => "External downloader",
        "advance.use_aria2_for_faster_downloads" => "Use Aria2 for faster downloads",
        "advance.download_control" => "Download control",
        "advance.concurrent_fragments" => "Concurrent fragments",
        "advance.1_default" => "1 (default)",
        "advance.rate_limit" => "Rate limit",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => {
            "e.g. 2M, 800K; leave empty for unlimited"
        }
        "advance.chapters" => "الفصول",
        "advance.chapter_download_compatibility_mode" => "Chapter download compatibility mode",
        "advance.file_time" => "وقت الملف",
        "advance.post_processing" => "Post-processing",
        "advance.thumbnail" => "الصورة المصغرة",
        "advance.download" => "تنزيل",
        "advance.embed" => "تضمين",
        "advance.subtitles" => "الترجمات",
        "item.stop_download" => "إيقاف التنزيل",
        "item.remove" => "إزالة",
        "item.save_as" => "حفظ باسم",
        "item.error" => "خطأ",
        "item.all" => "All",
        "item.queued" => "في الانتظار",
        "item.done" => "تم",
        "item.failed" => "فشل",
        "item.clear_all" => "مسح الكل",
        "item.add_a_video_url" => "Add a video URL",
        "item.after_adding_choose_the_video_format_here" => "اختيار تنسيق الفيديو",
        "item.after_adding_choose_the_audio_format_here" => "اختيار تنسيق الصوت",
        "item.loading_thumbnail" => "Loading thumbnail",
        "item.file_actions" => "File actions",
        "item.open_file" => "فتح الملف",
        "item.open_folder" => "فتح المجلد",
        "item.copy_path" => "نسخ المسار",
        "item.opened_output_file" => "Opened output file.",
        "item.file_not_found_opened_the_output_location" => {
            "File not found; opened the output location."
        }
        "item.opened_output_location" => "Opened output location.",
        "item.copied_output_path" => "Copied output path.",
        "item.file_actions_are_available_after_download_co" => {
            "File actions are available after download completes"
        }
        "prepare.language" => "اللغة",
        "prepare.back" => "رجوع",
        "prepare.auto_detect" => "اكتشاف تلقائي",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Install the required tools now, or skip and handle them later in Options."
        }
        "prepare.required" => "مطلوب",
        "prepare.recommended" => "موصى به",
        "prepare.optional" => "اختياري",
        "prepare.missing" => "مفقود",
        "prepare.install_later" => "Install later",
        "prepare.downloading_100" => "Downloading 100%",
        "prepare.extracting_100" => "Extracting 100%",
        "prepare.install_failed" => "Install failed",
        "prepare.install_all" => "Install all",
        "prepare.reinstall" => "Reinstall",
        "prepare.installing" => "Installing",
        "prepare.skip" => "Skip",
        "prepare.install" => "تثبيت",
        "prepare.another_tool_is_already_being_installed" => {
            "Another tool is already being installed."
        }
        "prepare.needs_attention" => "Needs attention",
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "This URL contains both a video and a playlist"
        }
        "options.detected" => "Detected ",
        "options.playlist_prompt" => "Playlist prompt",
        "options.which_one_should_be_loaded" => "Which one should be loaded?",
        "options.both_video_and_playlist_were_detected" => "Both video and playlist were detected",
        "options.this_playlist_may_contain_many_items" => "This playlist may contain many items.",
        "options.video" => "فيديو",
        "options.playlist" => "قائمة التشغيل",
        "options.cancel" => "إلغاء",
        "options.load" => "تحميل",
        "options.behavior" => "السلوك",
        "options.add_action" => "Add action",
        "options.download_directly" => "Download directly",
        "options.clipboard_change" => "Clipboard change",
        "options.run_immediately" => "Run immediately",
        "options.playlist_2" => "قائمة التشغيل",
        "options.with_playlist" => "With playlist",
        "options.ask" => "اسأل",
        "options.single_video" => "Single video",
        "options.full_playlist" => "Full playlist",
        "options.high_risk_prompt" => "High-risk prompt",
        "options.on" => "On",
        "options.playlist_count" => "Playlist count",
        "options.limit" => "Limit",
        "options.max" => "Max:",
        "options.items" => " items",
        "options.language" => "اللغة",
        "options.current_language" => "اللغة الحالية",
        "options.back" => "رجوع",
        "options.choose" => "اختيار",
        "options.auto_detect" => "اكتشاف تلقائي",
        "options.tool_paths" => "مسارات الأدوات",
        "options.file_actions" => "File actions",
        "options.action_button" => "Action button",
        "options.cache" => "ذاكرة التخزين المؤقت",
        "options.cache_location" => "Cache location",
        "options.appearance_window" => "Appearance & Window",
        "options.notifications" => "الإشعارات",
        "options.enable" => "تمكين",
        "options.ui_scale" => "مقياس الواجهة",
        "options.apply" => "تطبيق",
        "options.current" => "الحالي",
        "options.always_on_top" => "دائمًا في الأعلى",
        "options.window_position" => "Window position",
        "options.remember" => "تذكر",
        "options.window_size" => "Window size",
        "options.reinstall" => "Reinstall",
        "options.installing" => "Installing",
        "options.install" => "تثبيت",
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
        "picker.language" => "اللغة",
        "picker.subtitle_tab.none" => "No subtitles",
        "picker.subtitle_tab.original" => "Original subtitles",
        "picker.subtitle_tab.automatic" => "Automatic subtitles",
        "config.youtube_playlist_mode.ask" => "اسأل",
        "config.youtube_playlist_mode.video" => "فيديو",
        "config.youtube_playlist_mode.ignore" => "Ignore",
        "config.output_action.menu" => "Show menu",
        "config.output_action.open_folder" => "فتح المجلد",
        "config.output_action.open_file" => "فتح الملف",
        "tools.file_time.none" => "لا تغيّر",
        "tools.file_time.use_upload_date" => "استعمل تاريخ نشر الفيديو",
        "tools.file_time.use_download_time" => "استعمل وقت التنزيل",
        "tools.file_time.none_hint" => "لا يمرر --mtime/--no-mtime ولا يغيّر وقت الملف النهائي.",
        "tools.file_time.use_upload_date_hint" => {
            "بعد أن يبلّغ yt-dlp عن المسار النهائي، يتم ضبط وقت تعديل الملف على تاريخ نشر الفيديو."
        }
        "tools.file_time.use_download_time_hint" => "--no-mtime",
        "tools.cache_mode.default" => "افتراضي",
        "tools.subtitle_source.none" => "No subtitles",
        "tools.subtitle_source.original" => "Original subtitles",
        "tools.subtitle_source.automatic" => "Automatic subtitles",
        "tools.youtube_playlist.channel_generated" => "YouTube generated channel playlist",
        "tools.youtube_playlist.music_album" => "YouTube Music album/collection",
        "tools.youtube_playlist.liked_videos" => "Liked videos",
        "tools.youtube_playlist.favorites_legacy" => "Legacy favorites playlist",
        "prepare.severity.required" => "Required item",
        "prepare.severity.recommended" => "Recommended item",
        "prepare.severity.optional" => "Optional item",
        "prepare.status.ready" => "جاهز",
        "prepare.status.missing" => "مفقود",
        "prepare.status.warning" => "Needs attention",
        "prepare.status.failed" => "فشل",
        "tool_install.stage.preparing" => "Preparing",
        "tool_install.stage.downloading" => "Downloading",
        "tool_install.stage.extracting" => "Extracting",
        "tool_install.stage.installing" => "Installing",
        "tool_install.stage.completed" => "Completed",
        "tool_install.stage.failed" => "Failed",
        "domain.quality.best" => "الأفضل",
        "domain.quality.audio_only" => "صوت فقط",
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

        // English fallback translations added to keep every bundled language key-complete.
        "tab.log" => "Log",
        "advance.download_conversion" => "Convert after download",
        "advance.enable" => "Enable",
        "advance.settings" => "Settings",
        "options.tabs" => "Tabs",
        "options.log_tab" => "Log tab",
        "options.show_log_tab" => "Show log",
        "options.theme" => "Theme",
        "options.theme_color" => "Theme color",
        "config.theme.system" => "Follow system",
        "config.theme.light" => "Light",
        "config.theme.dark" => "Dark",
        "config.theme_color.off" => "Off",
        "config.theme_color.system" => "Blue",
        "config.theme_color.blue" => "Soft blue",
        "config.theme_color.purple" => "Purple",
        "config.theme_color.pink" => "Pink",
        "config.theme_color.green" => "Green",
        "config.theme_color.orange" => "Orange",
        "config.theme_color.slate" => "Slate",
        "state.transcode_post_processing_title" => "Converting with {profile}: {title}",
        "processing.transcode" => "Transcode",
        "transcode.intent.reduce_size" => "Smaller file",
        "transcode.intent.quality_first" => "Quality first",
        "transcode.intent.target_size" => "Size target",
        "transcode.intent.fast_transcode" => "Format",
        "transcode.intent.device_compat" => "Compatibility target",
        "transcode.compat.most_devices" => "Most devices / not sure",
        "transcode.compat.windows" => "Windows PC",
        "transcode.compat.mac" => "Mac",
        "transcode.compat.apple" => "Apple devices",
        "transcode.compat.tv_nas" => "Generic TV / NAS",
        "transcode.compat.old_device" => "Old TV / USB playback",
        "transcode.compat.apple_tv_legacy" => "Apple TV legacy",
        "transcode.compat.apple_tv_modern" => "Apple TV modern",
        "transcode.compat.iphone_ipad" => "iPhone / iPad",
        "transcode.compat.android_tv" => "Android TV / Chromecast",
        "transcode.compat.android_phone_tablet" => "Android phone / tablet",
        "transcode.compat.browser_mp4" => "Browser-safe MP4",
        "transcode.fps.source" => "Source",
        "transcode.fps.24" => "Up to 24 fps",
        "transcode.fps.25" => "Up to 25 fps",
        "transcode.fps.30" => "Up to 30 fps",
        "transcode.fps.60" => "Up to 60 fps",
        "transcode.setting.fps" => "FPS limit",
        "transcode.graph.axis.compatibility" => "Compatibility",
        "transcode.graph.axis.capacity" => "Capacity",
        "transcode.graph.axis.resolution" => "Resolution",
        "transcode.graph.axis.format" => "Format",
        "transcode.graph.compatibility_scope" => "Compatibility scope",
        "transcode.graph.capacity_target" => "Size target",
        "transcode.graph.resolution_limit" => "Resolution limit",
        "transcode.graph.format_goal" => "Format goal",
        "transcode.quality.standard" => "Standard",
        "transcode.quality.high" => "High quality",
        "transcode.quality.near_original" => "Near original",
        "transcode.resolution.auto_balance" => "Auto balance",
        "transcode.resolution.keep_original" => "Keep original",
        "transcode.resolution.max_1080p" => "Max 1080p",
        "transcode.resolution.max_720p" => "Max 720p",
        "transcode.effort.fast" => "Fast",
        "transcode.effort.normal" => "Normal",
        "transcode.effort.detailed" => "Detailed",
        "transcode.effort.extreme" => "Extreme",
        "transcode.setting.compatibility" => "Compatibility",
        "transcode.setting.video_codec" => "Video codec",
        "transcode.setting.container" => "Container",
        "transcode.setting.encoder" => "Encoder",
        "transcode.setting.quality" => "Quality",
        "transcode.setting.size_ratio" => "Size ratio",
        "transcode.setting.target_size" => "Target size",
        "transcode.setting.resolution" => "Resolution",
        "transcode.setting.effort" => "Effort",
        "transcode.setting.pass" => "Size control",
        "transcode.setting.audio" => "Audio",
        "transcode.support.executable" => "Executable",
        "transcode.support.partial" => "Partially supported",
        "transcode.support.preview_only" => "Preview only",
        "processing.video" => "Video",
        "processing.audio" => "Audio",
        "processing.container" => "Container",
        "processing.subtitle" => "Subtitles",
        "processing.choice.source" => "Source",
        "processing.video.h264" => "H.264",
        "processing.video.hevc" => "HEVC",
        "processing.video.av1" => "AV1",
        "processing.audio.aac" => "AAC",
        "processing.audio.opus" => "Opus",
        "processing.audio.flac" => "FLAC",
        "processing.container.mp4" => "MP4",
        "processing.container.mkv" => "MKV",
        "processing.container.mov" => "MOV",
        "processing.subtitle.preserve" => "Source",
        "processing.subtitle.embed" => "Embed",
        "processing.subtitle.burn" => "Burn in",
        "log.empty" => "No runtime log yet.",
        "log.clear" => "Clear",
        "log.copy" => "Copy",
        "transcode.audio.auto" => "Source",
        "transcode.audio.aac" => "AAC",
        "transcode.audio.opus" => "Opus",
        "transcode.audio.flac" => "FLAC",
        "runtime.subtitle_burn_no_source" => {
            "Subtitle burn-in needs a subtitle file or embedded subtitle. Download subtitles for this item first, or place an .srt/.ass subtitle file beside the video."
        }
        // English fallback translations keep bundled languages key-complete.
        "item.add_an_audio_url" => "Add an audio URL",
        "options.auto_detect_tool_hint" => {
            "Detect installed tools from the portable tools folder and system PATH."
        }
        "options.cache_usage" => "Usage",
        "options.cache_usage_detail" => "Total: {total} · Audio: {audio} · Expired: {expired}",
        "options.cache_cleanup" => "Cleanup",
        "options.cache_refresh" => "Refresh",
        "options.cache_clear_expired" => "Clear expired",
        "options.cache_clear_audio" => "Clear audio",
        "options.cache_clear_all" => "Clear all",
        "state.tool_auto_detected" => "{tool} detected from PATH: {path}",
        "state.tool_auto_detect_not_found" => "{tool} was not found in system PATH.",
        "state.tools_auto_detected" => "Detected {found}/{total} tools from PATH.",
        "state.tools_auto_detect_missing" => "Not found in PATH: {tools}.",
        "state.tools_auto_detect_none" => "No dependency tools were found in system PATH.",
        "state.cache_cleaned_expired" => "Cleared {count} expired cache entries ({size}).",
        "state.cache_cleaned_audio" => "Cleared audio cache: {count} entries ({size}).",
        "state.cache_cleaned_all" => "Cleared app cache: {count} entries ({size}).",
        "state.cache_cleanup_failed" => "Cache cleanup failed: {error}",
        "app_mode.origin" => "Origin Mode",
        "app_mode.standard" => "Standard Mode",
        "app_mode.audio" => "Audio Mode",
        "queue_display.normal" => "Standard",
        "queue_display.audio" => "Audio",
        "music.previous" => "Previous",
        "music.play" => "Play",
        "music.pause" => "Pause",
        "music.next" => "Next",
        "music.seek_cached_range_hint" => "Drag to seek; release snaps within the cached range",
        "music.seek_hint" => "Drag to seek",
        "music.status.completed" => "Done",
        "music.status.resolving" => "Resolving",
        "music.status.buffering" => "Buffering",
        "music.status.ready" => "Ready",
        "music.status.caching" => "Caching",
        "music.status.playing" => "Playing",
        "music.status.paused" => "Paused",
        "music.status.failed" => "Failed",
        "music.playback_mode.sequential" => "Sequence",
        "music.playback_mode.repeat_all" => "Repeat",
        "music.playback_mode.shuffle" => "Shuffle",
        "music.playback_mode.repeat_one" => "Repeat one",
        "music.playback_mode.sequential.tooltip" => "Play in order",
        "music.playback_mode.repeat_all.tooltip" => "Repeat list",
        "music.playback_mode.shuffle.tooltip" => "Shuffle play",
        "music.playback_mode.repeat_one.tooltip" => "Repeat one track",
        "options.music_download_format" => "Music download format",
        "options.music_download_format_title" => "Which audio format should be exported?",
        "options.music_download_format_body" => {
            "Completed music cache is used first; conversion only runs when the format differs."
        }
        "state.queue_display_mode_changed" => "List mode: {mode}",
        "state.downloading_music" => "Downloading music: {title}",
        "state.music_no_items_from_source" => "No music items could be added: {source}",
        "state.music_items_added" => "Added {count} music items.",
        "state.music_playlist_parse_failed" => "Music list analysis failed: {error}",
        "state.music_stream_ready" => "Music stream ready: {source}",
        "state.music_stream_parse_failed" => "Music stream analysis failed: {error}",
        "state.music_playback_finished" => "Playback finished.",
        "state.music_playback_failed" => "Playback failed: {error}",
        "state.music_duplicate_with_cache" => {
            "Music item is already in the list; local cache was used."
        }
        "state.music_duplicate" => "Music item is already in the list.",
        "state.music_added_from_cache" => "Added music from local cache: {title}",
        "state.music_seek_clamped" => {
            "Outside the cached range; moved back to a playable position."
        }
        "state.music_stream_still_preparing" => "Music stream is still preparing.",
        "state.no_playable_music_items" => "There are no playable music items.",
        "state.music_cache_prepare_failed" => "Music cache preparation failed: {error}",
        "state.preparing_music_playback" => "Preparing playback: {title}",
        "state.music_missing_source_url" => "Music item is missing a source URL.",
        "state.resolving_music_stream" => "Resolving music stream: {title}",
        "state.music_stream_still_resolving" => "Music stream is still resolving.",
        "state.music_buffering" => "Music is buffering.",
        "state.music_item_not_playable" => "This music item cannot be played right now.",
        "state.music_stream_not_ready" => "Music stream is not ready yet.",
        "state.no_previous_music" => "No previous track.",
        "state.no_next_music" => "No next track.",
        "state.music_playback_mode_changed" => "Playback mode: {mode}",
        "action.analyze" => "تحليل",
        "item.download_thumbnail" => "تنزيل الصورة المصغرة",
        "single.title" => "العنوان",
        "single.description" => "الوصف",
        "single.info.channel" => "القناة",
        "single.info.date" => "التاريخ",
        "single.info.views" => "المشاهدات",
        "thumbnail.filter.jpeg" => "صورة JPEG",
        "thumbnail.filter.png" => "صورة PNG",
        "thumbnail.filter.webp" => "صورة WebP",
        "thumbnail.filter.original" => "الصورة الأصلية",
        "state.single_mode_playlist_not_supported" => {
            "Origin Mode does not support playlist URLs. Switch to Standard Mode to import a playlist."
        }
        "state.single_mode_wait_for_current_item" => {
            "Wait for the current Origin Mode item to finish first."
        }
        "state.thumbnail_saved" => "تم حفظ الصورة المصغرة: {path}",
        _ => super::en_us::text(key),
    }
}
