pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.prepare" => "Preparar",
        "tab.main" => "Principal",
        "tab.advanced" => "Avanzado",
        "tab.options" => "Opciones",
        "main.url_hint" => "URL",
        "action.download" => "Descargar",
        "action.add" => "＋ Añadir",
        "action.stop" => "Detener",
        "action.stopping" => "Deteniendo",
        "action.cut" => "Cortar",
        "action.copy" => "Copiar",
        "action.paste" => "Pegar",
        "action.clear" => "Borrar",
        "item.thumbnail" => "Miniatura",
        "item.thumbnail_preview" => "Vista previa de miniatura",
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
        "item.file_name" => "Nombre de archivo",
        "main.target_folder" => "Carpeta de salida",
        "picker.title.video" => "Seleccionar formato de vídeo",
        "picker.title.audio" => "Seleccionar formato de audio",
        "picker.title.subtitle" => "Seleccionar subtítulos",
        "picker.title.section" => "Seleccionar rango",
        "action.back" => "Volver",
        "picker.mode.filter" => "Filtros",
        "picker.mode.table" => "Tabla",
        "action.confirm" => "Confirmar",
        "picker.empty_table" => "No hay formatos para mostrar",
        "picker.header.resolution" => "Resolución",
        "picker.header.range" => "Rango",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "Formato",
        "picker.header.codec" => "Códec",
        "picker.header.size" => "Tamaño",
        "picker.header.sample_rate" => "Frecuencia de muestreo",
        "picker.filter.resolution" => "Resolución",
        "picker.filter.range" => "Rango",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "Códec",
        "picker.filter.sample_rate" => "Frecuencia de muestreo",
        "main.tooltip.missing_yt_dlp" => {
            "Falta yt-dlp. Instálalo o selecciona yt-dlp.exe en Opciones."
        }
        "advance.source" => "Fuente",
        "advance.config" => "Configuración",
        "advance.none" => "Ninguno",
        "advance.network_access" => "Red y acceso",
        "advance.proxy" => "Proxy",
        "advance.enable_proxy" => "Activar proxy",
        "advance.certificate" => "Certificado",
        "advance.skip_certificate_verification" => "Omitir verificación de certificado",
        "advance.use_cookies" => "Usar cookies",
        "advance.enable_cookies" => "Activar cookies",
        "advance.cookie_source" => "Origen de cookies",
        "advance.cookie_file" => "Archivo de cookies",
        "advance.no_cookies_txt_selected" => "No se seleccionó cookies.txt",
        "advance.browse" => "Navegar",
        "advance.select_netscape_cookies_txt" => "Seleccionar cookies.txt de Netscape",
        "advance.clear" => "Borrar",
        "advance.browser" => "Navegador",
        "advance.default" => "Predeterminado",
        "advance.external_downloader" => "Descargador externo",
        "advance.use_aria2_for_faster_downloads" => "Usar Aria2 para descargas más rápidas",
        "advance.download_control" => "Control de descarga",
        "advance.concurrent_fragments" => "Fragmentos simultáneos",
        "advance.1_default" => "1 (predeterminado)",
        "advance.rate_limit" => "Límite de velocidad",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => {
            "p. ej. 2M, 800K; dejar vacío para ilimitado"
        }
        "advance.chapters" => "Capítulos",
        "advance.chapter_download_compatibility_mode" => {
            "Modo de compatibilidad para descarga de capítulos"
        }
        "advance.file_time" => "Hora del archivo",
        "advance.post_processing" => "Posprocesamiento",
        "advance.thumbnail" => "Miniatura",
        "advance.download" => "Descargar",
        "advance.embed" => "Incrustar",
        "advance.subtitles" => "Subtítulos",
        "item.stop_download" => "Detener descarga",
        "item.remove" => "Eliminar",
        "item.save_as" => "Guardar como",
        "item.error" => "Error",
        "item.all" => "Todo",
        "item.queued" => "En cola",
        "item.done" => "Completado",
        "item.failed" => "Falló",
        "item.clear_all" => "Borrar todo",
        "item.add_a_video_url" => "Añade una URL de vídeo",
        "item.after_adding_choose_the_video_format_here" => "Elegir formato de vídeo",
        "item.after_adding_choose_the_audio_format_here" => "Elegir formato de audio",
        "item.loading_thumbnail" => "Cargando miniatura",
        "item.file_actions" => "Acciones de archivo",
        "item.open_file" => "Abrir archivo",
        "item.open_folder" => "Abrir carpeta",
        "item.copy_path" => "Copiar ruta",
        "item.opened_output_file" => "Archivo de salida abierto.",
        "item.file_not_found_opened_the_output_location" => {
            "Archivo no encontrado; se abrió la ubicación de salida."
        }
        "item.opened_output_location" => "Ubicación de salida abierta.",
        "item.copied_output_path" => "Ruta de salida copiada.",
        "item.file_actions_are_available_after_download_co" => {
            "Las acciones de archivo estarán disponibles cuando termine la descarga"
        }
        "prepare.language" => "Idioma",
        "prepare.back" => "Volver",
        "prepare.auto_detect" => "Detectar automáticamente",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Instala ahora las herramientas necesarias u omite este paso y configúralas más tarde en Opciones."
        }
        "prepare.required" => "Obligatorio",
        "prepare.recommended" => "Recomendado",
        "prepare.optional" => "Opcional",
        "prepare.missing" => "Falta",
        "prepare.install_later" => "Instalar más tarde",
        "prepare.downloading_100" => "Descargando 100%",
        "prepare.extracting_100" => "Extrayendo 100%",
        "prepare.install_failed" => "Instalación fallida",
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
        "options.playlist_prompt" => "Pregunta de lista de reproducción",
        "options.which_one_should_be_loaded" => "¿Cuál se debe cargar?",
        "options.both_video_and_playlist_were_detected" => {
            "Se ha detectado tanto el vídeo como la lista de reproducción."
        }
        "options.this_playlist_may_contain_many_items" => {
            "Esta lista de reproducción puede contener muchos elementos."
        }
        "options.video" => "Vídeo",
        "options.playlist" => "Lista de reproducción",
        "options.cancel" => "Cancelar",
        "options.load" => "Cargar",
        "options.behavior" => "Comportamiento",
        "options.add_action" => "Añadir acción",
        "options.download_directly" => "Descargar directamente",
        "options.clipboard_change" => "Cambio del portapapeles",
        "options.run_immediately" => "Ejecutar de inmediato",
        "options.playlist_2" => "Lista de reproducción",
        "options.with_playlist" => "Con lista de reproducción",
        "options.ask" => "Preguntar",
        "options.single_video" => "Vídeo único",
        "options.full_playlist" => "Lista completa",
        "options.high_risk_prompt" => "Aviso de alto riesgo",
        "options.on" => "Activado",
        "options.playlist_count" => "Cantidad de la lista",
        "options.limit" => "Límite",
        "options.max" => "Máx:",
        "options.items" => " elementos",
        "options.language" => "Idioma",
        "options.current_language" => "Idioma actual",
        "options.back" => "Volver",
        "options.choose" => "Elegir",
        "options.auto_detect" => "Detectar automáticamente",
        "options.tool_paths" => "Rutas de herramientas",
        "options.file_actions" => "Acciones de archivo",
        "options.action_button" => "Botón de acción",
        "options.cache" => "Caché",
        "options.cache_location" => "Ubicación de caché",
        "options.appearance_window" => "Apariencia y ventana",
        "options.notifications" => "Notificaciones",
        "options.enable" => "Activar",
        "options.ui_scale" => "Escala de interfaz",
        "options.apply" => "Aplicar",
        "options.current" => "Actual",
        "options.always_on_top" => "Siempre arriba",
        "options.window_position" => "Posición de ventana",
        "options.remember" => "Recordar",
        "options.window_size" => "Tamaño de ventana",
        "options.reinstall" => "Reinstalar",
        "options.installing" => "Instalando",
        "options.install" => "Instalar",
        "options.file_not_found" => "Archivo no encontrado: ",
        "options.will_install_to" => "Se instalará en: ",
        "options.another_tool_is_being_installed_please_wait" => {
            "Se está instalando otra herramienta. Espere a que termine."
        }
        "options.install_to" => "Instalar en: ",
        "options.executable" => "ejecutable",
        "main.clipboard_monitor_on_the_next_youtube_url_ch" => {
            "Monitor del portapapeles: activado. El próximo cambio de URL de YouTube se añadirá inmediatamente."
        }
        "main.clipboard_monitor_on_the_next_youtube_url_ch_2" => {
            "Monitor del portapapeles: activado. El próximo cambio de URL de YouTube rellenará el campo URL."
        }
        "main.clipboard_monitor_off_turning_it_on_only_mem" => {
            "Monitor del portapapeles: desactivado. Al activarlo solo se memoriza el portapapeles actual; se gestionará el siguiente cambio."
        }
        "main.controlled_by_config" => "Controlado por configuración: ",
        "main.controlled_by_config_2" => "Controlado por configuración",
        "picker.no_chapters_available" => "No hay capítulos disponibles",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "Elige el rango de descarga para este elemento. El valor predeterminado es el vídeo completo."
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
            "Consejo: los subtítulos traducidos automáticamente de YouTube tienen más probabilidad de sufrir límites de solicitud que los subtítulos originales. Elige “Sin traducción” si solo necesitas el texto de origen."
        }
        "picker.no_subtitles_are_available_for_this_source" => {
            "No hay subtítulos disponibles para este origen."
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
        "tools.file_time.use_download_time" => "Usar hora de descarga",
        "tools.file_time.none_hint" => {
            "No pasa --mtime/--no-mtime ni modifica la hora del archivo final."
        }
        "tools.file_time.use_upload_date_hint" => {
            "Después de que yt-dlp informe la ruta final, ajusta la hora de modificación a la fecha de subida del video."
        }
        "tools.file_time.use_download_time_hint" => "--no-mtime",
        "tools.cache_mode.default" => "Predeterminado",
        "tools.subtitle_source.none" => "Sin subtítulos",
        "tools.subtitle_source.original" => "Subtítulos originales",
        "tools.subtitle_source.automatic" => "Subtítulos automáticos",
        "tools.youtube_playlist.channel_generated" => "Lista de reproducción generada por YouTube",
        "tools.youtube_playlist.music_album" => "Álbum o colección de YouTube Music",
        "tools.youtube_playlist.liked_videos" => "Vídeos que me gustan",
        "tools.youtube_playlist.favorites_legacy" => "Lista de favoritos heredada",
        "prepare.severity.required" => "Elemento obligatorio",
        "prepare.severity.recommended" => "Elemento recomendado",
        "prepare.severity.optional" => "Elemento opcional",
        "prepare.status.ready" => "Listo",
        "prepare.status.missing" => "Falta",
        "prepare.status.warning" => "Requiere atención",
        "prepare.status.failed" => "Falló",
        "tool_install.stage.preparing" => "Preparando",
        "tool_install.stage.downloading" => "Descargando",
        "tool_install.stage.extracting" => "Extrayendo",
        "tool_install.stage.installing" => "Instalando",
        "tool_install.stage.completed" => "Completado",
        "tool_install.stage.failed" => "Falló",
        "domain.quality.best" => "Mejor",
        "domain.quality.audio_only" => "Solo audio",
        "prepare.severity.short.required" => "Obligatorio",
        "prepare.severity.short.recommended" => "Recomendado",
        "prepare.severity.short.optional" => "Opcional",
        "item.status.idle" => "No iniciado",
        "item.status.queued" => "En cola",
        "item.status.running" => "En ejecución",
        "item.status.finished" => "Completado",
        "item.status.failed" => "Falló",
        "item.status.cancelled" => "Cancelado",
        "item.status.waiting_analysis" => "Esperando análisis",
        "item.status.analyzing" => "Analizando",
        "item.status.analysis_failed" => "Error al analizar",
        "picker.waiting_analysis" => "Esperando análisis",
        "picker.audio_from_video" => "Según el formato de vídeo",
        "picker.not_selected" => "No seleccionado",
        "picker.full_video" => "Vídeo completo",
        "picker.no_translation" => "Sin traducción",
        "picker.until_end" => "fin",
        "state.clipboard_detected_url" => "Se detectó una URL de YouTube en el portapapeles.",
        "state.no_url_to_analyze" => "No hay ninguna URL para analizar.",
        "state.analyzing_source" => "Analizando: {source}",
        "state.batch_add_running" => "La importación por lotes sigue en curso.",
        "state.no_url_to_add" => "No hay ninguna URL para añadir.",
        "state.video_url_contains_playlist" => {
            "Se detectó una URL de vídeo que también contiene una lista de reproducción."
        }
        "state.detected_high_risk_playlist" => {
            "Se detectó una lista de reproducción de YouTube de alto riesgo: {kind}"
        }
        "state.no_url_to_download_now" => "No hay ninguna URL para descargar inmediatamente.",
        "state.download_now_single_video_only" => "Descargar ahora solo admite una URL de vídeo.",
        "state.added_ready_download_now" => "Añadido y listo para descargar ahora: {title}",
        "state.current_action_cancelled" => "Acción cancelada.",
        "state.stopping_batch_add" => "Deteniendo importación por lotes...",
        "state.retrying_analysis_cookie" => "Reintentando análisis con cookies: {source}",
        "state.batch_no_new_items" => "No se encontraron elementos nuevos en el lote.",
        "state.playlist_added_limited" => {
            "Se añadieron {count} elementos de la lista de reproducción (límite aplicado)."
        }
        "state.batch_added_title" => "Añadido al lote: {title}",
        "state.playlist_added" => "Se añadieron {count} elementos de la lista de reproducción.",
        "state.batch_add_cancelled" => "Importación por lotes cancelada.",
        "state.batch_add_cancelled_with_count" => {
            "Importación por lotes cancelada; se añadieron {count} elementos."
        }
        "state.batch_add_interrupted" => "La importación por lotes se interrumpió.",
        "state.deployment_complete" => "Implementación completada",
        "state.tool_deployed" => "{tool} descargado e implementado.",
        "state.tool_deploy_failed" => "Falló la implementación de {tool}: {error}",
        "state.download_item_fallback" => "Elemento de descarga",
        "state.download_stopped" => "Descarga detenida.",
        "state.no_url_to_add_batch" => "No hay ninguna URL para añadir al lote.",
        "state.batch_input_added" => {
            "Se añadieron {count} elementos en cola desde la entrada por lotes."
        }
        "state.no_url_to_download" => "No hay ninguna URL para descargar.",
        "state.download_already_running" => {
            "Ya se está ejecutando una descarga. Espere a que finalice."
        }
        "state.no_runnable_batch_items" => "No hay elementos por lotes ejecutables.",
        "state.no_download_to_stop" => "No hay ninguna descarga que detener.",
        "state.stopping_download" => "Deteniendo la descarga...",
        "state.target_download_not_found" => "No se encontró el elemento de descarga seleccionado.",
        "state.analyze_before_download" => "Analizar el vídeo antes de iniciar la descarga.",
        "state.downloading_title" => "Descargando: {title}",
        "state.downloading_title_aria2_fallback" => {
            "Descargando: {title} (no se encontró Aria2; se usará la descarga nativa de yt-dlp)"
        }
        "state.target_export_not_found" => {
            "No se encontró el elemento de exportación seleccionado."
        }
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
        "state.controlled_by_config" => "Controlado por configuración",
        "state.install_blocked_by_prepare" => {
            "Resuelve {items} antes de instalar herramientas de dependencia."
        }
        "state.tool_deployment_running" => "La implementación de {tool} sigue en curso.",
        "state.no_tools_to_install" => "No hay herramientas para instalar.",
        "state.no_selected_tools_to_install" => "No hay elementos implementables seleccionados.",
        "state.prepare_skipped" => {
            "Página Preparar omitida. Puedes gestionar la implementación de dependencias más tarde en Opciones."
        }
        "state.skip_failed" => "No se pudo omitir: {error}",
        "state.preparing_deployment" => "Preparando implementación",
        "state.tool_downloading_deploying" => "{tool} se está descargando e implementando...",
        "state.found" => "Encontrado",
        "state.not_found" => "No encontrado",
        "state.clipboard_monitor_enabled_auto_add" => {
            "Monitor del portapapeles activado; el próximo cambio de URL de YouTube se añadirá inmediatamente."
        }
        "state.clipboard_monitor_enabled_fill" => {
            "Monitor del portapapeles activado; el próximo cambio de URL de YouTube rellenará el campo URL."
        }
        "state.clipboard_monitor_disabled" => "Monitor del portapapeles desactivado.",
        "state.clipboard_will_auto_add" => {
            "Las URL de YouTube se añadirán inmediatamente después de que cambie el portapapeles."
        }
        "state.clipboard_will_fill_only" => {
            "Los cambios del portapapeles solo rellenarán el campo URL."
        }
        "state.adding_source" => "Añadiendo: {source}",
        "state.added_to_list" => "Añadido a lista: {title}",
        "state.range_set_item_full" => {
            "Rango de descarga definido: elemento {index} / vídeo completo"
        }
        "state.range_set_item_value" => "Rango de descarga definido: elemento {index} / {value}",
        "state.format_selection_updated" => {
            "Selección de formato actualizada: elemento {index} / {kind} / {value}"
        }
        "state.range_set_title_full" => "Rango de descarga definido: {title} / vídeo completo",
        "state.range_set_title_value" => "Rango de descarga definido: {title} / {value}",
        "state.playlist_ignored_for_now" => {
            "La lista de reproducción se ignora por ahora: {target}"
        }
        "state.untitled_video" => "Vídeo sin título",
        "state.analysis_complete" => "Análisis completo: {title}",
        "state.action_aria2_fallback" => {
            "{action} (no se encontró Aria2; se usará la descarga nativa de yt-dlp)"
        }
        "state.cache_yt_dlp_default" => "yt-dlp predeterminado",
        "playlist.note.mix_radio" => {
            "Esta lista Mix / Radio puede contener muchos elementos y cambiar con el tiempo."
        }
        "playlist.note.channel_generated" => {
            "Trata esta lista de reproducción generada por YouTube con cautela."
        }
        "playlist.note.liked_videos" => {
            "Los vídeos que me gustan generalmente requieren inicio de sesión o cookies."
        }
        "playlist.note.favorites_legacy" => {
            "Esta es una lista de favoritos heredada y puede no ser estable actualmente."
        }
        "playlist.note.music_album" => "Suele ser un álbum o una colección de YouTube Music.",
        "prepare.tool.ytdlp.description" => "Análisis y descarga principal de vídeo.",
        "prepare.tool.deno.description" => "Mejora la estabilidad del análisis de YouTube.",
        "prepare.tool.ffmpeg.description" => {
            "Fusiona vídeo/audio, convierte formatos y gestiona miniaturas/subtítulos."
        }
        "prepare.req.app_root.title" => "Carpeta de la app",
        "prepare.req.app_root.description" => {
            "La carpeta portátil debe permitir escritura para la configuración y las carpetas de soporte."
        }
        "prepare.req.tools_dir.title" => "Carpeta de herramientas",
        "prepare.req.tools_dir.description" => {
            "La implementación de dependencias guarda yt-dlp, FFmpeg y Deno aquí."
        }
        "prepare.req.tool_install_cache.title" => "Temporal de implementación",
        "prepare.req.tool_install_cache.description" => {
            "La extracción de FFmpeg y Deno utiliza esta carpeta temporal."
        }
        "prepare.req.cache.title" => "Caché de descargas",
        "prepare.req.cache.description" => {
            "El modo de caché yt-dlp-gui guarda aquí la caché de yt-dlp."
        }
        "prepare.req.output.title" => "Carpeta de salida",
        "prepare.req.output.description" => "Aquí se guardan vídeos, audio y subtítulos.",
        "prepare.req.output.recommendation" => {
            "Elige una carpeta de salida válida desde Principal u Opciones."
        }
        "prepare.req.config.title" => "Archivo de configuración",
        "prepare.req.config.description" => {
            "La app debe poder guardar la omisión de la página Preparar y las rutas de herramientas."
        }
        "prepare.req.move_to_writable" => "Mueve la app a una carpeta portátil con escritura.",
        "prepare.req.move_to_writable_example" => {
            "Mueve la app a una carpeta portátil con escritura, por ejemplo D:\\Portable\\yt-dlp-gui-v2."
        }
        "runtime.download_cancelled" => "Descarga cancelada.",
        "runtime.yt_dlp_not_found" => {
            "No se encontró yt-dlp: {path}. Instala yt-dlp primero o gestiona la implementación de dependencias en Opciones."
        }
        "runtime.cookie_file_source_missing" => {
            "Las cookies están activadas y el origen es un archivo, pero no se seleccionó un cookies.txt de Netscape válido."
        }
        "runtime.cookie_source_missing" => {
            "Las cookies están activadas, pero no se seleccionó ningún navegador ni cookies.txt."
        }
        "runtime.cookie_file_not_found" => {
            "No se encontró el archivo de cookies: {path}. Elige de nuevo un cookies.txt de Netscape o cambia el origen de cookies al navegador."
        }
        "runtime.download_folder_empty" => "La carpeta de descarga no puede estar vacía.",
        "runtime.could_not_start_yt_dlp" => "No se puede iniciar yt-dlp: {error}",
        "runtime.yt_dlp_analysis_failed" => "Error en el análisis de yt-dlp: {error}",
        "runtime.could_not_parse_yt_dlp_json" => "No se pudo analizar el JSON de yt-dlp: {error}",
        "runtime.yt_dlp_download_failed" => "Error en la descarga de yt-dlp: {error}",
        "runtime.could_not_wait_yt_dlp" => "No se pudo esperar a que yt-dlp terminara: {error}",
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
        "runtime.could_not_create_download_folder" => {
            "No se puede crear la carpeta de descarga: {error}"
        }
        "runtime.file_does_not_exist" => "El archivo no existe: {error}",
        "runtime.file_location_does_not_exist" => "La ubicación del archivo no existe: {error}",
        "runtime.could_not_open_file" => "No se pudo abrir el archivo: {error}",
        "runtime.could_not_open_containing_folder" => {
            "No se puede abrir la carpeta que lo contiene: {error}"
        }
        "runtime.could_not_open_folder" => "No se puede abrir la carpeta: {error}",
        "runtime.thumbnail_empty_url" => "Error al cargar la miniatura: URL vacía",
        "runtime.thumbnail_no_data" => "Error al cargar la miniatura: no se han recibido datos",
        "runtime.thumbnail_too_large" => {
            "Error al cargar la miniatura: la imagen es demasiado grande"
        }
        "runtime.thumbnail_decode_failed" => "Error en la decodificación de miniaturas: {error}",
        "runtime.invalid_thumbnail_proxy" => {
            "Configuración de proxy de miniatura no válida: {error}"
        }
        "runtime.thumbnail_http" => "Error al cargar la miniatura: HTTP {error}",
        "runtime.thumbnail_load_failed" => "Error al cargar la miniatura: {error}",
        "runtime.config_create_folder" => "No se puede crear la carpeta de configuración: {error}",
        "runtime.config_serialize" => "No se puede serializar el archivo de configuración: {error}",
        "runtime.config_write" => "No se puede escribir el archivo de configuración: {error}",
        "runtime.toast_create_notifier" => {
            "No se puede crear el notificador de Windows Toast: {error}"
        }
        "runtime.toast_create_content" => "No se puede crear contenido de Windows Toast: {error}",
        "runtime.toast_send" => "No se puede enviar Windows Toast: {error}",
        "runtime.toast_create_registration" => {
            "No se pueden crear los datos de registro de Windows Toast: {error}"
        }
        "runtime.toast_register_aumid" => "No se pudo registrar el AUMID de Windows Toast: {error}",
        "runtime.dependency_windows_only" => {
            "Actualmente, la implementación de dependencias solo es compatible con Windows."
        }
        "runtime.could_not_create_tools_folder" => {
            "No se pudo crear la carpeta de herramientas {path}: {error}"
        }
        "runtime.install_finished_missing" => {
            "La instalación de {tool} terminó, pero no se encontró {path}."
        }
        "runtime.could_not_start_powershell" => "No se puede iniciar PowerShell: {error}",
        "runtime.could_not_read_powershell_stdout" => "No se puede leer PowerShell stdout.",
        "runtime.could_not_read_powershell_stderr" => "No se puede leer PowerShell stderr.",
        "runtime.could_not_read_powershell_output" => {
            "No se puede leer la salida de PowerShell: {error}"
        }
        "runtime.could_not_wait_powershell" => {
            "No se puede esperar a que finalice PowerShell: {error}"
        }
        "runtime.powershell_failed_exit" => "Error de PowerShell: código {error}",
        "runtime.could_not_read_playlist_output" => {
            "No se puede leer la salida de la lista de reproducción de yt-dlp: {error}"
        }
        "runtime.batch_import_failed" => "Error al importar por lotes con yt-dlp: {error}",
        "runtime.current_path" => "Ruta actual: {path}",
        "runtime.default_path" => "Ruta predeterminada: {path}",
        "runtime.not_found_path" => "No encontrado: {path}",
        "runtime.can_install_to" => "Se puede instalar en {path}.",
        "runtime.can_save_path" => "Se puede guardar: {path}",
        "runtime.system_check" => "Comprobación del sistema: {detail}",
        "runtime.save_test" => "Prueba de guardado: {detail}",
        "runtime.write_test" => "Prueba de escritura: {detail}",
        "runtime.path_is_folder" => "{path} es una carpeta",
        "runtime.path_is_not_folder" => "{path} no es una carpeta",
        "runtime.writable_path" => "Con escritura: {path}",
        "runtime.missing_parent_directory" => "falta el directorio principal",
        "runtime.could_not_create_config_folder" => "No se pudo crear la carpeta de configuración",
        "runtime.could_not_read_config_file_status" => {
            "No se pudo leer el estado del archivo de configuración"
        }
        "runtime.could_not_open_config_file_for_writing" => {
            "No se pudo abrir el archivo de configuración para escritura"
        }
        "runtime.could_not_create_folder" => "No se puede crear la carpeta",
        "runtime.could_not_create_rename_delete_test_file" => {
            "No se puede crear, renombrar o eliminar el archivo de prueba"
        }
        "runtime.reason_path_inaccessible" => {
            "La ruta no existe o la ruta principal es inaccesible"
        }
        "runtime.recommend_parent_exists" => {
            "Asegúrate de que existan la unidad y la carpeta principal."
        }
        "runtime.reason_permission_denied_windows" => {
            "Permiso denegado o bloqueado por la configuración de seguridad de Windows"
        }
        "runtime.recommend_move_portable_defender" => {
            "Mueve la app a una carpeta portátil con escritura; si Escritorio/Documentos/Descargas siguen fallando, puede que el acceso controlado a carpetas de Defender la esté bloqueando."
        }
        "runtime.reason_in_use" => "Otro programa está utilizando el archivo o carpeta",
        "runtime.recommend_close_program" => {
            "Cierra el programa que pueda estar usando esta carpeta o selecciona otra carpeta."
        }
        "runtime.reason_name_conflict" => {
            "El archivo de prueba ya existe o el nombre está en conflicto"
        }
        "runtime.reason_disk_space" => "No hay suficiente espacio en disco",
        "runtime.recommend_free_space" => "Libere espacio en disco o seleccione otro disco.",
        "runtime.reason_path_too_long" => "Ruta demasiado larga",
        "runtime.recommend_shorter_path" => {
            "Mueve la app a una ruta más corta, por ejemplo D:\\Portable\\yt-dlp-gui-v2."
        }
        "runtime.reason_windows_error_code" => "Código de error de Windows {code}",
        "runtime.recommend_writable_portable_folder" => {
            "Elige una carpeta portátil claramente escribible y vuelve a comprobar."
        }
        "runtime.reason_permission_denied" => {
            "Permiso denegado o bloqueado por la configuración de seguridad"
        }
        "runtime.reason_path_not_exist" => "La ruta no existe",
        "runtime.reason_file_already_exists" => "El archivo ya existe",
        "runtime.reason_write_failed" => "Error de escritura",
        "runtime.recommend_not_system_folder" => {
            "No coloques la app portátil en Archivos de programa ni en el directorio de Windows; muévela a D:\\Portable o a una carpeta de usuario."
        }
        "runtime.recommend_non_synced_folder" => {
            "Muévela a una carpeta no sincronizada, por ejemplo D:\\Portable\\yt-dlp-gui-v2."
        }
        "runtime.could_not_read_playlist_output_empty" => {
            "No se pudo leer la salida de lista de reproducción de yt-dlp."
        }
        "runtime.chromium_cookie_locked" => {
            "No se pudo leer directamente la base de datos de cookies de Chromium/Chrome. Puede que el navegador haya bloqueado la base de datos Network\\Cookies. Cierra el navegador y reintenta, o cambia el origen de cookies a Usar archivo (cookies.txt) en Avanzado. Mensaje original: {error}"
        }
        "advance.cookie_source_file" => "Usar archivo (cookies.txt)",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "Todos los archivos",
        "state.untitled_task" => "Tarea sin título",
        "state.imported_source" => "Importado {tail}",
        "state.chapter_fallback" => "Capítulo {index}",
        "runtime.config_path_unresolved" => {
            "No se pudo resolver la ruta del archivo de configuración"
        }
        "runtime.folder_readonly" => "La carpeta está marcada como de solo lectura.",
        "runtime.network_path_warning" => {
            "Ubicada en una ruta de red; los permisos o bloqueos de archivos pueden afectarla"
        }
        "runtime.protected_directory_warning" => "Ubicada en un directorio protegido de Windows",
        "runtime.onedrive_warning" => {
            "Ubicada en una ruta sincronizada con OneDrive; pueden producirse bloqueos de sincronización o seguridad."
        }
        "runtime.youtube_auto_translated_subtitle_429" => {
            "YouTube rechazó temporalmente la solicitud de subtítulos traducidos automáticamente (HTTP 429 Demasiadas solicitudes). Es una limitación de solicitudes del endpoint timedtext de traducción automática de YouTube. La GUI conserva el flujo nativo de yt-dlp y la salida de diagnóstico en lugar de cambiar a un descargador personalizado. Prueba a activar Cookie/cookies.txt para este elemento, o elige subtítulos automáticos originales/subtítulos originales y reintenta. Mensaje original: {error}"
        }
        "runtime.youtube_subtitle_429_conversion" => {
            "YouTube rechazó temporalmente la solicitud de subtítulos (HTTP 429 Demasiadas solicitudes). El archivo de subtítulos de origen no se descargó, por lo que no se ejecutará la conversión a SRT. Reintenta más tarde o activa las cookies del navegador antes de exportar. Mensaje original: {error}"
        }
        "runtime.youtube_subtitle_429_analysis" => {
            "YouTube rechazó la solicitud de subtítulos (HTTP 429 Demasiadas solicitudes). Esto suele ocurrir en el endpoint timedtext de traducción automática de YouTube. cookies.txt puede aportar estado de sesión, pero quizá no satisfaga los requisitos de PO Token o límite de solicitudes de ese endpoint. La GUI conserva el flujo nativo de yt-dlp y los registros de diagnóstico en lugar de cambiar a un descargador personalizado. Mensaje original: {error}"
        }
        "options.filter_executable" => "Ejecutable",

        // Additional Spanish translations keep this locale key-complete.
        "tab.log" => "Registro",
        "advance.download_conversion" => "Convertir después de la descarga",
        "advance.enable" => "Activar",
        "advance.settings" => "Ajustes",
        "options.tabs" => "Pestañas",
        "options.log_tab" => "Pestaña de registro",
        "options.show_log_tab" => "Mostrar registro",
        "options.theme" => "Tema",
        "options.theme_color" => "Color del tema",
        "config.theme.system" => "Seguir el sistema",
        "config.theme.light" => "Claro",
        "config.theme.dark" => "Oscuro",
        "config.theme_color.off" => "Desactivado",
        "config.theme_color.system" => "Azul",
        "config.theme_color.blue" => "Azul suave",
        "config.theme_color.purple" => "Púrpura",
        "config.theme_color.pink" => "Rosa",
        "config.theme_color.green" => "Verde",
        "config.theme_color.orange" => "Naranja",
        "config.theme_color.slate" => "Pizarra",
        "state.transcode_post_processing_title" => "Convirtiendo con {profile}: {title}",
        "processing.transcode" => "Transcodificar",
        "transcode.intent.reduce_size" => "Archivo más pequeño",
        "transcode.intent.quality_first" => "Calidad primero",
        "transcode.intent.target_size" => "Tamaño objetivo",
        "transcode.intent.fast_transcode" => "Formato",
        "transcode.intent.device_compat" => "Compatibilidad del destino",
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
        "transcode.compat.android_phone_tablet" => "Teléfono / tableta Android",
        "transcode.compat.browser_mp4" => "MP4 seguro para navegador",
        "transcode.fps.source" => "Original",
        "transcode.fps.24" => "Hasta 24 fps",
        "transcode.fps.25" => "Hasta 25 fps",
        "transcode.fps.30" => "Hasta 30 fps",
        "transcode.fps.60" => "Hasta 60 fps",
        "transcode.setting.fps" => "Límite de FPS",
        "transcode.graph.axis.compatibility" => "Compatibilidad",
        "transcode.graph.axis.capacity" => "Capacidad",
        "transcode.graph.axis.resolution" => "Resolución",
        "transcode.graph.axis.format" => "Formato",
        "transcode.graph.compatibility_scope" => "Ámbito de compatibilidad",
        "transcode.graph.capacity_target" => "Tamaño objetivo",
        "transcode.graph.resolution_limit" => "Límite de resolución",
        "transcode.graph.format_goal" => "Formato objetivo",
        "transcode.quality.standard" => "Estándar",
        "transcode.quality.high" => "Alta calidad",
        "transcode.quality.near_original" => "Casi original",
        "transcode.resolution.auto_balance" => "Balance automático",
        "transcode.resolution.keep_original" => "Mantener original",
        "transcode.resolution.max_1080p" => "Máx. 1080p",
        "transcode.resolution.max_720p" => "Máx. 720p",
        "transcode.effort.fast" => "Rápido",
        "transcode.effort.normal" => "Normal",
        "transcode.effort.detailed" => "Detallado",
        "transcode.effort.extreme" => "Extremo",
        "transcode.setting.compatibility" => "Compatibilidad",
        "transcode.setting.video_codec" => "Códec de vídeo",
        "transcode.setting.container" => "Contenedor",
        "transcode.setting.encoder" => "Codificador",
        "transcode.setting.quality" => "Calidad",
        "transcode.setting.size_ratio" => "Relación de tamaño",
        "transcode.setting.target_size" => "Tamaño objetivo",
        "transcode.setting.resolution" => "Resolución",
        "transcode.setting.effort" => "Esfuerzo",
        "transcode.setting.pass" => "Control de tamaño",
        "transcode.setting.audio" => "Audio",
        "transcode.support.executable" => "Ejecutable",
        "transcode.support.partial" => "Parcialmente compatible",
        "transcode.support.preview_only" => "Solo vista previa",
        "processing.video" => "Vídeo",
        "processing.audio" => "Audio",
        "processing.container" => "Contenedor",
        "processing.subtitle" => "Subtítulos",
        "processing.choice.source" => "Origen",
        "processing.video.h264" => "H.264",
        "processing.video.hevc" => "HEVC",
        "processing.video.av1" => "AV1",
        "processing.audio.aac" => "AAC",
        "processing.audio.opus" => "Opus",
        "processing.audio.flac" => "FLAC",
        "processing.container.mp4" => "MP4",
        "processing.container.mkv" => "MKV",
        "processing.container.mov" => "MOV",
        "processing.subtitle.preserve" => "Origen",
        "processing.subtitle.embed" => "Incrustar",
        "processing.subtitle.burn" => "Incrustar en vídeo",
        "log.empty" => "Todavía no hay registro de ejecución.",
        "log.clear" => "Borrar",
        "log.copy" => "Copiar",
        "transcode.audio.auto" => "Origen",
        "transcode.audio.aac" => "AAC",
        "transcode.audio.opus" => "Opus",
        "transcode.audio.flac" => "FLAC",
        "runtime.subtitle_burn_no_source" => {
            "La incrustación fija de subtítulos necesita un archivo de subtítulos o subtítulos incrustados. Descarga primero los subtítulos para este elemento o coloca un archivo .srt/.ass junto al vídeo."
        }
        // Additional Spanish translations keep this locale key-complete.
        "item.add_an_audio_url" => "Añade una URL de audio",
        "options.auto_detect_tool_hint" => {
            "Detecta herramientas instaladas desde la carpeta portátil de herramientas y el PATH del sistema."
        }
        "options.cache_usage" => "Uso",
        "options.cache_usage_detail" => "Total: {total} · Audio: {audio} · Caducado: {expired}",
        "options.cache_cleanup" => "Limpieza",
        "options.cache_refresh" => "Actualizar",
        "options.cache_clear_expired" => "Borrar caducados",
        "options.cache_clear_audio" => "Borrar audio",
        "options.cache_clear_all" => "Borrar todo",
        "state.tool_auto_detected" => "{tool} detectado desde PATH: {path}",
        "state.tool_auto_detect_not_found" => "{tool} no se ha encontrado en el sistema PATH.",
        "state.tools_auto_detected" => "Se detectaron {found}/{total} herramientas en PATH.",
        "state.tools_auto_detect_missing" => "No encontrado en PATH: {tools}.",
        "state.tools_auto_detect_none" => {
            "No se han encontrado herramientas de dependencia en el sistema PATH."
        }
        "state.cache_cleaned_expired" => {
            "Se borraron {count} entradas de caché caducadas ({size})."
        }
        "state.cache_cleaned_audio" => "Caché de audio borrada: {count} entradas ({size}).",
        "state.cache_cleaned_all" => "Caché de la app borrada: {count} entradas ({size}).",
        "state.cache_cleanup_failed" => "Error en la limpieza de la caché: {error}",
        "app_mode.origin" => "Modo clásico",
        "app_mode.standard" => "Modo estándar",
        "app_mode.audio" => "Modo de audio",
        "queue_display.normal" => "Estándar",
        "queue_display.audio" => "Audio",
        "music.previous" => "Anterior",
        "music.play" => "Reproducir",
        "music.pause" => "Pausa",
        "music.next" => "Siguiente",
        "music.seek_cached_range_hint" => {
            "Arrastra para buscar; al soltar se ajusta al rango en caché"
        }
        "music.seek_hint" => "Arrastra para buscar",
        "music.status.completed" => "Hecho",
        "music.status.resolving" => "Resolviendo",
        "music.status.buffering" => "En búfer",
        "music.status.ready" => "Listo",
        "music.status.caching" => "Guardando en caché",
        "music.status.playing" => "Reproduciendo",
        "music.status.paused" => "Pausado",
        "music.status.failed" => "Falló",
        "music.playback_mode.sequential" => "Secuencia",
        "music.playback_mode.repeat_all" => "Repetir",
        "music.playback_mode.shuffle" => "Aleatorio",
        "music.playback_mode.repeat_one" => "Repetir una pista",
        "music.playback_mode.sequential.tooltip" => "Reproducir en orden",
        "music.playback_mode.repeat_all.tooltip" => "Repetir lista",
        "music.playback_mode.shuffle.tooltip" => "Reproducción aleatoria",
        "music.playback_mode.repeat_one.tooltip" => "Repetir una pista",
        "options.music_download_format" => "Formato de descarga de audio",
        "options.music_download_format_title" => "¿Qué formato de audio se debe exportar?",
        "options.music_download_format_body" => {
            "Se usa primero la caché de audio ya completada; la conversión solo se ejecuta cuando el formato difiere."
        }
        "state.queue_display_mode_changed" => "Modo de lista: {mode}",
        "state.downloading_music" => "Descargando audio: {title}",
        "state.music_no_items_from_source" => "No se pudieron añadir elementos de audio: {source}",
        "state.music_items_added" => "Se añadieron {count} elementos de audio.",
        "state.music_playlist_parse_failed" => "Falló el análisis de la lista de audio: {error}",
        "state.music_stream_ready" => "Flujo de audio listo: {source}",
        "state.music_stream_parse_failed" => "Falló el análisis del flujo de audio: {error}",
        "state.music_playback_finished" => "Reproducción finalizada.",
        "state.music_playback_failed" => "Error al reproducir: {error}",
        "state.music_duplicate_with_cache" => {
            "El elemento de audio ya está en la lista; se usó la caché local."
        }
        "state.music_duplicate" => "El elemento de audio ya está en la lista.",
        "state.music_added_from_cache" => "Audio añadido desde la caché local: {title}",
        "state.music_seek_clamped" => {
            "Fuera del rango almacenado en caché; se volvió a una posición reproducible."
        }
        "state.music_stream_still_preparing" => "El flujo de audio aún se está preparando.",
        "state.no_playable_music_items" => "No hay elementos de audio reproducibles.",
        "state.music_cache_prepare_failed" => "Falló la preparación de la caché de audio: {error}",
        "state.preparing_music_playback" => "Preparando la reproducción: {title}",
        "state.music_missing_source_url" => "Al elemento de audio le falta una URL de origen.",
        "state.resolving_music_stream" => "Resolviendo flujo de audio: {title}",
        "state.music_stream_still_resolving" => "El flujo de audio aún se está resolviendo.",
        "state.music_buffering" => "El audio está en búfer.",
        "state.music_item_not_playable" => "Este elemento de audio no se puede reproducir ahora.",
        "state.music_stream_not_ready" => "El flujo de audio aún no está listo.",
        "state.no_previous_music" => "No hay pista anterior.",
        "state.no_next_music" => "No hay pista siguiente.",
        "state.music_playback_mode_changed" => "Modo de reproducción: {mode}",
        "action.analyze" => "Analizar",
        "item.download_thumbnail" => "Descargar miniatura",
        "single.title" => "Título",
        "single.description" => "Descripción",
        "single.info.channel" => "Canal",
        "single.info.date" => "Fecha",
        "single.info.views" => "Vistas",
        "thumbnail.filter.jpeg" => "Imagen JPEG",
        "thumbnail.filter.png" => "Imagen PNG",
        "thumbnail.filter.webp" => "Imagen WebP",
        "thumbnail.filter.original" => "Imagen original",
        "state.single_mode_playlist_not_supported" => {
            "El modo clásico no admite URL de listas de reproducción. Cambia al modo estándar para importar una lista."
        }
        "state.single_mode_wait_for_current_item" => {
            "Espera a que termine el elemento actual del modo clásico."
        }
        "state.thumbnail_saved" => "Miniatura guardada: {path}",
        _ => super::en_us::text(key),
    }
}
