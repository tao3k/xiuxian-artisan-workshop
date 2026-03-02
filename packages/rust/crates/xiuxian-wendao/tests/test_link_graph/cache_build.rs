use super::*;

#[test]
fn test_link_graph_build_with_cache_reuses_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_cache_prefix();
    clear_cache_keys(&prefix)?;

    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# Alpha\n\nThis is alpha.\n\n[[b]]\n",
    )?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "# Beta\n\nThis is beta.\n\n[[a]]\n",
    )?;

    let index1 = LinkGraphIndex::build_with_cache_with_valkey(
        tmp.path(),
        &[],
        &[],
        "redis://127.0.0.1:6379/0",
        Some(&prefix),
        Some(300),
    )
    .map_err(|e| e.clone())?;
    let index2 = LinkGraphIndex::build_with_cache_with_valkey(
        tmp.path(),
        &[],
        &[],
        "redis://127.0.0.1:6379/0",
        Some(&prefix),
        Some(300),
    )
    .map_err(|e| e.clone())?;

    assert_eq!(index1.stats().total_notes, 2);
    assert_eq!(index1.stats().total_notes, index2.stats().total_notes);
    assert_eq!(index1.stats().links_in_graph, index2.stats().links_in_graph);

    let key_count = count_cache_keys(&prefix)?;
    assert!(key_count >= 1, "expected at least one valkey cache key");
    clear_cache_keys(&prefix)?;
    Ok(())
}

#[test]
fn test_link_graph_build_with_cache_detects_file_change() -> Result<(), Box<dyn std::error::Error>>
{
    let prefix = unique_cache_prefix();
    clear_cache_keys(&prefix)?;

    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# Alpha\n\nlegacy phrase\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# Beta\n\nstable note\n")?;

    let _ = LinkGraphIndex::build_with_cache_with_valkey(
        tmp.path(),
        &[],
        &[],
        "redis://127.0.0.1:6379/0",
        Some(&prefix),
        Some(300),
    )
    .map_err(|e| e.clone())?;

    write_file(
        &tmp.path().join("docs/a.md"),
        "# Alpha\n\nupdated phrase for cache invalidation\n",
    )?;

    let refreshed = LinkGraphIndex::build_with_cache_with_valkey(
        tmp.path(),
        &[],
        &[],
        "redis://127.0.0.1:6379/0",
        Some(&prefix),
        Some(300),
    )
    .map_err(|e| e.clone())?;
    let hits = refreshed
        .search_planned(
            "updated phrase for cache invalidation",
            5,
            LinkGraphSearchOptions::default(),
        )
        .1;
    assert!(!hits.is_empty(), "updated content should be searchable");
    assert_eq!(hits[0].stem, "a");
    clear_cache_keys(&prefix)?;
    Ok(())
}

#[test]
fn test_link_graph_build_with_cache_seeds_saliency_from_frontmatter()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_cache_prefix();
    clear_cache_keys(&prefix)?;

    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "---\nsaliency_base: 9.0\ndecay_rate: 0.2\n---\n# Alpha\n\n[[b]]\n",
    )?;
    write_file(&tmp.path().join("docs/b.md"), "# Beta\n\n[[a]]\n")?;

    let _index = LinkGraphIndex::build_with_cache_with_valkey(
        tmp.path(),
        &[],
        &[],
        "redis://127.0.0.1:6379/0",
        Some(&prefix),
        Some(300),
    )
    .map_err(|e| e.clone())?;

    let state =
        valkey_saliency_get_with_valkey("docs/a", "redis://127.0.0.1:6379/0", Some(&prefix))
            .map_err(|e| e.clone())?;
    assert!(state.is_some(), "expected seeded saliency state");
    let seeded = state.ok_or("missing seeded saliency state for docs/a")?;
    let expected =
        compute_link_graph_saliency(9.0, 0.2, 0, 0.0, LinkGraphSaliencyPolicy::default());
    assert!((seeded.current_saliency - expected).abs() < 1e-9);

    clear_cache_keys(&prefix)?;
    Ok(())
}
