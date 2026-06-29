use super::{date_format::format_obsidian_date, obsidian::VaultSettings, template::apply_template};
use chrono::NaiveDate;
use std::path::{Path, PathBuf};

pub fn daily_note_path(settings: &VaultSettings, date: NaiveDate) -> PathBuf {
    let stem = format_obsidian_date(&settings.date_format, date);
    if settings.folder.is_empty() {
        PathBuf::from(format!("{stem}.md"))
    } else {
        PathBuf::from(&settings.folder).join(format!("{stem}.md"))
    }
}

pub fn ensure_daily_note_exists(
    vault: &Path,
    rel_path: &Path,
    settings: &VaultSettings,
    date: NaiveDate,
) -> std::io::Result<PathBuf> {
    let abs = vault.join(rel_path);
    if abs.exists() {
        return Ok(abs);
    }
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let stem = abs
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("note");
    let content = if let Some(tpl_rel) = &settings.template {
        let tpl_path = vault.join(format!("{tpl_rel}.md"));
        if tpl_path.exists() {
            let raw = std::fs::read_to_string(&tpl_path)?;
            apply_template(&raw, stem, &date.format("%H:%M").to_string())
        } else {
            format!("# {stem}\n")
        }
    } else {
        format!("# {stem}\n")
    };
    std::fs::write(&abs, content)?;
    Ok(abs)
}
