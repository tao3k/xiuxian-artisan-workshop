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

use omni_agent::summarise_drained_turns;

#[test]
fn summarise_drained_turns_intent_first_user() {
    let drained = vec![
        ("user".to_string(), "what is 2+2?".to_string(), 0),
        ("assistant".to_string(), "4".to_string(), 0),
    ];
    let (intent, experience, outcome) = summarise_drained_turns(&drained);
    assert_eq!(intent, "what is 2+2?");
    assert_eq!(experience, "4");
    assert_eq!(outcome, "completed");
}

#[test]
fn summarise_drained_turns_outcome_error() {
    let drained = vec![
        ("user".to_string(), "run tool".to_string(), 0),
        (
            "assistant".to_string(),
            "Error: connection failed".to_string(),
            1,
        ),
    ];
    let (_intent, _experience, outcome) = summarise_drained_turns(&drained);
    assert_eq!(outcome, "error");
}

#[test]
fn summarise_drained_turns_no_user_fallback() {
    let drained = vec![("assistant".to_string(), "ok".to_string(), 0)];
    let (intent, experience, outcome) = summarise_drained_turns(&drained);
    assert_eq!(intent, "(no user message)");
    assert_eq!(experience, "ok");
    assert_eq!(outcome, "completed");
}
