use anyhow::{Result, anyhow};
use arrow::array::{Float32Array, StringArray};
use arrow_ipc::reader::StreamReader;
use omni_types::VectorSearchResult;

use super::ipc::{search_results_to_ipc, tool_search_results_to_ipc};
use crate::skill::ToolSearchResult;

#[test]
fn test_search_results_to_ipc_empty() -> Result<()> {
    let bytes = search_results_to_ipc(&[], None).map_err(|error| anyhow!(error))?;
    assert!(!bytes.is_empty(), "IPC stream should contain schema");
    Ok(())
}

#[test]
fn test_search_results_to_ipc_one_row() -> Result<()> {
    let r = VectorSearchResult {
        id: "tool.a".to_string(),
        content: "Does something".to_string(),
        tool_name: "tool.a".to_string(),
        file_path: "/path/to/file".to_string(),
        routing_keywords: "kw1 kw2".to_string(),
        intents: "intent1 | intent2".to_string(),
        metadata: serde_json::json!({"x": 1}),
        distance: 0.5,
    };
    let bytes = search_results_to_ipc(&[r], None).map_err(|error| anyhow!(error))?;
    assert!(!bytes.is_empty());
    Ok(())
}

#[test]
fn test_search_results_to_ipc_projection() -> Result<()> {
    let r = VectorSearchResult {
        id: "a".to_string(),
        content: "text".to_string(),
        tool_name: "t".to_string(),
        file_path: "p".to_string(),
        routing_keywords: String::new(),
        intents: String::new(),
        metadata: serde_json::json!({}),
        distance: 0.1,
    };
    let proj = vec![
        "id".to_string(),
        "content".to_string(),
        "_distance".to_string(),
    ];
    let bytes = search_results_to_ipc(std::slice::from_ref(&r), Some(proj.as_slice()))
        .map_err(|error| anyhow!(error))?;
    assert!(!bytes.is_empty());
    let full = search_results_to_ipc(&[r], None).map_err(|error| anyhow!(error))?;
    assert!(bytes.len() < full.len(), "projected IPC should be smaller");
    Ok(())
}

#[test]
fn test_search_results_to_ipc_invalid_projection() {
    let r = VectorSearchResult {
        id: "a".to_string(),
        content: "b".to_string(),
        tool_name: "t".to_string(),
        file_path: "p".to_string(),
        routing_keywords: String::new(),
        intents: String::new(),
        metadata: serde_json::json!({}),
        distance: 0.0,
    };
    let bad = vec!["id".to_string(), "no_such_column".to_string()];
    let Err(err) = search_results_to_ipc(&[r], Some(bad.as_slice())) else {
        panic!("invalid projection should fail")
    };
    assert!(err.contains("invalid ipc_projection"));
}

#[test]
fn test_tool_search_results_to_ipc_empty() -> Result<()> {
    let bytes = tool_search_results_to_ipc(&[]).map_err(|error| anyhow!(error))?;
    assert!(!bytes.is_empty());
    Ok(())
}

#[test]
fn test_tool_search_results_to_ipc_one_row() -> Result<()> {
    let r = ToolSearchResult {
        name: "git.commit".to_string(),
        description: "Commit changes".to_string(),
        input_schema: serde_json::json!({"type": "object"}),
        score: 0.85,
        vector_score: Some(0.8),
        keyword_score: Some(0.5),
        skill_name: "git".to_string(),
        tool_name: "commit".to_string(),
        file_path: "git/scripts/commit.py".to_string(),
        routing_keywords: vec!["git".to_string(), "commit".to_string()],
        intents: vec!["Save changes".to_string()],
        category: "vcs".to_string(),
        parameters: vec![],
    };
    let bytes = tool_search_results_to_ipc(&[r]).map_err(|error| anyhow!(error))?;
    assert!(!bytes.is_empty());

    let mut reader = StreamReader::try_new(std::io::Cursor::new(bytes), None)?;
    let batch = reader
        .next()
        .ok_or_else(|| anyhow!("missing first record batch"))??;

    let final_scores = batch
        .column_by_name("final_score")
        .ok_or_else(|| anyhow!("missing final_score column"))?
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| anyhow!("final_score type mismatch"))?;
    let confidences = batch
        .column_by_name("confidence")
        .ok_or_else(|| anyhow!("missing confidence column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("confidence type mismatch"))?;
    let ranking_reasons = batch
        .column_by_name("ranking_reason")
        .ok_or_else(|| anyhow!("missing ranking_reason column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("ranking_reason type mismatch"))?;
    let digests = batch
        .column_by_name("input_schema_digest")
        .ok_or_else(|| anyhow!("missing input_schema_digest column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("input_schema_digest type mismatch"))?;

    assert!(final_scores.value(0) > 0.0);
    assert!(matches!(confidences.value(0), "high" | "medium" | "low"));
    assert!(ranking_reasons.value(0).contains("final="));
    assert!(digests.value(0).starts_with("sha256:"));
    Ok(())
}
