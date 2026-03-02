use std::path::{Path, PathBuf};

use super::SyncEngine;

impl SyncEngine {
    /// Discover files in project root.
    #[must_use]
    pub fn discover_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self.discover_recursive(&self.project_root, &mut files);
        files
    }

    fn discover_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if self.should_skip_dir(&path) {
                        continue;
                    }
                    self.discover_recursive(&path, files);
                } else if path.is_file() {
                    // Skip hidden files
                    if self.options.skip_hidden
                        && let Some(name) = path.file_name()
                        && name.to_string_lossy().starts_with('.')
                    {
                        continue;
                    }
                    if self.should_include_file(&path) {
                        files.push(path);
                    }
                }
            }
        }
    }

    fn should_skip_dir(&self, path: &Path) -> bool {
        // Skip hidden directories
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') {
                return true;
            }
        }

        // Skip configured directories
        for skip_dir in &self.options.skip_dirs {
            if let Some(name) = path.file_name()
                && name.to_string_lossy() == *skip_dir
            {
                return true;
            }
        }

        false
    }

    fn should_include_file(&self, path: &Path) -> bool {
        // Check extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if !self.options.extensions.is_empty() && !self.options.extensions.contains(&ext_str) {
                return false;
            }
        } else {
            return false;
        }

        // Check file size
        if let Ok(metadata) = path.metadata()
            && metadata.len() > self.options.max_file_size
        {
            return false;
        }

        true
    }
}
