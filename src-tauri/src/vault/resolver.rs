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
    let vaults = root.get("vaults").and_then(|v| v.as_object()).ok_or(VaultError::NoLastOpen)?;

    if let Some(last_open) = root.get("last_open").and_then(|v| v.as_str()) {
        if let Some(path) = vaults.get(last_open).and_then(|v| v.get("path")).and_then(|v| v.as_str()) {
            return Ok(PathBuf::from(path));
        }
    }

    let open_vaults: Vec<_> = vaults
        .iter()
        .filter(|(_, v)| v.get("open").and_then(|o| o.as_bool()) == Some(true))
        .collect();
    if open_vaults.len() == 1 {
        if let Some(path) = open_vaults[0].1.get("path").and_then(|v| v.as_str()) {
            return Ok(PathBuf::from(path));
        }
    }

    if vaults.len() == 1 {
        if let Some(path) = vaults.values().next().and_then(|v| v.get("path")).and_then(|v| v.as_str()) {
            return Ok(PathBuf::from(path));
        }
    }

    let most_recent = vaults
        .iter()
        .filter_map(|(_, v)| {
            let path = v.get("path")?.as_str()?;
            let ts = v.get("ts").and_then(|t| t.as_i64()).unwrap_or(0);
            Some((ts, path))
        })
        .max_by_key(|(ts, _)| *ts);

    if let Some((_, path)) = most_recent {
        return Ok(PathBuf::from(path));
    }

    Err(VaultError::NoLastOpen)
}
