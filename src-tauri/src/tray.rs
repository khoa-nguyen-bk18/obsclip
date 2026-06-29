use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, AppHandle, Manager,
};

use crate::clip::service::clip_from_config;
use crate::config::AppConfig;
use crate::platform;

pub const TRAY_ID: &str = "main";
const DEFAULT_TOOLTIP: &str = "Obsclip";

pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let handle = app.handle();
    let clip = MenuItem::with_id(handle, "clip", "Clip to daily note", true, None::<&str>)?;
    let settings = MenuItem::with_id(handle, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(handle, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(handle, &[&clip, &settings, &quit])?;

    let icon = tauri::include_image!("icons/icon.png");
    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip(DEFAULT_TOOLTIP)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "clip" => handle_clip(app),
            "settings" => show_settings(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

pub fn handle_clip(app: &AppHandle) {
    let config = match AppConfig::load(&platform::obsclip_config_path()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            flash_tray_error(app);
            return;
        }
    };

    match clip_from_config(&config) {
        Ok(()) => flash_tray_success(app),
        Err(e) => {
            eprintln!("Clip failed: {e}");
            flash_tray_error(app);
        }
    }
}

fn show_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn flash_tray_tooltip(app: &AppHandle, message: &str) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let _ = tray.set_tooltip(Some(message));
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1500));
        let app_handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            if let Some(tray) = app_handle.tray_by_id(TRAY_ID) {
                let _ = tray.set_tooltip(Some(DEFAULT_TOOLTIP));
            }
        });
    });
}

pub fn flash_tray_success(app: &AppHandle) {
    flash_tray_tooltip(app, "✓ Clipped");
}

pub fn flash_tray_error(app: &AppHandle) {
    flash_tray_tooltip(app, "✗ Error");
}
