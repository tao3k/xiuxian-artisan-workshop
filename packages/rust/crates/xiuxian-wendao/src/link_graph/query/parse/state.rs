use crate::link_graph::models::{
    LinkGraphEdgeType, LinkGraphLinkFilter, LinkGraphMatchStrategy, LinkGraphPprSubgraphMode,
    LinkGraphRelatedFilter, LinkGraphRelatedPprOptions, LinkGraphScope, LinkGraphSearchFilters,
    LinkGraphSortTerm,
};

#[derive(Debug, Clone, Default)]
pub(super) struct ParsedDirectiveState {
    pub match_strategy: Option<LinkGraphMatchStrategy>,
    pub sort_terms: Vec<LinkGraphSortTerm>,
    pub case_sensitive: Option<bool>,
    pub limit_override: Option<usize>,
    pub direct_id: Option<String>,

    pub filters: LinkGraphSearchFilters,
    pub tags_all: Vec<String>,
    pub tags_any: Vec<String>,
    pub tags_not: Vec<String>,
    pub link_to: LinkGraphLinkFilter,
    pub linked_by: LinkGraphLinkFilter,
    pub related: LinkGraphRelatedFilter,
    pub related_ppr: LinkGraphRelatedPprOptions,
    pub scope: Option<LinkGraphScope>,
    pub max_heading_level: Option<usize>,
    pub max_tree_hops: Option<usize>,
    pub collapse_to_doc: Option<bool>,
    pub edge_types: Vec<LinkGraphEdgeType>,
    pub per_doc_section_cap: Option<usize>,
    pub min_section_words: Option<usize>,

    pub created_after: Option<i64>,
    pub created_before: Option<i64>,
    pub modified_after: Option<i64>,
    pub modified_before: Option<i64>,
}

pub(super) fn parse_ppr_subgraph_mode(raw: &str) -> Option<LinkGraphPprSubgraphMode> {
    match raw.trim().to_lowercase().as_str() {
        "auto" => Some(LinkGraphPprSubgraphMode::Auto),
        "disabled" => Some(LinkGraphPprSubgraphMode::Disabled),
        "force" => Some(LinkGraphPprSubgraphMode::Force),
        _ => None,
    }
}

pub(super) fn has_related_ppr_options(value: &LinkGraphRelatedPprOptions) -> bool {
    value.alpha.is_some()
        || value.max_iter.is_some()
        || value.tol.is_some()
        || value.subgraph_mode.is_some()
}
