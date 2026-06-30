use crate::config::TextFormat;

pub fn format_text(format: TextFormat, text: &str, time: &str) -> String {
    format_text_with_annotation(format, text, time, None)
}

pub fn format_text_with_annotation(
    format: TextFormat,
    text: &str,
    time: &str,
    annotation: Option<&str>,
) -> String {
    let block = match format {
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
    };

    apply_annotation(&block, format, annotation)
}

pub fn format_image_link(time: &str, filename: &str) -> String {
    format_image_link_with_annotation(time, filename, None)
}

pub fn format_image_link_with_annotation(
    time: &str,
    filename: &str,
    annotation: Option<&str>,
) -> String {
    let block = format!("- {time} — ![[{filename}]]");
    apply_annotation(&block, TextFormat::Timestamped, annotation)
}

fn apply_annotation(block: &str, format: TextFormat, annotation: Option<&str>) -> String {
    let Some(annotation) = annotation.filter(|text| !text.is_empty()) else {
        return block.to_string();
    };

    match format {
        TextFormat::Timestamped => append_to_first_line(block, annotation),
        TextFormat::Blockquote => format!("{block}\n> {annotation}"),
        TextFormat::Bullet => format!("{block}\n- {annotation}"),
        TextFormat::Checkbox => format!("{block}\n- [ ] {annotation}"),
    }
}

fn append_to_first_line(block: &str, suffix: &str) -> String {
    match block.split_once('\n') {
        Some((first, rest)) => format!("{first} — {suffix}\n{rest}"),
        None => format!("{block} — {suffix}"),
    }
}
