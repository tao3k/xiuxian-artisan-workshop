use std::path::Path;

use crate::skills::metadata::{SkillStructure, ToolRecord};

use super::super::super::ToolsScanner;

impl ToolsScanner {
    /// Scan a single skill directory (scripts/ subdirectory).
    ///
    /// This is a convenience method that combines finding the scripts directory
    /// and scanning it.
    ///
    /// # Arguments
    ///
    /// * `skill_path` - Path to the skill directory (e.g., "assets/skills/writer")
    /// * `skill_name` - Name of the skill
    /// * `skill_keywords` - Routing keywords from SKILL.md
    /// * `skill_intents` - Intents from SKILL.md
    ///
    /// # Returns
    ///
    /// A vector of `ToolRecord` objects.
    ///
    /// # Errors
    ///
    /// Returns an error when script parsing fails.
    pub fn scan_skill_scripts(
        &self,
        skill_path: &Path,
        skill_name: &str,
        skill_keywords: &[String],
        skill_intents: &[String],
    ) -> Result<Vec<ToolRecord>, Box<dyn std::error::Error>> {
        let scripts_dir = skill_path.join("scripts");
        self.scan_scripts(&scripts_dir, skill_name, skill_keywords, skill_intents)
    }

    /// Scan a skill directory using the canonical skill structure.
    ///
    /// Only scans directories defined in the skill structure's `default` list.
    /// This ensures only intended directories (scripts/, templates/, etc.) are scanned.
    ///
    /// # Arguments
    ///
    /// * `skill_path` - Path to the skill directory
    /// * `skill_name` - Name of the skill
    /// * `skill_keywords` - Routing keywords from SKILL.md
    /// * `skill_intents` - Intents from SKILL.md
    /// * `structure` - Skill structure defining which directories to scan
    ///
    /// # Returns
    ///
    /// A vector of `ToolRecord` objects from all scanned directories.
    ///
    /// # Errors
    ///
    /// Returns an error when scanning any configured scripts directory fails.
    pub fn scan_with_structure(
        &self,
        skill_path: &Path,
        skill_name: &str,
        skill_keywords: &[String],
        skill_intents: &[String],
        structure: &SkillStructure,
    ) -> Result<Vec<ToolRecord>, Box<dyn std::error::Error>> {
        let mut all_tools = Vec::new();
        let script_dirs = structure.script_directories();

        for dir_name in script_dirs {
            let dir_path = skill_path.join(dir_name);
            if dir_path.exists() && dir_path.is_dir() {
                let tools =
                    self.scan_scripts(&dir_path, skill_name, skill_keywords, skill_intents)?;
                all_tools.extend(tools);
            }
        }

        Ok(all_tools)
    }
}
