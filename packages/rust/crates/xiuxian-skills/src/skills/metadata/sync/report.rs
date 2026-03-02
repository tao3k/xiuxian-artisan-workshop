use crate::skills::metadata::ToolRecord;

/// Report of sync operations between scanned tools and existing index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncReport {
    /// Tools that are new and need to be added.
    pub added: Vec<ToolRecord>,
    /// Tools that have changed and need to be updated.
    pub updated: Vec<ToolRecord>,
    /// Tool names that were deleted.
    pub deleted: Vec<String>,
    /// Count of unchanged tools (fast path hit).
    pub unchanged_count: usize,
}

impl SyncReport {
    /// Creates a new empty `SyncReport`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
