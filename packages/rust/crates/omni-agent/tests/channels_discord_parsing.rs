//! Discord gateway message parsing tests for session extraction and ACL context.

use omni_agent::{Channel, DiscordChannel, DiscordSessionPartition};

macro_rules! parse_message {
    ($channel:expr, $event:expr, $context:literal) => {{
        match $channel.parse_gateway_message($event) {
            Some(parsed) => parsed,
            None => panic!($context),
        }
    }};
}

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
    args: DiscordSlashInteractionEventArgs<'_>,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "id": args.interaction_id,
        "application_id": "5001",
        "type": args.interaction_type,
        "data": {
            "id": "6001",
            "name": args.command_name,
            "type": 1
        },
        "channel_id": args.channel_id,
        "token": "interaction-token",
        "version": 1,
        "locale": "en-US",
        "entitlements": [],
        "attachment_size_limit": 8_388_608,
        "user": {
            "id": args.user_id,
            "username": args.username
        }
    });
    if let Some(guild) = args.guild_id {
        payload["guild_id"] = serde_json::Value::String(guild.to_string());
        payload["guild_locale"] = serde_json::Value::String("en-US".to_string());
    }
    if !args.options.is_null() {
        payload["data"]["options"] = args.options;
    }
    payload
}

struct DiscordSlashInteractionEventArgs<'a> {
    interaction_id: &'a str,
    command_name: &'a str,
    channel_id: &'a str,
    guild_id: Option<&'a str>,
    user_id: &'a str,
    username: &'a str,
    options: serde_json::Value,
    interaction_type: u8,
}

#[test]
fn discord_parse_gateway_message_allows_allowed_user() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["alice".to_string()], vec![]);
    let event = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));

    let parsed = parse_message!(channel, &event, "message should parse");
    assert_eq!(parsed.sender, "1001");
    assert_eq!(parsed.recipient, "2001");
    assert_eq!(parsed.channel, "discord");
}

#[test]
fn discord_parse_gateway_message_allows_allowed_guild() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec![], vec!["3001".to_string()]);
    let event = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("unknown"));

    let parsed = parse_message!(channel, &event, "message should parse");
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

    let parsed = parse_message!(channel, &event, "message should parse");
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

    let parsed = parse_message!(channel, &event, "message should parse");
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

    let parsed_a = parse_message!(channel, &event_a, "message A should parse");
    let parsed_b = parse_message!(channel, &event_b, "message B should parse");
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

    let parsed_a = parse_message!(channel, &event_a, "message A should parse");
    let parsed_b = parse_message!(channel, &event_b, "message B should parse");
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

    let parsed_a = parse_message!(channel, &event_a, "message A should parse");
    let parsed_b = parse_message!(channel, &event_b, "message B should parse");
    let parsed_c = parse_message!(channel, &event_c, "message C should parse");
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

    let parsed_a = parse_message!(channel, &event_a, "message A should parse");
    let parsed_b = parse_message!(channel, &event_b, "message B should parse");
    assert_ne!(parsed_a.session_key, parsed_b.session_key);

    if let Err(error) = channel.set_session_partition_mode("channel") {
        panic!("mode should be accepted: {error}");
    }

    let parsed_a_shared = parse_message!(channel, &event_a, "message A shared should parse");
    let parsed_b_shared = parse_message!(channel, &event_b, "message B shared should parse");
    assert_eq!(parsed_a_shared.session_key, "3001:2001");
    assert_eq!(parsed_a_shared.session_key, parsed_b_shared.session_key);
}

#[test]
fn discord_session_partition_mode_rejects_invalid_value() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let mode_result = channel.set_session_partition_mode("invalid");
    let error = match mode_result {
        Ok(()) => panic!("invalid mode should fail"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("invalid discord session partition mode")
    );
}

#[test]
fn discord_parse_gateway_message_parses_slash_interaction_as_command_text() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["alice".to_string()], vec![]);
    let event = discord_slash_interaction_event(DiscordSlashInteractionEventArgs {
        interaction_id: "9001",
        command_name: "session",
        channel_id: "2001",
        guild_id: Some("3001"),
        user_id: "1001",
        username: "alice",
        options: serde_json::json!([
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
        interaction_type: 2,
    });

    let parsed = parse_message!(channel, &event, "slash interaction should parse");
    assert_eq!(parsed.sender, "1001");
    assert_eq!(parsed.content, "/session memory json");
    assert_eq!(parsed.session_key, "3001:2001:1001");
}

#[test]
fn discord_parse_gateway_message_parses_slash_prompt_option_with_spaces() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_slash_interaction_event(DiscordSlashInteractionEventArgs {
        interaction_id: "9002",
        command_name: "bg",
        channel_id: "2001",
        guild_id: Some("3001"),
        user_id: "1001",
        username: "alice",
        options: serde_json::json!([
            {
                "name": "prompt",
                "type": 3,
                "value": "collect logs and summarize failures"
            }
        ]),
        interaction_type: 2,
    });

    let parsed = parse_message!(channel, &event, "bg interaction should parse");
    assert_eq!(parsed.content, "/bg collect logs and summarize failures");
}

#[test]
fn discord_parse_gateway_message_ignores_non_command_interaction_payload() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_slash_interaction_event(DiscordSlashInteractionEventArgs {
        interaction_id: "9003",
        command_name: "session",
        channel_id: "2001",
        guild_id: Some("3001"),
        user_id: "1001",
        username: "alice",
        options: serde_json::json!([]),
        interaction_type: 4,
    });
    assert!(channel.parse_gateway_message(&event).is_none());
}
