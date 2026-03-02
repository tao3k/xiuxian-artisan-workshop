//! Integration tests for `omni-io` assembler.

#![cfg(feature = "assembler")]

use std::fs;

use omni_io::ContextAssembler;
use tempfile::TempDir;

#[test]
fn test_assemble_skill_basic() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("operation should succeed: {error}"));
    let main_path = temp_dir.path().join("SKILL.md");
    fs::write(&main_path, "Hello {{ name }}!")
        .unwrap_or_else(|error| panic!("operation should succeed: {error}"));

    let assembler = ContextAssembler::new();
    let variables = serde_json::json!({"name": "World"});

    let result = assembler
        .assemble_skill(main_path, Vec::new(), variables)
        .unwrap_or_else(|error| panic!("operation should succeed: {error}"));

    assert!(result.content.contains("Hello World!"));
    assert!(result.token_count > 0);
    assert!(result.missing_refs.is_empty());
}

#[test]
fn test_assemble_skill_with_references() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("operation should succeed: {error}"));
    let main_path = temp_dir.path().join("SKILL.md");
    fs::write(&main_path, "Main: {{ var1 }}")
        .unwrap_or_else(|error| panic!("operation should succeed: {error}"));

    let ref_path = temp_dir.path().join("ref.md");
    fs::write(&ref_path, "Reference content")
        .unwrap_or_else(|error| panic!("operation should succeed: {error}"));

    let assembler = ContextAssembler::new();
    let variables = serde_json::json!({"var1": "test"});

    let result = assembler
        .assemble_skill(main_path, vec![ref_path], variables)
        .unwrap_or_else(|error| panic!("operation should succeed: {error}"));

    assert!(result.content.contains("Main: test"));
    assert!(result.content.contains("Reference content"));
    assert!(result.content.contains("# Required References"));
}

#[test]
fn test_assemble_skill_missing_reference() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("operation should succeed: {error}"));
    let main_path = temp_dir.path().join("SKILL.md");
    fs::write(&main_path, "Main content")
        .unwrap_or_else(|error| panic!("operation should succeed: {error}"));

    let missing_path = temp_dir.path().join("missing.md");

    let assembler = ContextAssembler::new();

    let result = assembler
        .assemble_skill(main_path, vec![missing_path], serde_json::json!({}))
        .unwrap_or_else(|error| panic!("operation should succeed: {error}"));

    assert_eq!(result.missing_refs.len(), 1);
}
