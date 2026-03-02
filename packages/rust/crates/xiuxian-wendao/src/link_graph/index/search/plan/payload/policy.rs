use crate::link_graph::runtime_config::resolve_link_graph_retrieval_policy_runtime;
use crate::link_graph::{
    LINK_GRAPH_REASON_GRAPH_INSUFFICIENT, LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED,
    LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED_EMPTY, LINK_GRAPH_REASON_GRAPH_SUFFICIENT,
    LINK_GRAPH_REASON_VECTOR_ONLY_REQUESTED, LinkGraphConfidenceLevel, LinkGraphHit,
    LinkGraphRetrievalBudget, LinkGraphRetrievalMode, LinkGraphRetrievalPlanInput,
    LinkGraphRetrievalPlanRecord,
};
use std::collections::HashSet;

pub(super) struct LinkGraphPolicyDecision {
    pub requested_mode: LinkGraphRetrievalMode,
    pub selected_mode: LinkGraphRetrievalMode,
    pub reason: String,
    pub graph_hit_count: usize,
    pub source_hint_count: usize,
    pub graph_confidence_score: f64,
    pub graph_confidence_level: LinkGraphConfidenceLevel,
    pub retrieval_plan: LinkGraphRetrievalPlanRecord,
}

fn confidence_level_from_score(score: f64) -> LinkGraphConfidenceLevel {
    let bounded = score.clamp(0.0, 1.0);
    if bounded <= 0.0 {
        return LinkGraphConfidenceLevel::None;
    }
    if bounded < 0.35 {
        return LinkGraphConfidenceLevel::Low;
    }
    if bounded < 0.7 {
        return LinkGraphConfidenceLevel::Medium;
    }
    LinkGraphConfidenceLevel::High
}

fn compute_graph_confidence(
    hits: &[LinkGraphHit],
    min_hits: usize,
    min_top_score: f64,
) -> (f64, LinkGraphConfidenceLevel) {
    if hits.is_empty() {
        return (0.0, LinkGraphConfidenceLevel::None);
    }

    let count_score =
        (usize_to_f64_saturating(hits.len()) / usize_to_f64_saturating(min_hits.max(1))).min(1.0);
    let top_score = hits
        .iter()
        .map(|hit| hit.score.clamp(0.0, 1.0))
        .fold(0.0, f64::max);
    let threshold_score = if min_top_score > 0.0 {
        (top_score / min_top_score).clamp(0.0, 1.0)
    } else {
        top_score
    };
    let confidence =
        (0.45 * count_score + 0.35 * top_score + 0.2 * threshold_score).clamp(0.0, 1.0);
    (confidence, confidence_level_from_score(confidence))
}

fn usize_to_f64_saturating(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}

fn graph_is_sufficient(hits: &[LinkGraphHit], min_hits: usize, min_top_score: f64) -> bool {
    if hits.len() < min_hits.max(1) {
        return false;
    }
    let top_score = hits
        .iter()
        .map(|hit| hit.score.clamp(0.0, 1.0))
        .fold(0.0, f64::max);
    top_score >= min_top_score.clamp(0.0, 1.0)
}

fn count_source_hints(hits: &[LinkGraphHit], cap: usize) -> usize {
    if hits.is_empty() {
        return 0;
    }
    let mut seen: HashSet<String> = HashSet::new();
    for hit in hits {
        let normalized = hit.path.trim().to_lowercase();
        if normalized.is_empty() {
            continue;
        }
        seen.insert(normalized);
        if seen.len() >= cap.max(1) {
            break;
        }
    }
    seen.len()
}

pub(super) fn evaluate_link_graph_policy(
    hits: &[LinkGraphHit],
    effective_limit: usize,
) -> LinkGraphPolicyDecision {
    let runtime = resolve_link_graph_retrieval_policy_runtime();
    let requested_mode = runtime.mode;
    let graph_hit_count = hits.len();
    let source_hint_count = count_source_hints(hits, runtime.max_sources);
    let (graph_confidence_score, graph_confidence_level) =
        compute_graph_confidence(hits, runtime.hybrid_min_hits, runtime.hybrid_min_top_score);

    let (selected_mode, reason) = match requested_mode {
        LinkGraphRetrievalMode::VectorOnly => (
            LinkGraphRetrievalMode::VectorOnly,
            LINK_GRAPH_REASON_VECTOR_ONLY_REQUESTED.to_string(),
        ),
        LinkGraphRetrievalMode::GraphOnly => {
            let reason = if hits.is_empty() {
                LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED_EMPTY
            } else {
                LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED
            };
            (LinkGraphRetrievalMode::GraphOnly, reason.to_string())
        }
        LinkGraphRetrievalMode::Hybrid => {
            if graph_is_sufficient(hits, runtime.hybrid_min_hits, runtime.hybrid_min_top_score) {
                (
                    LinkGraphRetrievalMode::GraphOnly,
                    LINK_GRAPH_REASON_GRAPH_SUFFICIENT.to_string(),
                )
            } else {
                (
                    LinkGraphRetrievalMode::VectorOnly,
                    LINK_GRAPH_REASON_GRAPH_INSUFFICIENT.to_string(),
                )
            }
        }
    };

    let budget = LinkGraphRetrievalBudget {
        candidate_limit: effective_limit
            .max(1)
            .saturating_mul(runtime.candidate_multiplier.max(1)),
        max_sources: runtime.max_sources.max(1),
        rows_per_source: runtime.graph_rows_per_source.max(1),
    };
    let retrieval_plan = LinkGraphRetrievalPlanRecord::new(LinkGraphRetrievalPlanInput {
        requested_mode,
        selected_mode,
        reason: reason.clone(),
        backend_name: "wendao".to_string(),
        graph_hit_count,
        source_hint_count,
        graph_confidence_score,
        graph_confidence_level,
        budget,
    });

    LinkGraphPolicyDecision {
        requested_mode,
        selected_mode,
        reason,
        graph_hit_count,
        source_hint_count,
        graph_confidence_score,
        graph_confidence_level,
        retrieval_plan,
    }
}
