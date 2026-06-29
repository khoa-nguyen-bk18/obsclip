use chrono::NaiveDate;
use obsclip_lib::vault::date_format::format_obsidian_date;

#[test]
fn formats_yyyy_mm_dd() {
    let d = NaiveDate::from_ymd_opt(2026, 6, 29).unwrap();
    assert_eq!(format_obsidian_date("YYYY-MM-DD", d), "2026-06-29");
}

#[test]
fn formats_nested_folder_pattern() {
    let d = NaiveDate::from_ymd_opt(2026, 6, 29).unwrap();
    assert_eq!(format_obsidian_date("YYYY/MM/DD", d), "2026/06/29");
}
