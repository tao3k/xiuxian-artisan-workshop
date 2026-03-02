//! Integration tests for fusion kernels.

use omni_vector::keyword::fusion::{distance_to_score, rrf_term, rrf_term_batch};

#[test]
fn test_rrf_term() {
    assert!((rrf_term(10.0, 0) - (1.0 / 11.0)).abs() < 1e-6);
    assert!((rrf_term(10.0, 1) - (1.0 / 12.0)).abs() < 1e-6);
}

#[test]
fn test_rrf_term_batch() {
    let ranks = [0_usize, 1, 2];
    let scores = rrf_term_batch(&ranks, 10.0);
    assert_eq!(scores.len(), 3);
    assert!((scores[0] - (1.0 / 11.0)).abs() < 1e-6);
    assert!((scores[1] - (1.0 / 12.0)).abs() < 1e-6);
    assert!((scores[2] - (1.0 / 13.0)).abs() < 1e-6);
}

#[test]
fn test_distance_to_score() {
    assert!((distance_to_score(0.0) - 1.0).abs() < 1e-6);
    assert!((distance_to_score(1.0) - 0.5).abs() < 1e-6);
    assert!(distance_to_score(-0.5) <= 1.0);
}
