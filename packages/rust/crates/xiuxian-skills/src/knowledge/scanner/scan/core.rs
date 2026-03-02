use std::path::{Path, PathBuf};

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::knowledge::types::KnowledgeEntry;

use super::super::KnowledgeScanner;

impl KnowledgeScanner {
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

        let markdown_files: Vec<PathBuf> = WalkDir::new(base_path)
            .follow_links(false)
            .max_depth(walk_depth)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension == "md")
            })
            .map(|entry| entry.path().to_path_buf())
            .collect();

        let entries: Vec<KnowledgeEntry> = markdown_files
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
