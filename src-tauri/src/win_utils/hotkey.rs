use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::AppHandle;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE};
use crate::state::StateMachine;

static ESC_WATCHER_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct HotkeyManager;

impl HotkeyManager {
    /// Starts a low-overhead background listener for the Escape key during an active break.
    /// When Escape is pressed, it immediately dismisses the break.
    pub fn start_escape_listener(
        app: AppHandle,
        state_arc: Arc<Mutex<StateMachine>>,
    ) {
        // Stop any previous listener first
        ESC_WATCHER_ACTIVE.store(false, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(20));

        ESC_WATCHER_ACTIVE.store(true, Ordering::SeqCst);
        std::thread::spawn(move || {
            // Wait for key release first if Escape was already held down
            while ESC_WATCHER_ACTIVE.load(Ordering::Relaxed) {
                let is_down = unsafe { (GetAsyncKeyState(VK_ESCAPE.0 as i32) as u16 & 0x8000) != 0 };
                if !is_down {
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }

            while ESC_WATCHER_ACTIVE.load(Ordering::Relaxed) {
                let is_down = unsafe { (GetAsyncKeyState(VK_ESCAPE.0 as i32) as u16 & 0x8000) != 0 };
                if is_down {
                    // Escape key pressed!
                    ESC_WATCHER_ACTIVE.store(false, Ordering::SeqCst);
                    crate::scheduler::AppScheduler::dismiss_break_internal(&app, &state_arc);
                    break;
                }
                std::thread::sleep(Duration::from_millis(40));
            }
        });
    }

    /// Stops the Escape key listener
    pub fn stop_escape_listener() {
        ESC_WATCHER_ACTIVE.store(false, Ordering::SeqCst);
    }
}
