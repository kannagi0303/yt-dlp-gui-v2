use super::*;

impl AppState {
    pub fn available_cache_location_modes(&self) -> [CacheLocationMode; 3] {
        [
            CacheLocationMode::YtDlpDefault,
            CacheLocationMode::V2Cache,
            CacheLocationMode::WindowsTemp,
        ]
    }

    pub fn set_cache_location_mode(&mut self, mode: CacheLocationMode) {
        self.tool_paths.cache_mode = mode;
        self.config.cache_location_mode = match mode {
            CacheLocationMode::YtDlpDefault => SerializableCacheLocationMode::YtDlpDefault,
            CacheLocationMode::V2Cache => SerializableCacheLocationMode::V2Cache,
            CacheLocationMode::WindowsTemp => SerializableCacheLocationMode::WindowsTemp,
        };
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn should_show_prepare_tab(&self) -> bool {
        !self.config.prepare_skipped
            && !self.prepare_tab_snoozed
            && self.prepare_report.should_show_tab()
    }

    pub fn prepare_requirements(&self) -> &[PrepareRequirement] {
        &self.prepare_report.requirements
    }

    pub fn refresh_prepare_report(&mut self) {
        self.prepare_report =
            collect_prepare_report(&self.tool_paths, &self.item_defaults.output_dir);
        if !self.should_show_prepare_tab() && self.active_tab == AppTab::Prepare {
            self.active_tab = AppTab::Main;
        }
    }

    pub(super) fn sanitize_startup_prepare_component_update_snapshot(&mut self) {
        self.component_update_snapshot.running = false;

        for tool in self.prepare_install_order() {
            let id = ManagedComponentId::for_dependency_tool(tool);
            let Some(status) = self
                .component_update_snapshot
                .entry(id)
                .map(|entry| entry.status)
            else {
                continue;
            };
            if !matches!(
                status,
                ComponentUpdateStatus::Checking
                    | ComponentUpdateStatus::Downloading
                    | ComponentUpdateStatus::Staged
                    | ComponentUpdateStatus::Applying
                    | ComponentUpdateStatus::Failed
            ) {
                continue;
            }

            let installed = dependency_tool_is_available(tool, self.dependency_tool_path(tool));
            let entry = self.component_update_snapshot.ensure_entry_mut(id);
            entry.status = if installed {
                ComponentUpdateStatus::Unknown
            } else {
                ComponentUpdateStatus::Missing
            };
            entry.progress = None;
            entry.message = if installed {
                "not checked".to_owned()
            } else {
                "not installed".to_owned()
            };
        }
    }

    pub fn prepare_installable_tool_count(&self) -> usize {
        self.prepare_tools_to_install_all().len()
    }

    pub fn prepare_dependency_install_block_reason(&self) -> Option<String> {
        let blocking_issue = self.prepare_report.requirements.iter().find(|item| {
            item.action.is_none()
                && item.status == PrepareStatus::Failed
                && matches!(
                    item.id.as_str(),
                    "app-root" | "config-file" | "tools-dir" | "manifest-temp"
                )
        })?;

        Some(i18n::format_fixed_english(
            "Handle {items} before installing dependency tools.",
            &[("{items}", blocking_issue.title.as_str())],
        ))
    }

    fn component_update_block_reason(&self, target: Option<ManagedComponentId>) -> Option<String> {
        let needs_tools_dir = !matches!(target, Some(ManagedComponentId::App));
        let blocking_issue = self.prepare_report.requirements.iter().find(|item| {
            item.action.is_none()
                && item.status == PrepareStatus::Failed
                && (matches!(
                    item.id.as_str(),
                    "app-root" | "config-file" | "manifest-temp"
                ) || (needs_tools_dir && item.id == "tools-dir"))
        })?;

        Some(i18n::format_fixed_english(
            "Handle {items} before updating managed components.",
            &[("{items}", blocking_issue.title.as_str())],
        ))
    }

    pub fn prepare_footer_status_text(&self) -> Option<String> {
        if self.component_update_snapshot.running {
            return Some(self.prepare_component_update_status_summary_text());
        }

        let message = self.last_action.trim();
        if message.is_empty() {
            return None;
        }

        Some(match message {
            "checking updates" => self
                .ui_i18n_text_for_key("about.status.checking")
                .to_owned(),
            "updating managed components" => self.ui_i18n_text_for_key("about.running").to_owned(),
            "update check complete" => self
                .ui_i18n_text_for_key("tool_install.stage.completed")
                .to_owned(),
            _ if message.starts_with("updating ") => {
                self.ui_i18n_text_for_key("about.running").to_owned()
            }
            _ => self.localize_message(message),
        })
    }

    fn prepare_component_update_status_summary_text(&self) -> String {
        for status in [
            ComponentUpdateStatus::Applying,
            ComponentUpdateStatus::Downloading,
            ComponentUpdateStatus::Staged,
            ComponentUpdateStatus::Checking,
            ComponentUpdateStatus::UpdateAvailable,
            ComponentUpdateStatus::Missing,
        ] {
            if let Some(text) = self.prepare_first_component_status_text(status) {
                return text;
            }
        }

        self.ui_i18n_text_for_key("about.running").to_owned()
    }

    fn prepare_first_component_status_text(&self, status: ComponentUpdateStatus) -> Option<String> {
        for tool in self.prepare_install_order() {
            let entry = self
                .component_update_snapshot
                .entry(ManagedComponentId::for_dependency_tool(tool))?;
            if entry.status != status {
                continue;
            }

            let status_text = self.component_update_status_text(entry);
            return Some(format!("{}: {status_text}", tool.label()));
        }

        None
    }

    pub fn install_all_prepare_tools(&mut self) {
        if self.component_update_running() {
            self.last_action = "Update check is already running.".to_owned();
            return;
        }

        self.refresh_prepare_report();
        if let Some(reason) = self.prepare_dependency_install_block_reason() {
            self.last_action = reason;
            return;
        }

        let tools = self.prepare_tools_to_install_all();
        if tools.is_empty() {
            self.last_action = "There are no tools to install.".to_owned();
            return;
        }

        self.update_dependency_tools_for_prepare(tools);
    }

    pub fn snooze_prepare_tab(&mut self) {
        let previous_prepare_skipped = self.config.prepare_skipped;
        self.config.prepare_skipped = true;

        match self.config.save() {
            Ok(()) => {
                self.prepare_tab_snoozed = true;
                if self.active_tab == AppTab::Prepare {
                    self.active_tab = AppTab::Main;
                }
                self.last_action =
                    "Prepare page skipped. You can handle dependency deployment later in Options."
                        .to_owned();
            }
            Err(error) => {
                self.config.prepare_skipped = previous_prepare_skipped;
                self.prepare_tab_snoozed = false;
                let localized_error = self.localize_message(&error);
                self.last_action = i18n::format_fixed_english(
                    "Skip failed: {error}",
                    &[("{error}", localized_error.as_str())],
                );
                self.refresh_prepare_report();
            }
        }
    }

    pub fn reopen_prepare_tab(&mut self) {
        self.prepare_tab_snoozed = false;
        self.config.prepare_skipped = false;
        let _ = self.config.save();
        self.refresh_prepare_report();
        if self.should_show_prepare_tab() {
            self.active_tab = AppTab::Prepare;
        }
    }

    fn prepare_tools_to_install_all(&self) -> Vec<DependencyTool> {
        self.prepare_install_order()
            .into_iter()
            .filter(|tool| self.prepare_tool_needs_install(*tool))
            .collect()
    }

    fn prepare_tool_needs_install(&self, tool: DependencyTool) -> bool {
        self.prepare_report
            .requirements
            .iter()
            .any(|item| item.needs_attention() && item.has_install_action(tool))
    }

    fn prepare_install_order(&self) -> [DependencyTool; 3] {
        [
            DependencyTool::YtDlp,
            DependencyTool::Deno,
            DependencyTool::Ffmpeg,
        ]
    }

    pub fn select_about_detail(&mut self, target: AboutDetailTarget) {
        self.about_detail_target = target;
        match target {
            AboutDetailTarget::App => {
                self.component_update_snapshot.selected = Some(ManagedComponentId::App)
            }
            AboutDetailTarget::Tool(id) => self.component_update_snapshot.selected = Some(id),
        }
    }

    pub fn component_update_running(&self) -> bool {
        self.component_update_snapshot.running
    }

    pub fn component_update_attention_signal_visible(&self) -> bool {
        self.component_update_snapshot
            .entries
            .iter()
            .any(|entry| component_update_status_needs_attention_signal(entry.status))
    }

    pub fn check_component_updates(&mut self) {
        if self.component_update_running() {
            self.last_action = "Update check is already running.".to_owned();
            return;
        }
        let proxy_url = self.tool_paths.effective_proxy_url().map(str::to_owned);
        self.component_update_snapshot.running = true;
        self.component_update_snapshot.message = "checking updates".to_owned();
        run_component_update_worker(
            ComponentUpdateAction::CheckAll,
            proxy_url,
            self.component_update_result_tx.clone(),
        );
    }

    pub fn update_all_managed_components(&mut self) {
        if self.component_update_running() {
            self.last_action = "Update check is already running.".to_owned();
            return;
        }
        self.refresh_prepare_report();
        if let Some(reason) = self.component_update_block_reason(None) {
            self.last_action = reason;
            return;
        }
        let proxy_url = self.tool_paths.effective_proxy_url().map(str::to_owned);
        self.component_update_snapshot.running = true;
        self.component_update_snapshot.message = "updating managed components".to_owned();
        run_component_update_worker(
            ComponentUpdateAction::UpdateAllManaged,
            proxy_url,
            self.component_update_result_tx.clone(),
        );
    }

    fn update_dependency_tools_for_prepare(&mut self, tools: Vec<DependencyTool>) {
        let ids = tools
            .into_iter()
            .map(ManagedComponentId::for_dependency_tool)
            .collect::<Vec<_>>();
        if ids.is_empty() {
            self.last_action = "There are no tools to install.".to_owned();
            return;
        }

        let proxy_url = self.tool_paths.effective_proxy_url().map(str::to_owned);
        self.component_update_snapshot.running = true;
        self.component_update_snapshot.selected = ids.first().copied();
        self.component_update_snapshot.message = format!(
            "updating {}",
            ids.iter()
                .map(|id| id.label())
                .collect::<Vec<_>>()
                .join(", ")
        );
        for id in ids.iter().copied() {
            let entry = self.component_update_snapshot.ensure_entry_mut(id);
            entry.status = ComponentUpdateStatus::Checking;
            entry.progress = None;
            entry.message = "queued".to_owned();
        }
        run_component_update_worker(
            ComponentUpdateAction::UpdateMany(ids),
            proxy_url,
            self.component_update_result_tx.clone(),
        );
    }

    pub fn update_component(&mut self, id: ManagedComponentId) {
        if self.component_update_running() {
            self.last_action = "Update check is already running.".to_owned();
            return;
        }
        self.refresh_prepare_report();
        if let Some(reason) = self.component_update_block_reason(Some(id)) {
            self.last_action = reason;
            return;
        }
        let proxy_url = self.tool_paths.effective_proxy_url().map(str::to_owned);
        self.component_update_snapshot.running = true;
        self.component_update_snapshot.selected = Some(id);
        self.component_update_snapshot.message = format!("updating {}", id.label());
        run_component_update_worker(
            ComponentUpdateAction::UpdateOne(id),
            proxy_url,
            self.component_update_result_tx.clone(),
        );
    }

    pub fn restart_to_apply_app_update(&mut self) -> Result<(), String> {
        launch_pending_app_update(true)
    }

    pub fn app_update_pending_restart(&self) -> bool {
        self.component_update_snapshot
            .entry(ManagedComponentId::App)
            .is_some_and(|entry| entry.status == ComponentUpdateStatus::PendingRestart)
    }

    pub fn set_yt_dlp_path(&mut self, path: impl Into<String>) {
        self.config.set_yt_dlp_path(path);
        self.tool_paths.yt_dlp = self.config.yt_dlp_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn set_yt_dlp_config_path(&mut self, path: impl Into<String>) {
        self.config.set_yt_dlp_config_path(path);
        self.tool_paths.yt_dlp_config = self.config.yt_dlp_config_path.clone();
        let _ = self.config.save();
    }

    pub fn available_yt_dlp_config_files(&self) -> Vec<ConfigFileOption> {
        available_yt_dlp_config_files()
    }

    pub fn yt_dlp_configs_dir_display(&self) -> String {
        yt_dlp_configs_dir_display()
    }

    pub fn set_ffmpeg_path(&mut self, path: impl Into<String>) {
        self.config.set_ffmpeg_path(path);
        self.tool_paths.ffmpeg = self.config.ffmpeg_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn set_aria2c_path(&mut self, path: impl Into<String>) {
        self.config.set_aria2c_path(path);
        self.tool_paths.aria2c = self.config.aria2c_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub fn set_deno_path(&mut self, path: impl Into<String>) {
        self.config.set_deno_path(path);
        self.tool_paths.deno = self.config.deno_path.clone();
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    pub(super) fn sync_available_managed_tool_paths_from_update_snapshot(&mut self) {
        let available_tools = self
            .component_update_snapshot
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry.status,
                    ComponentUpdateStatus::Installed | ComponentUpdateStatus::UpToDate
                )
            })
            .filter_map(|entry| entry.id.as_dependency_tool())
            .collect::<Vec<_>>();
        if available_tools.is_empty() {
            return;
        }

        let mut changed = false;
        for tool in available_tools {
            let current_path = self.dependency_tool_path(tool).to_owned();
            if !current_path.trim().is_empty()
                && dependency_tool_is_available(tool, current_path.as_str())
            {
                continue;
            }
            let managed_path = tool.default_portable_path().to_owned();
            if !dependency_tool_is_available(tool, managed_path.as_str()) {
                continue;
            }
            changed |= self.set_dependency_tool_path_without_refresh(tool, managed_path);
        }

        if changed {
            let _ = self.config.save();
        }
    }

    pub fn install_dependency_tool(&mut self, tool: DependencyTool) {
        if self.active_tab == AppTab::Prepare {
            if let Some(reason) = self.prepare_dependency_install_block_reason() {
                self.last_action = reason;
                return;
            }
        }
        self.update_component(ManagedComponentId::for_dependency_tool(tool));
    }

    pub fn dependency_tool_update_is_running(&self, tool: DependencyTool) -> bool {
        let id = ManagedComponentId::for_dependency_tool(tool);
        self.component_update_snapshot.running
            && self
                .component_update_snapshot
                .entry(id)
                .is_some_and(|entry| {
                    matches!(
                        entry.status,
                        ComponentUpdateStatus::Checking
                            | ComponentUpdateStatus::Downloading
                            | ComponentUpdateStatus::Staged
                            | ComponentUpdateStatus::Applying
                    )
                })
    }

    pub fn dependency_tool_update_status(
        &self,
        tool: DependencyTool,
    ) -> Option<ComponentUpdateStatus> {
        self.visible_prepare_dependency_update_entry(tool)
            .map(|entry| entry.status)
    }

    pub fn dependency_tool_update_status_text(&self, tool: DependencyTool) -> Option<String> {
        self.visible_prepare_dependency_update_entry(tool)
            .map(|entry| self.component_update_status_text(entry))
    }

    fn visible_prepare_dependency_update_entry(
        &self,
        tool: DependencyTool,
    ) -> Option<&ComponentUpdateEntry> {
        let id = ManagedComponentId::for_dependency_tool(tool);
        let entry = self.component_update_snapshot.entry(id)?;
        prepare_dependency_update_status_is_visible(
            entry.status,
            self.component_update_snapshot.running,
            self.dependency_tool_update_is_running(tool),
            self.dependency_tool_is_installed(tool),
        )
        .then_some(entry)
    }

    fn component_update_status_text(&self, entry: &ComponentUpdateEntry) -> String {
        match entry.status {
            ComponentUpdateStatus::Unknown => self.ui_i18n_text_for_key("about.status.unknown"),
            ComponentUpdateStatus::Checking => self.ui_i18n_text_for_key("about.status.checking"),
            ComponentUpdateStatus::UpToDate => self.ui_i18n_text_for_key("about.status.up_to_date"),
            ComponentUpdateStatus::UpdateAvailable => {
                self.ui_i18n_text_for_key("about.status.update_available")
            }
            ComponentUpdateStatus::Missing => self.ui_i18n_text_for_key("about.status.missing"),
            ComponentUpdateStatus::Downloading => {
                let text = if let Some(percent) = entry.progress {
                    let percent = percent.to_string();
                    self.ui_i18n_text_with_replacements(
                        "about.status.downloading_percent",
                        &[("{percent}", percent.as_str())],
                    )
                } else {
                    self.ui_i18n_text_for_key("about.status.downloading")
                        .to_owned()
                };
                return self.component_update_status_size_text(text, entry.total_size_bytes);
            }
            ComponentUpdateStatus::Staged => self.ui_i18n_text_for_key("about.status.staged"),
            ComponentUpdateStatus::PendingRestart => {
                self.ui_i18n_text_for_key("about.status.pending_restart")
            }
            ComponentUpdateStatus::Applying if !entry.message.trim().is_empty() => {
                let text = self.localize_message(&entry.message);
                return if let Some(percent) = entry.progress {
                    format!("{text} {percent}%")
                } else {
                    text
                };
            }
            ComponentUpdateStatus::Applying => self.ui_i18n_text_for_key("about.status.applying"),
            ComponentUpdateStatus::Installed => self.ui_i18n_text_for_key("about.status.installed"),
            ComponentUpdateStatus::Skipped => self.ui_i18n_text_for_key("about.status.skipped"),
            ComponentUpdateStatus::Failed => self.ui_i18n_text_for_key("about.status.failed"),
        }
        .to_owned()
    }

    fn component_update_status_size_text(&self, text: String, size_bytes: Option<u64>) -> String {
        match size_bytes {
            Some(size_bytes) => format!("{text} ({})", format_byte_size(size_bytes)),
            None => text,
        }
    }

    pub fn dependency_tool_path(&self, tool: DependencyTool) -> &str {
        match tool {
            DependencyTool::YtDlp => &self.tool_paths.yt_dlp,
            DependencyTool::Ffmpeg => &self.tool_paths.ffmpeg,
            DependencyTool::Aria2c => &self.tool_paths.aria2c,
            DependencyTool::Deno => &self.tool_paths.deno,
        }
    }

    pub fn dependency_tool_is_installed(&self, tool: DependencyTool) -> bool {
        dependency_tool_is_available(tool, self.dependency_tool_path(tool))
    }

    pub fn auto_detect_dependency_tool_path(&mut self, tool: DependencyTool) {
        match detect_dependency_tool(tool) {
            Some(path) => {
                let display_path = path.display().to_string();
                self.set_dependency_tool_path(tool, display_path.clone());
                self.last_action = i18n::format_fixed_english(
                    "{tool} detected automatically: {path}",
                    &[("{tool}", tool.label()), ("{path}", display_path.as_str())],
                );
            }
            None => {
                self.last_action = i18n::format_fixed_english(
                    "{tool} was not found beside the app, its subfolders, or system PATH.",
                    &[("{tool}", tool.label())],
                );
            }
        }
    }

    pub fn auto_detect_dependency_tool_paths(&mut self) {
        const TOOLS: [DependencyTool; 4] = [
            DependencyTool::YtDlp,
            DependencyTool::Deno,
            DependencyTool::Ffmpeg,
            DependencyTool::Aria2c,
        ];

        let mut detected = Vec::new();
        let mut missing = Vec::new();

        for tool in TOOLS {
            match detect_dependency_tool(tool) {
                Some(path) => {
                    let display_path = path.display().to_string();
                    self.set_dependency_tool_path(tool, display_path.clone());
                    detected.push(format!("{}: {}", tool.label(), display_path));
                }
                None => missing.push(tool.label()),
            }
        }

        if detected.is_empty() {
            self.last_action =
                "No dependency tools were found beside the app, its subfolders, or system PATH."
                    .to_owned();
            return;
        }

        let found_count = detected.len().to_string();
        let total_count = TOOLS.len().to_string();
        let mut message = i18n::format_fixed_english(
            "Automatically detected {found}/{total} tools.",
            &[
                ("{found}", found_count.as_str()),
                ("{total}", total_count.as_str()),
            ],
        );
        message.push_str("\n");
        message.push_str(&detected.join("\n"));
        if !missing.is_empty() {
            message.push_str("\n");
            message.push_str(&i18n::format_fixed_english(
                "Not found beside the app, its subfolders, or system PATH: {tools}.",
                &[("{tools}", missing.join(", ").as_str())],
            ));
        }
        self.last_action = message;
    }

    fn set_dependency_tool_path(&mut self, tool: DependencyTool, path: String) {
        self.set_dependency_tool_path_without_refresh(tool, path);
        let _ = self.config.save();
        self.refresh_prepare_report();
    }

    fn set_dependency_tool_path_without_refresh(
        &mut self,
        tool: DependencyTool,
        path: String,
    ) -> bool {
        match tool {
            DependencyTool::YtDlp => {
                let before = self.config.yt_dlp_path.clone();
                self.config.set_yt_dlp_path(path);
                self.tool_paths.yt_dlp = self.config.yt_dlp_path.clone();
                before != self.config.yt_dlp_path
            }
            DependencyTool::Ffmpeg => {
                let before = self.config.ffmpeg_path.clone();
                self.config.set_ffmpeg_path(path);
                self.tool_paths.ffmpeg = self.config.ffmpeg_path.clone();
                before != self.config.ffmpeg_path
            }
            DependencyTool::Aria2c => {
                let before = self.config.aria2c_path.clone();
                self.config.set_aria2c_path(path);
                self.tool_paths.aria2c = self.config.aria2c_path.clone();
                before != self.config.aria2c_path
            }
            DependencyTool::Deno => {
                let before = self.config.deno_path.clone();
                self.config.set_deno_path(path);
                self.tool_paths.deno = self.config.deno_path.clone();
                before != self.config.deno_path
            }
        }
    }

    pub fn dependency_tool_status_text(&self, tool: DependencyTool) -> String {
        if let Some(status) = self.dependency_tool_update_status_text(tool) {
            return status;
        }
        if self.dependency_tool_is_installed(tool) {
            "Found".to_owned()
        } else {
            "Not found".to_owned()
        }
    }
}
