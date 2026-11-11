use crate::config::Settings;
use crate::scheduler::AppScheduler;
use crate::watchers;
use crate::win_utils;
use crate::AppState;

use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    let settings = state.scheduler.settings.lock().unwrap();
    settings.clone()
}

#[tauri::command]
pub fn save_settings(settings: Settings, state: State<'_, AppState>) -> Result<(), String> {
    settings.save()?;
    let mut current_settings = state.scheduler.settings.lock().unwrap();
    let mut sm = state.scheduler.state.lock().unwrap();
    sm.update_durations(settings.work_interval_mins, settings.break_duration_secs);
    *current_settings = settings;
    Ok(())
}

#[tauri::command]
pub fn skip_break(app: AppHandle, state: State<'_, AppState>) {
    AppScheduler::dismiss_break_internal(&app, &state.scheduler.state);
}

#[tauri::command]
pub fn take_break_now(app: AppHandle, state: State<'_, AppState>) {
    let break_duration;
    {
        let mut sm = state.scheduler.state.lock().unwrap();
        sm.trigger_break_now();
        break_duration = sm.break_duration;
    }

    if let Some(overlay_win) = app.get_webview_window("overlay") {
        let bounds = watchers::monitor::MonitorWatcher::get_active_monitor_bounds();
        if let Ok(hwnd_ptr) = overlay_win.hwnd() {
            let hwnd = windows::Win32::Foundation::HWND(hwnd_ptr.0);
            win_utils::overlay::WindowStyleHelper::apply_no_activate(hwnd);
            win_utils::overlay::WindowStyleHelper::position_no_activate(
                hwnd,
                bounds.x,
                bounds.y,
                bounds.width as i32,
                bounds.height as i32,
            );

            let (strict, sound, theme) = {
                let s = state.scheduler.settings.lock().unwrap();
                (s.strict_mode, s.sound_enabled, s.theme.clone())
            };

            if !strict {
                win_utils::hotkey::HotkeyManager::start_escape_listener(
                    app.clone(),
                    std::sync::Arc::clone(&state.scheduler.state),
                );
            }

            let _ = overlay_win.show();
            let _ = overlay_win.emit(
                "break-start",
                serde_json::json!({
                    "seconds": break_duration.as_secs(),
                    "strict": strict,
                    "sound": sound,
                    "theme": theme
                }),
            );
        }
    }
}

#[tauri::command]
pub fn snooze_break(minutes: u64, app: AppHandle, state: State<'_, AppState>) {
    AppScheduler::dismiss_break_internal(&app, &state.scheduler.state);
    let mut sm = state.scheduler.state.lock().unwrap();
    sm.snooze(Duration::from_secs(minutes * 60));
}

#[tauri::command]
pub fn close_settings_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.hide();
    }
}
