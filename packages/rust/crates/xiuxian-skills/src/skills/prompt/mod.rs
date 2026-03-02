//! Prompt Scanner - Parses Python scripts for @prompt decorated functions.
//!
//! Uses `TreeSitterPythonParser` for robust decorator extraction.

mod scan;

/// Scanner for @prompt decorated functions.
#[derive(Debug)]
pub struct PromptScanner;
