pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.advanced" => "Προχωρημένο",
        "tab.about" => "About",
        "about.tools" => "Εκδόσεις εργαλείων",
        "about.current_version" => "Τρέχουσα",
        "about.latest_version" => "Πιο πρόσφατη",
        "about.author" => "Συντάκτης",
        "about.source" => "Πηγή",
        "about.status" => "Κατάσταση",
        "about.message" => "Μήνυμα",
        "about.check_updates" => "Έλεγχος ενημερώσεων",
        "about.update_all" => "Ενημέρωση όλων",
        "about.restart" => "Επανεκκίνηση",
        "about.open_release" => "Άνοιγμα Release",
        "about.install" => "Εγκατάσταση",
        "about.update" => "Ενημέρωση",
        "about.running" => "Εκτελείται έλεγχος ενημερώσεων...",
        "about.last_check" => "Τελευταίος έλεγχος:",
        "about.relative.minutes" => "{count} λεπ.",
        "about.relative.hour" => "1 ώρα",
        "about.relative.hours" => "{count} ώρες",
        "about.relative.day" => "1 ημέρα",
        "about.relative.days" => "{count} ημέρες",
        "about.never_checked" => "Δεν έχει γίνει ακόμη έλεγχος ενημερώσεων",
        "about.no_release_notes_loaded" => {
            "Δεν έχουν φορτωθεί σημειώσεις έκδοσης. Πατήστε πρώτα Έλεγχος ενημερώσεων."
        }
        "about.ownership.managed_portable" => "Διαχείριση v2",
        "about.ownership.external" => "Εξωτερικό",
        "about.ownership.missing" => "Λείπει",
        "about.ownership.unknown" => "Άγνωστο",
        "about.status.unknown" => "Δεν ελέγχθηκε",
        "about.status.checking" => "Έλεγχος",
        "about.status.up_to_date" => "Ενημερωμένο ✓",
        "about.status.update_available" => "Διαθέσιμη ενημέρωση ↑",
        "about.status.missing" => "Λείπει +",
        "about.status.downloading" => "Λήψη",
        "about.status.downloading_percent" => "Λήψη {percent}%",
        "about.status.staged" => "Προετοιμασμένο",
        "about.status.pending_restart" => "Εκκρεμεί επανεκκίνηση",
        "about.status.applying" => "Εφαρμογή",
        "about.status.installed" => "Εγκατεστημένο",
        "about.status.skipped" => "Παραλείφθηκε",
        "about.status.failed" => "Απέτυχε !",
        "tab.options" => "Επιλογές",
        "tab.log" => "Καταγραφή",
        "main.url_hint" => "URL",
        "action.download" => "Λήψη",
        "action.add" => "Προσθήκη",
        "action.analyze" => "Ανάλυση",
        "action.stop" => "Διακοπή",
        "action.stopping" => "Διακοπή...",
        "action.cut" => "Αποκοπή",
        "action.copy" => "Αντιγραφή",
        "action.paste" => "Επικόλληση",
        "action.clear" => "Εκκαθάριση",
        "item.thumbnail" => "Μικρογραφία",
        "item.thumbnail_preview" => "Προεπισκόπηση μικρογραφίας",
        "single.title" => "Τίτλος",
        "single.description" => "Περιγραφή",
        "single.info.channel" => "Κανάλι",
        "single.info.date" => "Ημερομηνία",
        "single.info.views" => "Προβολές",
        "item.download_thumbnail" => "Λήψη μικρογραφίας",
        "media.video" => "Βίντεο",
        "media.audio" => "Ήχος",
        "media.subtitle" => "Υπότιτλοι",
        "media.section" => "Εύρος",
        "item.file_name" => "Όνομα αρχείου",
        "main.target_folder" => "Φάκελος εξόδου",
        "picker.title.video" => "Επιλογή μορφής βίντεο",
        "picker.title.audio" => "Επιλογή μορφής ήχου",
        "picker.title.subtitle" => "Επιλογή υποτίτλων",
        "picker.title.section" => "Επιλογή εύρους",
        "action.back" => "Πίσω",
        "picker.mode.filter" => "Φίλτρα",
        "picker.mode.table" => "Πίνακας",
        "action.confirm" => "Επιβεβαίωση",
        "picker.empty_table" => "Δεν υπάρχουν μορφές για εμφάνιση",
        "picker.header.resolution" => "Ανάλυση",
        "picker.header.range" => "Εύρος",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "Μορφή",
        "picker.header.codec" => "Κωδικοποιητής",
        "picker.header.size" => "Μέγεθος",
        "picker.header.sample_rate" => "ASR",
        "picker.filter.resolution" => "Ανάλυση",
        "picker.filter.range" => "Εύρος",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "Κωδικοποιητής",
        "picker.filter.sample_rate" => "ASR",
        "main.missing_yt_dlp_callout" => {
            "Το yt-dlp λείπει. Εγκαταστήστε το ή επιλέξτε το yt-dlp.exe στις Επιλογές."
        }
        "advance.source" => "Πηγή",
        "advance.config" => "Ρύθμιση",
        "advance.none" => "Κανένα",
        "advance.network_access" => "Δίκτυο & πρόσβαση",
        "advance.proxy" => "Διακομιστής μεσολάβησης",
        "advance.enable_proxy" => "Ενεργοποίηση proxy",
        "advance.certificate" => "Πιστοποιητικό",
        "advance.skip_certificate_verification" => "Παράκαμψη επαλήθευσης πιστοποιητικού",
        "advance.use_cookies" => "Χρήση cookies",
        "advance.enable_cookies" => "Ενεργοποίηση cookies",
        "advance.cookie_source" => "Πηγή cookies",
        "advance.cookie_source.auto" => "Αυτόματα ανά ιστότοπο",
        "advance.cookie_source.file" => "Χρήση αρχείου (cookies.txt)",
        "advance.cookie_auto" => "Αυτόματα",
        "advance.cookie_auto_note" => {
            "Οι λήψεις χρησιμοποιούν το αποθηκευμένο Cookie που ταιριάζει με το URL."
        }
        "advance.cookie_rescue" => "Ανάκτηση Cookie",
        "advance.cookie_file" => "Αρχείο cookies",
        "advance.get_cookie" => "Λήψη Cookie",
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
            "Άνοιγμα ξεχωριστού παραθύρου browser για λήψη cookies."
        }
        "youtube_login_rescue.title" => "Ανάκτηση Cookie",
        "youtube_login_rescue.confirm_heading" => "Άνοιγμα ξεχωριστού παραθύρου σύνδεσης",
        "youtube_login_rescue.confirm_body" => {
            "Ένα ανεξάρτητο παράθυρο {browser} θα ανοίξει το URL χωρίς να χρησιμοποιήσει τα προσωπικά δεδομένα του browser."
        }
        "youtube_login_rescue.target_url_label" => "URL ιστότοπου",
        "youtube_login_rescue.target_url_hint" => "https://www.youtube.com/",
        "youtube_login_rescue.clipboard_prefilled" => "Το URL συμπληρώθηκε από το πρόχειρο.",
        "youtube_login_rescue.drop_url_note" => {
            "Επικολλήστε ένα URL ή σύρετε ένα αρχείο .url / κειμένου."
        }
        "youtube_login_rescue.paste_clipboard" => "Επικόλληση προχείρου",
        "youtube_login_rescue.cookie_note" => {
            "Συνδεθείτε εκεί. Μόλις βρεθούν cookies, το παράθυρο θα κλείσει και θα εφαρμοστούν αυτόματα."
        }
        "youtube_login_rescue.no_browser_title" => "Δεν βρέθηκε υποστηριζόμενος browser",
        "youtube_login_rescue.no_browser_body" => {
            "Η λήψη cookies χρειάζεται προς το παρόν Chrome, Brave ή Microsoft Edge. Μπορείτε ακόμη να επιλέξετε χειροκίνητα cookies.txt."
        }
        "youtube_login_rescue.start" => "Έναρξη",
        "youtube_login_rescue.opening" => "Άνοιγμα {browser}...",
        "youtube_login_rescue.waiting_for_cdp" => {
            "Αναμονή σύνδεσης του παραθύρου σύνδεσης {browser}..."
        }
        "youtube_login_rescue.waiting_for_cookie" => {
            "Το παράθυρο σύνδεσης είναι συνδεδεμένο. Αναμονή για cookies ιστότοπου..."
        }
        "youtube_login_rescue.cookie_exported" => "Το Cookie αποθηκεύτηκε.",
        "youtube_login_rescue.cookie_exported_note" => {
            "Αποθηκεύτηκε Cookie για {site}. Οι λήψεις από αυτόν τον ιστότοπο θα το χρησιμοποιούν αυτόματα."
        }
        "youtube_login_rescue.do_not_close_note" => {
            "Κρατήστε ανοιχτό τον browser σύνδεσης όσο εκτελείται αυτός ο έλεγχος."
        }
        "youtube_login_rescue.cdp_ready" => "Το παράθυρο σύνδεσης είναι συνδεδεμένο.",
        "youtube_login_rescue.ready_next_step_note" => {
            "Ολοκληρώστε τη σύνδεση στο YouTube μέσα στον browser. Η εξαγωγή Cookie θα προστεθεί στο επόμενο βήμα."
        }
        "youtube_login_rescue.close_login_window" => "Κλείσιμο παραθύρου σύνδεσης",
        "youtube_login_rescue.failed" => "Η ανάκτηση Cookie απέτυχε",
        "youtube_login_rescue.retry" => "Επανάληψη",
        "advance.no_cookies_txt_selected" => "Δεν επιλέχθηκε cookies.txt",
        "advance.browse" => "Αναζήτηση",
        "advance.select_netscape_cookies_txt" => "Επιλογή Netscape cookies.txt",
        "advance.clear" => "Εκκαθάριση",
        "advance.browser" => "Πρόγραμμα περιήγησης",
        "advance.default" => "Προεπιλογή",
        "advance.external_downloader" => "Εξωτερικός λήπτης",
        "advance.use_aria2_for_faster_downloads" => "Χρήση Aria2 για ταχύτερες λήψεις",
        "advance.download_control" => "Έλεγχος λήψης",
        "advance.concurrent_fragments" => "Ταυτόχρονα τμήματα",
        "advance.live_streams" => "Ζωντανές μεταδόσεις",
        "advance.download_live_streams_from_start_experimental" => {
            "Λήψη ζωντανών μεταδόσεων από την αρχή (πειραματικό)"
        }
        "advance.1_default" => "1 (προεπιλογή)",
        "advance.rate_limit" => "Όριο Ταχύτητας",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => {
            "π.χ. 2M, 800K· αφήστε κενό για απεριόριστο"
        }
        "advance.chapters" => "Κεφάλαια",
        "advance.download_range" => "Εύρος λήψης",
        "advance.always_show_download_range" => "Να εμφανίζεται πάντα η επιλογή εύρους",
        "advance.chapter_download_compatibility_mode" => "Λειτουργία συμβατότητας λήψης κεφαλαίων",
        "advance.file_time" => "Χρόνος αρχείου",
        "advance.file_time.none" => "Να μην αλλάξει",
        "advance.file_time.upload_date" => "Χρήση ημερομηνίας μεταφόρτωσης βίντεο",
        "advance.file_time.download_time" => "Χρήση ώρας λήψης",
        "advance.post_processing" => "Μετα-επεξεργασία",
        "advance.thumbnail" => "Μικρογραφία",
        "advance.download" => "Λήψη",
        "advance.embed" => "Ενσωμάτωση",
        "advance.subtitles" => "Υπότιτλοι",
        "advance.download_conversion" => "Μετατροπή μετά τη λήψη",
        "advance.enable" => "Ενεργοποίηση",
        "advance.settings" => "Ρυθμίσεις",
        "item.save_as" => "Αποθήκευση ως",
        "item.error" => "Σφάλμα",
        "item.all" => "Όλα",
        "item.queued" => "Στην ουρά",
        "item.done" => "Ολοκληρώθηκε",
        "item.failed" => "Απέτυχε",
        "item.clear_all" => "Εκκαθάριση όλων",
        "item.add_a_video_url" => "Προσθήκη URL βίντεο",
        "item.add_an_audio_url" => "Προσθήκη URL ήχου",
        "item.after_adding_choose_the_video_format_here" => "Επιλογή μορφής βίντεο",
        "item.after_adding_choose_the_audio_format_here" => "Επιλογή μορφής ήχου",
        "item.loading_thumbnail" => "Φόρτωση μικρογραφίας",
        "item.file_actions" => "Ενέργειες αρχείου",
        "item.open_file" => "Άνοιγμα αρχείου",
        "item.open_folder" => "Άνοιγμα φακέλου",
        "item.copy_path" => "Αντιγραφή διαδρομής",
        "item.file_not_found_opened_the_output_location" => {
            "Το αρχείο δεν βρέθηκε· άνοιξε η τοποθεσία εξόδου."
        }
        "item.opened_output_location" => "Άνοιξε η τοποθεσία εξόδου.",
        "item.copied_output_path" => "Αντιγράφηκε η διαδρομή εξόδου.",
        "prepare.language" => "Γλώσσα",
        "prepare.back" => "Πίσω",
        "prepare.auto_detect" => "Αυτόματη ανίχνευση",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Εγκαταστήστε τώρα τα απαιτούμενα εργαλεία ή παραλείψτε το και ρυθμίστε τα αργότερα στις Επιλογές."
        }
        "prepare.optional" => "Προαιρετικό",
        "prepare.missing" => "Λείπει",
        "prepare.install_later" => "Εγκατάσταση αργότερα",
        "prepare.downloading_100" => "Λήψη 100%",
        "prepare.extracting_100" => "Εξαγωγή 100%",
        "prepare.install_failed" => "Η εγκατάσταση απέτυχε",
        "prepare.install_all" => "Εγκατάσταση όλων",
        "prepare.reinstall" => "Επανεγκατάσταση",
        "prepare.installing" => "Εγκατάσταση",
        "prepare.skip" => "Παράλειψη",
        "prepare.install" => "Εγκατάσταση",
        "prepare.another_tool_is_already_being_installed" => "Ένα άλλο εργαλείο εγκαθίσταται ήδη.",
        "prepare.needs_attention" => "Χρειάζεται προσοχή",
        "prepare.req.app_folder.title" => "Φάκελος εφαρμογής",
        "prepare.req.app_folder.description" => {
            "Ο φορητός φάκελος πρέπει να είναι εγγράψιμος για ρυθμίσεις και βοηθητικά δεδομένα."
        }
        "prepare.req.tools_folder.title" => "Φάκελος εργαλείων",
        "prepare.req.tools_folder.description" => {
            "Η εγκατάσταση εξαρτήσεων αποθηκεύει εδώ τα yt-dlp, FFmpeg και Deno."
        }
        "prepare.req.deployment_temp.title" => "Προσωρινός φάκελος εγκατάστασης",
        "prepare.req.deployment_temp.description" => {
            "Η εξαγωγή των FFmpeg και Deno χρησιμοποιεί αυτόν τον προσωρινό φάκελο."
        }
        "prepare.req.download_cache.title" => "Κρυφή μνήμη λήψεων",
        "prepare.req.download_cache.description" => {
            "Η λειτουργία cache του yt-dlp-gui αποθηκεύει εδώ την cache του yt-dlp."
        }
        "prepare.req.output_folder.title" => "Φάκελος εξόδου",
        "prepare.req.output_folder.description" => {
            "Τα βίντεο, ο ήχος και οι υπότιτλοι αποθηκεύονται εδώ."
        }
        "prepare.req.output_folder.recommendation" => {
            "Επιλέξτε έναν έγκυρο φάκελο εξόδου από την κύρια οθόνη ή τις Επιλογές."
        }
        "prepare.req.config_file.title" => "Αρχείο ρυθμίσεων",
        "prepare.req.config_file.description" => {
            "Η εφαρμογή πρέπει να μπορεί να αποθηκεύει την παράλειψη του Prepare και τις διαδρομές εργαλείων."
        }
        "prepare.req.generic_writable_recommendation" => {
            "Επιλέξτε έναν εγγράψιμο φάκελο και ελέγξτε τα δικαιώματα."
        }
        "prepare.req.config_not_folder" => {
            "Η διαδρομή ρυθμίσεων δείχνει σε φάκελο. Επιλέξτε διαδρομή αρχείου."
        }
        "prepare.req.config_readonly" => "Το αρχείο ρυθμίσεων είναι μόνο για ανάγνωση.",
        "prepare.req.config_readonly_recommendation" => {
            "Επιτρέψτε εγγραφή στο αρχείο ρυθμίσεων ή επιλέξτε άλλο φάκελο εφαρμογής."
        }
        "prepare.req.use_folder_path" => "Επιλέξτε διαδρομή φακέλου αντί για διαδρομή αρχείου.",
        "prepare.req.move_portable_folder" => {
            "Μετακινήστε την εφαρμογή σε έναν εγγράψιμο φορητό φάκελο."
        }
        "prepare.req.avoid_protected_folder" => {
            "Μην τοποθετείτε τη φορητή εφαρμογή στο Program Files ή στον φάκελο Windows. Μετακινήστε τη στο D:\\Portable ή σε φάκελο χρήστη."
        }
        "prepare.req.move_non_synced_folder" => {
            "Μετακινήστε τη σε μη συγχρονιζόμενο φάκελο, π.χ. D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.drive_parent_exists" => {
            "Βεβαιωθείτε ότι υπάρχει η μονάδα δίσκου και ο γονικός φάκελος."
        }
        "prepare.req.permission_denied" => {
            "Μετακινήστε την εφαρμογή σε εγγράψιμο φορητό φάκελο. Αν Επιφάνεια εργασίας/Έγγραφα/Λήψεις συνεχίζουν να αποτυγχάνουν, μπορεί να το μπλοκάρει το Controlled Folder Access του Defender."
        }
        "prepare.req.file_in_use" => {
            "Κλείστε το πρόγραμμα που μπορεί να χρησιμοποιεί αυτόν τον φάκελο ή επιλέξτε άλλο φάκελο."
        }
        "prepare.req.free_disk_space" => "Ελευθερώστε χώρο στον δίσκο ή επιλέξτε άλλο δίσκο.",
        "prepare.req.path_too_long" => {
            "Μετακινήστε την εφαρμογή σε συντομότερη διαδρομή, π.χ. D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.choose_writable_portable_folder" => {
            "Επιλέξτε έναν σαφώς εγγράψιμο φορητό φάκελο και ελέγξτε ξανά."
        }
        "prepare.req.clear_write_test" => {
            "Αφαιρέστε το υπόλοιπο αρχείο δοκιμής εγγραφής και ελέγξτε ξανά."
        }
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "Αυτό το URL περιέχει και βίντεο και λίστα αναπαραγωγής"
        }
        "options.detected" => "Εντοπίστηκε ",
        "options.playlist_prompt" => "Ερώτηση λίστας αναπαραγωγής",
        "options.which_one_should_be_loaded" => "Ποιο να φορτωθεί;",
        "options.both_video_and_playlist_were_detected" => {
            "Εντοπίστηκαν και βίντεο και λίστα αναπαραγωγής"
        }
        "options.this_playlist_may_contain_many_items" => {
            "Αυτή η λίστα αναπαραγωγής μπορεί να περιέχει πολλά στοιχεία."
        }
        "options.playlist_risk.kind.channel_generated" => {
            "Λίστα καναλιού δημιουργημένη από το YouTube"
        }
        "options.playlist_risk.kind.youtube_mix_radio" => "YouTube Mix / Radio",
        "options.playlist_risk.kind.youtube_music_album" => "Άλμπουμ/συλλογή YouTube Music",
        "options.playlist_risk.kind.liked_videos" => "Βίντεο που σας αρέσουν",
        "options.playlist_risk.kind.favorites_legacy" => "Παλαιά λίστα αγαπημένων",
        "options.playlist_risk.note.channel_generated" => {
            "Χειριστείτε συντηρητικά αυτή τη λίστα καναλιού που δημιουργήθηκε από το YouTube."
        }
        "options.playlist_risk.note.youtube_mix_radio" => {
            "Αυτή η λίστα Mix / Radio μπορεί να περιέχει πολλά στοιχεία και να αλλάζει με τον χρόνο."
        }
        "options.playlist_risk.note.youtube_music_album" => {
            "Συνήθως πρόκειται για άλμπουμ ή συλλογή YouTube Music."
        }
        "options.playlist_risk.note.liked_videos" => {
            "Τα βίντεο που σας αρέσουν συνήθως απαιτούν σύνδεση ή cookies."
        }
        "options.playlist_risk.note.favorites_legacy" => {
            "Πρόκειται για παλαιό τύπο λίστας αγαπημένων και ίσως δεν είναι πλέον σταθερός."
        }
        "options.video" => "Βίντεο",
        "options.playlist" => "Λίστα αναπαραγωγής",
        "options.cancel" => "Άκυρο",
        "options.load" => "Φόρτωση",
        "options.behavior" => "Συμπεριφορά",
        "options.add_action" => "Ενέργεια προσθήκης",
        "options.download_directly" => "Άμεση λήψη",
        "options.clipboard_change" => "Αλλαγή πρόχειρου",
        "options.run_immediately" => "Άμεση εκτέλεση",
        "options.tabs" => "Καρτέλες",
        "options.log_tab" => "Καρτέλα καταγραφής",
        "options.show_log_tab" => "Εμφάνιση καταγραφής",
        "options.playlist_2" => "Λίστα αναπαραγωγής",
        "options.with_playlist" => "Με λίστα αναπαραγωγής",
        "options.ask" => "Ερώτηση",
        "options.single_video" => "Βίντεο",
        "options.full_playlist" => "[Όλα]",
        "options.high_risk_prompt" => "Προειδοποίηση υψηλού κινδύνου",
        "options.on" => "Ενεργό",
        "options.playlist_count" => "Πλήθος λίστας αναπαραγωγής",
        "options.limit" => "Όριο",
        "options.max" => "Μέγ.:",
        "options.items" => " στοιχεία",
        "options.language" => "Γλώσσα",
        "options.current_language" => "Τρέχουσα γλώσσα",
        "options.back" => "Πίσω",
        "options.choose" => "Επιλογή",
        "options.auto_detect" => "Αυτόματη ανίχνευση",
        "options.tool_paths" => "Διαδρομές εργαλείων",
        "options.file_actions" => "Ενέργειες αρχείου",
        "options.action_button" => "Κουμπί ενέργειας",
        "options.file_action.show_menu" => "Εμφάνιση μενού",
        "options.cache" => "Κρυφή μνήμη",
        "options.cache_location" => "Θέση cache",
        "options.cache_location.default" => "Προεπιλογή",
        "options.cache_usage" => "Χρήση",
        "options.cache_usage_detail" => "Σύνολο: {total} · Ήχος: {audio} · Ληγμένα: {expired}",
        "options.cache_cleanup" => "Εκκαθάριση",
        "options.cache_refresh" => "Ανανέωση",
        "options.cache_clear_expired" => "Εκκαθάριση ληγμένων",
        "options.cache_clear_audio" => "Εκκαθάριση ήχου",
        "options.cache_clear_all" => "Εκκαθάριση όλων",
        "options.appearance_window" => "Εμφάνιση & παράθυρο",
        "options.notifications" => "Ειδοποιήσεις",
        "options.enable" => "Ενεργοποίηση",
        "options.theme" => "Θέμα",
        "options.theme_mode.system" => "Ακολούθηση συστήματος",
        "options.theme_mode.light" => "Φωτεινό",
        "options.theme_mode.dark" => "Σκούρο",
        "options.theme_color" => "Χρώμα θέματος",
        "options.theme_color.off" => "Ανενεργό",
        "options.theme_color.blue" => "Μπλε",
        "options.theme_color.soft_blue" => "Απαλό μπλε",
        "options.theme_color.purple" => "Μωβ",
        "options.theme_color.pink" => "Ροζ",
        "options.theme_color.green" => "Πράσινο",
        "options.theme_color.orange" => "Πορτοκαλί",
        "options.theme_color.slate" => "Σχιστόλιθος",
        "options.ui_scale" => "Κλίμακα UI",
        "options.apply" => "Εφαρμογή",
        "options.current" => "Τρέχον",
        "options.always_on_top" => "Πάντα μπροστά",
        "options.window_position" => "Θέση παραθύρου",
        "options.remember" => "Απομνημόνευση",
        "options.window_size" => "Μέγεθος παραθύρου",
        "options.reinstall" => "Επανεγκατάσταση",
        "options.installing" => "Εγκατάσταση",
        "options.install" => "Εγκατάσταση",
        "options.executable" => "εκτελέσιμο",
        "main.controlled_by_config" => "Ελέγχεται από τη ρύθμιση: ",
        "main.controlled_by_config_2" => "Ελέγχεται από τη ρύθμιση",
        "picker.section_tab.chapters" => "Κεφάλαια",
        "picker.section_tab.time_range" => "Χρονικό εύρος",
        "picker.section_chapter_instructions" => {
            "Επιλέξτε ένα ή περισσότερα κεφάλαια. Τα συνεχόμενα κεφάλαια γίνονται μία έξοδος."
        }
        "picker.section_time_instructions" => {
            "Μετακινήστε την κεφαλή, ορίστε αρχή και τέλος και προσθέστε το εύρος."
        }
        "picker.section_time_unavailable" => {
            "Η διάρκεια του βίντεο δεν είναι διαθέσιμη, οπότε δεν μπορεί να δημιουργηθεί προσαρμοσμένο εύρος."
        }
        "picker.section_select_all" => "Επιλογή όλων",
        "picker.section_from_selected_to_end" => "Από το πρώτο επιλεγμένο έως το τέλος",
        "picker.section_set_start" => "Ορισμός αρχής",
        "picker.section_set_end" => "Ορισμός τέλους",
        "picker.section_add_range" => "Προσθήκη εύρους",
        "picker.section_no_custom_ranges" => "Δεν προστέθηκαν προσαρμοσμένα χρονικά εύρη.",
        "picker.no_chapters_available" => "Δεν υπάρχουν διαθέσιμα κεφάλαια.",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "Επιλέξτε το εύρος λήψης για αυτό το στοιχείο. Προεπιλογή είναι ολόκληρο το βίντεο."
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "Η λειτουργία συμβατότητας κεφαλαίων είναι ενεργή: οι λήψεις κεφαλαίων θα χρησιμοποιούν πιο σταθερή μορφή ενός αρχείου."
        }
        "picker.subtitles_will_not_be_downloaded" => "Οι υπότιτλοι δεν θα ληφθούν.",
        "picker.no_subtitles_are_available_for_this_video" => {
            "Δεν υπάρχουν διαθέσιμοι υπότιτλοι για αυτό το βίντεο."
        }
        "picker.no_subtitles_are_available_in_this_tab" => {
            "Δεν υπάρχουν διαθέσιμοι υπότιτλοι σε αυτή την καρτέλα."
        }
        "picker.source_language" => "Γλώσσα πηγής",
        "picker.translation_target" => "Στόχος μετάφρασης",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "Συμβουλή: οι αυτόματα μεταφρασμένοι υπότιτλοι του YouTube είναι πιθανότερο να περιοριστούν σε σχέση με τους αρχικούς υπότιτλους. Επιλέξτε «Χωρίς μετάφραση» αν χρειάζεστε μόνο το αρχικό κείμενο."
        }
        "picker.no_subtitles_are_available_for_this_source" => {
            "Δεν υπάρχουν διαθέσιμοι υπότιτλοι για αυτή την πηγή."
        }
        "picker.target" => "Στόχος",
        "picker.available_subtitles" => "Διαθέσιμοι υπότιτλοι",
        "picker.language" => "Γλώσσα",
        "picker.subtitle_tab.none" => "Χωρίς υπότιτλους",
        "picker.subtitle_tab.original" => "Αρχικοί υπότιτλοι",
        "picker.subtitle_tab.automatic" => "Αυτόματοι υπότιτλοι",
        "picker.waiting_analysis" => "Αναμονή για ανάλυση",
        "picker.audio_from_video" => "Καθορίζεται από τη μορφή βίντεο",
        "picker.not_selected" => "Δεν επιλέχθηκε",
        "picker.full_video" => "Πλήρες βίντεο",
        "picker.section_summary.chapters" => "{chapters} επιλεγμένα κεφάλαια · {outputs} έξοδοι",
        "picker.section_summary.custom" => "{ranges} χρονικά εύρη · {outputs} έξοδοι",
        "picker.section_summary.combined" => {
            "{chapters} κεφάλαια + {ranges} εύρη · {outputs} έξοδοι"
        }
        "picker.no_translation" => "Χωρίς μετάφραση",
        "picker.until_end" => "τέλος",
        "prepare.status.ready" => "Έτοιμο",
        "prepare.status.missing" => "Λείπει",
        "prepare.status.warning" => "Χρειάζεται προσοχή",
        "prepare.status.failed" => "Απέτυχε",
        "tool_install.stage.preparing" => "Προετοιμασία",
        "tool_install.stage.downloading" => "Λήψη",
        "tool_install.stage.extracting" => "Εξαγωγή",
        "tool_install.stage.installing" => "Εγκατάσταση",
        "tool_install.stage.completed" => "Ολοκληρώθηκε",
        "tool_install.stage.failed" => "Απέτυχε",
        "item.status.queued" => "Σε ουρά",
        "item.status.running" => "Εκτελείται",
        "item.status.finished" => "Ολοκληρώθηκε",
        "item.status.failed" => "Απέτυχε",
        "item.status.cancelled" => "Ακυρώθηκε",
        "processing.transcode" => "Μετατροπή",
        "transcode.graph.axis.compatibility" => "Συμβατότητα",
        "transcode.graph.axis.capacity" => "Χωρητικότητα",
        "transcode.graph.axis.resolution" => "Ανάλυση",
        "transcode.graph.axis.format" => "Μορφή",
        "transcode.graph.compatibility_scope" => "Εύρος συμβατότητας",
        "transcode.graph.capacity_target" => "Στόχος μεγέθους",
        "transcode.graph.resolution_limit" => "Όριο ανάλυσης",
        "transcode.graph.format_goal" => "Στόχος μορφής",
        "processing.video" => "Βίντεο",
        "processing.audio" => "Ήχος",
        "processing.container" => "Κοντέινερ",
        "processing.subtitle" => "Υπότιτλοι",
        "processing.choice.source" => "Αρχικό",
        "processing.subtitle.preserve" => "Αρχικό",
        "processing.subtitle.embed" => "Ενσωμάτωση",
        "processing.subtitle.burn" => "Εγγραφή στο βίντεο",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "Όλα τα αρχεία",
        "options.filter_executable" => "Εκτελέσιμο",
        "app_mode.origin" => "Λειτουργία Origin",
        "app_mode.standard" => "Τυπική λειτουργία",
        "app_mode.audio" => "Λειτουργία ήχου",
        "music.status.completed" => "Ολοκληρώθηκε",
        "music.status.resolving" => "Επίλυση",
        "music.status.buffering" => "Προσωρινή αποθήκευση",
        "music.status.ready" => "Έτοιμο",
        "music.status.caching" => "Αποθήκευση στην cache",
        "music.status.playing" => "Αναπαραγωγή",
        "music.status.paused" => "Σε παύση",
        "music.status.failed" => "Απέτυχε",
        "notification.download_complete" => "Η λήψη ολοκληρώθηκε",
        "notification.download_failed" => "Η λήψη απέτυχε",
        "notification.completed_file" => "Ολοκληρώθηκε: {file}",
        "notification.download_completed" => "Η λήψη ολοκληρώθηκε.",
        "options.music_download_format" => "Music download format",
        "options.music_download_audio_label" => "Έξοδος ήχου",
        "options.music_download_preference_best" => "Καλύτερο",
        _ => key,
    }
}
