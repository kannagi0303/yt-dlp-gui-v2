pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.advanced" => "Erweitert",
        "tab.about" => "About",
        "about.tools" => "Tool-Versionen",
        "about.current_version" => "Aktuell",
        "about.latest_version" => "Neueste",
        "about.author" => "Autor",
        "about.source" => "Quelle",
        "about.status" => "Status",
        "about.message" => "Meldung",
        "about.check_updates" => "Nach Updates suchen",
        "about.update_all" => "Alle aktualisieren",
        "about.restart" => "Neu starten",
        "about.open_release" => "Release öffnen",
        "about.install" => "Installieren",
        "about.update" => "Aktualisieren",
        "about.running" => "Updateprüfung läuft...",
        "about.last_check" => "Letzte Prüfung:",
        "about.relative.minutes" => "{count} Min.",
        "about.relative.hour" => "1 Std.",
        "about.relative.hours" => "{count} Std.",
        "about.relative.day" => "1 Tag",
        "about.relative.days" => "{count} Tage",
        "about.never_checked" => "Noch nicht nach Updates gesucht",
        "about.no_release_notes_loaded" => {
            "Keine Release Notes geladen. Bitte zuerst nach Updates suchen."
        }
        "about.ownership.managed_portable" => "v2 verwaltet",
        "about.ownership.external" => "Extern",
        "about.ownership.missing" => "Fehlt",
        "about.ownership.unknown" => "Unbekannt",
        "about.status.unknown" => "Nicht geprüft",
        "about.status.checking" => "Prüfung läuft",
        "about.status.up_to_date" => "Aktuell ✓",
        "about.status.update_available" => "Update verfügbar ↑",
        "about.status.missing" => "Fehlt +",
        "about.status.downloading" => "Wird heruntergeladen",
        "about.status.downloading_percent" => "Wird heruntergeladen {percent}%",
        "about.status.staged" => "Bereitgestellt",
        "about.status.pending_restart" => "Neustart ausstehend",
        "about.status.applying" => "Wird angewendet",
        "about.status.installed" => "Installiert",
        "about.status.skipped" => "Übersprungen",
        "about.status.failed" => "Fehlgeschlagen !",
        "tab.options" => "Optionen",
        "tab.log" => "Protokoll",
        "main.url_hint" => "URL",
        "action.download" => "Herunterladen",
        "action.add" => "Hinzufügen",
        "action.analyze" => "Analysieren",
        "action.stop" => "Anhalten",
        "action.stopping" => "Wird angehalten",
        "action.cut" => "Ausschneiden",
        "action.copy" => "Kopieren",
        "action.paste" => "Einfügen",
        "action.clear" => "Leeren",
        "item.thumbnail" => "Miniaturbild",
        "item.thumbnail_preview" => "Miniaturvorschau",
        "single.title" => "Titel",
        "single.description" => "Beschreibung",
        "single.info.channel" => "Kanal",
        "single.info.date" => "Datum",
        "single.info.views" => "Aufrufe",
        "item.download_thumbnail" => "Miniaturbild herunterladen",
        "media.video" => "Video",
        "media.audio" => "Audio",
        "media.subtitle" => "Untertitel",
        "media.section" => "Bereich",
        "item.file_name" => "Dateiname",
        "main.target_folder" => "Ausgabeordner",
        "picker.title.video" => "Videoformat auswählen",
        "picker.title.audio" => "Audioformat auswählen",
        "picker.title.subtitle" => "Untertitel auswählen",
        "picker.title.section" => "Bereich auswählen",
        "action.back" => "Zurück",
        "picker.mode.filter" => "Filter",
        "picker.mode.table" => "Tabelle",
        "action.confirm" => "Bestätigen",
        "picker.empty_table" => "Keine Formate zum Anzeigen",
        "picker.header.resolution" => "Auflösung",
        "picker.header.range" => "Bereich",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "Format",
        "picker.header.codec" => "Codec",
        "picker.header.size" => "Größe",
        "picker.header.sample_rate" => "Abtastrate",
        "picker.filter.resolution" => "Auflösung",
        "picker.filter.range" => "Bereich",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "Codec",
        "picker.filter.sample_rate" => "Abtastrate",
        "main.missing_yt_dlp_callout" => {
            "yt-dlp fehlt. Installiere es oder wähle yt-dlp.exe in den Optionen aus."
        }
        "advance.source" => "Quelle",
        "advance.config" => "Konfiguration",
        "advance.none" => "Keine",
        "advance.network_access" => "Netzwerk & Zugriff",
        "advance.proxy" => "Proxy",
        "advance.enable_proxy" => "Proxy aktivieren",
        "advance.certificate" => "Zertifikat",
        "advance.skip_certificate_verification" => "Zertifikatsprüfung überspringen",
        "advance.use_cookies" => "Cookies verwenden",
        "advance.enable_cookies" => "Cookies aktivieren",
        "advance.cookie_source" => "Cookie-Quelle",
        "advance.cookie_source.auto" => "Automatisch nach Website",
        "advance.cookie_source.file" => "Datei verwenden (cookies.txt)",
        "advance.cookie_auto" => "Automatisch",
        "advance.cookie_auto_note" => {
            "Downloads verwenden das gespeicherte Cookie, das zur URL passt."
        }
        "advance.cookie_rescue" => "Cookie-Hilfe",
        "advance.cookie_file" => "Cookie-Datei",
        "advance.get_cookie" => "Cookie holen",
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
            "Ein separates Browserfenster öffnen, um Cookies zu holen."
        }
        "youtube_login_rescue.title" => "Cookie-Hilfe",
        "youtube_login_rescue.confirm_heading" => "Separates Browser-Anmeldefenster öffnen",
        "youtube_login_rescue.confirm_body" => {
            "Ein unabhängiges {browser}-Fenster öffnet die URL, ohne deine persönlichen Browserdaten zu verwenden."
        }
        "youtube_login_rescue.target_url_label" => "Website-URL",
        "youtube_login_rescue.target_url_hint" => "https://www.youtube.com/",
        "youtube_login_rescue.clipboard_prefilled" => {
            "URL wurde aus der Zwischenablage übernommen."
        }
        "youtube_login_rescue.drop_url_note" => {
            "Füge eine URL ein oder ziehe eine .url-/Textdatei hierher."
        }
        "youtube_login_rescue.paste_clipboard" => "Zwischenablage einfügen",
        "youtube_login_rescue.cookie_note" => {
            "Melde dich dort an. Sobald Cookies gefunden werden, wird das Fenster geschlossen und sie werden automatisch angewendet."
        }
        "youtube_login_rescue.no_browser_title" => "Kein unterstützter Browser gefunden",
        "youtube_login_rescue.no_browser_body" => {
            "Cookies holen benötigt derzeit Chrome, Brave oder Microsoft Edge. Du kannst weiterhin manuell cookies.txt auswählen."
        }
        "youtube_login_rescue.start" => "Starten",
        "youtube_login_rescue.opening" => "{browser} wird geöffnet...",
        "youtube_login_rescue.waiting_for_cdp" => {
            "Warte auf die Verbindung des {browser}-Anmeldefensters..."
        }
        "youtube_login_rescue.waiting_for_cookie" => {
            "Anmeldefenster verbunden. Warte auf Website-Cookies..."
        }
        "youtube_login_rescue.cookie_exported" => "Cookie wurde gespeichert.",
        "youtube_login_rescue.cookie_exported_note" => {
            "{site}-Cookie gespeichert. Downloads von dieser Website verwenden es automatisch."
        }
        "youtube_login_rescue.do_not_close_note" => {
            "Bitte lass den Anmeldebrowser geöffnet, solange diese Prüfung läuft."
        }
        "youtube_login_rescue.cdp_ready" => "Anmeldefenster verbunden.",
        "youtube_login_rescue.ready_next_step_note" => {
            "Bitte melde dich im Browser bei YouTube an. Der Cookie-Export wird im nächsten Schritt hinzugefügt."
        }
        "youtube_login_rescue.close_login_window" => "Anmeldefenster schließen",
        "youtube_login_rescue.failed" => "Cookie-Hilfe fehlgeschlagen",
        "youtube_login_rescue.retry" => "Erneut versuchen",
        "advance.no_cookies_txt_selected" => "Keine cookies.txt ausgewählt",
        "advance.browse" => "Durchsuchen",
        "advance.select_netscape_cookies_txt" => "Netscape-cookies.txt auswählen",
        "advance.clear" => "Löschen",
        "advance.browser" => "Browser",
        "advance.default" => "Standard",
        "advance.external_downloader" => "Externer Downloader",
        "advance.use_aria2_for_faster_downloads" => "Aria2 für schnellere Downloads verwenden",
        "advance.download_control" => "Download-Steuerung",
        "advance.concurrent_fragments" => "Gleichzeitige Fragmente",
        "advance.live_streams" => "Livestreams",
        "advance.download_live_streams_from_start_experimental" => {
            "Livestreams von Anfang an herunterladen (experimentell)"
        }
        "advance.1_default" => "1 (Standard)",
        "advance.rate_limit" => "Geschwindigkeitslimit",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => {
            "z. B. 2M, 800K; leer lassen für unbegrenzt"
        }
        "advance.chapters" => "Kapitel",
        "advance.download_range" => "Downloadbereich",
        "advance.always_show_download_range" => "Bereichsauswahl immer anzeigen",
        "advance.chapter_download_compatibility_mode" => {
            "Kompatibilitätsmodus für Kapitel-Download"
        }
        "advance.file_time" => "Dateizeit",
        "advance.file_time.none" => "Nicht ändern",
        "advance.file_time.upload_date" => "Upload-Datum des Videos verwenden",
        "advance.file_time.download_time" => "Downloadzeit verwenden",
        "advance.post_processing" => "Nachbearbeitung",
        "advance.thumbnail" => "Miniaturansicht",
        "advance.download" => "Herunterladen",
        "advance.embed" => "Einbetten",
        "advance.subtitles" => "Untertitel",
        "advance.download_conversion" => "Nach dem Download konvertieren",
        "advance.enable" => "Aktivieren",
        "advance.settings" => "Einstellungen",
        "item.save_as" => "Speichern unter",
        "item.error" => "Fehler",
        "item.all" => "Alle",
        "item.queued" => "In Warteschlange",
        "item.done" => "Fertig",
        "item.failed" => "Fehlgeschlagen",
        "item.clear_all" => "Alles löschen",
        "item.add_a_video_url" => "Video-URL hinzufügen",
        "item.add_an_audio_url" => "Audio-URL hinzufügen",
        "item.after_adding_choose_the_video_format_here" => "Videoformat wählen",
        "item.after_adding_choose_the_audio_format_here" => "Audioformat wählen",
        "item.loading_thumbnail" => "Miniaturansicht wird geladen",
        "item.file_actions" => "Dateiaktionen",
        "item.open_file" => "Datei öffnen",
        "item.open_folder" => "Ordner öffnen",
        "item.copy_path" => "Pfad kopieren",
        "item.file_not_found_opened_the_output_location" => {
            "Datei nicht gefunden; Ausgabeort wurde geöffnet."
        }
        "item.opened_output_location" => "Ausgabeort geöffnet.",
        "item.copied_output_path" => "Ausgabepfad kopiert.",
        "prepare.language" => "Sprache",
        "prepare.back" => "Zurück",
        "prepare.auto_detect" => "Automatisch erkennen",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Installiere die erforderlichen Tools jetzt oder überspringe dies und richte sie später in den Optionen ein."
        }
        "prepare.optional" => "Wahlweise",
        "prepare.missing" => "Fehlt",
        "prepare.install_later" => "Später installieren",
        "prepare.downloading_100" => "Download 100 %",
        "prepare.extracting_100" => "Entpacken 100 %",
        "prepare.install_failed" => "Installation fehlgeschlagen",
        "prepare.install_all" => "Alle installieren",
        "prepare.reinstall" => "Neu installieren",
        "prepare.installing" => "Installiere",
        "prepare.skip" => "Überspringen",
        "prepare.install" => "Installieren",
        "prepare.another_tool_is_already_being_installed" => {
            "Ein anderes Tool wird bereits installiert."
        }
        "prepare.needs_attention" => "Benötigt Aufmerksamkeit",
        "prepare.req.app_folder.title" => "App-Ordner",
        "prepare.req.app_folder.description" => {
            "Der portable Ordner muss für Einstellungen und Hilfsdaten beschreibbar sein."
        }
        "prepare.req.tools_folder.title" => "Werkzeugordner",
        "prepare.req.tools_folder.description" => {
            "Die Abhängigkeitsbereitstellung speichert hier yt-dlp, FFmpeg und Deno."
        }
        "prepare.req.deployment_temp.title" => "Bereitstellungs-Temp",
        "prepare.req.deployment_temp.description" => {
            "Die Entpackung von FFmpeg und Deno verwendet diesen temporären Ordner."
        }
        "prepare.req.download_cache.title" => "Download-Cache",
        "prepare.req.download_cache.description" => {
            "Der Cache-Modus von yt-dlp-gui speichert hier den yt-dlp-Cache."
        }
        "prepare.req.output_folder.title" => "Ausgabeordner",
        "prepare.req.output_folder.description" => {
            "Videos, Audio und Untertitel werden hier gespeichert."
        }
        "prepare.req.output_folder.recommendation" => {
            "Wähle im Hauptfenster oder in den Optionen einen gültigen Ausgabeordner."
        }
        "prepare.req.config_file.title" => "Konfigurationsdatei",
        "prepare.req.config_file.description" => {
            "Die App muss den Prepare-Überspringstatus und die Werkzeugpfade speichern können."
        }
        "prepare.req.generic_writable_recommendation" => {
            "Wähle einen beschreibbaren Ordner und prüfe die Berechtigungen."
        }
        "prepare.req.config_not_folder" => {
            "Der Konfigurationspfad zeigt auf einen Ordner. Wähle stattdessen einen Dateipfad."
        }
        "prepare.req.config_readonly" => "Die Konfigurationsdatei ist schreibgeschützt.",
        "prepare.req.config_readonly_recommendation" => {
            "Erlaube das Schreiben in die Konfigurationsdatei oder wähle einen anderen App-Ordner."
        }
        "prepare.req.use_folder_path" => "Wähle einen Ordnerpfad statt eines Dateipfads.",
        "prepare.req.move_portable_folder" => {
            "Verschiebe die App in einen beschreibbaren portablen Ordner."
        }
        "prepare.req.avoid_protected_folder" => {
            "Lege die portable App nicht unter Program Files oder im Windows-Ordner ab. Verschiebe sie nach D:\\Portable oder in einen Benutzerordner."
        }
        "prepare.req.move_non_synced_folder" => {
            "Verschiebe sie in einen nicht synchronisierten Ordner, z. B. D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.drive_parent_exists" => {
            "Stelle sicher, dass Laufwerk und übergeordneter Ordner vorhanden sind."
        }
        "prepare.req.permission_denied" => {
            "Verschiebe die App in einen beschreibbaren portablen Ordner. Wenn Desktop/Dokumente/Downloads weiterhin fehlschlagen, blockiert möglicherweise Defender Controlled Folder Access."
        }
        "prepare.req.file_in_use" => {
            "Schließe das Programm, das diesen Ordner möglicherweise verwendet, oder wähle einen anderen Ordner."
        }
        "prepare.req.free_disk_space" => "Gib Speicherplatz frei oder wähle ein anderes Laufwerk.",
        "prepare.req.path_too_long" => {
            "Verschiebe die App in einen kürzeren Pfad, z. B. D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.choose_writable_portable_folder" => {
            "Wähle einen eindeutig beschreibbaren portablen Ordner und prüfe erneut."
        }
        "prepare.req.clear_write_test" => {
            "Entferne die verbliebene Schreibtestdatei und prüfe erneut."
        }
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "Diese URL enthält sowohl ein Video als auch eine Playlist"
        }
        "options.detected" => "Erkannt ",
        "options.playlist_prompt" => "Playlist-Abfrage",
        "options.which_one_should_be_loaded" => "Was soll geladen werden?",
        "options.both_video_and_playlist_were_detected" => "Video und Playlist wurden erkannt",
        "options.this_playlist_may_contain_many_items" => {
            "Diese Playlist kann viele Elemente enthalten."
        }
        "options.playlist_risk.kind.channel_generated" => "Von YouTube erzeugte Kanal-Playlist",
        "options.playlist_risk.kind.youtube_mix_radio" => "YouTube Mix / Radio",
        "options.playlist_risk.kind.youtube_music_album" => "YouTube-Music-Album/Sammlung",
        "options.playlist_risk.kind.liked_videos" => "Mit „Gefällt mir“ markierte Videos",
        "options.playlist_risk.kind.favorites_legacy" => "Alte Favoriten-Playlist",
        "options.playlist_risk.note.channel_generated" => {
            "Diese von YouTube erzeugte Kanal-Playlist vorsichtig behandeln."
        }
        "options.playlist_risk.note.youtube_mix_radio" => {
            "Diese Mix-/Radio-Playlist kann viele Elemente enthalten und sich mit der Zeit ändern."
        }
        "options.playlist_risk.note.youtube_music_album" => {
            "Dies ist normalerweise ein YouTube-Music-Album oder eine Sammlung."
        }
        "options.playlist_risk.note.liked_videos" => {
            "Mit „Gefällt mir“ markierte Videos erfordern meist Anmeldung oder Cookies."
        }
        "options.playlist_risk.note.favorites_legacy" => {
            "Dies ist ein altes Favoriten-Playlist-Format und ist möglicherweise nicht mehr stabil."
        }
        "options.video" => "Video",
        "options.playlist" => "Wiedergabeliste",
        "options.cancel" => "Abbrechen",
        "options.load" => "Laden",
        "options.behavior" => "Verhalten",
        "options.add_action" => "Aktion hinzufügen",
        "options.download_directly" => "Direkt herunterladen",
        "options.clipboard_change" => "Zwischenablage geändert",
        "options.run_immediately" => "Sofort ausführen",
        "options.tabs" => "Registerkarten",
        "options.log_tab" => "Protokoll-Tab",
        "options.show_log_tab" => "Protokoll anzeigen",
        "options.playlist_2" => "Wiedergabeliste",
        "options.with_playlist" => "Mit Playlist",
        "options.ask" => "Fragen",
        "options.single_video" => "Einzelnes Video",
        "options.full_playlist" => "Gesamte Wiedergabeliste",
        "options.high_risk_prompt" => "Warnung bei hohem Risiko",
        "options.on" => "Ein",
        "options.playlist_count" => "Anzahl der Einträge",
        "options.limit" => "Begrenzen",
        "options.max" => "Max.:",
        "options.items" => " Elemente",
        "options.language" => "Sprache",
        "options.current_language" => "Aktuelle Sprache",
        "options.back" => "Zurück",
        "options.choose" => "Auswählen",
        "options.auto_detect" => "Automatisch erkennen",
        "options.tool_paths" => "Tool-Pfade",
        "options.file_actions" => "Dateiaktionen",
        "options.action_button" => "Aktionsschaltfläche",
        "options.file_action.show_menu" => "Menü anzeigen",
        "options.cache" => "Cache",
        "options.cache_location" => "Cache-Speicherort",
        "options.cache_location.default" => "Standard",
        "options.cache_usage" => "Nutzung",
        "options.cache_usage_detail" => "Gesamt: {total} · Audio: {audio} · Abgelaufen: {expired}",
        "options.cache_cleanup" => "Bereinigung",
        "options.cache_refresh" => "Aktualisieren",
        "options.cache_clear_expired" => "Abgelaufene löschen",
        "options.cache_clear_audio" => "Audio löschen",
        "options.cache_clear_all" => "Alles löschen",
        "options.appearance_window" => "Darstellung & Fenster",
        "options.notifications" => "Benachrichtigungen",
        "options.enable" => "Aktivieren",
        "options.theme" => "Design",
        "options.theme_mode.system" => "System folgen",
        "options.theme_mode.light" => "Hell",
        "options.theme_mode.dark" => "Dunkel",
        "options.theme_color" => "Designfarbe",
        "options.theme_color.off" => "Aus",
        "options.theme_color.blue" => "Blau",
        "options.theme_color.soft_blue" => "Sanftes Blau",
        "options.theme_color.purple" => "Lila",
        "options.theme_color.pink" => "Rosa",
        "options.theme_color.green" => "Grün",
        "options.theme_color.orange" => "Orange",
        "options.theme_color.slate" => "Schiefer",
        "options.ui_scale" => "UI-Skalierung",
        "options.apply" => "Anwenden",
        "options.current" => "Aktuell",
        "options.always_on_top" => "Immer im Vordergrund",
        "options.window_position" => "Fensterposition",
        "options.remember" => "Merken",
        "options.window_size" => "Fenstergröße",
        "options.reinstall" => "Neu installieren",
        "options.installing" => "Installiere",
        "options.install" => "Installieren",
        "options.executable" => "ausführbare Datei",
        "main.controlled_by_config" => "Durch Konfiguration gesteuert: ",
        "main.controlled_by_config_2" => "Durch Konfiguration gesteuert",
        "picker.section_tab.chapters" => "Kapitel",
        "picker.section_tab.time_range" => "Zeitbereich",
        "picker.section_chapter_instructions" => {
            "Wählen Sie ein oder mehrere Kapitel. Benachbarte Kapitel werden zu einer Ausgabe."
        }
        "picker.section_time_instructions" => {
            "Bewegen Sie den Abspielkopf, setzen Sie Start und Ende und fügen Sie den Bereich hinzu."
        }
        "picker.section_time_unavailable" => {
            "Die Videodauer ist nicht verfügbar; ein eigener Zeitbereich kann nicht erstellt werden."
        }
        "picker.section_select_all" => "Alle auswählen",
        "picker.section_from_selected_to_end" => "Vom ersten gewählten bis zum Ende",
        "picker.section_set_start" => "Start setzen",
        "picker.section_set_end" => "Ende setzen",
        "picker.section_add_range" => "Bereich hinzufügen",
        "picker.section_no_custom_ranges" => "Keine eigenen Zeitbereiche hinzugefügt.",
        "picker.no_chapters_available" => "Keine Kapitel verfügbar.",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "Wähle den Bereich, der für dieses Element heruntergeladen werden soll. Standard ist das ganze Video."
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "Der Kompatibilitätsmodus für Kapitel ist aktiviert: Kapitel-Downloads verwenden ein stabileres Einzelformat."
        }
        "picker.subtitles_will_not_be_downloaded" => "Untertitel werden nicht heruntergeladen.",
        "picker.no_subtitles_are_available_for_this_video" => {
            "Für dieses Video sind keine Untertitel verfügbar."
        }
        "picker.no_subtitles_are_available_in_this_tab" => {
            "In diesem Tab sind keine Untertitel verfügbar."
        }
        "picker.source_language" => "Ausgangssprache",
        "picker.translation_target" => "Zielsprache",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "Tipp: Automatisch übersetzte YouTube-Untertitel werden eher rate-limitiert als Originaluntertitel. Wähle „Keine Übersetzung“, wenn du nur den Originaltext brauchst."
        }
        "picker.no_subtitles_are_available_for_this_source" => {
            "Für diese Quelle sind keine Untertitel verfügbar."
        }
        "picker.target" => "Ziel",
        "picker.available_subtitles" => "Verfügbare Untertitel",
        "picker.language" => "Sprache",
        "picker.subtitle_tab.none" => "Keine Untertitel",
        "picker.subtitle_tab.original" => "Originaluntertitel",
        "picker.subtitle_tab.automatic" => "Automatische Untertitel",
        "picker.waiting_analysis" => "Warte auf Analyse",
        "picker.audio_from_video" => "Durch Videoformat bestimmt",
        "picker.not_selected" => "Nicht ausgewählt",
        "picker.full_video" => "Ganzes Video",
        "picker.section_summary.chapters" => "{chapters} Kapitel gewählt · {outputs} Ausgaben",
        "picker.section_summary.custom" => "{ranges} Zeitbereiche · {outputs} Ausgaben",
        "picker.section_summary.combined" => {
            "{chapters} Kapitel + {ranges} Zeitbereiche · {outputs} Ausgaben"
        }
        "picker.no_translation" => "Keine Übersetzung",
        "picker.until_end" => "Ende",
        "prepare.status.ready" => "Bereit",
        "prepare.status.missing" => "Fehlt",
        "prepare.status.warning" => "Benötigt Aufmerksamkeit",
        "prepare.status.failed" => "Fehlgeschlagen",
        "tool_install.stage.preparing" => "Vorbereitung",
        "tool_install.stage.downloading" => "Wird heruntergeladen",
        "tool_install.stage.extracting" => "Wird entpackt",
        "tool_install.stage.installing" => "Wird installiert",
        "tool_install.stage.completed" => "Abgeschlossen",
        "tool_install.stage.failed" => "Fehlgeschlagen",
        "item.status.queued" => "In Warteschlange",
        "item.status.running" => "Läuft",
        "item.status.finished" => "Fertig",
        "item.status.failed" => "Fehlgeschlagen",
        "item.status.cancelled" => "Abgebrochen",
        "processing.transcode" => "Transkodieren",
        "transcode.graph.axis.compatibility" => "Kompatibilität",
        "transcode.graph.axis.capacity" => "Kapazität",
        "transcode.graph.axis.resolution" => "Auflösung",
        "transcode.graph.axis.format" => "Format",
        "transcode.graph.compatibility_scope" => "Kompatibilitätsumfang",
        "transcode.graph.capacity_target" => "Größenziel",
        "transcode.graph.resolution_limit" => "Auflösungsgrenze",
        "transcode.graph.format_goal" => "Formatziel",
        "processing.video" => "Video",
        "processing.audio" => "Audio",
        "processing.container" => "Containerformat",
        "processing.subtitle" => "Untertitel",
        "processing.choice.source" => "Original",
        "processing.subtitle.preserve" => "Original",
        "processing.subtitle.embed" => "Einbetten",
        "processing.subtitle.burn" => "Einbrennen",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "Alle Dateien",
        "options.filter_executable" => "Ausführbare Datei",
        "app_mode.origin" => "Origin-Modus",
        "app_mode.standard" => "Standardmodus",
        "app_mode.audio" => "Audiomodus",
        "music.status.completed" => "Fertig",
        "music.status.resolving" => "Wird aufgelöst",
        "music.status.buffering" => "Puffert",
        "music.status.ready" => "Bereit",
        "music.status.caching" => "Wird zwischengespeichert",
        "music.status.playing" => "Wird abgespielt",
        "music.status.paused" => "Pausiert",
        "music.status.failed" => "Fehlgeschlagen",
        "notification.download_complete" => "Download abgeschlossen",
        "notification.download_failed" => "Download fehlgeschlagen",
        "notification.completed_file" => "Abgeschlossen: {file}",
        "notification.download_completed" => "Download abgeschlossen.",
        "options.music_download_format" => "Music download format",
        "options.music_download_audio_label" => "Audioausgabe",
        "options.music_download_preference_best" => "Beste",
        _ => key,
    }
}
