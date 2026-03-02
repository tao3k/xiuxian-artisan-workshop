//! Search command execution.

use crate::helpers::{
    build_optional_link_filter, build_optional_related_filter, build_optional_related_ppr_options,
    build_optional_tag_filter, build_phase_monitor_summary, build_promoted_overlay_monitor_phase,
    emit, parse_sort_term,
};
use crate::types::{Cli, Command, SearchArgs};
use anyhow::{Context, Result};
use serde_json::json;
use std::time::Instant;
use xiuxian_wendao::link_graph::{LINK_GRAPH_POLICY_REASON_VOCAB, LinkGraphPlannedSearchPayload};
use xiuxian_wendao::{
    LinkGraphIndex, LinkGraphMatchStrategy, LinkGraphSearchFilters, LinkGraphSearchOptions,
    LinkGraphSortTerm, parse_search_query,
};

pub(super) fn handle(cli: &Cli, index: Option<&LinkGraphIndex>) -> Result<()> {
    let Command::Search(args) = &cli.command else {
        unreachable!("search handler must be called with search command");
    };
    let args = args.as_ref();

    let index = index.context("link_graph index is required for search command")?;
    let bounded = args.limit.max(1);
    let base_options = build_base_options(args);
    let plan_parse_started = Instant::now();
    let preview = parse_search_query(&args.query, base_options.clone());
    let plan_parse_ms = plan_parse_started.elapsed().as_secs_f64() * 1000.0;
    let effective_limit = preview.limit_override.unwrap_or(bounded);
    let search_execute_started = Instant::now();
    let planned = index.search_planned_payload_with_agentic(
        &args.query,
        bounded,
        base_options,
        args.include_provisional,
        args.provisional_limit,
    );
    let reason_validated = LINK_GRAPH_POLICY_REASON_VOCAB.contains(&planned.reason.as_str());
    let search_execute_ms = search_execute_started.elapsed().as_secs_f64() * 1000.0;
    let phases = build_monitor_phases(
        args,
        &planned,
        plan_parse_ms,
        search_execute_ms,
        effective_limit,
        reason_validated,
    );
    let mut payload = build_payload(&planned, effective_limit);
    if args.verbosity.verbose
        && let Some(map) = payload.as_object_mut()
    {
        map.insert("phases".to_string(), json!(phases));
        map.insert(
            "monitor".to_string(),
            json!({
                "bottlenecks": build_phase_monitor_summary(&phases),
            }),
        );
    }
    emit(&payload, cli.output)
}

fn build_base_options(args: &SearchArgs) -> LinkGraphSearchOptions {
    let normalized_sort_terms: Vec<LinkGraphSortTerm> = if args.sort_terms.is_empty() {
        vec![LinkGraphSortTerm::default()]
    } else {
        args.sort_terms
            .iter()
            .map(|term| parse_sort_term(term))
            .collect()
    };
    let related_ppr = build_optional_related_ppr_options(
        args.related_ppr_alpha,
        args.related_ppr_max_iter,
        args.related_ppr_tol,
        args.related_ppr_subgraph_mode,
    );
    let filters = LinkGraphSearchFilters {
        include_paths: args.include_paths.clone(),
        exclude_paths: args.exclude_paths.clone(),
        tags: build_optional_tag_filter(&args.tags_all, &args.tags_any, &args.tags_not),
        link_to: build_optional_link_filter(
            &args.link_to,
            args.link_to_options.link_to_negate,
            args.link_to_options.link_to_recursive,
            args.link_to_max_distance,
        ),
        linked_by: build_optional_link_filter(
            &args.linked_by,
            args.linked_by_options.linked_by_negate,
            args.linked_by_options.linked_by_recursive,
            args.linked_by_max_distance,
        ),
        related: build_optional_related_filter(&args.related, args.max_distance, related_ppr),
        mentions_of: args.mentions_of.clone(),
        mentioned_by_notes: args.mentioned_by_notes.clone(),
        orphan: args.filter_flags.orphan,
        tagless: args.filter_flags.tagless,
        missing_backlink: args.filter_flags.missing_backlink,
        ..LinkGraphSearchFilters::default()
    };
    LinkGraphSearchOptions {
        match_strategy: LinkGraphMatchStrategy::from_alias(&args.match_strategy),
        case_sensitive: args.case_options.case_sensitive,
        sort_terms: normalized_sort_terms,
        filters,
        created_after: args.created_after,
        created_before: args.created_before,
        modified_after: args.modified_after,
        modified_before: args.modified_before,
    }
}

fn build_monitor_phases(
    args: &SearchArgs,
    planned: &LinkGraphPlannedSearchPayload,
    plan_parse_ms: f64,
    search_execute_ms: f64,
    effective_limit: usize,
    reason_validated: bool,
) -> Vec<serde_json::Value> {
    let mut phases = vec![
        json!({
            "phase": "link_graph.search.plan_parse",
            "duration_ms": plan_parse_ms,
            "extra": {
                "query_len": args.query.len(),
                "effective_limit": effective_limit,
            }
        }),
        json!({
            "phase": "link_graph.search.execute",
            "duration_ms": search_execute_ms,
            "extra": {
                "total": planned.hit_count,
                "section_hit_count": planned.section_hit_count,
            }
        }),
        json!({
            "phase": "link_graph.search.policy",
            "duration_ms": 0.0,
            "extra": {
                "requested_mode": planned.requested_mode,
                "selected_mode": planned.selected_mode,
                "reason": planned.reason.as_str(),
                "reason_validated": reason_validated,
                "graph_hit_count": planned.graph_hit_count,
                "source_hint_count": planned.source_hint_count,
                "graph_confidence_score": planned.graph_confidence_score,
                "graph_confidence_level": planned.graph_confidence_level,
            }
        }),
    ];
    if let Some(promoted_overlay) = planned.promoted_overlay.as_ref() {
        phases.push(build_promoted_overlay_monitor_phase(promoted_overlay));
    }
    if !planned.provisional_suggestions.is_empty()
        || planned.provisional_error.is_some()
        || args.include_provisional.is_some()
        || args.provisional_limit.is_some()
    {
        phases.push(json!({
            "phase": "link_graph.search.provisional",
            "duration_ms": 0.0,
            "extra": {
                "suggestion_count": planned.provisional_suggestions.len(),
                "has_error": planned.provisional_error.is_some(),
            }
        }));
    }
    phases
}

fn build_payload(
    planned: &LinkGraphPlannedSearchPayload,
    effective_limit: usize,
) -> serde_json::Value {
    json!({
        "query": planned.query,
        "limit": effective_limit,
        "match_strategy": planned.options.match_strategy,
        "sort_terms": planned.options.sort_terms,
        "case_sensitive": planned.options.case_sensitive,
        "filters": planned.options.filters,
        "created_after": planned.options.created_after,
        "created_before": planned.options.created_before,
        "modified_after": planned.options.modified_after,
        "modified_before": planned.options.modified_before,
        "total": planned.hit_count,
        "hits": planned.hits,
        "section_hit_count": planned.section_hit_count,
        "requested_mode": planned.requested_mode,
        "selected_mode": planned.selected_mode,
        "reason": planned.reason,
        "graph_hit_count": planned.graph_hit_count,
        "source_hint_count": planned.source_hint_count,
        "graph_confidence_score": planned.graph_confidence_score,
        "graph_confidence_level": planned.graph_confidence_level,
        "retrieval_plan": planned.retrieval_plan,
        "results": planned.results,
        "provisional_suggestions": planned.provisional_suggestions,
        "provisional_error": planned.provisional_error,
        "promoted_overlay": planned.promoted_overlay,
    })
}
