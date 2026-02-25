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

use std::fs;

use omni_agent::{
    Channel, TelegramChannel, TelegramControlCommandPolicy, TelegramSessionPartition,
    TelegramSlashCommandPolicy,
};
use serde_json::json;

const SCOPE_SESSION_STATUS: &str = "session.status";
const SCOPE_SESSION_MEMORY: &str = "session.memory";
const SCOPE_JOBS_SUMMARY: &str = "jobs.summary";

#[test]
fn telegram_slash_authorization_falls_back_to_admin_users() {
    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        TelegramControlCommandPolicy::new(vec!["2001".to_string()], None, Vec::new()),
        TelegramSessionPartition::ChatUser,
    )
    .expect("policy should compile");

    assert!(channel.is_authorized_for_slash_command("2001", SCOPE_SESSION_STATUS));
    assert!(!channel.is_authorized_for_slash_command("1001", SCOPE_SESSION_STATUS));
}

#[test]
fn telegram_slash_authorization_global_override_takes_precedence() {
    let slash_policy = TelegramSlashCommandPolicy {
        slash_command_allow_from: Some(vec!["3001".to_string()]),
        session_status_allow_from: Some(vec!["1001".to_string()]),
        ..TelegramSlashCommandPolicy::default()
    };
    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        TelegramControlCommandPolicy::new(vec!["2001".to_string()], None, Vec::new())
            .with_slash_command_policy(slash_policy),
        TelegramSessionPartition::ChatUser,
    )
    .expect("policy should compile");

    assert!(channel.is_authorized_for_slash_command("3001", SCOPE_SESSION_STATUS));
    assert!(
        !channel.is_authorized_for_slash_command("1001", SCOPE_SESSION_STATUS),
        "global override should ignore command-scoped allowlist"
    );
    assert!(
        !channel.is_authorized_for_slash_command("2001", SCOPE_SESSION_STATUS),
        "global override should ignore admin fallback"
    );
}

#[test]
fn telegram_slash_authorization_command_scope_rules_are_partial() {
    let slash_policy = TelegramSlashCommandPolicy {
        session_memory_allow_from: Some(vec!["1001".to_string()]),
        ..TelegramSlashCommandPolicy::default()
    };
    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        TelegramControlCommandPolicy::new(vec!["2001".to_string()], None, Vec::new())
            .with_slash_command_policy(slash_policy),
        TelegramSessionPartition::ChatUser,
    )
    .expect("policy should compile");

    assert!(channel.is_authorized_for_slash_command("1001", SCOPE_SESSION_MEMORY));
    assert!(channel.is_authorized_for_slash_command("2001", SCOPE_SESSION_MEMORY));
    assert!(!channel.is_authorized_for_slash_command("3001", SCOPE_SESSION_MEMORY));
    assert!(
        channel.is_authorized_for_slash_command("2001", SCOPE_JOBS_SUMMARY),
        "unconfigured command should still fall back to admin_users"
    );
}

#[test]
fn telegram_slash_authorization_ignores_invalid_identity_entries() {
    let slash_policy = TelegramSlashCommandPolicy {
        session_memory_allow_from: Some(vec!["alice".to_string(), "1001".to_string()]),
        ..TelegramSlashCommandPolicy::default()
    };
    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        TelegramControlCommandPolicy::new(
            vec!["owner".to_string(), "2001".to_string()],
            None,
            Vec::new(),
        )
        .with_slash_command_policy(slash_policy),
        TelegramSessionPartition::ChatUser,
    )
    .expect("policy should compile");

    assert!(
        channel.is_authorized_for_slash_command("1001", SCOPE_SESSION_MEMORY),
        "numeric command-scoped entries should remain authorized",
    );
    assert!(
        channel.is_authorized_for_slash_command("2001", SCOPE_SESSION_MEMORY),
        "numeric admin entries should still be merged into command-scoped policy",
    );
    assert!(
        !channel.is_authorized_for_slash_command("alice", SCOPE_SESSION_MEMORY),
        "username entries should be ignored by normalization",
    );
}

#[test]
fn telegram_acl_hot_reload_updates_authorization_without_restart() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let system_settings_path = temp_dir.path().join("settings-system.yaml");
    let user_settings_path = temp_dir.path().join("settings-user.yaml");

    let first_settings = r#"
telegram:
  acl:
    allow:
      users: ["111"]
      groups: []
    admin:
      users: ["111"]
    control:
      allow_from:
        users: ["111"]
    slash:
      global:
        users: ["111"]
"#;
    fs::write(&system_settings_path, first_settings).expect("write first settings");
    fs::write(&user_settings_path, "").expect("write empty user settings");

    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec![],
        vec![],
        TelegramControlCommandPolicy::new(vec![], None, Vec::new()),
        TelegramSessionPartition::ChatUser,
    )
    .expect("policy should compile");

    channel.set_acl_reload_paths_for_test(system_settings_path.clone(), user_settings_path.clone());
    channel.reload_acl_from_settings_for_test();

    let msg_111 = json!({
        "update_id": 1,
        "message": {
            "message_id": 1,
            "text": "/session memory",
            "chat": { "id": -200100, "type": "group", "title": "test" },
            "from": { "id": 111 }
        }
    });
    assert!(channel.parse_update_message(&msg_111).is_some());
    assert!(channel.is_authorized_for_slash_command("111", SCOPE_SESSION_MEMORY));
    assert!(!channel.is_authorized_for_slash_command("2222", SCOPE_SESSION_MEMORY));

    let second_settings = r#"
telegram:
  acl:
    allow:
      users: ["2222"]
      groups: []
    admin:
      users: ["2222"]
    control:
      allow_from:
        users: ["2222"]
    slash:
      global:
        users: ["2222"]
"#;
    fs::write(&system_settings_path, second_settings).expect("write second settings");
    channel.reload_acl_from_settings_for_test();

    let msg_2222 = json!({
        "update_id": 2,
        "message": {
            "message_id": 2,
            "text": "/session memory",
            "chat": { "id": -200100, "type": "group", "title": "test" },
            "from": { "id": 2222 }
        }
    });
    assert!(channel.parse_update_message(&msg_2222).is_some());
    assert!(channel.is_authorized_for_slash_command("2222", SCOPE_SESSION_MEMORY));
    assert!(!channel.is_authorized_for_slash_command("111", SCOPE_SESSION_MEMORY));
}
