use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager, Window, WindowEvent};

use crate::clip::service::{run_clip, ClipInput};
use crate::clipboard::ClipboardContent;
use crate::config::AppConfig;
use crate::ocr::health::OcrHealthState;
use crate::platform;
use crate::tray;

pub const COMPOSE_WINDOW_LABEL: &str = "compose";

pub struct ComposeState {
    session_id: AtomicU64,
    completed: AtomicBool,
    pending: Mutex<Option<PendingCompose>>,
}

struct PendingCompose {
    id: u64,
    config: AppConfig,
}

impl ComposeState {
    pub fn new() -> Self {
        Self {
            session_id: AtomicU64::new(0),
            completed: AtomicBool::new(false),
            pending: Mutex::new(None),
        }
    }
}

pub fn start_compose(app: &AppHandle, config: AppConfig) {
    let state = app.state::<ComposeState>();
    let id = state.session_id.fetch_add(1, Ordering::SeqCst) + 1;

    state.completed.store(false, Ordering::SeqCst);
    *state.pending.lock().unwrap() = Some(PendingCompose { id, config });

    let Some(window) = app.get_webview_window(COMPOSE_WINDOW_LABEL) else {
        eprintln!("Compose window not found");
        abandon(app, id);
        tray::flash_tray_error(app);
        return;
    };

    let _ = window.emit("compose-show", ());
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn cancel_if_open(app: &AppHandle) {
    let state = app.state::<ComposeState>();
    let id = state.session_id.load(Ordering::SeqCst);
    if state.completed.swap(true, Ordering::SeqCst) {
        if let Some(window) = app.get_webview_window(COMPOSE_WINDOW_LABEL) {
            let _ = window.hide();
        }
        return;
    }
    abandon(app, id);
}

#[tauri::command]
pub fn submit_compose(app: AppHandle, text: String) -> Result<(), String> {
    let state = app.state::<ComposeState>();
    let id = state.session_id.load(Ordering::SeqCst);
    if state.completed.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    match compose_payload(&text) {
        None => abandon(&app, id),
        Some(body) => finish_write(&app, id, body),
    }
    Ok(())
}

#[tauri::command]
pub fn cancel_compose(app: AppHandle) -> Result<(), String> {
    cancel_if_open(&app);
    Ok(())
}

fn take_pending(app: &AppHandle, session_id: u64) -> Option<PendingCompose> {
    let state = app.state::<ComposeState>();
    let pending = {
        let mut guard = state.pending.lock().unwrap();
        guard.take()
    };
    pending.filter(|pending| pending.id == session_id)
}

fn hide_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(COMPOSE_WINDOW_LABEL) {
        let _ = window.hide();
    }
}

fn abandon(app: &AppHandle, session_id: u64) {
    let _ = take_pending(app, session_id);
    hide_window(app);
}

fn finish_write(app: &AppHandle, session_id: u64, text: String) {
    let Some(pending) = take_pending(app, session_id) else {
        return;
    };
    hide_window(app);

    let obsidian_json = platform::obsidian_config_path();
    let bundled_eng = app.state::<crate::AppState>().bundled_eng.clone();
    let result = run_clip(ClipInput {
        content: ClipboardContent::Text(text),
        vault_override: pending.config.vault_path.clone(),
        text_format: pending.config.text_format.clone(),
        obsidian_json,
        annotation: None,
        image_ocr: pending.config.image_ocr,
        ocr_languages: pending.config.ocr_languages.clone(),
        tessdata_dir: platform::tessdata_dir(),
        tessdata_prefix: platform::tessdata_prefix(),
        bundled_eng,
        ocr_health: Some(app.state::<Arc<OcrHealthState>>().inner().clone()),
    });

    match result {
        Ok(_) => tray::flash_tray_success(app),
        Err(e) => {
            eprintln!("Write failed: {e}");
            tray::flash_tray_error(app);
        }
    }
}

pub fn handle_compose_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != COMPOSE_WINDOW_LABEL {
        return;
    }

    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        cancel_if_open(window.app_handle());
    }
}

pub fn compose_payload(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::compose_payload;

    #[test]
    fn empty_and_whitespace_are_none() {
        assert_eq!(compose_payload(""), None);
        assert_eq!(compose_payload("   \n\t  "), None);
    }

    #[test]
    fn trims_and_keeps_inner_newlines() {
        assert_eq!(
            compose_payload("  hello\nworld  \n"),
            Some("hello\nworld".to_string())
        );
    }
}
