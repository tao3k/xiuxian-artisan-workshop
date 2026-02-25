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

mod managed_runtime {
    pub mod parsing {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/channels/managed_runtime/parsing/types.rs"
        ));
    }
    pub mod session_partition {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/channels/managed_runtime/session_partition.rs"
        ));
    }
}

use managed_runtime::parsing::{
    SessionPartitionModeToken, parse_session_partition_mode_token, session_partition_mode_name,
};
use managed_runtime::session_partition::{SessionPartitionProfile, supported_modes};

#[test]
fn supported_modes_are_always_parseable() {
    for mode in supported_modes(SessionPartitionProfile::Telegram) {
        assert!(
            parse_session_partition_mode_token(mode).is_some(),
            "telegram mode should be parseable: {mode}",
        );
    }
    for mode in supported_modes(SessionPartitionProfile::Discord) {
        assert!(
            parse_session_partition_mode_token(mode).is_some(),
            "discord mode should be parseable: {mode}",
        );
    }
}

#[test]
fn partition_mode_name_and_parser_roundtrip() {
    let cases = [
        SessionPartitionModeToken::Chat,
        SessionPartitionModeToken::ChatUser,
        SessionPartitionModeToken::User,
        SessionPartitionModeToken::ChatThreadUser,
        SessionPartitionModeToken::GuildChannelUser,
        SessionPartitionModeToken::Channel,
        SessionPartitionModeToken::GuildUser,
    ];
    for case in cases {
        let name = session_partition_mode_name(case);
        assert_eq!(parse_session_partition_mode_token(name), Some(case));
    }
}
