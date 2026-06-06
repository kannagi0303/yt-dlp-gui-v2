#[cfg(target_os = "windows")]
pub(crate) use windows_app_identity::{APP_AUMID, APP_DISPLAY_NAME};

#[cfg(not(target_os = "windows"))]
pub(crate) const APP_AUMID: &str = "kannagi.ytdlpgui.v2";
#[cfg(not(target_os = "windows"))]
pub(crate) const APP_DISPLAY_NAME: &str = "yt-dlp-gui";
#[cfg(target_os = "windows")]
pub(crate) fn ensure_windows_app_identity() -> Result<(), String> {
    windows_app_identity::ensure_windows_app_identity()
}

#[cfg(target_os = "windows")]
pub(crate) fn set_windows_process_app_identity() -> Result<(), String> {
    windows_app_identity::set_windows_process_app_identity()
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn ensure_windows_app_identity() -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn set_windows_process_app_identity() -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
pub(crate) fn windows_relaunch_command() -> Option<String> {
    windows_app_identity::relaunch_command()
}

#[cfg(target_os = "windows")]
pub(crate) fn windows_icon_resource() -> Option<String> {
    windows_app_identity::icon_resource()
}

#[cfg(target_os = "windows")]
mod windows_app_identity {
    use std::path::PathBuf;
    use std::sync::OnceLock;

    use win32_notif::registration::RegistrationBuilder;
    use windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    pub(crate) const APP_AUMID: &str = "kannagi.ytdlpgui.v2";
    pub(crate) const APP_DISPLAY_NAME: &str = "yt-dlp-gui";

    static APP_IDENTITY_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

    pub(crate) fn ensure_windows_app_identity() -> Result<(), String> {
        APP_IDENTITY_RESULT
            .get_or_init(|| {
                set_process_app_user_model_id()?;
                ensure_registered()
            })
            .clone()
    }

    pub(crate) fn set_windows_process_app_identity() -> Result<(), String> {
        set_process_app_user_model_id()
    }

    fn set_process_app_user_model_id() -> Result<(), String> {
        let app_id = wide_null(APP_AUMID);
        let result = unsafe { SetCurrentProcessExplicitAppUserModelID(app_id.as_ptr()) };
        if result < 0 {
            Err(format!(
                "Could not set process AppUserModelID {APP_AUMID}: 0x{:08X}",
                result as u32
            ))
        } else {
            Ok(())
        }
    }

    fn ensure_registered() -> Result<(), String> {
        let icon_path = app_icon_path();
        let mut builder = RegistrationBuilder::new(APP_AUMID)
            .map_err(|error| format!("Could not create Windows app registration data: {error}"))?
            .with_display_name(APP_DISPLAY_NAME);

        if let Some(icon_path) = icon_path.as_deref() {
            builder = builder.with_icon_path(icon_path);
        }

        builder
            .register()
            .map_err(|error| format!("Could not register Windows app AUMID: {error}"))
    }

    fn app_icon_path() -> Option<String> {
        let candidates = [
            current_exe_assets_icon_path(),
            current_dir_assets_icon_path(),
            current_exe_path(),
        ];

        candidates
            .into_iter()
            .flatten()
            .find(|path| path.exists())
            .map(|path| path.display().to_string())
    }

    fn current_exe_assets_icon_path() -> Option<PathBuf> {
        std::env::current_exe().ok().and_then(|path| {
            path.parent()
                .map(|parent| parent.join("assets").join("logo.ico"))
        })
    }

    fn current_dir_assets_icon_path() -> Option<PathBuf> {
        std::env::current_dir()
            .ok()
            .map(|path| path.join("assets").join("logo.ico"))
    }

    pub(crate) fn relaunch_command() -> Option<String> {
        current_exe_path().map(|path| format!("\"{}\"", path.display()))
    }

    pub(crate) fn icon_resource() -> Option<String> {
        app_icon_path().map(|path| format!("{path},0"))
    }

    fn current_exe_path() -> Option<PathBuf> {
        std::env::current_exe().ok()
    }

    fn wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}
