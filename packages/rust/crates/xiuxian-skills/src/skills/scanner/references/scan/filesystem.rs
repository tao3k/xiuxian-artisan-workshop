use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn discover_reference_markdown_files(skill_path: &Path) -> Vec<PathBuf> {
    let refs_dir = skill_path.join("references");
    if !refs_dir.is_dir() {
        return Vec::new();
    }

    let read_dir = match fs::read_dir(&refs_dir) {
        Ok(entries) => entries,
        Err(error) => {
            log::debug!(
                "Could not read references dir {}: {error}",
                refs_dir.display()
            );
            return Vec::new();
        }
    };

    read_dir
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .collect()
}

pub(super) fn read_reference_content(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

pub(super) fn reference_identity(path: &Path) -> (String, String) {
    let reference_name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown")
        .to_string();
    let file_path = path.to_string_lossy().to_string();
    (reference_name, file_path)
}
