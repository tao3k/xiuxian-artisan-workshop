use super::*;

#[test]
fn test_link_graph_structural_priors_promote_architecture_hub_top3()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/hub.md"),
        "# Hub\n\nArchitecture decision ledger.\n",
    )?;
    write_file(
        &tmp.path().join("docs/leaf-a.md"),
        "# Leaf A\n\nArchitecture decision ledger.\n",
    )?;
    write_file(
        &tmp.path().join("docs/leaf-b.md"),
        "# Leaf B\n\nArchitecture decision ledger.\n",
    )?;
    write_file(
        &tmp.path().join("docs/leaf-c.md"),
        "# Leaf C\n\nArchitecture decision ledger.\n",
    )?;
    for idx in 0..6 {
        write_file(
            &tmp.path().join(format!("docs/ref-{idx}.md")),
            "# Ref\n\n[[hub]]\n",
        )?;
    }

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let boosted_hits = index
        .search_planned(
            "Architecture decision ledger",
            5,
            LinkGraphSearchOptions::default(),
        )
        .1;
    assert!(
        boosted_hits.len() >= 3,
        "expected at least three hits for architecture query"
    );

    let hub_rank = boosted_hits
        .iter()
        .position(|row| row.stem == "hub")
        .ok_or("missing hub hit with structural priors enabled")?;
    assert!(
        hub_rank < 3,
        "expected hub in top-3 with structural priors, got rank={hub_rank}"
    );

    let hub_score_with_priors = boosted_hits
        .iter()
        .find(|row| row.stem == "hub")
        .map(|row| row.score)
        .ok_or("missing hub score with structural priors")?;
    assert!(
        boosted_hits.iter().any(|row| {
            row.stem == "hub"
                && row
                    .match_reason
                    .as_deref()
                    .unwrap_or_default()
                    .contains("graph_rank")
        }),
        "expected hub ranking reason to include graph_rank boost"
    );

    let no_semantic_edge_options = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            edge_types: vec![LinkGraphEdgeType::Structural],
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let baseline_hits = index
        .search_planned("Architecture decision ledger", 5, no_semantic_edge_options)
        .1;
    let hub_score_without_semantic_boost = baseline_hits
        .iter()
        .find(|row| row.stem == "hub")
        .map(|row| row.score)
        .ok_or("missing hub score in structural-only baseline")?;

    assert!(
        hub_score_with_priors > hub_score_without_semantic_boost,
        "expected semantic graph-rank priors to improve hub score (with_priors={hub_score_with_priors} baseline={hub_score_without_semantic_boost})",
    );
    Ok(())
}
