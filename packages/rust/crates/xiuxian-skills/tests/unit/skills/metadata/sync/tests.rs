use std::path::PathBuf;

use crate::skills::metadata::{IndexToolEntry, ToolRecord};

use super::{ScanConfig, SyncReport, calculate_sync_ops};

fn scanned_tool(skill_name: &str, tool_name: &str, file_hash: &str) -> ToolRecord {
    let mut record = ToolRecord::new(
        tool_name.to_string(),
        format!("Execute {skill_name}.{tool_name}"),
        skill_name.to_string(),
        format!("assets/skills/{skill_name}/scripts/{tool_name}.py"),
        tool_name.to_string(),
    );
    record.file_hash = file_hash.to_string();
    record
}

fn existing_tool(name: &str, file_hash: &str) -> IndexToolEntry {
    IndexToolEntry {
        name: name.to_string(),
        description: format!("Existing {name}"),
        category: String::new(),
        input_schema: String::new(),
        file_hash: file_hash.to_string(),
    }
}

#[test]
fn test_scan_config_defaults_and_builder() {
    let defaults = ScanConfig::new();
    assert_eq!(defaults.skills_dir, PathBuf::from("assets/skills"));
    assert!(defaults.include_optional);
    assert!(!defaults.skip_validation);

    let custom = ScanConfig::new().with_skills_dir("tmp/skills");
    assert_eq!(custom.skills_dir, PathBuf::from("tmp/skills"));
    assert!(custom.include_optional);
    assert!(!custom.skip_validation);
}

#[test]
fn test_calculate_sync_ops_classifies_added_updated_deleted_and_unchanged() {
    let scanned = vec![
        scanned_tool("alpha", "tool_a", "hash-a"),
        scanned_tool("alpha", "tool_b", "hash-b-new"),
        scanned_tool("alpha", "tool_c", "hash-c"),
    ];
    let existing = vec![
        existing_tool("alpha.tool_a", "hash-a"),
        existing_tool("alpha.tool_b", "hash-b-old"),
        existing_tool("alpha.tool_d", "hash-d"),
    ];

    let report = calculate_sync_ops(scanned, &existing);

    assert_eq!(report.unchanged_count, 1);
    assert_eq!(report.added.len(), 1);
    assert_eq!(report.updated.len(), 1);
    assert_eq!(report.deleted, vec!["alpha.tool_d".to_string()]);

    assert_eq!(report.added[0].tool_name, "tool_c");
    assert_eq!(report.updated[0].tool_name, "tool_b");
}

#[test]
fn test_calculate_sync_ops_with_empty_inputs() {
    let report = calculate_sync_ops(Vec::new(), &[]);
    assert_eq!(report, SyncReport::new());
}
