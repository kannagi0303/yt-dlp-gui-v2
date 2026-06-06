pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.advanced" => "고급",
        "tab.about" => "About",
        "about.tools" => "도구 버전",
        "about.current_version" => "현재",
        "about.latest_version" => "최신",
        "about.author" => "작성자",
        "about.source" => "소스",
        "about.status" => "상태",
        "about.message" => "메시지",
        "about.check_updates" => "업데이트 확인",
        "about.update_all" => "모두 업데이트",
        "about.restart" => "다시 시작",
        "about.open_release" => "Release 열기",
        "about.install" => "설치",
        "about.update" => "업데이트",
        "about.running" => "업데이트 확인 중...",
        "about.last_check" => "마지막 확인:",
        "about.relative.minutes" => "{count}분",
        "about.relative.hour" => "1시간",
        "about.relative.hours" => "{count}시간",
        "about.relative.day" => "1일",
        "about.relative.days" => "{count}일",
        "about.never_checked" => "아직 업데이트를 확인하지 않음",
        "about.no_release_notes_loaded" => {
            "릴리스 노트를 불러오지 않았습니다. 먼저 업데이트를 확인하세요."
        }
        "about.ownership.managed_portable" => "v2 관리",
        "about.ownership.external" => "외부",
        "about.ownership.missing" => "없음",
        "about.ownership.unknown" => "알 수 없음",
        "about.status.unknown" => "확인 안 됨",
        "about.status.checking" => "확인 중",
        "about.status.up_to_date" => "최신 ✓",
        "about.status.update_available" => "업데이트 있음 ↑",
        "about.status.missing" => "없음 +",
        "about.status.downloading" => "다운로드 중",
        "about.status.downloading_percent" => "다운로드 중 {percent}%",
        "about.status.staged" => "준비됨",
        "about.status.pending_restart" => "다시 시작 필요",
        "about.status.applying" => "적용 중",
        "about.status.installed" => "설치됨",
        "about.status.skipped" => "건너뜀",
        "about.status.failed" => "실패 !",
        "tab.options" => "옵션",
        "tab.log" => "로그",
        "main.url_hint" => "URL",
        "action.download" => "다운로드",
        "action.add" => "추가",
        "action.analyze" => "분석",
        "action.stop" => "중지",
        "action.stopping" => "중지 중",
        "action.cut" => "잘라내기",
        "action.copy" => "복사",
        "action.paste" => "붙여넣기",
        "action.clear" => "지우기",
        "item.thumbnail" => "썸네일",
        "item.thumbnail_preview" => "썸네일 미리보기",
        "single.title" => "제목",
        "single.description" => "설명",
        "single.info.channel" => "채널",
        "single.info.date" => "날짜",
        "single.info.views" => "조회수",
        "item.download_thumbnail" => "썸네일 다운로드",
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
        "picker.empty_table" => "표시할 형식 항목이 없습니다",
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
        "main.missing_yt_dlp_callout" => {
            "yt-dlp가 없습니다. 설치하거나 옵션에서 yt-dlp.exe를 선택하세요."
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
        "advance.cookie_source.auto" => "웹사이트별 자동",
        "advance.cookie_source.file" => "파일 사용 (cookies.txt)",
        "advance.cookie_auto" => "자동",
        "advance.cookie_auto_note" => "다운로드는 URL과 일치하는 저장된 Cookie를 사용합니다.",
        "advance.cookie_rescue" => "Cookie 가져오기",
        "advance.cookie_file" => "쿠키 파일",
        "advance.get_cookie" => "Cookie 가져오기",
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
        "youtube_login_rescue.short_note" => "Cookie를 가져올 전용 브라우저 창을 엽니다.",
        "youtube_login_rescue.title" => "Cookie 가져오기",
        "youtube_login_rescue.confirm_heading" => "전용 브라우저 로그인 창 열기",
        "youtube_login_rescue.confirm_body" => {
            "독립된 {browser} 창이 개인 브라우저 데이터를 사용하지 않고 URL을 엽니다."
        }
        "youtube_login_rescue.target_url_label" => "웹사이트 URL",
        "youtube_login_rescue.target_url_hint" => "https://www.youtube.com/",
        "youtube_login_rescue.clipboard_prefilled" => "클립보드에서 URL을 채웠습니다.",
        "youtube_login_rescue.drop_url_note" => "URL을 붙여넣거나 .url / 텍스트 파일을 놓으세요.",
        "youtube_login_rescue.paste_clipboard" => "클립보드 붙여넣기",
        "youtube_login_rescue.cookie_note" => {
            "해당 창에서 로그인하세요. Cookie를 찾으면 창을 닫고 자동으로 적용합니다."
        }
        "youtube_login_rescue.no_browser_title" => "지원되는 브라우저를 찾을 수 없음",
        "youtube_login_rescue.no_browser_body" => {
            "Cookie를 가져오려면 현재 Chrome, Brave 또는 Microsoft Edge가 필요합니다. cookies.txt는 계속 수동으로 선택할 수 있습니다."
        }
        "youtube_login_rescue.start" => "시작",
        "youtube_login_rescue.opening" => "{browser} 여는 중...",
        "youtube_login_rescue.waiting_for_cdp" => "{browser} 로그인 창 연결 대기 중...",
        "youtube_login_rescue.waiting_for_cookie" => {
            "로그인 창이 연결되었습니다. 웹사이트 Cookie 대기 중..."
        }
        "youtube_login_rescue.cookie_exported" => "Cookie가 저장되었습니다.",
        "youtube_login_rescue.cookie_exported_note" => {
            "{site} Cookie를 저장했습니다. 해당 웹사이트의 다운로드에 자동으로 사용됩니다."
        }
        "youtube_login_rescue.do_not_close_note" => {
            "확인이 진행되는 동안 로그인 브라우저를 닫지 마세요."
        }
        "youtube_login_rescue.cdp_ready" => "로그인 창이 연결되었습니다.",
        "youtube_login_rescue.ready_next_step_note" => {
            "브라우저에서 YouTube 로그인을 완료하세요. Cookie 내보내기는 다음 단계에서 추가됩니다."
        }
        "youtube_login_rescue.close_login_window" => "로그인 창 닫기",
        "youtube_login_rescue.failed" => "Cookie 가져오기 실패",
        "youtube_login_rescue.retry" => "다시 시도",
        "advance.no_cookies_txt_selected" => "cookies.txt가 선택되지 않았습니다",
        "advance.browse" => "찾아보기",
        "advance.select_netscape_cookies_txt" => "Netscape cookies.txt 선택",
        "advance.clear" => "지우기",
        "advance.browser" => "브라우저",
        "advance.default" => "기본값",
        "advance.external_downloader" => "외부 다운로더",
        "advance.use_aria2_for_faster_downloads" => "더 빠른 다운로드에 Aria2 사용",
        "advance.download_control" => "다운로드 제어",
        "advance.concurrent_fragments" => "동시 조각 수",
        "advance.1_default" => "1 (기본값)",
        "advance.rate_limit" => "속도 제한",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => "예: 2M, 800K; 무제한이면 비워 두세요",
        "advance.chapters" => "챕터",
        "advance.chapter_download_compatibility_mode" => "챕터 다운로드 호환 모드",
        "advance.file_time" => "파일 시간",
        "advance.file_time.none" => "변경하지 않음",
        "advance.file_time.upload_date" => "동영상 업로드 날짜 사용",
        "advance.file_time.download_time" => "다운로드 시간 사용",
        "advance.post_processing" => "후처리",
        "advance.thumbnail" => "썸네일",
        "advance.download" => "다운로드",
        "advance.embed" => "삽입",
        "advance.subtitles" => "자막",
        "advance.download_conversion" => "다운로드 후 변환",
        "advance.enable" => "사용",
        "advance.settings" => "설정",
        "item.save_as" => "다른 이름으로 저장",
        "item.error" => "오류",
        "item.all" => "전체",
        "item.queued" => "대기 중",
        "item.done" => "완료",
        "item.failed" => "실패",
        "item.clear_all" => "모두 지우기",
        "item.add_a_video_url" => "동영상 URL 추가",
        "item.add_an_audio_url" => "오디오 URL 추가",
        "item.after_adding_choose_the_video_format_here" => "동영상 형식 선택",
        "item.after_adding_choose_the_audio_format_here" => "오디오 형식 선택",
        "item.loading_thumbnail" => "썸네일 로드 중",
        "item.file_actions" => "파일 작업",
        "item.open_file" => "파일 열기",
        "item.open_folder" => "폴더 열기",
        "item.copy_path" => "경로 복사",
        "item.file_not_found_opened_the_output_location" => {
            "파일을 찾을 수 없어 출력 위치를 열었습니다."
        }
        "item.opened_output_location" => "출력 위치를 열었습니다.",
        "item.copied_output_path" => "출력 경로를 복사했습니다.",
        "prepare.language" => "언어",
        "prepare.back" => "뒤로",
        "prepare.auto_detect" => "자동 감지",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "필요한 도구를 지금 설치하거나 건너뛰고 나중에 옵션에서 설정하세요."
        }
        "prepare.optional" => "선택",
        "prepare.missing" => "없음",
        "prepare.install_later" => "나중에 설치",
        "prepare.downloading_100" => "다운로드 100%",
        "prepare.extracting_100" => "압축 해제 100%",
        "prepare.install_failed" => "설치 실패",
        "prepare.install_all" => "모두 설치",
        "prepare.reinstall" => "다시 설치",
        "prepare.installing" => "설치 중",
        "prepare.skip" => "건너뛰기",
        "prepare.install" => "설치",
        "prepare.another_tool_is_already_being_installed" => "다른 도구가 이미 설치 중입니다.",
        "prepare.needs_attention" => "확인 필요",
        "prepare.req.app_folder.title" => "앱 폴더",
        "prepare.req.app_folder.description" => {
            "설정과 지원 데이터를 저장하려면 포터블 폴더에 쓰기 권한이 필요합니다."
        }
        "prepare.req.tools_folder.title" => "도구 폴더",
        "prepare.req.tools_folder.description" => {
            "의존성 배포 시 yt-dlp, FFmpeg, Deno가 여기에 저장됩니다."
        }
        "prepare.req.deployment_temp.title" => "배포 임시 폴더",
        "prepare.req.deployment_temp.description" => {
            "FFmpeg와 Deno 압축 해제에 이 임시 폴더를 사용합니다."
        }
        "prepare.req.download_cache.title" => "다운로드 캐시",
        "prepare.req.download_cache.description" => {
            "yt-dlp-gui 캐시 모드는 yt-dlp 캐시를 여기에 저장합니다."
        }
        "prepare.req.output_folder.title" => "출력 폴더",
        "prepare.req.output_folder.description" => "동영상, 오디오, 자막이 여기에 저장됩니다.",
        "prepare.req.output_folder.recommendation" => {
            "메인 화면이나 옵션에서 올바른 출력 폴더를 선택하세요."
        }
        "prepare.req.config_file.title" => "설정 파일",
        "prepare.req.config_file.description" => {
            "앱은 Prepare 건너뛰기 상태와 도구 경로 설정을 저장할 수 있어야 합니다."
        }
        "prepare.req.generic_writable_recommendation" => {
            "쓰기 가능한 폴더를 선택하고 권한을 확인하세요."
        }
        "prepare.req.config_not_folder" => "설정 경로가 폴더를 가리킵니다. 파일 경로를 선택하세요.",
        "prepare.req.config_readonly" => "설정 파일이 읽기 전용입니다.",
        "prepare.req.config_readonly_recommendation" => {
            "설정 파일에 쓰기를 허용하거나 다른 앱 폴더를 선택하세요."
        }
        "prepare.req.use_folder_path" => "파일 경로가 아니라 폴더 경로를 선택하세요.",
        "prepare.req.move_portable_folder" => "앱을 쓰기 가능한 포터블 폴더로 옮기세요.",
        "prepare.req.avoid_protected_folder" => {
            "포터블 앱을 Program Files 또는 Windows 폴더 아래에 두지 마세요. D:\\Portable 또는 사용자 폴더로 옮기세요."
        }
        "prepare.req.move_non_synced_folder" => {
            "동기화되지 않는 폴더로 옮기세요. 예: D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.drive_parent_exists" => "드라이브와 상위 폴더가 존재하는지 확인하세요.",
        "prepare.req.permission_denied" => {
            "앱을 쓰기 가능한 포터블 폴더로 옮기세요. 바탕 화면/문서/다운로드에서도 실패하면 Defender의 제어된 폴더 액세스가 차단 중일 수 있습니다."
        }
        "prepare.req.file_in_use" => {
            "이 폴더를 사용 중일 수 있는 프로그램을 닫거나 다른 폴더를 선택하세요."
        }
        "prepare.req.free_disk_space" => "디스크 공간을 확보하거나 다른 디스크를 선택하세요.",
        "prepare.req.path_too_long" => {
            "앱을 더 짧은 경로로 옮기세요. 예: D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.choose_writable_portable_folder" => {
            "명확히 쓰기 가능한 포터블 폴더를 선택한 뒤 다시 확인하세요."
        }
        "prepare.req.clear_write_test" => "남은 쓰기 테스트 파일을 삭제한 뒤 다시 확인하세요.",
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "이 URL에는 동영상과 재생목록이 모두 포함되어 있습니다"
        }
        "options.detected" => "감지됨 ",
        "options.playlist_prompt" => "재생목록 확인",
        "options.which_one_should_be_loaded" => "어느 것을 불러올까요?",
        "options.both_video_and_playlist_were_detected" => {
            "동영상과 재생목록이 모두 감지되었습니다"
        }
        "options.this_playlist_may_contain_many_items" => {
            "이 재생목록에는 많은 항목이 포함될 수 있습니다."
        }
        "options.playlist_risk.kind.channel_generated" => "YouTube에서 생성한 채널 재생목록",
        "options.playlist_risk.kind.youtube_mix_radio" => "YouTube Mix / Radio",
        "options.playlist_risk.kind.youtube_music_album" => "YouTube Music 앨범/컬렉션",
        "options.playlist_risk.kind.liked_videos" => "좋아요 표시한 동영상",
        "options.playlist_risk.kind.favorites_legacy" => "이전 즐겨찾기 재생목록",
        "options.playlist_risk.note.channel_generated" => {
            "YouTube에서 생성한 이 채널 재생목록은 보수적으로 처리하세요."
        }
        "options.playlist_risk.note.youtube_mix_radio" => {
            "이 Mix / Radio 재생목록은 항목이 많을 수 있으며 시간이 지나면 변경될 수 있습니다."
        }
        "options.playlist_risk.note.youtube_music_album" => {
            "일반적으로 YouTube Music 앨범 또는 컬렉션입니다."
        }
        "options.playlist_risk.note.liked_videos" => {
            "좋아요 표시한 동영상은 보통 로그인 또는 쿠키가 필요합니다."
        }
        "options.playlist_risk.note.favorites_legacy" => {
            "이전 즐겨찾기 재생목록 형식이며 현재 안정적이지 않을 수 있습니다."
        }
        "options.video" => "동영상",
        "options.playlist" => "재생목록",
        "options.cancel" => "취소",
        "options.load" => "불러오기",
        "options.behavior" => "동작",
        "options.add_action" => "추가 동작",
        "options.download_directly" => "바로 다운로드",
        "options.clipboard_change" => "클립보드 변경",
        "options.run_immediately" => "즉시 실행",
        "options.tabs" => "탭",
        "options.log_tab" => "로그 탭",
        "options.show_log_tab" => "로그 표시",
        "options.playlist_2" => "재생목록",
        "options.with_playlist" => "재생목록 포함",
        "options.ask" => "묻기",
        "options.single_video" => "단일 동영상",
        "options.full_playlist" => "전체 재생목록",
        "options.high_risk_prompt" => "고위험 확인",
        "options.on" => "켜짐",
        "options.playlist_count" => "재생목록 개수",
        "options.limit" => "제한",
        "options.max" => "최대:",
        "options.items" => "개 항목",
        "options.language" => "언어",
        "options.current_language" => "현재 언어",
        "options.back" => "뒤로",
        "options.choose" => "선택",
        "options.auto_detect" => "자동 감지",
        "options.tool_paths" => "도구 경로",
        "options.file_actions" => "파일 작업",
        "options.action_button" => "동작 버튼",
        "options.file_action.show_menu" => "메뉴 표시",
        "options.cache" => "캐시",
        "options.cache_location" => "캐시 위치",
        "options.cache_location.default" => "기본값",
        "options.cache_usage" => "사용량",
        "options.cache_usage_detail" => "전체: {total} · 오디오: {audio} · 만료됨: {expired}",
        "options.cache_cleanup" => "정리",
        "options.cache_refresh" => "새로 고침",
        "options.cache_clear_expired" => "만료 항목 지우기",
        "options.cache_clear_audio" => "오디오 지우기",
        "options.cache_clear_all" => "모두 지우기",
        "options.appearance_window" => "모양 및 창",
        "options.notifications" => "알림",
        "options.enable" => "사용",
        "options.theme" => "테마",
        "options.theme_mode.system" => "시스템 설정 따르기",
        "options.theme_mode.light" => "라이트",
        "options.theme_mode.dark" => "다크",
        "options.theme_color" => "테마 색상",
        "options.theme_color.off" => "꺼짐",
        "options.theme_color.blue" => "파란색",
        "options.theme_color.soft_blue" => "부드러운 파란색",
        "options.theme_color.purple" => "보라색",
        "options.theme_color.pink" => "분홍색",
        "options.theme_color.green" => "초록색",
        "options.theme_color.orange" => "주황색",
        "options.theme_color.slate" => "슬레이트",
        "options.ui_scale" => "UI 배율",
        "options.apply" => "적용",
        "options.current" => "현재",
        "options.always_on_top" => "항상 위",
        "options.window_position" => "창 위치",
        "options.remember" => "기억",
        "options.window_size" => "창 크기",
        "options.reinstall" => "다시 설치",
        "options.installing" => "설치 중",
        "options.install" => "설치",
        "options.executable" => "실행 파일",
        "main.controlled_by_config" => "구성에서 제어됨: ",
        "main.controlled_by_config_2" => "구성에서 제어됨",
        "picker.no_chapters_available" => "사용 가능한 챕터가 없습니다.",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "이 항목에서 다운로드할 범위를 선택하세요. 기본값은 전체 동영상입니다."
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "챕터 호환 모드가 켜져 있습니다. 챕터 다운로드에는 더 안정적인 단일 파일 형식이 사용됩니다."
        }
        "picker.subtitles_will_not_be_downloaded" => "자막을 다운로드하지 않습니다.",
        "picker.no_subtitles_are_available_for_this_video" => {
            "이 동영상에 사용할 수 있는 자막이 없습니다."
        }
        "picker.no_subtitles_are_available_in_this_tab" => {
            "이 탭에 사용할 수 있는 자막이 없습니다."
        }
        "picker.source_language" => "원본 언어",
        "picker.translation_target" => "번역 대상",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "팁: YouTube 자동 번역 자막은 원본 자막보다 요청 제한에 걸릴 가능성이 더 높습니다. 원문만 필요하면 “번역 없음”을 선택하세요."
        }
        "picker.no_subtitles_are_available_for_this_source" => {
            "이 원본에 사용할 수 있는 자막이 없습니다."
        }
        "picker.target" => "대상",
        "picker.available_subtitles" => "사용 가능한 자막",
        "picker.language" => "언어",
        "picker.subtitle_tab.none" => "자막 없음",
        "picker.subtitle_tab.original" => "원본 자막",
        "picker.subtitle_tab.automatic" => "자동 자막",
        "picker.waiting_analysis" => "분석 대기 중",
        "picker.audio_from_video" => "동영상 형식에 따라 결정",
        "picker.not_selected" => "선택 안 됨",
        "picker.full_video" => "전체 동영상",
        "picker.no_translation" => "번역 없음",
        "picker.until_end" => "끝",
        "prepare.status.ready" => "준비됨",
        "prepare.status.missing" => "없음",
        "prepare.status.warning" => "확인 필요",
        "prepare.status.failed" => "실패",
        "tool_install.stage.preparing" => "준비 중",
        "tool_install.stage.downloading" => "다운로드 중",
        "tool_install.stage.extracting" => "압축 해제 중",
        "tool_install.stage.installing" => "설치 중",
        "tool_install.stage.completed" => "완료",
        "tool_install.stage.failed" => "실패",
        "item.status.queued" => "대기 중",
        "item.status.running" => "실행 중",
        "item.status.finished" => "완료",
        "item.status.failed" => "실패",
        "item.status.cancelled" => "취소됨",
        "processing.transcode" => "트랜스코드",
        "transcode.graph.axis.compatibility" => "호환성",
        "transcode.graph.axis.capacity" => "용량",
        "transcode.graph.axis.resolution" => "해상도",
        "transcode.graph.axis.format" => "형식",
        "transcode.graph.compatibility_scope" => "호환성 범위",
        "transcode.graph.capacity_target" => "크기 목표",
        "transcode.graph.resolution_limit" => "해상도 제한",
        "transcode.graph.format_goal" => "형식 목표",
        "processing.video" => "동영상",
        "processing.audio" => "오디오",
        "processing.container" => "컨테이너",
        "processing.subtitle" => "자막",
        "processing.choice.source" => "원본",
        "processing.subtitle.preserve" => "원본",
        "processing.subtitle.embed" => "포함",
        "processing.subtitle.burn" => "영상에 입히기",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "모든 파일",
        "options.filter_executable" => "실행 파일",
        "app_mode.origin" => "Origin 모드",
        "app_mode.standard" => "표준 모드",
        "app_mode.audio" => "오디오 모드",
        "music.status.completed" => "완료",
        "music.status.resolving" => "확인 중",
        "music.status.buffering" => "버퍼링",
        "music.status.ready" => "준비됨",
        "music.status.caching" => "캐싱",
        "music.status.playing" => "재생 중",
        "music.status.paused" => "일시정지",
        "music.status.failed" => "실패",
        "notification.download_complete" => "다운로드 완료",
        "notification.download_failed" => "다운로드 실패",
        "notification.completed_file" => "완료됨: {file}",
        "notification.download_completed" => "다운로드가 완료되었습니다.",
        "options.music_download_format" => "Music download format",
        "options.music_download_audio_label" => "오디오 출력",
        "options.music_download_preference_best" => "최고",
        _ => key,
    }
}
