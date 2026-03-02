use std::collections::HashMap;
use std::path::Path;

use xxhash_rust::xxh3;

use super::{SyncEngine, SyncManifest};

impl SyncEngine {
    /// Load manifest from disk.
    #[must_use]
    pub fn load_manifest(&self) -> SyncManifest {
        if self.manifest_path.exists()
            && let Ok(content) = std::fs::read_to_string(&self.manifest_path)
            && let Ok(manifest) = serde_json::from_str(&content)
        {
            return SyncManifest(manifest);
        }
        SyncManifest(HashMap::new())
    }

    /// Save manifest to disk.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] if parent directory creation, manifest serialization,
    /// or file writing fails.
    pub fn save_manifest(&self, manifest: &SyncManifest) -> std::io::Result<()> {
        if let Some(parent) = self.manifest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&manifest.0)?;
        std::fs::write(&self.manifest_path, content)
    }

    /// Compute content hash using xxhash (fast).
    #[must_use]
    pub fn compute_hash(content: &str) -> String {
        format!("{:016x}", xxh3::xxh3_64(content.as_bytes()))
    }

    /// Compute hash for file content.
    #[must_use]
    pub fn compute_file_hash(path: &Path) -> Option<String> {
        std::fs::read_to_string(path)
            .ok()
            .map(|c| Self::compute_hash(&c))
    }
}
