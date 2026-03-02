use super::*;

#[test]
fn test_link_graph_parse_search_query_infers_regex_from_regex_markers() {
    let parsed = parse_search_query("^wendao.*plan$", LinkGraphSearchOptions::default());

    assert_eq!(parsed.query, "^wendao.*plan$");
    assert_eq!(parsed.options.match_strategy, LinkGraphMatchStrategy::Re);
}
