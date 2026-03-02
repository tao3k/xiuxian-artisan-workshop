use super::*;

#[test]
fn test_link_graph_search_filters_related_accepts_ppr_options()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\n[[c]]\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\n[[d]]\n")?;
    write_file(&tmp.path().join("docs/d.md"), "# D\n\nNo links.\n")?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options = LinkGraphSearchOptions {
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        filters: LinkGraphSearchFilters {
            related: Some(LinkGraphRelatedFilter {
                seeds: vec!["b".to_string()],
                max_distance: Some(2),
                ppr: Some(LinkGraphRelatedPprOptions {
                    alpha: Some(0.9),
                    max_iter: Some(64),
                    tol: Some(1e-6),
                    subgraph_mode: Some(LinkGraphPprSubgraphMode::Force),
                }),
            }),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let hits = index.search_planned("", 10, options).1;
    let paths: Vec<String> = hits.into_iter().map(|row| row.path).collect();
    assert_eq!(
        paths,
        vec![
            "docs/a.md".to_string(),
            "docs/c.md".to_string(),
            "docs/d.md".to_string(),
        ]
    );
    Ok(())
}
