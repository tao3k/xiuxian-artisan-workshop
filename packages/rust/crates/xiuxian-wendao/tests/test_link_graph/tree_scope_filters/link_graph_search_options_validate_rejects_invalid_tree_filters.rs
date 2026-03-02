use super::*;

#[test]
fn test_link_graph_search_options_validate_rejects_invalid_tree_filters()
-> Result<(), Box<dyn std::error::Error>> {
    let payload = json!({
        "match_strategy": "fts",
        "case_sensitive": false,
        "sort_terms": [{"field": "score", "order": "desc"}],
        "filters": {
            "max_heading_level": 9,
            "per_doc_section_cap": 0
        }
    });
    let parsed: LinkGraphSearchOptions = serde_json::from_value(payload)?;
    let Err(error) = parsed.validate() else {
        panic!("validation should reject invalid tree filters");
    };
    assert!(error.contains("max_heading_level") || error.contains("per_doc_section_cap"));
    Ok(())
}
