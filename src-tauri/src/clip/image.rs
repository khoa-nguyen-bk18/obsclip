use std::path::{Path, PathBuf};

pub fn clip_image_filename(date: &str, time_hms: &str) -> String {
    format!("clip-{date}-{time_hms}.png")
}

pub fn attachment_dir(vault: &Path, folder: &str) -> PathBuf {
    vault.join(folder)
}

pub fn save_png(dir: &Path, filename: &str, rgba: &[u8], width: u32, height: u32) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(filename);
    let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "bad rgba"))?;
    img.save(&path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(path)
}
