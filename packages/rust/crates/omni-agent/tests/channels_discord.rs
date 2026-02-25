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
    Channel, DiscordChannel, DiscordCommandAdminRule, DiscordControlCommandPolicy,
    build_discord_command_admin_rule,
};

fn admin_rule(selectors: &[&str], users: &[&str]) -> DiscordCommandAdminRule {
    build_discord_command_admin_rule(
        selectors.iter().map(|value| value.to_string()).collect(),
        users.iter().map(|value| value.to_string()).collect(),
    )
    .expect("typed admin rule should compile")
}

#[test]
fn discord_channel_name() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    assert_eq!(channel.name(), "discord");
}

#[test]
fn discord_control_command_authorization_supports_selector_rules() {
    let channel = DiscordChannel::new_with_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(
            vec!["ops".to_string()],
            None,
            vec![admin_rule(&["/session partition"], &["alice", "1001"])],
        ),
    )
    .expect("rule specs should compile");

    assert!(channel.is_authorized_for_control_command("alice", "/session partition on"));
    assert!(channel.is_authorized_for_control_command("1001", "/session partition json"));
    assert!(
        !channel.is_authorized_for_control_command("ops", "/session partition on"),
        "matched rule should take precedence over admin_users fallback",
    );
    assert!(
        channel.is_authorized_for_control_command("ops", "/resume status"),
        "non-matching commands should fall back to admin_users",
    );
}

#[test]
fn discord_control_command_authorization_normalizes_rule_and_sender_identities() {
    let channel = DiscordChannel::new_with_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(
            vec!["ops".to_string()],
            None,
            vec![admin_rule(&["/session partition"], &["@Owner"])],
        ),
    )
    .expect("rule specs should compile");

    assert!(channel.is_authorized_for_control_command("@OWNER", "/session partition chat"));
    assert!(channel.is_authorized_for_control_command("owner", "/session partition user"));
}

#[test]
fn discord_control_command_authorization_supports_selector_wildcards() {
    let channel = DiscordChannel::new_with_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(
            vec!["ops".to_string()],
            None,
            vec![
                admin_rule(&["session.*"], &["owner"]),
                admin_rule(&["/reset"], &["owner"]),
            ],
        ),
    )
    .expect("rule specs should compile");

    assert!(channel.is_authorized_for_control_command("owner", "/session partition chat"));
    assert!(channel.is_authorized_for_control_command("owner", "/session reset"));
    assert!(channel.is_authorized_for_control_command("owner", "/reset"));
    assert!(!channel.is_authorized_for_control_command("owner", "/resume status"));
}

#[test]
fn discord_control_command_authorization_supports_cmd_prefix_and_bot_suffix_in_rules() {
    let channel = DiscordChannel::new_with_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(
            vec!["ops".to_string()],
            None,
            vec![
                admin_rule(&["cmd:/session partition"], &["owner"]),
                admin_rule(&["cmd:/reset@mybot"], &["owner"]),
            ],
        ),
    )
    .expect("rule specs should compile");

    assert!(channel.is_authorized_for_control_command("owner", "/session@mybot partition on"));
    assert!(channel.is_authorized_for_control_command("owner", "/reset"));
    assert!(
        !channel.is_authorized_for_control_command("ops", "/session partition on"),
        "matched command-scoped rule should still take precedence over admin_users",
    );
}

#[test]
fn discord_control_command_authorization_rejects_invalid_wildcard_selector() {
    let result =
        build_discord_command_admin_rule(vec!["session*".to_string()], vec!["owner".to_string()]);

    let error = match result {
        Ok(_) => panic!("invalid wildcard selector should fail fast"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("wildcard `*` is only allowed as full selector `*` or suffix `.*`"),
        "unexpected error: {error}",
    );
}

#[test]
fn discord_control_command_authorization_control_allow_from_overrides_rules_and_admins() {
    let channel = DiscordChannel::new_with_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(
            vec!["ops".to_string()],
            Some(vec!["owner".to_string()]),
            vec![admin_rule(&["/session partition"], &["alice"])],
        ),
    )
    .expect("authorization policy should compile");

    assert!(channel.is_authorized_for_control_command("owner", "/session partition on"));
    assert!(channel.is_authorized_for_control_command("owner", "/resume"));
    assert!(
        !channel.is_authorized_for_control_command("alice", "/session partition on"),
        "control_command_allow_from should override command-scoped rules",
    );
    assert!(
        !channel.is_authorized_for_control_command("ops", "/resume"),
        "control_command_allow_from should override admin_users fallback",
    );
}

#[test]
fn discord_control_command_authorization_control_allow_from_empty_denies_all() {
    let channel = DiscordChannel::new_with_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordControlCommandPolicy::new(
            vec!["*".to_string()],
            Some(Vec::new()),
            vec![admin_rule(&["/reset", "/clear"], &["owner"])],
        ),
    )
    .expect("authorization policy should compile");

    assert!(!channel.is_authorized_for_control_command("owner", "/reset"));
    assert!(!channel.is_authorized_for_control_command("alice", "/resume"));
}

#[tokio::test]
async fn discord_listen_returns_not_implemented_error() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let (tx, _rx) = tokio::sync::mpsc::channel(1);
    let error = channel
        .listen(tx)
        .await
        .expect_err("listen should be unimplemented for skeleton");
    assert!(error.to_string().contains("not implemented"));
}
