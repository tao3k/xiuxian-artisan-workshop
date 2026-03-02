use std::path::Path;

use crate::skills::metadata::ToolRecord;

use super::ToolsScanner;

mod content;
mod decorated;
mod hashing;

use content::read_script_content;
use decorated::parse_decorated_tools;

impl ToolsScanner {
    /// Parse a single script file for tool definitions.
    ///
    /// Uses tree-sitter for robust parsing of @`skill_command` decorated functions
    /// with proper handling of triple-quoted strings in decorator arguments.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Python script file
    /// * `skill_name` - Name of the parent skill
    /// * `skill_keywords` - Routing keywords from SKILL.md
    /// * `skill_intents` - Intents from SKILL.md
    ///
    /// # Returns
    ///
    /// A vector of `ToolRecord` objects with enriched metadata.
    pub(super) fn parse_script(
        &self,
        path: &Path,
        skill_name: &str,
        skill_keywords: &[String],
        skill_intents: &[String],
    ) -> Result<Vec<ToolRecord>, Box<dyn std::error::Error>> {
        let _ = self;
        let content = read_script_content(path)?;
        let file_path = path.to_string_lossy().to_string();
        Ok(parse_decorated_tools(
            &content,
            &file_path,
            skill_name,
            skill_keywords,
            skill_intents,
        ))
    }

    /// Parse script content directly without reading from disk.
    ///
    /// Uses tree-sitter for robust parsing of @`skill_command` decorated functions
    /// with proper handling of triple-quoted strings in decorator arguments.
    ///
    /// # Arguments
    ///
    /// * `content` - The Python script content as a string
    /// * `file_path` - Virtual file path (for metadata/logging only)
    /// * `skill_name` - Name of the parent skill
    /// * `skill_keywords` - Routing keywords from SKILL.md
    /// * `skill_intents` - Intents from SKILL.md
    ///
    /// # Returns
    ///
    /// A vector of `ToolRecord` objects with enriched metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when `file_path` is empty.
    pub fn parse_content(
        &self,
        content: &str,
        file_path: &str,
        skill_name: &str,
        skill_keywords: &[String],
        skill_intents: &[String],
    ) -> Result<Vec<ToolRecord>, Box<dyn std::error::Error>> {
        let _ = self;
        if file_path.trim().is_empty() {
            return Err("file_path cannot be empty".into());
        }

        Ok(parse_decorated_tools(
            content,
            file_path,
            skill_name,
            skill_keywords,
            skill_intents,
        ))
    }
}
