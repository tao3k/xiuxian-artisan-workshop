//! Telegram ACL override loading and policy projection tests.

use std::path::PathBuf;

use omni_agent::{
    Channel, TelegramAclOverrides, TelegramChannel, TelegramControlCommandPolicy,
    TelegramSessionPartition, build_telegram_acl_overrides, load_runtime_settings_from_paths,
};
use tempfile::TempDir;

const STRUCTURED_ACL_TOML: &str = r#"
[telegram.acl.allow]
users = ["1001", "1002"]
groups = ["-2001", "*"]

[telegram.acl.admin]
users = ["1001"]

[telegram.acl.control.allow_from]
users = ["1001", "ops"]

[[telegram.acl.control.rules]]
commands = ["/session partition"]

[telegram.acl.control.rules.allow]
users = ["1001"]

[[telegram.acl.control.rules]]
commands = ["/reset", "/clear"]

[telegram.acl.control.rules.allow]
users = ["2001"]

[telegram.acl.slash.global]
users = ["1001", "ops"]

[telegram.acl.slash.session_status]
users = ["observer"]

[telegram.acl.slash.session_budget]
users = ["observer"]

[telegram.acl.slash.session_memory]
users = ["editor"]

[telegram.acl.slash.session_feedback]
users = ["editor"]

[telegram.acl.slash.job_status]
users = ["runner"]

[telegram.acl.slash.jobs_summary]
users = ["runner"]

[telegram.acl.slash.background_submit]
users = ["runner"]
"#;

fn write_file(path: PathBuf, content: &str) {
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        panic!("create parent dir: {error}");
    }
    if let Err(error) = std::fs::write(path, content) {
        panic!("write toml: {error}");
    }
}

fn assert_control_rule_projection(overrides: &TelegramAclOverrides) {
    assert_eq!(overrides.control_command_rules.len(), 2);
    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        TelegramControlCommandPolicy::new(
            vec!["9001".to_string()],
            None,
            overrides.control_command_rules.clone(),
        ),
        TelegramSessionPartition::ChatUser,
    );
    assert!(channel.is_authorized_for_control_command("1001", "/session partition"));
    assert!(channel.is_authorized_for_control_command("2001", "/reset"));
    assert!(
        !channel.is_authorized_for_control_command("9001", "/session partition"),
        "matched command-scoped rule should override admin fallback",
    );
}

fn assert_slash_acl_projection(overrides: &TelegramAclOverrides) {
    assert_eq!(
        overrides.slash_command_allow_from,
        Some(vec!["1001".to_string(), "ops".to_string()])
    );
    assert_eq!(
        overrides.slash_session_status_allow_from,
        Some(vec!["observer".to_string()])
    );
    assert_eq!(
        overrides.slash_session_budget_allow_from,
        Some(vec!["observer".to_string()])
    );
    assert_eq!(
        overrides.slash_session_memory_allow_from,
        Some(vec!["editor".to_string()])
    );
    assert_eq!(
        overrides.slash_session_feedback_allow_from,
        Some(vec!["editor".to_string()])
    );
    assert_eq!(
        overrides.slash_job_allow_from,
        Some(vec!["runner".to_string()])
    );
    assert_eq!(
        overrides.slash_jobs_allow_from,
        Some(vec!["runner".to_string()])
    );
    assert_eq!(
        overrides.slash_bg_allow_from,
        Some(vec!["runner".to_string()])
    );
}

#[test]
fn telegram_acl_overrides_build_from_structured_acl() {
    let tmp = match TempDir::new() {
        Ok(tmp) => tmp,
        Err(error) => panic!("tempdir: {error}"),
    };
    let system = tmp
        .path()
        .join("packages/rust/crates/omni-agent/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(system.clone(), STRUCTURED_ACL_TOML);
    write_file(user.clone(), "");

    let settings = load_runtime_settings_from_paths(&system, &user);
    let overrides = match build_telegram_acl_overrides(&settings) {
        Ok(overrides) => overrides,
        Err(error) => panic!("telegram acl overrides: {error}"),
    };

    assert_eq!(overrides.allowed_users, vec!["1001", "1002"]);
    assert_eq!(overrides.allowed_groups, vec!["-2001", "*"]);
    assert_eq!(overrides.admin_users, vec!["1001"]);
    assert_eq!(
        overrides.control_command_allow_from,
        Some(vec!["1001".to_string(), "ops".to_string()])
    );
    assert_control_rule_projection(&overrides);
    assert_slash_acl_projection(&overrides);
}

#[test]
fn telegram_acl_overrides_use_user_settings_for_acl_merge() {
    let tmp = match TempDir::new() {
        Ok(tmp) => tmp,
        Err(error) => panic!("tempdir: {error}"),
    };
    let system = tmp
        .path()
        .join("packages/rust/crates/omni-agent/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        system.clone(),
        r#"
[telegram.acl.allow]
users = ["1001"]
groups = ["-2001"]

[telegram.acl.admin]
users = ["1001"]
"#,
    );
    write_file(
        user.clone(),
        r#"
[telegram.acl.allow]
users = ["2002"]

[telegram.acl.admin]
users = ["2002"]
"#,
    );

    let settings = load_runtime_settings_from_paths(&system, &user);
    let overrides = match build_telegram_acl_overrides(&settings) {
        Ok(overrides) => overrides,
        Err(error) => panic!("telegram acl overrides: {error}"),
    };

    assert_eq!(overrides.allowed_users, vec!["2002"]);
    assert_eq!(
        overrides.allowed_groups,
        vec!["-2001"],
        "group allowlist should inherit from system when user override omits groups"
    );
    assert_eq!(overrides.admin_users, vec!["2002"]);
}

#[test]
fn telegram_acl_overrides_reject_invalid_control_rule_with_field_path() {
    let tmp = match TempDir::new() {
        Ok(tmp) => tmp,
        Err(error) => panic!("tempdir: {error}"),
    };
    let system = tmp
        .path()
        .join("packages/rust/crates/omni-agent/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        system.clone(),
        r#"
[[telegram.acl.control.rules]]
commands = ["session*"]

[telegram.acl.control.rules.allow]
users = ["1001"]
"#,
    );
    write_file(user, "");

    let settings = load_runtime_settings_from_paths(
        &system,
        &tmp.path()
            .join(".config/xiuxian-artisan-workshop/xiuxian.toml"),
    );
    let Err(error) = build_telegram_acl_overrides(&settings) else {
        panic!("invalid command selector should fail fast");
    };
    assert!(
        error
            .to_string()
            .contains("telegram.acl.control.rules[0].commands"),
        "unexpected error path: {error}",
    );
}
