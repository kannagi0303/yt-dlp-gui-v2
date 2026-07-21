use crate::infrastructure::ThemeAccentColor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeTitlebarAccentResult {
    Applied,
    NotReady,
    Unsupported,
}

#[cfg(windows)]
pub fn apply_titlebar_accent(
    accent: ThemeAccentColor,
    dark_mode: bool,
) -> NativeTitlebarAccentResult {
    windows::apply_titlebar_accent(accent, dark_mode)
}

#[cfg(not(windows))]
pub fn apply_titlebar_accent(
    _accent: ThemeAccentColor,
    _dark_mode: bool,
) -> NativeTitlebarAccentResult {
    NativeTitlebarAccentResult::Applied
}

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;

    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR, DWMWINDOWATTRIBUTE,
        DwmSetWindowAttribute,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible,
    };
    use windows_sys::core::BOOL;

    use super::NativeTitlebarAccentResult;
    use crate::infrastructure::ThemeAccentColor;

    pub fn apply_titlebar_accent(
        accent: ThemeAccentColor,
        dark_mode: bool,
    ) -> NativeTitlebarAccentResult {
        let Some(hwnd) = find_app_window() else {
            return NativeTitlebarAccentResult::NotReady;
        };

        if matches!(accent, ThemeAccentColor::Off) {
            let caption_ok = reset_dwm_color(hwnd, DWMWA_CAPTION_COLOR);
            let _ = reset_dwm_color(hwnd, DWMWA_TEXT_COLOR);
            let _ = reset_dwm_color(hwnd, DWMWA_BORDER_COLOR);
            return if caption_ok {
                NativeTitlebarAccentResult::Applied
            } else {
                NativeTitlebarAccentResult::Unsupported
            };
        }

        let caption = titlebar_caption_color(accent, dark_mode);
        let border = titlebar_border_color(accent, dark_mode);
        let text = if dark_mode {
            colorref(245, 247, 250)
        } else {
            colorref(28, 32, 38)
        };

        let caption_ok = set_dwm_color(hwnd, DWMWA_CAPTION_COLOR, caption);
        let _ = set_dwm_color(hwnd, DWMWA_TEXT_COLOR, text);
        let _ = set_dwm_color(hwnd, DWMWA_BORDER_COLOR, border);

        if caption_ok {
            NativeTitlebarAccentResult::Applied
        } else {
            NativeTitlebarAccentResult::Unsupported
        }
    }

    fn titlebar_caption_color(accent: ThemeAccentColor, dark_mode: bool) -> u32 {
        let (r, g, b) = accent.rgb();
        let base = if dark_mode {
            (30, 32, 36)
        } else {
            (244, 246, 249)
        };
        let amount = if dark_mode { 0.10 } else { 0.30 };
        let (r, g, b) = mix_rgb(base, (r, g, b), amount);
        colorref(r, g, b)
    }

    fn titlebar_border_color(accent: ThemeAccentColor, dark_mode: bool) -> u32 {
        let (r, g, b) = accent.rgb();
        let base = if dark_mode {
            (44, 47, 54)
        } else {
            (218, 224, 232)
        };
        let amount = if dark_mode { 0.20 } else { 0.50 };
        let (r, g, b) = mix_rgb(base, (r, g, b), amount);
        colorref(r, g, b)
    }

    fn mix_rgb(base: (u8, u8, u8), accent: (u8, u8, u8), amount: f32) -> (u8, u8, u8) {
        let amount = amount.clamp(0.0, 1.0);
        let inv = 1.0 - amount;
        (
            ((base.0 as f32 * inv) + (accent.0 as f32 * amount)).round() as u8,
            ((base.1 as f32 * inv) + (accent.1 as f32 * amount)).round() as u8,
            ((base.2 as f32 * inv) + (accent.2 as f32 * amount)).round() as u8,
        )
    }

    fn colorref(r: u8, g: u8, b: u8) -> u32 {
        u32::from(r) | (u32::from(g) << 8) | (u32::from(b) << 16)
    }

    fn reset_dwm_color(hwnd: HWND, attribute: DWMWINDOWATTRIBUTE) -> bool {
        const DWMWA_COLOR_DEFAULT: u32 = 0xFFFF_FFFF;
        set_dwm_color(hwnd, attribute, DWMWA_COLOR_DEFAULT)
    }

    fn set_dwm_color(hwnd: HWND, attribute: DWMWINDOWATTRIBUTE, color: u32) -> bool {
        unsafe {
            let hr = DwmSetWindowAttribute(
                hwnd,
                attribute as u32,
                &color as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );
            hr >= 0
        }
    }

    struct WindowSearch {
        process_id: u32,
        title: String,
        hwnd: Option<HWND>,
    }

    fn find_app_window() -> Option<HWND> {
        let mut search = WindowSearch {
            process_id: std::process::id(),
            title: crate::infrastructure::runtime_window_title(),
            hwnd: None,
        };

        unsafe {
            EnumWindows(
                Some(enum_window_proc),
                &mut search as *mut WindowSearch as LPARAM,
            );
        }

        search.hwnd
    }

    unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let search = unsafe { &mut *(lparam as *mut WindowSearch) };
        if unsafe { IsWindowVisible(hwnd) } == 0 {
            return 1;
        }

        let mut process_id = 0_u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut process_id as *mut u32);
        }
        if process_id != search.process_id {
            return 1;
        }

        if unsafe { window_title(hwnd) }.as_deref() == Some(search.title.as_str()) {
            search.hwnd = Some(hwnd);
            return 0;
        }

        1
    }

    unsafe fn window_title(hwnd: HWND) -> Option<String> {
        let len = unsafe { GetWindowTextLengthW(hwnd) };
        if len <= 0 {
            return None;
        }

        let mut buf = vec![0_u16; len as usize + 1];
        let written = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        if written <= 0 {
            return None;
        }

        Some(String::from_utf16_lossy(&buf[..written as usize]))
    }
}
