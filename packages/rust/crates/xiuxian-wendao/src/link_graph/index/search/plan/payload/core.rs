use super::super::super::super::{LinkGraphIndex, parse_search_query};
use super::policy::evaluate_link_graph_policy;
use crate::link_graph::runtime_config::resolve_link_graph_agentic_runtime;
use crate::link_graph::{
    LinkGraphDisplayHit, LinkGraphHit, LinkGraphPlannedSearchPayload,
    LinkGraphPromotedOverlayTelemetry, LinkGraphSuggestedLink, LinkGraphSuggestedLinkState,
    ParsedLinkGraphQuery, valkey_suggested_link_recent_latest,
};
use std::collections::HashMap;

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search::plan) fn search_planned_payload_with_agentic_core(
        &self,
        query: &str,
        limit: usize,
        base_options: crate::link_graph::LinkGraphSearchOptions,
        include_provisional: Option<bool>,
        provisional_limit: Option<usize>,
        promoted_overlay: Option<LinkGraphPromotedOverlayTelemetry>,
    ) -> LinkGraphPlannedSearchPayload {
        let parsed = parse_search_query(query, base_options);
        let effective_limit = parsed.limit_override.unwrap_or(limit);
        if let Some(direct_id) = parsed.direct_id.as_deref() {
            let rows = self.execute_direct_id_lookup(direct_id, effective_limit, &parsed.options);
            return Self::build_planned_payload(
                parsed,
                effective_limit,
                rows,
                Vec::new(),
                None,
                promoted_overlay,
            );
        }

        let agentic_runtime = resolve_link_graph_agentic_runtime();
        let include_provisional =
            include_provisional.unwrap_or(agentic_runtime.search_include_provisional_default);
        let provisional_limit = provisional_limit
            .unwrap_or(agentic_runtime.search_provisional_limit)
            .max(1);
        let (provisional_suggestions, provisional_error) = if include_provisional {
            match valkey_suggested_link_recent_latest(
                provisional_limit,
                Some(LinkGraphSuggestedLinkState::Provisional),
            ) {
                Ok(rows) => (rows, None),
                Err(err) => (Vec::new(), Some(err)),
            }
        } else {
            (Vec::new(), None)
        };
        let provisional_doc_boosts = if include_provisional {
            self.build_provisional_doc_boosts(
                &parsed.query,
                parsed.options.case_sensitive,
                &provisional_suggestions,
            )
        } else {
            HashMap::new()
        };
        let rows = self.execute_search_with_doc_boosts(
            &parsed.query,
            effective_limit,
            &parsed.options,
            if provisional_doc_boosts.is_empty() {
                None
            } else {
                Some(&provisional_doc_boosts)
            },
        );
        Self::build_planned_payload(
            parsed,
            effective_limit,
            rows,
            provisional_suggestions,
            provisional_error,
            promoted_overlay,
        )
    }

    fn build_planned_payload(
        parsed: ParsedLinkGraphQuery,
        effective_limit: usize,
        rows: Vec<LinkGraphHit>,
        provisional_suggestions: Vec<LinkGraphSuggestedLink>,
        provisional_error: Option<String>,
        promoted_overlay: Option<LinkGraphPromotedOverlayTelemetry>,
    ) -> LinkGraphPlannedSearchPayload {
        let hit_count = rows.len();
        let section_hit_count = rows
            .iter()
            .filter(|row| {
                row.best_section
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|value| !value.is_empty())
            })
            .count();
        let hits = rows
            .iter()
            .map(LinkGraphDisplayHit::from)
            .collect::<Vec<_>>();

        // 2026 Evolution Trigger: Automatically touch search hits to evolve graph weights.
        crate::link_graph::saliency::touch_search_hits_async(&hits);

        let policy = evaluate_link_graph_policy(&rows, effective_limit);
        LinkGraphPlannedSearchPayload {
            query: parsed.query,
            options: parsed.options,
            hits,
            hit_count,
            section_hit_count,
            requested_mode: policy.requested_mode,
            selected_mode: policy.selected_mode,
            reason: policy.reason,
            graph_hit_count: policy.graph_hit_count,
            source_hint_count: policy.source_hint_count,
            graph_confidence_score: policy.graph_confidence_score,
            graph_confidence_level: policy.graph_confidence_level,
            retrieval_plan: Some(policy.retrieval_plan),
            results: rows,
            provisional_suggestions,
            provisional_error,
            promoted_overlay,
        }
    }
}
