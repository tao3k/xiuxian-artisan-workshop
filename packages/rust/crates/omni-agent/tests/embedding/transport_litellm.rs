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

use super::{
    OLLAMA_PLACEHOLDER_API_KEY, normalize_litellm_embedding_target,
    normalize_openai_compatible_base_url,
};

#[test]
fn normalize_openai_base_url_appends_v1_for_plain_host() {
    assert_eq!(
        normalize_openai_compatible_base_url("http://127.0.0.1:11434"),
        "http://127.0.0.1:11434/v1"
    );
}

#[test]
fn normalize_litellm_target_ollama_uses_openai_compat_with_placeholder_key() {
    let (model, base, key, compat) = normalize_litellm_embedding_target(
        "ollama/qwen3-embedding:0.6b",
        "http://127.0.0.1:11434",
        None,
    );
    assert!(compat);
    assert_eq!(model, "openai/qwen3-embedding:0.6b");
    assert_eq!(base, "http://127.0.0.1:11434/v1");
    assert_eq!(key.as_deref(), Some(OLLAMA_PLACEHOLDER_API_KEY));
}

#[test]
fn normalize_litellm_target_non_ollama_is_passthrough() {
    let (model, base, key, compat) = normalize_litellm_embedding_target(
        "minimax/text-embedding",
        "https://api.minimax.io/v1",
        Some("k"),
    );
    assert!(!compat);
    assert_eq!(model, "minimax/text-embedding");
    assert_eq!(base, "https://api.minimax.io/v1");
    assert_eq!(key.as_deref(), Some("k"));
}
