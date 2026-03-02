use super::enums::{LinkGraphEdgeType, LinkGraphPprSubgraphMode, LinkGraphScope};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Boolean tag filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphTagFilter {
    /// Tags that must all be present.
    #[serde(default)]
    pub all: Vec<String>,
    /// At least one of these tags must be present.
    #[serde(default)]
    pub any: Vec<String>,
    /// Tags that must not be present.
    #[serde(default, rename = "not")]
    pub not_tags: Vec<String>,
}

/// Link filter for `link_to`/`linked_by`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphLinkFilter {
    /// Seed note ids/stems/paths.
    #[serde(default)]
    pub seeds: Vec<String>,
    /// Negate match semantics.
    #[serde(default)]
    pub negate: bool,
    /// Traverse recursively from seeds.
    #[serde(default)]
    pub recursive: bool,
    /// Optional traversal distance cap.
    #[serde(default)]
    pub max_distance: Option<usize>,
}

/// PPR tuning options for related retrieval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphRelatedPprOptions {
    /// Teleport probability parameter in `[0, 1]`.
    #[serde(default)]
    pub alpha: Option<f64>,
    /// Maximum power-iteration count.
    #[serde(default)]
    pub max_iter: Option<usize>,
    /// Convergence tolerance.
    #[serde(default)]
    pub tol: Option<f64>,
    /// Subgraph partitioning mode.
    #[serde(default)]
    pub subgraph_mode: Option<LinkGraphPprSubgraphMode>,
}

/// Related filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphRelatedFilter {
    /// Seed note ids/stems/paths.
    #[serde(default)]
    pub seeds: Vec<String>,
    /// Optional traversal distance cap.
    #[serde(default)]
    pub max_distance: Option<usize>,
    /// Optional PPR tuning block.
    #[serde(default)]
    pub ppr: Option<LinkGraphRelatedPprOptions>,
}

/// Structured search filters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphSearchFilters {
    /// Path prefixes/files to include.
    #[serde(default)]
    pub include_paths: Vec<String>,
    /// Path prefixes/files to exclude.
    #[serde(default)]
    pub exclude_paths: Vec<String>,
    /// Optional boolean tag filter.
    #[serde(default)]
    pub tags: Option<LinkGraphTagFilter>,
    /// Optional outgoing-link filter.
    #[serde(default)]
    pub link_to: Option<LinkGraphLinkFilter>,
    /// Optional incoming-link filter.
    #[serde(default)]
    pub linked_by: Option<LinkGraphLinkFilter>,
    /// Optional related-note filter.
    #[serde(default)]
    pub related: Option<LinkGraphRelatedFilter>,
    /// Content phrases that must be mentioned.
    #[serde(default)]
    pub mentions_of: Vec<String>,
    /// Notes that must mention the current note.
    #[serde(default)]
    pub mentioned_by_notes: Vec<String>,
    /// Keep only orphan notes.
    #[serde(default)]
    pub orphan: bool,
    /// Keep only notes without tags.
    #[serde(default)]
    pub tagless: bool,
    /// Keep only notes missing backlinks.
    #[serde(default)]
    pub missing_backlink: bool,
    /// Optional hit scope override.
    #[serde(default)]
    pub scope: Option<LinkGraphScope>,
    /// Optional heading depth cap (1..=6).
    #[serde(default)]
    pub max_heading_level: Option<usize>,
    /// Optional section tree-hop cap.
    #[serde(default)]
    pub max_tree_hops: Option<usize>,
    /// Collapse multiple section hits per doc.
    #[serde(default)]
    pub collapse_to_doc: Option<bool>,
    /// Allowed edge types for traversal/ranking.
    #[serde(default)]
    pub edge_types: Vec<LinkGraphEdgeType>,
    /// Optional section cap per document.
    #[serde(default)]
    pub per_doc_section_cap: Option<usize>,
    /// Optional minimum words for section hits.
    #[serde(default)]
    pub min_section_words: Option<usize>,
}
