//! Integration tests for the `KnowledgeGraph` module.
//!
//! Covers: CRUD, multi-hop search, persistence, skill registration,
//! query-time tool relevance, and export/import roundtrip.

use tempfile::TempDir;
use xiuxian_wendao::graph::{KnowledgeGraph, SkillDoc, entity_from_dict};
use xiuxian_wendao::{Entity, EntityType, Relation, RelationType};

fn has_valkey() -> bool {
    std::env::var("VALKEY_URL")
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
}

mod entity_relation_crud;
mod entity_search_scoring;
mod graph_persistence;
mod graph_traversal;
mod skill_registration;
mod tool_relevance;
mod valkey_persistence;
