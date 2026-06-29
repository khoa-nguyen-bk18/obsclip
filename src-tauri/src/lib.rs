pub mod clip;
pub mod clipboard;
pub mod config;
pub mod platform;
pub mod tray;
pub mod vault;

use std::sync::Mutex;

use config::AppConfig;
use platform::obsclip_config_path;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub struct AppState {
    pub config: Mutex<AppConfig>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load(&obsclip_config_path()).expect("failed to load config");

    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(config.clone()),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(move |app| {
            tray::setup_tray(app)?;

            let shortcut = config.shortcut.clone();
            let app_handle = app.handle().clone();
            app.handle()
                .global_shortcut()
                .on_shortcut(shortcut.as_str(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    tray::handle_clip(&app_handle);
                }
            })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
