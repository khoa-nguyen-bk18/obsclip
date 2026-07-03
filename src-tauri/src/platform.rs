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
    std::env::current_exe()
        .ok()
        .and_then(|exe| {
            #[cfg(target_os = "macos")]
            {
                exe.parent()
                    .and_then(|p| p.parent())
                    .map(|contents| contents.join("Resources/tessdata/eng.traineddata"))
            }
            #[cfg(target_os = "windows")]
            {
                exe.parent().map(|p| p.join("tessdata/eng.traineddata"))
            }
            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            {
                None
            }
        })
        .unwrap_or_else(|| PathBuf::from("resources/tessdata/eng.traineddata"))
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
