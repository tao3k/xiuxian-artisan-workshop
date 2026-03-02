//! Focused behavioral tests for `SkillScanner`.
//!
//! Data-contract-heavy `SKILL.md` coverage now lives in fixture+snapshot tests:
//! - `test_skill_scanner_snapshots.rs`
//! - `test_skill_scanner_matrix_snapshots.rs`

use std::fs;
use std::io;

use tempfile::TempDir;
use xiuxian_skills::{
    SkillMetadata, SkillScanner, SnifferRule, ToolAnnotations, ToolRecord, extract_frontmatter,
};

fn sample_metadata() -> SkillMetadata {
    SkillMetadata {
        skill_name: "test_skill".to_string(),
        version: "1.0.0".to_string(),
        description: "A test skill".to_string(),
        routing_keywords: vec!["test".to_string()],
        authors: vec!["omni-dev-fusion".to_string()],
        intents: vec![],
        require_refs: vec![],
        repository: String::new(),
        permissions: vec![],
    }
}

fn tool_record(
    tool_name: &str,
    description: &str,
    file_path: &str,
    function_name: &str,
    file_hash: &str,
) -> ToolRecord {
    ToolRecord {
        tool_name: tool_name.to_string(),
        description: description.to_string(),
        skill_name: "test_skill".to_string(),
        file_path: file_path.to_string(),
        function_name: function_name.to_string(),
        execution_mode: "script".to_string(),
        keywords: vec!["test".to_string()],
        intents: vec![],
        file_hash: file_hash.to_string(),
        input_schema: r#"{"type":"object"}"#.to_string(),
        docstring: String::new(),
        category: "test".to_string(),
        annotations: ToolAnnotations::default(),
        parameters: vec![],
        skill_tools_refers: vec![],
        resource_uri: String::new(),
    }
}

#[test]
fn test_skill_scanner_new() {
    let _ = SkillScanner::new();
}

#[test]
fn test_sniffer_rule_creation() {
    let rule = SnifferRule::new("file_exists", "Cargo.toml");
    assert_eq!(rule.rule_type, "file_exists");
    assert_eq!(rule.pattern, "Cargo.toml");
}

#[test]
fn test_scan_empty_skills_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let scanner = SkillScanner::new();
    let results = scanner.scan_all(&skills_dir, None)?;
    assert!(results.is_empty());

    Ok(())
}

#[test]
fn test_scan_skill_missing_skill_md() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("empty_skill");
    fs::create_dir_all(&skill_path)?;

    let result = scanner.scan_skill(&skill_path, None)?;
    assert!(result.is_none());

    Ok(())
}

#[test]
fn test_extract_frontmatter() -> Result<(), Box<dyn std::error::Error>> {
    let content = "---\nname: test\nversion: 1.0\n---\n# Content\n";
    let frontmatter = extract_frontmatter(content)
        .ok_or_else(|| io::Error::other("expected frontmatter block"))?;
    assert!(frontmatter.contains("name:"));
    assert!(frontmatter.contains("version:"));

    Ok(())
}

#[test]
fn test_extract_frontmatter_no_frontmatter() {
    let content = "# Just content\nNo frontmatter here.";
    assert!(extract_frontmatter(content).is_none());
}

#[test]
fn test_build_index_entry_deduplicates_tools() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("test_skill");
    let metadata = sample_metadata();

    let tools = vec![
        tool_record(
            "test_skill.real_tool",
            "The real tool",
            "/test/scripts/real.py",
            "real_tool",
            "hash1",
        ),
        tool_record(
            "test_skill.real_tool",
            "Example in docstring",
            "/test/scripts/other.py",
            "example_func",
            "hash2",
        ),
    ];

    let scanner = SkillScanner::new();
    let entry = scanner.build_index_entry(metadata, &tools, &skill_path);
    assert_eq!(entry.tools.len(), 1);
    assert_eq!(entry.tools[0].name, "test_skill.real_tool");
    assert_eq!(entry.tools[0].description, "The real tool");

    Ok(())
}

#[test]
fn test_build_index_entry_preserves_order() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("test_skill");
    let metadata = sample_metadata();

    let tools = vec![
        tool_record(
            "test_skill.tool_b",
            "Tool B",
            "/test/scripts/b.py",
            "tool_b",
            "hash1",
        ),
        tool_record(
            "test_skill.tool_a",
            "Tool A",
            "/test/scripts/a.py",
            "tool_a",
            "hash2",
        ),
        tool_record(
            "test_skill.tool_b",
            "Duplicate B",
            "/test/scripts/c.py",
            "dup_b",
            "hash3",
        ),
    ];

    let scanner = SkillScanner::new();
    let entry = scanner.build_index_entry(metadata, &tools, &skill_path);
    assert_eq!(entry.tools.len(), 2);
    assert_eq!(entry.tools[0].name, "test_skill.tool_b");
    assert_eq!(entry.tools[0].description, "Tool B");
    assert_eq!(entry.tools[1].name, "test_skill.tool_a");
    assert_eq!(entry.tools[1].description, "Tool A");

    Ok(())
}
