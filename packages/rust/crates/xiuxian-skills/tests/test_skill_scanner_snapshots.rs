//! Snapshot contracts for `SkillScanner` metadata and strict validation outputs.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xiuxian_skills::{SkillMetadata, SkillScanner, ToolAnnotations, ToolRecord};

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
        .join("skill_scanner_snapshots")
        .join(relative)
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

fn sanitize_path_in_string(input: &str, target: &Path) -> String {
    input.replace(target.to_string_lossy().as_ref(), "<SKILL_PATH>")
}

fn sanitize_json_paths(value: &mut serde_json::Value, target: &Path) {
    match value {
        serde_json::Value::String(text) => {
            *text = sanitize_path_in_string(text, target);
        }
        serde_json::Value::Array(items) => {
            for item in items {
                sanitize_json_paths(item, target);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, child) in map.iter_mut() {
                sanitize_json_paths(child, target);
            }
        }
        _ => {}
    }
}

fn canonicalize_json(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut entries: Vec<(String, serde_json::Value)> = map.into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut sorted = serde_json::Map::new();
            for (key, child) in entries {
                sorted.insert(key, canonicalize_json(child));
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonicalize_json).collect())
        }
        scalar => scalar,
    }
}

#[test]
fn snapshot_skill_metadata_parse_contract() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("auditor_neuron");
    fs::create_dir_all(&skill_path)?;
    let content = read_fixture("auditor_neuron_parse/SKILL.md");

    let metadata = scanner.parse_skill_md(content.as_str(), &skill_path)?;
    let actual = format!("{}\n", serde_json::to_string_pretty(&metadata)?);
    assert_snapshot_eq("skill_scanner/parsed_metadata.json", actual.as_str());
    Ok(())
}

#[test]
fn snapshot_structure_validation_summary_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("auditor_neuron");
    fs::create_dir_all(skill_path.join("scripts"))?;
    fs::create_dir_all(skill_path.join("references"))?;
    fs::create_dir_all(skill_path.join("scratch"))?;
    write_fixture_file(
        skill_path.join("SKILL.md").as_path(),
        "auditor_neuron_base/SKILL.md",
    )?;

    let structure = SkillScanner::default_structure();
    let report = SkillScanner::validate_structure_report(&skill_path, &structure);
    let mut summary = report.to_summary_json(&skill_path);
    sanitize_json_paths(&mut summary, &skill_path);
    let actual = format!("{}\n", serde_json::to_string_pretty(&summary)?);
    assert_snapshot_eq(
        "skill_scanner/structure_report_summary.json",
        actual.as_str(),
    );
    Ok(())
}

#[test]
fn snapshot_missing_type_error_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("auditor_neuron");
    fs::create_dir_all(skill_path.join("references"))?;
    write_fixture_file(
        skill_path.join("SKILL.md").as_path(),
        "auditor_neuron_base/SKILL.md",
    )?;
    write_fixture_file(
        skill_path.join("references/teacher.md").as_path(),
        "missing_type/teacher.md",
    )?;

    let scanner = SkillScanner::new();
    let structure = SkillScanner::default_structure();
    let error = scanner
        .scan_skill(&skill_path, Some(&structure))
        .err()
        .ok_or_else(|| std::io::Error::other("expected strict metadata error"))?
        .to_string();
    let normalized = format!("{}\n", sanitize_path_in_string(&error, &skill_path));
    assert_snapshot_eq("skill_scanner/missing_type_error.txt", normalized.as_str());
    Ok(())
}

#[test]
fn snapshot_structure_validation_rendered_lines_contract() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("auditor_neuron");
    fs::create_dir_all(skill_path.join("scripts"))?;
    fs::create_dir_all(skill_path.join("references"))?;
    fs::create_dir_all(skill_path.join("scratch"))?;
    write_fixture_file(
        skill_path.join("SKILL.md").as_path(),
        "auditor_neuron_base/SKILL.md",
    )?;

    let structure = SkillScanner::default_structure();
    let report = SkillScanner::validate_structure_report(&skill_path, &structure);
    let rendered: Vec<String> = report
        .render_for_skill(&skill_path)
        .into_iter()
        .map(|line| sanitize_path_in_string(line.as_str(), &skill_path))
        .collect();
    let actual = format!("{}\n", rendered.join("\n"));
    assert_snapshot_eq(
        "skill_scanner/structure_report_rendered_lines.txt",
        actual.as_str(),
    );
    Ok(())
}

#[test]
fn snapshot_scan_all_multiple_skills_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let writer_path = skills_dir.join("writer");
    fs::create_dir_all(&writer_path)?;
    write_fixture_file(
        writer_path.join("SKILL.md").as_path(),
        "scan_all/writer/SKILL.md",
    )?;

    let git_path = skills_dir.join("git");
    fs::create_dir_all(&git_path)?;
    write_fixture_file(git_path.join("SKILL.md").as_path(), "scan_all/git/SKILL.md")?;

    let scanner = SkillScanner::new();
    let mut metadatas = scanner.scan_all(&skills_dir, None)?;
    metadatas.sort_by(|left, right| left.skill_name.cmp(&right.skill_name));
    let actual = format!("{}\n", serde_json::to_string_pretty(&metadatas)?);
    assert_snapshot_eq(
        "skill_scanner/scan_all_multiple_skills.json",
        actual.as_str(),
    );
    Ok(())
}

#[test]
fn snapshot_canonical_payload_tool_reference_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("researcher");
    fs::create_dir_all(&skill_path)?;
    fs::create_dir_all(skill_path.join("references"))?;
    write_fixture_file(
        skill_path.join("SKILL.md").as_path(),
        "canonical_payload/researcher/SKILL.md",
    )?;
    write_fixture_file(
        skill_path
            .join("references/run_research_graph.md")
            .as_path(),
        "canonical_payload/researcher/references/run_research_graph.md",
    )?;

    let metadata = SkillMetadata {
        skill_name: "researcher".to_string(),
        version: "1.0.0".to_string(),
        description: "Research skill".to_string(),
        routing_keywords: vec!["research".to_string()],
        authors: vec![],
        intents: vec![],
        require_refs: vec![],
        repository: String::new(),
        permissions: vec![],
    };

    let tools = vec![ToolRecord {
        tool_name: "researcher.run_research_graph".to_string(),
        description: "Run the graph".to_string(),
        skill_name: "researcher".to_string(),
        file_path: "researcher/scripts/commands.py".to_string(),
        function_name: "run_research_graph".to_string(),
        execution_mode: "async".to_string(),
        keywords: vec!["research".to_string()],
        intents: vec![],
        file_hash: "abc".to_string(),
        input_schema: "{}".to_string(),
        docstring: String::new(),
        category: "research".to_string(),
        annotations: ToolAnnotations::default(),
        parameters: vec![],
        skill_tools_refers: vec![],
        resource_uri: String::new(),
    }];

    let scanner = SkillScanner::new();
    let payload = scanner.build_canonical_payload(metadata, &tools, &skill_path);
    let mut json_value = serde_json::to_value(payload)?;
    sanitize_json_paths(&mut json_value, &skill_path);
    let canonical = canonicalize_json(json_value);
    let actual = format!("{}\n", serde_json::to_string_pretty(&canonical)?);
    assert_snapshot_eq(
        "skill_scanner/canonical_payload_tool_reference.json",
        actual.as_str(),
    );
    Ok(())
}
