use obsclip_lib::clip::formatter::{
    format_image_link_with_ocr, format_text, format_text_with_annotation,
};
use obsclip_lib::config::TextFormat;

#[test]
fn timestamped_single_line() {
    let out = format_text(TextFormat::Timestamped, "hello", "14:32");
    assert_eq!(out, "- 14:32 — hello");
}

#[test]
fn blockquote_multiline() {
    let out = format_text(TextFormat::Blockquote, "line1\nline2", "14:32");
    assert_eq!(out, "> line1\n> line2");
}

#[test]
fn checkbox_format() {
    let out = format_text(TextFormat::Checkbox, "task", "09:00");
    assert_eq!(out, "- [ ] task");
}

#[test]
fn timestamped_with_annotation() {
    let out = format_text_with_annotation(
        TextFormat::Timestamped,
        "hello",
        "14:32",
        Some("extra context"),
    );
    assert_eq!(out, "- 14:32 — hello — extra context");
}

#[test]
fn blockquote_with_annotation() {
    let out = format_text_with_annotation(
        TextFormat::Blockquote,
        "line1",
        "14:32",
        Some("note"),
    );
    assert_eq!(out, "> line1\n> note");
}

#[test]
fn image_link_with_ocr_single_line() {
    let out = format_image_link_with_ocr("14:32", "clip.png", Some("hello"), None);
    assert_eq!(out, "- 14:32 — ![[clip.png]]\n  hello");
}

#[test]
fn image_link_with_ocr_multiline() {
    let out = format_image_link_with_ocr("14:32", "clip.png", Some("line one\nline two"), None);
    assert_eq!(out, "- 14:32 — ![[clip.png]]\n  line one\n  line two");
}

#[test]
fn image_link_with_ocr_and_annotation() {
    let out = format_image_link_with_ocr("14:32", "clip.png", Some("text"), Some("note"));
    assert_eq!(out, "- 14:32 — ![[clip.png]] — note\n  text");
}

#[test]
fn image_link_without_ocr_text() {
    let out = format_image_link_with_ocr("14:32", "clip.png", None, None);
    assert_eq!(out, "- 14:32 — ![[clip.png]]");
}
