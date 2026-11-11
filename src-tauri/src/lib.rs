pub mod commands;
pub mod config;
pub mod scheduler;
pub mod state;
pub mod tray;
pub mod watchers;
pub mod win_utils;

use chrono::Timelike;
use config::Settings;
use scheduler::AppScheduler;
use state::TimerPhase;
use tauri::{Emitter, Manager, State};

pub struct AppState {
    pub scheduler: AppScheduler,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = Settings::load();
    let scheduler = AppScheduler::new(settings);
    let app_state = AppState { scheduler };

    tauri::Builder::default()
        .manage(app_state)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "settings" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            let handle = app.handle();

            // Initialize tray
            let _ = tray::create_tray(&handle);

            // Handle tray menu events
            let app_handle_tray = handle.clone();
            handle.on_menu_event(move |app, event| {
                let state: State<'_, AppState> = app.state();
                match event.id().as_ref() {
                    "take_now" => {
                        commands::take_break_now(app_handle_tray.clone(), state);
                    }
                    "pause_resume" => {
                        let mut sm = state.scheduler.state.lock().unwrap();
                        if sm.phase() == TimerPhase::Paused {
                            sm.resume();
                        } else {
                            sm.pause();
                        }
                    }
                    "skip" => {
                        AppScheduler::dismiss_break_internal(app, &state.scheduler.state);
                    }
                    "snooze_5m" => {
                        commands::snooze_break(5, app_handle_tray.clone(), state);
                    }
                    "snooze_1h" => {
                        commands::snooze_break(60, app_handle_tray.clone(), state);
                    }
                    "snooze_tomorrow" => {
                        // Calculate seconds until next 4:00 AM
                        let now = chrono::Local::now();
                        let tomorrow_4am = if now.hour() < 4 {
                            now.date_naive().and_hms_opt(4, 0, 0).unwrap()
                        } else {
                            (now.date_naive() + chrono::Duration::days(1))
                                .and_hms_opt(4, 0, 0)
                                .unwrap()
                        };
                        let diff_secs = tomorrow_4am.signed_duration_since(now.naive_local()).num_seconds();
                        let snooze_secs = if diff_secs > 0 { diff_secs as u64 } else { 3600 * 8 };
                        commands::snooze_break(snooze_secs / 60, app_handle_tray.clone(), state);
                    }
                    "settings" => {
                        if let Some(settings_win) = app.get_webview_window("settings") {
                            let _ = settings_win.unminimize();
                            let _ = settings_win.show();
                            let _ = settings_win.set_focus();
                            let _ = settings_win.emit("refresh-settings", ());
                        } else {
                            if let Ok(settings_win) = tauri::WebviewWindowBuilder::new(
                                app,
                                "settings",
                                tauri::WebviewUrl::App("settings/index.html".into()),
                            )
                            .title("EyeRest Preferences")
                            .inner_size(480.0, 620.0)
                            .center()
                            .resizable(false)
                            .build()
                            {
                                let _ = settings_win.show();
                                let _ = settings_win.set_focus();
                            }
                        }
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            });

            // Start background timer and watcher scheduler
            let app_state: State<'_, AppState> = app.state();
            app_state.scheduler.start_loop(handle.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::skip_break,
            commands::take_break_now,
            commands::snooze_break,
            commands::close_settings_window,
        ])
        .build(tauri::generate_context!())
        .expect("error while building EyeRest application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
