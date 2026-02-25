#![allow(clippy::expect_used, clippy::uninlined_format_args)]

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::files::{find_files, find_rs_files};
use super::{DependencyConfig, DependencyIndexer};

#[test]
fn test_indexer_creation() {
    let indexer = DependencyIndexer::new(".", None);
    assert_eq!(indexer.stats().total_crates, 0);
}

#[test]
fn test_config_default() {
    let config = DependencyConfig::default();
    assert_eq!(config.project_root, ".");
}

#[test]
fn test_find_files() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let temp_path = temp_dir.path();

    // Create a Cargo.toml
    let cargo_path = temp_dir.path().join("Cargo.toml");
    let mut file = File::create(&cargo_path).expect("create cargo");
    writeln!(file, "[package]\nname = \"test\"").expect("write");

    // Create nested Cargo.toml
    let nested_dir = temp_dir.path().join("crates");
    std::fs::create_dir(&nested_dir).expect("create nested dir");
    let nested_cargo = nested_dir.join("Cargo.toml");
    let mut file = File::create(&nested_cargo).expect("create nested cargo");
    writeln!(file, "[package]\nname = \"nested\"").expect("write");

    let pattern = "**/Cargo.toml";
    let files = find_files(pattern, &PathBuf::from(temp_path));

    // Should find 2 Cargo.toml files
    assert_eq!(files.len(), 2);
}

#[test]
fn test_find_rs_files() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Create test Rust files
    let _ = File::create(temp_dir.path().join("lib.rs"));
    let _ = File::create(temp_dir.path().join("main.rs"));
    let _ = File::create(temp_dir.path().join("not_rust.txt"));

    // Create subdirectory
    let sub_dir = temp_dir.path().join("src");
    std::fs::create_dir(&sub_dir).expect("create src dir");
    let _ = File::create(sub_dir.join("module.rs"));

    let files = find_rs_files(&temp_path);

    // Should find 3 .rs files
    assert_eq!(files.len(), 3);
}

#[test]
fn test_build_performance() {
    use std::time::Instant;

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap_or_else(|| panic!("failed to resolve workspace root from CARGO_MANIFEST_DIR"))
        .to_path_buf();
    let project_root_str = project_root.to_string_lossy().to_string();
    let config_path = project_root.join("packages/conf/references.yaml");

    let mut indexer = DependencyIndexer::new(
        &project_root_str,
        Some(config_path.to_string_lossy().as_ref()),
    );

    let start = Instant::now();
    let result = indexer.build(false);
    let elapsed = start.elapsed();

    // Performance assertions
    // With parallel processing and pre-compiled regex:
    // - Should process all manifests in under 2 seconds locally, or 4 seconds on CI
    // - Should index at least 10 crates
    // - Should extract at least 100 symbols
    let max_duration = if std::env::var_os("CI").is_some() {
        std::time::Duration::from_secs(4)
    } else {
        std::time::Duration::from_secs(2)
    };
    assert!(
        elapsed < max_duration,
        "Build should complete in under {:.2}s, took: {:?}",
        max_duration.as_secs_f64(),
        elapsed,
    );
    assert!(
        result.crates_indexed >= 10,
        "Should index at least 10 crates, got: {}",
        result.crates_indexed
    );
    assert!(
        result.total_symbols >= 100,
        "Should extract at least 100 symbols, got: {}",
        result.total_symbols
    );

    println!(
        "Build performance: {:.2}s for {} crates, {} symbols",
        elapsed.as_secs_f64(),
        result.crates_indexed,
        result.total_symbols
    );
}
