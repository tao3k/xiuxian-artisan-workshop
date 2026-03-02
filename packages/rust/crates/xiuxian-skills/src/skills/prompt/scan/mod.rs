use super::PromptScanner;

mod build;
mod filesystem;
mod paths;

impl Default for PromptScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptScanner {
    /// Create a new prompt scanner.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}
