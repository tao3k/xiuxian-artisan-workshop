use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::super::filters::should_skip_script_file;

pub(super) fn script_paths_in_directory(scripts_dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(scripts_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .map(walkdir::DirEntry::into_path)
        .filter(|path| !should_skip_script_file(path))
        .collect()
}
