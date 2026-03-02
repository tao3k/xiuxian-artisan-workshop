//! XML-Lite parsing helpers shared across native tool and workflow runtimes.

/// Extract text content of the first `<tag>...</tag>` block.
///
/// The search is case-sensitive and returns trimmed content.
#[must_use]
pub fn extract_tag_value<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)?;
    let content_start = start + open.len();
    let content_end = text[content_start..].find(&close)? + content_start;
    Some(text[content_start..content_end].trim())
}

/// Parse the first `<tag>...</tag>` block as `f32`.
#[must_use]
pub fn extract_tag_f32(text: &str, tag: &str) -> Option<f32> {
    extract_tag_value(text, tag)?.parse::<f32>().ok()
}
