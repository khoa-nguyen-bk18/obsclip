use tauri::image::Image;

#[derive(Clone)]
pub struct TrayIcons {
    pub default: Image<'static>,
    pub success: Image<'static>,
    pub error: Image<'static>,
}

impl TrayIcons {
    pub fn new() -> Self {
        let default = tauri::include_image!("icons/icon.png");
        let success = tint_icon(&default, [34, 197, 94], 0.6);
        let error = tint_icon(&default, [239, 68, 68], 0.6);
        Self {
            default,
            success,
            error,
        }
    }
}

fn tint_icon(base: &Image<'static>, rgb: [u8; 3], strength: f32) -> Image<'static> {
    let mut rgba = base.rgba().to_vec();
    for chunk in rgba.chunks_mut(4) {
        if chunk[3] < 40 {
            continue;
        }
        let inv = 1.0 - strength;
        chunk[0] = (chunk[0] as f32 * inv + rgb[0] as f32 * strength) as u8;
        chunk[1] = (chunk[1] as f32 * inv + rgb[1] as f32 * strength) as u8;
        chunk[2] = (chunk[2] as f32 * inv + rgb[2] as f32 * strength) as u8;
    }
    Image::new_owned(rgba, base.width(), base.height())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tinted_icons_differ_from_default() {
        let icons = TrayIcons::new();
        assert_ne!(icons.default.rgba(), icons.success.rgba());
        assert_ne!(icons.default.rgba(), icons.error.rgba());
    }

    /// Run once to refresh README screenshots:
    /// `cargo test -p obsclip export_readme_icons -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn export_readme_icons() {
        let icons = TrayIcons::new();
        let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/screenshots");
        std::fs::create_dir_all(&out_dir).unwrap();
        write_icon(&icons.default, &out_dir.join("tray-default.png"));
        write_icon(&icons.success, &out_dir.join("tray-success.png"));
        write_icon(&icons.error, &out_dir.join("tray-error.png"));
    }

    fn write_icon(icon: &Image<'static>, path: &std::path::Path) {
        use image::{ImageBuffer, Rgba};
        let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
            icon.width(),
            icon.height(),
            icon.rgba().to_vec(),
        )
        .expect("icon rgba");
        buffer.save(path).expect("save icon png");
    }
}
