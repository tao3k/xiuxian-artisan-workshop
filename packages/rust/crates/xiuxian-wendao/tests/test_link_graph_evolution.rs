#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::implicit_clone,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::manual_string_new,
    clippy::needless_raw_string_hashes,
    clippy::format_push_string,
    clippy::map_unwrap_or,
    clippy::unnecessary_to_owned,
    clippy::too_many_lines
)]
use std::collections::HashMap;
use xiuxian_wendao::link_graph::ppr_hybrid::{HybridPprKernel, NodeType};

#[test]
fn test_hebbian_activation_boost() {
    let mut kernel = HybridPprKernel::new();

    // 1. Setup two nodes with different saliency levels
    // In a real run, these scores are computed via saliency::calc
    let saliency_a = 1.0; // Normal node
    let saliency_b = 8.5; // High saliency (frequently touched)

    kernel.add_node("A", NodeType::Entity, saliency_a);
    kernel.add_node("B", NodeType::Entity, saliency_b);
    kernel.add_edge("A", "B", 1.0);

    // 2. Run PPR with uniform seeds (both 0.5)
    let mut seeds = HashMap::new();
    seeds.insert("A".to_string(), 0.5);
    seeds.insert("B".to_string(), 0.5);

    kernel.run(&seeds, 0.15, 10, None);

    let top = kernel.top_k(2);

    // Node B should have higher rank because of higher saliency prior
    assert_eq!(
        top[0].0, "B",
        "Node B should be promoted via Hebbian saliency"
    );
    println!("Rank A: {}, Rank B: {}", top[1].1, top[0].1);
    assert!(top[0].1 > top[1].1);
}
