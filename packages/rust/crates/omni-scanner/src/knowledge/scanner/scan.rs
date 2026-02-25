use std::collections::HashMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::KnowledgeScanner;
use crate::knowledge::types::{KnowledgeCategory, KnowledgeEntry};

impl KnowledgeScanner {
    /// Create a new knowledge scanner with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Scan a knowledge directory for all documents with parallel processing.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Path to the knowledge directory
    /// * `depth` - Maximum directory depth (-1 for unlimited)
    ///
    /// # Returns
    ///
    /// Vector of discovered knowledge entries.
    ///
    /// # Errors
    ///
    /// Returns an error when:
    /// - `depth` is less than `-1`.
    /// - Directory traversal fails while reading entries.
    pub fn scan_all(
        &self,
        base_path: &Path,
        depth: Option<i32>,
    ) -> Result<Vec<KnowledgeEntry>, Box<dyn std::error::Error>> {
        use rayon::prelude::*;
        let _ = self;

        if !base_path.exists() {
            log::warn!(
                "Knowledge base directory not found: {}",
                base_path.display()
            );
            return Ok(Vec::new());
        }

        let max_depth = depth.unwrap_or(-1);
        let walk_depth = walk_depth_from(max_depth)?;

        // Collect all markdown files first
        let md_files: Vec<PathBuf> = WalkDir::new(base_path)
            .follow_links(false)
            .max_depth(walk_depth)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|e| {
                e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "md")
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        // Process in parallel using rayon
        let entries: Vec<KnowledgeEntry> = md_files
            .par_iter()
            .filter_map(|path| Self::scan_document(path, base_path))
            .collect();

        log::info!(
            "Scanned {} knowledge documents from {}",
            entries.len(),
            base_path.display()
        );

        Ok(entries)
    }

    /// Scan and filter knowledge by category.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Path to the knowledge directory
    /// * `category` - Category to filter by
    ///
    /// # Returns
    ///
    /// Vector of knowledge entries matching the category.
    ///
    /// # Errors
    ///
    /// Returns an error when underlying directory scanning fails.
    pub fn scan_category(
        &self,
        base_path: &Path,
        category: &str,
    ) -> Result<Vec<KnowledgeEntry>, Box<dyn std::error::Error>> {
        let target_category: KnowledgeCategory =
            category.parse().unwrap_or(KnowledgeCategory::Unknown);
        let all_entries = self.scan_all(base_path, None)?;

        Ok(all_entries
            .into_iter()
            .filter(|e| e.category == target_category)
            .collect())
    }

    /// Scan and filter knowledge by tags.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Path to the knowledge directory
    /// * `tags` - Tags to filter by (entries matching ANY tag)
    ///
    /// # Returns
    ///
    /// Vector of knowledge entries matching any of the tags.
    ///
    /// # Errors
    ///
    /// Returns an error when underlying directory scanning fails.
    pub fn scan_with_tags(
        &self,
        base_path: &Path,
        tags: &[String],
    ) -> Result<Vec<KnowledgeEntry>, Box<dyn std::error::Error>> {
        let all_entries = self.scan_all(base_path, None)?;

        if tags.is_empty() {
            return Ok(all_entries);
        }

        Ok(all_entries
            .into_iter()
            .filter(|e| e.tags.iter().any(|t| tags.contains(t)))
            .collect())
    }

    /// Get all unique tags from a knowledge directory.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Path to the knowledge directory
    ///
    /// # Returns
    ///
    /// Vector of unique tags with their counts.
    ///
    /// # Errors
    ///
    /// Returns an error when underlying directory scanning fails.
    pub fn get_tags(
        &self,
        base_path: &Path,
    ) -> Result<Vec<(String, usize)>, Box<dyn std::error::Error>> {
        let entries = self.scan_all(base_path, None)?;

        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        for entry in entries {
            for tag in &entry.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        let mut tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        tags.sort_by_key(|entry| std::cmp::Reverse(entry.1)); // Sort by count descending

        Ok(tags)
    }
}

fn walk_depth_from(depth: i32) -> Result<usize, Box<dyn std::error::Error>> {
    if depth == -1 {
        return Ok(usize::MAX);
    }
    if depth < -1 {
        return Err(format!("depth must be -1 or >= 0, got {depth}").into());
    }

    usize::try_from(depth)?
        .checked_add(1)
        .ok_or_else(|| format!("depth overflow for walkdir: {depth}").into())
}

impl Default for KnowledgeScanner {
    fn default() -> Self {
        Self::new()
    }
}
