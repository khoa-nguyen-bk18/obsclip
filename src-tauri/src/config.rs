use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TextFormat {
    Timestamped,
    Blockquote,
    Bullet,
    Checkbox,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub vault_path: Option<PathBuf>,
    pub shortcut: String,
    pub text_format: TextFormat,
    #[serde(default = "default_annotation_prompt")]
    pub annotation_prompt: bool,
}

fn default_annotation_prompt() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            vault_path: None,
            shortcut: "CommandOrControl+Shift+KeyV".into(),
            text_format: TextFormat::Timestamped,
            annotation_prompt: true,
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path)?;
        serde_json::from_str(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_serializes() {
        let cfg = AppConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(json.contains("timestamped"));
    }

    #[test]
    fn load_missing_returns_default() {
        let cfg = AppConfig::load(Path::new("/tmp/nonexistent-obsclip-config-xyz.json")).unwrap();
        assert_eq!(cfg, AppConfig::default());
    }

    #[test]
    fn load_missing_annotation_prompt_defaults_true() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"vault_path":null,"shortcut":"CommandOrControl+Shift+KeyV","text_format":"timestamped"}"#,
        )
        .unwrap();
        let cfg = AppConfig::load(&path).unwrap();
        assert!(cfg.annotation_prompt);
    }
}
