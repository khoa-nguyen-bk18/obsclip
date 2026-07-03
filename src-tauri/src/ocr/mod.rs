pub mod health;
pub mod languages;
pub mod tesseract;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("tesseract failed: {0}")]
    Engine(String),
}

pub fn recognize_text(
    tessdata_prefix: &std::path::Path,
    lang: &str,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<String, OcrError> {
    tesseract::recognize(tessdata_prefix, lang, rgba, width, height)
}
