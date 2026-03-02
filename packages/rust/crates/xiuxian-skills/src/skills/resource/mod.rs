//! Resource Scanner - Parses Python scripts for @`skill_resource` decorated functions.
//!
//! Uses `TreeSitterPythonParser` for robust decorator extraction.

mod scan;

/// Scanner for @`skill_resource` decorated functions.
#[derive(Debug)]
pub struct ResourceScanner;
