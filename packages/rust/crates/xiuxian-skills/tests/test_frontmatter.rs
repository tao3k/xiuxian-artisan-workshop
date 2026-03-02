//! Integration tests for frontmatter parsing helpers.

use serde::Deserialize;
use std::fmt::Display;
use xiuxian_skills::{
    parse_and_validate_asset, parse_frontmatter_from_markdown,
    parse_typed_frontmatter_from_markdown, split_frontmatter,
};

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct DemoFrontmatter {
    name: String,
}

fn must_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(inner) => inner,
        None => panic!("{context}"),
    }
}

fn must_ok<T, E: Display>(value: Result<T, E>, context: &str) -> T {
    match value {
        Ok(inner) => inner,
        Err(error) => panic!("{context}: {error}"),
    }
}

#[test]
fn split_frontmatter_returns_yaml_and_body() {
    let content = "---\nname: demo\n---\n# Body\nvalue\n";
    let parts = must_some(split_frontmatter(content), "frontmatter should be parsed");
    assert_eq!(parts.yaml, "name: demo\n");
    assert_eq!(parts.body, "# Body\nvalue\n");
}

#[test]
fn split_frontmatter_requires_leading_marker() {
    let content = "prefix\n---\nname: demo\n---\n# Body\n";
    assert!(split_frontmatter(content).is_none());
}

#[test]
fn split_frontmatter_supports_three_dot_closing_marker() {
    let content = "---\nname: demo\n...\n# Body\n";
    let parts = must_some(split_frontmatter(content), "frontmatter should be parsed");
    assert_eq!(parts.yaml, "name: demo\n");
    assert_eq!(parts.body, "# Body\n");
}

#[test]
fn split_frontmatter_supports_marker_whitespace() {
    let content = "---   \nname: demo\n---   \n# Body\n";
    let parts = must_some(split_frontmatter(content), "frontmatter should be parsed");
    assert_eq!(parts.yaml, "name: demo\n");
    assert_eq!(parts.body, "# Body\n");
}

#[test]
fn parse_typed_frontmatter_from_markdown_parses_typed_struct() {
    let content = "---\nname: demo\n---\n# Body\n";
    let typed = must_ok(
        parse_typed_frontmatter_from_markdown::<DemoFrontmatter>(content),
        "yaml should parse",
    );
    let parsed = must_some(typed, "frontmatter should exist");
    assert_eq!(
        parsed,
        DemoFrontmatter {
            name: "demo".into()
        }
    );
}

#[test]
fn parse_frontmatter_from_markdown_returns_none_without_frontmatter() {
    let content = "# No frontmatter";
    let parsed = must_ok(
        parse_frontmatter_from_markdown(content),
        "no yaml parse error expected",
    );
    assert!(parsed.is_none());
}

#[test]
fn parse_and_validate_asset_requires_markers() {
    let content = "# No frontmatter";
    let error = parse_and_validate_asset::<DemoFrontmatter>(content)
        .expect_err("expected missing frontmatter markers error");
    assert!(error.contains("Missing frontmatter markers"));
}

#[test]
fn parse_and_validate_asset_enforces_schema() {
    let content = "---\nother: demo\n---\n# Body\n";
    let error = parse_and_validate_asset::<DemoFrontmatter>(content)
        .expect_err("expected schema violation for missing `name`");
    assert!(error.contains("Frontmatter schema violation"));
}
