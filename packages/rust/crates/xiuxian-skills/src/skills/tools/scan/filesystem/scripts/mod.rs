use std::path::Path;

use crate::skills::metadata::ToolRecord;

use self::collect::collect_tools_from_directory;
use super::super::super::ToolsScanner;

mod collect;
mod entries;

impl ToolsScanner {
    /// Scan a scripts directory for @`skill_command` decorated functions.
    ///
    /// # Arguments
    ///
    /// * `scripts_dir` - Path to the scripts directory (e.g., "assets/skills/writer/scripts")
    /// * `skill_name` - Name of the parent skill (e.g., "writer")
    /// * `skill_keywords` - Routing keywords from SKILL.md (used for keyword boosting)
    ///
    /// # Returns
    ///
    /// A vector of `ToolRecord` objects representing discovered tools.
    ///
    /// # Errors
    ///
    /// Returns an error when script parsing fails for any scanned Python file.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let scanner = ToolsScanner::new();
    /// let tools = scanner.scan_scripts(
    ///     PathBuf::from("assets/skills/writer/scripts"),
    ///     "writer",
    ///     &["write", "edit", "polish"]
    /// ).unwrap();
    /// ```
    pub fn scan_scripts(
        &self,
        scripts_dir: &Path,
        skill_name: &str,
        skill_keywords: &[String],
        skill_intents: &[String],
    ) -> Result<Vec<ToolRecord>, Box<dyn std::error::Error>> {
        if !scripts_dir.exists() {
            log::debug!("Scripts directory not found: {}", scripts_dir.display());
            return Ok(Vec::new());
        }

        let tools = collect_tools_from_directory(
            self,
            scripts_dir,
            skill_name,
            skill_keywords,
            skill_intents,
        )?;

        if !tools.is_empty() {
            log::info!(
                "Scanned {} tools from {} for skill '{skill_name}'",
                tools.len(),
                scripts_dir.display()
            );
        }

        Ok(tools)
    }
}
