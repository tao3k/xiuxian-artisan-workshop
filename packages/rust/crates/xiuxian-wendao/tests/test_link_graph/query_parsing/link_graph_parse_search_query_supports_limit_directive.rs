use super::*;

#[test]
fn test_link_graph_parse_search_query_supports_limit_directive() {
    let parsed = parse_search_query(
        "query:hard constraints limit:8 scope:section_only",
        LinkGraphSearchOptions::default(),
    );

    assert_eq!(parsed.query, "hard constraints");
    assert_eq!(parsed.limit_override, Some(8));
    assert_eq!(
        parsed.options.filters.scope,
        Some(LinkGraphScope::SectionOnly)
    );
}
