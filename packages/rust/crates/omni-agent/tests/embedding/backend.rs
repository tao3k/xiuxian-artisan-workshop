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

use super::{EmbeddingBackendMode, parse_backend_mode};

#[test]
fn parse_backend_mode_supports_openai_and_mistral_aliases() {
    assert_eq!(
        parse_backend_mode(Some("openai_http")),
        EmbeddingBackendMode::OpenAiHttp
    );
    assert_eq!(
        parse_backend_mode(Some("mistral_rs")),
        EmbeddingBackendMode::MistralLocal
    );
    assert_eq!(
        parse_backend_mode(Some("mistral-http")),
        EmbeddingBackendMode::MistralLocal
    );
}

#[test]
fn parse_backend_mode_retains_legacy_http_alias() {
    assert_eq!(parse_backend_mode(Some("http")), EmbeddingBackendMode::Http);
    assert_eq!(
        parse_backend_mode(Some("client")),
        EmbeddingBackendMode::Http
    );
}
