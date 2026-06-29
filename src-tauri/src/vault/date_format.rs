use chrono::{Datelike, NaiveDate};

pub fn format_obsidian_date(pattern: &str, date: NaiveDate) -> String {
    let mut out = pattern.to_string();
    let replacements = [
        ("YYYY", format!("{:04}", date.year())),
        ("YY", format!("{:02}", date.year() % 100)),
        ("MMMM", date.format("%B").to_string()),
        ("MMM", date.format("%b").to_string()),
        ("MM", format!("{:02}", date.month())),
        ("DD", format!("{:02}", date.day())),
        ("dddd", date.format("%A").to_string()),
        ("ddd", date.format("%a").to_string()),
        ("M", format!("{}", date.month())),
        ("D", format!("{}", date.day())),
    ];
    for (token, value) in replacements {
        out = out.replace(token, &value);
    }
    out
}
