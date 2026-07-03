use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;

use crate::clip::formatter::{format_image_link_with_ocr, format_text_with_annotation};
use crate::clip::image::{attachment_dir, clip_image_filename, save_png};
use crate::clipboard::{read_clipboard, ClipboardContent};
use crate::config::{AppConfig, TextFormat};
use crate::ocr::health::OcrHealthState;
use crate::ocr::languages::{
    build_lang_string, ensure_english_installed, missing_enabled_languages, validate_ocr_languages,
};
use crate::ocr::recognize_text;
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
pub struct ClipOutcome {
    pub ocr_toast: bool,
}

#[derive(Debug)]
pub struct ClipInput {
    pub content: ClipboardContent,
    pub vault_override: Option<PathBuf>,
    pub text_format: TextFormat,
    pub obsidian_json: PathBuf,
    pub annotation: Option<String>,
    pub image_ocr: bool,
    pub ocr_languages: Vec<String>,
    pub tessdata_dir: PathBuf,
    pub tessdata_prefix: PathBuf,
    pub bundled_eng: PathBuf,
    pub ocr_health: Option<Arc<OcrHealthState>>,
}

pub fn run_clip(input: ClipInput) -> Result<ClipOutcome, ClipError> {
    let vault = resolve_vault(input.vault_override.as_deref(), &input.obsidian_json)?;
    let settings = VaultSettings::load(&vault)?;
    let today = chrono::Local::now().date_naive();
    let rel = daily_note_path(&settings, today);
    let note_path = ensure_daily_note_exists(&vault, &rel, &settings, today)?;
    let time = chrono::Local::now().format("%H:%M").to_string();
    let annotation = input.annotation.as_deref();
    let ocr_toast = match input.content {
        ClipboardContent::Text(t) => {
            let block = format_text_with_annotation(input.text_format, &t, &time, annotation);
            append_to_file(&note_path, &block)?;
            false
        }
        ClipboardContent::Image { rgba, width, height } => {
            let date = today.format("%Y-%m-%d").to_string();
            let hms = chrono::Local::now().format("%H%M%S").to_string();
            let filename = clip_image_filename(&date, &hms);
            let dir = attachment_dir(&vault, &settings.attachment_folder);
            save_png(&dir, &filename, &rgba, width, height)?;

            let mut ocr_text: Option<String> = None;
            let mut ocr_toast = false;

            if input.image_ocr {
                if input.ocr_languages.is_empty() {
                    if let Some(health) = &input.ocr_health {
                        health.set_error(
                            "No OCR languages enabled.",
                            "Enable at least one OCR language in Settings.",
                        );
                    }
                } else if let Err(msg) =
                    validate_ocr_languages(&input.ocr_languages)
                {
                    if let Some(health) = &input.ocr_health {
                        health.set_error(msg.clone(), msg);
                    }
                } else {
                    let _ = ensure_english_installed(&input.tessdata_dir, &input.bundled_eng);
                    let missing =
                        missing_enabled_languages(&input.tessdata_dir, &input.ocr_languages);
                    if !missing.is_empty() {
                        ocr_toast = true;
                        if let Some(health) = &input.ocr_health {
                            let lang = &missing[0];
                            health.set_error(
                                format!("Missing language pack: {lang}"),
                                format!("Download the {lang} language pack in Settings."),
                            );
                        }
                    } else {
                        let lang = build_lang_string(&input.ocr_languages);
                        match recognize_text(
                            &input.tessdata_prefix,
                            &lang,
                            &rgba,
                            width,
                            height,
                        ) {
                            Ok(text) if text.is_empty() => {
                                if let Some(health) = &input.ocr_health {
                                    health.clear();
                                }
                            }
                            Ok(text) => {
                                ocr_text = Some(text);
                                if let Some(health) = &input.ocr_health {
                                    health.clear();
                                }
                            }
                            Err(e) => {
                                ocr_toast = true;
                                if let Some(health) = &input.ocr_health {
                                    health.set_error(
                                        format!("OCR failed: {e}"),
                                        "Try clipping again. If it persists, restart Obsclip.",
                                    );
                                }
                            }
                        }
                    }
                }
            }

            let block =
                format_image_link_with_ocr(&time, &filename, ocr_text.as_deref(), annotation);
            append_to_file(&note_path, &block)?;
            ocr_toast
        }
        ClipboardContent::Empty => return Err(ClipError::EmptyClipboard),
    };
    Ok(ClipOutcome { ocr_toast })
}

pub fn clip_from_config(config: &AppConfig) -> Result<(), ClipError> {
    let obsidian_json = platform::obsidian_config_path();
    let content = read_clipboard().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;
    let _ = run_clip(ClipInput {
        content,
        vault_override: config.vault_path.clone(),
        text_format: config.text_format.clone(),
        obsidian_json,
        annotation: None,
        image_ocr: config.image_ocr,
        ocr_languages: config.ocr_languages.clone(),
        tessdata_dir: platform::tessdata_dir(),
        tessdata_prefix: platform::tessdata_prefix(),
        bundled_eng: platform::bundled_eng_traineddata(),
        ocr_health: None,
    })?;
    Ok(())
}

fn append_to_file(path: &Path, block: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new().append(true).open(path)?;
    write!(file, "\n\n{block}")?;
    Ok(())
}
