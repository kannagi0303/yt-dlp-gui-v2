pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.advanced" => "Avançado",
        "tab.about" => "About",
        "about.tools" => "Versões das ferramentas",
        "about.current_version" => "Atual",
        "about.latest_version" => "Mais recente",
        "about.author" => "Autor",
        "about.source" => "Fonte",
        "about.status" => "Status",
        "about.message" => "Mensagem",
        "about.check_updates" => "Verificar atualizações",
        "about.update_all" => "Atualizar tudo",
        "about.restart" => "Reiniciar",
        "about.open_release" => "Abrir Release",
        "about.install" => "Instalar",
        "about.update" => "Atualizar",
        "about.running" => "Verificação de atualizações em andamento...",
        "about.last_check" => "Última verificação:",
        "about.relative.minutes" => "{count} min",
        "about.relative.hour" => "1 hora",
        "about.relative.hours" => "{count} horas",
        "about.relative.day" => "1 dia",
        "about.relative.days" => "{count} dias",
        "about.never_checked" => "As atualizações ainda não foram verificadas",
        "about.no_release_notes_loaded" => {
            "Nenhuma nota de versão carregada. Verifique atualizações primeiro."
        }
        "about.ownership.managed_portable" => "Gerenciado pelo v2",
        "about.ownership.external" => "Externo",
        "about.ownership.missing" => "Ausente",
        "about.ownership.unknown" => "Desconhecido",
        "about.status.unknown" => "Não verificado",
        "about.status.checking" => "Verificando",
        "about.status.up_to_date" => "Atualizado ✓",
        "about.status.update_available" => "Atualização disponível ↑",
        "about.status.missing" => "Ausente +",
        "about.status.downloading" => "Baixando",
        "about.status.downloading_percent" => "Baixando {percent}%",
        "about.status.staged" => "Preparado",
        "about.status.pending_restart" => "Reinício pendente",
        "about.status.applying" => "Aplicando",
        "about.status.installed" => "Instalado",
        "about.status.skipped" => "Ignorado",
        "about.status.failed" => "Falhou !",
        "tab.options" => "Opções",
        "tab.log" => "Registro",
        "main.url_hint" => "URL",
        "action.download" => "Baixar",
        "action.add" => "Adicionar",
        "action.analyze" => "Analisar",
        "action.stop" => "Parar",
        "action.stopping" => "Parando...",
        "action.cut" => "Recortar",
        "action.copy" => "Copiar",
        "action.paste" => "Colar",
        "action.clear" => "Limpar",
        "item.thumbnail" => "Miniatura",
        "item.thumbnail_preview" => "Prévia da miniatura",
        "single.title" => "Título",
        "single.description" => "Descrição",
        "single.info.channel" => "Canal",
        "single.info.date" => "Data",
        "single.info.views" => "Visualizações",
        "item.download_thumbnail" => "Baixar miniatura",
        "media.video" => "Vídeo",
        "media.audio" => "Áudio",
        "media.subtitle" => "Legendas",
        "media.section" => "Intervalo",
        "item.file_name" => "Nome do arquivo",
        "main.target_folder" => "Pasta de saída",
        "picker.title.video" => "Selecionar formato de vídeo",
        "picker.title.audio" => "Selecionar formato de áudio",
        "picker.title.subtitle" => "Selecionar legendas",
        "picker.title.section" => "Selecionar intervalo",
        "action.back" => "Voltar",
        "picker.mode.filter" => "Filtros",
        "picker.mode.table" => "Tabela",
        "action.confirm" => "Confirmar",
        "picker.empty_table" => "Nenhum formato para exibir",
        "picker.header.resolution" => "Resolução",
        "picker.header.range" => "Intervalo",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "Formato",
        "picker.header.codec" => "Codec",
        "picker.header.size" => "Tamanho",
        "picker.header.sample_rate" => "Taxa de amostragem",
        "picker.filter.resolution" => "Resolução",
        "picker.filter.range" => "Intervalo",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "Codec",
        "picker.filter.sample_rate" => "Taxa de amostragem",
        "main.missing_yt_dlp_callout" => {
            "yt-dlp está ausente. Instale-o ou escolha yt-dlp.exe em Opções."
        }
        "advance.source" => "Origem",
        "advance.config" => "Configuração",
        "advance.none" => "Nenhum",
        "advance.network_access" => "Rede e acesso",
        "advance.proxy" => "Proxy",
        "advance.enable_proxy" => "Ativar proxy",
        "advance.certificate" => "Certificado",
        "advance.skip_certificate_verification" => "Ignorar verificação do certificado",
        "advance.use_cookies" => "Usar cookies",
        "advance.enable_cookies" => "Ativar cookies",
        "advance.cookie_source" => "Origem dos cookies",
        "advance.cookie_source.auto" => "Automático por site",
        "advance.cookie_source.file" => "Usar arquivo (cookies.txt)",
        "advance.cookie_auto" => "Automático",
        "advance.cookie_auto_note" => "Downloads usam o Cookie salvo que corresponde à URL.",
        "advance.cookie_rescue" => "Resgate de Cookie",
        "advance.cookie_file" => "Arquivo de cookies",
        "advance.get_cookie" => "Obter Cookie",
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
            "Abra uma janela de navegador dedicada para obter cookies."
        }
        "youtube_login_rescue.title" => "Resgate de Cookie",
        "youtube_login_rescue.confirm_heading" => "Abrir janela de login dedicada",
        "youtube_login_rescue.confirm_body" => {
            "Uma janela independente do {browser} abrirá a URL sem usar seus dados pessoais do navegador."
        }
        "youtube_login_rescue.target_url_label" => "URL do site",
        "youtube_login_rescue.target_url_hint" => "https://www.youtube.com/",
        "youtube_login_rescue.clipboard_prefilled" => {
            "A URL foi preenchida pela área de transferência."
        }
        "youtube_login_rescue.drop_url_note" => "Cole uma URL ou solte um arquivo .url / de texto.",
        "youtube_login_rescue.paste_clipboard" => "Colar área de transferência",
        "youtube_login_rescue.cookie_note" => {
            "Faça login nessa janela. Quando cookies forem encontrados, a janela será fechada e eles serão aplicados automaticamente."
        }
        "youtube_login_rescue.no_browser_title" => "Nenhum navegador compatível encontrado",
        "youtube_login_rescue.no_browser_body" => {
            "Obter cookies atualmente exige Chrome, Brave ou Microsoft Edge. Você ainda pode escolher cookies.txt manualmente."
        }
        "youtube_login_rescue.start" => "Iniciar",
        "youtube_login_rescue.opening" => "Abrindo {browser}...",
        "youtube_login_rescue.waiting_for_cdp" => {
            "Aguardando a conexão da janela de login do {browser}..."
        }
        "youtube_login_rescue.waiting_for_cookie" => {
            "A janela de login está conectada. Aguardando cookies do site..."
        }
        "youtube_login_rescue.cookie_exported" => "O Cookie foi salvo.",
        "youtube_login_rescue.cookie_exported_note" => {
            "Cookie de {site} salvo. Downloads desse site o usarão automaticamente."
        }
        "youtube_login_rescue.do_not_close_note" => {
            "Mantenha o navegador de login aberto enquanto esta verificação estiver em execução."
        }
        "youtube_login_rescue.cdp_ready" => "A janela de login está conectada.",
        "youtube_login_rescue.ready_next_step_note" => {
            "Conclua o login no YouTube no navegador. A exportação de Cookie será adicionada na próxima etapa."
        }
        "youtube_login_rescue.close_login_window" => "Fechar janela de login",
        "youtube_login_rescue.failed" => "Resgate de Cookie falhou",
        "youtube_login_rescue.retry" => "Tentar novamente",
        "advance.no_cookies_txt_selected" => "Nenhum cookies.txt selecionado",
        "advance.browse" => "Procurar",
        "advance.select_netscape_cookies_txt" => "Selecionar cookies.txt Netscape",
        "advance.clear" => "Limpar",
        "advance.browser" => "Navegador",
        "advance.default" => "Padrão",
        "advance.external_downloader" => "Downloader externo",
        "advance.use_aria2_for_faster_downloads" => "Usar Aria2 para downloads mais rápidos",
        "advance.download_control" => "Controle de download",
        "advance.concurrent_fragments" => "Fragmentos simultâneos",
        "advance.1_default" => "1 (padrão)",
        "advance.rate_limit" => "Limite de velocidade",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => {
            "ex.: 2M, 800K; deixe vazio para ilimitado"
        }
        "advance.chapters" => "Capítulos",
        "advance.chapter_download_compatibility_mode" => {
            "Modo de compatibilidade de download por capítulos"
        }
        "advance.file_time" => "Hora do arquivo",
        "advance.file_time.none" => "Não alterar",
        "advance.file_time.upload_date" => "Usar data de envio do vídeo",
        "advance.file_time.download_time" => "Usar hora do download",
        "advance.post_processing" => "Pós-processamento",
        "advance.thumbnail" => "Miniatura",
        "advance.download" => "Baixar",
        "advance.embed" => "Incorporar",
        "advance.subtitles" => "Legendas",
        "advance.download_conversion" => "Converter após baixar",
        "advance.enable" => "Ativar",
        "advance.settings" => "Configurações",
        "item.save_as" => "Salvar como",
        "item.error" => "Erro",
        "item.all" => "Todos",
        "item.queued" => "Na fila",
        "item.done" => "Concluído",
        "item.failed" => "Falhou",
        "item.clear_all" => "Limpar tudo",
        "item.add_a_video_url" => "Adicionar URL de vídeo",
        "item.add_an_audio_url" => "Adicionar URL de áudio",
        "item.after_adding_choose_the_video_format_here" => "Escolher formato de vídeo",
        "item.after_adding_choose_the_audio_format_here" => "Escolher formato de áudio",
        "item.loading_thumbnail" => "Carregando miniatura",
        "item.file_actions" => "Ações de arquivo",
        "item.open_file" => "Abrir arquivo",
        "item.open_folder" => "Abrir pasta",
        "item.copy_path" => "Copiar caminho",
        "item.file_not_found_opened_the_output_location" => {
            "Arquivo não encontrado; local de saída aberto."
        }
        "item.opened_output_location" => "Local de saída aberto.",
        "item.copied_output_path" => "Caminho de saída copiado.",
        "prepare.language" => "Idioma",
        "prepare.back" => "Voltar",
        "prepare.auto_detect" => "Detectar automaticamente",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "Instale agora as ferramentas necessárias ou pule e configure depois em Opções."
        }
        "prepare.optional" => "Opcional",
        "prepare.missing" => "Ausente",
        "prepare.install_later" => "Instalar depois",
        "prepare.downloading_100" => "Baixando 100%",
        "prepare.extracting_100" => "Extraindo 100%",
        "prepare.install_failed" => "Falha na instalação",
        "prepare.install_all" => "Instalar tudo",
        "prepare.reinstall" => "Reinstalar",
        "prepare.installing" => "Instalando",
        "prepare.skip" => "Pular",
        "prepare.install" => "Instalar",
        "prepare.another_tool_is_already_being_installed" => {
            "Outra ferramenta já está sendo instalada."
        }
        "prepare.needs_attention" => "Requer atenção",
        "prepare.req.app_folder.title" => "Pasta do app",
        "prepare.req.app_folder.description" => {
            "A pasta portátil precisa permitir gravação para salvar configurações e dados de suporte."
        }
        "prepare.req.tools_folder.title" => "Pasta de ferramentas",
        "prepare.req.tools_folder.description" => {
            "A implantação das dependências armazena yt-dlp, FFmpeg e Deno aqui."
        }
        "prepare.req.deployment_temp.title" => "Temporário de implantação",
        "prepare.req.deployment_temp.description" => {
            "A extração do FFmpeg e do Deno usa esta pasta temporária."
        }
        "prepare.req.download_cache.title" => "Cache de download",
        "prepare.req.download_cache.description" => {
            "O modo de cache do yt-dlp-gui armazena aqui o cache do yt-dlp."
        }
        "prepare.req.output_folder.title" => "Pasta de saída",
        "prepare.req.output_folder.description" => "Vídeos, áudio e legendas são salvos aqui.",
        "prepare.req.output_folder.recommendation" => {
            "Escolha uma pasta de saída válida na tela principal ou em Opções."
        }
        "prepare.req.config_file.title" => "Arquivo de configuração",
        "prepare.req.config_file.description" => {
            "O app precisa salvar o estado de pular o Prepare e os caminhos das ferramentas."
        }
        "prepare.req.generic_writable_recommendation" => {
            "Escolha uma pasta gravável e verifique as permissões."
        }
        "prepare.req.config_not_folder" => {
            "O caminho de configuração aponta para uma pasta. Escolha um caminho de arquivo."
        }
        "prepare.req.config_readonly" => "O arquivo de configuração é somente leitura.",
        "prepare.req.config_readonly_recommendation" => {
            "Permita gravar no arquivo de configuração ou escolha outra pasta do app."
        }
        "prepare.req.use_folder_path" => {
            "Escolha um caminho de pasta em vez de um caminho de arquivo."
        }
        "prepare.req.move_portable_folder" => "Mova o app para uma pasta portátil gravável.",
        "prepare.req.avoid_protected_folder" => {
            "Não coloque o app portátil em Program Files nem na pasta Windows. Mova-o para D:\\Portable ou uma pasta do usuário."
        }
        "prepare.req.move_non_synced_folder" => {
            "Mova-o para uma pasta não sincronizada, por exemplo D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.drive_parent_exists" => "Verifique se a unidade e a pasta pai existem.",
        "prepare.req.permission_denied" => {
            "Mova o app para uma pasta portátil gravável. Se Área de Trabalho/Documentos/Downloads ainda falharem, o Acesso controlado a pastas do Defender pode estar bloqueando."
        }
        "prepare.req.file_in_use" => {
            "Feche o programa que pode estar usando esta pasta ou escolha outra pasta."
        }
        "prepare.req.free_disk_space" => "Libere espaço em disco ou escolha outro disco.",
        "prepare.req.path_too_long" => {
            "Mova o app para um caminho mais curto, por exemplo D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.choose_writable_portable_folder" => {
            "Escolha uma pasta portátil claramente gravável e verifique novamente."
        }
        "prepare.req.clear_write_test" => {
            "Remova o arquivo restante do teste de gravação e verifique novamente."
        }
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "Este URL contém um vídeo e uma playlist"
        }
        "options.detected" => "Detectado ",
        "options.playlist_prompt" => "Pergunta de playlist",
        "options.which_one_should_be_loaded" => "Qual deve ser carregado?",
        "options.both_video_and_playlist_were_detected" => "Vídeo e playlist foram detectados",
        "options.this_playlist_may_contain_many_items" => "Esta playlist pode conter muitos itens.",
        "options.playlist_risk.kind.channel_generated" => "Playlist de canal gerada pelo YouTube",
        "options.playlist_risk.kind.youtube_mix_radio" => "YouTube Mix / Radio",
        "options.playlist_risk.kind.youtube_music_album" => "Álbum/coleção do YouTube Music",
        "options.playlist_risk.kind.liked_videos" => "Vídeos curtidos",
        "options.playlist_risk.kind.favorites_legacy" => "Playlist antiga de favoritos",
        "options.playlist_risk.note.channel_generated" => {
            "Trate esta playlist de canal gerada pelo YouTube com cautela."
        }
        "options.playlist_risk.note.youtube_mix_radio" => {
            "Esta playlist Mix / Radio pode conter muitos itens e mudar com o tempo."
        }
        "options.playlist_risk.note.youtube_music_album" => {
            "Geralmente é um álbum ou coleção do YouTube Music."
        }
        "options.playlist_risk.note.liked_videos" => {
            "Vídeos curtidos geralmente exigem login ou cookies."
        }
        "options.playlist_risk.note.favorites_legacy" => {
            "Este é um estilo antigo de playlist de favoritos e pode não estar estável agora."
        }
        "options.video" => "Vídeo",
        "options.playlist" => "Playlist",
        "options.cancel" => "Cancelar",
        "options.load" => "Carregar",
        "options.behavior" => "Comportamento",
        "options.add_action" => "Ação ao adicionar",
        "options.download_directly" => "Baixar diretamente",
        "options.clipboard_change" => "Alteração da área de transferência",
        "options.run_immediately" => "Executar imediatamente",
        "options.tabs" => "Abas",
        "options.log_tab" => "Aba de registro",
        "options.show_log_tab" => "Mostrar registro",
        "options.playlist_2" => "Playlist",
        "options.with_playlist" => "Com playlist",
        "options.ask" => "Perguntar",
        "options.single_video" => "Vídeo",
        "options.full_playlist" => "[Todos]",
        "options.high_risk_prompt" => "Aviso de alto risco",
        "options.on" => "Ligado",
        "options.playlist_count" => "Contagem da playlist",
        "options.limit" => "Limite",
        "options.max" => "Máx.:",
        "options.items" => " itens",
        "options.language" => "Idioma",
        "options.current_language" => "Idioma atual",
        "options.back" => "Voltar",
        "options.choose" => "Escolher",
        "options.auto_detect" => "Detectar automaticamente",
        "options.tool_paths" => "Caminhos das ferramentas",
        "options.file_actions" => "Ações de arquivo",
        "options.action_button" => "Botão de ação",
        "options.file_action.show_menu" => "Mostrar menu",
        "options.cache" => "Cache",
        "options.cache_location" => "Local do cache",
        "options.cache_location.default" => "Padrão",
        "options.cache_usage" => "Uso",
        "options.cache_usage_detail" => "Total: {total} · Áudio: {audio} · Expirado: {expired}",
        "options.cache_cleanup" => "Limpeza",
        "options.cache_refresh" => "Atualizar",
        "options.cache_clear_expired" => "Limpar expirados",
        "options.cache_clear_audio" => "Limpar áudio",
        "options.cache_clear_all" => "Limpar tudo",
        "options.appearance_window" => "Aparência e janela",
        "options.notifications" => "Notificações",
        "options.enable" => "Ativar",
        "options.theme" => "Tema",
        "options.theme_mode.system" => "Seguir o sistema",
        "options.theme_mode.light" => "Claro",
        "options.theme_mode.dark" => "Escuro",
        "options.theme_color" => "Cor do tema",
        "options.theme_color.off" => "Desativado",
        "options.theme_color.blue" => "Azul",
        "options.theme_color.soft_blue" => "Azul suave",
        "options.theme_color.purple" => "Roxo",
        "options.theme_color.pink" => "Rosa",
        "options.theme_color.green" => "Verde",
        "options.theme_color.orange" => "Laranja",
        "options.theme_color.slate" => "Ardósia",
        "options.ui_scale" => "Escala da interface",
        "options.apply" => "Aplicar",
        "options.current" => "Atual",
        "options.always_on_top" => "Sempre no topo",
        "options.window_position" => "Posição da janela",
        "options.remember" => "Lembrar",
        "options.window_size" => "Tamanho da janela",
        "options.reinstall" => "Reinstalar",
        "options.installing" => "Instalando",
        "options.install" => "Instalar",
        "options.executable" => "executável",
        "main.controlled_by_config" => "Controlado pela configuração: ",
        "main.controlled_by_config_2" => "Controlado pela configuração",
        "picker.no_chapters_available" => "Nenhum capítulo disponível.",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "Escolha o intervalo para baixar neste item. O padrão é o vídeo inteiro."
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "O modo de compatibilidade de capítulos está ativo: downloads por capítulo usarão um formato de arquivo único mais estável."
        }
        "picker.subtitles_will_not_be_downloaded" => "As legendas não serão baixadas.",
        "picker.no_subtitles_are_available_for_this_video" => {
            "Nenhuma legenda disponível para este vídeo."
        }
        "picker.no_subtitles_are_available_in_this_tab" => "Nenhuma legenda disponível nesta aba.",
        "picker.source_language" => "Idioma de origem",
        "picker.translation_target" => "Destino da tradução",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "Dica: legendas traduzidas automaticamente pelo YouTube têm maior chance de limite de taxa do que legendas originais. Escolha “Sem tradução” se precisar apenas do texto original."
        }
        "picker.no_subtitles_are_available_for_this_source" => {
            "Nenhuma legenda disponível para esta origem."
        }
        "picker.target" => "Destino",
        "picker.available_subtitles" => "Legendas disponíveis",
        "picker.language" => "Idioma",
        "picker.subtitle_tab.none" => "Sem legendas",
        "picker.subtitle_tab.original" => "Legendas originais",
        "picker.subtitle_tab.automatic" => "Legendas automáticas",
        "picker.waiting_analysis" => "Aguardando análise",
        "picker.audio_from_video" => "Definido pelo formato de vídeo",
        "picker.not_selected" => "Não selecionado",
        "picker.full_video" => "Vídeo completo",
        "picker.no_translation" => "Sem tradução",
        "picker.until_end" => "fim",
        "prepare.status.ready" => "Pronto",
        "prepare.status.missing" => "Ausente",
        "prepare.status.warning" => "Requer atenção",
        "prepare.status.failed" => "Falhou",
        "tool_install.stage.preparing" => "Preparando",
        "tool_install.stage.downloading" => "Baixando",
        "tool_install.stage.extracting" => "Extraindo",
        "tool_install.stage.installing" => "Instalando",
        "tool_install.stage.completed" => "Concluído",
        "tool_install.stage.failed" => "Falhou",
        "item.status.queued" => "Na fila",
        "item.status.running" => "Em execução",
        "item.status.finished" => "Concluído",
        "item.status.failed" => "Falhou",
        "item.status.cancelled" => "Cancelado",
        "processing.transcode" => "Transcodificar",
        "transcode.graph.axis.compatibility" => "Compatibilidade",
        "transcode.graph.axis.capacity" => "Capacidade",
        "transcode.graph.axis.resolution" => "Resolução",
        "transcode.graph.axis.format" => "Formato",
        "transcode.graph.compatibility_scope" => "Escopo de compatibilidade",
        "transcode.graph.capacity_target" => "Meta de tamanho",
        "transcode.graph.resolution_limit" => "Limite de resolução",
        "transcode.graph.format_goal" => "Meta de formato",
        "processing.video" => "Vídeo",
        "processing.audio" => "Áudio",
        "processing.container" => "Contêiner",
        "processing.subtitle" => "Legendas",
        "processing.choice.source" => "Original",
        "processing.subtitle.preserve" => "Original",
        "processing.subtitle.embed" => "Incorporar",
        "processing.subtitle.burn" => "Gravar no vídeo",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "Todos os arquivos",
        "options.filter_executable" => "Executável",
        "app_mode.origin" => "Modo Origin",
        "app_mode.standard" => "Modo padrão",
        "app_mode.audio" => "Modo áudio",
        "music.status.completed" => "Concluído",
        "music.status.resolving" => "Resolvendo",
        "music.status.buffering" => "Carregando buffer",
        "music.status.ready" => "Pronto",
        "music.status.caching" => "Armazenando em cache",
        "music.status.playing" => "Reproduzindo",
        "music.status.paused" => "Pausado",
        "music.status.failed" => "Falhou",
        "notification.download_complete" => "Download concluído",
        "notification.download_failed" => "Falha no download",
        "notification.completed_file" => "Concluído: {file}",
        "notification.download_completed" => "Download concluído.",
        "options.music_download_format" => "Music download format",
        "options.music_download_audio_label" => "Saída de áudio",
        "options.music_download_preference_best" => "Melhor",
        _ => key,
    }
}
