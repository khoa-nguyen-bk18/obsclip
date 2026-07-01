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

#[test]
fn validate_accepts_obsidian_vault() {
    use obsclip_lib::vault::obsidian::validate_obsidian_vault_path;

    let vault = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
    assert!(validate_obsidian_vault_path(&vault).is_ok());
}

#[test]
fn validate_rejects_non_vault_folder() {
    use obsclip_lib::vault::obsidian::validate_obsidian_vault_path;

    let dir = tempfile::tempdir().unwrap();
    let error = validate_obsidian_vault_path(dir.path()).unwrap_err();
    assert!(error.contains("not an Obsidian vault"));
}
