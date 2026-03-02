use super::*;

#[test]
fn test_link_graph_parse_search_query_supports_negated_directives_and_pipe_values() {
    let parsed = parse_search_query(
        "-tag:legacy -to:archive to:hub|index from:a|b",
        LinkGraphSearchOptions::default(),
    );

    assert_eq!(parsed.query, "");
    let Some(tags) = parsed.options.filters.tags else {
        panic!("expected tags filter");
    };
    assert_eq!(tags.not_tags, vec!["legacy".to_string()]);

    let Some(link_to) = parsed.options.filters.link_to else {
        panic!("expected link_to filter");
    };
    assert!(link_to.negate);
    assert_eq!(
        link_to.seeds,
        vec![
            "archive".to_string(),
            "hub".to_string(),
            "index".to_string()
        ]
    );

    let Some(linked_by) = parsed.options.filters.linked_by else {
        panic!("expected linked_by filter");
    };
    assert_eq!(linked_by.seeds, vec!["a".to_string(), "b".to_string()]);
}
