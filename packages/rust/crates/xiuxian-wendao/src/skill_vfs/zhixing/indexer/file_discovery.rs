use crate::skill_vfs::zhixing::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn collect_markdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(dir)
        .map_err(|error| Error::Internal(format!("Failed listing {}: {error}", dir.display())))?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            Error::Internal(format!(
                "Failed reading directory entry in {}: {error}",
                dir.display()
            ))
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
