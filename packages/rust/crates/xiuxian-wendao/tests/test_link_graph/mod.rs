//! Integration tests for `LinkGraph` parsing, retrieval, and cache behaviors.

use redis::Connection;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use xiuxian_wendao::link_graph::{
    LINK_GRAPH_RETRIEVAL_PLAN_SCHEMA_VERSION, LinkGraphAttachmentKind, LinkGraphConfidenceLevel,
    LinkGraphDirection, LinkGraphEdgeType, LinkGraphIndex, LinkGraphLinkFilter,
    LinkGraphMatchStrategy, LinkGraphPprSubgraphMode, LinkGraphRefreshMode, LinkGraphRelatedFilter,
    LinkGraphRelatedPprOptions, LinkGraphRetrievalMode, LinkGraphScope, LinkGraphSearchFilters,
    LinkGraphSearchOptions, LinkGraphSortField, LinkGraphSortOrder, LinkGraphSortTerm,
    parse_search_query,
};
use xiuxian_wendao::{
    LinkGraphSaliencyPolicy, compute_link_graph_saliency, valkey_saliency_get_with_valkey,
};

fn write_file(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn sort_term(field: LinkGraphSortField, order: LinkGraphSortOrder) -> LinkGraphSortTerm {
    LinkGraphSortTerm { field, order }
}

fn valkey_connection() -> Result<Connection, Box<dyn std::error::Error>> {
    let client = redis::Client::open("redis://127.0.0.1:6379/0")?;
    let conn = client.get_connection()?;
    Ok(conn)
}

fn clear_cache_keys(prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = valkey_connection()?;
    let pattern = format!("{prefix}:*");
    let keys: Vec<String> = redis::cmd("KEYS").arg(&pattern).query(&mut conn)?;
    if !keys.is_empty() {
        redis::cmd("DEL").arg(keys).query::<()>(&mut conn)?;
    }
    Ok(())
}

fn count_cache_keys(prefix: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let mut conn = valkey_connection()?;
    let pattern = format!("{prefix}:*");
    let keys: Vec<String> = redis::cmd("KEYS").arg(&pattern).query(&mut conn)?;
    Ok(keys.len())
}

fn unique_cache_prefix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_nanos())
        .unwrap_or(0);
    format!("omni:test:link_graph:{nanos}")
}

mod build_scope;
mod cache_build;
mod graph_navigation;
mod markdown_attachments;
mod query_parsing;
mod refresh;
mod search_core;
mod search_filters;
mod search_match_strategies;
mod tree_scope_filters;
