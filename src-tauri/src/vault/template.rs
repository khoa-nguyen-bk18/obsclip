use chrono::{Local, NaiveDate};

use super::date_format::format_obsidian_date;

fn resolve_date(title: &str) -> NaiveDate {
    NaiveDate::parse_from_str(title, "%Y-%m-%d").unwrap_or_else(|_| Local::now().date_naive())
}

fn replace_date_format_vars(content: &str, title: &str) -> String {
    let date = resolve_date(title);
    let mut out = String::with_capacity(content.len());
    let mut rest = content;

    while let Some(start) = rest.find("{{date:") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 7..];
        if let Some(end) = rest.find("}}") {
            let pattern = &rest[..end];
            out.push_str(&format_obsidian_date(pattern, date));
            rest = &rest[end + 2..];
        } else {
            out.push_str("{{date:");
            break;
        }
    }
    out.push_str(rest);
    out
}

pub fn apply_template(content: &str, title: &str, time: &str) -> String {
    let mut out = replace_date_format_vars(content, title);
    out = out.replace("{{title}}", title);
    out = out.replace("{{date}}", title);
    out = out.replace("{{time}}", time);
    out
}
