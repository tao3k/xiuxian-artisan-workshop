//! Telegram group policy and per-group override projection tests.

use std::fs;

use omni_agent::{
    Channel, RecipientCommandAdminUsersMutation, TelegramChannel, TelegramControlCommandPolicy,
    TelegramSessionPartition, load_runtime_settings_from_paths,
};

fn require_ok<T, E>(result: std::result::Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn require_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(value) => value,
        None => panic!("{context}"),
    }
}

fn build_channel_with_settings(settings_toml: &str) -> (tempfile::TempDir, TelegramChannel) {
    let temp_dir = require_ok(tempfile::tempdir(), "tempdir");
    let system_settings_path = temp_dir.path().join("settings-system.toml");
    let user_settings_path = temp_dir.path().join("settings-user.toml");
    require_ok(
        fs::write(&system_settings_path, settings_toml),
        "write system settings",
    );
    require_ok(fs::write(&user_settings_path, ""), "write user settings");

    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec![],
        vec![],
        TelegramControlCommandPolicy::default(),
        TelegramSessionPartition::ChatUser,
    );
    channel.set_acl_reload_paths_for_test(system_settings_path, user_settings_path);
    channel.reload_acl_from_settings_for_test();
    (temp_dir, channel)
}

fn settings_paths(temp_dir: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    (
        temp_dir.path().join("settings-system.toml"),
        temp_dir.path().join("settings-user.toml"),
    )
}

fn group_update_for_chat(chat_id: i64, user_id: i64, text: &str) -> serde_json::Value {
    serde_json::json!({
        "update_id": 90001,
        "message": {
            "message_id": 101,
            "text": text,
            "chat": { "id": chat_id, "type": "group", "title": "team" },
            "from": { "id": user_id, "username": format!("u{user_id}") }
        }
    })
}

fn group_update(user_id: i64, text: &str) -> serde_json::Value {
    group_update_for_chat(-200_100, user_id, text)
}

fn topic_update(user_id: i64, text: &str, topic_id: i64) -> serde_json::Value {
    let mut update = group_update(user_id, text);
    update["message"]["message_thread_id"] = serde_json::json!(topic_id);
    update
}

#[test]
fn telegram_group_policy_disabled_rejects_group_messages() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
group_policy = "disabled"

[telegram.acl.allow]
users = []
groups = ["-200100"]
"#,
    );

    assert!(
        channel
            .parse_update_message(&group_update(111, "hello"))
            .is_none()
    );
}

#[test]
fn telegram_group_policy_allowlist_uses_group_allow_from() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
group_policy = "allowlist"
group_allow_from = "111"

[telegram.acl.allow]
users = []
groups = ["-200100"]
"#,
    );

    assert!(
        channel
            .parse_update_message(&group_update(111, "hello"))
            .is_some()
    );
    assert!(
        channel
            .parse_update_message(&group_update(222, "hello"))
            .is_none()
    );
}

#[test]
fn telegram_group_policy_allowlist_falls_back_to_allowed_users_when_group_allow_from_unset() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
group_policy = "allowlist"

[telegram.acl.allow]
users = ["999"]
groups = ["-200100"]
"#,
    );

    assert!(
        channel
            .parse_update_message(&group_update(999, "hello"))
            .is_some()
    );
    assert!(
        channel
            .parse_update_message(&group_update(111, "hello"))
            .is_none()
    );
}

#[test]
fn telegram_group_policy_group_override_can_open_when_global_disabled() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
group_policy = "disabled"

[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.groups."-200100"]
group_policy = "open"
"#,
    );

    assert!(
        channel
            .parse_update_message(&group_update(333, "hello"))
            .is_some()
    );
}

#[test]
fn telegram_group_policy_require_mention_blocks_plain_group_text() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
group_policy = "open"
require_mention = true

[telegram.acl.allow]
users = []
groups = ["-200100"]
"#,
    );

    assert!(
        channel
            .parse_update_message(&group_update(111, "hello everyone"))
            .is_none()
    );
    assert!(
        channel
            .parse_update_message(&group_update(111, "/session status"))
            .is_some()
    );
    assert!(
        channel
            .parse_update_message(&group_update(111, "@bot hello"))
            .is_some()
    );
}

#[test]
fn telegram_group_policy_topic_override_has_higher_priority() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
group_policy = "open"

[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.groups."-200100".topics."42"]
group_policy = "allowlist"

[telegram.groups."-200100".topics."42".allow_from]
users = ["111"]
"#,
    );

    assert!(
        channel
            .parse_update_message(&topic_update(111, "topic hello", 42))
            .is_some()
    );
    assert!(
        channel
            .parse_update_message(&topic_update(222, "topic hello", 42))
            .is_none()
    );
    assert!(
        channel
            .parse_update_message(&topic_update(222, "other topic", 99))
            .is_some()
    );
}

#[test]
fn telegram_group_policy_wildcard_override_is_applied_before_specific_group_override() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
group_policy = "open"
require_mention = false

[telegram.acl.allow]
users = []
groups = ["-200100", "-200200"]

[telegram.groups."*"]
require_mention = true

[telegram.groups."-200100"]
require_mention = false
"#,
    );

    let group_specific = group_update_for_chat(-200_100, 111, "plain group text");
    assert!(channel.parse_update_message(&group_specific).is_some());

    let wildcard_only = group_update_for_chat(-200_200, 111, "plain group text");
    assert!(channel.parse_update_message(&wildcard_only).is_none());

    let wildcard_triggered = group_update_for_chat(-200_200, 111, "/session status");
    assert!(channel.parse_update_message(&wildcard_triggered).is_some());
}

#[test]
fn telegram_group_policy_require_mention_accepts_reply_to_bot_trigger() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
group_policy = "open"
require_mention = true

[telegram.acl.allow]
users = []
groups = ["-200100"]
"#,
    );

    let mut update = group_update(111, "hello in reply");
    update["message"]["reply_to_message"] = serde_json::json!({
      "from": { "is_bot": true }
    });
    assert!(channel.parse_update_message(&update).is_some());
}

#[test]
fn telegram_group_policy_require_mention_accepts_entity_mention_trigger() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
group_policy = "open"
require_mention = true

[telegram.acl.allow]
users = []
groups = ["-200100"]
"#,
    );

    let mut update = group_update(111, "hello");
    update["message"]["entities"] = serde_json::json!([
      { "type": "mention", "offset": 0, "length": 5 }
    ]);
    assert!(channel.parse_update_message(&update).is_some());
}

#[test]
fn telegram_group_policy_group_admin_users_are_scoped_by_recipient() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram.acl.allow]
users = []
groups = ["-200100", "-200200"]

[telegram.acl.admin]
users = []

[telegram.groups."-200100".admin_users]
users = ["111"]
"#,
    );

    assert!(channel.is_authorized_for_control_command_for_recipient(
        "telegram:111",
        "/reset",
        "-200100",
    ));
    assert!(!channel.is_authorized_for_control_command_for_recipient("111", "/reset", "-200200",));
    assert!(!channel.is_authorized_for_control_command_for_recipient("111", "/reset", "12345",));
    assert!(channel.is_authorized_for_slash_command_for_recipient(
        "111",
        "session.status",
        "-200100",
    ));
    assert!(!channel.is_authorized_for_slash_command_for_recipient(
        "111",
        "session.status",
        "-200200",
    ));
}

#[test]
fn telegram_group_policy_topic_admin_users_override_group_and_wildcard_admin_users() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram.acl.allow]
users = []
groups = ["-200100", "-200200"]

[telegram.acl.admin]
users = []

[telegram.groups."*".admin_users]
users = ["900"]

[telegram.groups."-200100".admin_users]
users = ["111"]

[telegram.groups."-200100".topics."42".admin_users]
users = ["222"]
"#,
    );

    assert!(channel.is_authorized_for_control_command_for_recipient(
        "222",
        "/session partition",
        "-200100:42",
    ));
    assert!(!channel.is_authorized_for_control_command_for_recipient(
        "111",
        "/session partition",
        "-200100:42",
    ));
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "111",
        "/session partition",
        "-200100:99",
    ));
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "900",
        "/session partition",
        "-200200",
    ));
    assert!(channel.is_authorized_for_slash_command_for_recipient(
        "222",
        "session.memory",
        "-200100:42",
    ));
}

#[test]
fn telegram_group_policy_group_admin_users_do_not_override_explicit_global_control_deny() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.acl.admin]
users = []

[telegram.acl.control.allow_from]
users = []

[telegram.groups."-200100".admin_users]
users = ["111"]
"#,
    );

    assert!(!channel.is_authorized_for_control_command_for_recipient("111", "/reset", "-200100",));
}

#[test]
fn telegram_group_policy_group_admin_users_do_not_override_explicit_global_slash_deny() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.acl.admin]
users = []

[telegram.acl.slash.global]
users = []

[telegram.groups."-200100".admin_users]
users = ["111"]
"#,
    );

    assert!(!channel.is_authorized_for_slash_command_for_recipient(
        "111",
        "session.status",
        "-200100",
    ));
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_group_scope() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.acl.admin]
users = []
"#,
    );

    assert_eq!(
        require_ok(
            channel.recipient_command_admin_users("-200100"),
            "group recipient query should succeed",
        ),
        None
    );

    assert_eq!(
        require_ok(
            channel.mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Add(vec!["telegram:111".to_string()]),
            ),
            "group add should succeed",
        ),
        Some(vec!["111".to_string()])
    );
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "111",
        "/session partition",
        "-200100",
    ));

    assert_eq!(
        require_ok(
            channel.mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Remove(vec!["111".to_string()]),
            ),
            "group remove should succeed",
        ),
        None
    );
    assert_eq!(
        require_ok(
            channel.recipient_command_admin_users("-200100"),
            "group recipient query should succeed",
        ),
        None
    );
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_topic_scope() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.acl.admin]
users = []
"#,
    );

    assert_eq!(
        require_ok(
            channel.mutate_recipient_command_admin_users(
                "-200100:42",
                RecipientCommandAdminUsersMutation::Set(vec!["222".to_string(), "222".to_string()]),
            ),
            "topic set should succeed",
        ),
        Some(vec!["222".to_string()])
    );
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "222",
        "/session admin",
        "-200100:42",
    ));
    assert!(!channel.is_authorized_for_control_command_for_recipient(
        "222",
        "/session admin",
        "-200100:99",
    ));

    assert_eq!(
        require_ok(
            channel.mutate_recipient_command_admin_users(
                "-200100:42",
                RecipientCommandAdminUsersMutation::Clear,
            ),
            "topic clear should succeed",
        ),
        None
    );
    assert_eq!(
        require_ok(
            channel.recipient_command_admin_users("-200100:42"),
            "topic query should succeed",
        ),
        None
    );
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_group_topic_isolation() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.acl.admin]
users = []
"#,
    );

    assert_eq!(
        require_ok(
            channel.mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Set(vec!["111".to_string()]),
            ),
            "group set should succeed",
        ),
        Some(vec!["111".to_string()])
    );
    assert_eq!(
        require_ok(
            channel.mutate_recipient_command_admin_users(
                "-200100:42",
                RecipientCommandAdminUsersMutation::Set(vec!["222".to_string()]),
            ),
            "topic set should succeed",
        ),
        Some(vec!["222".to_string()])
    );

    assert!(!channel.is_authorized_for_control_command_for_recipient(
        "111",
        "/session admin",
        "-200100:42",
    ));
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "111",
        "/session admin",
        "-200100:99",
    ));
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "222",
        "/session admin",
        "-200100:42",
    ));
    assert!(!channel.is_authorized_for_control_command_for_recipient(
        "222",
        "/session admin",
        "-200100:99",
    ));
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_rejects_invalid_identity_or_scope()
{
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.acl.admin]
users = []
"#,
    );

    assert!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Set(vec!["alice".to_string()]),
            )
            .is_err(),
        "set should reject non-numeric identity"
    );
    assert!(
        channel.recipient_command_admin_users("12345").is_err(),
        "direct-chat recipient should not support delegated admin mutation"
    );
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_persists_when_enabled() {
    let (temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
session_admin_persist = true

[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.acl.admin]
users = []
"#,
    );

    assert_eq!(
        require_ok(
            channel.mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Add(vec!["telegram:111".to_string()]),
            ),
            "group add should succeed",
        ),
        Some(vec!["111".to_string()])
    );
    assert_eq!(
        require_ok(
            channel.mutate_recipient_command_admin_users(
                "-200100:42",
                RecipientCommandAdminUsersMutation::Set(vec![
                    "222".to_string(),
                    "telegram:333".to_string(),
                    "222".to_string(),
                ]),
            ),
            "topic set should succeed",
        ),
        Some(vec!["222".to_string(), "333".to_string()])
    );

    let (system_settings_path, user_settings_path) = settings_paths(&temp_dir);
    let merged = load_runtime_settings_from_paths(&system_settings_path, &user_settings_path);
    let groups = require_some(merged.telegram.groups, "group overrides should persist");
    let group = require_some(groups.get("-200100"), "group override should exist");
    assert_eq!(
        group
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["111".to_string()])
    );
    let topics = require_some(group.topics.as_ref(), "topic override should persist");
    let topic = require_some(topics.get("42"), "topic override should exist");
    assert_eq!(
        topic
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["222".to_string(), "333".to_string()])
    );
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_clear_prunes_persisted_entries() {
    let (temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
session_admin_persist = true

[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.acl.admin]
users = []
"#,
    );

    require_ok(
        channel.mutate_recipient_command_admin_users(
            "-200100",
            RecipientCommandAdminUsersMutation::Set(vec!["111".to_string()]),
        ),
        "group set should succeed",
    );
    require_ok(
        channel.mutate_recipient_command_admin_users(
            "-200100:42",
            RecipientCommandAdminUsersMutation::Set(vec!["222".to_string()]),
        ),
        "topic set should succeed",
    );
    require_ok(
        channel.mutate_recipient_command_admin_users(
            "-200100:42",
            RecipientCommandAdminUsersMutation::Clear,
        ),
        "topic clear should succeed",
    );
    require_ok(
        channel.mutate_recipient_command_admin_users(
            "-200100",
            RecipientCommandAdminUsersMutation::Clear,
        ),
        "group clear should succeed",
    );

    let (system_settings_path, user_settings_path) = settings_paths(&temp_dir);
    let merged = load_runtime_settings_from_paths(&system_settings_path, &user_settings_path);
    assert!(
        merged.telegram.groups.is_none(),
        "clearing overrides should prune persisted group/topic admin entries"
    );
    let user_toml = require_ok(
        fs::read_to_string(user_settings_path),
        "user settings should be readable",
    );
    assert!(!user_toml.contains("admin_users"));
    assert!(!user_toml.contains("-200100"));
    assert!(!user_toml.contains("42"));
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_does_not_persist_when_disabled() {
    let (temp_dir, channel) = build_channel_with_settings(
        r#"
[telegram]
session_admin_persist = false

[telegram.acl.allow]
users = []
groups = ["-200100"]

[telegram.acl.admin]
users = []
"#,
    );

    assert_eq!(
        require_ok(
            channel.mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Set(vec!["111".to_string()]),
            ),
            "group set should succeed",
        ),
        Some(vec!["111".to_string()])
    );
    assert_eq!(
        require_ok(
            channel.recipient_command_admin_users("-200100"),
            "group query should succeed",
        ),
        Some(vec!["111".to_string()])
    );

    let (system_settings_path, user_settings_path) = settings_paths(&temp_dir);
    let user_toml = require_ok(
        fs::read_to_string(&user_settings_path),
        "user settings should be readable",
    );
    assert!(
        user_toml.trim().is_empty(),
        "persistence-disabled mode must not mutate user settings"
    );

    channel.reload_acl_from_settings_for_test();
    assert_eq!(
        require_ok(
            channel.recipient_command_admin_users("-200100"),
            "group query should succeed",
        ),
        None,
        "process-local override should disappear after reload when persistence is disabled"
    );

    let merged = load_runtime_settings_from_paths(&system_settings_path, &user_settings_path);
    assert!(merged.telegram.groups.is_none());
}
