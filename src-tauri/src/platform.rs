use std::path::PathBuf;

pub fn obsidian_config_path() -> PathBuf {
    config_root().join("obsidian").join("obsidian.json")
}

pub fn obsclip_config_dir() -> PathBuf {
    config_root().join("obsclip")
}

pub fn obsclip_config_path() -> PathBuf {
    obsclip_config_dir().join("config.json")
}

pub fn tessdata_dir() -> PathBuf {
    obsclip_config_dir().join("tessdata")
}

pub fn tessdata_prefix() -> PathBuf {
    obsclip_config_dir()
}

pub fn bundled_eng_traineddata() -> PathBuf {
    for candidate in bundled_eng_candidates() {
        if candidate.is_file() {
            return candidate;
        }
    }
    PathBuf::from("resources/tessdata/eng.traineddata")
}

fn bundled_eng_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        #[cfg(target_os = "macos")]
        if let Some(contents) = exe.parent().and_then(|p| p.parent()) {
            candidates.push(contents.join("Resources/tessdata/eng.traineddata"));
        }
        #[cfg(target_os = "windows")]
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("tessdata/eng.traineddata"));
        }
    }
    candidates.push(PathBuf::from("src-tauri/resources/tessdata/eng.traineddata"));
    candidates.push(PathBuf::from("resources/tessdata/eng.traineddata"));
    candidates
}

fn config_root() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .expect("home dir")
            .join("Library/Application Support")
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .expect("APPDATA")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        compile_error!("v1 supports macOS and Windows only");
    }
}
