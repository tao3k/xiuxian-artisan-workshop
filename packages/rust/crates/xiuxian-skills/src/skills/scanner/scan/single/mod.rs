use std::path::Path;

use crate::skills::metadata::{SkillMetadata, SkillStructure};

use self::core::scan_skill_result;
use super::super::SkillScanner;

mod core;

impl SkillScanner {
    /// Scan a single skill directory and extract its metadata.
    ///
    /// Returns `Ok(Some(metadata))` if SKILL.md is found and valid.
    /// Returns `Ok(None)` if SKILL.md is missing.
    /// Returns `Err(...)` if SKILL.md exists but cannot be parsed.
    ///
    /// # Arguments
    ///
    /// * `skill_path` - Path to the skill directory (e.g., "assets/skills/writer")
    /// * `structure` - Optional skill structure for validation (uses default if None)
    ///
    /// # Errors
    ///
    /// Returns an error if `SKILL.md` cannot be read or parsed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let scanner = SkillScanner::new();
    /// let metadata = scanner.scan_skill(PathBuf::from("assets/skills/writer"), None).unwrap();
    ///
    /// match metadata {
    ///     Some(m) => println!("Found skill: {}", m.skill_name),
    ///     None => println!("No SKILL.md found"),
    /// }
    /// ```
    pub fn scan_skill(
        &self,
        skill_path: &Path,
        structure: Option<&SkillStructure>,
    ) -> Result<Option<SkillMetadata>, Box<dyn std::error::Error>> {
        scan_skill_result(self, skill_path, structure)
    }
}
