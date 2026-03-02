use std::collections::HashMap;

pub(super) fn split_title_and_metadata_fields(raw: &str) -> (&str, HashMap<String, String>) {
    let Some((title, comment)) = raw.split_once("<!--") else {
        return (raw, HashMap::new());
    };
    let comment = comment
        .split_once("-->")
        .map_or(comment, |(prefix, _)| prefix)
        .trim();
    (title, parse_metadata_fields(comment))
}

fn parse_metadata_fields(comment: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    for chunk in comment.split(',') {
        let fragment = chunk.trim();
        if fragment.is_empty() {
            continue;
        }
        let parsed = fragment.split_once(": ").or_else(|| {
            (fragment.matches(':').count() == 1)
                .then(|| fragment.split_once(':'))
                .flatten()
        });
        let Some((key, value)) = parsed else {
            continue;
        };
        let normalized_key = key.trim();
        let normalized_value = value.trim();
        if normalized_key.is_empty() || normalized_value.is_empty() {
            continue;
        }
        fields.insert(normalized_key.to_string(), normalized_value.to_string());
    }
    fields
}
