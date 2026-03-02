use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::super::PromptScanner;
use super::build::build_prompt_records;
use crate::skills::metadata::PromptRecord;

impl PromptScanner {
    /// Scan a scripts directory for @prompt decorated functions.
    ///
    /// # Arguments
    ///
    /// * `scripts_dir` - Path to the scripts directory
    /// * `skill_name` - Name of the parent skill
    ///
    /// # Returns
    ///
    /// A vector of `PromptRecord` objects.
    ///
    /// # Errors
    ///
    /// Returns an error when `skill_name` is empty.
    pub fn scan(
        &self,
        scripts_dir: &Path,
        skill_name: &str,
    ) -> Result<Vec<PromptRecord>, Box<dyn std::error::Error>> {
        let _ = self;
        if skill_name.trim().is_empty() {
            return Err("skill_name cannot be empty".into());
        }

        let mut prompts = Vec::new();
        if !scripts_dir.exists() {
            log::debug!("Scripts directory not found: {}", scripts_dir.display());
            return Ok(prompts);
        }

        for entry in WalkDir::new(scripts_dir)
            .follow_links(true)
            .sort_by_file_name()
        {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    log::warn!("Error walking directory {}: {error}", scripts_dir.display());
                    continue;
                }
            };

            let path = entry.path();
            if !entry.file_type().is_file() {
                continue;
            }
            if path.extension().map(|ext| ext.to_string_lossy()) != Some("py".into()) {
                continue;
            }
            if path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("__"))
            {
                continue;
            }

            match Self::scan_file(path, skill_name) {
                Ok(file_prompts) => prompts.extend(file_prompts),
                Err(error) => log::warn!("Error scanning {}: {error}", path.display()),
            }
        }

        log::debug!(
            "PromptScanner: Found {} @prompt functions in {}",
            prompts.len(),
            scripts_dir.display()
        );

        Ok(prompts)
    }

    /// Scan a single file for @prompt decorated functions.
    fn scan_file(
        path: &Path,
        skill_name: &str,
    ) -> Result<Vec<PromptRecord>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let file_hash = hex::encode(Sha256::digest(content.as_bytes()));
        let file_path = path.to_string_lossy().to_string();
        Ok(build_prompt_records(
            &content, &file_path, skill_name, &file_hash,
        ))
    }
}
