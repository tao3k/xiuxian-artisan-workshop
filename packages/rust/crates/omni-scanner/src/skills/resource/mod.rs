//! Resource Scanner - Parses Python scripts for @`skill_resource` decorated functions.
//!
//! Uses `TreeSitterPythonParser` for robust decorator extraction.

mod scan;

#[cfg(test)]
mod tests;

/// Scanner for @`skill_resource` decorated functions.
#[derive(Debug)]
pub struct ResourceScanner;
