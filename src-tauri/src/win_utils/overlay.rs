use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
};

pub struct WindowStyleHelper;

impl WindowStyleHelper {
    /// Applies WS_EX_NOACTIVATE and WS_EX_TOOLWINDOW to prevent keyboard focus stealing
    pub fn apply_no_activate(hwnd: HWND) {
        unsafe {
            let cur_ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_ex = cur_ex | (WS_EX_NOACTIVATE.0 as i32) | (WS_EX_TOOLWINDOW.0 as i32) | (WS_EX_TOPMOST.0 as i32);
            let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);

            // Re-apply position with HWND_TOPMOST and SWP_NOACTIVATE
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }

    /// Repositions and shows window on specified coordinates without activating or stealing focus
    pub fn position_no_activate(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                x,
                y,
                width,
                height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }
}
