//! xiuxian-wendao - High-performance knowledge management library.
//!
//! Module layout (by domain):
//! - `types` / `knowledge_py`: Knowledge entries and categories
//! - `storage` / `storage_py`: Valkey-backed persistence
//! - `sync` / `sync_py`: Incremental file sync engine
//! - `entity` / `graph` / `graph_py`: Knowledge graph (entities, relations, search)
//! - `enhancer` / `enhancer_py`: `LinkGraph` note enhancement
//! - `link_graph_refs` / `link_graph_refs_py`: `LinkGraph` entity references
//! - `dependency_indexer` / `dep_indexer_py`: Dependency scanning
//! - `unified_symbol` / `unified_symbol_py`: Cross-language symbol index
//!
//! # Examples
//!
//! ```rust
//! use xiuxian_wendao::{KnowledgeEntry, KnowledgeCategory};
//!
//! let entry = KnowledgeEntry::new(
//!     "test-001".to_string(),
//!     "Error Handling Pattern".to_string(),
//!     "Best practices for error handling...".to_string(),
//!     KnowledgeCategory::Pattern,
//! ).with_tags(vec!["error".to_string(), "exception".to_string()]);
//! ```
//!
//! # Knowledge Graph Examples
//!
//! ```rust
//! use xiuxian_wendao::{Entity, Relation, EntityType, RelationType, KnowledgeGraph};
//!
//! let graph = KnowledgeGraph::new();
//!
//! let entity = Entity::new(
//!     "tool:claude-code".to_string(),
//!     "Claude Code".to_string(),
//!     EntityType::Tool,
//!     "AI coding assistant".to_string(),
//! );
//!
//! graph.add_entity(entity).unwrap();
//! ```
use pyo3::prelude::*;

// ---------------------------------------------------------------------------
// Core domain modules
// ---------------------------------------------------------------------------
mod entity;
pub mod graph;
/// HMAS blackboard protocol contracts and validators.
pub mod hmas;
pub mod kg_cache;
pub mod link_graph;
pub mod link_graph_py;
pub mod schemas;
pub mod skill_vfs;
mod storage;
mod sync;
mod types;

// ---------------------------------------------------------------------------
// PyO3 binding modules (one per domain)
// ---------------------------------------------------------------------------
pub mod graph_py;
pub mod knowledge_py;
mod python_module;
/// Python bindings for accessing bundled JSON schemas by canonical name.
pub mod schema_py;
pub mod storage_py;
pub mod sync_py;

// ---------------------------------------------------------------------------
// Fusion recall boost (Rust computation, Python thin wrapper)
// ---------------------------------------------------------------------------
mod fusion;
pub mod fusion_py;

// ---------------------------------------------------------------------------
// Feature modules (enhancer, link graph refs, dependency, unified symbol)
// ---------------------------------------------------------------------------
pub mod dep_indexer_py;
pub mod dependency_indexer;
pub mod enhancer;
pub mod enhancer_py;
pub mod link_graph_refs;
mod link_graph_refs_py;
pub mod unified_symbol;
pub mod unified_symbol_py;
#[cfg(feature = "zhenfa-router")]
/// Zhenfa HTTP/RPC router integration for Wendao retrieval capabilities.
pub mod zhenfa_router;

// ---------------------------------------------------------------------------
// Public re-exports (crate API)
// ---------------------------------------------------------------------------
pub use dep_indexer_py::{
    PyDependencyConfig, PyDependencyIndexResult, PyDependencyIndexer, PyDependencyStats,
    PyExternalDependency, PyExternalSymbol, PySymbolIndex,
};
pub use dependency_indexer::{
    ConfigExternalDependency, DependencyBuildConfig, DependencyConfig, DependencyIndexResult,
    DependencyIndexer, DependencyStats, ExternalSymbol, SymbolIndex, SymbolKind,
};
pub use enhancer::{
    EnhancedNote, EntityRefData, InferredRelation, MarkdownConfigBlock, MarkdownConfigLinkTarget,
    MarkdownConfigMemoryIndex, MissingEmbeddedLink, NoteFrontmatter, NoteInput, RefStatsData,
    SkillReferenceSemantics, WendaoResourceFile, WendaoResourceLinkTarget, WendaoResourceRegistry,
    WendaoResourceRegistryError, classify_skill_reference, enhance_note, enhance_notes_batch,
    extract_markdown_config_blocks, extract_markdown_config_link_targets_by_id,
    extract_markdown_config_links_by_id, parse_frontmatter,
};
pub use enhancer_py::{
    PyEnhancedNote, PyInferredRelation, PyNoteFrontmatter, link_graph_enhance_note,
    link_graph_enhance_notes_batch, link_graph_parse_frontmatter,
};
pub use entity::{
    Entity, EntitySearchQuery, EntityType, GraphStats, MultiHopOptions, Relation, RelationType,
};
pub use graph::{KnowledgeGraph, QueryIntent, SkillDoc, SkillRegistrationResult, extract_intent};
pub use hmas::{
    HmasConclusionPayload, HmasDigitalThreadPayload, HmasEvidencePayload, HmasRecordKind,
    HmasSourceNode, HmasTaskPayload, HmasValidationIssue, HmasValidationReport,
    validate_blackboard_file, validate_blackboard_markdown,
};
pub use link_graph::{
    LINK_GRAPH_RETRIEVAL_PLAN_SCHEMA_VERSION, LINK_GRAPH_SALIENCY_SCHEMA_VERSION,
    LINK_GRAPH_SUGGESTED_LINK_DECISION_SCHEMA_VERSION, LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION,
    LinkGraphAgenticCandidatePair, LinkGraphAgenticExecutionConfig,
    LinkGraphAgenticExecutionResult, LinkGraphAgenticExpansionConfig,
    LinkGraphAgenticExpansionPlan, LinkGraphAgenticWorkerExecution, LinkGraphAgenticWorkerPhase,
    LinkGraphAgenticWorkerPlan, LinkGraphAttachment, LinkGraphAttachmentHit,
    LinkGraphAttachmentKind, LinkGraphConfidenceLevel, LinkGraphDirection, LinkGraphDocument,
    LinkGraphEdgeType, LinkGraphHit, LinkGraphIndex, LinkGraphLinkFilter, LinkGraphMatchStrategy,
    LinkGraphMetadata, LinkGraphNeighbor, LinkGraphPassage, LinkGraphPprSubgraphMode,
    LinkGraphRelatedFilter, LinkGraphRelatedPprDiagnostics, LinkGraphRelatedPprOptions,
    LinkGraphRetrievalBudget, LinkGraphRetrievalMode, LinkGraphRetrievalPlanRecord,
    LinkGraphSaliencyPolicy, LinkGraphSaliencyState, LinkGraphSaliencyTouchRequest, LinkGraphScope,
    LinkGraphSearchFilters, LinkGraphSearchOptions, LinkGraphSortField, LinkGraphSortOrder,
    LinkGraphSortTerm, LinkGraphStats, LinkGraphSuggestedLink, LinkGraphSuggestedLinkDecision,
    LinkGraphSuggestedLinkDecisionRequest, LinkGraphSuggestedLinkDecisionResult,
    LinkGraphSuggestedLinkRequest, LinkGraphSuggestedLinkState, LinkGraphTagFilter,
    ParsedLinkGraphQuery, compute_link_graph_saliency, narrate_subgraph, parse_search_query,
    resolve_link_graph_index_runtime, set_link_graph_config_home_override,
    set_link_graph_wendao_config_override, valkey_saliency_del, valkey_saliency_get,
    valkey_saliency_get_with_valkey, valkey_saliency_touch, valkey_saliency_touch_with_valkey,
    valkey_suggested_link_decide, valkey_suggested_link_decide_with_valkey,
    valkey_suggested_link_decisions_recent, valkey_suggested_link_decisions_recent_with_valkey,
    valkey_suggested_link_log, valkey_suggested_link_log_with_valkey, valkey_suggested_link_recent,
    valkey_suggested_link_recent_latest, valkey_suggested_link_recent_latest_with_valkey,
    valkey_suggested_link_recent_with_valkey,
};
pub use link_graph_py::{
    PyLinkGraphEngine, link_graph_stats_cache_del, link_graph_stats_cache_get,
    link_graph_stats_cache_set,
};
pub use link_graph_refs::{
    LinkGraphEntityRef, LinkGraphRefStats, extract_entity_refs, find_notes_referencing_entity,
    get_ref_stats,
};
pub use link_graph_refs_py::{
    PyLinkGraphEntityRef, PyLinkGraphRefStats, link_graph_count_refs,
    link_graph_extract_entity_refs, link_graph_find_referencing_notes, link_graph_get_ref_stats,
    link_graph_is_valid_ref, link_graph_parse_entity_ref,
};
pub use skill_vfs::{
    AssetRequest, SkillNamespaceIndex, SkillNamespaceMount, SkillVfsError, SkillVfsResolver,
    WENDAO_URI_SCHEME, WendaoAssetHandle, WendaoResourceUri, ZHIXING_SKILL_DOC_PATH,
    ZhixingIndexSummary, ZhixingWendaoIndexer, build_embedded_wendao_registry,
    embedded_discover_canonical_uris, embedded_resource_text,
    embedded_resource_text_from_wendao_uri, embedded_skill_links_for_id,
    embedded_skill_links_for_reference_type, embedded_skill_links_index, embedded_skill_markdown,
};
pub use storage::KnowledgeStorage;
pub use sync::{
    DiscoveryOptions, FileChange, IncrementalSyncPolicy, SyncEngine, SyncManifest, SyncResult,
    extension_from_path, extract_extensions_from_glob_patterns, normalize_extension,
};
pub use types::{KnowledgeCategory, KnowledgeEntry, KnowledgeSearchQuery, KnowledgeStats};
pub use unified_symbol::{SymbolSource, UnifiedIndexStats, UnifiedSymbol, UnifiedSymbolIndex};
pub use unified_symbol_py::{PyUnifiedIndexStats, PyUnifiedSymbol, PyUnifiedSymbolIndex};
#[cfg(feature = "zhenfa-router")]
pub use zhenfa_router::WendaoZhenfaRouter;

// Re-export PyO3 types for convenience
pub use graph_py::{
    PyEntity, PyEntityType, PyKnowledgeGraph, PyQueryIntent, PyRelation, PySkillDoc,
    extract_query_intent, invalidate_kg_cache, load_kg_from_valkey_cached,
};
pub use knowledge_py::{PyKnowledgeCategory, PyKnowledgeEntry, create_knowledge_entry};
pub use storage_py::PyKnowledgeStorage;
pub use sync_py::{PySyncEngine, PySyncResult, compute_hash};

// ---------------------------------------------------------------------------
// Python module registration
// ---------------------------------------------------------------------------

/// Python module definition — delegates to domain-specific binding modules.
#[pymodule]
fn _xiuxian_wendao(py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    python_module::register(py, m)
}
