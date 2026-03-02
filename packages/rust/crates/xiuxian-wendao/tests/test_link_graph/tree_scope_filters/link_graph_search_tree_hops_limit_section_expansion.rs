use super::*;

#[test]
fn test_link_graph_search_tree_hops_limit_section_expansion()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# Root\n\n## Parent\n\nneedle parent context words here.\n\n### Needle Focus\n\nneedle focus context words here.\n\n### Sibling\n\nneedle sibling context words here.\n\n## Other\n\nneedle other branch words here.\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let base = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            scope: Some(LinkGraphScope::SectionOnly),
            per_doc_section_cap: Some(10),
            min_section_words: Some(0),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let hops_zero = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            max_tree_hops: Some(0),
            ..base.filters.clone()
        },
        ..base.clone()
    };
    let hits_zero = index.search_planned("needle focus", 20, hops_zero).1;
    assert_eq!(hits_zero.len(), 1);
    assert_eq!(
        hits_zero[0].best_section.as_deref(),
        Some("Root / Parent / Needle Focus")
    );

    let hops_one = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            max_tree_hops: Some(1),
            ..base.filters.clone()
        },
        ..base
    };
    let hits_one = index.search_planned("needle focus", 20, hops_one).1;
    let sections_one: Vec<String> = hits_one
        .iter()
        .filter_map(|row| row.best_section.clone())
        .collect();
    assert!(sections_one.contains(&"Root / Parent / Needle Focus".to_string()));
    assert!(sections_one.contains(&"Root / Parent".to_string()));
    assert!(!sections_one.contains(&"Root / Parent / Sibling".to_string()));
    assert!(!sections_one.contains(&"Root / Other".to_string()));
    Ok(())
}
