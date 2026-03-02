use super::*;

#[test]
fn test_link_graph_parse_search_query_supports_directives_and_time_filters() {
    let parsed = parse_search_query(
        "match:re sort:modified_desc case:true link-to:a,b linked-by:c related:seed~3 related_ppr_alpha:0.9 related_ppr_max_iter:64 related_ppr_tol:1e-6 related_ppr_subgraph_mode:force created>=2024-01-01 modified<=2024-01-31 hello",
        LinkGraphSearchOptions::default(),
    );

    assert_eq!(parsed.query, "hello");
    assert_eq!(parsed.options.match_strategy, LinkGraphMatchStrategy::Re);
    assert_eq!(
        parsed.options.sort_terms,
        vec![sort_term(
            LinkGraphSortField::Modified,
            LinkGraphSortOrder::Desc
        )]
    );
    assert!(parsed.options.case_sensitive);
    assert_eq!(
        parsed
            .options
            .filters
            .link_to
            .as_ref()
            .map(|row| row.seeds.clone()),
        Some(vec!["a".to_string(), "b".to_string()])
    );
    assert_eq!(
        parsed
            .options
            .filters
            .linked_by
            .as_ref()
            .map(|row| row.seeds.clone()),
        Some(vec!["c".to_string()])
    );
    assert_eq!(
        parsed
            .options
            .filters
            .related
            .as_ref()
            .map(|row| row.seeds.clone()),
        Some(vec!["seed".to_string()])
    );
    assert_eq!(
        parsed
            .options
            .filters
            .related
            .as_ref()
            .and_then(|row| row.max_distance),
        Some(3)
    );
    assert_eq!(
        parsed
            .options
            .filters
            .related
            .as_ref()
            .and_then(|row| row.ppr.as_ref())
            .and_then(|ppr| ppr.alpha),
        Some(0.9)
    );
    assert_eq!(
        parsed
            .options
            .filters
            .related
            .as_ref()
            .and_then(|row| row.ppr.as_ref())
            .and_then(|ppr| ppr.max_iter),
        Some(64)
    );
    assert_eq!(
        parsed
            .options
            .filters
            .related
            .as_ref()
            .and_then(|row| row.ppr.as_ref())
            .and_then(|ppr| ppr.tol),
        Some(1e-6)
    );
    assert_eq!(
        parsed
            .options
            .filters
            .related
            .as_ref()
            .and_then(|row| row.ppr.as_ref())
            .and_then(|ppr| ppr.subgraph_mode),
        Some(LinkGraphPprSubgraphMode::Force)
    );
    assert_eq!(parsed.options.created_after, Some(1_704_067_200));
    assert_eq!(parsed.options.modified_before, Some(1_706_659_200));
}
