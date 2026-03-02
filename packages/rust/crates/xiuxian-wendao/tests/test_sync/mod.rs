//! Integration tests for Rust `SyncEngine`

use std::fs;
use tempfile::TempDir;

mod batch_diff_computation;
mod compute_diff;
mod compute_hash;
mod custom_discovery_options;
mod deleted_files_detection;
mod discover_files;
/// Test `SyncEngine` manifest load/save operations.
mod manifest_load_save;
mod skip_hidden_and_directories;
