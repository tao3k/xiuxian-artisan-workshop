use super::*;

#[test]
fn test_link_graph_search_filters_related_with_distance() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\n[[c]]\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\n[[d]]\n")?;
    write_file(&tmp.path().join("docs/d.md"), "# D\n\nNo links.\n")?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options_distance_1 = LinkGraphSearchOptions {
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        filters: LinkGraphSearchFilters {
            related: Some(LinkGraphRelatedFilter {
                seeds: vec!["b".to_string()],
                max_distance: Some(1),
                ppr: None,
            }),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let hits_1 = index.search_planned("", 10, options_distance_1).1;
    let paths_1: Vec<String> = hits_1.into_iter().map(|row| row.path).collect();
    assert_eq!(
        paths_1,
        vec!["docs/a.md".to_string(), "docs/c.md".to_string()]
    );

    let options_distance_2 = LinkGraphSearchOptions {
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        filters: LinkGraphSearchFilters {
            related: Some(LinkGraphRelatedFilter {
                seeds: vec!["b".to_string()],
                max_distance: Some(2),
                ppr: None,
            }),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let hits_2 = index.search_planned("", 10, options_distance_2).1;
    let paths_2: Vec<String> = hits_2.into_iter().map(|row| row.path).collect();
    assert_eq!(
        paths_2,
        vec![
            "docs/a.md".to_string(),
            "docs/c.md".to_string(),
            "docs/d.md".to_string(),
        ]
    );
    Ok(())
}
