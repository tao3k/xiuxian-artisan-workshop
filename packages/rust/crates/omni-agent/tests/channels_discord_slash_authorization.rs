//! Discord slash-command authorization tests for scope and role policies.

use omni_agent::{
    Channel, DiscordChannel, DiscordControlCommandPolicy, DiscordSessionPartition,
    DiscordSlashCommandPolicy,
};

const SCOPE_SESSION_STATUS: &str = "session.status";
const SCOPE_SESSION_MEMORY: &str = "session.memory";
const SCOPE_JOBS_SUMMARY: &str = "jobs.summary";

fn discord_message_event(
    message_id: &str,
    content: &str,
    channel_id: &str,
    guild_id: &str,
    user_id: &str,
    username: &str,
    role_ids: &[&str],
) -> serde_json::Value {
    serde_json::json!({
        "id": message_id,
        "content": content,
        "channel_id": channel_id,
        "guild_id": guild_id,
        "author": {
            "id": user_id,
            "username": username,
        },
        "member": {
            "roles": role_ids,
        }
    })
}

#[test]
fn discord_slash_authorization_falls_back_to_admin_users() {
    let channel = DiscordChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(vec!["ops".to_string()], None, Vec::new()),
        DiscordSessionPartition::GuildChannelUser,
    );

    assert!(channel.is_authorized_for_slash_command("ops", SCOPE_SESSION_STATUS));
    assert!(!channel.is_authorized_for_slash_command("alice", SCOPE_SESSION_STATUS));
}

#[test]
fn discord_slash_authorization_global_override_takes_precedence() {
    let slash_policy = DiscordSlashCommandPolicy {
        global: Some(vec!["owner".to_string()]),
        session_status: Some(vec!["alice".to_string()]),
        ..DiscordSlashCommandPolicy::default()
    };
    let channel = DiscordChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(vec!["ops".to_string()], None, Vec::new())
            .with_slash_command_policy(slash_policy),
        DiscordSessionPartition::GuildChannelUser,
    );

    assert!(channel.is_authorized_for_slash_command("owner", SCOPE_SESSION_STATUS));
    assert!(
        !channel.is_authorized_for_slash_command("alice", SCOPE_SESSION_STATUS),
        "global override should ignore command-scoped allowlist"
    );
    assert!(
        !channel.is_authorized_for_slash_command("ops", SCOPE_SESSION_STATUS),
        "global override should ignore admin fallback"
    );
}

#[test]
fn discord_slash_authorization_command_scope_rules_are_partial() {
    let slash_policy = DiscordSlashCommandPolicy {
        session_memory: Some(vec!["alice".to_string()]),
        ..DiscordSlashCommandPolicy::default()
    };
    let channel = DiscordChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(vec!["ops".to_string()], None, Vec::new())
            .with_slash_command_policy(slash_policy),
        DiscordSessionPartition::GuildChannelUser,
    );

    assert!(channel.is_authorized_for_slash_command("alice", SCOPE_SESSION_MEMORY));
    assert!(channel.is_authorized_for_slash_command("ops", SCOPE_SESSION_MEMORY));
    assert!(!channel.is_authorized_for_slash_command("bob", SCOPE_SESSION_MEMORY));
    assert!(
        channel.is_authorized_for_slash_command("ops", SCOPE_JOBS_SUMMARY),
        "unconfigured command should still fall back to admin_users"
    );
}

#[test]
fn discord_control_authorization_recipient_override_fallback() {
    let channel = DiscordChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(vec!["ops".to_string()], None, Vec::new()),
        DiscordSessionPartition::GuildChannelUser,
    );

    assert!(!channel.is_authorized_for_control_command_for_recipient("alice", "/reset", "2001"));

    if let Err(error) = channel.mutate_recipient_command_admin_users(
        "2001",
        omni_agent::RecipientCommandAdminUsersMutation::Set(vec!["alice".to_string()]),
    ) {
        panic!("recipient override update should succeed: {error}");
    }

    assert!(channel.is_authorized_for_control_command_for_recipient("alice", "/reset", "2001"));
}

#[test]
fn discord_slash_authorization_recipient_override_fallback() {
    let channel = DiscordChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(vec!["ops".to_string()], None, Vec::new()),
        DiscordSessionPartition::GuildChannelUser,
    );

    assert!(!channel.is_authorized_for_slash_command_for_recipient(
        "alice",
        SCOPE_SESSION_STATUS,
        "2001"
    ));

    if let Err(error) = channel.mutate_recipient_command_admin_users(
        "2001",
        omni_agent::RecipientCommandAdminUsersMutation::Set(vec!["alice".to_string()]),
    ) {
        panic!("recipient override update should succeed: {error}");
    }

    assert!(channel.is_authorized_for_slash_command_for_recipient(
        "alice",
        SCOPE_SESSION_STATUS,
        "2001"
    ));
}

#[test]
fn discord_control_authorization_supports_role_principals_per_recipient() {
    let channel = DiscordChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["role:9001".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(vec!["role:9001".to_string()], None, Vec::new()),
        DiscordSessionPartition::GuildChannelUser,
    );

    let _ = channel.parse_gateway_message(&discord_message_event(
        "1",
        "/reset",
        "2001",
        "3001",
        "1001",
        "alice",
        &["9001"],
    ));

    assert!(
        channel.is_authorized_for_control_command_for_recipient("1001", "/reset", "2001"),
        "role principal should authorize privileged commands"
    );
}

#[test]
fn discord_slash_authorization_supports_cached_username_alias_per_recipient() {
    let channel = DiscordChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["owner".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(vec!["owner".to_string()], None, Vec::new()),
        DiscordSessionPartition::GuildChannelUser,
    );

    let _ = channel.parse_gateway_message(&discord_message_event(
        "1",
        "/session memory",
        "2001",
        "3001",
        "1001",
        "owner",
        &[],
    ));

    assert!(
        channel.is_authorized_for_slash_command_for_recipient("1001", SCOPE_SESSION_MEMORY, "2001"),
        "cached username alias should satisfy admin fallback"
    );
}
