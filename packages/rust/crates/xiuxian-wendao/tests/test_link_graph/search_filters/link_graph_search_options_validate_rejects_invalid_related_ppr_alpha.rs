use super::*;

#[test]
fn test_link_graph_search_options_validate_rejects_invalid_related_ppr_alpha() {
    let options = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            related: Some(LinkGraphRelatedFilter {
                seeds: vec!["b".to_string()],
                max_distance: Some(2),
                ppr: Some(LinkGraphRelatedPprOptions {
                    alpha: Some(1.2),
                    max_iter: Some(32),
                    tol: Some(1e-6),
                    subgraph_mode: Some(LinkGraphPprSubgraphMode::Auto),
                }),
            }),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let Err(err) = options.validate() else {
        panic!("alpha > 1 must fail");
    };
    assert!(err.contains("filters.related.ppr.alpha"));
}
