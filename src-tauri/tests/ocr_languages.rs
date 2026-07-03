use obsclip_lib::ocr::languages::{build_lang_string, validate_ocr_languages};

#[test]
fn build_lang_string_joins_with_plus() {
    assert_eq!(build_lang_string(&["eng".into(), "vie".into()]), "eng+vie");
}

#[test]
fn validate_ocr_languages_rejects_more_than_two() {
    let err = validate_ocr_languages(&["eng".into(), "vie".into(), "deu".into()]);
    assert!(err.is_err());
}

#[test]
fn list_languages_sorts_bundled_and_installed_first() {
    use obsclip_lib::ocr::languages::{list_languages, LanguageStatus};
    use std::fs;
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let tessdata_dir = dir.path().join("tessdata");
    fs::create_dir_all(&tessdata_dir).unwrap();
    fs::write(tessdata_dir.join("eng.traineddata"), b"eng").unwrap();
    fs::write(tessdata_dir.join("deu.traineddata"), b"deu").unwrap();

    let manifest_path = dir.path().join("manifest.json");
    fs::write(
        &manifest_path,
        r#"[
          {"code":"deu","name":"German"},
          {"code":"eng","name":"English"},
          {"code":"vie","name":"Vietnamese"}
        ]"#,
    )
    .unwrap();

    let entries = list_languages(&manifest_path, &tessdata_dir, &["eng".into()]).unwrap();
    assert_eq!(entries[0].code, "eng");
    assert_eq!(entries[0].status, LanguageStatus::Bundled);
    assert_eq!(entries[1].code, "deu");
    assert_eq!(entries[1].status, LanguageStatus::Installed);
    assert_eq!(entries[2].code, "vie");
    assert_eq!(entries[2].status, LanguageStatus::NotDownloaded);
}
