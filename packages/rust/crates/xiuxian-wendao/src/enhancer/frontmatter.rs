use crate::enhancer::types::NoteFrontmatter;
use xiuxian_skills::parse_frontmatter_from_markdown;

/// Parse frontmatter from markdown content.
#[must_use]
pub fn parse_frontmatter(content: &str) -> NoteFrontmatter {
    let Ok(Some(value)) = parse_frontmatter_from_markdown(content) else {
        return NoteFrontmatter::default();
    };

    let Some(mapping) = value.as_mapping() else {
        return NoteFrontmatter::default();
    };

    let get_str = |key: &str| -> Option<String> {
        mapping
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_str())
            .map(str::to_string)
    };

    let get_str_vec = |key: &str| -> Vec<String> {
        mapping
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };

    // Check nested metadata block
    let metadata = mapping
        .get(serde_yaml::Value::String("metadata".to_string()))
        .and_then(|v| v.as_mapping());

    let get_metadata_vec = |key: &str| -> Vec<String> {
        metadata
            .and_then(|m| m.get(serde_yaml::Value::String(key.to_string())))
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };

    let mut tags = get_str_vec("tags");
    if tags.is_empty() {
        tags = get_metadata_vec("tags");
    }

    NoteFrontmatter {
        title: get_str("title"),
        description: get_str("description"),
        name: get_str("name"),
        category: get_str("category"),
        tags,
        routing_keywords: get_metadata_vec("routing_keywords"),
        intents: get_metadata_vec("intents"),
    }
}
