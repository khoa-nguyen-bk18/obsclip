use obsclip_lib::clip::image::{attachment_dir, clip_image_filename};
use std::path::PathBuf;

#[test]
fn clip_image_filename_pattern() {
    let name = clip_image_filename("2026-06-29", "143052");
    assert_eq!(name, "clip-2026-06-29-143052.png");
}

#[test]
fn attachment_dir_joins_vault() {
    let vault = PathBuf::from("/vault");
    let dir = attachment_dir(&vault, "attachments");
    assert_eq!(dir, PathBuf::from("/vault/attachments"));
}
