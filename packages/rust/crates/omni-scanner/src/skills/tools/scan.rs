use std::path::Path;

use walkdir::WalkDir;

use crate::skills::metadata::{SkillStructure, ToolRecord};

use super::ToolsScanner;

fn should_skip_script_file(path: &Path) -> bool {
    if path.is_dir() {
        return true;
    }

    if path.extension().is_none_or(|ext| ext != "py") {
        return true;
    }

    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    file_name == "__init__.py" || file_name.starts_with('_')
}

fn should_skip_virtual_file(file_path: &str) -> bool {
    let path = Path::new(file_path);
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if file_name == "__init__.py" || file_name.starts_with('_') {
        return true;
    }

    !path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("py"))
}

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
        let mut tools = Vec::new();

        if !scripts_dir.exists() {
            log::debug!("Scripts directory not found: {}", scripts_dir.display());
            return Ok(tools);
        }

        for entry in WalkDir::new(scripts_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if should_skip_script_file(path) {
                continue;
            }

            let parsed_tools =
                self.parse_script(path, skill_name, skill_keywords, skill_intents)?;
            if !parsed_tools.is_empty() {
                log::debug!(
                    "ToolsScanner: Found {} tools in {}",
                    parsed_tools.len(),
                    path.display()
                );
            }
            tools.extend(parsed_tools);
        }

        if !tools.is_empty() {
            log::info!(
                "Scanned {} tools from {} for skill '{skill_name}'",
                tools.len(),
                scripts_dir.display()
            );
        }

        Ok(tools)
    }

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

    /// Scan a list of virtual file paths with their content.
    ///
    /// This method allows scanning files without filesystem access, which is
    /// useful for:
    /// - Testing with temporary directories (no cleanup needed)
    /// - Processing file content from databases or APIs
    /// - Batch scanning with full control over file content
    ///
    /// # Arguments
    ///
    /// * `files` - Vector of tuples: (`file_path`: String, content: String)
    /// * `skill_name` - Name of the parent skill
    /// * `skill_keywords` - Routing keywords from SKILL.md
    /// * `skill_intents` - Intents from SKILL.md
    ///
    /// # Returns
    ///
    /// A vector of `ToolRecord` objects from all scanned files.
    ///
    /// # Errors
    ///
    /// Returns an error when parsing any file content fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let scanner = ToolsScanner::new();
    /// let files = vec![
    ///     ("/tmp/skill/scripts/tool_a.py", r#"
    ///         @skill_command(name="tool_a")
    ///         def tool_a(param: str) -> str:
    ///             '''Tool A implementation.'''
    ///             return param
    ///     "#.to_string()),
    ///     ("/tmp/skill/scripts/tool_b.py", r#"
    ///         @skill_command(name="tool_b")
    ///         def tool_b(value: int) -> int:
    ///             '''Tool B implementation.'''
    ///             value * 2
    ///     "#.to_string()),
    /// ];
    ///
    /// let tools = scanner.scan_paths(&files, "test_skill", &[], &[])?;
    /// ```
    pub fn scan_paths(
        &self,
        files: &[(String, String)],
        skill_name: &str,
        skill_keywords: &[String],
        skill_intents: &[String],
    ) -> Result<Vec<ToolRecord>, Box<dyn std::error::Error>> {
        let mut all_tools = Vec::new();

        for (file_path, content) in files {
            if should_skip_virtual_file(file_path) {
                continue;
            }

            let parsed_tools = self.parse_content(
                content,
                file_path,
                skill_name,
                skill_keywords,
                skill_intents,
            )?;

            if !parsed_tools.is_empty() {
                log::debug!(
                    "ToolsScanner: Found {} tools in {}",
                    parsed_tools.len(),
                    file_path
                );
            }

            all_tools.extend(parsed_tools);
        }

        if !all_tools.is_empty() {
            log::info!(
                "Scanned {} tools from {} files for skill '{}'",
                all_tools.len(),
                files.len(),
                skill_name
            );
        }

        Ok(all_tools)
    }
}
