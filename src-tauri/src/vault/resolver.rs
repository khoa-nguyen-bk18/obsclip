use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("obsidian config not found: {0}")]
    NotFound(PathBuf),
    #[error("no last_open vault in obsidian.json")]
    NoLastOpen,
    #[error("invalid obsidian.json: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn resolve_vault(override_path: Option<&Path>, obsidian_json: &Path) -> Result<PathBuf, VaultError> {
    if let Some(p) = override_path {
        return Ok(p.to_path_buf());
    }
    resolve_vault_from_obsidian_json(obsidian_json)
}

pub fn resolve_vault_from_obsidian_json(path: &Path) -> Result<PathBuf, VaultError> {
    if !path.exists() {
        return Err(VaultError::NotFound(path.to_path_buf()));
    }
    let data = std::fs::read_to_string(path)?;
    let root: serde_json::Value = serde_json::from_str(&data)?;
    let last_open = root.get("last_open").and_then(|v| v.as_str()).ok_or(VaultError::NoLastOpen)?;
    let vault_path = root
        .pointer(&format!("/vaults/{last_open}/path"))
        .and_then(|v| v.as_str())
        .ok_or(VaultError::NoLastOpen)?;
    Ok(PathBuf::from(vault_path))
}
