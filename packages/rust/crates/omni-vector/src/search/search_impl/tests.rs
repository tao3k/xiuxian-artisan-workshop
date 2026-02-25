use arrow::array::{Float32Array, StringArray};
use arrow_ipc::reader::StreamReader;
use omni_types::VectorSearchResult;

use super::ipc::{search_results_to_ipc, tool_search_results_to_ipc};
use crate::skill::ToolSearchResult;

#[test]
fn test_search_results_to_ipc_empty() {
    let bytes = search_results_to_ipc(&[], None).expect("empty search IPC should serialize schema");
    assert!(!bytes.is_empty(), "IPC stream should contain schema");
}

#[test]
fn test_search_results_to_ipc_one_row() {
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
    let bytes = search_results_to_ipc(&[r], None).expect("one-row search IPC should serialize");
    assert!(!bytes.is_empty());
}

#[test]
fn test_search_results_to_ipc_projection() {
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
    let bytes = search_results_to_ipc(&[r.clone()], Some(proj.as_slice()))
        .expect("projected search IPC should serialize");
    assert!(!bytes.is_empty());
    let full = search_results_to_ipc(&[r], None).expect("full search IPC should serialize");
    assert!(bytes.len() < full.len(), "projected IPC should be smaller");
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
    let err = search_results_to_ipc(&[r], Some(bad.as_slice()))
        .expect_err("invalid projection should fail");
    assert!(err.contains("invalid ipc_projection"));
}

#[test]
fn test_tool_search_results_to_ipc_empty() {
    let bytes = tool_search_results_to_ipc(&[]).expect("empty tool IPC should serialize schema");
    assert!(!bytes.is_empty());
}

#[test]
fn test_tool_search_results_to_ipc_one_row() {
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
    let bytes = tool_search_results_to_ipc(&[r]).expect("tool IPC should serialize");
    assert!(!bytes.is_empty());

    let mut reader = StreamReader::try_new(std::io::Cursor::new(bytes), None)
        .expect("failed to read IPC stream");
    let batch = reader
        .next()
        .expect("missing first record batch")
        .expect("failed to decode record batch");

    let final_scores = batch
        .column_by_name("final_score")
        .expect("missing final_score column")
        .as_any()
        .downcast_ref::<Float32Array>()
        .expect("final_score type mismatch");
    let confidences = batch
        .column_by_name("confidence")
        .expect("missing confidence column")
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("confidence type mismatch");
    let ranking_reasons = batch
        .column_by_name("ranking_reason")
        .expect("missing ranking_reason column")
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("ranking_reason type mismatch");
    let digests = batch
        .column_by_name("input_schema_digest")
        .expect("missing input_schema_digest column")
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("input_schema_digest type mismatch");

    assert!(final_scores.value(0) > 0.0);
    assert!(matches!(confidences.value(0), "high" | "medium" | "low"));
    assert!(ranking_reasons.value(0).contains("final="));
    assert!(digests.value(0).starts_with("sha256:"));
}
