pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.prepare" => "準備",
        "tab.main" => "メイン",
        "tab.advanced" => "詳細",
        "tab.options" => "設定",
        "main.url_hint" => "URL",
        "action.download" => "ダウンロード",
        "action.add" => "＋ 追加",
        "action.analyze" => "解析",
        "action.stop" => "停止",
        "action.stopping" => "停止中",
        "action.cut" => "切り取り",
        "action.copy" => "コピー",
        "action.paste" => "貼り付け",
        "action.clear" => "クリア",
        "item.thumbnail" => "サムネイル",
        "item.thumbnail_preview" => "サムネイルプレビュー",
        "single.title" => "タイトル",
        "single.description" => "説明",
        "single.info.channel" => "チャンネル",
        "single.info.date" => "日付",
        "single.info.views" => "再生数",
        "thumbnail.filter.jpeg" => "JPEG 画像",
        "thumbnail.filter.png" => "PNG 画像",
        "thumbnail.filter.webp" => "WebP 画像",
        "thumbnail.filter.original" => "元の画像",
        "item.download_thumbnail" => "サムネイルをダウンロード",
        "notification.download_finished" => "ダウンロード完了",
        "notification.download_failed" => "ダウンロード失敗",
        "notification.download_finished_detail_prefix" => "完了: ",
        "notification.download_finished_detail" => "ダウンロードが完了しました。",
        "notification.windows_toast_windows_only" => "Windows Toast は Windows のみ対応です。",
        "media.video" => "動画",
        "media.audio" => "音声",
        "media.subtitle" => "字幕",
        "media.section" => "範囲",
        "item.file_name" => "ファイル名",
        "main.target_folder" => "出力フォルダー",
        "picker.title.video" => "動画形式を選択",
        "picker.title.audio" => "音声形式を選択",
        "picker.title.subtitle" => "字幕を選択",
        "picker.title.section" => "範囲を選択",
        "action.back" => "戻る",
        "picker.mode.filter" => "フィルター",
        "picker.mode.table" => "リスト",
        "action.confirm" => "確認",
        "picker.empty_table" => "表示できる形式がありません",
        "picker.header.resolution" => "解像度",
        "picker.header.range" => "範囲",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "形式",
        "picker.header.codec" => "コーデック",
        "picker.header.size" => "サイズ",
        "picker.header.sample_rate" => "サンプルレート",
        "picker.filter.resolution" => "解像度",
        "picker.filter.range" => "範囲",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "コーデック",
        "picker.filter.sample_rate" => "サンプルレート",
        "main.tooltip.missing_yt_dlp" => {
            "yt-dlpがありません。設定でインストールするかyt-dlp.exeを指定してください。"
        }
        "advance.source" => "ソース",
        "advance.config" => "設定ファイル",
        "advance.none" => "なし",
        "advance.network_access" => "ネットワークとアクセス",
        "advance.proxy" => "プロキシ",
        "advance.enable_proxy" => "プロキシを有効化",
        "advance.certificate" => "証明書",
        "advance.skip_certificate_verification" => "証明書検証をスキップ",
        "advance.use_cookies" => "Cookieを使用",
        "advance.enable_cookies" => "Cookieを有効化",
        "advance.cookie_source" => "Cookieソース",
        "advance.cookie_file" => "Cookieファイル",
        "advance.no_cookies_txt_selected" => "cookies.txtが未選択です",
        "advance.browse" => "選択",
        "advance.select_netscape_cookies_txt" => "Netscape cookies.txtを選択",
        "advance.clear" => "クリア",
        "advance.browser" => "ブラウザー",
        "advance.default" => "既定",
        "advance.external_downloader" => "外部ダウンローダー",
        "advance.use_aria2_for_faster_downloads" => "Aria2で高速化",
        "advance.download_control" => "ダウンロード制御",
        "advance.concurrent_fragments" => "同時フラグメント",
        "advance.1_default" => "1（既定）",
        "advance.rate_limit" => "速度制限",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => "例: 2M、800K。空欄なら無制限",
        "advance.chapters" => "チャプター",
        "advance.chapter_download_compatibility_mode" => "チャプター互換モード",
        "advance.file_time" => "ファイル日時",
        "advance.post_processing" => "後処理",
        "advance.thumbnail" => "サムネイル",
        "advance.download" => "ダウンロード",
        "advance.embed" => "埋め込み",
        "advance.subtitles" => "字幕",
        "advance.download_conversion" => "ダウンロード後変換",
        "advance.enable" => "有効",
        "advance.settings" => "設定",
        "item.stop_download" => "停止",
        "item.remove" => "削除",
        "item.save_as" => "名前を付けて保存",
        "item.error" => "エラー",
        "item.all" => "すべて",
        "item.queued" => "待機",
        "item.done" => "完了",
        "item.failed" => "失敗",
        "item.clear_all" => "すべてクリア",
        "item.add_a_video_url" => "動画URLを追加してください",
        "item.add_an_audio_url" => "音声 URL を追加",
        "item.after_adding_choose_the_video_format_here" => "動画形式を選択",
        "item.after_adding_choose_the_audio_format_here" => "音声形式を選択",
        "item.loading_thumbnail" => "サムネイル読み込み中",
        "item.file_actions" => "ファイル操作",
        "item.open_file" => "ファイルを開く",
        "item.open_folder" => "フォルダーを開く",
        "item.copy_path" => "パスをコピー",
        "item.opened_output_file" => "出力ファイルを開きました。",
        "item.file_not_found_opened_the_output_location" => {
            "ファイルが存在しないため、出力場所を開きました。"
        }
        "item.opened_output_location" => "出力場所を開きました。",
        "item.copied_output_path" => "出力パスをコピーしました。",
        "item.file_actions_are_available_after_download_co" => {
            "ダウンロード完了後にファイル操作が使えます"
        }
        "prepare.language" => "言語",
        "prepare.back" => "戻る",
        "prepare.auto_detect" => "自動検出",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "必要なツールをインストールします。スキップして後で設定から処理できます。"
        }
        "prepare.required" => "必須",
        "prepare.recommended" => "推奨",
        "prepare.optional" => "任意",
        "prepare.missing" => "未インストール",
        "prepare.install_later" => "必要時にインストール",
        "prepare.downloading_100" => "ダウンロード中 100%",
        "prepare.extracting_100" => "展開中 100%",
        "prepare.install_failed" => "インストール失敗",
        "prepare.install_all" => "すべてインストール",
        "prepare.reinstall" => "再インストール",
        "prepare.installing" => "インストール中",
        "prepare.skip" => "スキップ",
        "prepare.install" => "インストール",
        "prepare.another_tool_is_already_being_installed" => "他のツールをインストール中です。",
        "prepare.needs_attention" => "対応が必要",
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "このURLには動画とプレイリストが含まれています"
        }
        "options.detected" => "検出: ",
        "options.playlist_prompt" => "プレイリスト確認",
        "options.which_one_should_be_loaded" => "どちらを読み込みますか？",
        "options.both_video_and_playlist_were_detected" => "動画とプレイリストの両方を検出しました",
        "options.this_playlist_may_contain_many_items" => {
            "このプレイリストには多くの項目が含まれる可能性があります。"
        }
        "options.video" => "動画",
        "options.playlist" => "プレイリスト",
        "options.cancel" => "キャンセル",
        "options.load" => "読み込み",
        "options.behavior" => "動作",
        "options.add_action" => "追加時の動作",
        "options.download_directly" => "直接ダウンロード",
        "options.clipboard_change" => "クリップボード変更",
        "options.run_immediately" => "すぐ実行",
        "options.playlist_2" => "プレイリスト",
        "options.with_playlist" => "プレイリスト付き",
        "options.ask" => "確認する",
        "options.single_video" => "単一動画",
        "options.full_playlist" => "プレイリスト全体",
        "options.high_risk_prompt" => "高リスク確認",
        "options.on" => "オン",
        "options.playlist_count" => "リスト数",
        "options.limit" => "制限",
        "options.max" => "最大:",
        "options.items" => " 件",
        "options.language" => "言語",
        "options.current_language" => "現在の言語",
        "options.back" => "戻る",
        "options.choose" => "選択",
        "options.auto_detect" => "自動検出",
        "options.auto_detect_tool_hint" => {
            "ポータブルツールフォルダーとシステム PATH からインストール済みツールを検出します。"
        }
        "options.tool_paths" => "ツールのパス",
        "options.file_actions" => "ファイル操作",
        "options.action_button" => "操作ボタン",
        "options.cache" => "キャッシュ",
        "options.cache_location" => "キャッシュ位置",
        "options.cache_usage" => "使用量",
        "options.cache_usage_detail" => "合計: {total} · 音声: {audio} · 期限切れ: {expired}",
        "options.cache_cleanup" => "クリーンアップ",
        "options.cache_refresh" => "更新",
        "options.cache_clear_expired" => "期限切れを削除",
        "options.cache_clear_audio" => "音声を削除",
        "options.cache_clear_all" => "すべて削除",
        "options.appearance_window" => "外観とウィンドウ",
        "options.notifications" => "通知",
        "options.enable" => "有効",
        "options.theme" => "テーマ",
        "options.theme_color" => "テーマ色",
        "options.ui_scale" => "UIスケール",
        "options.apply" => "適用",
        "options.current" => "現在",
        "options.always_on_top" => "常に最前面",
        "options.window_position" => "ウィンドウ位置",
        "options.remember" => "記憶",
        "options.window_size" => "ウィンドウサイズ",
        "options.tabs" => "タブ",
        "options.log_tab" => "記録タブ",
        "options.show_log_tab" => "記録を表示",
        "options.reinstall" => "再インストール",
        "options.installing" => "インストール中",
        "options.install" => "インストール",
        "options.file_not_found" => "ファイルが存在しません: ",
        "options.will_install_to" => "インストール先: ",
        "options.another_tool_is_being_installed_please_wait" => {
            "他のツールをインストール中です。完了までお待ちください。"
        }
        "options.install_to" => "インストール先: ",
        "options.executable" => "実行ファイル",
        "main.clipboard_monitor_on_the_next_youtube_url_ch" => {
            "クリップボード監視: オン。次のYouTube URL変更時にすぐ追加します。"
        }
        "main.clipboard_monitor_on_the_next_youtube_url_ch_2" => {
            "クリップボード監視: オン。次のYouTube URL変更時にURL欄へ入力します。"
        }
        "main.clipboard_monitor_off_turning_it_on_only_mem" => {
            "クリップボード監視: オフ。有効化時は現在の内容を記録し、次の変更から動作します。"
        }
        "main.controlled_by_config" => "設定ファイルで制御: ",
        "main.controlled_by_config_2" => "設定ファイルで制御",
        "picker.no_chapters_available" => "選択できるチャプターがありません。",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "この項目でダウンロードする範囲を選択します。既定は動画全体です。"
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "チャプター互換モードが有効です。チャプター選択時は安定した単一ファイル形式を使用します。"
        }
        "picker.subtitles_will_not_be_downloaded" => "字幕はダウンロードされません。",
        "picker.no_subtitles_are_available_for_this_video" => {
            "この動画には利用可能な字幕がありません。"
        }
        "picker.no_subtitles_are_available_in_this_tab" => {
            "このタブには利用可能な字幕がありません。"
        }
        "picker.source_language" => "元の言語",
        "picker.translation_target" => "翻訳先",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "ヒント: YouTubeの自動翻訳字幕は元字幕より制限されやすいです。原文だけ必要な場合は「翻訳なし」を選んでください。"
        }
        "picker.no_subtitles_are_available_for_this_source" => {
            "このソースには利用可能な字幕がありません。"
        }
        "picker.target" => "対象",
        "picker.available_subtitles" => "利用可能な字幕",
        "picker.language" => "言語",
        "picker.subtitle_tab.none" => "字幕なし",
        "picker.subtitle_tab.original" => "元の字幕",
        "picker.subtitle_tab.automatic" => "自動字幕",
        "config.youtube_playlist_mode.ask" => "確認する",
        "config.youtube_playlist_mode.video" => "動画",
        "config.youtube_playlist_mode.ignore" => "無視",
        "config.output_action.menu" => "メニューを表示",
        "config.output_action.open_folder" => "フォルダーを開く",
        "config.output_action.open_file" => "ファイルを開く",
        "config.theme.system" => "システムに従う",
        "config.theme.light" => "ライト",
        "config.theme.dark" => "ダーク",
        "config.theme_color.off" => "オフ",
        "config.theme_color.system" => "ブルー",
        "config.theme_color.blue" => "淡いブルー",
        "config.theme_color.purple" => "紫",
        "config.theme_color.pink" => "ピンク",
        "config.theme_color.green" => "緑",
        "config.theme_color.orange" => "オレンジ",
        "config.theme_color.slate" => "スレート",
        "tools.file_time.none" => "変更しない",
        "tools.file_time.use_upload_date" => "動画の公開日を使用",
        "tools.file_time.use_download_time" => "ダウンロード時刻を使用",
        "tools.file_time.none_hint" => {
            "--mtime / --no-mtime を渡さず、最終ファイルの日時も変更しません。"
        }
        "tools.file_time.use_upload_date_hint" => {
            "yt-dlp が最終ファイルパスを返したあと、ファイルの更新日時を動画の公開日に設定します。"
        }
        "tools.file_time.use_download_time_hint" => "--no-mtime",
        "tools.cache_mode.default" => "既定",
        "tools.subtitle_source.none" => "字幕なし",
        "tools.subtitle_source.original" => "元の字幕",
        "tools.subtitle_source.automatic" => "自動字幕",
        "tools.youtube_playlist.channel_generated" => "YouTube が生成したチャンネルリスト",
        "tools.youtube_playlist.music_album" => "YouTube Music のアルバム/コレクション",
        "tools.youtube_playlist.liked_videos" => "高評価した動画",
        "tools.youtube_playlist.favorites_legacy" => "旧お気に入りリスト",
        "prepare.severity.required" => "必須項目",
        "prepare.severity.recommended" => "推奨項目",
        "prepare.severity.optional" => "任意項目",
        "prepare.status.ready" => "準備完了",
        "prepare.status.missing" => "未インストール",
        "prepare.status.warning" => "注意が必要",
        "prepare.status.failed" => "失敗",
        "tool_install.stage.preparing" => "準備中",
        "tool_install.stage.downloading" => "ダウンロード中",
        "tool_install.stage.extracting" => "展開中",
        "tool_install.stage.installing" => "インストール中",
        "tool_install.stage.completed" => "完了",
        "tool_install.stage.failed" => "失敗",
        "domain.quality.best" => "最適",
        "domain.quality.audio_only" => "音声のみ",
        "prepare.severity.short.required" => "必須",
        "prepare.severity.short.recommended" => "推奨",
        "prepare.severity.short.optional" => "任意",
        "item.status.idle" => "未開始",
        "item.status.queued" => "ダウンロード待ち",
        "item.status.running" => "ダウンロード中",
        "item.status.finished" => "完了",
        "item.status.failed" => "ダウンロード失敗",
        "item.status.cancelled" => "キャンセル済み",
        "item.status.waiting_analysis" => "分析待ち",
        "item.status.analyzing" => "分析中",
        "item.status.analysis_failed" => "分析失敗",
        "picker.waiting_analysis" => "分析待ち",
        "picker.audio_from_video" => "Video形式で決定",
        "picker.not_selected" => "未選択",
        "picker.full_video" => "フル動画",
        "picker.no_translation" => "翻訳なし",
        "picker.until_end" => "最後",
        "state.clipboard_detected_url" => "Detected a YouTube URL from the clipboard.",
        "state.no_url_to_analyze" => "There is no URL to analyze.",
        "state.analyzing_source" => "Analyzing: {source}",
        "state.batch_add_running" => "Batch add is still running.",
        "state.no_url_to_add" => "There is no URL to add.",
        "state.single_mode_playlist_not_supported" => {
            "伝統モードではプレイリストURLを扱えません。プレイリストを取り込むには標準モードへ切り替えてください。"
        }
        "state.single_mode_wait_for_current_item" => {
            "現在の伝統モード項目が完了するまで待ってください。"
        }
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
        "state.thumbnail_saved" => "サムネイルを保存しました: {path}",
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
        "state.tool_auto_detected" => "{tool} を PATH から検出しました: {path}",
        "state.tool_auto_detect_not_found" => "システム PATH で {tool} が見つかりませんでした。",
        "state.tools_auto_detected" => "PATH から {found}/{total} 個のツールを検出しました。",
        "state.tools_auto_detect_missing" => "PATH で見つかりません: {tools}。",
        "state.tools_auto_detect_none" => "システム PATH で依存ツールは見つかりませんでした。",
        "state.cache_cleaned_expired" => "期限切れキャッシュを {count} 件削除しました（{size}）。",
        "state.cache_cleaned_audio" => "音声キャッシュを削除しました: {count} 件（{size}）。",
        "state.cache_cleaned_all" => "App キャッシュを削除しました: {count} 件（{size}）。",
        "state.cache_cleanup_failed" => "キャッシュのクリーンアップに失敗しました: {error}",
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
            "動画/音声の結合、形式変換、メディア情報の分析、サムネイル/字幕処理に使います。"
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
        "runtime.download_cancelled" => "ダウンロードをキャンセルしました。",
        "runtime.yt_dlp_not_found" => {
            "yt-dlp が見つかりません: {path}。先に yt-dlp をインストールするか、オプションで依存ツールを配置してください。"
        }
        "runtime.cookie_file_source_missing" => {
            "Cookie が有効で、ソースはファイルですが、有効な Netscape cookies.txt が選択されていません。"
        }
        "runtime.cookie_source_missing" => {
            "Cookie が有効ですが、ブラウザーまたは cookies.txt のソースが選択されていません。"
        }
        "runtime.cookie_file_not_found" => {
            "Cookie ファイルが見つかりません: {path}。Netscape cookies.txt を選び直すか、Cookie ソースをブラウザーに戻してください。"
        }
        "runtime.download_folder_empty" => "ダウンロードフォルダーは空にできません。",
        "runtime.could_not_start_yt_dlp" => "yt-dlp を起動できませんでした: {error}",
        "runtime.yt_dlp_analysis_failed" => "yt-dlp の解析に失敗しました: {error}",
        "runtime.could_not_parse_yt_dlp_json" => "yt-dlp JSON を解析できませんでした: {error}",
        "runtime.yt_dlp_download_failed" => "yt-dlp のダウンロードに失敗しました: {error}",
        "runtime.could_not_wait_yt_dlp" => "yt-dlp の終了待機に失敗しました: {error}",
        "runtime.could_not_wait_yt_dlp_missing" => {
            "yt-dlp の終了待機に失敗しました: 子プロセスが見つかりません"
        }
        "runtime.could_not_determine_subtitle_output" => {
            "字幕の出力ファイル名を判定できませんでした: {error}"
        }
        "runtime.converted_subtitle_missing" => {
            "yt-dlp は終了しましたが、変換後の字幕ファイルが見つかりません: {error}"
        }
        "runtime.could_not_overwrite_subtitle" => {
            "既存の字幕ファイルを上書きできませんでした: {error}"
        }
        "runtime.could_not_copy_subtitle" => {
            "字幕ファイルを出力先にコピーできませんでした: {error}"
        }
        "runtime.could_not_remove_temp_subtitle" => {
            "一時字幕ファイルを削除できませんでした: {error}"
        }
        "runtime.could_not_create_download_folder" => {
            "ダウンロードフォルダーを作成できませんでした: {error}"
        }
        "runtime.file_does_not_exist" => "ファイルが存在しません: {error}",
        "runtime.file_location_does_not_exist" => "ファイルの場所が存在しません: {error}",
        "runtime.could_not_open_file" => "ファイルを開けませんでした: {error}",
        "runtime.could_not_open_containing_folder" => "保存先フォルダーを開けませんでした: {error}",
        "runtime.could_not_open_folder" => "フォルダーを開けませんでした: {error}",
        "runtime.thumbnail_empty_url" => "サムネイルの読み込みに失敗しました: URL が空です",
        "runtime.thumbnail_no_data" => {
            "サムネイルの読み込みに失敗しました: データを受信できませんでした"
        }
        "runtime.thumbnail_too_large" => "サムネイルの読み込みに失敗しました: 画像が大きすぎます",
        "runtime.thumbnail_decode_failed" => "サムネイルのデコードに失敗しました: {error}",
        "runtime.invalid_thumbnail_proxy" => "サムネイル用プロキシ設定が無効です: {error}",
        "runtime.thumbnail_http" => "サムネイルの読み込みに失敗しました: HTTP {error}",
        "runtime.thumbnail_load_failed" => "サムネイルの読み込みに失敗しました: {error}",
        "runtime.config_create_folder" => "設定フォルダーを作成できませんでした: {error}",
        "runtime.config_serialize" => "設定ファイルをシリアライズできませんでした: {error}",
        "runtime.config_write" => "設定ファイルを書き込めませんでした: {error}",
        "runtime.toast_create_notifier" => "Windows Toast 通知器を作成できませんでした: {error}",
        "runtime.toast_create_content" => "Windows Toast 内容を作成できませんでした: {error}",
        "runtime.toast_send" => "Windows Toast を送信できませんでした: {error}",
        "runtime.toast_create_registration" => {
            "Windows Toast 登録データを作成できませんでした: {error}"
        }
        "runtime.toast_register_aumid" => "Windows Toast AUMID を登録できませんでした: {error}",
        "runtime.dependency_windows_only" => "依存ツールの配置は現在 Windows のみ対応です。",
        "runtime.could_not_create_tools_folder" => {
            "ツールフォルダー {path} を作成できませんでした: {error}"
        }
        "runtime.install_finished_missing" => {
            "{tool} のインストールは完了しましたが、{path} が見つかりません。"
        }
        "runtime.could_not_start_powershell" => "PowerShell を起動できませんでした: {error}",
        "runtime.could_not_read_powershell_stdout" => "PowerShell stdout を読み取れませんでした。",
        "runtime.could_not_read_powershell_stderr" => "PowerShell stderr を読み取れませんでした。",
        "runtime.could_not_read_powershell_output" => {
            "PowerShell 出力を読み取れませんでした: {error}"
        }
        "runtime.could_not_wait_powershell" => "PowerShell の終了待機に失敗しました: {error}",
        "runtime.powershell_failed_exit" => "PowerShell が失敗しました: exit code {error}",
        "runtime.could_not_read_playlist_output" => {
            "yt-dlp のプレイリスト出力を読み取れませんでした: {error}"
        }
        "runtime.batch_import_failed" => "yt-dlp の一括取り込みに失敗しました: {error}",
        "runtime.current_path" => "現在のパス: {path}",
        "runtime.default_path" => "既定のパス: {path}",
        "runtime.not_found_path" => "見つかりません: {path}",
        "runtime.can_install_to" => "{path} にインストールできます。",
        "runtime.can_save_path" => "保存可能: {path}",
        "runtime.system_check" => "システムチェック: {detail}",
        "runtime.save_test" => "保存テスト: {detail}",
        "runtime.write_test" => "書き込みテスト: {detail}",
        "runtime.path_is_folder" => "{path} はフォルダーです",
        "runtime.path_is_not_folder" => "{path} はフォルダーではありません",
        "runtime.writable_path" => "書き込み可能: {path}",
        "runtime.missing_parent_directory" => "親フォルダーがありません",
        "runtime.could_not_create_config_folder" => "設定フォルダーを作成できませんでした",
        "runtime.could_not_read_config_file_status" => "設定ファイルの状態を読み取れませんでした",
        "runtime.could_not_open_config_file_for_writing" => {
            "設定ファイルを書き込み用に開けませんでした"
        }
        "runtime.could_not_create_folder" => "フォルダーを作成できませんでした",
        "runtime.could_not_create_rename_delete_test_file" => {
            "テストファイルの作成、名前変更、削除ができませんでした"
        }
        "runtime.reason_path_inaccessible" => "パスが存在しないか、親パスにアクセスできません",
        "runtime.recommend_parent_exists" => {
            "ドライブと親フォルダーが存在することを確認してください。"
        }
        "runtime.reason_permission_denied_windows" => {
            "権限が拒否されたか、Windows セキュリティ設定にブロックされています"
        }
        "runtime.recommend_move_portable_defender" => {
            "アプリをしっかり書き込めるポータブルフォルダーへ移動してください。デスクトップ、ドキュメント、ダウンロードでも失敗する場合は、Defender のフォルダーアクセス制御が原因かもしれません。"
        }
        "runtime.reason_in_use" => "ファイルまたはフォルダーが別のプログラムで使用中です",
        "runtime.recommend_close_program" => {
            "このフォルダーを使用している可能性があるプログラムを閉じるか、別のフォルダーを選んでください。"
        }
        "runtime.reason_name_conflict" => "テストファイルが既に存在するか、名前が競合しています",
        "runtime.reason_disk_space" => "ディスク空き容量が不足しています",
        "runtime.recommend_free_space" => "ディスク容量を空けるか、別のディスクを選んでください。",
        "runtime.reason_path_too_long" => "パスが長すぎます",
        "runtime.recommend_shorter_path" => {
            "アプリを短いパスへ移動してください。例: D:\\Portable\\yt-dlp-gui-v2。"
        }
        "runtime.reason_windows_error_code" => "Windows エラーコード {code}",
        "runtime.recommend_writable_portable_folder" => {
            "明確に書き込めるポータブルフォルダーを選んで、もう一度確認してください。"
        }
        "runtime.reason_permission_denied" => {
            "権限が拒否されたか、セキュリティ設定にブロックされています"
        }
        "runtime.reason_path_not_exist" => "パスが存在しません",
        "runtime.reason_file_already_exists" => "ファイルは既に存在します",
        "runtime.reason_write_failed" => "書き込みに失敗しました",
        "runtime.recommend_not_system_folder" => {
            "ポータブル版アプリを Program Files や Windows ディレクトリ下に置かないでください。D:\\Portable またはユーザーフォルダーへ移動してください。"
        }
        "runtime.recommend_non_synced_folder" => {
            "同期されないフォルダーへ移動してください。例: D:\\Portable\\yt-dlp-gui-v2。"
        }
        "runtime.could_not_read_playlist_output_empty" => {
            "yt-dlp のプレイリスト出力を読み取れませんでした。"
        }
        "runtime.chromium_cookie_locked" => {
            "Chromium/Chrome の Cookie データベースを直接読み取れませんでした。ブラウザーが Network\\Cookies データベースをロックしている可能性があります。ブラウザーを閉じて再試行するか、詳細設定で Cookie ソースをファイル（cookies.txt）に変更してください。元のメッセージ: {error}"
        }
        "advance.cookie_source_file" => "ファイルを使用（cookies.txt）",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "すべてのファイル",
        "state.untitled_task" => "無題のタスク",
        "state.imported_source" => "インポート済み {tail}",
        "state.chapter_fallback" => "チャプター {index}",
        "runtime.config_path_unresolved" => "設定ファイルのパスを解決できません",
        "runtime.folder_readonly" => "フォルダーが読み取り専用に設定されています",
        "runtime.network_path_warning" => {
            "ネットワークパス上にあるため、権限やファイルロックの影響を受ける可能性があります"
        }
        "runtime.protected_directory_warning" => "Windows の保護されたディレクトリ内にあります",
        "runtime.onedrive_warning" => {
            "OneDrive 同期パス上にあるため、同期ロックやセキュリティブロックが発生する可能性があります"
        }
        "runtime.subtitle_burn_no_source" => {
            "字幕の焼き込みには字幕ファイルまたは埋め込み字幕が必要です。先にこの項目の字幕をダウンロードするか、動画の横に .srt/.ass 字幕ファイルを置いてください。"
        }
        "runtime.youtube_auto_translated_subtitle_429" => {
            "YouTube が自動翻訳字幕リクエストを一時的に拒否しました（HTTP 429 Too Many Requests）。これは YouTube timedtext 自動翻訳のレート制限です。GUI はカスタムダウンローダーへ切り替えず、yt-dlp のネイティブ処理と診断出力を維持します。この項目で Cookie/cookies.txt を有効にするか、元の自動字幕／元の字幕を選んで再試行してください。元のメッセージ：{error}"
        }
        "runtime.youtube_subtitle_429_conversion" => {
            "YouTube が字幕リクエストを一時的に拒否しました（HTTP 429 Too Many Requests）。元の字幕ファイルがダウンロードされていないため、SRT 変換は実行されません。後で再試行するか、エクスポート前にブラウザー Cookie を有効にしてください。元のメッセージ：{error}"
        }
        "runtime.youtube_subtitle_429_analysis" => {
            "YouTube が字幕リクエストを拒否しました（HTTP 429 Too Many Requests）。これは YouTube 自動翻訳 timedtext エンドポイントでよく発生します。cookies.txt はログイン状態を提供できますが、そのエンドポイントの PO Token／レート制限要件を満たせない場合があります。GUI はカスタムダウンローダーへ切り替えず、yt-dlp のネイティブ処理と診断ログを維持します。元のメッセージ：{error}"
        }
        "options.filter_executable" => "実行ファイル",
        "processing.video" => "映像",
        "processing.audio" => "音声",
        "processing.container" => "コンテナ",
        "processing.subtitle" => "字幕",
        "processing.choice.source" => "変更なし",
        "processing.video.h264" => "H.264",
        "processing.video.hevc" => "HEVC",
        "processing.video.av1" => "AV1",
        "processing.audio.aac" => "AAC",
        "processing.audio.opus" => "Opus",
        "processing.audio.flac" => "FLAC",
        "processing.container.mp4" => "MP4",
        "processing.container.mkv" => "MKV",
        "processing.container.mov" => "MOV",
        "processing.subtitle.preserve" => "変更なし",
        "processing.subtitle.embed" => "埋め込み",
        "processing.subtitle.burn" => "焼き込み",
        "log.empty" => "実行記録はまだありません。",
        "log.clear" => "クリア",
        "log.copy" => "コピー",
        "transcode.audio.auto" => "変更なし",
        "transcode.audio.aac" => "AAC",
        "transcode.audio.opus" => "Opus",
        "transcode.audio.flac" => "FLAC",

        // English fallback translations added to keep every bundled language key-complete.
        "tab.log" => "Log",
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
        "app_mode.origin" => "伝統モード",
        "app_mode.standard" => "標準モード",
        "app_mode.audio" => "音声モード",
        "queue_display.normal" => "標準",
        "queue_display.audio" => "音声",
        "music.previous" => "前の曲",
        "music.play" => "再生",
        "music.pause" => "一時停止",
        "music.next" => "次の曲",
        "music.seek_cached_range_hint" => {
            "ドラッグしてシーク。離すとキャッシュ済み範囲内に戻ります"
        }
        "music.seek_hint" => "ドラッグして再生位置を調整",
        "music.status.completed" => "完了",
        "music.status.resolving" => "解析中",
        "music.status.buffering" => "バッファ中",
        "music.status.ready" => "再生可能",
        "music.status.caching" => "キャッシュ中",
        "music.status.playing" => "再生中",
        "music.status.paused" => "一時停止",
        "music.status.failed" => "失敗",
        "music.playback_mode.sequential" => "順番",
        "music.playback_mode.repeat_all" => "リピート",
        "music.playback_mode.shuffle" => "シャッフル",
        "music.playback_mode.repeat_one" => "1曲リピート",
        "music.playback_mode.sequential.tooltip" => "順番に再生",
        "music.playback_mode.repeat_all.tooltip" => "リストをリピート",
        "music.playback_mode.shuffle.tooltip" => "シャッフル再生",
        "music.playback_mode.repeat_one.tooltip" => "1曲をリピート",
        "options.music_download_format" => "音楽ダウンロード形式",
        "options.music_download_format_title" => "どの音声形式で出力しますか？",
        "options.music_download_format_body" => {
            "完了済みの音楽キャッシュを優先して使用し、形式が異なる場合のみ変換します。"
        }
        "state.queue_display_mode_changed" => "リストモード: {mode}",
        "state.downloading_music" => "音楽をダウンロード中: {title}",
        "state.music_no_items_from_source" => "追加できる音楽項目がありません: {source}",
        "state.music_items_added" => "{count} 件の音楽項目を追加しました。",
        "state.music_playlist_parse_failed" => "音楽リストの解析に失敗しました: {error}",
        "state.music_stream_ready" => "音楽ストリームの準備ができました: {source}",
        "state.music_stream_parse_failed" => "音楽ストリームの解析に失敗しました: {error}",
        "state.music_playback_finished" => "再生が完了しました。",
        "state.music_playback_failed" => "再生に失敗しました: {error}",
        "state.music_duplicate_with_cache" => {
            "音楽項目はすでにリストにあります。ローカルキャッシュを使用しました。"
        }
        "state.music_duplicate" => "音楽項目はすでにリストにあります。",
        "state.music_added_from_cache" => "ローカルキャッシュから音楽を追加しました: {title}",
        "state.music_seek_clamped" => "キャッシュ済み範囲外のため、再生可能な位置に戻しました。",
        "state.music_stream_still_preparing" => "音楽ストリームをまだ準備中です。",
        "state.no_playable_music_items" => "再生できる音楽項目がありません。",
        "state.music_cache_prepare_failed" => "音楽キャッシュの準備に失敗しました: {error}",
        "state.preparing_music_playback" => "再生を準備中: {title}",
        "state.music_missing_source_url" => "音楽項目にソースURLがありません。",
        "state.resolving_music_stream" => "音楽ストリームを解析中: {title}",
        "state.music_stream_still_resolving" => "音楽ストリームをまだ解析中です。",
        "state.music_buffering" => "音楽をバッファ中です。",
        "state.music_item_not_playable" => "この音楽項目は現在再生できません。",
        "state.music_stream_not_ready" => "音楽ストリームはまだ準備できていません。",
        "state.no_previous_music" => "前の曲はありません。",
        "state.no_next_music" => "次の曲はありません。",
        "state.music_playback_mode_changed" => "再生モード: {mode}",
        _ => super::en_us::text(key),
    }
}
