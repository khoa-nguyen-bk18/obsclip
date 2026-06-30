use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, AppHandle, Manager, WindowEvent,
};

use crate::clip::service::clip_from_config;
use crate::config::AppConfig;
use crate::platform;
use crate::tray_icons::TrayIcons;
use crate::AppState;

pub const TRAY_ID: &str = "main";

pub fn setup_tray(app: &App, icons: &TrayIcons) -> tauri::Result<()> {
    let handle = app.handle();
    let clip = MenuItem::with_id(handle, "clip", "Clip to daily note", true, None::<&str>)?;
    let settings = MenuItem::with_id(handle, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(handle, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(handle, &[&clip, &settings, &quit])?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icons.default.clone())
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

pub fn setup_settings_window(app: &App) -> tauri::Result<()> {
    if let Some(settings) = app.get_webview_window("settings") {
        let settings_window = settings.clone();
        settings.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = settings_window.hide();
            }
        });
    }

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

enum FlashKind {
    Success,
    Error,
}

fn flash_tray(app: &AppHandle, kind: FlashKind) {
    let icons = app.state::<AppState>().tray_icons.clone();
    let app_for_flash = app.clone();
    let icons_for_flash = icons.clone();
    let _ = app.run_on_main_thread(move || {
        apply_flash(&app_for_flash, &icons_for_flash, kind);

        let app_for_restore = app_for_flash.clone();
        let icons_for_restore = icons_for_flash.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1800));
            let app = app_for_restore.clone();
            let _ = app_for_restore.run_on_main_thread(move || {
                restore_tray(&app, &icons_for_restore);
            });
        });
    });
}

fn apply_flash(app: &AppHandle, icons: &TrayIcons, kind: FlashKind) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        eprintln!("Tray icon not found for flash feedback");
        return;
    };

    let icon = match kind {
        FlashKind::Success => &icons.success,
        FlashKind::Error => &icons.error,
    };

    if let Err(error) = tray.set_icon(Some(icon.clone())) {
        eprintln!("Failed to set tray flash icon: {error}");
    }
}

fn restore_tray(app: &AppHandle, icons: &TrayIcons) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };

    let _ = tray.set_icon(Some(icons.default.clone()));
}

pub fn flash_tray_success(app: &AppHandle) {
    flash_tray(app, FlashKind::Success);
}

pub fn flash_tray_error(app: &AppHandle) {
    flash_tray(app, FlashKind::Error);
}
