//! Precision regression for weighted-seed PPR ranking.

use std::collections::HashMap;
use xiuxian_wendao::LinkGraphIndex;

/// Precision test for Non-uniform Seed Distribution (Ref: `HippoRAG` 2).
///
/// Validates that higher semantic weights on seeds correctly influence
/// the structural diffusion results compared to uniform distribution.
#[tokio::test]
async fn test_ppr_weight_precision_impact() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a minimal synthetic graph:
    // A -> B (Standard reference)
    // C -> D (Weak reference)

    // Using a temporary directory for the "notebook"
    let temp = tempfile::tempdir()?;
    let root = temp.path();

    // Create nodes. We use distinct content to ensure they are picked up.
    // A links to B, C links to D.
    let notes = vec![
        ("A", "Content A linking to [[B]]"),
        ("B", "Content B is the target of A"),
        ("C", "Content C linking to [[D]]"),
        ("D", "Content D is the target of C"),
    ];

    for (id, content) in notes {
        let path = root.join(format!("{id}.md"));
        std::fs::write(path, content)?;
    }

    // 2. Build the index
    let index = LinkGraphIndex::build(root)?;

    // 3. Scenario: Weighted Seeds (A=0.99, C=0.01)
    // We want to see if B (neighbor of A) ranks significantly higher than D (neighbor of C).
    let mut seeds_weighted = HashMap::new();
    seeds_weighted.insert("A".to_string(), 0.99);
    seeds_weighted.insert("C".to_string(), 0.01);

    let (related_weighted, _) =
        index.related_from_weighted_seeds_with_diagnostics(&seeds_weighted, 2, 10, None);

    // 4. Verification:
    // In a weighted PPR, B should inherit much more 'probability mass' from A than D does from C.
    // Thus B should be the first non-seed result.

    let stems: Vec<String> = related_weighted.iter().map(|n| n.stem.clone()).collect();
    println!("Ranked stems: {stems:?}");

    // Check relative ranking
    let pos_b = stems.iter().position(|s| s == "B");
    let pos_d = stems.iter().position(|s| s == "D");

    match (pos_b, pos_d) {
        (Some(pb), Some(pd)) => {
            assert!(
                pb < pd,
                "B (neighbor of 0.99 seed) should rank higher than D (neighbor of 0.01 seed). B at {pb}, D at {pd}",
            );
        }
        _ => panic!("Expected both B and D to be in related results. Found stems: {stems:?}"),
    }
    Ok(())
}
