use crate::skills::metadata::ToolRecord;

use super::super::super::ToolsScanner;
use super::filter::should_skip_virtual_file;

impl ToolsScanner {
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
