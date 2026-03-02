//! Fixture-backed tests for markdown syntax parsing and graph algorithm behavior.
//!
//! This suite uses synthetic markdown-only fixtures (no business content) to verify:
//! - heading/link parsing behavior;
//! - frontmatter/tag search behavior;
//! - graph neighbor/related traversal behavior.

use std::path::PathBuf;
use xiuxian_wendao::link_graph::{
    LinkGraphDirection, LinkGraphIndex, LinkGraphMatchStrategy, LinkGraphSearchOptions,
};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("markdown-syntax-algorithm")
}

#[test]
fn test_fixture_corpus_builds_and_has_expected_graph_shape()
-> Result<(), Box<dyn std::error::Error>> {
    let index = LinkGraphIndex::build(&fixture_root()).map_err(|e| e.clone())?;
    let stats = index.stats();
    assert_eq!(stats.total_notes, 8);
    assert!(stats.links_in_graph >= 9);
    assert_eq!(stats.nodes_in_graph, 8);
    Ok(())
}

#[test]
fn test_fixture_search_hits_frontmatter_and_heading_markers()
-> Result<(), Box<dyn std::error::Error>> {
    let index = LinkGraphIndex::build(&fixture_root()).map_err(|e| e.clone())?;

    let frontmatter_hits = index
        .search_planned(
            "frontmatter-tag-marker",
            5,
            LinkGraphSearchOptions::default(),
        )
        .1;
    assert!(!frontmatter_hits.is_empty());
    assert_eq!(frontmatter_hits[0].path, "frontmatter-tags.md");

    let fuzzy_options = LinkGraphSearchOptions {
        match_strategy: LinkGraphMatchStrategy::PathFuzzy,
        ..LinkGraphSearchOptions::default()
    };
    let heading_hits = index
        .search_planned("sub heading marker", 5, fuzzy_options)
        .1;
    assert!(!heading_hits.is_empty());
    assert!(
        heading_hits
            .iter()
            .any(|row| row.path == "syntax-headings.md")
    );

    Ok(())
}

#[test]
fn test_fixture_code_fence_links_do_not_create_edges() -> Result<(), Box<dyn std::error::Error>> {
    let index = LinkGraphIndex::build(&fixture_root()).map_err(|e| e.clone())?;

    let neighbors = index.neighbors("code-fence-only", LinkGraphDirection::Both, 1, 10);
    assert!(
        neighbors.is_empty(),
        "code-fence-only fixture should not generate graph edges from fenced pseudo-links"
    );

    Ok(())
}

#[test]
fn test_fixture_attachments_and_embeds_are_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let index = LinkGraphIndex::build(&fixture_root()).map_err(|e| e.clone())?;

    let neighbors = index.neighbors("syntax-attachments-embeds", LinkGraphDirection::Both, 1, 10);
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].stem, "graph-c");

    Ok(())
}

#[test]
fn test_fixture_related_and_neighbors_cover_graph_chain() -> Result<(), Box<dyn std::error::Error>>
{
    let index = LinkGraphIndex::build(&fixture_root()).map_err(|e| e.clone())?;

    let neighbors = index.neighbors("graph-a", LinkGraphDirection::Both, 1, 10);
    assert!(neighbors.iter().any(|row| row.stem == "graph-b"));
    assert!(neighbors.iter().any(|row| row.stem == "graph-c"));

    let related = index.related("graph-a", 2, 10);
    assert!(related.iter().any(|row| row.stem == "graph-b"));
    assert!(related.iter().any(|row| row.stem == "graph-c"));

    Ok(())
}
