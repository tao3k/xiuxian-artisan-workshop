use super::*;

#[test]
fn test_link_graph_parse_search_query_keeps_fts_for_extension_only_query() {
    let parsed = parse_search_query(".md", LinkGraphSearchOptions::default());
    assert_eq!(parsed.query, ".md");
    assert_eq!(parsed.options.match_strategy, LinkGraphMatchStrategy::Fts);
}
