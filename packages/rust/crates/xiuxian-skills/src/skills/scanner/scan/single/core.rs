use std::fs;
use std::path::Path;

use crate::frontmatter::parse_frontmatter_from_markdown;
use crate::skills::metadata::{SkillMetadata, SkillStructure};

use super::super::super::SkillScanner;
use super::super::super::references::validate_references_strict;

pub(super) fn scan_skill_result(
    scanner: &SkillScanner,
    skill_path: &Path,
    structure: Option<&SkillStructure>,
) -> Result<Option<SkillMetadata>, Box<dyn std::error::Error>> {
    let skill_md_path = skill_path.join("SKILL.md");
    if !skill_md_path.exists() {
        log::debug!("SKILL.md not found for skill: {}", skill_path.display());
        return Ok(None);
    }
    warn_if_structure_mismatch(skill_path, structure);

    let content = fs::read_to_string(&skill_md_path)?;
    validate_frontmatter_requirement(&content, skill_path, structure)?;
    let metadata = scanner.parse_skill_md(&content, skill_path)?;
    validate_references_strict(skill_path)
        .map_err(|error| anyhow::anyhow!("reference metadata validation failed: {error}"))?;
    log_scan_success(&metadata);

    Ok(Some(metadata))
}

fn warn_if_structure_mismatch(skill_path: &Path, structure: Option<&SkillStructure>) {
    let Some(structure) = structure else {
        return;
    };
    let report = SkillScanner::validate_structure_report(skill_path, structure);
    if !report.has_findings() {
        return;
    }
    for line in report.render_for_skill(skill_path) {
        log::warn!("{line}");
    }
    log::warn!(
        "skill validation summary json: {}",
        report.to_summary_json_line(skill_path)
    );
}

fn validate_frontmatter_requirement(
    content: &str,
    skill_path: &Path,
    structure: Option<&SkillStructure>,
) -> Result<(), Box<dyn std::error::Error>> {
    let require_yaml_frontmatter = structure
        .is_some_and(|candidate| candidate.validation.frontmatter.require_yaml_frontmatter);
    if require_yaml_frontmatter {
        let parsed = parse_frontmatter_from_markdown(content).map_err(|error| {
            anyhow::anyhow!(
                "SKILL.md frontmatter parse failed for {}: {}",
                skill_path.display(),
                error
            )
        })?;
        let Some(frontmatter) = parsed else {
            return Err(anyhow::anyhow!(
                "SKILL.md frontmatter is required but missing for {}",
                skill_path.display()
            )
            .into());
        };
        validate_gemini_frontmatter_contract(&frontmatter, skill_path)?;
    }
    Ok(())
}

fn validate_gemini_frontmatter_contract(
    frontmatter: &serde_yaml::Value,
    skill_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(table) = frontmatter.as_mapping() else {
        return Err(anyhow::anyhow!(
            "SKILL.md frontmatter must be a YAML mapping at {}",
            skill_path.display()
        )
        .into());
    };
    let name_key = serde_yaml::Value::String("name".to_string());
    let metadata_key = serde_yaml::Value::String("metadata".to_string());

    let Some(name_value) = table.get(&name_key) else {
        return Err(anyhow::anyhow!(
            "SKILL.md frontmatter must contain top-level `name` at {}",
            skill_path.display()
        )
        .into());
    };
    let name = name_value.as_str().map(str::trim).unwrap_or_default();
    if name.is_empty() {
        return Err(anyhow::anyhow!(
            "SKILL.md frontmatter `name` must be a non-empty string at {}",
            skill_path.display()
        )
        .into());
    }

    let Some(metadata_value) = table.get(&metadata_key) else {
        return Err(anyhow::anyhow!(
            "SKILL.md frontmatter must contain nested `metadata` mapping at {}",
            skill_path.display()
        )
        .into());
    };
    if !metadata_value.is_mapping() {
        return Err(anyhow::anyhow!(
            "SKILL.md frontmatter `metadata` must be a mapping at {}",
            skill_path.display()
        )
        .into());
    }
    Ok(())
}

fn log_scan_success(metadata: &SkillMetadata) {
    log::info!(
        "Scanned skill metadata: {} (v{}) - {} keywords",
        metadata.skill_name,
        metadata.version,
        metadata.routing_keywords.len()
    );
}
