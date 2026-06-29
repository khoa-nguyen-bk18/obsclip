use obsclip_lib::clip::service::{run_clip, ClipInput};
use obsclip_lib::clipboard::ClipboardContent;
use obsclip_lib::config::TextFormat;
use obsclip_lib::platform::obsidian_config_path;

/// Manual integration test against the real Obsidian vault on this machine.
/// Run: `cargo test --test live_clip -- --nocapture`
#[test]
fn live_clip_text_to_daily_note() {
    let marker = format!(
        "Obsclip live test {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    run_clip(ClipInput {
        content: ClipboardContent::Text(marker.clone()),
        vault_override: None,
        text_format: TextFormat::Timestamped,
        obsidian_json: obsidian_config_path(),
    })
    .expect("clip to real vault should succeed");

    let vault = obsclip_lib::vault::resolver::resolve_vault_from_obsidian_json(
        &obsidian_config_path(),
    )
    .unwrap();
    let settings = obsclip_lib::vault::obsidian::VaultSettings::load(&vault).unwrap();
    let today = chrono::Local::now().date_naive();
    let rel = obsclip_lib::vault::daily_note::daily_note_path(&settings, today);
    let note = std::fs::read_to_string(vault.join(rel)).expect("daily note should exist");
    assert!(
        note.contains(&marker),
        "daily note should contain clipped text"
    );
    println!("Clipped to daily note: {marker}");
}
