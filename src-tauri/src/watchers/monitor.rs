use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

#[derive(Debug, Clone, Copy)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub work_x: i32,
    pub work_y: i32,
    pub work_width: u32,
    pub work_height: u32,
}

pub struct MonitorWatcher;

impl MonitorWatcher {
    /// Gets the monitor geometry for the currently active foreground window
    pub fn get_active_monitor_bounds() -> MonitorBounds {
        unsafe {
            let hwnd = GetForegroundWindow();
            let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };

            if GetMonitorInfoW(hmon, &mut mi).as_bool() {
                let mon = mi.rcMonitor;
                let work = mi.rcWork;
                MonitorBounds {
                    x: mon.left,
                    y: mon.top,
                    width: (mon.right - mon.left) as u32,
                    height: (mon.bottom - mon.top) as u32,
                    work_x: work.left,
                    work_y: work.top,
                    work_width: (work.right - work.left) as u32,
                    work_height: (work.bottom - work.top) as u32,
                }
            } else {
                // Default fallback
                MonitorBounds {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                    work_x: 0,
                    work_y: 0,
                    work_width: 1920,
                    work_height: 1040,
                }
            }
        }
    }
}
