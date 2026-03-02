pub(in crate::skill_vfs::zhixing::indexer) fn normalize_identity_token(raw: &str) -> String {
    let normalized = raw
        .trim()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_ascii_lowercase();
    if normalized.is_empty() {
        "unknown".to_string()
    } else {
        normalized
    }
}
