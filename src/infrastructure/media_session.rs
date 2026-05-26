#[derive(Clone, Debug)]
pub struct MediaSessionTrack {
    pub key: String,
    pub title: String,
    pub artist: String,
    pub thumbnail_url: String,
    pub duration_seconds: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaSessionPlaybackStatus {
    Closed,
    Stopped,
    Changing,
    Playing,
    Paused,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MediaSessionTimeline {
    pub position_seconds: f64,
    pub duration_seconds: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaSessionCommand {
    Play,
    Pause,
    Previous,
    Next,
    Stop,
}

pub struct MediaSession {
    inner: imp::MediaSession,
}

impl MediaSession {
    pub fn new() -> Self {
        Self {
            inner: imp::MediaSession::new(),
        }
    }

    pub fn poll_command(&mut self) -> Option<MediaSessionCommand> {
        self.inner.poll_command()
    }

    pub fn update(
        &mut self,
        track: &MediaSessionTrack,
        status: MediaSessionPlaybackStatus,
        timeline: MediaSessionTimeline,
    ) {
        self.inner.update(track, status, timeline);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl Default for MediaSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use std::sync::mpsc::{Receiver, Sender};
    use std::time::{Duration, Instant};

    use super::{
        MediaSessionCommand, MediaSessionPlaybackStatus, MediaSessionTimeline, MediaSessionTrack,
    };
    use crate::infrastructure::app_identity;
    use windows::Foundation::{TimeSpan, TypedEventHandler, Uri};
    use windows::Media::{
        MediaPlaybackStatus as WinMediaPlaybackStatus, MediaPlaybackType,
        SystemMediaTransportControls, SystemMediaTransportControlsButton,
        SystemMediaTransportControlsButtonPressedEventArgs,
        SystemMediaTransportControlsTimelineProperties,
    };
    use windows::Storage::Streams::RandomAccessStreamReference;
    use windows::Win32::Foundation::{HWND as WindowsHwnd, PROPERTYKEY};
    use windows::Win32::Storage::EnhancedStorage::{
        PKEY_AppUserModel_ID, PKEY_AppUserModel_RelaunchCommand,
        PKEY_AppUserModel_RelaunchDisplayNameResource, PKEY_AppUserModel_RelaunchIconResource,
    };
    use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
    use windows::Win32::System::WinRT::{
        ISystemMediaTransportControlsInterop, RO_INIT_MULTITHREADED, RoGetActivationFactory,
        RoInitialize,
    };
    use windows::Win32::UI::Shell::PropertiesSystem::{
        IPropertyStore, SHGetPropertyStoreForWindow,
    };
    use windows::core::{HSTRING, Result as WinResult};
    use windows_sys::Win32::Foundation::{HWND as SysHwnd, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GW_OWNER, GetWindow, GetWindowThreadProcessId, IsWindowVisible,
    };
    use windows_sys::core::BOOL;

    pub struct MediaSession {
        session: Option<WindowsMediaSession>,
        rx: Receiver<MediaSessionCommand>,
        tx: Sender<MediaSessionCommand>,
        disabled: bool,
        last_error: Option<String>,
    }

    impl MediaSession {
        pub fn new() -> Self {
            let (tx, rx) = std::sync::mpsc::channel();
            Self {
                session: None,
                rx,
                tx,
                disabled: false,
                last_error: None,
            }
        }

        pub fn poll_command(&mut self) -> Option<MediaSessionCommand> {
            self.rx.try_recv().ok()
        }

        pub fn update(
            &mut self,
            track: &MediaSessionTrack,
            status: MediaSessionPlaybackStatus,
            timeline: MediaSessionTimeline,
        ) {
            if self.disabled {
                return;
            }
            if self.session.is_none() {
                match WindowsMediaSession::new(self.tx.clone()) {
                    Ok(session) => self.session = Some(session),
                    Err(error) => {
                        let message = format!("Windows media session unavailable: {error}");
                        if self.last_error.as_deref() != Some(message.as_str()) {
                            eprintln!("[media-session] {message}");
                            self.last_error = Some(message);
                        }
                        self.disabled = true;
                        return;
                    }
                }
            }
            if let Some(session) = &mut self.session {
                if let Err(error) = session.update(track, status, timeline) {
                    let message = format!("Windows media session update failed: {error}");
                    if self.last_error.as_deref() != Some(message.as_str()) {
                        eprintln!("[media-session] {message}");
                        self.last_error = Some(message);
                    }
                }
            }
        }

        pub fn clear(&mut self) {
            if let Some(session) = &mut self.session {
                if let Err(error) = session.clear() {
                    let message = format!("Windows media session clear failed: {error}");
                    if self.last_error.as_deref() != Some(message.as_str()) {
                        eprintln!("[media-session] {message}");
                        self.last_error = Some(message);
                    }
                }
            }
        }
    }

    struct WindowsMediaSession {
        controls: SystemMediaTransportControls,
        button_token: i64,
        current_track_signature: Option<String>,
        current_status: Option<MediaSessionPlaybackStatus>,
        last_timeline_update: Option<Instant>,
    }

    impl WindowsMediaSession {
        fn new(tx: Sender<MediaSessionCommand>) -> WinResult<Self> {
            // It is fine if WinRT was already initialized elsewhere with a different mode.
            unsafe {
                let _ = RoInitialize(RO_INIT_MULTITHREADED);
            }

            if let Err(error) = app_identity::ensure_windows_app_identity() {
                eprintln!("[media-session] Windows app identity unavailable: {error}");
            }

            let hwnd = find_current_process_window()
                .map(to_windows_hwnd)
                .ok_or_else(|| {
                    windows::core::Error::from_hresult(windows::core::HRESULT(
                        0x80004005_u32 as i32,
                    ))
                })?;
            if let Err(error) = set_window_app_identity(hwnd) {
                eprintln!("[media-session] Windows window AppUserModelID unavailable: {error}");
            }
            let class_name = HSTRING::from("Windows.Media.SystemMediaTransportControls");
            let interop: ISystemMediaTransportControlsInterop =
                unsafe { RoGetActivationFactory(&class_name)? };
            let controls: SystemMediaTransportControls = unsafe { interop.GetForWindow(hwnd)? };
            controls.SetIsEnabled(true)?;
            controls.SetIsPlayEnabled(true)?;
            controls.SetIsPauseEnabled(true)?;
            controls.SetIsPreviousEnabled(true)?;
            controls.SetIsNextEnabled(true)?;
            controls.SetIsStopEnabled(true)?;

            let button_tx = tx;
            let handler = TypedEventHandler::<
                SystemMediaTransportControls,
                SystemMediaTransportControlsButtonPressedEventArgs,
            >::new(move |_sender, args| {
                if let Ok(args) = args.ok() {
                    if let Ok(button) = args.Button() {
                        if let Some(command) = media_button_to_command(button) {
                            let _ = button_tx.send(command);
                        }
                    }
                }
                Ok(())
            });
            let button_token = controls.ButtonPressed(&handler)?;

            Ok(Self {
                controls,
                button_token,
                current_track_signature: None,
                current_status: None,
                last_timeline_update: None,
            })
        }

        fn update(
            &mut self,
            track: &MediaSessionTrack,
            status: MediaSessionPlaybackStatus,
            timeline: MediaSessionTimeline,
        ) -> WinResult<()> {
            let signature = track_signature(track);
            if self.current_track_signature.as_deref() != Some(signature.as_str()) {
                self.update_track(track)?;
                self.current_track_signature = Some(signature);
                self.last_timeline_update = None;
            }
            if self.current_status != Some(status) {
                self.controls.SetPlaybackStatus(to_windows_status(status))?;
                self.current_status = Some(status);
                self.last_timeline_update = None;
            }
            if should_update_timeline(self.last_timeline_update, status) {
                self.update_timeline(timeline)?;
                self.last_timeline_update = Some(Instant::now());
            }
            Ok(())
        }

        fn update_track(&self, track: &MediaSessionTrack) -> WinResult<()> {
            let updater = self.controls.DisplayUpdater()?;
            updater.ClearAll()?;
            updater.SetType(MediaPlaybackType::Music)?;
            let music = updater.MusicProperties()?;
            music.SetTitle(&HSTRING::from(track.title.trim()))?;
            if !track.artist.trim().is_empty() {
                music.SetArtist(&HSTRING::from(track.artist.trim()))?;
            }
            if let Some(url) = clean_cover_url(&track.thumbnail_url) {
                let uri = Uri::CreateUri(&HSTRING::from(url))?;
                let stream = RandomAccessStreamReference::CreateFromUri(&uri)?;
                updater.SetThumbnail(&stream)?;
            }
            updater.Update()?;
            Ok(())
        }

        fn update_timeline(&self, timeline: MediaSessionTimeline) -> WinResult<()> {
            let properties = SystemMediaTransportControlsTimelineProperties::new()?;
            properties.SetStartTime(TimeSpan { Duration: 0 })?;
            properties.SetMinSeekTime(TimeSpan { Duration: 0 })?;
            properties.SetPosition(seconds_to_timespan(timeline.position_seconds))?;
            if let Some(duration) = timeline
                .duration_seconds
                .filter(|value| value.is_finite() && *value > 0.0)
            {
                let end = seconds_to_timespan(duration);
                properties.SetEndTime(end)?;
                properties.SetMaxSeekTime(end)?;
            }
            self.controls.UpdateTimelineProperties(&properties)?;
            Ok(())
        }

        fn clear(&mut self) -> WinResult<()> {
            if self.current_status == Some(MediaSessionPlaybackStatus::Closed)
                && self.current_track_signature.is_none()
            {
                return Ok(());
            }
            self.controls
                .SetPlaybackStatus(WinMediaPlaybackStatus::Closed)?;
            let updater = self.controls.DisplayUpdater()?;
            updater.ClearAll()?;
            updater.Update()?;
            self.current_track_signature = None;
            self.current_status = Some(MediaSessionPlaybackStatus::Closed);
            self.last_timeline_update = None;
            Ok(())
        }
    }

    impl Drop for WindowsMediaSession {
        fn drop(&mut self) {
            let _ = self.controls.RemoveButtonPressed(self.button_token);
            let _ = self.clear();
        }
    }

    fn set_window_app_identity(hwnd: WindowsHwnd) -> WinResult<()> {
        let store: IPropertyStore = unsafe { SHGetPropertyStoreForWindow(hwnd)? };

        if let Some(command) = app_identity::windows_relaunch_command() {
            set_property_string(&store, &PKEY_AppUserModel_RelaunchCommand, &command)?;
        }
        set_property_string(
            &store,
            &PKEY_AppUserModel_RelaunchDisplayNameResource,
            app_identity::APP_DISPLAY_NAME,
        )?;
        if let Some(icon) = app_identity::windows_icon_resource() {
            set_property_string(&store, &PKEY_AppUserModel_RelaunchIconResource, &icon)?;
        }
        set_property_string(&store, &PKEY_AppUserModel_ID, app_identity::APP_AUMID)?;

        unsafe { store.Commit()? };
        Ok(())
    }

    fn set_property_string(
        store: &IPropertyStore,
        key: *const PROPERTYKEY,
        value: &str,
    ) -> WinResult<()> {
        let prop = PROPVARIANT::from(value);
        unsafe { store.SetValue(key, &prop) }
    }

    fn track_signature(track: &MediaSessionTrack) -> String {
        format!(
            "{}
{}
{}
{}",
            track.key,
            track.title.trim(),
            track.artist.trim(),
            track.thumbnail_url.trim()
        )
    }

    fn media_button_to_command(
        button: SystemMediaTransportControlsButton,
    ) -> Option<MediaSessionCommand> {
        match button {
            SystemMediaTransportControlsButton::Play => Some(MediaSessionCommand::Play),
            SystemMediaTransportControlsButton::Pause => Some(MediaSessionCommand::Pause),
            SystemMediaTransportControlsButton::Previous => Some(MediaSessionCommand::Previous),
            SystemMediaTransportControlsButton::Next => Some(MediaSessionCommand::Next),
            SystemMediaTransportControlsButton::Stop => Some(MediaSessionCommand::Stop),
            _ => None,
        }
    }

    fn to_windows_status(status: MediaSessionPlaybackStatus) -> WinMediaPlaybackStatus {
        match status {
            MediaSessionPlaybackStatus::Closed => WinMediaPlaybackStatus::Closed,
            MediaSessionPlaybackStatus::Stopped => WinMediaPlaybackStatus::Stopped,
            MediaSessionPlaybackStatus::Changing => WinMediaPlaybackStatus::Changing,
            MediaSessionPlaybackStatus::Playing => WinMediaPlaybackStatus::Playing,
            MediaSessionPlaybackStatus::Paused => WinMediaPlaybackStatus::Paused,
        }
    }

    fn should_update_timeline(
        last_update: Option<Instant>,
        status: MediaSessionPlaybackStatus,
    ) -> bool {
        match status {
            MediaSessionPlaybackStatus::Playing => last_update
                .map(|last| last.elapsed() >= Duration::from_secs(5))
                .unwrap_or(true),
            _ => true,
        }
    }

    fn seconds_to_timespan(seconds: f64) -> TimeSpan {
        let ticks = (seconds.max(0.0) * 10_000_000.0).round();
        TimeSpan {
            Duration: ticks.clamp(0.0, i64::MAX as f64) as i64,
        }
    }

    fn clean_cover_url(value: &str) -> Option<&str> {
        let trimmed = value.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            Some(trimmed)
        } else {
            None
        }
    }

    fn to_windows_hwnd(hwnd: SysHwnd) -> WindowsHwnd {
        WindowsHwnd(hwnd as _)
    }

    struct WindowSearch {
        process_id: u32,
        hwnd: SysHwnd,
    }

    fn find_current_process_window() -> Option<SysHwnd> {
        let mut search = WindowSearch {
            process_id: std::process::id(),
            hwnd: 0 as SysHwnd,
        };
        unsafe {
            EnumWindows(
                Some(enum_window_proc),
                &mut search as *mut WindowSearch as LPARAM,
            );
        }
        if is_null_hwnd(search.hwnd) {
            None
        } else {
            Some(search.hwnd)
        }
    }

    unsafe extern "system" fn enum_window_proc(hwnd: SysHwnd, lparam: LPARAM) -> BOOL {
        let search = unsafe { &mut *(lparam as *mut WindowSearch) };
        let mut process_id = 0_u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut process_id);
        }
        if process_id != search.process_id || unsafe { IsWindowVisible(hwnd) } == 0 {
            return 1;
        }
        let owner = unsafe { GetWindow(hwnd, GW_OWNER) };
        if !is_null_hwnd(owner) {
            return 1;
        }
        search.hwnd = hwnd;
        0
    }

    fn is_null_hwnd(hwnd: SysHwnd) -> bool {
        (hwnd as isize) == 0
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::{
        MediaSessionCommand, MediaSessionPlaybackStatus, MediaSessionTimeline, MediaSessionTrack,
    };

    pub struct MediaSession;

    impl MediaSession {
        pub fn new() -> Self {
            Self
        }

        pub fn poll_command(&mut self) -> Option<MediaSessionCommand> {
            None
        }

        pub fn update(
            &mut self,
            _track: &MediaSessionTrack,
            _status: MediaSessionPlaybackStatus,
            _timeline: MediaSessionTimeline,
        ) {
        }

        pub fn clear(&mut self) {}
    }
}
