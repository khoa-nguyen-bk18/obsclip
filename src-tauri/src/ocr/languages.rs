use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LanguageStatus {
    Bundled,
    Installed,
    NotDownloaded,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageEntry {
    pub code: String,
    pub name: String,
    pub status: LanguageStatus,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
struct ManifestEntry {
    code: String,
    name: String,
}

pub fn build_lang_string(codes: &[String]) -> String {
    codes.join("+")
}

pub fn validate_ocr_languages(codes: &[String]) -> Result<(), String> {
    if codes.len() > 2 {
        return Err("Select at most two languages.".into());
    }
    Ok(())
}

pub fn traineddata_path(tessdata_dir: &Path, code: &str) -> PathBuf {
    tessdata_dir.join(format!("{code}.traineddata"))
}

pub fn is_installed(tessdata_dir: &Path, code: &str) -> bool {
    traineddata_path(tessdata_dir, code).is_file()
}

pub fn load_manifest(manifest_path: &Path) -> Result<Vec<ManifestEntry>, String> {
    let data = fs::read_to_string(manifest_path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

fn language_status(tessdata_dir: &Path, code: &str) -> LanguageStatus {
    if code == "eng" {
        LanguageStatus::Bundled
    } else if is_installed(tessdata_dir, code) {
        LanguageStatus::Installed
    } else {
        LanguageStatus::NotDownloaded
    }
}

fn language_sort_key(status: &LanguageStatus, name: &str) -> (u8, String) {
    let rank = match status {
        LanguageStatus::Bundled => 0,
        LanguageStatus::Installed => 1,
        LanguageStatus::NotDownloaded => 2,
    };
    (rank, name.to_lowercase())
}

pub fn list_languages(
    manifest_path: &Path,
    tessdata_dir: &Path,
    enabled: &[String],
) -> Result<Vec<LanguageEntry>, String> {
    let manifest = load_manifest(manifest_path)?;
    let mut entries: Vec<LanguageEntry> = manifest
        .into_iter()
        .map(|m| {
            let status = language_status(tessdata_dir, &m.code);
            LanguageEntry {
                enabled: enabled.contains(&m.code),
                code: m.code.clone(),
                name: m.name,
                status,
            }
        })
        .collect();
    entries.sort_by(|a, b| {
        language_sort_key(&a.status, &a.name).cmp(&language_sort_key(&b.status, &b.name))
    });
    Ok(entries)
}

pub fn ensure_english_installed(tessdata_dir: &Path, bundled_eng: &Path) -> Result<(), String> {
    fs::create_dir_all(tessdata_dir).map_err(|e| e.to_string())?;
    let dest = traineddata_path(tessdata_dir, "eng");
    if dest.is_file() {
        return Ok(());
    }
    fs::copy(bundled_eng, &dest).map_err(|e| e.to_string())?;
    Ok(())
}

const TESSDATA_DOWNLOAD_BASE: &str = "https://github.com/tesseract-ocr/tessdata/raw/main";

pub fn download_language(tessdata_dir: &Path, code: &str) -> Result<(), String> {
    if code == "eng" {
        return Ok(());
    }
    fs::create_dir_all(tessdata_dir).map_err(|e| e.to_string())?;
    let url = format!("{TESSDATA_DOWNLOAD_BASE}/{code}.traineddata");
    let bytes = reqwest::blocking::get(&url)
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;
    fs::write(traineddata_path(tessdata_dir, code), bytes).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_language(tessdata_dir: &Path, code: &str) -> Result<(), String> {
    if code == "eng" {
        return Err("English cannot be removed.".into());
    }
    let path = traineddata_path(tessdata_dir, code);
    if path.is_file() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn missing_enabled_languages(tessdata_dir: &Path, enabled: &[String]) -> Vec<String> {
    enabled
        .iter()
        .filter(|code| !is_installed(tessdata_dir, code))
        .cloned()
        .collect()
}
