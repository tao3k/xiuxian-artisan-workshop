use super::*;

#[test]
fn test_skip_hidden_and_directories() -> Result<(), Box<dyn std::error::Error>> {
    use xiuxian_wendao::SyncEngine;

    let temp_dir = TempDir::new()?;

    // Create hidden file/dir
    fs::write(temp_dir.path().join(".hidden.py"), "hidden")?;
    fs::create_dir_all(temp_dir.path().join(".git"))?;
    fs::write(temp_dir.path().join(".git").join("config"), "config")?;

    // Create normal files
    fs::write(temp_dir.path().join("visible.py"), "visible")?;

    let manifest_path = temp_dir.path().join("manifest.json");
    let engine = SyncEngine::new(temp_dir.path(), &manifest_path);
    let files = engine.discover_files();

    // Should not include hidden files (file name starts with .)
    assert!(!files.iter().any(|p| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
    }));
    // Should include visible file
    assert!(
        files
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "visible.py"))
    );
    Ok(())
}
