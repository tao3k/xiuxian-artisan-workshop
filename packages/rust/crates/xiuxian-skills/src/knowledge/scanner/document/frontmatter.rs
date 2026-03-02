use std::path::Path;

use super::super::metadata::KnowledgeFrontmatter;
use crate::frontmatter::split_frontmatter;

pub(super) fn parse_metadata_and_content(content: &str) -> Option<(KnowledgeFrontmatter, String)> {
    match split_frontmatter(content) {
        Some(parts) => {
            let metadata: KnowledgeFrontmatter = serde_yaml::from_str(parts.yaml).ok()?;
            Some((metadata, parts.body.to_string()))
        }
        None => Some((KnowledgeFrontmatter::default(), content.to_string())),
    }
}

pub(super) fn title_from_metadata_or_path(
    metadata: &KnowledgeFrontmatter,
    path: &Path,
    relative_path: &Path,
) -> String {
    metadata.title.clone().unwrap_or_else(|| {
        path.file_stem().and_then(|s| s.to_str()).map_or_else(
            || relative_path.to_string_lossy().into_owned(),
            |s| s.replace(['-', '_'], " "),
        )
    })
}
