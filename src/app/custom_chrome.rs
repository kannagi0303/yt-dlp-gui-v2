#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomChromeResult {
    Applied,
    NotReady,
    Unsupported,
}

#[cfg(windows)]
pub fn install() -> CustomChromeResult {
    windows::install()
}

#[cfg(not(windows))]
pub fn install() -> CustomChromeResult {
    CustomChromeResult::Applied
}

#[cfg(windows)]
pub(crate) fn set_titlebar_client_hit_test_for_popup(enabled: bool) {
    windows::set_titlebar_client_hit_test_for_popup(enabled);
}

#[cfg(not(windows))]
pub(crate) fn set_titlebar_client_hit_test_for_popup(_enabled: bool) {}

#[cfg(windows)]
pub(crate) fn set_titlebar_hit_test_metrics(
    titlebar_height_points: f32,
    right_client_area_width_points: f32,
    pixels_per_point: f32,
) {
    windows::set_titlebar_hit_test_metrics(
        titlebar_height_points,
        right_client_area_width_points,
        pixels_per_point,
    );
}

#[cfg(not(windows))]
pub(crate) fn set_titlebar_hit_test_metrics(
    _titlebar_height_points: f32,
    _right_client_area_width_points: f32,
    _pixels_per_point: f32,
) {
}

#[cfg(windows)]
mod windows {
    use std::{
        ffi::c_void,
        mem,
        sync::{
            Mutex, OnceLock,
            atomic::{AtomicBool, AtomicI32, Ordering},
        },
    };

    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute;
    use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, EnumWindows, GWLP_WNDPROC, GetSystemMetrics,
        GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTCAPTION, HTCLIENT,
        HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT, IsWindowVisible, IsZoomed,
        NCCALCSIZE_PARAMS, SM_CXFRAME, SM_CXPADDEDBORDER, SM_CYFRAME, SWP_FRAMECHANGED,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos,
        WM_NCCALCSIZE, WM_NCDESTROY, WM_NCHITTEST, WNDPROC,
    };
    use windows_sys::core::BOOL;

    use super::CustomChromeResult;

    const TITLEBAR_HEIGHT_LOGICAL: i32 = 26;
    const DEFAULT_RIGHT_CLIENT_AREA_WIDTH_LOGICAL: i32 = 160;
    const MIN_RESIZE_BORDER_PX: i32 = 6;

    #[derive(Default)]
    struct ChromeState {
        hwnd: isize,
        previous_wndproc: isize,
    }

    static CHROME_STATE: OnceLock<Mutex<ChromeState>> = OnceLock::new();
    static TITLEBAR_CLIENT_HIT_TEST_FOR_POPUP: AtomicBool = AtomicBool::new(false);
    static TITLEBAR_HIT_TEST_HEIGHT_PX: AtomicI32 = AtomicI32::new(TITLEBAR_HEIGHT_LOGICAL);
    static TITLEBAR_RIGHT_CLIENT_AREA_WIDTH_PX: AtomicI32 =
        AtomicI32::new(DEFAULT_RIGHT_CLIENT_AREA_WIDTH_LOGICAL);

    pub fn set_titlebar_client_hit_test_for_popup(enabled: bool) {
        TITLEBAR_CLIENT_HIT_TEST_FOR_POPUP.store(enabled, Ordering::Relaxed);
    }

    pub fn set_titlebar_hit_test_metrics(
        titlebar_height_points: f32,
        right_client_area_width_points: f32,
        pixels_per_point: f32,
    ) {
        let pixels_per_point = pixels_per_point.max(0.1);
        let titlebar_height = (titlebar_height_points * pixels_per_point).round().max(1.0) as i32;
        let min_right_client_width = (DEFAULT_RIGHT_CLIENT_AREA_WIDTH_LOGICAL as f32
            * pixels_per_point)
            .round()
            .max(1.0);
        let right_client_width = (right_client_area_width_points * pixels_per_point)
            .round()
            .max(min_right_client_width) as i32;

        TITLEBAR_HIT_TEST_HEIGHT_PX.store(titlebar_height, Ordering::Relaxed);
        TITLEBAR_RIGHT_CLIENT_AREA_WIDTH_PX.store(right_client_width, Ordering::Relaxed);
    }

    pub fn install() -> CustomChromeResult {
        let Some(hwnd) = find_app_window() else {
            return CustomChromeResult::NotReady;
        };

        if is_installed_for(hwnd) {
            apply_rounded_corners(hwnd);
            return CustomChromeResult::Applied;
        }

        if !subclass_window(hwnd) {
            return CustomChromeResult::Unsupported;
        }

        apply_rounded_corners(hwnd);
        refresh_non_client_frame(hwnd);
        CustomChromeResult::Applied
    }

    fn is_installed_for(hwnd: HWND) -> bool {
        let state = chrome_state().lock().ok();
        state
            .as_ref()
            .is_some_and(|state| state.hwnd == hwnd as isize && state.previous_wndproc != 0)
    }

    fn subclass_window(hwnd: HWND) -> bool {
        unsafe {
            let current = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
            if current == custom_window_proc_ptr() {
                return true;
            }

            let previous = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, custom_window_proc_ptr());
            if previous == 0 {
                return false;
            }

            if let Ok(mut state) = chrome_state().lock() {
                state.hwnd = hwnd as isize;
                state.previous_wndproc = previous;
                true
            } else {
                let _ = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, previous);
                false
            }
        }
    }

    fn chrome_state() -> &'static Mutex<ChromeState> {
        CHROME_STATE.get_or_init(|| Mutex::new(ChromeState::default()))
    }

    fn custom_window_proc_ptr() -> isize {
        custom_window_proc as *const () as usize as isize
    }

    fn refresh_non_client_frame(hwnd: HWND) {
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }
    }

    fn apply_rounded_corners(hwnd: HWND) {
        const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
        const DWMWCP_ROUND: u32 = 2;

        unsafe {
            let preference = DWMWCP_ROUND;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &preference as *const u32 as *const c_void,
                mem::size_of::<u32>() as u32,
            );
        }
    }

    unsafe extern "system" fn custom_window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_NCCALCSIZE => {
                if wparam != 0 {
                    adjust_maximized_client_rect(hwnd, lparam);
                }
                return 0;
            }
            WM_NCHITTEST => {
                let hit = hit_test(hwnd, lparam);
                if hit != HTCLIENT as i32 {
                    return hit as LRESULT;
                }
            }
            WM_NCDESTROY => {
                let result = call_previous_window_proc(hwnd, msg, wparam, lparam);
                uninstall_if_current(hwnd);
                return result;
            }
            _ => {}
        }

        call_previous_window_proc(hwnd, msg, wparam, lparam)
    }

    fn adjust_maximized_client_rect(hwnd: HWND, lparam: LPARAM) {
        unsafe {
            if IsZoomed(hwnd) == 0 || lparam == 0 {
                return;
            }

            let params = &mut *(lparam as *mut NCCALCSIZE_PARAMS);
            let frame_x = resize_border_x(hwnd);
            let frame_y = resize_border_y(hwnd);
            params.rgrc[0].left += frame_x;
            params.rgrc[0].top += frame_y;
            params.rgrc[0].right -= frame_x;
            params.rgrc[0].bottom -= frame_y;
        }
    }

    fn hit_test(hwnd: HWND, lparam: LPARAM) -> i32 {
        let Some(window_rect) = window_rect(hwnd) else {
            return HTCLIENT as i32;
        };

        let cursor_x = signed_low_word(lparam);
        let cursor_y = signed_high_word(lparam);
        let maximized = unsafe { IsZoomed(hwnd) != 0 };

        if !maximized {
            let border_x = resize_border_x(hwnd).max(MIN_RESIZE_BORDER_PX);
            let border_y = resize_border_y(hwnd).max(MIN_RESIZE_BORDER_PX);
            let on_left = cursor_x < window_rect.left + border_x;
            let on_right = cursor_x >= window_rect.right - border_x;
            let on_top = cursor_y < window_rect.top + border_y;
            let on_bottom = cursor_y >= window_rect.bottom - border_y;

            match (on_left, on_right, on_top, on_bottom) {
                (true, false, true, false) => return HTTOPLEFT as i32,
                (false, true, true, false) => return HTTOPRIGHT as i32,
                (true, false, false, true) => return HTBOTTOMLEFT as i32,
                (false, true, false, true) => return HTBOTTOMRIGHT as i32,
                (true, false, false, false) => return HTLEFT as i32,
                (false, true, false, false) => return HTRIGHT as i32,
                (false, false, true, false) => return HTTOP as i32,
                (false, false, false, true) => return HTBOTTOM as i32,
                _ => {}
            }
        }

        let titlebar_height = TITLEBAR_HIT_TEST_HEIGHT_PX.load(Ordering::Relaxed).max(1);
        let right_client_width = TITLEBAR_RIGHT_CLIENT_AREA_WIDTH_PX
            .load(Ordering::Relaxed)
            .max(1);
        let in_titlebar =
            cursor_y >= window_rect.top && cursor_y < window_rect.top + titlebar_height;
        if in_titlebar {
            let in_right_client_area = cursor_x >= window_rect.right - right_client_width;
            if in_right_client_area || TITLEBAR_CLIENT_HIT_TEST_FOR_POPUP.load(Ordering::Relaxed) {
                return HTCLIENT as i32;
            }
            return HTCAPTION as i32;
        }

        HTCLIENT as i32
    }

    fn resize_border_x(hwnd: HWND) -> i32 {
        unsafe {
            GetSystemMetrics(SM_CXFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER) + dpi_padding(hwnd)
        }
    }

    fn resize_border_y(hwnd: HWND) -> i32 {
        unsafe {
            GetSystemMetrics(SM_CYFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER) + dpi_padding(hwnd)
        }
    }

    fn dpi_padding(hwnd: HWND) -> i32 {
        let dpi = unsafe { GetDpiForWindow(hwnd) as i32 };
        if dpi <= 96 {
            0
        } else {
            ((dpi - 96) / 96).max(0)
        }
    }

    fn window_rect(hwnd: HWND) -> Option<RECT> {
        unsafe {
            let mut rect = RECT::default();
            (GetWindowRect(hwnd, &mut rect) != 0).then_some(rect)
        }
    }

    fn signed_low_word(value: LPARAM) -> i32 {
        (value as u32 & 0xffff) as i16 as i32
    }

    fn signed_high_word(value: LPARAM) -> i32 {
        ((value as u32 >> 16) & 0xffff) as i16 as i32
    }

    fn call_previous_window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if let Some(previous) = previous_window_proc(hwnd) {
            unsafe { CallWindowProcW(Some(previous), hwnd, msg, wparam, lparam) }
        } else {
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }

    fn previous_window_proc(hwnd: HWND) -> WNDPROC {
        let state = chrome_state().lock().ok()?;
        if state.hwnd != hwnd as isize || state.previous_wndproc == 0 {
            return None;
        }
        unsafe { mem::transmute::<isize, WNDPROC>(state.previous_wndproc) }
    }

    fn uninstall_if_current(hwnd: HWND) {
        let Ok(mut state) = chrome_state().lock() else {
            return;
        };
        if state.hwnd != hwnd as isize || state.previous_wndproc == 0 {
            return;
        }

        unsafe {
            let current = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
            if current == custom_window_proc_ptr() {
                let _ = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, state.previous_wndproc);
            }
        }

        state.hwnd = 0;
        state.previous_wndproc = 0;
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
