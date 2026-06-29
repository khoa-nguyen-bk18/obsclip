#[cfg(test)]
mod tests {
    use obsclip_lib::platform::{obsclip_config_dir, obsidian_config_path};

    #[test]
    fn obsidian_config_path_ends_with_obsidian_json() {
        let path = obsidian_config_path();
        assert!(path.to_string_lossy().ends_with("obsidian.json"));
    }

    #[test]
    fn obsclip_config_dir_is_named_obsclip() {
        let path = obsclip_config_dir();
        assert!(path.to_string_lossy().contains("obsclip"));
    }
}
