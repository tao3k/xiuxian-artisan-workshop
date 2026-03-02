use super::*;

#[test]
fn test_link_graph_parse_search_query_supports_related_ppr_key_variants() {
    let parsed = parse_search_query(
        "related:seed related.ppr.alpha:0.75 related-ppr-max-iter:32 ppr_tol:1e-5 ppr-subgraph-mode:auto",
        LinkGraphSearchOptions::default(),
    );

    let Some(related) = parsed.options.filters.related.as_ref() else {
        panic!("expected related filter");
    };
    assert_eq!(related.seeds, vec!["seed".to_string()]);
    let Some(ppr) = related.ppr.as_ref() else {
        panic!("expected related ppr options");
    };
    assert_eq!(ppr.alpha, Some(0.75));
    assert_eq!(ppr.max_iter, Some(32));
    assert_eq!(ppr.tol, Some(1e-5));
    assert_eq!(ppr.subgraph_mode, Some(LinkGraphPprSubgraphMode::Auto));
}
