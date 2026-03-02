//! Shared YAML Frontmatter Parsing
//!
//! Provides common utilities for parsing YAML frontmatter from markdown files.
//! Used by both `skills` (SKILL.md) and `knowledge` (*.md) modules.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::fs;

/// Borrowed frontmatter and markdown body slices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontmatterParts<'a> {
    /// YAML payload between opening and closing `---` markers.
    pub yaml: &'a str,
    /// Markdown body after the closing marker line.
    pub body: &'a str,
}

/// Split markdown content into frontmatter YAML and remaining body.
///
/// Frontmatter is considered valid only when the first line is exactly `---`.
#[must_use]
pub fn split_frontmatter(content: &str) -> Option<FrontmatterParts<'_>> {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let mut lines = content.split_inclusive('\n');
    let first = lines.next()?;
    let first_rendered = first.trim_end_matches(['\r', '\n']).trim();
    if first_rendered != "---" {
        return None;
    }

    let mut offset = first.len();
    let yaml_start = offset;
    for line in lines {
        let rendered = line.trim_end_matches(['\r', '\n']).trim();
        if rendered == "---" || rendered == "..." {
            let yaml = &content[yaml_start..offset];
            let body = &content[offset + line.len()..];
            return Some(FrontmatterParts { yaml, body });
        }
        offset += line.len();
    }

    None
}

/// Extract YAML frontmatter from markdown content.
///
/// Returns `Some(String)` if frontmatter is found, `None` otherwise.
///
/// # Examples
///
/// ```ignore
/// let content = r#"---
/// name: "test"
/// version: "1.0"
/// ---
/// # Content
/// "#;
///
/// let frontmatter = extract_frontmatter(content).unwrap();
/// assert!(frontmatter.contains("name:"));
/// ```
#[must_use]
pub fn extract_frontmatter(content: &str) -> Option<String> {
    split_frontmatter(content).map(|parts| parts.yaml.to_string())
}

/// Parse frontmatter from markdown content into typed data.
///
/// # Errors
///
/// Returns an error if YAML exists but parsing fails.
pub fn parse_typed_frontmatter_from_markdown<T: DeserializeOwned>(
    content: &str,
) -> Result<Option<T>, serde_yaml::Error> {
    let Some(parts) = split_frontmatter(content) else {
        return Ok(None);
    };
    serde_yaml::from_str(parts.yaml).map(Some)
}

/// Parse markdown frontmatter and validate it against a typed schema.
///
/// Unlike `parse_typed_frontmatter_from_markdown`, this function fails when
/// frontmatter markers are missing.
///
/// # Errors
///
/// Returns `Err(String)` when markers are missing or schema parsing fails.
pub fn parse_and_validate_asset<T: DeserializeOwned>(content: &str) -> Result<T, String> {
    strict_parse(content)
}

/// Strictly parse markdown frontmatter into a typed schema.
///
/// This is the canonical strict parser for validation gates.
///
/// # Errors
///
/// Returns `Err(String)` when markers are missing or schema parsing fails.
pub fn strict_parse<T: DeserializeOwned>(content: &str) -> Result<T, String> {
    let Some(parts) = split_frontmatter(content) else {
        return Err("Missing frontmatter markers (`---`)".to_string());
    };
    serde_yaml::from_str::<T>(parts.yaml)
        .map_err(|error| format!("Frontmatter schema violation: {error}"))
}

/// Parse frontmatter from markdown content into a generic YAML value.
///
/// # Errors
///
/// Returns an error if YAML exists but parsing fails.
pub fn parse_frontmatter_from_markdown(
    content: &str,
) -> Result<Option<serde_yaml::Value>, serde_yaml::Error> {
    parse_typed_frontmatter_from_markdown(content)
}

/// Parse YAML frontmatter content into a serde value.
///
/// # Errors
///
/// Returns an error if the YAML is invalid.
pub fn parse_frontmatter(yaml_content: &str) -> Result<serde_yaml::Value, serde_yaml::Error> {
    serde_yaml::from_str(yaml_content)
}

/// Generic frontmatter structure that can be extended for different use cases.
#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct GenericFrontmatter {
    /// Document title.
    #[serde(default)]
    pub title: Option<String>,
    /// Human-readable description of the document.
    #[serde(default)]
    pub description: Option<String>,
    /// Category for organizing documents (e.g., "pattern", "technique").
    #[serde(default)]
    pub category: Option<String>,
    /// Tags for discovery and routing.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// Additional metadata as key-value pairs.
    #[serde(default)]
    pub metadata: Option<serde_yaml::Value>,
}

/// Read and parse frontmatter from a markdown file.
///
/// Returns the parsed frontmatter as a `GenericFrontmatter`, or None if the file doesn't exist
/// or has no valid frontmatter.
#[must_use]
pub fn read_frontmatter_from_file(path: &std::path::Path) -> Option<GenericFrontmatter> {
    let content = fs::read_to_string(path).ok()?;
    parse_frontmatter_from_content(&content)
}

/// Read file and extract frontmatter content as string.
#[must_use]
pub fn extract_frontmatter_from_file(path: &std::path::Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    extract_frontmatter(&content)
}

/// Parse frontmatter from markdown content string.
#[must_use]
pub fn parse_frontmatter_from_content(content: &str) -> Option<GenericFrontmatter> {
    parse_typed_frontmatter_from_markdown(content)
        .ok()
        .flatten()
}
