use super::*;

#[test]
fn test_link_graph_search_edge_type_filter_allows_verified_for_graph_filters()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nNo links.\n")?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            link_to: Some(LinkGraphLinkFilter {
                seeds: vec!["b".to_string()],
                ..LinkGraphLinkFilter::default()
            }),
            edge_types: vec![LinkGraphEdgeType::Verified],
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let hits = index.search_planned("", 10, options).1;
    assert!(
        hits.iter().any(|row| row.stem == "a"),
        "expected verified edge type to keep graph filters enabled"
    );
    Ok(())
}
