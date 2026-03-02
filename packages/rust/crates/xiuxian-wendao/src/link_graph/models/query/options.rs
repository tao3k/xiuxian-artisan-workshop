use super::enums::LinkGraphMatchStrategy;
use super::filters::LinkGraphSearchFilters;
use super::sort::LinkGraphSortTerm;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Search options for link-graph index retrieval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphSearchOptions {
    /// Matching strategy (fts/exact/re).
    pub match_strategy: LinkGraphMatchStrategy,
    /// Whether matching is case-sensitive.
    pub case_sensitive: bool,
    /// Result ordering terms (priority order).
    #[serde(default)]
    pub sort_terms: Vec<LinkGraphSortTerm>,
    /// Structured filters.
    #[serde(default)]
    pub filters: LinkGraphSearchFilters,
    /// Keep rows with `created_ts >= created_after`.
    #[serde(default)]
    pub created_after: Option<i64>,
    /// Keep rows with `created_ts <= created_before`.
    #[serde(default)]
    pub created_before: Option<i64>,
    /// Keep rows with `modified_ts >= modified_after`.
    #[serde(default)]
    pub modified_after: Option<i64>,
    /// Keep rows with `modified_ts <= modified_before`.
    #[serde(default)]
    pub modified_before: Option<i64>,
}

impl Default for LinkGraphSearchOptions {
    fn default() -> Self {
        Self {
            match_strategy: LinkGraphMatchStrategy::Fts,
            case_sensitive: false,
            sort_terms: vec![LinkGraphSortTerm::default()],
            filters: LinkGraphSearchFilters::default(),
            created_after: None,
            created_before: None,
            modified_after: None,
            modified_before: None,
        }
    }
}

impl LinkGraphSearchOptions {
    /// Validate schema-equivalent constraints for runtime safety.
    ///
    /// # Errors
    ///
    /// Returns an error when any option violates schema constraints.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(filter) = &self.filters.link_to
            && filter.max_distance.is_some_and(|distance| distance == 0)
        {
            return Err(
                "link_graph search options schema violation at filters.link_to.max_distance: must be >= 1"
                    .to_string(),
            );
        }
        if let Some(filter) = &self.filters.linked_by
            && filter.max_distance.is_some_and(|distance| distance == 0)
        {
            return Err(
                "link_graph search options schema violation at filters.linked_by.max_distance: must be >= 1"
                    .to_string(),
            );
        }
        if let Some(filter) = &self.filters.related
            && filter.max_distance.is_some_and(|distance| distance == 0)
        {
            return Err(
                "link_graph search options schema violation at filters.related.max_distance: must be >= 1"
                    .to_string(),
            );
        }
        if let Some(filter) = &self.filters.related
            && let Some(ppr) = &filter.ppr
        {
            if let Some(alpha) = ppr.alpha
                && !(0.0..=1.0).contains(&alpha)
            {
                return Err(
                    "link_graph search options schema violation at filters.related.ppr.alpha: must be between 0 and 1"
                        .to_string(),
                );
            }
            if let Some(max_iter) = ppr.max_iter
                && max_iter == 0
            {
                return Err(
                    "link_graph search options schema violation at filters.related.ppr.max_iter: must be >= 1"
                        .to_string(),
                );
            }
            if let Some(tol) = ppr.tol
                && tol <= 0.0
            {
                return Err(
                    "link_graph search options schema violation at filters.related.ppr.tol: must be > 0"
                        .to_string(),
                );
            }
        }
        if let Some(level) = self.filters.max_heading_level
            && !(1..=6).contains(&level)
        {
            return Err(
                "link_graph search options schema violation at filters.max_heading_level: must be between 1 and 6"
                    .to_string(),
            );
        }
        if let Some(cap) = self.filters.per_doc_section_cap
            && cap == 0
        {
            return Err(
                "link_graph search options schema violation at filters.per_doc_section_cap: must be >= 1"
                    .to_string(),
            );
        }
        Ok(())
    }
}
