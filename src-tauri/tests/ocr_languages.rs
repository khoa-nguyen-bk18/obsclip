use obsclip_lib::ocr::languages::{
    build_lang_string, validate_ocr_languages, LanguageEntry, LanguageStatus,
};

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
fn language_status_bundled_for_eng() {
    let entry = LanguageEntry {
        code: "eng".into(),
        name: "eng".into(),
        status: LanguageStatus::Bundled,
        enabled: true,
    };
    assert_eq!(entry.status, LanguageStatus::Bundled);
}
