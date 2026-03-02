use super::ResourceScanner;

mod build;
mod filesystem;
mod paths;

impl Default for ResourceScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceScanner {
    /// Create a new resource scanner.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}
