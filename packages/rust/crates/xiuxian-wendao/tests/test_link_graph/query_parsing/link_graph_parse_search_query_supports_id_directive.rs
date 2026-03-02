use super::*;

#[test]
fn test_link_graph_parse_search_query_supports_id_directive() {
    let parsed = parse_search_query(
        "id:\"docs/agenda\" query:\"fallback phrase\"",
        LinkGraphSearchOptions::default(),
    );

    assert_eq!(parsed.direct_id.as_deref(), Some("docs/agenda"));
    assert_eq!(parsed.query, "fallback phrase");
}
