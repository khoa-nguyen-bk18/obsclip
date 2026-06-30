use std::io::Write;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::clip::formatter::{format_image_link_with_annotation, format_text_with_annotation};
use crate::clip::image::{attachment_dir, clip_image_filename, save_png};
use crate::clipboard::{read_clipboard, ClipboardContent};
use crate::config::{AppConfig, TextFormat};
use crate::platform;
use crate::vault::daily_note::{daily_note_path, ensure_daily_note_exists};
use crate::vault::obsidian::{ObsidianConfigError, VaultSettings};
use crate::vault::resolver::{resolve_vault, VaultError};

#[derive(Debug, Error)]
pub enum ClipError {
    #[error("clipboard is empty")]
    EmptyClipboard,
    #[error(transparent)]
    Vault(#[from] VaultError),
    #[error(transparent)]
    Config(#[from] ObsidianConfigError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct ClipInput {
    pub content: ClipboardContent,
    pub vault_override: Option<PathBuf>,
    pub text_format: TextFormat,
    pub obsidian_json: PathBuf,
    pub annotation: Option<String>,
}

pub fn run_clip(input: ClipInput) -> Result<(), ClipError> {
    let vault = resolve_vault(input.vault_override.as_deref(), &input.obsidian_json)?;
    let settings = VaultSettings::load(&vault)?;
    let today = chrono::Local::now().date_naive();
    let rel = daily_note_path(&settings, today);
    let note_path = ensure_daily_note_exists(&vault, &rel, &settings, today)?;
    let time = chrono::Local::now().format("%H:%M").to_string();
    let annotation = input.annotation.as_deref();
    let block = match input.content {
        ClipboardContent::Text(t) => {
            format_text_with_annotation(input.text_format, &t, &time, annotation)
        }
        ClipboardContent::Image { rgba, width, height } => {
            let date = today.format("%Y-%m-%d").to_string();
            let hms = chrono::Local::now().format("%H%M%S").to_string();
            let filename = clip_image_filename(&date, &hms);
            let dir = attachment_dir(&vault, &settings.attachment_folder);
            save_png(&dir, &filename, &rgba, width, height)?;
            format_image_link_with_annotation(&time, &filename, annotation)
        }
        ClipboardContent::Empty => return Err(ClipError::EmptyClipboard),
    };
    append_to_file(&note_path, &block)?;
    Ok(())
}

pub fn clip_from_config(config: &AppConfig) -> Result<(), ClipError> {
    let obsidian_json = platform::obsidian_config_path();
    let content = read_clipboard().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;
    run_clip(ClipInput {
        content,
        vault_override: config.vault_path.clone(),
        text_format: config.text_format.clone(),
        obsidian_json,
        annotation: None,
    })
}

fn append_to_file(path: &Path, block: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new().append(true).open(path)?;
    write!(file, "\n\n{block}")?;
    Ok(())
}
