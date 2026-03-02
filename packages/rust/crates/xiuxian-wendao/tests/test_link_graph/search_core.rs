use super::*;

#[test]
fn test_link_graph_build_search_and_stats() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# Alpha\n\nThis is alpha.\n\n[[b]]\n",
    )?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "---\ntitle: Beta Doc\ntags:\n  - tag-x\n---\n\n[[a]]\n",
    )?;
    write_file(&tmp.path().join("docs/c.md"), "# Gamma\n\nNo links here.\n")?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let stats = index.stats();
    assert_eq!(stats.total_notes, 3);
    assert_eq!(stats.nodes_in_graph, 3);
    assert_eq!(stats.links_in_graph, 2);
    assert_eq!(stats.orphans, 1);

    let hits = index
        .search_planned("beta", 5, LinkGraphSearchOptions::default())
        .1;
    assert!(!hits.is_empty());
    assert_eq!(hits[0].stem, "b");
    assert_eq!(hits[0].path, "docs/b.md");

    Ok(())
}

#[test]
fn test_link_graph_search_limit_is_enforced() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nshared keyword\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nshared keyword\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\nshared keyword\n")?;
    write_file(&tmp.path().join("docs/d.md"), "# D\n\nshared keyword\n")?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let hits = index
        .search_planned("shared keyword", 2, LinkGraphSearchOptions::default())
        .1;
    assert_eq!(hits.len(), 2);
    Ok(())
}

#[test]
fn test_link_graph_search_short_circuits_id_directive() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# Alpha\n\nalpha keyword only\n",
    )?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "# Beta\n\nbeta specific content\n",
    )?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let (parsed, hits) = index.search_planned(
        "id:docs/b this phrase should not be required",
        5,
        LinkGraphSearchOptions::default(),
    );

    assert_eq!(parsed.direct_id.as_deref(), Some("docs/b"));
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, "docs/b.md");
    assert_eq!(hits[0].match_reason.as_deref(), Some("direct_id"));
    Ok(())
}

#[test]
fn test_link_graph_search_payload_short_circuits_id_directive()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("notes/release.md"),
        "# Release\n\nrelease notes\n",
    )?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let payload =
        index.search_planned_payload("id:notes/release", 5, LinkGraphSearchOptions::default());

    assert_eq!(payload.hit_count, 1);
    assert_eq!(payload.results.len(), 1);
    assert_eq!(payload.results[0].path, "notes/release.md");
    assert_eq!(
        payload.results[0].match_reason.as_deref(),
        Some("direct_id")
    );
    Ok(())
}

#[test]
fn test_link_graph_search_fts_boosts_high_reference_notes() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/hub.md"), "# Hub\n\nshared phrase\n")?;
    write_file(
        &tmp.path().join("docs/leaf.md"),
        "# Leaf\n\nshared phrase\n",
    )?;
    write_file(&tmp.path().join("docs/ref-1.md"), "# R1\n\n[[hub]]\n")?;
    write_file(&tmp.path().join("docs/ref-2.md"), "# R2\n\n[[hub]]\n")?;
    write_file(&tmp.path().join("docs/ref-3.md"), "# R3\n\n[[hub]]\n")?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let hits = index
        .search_planned("shared phrase", 5, LinkGraphSearchOptions::default())
        .1;
    assert!(hits.len() >= 2);
    assert_eq!(hits[0].stem, "hub");
    assert_eq!(hits[1].stem, "leaf");
    assert!(hits[0].score > hits[1].score);
    Ok(())
}

#[test]
fn test_link_graph_search_fts_prefers_phrase_specific_note_over_generic_index()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/reference/README.md"),
        "# Reference Documentation\n\nThis index briefly mentions checkpoint schema.\n",
    )?;
    write_file(
        &tmp.path().join("docs/explanation/vector-checkpoint.md"),
        "# Checkpoint Schema and Vector Checkpoint System\n\n\
checkpoint schema is the canonical runtime contract.\n\n\
The checkpoint schema defines validation behavior for checkpoint records.\n",
    )?;
    for idx in 0..6 {
        write_file(
            &tmp.path().join(format!("docs/reference/ref-{idx}.md")),
            "# Ref\n\n[[README]]\n",
        )?;
    }

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let hits = index
        .search_planned("checkpoint schema", 5, LinkGraphSearchOptions::default())
        .1;

    assert!(
        !hits.is_empty(),
        "expected at least one hit for 'checkpoint schema'"
    );
    assert_eq!(hits[0].stem, "vector-checkpoint");
    Ok(())
}

#[test]
fn test_link_graph_search_sort_by_path() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("zeta.md"), "# Zeta\n\nkeyword\n")?;
    write_file(&tmp.path().join("alpha.md"), "# Alpha\n\nkeyword\n")?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options = LinkGraphSearchOptions {
        match_strategy: LinkGraphMatchStrategy::Fts,
        case_sensitive: false,
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        ..LinkGraphSearchOptions::default()
    };
    let hits = index.search_planned(".md", 5, options).1;
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].path, "alpha.md");
    assert_eq!(hits[1].path, "zeta.md");
    Ok(())
}

#[test]
fn test_link_graph_search_planned_payload_has_consistent_counts()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# Alpha\n\n## Architecture\n\ngraph engine planner token\n",
    )?;
    write_file(&tmp.path().join("docs/b.md"), "# Beta\n\ngraph token\n")?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let payload =
        index.search_planned_payload("architecture graph", 10, LinkGraphSearchOptions::default());
    assert_eq!(payload.hit_count, payload.results.len());
    assert_eq!(payload.hit_count, payload.hits.len());
    assert_eq!(payload.graph_hit_count, payload.hit_count);
    assert_eq!(payload.query, "architecture graph");
    assert!(payload.section_hit_count <= payload.hit_count);
    assert!(payload.hits.iter().all(|hit| hit.score >= 0.0));
    assert!((0.0..=1.0).contains(&payload.graph_confidence_score));
    assert!(
        matches!(
            payload.graph_confidence_level,
            LinkGraphConfidenceLevel::None
                | LinkGraphConfidenceLevel::Low
                | LinkGraphConfidenceLevel::Medium
                | LinkGraphConfidenceLevel::High
        ),
        "invalid confidence level in payload"
    );
    assert!(
        matches!(
            payload.requested_mode,
            LinkGraphRetrievalMode::GraphOnly
                | LinkGraphRetrievalMode::Hybrid
                | LinkGraphRetrievalMode::VectorOnly
        ),
        "invalid requested_mode in payload"
    );
    assert!(
        matches!(
            payload.selected_mode,
            LinkGraphRetrievalMode::GraphOnly
                | LinkGraphRetrievalMode::Hybrid
                | LinkGraphRetrievalMode::VectorOnly
        ),
        "invalid selected_mode in payload"
    );
    let retrieval_plan = payload
        .retrieval_plan
        .ok_or("missing retrieval_plan in planned payload")?;
    assert_eq!(
        retrieval_plan.schema,
        LINK_GRAPH_RETRIEVAL_PLAN_SCHEMA_VERSION
    );
    assert_eq!(retrieval_plan.graph_hit_count, payload.hit_count);
    assert_eq!(retrieval_plan.source_hint_count, payload.source_hint_count);
    assert!((0.0..=1.0).contains(&retrieval_plan.graph_confidence_score));
    assert!(!payload.reason.trim().is_empty());
    Ok(())
}

#[test]
fn test_link_graph_search_planned_payload_escalates_when_graph_hits_are_empty()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# Alpha\n\ngraph token\n")?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let payload = index.search_planned_payload(
        "missing-term-never-hit",
        5,
        LinkGraphSearchOptions::default(),
    );
    assert_eq!(payload.hit_count, 0);
    assert_eq!(payload.graph_hit_count, 0);
    assert_eq!(payload.selected_mode, LinkGraphRetrievalMode::VectorOnly);
    assert_eq!(
        payload.graph_confidence_level,
        LinkGraphConfidenceLevel::None
    );
    assert!((payload.graph_confidence_score - 0.0_f64).abs() < 1e-12_f64);
    let reason = &payload.reason;
    assert!(
        payload.reason == "graph_insufficient" || payload.reason == "vector_only_requested",
        "unexpected reason for empty graph hits: {reason}",
    );
    Ok(())
}
