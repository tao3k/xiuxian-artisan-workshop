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

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use axum::{Json, Router, extract::State, routing::post};
use omni_agent::{Agent, AgentConfig, MemoryConfig};

const SESSION_A: &str = "telegram:-200:1001";
const SESSION_B: &str = "telegram:-200:1002";
const TELEGRAM_SHARED_SUFFIX_SESSION: &str = "telegram:-200:4001";
const DISCORD_SHARED_SUFFIX_SESSION: &str = "discord:-200:4001";

fn build_agent_config(memory: MemoryConfig) -> AgentConfig {
    AgentConfig {
        inference_url: "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        model: "test-model".to_string(),
        memory: Some(memory),
        ..AgentConfig::default()
    }
}

fn episodes_path(memory_path: &str, table_name: &str) -> String {
    Path::new(memory_path)
        .join(format!("{table_name}.episodes.json"))
        .to_string_lossy()
        .to_string()
}

async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve local addr");
    let addr = probe.local_addr().expect("read reserved local addr");
    drop(probe);
    addr
}

async fn embed_handler(
    State(embedding_dim): State<usize>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let vector_count = payload
        .get("texts")
        .and_then(|value| value.as_array())
        .map_or(1, |texts| texts.len());
    tokio::time::sleep(Duration::from_millis(1)).await;
    let vectors: Vec<Vec<f32>> = (0..vector_count)
        .map(|_| vec![0.0_f32; embedding_dim])
        .collect();
    Json(serde_json::json!({ "vectors": vectors }))
}

async fn spawn_embedding_server(
    addr: std::net::SocketAddr,
    embedding_dim: usize,
) -> tokio::task::JoinHandle<()> {
    let app = Router::new()
        .route("/embed/batch", post(embed_handler))
        .with_state(embedding_dim);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind embedding listener");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    })
}

#[tokio::test]
async fn turn_memory_persistence_is_scoped_by_session_id() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let mut memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name: "scope_isolation".to_string(),
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let addr = reserve_local_addr().await;
    let embedding_server_handle = spawn_embedding_server(addr, memory.embedding_dim).await;
    memory.embedding_base_url = Some(format!("http://{addr}"));
    let file_path = episodes_path(&memory.path, &memory.table_name);

    let agent = Agent::from_config(build_agent_config(memory)).await?;
    agent
        .append_turn_for_session(SESSION_A, "session A question", "session A answer")
        .await?;
    agent
        .append_turn_for_session(SESSION_B, "session B question", "session B answer")
        .await?;

    let raw = fs::read_to_string(&file_path)?;
    let payload: serde_json::Value = serde_json::from_str(&raw)?;
    let episodes = payload
        .as_array()
        .expect("episodes persistence payload should be an array");
    assert!(
        episodes.len() >= 2,
        "expected at least two stored episodes, got {}",
        episodes.len()
    );

    let mut scopes: HashSet<String> = HashSet::new();
    for episode in episodes {
        let scope = episode
            .get("scope")
            .and_then(serde_json::Value::as_str)
            .expect("every persisted episode should contain scope")
            .to_string();
        scopes.insert(scope);
    }

    assert!(
        scopes.contains(SESSION_A),
        "session A scope should exist in persisted episodes"
    );
    assert!(
        scopes.contains(SESSION_B),
        "session B scope should exist in persisted episodes"
    );

    embedding_server_handle.abort();
    let _ = embedding_server_handle.await;
    Ok(())
}

#[tokio::test]
async fn turn_memory_persistence_isolated_when_channel_prefix_differs() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let mut memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name: "scope_channel_isolation".to_string(),
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let addr = reserve_local_addr().await;
    let embedding_server_handle = spawn_embedding_server(addr, memory.embedding_dim).await;
    memory.embedding_base_url = Some(format!("http://{addr}"));
    let file_path = episodes_path(&memory.path, &memory.table_name);

    let agent = Agent::from_config(build_agent_config(memory)).await?;
    let prompt = "same prompt for cross-channel scope isolation";
    let reply = "same answer for cross-channel scope isolation";
    agent
        .append_turn_for_session(TELEGRAM_SHARED_SUFFIX_SESSION, prompt, reply)
        .await?;
    agent
        .append_turn_for_session(DISCORD_SHARED_SUFFIX_SESSION, prompt, reply)
        .await?;

    let raw = fs::read_to_string(&file_path)?;
    let payload: serde_json::Value = serde_json::from_str(&raw)?;
    let episodes = payload
        .as_array()
        .expect("episodes persistence payload should be an array");
    assert!(
        episodes.len() >= 2,
        "expected at least two episodes for distinct channel-scoped sessions, got {}",
        episodes.len()
    );

    let mut scopes: HashSet<String> = HashSet::new();
    for episode in episodes {
        let scope = episode
            .get("scope")
            .and_then(serde_json::Value::as_str)
            .expect("every persisted episode should contain scope")
            .to_string();
        scopes.insert(scope);
    }

    assert!(
        scopes.contains(TELEGRAM_SHARED_SUFFIX_SESSION),
        "telegram-scoped episode should exist in persisted episodes"
    );
    assert!(
        scopes.contains(DISCORD_SHARED_SUFFIX_SESSION),
        "discord-scoped episode should exist in persisted episodes"
    );

    embedding_server_handle.abort();
    let _ = embedding_server_handle.await;
    Ok(())
}
