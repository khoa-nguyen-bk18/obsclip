//! Set accessory activation policy before Tauri initializes.
//! Launch Services (`open -a`, Finder) can register a Dock icon before
//! Tauri's setup hook runs; this runs first in `main()`.

#[cfg(target_os = "macos")]
pub fn hide_from_dock() {
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    use objc2_foundation::MainThreadMarker;

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };

    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}

#[cfg(not(target_os = "macos"))]
pub fn hide_from_dock() {}
