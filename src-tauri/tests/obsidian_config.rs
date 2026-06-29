use obsclip_lib::vault::obsidian::VaultSettings;
use std::path::PathBuf;

#[test]
fn loads_daily_notes_and_attachment_settings() {
    let vault = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
    let settings = VaultSettings::load(&vault).unwrap();
    assert_eq!(settings.date_format, "YYYY-MM-DD");
    assert_eq!(settings.folder, "Daily");
    assert_eq!(settings.template, Some("Templates/Daily".into()));
    assert_eq!(settings.attachment_folder, "attachments");
}
