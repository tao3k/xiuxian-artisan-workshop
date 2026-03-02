//! Debug test for dependency parsing.
//!
//! Note: These tests verify the integration points. Full parsing/fetching
//! is implemented incrementally. See `indexer.rs` `build()` for progress.

use std::fs;
use tempfile::TempDir;
use xiuxian_wendao::DependencyIndexer;

/// Test that the dependency indexer can be created with custom config
#[test]
fn test_indexer_creation_with_config() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let temp_root = temp_dir.path().to_string_lossy().into_owned();

    // Create a custom xiuxian.toml with manifest pattern
    let config_path = format!("{temp_root}/xiuxian.toml");
    let config_content = r#"
[[ast_symbols_external]]
type = "rust"
manifests = ["**/Cargo.toml"]
"#;
    fs::write(&config_path, config_content)?;

    // Create indexer with config - should not panic
    let indexer = DependencyIndexer::new(&temp_root, Some(&config_path));

    // Verify indexer is created correctly
    let crates = indexer.get_indexed();
    assert!(
        crates.is_empty(),
        "New indexer should have no indexed crates"
    );
    Ok(())
}

/// Test that build returns a valid result structure
#[test]
fn test_build_returns_valid_structure() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let temp_root = temp_dir.path().to_string_lossy().into_owned();

    // Create a minimal Cargo.toml
    let cargo_content = r#"[package]
name = "test-project"
version = "0.1.0"

[dependencies]
anyhow = "1.0.100"
"#;
    let cargo_path = format!("{temp_root}/Cargo.toml");
    fs::write(&cargo_path, cargo_content)?;

    // Provide explicit config so this test is independent from workspace defaults
    let config_path = format!("{temp_root}/xiuxian.toml");
    let config_content = r#"
[[ast_symbols_external]]
type = "rust"
manifests = ["**/Cargo.toml"]
"#;
    fs::write(&config_path, config_content)?;

    // Create indexer
    let mut indexer = DependencyIndexer::new(&temp_root, Some(&config_path));

    // Build should return a valid result (placeholder returns zeros)
    let result = indexer.build(true);

    assert!(
        result.files_processed >= 1,
        "Build should process at least the fixture Cargo.toml"
    );
    assert_eq!(result.errors, 0);
    assert!(
        result.crates_indexed >= 1,
        "Build should index at least one crate from fixture"
    );
    assert_eq!(
        result.total_symbols, 0,
        "No source files in fixture means no extracted symbols"
    );
    Ok(())
}

/// Test that search methods work on empty index
#[test]
fn test_search_on_empty_index() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let temp_root = temp_dir.path().to_string_lossy().into_owned();

    let indexer = DependencyIndexer::new(&temp_root, None);

    // Search should return empty on new index
    let results = indexer.search("anyhow", 10);
    assert!(
        results.is_empty(),
        "Search on empty index should return empty"
    );

    let crate_results = indexer.search_crate("anyhow", "Error", 10);
    assert!(
        crate_results.is_empty(),
        "Crate search on empty index should return empty"
    );
    Ok(())
}

/// Test config loading from file
#[test]
fn test_config_loading() -> Result<(), Box<dyn std::error::Error>> {
    use xiuxian_wendao::DependencyBuildConfig;

    let temp_dir = TempDir::new()?;
    let temp_root = temp_dir.path().to_string_lossy().into_owned();

    // Create a custom xiuxian.toml
    let config_path = format!("{temp_root}/xiuxian.toml");
    let config_content = r#"
[[ast_symbols_external]]
type = "rust"
registry = "cargo"
manifests = ["**/Cargo.toml"]

[[ast_symbols_external]]
type = "python"
registry = "pip"
manifests = ["**/pyproject.toml"]
"#;
    fs::write(&config_path, config_content)?;

    // Load config
    let config = DependencyBuildConfig::load(&config_path);

    // Should have loaded manifests
    assert!(!config.manifests.is_empty(), "Config should have manifests");

    let rust_dep = config.manifests.iter().find(|d| d.pkg_type == "rust");
    let Some(rust_dep) = rust_dep else {
        panic!("Should have rust dependency config");
    };
    assert_eq!(rust_dep.registry, Some("cargo".to_string()));
    Ok(())
}
