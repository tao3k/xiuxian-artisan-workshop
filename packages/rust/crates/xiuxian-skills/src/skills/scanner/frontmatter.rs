use serde::Deserialize;

/// YAML frontmatter structure (Anthropic official format with metadata block).
///
/// ```yaml
/// ---
/// name: <skill-identifier>
/// description: Use when <use-case-1>, <use-case-2>, or <use-case-3>.
/// metadata:
///   author: <name>
///   version: "x.x.x"
///   source: <url>
///   routing_keywords:
///     - "keyword1"
///     - "keyword2"
///   intents:
///     - "Intent description 1"
///     - "Intent description 2"
/// ---
/// ```
#[derive(Debug, Deserialize, PartialEq, Default)]
pub(super) struct SkillFrontmatter {
    #[serde(default)]
    pub(super) name: Option<String>,
    #[serde(default)]
    pub(super) description: Option<String>,
    #[serde(default)]
    pub(super) metadata: Option<InnerArtisanMetadata>,
}

/// Nested artisan metadata block for YAML parsing (internal use only).
#[derive(Debug, Deserialize, PartialEq, Default)]
pub(super) struct InnerArtisanMetadata {
    #[serde(default)]
    pub(super) author: Option<String>,
    #[serde(default)]
    pub(super) authors: Option<Vec<String>>,
    #[serde(default)]
    pub(super) version: Option<String>,
    #[serde(default)]
    pub(super) source: Option<String>,
    #[serde(default)]
    pub(super) routing_keywords: Option<Vec<String>>,
    #[serde(default)]
    pub(super) intents: Option<Vec<String>>,
    #[serde(default)]
    pub(super) require_refs: Option<Vec<String>>,
    #[serde(default)]
    pub(super) repository: Option<String>,
    /// Permissions required by this skill (e.g., "filesystem:read", "network:http")
    #[serde(default)]
    pub(super) permissions: Option<Vec<String>>,
}

/// Backward-compatible alias for older internal naming.
pub(super) type SkillMetadataBlock = InnerArtisanMetadata;
