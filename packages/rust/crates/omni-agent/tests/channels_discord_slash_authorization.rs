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
    )
    .expect("policy should compile");

    assert!(channel.is_authorized_for_slash_command("ops", SCOPE_SESSION_STATUS));
    assert!(!channel.is_authorized_for_slash_command("alice", SCOPE_SESSION_STATUS));
}

#[test]
fn discord_slash_authorization_global_override_takes_precedence() {
    let slash_policy = DiscordSlashCommandPolicy {
        slash_command_allow_from: Some(vec!["owner".to_string()]),
        session_status_allow_from: Some(vec!["alice".to_string()]),
        ..DiscordSlashCommandPolicy::default()
    };
    let channel = DiscordChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(vec!["ops".to_string()], None, Vec::new())
            .with_slash_command_policy(slash_policy),
        DiscordSessionPartition::GuildChannelUser,
    )
    .expect("policy should compile");

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
        session_memory_allow_from: Some(vec!["alice".to_string()]),
        ..DiscordSlashCommandPolicy::default()
    };
    let channel = DiscordChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(vec!["ops".to_string()], None, Vec::new())
            .with_slash_command_policy(slash_policy),
        DiscordSessionPartition::GuildChannelUser,
    )
    .expect("policy should compile");

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
    )
    .expect("policy should compile");

    assert!(!channel.is_authorized_for_control_command_for_recipient("alice", "/reset", "2001"));

    channel
        .mutate_recipient_command_admin_users(
            "2001",
            omni_agent::RecipientCommandAdminUsersMutation::Set(vec!["alice".to_string()]),
        )
        .expect("recipient override update should succeed");

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
    )
    .expect("policy should compile");

    assert!(!channel.is_authorized_for_slash_command_for_recipient(
        "alice",
        SCOPE_SESSION_STATUS,
        "2001"
    ));

    channel
        .mutate_recipient_command_admin_users(
            "2001",
            omni_agent::RecipientCommandAdminUsersMutation::Set(vec!["alice".to_string()]),
        )
        .expect("recipient override update should succeed");

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
    )
    .expect("policy should compile");

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
    )
    .expect("policy should compile");

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
