//! Markdown link graph index + retrieval algorithms.

pub mod agentic;
mod index;
mod models;
/// Subgraph narration helpers for relationship summaries.
pub mod narrator;
mod parser;
/// Advanced hybrid PPR implementation.
pub mod ppr_hybrid;
mod query;
mod runtime_config;
/// `GraphMem` saliency models, scoring, and `Valkey` persistence adapters.
pub mod saliency;
mod stats_cache;

pub use agentic::{
    LINK_GRAPH_SUGGESTED_LINK_DECISION_SCHEMA_VERSION, LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION,
    LinkGraphAgenticCandidatePair, LinkGraphAgenticExecutionConfig,
    LinkGraphAgenticExecutionResult, LinkGraphAgenticExpansionConfig,
    LinkGraphAgenticExpansionPlan, LinkGraphAgenticWorkerExecution, LinkGraphAgenticWorkerPhase,
    LinkGraphAgenticWorkerPlan, LinkGraphSuggestedLink, LinkGraphSuggestedLinkDecision,
    LinkGraphSuggestedLinkDecisionRequest, LinkGraphSuggestedLinkDecisionResult,
    LinkGraphSuggestedLinkRequest, LinkGraphSuggestedLinkState, valkey_suggested_link_decide,
    valkey_suggested_link_decide_with_valkey, valkey_suggested_link_decisions_recent,
    valkey_suggested_link_decisions_recent_with_valkey, valkey_suggested_link_log,
    valkey_suggested_link_log_with_valkey, valkey_suggested_link_recent,
    valkey_suggested_link_recent_latest, valkey_suggested_link_recent_latest_with_valkey,
    valkey_suggested_link_recent_with_valkey,
};
pub use index::{LinkGraphCacheBuildMeta, LinkGraphIndex, LinkGraphRefreshMode};
pub use models::{
    LINK_GRAPH_POLICY_REASON_VOCAB, LINK_GRAPH_REASON_BACKEND_UNAVAILABLE,
    LINK_GRAPH_REASON_GRAPH_INSUFFICIENT, LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_MODE_CONFLICT,
    LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_OVERRIDDEN, LINK_GRAPH_REASON_GRAPH_ONLY_POLICY_MISSING,
    LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED, LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED_EMPTY,
    LINK_GRAPH_REASON_GRAPH_ONLY_SEARCH_TIMEOUT, LINK_GRAPH_REASON_GRAPH_POLICY_MISSING,
    LINK_GRAPH_REASON_GRAPH_POLICY_MODE_CONFLICT, LINK_GRAPH_REASON_GRAPH_SEARCH_TIMEOUT,
    LINK_GRAPH_REASON_GRAPH_SUFFICIENT, LINK_GRAPH_REASON_HYBRID_SELECTED,
    LINK_GRAPH_REASON_VECTOR_ONLY_REQUESTED, LINK_GRAPH_RETRIEVAL_PLAN_SCHEMA_VERSION,
    LinkGraphAttachment, LinkGraphAttachmentHit, LinkGraphAttachmentKind, LinkGraphConfidenceLevel,
    LinkGraphDirection, LinkGraphDisplayHit, LinkGraphDocument, LinkGraphEdgeType, LinkGraphHit,
    LinkGraphLinkFilter, LinkGraphMatchStrategy, LinkGraphMetadata, LinkGraphNeighbor,
    LinkGraphPassage, LinkGraphPlannedSearchPayload, LinkGraphPprSubgraphMode,
    LinkGraphPromotedOverlayTelemetry, LinkGraphRelatedFilter, LinkGraphRelatedPprDiagnostics,
    LinkGraphRelatedPprOptions, LinkGraphRetrievalBudget, LinkGraphRetrievalMode,
    LinkGraphRetrievalPlanInput, LinkGraphRetrievalPlanRecord, LinkGraphScope,
    LinkGraphSearchFilters, LinkGraphSearchOptions, LinkGraphSortField, LinkGraphSortOrder,
    LinkGraphSortTerm, LinkGraphStats, LinkGraphTagFilter,
};
pub use narrator::narrate_subgraph;
pub use query::{ParsedLinkGraphQuery, parse_search_query};
pub use runtime_config::{
    LinkGraphIndexRuntimeConfig, resolve_link_graph_index_runtime,
    set_link_graph_config_home_override, set_link_graph_wendao_config_override,
};
pub use saliency::{
    LINK_GRAPH_SALIENCY_SCHEMA_VERSION, LinkGraphSaliencyPolicy, LinkGraphSaliencyState,
    LinkGraphSaliencyTouchRequest, compute_link_graph_saliency, valkey_saliency_del,
    valkey_saliency_get, valkey_saliency_get_with_valkey, valkey_saliency_touch,
    valkey_saliency_touch_with_valkey,
};
pub use stats_cache::{
    LINK_GRAPH_STATS_CACHE_SCHEMA_VERSION, valkey_stats_cache_del, valkey_stats_cache_get,
    valkey_stats_cache_set,
};
