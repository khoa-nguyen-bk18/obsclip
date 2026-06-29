use obsclip_lib::vault::resolver::resolve_vault_from_obsidian_json;
use std::path::PathBuf;

#[test]
fn resolves_last_open_vault() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/obsidian.json");
    let path = resolve_vault_from_obsidian_json(&fixture).unwrap();
    assert_eq!(path, PathBuf::from("/Users/test/MyVault"));
}

#[test]
fn resolves_open_vault_without_last_open() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/obsidian-open-only.json");
    let path = resolve_vault_from_obsidian_json(&fixture).unwrap();
    assert_eq!(path, PathBuf::from("/Users/test/OpenVault"));
}

#[test]
fn override_takes_precedence() {
    use obsclip_lib::vault::resolver::resolve_vault;
    let override_path = PathBuf::from("/override/vault");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/obsidian.json");
    let path = resolve_vault(Some(&override_path), &fixture).unwrap();
    assert_eq!(path, override_path);
}
