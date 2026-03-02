//! Tests for config resolve read-through cache behavior.

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use xiuxian_config_core::{ConfigCascadeSpec, resolve_and_merge_toml_with_paths};

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("create parent {}: {error}", parent.display()));
    }
    std::fs::write(path, content)
        .unwrap_or_else(|error| panic!("write fixture {}: {error}", path.display()));
}

fn temp_workspace() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap_or_else(|error| panic!("create temp workspace: {error}"));
    let root = temp.path().to_path_buf();
    std::fs::create_dir_all(root.join(".config/xiuxian-artisan-workshop"))
        .unwrap_or_else(|error| panic!("create .config/xiuxian-artisan-workshop: {error}"));
    (temp, root)
}

fn strict_mode_from_merged(value: &toml::Value) -> Option<bool> {
    value
        .get("validation")
        .and_then(|node| node.get("strict_mode"))
        .and_then(toml::Value::as_bool)
}

#[test]
fn cache_invalidation_reflects_file_changes() {
    let (_temp, root) = temp_workspace();
    let xiuxian_path = root.join(".config/xiuxian-artisan-workshop/xiuxian.toml");
    write_text(
        xiuxian_path.as_path(),
        r#"
[skills.validation]
strict_mode = false
"#,
    );

    let spec = ConfigCascadeSpec::new(
        "skills",
        r#"
[validation]
strict_mode = true
"#,
        "skills.toml",
    );

    let first = resolve_and_merge_toml_with_paths(
        spec,
        Some(root.as_path()),
        Some(root.join(".config").as_path()),
    )
    .unwrap_or_else(|error| panic!("resolve first pass: {error}"));
    assert_eq!(strict_mode_from_merged(&first), Some(false));

    write_text(
        xiuxian_path.as_path(),
        r#"
[skills.validation]
strict_mode = true
"#,
    );

    let second = resolve_and_merge_toml_with_paths(
        spec,
        Some(root.as_path()),
        Some(root.join(".config").as_path()),
    )
    .unwrap_or_else(|error| panic!("resolve second pass after config update: {error}"));
    assert_eq!(strict_mode_from_merged(&second), Some(true));
}

#[test]
fn cache_concurrent_reads_are_stable() {
    let (_temp, root) = temp_workspace();
    write_text(
        root.join(".config/xiuxian-artisan-workshop/xiuxian.toml")
            .as_path(),
        r#"
[skills.validation]
strict_mode = false
"#,
    );

    let spec = ConfigCascadeSpec::new(
        "skills",
        r#"
[validation]
strict_mode = true
"#,
        "skills.toml",
    );
    let config_home = root.join(".config");

    let mut handles = Vec::new();
    for _ in 0..8 {
        let root_clone = root.clone();
        let config_home_clone = config_home.clone();
        handles.push(std::thread::spawn(move || {
            for _ in 0..32 {
                let merged = resolve_and_merge_toml_with_paths(
                    spec,
                    Some(root_clone.as_path()),
                    Some(config_home_clone.as_path()),
                )
                .unwrap_or_else(|error| panic!("resolve in concurrent reader: {error}"));
                assert_eq!(strict_mode_from_merged(&merged), Some(false));
            }
        }));
    }

    for handle in handles {
        handle
            .join()
            .unwrap_or_else(|_| panic!("cache concurrent reader thread panicked"));
    }
}
