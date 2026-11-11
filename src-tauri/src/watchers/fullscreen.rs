use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUNS_PRESENTATION_MODE,
    QUNS_RUNNING_D3D_FULL_SCREEN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowRect, GetWindowLongW, GWL_STYLE, WS_POPUP,
};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};

pub struct FullscreenWatcher;

impl FullscreenWatcher {
    /// Detects if the user is currently running a fullscreen app, game, or presentation
    pub fn is_fullscreen_active() -> bool {
        unsafe {
            // Check shell notification state (Direct3D fullscreen, PowerPoint presentation)
            if let Ok(state) = SHQueryUserNotificationState() {
                if state == QUNS_RUNNING_D3D_FULL_SCREEN
                    || state == QUNS_PRESENTATION_MODE
                {
                    return true;
                }
            }

            // Fallback: Check if the foreground window matches its monitor dimensions
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return false;
            }

            // Check if window is borderless/popup
            let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            let is_popup = (style & WS_POPUP.0) != 0;

            let mut win_rect = windows::Win32::Foundation::RECT::default();
            if GetWindowRect(hwnd, &mut win_rect).is_ok() {
                let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                let mut mi = MONITORINFO {
                    cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                    ..Default::default()
                };
                if GetMonitorInfoW(hmonitor, &mut mi).as_bool() {
                    let mon_rect = mi.rcMonitor;
                    if win_rect.left <= mon_rect.left
                        && win_rect.top <= mon_rect.top
                        && win_rect.right >= mon_rect.right
                        && win_rect.bottom >= mon_rect.bottom
                        && is_popup
                    {
                        return true;
                    }
                }
            }
        }
        false
    }
}
