//! Script Scanner - Parses Python scripts for @`skill_command` decorated functions.
//!
//! Uses tree-sitter based parsing to find functions decorated with
//! `@skill_command` in skill script directories.

mod parse;
mod scan;
mod schema;

/// Scans Python script surfaces for `@skill_command` tool definitions.
#[derive(Debug)]
pub struct ToolsScanner;

impl ToolsScanner {
    /// Create a new script scanner.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToolsScanner {
    fn default() -> Self {
        Self::new()
    }
}
