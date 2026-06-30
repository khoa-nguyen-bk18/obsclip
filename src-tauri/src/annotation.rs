use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, Window, WindowEvent};

use crate::clip::formatter::{format_image_link, format_text};
use crate::clip::image::clip_image_filename;
use crate::clipboard::ClipboardContent;
use crate::clip::service::{run_clip, ClipInput};
use crate::config::AppConfig;
use crate::platform;
use crate::tray;

pub const ANNOTATION_WINDOW_LABEL: &str = "annotation";

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AnnotationShowPayload {
    entry_preview: String,
}

pub struct AnnotationState {
    session_id: AtomicU64,
    completed: AtomicBool,
    pending: Mutex<Option<PendingClip>>,
}

struct PendingClip {
    id: u64,
    content: ClipboardContent,
    config: AppConfig,
}

impl AnnotationState {
    pub fn new() -> Self {
        Self {
            session_id: AtomicU64::new(0),
            completed: AtomicBool::new(false),
            pending: Mutex::new(None),
        }
    }
}

pub fn start_clip_with_annotation(app: &AppHandle, config: AppConfig, content: ClipboardContent) {
    let state = app.state::<AnnotationState>();
    let id = state.session_id.fetch_add(1, Ordering::SeqCst) + 1;

    state.completed.store(false, Ordering::SeqCst);

    let preview = entry_preview(&content, &config.text_format);
    let payload = AnnotationShowPayload {
        entry_preview: preview,
    };

    *state.pending.lock().unwrap() = Some(PendingClip {
        id,
        content,
        config,
    });

    let Some(window) = app.get_webview_window(ANNOTATION_WINDOW_LABEL) else {
        eprintln!("Annotation window not found");
        finish_clip(app, id, None);
        return;
    };

    let _ = window.emit("annotation-show", payload);
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
}

fn entry_preview(content: &ClipboardContent, text_format: &crate::config::TextFormat) -> String {
    let time = chrono::Local::now().format("%H:%M").to_string();
    match content {
        ClipboardContent::Text(t) => format_text(text_format.clone(), t, &time),
        ClipboardContent::Image { .. } => {
            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            let hms = chrono::Local::now().format("%H%M%S").to_string();
            let filename = clip_image_filename(&date, &hms);
            format_image_link(&time, &filename)
        }
        ClipboardContent::Empty => String::new(),
    }
}

#[tauri::command]
pub fn submit_annotation(app: AppHandle, text: String) -> Result<(), String> {
    let state = app.state::<AnnotationState>();
    let id = state.session_id.load(Ordering::SeqCst);
    if state.completed.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let trimmed = text.trim();
    let annotation = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    finish_clip(&app, id, annotation);
    Ok(())
}

#[tauri::command]
pub fn cancel_annotation(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AnnotationState>();
    let id = state.session_id.load(Ordering::SeqCst);
    if state.completed.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    abandon_clip(&app, id);
    Ok(())
}

fn abandon_clip(app: &AppHandle, session_id: u64) {
    let state = app.state::<AnnotationState>();
    let pending = {
        let mut guard = state.pending.lock().unwrap();
        guard.take()
    };

    let Some(pending) = pending else {
        return;
    };
    if pending.id != session_id {
        return;
    }

    if let Some(window) = app.get_webview_window(ANNOTATION_WINDOW_LABEL) {
        let _ = window.hide();
    }
}

fn finish_clip(app: &AppHandle, session_id: u64, annotation: Option<String>) {
    let state = app.state::<AnnotationState>();
    let pending = {
        let mut guard = state.pending.lock().unwrap();
        guard.take()
    };

    let Some(pending) = pending else {
        return;
    };
    if pending.id != session_id {
        return;
    }

    if let Some(window) = app.get_webview_window(ANNOTATION_WINDOW_LABEL) {
        let _ = window.hide();
    }

    let obsidian_json = platform::obsidian_config_path();
    let result = run_clip(ClipInput {
        content: pending.content,
        vault_override: pending.config.vault_path.clone(),
        text_format: pending.config.text_format.clone(),
        obsidian_json,
        annotation,
    });

    match result {
        Ok(()) => tray::flash_tray_success(app),
        Err(e) => {
            eprintln!("Clip failed: {e}");
            tray::flash_tray_error(app);
        }
    }
}

pub fn handle_annotation_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != ANNOTATION_WINDOW_LABEL {
        return;
    }

    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let app = window.app_handle();
        let state = app.state::<AnnotationState>();
        let id = state.session_id.load(Ordering::SeqCst);
        if state.completed.swap(true, Ordering::SeqCst) {
            let _ = window.hide();
        } else {
            abandon_clip(app, id);
        }
    }
}
