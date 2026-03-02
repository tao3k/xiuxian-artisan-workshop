//! Snapshot contracts for schema validation and tool name formatting.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xiuxian_skills::{IndexToolEntry, SkillIndexEntry, ToolsScanner};

fn snapshot_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(relative)
}

fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("schema_validation_matrix")
        .join(relative)
}

fn read_snapshot(relative: &str) -> String {
    let path = snapshot_path(relative);
    fs::read_to_string(path.as_path())
        .unwrap_or_else(|error| panic!("failed to read snapshot {}: {error}", path.display()))
}

fn assert_snapshot_eq(relative: &str, actual: &str) {
    let expected = read_snapshot(relative);
    assert_eq!(
        expected, actual,
        "snapshot mismatch: {relative}\n--- expected ---\n{expected}\n--- actual ---\n{actual}"
    );
}

fn read_fixture(relative: &str) -> String {
    let path = fixture_path(relative);
    fs::read_to_string(path.as_path())
        .unwrap_or_else(|error| panic!("failed to read fixture {}: {error}", path.display()))
}

fn write_fixture_file(target: &Path, relative: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(target, read_fixture(relative))?;
    Ok(())
}

#[test]
fn snapshot_tool_name_format_matrix_contract() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = ToolsScanner::new();
    let temp_dir = TempDir::new()?;
    let matrix_root = temp_dir.path().join("scripts");
    fs::create_dir_all(&matrix_root)?;

    let cases = [
        ("named", "git", "scripts/named/commit.py"),
        ("multiple", "git", "scripts/multiple/commit.py"),
        ("underscore", "my_skill", "scripts/underscore/utils.py"),
        (
            "function_fallback",
            "test",
            "scripts/function_fallback/hello.py",
        ),
        ("path_like", "my_skill", "scripts/path_like/cmd.py"),
    ];

    let mut rows = Vec::new();
    for (case_id, skill_name, fixture_rel) in cases {
        let scripts_dir = matrix_root.join(case_id);
        fs::create_dir_all(&scripts_dir)?;
        let filename = fixture_rel
            .rsplit('/')
            .next()
            .ok_or_else(|| std::io::Error::other("fixture file name missing"))?;
        write_fixture_file(scripts_dir.join(filename).as_path(), fixture_rel)?;

        let mut tool_names = scanner
            .scan_scripts(&scripts_dir, skill_name, &[], &[])?
            .into_iter()
            .map(|tool| tool.tool_name)
            .collect::<Vec<_>>();
        tool_names.sort();

        let repeated_prefix = format!("{skill_name}.{skill_name}.");
        let invalid_part_count = tool_names
            .iter()
            .filter(|name| name.split('.').count() != 2)
            .count();
        let repeated_prefix_count = tool_names
            .iter()
            .filter(|name| name.starts_with(repeated_prefix.as_str()))
            .count();
        let first_part_mismatch_count = tool_names
            .iter()
            .filter(|name| name.split('.').next() != Some(skill_name))
            .count();

        rows.push(serde_json::json!({
            "case": case_id,
            "skill_name": skill_name,
            "tool_count": tool_names.len(),
            "tool_names": tool_names,
            "invalid_part_count": invalid_part_count,
            "repeated_prefix_count": repeated_prefix_count,
            "first_part_mismatch_count": first_part_mismatch_count
        }));
    }

    let actual = format!("{}\n", serde_json::to_string_pretty(&rows)?);
    assert_snapshot_eq(
        "schema_validation/tool_name_format_matrix.json",
        actual.as_str(),
    );
    Ok(())
}

#[test]
fn snapshot_skill_index_json_schema_contract() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = SkillIndexEntry::new(
        "git".to_string(),
        "Git skill".to_string(),
        "1.0.0".to_string(),
        "assets/skills/git".to_string(),
    );

    entry.add_tool(IndexToolEntry {
        name: "git.commit".to_string(),
        description: "Create commit".to_string(),
        category: String::new(),
        input_schema: String::new(),
        file_hash: String::new(),
    });
    entry.add_tool(IndexToolEntry {
        name: "git.smart_commit".to_string(),
        description: "Smart commit workflow".to_string(),
        category: String::new(),
        input_schema: String::new(),
        file_hash: String::new(),
    });

    let tool_names = entry
        .tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();
    let actual = serde_json::json!({
        "name": entry.name,
        "version": entry.version,
        "tool_count": tool_names.len(),
        "tool_names": tool_names
    });
    let actual = format!("{}\n", serde_json::to_string_pretty(&actual)?);
    assert_snapshot_eq("schema_validation/skill_index_schema.json", actual.as_str());
    Ok(())
}
