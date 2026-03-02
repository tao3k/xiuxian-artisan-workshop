use std::collections::{HashMap, HashSet};

use crate::skills::metadata::{IndexToolEntry, ToolRecord};

use super::SyncReport;

/// Calculate sync operations between scanned tools and existing index.
///
/// Uses `file_hash` for fast-path comparison to skip unchanged tools.
///
/// Args:
///   scanned: Vector of scanned `ToolRecord` objects
///   existing: Slice of existing `IndexToolEntry` objects
///
/// Returns:
///   `SyncReport` with lists of added, updated, deleted, and unchanged tools.
#[must_use]
pub fn calculate_sync_ops(scanned: Vec<ToolRecord>, existing: &[IndexToolEntry]) -> SyncReport {
    let mut report = SyncReport::new();
    let existing_map = existing_tools_by_name(existing);
    let mut matched_existing = HashSet::new();

    for tool in scanned {
        let tool_name = scanned_tool_name(&tool);
        if let Some(existing_tool) = existing_map.get(&tool_name) {
            if tool.file_hash == existing_tool.file_hash {
                report.unchanged_count += 1;
            } else {
                report.updated.push(tool);
            }
            matched_existing.insert(tool_name);
        } else {
            report.added.push(tool);
        }
    }

    for existing_tool in existing {
        if !matched_existing.contains(&existing_tool.name) {
            report.deleted.push(existing_tool.name.clone());
        }
    }

    report
}

fn existing_tools_by_name(existing: &[IndexToolEntry]) -> HashMap<String, &IndexToolEntry> {
    existing
        .iter()
        .map(|tool| (tool.name.clone(), tool))
        .collect()
}

fn scanned_tool_name(tool: &ToolRecord) -> String {
    format!("{}.{}", tool.skill_name, tool.tool_name)
}
