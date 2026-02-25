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

use omni_agent::{Channel, DiscordChannel, DiscordSessionPartition};

fn discord_event(
    message_id: &str,
    content: &str,
    channel_id: &str,
    guild_id: Option<&str>,
    user_id: &str,
    username: Option<&str>,
) -> serde_json::Value {
    discord_event_with_roles(
        message_id,
        content,
        channel_id,
        guild_id,
        user_id,
        username,
        &[],
    )
}

fn discord_event_with_roles(
    message_id: &str,
    content: &str,
    channel_id: &str,
    guild_id: Option<&str>,
    user_id: &str,
    username: Option<&str>,
    role_ids: &[&str],
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "id": message_id,
        "content": content,
        "channel_id": channel_id,
        "author": {
            "id": user_id
        }
    });
    if let Some(guild) = guild_id {
        payload["guild_id"] = serde_json::Value::String(guild.to_string());
    }
    if let Some(name) = username {
        payload["author"]["username"] = serde_json::Value::String(name.to_string());
    }
    if !role_ids.is_empty() {
        payload["member"] = serde_json::json!({
            "roles": role_ids,
        });
    }
    payload
}

fn discord_slash_interaction_event(
    interaction_id: &str,
    command_name: &str,
    channel_id: &str,
    guild_id: Option<&str>,
    user_id: &str,
    username: &str,
    options: serde_json::Value,
    interaction_type: u8,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "id": interaction_id,
        "application_id": "5001",
        "type": interaction_type,
        "data": {
            "id": "6001",
            "name": command_name,
            "type": 1
        },
        "channel_id": channel_id,
        "token": "interaction-token",
        "version": 1,
        "locale": "en-US",
        "entitlements": [],
        "attachment_size_limit": 8388608,
        "user": {
            "id": user_id,
            "username": username
        }
    });
    if let Some(guild) = guild_id {
        payload["guild_id"] = serde_json::Value::String(guild.to_string());
        payload["guild_locale"] = serde_json::Value::String("en-US".to_string());
    }
    if !options.is_null() {
        payload["data"]["options"] = options;
    }
    payload
}

#[test]
fn discord_parse_gateway_message_allows_allowed_user() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["alice".to_string()], vec![]);
    let event = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));

    let parsed = channel
        .parse_gateway_message(&event)
        .expect("message should parse");
    assert_eq!(parsed.sender, "1001");
    assert_eq!(parsed.recipient, "2001");
    assert_eq!(parsed.channel, "discord");
}

#[test]
fn discord_parse_gateway_message_allows_allowed_guild() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec![], vec!["3001".to_string()]);
    let event = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("unknown"));

    let parsed = channel
        .parse_gateway_message(&event)
        .expect("message should parse");
    assert_eq!(parsed.sender, "1001");
    assert_eq!(parsed.session_key, "3001:2001:1001");
}

#[test]
fn discord_parse_gateway_message_allows_allowed_role_identity() {
    let channel = DiscordChannel::new(
        "fake-token".to_string(),
        vec!["role:9001".to_string()],
        vec![],
    );
    let event = discord_event_with_roles(
        "1",
        "hello",
        "2001",
        Some("3001"),
        "1001",
        Some("alice"),
        &["9001"],
    );

    let parsed = channel
        .parse_gateway_message(&event)
        .expect("message should parse");
    assert_eq!(parsed.sender, "1001");
    assert_eq!(parsed.recipient, "2001");
}

#[test]
fn discord_parse_gateway_message_rejects_unauthorized_sender() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["owner".to_string()], vec![]);
    let event = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));

    assert!(channel.parse_gateway_message(&event).is_none());
}

#[test]
fn discord_parse_gateway_message_rejects_empty_content() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_event("1", "   ", "2001", Some("3001"), "1001", Some("alice"));

    assert!(channel.parse_gateway_message(&event).is_none());
}

#[test]
fn discord_parse_gateway_message_rejects_invalid_snowflake_payload() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_event(
        "not-a-snowflake",
        "hello",
        "2001",
        Some("3001"),
        "1001",
        Some("alice"),
    );

    assert!(channel.parse_gateway_message(&event).is_none());
}

#[test]
fn discord_parse_gateway_message_defaults_dm_scope() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_event("1", "hello", "2001", None, "1001", Some("alice"));

    let parsed = channel
        .parse_gateway_message(&event)
        .expect("message should parse");
    assert_eq!(parsed.session_key, "dm:2001:1001");
}

#[test]
fn discord_parse_gateway_message_partition_channel_only() {
    let channel = DiscordChannel::new_with_partition(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordSessionPartition::ChannelOnly,
    );
    let event_a = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));
    let event_b = discord_event("2", "hello", "2001", Some("3001"), "1002", Some("bob"));

    let parsed_a = channel
        .parse_gateway_message(&event_a)
        .expect("message A should parse");
    let parsed_b = channel
        .parse_gateway_message(&event_b)
        .expect("message B should parse");
    assert_eq!(parsed_a.session_key, "3001:2001");
    assert_eq!(parsed_a.session_key, parsed_b.session_key);
}

#[test]
fn discord_parse_gateway_message_partition_user_only() {
    let channel = DiscordChannel::new_with_partition(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordSessionPartition::UserOnly,
    );
    let event_a = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));
    let event_b = discord_event("2", "hello", "2002", Some("3001"), "1001", Some("alice"));

    let parsed_a = channel
        .parse_gateway_message(&event_a)
        .expect("message A should parse");
    let parsed_b = channel
        .parse_gateway_message(&event_b)
        .expect("message B should parse");
    assert_eq!(parsed_a.session_key, "1001");
    assert_eq!(parsed_a.session_key, parsed_b.session_key);
}

#[test]
fn discord_parse_gateway_message_partition_guild_user() {
    let channel = DiscordChannel::new_with_partition(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordSessionPartition::GuildUser,
    );
    let event_a = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));
    let event_b = discord_event("2", "hello", "2002", Some("3001"), "1001", Some("alice"));
    let event_c = discord_event("3", "hello", "2003", Some("3002"), "1001", Some("alice"));

    let parsed_a = channel
        .parse_gateway_message(&event_a)
        .expect("message A should parse");
    let parsed_b = channel
        .parse_gateway_message(&event_b)
        .expect("message B should parse");
    let parsed_c = channel
        .parse_gateway_message(&event_c)
        .expect("message C should parse");
    assert_eq!(parsed_a.session_key, "3001:1001");
    assert_eq!(parsed_a.session_key, parsed_b.session_key);
    assert_ne!(parsed_a.session_key, parsed_c.session_key);
}

#[test]
fn discord_session_partition_runtime_toggle_changes_strategy() {
    let channel = DiscordChannel::new_with_partition(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordSessionPartition::GuildChannelUser,
    );
    let event_a = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));
    let event_b = discord_event("2", "hello", "2001", Some("3001"), "1002", Some("bob"));

    let parsed_a = channel
        .parse_gateway_message(&event_a)
        .expect("message A should parse");
    let parsed_b = channel
        .parse_gateway_message(&event_b)
        .expect("message B should parse");
    assert_ne!(parsed_a.session_key, parsed_b.session_key);

    channel
        .set_session_partition_mode("channel")
        .expect("mode should be accepted");

    let parsed_a_shared = channel
        .parse_gateway_message(&event_a)
        .expect("message A shared should parse");
    let parsed_b_shared = channel
        .parse_gateway_message(&event_b)
        .expect("message B shared should parse");
    assert_eq!(parsed_a_shared.session_key, "3001:2001");
    assert_eq!(parsed_a_shared.session_key, parsed_b_shared.session_key);
}

#[test]
fn discord_session_partition_mode_rejects_invalid_value() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let error = channel
        .set_session_partition_mode("invalid")
        .expect_err("invalid mode should fail");
    assert!(
        error
            .to_string()
            .contains("invalid discord session partition mode")
    );
}

#[test]
fn discord_parse_gateway_message_parses_slash_interaction_as_command_text() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["alice".to_string()], vec![]);
    let event = discord_slash_interaction_event(
        "9001",
        "session",
        "2001",
        Some("3001"),
        "1001",
        "alice",
        serde_json::json!([
            {
                "name": "memory",
                "type": 1,
                "options": [
                    {
                        "name": "format",
                        "type": 3,
                        "value": "json"
                    }
                ]
            }
        ]),
        2,
    );

    let parsed = channel
        .parse_gateway_message(&event)
        .expect("slash interaction should parse");
    assert_eq!(parsed.sender, "1001");
    assert_eq!(parsed.content, "/session memory json");
    assert_eq!(parsed.session_key, "3001:2001:1001");
}

#[test]
fn discord_parse_gateway_message_parses_slash_prompt_option_with_spaces() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_slash_interaction_event(
        "9002",
        "bg",
        "2001",
        Some("3001"),
        "1001",
        "alice",
        serde_json::json!([
            {
                "name": "prompt",
                "type": 3,
                "value": "collect logs and summarize failures"
            }
        ]),
        2,
    );

    let parsed = channel
        .parse_gateway_message(&event)
        .expect("bg interaction should parse");
    assert_eq!(parsed.content, "/bg collect logs and summarize failures");
}

#[test]
fn discord_parse_gateway_message_ignores_non_command_interaction_payload() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_slash_interaction_event(
        "9003",
        "session",
        "2001",
        Some("3001"),
        "1001",
        "alice",
        serde_json::json!([]),
        4,
    );
    assert!(channel.parse_gateway_message(&event).is_none());
}
