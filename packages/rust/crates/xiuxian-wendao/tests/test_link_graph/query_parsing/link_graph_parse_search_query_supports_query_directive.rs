use super::*;

#[test]
fn test_link_graph_parse_search_query_supports_query_directive() {
    let parsed = parse_search_query(
        "query:search_optimized_ipc scope:section_only max_heading_level:2",
        LinkGraphSearchOptions::default(),
    );

    assert_eq!(parsed.query, "search_optimized_ipc");
    assert_eq!(
        parsed.options.filters.scope,
        Some(LinkGraphScope::SectionOnly)
    );
    assert_eq!(parsed.options.filters.max_heading_level, Some(2));
}
