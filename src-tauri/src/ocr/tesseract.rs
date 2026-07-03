use kreuzberg_tesseract::TesseractAPI;
use std::path::Path;

use super::OcrError;

pub fn recognize(
    tessdata_prefix: &Path,
    lang: &str,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<String, OcrError> {
    let tessdata_dir = tessdata_prefix.join("tessdata");
    let datapath = tessdata_dir
        .to_str()
        .ok_or_else(|| OcrError::Engine("invalid tessdata path".into()))?;

    let api = TesseractAPI::new().map_err(|e| OcrError::Engine(e.to_string()))?;
    api.init(datapath, lang)
        .map_err(|e| OcrError::Engine(e.to_string()))?;

    let pixel_count = width as usize * height as usize;
    let expected_len = pixel_count * 4;
    if rgba.len() < expected_len {
        return Err(OcrError::Engine(format!(
            "RGBA buffer too small: got {}, expected {expected_len}",
            rgba.len(),
        )));
    }

    let mut rgb = Vec::with_capacity(pixel_count * 3);
    for chunk in rgba[..expected_len].chunks_exact(4) {
        rgb.extend_from_slice(&chunk[..3]);
    }

    let w = width as i32;
    let h = height as i32;
    let bytes_per_pixel = 3;
    let bytes_per_line = w * bytes_per_pixel;

    api.set_image(&rgb, w, h, bytes_per_pixel, bytes_per_line)
        .map_err(|e| OcrError::Engine(e.to_string()))?;

    let text = api
        .get_utf8_text()
        .map_err(|e| OcrError::Engine(e.to_string()))?;

    Ok(text.trim().to_string())
}
