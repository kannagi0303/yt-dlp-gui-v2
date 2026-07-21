use std::sync::mpsc::Receiver;

use crate::infrastructure::{
    YoutubeLoginRescueBrowserInfo, YoutubeLoginRescueCookieExport, YoutubeLoginRescueEvent,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YoutubeLoginRescuePhase {
    Idle,
    Confirm,
    NoSupportedBrowser,
    Starting,
    WaitingForCdp,
    WaitingForCookie,
    CookieExported,
    Failed,
    Closed,
}

impl YoutubeLoginRescuePhase {
    pub fn is_blocking_prompt(self) -> bool {
        !matches!(self, Self::Idle | Self::Closed)
    }
}

pub(super) struct CookieRescueState {
    pub(super) phase: YoutubeLoginRescuePhase,
    pub(super) browser: Option<YoutubeLoginRescueBrowserInfo>,
    pub(super) site_name: Option<String>,
    pub(super) target_url: String,
    pub(super) target_error: Option<String>,
    clipboard_prefilled: bool,
    pub(super) error: Option<String>,
    rx: Option<Receiver<YoutubeLoginRescueEvent>>,
}

impl Default for CookieRescueState {
    fn default() -> Self {
        Self {
            phase: YoutubeLoginRescuePhase::Idle,
            browser: None,
            site_name: None,
            target_url: String::new(),
            target_error: None,
            clipboard_prefilled: false,
            error: None,
            rx: None,
        }
    }
}

impl CookieRescueState {
    pub(super) fn dialog_visible(&self) -> bool {
        self.phase.is_blocking_prompt()
    }

    pub(super) fn is_running(&self) -> bool {
        self.rx.is_some()
    }

    pub(super) fn reset_prompt_state(&mut self) {
        self.rx = None;
        self.error = None;
        self.target_error = None;
        self.browser = None;
        self.site_name = None;
        self.clipboard_prefilled = false;
    }

    pub(super) fn reset_prompt_for_current_target(&mut self) {
        self.reset_prompt_state();
    }

    pub(super) fn reset_prompt_for_target(&mut self, target_url: String) {
        self.reset_prompt_state();
        self.target_url = target_url;
    }

    pub(super) fn mark_browser_detected(&mut self, browser: YoutubeLoginRescueBrowserInfo) {
        self.browser = Some(browser);
        self.error = None;
        self.phase = YoutubeLoginRescuePhase::Confirm;
    }

    pub(super) fn mark_no_supported_browser(&mut self) {
        self.phase = YoutubeLoginRescuePhase::NoSupportedBrowser;
    }

    pub(super) fn mark_failed(&mut self, error: String) {
        self.error = Some(error);
        self.phase = YoutubeLoginRescuePhase::Failed;
    }

    pub(super) fn mark_prefilled_target_url(
        &mut self,
        target_url: String,
        clipboard_prefilled: bool,
    ) {
        self.target_url = target_url;
        self.clipboard_prefilled = clipboard_prefilled;
    }

    pub(super) fn mark_accepted_target_url(
        &mut self,
        target_url: String,
        clipboard_prefilled: bool,
    ) {
        self.target_url = target_url;
        self.target_error = None;
        self.clipboard_prefilled = clipboard_prefilled;
    }

    pub(super) fn mark_validated_target_url(&mut self, target_url: String) {
        self.target_url = target_url;
        self.target_error = None;
    }

    pub(super) fn mark_target_error(&mut self, error: String) {
        self.target_error = Some(error);
    }

    pub(super) fn set_manual_target_url(&mut self, target_url: String) {
        self.target_url = target_url;
        self.target_error = None;
        self.clipboard_prefilled = false;
    }

    pub(super) fn start_worker(&mut self, rx: Receiver<YoutubeLoginRescueEvent>) {
        self.rx = Some(rx);
        self.error = None;
        self.phase = YoutubeLoginRescuePhase::Starting;
    }

    pub(super) fn cancel_prompt(&mut self) {
        if self.is_running() {
            return;
        }
        self.phase = YoutubeLoginRescuePhase::Idle;
        self.browser = None;
        self.site_name = None;
        self.target_error = None;
        self.clipboard_prefilled = false;
        self.error = None;
    }

    pub(super) fn mark_closed(&mut self) {
        self.phase = YoutubeLoginRescuePhase::Closed;
        self.site_name = None;
        self.target_error = None;
        self.clipboard_prefilled = false;
        self.error = None;
        self.rx = None;
    }

    pub(super) fn take_event_receiver(&mut self) -> Option<Receiver<YoutubeLoginRescueEvent>> {
        self.rx.take()
    }

    pub(super) fn keep_event_receiver(&mut self, rx: Receiver<YoutubeLoginRescueEvent>) {
        self.rx = Some(rx);
    }

    pub(super) fn mark_cdp_ready(&mut self, browser: YoutubeLoginRescueBrowserInfo) {
        self.browser = Some(browser);
        self.error = None;
        self.phase = YoutubeLoginRescuePhase::WaitingForCookie;
    }

    pub(super) fn mark_cookie_exported(&mut self, export: &YoutubeLoginRescueCookieExport) {
        self.browser = Some(export.browser.clone());
        self.site_name = Some(export.site_display_name.clone());
        self.error = None;
        self.phase = YoutubeLoginRescuePhase::CookieExported;
    }

    pub(super) fn mark_waiting_for_cdp_if_starting(&mut self) {
        if matches!(self.phase, YoutubeLoginRescuePhase::Starting) {
            self.phase = YoutubeLoginRescuePhase::WaitingForCdp;
        }
    }

    pub(super) fn mark_worker_disconnected(&mut self) {
        self.error = Some("Cookie Rescue worker stopped before returning a result.".to_owned());
        self.phase = YoutubeLoginRescuePhase::Failed;
    }
}
