//! Integration tests for Cargo dependency parsing.

use std::io::Write as StdWrite;

use tempfile::NamedTempFile;
use xiuxian_wendao::dependency_indexer::parse_cargo_dependencies;

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_parse_workspace_dependencies() -> TestResult {
    let content = r#"
[workspace]
members = ["crates/*"]

[workspace.dependencies]
tokio = { version = "1.49.0", features = ["full"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
anyhow = "1.0.100"
thiserror = "2.0.17"
"#;

    let mut file = NamedTempFile::new()?;
    file.write_all(content.as_bytes())?;
    let path = file.path().to_path_buf();

    let deps = parse_cargo_dependencies(&path)?;

    assert!(deps.iter().any(|d| d.name == "tokio"), "tokio not found");
    assert!(deps.iter().any(|d| d.name == "serde"), "serde not found");
    assert!(deps.iter().any(|d| d.name == "anyhow"), "anyhow not found");
    assert_eq!(
        deps.iter()
            .find(|d| d.name == "serde")
            .map(|d| d.version.as_str()),
        Some("1.0.228")
    );
    Ok(())
}

#[tokio::test]
async fn test_parse_regular_dependencies() -> TestResult {
    let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
anyhow = "1.0"
thiserror = "1.0"
"#;

    let mut file = NamedTempFile::new()?;
    file.write_all(content.as_bytes())?;
    let path = file.path().to_path_buf();

    let deps = parse_cargo_dependencies(&path)?;

    assert!(deps.iter().any(|d| d.name == "serde"), "serde not found");
    assert!(deps.iter().any(|d| d.name == "anyhow"), "anyhow not found");
    Ok(())
}
