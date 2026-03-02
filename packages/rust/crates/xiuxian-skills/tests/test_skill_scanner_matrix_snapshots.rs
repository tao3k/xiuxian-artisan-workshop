//! Matrix snapshot contracts for frontmatter diversity and strict validation behavior.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xiuxian_skills::SkillScanner;

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
        .join("skill_scanner_matrix")
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

fn sanitize_path(text: &str, skill_path: &Path) -> String {
    text.replace(skill_path.to_string_lossy().as_ref(), "<SKILL_PATH>")
}

#[test]
fn snapshot_skill_frontmatter_matrix_contract() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let structure = SkillScanner::default_structure();
    let temp_dir = TempDir::new()?;
    let matrix_root = temp_dir.path().join("skills");
    fs::create_dir_all(&matrix_root)?;

    let cases = [
        ("valid", true),
        ("missing_markers", false),
        ("missing_name", false),
        ("missing_metadata", false),
        ("metadata_not_mapping", false),
        ("name_empty", false),
        ("malformed_yaml", false),
    ];

    let mut outcomes = Vec::new();
    for (case_id, expected_ok) in cases {
        let skill_path = matrix_root.join(case_id);
        fs::create_dir_all(&skill_path)?;
        write_fixture_file(
            skill_path.join("SKILL.md").as_path(),
            &format!("skill_frontmatter/{case_id}/SKILL.md"),
        )?;

        let mut row = serde_json::Map::new();
        row.insert("case".to_string(), serde_json::json!(case_id));
        row.insert("expected_ok".to_string(), serde_json::json!(expected_ok));
        match scanner.scan_skill(&skill_path, Some(&structure)) {
            Ok(Some(metadata)) => {
                row.insert("actual_ok".to_string(), serde_json::json!(true));
                row.insert(
                    "skill_name".to_string(),
                    serde_json::json!(metadata.skill_name),
                );
            }
            Ok(None) => {
                row.insert("actual_ok".to_string(), serde_json::json!(false));
                row.insert("error".to_string(), serde_json::json!("scan returned None"));
            }
            Err(error) => {
                row.insert("actual_ok".to_string(), serde_json::json!(false));
                row.insert(
                    "error".to_string(),
                    serde_json::json!(sanitize_path(error.to_string().as_str(), &skill_path)),
                );
            }
        }
        outcomes.push(serde_json::Value::Object(row));
    }

    let actual = format!("{}\n", serde_json::to_string_pretty(&outcomes)?);
    assert_snapshot_eq(
        "skill_scanner/skill_frontmatter_matrix.json",
        actual.as_str(),
    );
    Ok(())
}

#[test]
fn snapshot_reference_frontmatter_matrix_contract() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let structure = SkillScanner::default_structure();
    let temp_dir = TempDir::new()?;
    let matrix_root = temp_dir.path().join("skills");
    fs::create_dir_all(&matrix_root)?;

    let cases = [
        ("valid_knowledge", true),
        ("valid_persona", true),
        ("missing_type", false),
        ("invalid_type", false),
        ("persona_missing_role_class", false),
        ("missing_markers", false),
        ("malformed_yaml", false),
    ];

    let mut outcomes = Vec::new();
    for (case_id, expected_ok) in cases {
        let skill_path = matrix_root.join(case_id);
        fs::create_dir_all(skill_path.join("references"))?;
        write_fixture_file(
            skill_path.join("SKILL.md").as_path(),
            "reference_frontmatter/base/SKILL.md",
        )?;
        write_fixture_file(
            skill_path.join("references/doc.md").as_path(),
            &format!("reference_frontmatter/{case_id}/doc.md"),
        )?;

        let mut row = serde_json::Map::new();
        row.insert("case".to_string(), serde_json::json!(case_id));
        row.insert("expected_ok".to_string(), serde_json::json!(expected_ok));
        match scanner.scan_skill(&skill_path, Some(&structure)) {
            Ok(Some(metadata)) => {
                row.insert("actual_ok".to_string(), serde_json::json!(true));
                row.insert(
                    "skill_name".to_string(),
                    serde_json::json!(metadata.skill_name),
                );
            }
            Ok(None) => {
                row.insert("actual_ok".to_string(), serde_json::json!(false));
                row.insert("error".to_string(), serde_json::json!("scan returned None"));
            }
            Err(error) => {
                row.insert("actual_ok".to_string(), serde_json::json!(false));
                row.insert(
                    "error".to_string(),
                    serde_json::json!(sanitize_path(error.to_string().as_str(), &skill_path)),
                );
            }
        }
        outcomes.push(serde_json::Value::Object(row));
    }

    let actual = format!("{}\n", serde_json::to_string_pretty(&outcomes)?);
    assert_snapshot_eq(
        "skill_scanner/reference_frontmatter_matrix.json",
        actual.as_str(),
    );
    Ok(())
}

#[test]
fn snapshot_parse_skill_md_matrix_contract() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let temp_dir = TempDir::new()?;
    let matrix_root = temp_dir.path().join("skills");
    fs::create_dir_all(&matrix_root)?;

    let cases = [
        ("writer_full", "writer", true),
        ("researcher_spaces", "researcher", true),
        ("no_frontmatter", "minimal", true),
        ("malformed_yaml", "broken", false),
    ];

    let mut outcomes = Vec::new();
    for (case_id, skill_dir, expected_ok) in cases {
        let skill_path = matrix_root.join(skill_dir);
        fs::create_dir_all(&skill_path)?;
        let content = read_fixture(&format!("parse_skill_md/{case_id}/SKILL.md"));

        let mut row = serde_json::Map::new();
        row.insert("case".to_string(), serde_json::json!(case_id));
        row.insert("expected_ok".to_string(), serde_json::json!(expected_ok));
        match scanner.parse_skill_md(content.as_str(), &skill_path) {
            Ok(metadata) => {
                row.insert("actual_ok".to_string(), serde_json::json!(true));
                row.insert("metadata".to_string(), serde_json::to_value(metadata)?);
            }
            Err(error) => {
                row.insert("actual_ok".to_string(), serde_json::json!(false));
                row.insert("error".to_string(), serde_json::json!(error.to_string()));
            }
        }
        outcomes.push(serde_json::Value::Object(row));
    }

    let actual = format!("{}\n", serde_json::to_string_pretty(&outcomes)?);
    assert_snapshot_eq("skill_scanner/parse_skill_md_matrix.json", actual.as_str());
    Ok(())
}
