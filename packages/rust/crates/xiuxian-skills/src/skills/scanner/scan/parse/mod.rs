use std::path::Path;

use crate::skills::metadata::SkillMetadata;

use self::extract::{parse_skill_frontmatter, skill_name_from_path};
use self::metadata::build_skill_metadata;
use super::super::SkillScanner;

mod extract;
mod metadata;

impl SkillScanner {
    /// Parse YAML frontmatter from SKILL.md content.
    ///
    /// This is a public method to allow external parsing if needed.
    ///
    /// # Arguments
    ///
    /// * `content` - Raw content of the SKILL.md file
    /// * `skill_path` - Path to the skill directory (for extracting skill name)
    ///
    /// # Errors
    ///
    /// Returns an error if YAML frontmatter cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let content = std::fs::read_to_string("assets/skills/writer/SKILL.md").unwrap();
    /// let metadata = scanner.parse_skill_md(&content, PathBuf::from("writer")).unwrap();
    /// ```
    pub fn parse_skill_md(
        &self,
        content: &str,
        skill_path: &Path,
    ) -> Result<SkillMetadata, Box<dyn std::error::Error>> {
        let _ = self;
        let skill_name = skill_name_from_path(skill_path);
        let frontmatter_data = parse_skill_frontmatter(content, &skill_name)?;
        Ok(build_skill_metadata(skill_name, frontmatter_data))
    }
}
