use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};

const TOAST_LABEL: &str = "toast";
const TOAST_MS: u64 = 3000;
const TOAST_MARGIN: i32 = 16;

pub fn show_ocr_failure_toast(app: &AppHandle) {
    let message = "OCR failed — image saved. Open Settings for details.";
    let Some(window) = app.get_webview_window(TOAST_LABEL) else {
        return;
    };

    let _ = window.emit(
        "toast-show",
        serde_json::json!({ "message": message }),
    );
    let _ = window.show();
    position_bottom_right(&window);

    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(TOAST_MS));
        if let Some(w) = app_clone.get_webview_window(TOAST_LABEL) {
            let _ = w.hide();
        }
    });
}

fn position_bottom_right(window: &tauri::WebviewWindow) {
    let monitor = window
        .primary_monitor()
        .ok()
        .flatten()
        .or_else(|| window.current_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return;
    };

    let monitor_size = monitor.size();
    let monitor_pos = monitor.position();
    let window_size = window
        .outer_size()
        .unwrap_or(tauri::PhysicalSize::new(320, 56));

    let x = monitor_pos.x + monitor_size.width as i32 - window_size.width as i32 - TOAST_MARGIN;
    let y = monitor_pos.y + monitor_size.height as i32 - window_size.height as i32 - TOAST_MARGIN;
    let _ = window.set_position(PhysicalPosition::new(x, y));
}
