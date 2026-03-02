use std::path::PathBuf;

use serde_json::{Value, json};
use xiuxian_zhenfa::{INTERNAL_ERROR_CODE, JsonRpcErrorObject};

use super::models::{WendaoSearchRequest, WendaoSearchResponseFormat};
use crate::LinkGraphIndex;
use crate::link_graph::LinkGraphPlannedSearchPayload;

const DEFAULT_SEARCH_LIMIT: usize = 20;
const MAX_SEARCH_LIMIT: usize = 200;

/// Execute `wendao.search` from JSON-RPC parameters.
///
/// # Errors
/// Returns JSON-RPC error payloads when params are invalid or search execution fails.
pub fn search_from_rpc_params(params: Value) -> Result<String, JsonRpcErrorObject> {
    let request: WendaoSearchRequest = serde_json::from_value(params).map_err(|error| {
        JsonRpcErrorObject::invalid_params(format!("invalid wendao.search params: {error}"))
    })?;
    execute_search(&request).map_err(|error| {
        JsonRpcErrorObject::new(
            INTERNAL_ERROR_CODE,
            "wendao search failed",
            Some(json!({ "details": error })),
        )
    })
}

/// Execute one Wendao search request.
///
/// # Errors
/// Returns an error when index construction, query execution, or payload serialization fails.
pub fn execute_search(request: &WendaoSearchRequest) -> Result<String, String> {
    let query = request.query.trim();
    if query.is_empty() {
        return Err("`query` must be a non-empty string".to_string());
    }

    let root_dir = request.root_dir.as_deref().unwrap_or(".").trim();
    if root_dir.is_empty() {
        return Err("`root_dir` must be non-empty when provided".to_string());
    }

    let root = PathBuf::from(root_dir);
    let limit = normalize_limit(request.limit);
    let base_options = request.options.clone().unwrap_or_default();

    let index = LinkGraphIndex::build(&root)
        .map_err(|error| format!("failed to build LinkGraph index at `{root_dir}`: {error}"))?;

    let payload = index.search_planned_payload_with_agentic(
        query,
        limit,
        base_options,
        request.include_provisional,
        request.provisional_limit,
    );

    match request.response_format {
        WendaoSearchResponseFormat::Markdown => Ok(render_markdown(&payload)),
        WendaoSearchResponseFormat::Json => serde_json::to_string(&payload)
            .map_err(|error| format!("failed to serialize search payload: {error}")),
    }
}

fn normalize_limit(raw: Option<usize>) -> usize {
    raw.unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, MAX_SEARCH_LIMIT)
}

fn render_markdown(payload: &LinkGraphPlannedSearchPayload) -> String {
    let mut lines = Vec::new();
    lines.push("## Wendao Search Results".to_string());
    lines.push(format!("- query: {}", payload.query));
    lines.push(format!("- total_hits: {}", payload.hit_count));
    lines.push(format!(
        "- retrieval_mode: {:?} (reason: {})",
        payload.selected_mode, payload.reason
    ));

    if payload.hits.is_empty() {
        lines.push("- hits: none".to_string());
        return lines.join("\n");
    }

    lines.push("### Hits".to_string());
    for (index, hit) in payload.hits.iter().enumerate() {
        let title = if hit.title.trim().is_empty() {
            hit.stem.as_str()
        } else {
            hit.title.as_str()
        };
        lines.push(format!(
            "{}. {} (`{}`) score={:.3}",
            index + 1,
            title,
            hit.path,
            hit.score
        ));
        if !hit.best_section.trim().is_empty() {
            lines.push(format!("   section: {}", hit.best_section));
        }
    }

    lines.join("\n")
}
