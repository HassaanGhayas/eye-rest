use crate::config::Settings;
use crate::state::{StateMachine, TimerPhase, TimerState};
use crate::watchers::fullscreen::FullscreenWatcher;
use crate::watchers::idle::IdleWatcher;
use crate::watchers::monitor::MonitorWatcher;
use crate::watchers::power::PowerWatcher;
use crate::win_utils::hotkey::HotkeyManager;
use crate::win_utils::memory::MemoryOptimizer;
use crate::win_utils::overlay::WindowStyleHelper;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::HWND;

pub struct AppScheduler {
    pub state: Arc<Mutex<StateMachine>>,
    pub settings: Arc<Mutex<Settings>>,
    pub power: Arc<PowerWatcher>,
    pub idle: IdleWatcher,
}

impl AppScheduler {
    pub fn new(settings: Settings) -> Self {
        let sm = StateMachine::new(settings.work_interval_mins, settings.break_duration_secs);
        Self {
            state: Arc::new(Mutex::new(sm)),
            settings: Arc::new(Mutex::new(settings)),
            power: Arc::new(PowerWatcher::new()),
            idle: IdleWatcher::new(120), // 2 minutes idle threshold
        }
    }

    pub fn start_loop(&self, app: AppHandle) {
        let state_arc = Arc::clone(&self.state);
        let settings_arc = Arc::clone(&self.settings);
        let power_arc = Arc::clone(&self.power);
        let idle_watcher = IdleWatcher::new(120);

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(1));

                // 1. Power / lock check
                if power_arc.is_paused() {
                    continue;
                }

                // 2. Idle check (2+ min of inactivity resets working timer)
                if idle_watcher.is_idle() {
                    let mut sm = state_arc.lock().unwrap();
                    if matches!(sm.phase(), TimerPhase::Working | TimerPhase::PreBreak) {
                        sm.reset_working();
                        // Hide pre-break toast if it was showing
                        if let Some(toast_win) = app.get_webview_window("toast") {
                            let _ = toast_win.hide();
                            let _ = toast_win.emit("prebreak-end", ());
                        }
                    }
                    continue;
                }

                // 3. Process phase transitions
                let current_phase;
                let remaining;
                {
                    let sm = state_arc.lock().unwrap();
                    current_phase = sm.phase();
                    remaining = sm.remaining_seconds();
                }

                match current_phase {
                    TimerPhase::Working => {
                        if remaining <= 10 {
                            // Check fullscreen deferral
                            if FullscreenWatcher::is_fullscreen_active() {
                                // Defer break silently while in fullscreen
                                continue;
                            }

                            // Transition to PreBreak (10s toast warning)
                            {
                                let mut sm = state_arc.lock().unwrap();
                                sm.current = TimerState::PreBreak {
                                    started_at: std::time::Instant::now(),
                                    duration: Duration::from_secs(10),
                                };
                            }

                            // Show gentle prebreak toast on active monitor
                            if let Some(toast_win) = app.get_webview_window("toast") {
                                let bounds = MonitorWatcher::get_active_monitor_bounds();
                                let toast_w = 340;
                                let toast_h = 72;
                                let toast_x = bounds.work_x + (bounds.work_width as i32) - toast_w - 20;
                                let toast_y = bounds.work_y + (bounds.work_height as i32) - toast_h - 20;

                                let _ = toast_win.set_position(tauri::Position::Physical(
                                    tauri::PhysicalPosition::new(toast_x, toast_y),
                                ));
                                let _ = toast_win.set_size(tauri::Size::Physical(
                                    tauri::PhysicalSize::new(toast_w as u32, toast_h as u32),
                                ));

                                let current_theme = {
                                    let s = settings_arc.lock().unwrap();
                                    s.theme.clone()
                                };

                                let _ = toast_win.emit(
                                    "prebreak-start",
                                    serde_json::json!({
                                        "seconds": 10,
                                        "theme": current_theme
                                    }),
                                );
                                let _ = toast_win.show();
                            }
                        }
                    }

                    TimerPhase::PreBreak => {
                        if let Some(toast_win) = app.get_webview_window("toast") {
                            let _ = toast_win.emit(
                                "prebreak-tick",
                                serde_json::json!({ "remaining": remaining }),
                            );
                        }

                        if remaining == 0 {
                            // Hide pre-break toast
                            if let Some(toast_win) = app.get_webview_window("toast") {
                                let _ = toast_win.hide();
                                let _ = toast_win.emit("prebreak-end", ());
                            }

                            // Transition to Break
                            let break_duration;
                            {
                                let mut sm = state_arc.lock().unwrap();
                                break_duration = sm.break_duration;
                                sm.current = TimerState::Break {
                                    started_at: std::time::Instant::now(),
                                    duration: break_duration,
                                };
                            }

                            // Show overlay on active monitor without stealing focus
                            if let Some(overlay_win) = app.get_webview_window("overlay") {
                                let bounds = MonitorWatcher::get_active_monitor_bounds();
                                if let Ok(hwnd_ptr) = overlay_win.hwnd() {
                                    let hwnd = HWND(hwnd_ptr.0);
                                    WindowStyleHelper::apply_no_activate(hwnd);
                                    WindowStyleHelper::position_no_activate(
                                        hwnd,
                                        bounds.x,
                                        bounds.y,
                                        bounds.width as i32,
                                        bounds.height as i32,
                                    );

                                    let (is_strict, sound, theme) = {
                                        let s = settings_arc.lock().unwrap();
                                        (s.strict_mode, s.sound_enabled, s.theme.clone())
                                    };

                                    // Register ephemeral Esc hotkey listener if not strict
                                    if !is_strict {
                                        HotkeyManager::start_escape_listener(app.clone(), Arc::clone(&state_arc));
                                    }

                                    let _ = overlay_win.show();
                                    let _ = overlay_win.emit(
                                        "break-start",
                                        serde_json::json!({
                                            "seconds": break_duration.as_secs(),
                                            "strict": is_strict,
                                            "sound": sound,
                                            "theme": theme
                                        }),
                                    );
                                }
                            }
                        }
                    }

                    TimerPhase::Break => {
                        if let Some(overlay_win) = app.get_webview_window("overlay") {
                            let _ = overlay_win.emit(
                                "break-tick",
                                serde_json::json!({ "remaining": remaining }),
                            );
                        }

                        if remaining == 0 {
                            // Break completed naturally
                            Self::dismiss_break_internal(&app, &state_arc);
                        }
                    }

                    TimerPhase::Paused => {
                        // Keep idle while paused
                    }

                    TimerPhase::Snoozed => {
                        if remaining == 0 {
                            let mut sm = state_arc.lock().unwrap();
                            sm.reset_working();
                        }
                    }
                }
            }
        });
    }

    pub fn dismiss_break_internal(app: &AppHandle, state_arc: &Arc<Mutex<StateMachine>>) {
        HotkeyManager::stop_escape_listener();

        if let Some(overlay_win) = app.get_webview_window("overlay") {
            let _ = overlay_win.emit("break-end", ());
            let _ = overlay_win.hide();
        }

        // Return to Working phase
        {
            let mut sm = state_arc.lock().unwrap();
            sm.reset_working();
        }

        // Trim application working set memory immediately to reclaim RAM
        MemoryOptimizer::trim_working_set();
    }
}
