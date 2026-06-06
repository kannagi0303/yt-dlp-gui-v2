pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.advanced" => "Zaawansowane",
        "tab.about" => "About",
        "about.tools" => "Wersje narzędzi",
        "about.current_version" => "Bieżąca",
        "about.latest_version" => "Najnowsza",
        "about.author" => "Autor",
        "about.source" => "Źródło",
        "about.status" => "Stan",
        "about.message" => "Komunikat",
        "about.check_updates" => "Sprawdź aktualizacje",
        "about.update_all" => "Aktualizuj wszystko",
        "about.restart" => "Uruchom ponownie",
        "about.open_release" => "Otwórz Release",
        "about.install" => "Zainstaluj",
        "about.update" => "Aktualizuj",
        "about.running" => "Trwa sprawdzanie aktualizacji...",
        "about.last_check" => "Ostatnie sprawdzenie:",
        "about.relative.minutes" => "{count} min",
        "about.relative.hour" => "1 godz.",
        "about.relative.hours" => "{count} godz.",
        "about.relative.day" => "1 dzień",
        "about.relative.days" => "{count} dni",
        "about.never_checked" => "Aktualizacje nie były jeszcze sprawdzane",
        "about.no_release_notes_loaded" => {
            "Nie wczytano notatek wydania. Najpierw sprawdź aktualizacje."
        }
        "about.ownership.managed_portable" => "Zarządzane przez v2",
        "about.ownership.external" => "Zewnętrzne",
        "about.ownership.missing" => "Brak",
        "about.ownership.unknown" => "Nieznane",
        "about.status.unknown" => "Nie sprawdzono",
        "about.status.checking" => "Sprawdzanie",
        "about.status.up_to_date" => "Aktualne ✓",
        "about.status.update_available" => "Dostępna aktualizacja ↑",
        "about.status.missing" => "Brak +",
        "about.status.downloading" => "Pobieranie",
        "about.status.downloading_percent" => "Pobieranie {percent}%",
        "about.status.staged" => "Przygotowane",
        "about.status.pending_restart" => "Oczekuje na restart",
        "about.status.applying" => "Stosowanie",
        "about.status.installed" => "Zainstalowane",
        "about.status.skipped" => "Pominięte",
        "about.status.failed" => "Niepowodzenie !",
        "tab.options" => "Opcje",
        "tab.log" => "Dziennik",
        "main.url_hint" => "URL",
        "action.download" => "Pobierz",
        "action.add" => "Dodaj",
        "action.analyze" => "Analizuj",
        "action.stop" => "Zatrzymaj",
        "action.stopping" => "Zatrzymywanie...",
        "action.cut" => "Wytnij",
        "action.copy" => "Kopiuj",
        "action.paste" => "Wklej",
        "action.clear" => "Wyczyść",
        "item.thumbnail" => "Miniatura",
        "item.thumbnail_preview" => "Podgląd miniatury",
        "single.title" => "Tytuł",
        "single.description" => "Opis",
        "single.info.channel" => "Kanał",
        "single.info.date" => "Data",
        "single.info.views" => "Wyświetlenia",
        "item.download_thumbnail" => "Pobierz miniaturę",
        "media.video" => "Wideo",
        "media.audio" => "Audio",
        "media.subtitle" => "Napisy",
        "media.section" => "Zakres",
        "item.file_name" => "Nazwa pliku",
        "main.target_folder" => "Folder wyjściowy",
        "picker.title.video" => "Wybierz format wideo",
        "picker.title.audio" => "Wybierz format audio",
        "picker.title.subtitle" => "Wybierz napisy",
        "picker.title.section" => "Wybierz zakres",
        "action.back" => "Wstecz",
        "picker.mode.filter" => "Filtry",
        "picker.mode.table" => "Tabela",
        "action.confirm" => "Potwierdź",
        "picker.empty_table" => "Brak formatów do wyświetlenia",
        "picker.header.resolution" => "Rozdzielczość",
        "picker.header.range" => "Zakres",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "Format",
        "picker.header.codec" => "Kodek",
        "picker.header.size" => "Rozmiar",
        "picker.header.sample_rate" => "Częstotliwość próbkowania",
        "picker.filter.resolution" => "Rozdzielczość",
        "picker.filter.range" => "Zakres",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "Kodek",
        "picker.filter.sample_rate" => "Częstotliwość próbkowania",
        "main.missing_yt_dlp_callout" => {
            "Brakuje yt-dlp. Zainstaluj go albo wybierz yt-dlp.exe w Opcjach."
        }
        "advance.source" => "Źródło",
        "advance.config" => "Konfiguracja",
        "advance.none" => "Brak",
        "advance.network_access" => "Sieć i dostęp",
        "advance.proxy" => "Proxy",
        "advance.enable_proxy" => "Włącz proxy",
        "advance.certificate" => "Certyfikat",
        "advance.skip_certificate_verification" => "Pomiń weryfikację certyfikatu",
        "advance.use_cookies" => "Użyj cookies",
        "advance.enable_cookies" => "Włącz cookies",
        "advance.cookie_source" => "Źródło cookies",
        "advance.cookie_source.auto" => "Automatycznie według witryny",
        "advance.cookie_source.file" => "Użyj pliku (cookies.txt)",
        "advance.cookie_auto" => "Automatycznie",
        "advance.cookie_auto_note" => "Pobrania używają zapisanego Cookie pasującego do URL.",
        "advance.cookie_rescue" => "Odzyskiwanie Cookie",
        "advance.cookie_file" => "Plik cookie",
        "advance.get_cookie" => "Pobierz Cookie",
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
            "Otwórz dedykowane okno przeglądarki, aby pobrać ciasteczka."
        }
        "youtube_login_rescue.title" => "Odzyskiwanie Cookie",
        "youtube_login_rescue.confirm_heading" => "Otwórz dedykowane okno logowania",
        "youtube_login_rescue.confirm_body" => {
            "Niezależne okno {browser} otworzy URL bez używania prywatnych danych przeglądarki."
        }
        "youtube_login_rescue.target_url_label" => "URL witryny",
        "youtube_login_rescue.target_url_hint" => "https://www.youtube.com/",
        "youtube_login_rescue.clipboard_prefilled" => "URL został wstawiony ze schowka.",
        "youtube_login_rescue.drop_url_note" => "Wklej URL albo przeciągnij plik .url / tekstowy.",
        "youtube_login_rescue.paste_clipboard" => "Wklej schowek",
        "youtube_login_rescue.cookie_note" => {
            "Zaloguj się w tym oknie. Po znalezieniu ciasteczek okno zostanie zamknięte i zostaną one zastosowane automatycznie."
        }
        "youtube_login_rescue.no_browser_title" => "Nie znaleziono obsługiwanej przeglądarki",
        "youtube_login_rescue.no_browser_body" => {
            "Pobieranie ciasteczek wymaga obecnie Chrome, Brave albo Microsoft Edge. Nadal możesz ręcznie wybrać cookies.txt."
        }
        "youtube_login_rescue.start" => "Start",
        "youtube_login_rescue.opening" => "Otwieranie {browser}...",
        "youtube_login_rescue.waiting_for_cdp" => {
            "Oczekiwanie na połączenie okna logowania {browser}..."
        }
        "youtube_login_rescue.waiting_for_cookie" => {
            "Okno logowania jest połączone. Oczekiwanie na ciasteczka witryny..."
        }
        "youtube_login_rescue.cookie_exported" => "Cookie zostało zapisane.",
        "youtube_login_rescue.cookie_exported_note" => {
            "Zapisano Cookie {site}. Pobrania z tej witryny użyją go automatycznie."
        }
        "youtube_login_rescue.do_not_close_note" => {
            "Pozostaw przeglądarkę logowania otwartą podczas tej kontroli."
        }
        "youtube_login_rescue.cdp_ready" => "Okno logowania jest połączone.",
        "youtube_login_rescue.ready_next_step_note" => {
            "Dokończ logowanie do YouTube w przeglądarce. Eksport Cookie zostanie dodany w następnym kroku."
        }
        "youtube_login_rescue.close_login_window" => "Zamknij okno logowania",
        "youtube_login_rescue.failed" => "Odzyskiwanie Cookie nie powiodło się",
        "youtube_login_rescue.retry" => "Ponów",
        "advance.no_cookies_txt_selected" => "Nie wybrano cookies.txt",
        "advance.browse" => "Przeglądaj",
        "advance.select_netscape_cookies_txt" => "Wybierz cookies.txt Netscape",
        "advance.clear" => "Wyczyść",
        "advance.browser" => "Przeglądarka",
        "advance.default" => "Domyślne",
        "advance.external_downloader" => "Zewnętrzny program pobierający",
        "advance.use_aria2_for_faster_downloads" => "Użyj Aria2 dla szybszych pobrań",
        "advance.download_control" => "Kontrola pobierania",
        "advance.concurrent_fragments" => "Równoczesne fragmenty",
        "advance.1_default" => "1 (domyślnie)",
        "advance.rate_limit" => "Limit prędkości",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => "np. 2M, 800K; zostaw puste bez limitu",
        "advance.chapters" => "Rozdziały",
        "advance.chapter_download_compatibility_mode" => "Tryb zgodności pobierania rozdziałów",
        "advance.file_time" => "Czas pliku",
        "advance.file_time.none" => "Nie zmieniaj",
        "advance.file_time.upload_date" => "Użyj daty przesłania wideo",
        "advance.file_time.download_time" => "Użyj czasu pobrania",
        "advance.post_processing" => "Przetwarzanie końcowe",
        "advance.thumbnail" => "Miniatura",
        "advance.download" => "Pobierz",
        "advance.embed" => "Osadź",
        "advance.subtitles" => "Napisy",
        "advance.download_conversion" => "Konwertuj po pobraniu",
        "advance.enable" => "Włącz",
        "advance.settings" => "Ustawienia",
        "item.save_as" => "Zapisz jako",
        "item.error" => "Błąd",
        "item.all" => "Wszystkie",
        "item.queued" => "W kolejce",
        "item.done" => "Gotowe",
        "item.failed" => "Niepowodzenie",
        "item.clear_all" => "Wyczyść wszystko",
        "item.add_a_video_url" => "Dodaj URL wideo",
        "item.add_an_audio_url" => "Dodaj URL audio",
        "item.after_adding_choose_the_video_format_here" => "Wybierz format wideo",
        "item.after_adding_choose_the_audio_format_here" => "Wybierz format audio",
        "item.loading_thumbnail" => "Ładowanie miniatury",
        "item.file_actions" => "Akcje pliku",
        "item.open_file" => "Otwórz plik",
        "item.open_folder" => "Otwórz folder",
        "item.copy_path" => "Kopiuj ścieżkę",
        "item.file_not_found_opened_the_output_location" => {
            "Nie znaleziono pliku; otwarto lokalizację wyjściową."
        }
        "item.opened_output_location" => "Otwarto lokalizację wyjściową.",
        "item.copied_output_path" => "Skopiowano ścieżkę wyjściową.",
        "prepare.language" => "Język",
        "prepare.back" => "Wstecz",
        "prepare.auto_detect" => "Wykryj automatycznie",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Zainstaluj teraz wymagane narzędzia albo pomiń i skonfiguruj je później w Opcjach."
        }
        "prepare.optional" => "Opcjonalne",
        "prepare.missing" => "Brak",
        "prepare.install_later" => "Zainstaluj później",
        "prepare.downloading_100" => "Pobieranie 100%",
        "prepare.extracting_100" => "Wypakowywanie 100%",
        "prepare.install_failed" => "Instalacja nie powiodła się",
        "prepare.install_all" => "Zainstaluj wszystko",
        "prepare.reinstall" => "Zainstaluj ponownie",
        "prepare.installing" => "Instalowanie",
        "prepare.skip" => "Pomiń",
        "prepare.install" => "Zainstaluj",
        "prepare.another_tool_is_already_being_installed" => "Inne narzędzie jest już instalowane.",
        "prepare.needs_attention" => "Wymaga uwagi",
        "prepare.req.app_folder.title" => "Folder aplikacji",
        "prepare.req.app_folder.description" => {
            "Folder przenośny musi być zapisywalny, aby zapisywać ustawienia i dane pomocnicze."
        }
        "prepare.req.tools_folder.title" => "Folder narzędzi",
        "prepare.req.tools_folder.description" => {
            "Wdrażanie zależności zapisuje tutaj yt-dlp, FFmpeg i Deno."
        }
        "prepare.req.deployment_temp.title" => "Tymczasowy folder wdrażania",
        "prepare.req.deployment_temp.description" => {
            "Rozpakowywanie FFmpeg i Deno używa tego folderu tymczasowego."
        }
        "prepare.req.download_cache.title" => "Pamięć podręczna pobierania",
        "prepare.req.download_cache.description" => {
            "Tryb pamięci podręcznej yt-dlp-gui zapisuje tutaj cache yt-dlp."
        }
        "prepare.req.output_folder.title" => "Folder wyjściowy",
        "prepare.req.output_folder.description" => "Filmy, dźwięk i napisy są zapisywane tutaj.",
        "prepare.req.output_folder.recommendation" => {
            "Wybierz prawidłowy folder wyjściowy na ekranie głównym lub w Opcjach."
        }
        "prepare.req.config_file.title" => "Plik konfiguracji",
        "prepare.req.config_file.description" => {
            "Aplikacja musi móc zapisać stan pominięcia Prepare oraz ścieżki narzędzi."
        }
        "prepare.req.generic_writable_recommendation" => {
            "Wybierz folder z możliwością zapisu i sprawdź uprawnienia."
        }
        "prepare.req.config_not_folder" => {
            "Ścieżka konfiguracji wskazuje folder. Wybierz ścieżkę do pliku."
        }
        "prepare.req.config_readonly" => "Plik konfiguracji jest tylko do odczytu.",
        "prepare.req.config_readonly_recommendation" => {
            "Zezwól na zapis do pliku konfiguracji albo wybierz inny folder aplikacji."
        }
        "prepare.req.use_folder_path" => "Wybierz ścieżkę folderu zamiast ścieżki pliku.",
        "prepare.req.move_portable_folder" => {
            "Przenieś aplikację do zapisywalnego folderu przenośnego."
        }
        "prepare.req.avoid_protected_folder" => {
            "Nie umieszczaj aplikacji przenośnej w Program Files ani w folderze Windows. Przenieś ją do D:\\Portable lub folderu użytkownika."
        }
        "prepare.req.move_non_synced_folder" => {
            "Przenieś ją do folderu niesynchronizowanego, np. D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.drive_parent_exists" => "Upewnij się, że dysk i folder nadrzędny istnieją.",
        "prepare.req.permission_denied" => {
            "Przenieś aplikację do zapisywalnego folderu przenośnego. Jeśli Pulpit/Dokumenty/Pobrane nadal zawodzą, może blokować ją Kontrolowany dostęp do folderów w Defenderze."
        }
        "prepare.req.file_in_use" => {
            "Zamknij program, który może używać tego folderu, albo wybierz inny folder."
        }
        "prepare.req.free_disk_space" => "Zwolnij miejsce na dysku albo wybierz inny dysk.",
        "prepare.req.path_too_long" => {
            "Przenieś aplikację do krótszej ścieżki, np. D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.choose_writable_portable_folder" => {
            "Wybierz wyraźnie zapisywalny folder przenośny i sprawdź ponownie."
        }
        "prepare.req.clear_write_test" => "Usuń pozostały plik testu zapisu i sprawdź ponownie.",
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "Ten URL zawiera zarówno wideo, jak i playlistę"
        }
        "options.detected" => "Wykryto ",
        "options.playlist_prompt" => "Pytanie o playlistę",
        "options.which_one_should_be_loaded" => "Co załadować?",
        "options.both_video_and_playlist_were_detected" => "Wykryto wideo i playlistę",
        "options.this_playlist_may_contain_many_items" => {
            "Ta playlista może zawierać wiele elementów."
        }
        "options.playlist_risk.kind.channel_generated" => {
            "Playlista kanału wygenerowana przez YouTube"
        }
        "options.playlist_risk.kind.youtube_mix_radio" => "YouTube Mix / Radio",
        "options.playlist_risk.kind.youtube_music_album" => "Album/kolekcja YouTube Music",
        "options.playlist_risk.kind.liked_videos" => "Polubione filmy",
        "options.playlist_risk.kind.favorites_legacy" => "Stara playlista ulubionych",
        "options.playlist_risk.note.channel_generated" => {
            "Traktuj tę playlistę kanału wygenerowaną przez YouTube ostrożnie."
        }
        "options.playlist_risk.note.youtube_mix_radio" => {
            "Ta playlista Mix / Radio może zawierać wiele elementów i zmieniać się z czasem."
        }
        "options.playlist_risk.note.youtube_music_album" => {
            "Zwykle jest to album lub kolekcja YouTube Music."
        }
        "options.playlist_risk.note.liked_videos" => {
            "Polubione filmy zwykle wymagają logowania lub cookies."
        }
        "options.playlist_risk.note.favorites_legacy" => {
            "To stary typ playlisty ulubionych i może nie być już stabilny."
        }
        "options.video" => "Wideo",
        "options.playlist" => "Lista odtwarzania",
        "options.cancel" => "Anuluj",
        "options.load" => "Wczytaj",
        "options.behavior" => "Zachowanie",
        "options.add_action" => "Akcja dodawania",
        "options.download_directly" => "Pobierz bezpośrednio",
        "options.clipboard_change" => "Zmiana schowka",
        "options.run_immediately" => "Uruchom od razu",
        "options.tabs" => "Karty",
        "options.log_tab" => "Karta dziennika",
        "options.show_log_tab" => "Pokaż dziennik",
        "options.playlist_2" => "Lista odtwarzania",
        "options.with_playlist" => "Z playlistą",
        "options.ask" => "Pytaj",
        "options.single_video" => "Wideo",
        "options.full_playlist" => "[Wszystkie]",
        "options.high_risk_prompt" => "Ostrzeżenie wysokiego ryzyka",
        "options.on" => "Wł.",
        "options.playlist_count" => "Liczba elementów playlisty",
        "options.limit" => "Ogranicz",
        "options.max" => "Maks.:",
        "options.items" => " elementów",
        "options.language" => "Język",
        "options.current_language" => "Bieżący język",
        "options.back" => "Wstecz",
        "options.choose" => "Wybierz",
        "options.auto_detect" => "Wykryj automatycznie",
        "options.tool_paths" => "Ścieżki narzędzi",
        "options.file_actions" => "Akcje pliku",
        "options.action_button" => "Przycisk akcji",
        "options.file_action.show_menu" => "Pokaż menu",
        "options.cache" => "Pamięć podręczna",
        "options.cache_location" => "Lokalizacja pamięci podręcznej",
        "options.cache_location.default" => "Domyślne",
        "options.cache_usage" => "Użycie",
        "options.cache_usage_detail" => "Razem: {total} · Audio: {audio} · Wygasłe: {expired}",
        "options.cache_cleanup" => "Czyszczenie",
        "options.cache_refresh" => "Odśwież",
        "options.cache_clear_expired" => "Wyczyść wygasłe",
        "options.cache_clear_audio" => "Wyczyść audio",
        "options.cache_clear_all" => "Wyczyść wszystko",
        "options.appearance_window" => "Wygląd i okno",
        "options.notifications" => "Powiadomienia",
        "options.enable" => "Włącz",
        "options.theme" => "Motyw",
        "options.theme_mode.system" => "Zgodnie z systemem",
        "options.theme_mode.light" => "Jasny",
        "options.theme_mode.dark" => "Ciemny",
        "options.theme_color" => "Kolor motywu",
        "options.theme_color.off" => "Wyłączony",
        "options.theme_color.blue" => "Niebieski",
        "options.theme_color.soft_blue" => "Łagodny niebieski",
        "options.theme_color.purple" => "Fioletowy",
        "options.theme_color.pink" => "Różowy",
        "options.theme_color.green" => "Zielony",
        "options.theme_color.orange" => "Pomarańczowy",
        "options.theme_color.slate" => "Łupkowy",
        "options.ui_scale" => "Skala interfejsu",
        "options.apply" => "Zastosuj",
        "options.current" => "Bieżące",
        "options.always_on_top" => "Zawsze na wierzchu",
        "options.window_position" => "Pozycja okna",
        "options.remember" => "Zapamiętaj",
        "options.window_size" => "Rozmiar okna",
        "options.reinstall" => "Zainstaluj ponownie",
        "options.installing" => "Instalowanie",
        "options.install" => "Zainstaluj",
        "options.executable" => "plik wykonywalny",
        "main.controlled_by_config" => "Kontrolowane przez konfigurację: ",
        "main.controlled_by_config_2" => "Kontrolowane przez konfigurację",
        "picker.no_chapters_available" => "Brak dostępnych rozdziałów.",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "Wybierz zakres pobierania dla tego elementu. Domyślnie całe wideo."
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "Tryb zgodności rozdziałów jest włączony: pobieranie rozdziałów użyje stabilniejszego formatu jednego pliku."
        }
        "picker.subtitles_will_not_be_downloaded" => "Napisy nie zostaną pobrane.",
        "picker.no_subtitles_are_available_for_this_video" => "Brak napisów dla tego wideo.",
        "picker.no_subtitles_are_available_in_this_tab" => "Brak napisów w tej karcie.",
        "picker.source_language" => "Język źródłowy",
        "picker.translation_target" => "Cel tłumaczenia",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "Wskazówka: automatycznie tłumaczone napisy YouTube częściej podlegają limitom niż oryginalne napisy. Wybierz „Bez tłumaczenia”, jeśli potrzebujesz tylko tekstu źródłowego."
        }
        "picker.no_subtitles_are_available_for_this_source" => "Brak napisów dla tego źródła.",
        "picker.target" => "Cel",
        "picker.available_subtitles" => "Dostępne napisy",
        "picker.language" => "Język",
        "picker.subtitle_tab.none" => "Brak napisów",
        "picker.subtitle_tab.original" => "Oryginalne napisy",
        "picker.subtitle_tab.automatic" => "Automatyczne napisy",
        "picker.waiting_analysis" => "Oczekiwanie na analizę",
        "picker.audio_from_video" => "Określone przez format wideo",
        "picker.not_selected" => "Nie wybrano",
        "picker.full_video" => "Całe wideo",
        "picker.no_translation" => "Bez tłumaczenia",
        "picker.until_end" => "koniec",
        "prepare.status.ready" => "Gotowe",
        "prepare.status.missing" => "Brak",
        "prepare.status.warning" => "Wymaga uwagi",
        "prepare.status.failed" => "Niepowodzenie",
        "tool_install.stage.preparing" => "Przygotowywanie",
        "tool_install.stage.downloading" => "Pobieranie",
        "tool_install.stage.extracting" => "Wypakowywanie",
        "tool_install.stage.installing" => "Instalowanie",
        "tool_install.stage.completed" => "Ukończono",
        "tool_install.stage.failed" => "Niepowodzenie",
        "item.status.queued" => "W kolejce",
        "item.status.running" => "W toku",
        "item.status.finished" => "Gotowe",
        "item.status.failed" => "Niepowodzenie",
        "item.status.cancelled" => "Anulowano",
        "processing.transcode" => "Transkodowanie",
        "transcode.graph.axis.compatibility" => "Zgodność",
        "transcode.graph.axis.capacity" => "Pojemność",
        "transcode.graph.axis.resolution" => "Rozdzielczość",
        "transcode.graph.axis.format" => "Format",
        "transcode.graph.compatibility_scope" => "Zakres zgodności",
        "transcode.graph.capacity_target" => "Cel rozmiaru",
        "transcode.graph.resolution_limit" => "Limit rozdzielczości",
        "transcode.graph.format_goal" => "Cel formatu",
        "processing.video" => "Wideo",
        "processing.audio" => "Audio",
        "processing.container" => "Kontener",
        "processing.subtitle" => "Napisy",
        "processing.choice.source" => "Oryginał",
        "processing.subtitle.preserve" => "Oryginał",
        "processing.subtitle.embed" => "Osadź",
        "processing.subtitle.burn" => "Wypal w wideo",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "Wszystkie pliki",
        "options.filter_executable" => "Plik wykonywalny",
        "app_mode.origin" => "Tryb Origin",
        "app_mode.standard" => "Tryb standardowy",
        "app_mode.audio" => "Tryb audio",
        "music.status.completed" => "Gotowe",
        "music.status.resolving" => "Rozpoznawanie",
        "music.status.buffering" => "Buforowanie",
        "music.status.ready" => "Gotowe",
        "music.status.caching" => "Zapisywanie w pamięci podręcznej",
        "music.status.playing" => "Odtwarzanie",
        "music.status.paused" => "Wstrzymano",
        "music.status.failed" => "Niepowodzenie",
        "notification.download_complete" => "Pobieranie ukończone",
        "notification.download_failed" => "Pobieranie nie powiodło się",
        "notification.completed_file" => "Ukończono: {file}",
        "notification.download_completed" => "Pobieranie ukończone.",
        "options.music_download_format" => "Music download format",
        "options.music_download_audio_label" => "Wyjście audio",
        "options.music_download_preference_best" => "Najlepsze",
        _ => key,
    }
}
