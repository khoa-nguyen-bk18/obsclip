pub enum ClipboardContent {
    Text(String),
    Image {
        rgba: Vec<u8>,
        width: u32,
        height: u32,
    },
    Empty,
}

pub fn read_clipboard() -> Result<ClipboardContent, arboard::Error> {
    let mut clipboard = arboard::Clipboard::new()?;
    if let Ok(img) = clipboard.get_image() {
        return Ok(ClipboardContent::Image {
            rgba: img.bytes.to_vec(),
            width: img.width as u32,
            height: img.height as u32,
        });
    }
    if let Ok(text) = clipboard.get_text() {
        if text.trim().is_empty() {
            return Ok(ClipboardContent::Empty);
        }
        return Ok(ClipboardContent::Text(text));
    }
    Ok(ClipboardContent::Empty)
}
