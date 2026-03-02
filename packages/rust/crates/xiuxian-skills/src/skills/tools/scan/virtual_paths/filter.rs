use std::path::Path;

pub(super) fn should_skip_virtual_file(file_path: &str) -> bool {
    let path = Path::new(file_path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    if file_name == "__init__.py" || file_name.starts_with('_') {
        return true;
    }

    !path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("py"))
}
