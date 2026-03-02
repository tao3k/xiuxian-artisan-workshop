use super::*;

#[test]
fn test_compute_link_graph_saliency_clamps_bounds() {
    let policy = LinkGraphSaliencyPolicy {
        alpha: 0.5,
        minimum: 1.0,
        maximum: 10.0,
    };

    let decayed = compute_link_graph_saliency(5.0, 0.10, 0, 30.0, policy);
    assert!(decayed >= 1.0);
    assert!(decayed < 5.0);

    let boosted = compute_link_graph_saliency(5.0, 0.0, 10_000, 0.0, policy);
    assert!(boosted <= 10.0);
    assert!(boosted > 9.0);
}
