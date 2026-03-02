use std::path::PathBuf;

/// Configuration for scanning skills.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Path to the skills directory.
    pub skills_dir: PathBuf,
    /// Whether to include optional items in the scan.
    pub include_optional: bool,
    /// Whether to skip structure validation.
    pub skip_validation: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            skills_dir: PathBuf::from("assets/skills"),
            include_optional: true,
            skip_validation: false,
        }
    }
}

impl ScanConfig {
    /// Creates a new `ScanConfig` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the skills directory path.
    #[must_use]
    pub fn with_skills_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.skills_dir = dir.into();
        self
    }
}
