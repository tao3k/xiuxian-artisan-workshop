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
    Channel, TelegramChannel, TelegramCommandAdminRule, TelegramSessionPartition,
    build_telegram_command_admin_rule,
};

fn admin_rule(selectors: &[&str], users: &[&str]) -> TelegramCommandAdminRule {
    build_telegram_command_admin_rule(
        selectors.iter().map(|value| value.to_string()).collect(),
        users.iter().map(|value| value.to_string()).collect(),
    )
    .expect("typed admin rule should compile")
}

#[test]
fn telegram_control_command_authorization_supports_selector_rules() {
    let channel = TelegramChannel::new_with_partition_and_admin_users_and_control_command_allow_from_and_command_rules(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        vec!["2001".to_string()],
        None,
        vec![admin_rule(&["/session partition"], &["1001", "1002"])],
        TelegramSessionPartition::ChatUser,
    )
    .expect("rules should compile");

    assert!(channel.is_authorized_for_control_command("1001", "/session partition on"));
    assert!(channel.is_authorized_for_control_command("1001", "/session partition json"));
    assert!(channel.is_authorized_for_control_command("1002", "/session partition chat"));
    assert!(
        !channel.is_authorized_for_control_command("2001", "/session partition on"),
        "matched rule should take precedence over admin_users fallback",
    );
    assert!(
        channel.is_authorized_for_control_command("2001", "/resume status"),
        "non-matching commands should fall back to admin_users",
    );
}

#[test]
fn telegram_control_command_authorization_normalizes_rule_and_sender_identities() {
    let channel = TelegramChannel::new_with_partition_and_admin_users_and_control_command_allow_from_and_command_rules(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        vec!["2001".to_string()],
        None,
        vec![admin_rule(&["/session partition"], &["telegram:1001"])],
        TelegramSessionPartition::ChatUser,
    )
    .expect("rules should compile");

    assert!(channel.is_authorized_for_control_command("1001", "/session partition chat"));
    assert!(channel.is_authorized_for_control_command("tg:1001", "/session partition user"));
}

#[test]
fn telegram_control_command_authorization_supports_selector_wildcards_and_bot_mentions() {
    let channel = TelegramChannel::new_with_partition_and_admin_users_and_control_command_allow_from_and_command_rules(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        vec!["2001".to_string()],
        None,
        vec![
            admin_rule(&["session.*"], &["3001"]),
            admin_rule(&["/reset"], &["3001"]),
        ],
        TelegramSessionPartition::ChatUser,
    )
    .expect("rules should compile");

    assert!(channel.is_authorized_for_control_command("3001", "/session partition chat"));
    assert!(channel.is_authorized_for_control_command("3001", "/session reset"));
    assert!(channel.is_authorized_for_control_command("3001", "/reset@mybot"));
    assert!(!channel.is_authorized_for_control_command("3001", "/resume status"));
}

#[test]
fn telegram_control_command_authorization_supports_cmd_prefix_and_bot_suffix_in_rules() {
    let channel = TelegramChannel::new_with_partition_and_admin_users_and_control_command_allow_from_and_command_rules(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        vec!["4001".to_string()],
        None,
        vec![
            admin_rule(&["cmd:/session partition"], &["3001"]),
            admin_rule(&["cmd:/reset@mybot"], &["3001"]),
        ],
        TelegramSessionPartition::ChatUser,
    )
    .expect("rules should compile");

    assert!(channel.is_authorized_for_control_command("3001", "/session@mybot partition on"));
    assert!(channel.is_authorized_for_control_command("3001", "/reset"));
    assert!(
        !channel.is_authorized_for_control_command("4001", "/session partition on"),
        "matched command-scoped rule should still take precedence over admin_users",
    );
}

#[test]
fn telegram_control_command_authorization_rejects_invalid_wildcard_selector() {
    let result =
        build_telegram_command_admin_rule(vec!["session*".to_string()], vec!["owner".to_string()]);

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
fn telegram_control_command_authorization_control_allow_from_overrides_rules_and_admins() {
    let channel = TelegramChannel::new_with_partition_and_admin_users_and_control_command_allow_from_and_command_rules(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        vec!["4001".to_string()],
        Some(vec!["3001".to_string()]),
        vec![admin_rule(&["/session partition"], &["1001"])],
        TelegramSessionPartition::ChatUser,
    )
    .expect("authorization policy should compile");

    assert!(channel.is_authorized_for_control_command("3001", "/session partition on"));
    assert!(channel.is_authorized_for_control_command("3001", "/resume"));
    assert!(
        !channel.is_authorized_for_control_command("1001", "/session partition on"),
        "control_command_allow_from should override command-scoped rules",
    );
    assert!(
        !channel.is_authorized_for_control_command("4001", "/resume"),
        "control_command_allow_from should override admin_users fallback",
    );
}

#[test]
fn telegram_control_command_authorization_control_allow_from_empty_denies_all() {
    let channel = TelegramChannel::new_with_partition_and_admin_users_and_control_command_allow_from_and_command_rules(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        vec!["*".to_string()],
        Some(Vec::new()),
        vec![admin_rule(&["/reset", "/clear"], &["3001"])],
        TelegramSessionPartition::ChatUser,
    )
    .expect("authorization policy should compile");

    assert!(!channel.is_authorized_for_control_command("3001", "/reset"));
    assert!(!channel.is_authorized_for_control_command("1001", "/resume"));
}

#[test]
fn telegram_control_command_authorization_ignores_invalid_control_allow_from_entries() {
    let channel = TelegramChannel::new_with_partition_and_admin_users_and_control_command_allow_from_and_command_rules(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        vec!["owner".to_string(), "2001".to_string()],
        Some(vec!["alice".to_string(), "1001".to_string()]),
        Vec::new(),
        TelegramSessionPartition::ChatUser,
    )
    .expect("authorization policy should compile");

    assert!(
        channel.is_authorized_for_control_command("1001", "/session partition on"),
        "numeric entries should remain authorized after normalization",
    );
    assert!(
        !channel.is_authorized_for_control_command("alice", "/session partition on"),
        "username entries should be ignored by normalization",
    );
    assert!(
        !channel.is_authorized_for_control_command("2001", "/session partition on"),
        "global control allowlist override should still take precedence over admin fallback",
    );
}

#[test]
fn telegram_control_command_authorization_does_not_implicitly_promote_allowed_users() {
    let channel = TelegramChannel::new_with_partition(
        "fake-token".to_string(),
        vec!["1001".to_string()],
        vec![],
        TelegramSessionPartition::ChatUser,
    );

    assert!(
        !channel.is_authorized_for_control_command("1001", "/reset"),
        "allowed_users should not implicitly grant privileged command access"
    );
}
