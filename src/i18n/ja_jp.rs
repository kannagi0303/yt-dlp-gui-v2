pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.advanced" => "詳細",
        "tab.about" => "情報",
        "about.tools" => "ツール版本",
        "about.current_version" => "現在の版本",
        "about.latest_version" => "最新版本",
        "about.author" => "作者",
        "about.source" => "取得元",
        "about.status" => "状態",
        "about.message" => "メッセージ",
        "about.check_updates" => "更新確認",
        "about.update_all" => "すべて更新",
        "about.restart" => "再起動",
        "about.open_release" => "Release を開く",
        "about.install" => "インストール",
        "about.update" => "更新",
        "about.running" => "更新確認中...",
        "about.last_check" => "前回確認：",
        "about.relative.minutes" => "{count} 分",
        "about.relative.hour" => "1 時間",
        "about.relative.hours" => "{count} 時間",
        "about.relative.day" => "1 日",
        "about.relative.days" => "{count} 日",
        "about.never_checked" => "まだ更新確認していません",
        "about.no_release_notes_loaded" => {
            "更新内容はまだ読み込まれていません。先に更新確認を実行してください。"
        }
        "about.ownership.managed_portable" => "v2 管理",
        "about.ownership.external" => "外部",
        "about.ownership.missing" => "未インストール",
        "about.ownership.unknown" => "不明",
        "about.status.unknown" => "未確認",
        "about.status.checking" => "確認中",
        "about.status.up_to_date" => "最新 ✓",
        "about.status.update_available" => "更新可 ↑",
        "about.status.missing" => "未インストール +",
        "about.status.downloading" => "ダウンロード中",
        "about.status.downloading_percent" => "ダウンロード中 {percent}%",
        "about.status.staged" => "準備済み",
        "about.status.pending_restart" => "再起動待ち",
        "about.status.applying" => "適用中",
        "about.status.installed" => "インストール済み",
        "about.status.skipped" => "スキップ済み",
        "about.status.failed" => "失敗 !",
        "tab.options" => "設定",
        "tab.log" => "ログ",
        "main.url_hint" => "URL",
        "action.download" => "ダウンロード",
        "action.add" => "追加",
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
        "item.download_thumbnail" => "サムネイルをダウンロード",
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
        "main.missing_yt_dlp_callout" => {
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
        "advance.cookie_source.auto" => "サイトごとに自動適用",
        "advance.cookie_source.file" => "ファイルを使う（cookies.txt）",
        "advance.cookie_auto" => "自動",
        "advance.cookie_auto_note" => "ダウンロード時にURLに合う保存済みCookieを使います。",
        "advance.cookie_rescue" => "Cookie",
        "advance.cookie_file" => "Cookieファイル",
        "advance.get_cookie" => "Cookieを取得",
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
        "youtube_login_rescue.short_note" => {
            "専用ブラウザーウィンドウを開いて Cookie を取得します。"
        }
        "youtube_login_rescue.title" => "Cookieを取得",
        "youtube_login_rescue.confirm_heading" => "専用ブラウザーのログインウィンドウを開く",
        "youtube_login_rescue.confirm_body" => {
            "独立した {browser} でURLを開きます。普段のブラウザデータは読みません。"
        }
        "youtube_login_rescue.target_url_label" => "Web サイト URL",
        "youtube_login_rescue.target_url_hint" => "https://www.youtube.com/",
        "youtube_login_rescue.clipboard_prefilled" => "クリップボードから URL を入力しました。",
        "youtube_login_rescue.drop_url_note" => {
            "URL の貼り付け、または .url / テキストをドロップできます。"
        }
        "youtube_login_rescue.paste_clipboard" => "クリップボードを貼り付け",
        "youtube_login_rescue.cookie_note" => "ログイン操作を行うと、自動で閉じて適用します。",
        "youtube_login_rescue.no_browser_title" => "対応ブラウザーが見つかりません",
        "youtube_login_rescue.no_browser_body" => {
            "Cookie取得には Chrome、Brave、Microsoft Edge のいずれかが必要です。cookies.txt を手動で選択することもできます。"
        }
        "youtube_login_rescue.start" => "開始",
        "youtube_login_rescue.opening" => "{browser}を開いています...",
        "youtube_login_rescue.waiting_for_cdp" => "{browser} のログインウィンドウ接続を待機中...",
        "youtube_login_rescue.waiting_for_cookie" => {
            "ログインウィンドウに接続しました。サイト Cookie を待機中..."
        }
        "youtube_login_rescue.cookie_exported" => "Cookieを保存しました。",
        "youtube_login_rescue.cookie_exported_note" => {
            "{site} の Cookie を保存しました。そのサイトのダウンロード時に自動で使います。"
        }
        "youtube_login_rescue.do_not_close_note" => {
            "確認中はログインブラウザーを閉じないでください。"
        }
        "youtube_login_rescue.cdp_ready" => "ログインウィンドウに接続しました。",
        "youtube_login_rescue.ready_next_step_note" => {
            "ブラウザーでYouTubeへのログインを完了してください。Cookieの書き出しは次の段階で追加します。"
        }
        "youtube_login_rescue.close_login_window" => "ログインウィンドウを閉じる",
        "youtube_login_rescue.failed" => "Cookieの取得に失敗しました",
        "youtube_login_rescue.retry" => "再試行",
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
        "advance.file_time.none" => "変更しない",
        "advance.file_time.upload_date" => "動画のアップロード日を使う",
        "advance.file_time.download_time" => "ダウンロード時刻を使う",
        "advance.post_processing" => "後処理",
        "advance.thumbnail" => "サムネイル",
        "advance.download" => "ダウンロード",
        "advance.embed" => "埋め込み",
        "advance.subtitles" => "字幕",
        "advance.download_conversion" => "ダウンロード後変換",
        "advance.enable" => "有効",
        "advance.settings" => "設定",
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
        "item.file_not_found_opened_the_output_location" => {
            "ファイルが存在しないため、出力場所を開きました。"
        }
        "item.opened_output_location" => "出力場所を開きました。",
        "item.copied_output_path" => "出力パスをコピーしました。",
        "prepare.language" => "言語",
        "prepare.back" => "戻る",
        "prepare.auto_detect" => "自動検出",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "必要なツールをインストールします。スキップして後で設定から処理できます。"
        }
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
        "prepare.req.app_folder.title" => "アプリフォルダー",
        "prepare.req.app_folder.description" => {
            "設定とサポートデータを保存するため、ポータブルフォルダーは書き込み可能である必要があります。"
        }
        "prepare.req.tools_folder.title" => "ツールフォルダー",
        "prepare.req.tools_folder.description" => {
            "依存ツールの配置先として、yt-dlp、FFmpeg、Deno がここに保存されます。"
        }
        "prepare.req.deployment_temp.title" => "配置用一時フォルダー",
        "prepare.req.deployment_temp.description" => {
            "FFmpeg と Deno の展開時にこの一時フォルダーを使用します。"
        }
        "prepare.req.download_cache.title" => "ダウンロードキャッシュ",
        "prepare.req.download_cache.description" => {
            "yt-dlp-gui のキャッシュモードでは、yt-dlp のキャッシュをここに保存します。"
        }
        "prepare.req.output_folder.title" => "出力フォルダー",
        "prepare.req.output_folder.description" => "動画、音声、字幕はここに保存されます。",
        "prepare.req.output_folder.recommendation" => {
            "メイン画面またはオプションから有効な出力フォルダーを選んでください。"
        }
        "prepare.req.config_file.title" => "設定ファイル",
        "prepare.req.config_file.description" => {
            "Prepare のスキップ状態とツールパス設定を保存できる必要があります。"
        }
        "prepare.req.generic_writable_recommendation" => {
            "書き込み可能なフォルダーを選び、権限を確認してください。"
        }
        "prepare.req.config_not_folder" => {
            "設定ファイルのパスがフォルダーを指しています。ファイルパスを選んでください。"
        }
        "prepare.req.config_readonly" => "設定ファイルは読み取り専用です。",
        "prepare.req.config_readonly_recommendation" => {
            "設定ファイルへの書き込みを許可するか、別のアプリフォルダーを選んでください。"
        }
        "prepare.req.use_folder_path" => "ファイルパスではなくフォルダーパスを選んでください。",
        "prepare.req.move_portable_folder" => {
            "アプリを書き込み可能なポータブルフォルダーへ移動してください。"
        }
        "prepare.req.avoid_protected_folder" => {
            "ポータブル版アプリを Program Files や Windows フォルダー内に置かないでください。D:\\Portable またはユーザーフォルダーへ移動してください。"
        }
        "prepare.req.move_non_synced_folder" => {
            "同期対象外のフォルダーへ移動してください。例: D:\\Portable\\yt-dlp-gui-v2。"
        }
        "prepare.req.drive_parent_exists" => {
            "ドライブと親フォルダーが存在することを確認してください。"
        }
        "prepare.req.permission_denied" => {
            "アプリを書き込み可能なポータブルフォルダーへ移動してください。デスクトップ、ドキュメント、ダウンロードでも失敗する場合は、Defender の制御されたフォルダーアクセスがブロックしている可能性があります。"
        }
        "prepare.req.file_in_use" => {
            "このフォルダーを使用している可能性のあるプログラムを閉じるか、別のフォルダーを選んでください。"
        }
        "prepare.req.free_disk_space" => "ディスク容量を空けるか、別のディスクを選んでください。",
        "prepare.req.path_too_long" => {
            "アプリを短いパスへ移動してください。例: D:\\Portable\\yt-dlp-gui-v2。"
        }
        "prepare.req.choose_writable_portable_folder" => {
            "確実に書き込み可能なポータブルフォルダーを選び、もう一度確認してください。"
        }
        "prepare.req.clear_write_test" => {
            "残った書き込みテストファイルを削除して、もう一度確認してください。"
        }
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
        "options.playlist_risk.kind.channel_generated" => "YouTube 生成チャンネルプレイリスト",
        "options.playlist_risk.kind.youtube_mix_radio" => "YouTube Mix / Radio",
        "options.playlist_risk.kind.youtube_music_album" => "YouTube Music アルバム / コレクション",
        "options.playlist_risk.kind.liked_videos" => "高く評価した動画",
        "options.playlist_risk.kind.favorites_legacy" => "旧式のお気に入りプレイリスト",
        "options.playlist_risk.note.channel_generated" => {
            "この YouTube 生成チャンネルプレイリストは保守的に扱ってください。"
        }
        "options.playlist_risk.note.youtube_mix_radio" => {
            "この Mix / Radio プレイリストには多くの項目が含まれる可能性があり、時間とともに変わる場合があります。"
        }
        "options.playlist_risk.note.youtube_music_album" => {
            "通常は YouTube Music のアルバムまたはコレクションです。"
        }
        "options.playlist_risk.note.liked_videos" => {
            "高く評価した動画は通常、ログインまたは cookies が必要です。"
        }
        "options.playlist_risk.note.favorites_legacy" => {
            "これは旧式のお気に入りプレイリスト形式で、現在は安定しない場合があります。"
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
        "options.tabs" => "タブ",
        "options.log_tab" => "ログタブ",
        "options.show_log_tab" => "ログを表示",
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
        "options.tool_paths" => "ツールのパス",
        "options.file_actions" => "ファイル操作",
        "options.action_button" => "操作ボタン",
        "options.file_action.show_menu" => "メニューを表示",
        "options.cache" => "キャッシュ",
        "options.cache_location" => "キャッシュ位置",
        "options.cache_location.default" => "既定",
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
        "options.theme_mode.system" => "システムに合わせる",
        "options.theme_mode.light" => "ライト",
        "options.theme_mode.dark" => "ダーク",
        "options.theme_color" => "テーマ色",
        "options.theme_color.off" => "オフ",
        "options.theme_color.blue" => "ブルー",
        "options.theme_color.soft_blue" => "ソフトブルー",
        "options.theme_color.purple" => "パープル",
        "options.theme_color.pink" => "ピンク",
        "options.theme_color.green" => "グリーン",
        "options.theme_color.orange" => "オレンジ",
        "options.theme_color.slate" => "スレート",
        "options.ui_scale" => "UIスケール",
        "options.apply" => "適用",
        "options.current" => "現在",
        "options.always_on_top" => "常に最前面",
        "options.window_position" => "ウィンドウ位置",
        "options.remember" => "記憶",
        "options.window_size" => "ウィンドウサイズ",
        "options.reinstall" => "再インストール",
        "options.installing" => "インストール中",
        "options.install" => "インストール",
        "options.executable" => "実行ファイル",
        "main.controlled_by_config" => "設定ファイルで制御: ",
        "main.controlled_by_config_2" => "設定ファイルで制御",
        "picker.no_chapters_available" => "選択できるチャプターがありません。",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "この項目でダウンロードする範囲を選択します。既定は動画全体です。"
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "チャプター互換モードが有効です。チャプター選択時は、より安定した単一ファイル形式を使用します。"
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
        "picker.waiting_analysis" => "分析待ち",
        "picker.audio_from_video" => "Video形式で決定",
        "picker.not_selected" => "未選択",
        "picker.full_video" => "フル動画",
        "picker.no_translation" => "翻訳なし",
        "picker.until_end" => "最後",
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
        "item.status.queued" => "ダウンロード待ち",
        "item.status.running" => "ダウンロード中",
        "item.status.finished" => "完了",
        "item.status.failed" => "ダウンロード失敗",
        "item.status.cancelled" => "キャンセル済み",
        "processing.transcode" => "変換",
        "transcode.graph.axis.compatibility" => "互換性",
        "transcode.graph.axis.capacity" => "容量",
        "transcode.graph.axis.resolution" => "解像度",
        "transcode.graph.axis.format" => "Format",
        "transcode.graph.compatibility_scope" => "互換性の範囲",
        "transcode.graph.capacity_target" => "容量目標",
        "transcode.graph.resolution_limit" => "解像度上限",
        "transcode.graph.format_goal" => "形式の目標",
        "processing.video" => "映像",
        "processing.audio" => "音声",
        "processing.container" => "コンテナ",
        "processing.subtitle" => "字幕",
        "processing.choice.source" => "変更なし",
        "processing.subtitle.preserve" => "変更なし",
        "processing.subtitle.embed" => "埋め込み",
        "processing.subtitle.burn" => "焼き込み",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "すべてのファイル",
        "options.filter_executable" => "実行ファイル",
        "app_mode.origin" => "伝統モード",
        "app_mode.standard" => "標準モード",
        "app_mode.audio" => "音声モード",
        "music.status.completed" => "完了",
        "music.status.resolving" => "解析中",
        "music.status.buffering" => "バッファ中",
        "music.status.ready" => "再生可能",
        "music.status.caching" => "キャッシュ中",
        "music.status.playing" => "再生中",
        "music.status.paused" => "一時停止",
        "music.status.failed" => "失敗",
        "notification.download_complete" => "ダウンロード完了",
        "notification.download_failed" => "ダウンロード失敗",
        "notification.completed_file" => "完了：{file}",
        "notification.download_completed" => "ダウンロードが完了しました。",
        "options.music_download_format" => "音声ダウンロード",
        "options.music_download_audio_label" => "音声出力",
        "options.music_download_preference_best" => "最適",
        _ => key,
    }
}
