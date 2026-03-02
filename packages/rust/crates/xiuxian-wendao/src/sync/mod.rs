//! Sync Engine - Incremental Knowledge Synchronization.
//!
//! Provides efficient file discovery, manifest management, and diff computation
//! for incremental knowledge base updates.

mod diff;
mod discovery;
mod incremental;
mod manifest;
mod types;

use std::path::{Path, PathBuf};

pub use incremental::{
    IncrementalSyncPolicy, extension_from_path, extract_extensions_from_glob_patterns,
    normalize_extension,
};
pub use types::{DiscoveryOptions, FileChange, SyncManifest, SyncResult};

/// Sync engine for incremental knowledge sync.
#[derive(Debug, Clone)]
pub struct SyncEngine {
    pub(super) project_root: PathBuf,
    pub(super) manifest_path: PathBuf,
    pub(super) options: DiscoveryOptions,
}

impl SyncEngine {
    /// Create a new sync engine.
    pub fn new<P: AsRef<Path>>(project_root: P, manifest_path: P) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
            manifest_path: manifest_path.as_ref().to_path_buf(),
            options: DiscoveryOptions::default(),
        }
    }

    /// Set discovery options.
    #[must_use]
    pub fn with_options(mut self, options: DiscoveryOptions) -> Self {
        self.options = options;
        self
    }

    /// Get project root.
    #[must_use]
    pub fn project_root(&self) -> &PathBuf {
        &self.project_root
    }

    /// Get manifest path.
    #[must_use]
    pub fn manifest_path(&self) -> &PathBuf {
        &self.manifest_path
    }
}
