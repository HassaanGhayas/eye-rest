use std::time::Duration;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

pub struct IdleWatcher {
    threshold: Duration,
}

impl IdleWatcher {
    pub fn new(threshold_secs: u64) -> Self {
        Self {
            threshold: Duration::from_secs(threshold_secs),
        }
    }

    /// Returns idle time in seconds, or None if query fails
    pub fn get_idle_time() -> Option<Duration> {
        unsafe {
            let mut lii = LASTINPUTINFO {
                cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };
            if GetLastInputInfo(&mut lii).as_bool() {
                let now = GetTickCount();
                let diff_ms = now.saturating_sub(lii.dwTime);
                Some(Duration::from_millis(diff_ms as u64))
            } else {
                None
            }
        }
    }

    /// Check if idle duration exceeds threshold (default 2 minutes)
    pub fn is_idle(&self) -> bool {
        if let Some(idle) = Self::get_idle_time() {
            idle >= self.threshold
        } else {
            false
        }
    }
}
