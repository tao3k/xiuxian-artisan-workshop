use std::collections::HashMap;
use std::path::PathBuf;

use super::{SyncEngine, SyncManifest, SyncResult};

impl SyncEngine {
    /// Compute sync diff: added, modified, deleted files.
    #[must_use]
    pub fn compute_diff(&self, manifest: &SyncManifest, files: &[PathBuf]) -> SyncResult {
        let mut result = SyncResult::default();

        // Build current files map (relative -> absolute)
        let current_files: HashMap<String, PathBuf> = files
            .iter()
            .filter_map(|f| {
                f.strip_prefix(&self.project_root)
                    .ok()
                    .map(|p| (p.to_string_lossy().to_string(), f.clone()))
            })
            .collect();

        // Check for added/modified files
        for (rel_path, abs_path) in &current_files {
            if let Ok(content) = std::fs::read_to_string(abs_path) {
                let hash = Self::compute_hash(&content);

                match manifest.0.get(rel_path) {
                    Some(existing_hash) => {
                        if &hash == existing_hash {
                            result.unchanged += 1;
                        } else {
                            result.modified.push(abs_path.clone());
                        }
                    }
                    None => {
                        result.added.push(abs_path.clone());
                    }
                }
            }
        }

        // Check for deleted files
        for rel_path in manifest.0.keys() {
            if !current_files.contains_key(rel_path) {
                result.deleted.push(self.project_root.join(rel_path));
            }
        }

        result
    }
}
