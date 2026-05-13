pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.prepare" => "准备",
        "tab.main" => "主页",
        "tab.advanced" => "高级",
        "tab.options" => "选项",
        "main.url_hint" => "粘贴网址",
        "action.download" => "下载",
        "action.add" => "＋ 添加",
        "action.stop" => "停止",
        "action.stopping" => "停止中",
        "action.cut" => "剪切",
        "action.copy" => "复制",
        "action.paste" => "粘贴",
        "action.clear" => "清除",
        "item.thumbnail" => "缩略图",
        "item.thumbnail_preview" => "缩略图预览",
        "notification.download_finished" => "下载完成",
        "notification.download_failed" => "下载失败",
        "notification.download_finished_detail_prefix" => "已完成：",
        "notification.download_finished_detail" => "下载已完成。",
        "notification.windows_toast_windows_only" => "Windows Toast 仅支持 Windows。",
        "media.video" => "视频",
        "media.audio" => "音频",
        "media.subtitle" => "字幕",
        "media.section" => "范围",
        "item.file_name" => "文件名",
        "main.target_folder" => "输出文件夹",
        "picker.title.video" => "选择视频格式",
        "picker.title.audio" => "选择音频格式",
        "picker.title.subtitle" => "选择字幕",
        "picker.title.section" => "选择范围",
        "action.back" => "返回",
        "picker.mode.filter" => "筛选式",
        "picker.mode.table" => "列表式",
        "action.confirm" => "确认",
        "picker.empty_table" => "没有可显示的格式项目",
        "picker.header.resolution" => "分辨率",
        "picker.header.range" => "范围",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "格式",
        "picker.header.codec" => "编码",
        "picker.header.size" => "大小",
        "picker.header.sample_rate" => "采样率",
        "picker.filter.resolution" => "分辨率",
        "picker.filter.range" => "范围",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "编码",
        "picker.filter.sample_rate" => "采样率",
        "main.tooltip.missing_yt_dlp" => {
            "yt-dlp is missing. Install it or choose yt-dlp.exe in Options."
        }
        "advance.source" => "来源",
        "advance.config" => "配置",
        "advance.none" => "无",
        "advance.network_access" => "网络与访问",
        "advance.proxy" => "代理",
        "advance.enable_proxy" => "启用代理",
        "advance.certificate" => "证书",
        "advance.skip_certificate_verification" => "跳过证书验证",
        "advance.use_cookies" => "使用 Cookie",
        "advance.enable_cookies" => "启用 Cookie",
        "advance.cookie_source" => "Cookie 来源",
        "advance.cookie_file" => "Cookie 文件",
        "advance.no_cookies_txt_selected" => "尚未选择 cookies.txt",
        "advance.browse" => "选择文件",
        "advance.select_netscape_cookies_txt" => "选择 Netscape cookies.txt",
        "advance.clear" => "清除",
        "advance.browser" => "浏览器",
        "advance.default" => "默认",
        "advance.external_downloader" => "外部下载器",
        "advance.use_aria2_for_faster_downloads" => "使用 Aria2 加速下载",
        "advance.download_control" => "下载控制",
        "advance.concurrent_fragments" => "并行分片",
        "advance.1_default" => "1（默认）",
        "advance.rate_limit" => "限速",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => "例如 2M、800K，留空不限速",
        "advance.chapters" => "章节",
        "advance.chapter_download_compatibility_mode" => "章节下载兼容模式",
        "advance.file_time" => "文件时间",
        "advance.post_processing" => "后处理",
        "advance.thumbnail" => "缩略图",
        "advance.download" => "下载",
        "advance.embed" => "嵌入",
        "advance.subtitles" => "字幕",
        "item.stop_download" => "停止下载",
        "item.remove" => "删除",
        "item.save_as" => "另存为",
        "item.error" => "错误",
        "item.all" => "全部",
        "item.queued" => "队列",
        "item.done" => "完成",
        "item.failed" => "失败",
        "item.clear_all" => "全部清除",
        "item.add_a_video_url" => "请新增视频网址",
        "item.after_adding_choose_the_video_format_here" => {
            "After adding, choose the video format here."
        }
        "item.after_adding_choose_the_audio_format_here" => {
            "After adding, choose the audio format here."
        }
        "item.loading_thumbnail" => "缩略图载入中",
        "item.file_actions" => "文件操作",
        "item.open_file" => "打开文件",
        "item.open_folder" => "打开所在文件夹",
        "item.copy_path" => "复制路径",
        "item.opened_output_file" => "已打开输出文件。",
        "item.file_not_found_opened_the_output_location" => {
            "File not found; opened the output location."
        }
        "item.opened_output_location" => "已打开输出位置。",
        "item.copied_output_path" => "已复制输出路径。",
        "item.file_actions_are_available_after_download_co" => {
            "File actions are available after download completes"
        }
        "prepare.language" => "语言",
        "prepare.back" => "返回",
        "prepare.choose" => "选择",
        "prepare.auto_detect" => "自动检测",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Install the required tools now, or skip and handle them later in Options."
        }
        "prepare.required" => "必需",
        "prepare.recommended" => "建议",
        "prepare.optional" => "可选",
        "prepare.missing" => "未安装",
        "prepare.install_later" => "需要时再安装",
        "prepare.downloading_100" => "Downloading 100%",
        "prepare.extracting_100" => "Extracting 100%",
        "prepare.install_failed" => "安装失败",
        "prepare.install_all" => "全部安装",
        "prepare.reinstall" => "重新安装",
        "prepare.installing" => "安装中",
        "prepare.skip" => "跳过",
        "prepare.install" => "安装",
        "prepare.another_tool_is_already_being_installed" => {
            "Another tool is already being installed."
        }
        "prepare.needs_attention" => "需要处理",
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "This URL contains both a video and a playlist"
        }
        "options.detected" => "Detected ",
        "options.playlist_prompt" => "Playlist prompt",
        "options.which_one_should_be_loaded" => "Which one should be loaded?",
        "options.both_video_and_playlist_were_detected" => "Both video and playlist were detected",
        "options.this_playlist_may_contain_many_items" => "This playlist may contain many items.",
        "options.video" => "视频",
        "options.playlist" => "列表",
        "options.cancel" => "取消",
        "options.load" => "载入",
        "options.behavior" => "行为",
        "options.add_action" => "新增动作",
        "options.download_directly" => "直接下载",
        "options.clipboard_change" => "剪贴板变更",
        "options.run_immediately" => "立即执行",
        "options.playlist_2" => "播放列表",
        "options.with_playlist" => "包含播放列表",
        "options.ask" => "询问",
        "options.single_video" => "单个视频",
        "options.full_playlist" => "整个列表",
        "options.high_risk_prompt" => "高风险提示",
        "options.on" => "开启",
        "options.playlist_count" => "列表数量",
        "options.limit" => "限制",
        "options.max" => "最多：",
        "options.items" => " 笔",
        "options.language" => "语言",
        "options.current_language" => "当前语言",
        "options.back" => "返回",
        "options.choose" => "选择",
        "options.auto_detect" => "自动检测",
        "options.tool_paths" => "工具路径",
        "options.file_actions" => "文件操作",
        "options.action_button" => "操作按钮",
        "options.cache" => "缓存",
        "options.cache_location" => "缓存位置",
        "options.appearance_window" => "外观与窗口",
        "options.notifications" => "通知",
        "options.enable" => "启用",
        "options.ui_scale" => "界面缩放",
        "options.apply" => "应用",
        "options.current" => "当前",
        "options.always_on_top" => "始终置顶",
        "options.window_position" => "窗口位置",
        "options.remember" => "记住",
        "options.window_size" => "窗口大小",
        "options.reinstall" => "重新安装",
        "options.installing" => "安装中",
        "options.browse" => "选择文件",
        "options.install" => "安装",
        "options.file_not_found" => "文件不存在：",
        "options.will_install_to" => "将安装到：",
        "options.another_tool_is_being_installed_please_wait" => {
            "Another tool is being installed. Please wait for it to finish."
        }
        "options.install_to" => "安装到：",
        "options.executable" => "可执行文件",
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
        "picker.source_language" => "来源语言",
        "picker.translation_target" => "翻译目标",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "Tip: YouTube auto-translated subtitles are more likely to be rate-limited than original subtitles. Choose “No translation” if you only need the source text."
        }
        "picker.no_subtitles_are_available_for_this_source" => {
            "No subtitles are available for this source."
        }
        "picker.target" => "目标",
        "picker.available_subtitles" => "可用字幕",
        "picker.language" => "语言",
        "picker.subtitle_tab.none" => "无字幕",
        "picker.subtitle_tab.original" => "原始字幕",
        "picker.subtitle_tab.automatic" => "自动字幕",
        "config.youtube_playlist_mode.ask" => "询问",
        "config.youtube_playlist_mode.video" => "视频",
        "config.youtube_playlist_mode.ignore" => "忽略",
        "config.output_action.menu" => "显示菜单",
        "config.output_action.open_folder" => "打开所在文件夹",
        "config.output_action.open_file" => "打开文件",
        "tools.file_time.none" => "不处理",
        "tools.file_time.use_upload_date" => "使用视频发布时间",
        "tools.file_time.use_download_time" => "使用下载时间",
        "tools.file_time.none_hint" => "不传入 --mtime / --no-mtime，也不修改最终文件时间。",
        "tools.file_time.use_upload_date_hint" => {
            "yt-dlp 回报最终文件路径后，将文件修改时间设为视频发布日期。"
        }
        "tools.file_time.use_download_time_hint" => "--no-mtime",
        "tools.cache_mode.default" => "默认",
        "tools.cache_mode.v2_cache" => "yt-dlp-gui",
        "tools.cache_mode.windows_temp" => "Windows",
        "tools.subtitle_source.none" => "无字幕",
        "tools.subtitle_source.original" => "原始字幕",
        "tools.subtitle_source.automatic" => "自动字幕",
        "tools.quality.best" => "最佳",
        "tools.quality.audio_only" => "仅音频",
        "tools.youtube_playlist.channel_generated" => "YouTube 生成的频道列表",
        "tools.youtube_playlist.mix_radio" => "YouTube Mix / Radio",
        "tools.youtube_playlist.music_album" => "YouTube Music 专辑/合集",
        "tools.youtube_playlist.liked_videos" => "喜欢的视频",
        "tools.youtube_playlist.favorites_legacy" => "旧版收藏列表",
        "prepare.severity.required" => "必要项目",
        "prepare.severity.recommended" => "建议项目",
        "prepare.severity.optional" => "可选项目",
        "prepare.status.ready" => "已就绪",
        "prepare.status.missing" => "未安装",
        "prepare.status.warning" => "需注意",
        "prepare.status.failed" => "失败",
        "tool_install.stage.preparing" => "準備中",
        "tool_install.stage.downloading" => "下載中",
        "tool_install.stage.extracting" => "解压中",
        "tool_install.stage.installing" => "安裝中",
        "tool_install.stage.completed" => "已完成",
        "tool_install.stage.failed" => "失敗",
        "domain.media.video" => "video",
        "domain.media.audio" => "audio",
        "domain.media.muxed" => "muxed",
        "domain.media.subtitle" => "subtitle",
        "domain.media.other" => "other",
        "domain.quality.best" => "最佳",
        "domain.quality.audio_only" => "仅音频",
        "prepare.severity.short.required" => "必须",
        "prepare.severity.short.recommended" => "建议",
        "prepare.severity.short.optional" => "可选",
        "item.status.idle" => "未开始",
        "item.status.queued" => "待下载",
        "item.status.running" => "下载中",
        "item.status.finished" => "完成",
        "item.status.failed" => "下载失败",
        "item.status.cancelled" => "已取消",
        "item.status.waiting_analysis" => "等待分析",
        "item.status.analyzing" => "分析中",
        "item.status.analysis_failed" => "分析失败",
        "picker.waiting_analysis" => "等待分析",
        "picker.audio_from_video" => "由 Video 格式决定",
        "picker.not_selected" => "未选择",
        "picker.full_video" => "完整视频",
        "picker.no_translation" => "无翻译",
        "picker.until_end" => "结尾",
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
        "runtime.download_cancelled" => "下載已取消。",
        "runtime.yt_dlp_not_found" => {
            "找不到 yt-dlp：{path}。请先安裝 yt-dlp，或到选项处理工具部署。"
        }
        "runtime.cookie_file_source_missing" => {
            "已启用 Cookie，且来源是文件，但尚未选择有效的 Netscape cookies.txt。"
        }
        "runtime.cookie_source_missing" => "已启用 Cookie，但尚未选择浏览器或 cookies.txt 来源。",
        "runtime.cookie_file_not_found" => {
            "找不到 Cookie 文件：{path}。请重新选择 Netscape cookies.txt，或把 Cookie 来源改回浏览器。"
        }
        "runtime.download_folder_empty" => "下載文件夹不能空白。",
        "runtime.could_not_start_yt_dlp" => "无法啟動 yt-dlp：{error}",
        "runtime.yt_dlp_analysis_failed" => "yt-dlp 分析失敗：{error}",
        "runtime.could_not_parse_yt_dlp_json" => "无法解析 yt-dlp JSON：{error}",
        "runtime.yt_dlp_download_failed" => "yt-dlp 下載失敗：{error}",
        "runtime.could_not_wait_yt_dlp" => "等待 yt-dlp 结束時失敗：{error}",
        "runtime.could_not_wait_yt_dlp_missing" => "等待 yt-dlp 结束時失敗：子程序不存在",
        "runtime.could_not_determine_subtitle_output" => "无法判斷字幕输出檔名：{error}",
        "runtime.converted_subtitle_missing" => "yt-dlp 已结束，但找不到轉換後的字幕檔：{error}",
        "runtime.could_not_overwrite_subtitle" => "无法覆寫既有字幕檔：{error}",
        "runtime.could_not_copy_subtitle" => "无法把字幕檔複製到目標位置：{error}",
        "runtime.could_not_remove_temp_subtitle" => "无法移除暫存字幕檔：{error}",
        "runtime.could_not_create_download_folder" => "无法建立下載文件夹：{error}",
        "runtime.file_does_not_exist" => "文件不存在：{error}",
        "runtime.file_location_does_not_exist" => "文件位置不存在：{error}",
        "runtime.could_not_open_file" => "无法打开文件：{error}",
        "runtime.could_not_open_containing_folder" => "无法打开所在文件夹：{error}",
        "runtime.could_not_open_folder" => "无法打开文件夹：{error}",
        "runtime.thumbnail_empty_url" => "缩略图載入失敗：URL 空白",
        "runtime.thumbnail_no_data" => "缩略图載入失敗：沒有收到数据",
        "runtime.thumbnail_too_large" => "缩略图載入失敗：圖片过大",
        "runtime.thumbnail_decode_failed" => "缩略图解碼失敗：{error}",
        "runtime.invalid_thumbnail_proxy" => "缩略图代理设置無效：{error}",
        "runtime.thumbnail_http" => "缩略图載入失敗：HTTP {error}",
        "runtime.thumbnail_load_failed" => "缩略图載入失敗：{error}",
        "runtime.config_create_folder" => "无法建立设置文件夹：{error}",
        "runtime.config_serialize" => "无法序列化设置檔：{error}",
        "runtime.config_write" => "无法写入设置檔：{error}",
        "runtime.toast_create_notifier" => "无法建立 Windows Toast 通知器：{error}",
        "runtime.toast_create_content" => "无法建立 Windows Toast 内容：{error}",
        "runtime.toast_send" => "无法送出 Windows Toast：{error}",
        "runtime.toast_create_registration" => "无法建立 Windows Toast 注册数据：{error}",
        "runtime.toast_register_aumid" => "无法注册 Windows Toast AUMID：{error}",
        "runtime.dependency_windows_only" => "工具部署当前只支援 Windows。",
        "runtime.could_not_create_tools_folder" => "无法建立工具文件夹 {path}：{error}",
        "runtime.install_finished_missing" => "{tool} 安裝已完成，但找不到 {path}。",
        "runtime.could_not_start_powershell" => "无法啟動 PowerShell：{error}",
        "runtime.could_not_read_powershell_stdout" => "无法读取 PowerShell stdout。",
        "runtime.could_not_read_powershell_stderr" => "无法读取 PowerShell stderr。",
        "runtime.could_not_read_powershell_output" => "无法读取 PowerShell 输出：{error}",
        "runtime.could_not_wait_powershell" => "等待 PowerShell 结束時失敗：{error}",
        "runtime.powershell_failed_exit" => "PowerShell 失敗：exit code {error}",
        "runtime.could_not_read_playlist_output" => "无法读取 yt-dlp 播放清單输出：{error}",
        "runtime.batch_import_failed" => "yt-dlp 批次导入失敗：{error}",
        "runtime.current_path" => "当前路径：{path}",
        "runtime.default_path" => "默认路径：{path}",
        "runtime.not_found_path" => "找不到：{path}",
        "runtime.can_install_to" => "可安裝到 {path}。",
        "runtime.can_save_path" => "可儲存：{path}",
        "runtime.system_check" => "系統檢查：{detail}",
        "runtime.save_test" => "儲存测试：{detail}",
        "runtime.write_test" => "写入测试：{detail}",
        "runtime.path_is_folder" => "{path} 是文件夹",
        "runtime.path_is_not_folder" => "{path} 不是文件夹",
        "runtime.writable_path" => "可写入：{path}",
        "runtime.missing_parent_directory" => "上层文件夹不存在",
        "runtime.could_not_create_config_folder" => "无法建立设置文件夹",
        "runtime.could_not_read_config_file_status" => "无法读取设置檔狀態",
        "runtime.could_not_open_config_file_for_writing" => "无法打开设置檔以写入",
        "runtime.could_not_create_folder" => "无法建立文件夹",
        "runtime.could_not_create_rename_delete_test_file" => "无法建立、重新命名或刪除测试檔",
        "runtime.reason_path_inaccessible" => "路径不存在，或上层路径无法存取",
        "runtime.recommend_parent_exists" => "请确认磁盘與上层文件夹存在。",
        "runtime.reason_permission_denied_windows" => "权限被拒，或被 Windows 安全性设置阻擋",
        "runtime.recommend_move_portable_defender" => {
            "请把程序移到可写入的可攜文件夹；如果桌面、文件、下載仍失敗，可能是 Defender 受控文件夹存取阻擋。"
        }
        "runtime.reason_in_use" => "文件或文件夹正被其他程序使用",
        "runtime.recommend_close_program" => "请關閉可能正在使用此文件夹的程序，或选择其他文件夹。",
        "runtime.reason_name_conflict" => "测试檔已存在或名称冲突",
        "runtime.reason_disk_space" => "磁盘空間不足",
        "runtime.recommend_free_space" => "请释放磁盘空間，或选择其他磁盘。",
        "runtime.reason_path_too_long" => "路径太長",
        "runtime.recommend_shorter_path" => {
            "请把程序移到较短路径，例如 D:\\Portable\\yt-dlp-gui-v2。"
        }
        "runtime.reason_windows_error_code" => "Windows 错误碼 {code}",
        "runtime.recommend_writable_portable_folder" => {
            "请选择明確可写入的可攜文件夹，然後再檢查一次。"
        }
        "runtime.reason_permission_denied" => "权限被拒，或被安全性设置阻擋",
        "runtime.reason_path_not_exist" => "路径不存在",
        "runtime.reason_file_already_exists" => "文件已存在",
        "runtime.reason_write_failed" => "写入失敗",
        "runtime.recommend_not_system_folder" => {
            "不要把可攜版程序放在 Program Files 或 Windows 目錄下；请移到 D:\\Portable 或使用者文件夹。"
        }
        "runtime.recommend_non_synced_folder" => {
            "请移到非同步文件夹，例如 D:\\Portable\\yt-dlp-gui-v2。"
        }
        "runtime.could_not_read_playlist_output_empty" => "无法读取 yt-dlp 播放列表輸出。",
        "runtime.chromium_cookie_locked" => {
            "无法直接读取 Chromium/Chrome Cookie 数据库。通常是浏览器锁住 Network\\Cookies 数据库，不是勾选狀態错误。请关闭浏览器後重試，或在高级中把 Cookie 来源改成使用文件（cookies.txt）。原始消息：{error}"
        }
        "advance.cookie_source_file" => "使用文件（cookies.txt）",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "所有文件",
        "state.untitled_task" => "未命名任务",
        "state.imported_source" => "已导入 {tail}",
        "state.chapter_fallback" => "章节 {index}",
        "runtime.config_path_unresolved" => "无法解析配置文件路径",
        "runtime.folder_readonly" => "文件夹标记为只读",
        "runtime.network_path_warning" => "位于网络路径，权限或文件锁定可能造成影响",
        "runtime.protected_directory_warning" => "位于 Windows 受保护目录",
        "runtime.onedrive_warning" => "位于 OneDrive 同步路径，可能发生同步锁定或安全性阻挡",
        "runtime.youtube_auto_translated_subtitle_429" => {
            "YouTube 暂时拒绝自动翻译字幕请求（HTTP 429 Too Many Requests）。这是 YouTube timedtext 自动翻译的速率限制。GUI 会保留原生 yt-dlp 流程与诊断输出，不改用自定义下载器。可尝试为此项目启用 Cookie/cookies.txt，或改选原始自动字幕／原始字幕后重试。原始信息：{error}"
        }
        "runtime.youtube_subtitle_429_conversion" => {
            "YouTube 暂时拒绝字幕请求（HTTP 429 Too Many Requests）。源字幕文件未下载，因此不会进行 SRT 转换。请稍后重试，或在导出前启用浏览器 Cookie。原始信息：{error}"
        }
        "runtime.youtube_subtitle_429_analysis" => {
            "YouTube 拒绝字幕请求（HTTP 429 Too Many Requests）。这通常发生在 YouTube 自动翻译 timedtext 端点。cookies.txt 可提供登录状态，但不一定能满足该端点的 PO Token／速率限制需求。GUI 会保留原生 yt-dlp 流程与诊断日志，不改用自定义下载器。原始信息：{error}"
        }
        "options.filter_executable" => "可执行文件",
        _ => super::en_us::text(key),
    }
}
