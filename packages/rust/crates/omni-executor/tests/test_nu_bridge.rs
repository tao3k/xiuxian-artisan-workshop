//! Integration tests for Nushell system bridge.

use omni_executor::{ActionType, NuConfig, NuSystemBridge};
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn must_ok<T, E: Display>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn must_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(inner) => inner,
        None => panic!("{context}"),
    }
}

fn create_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos));
    must_ok(fs::create_dir_all(&dir), "failed to create temp dir");
    dir
}

#[test]
fn test_new_bridge_has_default_config() {
    let bridge = NuSystemBridge::new();
    assert_eq!(bridge.config.nu_path, "nu");
    assert!(bridge.config.enable_shellcheck);
}

#[test]
fn test_bridge_with_custom_config() {
    let config = NuConfig {
        nu_path: "/usr/bin/nu".to_string(),
        enable_shellcheck: false,
        ..Default::default()
    };
    let bridge = NuSystemBridge::with_config(config);

    assert_eq!(bridge.config.nu_path, "/usr/bin/nu");
    assert!(!bridge.config.enable_shellcheck);
}

#[test]
fn test_classify_action_ls() {
    assert_eq!(NuSystemBridge::classify_action("ls"), ActionType::Observe);
    assert_eq!(
        NuSystemBridge::classify_action("ls -la"),
        ActionType::Observe
    );
}

#[test]
fn test_classify_action_cat() {
    assert_eq!(
        NuSystemBridge::classify_action("cat file.txt"),
        ActionType::Observe
    );
}

#[test]
fn test_classify_action_rm() {
    assert_eq!(
        NuSystemBridge::classify_action("rm file.txt"),
        ActionType::Mutate
    );
}

#[test]
fn test_classify_action_cp() {
    assert_eq!(
        NuSystemBridge::classify_action("cp a b"),
        ActionType::Mutate
    );
}

#[test]
fn test_classify_action_mv() {
    assert_eq!(
        NuSystemBridge::classify_action("mv old new"),
        ActionType::Mutate
    );
}

#[test]
fn test_classify_action_mkdir() {
    assert_eq!(
        NuSystemBridge::classify_action("mkdir -p dir"),
        ActionType::Mutate
    );
}

#[test]
fn test_classify_action_echo() {
    assert_eq!(
        NuSystemBridge::classify_action("echo hello"),
        ActionType::Mutate
    );
}

#[test]
fn test_classify_action_with_pipe() {
    assert_eq!(
        NuSystemBridge::classify_action("ls | grep txt"),
        ActionType::Observe
    );
    assert_eq!(
        NuSystemBridge::classify_action("cat file.txt | wc -l"),
        ActionType::Observe
    );
}

#[test]
fn test_validate_safety_allows_safe_commands() {
    let bridge = NuSystemBridge::new();

    assert!(bridge.validate_safety("ls -la").is_ok());
    assert!(bridge.validate_safety("cat config.toml").is_ok());
    assert!(bridge.validate_safety("pwd").is_ok());
    assert!(bridge.validate_safety("echo hello").is_ok());
}

#[test]
fn test_validate_safety_blocks_dangerous() {
    let bridge = NuSystemBridge::new();

    assert!(bridge.validate_safety("rm -rf /").is_err());
    assert!(bridge.validate_safety("mkfs.ext4 /dev/sda").is_err());
}

#[test]
fn test_validate_safety_blocks_fork_bomb() {
    let bridge = NuSystemBridge::new();

    assert!(bridge.validate_safety(":(){ :|:& };:").is_err());
}

#[test]
fn test_validate_safety_repeated_command_is_stable() {
    let bridge = NuSystemBridge::new();
    let cmd = "echo hello";

    assert!(bridge.validate_safety(cmd).is_ok());
    assert!(bridge.validate_safety(cmd).is_ok());
}

#[test]
fn test_config_default_values() {
    let config = NuConfig::default();

    assert_eq!(config.nu_path, "nu");
    assert!(config.no_config);
    assert!(config.enable_shellcheck);
    assert!(config.allowed_commands.is_empty());
}

#[test]
fn test_config_with_whitelist() {
    let config = NuConfig {
        allowed_commands: vec!["ls".to_string(), "cat".to_string()],
        ..Default::default()
    };
    let bridge = NuSystemBridge::with_config(config);

    assert!(bridge.validate_safety("ls file.txt").is_ok());
    assert!(bridge.validate_safety("cat file.txt").is_ok());
    assert!(bridge.validate_safety("rm file.txt").is_err());
}

#[test]
fn test_action_type_variants() {
    assert_eq!(ActionType::Observe, ActionType::Observe);
    assert_eq!(ActionType::Mutate, ActionType::Mutate);
    assert_ne!(ActionType::Observe, ActionType::Mutate);
}

#[test]
fn test_execute_observe_ls_fast_path_works_without_nu_binary() {
    let bridge = NuSystemBridge::with_config(NuConfig {
        nu_path: "/path/that/does/not/exist/nu".to_string(),
        enable_shellcheck: false,
        ..Default::default()
    });

    let result = bridge.execute_with_action("ls .", ActionType::Observe, true);
    assert!(result.is_ok());
    assert!(must_ok(result, "ls fast-path should succeed").is_array());
}

#[test]
fn test_execute_observe_ls_fast_path_hides_dotfiles_by_default() {
    let temp_dir = create_temp_dir("omni_executor_ls_default");
    must_ok(
        fs::write(temp_dir.join("visible.txt"), b"visible"),
        "failed to create visible file",
    );
    must_ok(
        fs::write(temp_dir.join(".hidden.txt"), b"hidden"),
        "failed to create hidden file",
    );

    let bridge = NuSystemBridge::with_config(NuConfig {
        nu_path: "/path/that/does/not/exist/nu".to_string(),
        enable_shellcheck: false,
        ..Default::default()
    });
    let command = format!("ls {}", temp_dir.display());
    let response = must_ok(
        bridge.execute_with_action(&command, ActionType::Observe, true),
        "ls fast-path should succeed",
    );
    let rows = must_some(
        response.as_array().cloned(),
        "ls fast-path should return array",
    );

    let names: Vec<String> = rows
        .iter()
        .filter_map(|row| {
            row.get("name")
                .and_then(|v| v.as_str())
                .map(ToOwned::to_owned)
        })
        .collect();
    assert!(names.iter().any(|name| name == "visible.txt"));
    assert!(!names.iter().any(|name| name == ".hidden.txt"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_execute_observe_ls_fast_path_can_include_dotfiles() {
    let temp_dir = create_temp_dir("omni_executor_ls_all");
    must_ok(
        fs::write(temp_dir.join("visible.txt"), b"visible"),
        "failed to create visible file",
    );
    must_ok(
        fs::write(temp_dir.join(".hidden.txt"), b"hidden"),
        "failed to create hidden file",
    );

    let bridge = NuSystemBridge::with_config(NuConfig {
        nu_path: "/path/that/does/not/exist/nu".to_string(),
        enable_shellcheck: false,
        ..Default::default()
    });
    let command = format!("ls -a {}", temp_dir.display());
    let response = must_ok(
        bridge.execute_with_action(&command, ActionType::Observe, true),
        "ls fast-path should succeed",
    );
    let rows = must_some(
        response.as_array().cloned(),
        "ls fast-path should return array",
    );

    let names: Vec<String> = rows
        .iter()
        .filter_map(|row| {
            row.get("name")
                .and_then(|v| v.as_str())
                .map(ToOwned::to_owned)
        })
        .collect();
    assert!(names.iter().any(|name| name == "visible.txt"));
    assert!(names.iter().any(|name| name == ".hidden.txt"));

    let _ = fs::remove_dir_all(&temp_dir);
}
