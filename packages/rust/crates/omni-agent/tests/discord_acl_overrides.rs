#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use std::path::PathBuf;

use omni_agent::{
    Channel, DiscordChannel, DiscordControlCommandPolicy, build_discord_acl_overrides,
    load_runtime_settings_from_paths,
};
use tempfile::TempDir;

fn write_file(path: PathBuf, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
    std::fs::write(path, content).expect("write yaml");
}

#[test]
fn discord_acl_overrides_build_from_structured_acl() {
    let tmp = TempDir::new().expect("tempdir");
    let system = tmp.path().join("packages/conf/settings.yaml");
    let user = tmp.path().join(".config/omni-dev-fusion/settings.yaml");

    write_file(
        system.clone(),
        r#"
discord:
  acl:
    role_aliases:
      maintainers: "987654321012345678"
      auditors: "<@&123456789012345678>"
    allow:
      users: ["owner"]
      roles: ["maintainers"]
      guilds: ["3001", "3002"]
    admin:
      users: ["owner"]
      roles: ["maintainers"]
    control:
      allow_from:
        roles: ["maintainers"]
      rules:
        - commands: ["/session partition"]
          allow:
            roles: ["maintainers"]
        - commands: ["/reset", "/clear"]
          allow:
            users: ["owner"]
    slash:
      global:
        roles: ["maintainers"]
      session_status:
        roles: ["auditors"]
      session_budget:
        roles: ["auditors"]
      session_memory:
        users: ["owner"]
      job_status:
        roles: ["maintainers"]
      jobs_summary:
        roles: ["maintainers"]
      background_submit:
        roles: ["maintainers"]
"#,
    );
    write_file(user.clone(), "");

    let settings = load_runtime_settings_from_paths(&system, &user);
    let overrides = build_discord_acl_overrides(&settings).expect("discord acl overrides");

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
    )
    .expect("typed control rules should compile");
    assert!(
        channel.is_authorized_for_control_command("role:987654321012345678", "/session partition")
    );
    assert!(channel.is_authorized_for_control_command("owner", "/reset"));
    assert!(
        !channel.is_authorized_for_control_command("ops", "/session partition"),
        "matched command-scoped rule should override admin fallback",
    );
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
fn discord_acl_overrides_use_merged_role_aliases_from_user_settings() {
    let tmp = TempDir::new().expect("tempdir");
    let system = tmp.path().join("packages/conf/settings.yaml");
    let user = tmp.path().join(".config/omni-dev-fusion/settings.yaml");

    write_file(
        system.clone(),
        r#"
discord:
  acl:
    role_aliases:
      maintainers: "111111111111111111"
      observers: "222222222222222222"
    allow:
      roles: ["maintainers"]
"#,
    );
    write_file(
        user.clone(),
        r#"
discord:
  acl:
    role_aliases:
      maintainers: "999999999999999999"
"#,
    );

    let settings = load_runtime_settings_from_paths(&system, &user);
    let overrides = build_discord_acl_overrides(&settings).expect("discord acl overrides");

    assert_eq!(
        overrides.allowed_users,
        vec!["role:999999999999999999".to_string()],
        "user settings role alias should override system alias"
    );
}

#[test]
fn discord_acl_overrides_reject_invalid_control_rule_with_field_path() {
    let tmp = TempDir::new().expect("tempdir");
    let system = tmp.path().join("packages/conf/settings.yaml");
    let user = tmp.path().join(".config/omni-dev-fusion/settings.yaml");

    write_file(
        system.clone(),
        r#"
discord:
  acl:
    control:
      rules:
        - commands: ["session*"]
          allow:
            users: ["owner"]
"#,
    );
    write_file(user, "");

    let settings = load_runtime_settings_from_paths(
        &system,
        &tmp.path().join(".config/omni-dev-fusion/settings.yaml"),
    );
    let error =
        build_discord_acl_overrides(&settings).expect_err("invalid command selector should fail");
    assert!(
        error
            .to_string()
            .contains("discord.acl.control.rules[0].commands"),
        "unexpected error path: {error}",
    );
}
