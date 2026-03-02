use std::path::Path;

pub(super) fn should_skip_script_file(path: &Path) -> bool {
    if path.is_dir() {
        return true;
    }

    if path.extension().is_none_or(|extension| extension != "py") {
        return true;
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    file_name == "__init__.py" || file_name.starts_with('_')
}
