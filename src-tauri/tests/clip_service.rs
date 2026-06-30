use chrono::Local;
use obsclip_lib::clip::service::{run_clip, ClipInput};
use obsclip_lib::clipboard::ClipboardContent;
use obsclip_lib::config::TextFormat;
use obsclip_lib::vault::daily_note::daily_note_path;
use obsclip_lib::vault::obsidian::VaultSettings;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_vault() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let vault = dir.path().to_path_buf();

    let obsidian_dir = vault.join(".obsidian");
    fs::create_dir_all(&obsidian_dir).unwrap();
    fs::write(
        obsidian_dir.join("daily-notes.json"),
        r#"{ "format": "YYYY-MM-DD", "folder": "Daily", "template": null }"#,
    )
    .unwrap();
    fs::write(
        obsidian_dir.join("app.json"),
        r#"{ "attachmentFolderPath": "./attachments" }"#,
    )
    .unwrap();

    let settings = VaultSettings {
        date_format: "YYYY-MM-DD".into(),
        folder: "Daily".into(),
        template: None,
        attachment_folder: "attachments".into(),
    };
    let today = Local::now().date_naive();
    let rel = daily_note_path(&settings, today);
    let note_path = vault.join(&rel);
    fs::create_dir_all(note_path.parent().unwrap()).unwrap();
    fs::write(&note_path, "# Today's note\n").unwrap();

    (dir, vault)
}

#[test]
fn appends_text_to_existing_daily_note() {
    let (_dir, vault) = setup_vault();

    run_clip(ClipInput {
        content: ClipboardContent::Text("hello world".into()),
        vault_override: Some(vault.clone()),
        text_format: TextFormat::Timestamped,
        obsidian_json: PathBuf::from("/tmp/nonexistent-obsidian.json"),
        annotation: None,
    })
    .unwrap();

    let settings = VaultSettings::load(&vault).unwrap();
    let today = Local::now().date_naive();
    let rel = daily_note_path(&settings, today);
    let note_path = vault.join(rel);
    let content = fs::read_to_string(note_path).unwrap();

    assert!(content.contains("\n\n- "));
    assert!(content.contains(" — hello world"));
}
