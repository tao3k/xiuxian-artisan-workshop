use std::path::Path;

use crate::skills::metadata::{SkillStructure, SkillValidationReport};

use super::SkillScanner;

mod all;
mod parse;
mod single;

impl SkillScanner {
    /// Create a new skill scanner with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Get the default skill structure (from embedded `resources/config/skills.toml`).
    #[must_use]
    pub fn default_structure() -> SkillStructure {
        SkillStructure::default()
    }

    /// Validate a skill directory against the canonical structure.
    ///
    /// Returns `true` if the skill has all required files.
    #[must_use]
    pub fn validate_structure(skill_path: &Path, structure: &SkillStructure) -> bool {
        Self::validate_structure_report(skill_path, structure).valid
    }

    /// Validate a skill directory and return a structured report.
    #[must_use]
    pub fn validate_structure_report(
        skill_path: &Path,
        structure: &SkillStructure,
    ) -> SkillValidationReport {
        structure.validate_skill_path(skill_path)
    }
}

impl Default for SkillScanner {
    fn default() -> Self {
        Self::new()
    }
}
