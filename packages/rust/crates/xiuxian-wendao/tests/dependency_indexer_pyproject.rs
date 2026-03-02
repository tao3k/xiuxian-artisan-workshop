//! Integration tests for `xiuxian_wendao::dependency_indexer::pyproject`.

use std::io::Write as StdWrite;

use tempfile::NamedTempFile;
use xiuxian_wendao::dependency_indexer::parse_pyproject_dependencies;

#[tokio::test]
async fn test_parse_pyproject_dependencies() {
    let content = r#"
[project]
name = "test"
version = "0.1.0"
dependencies = [
    "requests>=2.0",
    "click>=8.0",
    "rich>=13.0",
]
"#;

    let mut file = NamedTempFile::new().unwrap_or_else(|error| {
        panic!("failed to create temp file: {error}");
    });
    file.write_all(content.as_bytes()).unwrap_or_else(|error| {
        panic!("failed to write temp file: {error}");
    });
    let path = file.path().to_path_buf();

    let deps = parse_pyproject_dependencies(&path).unwrap_or_else(|error| {
        panic!("failed to parse pyproject dependencies: {error}");
    });

    assert!(deps.iter().any(|d| d.name == "requests"));
    assert!(deps.iter().any(|d| d.name == "click"));
    assert!(deps.iter().any(|d| d.name == "rich"));
}
