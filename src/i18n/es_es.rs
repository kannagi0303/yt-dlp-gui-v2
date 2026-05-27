pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.prepare" => "Preparar",
        "tab.main" => "Principal",
        "tab.advanced" => "Avanzado",
        "tab.options" => "Opciones",
        "main.url_hint" => "Pegar URL",
        "action.download" => "Descargar",
        "action.add" => "＋ Añadir",
        "action.stop" => "Detener",
        "action.stopping" => "Deteniendo",
        "action.cut" => "Cortar",
        "action.copy" => "Copiar",
        "action.paste" => "Pegar",
        "action.clear" => "Borrar",
        "item.thumbnail" => "Miniatura",
        "item.thumbnail_preview" => "Previo miniatura",
        "notification.download_finished" => "Descarga completada",
        "notification.download_failed" => "Descarga fallida",
        "notification.download_finished_detail_prefix" => "Completado: ",
        "notification.download_finished_detail" => "Descarga completada.",
        "notification.windows_toast_windows_only" => {
            "Windows Toast solo es compatible con Windows."
        }
        "media.video" => "Vídeo",
        "media.audio" => "Audio",
        "media.subtitle" => "Subtítulos",
        "media.section" => "Rango",
        "item.file_name" => "Archivo",
        "main.target_folder" => "Carpeta de salida",
        "picker.title.video" => "Seleccionar formato de vídeo",
        "picker.title.audio" => "Seleccionar formato de audio",
        "picker.title.subtitle" => "Seleccionar subtítulos",
        "picker.title.section" => "Seleccionar sección",
        "action.back" => "Atrás",
        "picker.mode.filter" => "Filtros",
        "picker.mode.table" => "Tabla",
        "action.confirm" => "Confirmar",
        "picker.empty_table" => "No hay elementos para mostrar",
        "picker.header.resolution" => "Resolución",
        "picker.header.range" => "Rango",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "Formato",
        "picker.header.codec" => "Códec",
        "picker.header.size" => "Tamaño",
        "picker.header.sample_rate" => "Frecuencia",
        "picker.filter.resolution" => "Resolución",
        "picker.filter.range" => "Rango",
        "picker.filter.fps" => "CPS",
        "picker.filter.codec" => "Códec",
        "picker.filter.sample_rate" => "Frecuencia",
        "main.tooltip.missing_yt_dlp" => {
            "Falta yt-dlp. Instalar o seleccionar yt-dlp.exe en Opciones."
        }
        "advance.source" => "Entrada",
        "advance.config" => "Configuración",
        "advance.none" => "Ninguna",
        "advance.network_access" => "Red",
        "advance.proxy" => "Proxy",
        "advance.enable_proxy" => "Activar proxy",
        "advance.certificate" => "Certificado",
        "advance.skip_certificate_verification" => "Omitir verificación certificado",
        "advance.use_cookies" => "Usar cookies",
        "advance.enable_cookies" => "Activar cookies",
        "advance.cookie_source" => "Cookies",
        "advance.cookie_file" => "Archivo cookies",
        "advance.no_cookies_txt_selected" => "No se ha seleccionado cookies.txt",
        "advance.browse" => "Buscar",
        "advance.select_netscape_cookies_txt" => "Seleccionar Netscape cookies.txt",
        "advance.clear" => "Borrar",
        "advance.browser" => "Navegador",
        "advance.default" => "Predeterminado",
        "advance.external_downloader" => "Descargador externo",
        "advance.use_aria2_for_faster_downloads" => "Usar Aria2 para descargas más rápidas",
        "advance.download_control" => "Control descargas",
        "advance.concurrent_fragments" => "Fragmentos actuales",
        "advance.1_default" => "1 (predeterminado)",
        "advance.rate_limit" => "Límite transferencia",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => {
            "e.j. 2M, 800K; dejar vacío para ilimitado"
        }
        "advance.chapters" => "Capítulos",
        "advance.chapter_download_compatibility_mode" => "Modo compatibilidad de descarga de capítulos",
        "advance.file_time" => "Rango tiempo",
        "advance.post_processing" => "Posprocesado",
        "advance.thumbnail" => "Miniatura",
        "advance.download" => "Descargar",
        "advance.embed" => "Integrar",
        "advance.subtitles" => "Subtítulos",
        "item.stop_download" => "Detener descarga",
        "item.remove" => "Eliminar",
        "item.save_as" => "Guardar como",
        "item.error" => "Error",
        "item.all" => "Todo",
        "item.queued" => "En cola",
        "item.done" => "Completado",
        "item.failed" => "Error",
        "item.clear_all" => "Borrar todo",
        "item.add_a_video_url" => "Añadir una URL de vídeo",
        "item.after_adding_choose_the_video_format_here" => {
            "Después de añadir, selecciona el formato de video aquí."
        }
        "item.after_adding_choose_the_audio_format_here" => {
            "Después de añadir, selecciona el formato de audio aquí."
        }
        "item.loading_thumbnail" => "Cargando miniatura",
        "item.file_actions" => "Acciones archivo",
        "item.open_file" => "Abrir archivo",
        "item.open_folder" => "Abrir carpeta",
        "item.copy_path" => "Copiar ruta",
        "item.opened_output_file" => "Abrir archivo de salida",
        "item.file_not_found_opened_the_output_location" => {
            "Archivo no encontrado; abierta la ubicación de salida."
        }
        "item.opened_output_location" => "Abrir ubicación de archivo",
        "item.copied_output_path" => "Ruta de salida copiada.",
        "item.file_actions_are_available_after_download_co" => {
            "Las acciones de los archivos están disponibles una vez completada la descarga"
        }
        "prepare.language" => "Idioma",
        "prepare.back" => "Atrás",
        "prepare.choose" => "Seleccionar",
        "prepare.auto_detect" => "Detectar automáticamente",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Instalar las herramientas necesarias ahora u omitir para más tarde en Opciones."
        }
        "prepare.required" => "Requerido",
        "prepare.recommended" => "Recomendado",
        "prepare.optional" => "Opcional",
        "prepare.missing" => "Falta",
        "prepare.install_later" => "Instalar más tarde",
        "prepare.downloading_100" => "Descargando 100%",
        "prepare.extracting_100" => "Extrayendo 100%",
        "prepare.install_failed" => "Error en la instalación",
        "prepare.install_all" => "Instalar todo",
        "prepare.reinstall" => "Reinstalar",
        "prepare.installing" => "Instalando",
        "prepare.skip" => "Omitir",
        "prepare.install" => "Instalar",
        "prepare.another_tool_is_already_being_installed" => {
            "Ya se está instalando otra herramienta."
        }
        "prepare.needs_attention" => "Requiere atención",
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "Esta URL contiene un vídeo y una lista de reproducción."
        }
        "options.detected" => "Detectado ",
        "options.playlist_prompt" => "Lista de reproducción",
        "options.which_one_should_be_loaded" => "¿Cuál debería descargar?",
        "options.both_video_and_playlist_were_detected" => "Se ha detectado tanto el vídeo como la lista de reproducción.",
        "options.this_playlist_may_contain_many_items" => "Esta lista de reproducción puede contener muchos elementos.",
        "options.video" => "Vídeo",
        "options.playlist" => "Lista de reproducción",
        "options.cancel" => "Cancelar",
        "options.load" => "Cargar",
        "options.behavior" => "Comportamiento",
        "options.add_action" => "Añadir acción",
        "options.download_directly" => "Descargar directamente",
        "options.clipboard_change" => "Cambio del portapapeles",
        "options.run_immediately" => "Ejecutar",
        "options.playlist_2" => "Lista de reproducción",
        "options.with_playlist" => "Con lista de reproducción",
        "options.ask" => "Preguntar",
        "options.single_video" => "Vídeo único",
        "options.full_playlist" => "Lista completa",
        "options.high_risk_prompt" => "Alto riesgo",
        "options.on" => "Activado",
        "options.playlist_count" => "Recuento lista de reproducción",
        "options.limit" => "Límite",
        "options.max" => "Max:",
        "options.items" => " elementos",
        "options.language" => "Idioma",
        "options.current_language" => "Idioma actual",
        "options.back" => "Atrás",
        "options.choose" => "Seleccionar",
        "options.auto_detect" => "Detectar automáticamente",
        "options.tool_paths" => "Ruta herramientas",
        "options.file_actions" => "Acciones archivo",
        "options.action_button" => "Botón acción",
        "options.cache" => "Caché",
        "options.cache_location" => "Ubicación del caché",
        "options.appearance_window" => "Apariencia ventana",
        "options.notifications" => "Notificaciones",
        "options.enable" => "Activar",
        "options.ui_scale" => "Escala interfaz",
        "options.apply" => "Aplicar",
        "options.current" => "Actual",
        "options.always_on_top" => "Siempre visible",
        "options.window_position" => "Posición ventana",
        "options.remember" => "Recordar",
        "options.window_size" => "Tamaño ventana",
        "options.reinstall" => "Reinstalar",
        "options.installing" => "Instalando",
        "options.browse" => "Buscar",
        "options.install" => "Instalar",
        "options.file_not_found" => "Archivo no encontrado: ",
        "options.will_install_to" => "Se instalará en: ",
        "options.another_tool_is_being_installed_please_wait" => {
            "Se está instalando otra herramienta. Espere a que termine."
        }
        "options.install_to" => "Instalar en: ",
        "options.executable" => "executable",
        "main.clipboard_monitor_on_the_next_youtube_url_ch" => {
            "Monitor del portapapeles: activado. El próximo cambio de URL de YouTube se agregará inmediatamente."
        }
        "main.clipboard_monitor_on_the_next_youtube_url_ch_2" => {
            "Monitor del portapapeles: activado. El próximo cambio de la URL de YouTube llenará el campo URL."
        }
        "main.clipboard_monitor_off_turning_it_on_only_mem" => {
            "Monitor del portapapeles: desactivado. Al activarlo sólo se memoriza el portapapeles actual; se gestionará en el siguiente cambio."
        }
        "main.controlled_by_config" => "Controlado por config: ",
        "main.controlled_by_config_2" => "Controlado por config",
        "main.actual_path" => "Ruta actual: ",
        "picker.no_chapters_available" => "No hay capítulos disponibles",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "Seleccionar que descargar. El valor predeterminado es el vídeo completo."
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "El modo de compatibilidad de capítulos está activado: las descargas de capítulos utilizarán un formato de archivo único más estable."
        }
        "picker.subtitles_will_not_be_downloaded" => "Los subtítulos no se descargarán.",
        "picker.no_subtitles_are_available_for_this_video" => {
            "No hay subtítulos disponibles para este vídeo."
        }
        "picker.no_subtitles_are_available_in_this_tab" => {
            "No hay subtítulos disponibles en esta pestaña."
        }
        "picker.source_language" => "Idioma de origen",
        "picker.translation_target" => "Idioma de destino",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "Consejo: Los subtítulos autotraducidos de YouTube tienen más probabilidades de estar limitados que los subtítulos originales. Seleccionar "Sin traducción" si sólo necesita el texto original."
        }
        "picker.no_subtitles_are_available_for_this_source" => {
            "No hay subtítulos disponibles para esta entrada."
        }
        "picker.target" => "Destino",
        "picker.available_subtitles" => "Subtítulos disponibles",
        "picker.language" => "Idioma",
        "picker.subtitle_tab.none" => "Sin subtítulos",
        "picker.subtitle_tab.original" => "Subtítulos originales",
        "picker.subtitle_tab.automatic" => "Subtítulos automáticos",
        "config.youtube_playlist_mode.ask" => "Preguntar",
        "config.youtube_playlist_mode.video" => "Vídeo",
        "config.youtube_playlist_mode.ignore" => "Ignorar",
        "config.output_action.menu" => "Mostrar menú",
        "config.output_action.open_folder" => "Abrir carpeta",
        "config.output_action.open_file" => "Abrir archivo",
        "tools.file_time.none" => "No cambiar",
        "tools.file_time.use_upload_date" => "Usar fecha de subida del video",
        "tools.file_time.use_download_time" => "Usar fecha de descarga",
        "tools.file_time.none_hint" => {
            "No pasa --mtime/--no-mtime ni modifica el tiempo del archivo final."
        }
        "tools.file_time.use_upload_date_hint" => {
            "Después de que yt-dlp informe la ruta final, ajusta el tiempo a la fecha de subida del video."
        }
        "tools.file_time.use_download_time_hint" => "--no-mtime",
        "tools.cache_mode.default" => "Predeterminado",
        "tools.cache_mode.v2_cache" => "yt-dlp-gui",
        "tools.cache_mode.windows_temp" => "Windows",
        "tools.subtitle_source.none" => "Sin subtítulos",
        "tools.subtitle_source.original" => "Subtítulos originales",
        "tools.subtitle_source.automatic" => "Subtítulos automáticos",
        "tools.quality.best" => "Mejor",
        "tools.quality.audio_only" => "Solo audio",
        "tools.youtube_playlist.channel_generated" => "Lista de reproducción generada por YouTube",
        "tools.youtube_playlist.mix_radio" => "YouTube Mix / Radio",
        "tools.youtube_playlist.music_album" => "YouTube Music album/collection",
        "tools.youtube_playlist.liked_videos" => "Vídeos que me gustan",
        "tools.youtube_playlist.favorites_legacy" => "Lista de reproducción de favoritos heredada",
        "prepare.severity.required" => "Elemento requerido",
        "prepare.severity.recommended" => "Elemento recomendado",
        "prepare.severity.optional" => "Elemento opcional",
        "prepare.status.ready" => "Listo",
        "prepare.status.missing" => "Falta",
        "prepare.status.warning" => "Requiere atención",
        "prepare.status.failed" => "Error",
        "tool_install.stage.preparing" => "Preparando",
        "tool_install.stage.downloading" => "Descargando",
        "tool_install.stage.extracting" => "Extrayendo",
        "tool_install.stage.installing" => "Instalación",
        "tool_install.stage.completed" => "Completado",
        "tool_install.stage.failed" => "Error",
        "domain.media.video" => "video",
        "domain.media.audio" => "audio",
        "domain.media.muxed" => "multiplexado",
        "domain.media.subtitle" => "subtítulo",
        "domain.media.other" => "otro",
        "domain.quality.best" => "Mejor",
        "domain.quality.audio_only" => "Solo audio",
        "prepare.severity.short.required" => "Necesario",
        "prepare.severity.short.recommended" => "Recomendado",
        "prepare.severity.short.optional" => "Opcional",
        "item.status.idle" => "No iniciado",
        "item.status.queued" => "En cola",
        "item.status.running" => "Ejecutando",
        "item.status.finished" => "Hecho",
        "item.status.failed" => "Error",
        "item.status.cancelled" => "Cancelado",
        "item.status.waiting_analysis" => "Esperando análisis",
        "item.status.analyzing" => "Analizando",
        "item.status.analysis_failed" => "Error al analizar",
        "picker.waiting_analysis" => "Esperando análisis",
        "picker.audio_from_video" => "Decidido por formato de vídeo",
        "picker.not_selected" => "No seleccionado",
        "picker.full_video" => "Vídeo completo",
        "picker.no_translation" => "Sin traducción",
        "picker.until_end" => "final",
        "state.clipboard_detected_url" => "Detectada una URL de YouTube desde el portapapeles",
        "state.no_url_to_analyze" => "No hay ninguna URL para analizar.",
        "state.analyzing_source" => "Analizando: {source}",
        "state.batch_add_running" => "Añadir por lotes aún se está ejecutando.",
        "state.no_url_to_add" => "No hay ninguna URL para añadir.",
        "state.video_url_contains_playlist" => {
            "Se ha detectado una URL de video que también contiene una lista de reproducción."
        }
        "state.detected_high_risk_playlist" => "Detectada lista de reproducción de YouTube de alto riesgo: {kind}",
        "state.no_url_to_download_now" => "No hay ninguna URL para descargar.",
        "state.download_now_single_video_only" => {
            "Descargar ahora solo maneja una URL de video."
        }
        "state.added_ready_download_now" => "Añadido y listo para descargar: {title}",
        "state.current_action_cancelled" => "Acción cancelada.",
        "state.stopping_batch_add" => "Deteniendo añadir por lotes...",
        "state.retrying_analysis_cookie" => "Reintentar el análisis con cookies: {source}",
        "state.batch_no_new_items" => "No se han encontrado elementos nuevos en el lote.",
        "state.playlist_added_limited" => {
            "Añadido {count} elementos por lotes de la lista de reproducción (se aplica límite)."
        }
        "state.batch_added_title" => "Añadido a lote: {title}",
        "state.playlist_added" => "Añadir {count} elementos por lotes de la lista de reproducción.",
        "state.batch_add_cancelled" => "Cancelado añadir a lote.",
        "state.batch_add_cancelled_with_count" => "Cancelado añadir a lote; {count} elementos añadidos.",
        "state.batch_add_interrupted" => "Interrumpido añadir a lotes..",
        "state.deployment_complete" => "Implementación completa",
        "state.tool_deployed" => "{tool} descargado e implementado.",
        "state.tool_deploy_failed" => "{tool} la implementación ha fallado: {error}",
        "state.download_item_fallback" => "Descargar elemento",
        "state.download_stopped" => "Descarga detenida",
        "state.no_url_to_add_batch" => "No hay ninguna URL para añadir al lote.",
        "state.batch_input_added" => "Añadido {count} elementos en cola de entrada por lotes.",
        "state.no_url_to_download" => "No hay URL para descargar.",
        "state.download_already_running" => {
            "Ya se está ejecutando una descarga. Espere a que finalice."
        }
        "state.no_runnable_batch_items" => "No hay elementos por lotes ejecutables.",
        "state.no_download_to_stop" => "No hay ninguna descarga que detener.",
        "state.stopping_download" => "Deteniendo la descarga...",
        "state.target_download_not_found" => "No se ha encontrado el elemento de descarga de destino.",
        "state.analyze_before_download" => "Analizar el vídeo antes de iniciar la descarga.",
        "state.downloading_title" => "Descargando: {title}",
        "state.downloading_title_aria2_fallback" => {
            "Descargando: {title} (Aria2 no encontrado; usando la descarga nativa de yt-dlp)"
        }
        "state.target_export_not_found" => "No se ha encontrado el elemento a exportar.",
        "state.cannot_export_item" => "Este elemento no se puede exportar en este momento.",
        "state.analyze_before_export" => "Analizar el vídeo antes de exportar.",
        "state.choose_subtitles_before_export" => "Seleccionar los subtítulos antes de exportar.",
        "state.specify_file_extension" => "Especificar una extensión de archivo.",
        "state.exporting_video" => "Exportando vídeo: {title}",
        "state.exporting_audio" => "Exportando audio: {title}",
        "state.exporting_subtitles" => "Exportando subtítulos: {title}",
        "state.cleared_queue" => "Cola borrada.",
        "state.cannot_remove_running_item" => "Los elementos en ejecución no se pueden eliminar.",
        "state.removed_item" => "Eliminado: {title}",
        "state.controlled_by_config" => "Controlado por config",
        "state.install_blocked_by_prepare" => "Manejar {items} antes de instalar herramientas de dependencia.",
        "state.tool_deployment_running" => "{tool} implementación aún está en ejecución.",
        "state.no_tools_to_install" => "No hay herramientas para instalar.",
        "state.no_selected_tools_to_install" => "No hay elementos desplegables seleccionados.",
        "state.prepare_skipped" => {
            "Omitida la página de preparación. Puede manejar la implementación de dependencias más adelante en Opciones."
        }
        "state.skip_failed" => "Error omitir: {error}",
        "state.preparing_deployment" => "Preparando implementación",
        "state.tool_downloading_deploying" => "{tool} descargando e implementando...",
        "state.found" => "Encontrado",
        "state.not_found" => "No encontrado",
        "state.clipboard_monitor_enabled_auto_add" => {
            "Monitor de portapapeles activado; El próximo cambio de la URL de YouTube se añadirá inmediatamente."
        }
        "state.clipboard_monitor_enabled_fill" => {
            "Monitor de portapapeles activado; el próximo cambio de la URL de YouTube llenará el campo de URL."
        }
        "state.clipboard_monitor_disabled" => "Monitor del portapapeles desactivado.",
        "state.clipboard_will_auto_add" => {
            "Las URL de YouTube se añadirán inmediatamente después de que cambie el portapapeles."
        }
        "state.clipboard_will_fill_only" => "Los cambios en el portapapeles solo llenarán el campo URL.",
        "state.adding_source" => "Añadiendo: {source}",
        "state.added_to_list" => "Añadido a lista: {title}",
        "state.range_set_item_full" => "Descargar rangos: Elemento {index} / vídeo completo",
        "state.range_set_item_value" => "Descargar rangos: Elemento {index} / {value}",
        "state.format_selection_updated" => {
            "Selección de formato actualizado: Item {index} / {kind} / {value}"
        }
        "state.range_set_title_full" => "Descargar rangos: {title} / vídeo completo",
        "state.range_set_title_value" => "Descargar rangos: {title} / {value}",
        "state.playlist_ignored_for_now" => "La lista de reproducción se ignora por ahora: {target}",
        "state.untitled_video" => "Vídeo sin título",
        "state.analysis_complete" => "Análisis completo: {title}",
        "state.video_extension_error" => "La exportación de vídeo solo es compatible con mkv / mp4 / webm / mov / flv.",
        "state.audio_extension_error" => {
            "La exportación de audio solo es compatible con opus / aac / m4a / mp3 / vorbis / alac / flac / wav."
        }
        "state.subtitle_extension_error" => {
            "La extensión de los subtítulos debe ser srt, vtt, ass, ssa, lrc, ttml, dfxp, json3, srv3, srv2, o srv1."
        }
        "state.action_aria2_fallback" => "{action} (Aria2 no encontrada; usando la descarga nativa de yt-dlp)",
        "state.cache_yt_dlp_default" => "yt-dlp predeterminado",
        "playlist.note.mix_radio" => {
            "Esta lista de reproducción Mix/Radio puede contener muchos elementos y puede cambiar con el tiempo."
        }
        "playlist.note.channel_generated" => {
            "Trate esta lista de reproducción de canal generada por YouTube de manera conservadora."
        }
        "playlist.note.liked_videos" => "Los videos que me gustan generalmente requieren inicio de sesión o cookies.",
        "playlist.note.favorites_legacy" => {
            "Este es un estilo de lista de reproducción de favoritos heredado y es posible que no sea estable ahora."
        }
        "playlist.note.music_album" => "Suele ser un álbum o una colección de YouTube Music.",
        "prepare.tool.ytdlp.description" => "Análisis y descarga de videos principales.",
        "prepare.tool.deno.description" => "Mejora la estabilidad del análisis de YouTube.",
        "prepare.tool.ffmpeg.description" => {
            "Combina video/audio, convierte formatos y maneja miniaturas/subtítulos."
        }
        "prepare.req.app_root.title" => "Carpeta App",
        "prepare.req.app_root.description" => {
            "La carpeta portable debe poder escribirse en las carpetas de configuración y soporte."
        }
        "prepare.req.tools_dir.title" => "carpeta herramientas",
        "prepare.req.tools_dir.description" => {
            "Almacenes de implementación de dependencia yt-dlp, FFmpeg, y Deno aquí."
        }
        "prepare.req.tool_install_cache.title" => "Temp implementación",
        "prepare.req.tool_install_cache.description" => {
            "La extracción de FFmpeg y Deno utiliza esta carpeta temporal."
        }
        "prepare.req.cache.title" => "Caché descarga",
        "prepare.req.cache.description" => "El modo de caché yt-dlp-gui almacena el caché de yt-dlp aquí.",
        "prepare.req.output.title" => "Carpeta de salida",
        "prepare.req.output.description" => "Aquí se guardan vídeos, audio y subtítulos.",
        "prepare.req.output.recommendation" => "Seleccionar una carpeta de salida válida entre Principal u Opciones.",
        "prepare.req.config.title" => "Archivo config",
        "prepare.req.config.description" => {
            "La aplicación debe poder guardar la configuración de ruta de herramientas y página de preparación."
        }
        "prepare.req.move_to_writable" => "Mover la aplicación a una carpeta portable grabable.",
        "prepare.req.move_to_writable_example" => {
            "Mueva la aplicación a una carpeta portable grabable, por ejemplo D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.generic_writable_recommendation" => {
            "Si la implementación o el guardado de la configuración falla, mueva la aplicación a una carpeta portable grabable no sincronizada."
        }
        "prepare.req.use_folder_path" => "Usar una carpeta en su lugar.",
        "prepare.req.path_not_folder" => "{path} no es una carpeta",
        "prepare.req.config_not_folder" => "Asegúrese de que la ruta de configuración no sea una carpeta.",
        "prepare.req.config_readonly" => "El archivo config es de solo lectura",
        "prepare.req.config_readonly_recommendation" => {
            "Borra el atributo de solo lectura del archivo cofig o mover a una carpeta portable grabable."
        }
        "prepare.req.clear_write_test" => {
            "Vuelva a intentarlo más tarde o elimine el archivo .yt-dlp-gui-write-test sobrante de la carpeta."
        }
        "runtime.download_cancelled" => "Descarga cancelada.",
        "runtime.yt_dlp_not_found" => {
            "yt-dlp no se ha encontrado: {path}. Instalar yt-dlp primero o maneje la implementación de dependencias en Opciones."
        }
        "runtime.cookie_file_source_missing" => {
            "Las cookies están activadas y el origen de las cookies es un archivo, pero no se ha seleccionado ningún archivo Netscape cookie.txt válido."
        }
        "runtime.cookie_source_missing" => {
            "Las cookies están activadas, pero no se selecciona ningún navegador ni fuente de cookies.txt."
        }
        "runtime.cookie_file_not_found" => {
            "No se ha encontrado el archivo cookie: {path}. Seleccionar nuevamente un archivo cookie.txt de Netscape o cambie la fuente de la cookie nuevamente del navegador."
        }
        "runtime.download_folder_empty" => "La carpeta de descarga no puede estar vacía.",
        "runtime.could_not_start_yt_dlp" => "No se puede iniciar yt-dlp: {error}",
        "runtime.yt_dlp_analysis_failed" => "Error en el análisis de yt-dlp: {error}",
        "runtime.could_not_parse_yt_dlp_json" => "No se puede analizar yt-dlp JSON: {error}",
        "runtime.yt_dlp_download_failed" => "Error en la descarga de yt-dlp: {error}",
        "runtime.could_not_wait_yt_dlp" => "No se puede esperar a finalizar yt-dlp.: {error}",
        "runtime.could_not_wait_yt_dlp_missing" => {
            "No se puede esperar a que finalice yt-dlp: falta el proceso secundario"
        }
        "runtime.could_not_determine_subtitle_output" => {
            "No se puede determinar el nombre del archivo de salida de subtítulos: {error}"
        }
        "runtime.converted_subtitle_missing" => {
            "yt-dlp ha finalizado, pero no se ha encontrado el archivo de subtítulos convertido: {error}"
        }
        "runtime.could_not_overwrite_subtitle" => {
            "No se puede sobrescribir el archivo de subtítulos existente: {error}"
        }
        "runtime.could_not_copy_subtitle" => {
            "No se puede copiar el archivo de subtítulos a la ubicación de destino: {error}"
        }
        "runtime.could_not_remove_temp_subtitle" => {
            "No se puede eliminar el archivo de subtítulos temporal: {error}"
        }
        "runtime.could_not_create_download_folder" => "No se puede crear la carpeta de descarga: {error}",
        "runtime.file_does_not_exist" => "El archivo no existe: {error}",
        "runtime.file_location_does_not_exist" => "La ubicación del archivo no existe: {error}",
        "runtime.could_not_open_file" => No se puede abrir el archivo: {error}",
        "runtime.could_not_open_containing_folder" => "No se puede abrir la carpeta que lo contiene: {error}",
        "runtime.could_not_open_folder" => "No se puede abrir la carpeta: {error}",
        "runtime.thumbnail_empty_url" => "Error al cargar la miniatura: URL vacía",
        "runtime.thumbnail_no_data" => "Error al cargar la miniatura: no se han recibido datos",
        "runtime.thumbnail_too_large" => "Error al cargar la miniatura: la imagen es demasiado grande",
        "runtime.thumbnail_decode_failed" => "Error en la decodificación de miniaturas: {error}",
        "runtime.invalid_thumbnail_proxy" => "Configuración de proxy de miniatura no válida: {error}",
        "runtime.thumbnail_http" => "Error al cargar la miniatura: HTTP {error}",
        "runtime.thumbnail_load_failed" => "Error al cargar la miniatura: {error}",
        "runtime.config_create_folder" => "No se puede crear la carpeta config: {error}",
        "runtime.config_serialize" => "No se puede serializar el archivo config: {error}",
        "runtime.config_write" => "No se puede escribir el archivo config: {error}",
        "runtime.toast_create_notifier" => "No se puede crear el notificador de Windows Toast: {error}",
        "runtime.toast_create_content" => "No se puede crear contenido de Windows Toast: {error}",
        "runtime.toast_send" => "No se puede enviar Windows Toast: {error}",
        "runtime.toast_create_registration" => {
            "No se pueden crear los datos de registro de Windows Toast: {error}"
        }
        "runtime.toast_register_aumid" => "No se puede crear la carpeta de herramientas {path}: {error}",
        "runtime.dependency_windows_only" => {
            "Actualmente, la implementación de dependencias solo es compatible con Windows."
        }
        "runtime.could_not_create_tools_folder" => "Could not create tools folder {path}: {error}",
        "runtime.install_finished_missing" => {
            "{tool} instalación terminada, pero {path} no se ha encontrado."
        }
        "runtime.could_not_start_powershell" => "No se puede iniciar PowerShell: {error}",
        "runtime.could_not_read_powershell_stdout" => "No se puede leer PowerShell stdout.",
        "runtime.could_not_read_powershell_stderr" => "No se puede leer PowerShell stderr.",
        "runtime.could_not_read_powershell_output" => "No se puede leer la salida de PowerShell: {error}",
        "runtime.could_not_wait_powershell" => "No se puede esperar a que finalice PowerShell: {error}",
        "runtime.powershell_failed_exit" => "Error de PowerShell: código {error}",
        "runtime.could_not_read_playlist_output" => {
            "No se puede leer la salida de la lista de reproducción de yt-dlp: {error}"
        }
        "runtime.batch_import_failed" => "Error al importar por lotes con yt-dlp: {error}",
        "runtime.current_path" => "Ruta actual: {path}",
        "runtime.default_path" => "Ruta predeterminada: {path}",
        "runtime.not_found_path" => "No encontrado: {path}",
        "runtime.can_install_to" => "Se puede instalar en {path}.",
        "runtime.can_save_path" => "Puede guardar: {path}",
        "runtime.system_check" => "Comprobar sistema: {detail}",
        "runtime.save_test" => "Guardar prueba: {detail}",
        "runtime.write_test" => "Escribir prueba: {detail}",
        "runtime.path_is_folder" => "{path} es una carpeta",
        "runtime.path_is_not_folder" => "{path} no es una carpeta",
        "runtime.writable_path" => "Escribible: {path}",
        "runtime.missing_parent_directory" => "falta el directorio principal",
        "runtime.could_not_create_config_folder" => "No se puede crear la carpeta config",
        "runtime.could_not_read_config_file_status" => "No se puede leer el estado del archivo config",
        "runtime.could_not_open_config_file_for_writing" => {
            "No se puede abrir el archivo config para escribir"
        }
        "runtime.could_not_create_folder" => "No se puede crear la carpeta",
        "runtime.could_not_create_rename_delete_test_file" => {
             "No se puede crear, renombrar o eliminar el archivo de prueba"
        }
        "runtime.reason_path_inaccessible" => {
            "La ruta no existe o la ruta principal es inaccesible"
        }
        "runtime.recommend_parent_exists" => "Asegúrese de que existan la unidad y la carpeta principal.",
        "runtime.reason_permission_denied_windows" => {
            "Permiso denegado o bloqueado por la configuración de seguridad de Windows"
        }
        "runtime.recommend_move_portable_defender" => {
            "Mueva la aplicación a una carpeta portable grabable; si el Escritorio/Documentos/Descargas aún fallan, el acceso a carpetas controladas por Defender puede estar bloqueándolo."
        }
        "runtime.reason_in_use" => "Otro programa está utilizando el archivo o carpeta",
        "runtime.recommend_close_program" => {
            "Cierre el programa que pueda estar usando esta carpeta o seleccione otra carpeta."
        }
        "runtime.reason_name_conflict" => "El archivo de prueba ya existe o el nombre está en conflicto",,
        "runtime.reason_disk_space" => "No hay suficiente espacio en disco",
        "runtime.recommend_free_space" => "Libere espacio en disco o seleccione otro disco.",
        "runtime.reason_path_too_long" => "Ruta demasiado larga",
        "runtime.recommend_shorter_path" => {
            "Mover la aplicación a una ruta más corta, por ejemplo D:\\Portable\\yt-dlp-gui-v2."
        }
        "runtime.reason_windows_error_code" => "Código de error de Windows {code}",
        "runtime.recommend_writable_portable_folder" => {
            "Seleccionar una carpeta portable en la que se pueda escribir claramente y verificar."
        }
        "runtime.reason_permission_denied" => "Permiso denegado o bloqueado por la configuración de seguridad",
        "runtime.reason_path_not_exist" => "La ruta no existe",
        "runtime.reason_file_already_exists" => "El archivo ya existe",
        "runtime.reason_write_failed" => "Error de escritura",
        "runtime.recommend_not_system_folder" => {
            "No coloque la aplicación portable en Archivos de programa o en el directorio de Windows; moverlo a D:\\Portable o una carpeta de usuario."
        }
        "runtime.recommend_non_synced_folder" => {
            "Mover a una carpeta no sincronizada, por ejemplo D:\\Portable\\yt-dlp-gui-v2."
        }
        "runtime.could_not_read_playlist_output_empty" => "No se puede leer la salida de la lista de reproducción yt-dlp.",
        "runtime.chromium_cookie_locked" => {
            "No se puede leer la base de datos de cookies de Chromium/Chrome directamente. Es posible que el navegador haya bloqueado la base de datos Network\\Cookies. Cierre el navegador y vuelva a intentarlo, o cambie el origen de las cookies a Usar archivo (cookies.txt) en Avanzado. Mensaje original: {error}"
        }
        "advance.cookie_source_file" => "Usar archivo (cookies.txt)",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "Todos los archivos",
        "state.untitled_task" => "Tarea sin título",
        "state.imported_source" => "Importado {tail}",
        "state.chapter_fallback" => "Capítulo {index}",
        "runtime.config_path_unresolved" => "La ruta del archivo config no se puede resolver",
        "runtime.folder_readonly" => "La carpeta está marcada como de solo lectura.",
        "runtime.network_path_warning" => {
            "Ubicado en una ruta de red; permisos o bloqueos de archivos pueden afectarlo"
        }
        "runtime.protected_directory_warning" => "Ubicado en un directorio protegido de Windows",
        "runtime.onedrive_warning" => {
            "Ubicado en una ruta de sincronización de OneDrive; Pueden ocurrir bloqueos de sincronización o bloqueos de seguridad."
        }
        "runtime.youtube_auto_translated_subtitle_429" => {
            "YouTube ha rechazado temporalmente la solicitud de subtítulos traducidos automáticamente (HTTP 429 Demasiadas solicitudes). Esta es una limitación en la traducción automática de YouTube. La GUI mantiene el flujo nativo de yt-dlp y la salida de diagnóstico en lugar de cambiar a un descargador personalizado. Intente habilitar Cookie/cookies.txt para este elemento, o elija subtítulos automáticos originales/subtítulos originales y vuelva a intentarlo. Mensaje original: {error}"
        }
        "runtime.youtube_subtitle_429_conversion" => {
            "YouTube ha rechazado temporalmente la solicitud de subtítulos (HTTP 429 Demasiadas solicitudes). El archivo de subtítulos fuente no se ha descargado, por lo que no se ejecutará la conversión SRT. Vuelva a intentarlo más tarde o habilite las cookies del navegador antes de exportar. mensaje original: {error}"
        }
        "runtime.youtube_subtitle_429_analysis" => {
            "YouTube ha rechazado la solicitud de subtítulos (HTTP 429 Demasiadas solicitudes). Esto sucede a menudo en el punto final de texto cronometrado de traducción automática de YouTube. cookies.txt puede proporcionar un estado de inicio de sesión, pero es posible que no cumpla con los requisitos de límite de velocidad/token de PO para ese punto final. La GUI mantiene el flujo nativo de yt-dlp y los registros de diagnóstico en lugar de cambiar a un descargador personalizado. Mensaje original: {error}"
        }
        "options.filter_executable" => "Ejecutable",

        // English fallback translations added to keep every bundled language key-complete.
        "tab.processing" => "Procesando",
        "tab.log" => "Log",
        "advance.convert" => "Convertir",
        "advance.apple_tv_hevc_h265" => "Apple TV HEVC / H.265",
        "advance.download_conversion" => "Convertir después de descargar",
        "advance.enable" => "Activar",
        "advance.settings" => "Ajustes",
        "options.tabs" => "Pestañas",
        "options.processing_tab" => "Procesando pestaña",
        "options.enable_processing_tab" => "Activar procesado",
        "options.log_tab" => "Pestaña Log",
        "options.show_log_tab" => "Mostrar log",
        "options.theme" => "Tema",
        "options.theme_color" => "Color tema",
        "config.theme.system" => "Seguir con sistema",
        "config.theme.light" => "Claro",
        "config.theme.dark" => "Oscuro",
        "config.theme_color.off" => "Apagado",
        "config.theme_color.system" => "Azul",
        "config.theme_color.blue" => "Azul suave",
        "config.theme_color.purple" => "Púrpura",
        "config.theme_color.pink" => "Rosa",
        "config.theme_color.green" => "Verde",
        "config.theme_color.orange" => "Naranja",
        "config.theme_color.slate" => "Pizarra",
        "state.apple_tv_hevc_post_processing_title" => "Convirtiendo para Apple TV: {title}",
        "state.transcode_post_processing_title" => "Convirtiendo con {profile}: {title}",
        "processing.convert" => "Convertir",
        "processing.tools" => "Herramientas",
        "processing.transcode" => "Transcodificar",
        "processing.transcode_workbench" => "Transcodificar Intent Workbench",
        "processing.transcode_intent_graph" => "Transcodificar intent graph",
        "processing.intent_graph" => "Intent graph",
        "processing.what_do_you_want" => "¿Qué es lo que quieres hacer?",
        "processing.result_card" => "Resultado",
        "processing.primary_control" => "Control primario",
        "processing.choose_graph_branch_hint" => "Seleccionar esta rama en el gráfico.",
        "processing.current_size_ratio" => "Actual",
        "processing.adjustments" => "Ajustes actuales",
        "processing.locks" => "Elementos bloqueados",
        "processing.locked" => "Bloqueado",
        "processing.auto_recompute" => "Automático",
        "processing.apply" => "Aplicar",
        "processing.apply_after_download" => {
            "Aplicar la transcodificación MP4 segura actualmente admitida después de la descarga"
        }
        "processing.apply_after_download_hint" => {
            "Sólo se aplica el backend MP4 seguro ejecutable actual; No todas las configuraciones están conectadas todavía."
        }
        "processing.affects_command" => "Comando afectado",
        "processing.preview_only_settings" => "Solo previo",
        "processing.disconnected_settings" => "No conectado",
        "processing.backend_available" => "El backend actual puede ejecutar este plan MP4 seguro.",
        "processing.preview_only" => "Actualmente, este plan solo genera un previo del comando.",
        "processing.apple_tv" => "Apple TV",
        "processing.apple_tv_hevc_h265" => "Apple TV HEVC / H.265",
        "transcode.intent.reduce_size" => "Archivo más pequeño",
        "transcode.intent.quality_first" => "Calidad primero",
        "transcode.intent.target_size" => "Tamaño salida",
        "transcode.intent.fast_transcode" => "Formato",
        "transcode.intent.device_compat" => "Compatibilidad salida",
        "transcode.graph.target_ratio" => "Relación salida",
        "transcode.graph.quality_target" => "Calidad salida",
        "transcode.graph.size_input" => "Tamaño entrada",
        "transcode.graph.encode_effort" => "Esfuerzo de codificación",
        "transcode.compat.most_devices" => "La mayoría de los dispositivos / no estoy seguro",
        "transcode.compat.windows" => "Windows PC",
        "transcode.compat.mac" => "Mac",
        "transcode.compat.apple" => "Dispositivos Apple",
        "transcode.compat.tv_nas" => "TV genérica / NAS",
        "transcode.compat.old_device" => "TV antigua / Reproducción USB",
        "transcode.compat.apple_tv_legacy" => "Apple TV heredado",
        "transcode.compat.apple_tv_modern" => "Apple TV moderno",
        "transcode.compat.iphone_ipad" => "iPhone / iPad",
        "transcode.compat.android_tv" => "Android TV / Chromecast",
        "transcode.compat.android_phone_tablet" => "Android phone / tablet",
        "transcode.compat.browser_mp4" => "Navegador seguro MP4",
        "transcode.fps.source" => "Entrada",
        "transcode.fps.24" => "Sobre 24 cps",
        "transcode.fps.25" => "Sobre 25 cps",
        "transcode.fps.30" => "Sobre 30 cps",
        "transcode.fps.60" => "Sobre 60 cps",
        "transcode.setting.fps" => "Límite CPS",
        "transcode.graph.axis.compatibility" => "Compatibilidad".,
        "transcode.graph.axis.capacity" => "Capacidad",
        "transcode.graph.axis.resolution" => "Resolución",
        "transcode.graph.axis.format" => "Formato",
        "transcode.graph.compatibility_scope" => "Ámbito compatibilidad",
        "transcode.graph.capacity_target" => "Tamaño salida",
        "transcode.graph.resolution_limit" => "Límite resolución",
        "transcode.graph.format_goal" => "Formato objetivo",
        "transcode.quality.standard" => "Estándar",
        "transcode.quality.high" => "Alta calidad",
        "transcode.quality.near_original" => "Casi original",
        "transcode.resolution.auto_balance" => "Balance automático",
        "transcode.resolution.keep_original" => "Mantener original",
        "transcode.resolution.max_1080p" => "Max 1080p",
        "transcode.resolution.max_720p" => "Max 720p",
        "transcode.effort.fast" => "Rápido",
        "transcode.effort.normal" => "Normal",
        "transcode.effort.detailed" => "Detalado",
        "transcode.effort.extreme" => "Extremo",
        "transcode.audio.compatible" => "Compatible",
        "transcode.audio.preserve_surround" => "Preservar surround",
        "transcode.encoder.auto" => "Automático",
        "transcode.encoder.hardware_first" => "Hardware primero",
        "transcode.encoder.software" => "Software",
        "transcode.setting.compatibility" => "Compatibilidad",
        "transcode.setting.video_codec" => "Códec de vídeo",
        "transcode.setting.container" => "Contenedor",
        "transcode.setting.encoder" => "Codificador",
        "transcode.setting.quality" => "Calidad",
        "transcode.setting.size_ratio" => "Relación tamaño",
        "transcode.setting.target_size" => "Tamaño salida",
        "transcode.setting.resolution" => "Resolución",
        "transcode.setting.effort" => "Esfuerzo",
        "transcode.setting.pass" => "Control tamaño",
        "transcode.setting.audio" => "Audio",
        "transcode.support.executable" => "Ejecutable",
        "transcode.support.partial" => "Parcialmente soportado",
        "transcode.support.preview_only" => "Solo previo",
        "processing.output_conversion" => "Salida posterior a la descarga",
        "processing.convert_after_download" => "Convertir después de la descarga",
        "processing.convert_after_download_hint" => {
            "Se ejecuta cuando hay que cambiar el vídeo, el audio, el contenedor o los subtítulos."
        }
        "processing.video" => "Vídeo",
        "processing.audio" => "Audio",
        "processing.container" => "Contenedor",
        "processing.subtitle" => "Subtítulos",
        "processing.choice.source" => "Entrada",
        "processing.video.h264" => "H.264",
        "processing.video.hevc" => "HEVC",
        "processing.video.av1" => "AV1",
        "processing.audio.aac" => "AAC",
        "processing.audio.opus" => "Opus",
        "processing.audio.flac" => "FLAC",
        "processing.container.mp4" => "MP4",
        "processing.container.mkv" => "MKV",
        "processing.container.mov" => "MOV",
        "processing.subtitle.preserve" => "Entrada",
        "processing.subtitle.embed" => "Integrado",
        "processing.subtitle.burn" => "Grabar en",
        "processing.disabled_summary" => "La salida yt-dlp descargada se mantendrá tal cual.",
        "processing.no_conversion_summary" => {
            "Todas las opciones están configuradas como entrada, por lo que no se ejecutará ningún postproceso."
        }
        "processing.output_summary" => "Resumen de salida",
        "processing.visual_quality" => "Imagen,
        "processing.visual_quality_near_source" => "Mantener visualmente cerca de la entrada",
        "processing.method" => "Método",
        "processing.encoder" => "Codificador",
        "processing.status" => "Estado",
        "processing.command_preview" => "Previo comando",
        "log.empty" => "Aún no hay registro de ejecución",
        "log.clear" => "Borrar",
        "log.copy" => "Copiar",
        "transcode.audio.auto" => "Entrada",
        "transcode.audio.aac" => "AAC",
        "transcode.audio.opus" => "Opus",
        "transcode.audio.flac" => "FLAC",
        "log.runtime" => "Tiempo de ejecución log",
        "log.not_implemented" => "Aún no se ha implementado la recogida de registros en tiempo de ejecución.",
        "runtime.subtitle_burn_no_source" => {
            "La grabación de subtítulos necesita un archivo de subtítulos o subtítulos integrados. Descargue primero los subtítulos para este elemento o coloque un archivo de subtítulos .srt/.ass junto al vídeo."
        }
        // English fallback translations keep bundled languages key-complete.
        "item.add_an_audio_url" => "Añadir una URL de audio",
        "options.auto_detect_tool_from" => "Detección automática de",
        "options.auto_detect_tool_hint" => {
            "Detectar las herramientas instaladas de la carpeta de herramientas portables y del sistema PATH."
        }
        "options.cache_usage" => "Uso",
        "options.cache_usage_detail" => "Total: {total} · Audio: {audio} · Expirado: {expired}",
        "options.cache_cleanup" => "Limpieza",
        "options.cache_refresh" => "Actualizar",
        "options.cache_clear_expired" => "Borrar expirado",
        "options.cache_clear_audio" => "Borrar audio",
        "options.cache_clear_all" => "Borrar todo",
        "state.tool_auto_detected" => "{tool} detectado a partir de PATH: {path}",
        "state.tool_auto_detect_not_found" => "{tool} no se ha encontrado en el sistema PATH.",
        "state.tools_auto_detected" => "Detectado {found}/{total} herramientas de PATH.",
        "state.tools_auto_detect_missing" => "No se encuentra en PATH: {tools}.",
        "state.tools_auto_detect_none" => "No se han encontrado herramientas de dependencia en el sistema PATH.",
        "state.cache_cleaned_expired" => "Borrado {count} entradas de caché caducadas ({size}).",
        "state.cache_cleaned_audio" => "Borrado de la caché de audio: {count} entradas ({size}).",
        "state.cache_cleaned_all" => "Borrado la caché de la aplicación: {count} entradas ({size}).",
        "state.cache_cleanup_failed" => "Error en la limpieza de la caché: {error}",
        "queue_display.normal" => "Normal",
        "queue_display.audio" => "Audio",
        "queue_display.normal.tooltip" => "Lista normal de descargas",
        "queue_display.audio.tooltip" => "Lista y reproducción de audio",
        "main.queue_display_mode_hint" => "Cambiar la visualización de la lista y el comportamiento",
        "music.previous" => "Anterior",
        "music.play" => "Reproducir",
        "music.pause" => "Pausa",
        "music.next" => "Siguiente",
        "music.seek_cached_range_hint" => "Arrastrar para buscar; liberar instantáneas dentro del intervalo en caché",
        "music.seek_hint" => "Arrastrar para buscar",
        "music.status.completed" => "hecho",
        "music.status.resolving" => "Resolviendo",
        "music.status.buffering" => "Almacenamiento en búfer",
        "music.status.ready" => "Listo",
        "music.status.caching" => "Almacenamiento en caché",
        "music.status.playing" => "Reproduciendo",
        "music.status.paused" => "Pausado",
        "music.status.failed" => "Error",
        "music.playback_mode.sequential" => "Secuencia",
        "music.playback_mode.repeat_all" => "Repetir",
        "music.playback_mode.shuffle" => "Aleatorio",
        "music.playback_mode.repeat_one" => "Repetir una vez",
        "music.playback_mode.sequential.tooltip" => "Reproducir en orden",
        "music.playback_mode.repeat_all.tooltip" => "Repetir lista",
        "music.playback_mode.shuffle.tooltip" => "Repetir aleatorio",
        "music.playback_mode.repeat_one.tooltip" => "Repetir una pista",
        "options.music_download_format" => "Formato de descarga de música",
        "options.music_download_format_title" => "¿Qué formato de audio se debe exportar?",
        "options.music_download_format_body" => {
            "Primero se utiliza el caché de música completo; la conversión solo se ejecuta cuando el formato difiere."
        }
        "state.queue_display_mode_changed" => "Modo de lista: {mode}",
        "state.downloading_music" => "Descargando música: {title}",
        "state.music_no_items_from_source" => "No se pueden añadir elementos de música: {source}",
        "state.music_items_added" => "Añadido {count} elementos de música.",
        "state.music_playlist_parse_failed" => "Error en el análisis de la lista de música: {error}",
        "state.music_stream_ready" => "Transmisión de música lista: {source}",
        "state.music_stream_parse_failed" => "Error en el análisis de la transmisión de música: {error}",
        "state.music_playback_finished" => "Reproducción finalizada.",
        "state.music_playback_failed" => "Error al reproducir: {error}",
        "state.music_duplicate_with_cache" => {
            "El elemento de música ya está en la lista; se ha usado la memoria caché local."
        }
        "state.music_duplicate" => "El elemento de música ya está en la lista.",
        "state.music_added_from_cache" => "Música añadida desde el caché local: {title}",
        "state.music_seek_clamped" => {
            "Fuera del rango almacenado en caché; ha regresado a una posición reproducible."
        }
        "state.music_stream_still_preparing" => "La transmisión de música aún se está preparando.",
        "state.no_playable_music_items" => "No hay elementos de música reproducibles.",
        "state.music_cache_prepare_failed" => "Error en la preparación de la caché de música: {error}",
        "state.preparing_music_playback" => "Preparando la reproducción: {title}",
        "state.music_missing_source_url" => "Al elemento de música le falta una URL de origen.",
        "state.resolving_music_stream" => "Resolviendo la transmisión de música: {title}",
        "state.music_stream_still_resolving" => "La transmisión de música aún se está resolviendo.",
        "state.music_buffering" => "Música en almacenamiento en búfer",
        "state.music_item_not_playable" => "Este elemento de música no se puede reproducir en este momento.",
        "state.music_stream_not_ready" => "La transmisión de música aún no está lista.",
        "state.no_previous_music" => "Sin pista anterior.",
        "state.no_next_music" => "No hay siguiente pista".,
        "state.music_playback_mode_changed" => "Modo de reproducción: {mode}",
        _ => super::en_us::text(key),
    }
}
