use super::*;

#[test]
fn test_link_graph_neighbors_related_metadata_and_toc() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("root/a.md"),
        "# Alpha\n\n[[b]]\n[[sub/c]]\n",
    )?;
    write_file(
        &tmp.path().join("root/b.md"),
        "---\ntags:\n  - one\n  - two\n---\n\n[[a]]\n",
    )?;
    write_file(&tmp.path().join("root/sub/c.md"), "# C\n\n[[a]]\n")?;

    let index = LinkGraphIndex::build(&tmp.path().join("root")).map_err(|e| e.clone())?;

    let neighbors = index.neighbors("a", LinkGraphDirection::Both, 1, 10);
    assert_eq!(neighbors.len(), 2);
    assert!(neighbors.iter().any(|row| row.stem == "b"));
    assert!(neighbors.iter().any(|row| row.stem == "c"));
    for row in &neighbors {
        assert_eq!(row.distance, 1);
        assert_eq!(row.direction, LinkGraphDirection::Both);
    }

    let related = index.related("a", 2, 10);
    assert!(related.iter().any(|row| row.stem == "b"));

    let metadata = index.metadata("b").ok_or("missing metadata")?;
    assert_eq!(metadata.stem, "b");
    assert_eq!(metadata.path, "b.md");
    assert_eq!(metadata.tags, vec!["one".to_string(), "two".to_string()]);

    let toc = index.toc(10);
    assert_eq!(toc.len(), 3);
    assert!(toc.iter().any(|row| row.path == "a.md"));
    assert!(toc.iter().any(|row| row.path == "b.md"));
    assert!(toc.iter().any(|row| row.path == "sub/c.md"));

    Ok(())
}

#[test]
fn test_link_graph_related_with_diagnostics_returns_metrics()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("root/a.md"), "# A\n\n[[b]]\n")?;
    write_file(&tmp.path().join("root/b.md"), "# B\n\n[[c]]\n")?;
    write_file(&tmp.path().join("root/c.md"), "# C\n\n[[d]]\n")?;
    write_file(&tmp.path().join("root/d.md"), "# D\n\nNo links.\n")?;
    let index = LinkGraphIndex::build(&tmp.path().join("root")).map_err(|e| e.clone())?;

    let ppr = LinkGraphRelatedPprOptions {
        alpha: Some(0.9),
        max_iter: Some(64),
        tol: Some(1e-6),
        subgraph_mode: Some(LinkGraphPprSubgraphMode::Force),
    };
    let (rows, diagnostics) = index.related_with_diagnostics("b", 2, 10, Some(&ppr));
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().any(|row| row.stem == "a"));
    assert!(rows.iter().any(|row| row.stem == "c"));
    assert!(rows.iter().any(|row| row.stem == "d"));

    let metrics = diagnostics.ok_or("missing related diagnostics")?;
    assert!((metrics.alpha - 0.9_f64).abs() < 1e-12_f64);
    assert_eq!(metrics.max_iter, 64);
    assert!((metrics.tol - 1e-6_f64).abs() < 1e-12_f64);
    assert!(metrics.iteration_count >= 1);
    assert!(metrics.final_residual >= 0.0);
    assert_eq!(metrics.candidate_count, 3);
    assert!(metrics.candidate_cap >= metrics.candidate_count);
    assert!(!metrics.candidate_capped);
    assert_eq!(metrics.graph_node_count, 8);
    assert_eq!(metrics.subgraph_count, 1);
    assert_eq!(metrics.partition_max_node_count, 8);
    assert_eq!(metrics.partition_min_node_count, 8);
    assert!((metrics.partition_avg_node_count - 8.0_f64).abs() < 1e-12_f64);
    assert!(metrics.total_duration_ms >= 0.0);
    assert!(metrics.partition_duration_ms >= 0.0);
    assert!(metrics.kernel_duration_ms >= 0.0);
    assert!(metrics.fusion_duration_ms >= 0.0);
    assert_eq!(metrics.subgraph_mode, LinkGraphPprSubgraphMode::Force);
    assert!(metrics.horizon_restricted);
    assert!(metrics.time_budget_ms > 0.0);
    assert!(!metrics.timed_out);

    Ok(())
}

#[test]
fn test_link_graph_related_from_seeds_with_diagnostics_partitions_when_forced()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("root/a.md"), "# A\n\n[[b]]\n")?;
    write_file(&tmp.path().join("root/b.md"), "# B\n\n[[c]]\n")?;
    write_file(&tmp.path().join("root/c.md"), "# C\n\nNo links.\n")?;
    write_file(&tmp.path().join("root/d.md"), "# D\n\n[[e]]\n")?;
    write_file(&tmp.path().join("root/e.md"), "# E\n\n[[f]]\n")?;
    write_file(&tmp.path().join("root/f.md"), "# F\n\nNo links.\n")?;
    let index = LinkGraphIndex::build(&tmp.path().join("root")).map_err(|e| e.clone())?;

    let seeds = vec!["b".to_string(), "e".to_string()];
    let ppr = LinkGraphRelatedPprOptions {
        alpha: Some(0.85),
        max_iter: Some(48),
        tol: Some(1e-6),
        subgraph_mode: Some(LinkGraphPprSubgraphMode::Force),
    };
    let (rows, diagnostics) = index.related_from_seeds_with_diagnostics(&seeds, 2, 20, Some(&ppr));
    let metrics = diagnostics.ok_or("missing related diagnostics")?;
    assert_eq!(metrics.subgraph_mode, LinkGraphPprSubgraphMode::Force);
    assert!(metrics.horizon_restricted);
    assert_eq!(metrics.subgraph_count, 2);
    assert_eq!(metrics.partition_max_node_count, 3);
    assert_eq!(metrics.partition_min_node_count, 3);
    assert!((metrics.partition_avg_node_count - 3.0_f64).abs() < 1e-12_f64);
    assert!(metrics.total_duration_ms >= 0.0);
    assert!(metrics.partition_duration_ms >= 0.0);
    assert!(metrics.kernel_duration_ms >= 0.0);
    assert!(metrics.fusion_duration_ms >= 0.0);
    assert!(metrics.candidate_cap >= metrics.candidate_count);
    assert!(metrics.time_budget_ms > 0.0);

    let mut stems: Vec<String> = rows.into_iter().map(|row| row.stem).collect();
    stems.sort_unstable();
    assert_eq!(stems, vec!["a", "c", "d", "f"]);
    Ok(())
}
