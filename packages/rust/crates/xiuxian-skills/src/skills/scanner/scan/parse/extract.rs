use std::path::Path;

use crate::frontmatter::extract_frontmatter;

use super::super::super::frontmatter::SkillFrontmatter;

pub(super) fn skill_name_from_path(skill_path: &Path) -> String {
    skill_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

pub(super) fn parse_skill_frontmatter(
    content: &str,
    skill_name: &str,
) -> Result<Option<SkillFrontmatter>, Box<dyn std::error::Error>> {
    let Some(frontmatter) = extract_frontmatter(content) else {
        log::warn!("No YAML frontmatter found in SKILL.md for: {skill_name}");
        return Ok(None);
    };

    let frontmatter_data: SkillFrontmatter = serde_yaml::from_str(&frontmatter)
        .map_err(|error| anyhow::anyhow!("Failed to parse SKILL.md frontmatter: {error}"))?;
    Ok(Some(frontmatter_data))
}
