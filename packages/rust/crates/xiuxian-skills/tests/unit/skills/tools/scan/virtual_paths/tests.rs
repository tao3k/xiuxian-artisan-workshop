use super::filter::should_skip_virtual_file;

#[test]
fn test_should_skip_virtual_file_for_init_and_private() {
    assert!(should_skip_virtual_file("scripts/__init__.py"));
    assert!(should_skip_virtual_file("scripts/_internal.py"));
}

#[test]
fn test_should_skip_virtual_file_for_non_python_extension() {
    assert!(should_skip_virtual_file("scripts/tool.txt"));
    assert!(should_skip_virtual_file("scripts/tool"));
}

#[test]
fn test_should_not_skip_virtual_file_for_python_extensions() {
    assert!(!should_skip_virtual_file("scripts/tool.py"));
    assert!(!should_skip_virtual_file("scripts/tool.PY"));
}
