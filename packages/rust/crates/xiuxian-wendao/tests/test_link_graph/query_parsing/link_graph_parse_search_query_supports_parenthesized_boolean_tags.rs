use super::*;

#[test]
fn test_link_graph_parse_search_query_supports_parenthesized_boolean_tags() {
    let parsed = parse_search_query(
        "tag:(core OR infra) roadmap",
        LinkGraphSearchOptions::default(),
    );

    assert_eq!(parsed.query, "roadmap");
    let Some(tags) = parsed.options.filters.tags else {
        panic!("expected tags filter");
    };
    assert!(tags.all.is_empty());
    assert_eq!(tags.any, vec!["core".to_string(), "infra".to_string()]);
    assert!(tags.not_tags.is_empty());
}
