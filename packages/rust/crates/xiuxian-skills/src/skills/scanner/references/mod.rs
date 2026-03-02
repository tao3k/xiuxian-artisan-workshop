//! Reference document scanning for skill directories.
//!
//! Scans `references/*.md` files and maps frontmatter metadata into
//! [`ReferenceRecord`] entries used by skill index and canonical payload builders.

use std::path::Path;

use crate::skills::metadata::ReferenceRecord;

mod model;
mod scan;
mod values;

pub(super) fn scan_references(skill_path: &Path, skill_name: &str) -> Vec<ReferenceRecord> {
    scan::scan_references(skill_path, skill_name)
}

pub(super) fn validate_references_strict(skill_path: &Path) -> Result<(), String> {
    scan::validate_references_strict(skill_path)
}
