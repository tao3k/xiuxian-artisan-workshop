//! Weighted-seed PPR behavior checks.

use std::collections::HashMap;
use xiuxian_wendao::LinkGraphIndex;

#[tokio::test]
async fn test_ppr_non_uniform_seed_bias() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let root = temp.path();

    std::fs::write(root.join("A.md"), "A links to [[B]]")?;
    std::fs::write(root.join("B.md"), "B node")?;
    std::fs::write(root.join("C.md"), "C links to [[D]]")?;
    std::fs::write(root.join("D.md"), "D node")?;

    let index = LinkGraphIndex::build(root)?;

    let mut seeds = HashMap::new();
    seeds.insert("A".to_string(), 0.9);
    seeds.insert("C".to_string(), 0.1);

    let (related, _) = index.related_from_weighted_seeds_with_diagnostics(&seeds, 2, 10, None);
    let stems: Vec<String> = related.iter().map(|node| node.stem.clone()).collect();

    let pos_b = stems.iter().position(|stem| stem == "B");
    let pos_d = stems.iter().position(|stem| stem == "D");
    match (pos_b, pos_d) {
        (Some(b), Some(d)) => assert!(
            b < d,
            "expected B to outrank D under non-uniform seeds, got stems: {stems:?}"
        ),
        _ => panic!("expected both B and D in results, got stems: {stems:?}"),
    }
    Ok(())
}
