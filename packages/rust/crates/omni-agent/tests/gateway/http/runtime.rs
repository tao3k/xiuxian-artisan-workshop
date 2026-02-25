#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use axum::http::StatusCode;
use std::net::TcpListener;

use crate::config::RuntimeSettings;
use crate::gateway::http::runtime::{
    build_mistral_server_config, resolve_embed_base_url, resolve_embed_model,
    resolve_runtime_embed_base_url, should_auto_start_mistral,
};

#[test]
fn resolve_embed_model_prefers_configured_default_over_request_override() {
    let resolved = resolve_embed_model(
        Some("openai/qwen3-embedding:0.6b"),
        Some("ollama/qwen3-embedding:0.6b"),
    )
    .expect("expected configured default model");
    assert_eq!(resolved, "ollama/qwen3-embedding:0.6b");
}

#[test]
fn resolve_embed_model_uses_requested_when_default_missing() {
    let resolved = resolve_embed_model(Some("openai/text-embedding-3-small"), None)
        .expect("expected request model when no configured default exists");
    assert_eq!(resolved, "openai/text-embedding-3-small");
}

#[test]
fn resolve_embed_model_rejects_when_both_request_and_default_are_missing() {
    let error = resolve_embed_model(None, None).expect_err("expected missing model error");
    assert_eq!(error.0, StatusCode::BAD_REQUEST);
    assert!(error.1.contains("embedding model must be provided"));
}

#[test]
fn resolve_embed_base_url_prefers_litellm_api_base_for_litellm_backend() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_base_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());
    settings.embedding.client_url = Some("http://127.0.0.1:3900".to_string());

    let resolved = resolve_embed_base_url(&settings, Some("litellm_rs"));
    assert_eq!(resolved, "http://127.0.0.1:11434");
}

#[test]
fn resolve_embed_base_url_prefers_memory_base_url_for_http_backend() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_base_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());
    settings.embedding.client_url = Some("http://127.0.0.1:3900".to_string());

    let resolved = resolve_embed_base_url(&settings, Some("http"));
    assert_eq!(resolved, "http://127.0.0.1:3002");
}

#[test]
fn should_auto_start_mistral_requires_enabled_auto_start_and_mistral_backend() {
    let mut settings = RuntimeSettings::default();
    settings.mistral.enabled = Some(true);
    settings.mistral.auto_start = Some(true);

    assert!(should_auto_start_mistral(&settings, Some("mistral_server")));
    assert!(!should_auto_start_mistral(&settings, Some("litellm_rs")));
}

#[test]
fn build_mistral_server_config_prefers_settings_values() {
    let mut settings = RuntimeSettings::default();
    settings.mistral.command = Some("/usr/local/bin/mistralrs-server".to_string());
    settings.mistral.args = Some(vec![
        "--port".to_string(),
        "11500".to_string(),
        "".to_string(),
    ]);
    settings.mistral.base_url = Some("http://127.0.0.1:11500".to_string());
    settings.mistral.startup_timeout_secs = Some(120);
    settings.mistral.probe_timeout_ms = Some(2_500);
    settings.mistral.probe_interval_ms = Some(400);

    let config = build_mistral_server_config(&settings);
    assert_eq!(config.command, "/usr/local/bin/mistralrs-server");
    assert_eq!(config.args, vec!["--port".to_string(), "11500".to_string()]);
    assert_eq!(config.base_url, "http://127.0.0.1:11500");
    assert_eq!(config.startup_timeout_secs, 120);
    assert_eq!(config.probe_timeout_ms, 2_500);
    assert_eq!(config.probe_interval_ms, 400);
}

#[test]
fn resolve_runtime_embed_base_url_ignores_mistral_base_url_for_non_mistral_backend() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_base_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());
    settings.mistral.base_url = Some("http://127.0.0.1:11500".to_string());

    let resolved = resolve_runtime_embed_base_url(&settings, Some("litellm_rs"), None);
    assert_eq!(resolved, "http://127.0.0.1:11434");
}

fn reserve_local_port() -> Option<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").ok()?;
    Some(listener.local_addr().ok()?.port())
}

#[tokio::test]
async fn build_embedding_runtime_for_settings_autostarts_mistral_and_embeds() {
    let Some(port) = reserve_local_port() else {
        eprintln!("skipping test: local socket bind is not permitted");
        return;
    };

    let server_script = r#"
import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

port = int(sys.argv[1])

class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        return

    def _write_json(self, payload):
        body = json.dumps(payload).encode("utf-8")
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path == "/v1/models":
            self._write_json(
                {
                    "object": "list",
                    "data": [{"id": "mistral-embed-small", "object": "model"}],
                }
            )
            return
        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        if self.path != "/v1/embeddings":
            self.send_response(404)
            self.end_headers()
            return

        content_length = int(self.headers.get("content-length", "0"))
        raw = self.rfile.read(content_length).decode("utf-8") if content_length else "{}"
        payload = json.loads(raw or "{}")
        model = payload.get("model", "mistral-embed-small")
        raw_input = payload.get("input", [])
        if isinstance(raw_input, str):
            inputs = [raw_input]
        elif isinstance(raw_input, list):
            inputs = raw_input
        else:
            inputs = []
        data = [
            {"object": "embedding", "index": idx, "embedding": [0.11, 0.22, 0.33]}
            for idx, _ in enumerate(inputs)
        ]
        self._write_json(
            {
                "object": "list",
                "data": data,
                "model": model,
                "usage": {"prompt_tokens": 0, "total_tokens": 0},
            }
        )

HTTPServer(("127.0.0.1", port), Handler).serve_forever()
"#;

    let base_url = format!("http://127.0.0.1:{port}");
    let mut runtime_settings = RuntimeSettings::default();
    runtime_settings.memory.embedding_backend = Some("mistral_server".to_string());
    runtime_settings.memory.embedding_model = Some("mistral-embed-small".to_string());
    runtime_settings.embedding.timeout_secs = Some(3);
    runtime_settings.mistral.enabled = Some(true);
    runtime_settings.mistral.auto_start = Some(true);
    runtime_settings.mistral.command = Some("python3".to_string());
    runtime_settings.mistral.args = Some(vec![
        "-u".to_string(),
        "-c".to_string(),
        server_script.to_string(),
        port.to_string(),
    ]);
    runtime_settings.mistral.base_url = Some(base_url.clone());
    runtime_settings.mistral.startup_timeout_secs = Some(10);
    runtime_settings.mistral.probe_timeout_ms = Some(500);
    runtime_settings.mistral.probe_interval_ms = Some(50);

    let runtime = super::build_embedding_runtime_for_settings(runtime_settings).await;
    assert!(
        runtime.managed_mistral_server.is_some(),
        "expected managed mistral server to be started"
    );

    let texts = vec!["gateway mistral autostart test".to_string()];
    let vectors = runtime
        .client
        .embed_batch_with_model(&texts, Some("mistral-embed-small"))
        .await
        .expect("expected embeddings from managed mistral runtime");

    assert_eq!(vectors.len(), 1);
    assert_eq!(vectors[0].len(), 3);
    assert!((vectors[0][0] - 0.11).abs() < 1e-6);
    assert!((vectors[0][1] - 0.22).abs() < 1e-6);
    assert!((vectors[0][2] - 0.33).abs() < 1e-6);
}
