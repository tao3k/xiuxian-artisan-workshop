//! Prompt Scanner - Parses Python scripts for @prompt decorated functions.
//!
//! Uses `TreeSitterPythonParser` for robust decorator extraction.

mod scan;

#[cfg(test)]
mod tests;

/// Scanner for @prompt decorated functions.
#[derive(Debug)]
pub struct PromptScanner;
