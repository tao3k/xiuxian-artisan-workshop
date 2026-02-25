pub(super) fn normalize_command_input(input: &str) -> &str {
    let mut normalized = input.trim();
    if normalized.starts_with('[')
        && let Some(end) = normalized.find(']')
    {
        let tag = &normalized[1..end];
        if tag.to_ascii_lowercase().starts_with("bbx-") {
            normalized = normalized[end + 1..].trim_start();
        }
    }
    normalized.trim_start_matches('/')
}
