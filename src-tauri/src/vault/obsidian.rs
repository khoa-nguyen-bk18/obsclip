use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultSettings {
    pub date_format: String,
    pub folder: String,
    pub template: Option<String>,
    pub attachment_folder: String,
}

#[derive(Debug, Error)]
pub enum ObsidianConfigError {
    #[error("daily-notes.json missing — enable Daily notes in Obsidian")]
    DailyNotesMissing,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn validate_obsidian_vault_path(path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err("Vault folder not found.".into());
    }
    if !path.join(".obsidian").is_dir() {
        return Err(
            "This folder is not an Obsidian vault. Choose a folder that contains a .obsidian directory.".into(),
        );
    }
    Ok(())
}

impl VaultSettings {
    pub fn load(vault_root: &Path) -> Result<Self, ObsidianConfigError> {
        let obsidian_dir = vault_root.join(".obsidian");
        let daily_path = obsidian_dir.join("daily-notes.json");
        if !daily_path.exists() {
            return Err(ObsidianConfigError::DailyNotesMissing);
        }
        let daily: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&daily_path)?)?;
        let app_path = obsidian_dir.join("app.json");
        let attachment_folder = if app_path.exists() {
            let app: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&app_path)?)?;
            normalize_attachment_path(
                app.get("attachmentFolderPath")
                    .and_then(|v| v.as_str())
                    .unwrap_or("attachments"),
            )
        } else {
            "attachments".into()
        };
        Ok(Self {
            date_format: daily
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("YYYY-MM-DD")
                .into(),
            folder: daily.get("folder").and_then(|v| v.as_str()).unwrap_or("").into(),
            template: daily.get("template").and_then(|v| v.as_str()).map(str::to_string),
            attachment_folder,
        })
    }
}

fn normalize_attachment_path(raw: &str) -> String {
    raw.trim_start_matches("./").trim_start_matches('/').to_string()
}
