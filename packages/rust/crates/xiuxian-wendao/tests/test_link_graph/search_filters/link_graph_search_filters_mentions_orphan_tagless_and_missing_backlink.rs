use super::*;

#[test]
fn test_link_graph_search_filters_mentions_orphan_tagless_and_missing_backlink()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "---\ntags:\n  - core\n---\n# A\n\nAlpha signal appears here.\n\n[[b]]\n",
    )?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "---\ntags:\n  - team\n---\n# B\n\nBeta note.\n",
    )?;
    write_file(
        &tmp.path().join("docs/c.md"),
        "# C\n\nAlpha signal appears here too.\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let mentions_options = LinkGraphSearchOptions {
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        filters: LinkGraphSearchFilters {
            mentions_of: vec!["alpha signal".to_string()],
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let mention_hits = index.search_planned("", 10, mentions_options).1;
    let mention_paths: Vec<String> = mention_hits.into_iter().map(|row| row.path).collect();
    assert_eq!(
        mention_paths,
        vec!["docs/a.md".to_string(), "docs/c.md".to_string()]
    );

    let mentioned_by_options = LinkGraphSearchOptions {
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        filters: LinkGraphSearchFilters {
            mentioned_by_notes: vec!["a".to_string()],
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let mentioned_by_hits = index.search_planned("", 10, mentioned_by_options).1;
    assert_eq!(mentioned_by_hits.len(), 1);
    assert_eq!(mentioned_by_hits[0].path, "docs/b.md");

    let orphan_options = LinkGraphSearchOptions {
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        filters: LinkGraphSearchFilters {
            orphan: true,
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let orphan_hits = index.search_planned("", 10, orphan_options).1;
    assert_eq!(orphan_hits.len(), 1);
    assert_eq!(orphan_hits[0].path, "docs/c.md");

    let tagless_options = LinkGraphSearchOptions {
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        filters: LinkGraphSearchFilters {
            tagless: true,
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let tagless_hits = index.search_planned("", 10, tagless_options).1;
    let tagless_paths: Vec<String> = tagless_hits.into_iter().map(|row| row.path).collect();
    assert_eq!(tagless_paths, vec!["docs/c.md".to_string()]);

    let missing_backlink_options = LinkGraphSearchOptions {
        sort_terms: vec![sort_term(LinkGraphSortField::Path, LinkGraphSortOrder::Asc)],
        filters: LinkGraphSearchFilters {
            missing_backlink: true,
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };
    let missing_backlink_hits = index.search_planned("", 10, missing_backlink_options).1;
    assert_eq!(missing_backlink_hits.len(), 1);
    assert_eq!(missing_backlink_hits[0].path, "docs/a.md");

    Ok(())
}
