//! Telegram slash-command authorization tests for scope and role policies.

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
    );

    assert!(channel.is_authorized_for_slash_command("2001", SCOPE_SESSION_STATUS));
    assert!(!channel.is_authorized_for_slash_command("1001", SCOPE_SESSION_STATUS));
}

#[test]
fn telegram_slash_authorization_global_override_takes_precedence() {
    let slash_policy = TelegramSlashCommandPolicy {
        global: Some(vec!["3001".to_string()]),
        session_status: Some(vec!["1001".to_string()]),
        ..TelegramSlashCommandPolicy::default()
    };
    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        TelegramControlCommandPolicy::new(vec!["2001".to_string()], None, Vec::new())
            .with_slash_command_policy(slash_policy),
        TelegramSessionPartition::ChatUser,
    );

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
        session_memory: Some(vec!["1001".to_string()]),
        ..TelegramSlashCommandPolicy::default()
    };
    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        TelegramControlCommandPolicy::new(vec!["2001".to_string()], None, Vec::new())
            .with_slash_command_policy(slash_policy),
        TelegramSessionPartition::ChatUser,
    );

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
        session_memory: Some(vec!["alice".to_string(), "1001".to_string()]),
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
    );

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
    let temp_dir = match tempfile::tempdir() {
        Ok(temp_dir) => temp_dir,
        Err(error) => panic!("tempdir: {error}"),
    };
    let system_settings_path = temp_dir.path().join("settings-system.toml");
    let user_settings_path = temp_dir.path().join("settings-user.toml");

    let first_settings = r#"
[telegram.acl.allow]
users = ["111"]
groups = []

[telegram.acl.admin]
users = ["111"]

[telegram.acl.control.allow_from]
users = ["111"]

[telegram.acl.slash.global]
users = ["111"]
"#;
    if let Err(error) = fs::write(&system_settings_path, first_settings) {
        panic!("write first settings: {error}");
    }
    if let Err(error) = fs::write(&user_settings_path, "") {
        panic!("write empty user settings: {error}");
    }

    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec![],
        vec![],
        TelegramControlCommandPolicy::new(vec![], None, Vec::new()),
        TelegramSessionPartition::ChatUser,
    );

    channel.set_acl_reload_paths_for_test(system_settings_path.clone(), user_settings_path.clone());
    channel.reload_acl_from_settings_for_test();

    let msg_111 = json!({
        "update_id": 1,
        "message": {
            "message_id": 1,
            "text": "/session memory",
            "chat": { "id": -200_100, "type": "group", "title": "test" },
            "from": { "id": 111 }
        }
    });
    assert!(channel.parse_update_message(&msg_111).is_some());
    assert!(channel.is_authorized_for_slash_command("111", SCOPE_SESSION_MEMORY));
    assert!(!channel.is_authorized_for_slash_command("2222", SCOPE_SESSION_MEMORY));

    let second_settings = r#"
[telegram.acl.allow]
users = ["2222"]
groups = []

[telegram.acl.admin]
users = ["2222"]

[telegram.acl.control.allow_from]
users = ["2222"]

[telegram.acl.slash.global]
users = ["2222"]
"#;
    if let Err(error) = fs::write(&system_settings_path, second_settings) {
        panic!("write second settings: {error}");
    }
    channel.reload_acl_from_settings_for_test();

    let msg_2222 = json!({
        "update_id": 2,
        "message": {
            "message_id": 2,
            "text": "/session memory",
            "chat": { "id": -200_100, "type": "group", "title": "test" },
            "from": { "id": 2222 }
        }
    });
    assert!(channel.parse_update_message(&msg_2222).is_some());
    assert!(channel.is_authorized_for_slash_command("2222", SCOPE_SESSION_MEMORY));
    assert!(!channel.is_authorized_for_slash_command("111", SCOPE_SESSION_MEMORY));
}
