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

use omni_agent::TelegramSessionPartition;

#[test]
fn session_partition_default_is_chat_only() {
    assert_eq!(
        TelegramSessionPartition::default(),
        TelegramSessionPartition::ChatOnly
    );
}

#[test]
fn session_partition_parse_aliases() {
    assert_eq!(
        "chat_user".parse::<TelegramSessionPartition>().ok(),
        Some(TelegramSessionPartition::ChatUser)
    );
    assert_eq!(
        "chat".parse::<TelegramSessionPartition>().ok(),
        Some(TelegramSessionPartition::ChatOnly)
    );
    assert_eq!(
        "user".parse::<TelegramSessionPartition>().ok(),
        Some(TelegramSessionPartition::UserOnly)
    );
    assert_eq!(
        "topic-user".parse::<TelegramSessionPartition>().ok(),
        Some(TelegramSessionPartition::ChatThreadUser)
    );
}

#[test]
fn session_partition_build_session_key() {
    assert_eq!(
        TelegramSessionPartition::ChatUser.build_session_key("-200", "888", None),
        "-200:888"
    );
    assert_eq!(
        TelegramSessionPartition::ChatOnly.build_session_key("-200", "888", None),
        "-200"
    );
    assert_eq!(
        TelegramSessionPartition::UserOnly.build_session_key("-200", "888", None),
        "888"
    );
    assert_eq!(
        TelegramSessionPartition::ChatThreadUser.build_session_key("-200", "888", Some(42)),
        "-200:42:888"
    );
}
