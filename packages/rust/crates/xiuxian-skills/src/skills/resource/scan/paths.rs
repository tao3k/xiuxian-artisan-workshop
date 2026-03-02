use sha2::{Digest, Sha256};

use super::super::ResourceScanner;
use super::build::build_resource_records;
use crate::skills::metadata::ResourceRecord;

impl ResourceScanner {
    /// Scan multiple files for @`skill_resource` decorated functions.
    ///
    /// Used for testing.
    ///
    /// # Errors
    ///
    /// Returns an error when `skill_name` is empty.
    pub fn scan_paths(
        &self,
        files: &[(String, String)],
        skill_name: &str,
    ) -> Result<Vec<ResourceRecord>, Box<dyn std::error::Error>> {
        let _ = self;
        if skill_name.trim().is_empty() {
            return Err("skill_name cannot be empty".into());
        }

        let mut all_resources = Vec::new();
        for (file_path, content) in files {
            let file_hash = hex::encode(Sha256::digest(content.as_bytes()));
            let resources = build_resource_records(content, file_path, skill_name, &file_hash);
            all_resources.extend(resources);
        }

        Ok(all_resources)
    }
}
