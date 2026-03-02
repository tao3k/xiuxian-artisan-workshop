//! Integration tests for `xiuxian_wendao::sync`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use tempfile::TempDir;
use xiuxian_wendao::{
    IncrementalSyncPolicy, SyncEngine, SyncManifest, extract_extensions_from_glob_patterns,
};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_manifest_load_save() -> TestResult {
    let temp_dir = TempDir::new()?;
    let manifest_path = temp_dir.path().join("manifest.json");
    let engine = SyncEngine::new(temp_dir.path(), &manifest_path);

    let mut manifest = SyncManifest(HashMap::new());
    manifest
        .0
        .insert("test.py".to_string(), "hash123".to_string());

    engine.save_manifest(&manifest)?;
    let loaded = engine.load_manifest();

    assert_eq!(loaded.0.get("test.py"), Some(&"hash123".to_string()));
    Ok(())
}

#[test]
fn test_compute_hash() {
    let hash1 = SyncEngine::compute_hash("hello world");
    let hash2 = SyncEngine::compute_hash("hello world");
    let hash3 = SyncEngine::compute_hash("different");

    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
    assert_eq!(hash1.len(), 16);
}

#[test]
fn test_discover_files() -> TestResult {
    let temp_dir = TempDir::new()?;

    fs::write(temp_dir.path().join("test.py"), "print('hello')")?;
    fs::write(temp_dir.path().join("test.md"), "# Hello")?;
    fs::write(temp_dir.path().join("test.txt"), "hello")?;

    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir)?;
    fs::write(subdir.join("module.py"), "def foo(): pass")?;

    let manifest_path = temp_dir.path().join("manifest.json");
    let engine = SyncEngine::new(temp_dir.path(), &manifest_path);
    let files = engine.discover_files();

    assert!(
        files
            .iter()
            .any(|p| p.extension().is_some_and(|e| e == "py"))
    );
    assert!(
        files
            .iter()
            .any(|p| p.extension().is_some_and(|e| e == "md"))
    );
    assert!(
        !files
            .iter()
            .any(|p| p.extension().is_some_and(|e| e == "txt"))
    );
    Ok(())
}

#[test]
fn test_compute_diff() -> TestResult {
    let temp_dir = TempDir::new()?;

    fs::write(temp_dir.path().join("new.py"), "new content")?;
    fs::write(temp_dir.path().join("modified.py"), "modified content")?;
    fs::write(temp_dir.path().join("existing.py"), "existing")?;

    let manifest_path = temp_dir.path().join("manifest.json");
    let engine = SyncEngine::new(temp_dir.path(), &manifest_path);

    let mut old_manifest = SyncManifest(HashMap::new());
    old_manifest.0.insert(
        "existing.py".to_string(),
        SyncEngine::compute_hash("existing"),
    );
    old_manifest
        .0
        .insert("modified.py".to_string(), "old_hash".to_string());

    let files = engine.discover_files();
    let diff = engine.compute_diff(&old_manifest, &files);

    assert!(
        diff.added
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "new.py"))
    );

    assert!(
        diff.modified
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "modified.py"))
    );

    assert_eq!(diff.unchanged, 1);
    Ok(())
}

#[test]
fn test_skip_hidden_and_directories() -> TestResult {
    let temp_dir = TempDir::new()?;

    fs::write(temp_dir.path().join(".hidden.py"), "hidden")?;
    fs::create_dir_all(temp_dir.path().join(".git"))?;
    fs::write(temp_dir.path().join(".git").join("config"), "config")?;

    fs::write(temp_dir.path().join("visible.py"), "visible")?;

    let manifest_path = temp_dir.path().join("manifest.json");
    let engine = SyncEngine::new(temp_dir.path(), &manifest_path);
    let files = engine.discover_files();

    assert!(!files.iter().any(|p| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
    }));
    assert!(
        files
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "visible.py"))
    );
    Ok(())
}

#[test]
fn test_incremental_sync_policy_supports_configured_extensions() {
    let configured_extensions = vec![
        "md".to_string(),
        ".org".to_string(),
        "J2".to_string(),
        "toml".to_string(),
    ];
    let policy = IncrementalSyncPolicy::new(&configured_extensions);

    assert!(policy.supports_path(Path::new("note.md")));
    assert!(policy.supports_path(Path::new("agenda.org")));
    assert!(policy.supports_path(Path::new("template.j2")));
    assert!(policy.supports_path(Path::new("config.toml")));
    assert!(!policy.supports_path(Path::new("README.txt")));
}

#[test]
fn test_extract_extensions_from_glob_patterns() {
    let patterns = vec![
        "**/*.md".to_string(),
        "**/*.org".to_string(),
        "templates/*.j2".to_string(),
        "**/*.toml".to_string(),
        "**/*.{md,markdown}".to_string(),
    ];

    let extensions = extract_extensions_from_glob_patterns(&patterns);
    assert_eq!(
        extensions,
        vec![
            "j2".to_string(),
            "markdown".to_string(),
            "md".to_string(),
            "org".to_string(),
            "toml".to_string()
        ]
    );
}

#[test]
fn test_extract_extensions_from_brace_glob_patterns() {
    let patterns = vec!["**/*.{md,org,j2,toml}".to_string()];
    let extensions = extract_extensions_from_glob_patterns(&patterns);
    assert_eq!(
        extensions,
        vec![
            "j2".to_string(),
            "md".to_string(),
            "org".to_string(),
            "toml".to_string()
        ]
    );
}

#[test]
fn test_extract_extensions_from_compound_suffix_patterns() {
    let patterns = vec![
        "**/*.md.j2".to_string(),
        "**/*.agenda.toml".to_string(),
        "**/*.{org,template.md.j2}".to_string(),
    ];
    let extensions = extract_extensions_from_glob_patterns(&patterns);
    assert_eq!(
        extensions,
        vec!["j2".to_string(), "org".to_string(), "toml".to_string()]
    );
}

#[test]
fn test_incremental_policy_prefers_explicit_extensions_over_patterns() {
    let patterns = vec!["**/*.md".to_string()];
    let explicit = vec!["toml".to_string(), "j2".to_string()];
    let policy = IncrementalSyncPolicy::from_patterns_and_extensions(
        &patterns,
        &explicit,
        &["md", "markdown", "org"],
    );
    assert!(policy.supports_path(Path::new("task.toml")));
    assert!(policy.supports_path(Path::new("template.j2")));
    assert!(!policy.supports_path(Path::new("note.md")));
}
