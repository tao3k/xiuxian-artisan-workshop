//! Integration tests for `omni-io` discovery helpers.

use std::fs::File;

use omni_io::{DiscoverOptions, discover_files, discover_files_in_dir, should_skip_path};
use tempfile::TempDir;

#[test]
fn test_discover_files_in_dir() -> std::io::Result<()> {
    let temp = TempDir::new()?;
    let dir = temp.path().to_string_lossy().to_string();
    let dir_path = std::path::Path::new(&dir);

    File::create(dir_path.join("test.py"))?;
    File::create(dir_path.join("readme.md"))?;
    File::create(dir_path.join("data.txt"))?;

    let extensions = vec!["py".to_string(), "md".to_string()];
    let files = discover_files_in_dir(&dir, &extensions, 1024 * 1024, true);

    assert_eq!(files.len(), 2, "Expected 2 files, got: {files:?}");
    assert!(
        files.iter().any(|f| f.ends_with("test.py")),
        "Missing test.py in {files:?}"
    );
    assert!(
        files.iter().any(|f| f.ends_with("readme.md")),
        "Missing readme.md in {files:?}"
    );
    Ok(())
}

#[test]
fn test_discover_files_recursive() -> std::io::Result<()> {
    let temp = TempDir::new()?;
    let dir = temp.path().to_string_lossy().to_string();
    let dir_path = std::path::Path::new(&dir);

    File::create(dir_path.join("root.py"))?;
    std::fs::create_dir(dir_path.join("src"))?;
    File::create(dir_path.join("src").join("module.py"))?;
    std::fs::create_dir(dir_path.join("src").join("nested"))?;
    File::create(dir_path.join("src").join("nested").join("deep.py"))?;

    let options = DiscoverOptions {
        extensions: vec!["py".to_string()],
        recursive: true,
        ..Default::default()
    };

    let files = discover_files(&dir, &options);
    assert!(files.len() >= 3, "Expected >=3 files, got: {files:?}");
    assert!(
        files.iter().any(|f| f.ends_with("root.py")),
        "Missing root.py in {files:?}"
    );
    assert!(
        files.iter().any(|f| f.ends_with("module.py")),
        "Missing module.py in {files:?}"
    );
    assert!(
        files.iter().any(|f| f.ends_with("deep.py")),
        "Missing deep.py in {files:?}"
    );
    Ok(())
}

#[test]
fn test_should_skip_path() {
    let skip_dirs = vec!["target".to_string(), "node_modules".to_string()];

    assert!(should_skip_path(
        "/project/target/file.py",
        true,
        &skip_dirs
    ));
    assert!(should_skip_path(
        "/project/node_modules/pkg",
        true,
        &skip_dirs
    ));
    assert!(!should_skip_path("/project/src/main.py", true, &skip_dirs));
    assert!(!should_skip_path("/project/.config.yml", false, &skip_dirs));
    assert!(should_skip_path("/project/.env", true, &skip_dirs));
}

#[test]
fn test_extension_normalization_via_discovery() -> std::io::Result<()> {
    let temp = TempDir::new()?;
    let dir = temp.path().to_string_lossy().to_string();
    let dir_path = std::path::Path::new(&dir);

    File::create(dir_path.join("upper.PY"))?;
    let files = discover_files_in_dir(&dir, &[".py".to_string()], 1024 * 1024, true);
    assert!(
        files.iter().any(|f| f.ends_with("upper.PY")),
        "Expected uppercase extension file to match normalized extension"
    );
    Ok(())
}
