use super::*;

#[test]
fn test_link_graph_search_section_scope_respects_per_doc_cap()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# A\n\n## Alpha One\n\nalpha marker content line one.\n\n## Alpha Two\n\nalpha marker content line two.\n",
    )?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "# B\n\n## Beta One\n\nalpha marker content line one.\n\n## Beta Two\n\nalpha marker content line two.\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            scope: Some(LinkGraphScope::SectionOnly),
            per_doc_section_cap: Some(1),
            min_section_words: Some(0),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let hits = index.search_planned("alpha marker", 20, options).1;
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|row| row.best_section.is_some()));

    let mut per_path: HashMap<String, usize> = HashMap::new();
    for row in hits {
        *per_path.entry(row.path).or_insert(0) += 1;
    }
    assert!(per_path.values().all(|count| *count == 1));
    Ok(())
}
