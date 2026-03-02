use super::*;

#[test]
fn test_compute_link_graph_saliency_activation_boosts_score() {
    let policy = LinkGraphSaliencyPolicy::default();
    let without_activation = compute_link_graph_saliency(5.0, 0.02, 0, 2.0, policy);
    let with_activation = compute_link_graph_saliency(5.0, 0.02, 8, 2.0, policy);
    assert!(with_activation > without_activation);
}
