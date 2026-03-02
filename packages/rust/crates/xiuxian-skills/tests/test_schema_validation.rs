//! Focused behavior tests for schema invariants.
//!
//! Data-contract matrix coverage lives in fixture+snapshot tests:
//! - `test_schema_validation_matrix_snapshots.rs`

use std::fs;
use std::io;
use tempfile::TempDir;
use xiuxian_skills::{SkillMetadata, SkillScanner, ToolAnnotations, ToolRecord};

fn sample_metadata() -> SkillMetadata {
    SkillMetadata {
        skill_name: "git".to_string(),
        version: "1.0.0".to_string(),
        description: "Git skill".to_string(),
        routing_keywords: vec!["status".to_string()],
        authors: vec![],
        intents: vec![],
        require_refs: vec![],
        repository: String::new(),
        permissions: vec![],
    }
}

#[test]
fn test_tool_record_json_serialization_schema() -> Result<(), Box<dyn std::error::Error>> {
    let tool = ToolRecord {
        tool_name: "git.smart_commit".to_string(),
        description: "Execute git.smart_commit".to_string(),
        skill_name: "git".to_string(),
        file_path: "assets/skills/git/scripts/commit.py".to_string(),
        function_name: "smart_commit".to_string(),
        execution_mode: "script".to_string(),
        keywords: vec!["git".to_string(), "smart_commit".to_string()],
        intents: vec![],
        file_hash: "abc123".to_string(),
        input_schema: "{}".to_string(),
        docstring: "Smart commit workflow".to_string(),
        category: "commit".to_string(),
        annotations: ToolAnnotations::default(),
        parameters: vec!["action".to_string()],
        skill_tools_refers: vec![],
        resource_uri: String::new(),
    };

    let json = serde_json::to_string(&tool)?;
    let deserialized: ToolRecord = serde_json::from_str(&json)?;
    assert_eq!(deserialized.tool_name, "git.smart_commit");
    assert!(!deserialized.tool_name.starts_with("git.git."));

    Ok(())
}

#[test]
fn test_build_index_entry_no_double_prefix_regression() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let tools = vec![ToolRecord {
        tool_name: "git.status".to_string(),
        description: "Show status".to_string(),
        skill_name: "git".to_string(),
        file_path: "scripts/status.py".to_string(),
        function_name: "status".to_string(),
        execution_mode: "script".to_string(),
        keywords: vec![],
        intents: vec![],
        file_hash: "hash".to_string(),
        input_schema: "{}".to_string(),
        docstring: String::new(),
        category: "status".to_string(),
        annotations: ToolAnnotations::default(),
        parameters: vec![],
        skill_tools_refers: vec![],
        resource_uri: String::new(),
    }];

    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("git");
    let entry = scanner.build_index_entry(sample_metadata(), &tools, &skill_path);
    assert_eq!(entry.tools.len(), 1);
    assert_eq!(entry.tools[0].name, "git.status");
    assert!(!entry.tools[0].name.starts_with("git.git."));

    Ok(())
}

#[test]
fn test_empty_tools_schema() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("empty_skill");

    fs::create_dir_all(&skill_path)?;
    fs::write(
        skill_path.join("SKILL.md"),
        "---\nname: empty_skill\nmetadata:\n  version: 1.0.0\n---\n",
    )?;

    let entry = scanner.build_index_entry(
        SkillMetadata {
            skill_name: "empty_skill".to_string(),
            version: "1.0.0".to_string(),
            description: "Empty skill".to_string(),
            routing_keywords: vec![],
            authors: vec![],
            intents: vec![],
            require_refs: vec![],
            repository: String::new(),
            permissions: vec![],
        },
        &[],
        &skill_path,
    );
    assert!(entry.tools.is_empty());

    Ok(())
}

/// Validates `test_skill_index.json` for data integrity.
#[test]
fn test_skill_index_json_data_integrity() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;

    let test_file_path = Path::new("../../bindings/python/test_skill_index.json");
    let content = fs::read_to_string(test_file_path).map_err(|error| {
        io::Error::other(format!(
            "failed to read test file {test_file_path:?}: {error}"
        ))
    })?;

    let skills: Vec<serde_json::Value> = serde_json::from_str(&content).map_err(|error| {
        io::Error::other(format!(
            "failed to parse test_skill_index.json as valid JSON: {error}"
        ))
    })?;

    let mut errors: Vec<String> = Vec::new();
    for (i, skill) in skills.iter().enumerate() {
        let skill_name = skill["name"].as_str().unwrap_or("UNKNOWN");
        let skill_path = skill["path"].as_str().unwrap_or("");
        if skill_path.starts_with('"') && skill_path.ends_with('"') {
            errors.push(format!(
                "[{i}] Skill '{skill_name}': path has extra quotes: {skill_path}"
            ));
        }
        if !skill_path.starts_with("assets/skills/") {
            errors.push(format!(
                "[{i}] Skill '{skill_name}': path doesn't start with 'assets/skills/': {skill_path}"
            ));
        }
        if let Some(tools) = skill["tools"].as_array() {
            for (j, tool) in tools.iter().enumerate() {
                let tool_name = tool["name"].as_str().unwrap_or("UNKNOWN");
                let repeated_prefix = format!("{skill_name}.{skill_name}");
                if tool_name.starts_with(&repeated_prefix) {
                    errors.push(format!(
                        "[{i}.{j}] Tool '{tool_name}': repeated skill prefix",
                    ));
                }
                let parts: Vec<&str> = tool_name.split('.').collect();
                if parts.len() != 2 {
                    errors.push(format!(
                        "[{}.{}] Tool '{}': wrong format (expected 'skill.command', got {} parts)",
                        i,
                        j,
                        tool_name,
                        parts.len()
                    ));
                }
                if parts.first().copied() != Some(skill_name) {
                    errors.push(format!(
                        "[{i}.{j}] Tool '{tool_name}': first part doesn't match skill name '{skill_name}'"
                    ));
                }
            }
        }
    }

    assert!(
        errors.is_empty(),
        "test_skill_index.json has {} data integrity issues:\n{}",
        errors.len(),
        errors.join("\n")
    );

    Ok(())
}
