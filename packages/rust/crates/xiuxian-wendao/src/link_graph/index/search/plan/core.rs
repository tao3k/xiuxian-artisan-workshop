use super::super::super::{
    LinkGraphHit, LinkGraphIndex, LinkGraphScope, LinkGraphSearchOptions, ParsedLinkGraphQuery,
    parse_search_query,
};
use super::super::context::SearchRuntimePolicy;
use std::collections::HashMap;

impl LinkGraphIndex {
    /// Parse query directives/options once and execute the resulting search plan.
    #[must_use]
    pub fn search_planned(
        &self,
        query: &str,
        limit: usize,
        base_options: LinkGraphSearchOptions,
    ) -> (ParsedLinkGraphQuery, Vec<LinkGraphHit>) {
        if let Some(overlay) = self.with_promoted_edges_overlay() {
            return overlay.search_planned_core(query, limit, base_options);
        }
        self.search_planned_core(query, limit, base_options)
    }

    fn search_planned_core(
        &self,
        query: &str,
        limit: usize,
        base_options: LinkGraphSearchOptions,
    ) -> (ParsedLinkGraphQuery, Vec<LinkGraphHit>) {
        let parsed = parse_search_query(query, base_options);
        let effective_limit = parsed.limit_override.unwrap_or(limit);
        if let Some(direct_id) = parsed.direct_id.as_deref() {
            let rows = self.execute_direct_id_lookup(direct_id, effective_limit, &parsed.options);
            return (parsed, rows);
        }
        let rows = self.execute_search(&parsed.query, effective_limit, &parsed.options);
        (parsed, rows)
    }

    #[must_use]
    pub(in crate::link_graph::index::search::plan) fn execute_direct_id_lookup(
        &self,
        direct_id: &str,
        limit: usize,
        options: &LinkGraphSearchOptions,
    ) -> Vec<LinkGraphHit> {
        if limit == 0 {
            return Vec::new();
        }

        let Some(doc) = self.resolve_doc(direct_id) else {
            return Vec::new();
        };
        let Some(context) = Self::prepare_execution_context("", limit, options) else {
            return Vec::new();
        };
        if !self.matches_structured_filters(doc, options, &context) {
            return Vec::new();
        }
        if !Self::matches_temporal_filters(doc, options) {
            return Vec::new();
        }
        if let Some(graph_candidates) = self.graph_filter_candidates(options)
            && !graph_candidates.contains(&doc.id)
        {
            return Vec::new();
        }

        vec![LinkGraphHit {
            stem: doc.stem.clone(),
            title: doc.title.clone(),
            path: doc.path.clone(),
            doc_type: doc.doc_type.clone(),
            tags: doc.tags.clone(),
            score: 1.0,
            best_section: None,
            match_reason: Some("direct_id".to_string()),
        }]
    }

    /// Execute query plan with explicit matching and sorting options.
    #[must_use]
    fn execute_search(
        &self,
        query: &str,
        limit: usize,
        options: &LinkGraphSearchOptions,
    ) -> Vec<LinkGraphHit> {
        self.execute_search_with_doc_boosts(query, limit, options, None)
    }

    /// Execute query plan with explicit matching/sorting options and
    /// optional agentic provisional doc-score boosts.
    #[must_use]
    pub(in crate::link_graph::index::search::plan) fn execute_search_with_doc_boosts(
        &self,
        query: &str,
        limit: usize,
        options: &LinkGraphSearchOptions,
        doc_boosts: Option<&HashMap<String, f64>>,
    ) -> Vec<LinkGraphHit> {
        let Some(context) = Self::prepare_execution_context(query, limit, options) else {
            return Vec::new();
        };
        let raw_query = context.raw_query.as_str();
        let graph_candidates = self.graph_filter_candidates(options);
        if raw_query.is_empty()
            && graph_candidates.is_none()
            && !Self::has_non_query_filters(options)
        {
            return Vec::new();
        }

        let scope = Self::effective_scope(&options.filters);
        let structural_edges_enabled = Self::allows_structural_edges(&options.filters);
        let semantic_edges_enabled = Self::allows_semantic_edges(&options.filters);
        if matches!(scope, LinkGraphScope::SectionOnly) && !structural_edges_enabled {
            return Vec::new();
        }
        let collapse_to_doc = options.filters.collapse_to_doc.unwrap_or(true);
        let runtime_policy = SearchRuntimePolicy {
            scope,
            structural_edges_enabled,
            semantic_edges_enabled,
            collapse_to_doc,
            per_doc_section_cap: Self::effective_per_doc_section_cap(&options.filters, scope),
            min_section_words: Self::effective_min_section_words(&options.filters, scope),
            max_heading_level: Self::effective_max_heading_level(&options.filters),
            max_tree_hops: options.filters.max_tree_hops,
        };

        let rows = self.collect_search_rows(
            options,
            &context,
            graph_candidates.as_ref(),
            &runtime_policy,
        );
        self.finalize_search_rows(rows, options, context.bounded, doc_boosts)
    }
}
