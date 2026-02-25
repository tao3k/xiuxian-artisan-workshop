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

use super::repair_embedding_dimension;

#[test]
fn repair_embedding_dimension_preserves_exact_dimension() {
    let input = vec![0.2_f32, 0.4, 0.6, 0.8];
    let repaired = repair_embedding_dimension(&input, input.len());
    assert_eq!(repaired, input);
}

#[test]
fn repair_embedding_dimension_downsamples_and_normalizes() {
    let input: Vec<f32> = (0..1024).map(|idx| (idx as f32) / 1024.0).collect();
    let repaired = repair_embedding_dimension(&input, 384);
    assert_eq!(repaired.len(), 384);
    let norm = repaired
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    assert!((norm - 1.0).abs() < 1e-4);
}

#[test]
fn repair_embedding_dimension_upsamples_single_value() {
    let repaired = repair_embedding_dimension(&[0.5], 8);
    assert_eq!(repaired, vec![0.5; 8]);
}

#[test]
fn repair_embedding_dimension_handles_zero_target() {
    let repaired = repair_embedding_dimension(&[0.1, 0.2, 0.3], 0);
    assert!(repaired.is_empty());
}
