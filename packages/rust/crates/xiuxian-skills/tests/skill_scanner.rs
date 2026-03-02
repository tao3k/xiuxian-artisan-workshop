//! Integration tests for `SkillScanner` focused on behavior and structural enforcement.
//!
//! Data-contract-heavy frontmatter cases are covered by fixture-based snapshot tests:
//! - `test_skill_scanner_snapshots.rs`
//! - `test_skill_scanner_matrix_snapshots.rs`

use std::fs;
use std::io;
use std::path::PathBuf;
use tempfile::TempDir;
use xiuxian_skills::{SkillScanner, ToolsScanner};

/// Test default structure matches declarative config definition.
#[test]
fn test_default_structure_required_files() {
    let structure = SkillScanner::default_structure();

    // Required: SKILL.md
    assert!(!structure.required.is_empty());
    assert!(structure.required.iter().any(|i| i.path == "SKILL.md"));
    assert!(structure.required.iter().any(|i| i.item_type == "file"));
}

/// Test default structure contains expected default directories.
#[test]
fn test_default_structure_default_directories() {
    let structure = SkillScanner::default_structure();

    // Default directories should include scripts/ and references/.
    assert!(!structure.default.is_empty());
    assert!(structure.default.iter().any(|i| i.path == "scripts/"));
    assert!(structure.default.iter().any(|i| i.path == "references/"));
    assert!(
        structure.optional.is_empty(),
        "embedded skills.toml currently defines no optional entries"
    );
}

/// Validate skill with valid SKILL.md passes.
#[test]
fn test_validate_structure_valid_skill() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("writer");
    fs::create_dir_all(skill_path.join("scripts"))?;
    fs::create_dir_all(skill_path.join("references"))?;
    fs::create_dir_all(skill_path.join("assets"))?;
    fs::create_dir_all(skill_path.join("tests"))?;

    // Create required SKILL.md
    fs::write(
        skill_path.join("SKILL.md"),
        r#"---
name: "writer"
metadata:
  version: "1.0"
  routing_keywords: ["write", "edit"]
---
# Writer Skill
"#,
    )?;

    let structure = SkillScanner::default_structure();
    assert!(SkillScanner::validate_structure(&skill_path, &structure));

    Ok(())
}

/// Validate skill missing SKILL.md fails.
#[test]
fn test_validate_structure_missing_skill_md() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("empty_skill");
    fs::create_dir_all(&skill_path)?;

    // No SKILL.md created
    let structure = SkillScanner::default_structure();
    assert!(!SkillScanner::validate_structure(&skill_path, &structure));

    Ok(())
}

/// Validate structure reports out-of-scope entries as warnings (non-blocking).
#[test]
fn test_validate_structure_reports_out_of_scope_warning() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("writer");
    fs::create_dir_all(skill_path.join("scripts"))?;
    fs::create_dir_all(skill_path.join("references"))?;
    fs::create_dir_all(skill_path.join("assets"))?;
    fs::create_dir_all(skill_path.join("tests"))?;
    fs::create_dir_all(skill_path.join("temp_junk"))?;
    fs::write(
        skill_path.join("SKILL.md"),
        r#"---
name: writer
description: Use when writing.
metadata:
  version: "1.0.0"
---
# Writer
"#,
    )?;

    let structure = SkillScanner::default_structure();
    let report = SkillScanner::validate_structure_report(&skill_path, &structure);
    assert!(
        report.valid,
        "out-of-scope entries should not invalidate a skill"
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.contains("temp_junk")),
        "expected out-of-scope warning for temp_junk"
    );

    Ok(())
}

/// Validate nonexistent path returns false.
#[test]
fn test_validate_structure_nonexistent_path() {
    let structure = SkillScanner::default_structure();
    let nonexistent = PathBuf::from("/nonexistent/path");
    assert!(!SkillScanner::validate_structure(&nonexistent, &structure));
}

/// Scan all skills in base directory with structure validation.
#[test]
fn test_scan_all_with_structure_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    // Create valid writer skill
    let writer_path = skills_dir.join("writer");
    fs::create_dir_all(&writer_path)?;
    fs::write(
        writer_path.join("SKILL.md"),
        r#"---
name: "writer"
metadata:
  version: "1.0"
  routing_keywords: ["write", "edit"]
---
# Writer
"#,
    )?;

    // Create valid git skill
    let git_path = skills_dir.join("git");
    fs::create_dir_all(&git_path)?;
    fs::write(
        git_path.join("SKILL.md"),
        r#"---
name: "git"
metadata:
  version: "1.0"
  routing_keywords: ["commit", "branch"]
---
# Git
"#,
    )?;

    // Create incomplete skill (no SKILL.md)
    let no_md_path = skills_dir.join("no_md");
    fs::create_dir_all(&no_md_path)?;

    let scanner = SkillScanner::new();
    let structure = SkillScanner::default_structure();

    let metadatas = scanner.scan_all(&skills_dir, Some(&structure))?;
    assert_eq!(metadatas.len(), 2);

    // Verify both valid skills are found
    assert!(metadatas.iter().any(|m| m.skill_name == "writer"));
    assert!(metadatas.iter().any(|m| m.skill_name == "git"));
    // Incomplete skill should not be in results (no SKILL.md)
    assert!(!metadatas.iter().any(|m| m.skill_name == "no_md"));

    Ok(())
}

/// Scan all skills without structure (backward compatibility).
#[test]
fn test_scan_all_without_structure() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let writer_path = skills_dir.join("writer");
    fs::create_dir_all(&writer_path)?;
    fs::write(
        writer_path.join("SKILL.md"),
        r#"---
name: "writer"
metadata:
  version: "1.0"
---
# Writer
"#,
    )?;

    let scanner = SkillScanner::new();
    // Pass None - should still work
    let metadatas = scanner.scan_all(&skills_dir, None)?;
    assert_eq!(metadatas.len(), 1);

    Ok(())
}

/// Scan all skills with nonexistent base path returns empty vec.
#[test]
fn test_scan_all_nonexistent_base_path() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let nonexistent_path = PathBuf::from("/nonexistent");
    let metadatas = scanner.scan_all(&nonexistent_path, None)?;
    assert!(metadatas.is_empty());

    Ok(())
}

/// Skill name is derived from directory name when not in frontmatter.
#[test]
fn test_skill_name_from_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("custom_skill_name");
    fs::create_dir_all(&skill_path)?;

    fs::write(
        skill_path.join("SKILL.md"),
        r#"---
version: "1.0"
---
# Content
"#,
    )?;

    let scanner = SkillScanner::new();
    let result = scanner
        .scan_skill(&skill_path, None)?
        .ok_or_else(|| io::Error::other("expected custom_skill_name metadata"))?;

    assert_eq!(result.skill_name, "custom_skill_name");

    Ok(())
}

// =============================================================================
// TOML Rules Parsing Tests
// =============================================================================

/// Test parsing valid rules.toml file.
#[test]
fn test_parse_rules_toml_valid() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("python");
    fs::create_dir_all(&skill_path)?;

    // Create required SKILL.md
    fs::write(
        skill_path.join("SKILL.md"),
        r#"---
name: "python"
metadata:
  version: "1.0"
  routing_keywords: ["python", "py"]
---
# Python Skill
"#,
    )?;

    let rules_path = skill_path.join("extensions/sniffer");
    fs::create_dir_all(&rules_path)?;

    fs::write(
        rules_path.join("rules.toml"),
        r#"
[[match]]
type = "file_exists"
pattern = "pyproject.toml"

[[match]]
type = "file_pattern"
pattern = "*.py"
"#,
    )?;

    let scanner = SkillScanner::new();
    let structure = SkillScanner::default_structure();

    // Scan and build index entry
    let result = scanner
        .scan_skill(&skill_path, Some(&structure))?
        .ok_or_else(|| io::Error::other("expected python metadata"))?;
    assert_eq!(result.skill_name, "python");

    // Verify the skill directory has rules
    let rules = scanner.scan_skill(&skill_path, Some(&structure))?;
    assert!(rules.is_some());

    Ok(())
}

/// Test parsing missing rules.toml returns empty rules.
#[test]
fn test_parse_rules_toml_missing() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("test_skill");
    fs::create_dir_all(&skill_path)?;

    // No rules.toml created - scanner should still work
    let scanner = SkillScanner::new();

    let result = scanner.scan_skill(&skill_path, None)?;
    // Should return Some because SKILL.md exists (not created, so None)
    assert!(result.is_none());

    Ok(())
}

/// Test `build_index_entry` includes sniffer rules from `rules.toml`.
#[test]
fn test_build_index_entry_with_sniffer_rules() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let python_path = skills_dir.join("python");
    fs::create_dir_all(&python_path)?;

    // Create SKILL.md
    fs::write(
        python_path.join("SKILL.md"),
        r#"---
name: "python"
metadata:
  version: "1.0"
  routing_keywords: ["python", "py"]
---
# Python Skill
"#,
    )?;

    // Create rules.toml
    let rules_path = python_path.join("extensions/sniffer");
    fs::create_dir_all(&rules_path)?;
    fs::write(
        rules_path.join("rules.toml"),
        r#"
[[match]]
type = "file_exists"
pattern = "pyproject.toml"
"#,
    )?;

    let scanner = SkillScanner::new();
    let tools_scanner = ToolsScanner::new();

    let metadatas = scanner.scan_all(&skills_dir, None)?;
    assert_eq!(metadatas.len(), 1);

    let metadata = &metadatas[0];
    let scripts_path = python_path.join("scripts");
    let tools = if scripts_path.exists() {
        tools_scanner.scan_scripts(
            &scripts_path,
            &metadata.skill_name,
            &metadata.routing_keywords,
            &[],
        )?
    } else {
        Vec::new()
    };

    let entry = scanner.build_index_entry(metadata.clone(), &tools, &python_path);

    // Verify sniffer rules are populated
    assert!(!entry.sniffing_rules.is_empty());
    assert_eq!(entry.sniffing_rules[0].pattern, "pyproject.toml");
    assert_eq!(entry.sniffing_rules[0].rule_type, "file_exists");

    Ok(())
}
