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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            vault_path: None,
            shortcut: "CommandOrControl+Shift+KeyV".into(),
            text_format: TextFormat::Timestamped,
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
}
