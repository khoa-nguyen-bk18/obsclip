use obsclip_lib::clip::formatter::format_text;
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
