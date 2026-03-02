use super::*;

#[test]
fn test_link_graph_parse_search_query_supports_tree_filter_directives() {
    let parsed = parse_search_query(
        "scope:section_only edge_types:structural,verified max_heading_level:3 max_tree_hops:2 collapse_to_doc:false per_doc_section_cap:4 min_section_words:18 architecture",
        LinkGraphSearchOptions::default(),
    );

    assert_eq!(parsed.query, "architecture");
    assert_eq!(
        parsed.options.filters.scope,
        Some(LinkGraphScope::SectionOnly)
    );
    assert_eq!(
        parsed.options.filters.edge_types,
        vec![LinkGraphEdgeType::Structural, LinkGraphEdgeType::Verified]
    );
    assert_eq!(parsed.options.filters.max_heading_level, Some(3));
    assert_eq!(parsed.options.filters.max_tree_hops, Some(2));
    assert_eq!(parsed.options.filters.collapse_to_doc, Some(false));
    assert_eq!(parsed.options.filters.per_doc_section_cap, Some(4));
    assert_eq!(parsed.options.filters.min_section_words, Some(18));
}
