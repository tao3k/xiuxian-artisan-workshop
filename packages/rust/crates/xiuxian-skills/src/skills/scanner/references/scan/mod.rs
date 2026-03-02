use std::path::Path;

use crate::skills::metadata::ReferenceRecord;

use super::model::ReferenceMetadataBlock;

mod build;
mod filesystem;

/// Scan `references/` under a skill directory and return reference records.
/// Parses YAML frontmatter for `for_tools`; `for_skills` and `skill_name`
/// are derived from tool full names.
pub(super) fn scan_references(skill_path: &Path, skill_name: &str) -> Vec<ReferenceRecord> {
    let paths = filesystem::discover_reference_markdown_files(skill_path);
    let records: Vec<ReferenceRecord> = paths
        .iter()
        .filter_map(|path| scan_reference_file(path.as_path(), skill_name))
        .collect();

    if log::log_enabled!(log::Level::Debug) && !records.is_empty() {
        log::debug!(
            "Scanned {} reference(s) for skill {}",
            records.len(),
            skill_name
        );
    }

    records
}

fn scan_reference_file(path: &Path, skill_name: &str) -> Option<ReferenceRecord> {
    let content = filesystem::read_reference_content(path)?;
    let (reference_name, file_path) = filesystem::reference_identity(path);
    let metadata: Option<ReferenceMetadataBlock> =
        match build::parse_reference_metadata_strict(content.as_str(), path) {
            Ok(metadata) => Some(metadata),
            Err(error) => {
                log::warn!("{error}");
                return None;
            }
        };

    Some(build::build_reference_record(
        reference_name,
        file_path,
        skill_name,
        metadata.as_ref(),
    ))
}

pub(super) fn validate_references_strict(skill_path: &Path) -> Result<(), String> {
    let paths = filesystem::discover_reference_markdown_files(skill_path);
    for path in &paths {
        let Some(content) = filesystem::read_reference_content(path) else {
            return Err(format!(
                "failed to read reference markdown: {}",
                path.display()
            ));
        };
        build::parse_reference_metadata_strict(content.as_str(), path)?;
    }
    Ok(())
}
