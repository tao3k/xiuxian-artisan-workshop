use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::KnowledgeScanner;
use super::metadata::KnowledgeFrontmatter;
use crate::frontmatter::extract_frontmatter;
use crate::knowledge::types::{KnowledgeCategory, KnowledgeEntry};

fn content_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn knowledge_id_from_relative_path(relative_path: &Path) -> String {
    let path_str = relative_path.to_string_lossy();
    let mut path_hasher = Sha256::new();
    path_hasher.update(path_str.as_bytes());
    let hash_result = path_hasher.finalize();
    hex::encode(&hash_result[..16])
}

fn parse_metadata_and_content(content: &str) -> Option<(KnowledgeFrontmatter, String)> {
    match extract_frontmatter(content) {
        Some(fm) => {
            let metadata: KnowledgeFrontmatter = serde_yaml::from_str(&fm).ok()?;
            let content_after = content[fm.len() + 6..].to_string(); // Remove frontmatter
            Some((metadata, content_after))
        }
        None => Some((KnowledgeFrontmatter::default(), content.to_string())),
    }
}

fn title_from_metadata_or_path(
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

        // Only process markdown files
        if path.extension().is_none_or(|ext| ext != "md") {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        let relative_path = path.strip_prefix(base_path).unwrap_or(path);
        let file_hash = content_sha256(&content);
        let id = knowledge_id_from_relative_path(relative_path);

        let (metadata, content_without_frontmatter) = parse_metadata_and_content(&content)?;
        let title = title_from_metadata_or_path(&metadata, path, relative_path);

        // Parse category
        let category = metadata
            .category
            .as_ref()
            .and_then(|c| c.parse().ok())
            .unwrap_or(KnowledgeCategory::Unknown);

        // Generate content preview (first 500 chars)
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
