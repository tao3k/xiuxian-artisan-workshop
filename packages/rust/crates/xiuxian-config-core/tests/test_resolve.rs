//! Tests for cascading resolver behavior.

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use xiuxian_config_core::{
    ArrayMergeStrategy, ConfigCascadeSpec, ConfigCoreError, resolve_and_merge_toml_with_paths,
};

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

#[test]
fn resolver_skips_orphan_scan_when_orphan_file_is_blank() {
    let (_temp, root) = temp_workspace();

    write_text(
        root.join(".config/xiuxian-artisan-workshop/ignored.toml")
            .as_path(),
        r#"
[validation]
strict_mode = false
"#,
    );
    let spec = ConfigCascadeSpec::new(
        "skills",
        r#"
[validation]
strict_mode = true
"#,
        "",
    );

    let merged = resolve_and_merge_toml_with_paths(
        spec,
        Some(root.as_path()),
        Some(root.join(".config").as_path()),
    )
    .unwrap_or_else(|error| {
        panic!("resolve_and_merge_toml should ignore orphan when disabled: {error}")
    });
    let strict_mode = merged
        .get("validation")
        .and_then(|value| value.get("strict_mode"))
        .and_then(toml::Value::as_bool);

    assert_eq!(strict_mode, Some(true));
}

#[test]
fn resolver_appends_arrays_when_strategy_is_append() {
    let (_temp, root) = temp_workspace();

    write_text(
        root.join(".config/xiuxian-artisan-workshop/xiuxian.toml")
            .as_path(),
        r#"
[skills.items]
values = ["b"]
"#,
    );

    let spec = ConfigCascadeSpec::new(
        "skills",
        r#"
[items]
values = ["a"]
"#,
        "orphan.toml",
    )
    .with_array_merge_strategy(ArrayMergeStrategy::Append);

    let merged = resolve_and_merge_toml_with_paths(
        spec,
        Some(root.as_path()),
        Some(root.join(".config").as_path()),
    )
    .unwrap_or_else(|error| panic!("resolve append strategy config: {error}"));
    let values = merged
        .get("items")
        .and_then(|value| value.get("values"))
        .and_then(toml::Value::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        });

    assert_eq!(values, Some(vec!["a".to_string(), "b".to_string()]));
}

#[test]
fn resolver_overwrites_arrays_when_strategy_is_default() {
    let (_temp, root) = temp_workspace();

    write_text(
        root.join(".config/xiuxian-artisan-workshop/xiuxian.toml")
            .as_path(),
        r#"
[skills.items]
values = ["b"]
"#,
    );

    let spec = ConfigCascadeSpec::new(
        "skills",
        r#"
[items]
values = ["a"]
"#,
        "orphan.toml",
    );

    let merged = resolve_and_merge_toml_with_paths(
        spec,
        Some(root.as_path()),
        Some(root.join(".config").as_path()),
    )
    .unwrap_or_else(|error| panic!("resolve default overwrite strategy config: {error}"));
    let values = merged
        .get("items")
        .and_then(|value| value.get("values"))
        .and_then(toml::Value::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        });

    assert_eq!(values, Some(vec!["b".to_string()]));
}

#[test]
fn resolver_errors_when_global_and_orphan_configs_coexist() {
    let (_temp, root) = temp_workspace();

    write_text(
        root.join(".config/xiuxian-artisan-workshop/xiuxian.toml")
            .as_path(),
        r#"
[skills]
enabled = true
"#,
    );
    write_text(
        root.join(".config/xiuxian-artisan-workshop/skills.toml")
            .as_path(),
        r#"
enabled = false
"#,
    );

    let spec = ConfigCascadeSpec::new(
        "skills",
        r#"
enabled = true
"#,
        "skills.toml",
    );

    let error = resolve_and_merge_toml_with_paths(
        spec,
        Some(root.as_path()),
        Some(root.join(".config").as_path()),
    )
    .expect_err("coexisting global+orphan configs must fail");

    match error {
        ConfigCoreError::RedundantOrphan { namespace, orphans } => {
            assert_eq!(namespace, "skills");
            assert!(orphans.contains("skills.toml"), "orphans={orphans}");
        }
        other => panic!("expected RedundantOrphan, got {other}"),
    }
}

#[test]
fn resolver_supports_dotted_namespace_projection_from_xiuxian_toml() {
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
        "skills.validation",
        r#"
strict_mode = true
"#,
        "skills.toml",
    );

    let merged = resolve_and_merge_toml_with_paths(
        spec,
        Some(root.as_path()),
        Some(root.join(".config").as_path()),
    )
    .unwrap_or_else(|error| panic!("resolve dotted namespace config: {error}"));
    let strict_mode = merged.get("strict_mode").and_then(toml::Value::as_bool);

    assert_eq!(strict_mode, Some(false));
}
