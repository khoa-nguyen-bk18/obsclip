pub fn compose_payload(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::compose_payload;

    #[test]
    fn empty_and_whitespace_are_none() {
        assert_eq!(compose_payload(""), None);
        assert_eq!(compose_payload("   \n\t  "), None);
    }

    #[test]
    fn trims_and_keeps_inner_newlines() {
        assert_eq!(
            compose_payload("  hello\nworld  \n"),
            Some("hello\nworld".to_string())
        );
    }
}
