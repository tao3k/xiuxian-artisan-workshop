use super::*;

#[test]
fn test_link_graph_search_edge_type_filter_restricts_structural_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# A\n\n## Section\n\nalpha words here.\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            scope: Some(LinkGraphScope::SectionOnly),
            edge_types: vec![LinkGraphEdgeType::Semantic],
            min_section_words: Some(0),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let hits = index.search_planned("alpha", 10, options).1;
    assert!(hits.is_empty());
    Ok(())
}
