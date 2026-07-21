use super::*;

impl AppState {
    pub fn youtube_login_rescue_dialog_visible(&self) -> bool {
        self.cookie_rescue.dialog_visible()
    }

    pub fn youtube_login_rescue_phase(&self) -> YoutubeLoginRescuePhase {
        self.cookie_rescue.phase
    }

    pub fn youtube_login_rescue_browser_display_name(&self) -> Option<&str> {
        self.cookie_rescue
            .browser
            .as_ref()
            .map(|browser| browser.display_name.as_str())
    }

    pub fn youtube_login_rescue_site_name(&self) -> Option<&str> {
        self.cookie_rescue.site_name.as_deref()
    }

    pub fn youtube_login_rescue_target_url(&self) -> &str {
        self.cookie_rescue.target_url.as_str()
    }

    pub fn youtube_login_rescue_target_error(&self) -> Option<&str> {
        self.cookie_rescue.target_error.as_deref()
    }

    pub fn youtube_login_rescue_error(&self) -> Option<&str> {
        self.cookie_rescue.error.as_deref()
    }

    pub fn open_youtube_login_rescue_prompt(&mut self) {
        self.cookie_rescue.reset_prompt_for_current_target();
        self.prefill_youtube_login_rescue_target_url();
        self.open_youtube_login_rescue_prompt_with_current_target();
    }

    pub fn open_youtube_login_rescue_prompt_for_url(&mut self, target_url: String) {
        self.cookie_rescue.reset_prompt_for_target(target_url);
        self.open_youtube_login_rescue_prompt_with_current_target();
    }

    fn open_youtube_login_rescue_prompt_with_current_target(&mut self) {
        match detect_default_youtube_login_rescue_browser() {
            Ok(Some(browser)) => {
                self.cookie_rescue.mark_browser_detected(browser);
            }
            Ok(None) => {
                self.cookie_rescue.mark_no_supported_browser();
            }
            Err(error) => {
                self.cookie_rescue.mark_failed(error);
            }
        }
    }

    fn prefill_youtube_login_rescue_target_url(&mut self) {
        if let Ok(url) = normalize_cookie_rescue_target_url(&self.url_input) {
            self.cookie_rescue.mark_prefilled_target_url(url, false);
            return;
        }

        if let Some(url) = read_clipboard_text()
            .and_then(|text| single_cookie_rescue_clipboard_url_candidate(&text))
            .and_then(|candidate| normalize_cookie_rescue_target_url(&candidate).ok())
        {
            self.cookie_rescue.mark_prefilled_target_url(url, true);
        }
    }

    pub fn paste_clipboard_to_youtube_login_rescue_target(&mut self) {
        match read_clipboard_text()
            .and_then(|text| single_cookie_rescue_clipboard_url_candidate(&text))
            .map(|candidate| normalize_cookie_rescue_target_url(&candidate))
        {
            Some(Ok(url)) => {
                self.cookie_rescue.mark_accepted_target_url(url, true);
            }
            Some(Err(error)) => {
                self.cookie_rescue.mark_target_error(error);
            }
            None => {
                self.cookie_rescue
                    .mark_target_error("Clipboard does not contain a website URL.".to_owned());
            }
        }
    }

    pub fn set_youtube_login_rescue_target_url(&mut self, value: String) {
        self.cookie_rescue.set_manual_target_url(value);
    }

    pub fn apply_youtube_login_rescue_dropped_paths(&mut self, paths: Vec<PathBuf>) {
        if let Some(url) = paths
            .iter()
            .find_map(|path| cookie_rescue_url_from_dropped_path(path))
            .and_then(|candidate| normalize_cookie_rescue_target_url(&candidate).ok())
        {
            self.cookie_rescue.mark_accepted_target_url(url, false);
        }
    }

    fn cookie_rescue_profile_root_path(&self) -> PathBuf {
        self.app_cache_root_path()
            .join("temp")
            .join("cookie-rescue")
    }

    pub fn start_youtube_login_rescue(&mut self) {
        if self.cookie_rescue.is_running() {
            return;
        }
        let Some(browser) = self.cookie_rescue.browser.clone() else {
            self.cookie_rescue.mark_no_supported_browser();
            return;
        };

        let target_url = match normalize_cookie_rescue_target_url(&self.cookie_rescue.target_url) {
            Ok(url) => url,
            Err(error) => {
                self.cookie_rescue.mark_target_error(error);
                return;
            }
        };
        self.cookie_rescue
            .mark_validated_target_url(target_url.clone());

        let cookie_dir_path = cookie_rescue_cookie_dir_path();
        let profile_root_path = self.cookie_rescue_profile_root_path();
        let (tx, rx) = mpsc::channel();
        self.cookie_rescue.start_worker(rx);
        self.last_action = format!(
            "Opening {} for Cookie Rescue: {}",
            browser.display_name, target_url
        );

        thread::spawn(move || {
            run_youtube_login_rescue_cookie_export(
                browser,
                target_url,
                cookie_dir_path,
                profile_root_path,
                tx,
            );
        });
    }

    pub fn cancel_youtube_login_rescue_prompt(&mut self) {
        self.cookie_rescue.cancel_prompt();
    }

    pub fn close_youtube_login_rescue_browser(&mut self) {
        self.cookie_rescue.mark_closed();
        self.last_action = "Cookie Rescue closed.".to_owned();
    }

    pub fn retry_youtube_login_rescue_detection(&mut self) {
        self.open_youtube_login_rescue_prompt();
    }

    pub fn youtube_login_rescue_is_starting(&self) -> bool {
        self.cookie_rescue.is_running()
    }

    pub(super) fn poll_youtube_login_rescue(&mut self) {
        let Some(rx) = self.cookie_rescue.take_event_receiver() else {
            return;
        };

        let mut keep_rx = true;
        loop {
            match rx.try_recv() {
                Ok(YoutubeLoginRescueEvent::CdpReady(browser)) => {
                    let browser_name = browser.display_name.clone();
                    self.cookie_rescue.mark_cdp_ready(browser);
                    self.last_action = format!(
                        "{browser_name} Cookie Rescue window is connected. Use the button in the login page after signing in."
                    );
                }
                Ok(YoutubeLoginRescueEvent::CookieExported(export)) => {
                    self.cookie_rescue.mark_cookie_exported(&export);
                    self.set_use_browser_cookies(true);
                    self.set_browser_cookie_source("auto");
                    self.last_action = format!(
                        "{} cookies saved: {} cookies.",
                        export.site_display_name, export.exported_cookie_count
                    );
                    keep_rx = false;
                    break;
                }
                Ok(YoutubeLoginRescueEvent::Failed(error)) => {
                    self.cookie_rescue.mark_failed(error.clone());
                    self.last_action = error;
                    keep_rx = false;
                    break;
                }
                Err(TryRecvError::Empty) => {
                    self.cookie_rescue.mark_waiting_for_cdp_if_starting();
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    self.cookie_rescue.mark_worker_disconnected();
                    keep_rx = false;
                    break;
                }
            }
        }

        if keep_rx {
            self.cookie_rescue.keep_event_receiver(rx);
        }
    }

    pub fn available_browser_cookie_sources(
        &self,
    ) -> Vec<crate::infrastructure::BrowserCookieSourceOption> {
        self.tool_paths.available_browser_cookie_sources()
    }

    pub fn cookie_usage_mode(&self) -> CookieUsageMode {
        if !self.item_defaults.use_cookies {
            CookieUsageMode::Off
        } else if self.cookie_source_uses_file() || self.cookie_source_uses_auto() {
            CookieUsageMode::File
        } else {
            CookieUsageMode::Browser
        }
    }

    pub fn set_cookie_usage_mode(&mut self, mode: CookieUsageMode) {
        match mode {
            CookieUsageMode::Off => {
                self.set_use_browser_cookies(false);
            }
            CookieUsageMode::Browser => {
                self.set_use_browser_cookies(true);
                if self.cookie_source_uses_file() || self.cookie_source_uses_auto() {
                    self.set_browser_cookie_source(self.default_browser_cookie_source_value());
                }
            }
            CookieUsageMode::File => {
                self.set_use_browser_cookies(true);
                if !(self.cookie_source_uses_file() || self.cookie_source_uses_auto()) {
                    self.set_browser_cookie_source("file");
                }
            }
        }
    }

    fn default_browser_cookie_source_value(&self) -> String {
        self.available_browser_cookie_sources()
            .into_iter()
            .find(|option| option.value != "auto" && option.value != "file")
            .map(|option| option.value.to_owned())
            .unwrap_or_else(|| "chrome".to_owned())
    }

    pub fn cookie_file_source_mode(&self) -> CookieFileSourceMode {
        if self.cookie_source_uses_auto() {
            CookieFileSourceMode::AutoSelect
        } else {
            CookieFileSourceMode::Custom
        }
    }

    pub fn set_cookie_file_source_mode(&mut self, mode: CookieFileSourceMode) {
        self.set_use_browser_cookies(true);
        match mode {
            CookieFileSourceMode::Custom => self.set_browser_cookie_source("file"),
            CookieFileSourceMode::AutoSelect => self.set_browser_cookie_source("auto"),
        }
    }

    pub fn available_browser_cookie_profiles(
        &self,
    ) -> Vec<crate::infrastructure::BrowserCookieProfileOption> {
        self.tool_paths.available_browser_cookie_profiles()
    }

    pub fn set_browser_cookie_source(&mut self, source: impl Into<String>) {
        let source = source.into();
        self.tool_paths.browser_cookie_source = source.clone();
        self.config.browser_cookie_source = source;
        let profiles = self.tool_paths.available_browser_cookie_profiles();
        if self.cookie_source_uses_file()
            || self.cookie_source_uses_auto()
            || (!self.tool_paths.browser_cookie_profile.trim().is_empty()
                && !profiles
                    .iter()
                    .any(|option| option.value == self.tool_paths.browser_cookie_profile))
        {
            self.tool_paths.browser_cookie_profile.clear();
            self.config.browser_cookie_profile.clear();
        }
        let _ = self.config.save();
    }

    pub fn set_browser_cookie_profile(&mut self, profile: impl Into<String>) {
        let profile = profile.into();
        self.tool_paths.browser_cookie_profile = profile.clone();
        self.config.browser_cookie_profile = profile;
        let _ = self.config.save();
    }

    pub fn set_browser_cookie_file(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.tool_paths.browser_cookie_file = path.clone();
        self.config.browser_cookie_file = path;
        let _ = self.config.save();
    }

    pub fn cookie_source_uses_auto(&self) -> bool {
        self.tool_paths
            .browser_cookie_source
            .trim()
            .eq_ignore_ascii_case("auto")
    }

    pub fn cookie_source_uses_file(&self) -> bool {
        self.tool_paths
            .browser_cookie_source
            .trim()
            .eq_ignore_ascii_case("file")
    }

    pub fn saved_cookie_files(&self) -> Vec<SavedCookieFile> {
        let cookie_dir = cookie_rescue_cookie_dir_path();
        let mut entries = read_cookie_site_index_or_default(&cookie_dir)
            .sites
            .into_iter()
            .filter(|entry| !entry.id.trim().is_empty())
            .map(saved_cookie_file_from_index_entry)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            left.display_name
                .to_ascii_lowercase()
                .cmp(&right.display_name.to_ascii_lowercase())
        });
        entries
    }

    pub fn refresh_saved_cookie_file(&mut self, id: &str) {
        let cookie_dir = cookie_rescue_cookie_dir_path();
        let index = read_cookie_site_index_or_default(&cookie_dir);
        let Some(entry) = index.sites.iter().find(|entry| entry.id == id) else {
            self.last_action = "Cookie file entry was not found.".to_owned();
            return;
        };
        let login_url = entry.login_url.trim();
        if login_url.is_empty() {
            self.last_action = "Cookie file entry has no saved login URL.".to_owned();
            return;
        }
        self.open_youtube_login_rescue_prompt_for_url(login_url.to_owned());
    }

    pub fn delete_saved_cookie_file(&mut self, id: &str) {
        let cookie_dir = cookie_rescue_cookie_dir_path();
        let mut index = read_cookie_site_index_or_default(&cookie_dir);
        let Some(position) = index.sites.iter().position(|entry| entry.id == id) else {
            self.last_action = "Cookie file entry was not found.".to_owned();
            return;
        };
        let entry = index.sites.remove(position);
        if let Some(path) = cookie_file_path_owned_by_cookie_dir(&cookie_dir, &entry.cookie_file) {
            if path.is_file() {
                if let Err(error) = fs::remove_file(&path) {
                    self.last_action =
                        format!("Could not delete Cookie file {}: {error}", path.display());
                    return;
                }
            }
        }
        match write_cookie_site_index(&cookie_dir, &index) {
            Ok(()) => {
                self.last_action = format!(
                    "Cookie file removed: {}",
                    saved_cookie_file_from_index_entry(entry).display_name
                );
            }
            Err(error) => {
                self.last_action = error;
            }
        }
    }
}

fn first_cookie_rescue_url_candidate(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|part| trim_cookie_rescue_url_wrappers(part))
        .find(|candidate| normalize_cookie_rescue_target_url(candidate).is_ok())
        .map(ToOwned::to_owned)
}

fn single_cookie_rescue_clipboard_url_candidate(text: &str) -> Option<String> {
    let trimmed = trim_cookie_rescue_url_wrappers(text.trim());
    if trimmed.is_empty() || trimmed.len() > 2048 || trimmed.chars().any(char::is_whitespace) {
        return None;
    }
    normalize_cookie_rescue_target_url(trimmed)
        .ok()
        .map(|_| trimmed.to_owned())
}

fn trim_cookie_rescue_url_wrappers(value: &str) -> &str {
    value.trim_matches(|ch: char| {
        matches!(
            ch,
            '<' | '>' | '"' | '\'' | '`' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
        )
    })
}

fn cookie_rescue_url_from_dropped_path(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    if extension == "url" {
        let data = fs::read_to_string(path).ok()?;
        for line in data.lines() {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("URL=") {
                return Some(value.trim().to_owned());
            }
        }
        return None;
    }

    if extension == "txt" {
        let data = fs::read_to_string(path).ok()?;
        return first_cookie_rescue_url_candidate(&data);
    }

    None
}

fn saved_cookie_file_from_index_entry(entry: CookieSiteIndexEntry) -> SavedCookieFile {
    let display_name = entry.display_name.trim();
    SavedCookieFile {
        id: entry.id,
        display_name: if display_name.is_empty() {
            entry
                .match_domains
                .first()
                .cloned()
                .unwrap_or_else(|| "Cookie".to_owned())
        } else {
            display_name.to_owned()
        },
        login_url: entry.login_url,
        updated_unix: entry.updated_unix,
    }
}

fn cookie_file_path_owned_by_cookie_dir(cookie_dir: &Path, cookie_file: &str) -> Option<PathBuf> {
    let cookie_file = cookie_file.trim();
    if cookie_file.is_empty() {
        return None;
    }

    let candidate = PathBuf::from(cookie_file);
    let path = if candidate.is_absolute() {
        candidate
    } else {
        cookie_dir.join(candidate)
    };
    let normalized_cookie_dir = normalized_path_for_safety(cookie_dir);
    let normalized_path = normalized_path_for_safety(&path);
    normalized_path
        .starts_with(&normalized_cookie_dir)
        .then_some(path)
}

fn cookie_rescue_cookie_dir_path() -> PathBuf {
    app_portable_root_path().join("data").join("cookies")
}
