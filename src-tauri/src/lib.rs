pub mod annotation;
pub mod clip;
pub mod ocr;
pub mod clipboard;
pub mod config;
#[cfg(target_os = "macos")]
pub mod macos_prelaunch;
pub mod platform;
pub mod tray;
pub mod tray_icons;
pub mod vault;

use std::path::Path;
use std::sync::Mutex;

use config::AppConfig;
use platform::obsclip_config_path;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tray_icons::TrayIcons;
use vault::obsidian::validate_obsidian_vault_path;
use vault::resolver::{resolve_effective_vault, ResolvedVault};

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub tray_icons: TrayIcons,
}
#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn get_ocr_health(state: tauri::State<ocr::health::OcrHealthState>) -> ocr::health::OcrHealth {
    state.snapshot()
}

#[tauri::command]
fn get_ocr_languages(
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
) -> Result<Vec<ocr::languages::LanguageEntry>, String> {
    let config = state.config.lock().unwrap();
    let manifest = app
        .path()
        .resolve("tessdata_manifest.json", tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    ocr::languages::list_languages(&manifest, &platform::tessdata_dir(), &config.ocr_languages)
}

#[tauri::command]
fn download_ocr_language(code: String) -> Result<(), String> {
    ocr::languages::download_language(&platform::tessdata_dir(), &code)
}

#[tauri::command]
fn remove_ocr_language(code: String) -> Result<(), String> {
    ocr::languages::remove_language(&platform::tessdata_dir(), &code)
}

#[tauri::command]
fn get_resolved_vault_path(state: tauri::State<AppState>) -> ResolvedVault {
    let config = state.config.lock().unwrap();
    resolve_effective_vault(
        config.vault_path.as_deref(),
        &platform::obsidian_config_path(),
    )
}

#[tauri::command]
fn validate_obsidian_vault(path: String) -> Result<(), String> {
    validate_obsidian_vault_path(Path::new(&path))
}

#[tauri::command]
fn save_config(
    app: AppHandle,
    state: tauri::State<AppState>,
    config: AppConfig,
) -> Result<(), String> {
    ocr::languages::validate_ocr_languages(&config.ocr_languages)?;
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
        .manage(annotation::AnnotationState::new())
        .manage(ocr::health::OcrHealthState::new())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_ocr_health,
            get_ocr_languages,
            download_ocr_language,
            remove_ocr_language,
            get_resolved_vault_path,
            validate_obsidian_vault,
            save_config,
            pick_vault_folder,
            annotation::submit_annotation,
            annotation::cancel_annotation
        ])
        .on_window_event(|window, event| {
            tray::handle_settings_window_event(window, event);
            annotation::handle_annotation_window_event(window, event);
        })
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::setup_tray(app, &tray_icons)?;

            let shortcut = config.shortcut.clone();
            let app_handle = app.handle().clone();
            app.handle()
                .global_shortcut()
                .on_shortcut(shortcut.as_str(), move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        tray::handle_clip(&app_handle);
                    }
                })?;

            let prompt_app = app.handle().clone();
            let prompt_config = config.clone();
            tauri::async_runtime::spawn(async move {
                tray::prompt_vault_setup_if_needed(&prompt_app, &prompt_config).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
