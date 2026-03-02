use super::*;

#[test]
fn test_link_graph_parse_search_query_supports_path_fuzzy_strategy() {
    let parsed = parse_search_query(
        "match:path_fuzzy architecture graph",
        LinkGraphSearchOptions::default(),
    );
    assert_eq!(parsed.query, "architecture graph");
    assert_eq!(
        parsed.options.match_strategy,
        LinkGraphMatchStrategy::PathFuzzy
    );
}

#[test]
fn test_link_graph_search_path_fuzzy_prefers_path_and_section()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/architecture/graph.md"),
        "# Architecture\n\n## Graph Engine\n\nImplementation details.\n",
    )?;
    write_file(
        &tmp.path().join("docs/notes/misc.md"),
        "# Misc\n\nSome graph mention without architecture path.\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options = LinkGraphSearchOptions {
        match_strategy: LinkGraphMatchStrategy::PathFuzzy,
        case_sensitive: false,
        ..LinkGraphSearchOptions::default()
    };
    let hits = index
        .search_planned("architecture graph engine", 5, options)
        .1;
    assert!(!hits.is_empty());
    assert_eq!(hits[0].path, "docs/architecture/graph.md");
    assert_eq!(
        hits[0].best_section,
        Some("Architecture / Graph Engine".to_string())
    );
    assert!(
        hits[0]
            .match_reason
            .as_deref()
            .unwrap_or_default()
            .contains("path_fuzzy")
    );
    Ok(())
}

#[test]
fn test_link_graph_search_path_fuzzy_ignores_fenced_headings()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/architecture/engine.md"),
        "# Architecture\n\n```md\n## Fake Heading\n```\n\n## Real Heading\n\nGraph runtime pipeline.\n",
    )?;
    write_file(
        &tmp.path().join("docs/notes/misc.md"),
        "# Misc\n\nGraph runtime note.\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options = LinkGraphSearchOptions {
        match_strategy: LinkGraphMatchStrategy::PathFuzzy,
        case_sensitive: false,
        ..LinkGraphSearchOptions::default()
    };
    let hits = index
        .search_planned("architecture real heading graph", 5, options)
        .1;
    assert!(!hits.is_empty());
    assert_eq!(hits[0].path, "docs/architecture/engine.md");
    assert_eq!(
        hits[0].best_section,
        Some("Architecture / Real Heading".to_string())
    );
    Ok(())
}

#[test]
fn test_link_graph_search_path_fuzzy_handles_duplicate_headings()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/architecture/api.md"),
        "# Architecture\n\n## API\n\nOverview.\n\n## API\n\nRouter graph query constraints.\n",
    )?;
    write_file(
        &tmp.path().join("docs/notes/other.md"),
        "# Other\n\nRouter query text.\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options = LinkGraphSearchOptions {
        match_strategy: LinkGraphMatchStrategy::PathFuzzy,
        case_sensitive: false,
        ..LinkGraphSearchOptions::default()
    };
    let hits = index
        .search_planned("architecture api router", 5, options)
        .1;
    assert!(!hits.is_empty());
    assert_eq!(hits[0].path, "docs/architecture/api.md");
    assert_eq!(hits[0].best_section, Some("Architecture / API".to_string()));
    assert!(
        hits[0]
            .match_reason
            .as_deref()
            .unwrap_or_default()
            .contains("path_fuzzy")
    );
    Ok(())
}

#[test]
fn test_link_graph_search_with_exact_strategy() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# Alpha\n\n[[b]]\n")?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "---\ntitle: Rust Tokenizer\ntags:\n  - rust\n---\n",
    )?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let options = LinkGraphSearchOptions {
        match_strategy: LinkGraphMatchStrategy::Exact,
        case_sensitive: false,
        ..LinkGraphSearchOptions::default()
    };
    let hits = index.search_planned("rust tokenizer", 5, options).1;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].stem, "b");
    Ok(())
}

#[test]
fn test_link_graph_search_with_regex_strategy() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/alpha-note.md"),
        "# Alpha Note\n\n[[beta-note]]\n",
    )?;
    write_file(
        &tmp.path().join("docs/beta-note.md"),
        "# Beta Note\n\n[[alpha-note]]\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let options = LinkGraphSearchOptions {
        match_strategy: LinkGraphMatchStrategy::Re,
        case_sensitive: false,
        ..LinkGraphSearchOptions::default()
    };
    let hits = index.search_planned("^beta", 5, options).1;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].stem, "beta-note");
    Ok(())
}
