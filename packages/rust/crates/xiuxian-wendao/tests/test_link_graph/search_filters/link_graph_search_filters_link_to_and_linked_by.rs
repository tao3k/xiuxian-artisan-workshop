use super::*;

#[test]
fn test_link_graph_search_filters_link_to_and_linked_by() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\n[[d]]\n")?;
    write_file(&tmp.path().join("docs/d.md"), "# D\n\nNo links.\n")?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let link_to_options = LinkGraphSearchOptions {
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        filters: LinkGraphSearchFilters {
            link_to: Some(LinkGraphLinkFilter {
                seeds: vec!["b".to_string()],
                ..LinkGraphLinkFilter::default()
            }),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let link_to_hits = index.search_planned("", 10, link_to_options).1;
    let link_to_paths: Vec<String> = link_to_hits.into_iter().map(|row| row.path).collect();
    assert_eq!(
        link_to_paths,
        vec!["docs/a.md".to_string(), "docs/c.md".to_string()]
    );

    let linked_by_options = LinkGraphSearchOptions {
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        filters: LinkGraphSearchFilters {
            linked_by: Some(LinkGraphLinkFilter {
                seeds: vec!["b".to_string()],
                ..LinkGraphLinkFilter::default()
            }),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let linked_by_hits = index.search_planned("", 10, linked_by_options).1;
    assert_eq!(linked_by_hits.len(), 1);
    assert_eq!(linked_by_hits[0].path, "docs/d.md");
    Ok(())
}
