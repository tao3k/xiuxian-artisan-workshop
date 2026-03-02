//! Discord ACL override loading and policy projection tests.

use std::path::PathBuf;

use omni_agent::{
    Channel, DiscordAclOverrides, DiscordChannel, DiscordControlCommandPolicy,
    build_discord_acl_overrides, load_runtime_settings_from_paths,
};
use tempfile::TempDir;

const STRUCTURED_DISCORD_ACL_TOML: &str = r#"
[discord.acl.role_aliases]
maintainers = "987654321012345678"
auditors = "<@&123456789012345678>"

[discord.acl.allow]
users = ["owner"]
roles = ["maintainers"]
guilds = ["3001", "3002"]

[discord.acl.admin]
users = ["owner"]
roles = ["maintainers"]

[discord.acl.control.allow_from]
roles = ["maintainers"]

[[discord.acl.control.rules]]
commands = ["/session partition"]

[discord.acl.control.rules.allow]
roles = ["maintainers"]

[[discord.acl.control.rules]]
commands = ["/reset", "/clear"]

[discord.acl.control.rules.allow]
users = ["owner"]

[discord.acl.slash.global]
roles = ["maintainers"]

[discord.acl.slash.session_status]
roles = ["auditors"]

[discord.acl.slash.session_budget]
roles = ["auditors"]

[discord.acl.slash.session_memory]
users = ["owner"]

[discord.acl.slash.job_status]
roles = ["maintainers"]

[discord.acl.slash.jobs_summary]
roles = ["maintainers"]

[discord.acl.slash.background_submit]
roles = ["maintainers"]
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

fn assert_discord_control_rule_projection(overrides: &DiscordAclOverrides) {
    assert_eq!(overrides.control_command_rules.len(), 2);
    let channel = DiscordChannel::new_with_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(
            vec!["ops".to_string()],
            None,
            overrides.control_command_rules.clone(),
        ),
    );
    assert!(
        channel.is_authorized_for_control_command("role:987654321012345678", "/session partition")
    );
    assert!(channel.is_authorized_for_control_command("owner", "/reset"));
    assert!(
        !channel.is_authorized_for_control_command("ops", "/session partition"),
        "matched command-scoped rule should override admin fallback",
    );
}

fn assert_discord_slash_acl_projection(overrides: &DiscordAclOverrides) {
    assert_eq!(
        overrides.slash_command_allow_from,
        Some(vec!["role:987654321012345678".to_string()])
    );
    assert_eq!(
        overrides.slash_session_status_allow_from,
        Some(vec!["role:123456789012345678".to_string()])
    );
    assert_eq!(
        overrides.slash_session_budget_allow_from,
        Some(vec!["role:123456789012345678".to_string()])
    );
    assert_eq!(
        overrides.slash_session_memory_allow_from,
        Some(vec!["owner".to_string()])
    );
    assert_eq!(
        overrides.slash_job_allow_from,
        Some(vec!["role:987654321012345678".to_string()])
    );
    assert_eq!(
        overrides.slash_jobs_allow_from,
        Some(vec!["role:987654321012345678".to_string()])
    );
    assert_eq!(
        overrides.slash_bg_allow_from,
        Some(vec!["role:987654321012345678".to_string()])
    );
}

#[test]
fn discord_acl_overrides_build_from_structured_acl() {
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

    write_file(system.clone(), STRUCTURED_DISCORD_ACL_TOML);
    write_file(user.clone(), "");

    let settings = load_runtime_settings_from_paths(&system, &user);
    let overrides = match build_discord_acl_overrides(&settings) {
        Ok(overrides) => overrides,
        Err(error) => panic!("discord acl overrides: {error}"),
    };

    assert_eq!(
        overrides.allowed_users,
        vec!["owner".to_string(), "role:987654321012345678".to_string()]
    );
    assert_eq!(
        overrides.allowed_guilds,
        vec!["3001".to_string(), "3002".to_string()]
    );
    assert_eq!(
        overrides.admin_users,
        Some(vec![
            "owner".to_string(),
            "role:987654321012345678".to_string()
        ])
    );
    assert_eq!(
        overrides.control_command_allow_from,
        Some(vec!["role:987654321012345678".to_string()])
    );
    assert_discord_control_rule_projection(&overrides);
    assert_discord_slash_acl_projection(&overrides);
}

#[test]
fn discord_acl_overrides_use_merged_role_aliases_from_user_settings() {
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
[discord.acl.role_aliases]
maintainers = "111111111111111111"
observers = "222222222222222222"

[discord.acl.allow]
roles = ["maintainers"]
"#,
    );
    write_file(
        user.clone(),
        r#"
[discord.acl.role_aliases]
maintainers = "999999999999999999"
"#,
    );

    let settings = load_runtime_settings_from_paths(&system, &user);
    let overrides = match build_discord_acl_overrides(&settings) {
        Ok(overrides) => overrides,
        Err(error) => panic!("discord acl overrides: {error}"),
    };

    assert_eq!(
        overrides.allowed_users,
        vec!["role:999999999999999999".to_string()],
        "user settings role alias should override system alias"
    );
}

#[test]
fn discord_acl_overrides_reject_invalid_control_rule_with_field_path() {
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
[[discord.acl.control.rules]]
commands = ["session*"]

[discord.acl.control.rules.allow]
users = ["owner"]
"#,
    );
    write_file(user, "");

    let settings = load_runtime_settings_from_paths(
        &system,
        &tmp.path()
            .join(".config/xiuxian-artisan-workshop/xiuxian.toml"),
    );
    let Err(error) = build_discord_acl_overrides(&settings) else {
        panic!("invalid command selector should fail");
    };
    assert!(
        error
            .to_string()
            .contains("discord.acl.control.rules[0].commands"),
        "unexpected error path: {error}",
    );
}
