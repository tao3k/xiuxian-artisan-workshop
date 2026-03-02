use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;

use crate::skills::metadata::{SkillMetadata, SkillStructure};

use super::super::SkillScanner;

impl SkillScanner {
    /// Scan all skills in a base directory with parallel processing.
    ///
    /// Returns a vector of skill metadata for all skills with valid SKILL.md.
    /// Skills without SKILL.md are silently skipped.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Path to the skills directory (e.g., "assets/skills")
    /// * `structure` - Optional skill structure for validation (uses default if None)
    ///
    /// # Errors
    ///
    /// Returns an error if the skills directory cannot be read.
    pub fn scan_all(
        &self,
        base_path: &Path,
        structure: Option<&SkillStructure>,
    ) -> Result<Vec<SkillMetadata>, Box<dyn std::error::Error>> {
        if !base_path.exists() {
            log::warn!("Skills base directory not found: {}", base_path.display());
            return Ok(Vec::new());
        }

        let skill_dirs: Vec<PathBuf> = fs::read_dir(base_path)?
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.path())
            .collect();

        let effective_structure = Arc::new(
            structure
                .cloned()
                .unwrap_or_else(SkillScanner::default_structure),
        );
        let scan_results: Vec<Result<Option<SkillMetadata>, String>> = skill_dirs
            .par_iter()
            .map(|skill_path| {
                self.scan_skill(skill_path, Some(effective_structure.as_ref()))
                    .map_err(|error| format!("{}: {error}", skill_path.display()))
            })
            .collect();
        let mut metadatas = Vec::new();
        let mut errors = Vec::new();
        for result in scan_results {
            match result {
                Ok(Some(metadata)) => metadatas.push(metadata),
                Ok(None) => {}
                Err(error) => errors.push(error),
            }
        }
        if !errors.is_empty() {
            return Err(anyhow::anyhow!(
                "skill scanning failed with {} error(s): {}",
                errors.len(),
                errors.join(" | ")
            )
            .into());
        }

        log::info!(
            "Scanned {} skills from {}",
            metadatas.len(),
            base_path.display()
        );

        Ok(metadatas)
    }
}
