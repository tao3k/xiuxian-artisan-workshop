use std::path::Path;

use super::ResourceScanner;

#[test]
fn test_scan_empty_dir() {
    let scanner = ResourceScanner::new();
    let resources = scanner.scan(Path::new("/nonexistent"), "test").unwrap();
    assert!(resources.is_empty());
}

#[test]
fn test_scan_finds_skill_resource() {
    let scanner = ResourceScanner::new();
    let files = vec![(
        "/virtual/skill/scripts/resource.py".to_string(),
        r#"
@skill_resource(
    name="status",
    description="Get system status",
    resource_uri="omni://skill/test/status"
)
def status_resource():
    '''Returns system status.'''
    return {"status": "ok"}
"#
        .to_string(),
    )];

    let resources = scanner.scan_paths(&files, "test").unwrap();
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].name, "status");
    assert_eq!(resources[0].resource_uri, "omni://skill/test/status");
}

#[test]
fn test_scan_skips_non_resource() {
    let scanner = ResourceScanner::new();
    let files = vec![(
        "/virtual/skill/scripts/command.py".to_string(),
        r#"
@skill_command(name="do_something")
def do_something():
    pass
"#
        .to_string(),
    )];

    let resources = scanner.scan_paths(&files, "test").unwrap();
    assert!(resources.is_empty());
}
