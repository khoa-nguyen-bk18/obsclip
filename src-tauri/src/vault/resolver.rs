use std::path::{Path, PathBuf};
use serde::Serialize;
use thiserror::Error;

use super::obsidian::validate_obsidian_vault_path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedVault {
    pub path: Option<String>,
    pub error: Option<String>,
}

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

pub fn resolve_effective_vault(vault_override: Option<&Path>, obsidian_json: &Path) -> ResolvedVault {
    match resolve_vault(vault_override, obsidian_json) {
        Ok(path) => {
            if !path.is_dir() {
                return ResolvedVault {
                    path: None,
                    error: Some(format!("Vault folder not found: {}", path.display())),
                };
            }
            if let Err(error) = validate_obsidian_vault_path(&path) {
                return ResolvedVault {
                    path: None,
                    error: Some(error),
                };
            }
            ResolvedVault {
                path: Some(path.to_string_lossy().into_owned()),
                error: None,
            }
        }
        Err(error) => ResolvedVault {
            path: None,
            error: Some(error.to_string()),
        },
    }
}
