//! Snapshot-backed full workflow integration contracts for `xiuxian-skills`.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xiuxian_skills::VERSION;
use xiuxian_skills::{SkillScanner, ToolsScanner};

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
        .join("full_workflow_snapshots")
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
fn test_version_constant() {
    assert!(!VERSION.is_empty());
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
}

#[test]
fn snapshot_full_scan_workflow_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let writer_path = skills_dir.join("writer");
    fs::create_dir_all(writer_path.join("scripts"))?;
    write_fixture_file(
        writer_path.join("SKILL.md").as_path(),
        "full_scan/writer/SKILL.md",
    )?;
    write_fixture_file(
        writer_path.join("scripts/text.py").as_path(),
        "full_scan/writer/scripts/text.py",
    )?;

    let git_path = skills_dir.join("git");
    fs::create_dir_all(&git_path)?;
    write_fixture_file(
        git_path.join("SKILL.md").as_path(),
        "full_scan/git/SKILL.md",
    )?;

    let skill_scanner = SkillScanner::new();
    let mut metadatas = skill_scanner.scan_all(&skills_dir, None)?;
    metadatas.sort_by(|left, right| left.skill_name.cmp(&right.skill_name));

    let tools_scanner = ToolsScanner::new();
    let writer_metadata = metadatas
        .iter()
        .find(|m| m.skill_name == "writer")
        .ok_or_else(|| std::io::Error::other("writer metadata should exist"))?;
    let writer_tools = tools_scanner.scan_scripts(
        &writer_path.join("scripts"),
        "writer",
        &writer_metadata.routing_keywords,
        &writer_metadata.intents,
    )?;

    let mut tool_names = writer_tools
        .iter()
        .map(|tool| tool.tool_name.clone())
        .collect::<Vec<_>>();
    tool_names.sort();
    let mut writer_keywords = writer_tools
        .iter()
        .flat_map(|tool| tool.keywords.iter().cloned())
        .collect::<Vec<_>>();
    writer_keywords.sort();
    writer_keywords.dedup();

    let metadata_projection = metadatas
        .iter()
        .map(|metadata| {
            serde_json::json!({
                "skill_name": metadata.skill_name,
                "version": metadata.version,
                "description": metadata.description,
                "routing_keywords": metadata.routing_keywords,
                "intents": metadata.intents
            })
        })
        .collect::<Vec<_>>();

    let actual = serde_json::json!({
        "metadata_projection": metadata_projection,
        "writer_tool_count": writer_tools.len(),
        "writer_tool_names": tool_names,
        "writer_keyword_checks": {
            "contains_write": writer_keywords.contains(&"write".to_string()),
            "contains_edit": writer_keywords.contains(&"edit".to_string()),
            "contains_polish": writer_keywords.contains(&"polish".to_string())
        }
    });
    let actual = format!("{}\n", serde_json::to_string_pretty(&actual)?);
    assert_snapshot_eq("full_workflow/full_scan_workflow.json", actual.as_str());
    Ok(())
}

#[test]
fn snapshot_scanner_reports_duplicate_tools_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("skills/test");
    fs::create_dir_all(skill_path.join("scripts"))?;
    write_fixture_file(
        skill_path.join("SKILL.md").as_path(),
        "duplicate_tools/test/SKILL.md",
    )?;
    write_fixture_file(
        skill_path.join("scripts/commands.py").as_path(),
        "duplicate_tools/test/scripts/commands.py",
    )?;
    write_fixture_file(
        skill_path.join("scripts/more_commands.py").as_path(),
        "duplicate_tools/test/scripts/more_commands.py",
    )?;

    let tools_scanner = ToolsScanner::new();
    let tools = tools_scanner.scan_scripts(
        &skill_path.join("scripts"),
        "test",
        &["test".to_string()],
        &[],
    )?;

    let mut tool_names = tools
        .iter()
        .map(|tool| tool.tool_name.clone())
        .collect::<Vec<_>>();
    tool_names.sort();
    let unique_count = tool_names
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let file_hashes_unique = tools
        .iter()
        .map(|tool| tool.file_hash.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    let actual = serde_json::json!({
        "tool_count": tools.len(),
        "tool_names": tool_names,
        "unique_tool_name_count": unique_count,
        "unique_file_hash_count": file_hashes_unique
    });
    let actual = format!("{}\n", serde_json::to_string_pretty(&actual)?);
    assert_snapshot_eq("full_workflow/duplicate_tools.json", actual.as_str());
    Ok(())
}

#[test]
fn snapshot_same_function_name_different_skills_contract() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let skill1_path = skills_dir.join("skill1");
    fs::create_dir_all(skill1_path.join("scripts"))?;
    write_fixture_file(
        skill1_path.join("SKILL.md").as_path(),
        "cross_skill/skill1/SKILL.md",
    )?;
    write_fixture_file(
        skill1_path.join("scripts/main.py").as_path(),
        "cross_skill/skill1/scripts/main.py",
    )?;

    let skill2_path = skills_dir.join("skill2");
    fs::create_dir_all(skill2_path.join("scripts"))?;
    write_fixture_file(
        skill2_path.join("SKILL.md").as_path(),
        "cross_skill/skill2/SKILL.md",
    )?;
    write_fixture_file(
        skill2_path.join("scripts/main.py").as_path(),
        "cross_skill/skill2/scripts/main.py",
    )?;

    let tools_scanner = ToolsScanner::new();
    let mut skill1_tools = tools_scanner
        .scan_scripts(
            &skill1_path.join("scripts"),
            "skill1",
            &["s1".to_string()],
            &[],
        )?
        .into_iter()
        .map(|tool| tool.tool_name)
        .collect::<Vec<_>>();
    skill1_tools.sort();
    let mut skill2_tools = tools_scanner
        .scan_scripts(
            &skill2_path.join("scripts"),
            "skill2",
            &["s2".to_string()],
            &[],
        )?
        .into_iter()
        .map(|tool| tool.tool_name)
        .collect::<Vec<_>>();
    skill2_tools.sort();

    let actual = serde_json::json!({
        "skill1_tools": skill1_tools,
        "skill2_tools": skill2_tools
    });
    let actual = format!("{}\n", serde_json::to_string_pretty(&actual)?);
    assert_snapshot_eq(
        "full_workflow/cross_skill_same_function.json",
        actual.as_str(),
    );
    Ok(())
}
