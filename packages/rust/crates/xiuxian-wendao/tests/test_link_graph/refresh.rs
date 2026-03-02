use super::*;

#[test]
fn test_link_graph_refresh_incremental_updates_and_deletes_notes()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let b_path = tmp.path().join("docs/b.md");
    write_file(&tmp.path().join("docs/a.md"), "# Alpha\n\n[[b]]\n")?;
    write_file(&b_path, "# Beta\n\nold keyword\n")?;

    let mut index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let old_hits = index
        .search_planned("old keyword", 5, LinkGraphSearchOptions::default())
        .1;
    assert_eq!(old_hits.len(), 1);

    write_file(&b_path, "# Beta\n\nnew keyword\n")?;
    let mode = index
        .refresh_incremental_with_threshold(std::slice::from_ref(&b_path), 256)
        .map_err(|e| e.clone())?;
    assert_eq!(mode, LinkGraphRefreshMode::Delta);
    let new_hits = index
        .search_planned("new keyword", 5, LinkGraphSearchOptions::default())
        .1;
    assert_eq!(new_hits.len(), 1);
    assert_eq!(new_hits[0].stem, "b");

    fs::remove_file(&b_path)?;
    let mode = index
        .refresh_incremental_with_threshold(std::slice::from_ref(&b_path), 256)
        .map_err(|e| e.clone())?;
    assert_eq!(mode, LinkGraphRefreshMode::Delta);
    let stats = index.stats();
    assert_eq!(stats.total_notes, 1);
    assert_eq!(stats.links_in_graph, 0);
    Ok(())
}

#[test]
fn test_link_graph_refresh_incremental_with_threshold_modes()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let a_path = tmp.path().join("docs/a.md");
    let b_path = tmp.path().join("docs/b.md");
    write_file(&a_path, "# Alpha\n\n[[b]]\n")?;
    write_file(&b_path, "# Beta\n\n[[a]]\n")?;

    let mut index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let noop = index
        .refresh_incremental_with_threshold(&[], 1)
        .map_err(|e| e.clone())?;
    assert_eq!(noop, LinkGraphRefreshMode::Noop);

    write_file(&a_path, "# Alpha\n\n[[b]]\n\nnew token\n")?;
    let full = index
        .refresh_incremental_with_threshold(std::slice::from_ref(&a_path), 1)
        .map_err(|e| e.clone())?;
    assert_eq!(full, LinkGraphRefreshMode::Full);

    let hits = index
        .search_planned("new token", 5, LinkGraphSearchOptions::default())
        .1;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].stem, "a");
    Ok(())
}
