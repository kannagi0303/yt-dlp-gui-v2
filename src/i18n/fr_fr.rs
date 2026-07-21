pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.advanced" => "Avancé",
        "tab.about" => "About",
        "about.tools" => "Versions des outils",
        "about.current_version" => "Actuelle",
        "about.latest_version" => "Dernière",
        "about.author" => "Auteur",
        "about.source" => "Source",
        "about.status" => "État",
        "about.message" => "Message",
        "about.check_updates" => "Rechercher des mises à jour",
        "about.update_all" => "Tout mettre à jour",
        "about.restart" => "Redémarrer",
        "about.open_release" => "Ouvrir la release",
        "about.install" => "Installer",
        "about.update" => "Mettre à jour",
        "about.running" => "Recherche de mises à jour...",
        "about.last_check" => "Dernière vérification :",
        "about.relative.minutes" => "{count} min",
        "about.relative.hour" => "1 heure",
        "about.relative.hours" => "{count} heures",
        "about.relative.day" => "1 jour",
        "about.relative.days" => "{count} jours",
        "about.never_checked" => "Aucune recherche de mise à jour effectuée",
        "about.no_release_notes_loaded" => {
            "Aucune note de version chargée. Lancez d'abord une recherche de mises à jour."
        }
        "about.ownership.managed_portable" => "Géré par v2",
        "about.ownership.external" => "Externe",
        "about.ownership.missing" => "Manquant",
        "about.ownership.unknown" => "Inconnu",
        "about.status.unknown" => "Non vérifié",
        "about.status.checking" => "Vérification",
        "about.status.up_to_date" => "À jour ✓",
        "about.status.update_available" => "Mise à jour disponible ↑",
        "about.status.missing" => "Manquant +",
        "about.status.downloading" => "Téléchargement",
        "about.status.downloading_percent" => "Téléchargement {percent}%",
        "about.status.staged" => "Préparé",
        "about.status.pending_restart" => "Redémarrage en attente",
        "about.status.applying" => "Application",
        "about.status.installed" => "Installé",
        "about.status.skipped" => "Ignoré",
        "about.status.failed" => "Échec !",
        "tab.options" => "Options",
        "tab.log" => "Journal",
        "main.url_hint" => "URL",
        "action.download" => "Télécharger",
        "action.add" => "Ajouter",
        "action.analyze" => "Analyser",
        "action.stop" => "Arrêter",
        "action.stopping" => "Arrêt...",
        "action.cut" => "Couper",
        "action.copy" => "Copier",
        "action.paste" => "Coller",
        "action.clear" => "Effacer",
        "item.thumbnail" => "Miniature",
        "item.thumbnail_preview" => "Aperçu de la miniature",
        "single.title" => "Titre",
        "single.description" => "Description",
        "single.info.channel" => "Chaîne",
        "single.info.date" => "Date",
        "single.info.views" => "Vues",
        "item.download_thumbnail" => "Télécharger la miniature",
        "media.video" => "Vidéo",
        "media.audio" => "Audio",
        "media.subtitle" => "Sous-titres",
        "media.section" => "Plage",
        "item.file_name" => "Nom du fichier",
        "main.target_folder" => "Dossier de sortie",
        "picker.title.video" => "Choisir le format vidéo",
        "picker.title.audio" => "Choisir le format audio",
        "picker.title.subtitle" => "Choisir les sous-titres",
        "picker.title.section" => "Choisir la plage",
        "action.back" => "Retour",
        "picker.mode.filter" => "Filtres",
        "picker.mode.table" => "Tableau",
        "action.confirm" => "Confirmer",
        "picker.empty_table" => "Aucun format à afficher",
        "picker.header.resolution" => "Résolution",
        "picker.header.range" => "Plage",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "Format",
        "picker.header.codec" => "Codec",
        "picker.header.size" => "Taille",
        "picker.header.sample_rate" => "Fréquence d’échantillonnage",
        "picker.filter.resolution" => "Résolution",
        "picker.filter.range" => "Plage",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "Codec",
        "picker.filter.sample_rate" => "Fréquence d’échantillonnage",
        "main.missing_yt_dlp_callout" => {
            "yt-dlp est manquant. Installez-le ou choisissez yt-dlp.exe dans Options."
        }
        "advance.source" => "Source",
        "advance.config" => "Configuration",
        "advance.none" => "Aucun",
        "advance.network_access" => "Réseau et accès",
        "advance.proxy" => "Proxy",
        "advance.enable_proxy" => "Activer le proxy",
        "advance.certificate" => "Certificat",
        "advance.skip_certificate_verification" => "Ignorer la vérification du certificat",
        "advance.use_cookies" => "Utiliser les cookies",
        "advance.enable_cookies" => "Activer les cookies",
        "advance.cookie_source" => "Source des cookies",
        "advance.cookie_source.auto" => "Automatique selon le site",
        "advance.cookie_source.file" => "Utiliser un fichier (cookies.txt)",
        "advance.cookie_auto" => "Automatique",
        "advance.cookie_auto_note" => {
            "Les téléchargements utilisent le Cookie enregistré qui correspond à l'URL."
        }
        "advance.cookie_rescue" => "Récupération des cookies",
        "advance.cookie_file" => "Fichier de cookies",
        "advance.get_cookie" => "Récupérer Cookie",
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
            "Ouvrir une fenêtre de navigateur dédiée pour récupérer les cookies."
        }
        "youtube_login_rescue.title" => "Récupération des cookies",
        "youtube_login_rescue.confirm_heading" => "Ouvrir une fenêtre de connexion dédiée",
        "youtube_login_rescue.confirm_body" => {
            "Une fenêtre {browser} indépendante ouvrira l'URL sans utiliser les données de votre navigateur personnel."
        }
        "youtube_login_rescue.target_url_label" => "URL du site web",
        "youtube_login_rescue.target_url_hint" => "https://www.youtube.com/",
        "youtube_login_rescue.clipboard_prefilled" => {
            "L'URL a été remplie depuis le presse-papiers."
        }
        "youtube_login_rescue.drop_url_note" => {
            "Collez une URL ou déposez un fichier .url / texte."
        }
        "youtube_login_rescue.paste_clipboard" => "Coller le presse-papiers",
        "youtube_login_rescue.cookie_note" => {
            "Connectez-vous dans cette fenêtre. Une fois les cookies trouvés, la fenêtre se fermera et ils seront appliqués automatiquement."
        }
        "youtube_login_rescue.no_browser_title" => "Aucun navigateur pris en charge trouvé",
        "youtube_login_rescue.no_browser_body" => {
            "La récupération des cookies nécessite actuellement Chrome, Brave ou Microsoft Edge. Vous pouvez toujours choisir un fichier cookies.txt manuellement."
        }
        "youtube_login_rescue.start" => "Démarrer",
        "youtube_login_rescue.opening" => "Ouverture de {browser}...",
        "youtube_login_rescue.waiting_for_cdp" => {
            "En attente de la connexion de la fenêtre de connexion {browser}..."
        }
        "youtube_login_rescue.waiting_for_cookie" => {
            "La fenêtre de connexion est connectée. En attente des cookies du site web..."
        }
        "youtube_login_rescue.cookie_exported" => "Le Cookie a été enregistré.",
        "youtube_login_rescue.cookie_exported_note" => {
            "Cookie {site} enregistré. Les téléchargements de ce site l'utiliseront automatiquement."
        }
        "youtube_login_rescue.do_not_close_note" => {
            "Gardez le navigateur de connexion ouvert pendant cette vérification."
        }
        "youtube_login_rescue.cdp_ready" => "La fenêtre de connexion est connectée.",
        "youtube_login_rescue.ready_next_step_note" => {
            "Terminez la connexion à YouTube dans le navigateur. L'exportation des cookies sera ajoutée à l'étape suivante."
        }
        "youtube_login_rescue.close_login_window" => "Fermer la fenêtre de connexion",
        "youtube_login_rescue.failed" => "Échec de la récupération des cookies",
        "youtube_login_rescue.retry" => "Réessayer",
        "advance.no_cookies_txt_selected" => "Aucun cookies.txt sélectionné",
        "advance.browse" => "Parcourir",
        "advance.select_netscape_cookies_txt" => "Sélectionner le cookies.txt Netscape",
        "advance.clear" => "Effacer",
        "advance.browser" => "Navigateur",
        "advance.default" => "Par défaut",
        "advance.external_downloader" => "Téléchargeur externe",
        "advance.use_aria2_for_faster_downloads" => {
            "Utiliser Aria2 pour des téléchargements plus rapides"
        }
        "advance.download_control" => "Contrôle du téléchargement",
        "advance.concurrent_fragments" => "Fragments simultanés",
        "advance.live_streams" => "Diffusions en direct",
        "advance.download_live_streams_from_start_experimental" => {
            "Télécharger les diffusions depuis le début (expérimental)"
        }
        "advance.1_default" => "1 (par défaut)",
        "advance.rate_limit" => "Limite de débit",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => {
            "p. ex. 2M, 800K ; laisser vide pour illimité"
        }
        "advance.chapters" => "Chapitres",
        "advance.download_range" => "Plage de téléchargement",
        "advance.always_show_download_range" => "Toujours afficher la sélection de plage",
        "advance.chapter_download_compatibility_mode" => {
            "Mode de compatibilité du téléchargement par chapitres"
        }
        "advance.file_time" => "Date du fichier",
        "advance.file_time.none" => "Ne pas modifier",
        "advance.file_time.upload_date" => "Utiliser la date de mise en ligne",
        "advance.file_time.download_time" => "Utiliser l’heure du téléchargement",
        "advance.post_processing" => "Post-traitement",
        "advance.thumbnail" => "Miniature",
        "advance.download" => "Télécharger",
        "advance.embed" => "Intégrer",
        "advance.subtitles" => "Sous-titres",
        "advance.download_conversion" => "Convertir après le téléchargement",
        "advance.enable" => "Activer",
        "advance.settings" => "Paramètres",
        "item.save_as" => "Enregistrer sous",
        "item.error" => "Erreur",
        "item.all" => "Tout",
        "item.queued" => "En file",
        "item.done" => "Terminé",
        "item.failed" => "Échec",
        "item.clear_all" => "Tout effacer",
        "item.add_a_video_url" => "Ajouter une URL vidéo",
        "item.add_an_audio_url" => "Ajouter une URL audio",
        "item.after_adding_choose_the_video_format_here" => "Choisir le format vidéo",
        "item.after_adding_choose_the_audio_format_here" => "Choisir le format audio",
        "item.loading_thumbnail" => "Chargement de la miniature",
        "item.file_actions" => "Actions de fichier",
        "item.open_file" => "Ouvrir le fichier",
        "item.open_folder" => "Ouvrir le dossier",
        "item.copy_path" => "Copier le chemin",
        "item.file_not_found_opened_the_output_location" => {
            "Fichier introuvable ; emplacement de sortie ouvert."
        }
        "item.opened_output_location" => "Emplacement de sortie ouvert.",
        "item.copied_output_path" => "Chemin de sortie copié.",
        "prepare.language" => "Langue",
        "prepare.back" => "Retour",
        "prepare.auto_detect" => "Détection automatique",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Installez les outils requis maintenant, ou ignorez cette étape et configurez-les plus tard dans Options."
        }
        "prepare.optional" => "Facultatif",
        "prepare.missing" => "Manquant",
        "prepare.install_later" => "Installer plus tard",
        "prepare.downloading_100" => "Téléchargement 100 %",
        "prepare.extracting_100" => "Extraction 100 %",
        "prepare.install_failed" => "Échec de l’installation",
        "prepare.install_all" => "Tout installer",
        "prepare.reinstall" => "Réinstaller",
        "prepare.installing" => "Installation",
        "prepare.skip" => "Ignorer",
        "prepare.install" => "Installer",
        "prepare.another_tool_is_already_being_installed" => {
            "Un autre outil est déjà en cours d’installation."
        }
        "prepare.needs_attention" => "Attention requise",
        "prepare.req.app_folder.title" => "Dossier de l’application",
        "prepare.req.app_folder.description" => {
            "Le dossier portable doit être accessible en écriture pour les paramètres et les données de support."
        }
        "prepare.req.tools_folder.title" => "Dossier des outils",
        "prepare.req.tools_folder.description" => {
            "Le déploiement des dépendances y stocke yt-dlp, FFmpeg et Deno."
        }
        "prepare.req.deployment_temp.title" => "Temporaire de déploiement",
        "prepare.req.deployment_temp.description" => {
            "L’extraction de FFmpeg et Deno utilise ce dossier temporaire."
        }
        "prepare.req.download_cache.title" => "Cache de téléchargement",
        "prepare.req.download_cache.description" => {
            "Le mode cache de yt-dlp-gui y stocke le cache de yt-dlp."
        }
        "prepare.req.output_folder.title" => "Dossier de sortie",
        "prepare.req.output_folder.description" => {
            "Les vidéos, l’audio et les sous-titres sont enregistrés ici."
        }
        "prepare.req.output_folder.recommendation" => {
            "Choisissez un dossier de sortie valide depuis l’écran principal ou les options."
        }
        "prepare.req.config_file.title" => "Fichier de configuration",
        "prepare.req.config_file.description" => {
            "L’application doit pouvoir enregistrer l’état d’ignorance de Prepare et les chemins des outils."
        }
        "prepare.req.generic_writable_recommendation" => {
            "Choisissez un dossier accessible en écriture et vérifiez les autorisations."
        }
        "prepare.req.config_not_folder" => {
            "Le chemin de configuration pointe vers un dossier. Choisissez un chemin de fichier."
        }
        "prepare.req.config_readonly" => "Le fichier de configuration est en lecture seule.",
        "prepare.req.config_readonly_recommendation" => {
            "Autorisez l’écriture dans le fichier de configuration ou choisissez un autre dossier d’application."
        }
        "prepare.req.use_folder_path" => {
            "Choisissez un chemin de dossier plutôt qu’un chemin de fichier."
        }
        "prepare.req.move_portable_folder" => {
            "Déplacez l’application vers un dossier portable accessible en écriture."
        }
        "prepare.req.avoid_protected_folder" => {
            "Ne placez pas l’application portable dans Program Files ni dans le dossier Windows. Déplacez-la vers D:\\Portable ou un dossier utilisateur."
        }
        "prepare.req.move_non_synced_folder" => {
            "Déplacez-la vers un dossier non synchronisé, par exemple D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.drive_parent_exists" => {
            "Vérifiez que le lecteur et le dossier parent existent."
        }
        "prepare.req.permission_denied" => {
            "Déplacez l’application vers un dossier portable accessible en écriture. Si Bureau/Documents/Téléchargements échouent encore, l’accès contrôlé aux dossiers de Defender peut la bloquer."
        }
        "prepare.req.file_in_use" => {
            "Fermez le programme qui utilise peut-être ce dossier ou choisissez un autre dossier."
        }
        "prepare.req.free_disk_space" => {
            "Libérez de l’espace disque ou choisissez un autre disque."
        }
        "prepare.req.path_too_long" => {
            "Déplacez l’application vers un chemin plus court, par exemple D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.choose_writable_portable_folder" => {
            "Choisissez un dossier portable clairement accessible en écriture, puis vérifiez à nouveau."
        }
        "prepare.req.clear_write_test" => {
            "Supprimez le fichier de test d’écriture restant, puis vérifiez à nouveau."
        }
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "Cette URL contient à la fois une vidéo et une playlist"
        }
        "options.detected" => "Détecté ",
        "options.playlist_prompt" => "Invite de playlist",
        "options.which_one_should_be_loaded" => "Lequel faut-il charger ?",
        "options.both_video_and_playlist_were_detected" => {
            "Une vidéo et une playlist ont été détectées"
        }
        "options.this_playlist_may_contain_many_items" => {
            "Cette playlist peut contenir de nombreux éléments."
        }
        "options.playlist_risk.kind.channel_generated" => "Playlist de chaîne générée par YouTube",
        "options.playlist_risk.kind.youtube_mix_radio" => "YouTube Mix / Radio",
        "options.playlist_risk.kind.youtube_music_album" => "Album/collection YouTube Music",
        "options.playlist_risk.kind.liked_videos" => "Vidéos aimées",
        "options.playlist_risk.kind.favorites_legacy" => "Ancienne playlist de favoris",
        "options.playlist_risk.note.channel_generated" => {
            "Traiter cette playlist de chaîne générée par YouTube avec prudence."
        }
        "options.playlist_risk.note.youtube_mix_radio" => {
            "Cette playlist Mix / Radio peut contenir de nombreux éléments et changer au fil du temps."
        }
        "options.playlist_risk.note.youtube_music_album" => {
            "Il s’agit généralement d’un album ou d’une collection YouTube Music."
        }
        "options.playlist_risk.note.liked_videos" => {
            "Les vidéos aimées nécessitent généralement une connexion ou des cookies."
        }
        "options.playlist_risk.note.favorites_legacy" => {
            "Il s’agit d’un ancien type de playlist de favoris et il peut ne plus être stable."
        }
        "options.video" => "Vidéo",
        "options.playlist" => "Liste de lecture",
        "options.cancel" => "Annuler",
        "options.load" => "Charger",
        "options.behavior" => "Comportement",
        "options.add_action" => "Action d’ajout",
        "options.download_directly" => "Télécharger directement",
        "options.clipboard_change" => "Changement du presse-papiers",
        "options.run_immediately" => "Exécuter immédiatement",
        "options.tabs" => "Onglets",
        "options.log_tab" => "Onglet Journal",
        "options.show_log_tab" => "Afficher le journal",
        "options.playlist_2" => "Liste de lecture",
        "options.with_playlist" => "Avec playlist",
        "options.ask" => "Demander",
        "options.single_video" => "Vidéo unique",
        "options.full_playlist" => "Liste complète",
        "options.high_risk_prompt" => "Invite à risque élevé",
        "options.on" => "Activé",
        "options.playlist_count" => "Nombre de playlists",
        "options.limit" => "Limite",
        "options.max" => "Max :",
        "options.items" => " éléments",
        "options.language" => "Langue",
        "options.current_language" => "Langue actuelle",
        "options.back" => "Retour",
        "options.choose" => "Choisir",
        "options.auto_detect" => "Détection automatique",
        "options.tool_paths" => "Chemins des outils",
        "options.file_actions" => "Actions de fichier",
        "options.action_button" => "Bouton d’action",
        "options.file_action.show_menu" => "Afficher le menu",
        "options.cache" => "Cache",
        "options.cache_location" => "Emplacement du cache",
        "options.cache_location.default" => "Par défaut",
        "options.cache_usage" => "Utilisation",
        "options.cache_usage_detail" => "Total : {total} · Audio : {audio} · Expiré : {expired}",
        "options.cache_cleanup" => "Nettoyage",
        "options.cache_refresh" => "Actualiser",
        "options.cache_clear_expired" => "Effacer les expirés",
        "options.cache_clear_audio" => "Effacer l’audio",
        "options.cache_clear_all" => "Tout effacer",
        "options.appearance_window" => "Apparence et fenêtre",
        "options.notifications" => "Notifications",
        "options.enable" => "Activer",
        "options.theme" => "Thème",
        "options.theme_mode.system" => "Suivre le système",
        "options.theme_mode.light" => "Clair",
        "options.theme_mode.dark" => "Sombre",
        "options.theme_color" => "Couleur du thème",
        "options.theme_color.off" => "Désactivé",
        "options.theme_color.blue" => "Bleu",
        "options.theme_color.soft_blue" => "Bleu doux",
        "options.theme_color.purple" => "Violet",
        "options.theme_color.pink" => "Rose",
        "options.theme_color.green" => "Vert",
        "options.theme_color.orange" => "Orange",
        "options.theme_color.slate" => "Ardoise",
        "options.ui_scale" => "Échelle de l’interface",
        "options.apply" => "Appliquer",
        "options.current" => "Actuel",
        "options.always_on_top" => "Toujours au premier plan",
        "options.window_position" => "Position de la fenêtre",
        "options.remember" => "Mémoriser",
        "options.window_size" => "Taille de la fenêtre",
        "options.reinstall" => "Réinstaller",
        "options.installing" => "Installation",
        "options.install" => "Installer",
        "options.executable" => "exécutable",
        "main.controlled_by_config" => "Contrôlé par la configuration : ",
        "main.controlled_by_config_2" => "Contrôlé par la configuration",
        "picker.section_tab.chapters" => "Chapitres",
        "picker.section_tab.time_range" => "Plage temporelle",
        "picker.section_chapter_instructions" => {
            "Sélectionnez un ou plusieurs chapitres. Les chapitres adjacents forment une seule sortie."
        }
        "picker.section_time_instructions" => {
            "Déplacez la tête de lecture, définissez le début et la fin, puis ajoutez la plage."
        }
        "picker.section_time_unavailable" => {
            "La durée de la vidéo est indisponible ; aucune plage personnalisée ne peut être créée."
        }
        "picker.section_select_all" => "Tout sélectionner",
        "picker.section_from_selected_to_end" => "Du premier sélectionné à la fin",
        "picker.section_set_start" => "Définir le début",
        "picker.section_set_end" => "Définir la fin",
        "picker.section_add_range" => "Ajouter la plage",
        "picker.section_no_custom_ranges" => "Aucune plage temporelle personnalisée.",
        "picker.no_chapters_available" => "Aucun chapitre disponible.",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "Choisissez la plage à télécharger pour cet élément. Par défaut, toute la vidéo est utilisée."
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "Le mode de compatibilité des chapitres est activé : les téléchargements par chapitre utiliseront un format de fichier unique plus stable."
        }
        "picker.subtitles_will_not_be_downloaded" => "Les sous-titres ne seront pas téléchargés.",
        "picker.no_subtitles_are_available_for_this_video" => {
            "Aucun sous-titre n’est disponible pour cette vidéo."
        }
        "picker.no_subtitles_are_available_in_this_tab" => {
            "Aucun sous-titre n’est disponible dans cet onglet."
        }
        "picker.source_language" => "Langue source",
        "picker.translation_target" => "Langue cible",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "Astuce : les sous-titres YouTube traduits automatiquement sont plus susceptibles d’être limités que les sous-titres originaux. Choisissez « Aucune traduction » si vous avez seulement besoin du texte source."
        }
        "picker.no_subtitles_are_available_for_this_source" => {
            "Aucun sous-titre n’est disponible pour cette source."
        }
        "picker.target" => "Cible",
        "picker.available_subtitles" => "Sous-titres disponibles",
        "picker.language" => "Langue",
        "picker.subtitle_tab.none" => "Aucun sous-titre",
        "picker.subtitle_tab.original" => "Sous-titres originaux",
        "picker.subtitle_tab.automatic" => "Sous-titres automatiques",
        "picker.waiting_analysis" => "En attente de l’analyse",
        "picker.audio_from_video" => "Défini par le format vidéo",
        "picker.not_selected" => "Non sélectionné",
        "picker.full_video" => "Vidéo complète",
        "picker.section_summary.chapters" => {
            "{chapters} chapitres sélectionnés · {outputs} sorties"
        }
        "picker.section_summary.custom" => "{ranges} plages temporelles · {outputs} sorties",
        "picker.section_summary.combined" => {
            "{chapters} chapitres + {ranges} plages · {outputs} sorties"
        }
        "picker.no_translation" => "Aucune traduction",
        "picker.until_end" => "fin",
        "prepare.status.ready" => "Prêt",
        "prepare.status.missing" => "Manquant",
        "prepare.status.warning" => "Attention requise",
        "prepare.status.failed" => "Échec",
        "tool_install.stage.preparing" => "Préparation",
        "tool_install.stage.downloading" => "Téléchargement",
        "tool_install.stage.extracting" => "Extraction",
        "tool_install.stage.installing" => "Installation",
        "tool_install.stage.completed" => "Terminé",
        "tool_install.stage.failed" => "Échec",
        "item.status.queued" => "En file",
        "item.status.running" => "En cours",
        "item.status.finished" => "Terminé",
        "item.status.failed" => "Échec",
        "item.status.cancelled" => "Annulé",
        "processing.transcode" => "Transcoder",
        "transcode.graph.axis.compatibility" => "Compatibilité",
        "transcode.graph.axis.capacity" => "Capacité",
        "transcode.graph.axis.resolution" => "Résolution",
        "transcode.graph.axis.format" => "Format",
        "transcode.graph.compatibility_scope" => "Cible de compatibilité",
        "transcode.graph.capacity_target" => "Objectif de taille",
        "transcode.graph.resolution_limit" => "Limite de résolution",
        "transcode.graph.format_goal" => "Objectif de format",
        "processing.video" => "Vidéo",
        "processing.audio" => "Audio",
        "processing.container" => "Conteneur",
        "processing.subtitle" => "Sous-titres",
        "processing.choice.source" => "Original",
        "processing.subtitle.preserve" => "Original",
        "processing.subtitle.embed" => "Intégrer",
        "processing.subtitle.burn" => "Incruster",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "Tous les fichiers",
        "options.filter_executable" => "Exécutable",
        "app_mode.origin" => "Mode Origin",
        "app_mode.standard" => "Mode standard",
        "app_mode.audio" => "Mode audio",
        "music.status.completed" => "Terminé",
        "music.status.resolving" => "Résolution",
        "music.status.buffering" => "Mise en mémoire tampon",
        "music.status.ready" => "Prêt",
        "music.status.caching" => "Mise en cache",
        "music.status.playing" => "Lecture",
        "music.status.paused" => "En pause",
        "music.status.failed" => "Échec",
        "notification.download_complete" => "Téléchargement terminé",
        "notification.download_failed" => "Échec du téléchargement",
        "notification.completed_file" => "Terminé : {file}",
        "notification.download_completed" => "Téléchargement terminé.",
        "options.music_download_format" => "Music download format",
        "options.music_download_audio_label" => "Sortie audio",
        "options.music_download_preference_best" => "Meilleur",
        _ => key,
    }
}
