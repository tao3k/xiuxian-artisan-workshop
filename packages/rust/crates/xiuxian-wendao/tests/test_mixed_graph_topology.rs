//! Topology regression tests for mixed outbound link structures.

use std::collections::HashMap;
use xiuxian_wendao::LinkGraphIndex;

#[tokio::test]
async fn test_mixed_graph_topology_related_from_weighted_seed()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let root = temp.path();

    std::fs::write(
        root.join("note.md"),
        r"
# Section 1
This talks about [[EntityA]].

# Section 2
This links to [[EntityB]].
",
    )?;
    std::fs::write(root.join("EntityA.md"), "Entity A canonical node.")?;
    std::fs::write(root.join("EntityB.md"), "Entity B canonical node.")?;

    let index = LinkGraphIndex::build(root)?;

    let mut seeds = HashMap::new();
    seeds.insert("note".to_string(), 1.0);
    let (related, _) = index.related_from_weighted_seeds_with_diagnostics(&seeds, 1, 10, None);

    let stems: Vec<String> = related.iter().map(|n| n.stem.clone()).collect();
    assert!(
        stems.iter().any(|stem| stem == "EntityA") && stems.iter().any(|stem| stem == "EntityB"),
        "seed note should expose both linked entities, got: {stems:?}"
    );
    Ok(())
}
