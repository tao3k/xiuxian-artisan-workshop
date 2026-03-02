use std::fs;
use std::path::Path;

use super::KnowledgeScanner;
use crate::knowledge::types::{KnowledgeCategory, KnowledgeEntry};

mod frontmatter;
mod identity;

use frontmatter::{parse_metadata_and_content, title_from_metadata_or_path};
use identity::{content_sha256, knowledge_id_from_relative_path};

impl KnowledgeScanner {
    /// Scan a single knowledge document.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the markdown file
    /// * `base_path` - Base path for calculating relative paths
    ///
    /// # Returns
    ///
    /// `Some(entry)` if the document is valid, otherwise `None`.
    #[must_use]
    pub fn scan_document(path: &Path, base_path: &Path) -> Option<KnowledgeEntry> {
        if !path.exists() || !path.is_file() {
            return None;
        }

        if path.extension().is_none_or(|ext| ext != "md") {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        let relative_path = path.strip_prefix(base_path).unwrap_or(path);
        let file_hash = content_sha256(&content);
        let id = knowledge_id_from_relative_path(relative_path);

        let (metadata, content_without_frontmatter) = parse_metadata_and_content(&content)?;
        let title = title_from_metadata_or_path(&metadata, path, relative_path);

        let category = metadata
            .category
            .as_ref()
            .and_then(|c| c.parse().ok())
            .unwrap_or(KnowledgeCategory::Unknown);

        let content_preview = content_without_frontmatter
            .lines()
            .take(10)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .chars()
            .take(500)
            .collect();

        Some(KnowledgeEntry {
            id,
            file_path: relative_path.to_string_lossy().into_owned(),
            title,
            description: metadata.description.unwrap_or_default(),
            category,
            tags: metadata.tags.unwrap_or_default(),
            authors: metadata.authors.unwrap_or_default(),
            source: metadata.source,
            version: metadata.version.unwrap_or_default(),
            file_hash,
            content_preview,
        })
    }
}
