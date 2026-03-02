use std::collections::HashMap;
use std::path::Path;

use crate::knowledge::types::{KnowledgeCategory, KnowledgeEntry};

use super::super::KnowledgeScanner;

impl KnowledgeScanner {
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
            .filter(|entry| entry.category == target_category)
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
            .filter(|entry| entry.tags.iter().any(|tag| tags.contains(tag)))
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
        tags.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        Ok(tags)
    }
}
