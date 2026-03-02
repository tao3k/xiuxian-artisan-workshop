use super::*;

#[test]
fn test_link_graph_parse_search_query_supports_multi_sort_terms_in_directive() {
    let parsed = parse_search_query(
        "sort:path_asc,modified_desc,score_desc hello",
        LinkGraphSearchOptions::default(),
    );

    assert_eq!(parsed.query, "hello");
    assert_eq!(
        parsed.options.sort_terms,
        vec![
            sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc),
            sort_term(LinkGraphSortField::Modified, LinkGraphSortOrder::Desc),
            sort_term(LinkGraphSortField::Score, LinkGraphSortOrder::Desc),
        ]
    );
}
