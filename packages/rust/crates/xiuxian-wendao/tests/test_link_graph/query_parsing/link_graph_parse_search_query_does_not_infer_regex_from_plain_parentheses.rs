use super::*;

#[test]
fn test_link_graph_parse_search_query_does_not_infer_regex_from_plain_parentheses() {
    let parsed = parse_search_query(
        "Wendao Plan Consolidation (2026)",
        LinkGraphSearchOptions::default(),
    );

    assert_eq!(parsed.query, "Wendao Plan Consolidation (2026)");
    assert_eq!(parsed.options.match_strategy, LinkGraphMatchStrategy::Fts);
}
