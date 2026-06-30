pub mod clip;
pub mod clipboard;
pub mod config;
pub mod platform;
pub mod tray;
pub mod tray_icons;
pub mod vault;

use std::sync::Mutex;

use config::AppConfig;
use platform::obsclip_config_path;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tray_icons::TrayIcons;

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub tray_icons: TrayIcons,
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn save_config(
    app: AppHandle,
    state: tauri::State<AppState>,
    config: AppConfig,
) -> Result<(), String> {
    let old_shortcut = state.config.lock().unwrap().shortcut.clone();
    config
        .save(&obsclip_config_path())
        .map_err(|e| e.to_string())?;
    *state.config.lock().unwrap() = config.clone();
    rebind_shortcut(&app, &old_shortcut, &config.shortcut)?;
    Ok(())
}

#[tauri::command]
async fn pick_vault_folder(app: AppHandle) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .blocking_pick_folder()
            .map(|p| p.to_string())
    })
    .await
    .ok()
    .flatten()
}

fn rebind_shortcut(app: &AppHandle, old_shortcut: &str, new_shortcut: &str) -> Result<(), String> {
    if old_shortcut == new_shortcut {
        return Ok(());
    }

    let gs = app.global_shortcut();
    if gs.is_registered(old_shortcut) {
        gs.unregister(old_shortcut)
            .map_err(|e| e.to_string())?;
    }

    let app_handle = app.clone();
    gs.on_shortcut(new_shortcut, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            tray::handle_clip(&app_handle);
        }
    })
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load(&obsclip_config_path()).expect("failed to load config");
    let tray_icons = TrayIcons::new();

    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(config.clone()),
            tray_icons: tray_icons.clone(),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            pick_vault_folder
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::setup_tray(app, &tray_icons)?;
            tray::setup_settings_window(app)?;

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
