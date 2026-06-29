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
