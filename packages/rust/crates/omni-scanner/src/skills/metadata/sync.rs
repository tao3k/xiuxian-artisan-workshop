use std::path::PathBuf;

use super::{IndexToolEntry, ToolRecord};

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

/// Calculate sync operations between scanned tools and existing index.
///
/// Uses `file_hash` for fast-path comparison to skip unchanged tools.
///
/// Args:
///   scanned: Vector of scanned `ToolRecord` objects
///   existing: Vector of existing `IndexToolEntry` objects
///
/// Returns:
///   `SyncReport` with lists of added, updated, deleted, and unchanged tools.
#[must_use]
pub fn calculate_sync_ops(scanned: Vec<ToolRecord>, existing: &[IndexToolEntry]) -> SyncReport {
    let mut report = SyncReport::new();

    // Build a map of existing tools by name for quick lookup
    let existing_map: std::collections::HashMap<String, &IndexToolEntry> = existing
        .iter()
        .map(|tool| (tool.name.clone(), tool))
        .collect();

    // Track which existing tools were matched
    let mut matched_existing: std::collections::HashSet<String> = std::collections::HashSet::new();

    for tool in scanned {
        let tool_name = format!("{}.{}", tool.skill_name, tool.tool_name);

        if let Some(existing_tool) = existing_map.get(&tool_name) {
            // Tool exists - check if it changed via file_hash
            if tool.file_hash == existing_tool.file_hash {
                // Unchanged - fast path
                report.unchanged_count += 1;
            } else {
                // Changed - needs update
                report.updated.push(tool);
            }
            matched_existing.insert(tool_name);
        } else {
            // New tool - needs to be added
            report.added.push(tool);
        }
    }

    // Find deleted tools (in existing but not in scanned)
    for (tool_name, _) in existing_map {
        if !matched_existing.contains(&tool_name) {
            report.deleted.push(tool_name);
        }
    }

    report
}
