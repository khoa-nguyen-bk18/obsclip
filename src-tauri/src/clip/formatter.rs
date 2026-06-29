use crate::config::TextFormat;

pub fn format_text(format: TextFormat, text: &str, time: &str) -> String {
    match format {
        TextFormat::Timestamped => {
            let lines: Vec<&str> = text.lines().collect();
            if lines.len() <= 1 {
                format!("- {time} — {text}")
            } else {
                let mut out = format!("- {time} — {}", lines[0]);
                for line in &lines[1..] {
                    out.push_str(&format!("\n  {line}"));
                }
                out
            }
        }
        TextFormat::Blockquote => text
            .lines()
            .map(|l| format!("> {l}"))
            .collect::<Vec<_>>()
            .join("\n"),
        TextFormat::Bullet => text
            .lines()
            .map(|l| format!("- {l}"))
            .collect::<Vec<_>>()
            .join("\n"),
        TextFormat::Checkbox => text
            .lines()
            .map(|l| format!("- [ ] {l}"))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

pub fn format_image_link(time: &str, filename: &str) -> String {
    format!("- {time} — ![[{filename}]]")
}
