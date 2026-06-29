use chrono::NaiveDate;
use obsclip_lib::vault::daily_note::daily_note_path;
use obsclip_lib::vault::obsidian::VaultSettings;
use obsclip_lib::vault::template::apply_template;
use std::path::PathBuf;

#[test]
fn builds_daily_note_relative_path() {
    let settings = VaultSettings {
        date_format: "YYYY-MM-DD".into(),
        folder: "Daily".into(),
        template: None,
        attachment_folder: "attachments".into(),
    };
    let d = NaiveDate::from_ymd_opt(2026, 6, 29).unwrap();
    let path = daily_note_path(&settings, d);
    assert_eq!(path, PathBuf::from("Daily/2026-06-29.md"));
}

#[test]
fn apply_template_substitutes_core_variables() {
    let content = "# {{title}}\n{{date}} at {{time}}";
    let out = apply_template(content, "2026-06-29", "09:15");
    assert_eq!(out, "# 2026-06-29\n2026-06-29 at 09:15");
}
