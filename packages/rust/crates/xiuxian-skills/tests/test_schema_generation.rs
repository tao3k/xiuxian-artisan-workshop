//! Tests for JSON Schema generation.
//!
//! Verifies that schema generation produces valid JSON and
//! writes to the correct location.

use std::fs;
use std::path::PathBuf;
use xiuxian_skills::SkillMetadata;

/// Test that `skill_index_schema` produces valid JSON.
#[test]
fn test_skill_index_schema_produces_json() -> Result<(), Box<dyn std::error::Error>> {
    let schema_json = xiuxian_skills::skill_index_schema();
    assert!(!schema_json.is_empty());

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&schema_json)?;

    // Verify it has expected fields
    assert_eq!(parsed["title"], "SkillIndexEntry");
    assert_eq!(
        parsed["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );

    Ok(())
}

/// Test that `SkillMetadata` derives `JsonSchema`.
#[test]
fn test_skill_metadata_schema_derives() -> Result<(), Box<dyn std::error::Error>> {
    // This test verifies that SkillMetadata can be used with schemars
    let schema = schemars::schema_for!(SkillMetadata);
    let schema_json = serde_json::to_string_pretty(&schema)?;

    assert!(!schema_json.is_empty());
    assert!(schema_json.contains("SkillMetadata"));

    Ok(())
}

/// Generate JSON Schema for `SkillMetadata` and write to crate resources.
///
/// Run with: `cargo test -p xiuxian-skills generate_skill_metadata_schema`
#[test]
fn generate_skill_metadata_schema() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Generate Schema object for SkillMetadata
    let schema = schemars::schema_for!(SkillMetadata);

    // 2. Serialize to JSON string
    let schema_json = serde_json::to_string_pretty(&schema)?;

    // 3. Resolve output path using PRJ_ROOT or CARGO_MANIFEST_DIR
    let output_path = resolve_output_path("skill_metadata.schema.json");

    // 4. Create parent directory if needed and write
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, schema_json)?;

    let canonical_output = match output_path.canonicalize() {
        Ok(path) => path,
        Err(_) => output_path,
    };

    println!("Schema generated at: {canonical_output:?}");

    Ok(())
}

/// Resolve output path for schema files.
///
/// Uses `PRJ_ROOT` environment variable if set, otherwise falls back to
/// `CARGO_MANIFEST_DIR` + "../resources/"
fn resolve_output_path(filename: &str) -> PathBuf {
    // Try PRJ_ROOT first
    if let Ok(prj_root) = std::env::var("PRJ_ROOT") {
        let path = PathBuf::from(prj_root)
            .join("packages/rust/crates/xiuxian-skills/resources")
            .join(filename);
        if path.parent().is_some_and(std::path::Path::exists) {
            return path;
        }
    }

    // Fall back to CARGO_MANIFEST_DIR
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").map_or_else(|_| PathBuf::from("."), PathBuf::from);

    manifest_dir.join("../resources").join(filename)
}
