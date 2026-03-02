use super::KnowledgeScanner;

mod core;
mod filters;

impl KnowledgeScanner {
    /// Create a new knowledge scanner with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for KnowledgeScanner {
    fn default() -> Self {
        Self::new()
    }
}
