pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.advanced" => "Расширенные",
        "tab.about" => "About",
        "about.tools" => "Версии инструментов",
        "about.current_version" => "Текущая",
        "about.latest_version" => "Последняя",
        "about.author" => "Автор",
        "about.source" => "Источник",
        "about.status" => "Статус",
        "about.message" => "Сообщение",
        "about.check_updates" => "Проверить обновления",
        "about.update_all" => "Обновить всё",
        "about.restart" => "Перезапустить",
        "about.open_release" => "Открыть Release",
        "about.install" => "Установить",
        "about.update" => "Обновить",
        "about.running" => "Идет проверка обновлений...",
        "about.last_check" => "Последняя проверка:",
        "about.relative.minutes" => "{count} мин",
        "about.relative.hour" => "1 час",
        "about.relative.hours" => "{count} ч",
        "about.relative.day" => "1 день",
        "about.relative.days" => "{count} дн.",
        "about.never_checked" => "Обновления еще не проверялись",
        "about.no_release_notes_loaded" => {
            "Заметки к выпуску не загружены. Сначала проверьте обновления."
        }
        "about.ownership.managed_portable" => "Управляется v2",
        "about.ownership.external" => "Внешний",
        "about.ownership.missing" => "Отсутствует",
        "about.ownership.unknown" => "Неизвестно",
        "about.status.unknown" => "Не проверено",
        "about.status.checking" => "Проверка",
        "about.status.up_to_date" => "Актуально ✓",
        "about.status.update_available" => "Доступно обновление ↑",
        "about.status.missing" => "Отсутствует +",
        "about.status.downloading" => "Загрузка",
        "about.status.downloading_percent" => "Загрузка {percent}%",
        "about.status.staged" => "Подготовлено",
        "about.status.pending_restart" => "Ожидает перезапуска",
        "about.status.applying" => "Применение",
        "about.status.installed" => "Установлено",
        "about.status.skipped" => "Пропущено",
        "about.status.failed" => "Ошибка !",
        "tab.options" => "Параметры",
        "tab.log" => "Журнал",
        "main.url_hint" => "URL",
        "action.download" => "Скачать",
        "action.add" => "Добавить",
        "action.analyze" => "Анализировать",
        "action.stop" => "Остановить",
        "action.stopping" => "Остановка...",
        "action.cut" => "Вырезать",
        "action.copy" => "Копировать",
        "action.paste" => "Вставить",
        "action.clear" => "Очистить",
        "item.thumbnail" => "Миниатюра",
        "item.thumbnail_preview" => "Предпросмотр превью",
        "single.title" => "Название",
        "single.description" => "Описание",
        "single.info.channel" => "Канал",
        "single.info.date" => "Дата",
        "single.info.views" => "Просмотры",
        "item.download_thumbnail" => "Скачать миниатюру",
        "media.video" => "Видео",
        "media.audio" => "Аудио",
        "media.subtitle" => "Субтитры",
        "media.section" => "Диапазон",
        "item.file_name" => "Имя файла",
        "main.target_folder" => "Папка вывода",
        "picker.title.video" => "Выбрать формат видео",
        "picker.title.audio" => "Выбрать формат аудио",
        "picker.title.subtitle" => "Выбрать субтитры",
        "picker.title.section" => "Выбрать диапазон",
        "action.back" => "Назад",
        "picker.mode.filter" => "Фильтры",
        "picker.mode.table" => "Таблица",
        "action.confirm" => "Подтвердить",
        "picker.empty_table" => "Нет форматов для отображения",
        "picker.header.resolution" => "Разрешение",
        "picker.header.range" => "Диапазон",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "Формат",
        "picker.header.codec" => "Кодек",
        "picker.header.size" => "Размер",
        "picker.header.sample_rate" => "Частота дискретизации",
        "picker.filter.resolution" => "Разрешение",
        "picker.filter.range" => "Диапазон",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "Кодек",
        "picker.filter.sample_rate" => "Частота дискретизации",
        "main.missing_yt_dlp_callout" => {
            "yt-dlp отсутствует. Установите его или выберите yt-dlp.exe в настройках."
        }
        "advance.source" => "Источник",
        "advance.config" => "Конфигурация",
        "advance.none" => "Нет",
        "advance.network_access" => "Сеть и доступ",
        "advance.proxy" => "Прокси",
        "advance.enable_proxy" => "Включить прокси",
        "advance.certificate" => "Сертификат",
        "advance.skip_certificate_verification" => "Пропустить проверку сертификата",
        "advance.use_cookies" => "Использовать cookies",
        "advance.enable_cookies" => "Включить cookies",
        "advance.cookie_source" => "Источник cookies",
        "advance.cookie_source.auto" => "Автоматически по сайту",
        "advance.cookie_source.file" => "Использовать файл (cookies.txt)",
        "advance.cookie_auto" => "Автоматически",
        "advance.cookie_auto_note" => {
            "Загрузки используют сохраненный Cookie, соответствующий URL."
        }
        "advance.cookie_rescue" => "Получение Cookie",
        "advance.cookie_file" => "Файл cookies",
        "advance.get_cookie" => "Получить Cookie",
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
            "Открыть отдельное окно браузера для получения cookies."
        }
        "youtube_login_rescue.title" => "Получение Cookie",
        "youtube_login_rescue.confirm_heading" => "Открыть отдельное окно входа",
        "youtube_login_rescue.confirm_body" => {
            "Отдельное окно {browser} откроет URL без использования личных данных браузера."
        }
        "youtube_login_rescue.target_url_label" => "URL сайта",
        "youtube_login_rescue.target_url_hint" => "https://www.youtube.com/",
        "youtube_login_rescue.clipboard_prefilled" => "URL был вставлен из буфера обмена.",
        "youtube_login_rescue.drop_url_note" => {
            "Вставьте URL или перетащите файл .url / текстовый файл."
        }
        "youtube_login_rescue.paste_clipboard" => "Вставить из буфера",
        "youtube_login_rescue.cookie_note" => {
            "Войдите в систему в этом окне. Когда cookies будут найдены, окно закроется и они применятся автоматически."
        }
        "youtube_login_rescue.no_browser_title" => "Поддерживаемый браузер не найден",
        "youtube_login_rescue.no_browser_body" => {
            "Для получения cookies сейчас нужен Chrome, Brave или Microsoft Edge. Вы также можете выбрать cookies.txt вручную."
        }
        "youtube_login_rescue.start" => "Начать",
        "youtube_login_rescue.opening" => "Открытие {browser}...",
        "youtube_login_rescue.waiting_for_cdp" => "Ожидание подключения окна входа {browser}...",
        "youtube_login_rescue.waiting_for_cookie" => {
            "Окно входа подключено. Ожидание cookies сайта..."
        }
        "youtube_login_rescue.cookie_exported" => "Cookie сохранен.",
        "youtube_login_rescue.cookie_exported_note" => {
            "Cookie {site} сохранен. Загрузки с этого сайта будут использовать его автоматически."
        }
        "youtube_login_rescue.do_not_close_note" => {
            "Не закрывайте браузер входа, пока выполняется эта проверка."
        }
        "youtube_login_rescue.cdp_ready" => "Окно входа подключено.",
        "youtube_login_rescue.ready_next_step_note" => {
            "Завершите вход в YouTube в браузере. Экспорт Cookie будет добавлен на следующем шаге."
        }
        "youtube_login_rescue.close_login_window" => "Закрыть окно входа",
        "youtube_login_rescue.failed" => "Не удалось получить Cookie",
        "youtube_login_rescue.retry" => "Повторить",
        "advance.no_cookies_txt_selected" => "cookies.txt не выбран",
        "advance.browse" => "Обзор",
        "advance.select_netscape_cookies_txt" => "Выбрать Netscape cookies.txt",
        "advance.clear" => "Очистить",
        "advance.browser" => "Браузер",
        "advance.default" => "По умолчанию",
        "advance.external_downloader" => "Внешний загрузчик",
        "advance.use_aria2_for_faster_downloads" => "Использовать Aria2 для более быстрых загрузок",
        "advance.download_control" => "Управление загрузкой",
        "advance.concurrent_fragments" => "Одновременные фрагменты",
        "advance.live_streams" => "Прямые трансляции",
        "advance.download_live_streams_from_start_experimental" => {
            "Скачивать прямые трансляции с начала (экспериментально)"
        }
        "advance.1_default" => "1 (по умолчанию)",
        "advance.rate_limit" => "Ограничение скорости",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => {
            "например 2M, 800K; оставьте пустым без ограничения"
        }
        "advance.chapters" => "Главы",
        "advance.download_range" => "Диапазон загрузки",
        "advance.always_show_download_range" => "Всегда показывать выбор диапазона",
        "advance.chapter_download_compatibility_mode" => "Режим совместимости загрузки глав",
        "advance.file_time" => "Время файла",
        "advance.file_time.none" => "Не изменять",
        "advance.file_time.upload_date" => "Использовать дату загрузки видео",
        "advance.file_time.download_time" => "Использовать время скачивания",
        "advance.post_processing" => "Постобработка",
        "advance.thumbnail" => "Миниатюра",
        "advance.download" => "Скачать",
        "advance.embed" => "Встроить",
        "advance.subtitles" => "Субтитры",
        "advance.download_conversion" => "Конвертировать после скачивания",
        "advance.enable" => "Включить",
        "advance.settings" => "Настройки",
        "item.save_as" => "Сохранить как",
        "item.error" => "Ошибка",
        "item.all" => "Все",
        "item.queued" => "В очереди",
        "item.done" => "Готово",
        "item.failed" => "Ошибка",
        "item.clear_all" => "Очистить всё",
        "item.add_a_video_url" => "Добавить URL видео",
        "item.add_an_audio_url" => "Добавить URL аудио",
        "item.after_adding_choose_the_video_format_here" => "Выбрать формат видео",
        "item.after_adding_choose_the_audio_format_here" => "Выбрать формат аудио",
        "item.loading_thumbnail" => "Загрузка миниатюры",
        "item.file_actions" => "Действия с файлом",
        "item.open_file" => "Открыть файл",
        "item.open_folder" => "Открыть папку",
        "item.copy_path" => "Копировать путь",
        "item.file_not_found_opened_the_output_location" => {
            "Файл не найден; открыто расположение вывода."
        }
        "item.opened_output_location" => "Расположение вывода открыто.",
        "item.copied_output_path" => "Путь вывода скопирован.",
        "prepare.language" => "Язык",
        "prepare.back" => "Назад",
        "prepare.auto_detect" => "Автоопределение",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Установите необходимые инструменты сейчас или пропустите и настройте их позже в параметрах."
        }
        "prepare.optional" => "Необязательно",
        "prepare.missing" => "Отсутствует",
        "prepare.install_later" => "Установить позже",
        "prepare.downloading_100" => "Загрузка 100%",
        "prepare.extracting_100" => "Распаковка 100%",
        "prepare.install_failed" => "Установка не удалась",
        "prepare.install_all" => "Установить всё",
        "prepare.reinstall" => "Переустановить",
        "prepare.installing" => "Установка",
        "prepare.skip" => "Пропустить",
        "prepare.install" => "Установить",
        "prepare.another_tool_is_already_being_installed" => {
            "Другой инструмент уже устанавливается."
        }
        "prepare.needs_attention" => "Требует внимания",
        "prepare.req.app_folder.title" => "Папка приложения",
        "prepare.req.app_folder.description" => {
            "Папка портативной версии должна быть доступна для записи настроек и служебных данных."
        }
        "prepare.req.tools_folder.title" => "Папка инструментов",
        "prepare.req.tools_folder.description" => {
            "Развёртывание зависимостей сохраняет здесь yt-dlp, FFmpeg и Deno."
        }
        "prepare.req.deployment_temp.title" => "Временная папка развёртывания",
        "prepare.req.deployment_temp.description" => {
            "Для распаковки FFmpeg и Deno используется эта временная папка."
        }
        "prepare.req.download_cache.title" => "Кэш загрузок",
        "prepare.req.download_cache.description" => {
            "Режим кэша yt-dlp-gui сохраняет здесь кэш yt-dlp."
        }
        "prepare.req.output_folder.title" => "Папка вывода",
        "prepare.req.output_folder.description" => "Видео, аудио и субтитры сохраняются здесь.",
        "prepare.req.output_folder.recommendation" => {
            "Выберите допустимую папку вывода на главном экране или в параметрах."
        }
        "prepare.req.config_file.title" => "Файл настроек",
        "prepare.req.config_file.description" => {
            "Приложение должно сохранять состояние пропуска Prepare и пути к инструментам."
        }
        "prepare.req.generic_writable_recommendation" => {
            "Выберите папку с правом записи и проверьте разрешения."
        }
        "prepare.req.config_not_folder" => {
            "Путь настроек указывает на папку. Выберите путь к файлу."
        }
        "prepare.req.config_readonly" => "Файл настроек доступен только для чтения.",
        "prepare.req.config_readonly_recommendation" => {
            "Разрешите запись в файл настроек или выберите другую папку приложения."
        }
        "prepare.req.use_folder_path" => "Выберите путь к папке, а не к файлу.",
        "prepare.req.move_portable_folder" => {
            "Переместите приложение в портативную папку с правом записи."
        }
        "prepare.req.avoid_protected_folder" => {
            "Не размещайте портативное приложение в Program Files или папке Windows. Переместите его в D:\\Portable или папку пользователя."
        }
        "prepare.req.move_non_synced_folder" => {
            "Переместите его в несинхронизируемую папку, например D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.drive_parent_exists" => "Убедитесь, что диск и родительская папка существуют.",
        "prepare.req.permission_denied" => {
            "Переместите приложение в портативную папку с правом записи. Если Рабочий стол/Документы/Загрузки всё равно не работают, возможно, блокирует контролируемый доступ к папкам Defender."
        }
        "prepare.req.file_in_use" => {
            "Закройте программу, которая может использовать эту папку, или выберите другую папку."
        }
        "prepare.req.free_disk_space" => "Освободите место на диске или выберите другой диск.",
        "prepare.req.path_too_long" => {
            "Переместите приложение в более короткий путь, например D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.choose_writable_portable_folder" => {
            "Выберите явно доступную для записи портативную папку и проверьте снова."
        }
        "prepare.req.clear_write_test" => {
            "Удалите оставшийся файл проверки записи и проверьте снова."
        }
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "Этот URL содержит и видео, и плейлист"
        }
        "options.detected" => "Обнаружено ",
        "options.playlist_prompt" => "Запрос плейлиста",
        "options.which_one_should_be_loaded" => "Что загрузить?",
        "options.both_video_and_playlist_were_detected" => "Обнаружены видео и плейлист",
        "options.this_playlist_may_contain_many_items" => {
            "Этот плейлист может содержать много элементов."
        }
        "options.playlist_risk.kind.channel_generated" => "Плейлист канала, созданный YouTube",
        "options.playlist_risk.kind.youtube_mix_radio" => "YouTube Mix / Radio",
        "options.playlist_risk.kind.youtube_music_album" => "Альбом/коллекция YouTube Music",
        "options.playlist_risk.kind.liked_videos" => "Понравившиеся видео",
        "options.playlist_risk.kind.favorites_legacy" => "Старый плейлист избранного",
        "options.playlist_risk.note.channel_generated" => {
            "Обрабатывайте этот плейлист канала, созданный YouTube, осторожно."
        }
        "options.playlist_risk.note.youtube_mix_radio" => {
            "Этот плейлист Mix / Radio может содержать много элементов и меняться со временем."
        }
        "options.playlist_risk.note.youtube_music_album" => {
            "Обычно это альбом или коллекция YouTube Music."
        }
        "options.playlist_risk.note.liked_videos" => {
            "Понравившиеся видео обычно требуют входа или cookies."
        }
        "options.playlist_risk.note.favorites_legacy" => {
            "Это старый тип плейлиста избранного, сейчас он может быть нестабильным."
        }
        "options.video" => "Видео",
        "options.playlist" => "Плейлист",
        "options.cancel" => "Отмена",
        "options.load" => "Загрузить",
        "options.behavior" => "Поведение",
        "options.add_action" => "Действие добавления",
        "options.download_directly" => "Скачать напрямую",
        "options.clipboard_change" => "Изменение буфера обмена",
        "options.run_immediately" => "Запускать сразу",
        "options.tabs" => "Вкладки",
        "options.log_tab" => "Вкладка журнала",
        "options.show_log_tab" => "Показать журнал",
        "options.playlist_2" => "Плейлист",
        "options.with_playlist" => "С плейлистом",
        "options.ask" => "Спросить",
        "options.single_video" => "Видео",
        "options.full_playlist" => "[Все]",
        "options.high_risk_prompt" => "Предупреждение высокого риска",
        "options.on" => "Вкл.",
        "options.playlist_count" => "Количество в плейлисте",
        "options.limit" => "Ограничение",
        "options.max" => "Макс.:",
        "options.items" => " элементов",
        "options.language" => "Язык",
        "options.current_language" => "Текущий язык",
        "options.back" => "Назад",
        "options.choose" => "Выбрать",
        "options.auto_detect" => "Автоопределение",
        "options.tool_paths" => "Пути инструментов",
        "options.file_actions" => "Действия с файлом",
        "options.action_button" => "Кнопка действия",
        "options.file_action.show_menu" => "Показать меню",
        "options.cache" => "Кэш",
        "options.cache_location" => "Расположение кэша",
        "options.cache_location.default" => "По умолчанию",
        "options.cache_usage" => "Использование",
        "options.cache_usage_detail" => "Всего: {total} · Аудио: {audio} · Истекшие: {expired}",
        "options.cache_cleanup" => "Очистка",
        "options.cache_refresh" => "Обновить",
        "options.cache_clear_expired" => "Очистить устаревшие",
        "options.cache_clear_audio" => "Очистить аудио",
        "options.cache_clear_all" => "Очистить всё",
        "options.appearance_window" => "Внешний вид и окно",
        "options.notifications" => "Уведомления",
        "options.enable" => "Включить",
        "options.theme" => "Тема",
        "options.theme_mode.system" => "Как в системе",
        "options.theme_mode.light" => "Светлая",
        "options.theme_mode.dark" => "Тёмная",
        "options.theme_color" => "Цвет темы",
        "options.theme_color.off" => "Выкл.",
        "options.theme_color.blue" => "Синий",
        "options.theme_color.soft_blue" => "Мягкий синий",
        "options.theme_color.purple" => "Фиолетовый",
        "options.theme_color.pink" => "Розовый",
        "options.theme_color.green" => "Зелёный",
        "options.theme_color.orange" => "Оранжевый",
        "options.theme_color.slate" => "Сланцевый",
        "options.ui_scale" => "Масштаб интерфейса",
        "options.apply" => "Применить",
        "options.current" => "Текущий",
        "options.always_on_top" => "Всегда сверху",
        "options.window_position" => "Положение окна",
        "options.remember" => "Запомнить",
        "options.window_size" => "Размер окна",
        "options.reinstall" => "Переустановить",
        "options.installing" => "Установка",
        "options.install" => "Установить",
        "options.executable" => "исполняемый файл",
        "main.controlled_by_config" => "Задано конфигурацией: ",
        "main.controlled_by_config_2" => "Задано конфигурацией",
        "picker.section_tab.chapters" => "Главы",
        "picker.section_tab.time_range" => "Диапазон времени",
        "picker.section_chapter_instructions" => {
            "Выберите одну или несколько глав. Смежные главы образуют один выходной файл."
        }
        "picker.section_time_instructions" => {
            "Переместите позицию, задайте начало и конец, затем добавьте диапазон."
        }
        "picker.section_time_unavailable" => {
            "Длительность видео недоступна, поэтому создать свой диапазон нельзя."
        }
        "picker.section_select_all" => "Выбрать все",
        "picker.section_from_selected_to_end" => "От первой выбранной до конца",
        "picker.section_set_start" => "Задать начало",
        "picker.section_set_end" => "Задать конец",
        "picker.section_add_range" => "Добавить диапазон",
        "picker.section_no_custom_ranges" => "Пользовательские диапазоны времени не добавлены.",
        "picker.no_chapters_available" => "Нет доступных глав.",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "Выберите диапазон загрузки для этого элемента. По умолчанию загружается всё видео."
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "Режим совместимости глав включён: при выборе главы будет использоваться более стабильный единый формат файла."
        }
        "picker.subtitles_will_not_be_downloaded" => "Субтитры не будут скачаны.",
        "picker.no_subtitles_are_available_for_this_video" => "Для этого видео нет субтитров.",
        "picker.no_subtitles_are_available_in_this_tab" => "В этой вкладке нет субтитров.",
        "picker.source_language" => "Исходный язык",
        "picker.translation_target" => "Цель перевода",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "Совет: автоматически переведённые субтитры YouTube чаще попадают под ограничения, чем оригинальные субтитры. Выберите «Без перевода», если нужен только исходный текст."
        }
        "picker.no_subtitles_are_available_for_this_source" => "Для этого источника нет субтитров.",
        "picker.target" => "Цель",
        "picker.available_subtitles" => "Доступные субтитры",
        "picker.language" => "Язык",
        "picker.subtitle_tab.none" => "Без субтитров",
        "picker.subtitle_tab.original" => "Оригинальные субтитры",
        "picker.subtitle_tab.automatic" => "Автоматические субтитры",
        "picker.waiting_analysis" => "Ожидание анализа",
        "picker.audio_from_video" => "Определяется форматом видео",
        "picker.not_selected" => "Не выбрано",
        "picker.full_video" => "Полное видео",
        "picker.section_summary.chapters" => "Выбрано глав: {chapters} · файлов: {outputs}",
        "picker.section_summary.custom" => "Диапазонов: {ranges} · файлов: {outputs}",
        "picker.section_summary.combined" => {
            "Глав: {chapters} + диапазонов: {ranges} · файлов: {outputs}"
        }
        "picker.no_translation" => "Без перевода",
        "picker.until_end" => "конец",
        "prepare.status.ready" => "Готово",
        "prepare.status.missing" => "Отсутствует",
        "prepare.status.warning" => "Требует внимания",
        "prepare.status.failed" => "Ошибка",
        "tool_install.stage.preparing" => "Подготовка",
        "tool_install.stage.downloading" => "Загрузка",
        "tool_install.stage.extracting" => "Распаковка",
        "tool_install.stage.installing" => "Установка",
        "tool_install.stage.completed" => "Завершено",
        "tool_install.stage.failed" => "Ошибка",
        "item.status.queued" => "В очереди",
        "item.status.running" => "Выполняется",
        "item.status.finished" => "Готово",
        "item.status.failed" => "Ошибка",
        "item.status.cancelled" => "Отменено",
        "processing.transcode" => "Транскодирование",
        "transcode.graph.axis.compatibility" => "Совместимость",
        "transcode.graph.axis.capacity" => "Ёмкость",
        "transcode.graph.axis.resolution" => "Разрешение",
        "transcode.graph.axis.format" => "Формат",
        "transcode.graph.compatibility_scope" => "Область совместимости",
        "transcode.graph.capacity_target" => "Целевой размер",
        "transcode.graph.resolution_limit" => "Ограничение разрешения",
        "transcode.graph.format_goal" => "Цель формата",
        "processing.video" => "Видео",
        "processing.audio" => "Аудио",
        "processing.container" => "Контейнер",
        "processing.subtitle" => "Субтитры",
        "processing.choice.source" => "Оригинал",
        "processing.subtitle.preserve" => "Оригинал",
        "processing.subtitle.embed" => "Встроить",
        "processing.subtitle.burn" => "Вшить в видео",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "Все файлы",
        "options.filter_executable" => "Исполняемый файл",
        "app_mode.origin" => "Режим Origin",
        "app_mode.standard" => "Стандартный режим",
        "app_mode.audio" => "Аудиорежим",
        "music.status.completed" => "Готово",
        "music.status.resolving" => "Разрешение",
        "music.status.buffering" => "Буферизация",
        "music.status.ready" => "Готово",
        "music.status.caching" => "Кэширование",
        "music.status.playing" => "Воспроизведение",
        "music.status.paused" => "Пауза",
        "music.status.failed" => "Ошибка",
        "notification.download_complete" => "Загрузка завершена",
        "notification.download_failed" => "Ошибка загрузки",
        "notification.completed_file" => "Завершено: {file}",
        "notification.download_completed" => "Загрузка завершена.",
        "options.music_download_format" => "Music download format",
        "options.music_download_audio_label" => "Вывод аудио",
        "options.music_download_preference_best" => "Лучшее",
        _ => key,
    }
}
