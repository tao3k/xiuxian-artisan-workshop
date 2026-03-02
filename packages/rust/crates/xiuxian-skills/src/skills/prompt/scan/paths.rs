use sha2::{Digest, Sha256};

use super::super::PromptScanner;
use super::build::build_prompt_records;
use crate::skills::metadata::PromptRecord;

impl PromptScanner {
    /// Scan multiple files for @prompt decorated functions.
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
    ) -> Result<Vec<PromptRecord>, Box<dyn std::error::Error>> {
        let _ = self;
        if skill_name.trim().is_empty() {
            return Err("skill_name cannot be empty".into());
        }

        let mut all_prompts = Vec::new();
        for (file_path, content) in files {
            let file_hash = hex::encode(Sha256::digest(content.as_bytes()));
            let prompts = build_prompt_records(content, file_path, skill_name, &file_hash);
            all_prompts.extend(prompts);
        }

        Ok(all_prompts)
    }
}
