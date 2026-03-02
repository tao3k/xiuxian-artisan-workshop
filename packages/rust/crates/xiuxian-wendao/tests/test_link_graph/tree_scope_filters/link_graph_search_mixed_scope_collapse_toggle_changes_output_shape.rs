use super::*;

#[test]
fn test_link_graph_search_mixed_scope_collapse_toggle_changes_output_shape()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# A\n\n## One\n\nalpha context words one.\n\n## Two\n\nalpha context words two.\n",
    )?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "# B\n\n## B One\n\nalpha context words.\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let collapse_true = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            scope: Some(LinkGraphScope::Mixed),
            collapse_to_doc: Some(true),
            per_doc_section_cap: Some(3),
            min_section_words: Some(0),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let hits_collapsed = index.search_planned("alpha context", 20, collapse_true).1;
    let mut collapsed_counts: HashMap<String, usize> = HashMap::new();
    for row in hits_collapsed {
        *collapsed_counts.entry(row.path).or_insert(0) += 1;
    }
    assert!(collapsed_counts.values().all(|count| *count == 1));

    let collapse_false = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            scope: Some(LinkGraphScope::Mixed),
            collapse_to_doc: Some(false),
            per_doc_section_cap: Some(3),
            min_section_words: Some(0),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let hits_expanded = index.search_planned("alpha context", 20, collapse_false).1;
    let mut expanded_counts: HashMap<String, usize> = HashMap::new();
    for row in hits_expanded {
        *expanded_counts.entry(row.path).or_insert(0) += 1;
    }
    assert!(expanded_counts.values().any(|count| *count > 1));
    Ok(())
}
