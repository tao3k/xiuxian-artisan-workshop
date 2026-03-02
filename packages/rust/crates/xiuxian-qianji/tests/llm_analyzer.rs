//! Integration tests for structured LLM analyzer output handling.

#![cfg(feature = "llm")]

use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use xiuxian_llm::llm::{ChatRequest, LlmClient};
use xiuxian_qianji::contracts::{FlowInstruction, QianjiMechanism};
use xiuxian_qianji::executors::llm::LlmAnalyzer;

struct MockLlmClient {
    response: String,
    seen_model: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, request: ChatRequest) -> Result<String> {
        if let Ok(mut guard) = self.seen_model.lock() {
            *guard = Some(request.model);
        }
        Ok(self.response.clone())
    }
}

struct PromptCaptureLlmClient {
    seen_system_prompt: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl LlmClient for PromptCaptureLlmClient {
    async fn chat(&self, request: ChatRequest) -> Result<String> {
        let prompt = request
            .messages
            .first()
            .map(|message| message.content.clone())
            .unwrap_or_default();
        if let Ok(mut guard) = self.seen_system_prompt.lock() {
            *guard = Some(prompt);
        }
        Ok("ok".to_string())
    }
}

fn must_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

fn must_array<'a>(value: &'a serde_json::Value, context: &str) -> &'a [serde_json::Value] {
    value
        .as_array()
        .map_or_else(|| panic!("{context}"), std::vec::Vec::as_slice)
}

fn make_analyzer(
    response: &str,
    parse_json_output: bool,
    fallback_repo_tree: bool,
) -> (LlmAnalyzer, Arc<Mutex<Option<String>>>) {
    let seen_model = Arc::new(Mutex::new(None));
    let analyzer = LlmAnalyzer {
        client: Arc::new(MockLlmClient {
            response: response.to_string(),
            seen_model: Arc::clone(&seen_model),
        }),
        model: "test-model".to_string(),
        context_keys: vec!["repo_tree".to_string()],
        prompt_template: "Analyze the tree".to_string(),
        output_key: "analysis_trace".to_string(),
        parse_json_output,
        fallback_repo_tree_on_parse_failure: fallback_repo_tree,
    };
    (analyzer, seen_model)
}

#[tokio::test]
async fn llm_analyzer_parses_json_output_when_enabled() {
    let (analyzer, _) = make_analyzer(
        "```json\n[{\"shard_id\":\"core\",\"paths\":[\"src\"]}]\n```",
        true,
        false,
    );
    let context = serde_json::json!({
        "request": "plan shards",
        "repo_tree": "./src\n./tests\n",
    });

    let output = must_ok(analyzer.execute(&context).await, "execute should succeed");
    assert_eq!(output.data["analysis_trace"][0]["shard_id"], "core");
    assert_eq!(output.data["analysis_trace"][0]["paths"][0], "src");
    assert_eq!(
        output.data["analysis_trace_raw"].as_str().unwrap_or(""),
        "```json\n[{\"shard_id\":\"core\",\"paths\":[\"src\"]}]\n```"
    );
    match output.instruction {
        FlowInstruction::Continue => {}
        _ => panic!("expected continue"),
    }
}

#[tokio::test]
async fn llm_analyzer_uses_repo_tree_fallback_when_json_parse_fails() {
    let (analyzer, _) = make_analyzer("non-json content", true, true);
    let context = serde_json::json!({
        "request": "plan shards",
        "repo_tree": "./src\n./tests\n./README.md\n./src/core\n",
    });

    let output = must_ok(analyzer.execute(&context).await, "execute should succeed");
    assert_eq!(
        output.data["analysis_trace"][0]["shard_id"],
        "repository-overview"
    );
    let paths = must_array(
        &output.data["analysis_trace"][0]["paths"],
        "paths should be array",
    );
    assert!(paths.iter().any(|v| v.as_str() == Some("src")));
    assert!(paths.iter().any(|v| v.as_str() == Some("tests")));
}

#[tokio::test]
async fn llm_analyzer_keeps_text_mode_when_json_parse_disabled() {
    let (analyzer, _) = make_analyzer("plain text answer", false, false);
    let context = serde_json::json!({
        "request": "plan shards",
        "repo_tree": "./src\n",
    });

    let output = must_ok(analyzer.execute(&context).await, "execute should succeed");
    assert_eq!(output.data["analysis_trace"], "plain text answer");
}

#[tokio::test]
async fn llm_analyzer_uses_llm_model_from_context_when_present() {
    let (analyzer, seen_model) = make_analyzer("plain text answer", false, false);
    let context = serde_json::json!({
        "request": "plan shards",
        "repo_tree": "./src\n",
        "llm_model": "context-model",
    });

    let output = must_ok(analyzer.execute(&context).await, "execute should succeed");
    assert_eq!(output.data["analysis_trace"], "plain text answer");

    let model = seen_model
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_default();
    assert_eq!(model, "context-model");
}

#[tokio::test]
async fn llm_analyzer_uses_fallback_model_when_default_is_empty() {
    let seen_model = Arc::new(Mutex::new(None));
    let analyzer = LlmAnalyzer {
        client: Arc::new(MockLlmClient {
            response: "ok".to_string(),
            seen_model: Arc::clone(&seen_model),
        }),
        model: String::new(),
        context_keys: vec!["repo_tree".to_string()],
        prompt_template: "Analyze the tree".to_string(),
        output_key: "analysis_trace".to_string(),
        parse_json_output: false,
        fallback_repo_tree_on_parse_failure: false,
    };

    let context = serde_json::json!({
        "request": "plan shards",
        "repo_tree": "./src\n",
        "llm_model_fallback": "runtime-fallback-model",
    });

    let output = must_ok(analyzer.execute(&context).await, "execute should succeed");
    assert_eq!(output.data["analysis_trace"], "ok");

    let model = seen_model
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_default();
    assert_eq!(model, "runtime-fallback-model");
}

#[tokio::test]
async fn llm_analyzer_resolves_semantic_prompt_template_uri() {
    let seen_prompt = Arc::new(Mutex::new(None));
    let analyzer = LlmAnalyzer {
        client: Arc::new(PromptCaptureLlmClient {
            seen_system_prompt: Arc::clone(&seen_prompt),
        }),
        model: "test-model".to_string(),
        context_keys: Vec::new(),
        prompt_template: "$wendao://skills/agenda-management/references/prompts/classifier.md"
            .to_string(),
        output_key: "analysis_trace".to_string(),
        parse_json_output: false,
        fallback_repo_tree_on_parse_failure: false,
    };
    let context = serde_json::json!({
        "request": "classify this turn",
    });

    let output = must_ok(analyzer.execute(&context).await, "execute should succeed");
    assert_eq!(output.data["analysis_trace"], "ok");

    let prompt = seen_prompt
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_default();
    assert!(
        prompt.contains("agenda-validation preflight classifier"),
        "semantic URI prompt should be resolved before LLM call"
    );
}
