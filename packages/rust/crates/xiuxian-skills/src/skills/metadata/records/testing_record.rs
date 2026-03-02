use serde::{Deserialize, Serialize};

/// Represents a test file discovered in a skill.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TestRecord {
    /// Name of the test.
    pub test_name: String,
    /// Skill this test belongs to.
    pub skill_name: String,
    /// Path to the test file.
    pub file_path: String,
    /// Names of test functions.
    pub test_functions: Vec<String>,
    /// Names of test classes.
    pub test_classes: Vec<String>,
    /// Docstring of the test module.
    #[serde(default)]
    pub docstring: String,
    /// Keywords for test discovery.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Hash of the test file.
    #[serde(default)]
    pub file_hash: String,
}

impl TestRecord {
    /// Creates a new `TestRecord` with required fields.
    #[must_use]
    pub fn new(
        test_name: String,
        skill_name: String,
        file_path: String,
        test_functions: Vec<String>,
        test_classes: Vec<String>,
    ) -> Self {
        Self {
            test_name,
            skill_name,
            file_path,
            test_functions,
            test_classes,
            docstring: String::new(),
            keywords: Vec::new(),
            file_hash: String::new(),
        }
    }
}
