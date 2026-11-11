use tauri::{
    image::Image,
    menu::{Menu, MenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle,
};

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let status_item = MenuItem::with_id(app, "status", "EyeRest: Ready", false, None::<&str>)?;
    let take_now_item = MenuItem::with_id(app, "take_now", "Take break now", true, None::<&str>)?;
    let pause_resume_item = MenuItem::with_id(app, "pause_resume", "Pause", true, None::<&str>)?;
    let skip_item = MenuItem::with_id(app, "skip", "Skip next break", true, None::<&str>)?;

    // Snooze submenu
    let snooze_5m = MenuItem::with_id(app, "snooze_5m", "5 minutes", true, None::<&str>)?;
    let snooze_1h = MenuItem::with_id(app, "snooze_1h", "1 hour", true, None::<&str>)?;
    let snooze_tomorrow = MenuItem::with_id(app, "snooze_tomorrow", "Until tomorrow (4:00 AM)", true, None::<&str>)?;
    let snooze_menu = Submenu::with_id_and_items(
        app,
        "snooze_menu",
        "Snooze",
        true,
        &[&snooze_5m, &snooze_1h, &snooze_tomorrow],
    )?;

    let settings_item = MenuItem::with_id(app, "settings", "Preferences...", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit EyeRest", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &status_item,
            &take_now_item,
            &pause_resume_item,
            &skip_item,
            &snooze_menu,
            &settings_item,
            &quit_item,
        ],
    )?;

    let icon = app
        .default_window_icon()
        .cloned()
        .or_else(|| Image::from_app_icon_resource(32).ok());

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("EyeRest - 20-20-20 Eye Break Reminders");

    if let Some(tray_icon) = icon {
        builder = builder.icon(tray_icon);
    }

    let _tray = builder.build(app)?;

    Ok(())
}
