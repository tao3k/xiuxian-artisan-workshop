pub(super) fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn normalize_unit_f32(value: f32, source: &str) -> Option<f32> {
    if (0.0..=1.0).contains(&value) {
        return Some(value);
    }
    tracing::warn!(
        source,
        value,
        "invalid memory gate unit value (expected 0.0..=1.0); keeping previous/default"
    );
    None
}
