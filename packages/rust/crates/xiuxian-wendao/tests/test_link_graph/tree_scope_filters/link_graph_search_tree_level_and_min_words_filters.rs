use super::*;

#[test]
fn test_link_graph_search_tree_level_and_min_words_filters()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# Root\n\n## Allowed\n\nneedle appears with enough words for filtering.\n\n#### Too Deep\n\nneedle appears here but must be filtered by heading depth.\n\n## Too Short\n\nneedle\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            scope: Some(LinkGraphScope::SectionOnly),
            max_heading_level: Some(2),
            min_section_words: Some(4),
            per_doc_section_cap: Some(10),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let hits = index.search_planned("needle", 20, options).1;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].best_section.as_deref(), Some("Root / Allowed"));
    Ok(())
}
