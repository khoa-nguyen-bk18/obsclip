// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "macos")]
    obsclip_lib::macos_prelaunch::hide_from_dock();

    obsclip_lib::run()
}
