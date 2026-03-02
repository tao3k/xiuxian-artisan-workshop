use super::*;

#[test]
fn test_link_graph_search_options_deserialize_accepts_tree_filters()
-> Result<(), Box<dyn std::error::Error>> {
    let payload = json!({
        "match_strategy": "fts",
        "case_sensitive": false,
        "sort_terms": [{"field": "score", "order": "desc"}],
        "filters": {
            "scope": "mixed",
            "max_heading_level": 4,
            "max_tree_hops": 3,
            "collapse_to_doc": true,
            "edge_types": ["semantic", "verified"],
            "per_doc_section_cap": 5,
            "min_section_words": 12
        }
    });
    let parsed: LinkGraphSearchOptions = serde_json::from_value(payload)?;
    assert_eq!(parsed.filters.scope, Some(LinkGraphScope::Mixed));
    assert_eq!(
        parsed.filters.edge_types,
        vec![LinkGraphEdgeType::Semantic, LinkGraphEdgeType::Verified]
    );
    assert_eq!(parsed.filters.max_heading_level, Some(4));
    assert_eq!(parsed.filters.max_tree_hops, Some(3));
    assert_eq!(parsed.filters.collapse_to_doc, Some(true));
    assert_eq!(parsed.filters.per_doc_section_cap, Some(5));
    assert_eq!(parsed.filters.min_section_words, Some(12));
    Ok(())
}
