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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use axum::{Json, Router, extract::State, routing::post};
use omni_agent::{Agent, AgentConfig, MemoryConfig, set_config_home_override};

fn base_agent_config(memory: MemoryConfig) -> AgentConfig {
    AgentConfig {
        inference_url: "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        model: "test-model".to_string(),
        memory: Some(memory),
        ..AgentConfig::default()
    }
}

fn ensure_test_config_home_override() {
    static CONFIG_HOME: OnceLock<PathBuf> = OnceLock::new();
    let path = CONFIG_HOME.get_or_init(|| {
        let root = std::env::temp_dir()
            .join("omni-agent-tests")
            .join("agent_memory_persistence_backend");
        std::fs::create_dir_all(&root).expect("create isolated config home for tests");
        root
    });
    set_config_home_override(path.clone());
}

async fn build_agent_with_optional_session_valkey_url(
    mut memory: MemoryConfig,
    session_valkey_url: Option<&str>,
) -> anyhow::Result<Agent> {
    // Isolate from developer-local ~/.config or PRJ_CONFIG_HOME overrides.
    ensure_test_config_home_override();
    if let Some(url) = session_valkey_url {
        memory.persistence_valkey_url = Some(url.to_string());
    }
    let config = base_agent_config(memory);
    Agent::from_config(config).await
}

fn state_paths(memory_path: &str, table_name: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(memory_path);
    (
        root.join(format!("{table_name}.episodes.json")),
        root.join(format!("{table_name}.q_table.json")),
    )
}

async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve local addr");
    let addr = probe.local_addr().expect("read reserved local addr");
    drop(probe);
    addr
}

async fn slow_embed_handler(
    State((sleep_ms, embedding_dim)): State<(u64, usize)>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let vector_count = payload
        .get("texts")
        .and_then(|value| value.as_array())
        .map_or(1, |texts| texts.len());
    tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
    let vectors: Vec<Vec<f32>> = (0..vector_count)
        .map(|_| vec![0.0_f32; embedding_dim])
        .collect();
    Json(serde_json::json!({ "vectors": vectors }))
}

async fn spawn_slow_embedding_server(
    addr: std::net::SocketAddr,
    sleep_ms: u64,
    embedding_dim: usize,
) -> tokio::task::JoinHandle<()> {
    let app = Router::new()
        .route("/embed/batch", post(slow_embed_handler))
        .with_state((sleep_ms, embedding_dim));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind slow embedding listener");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    })
}

#[tokio::test]
async fn local_memory_backend_initializes_without_valkey() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let agent = build_agent_with_optional_session_valkey_url(memory, None).await;
    assert!(
        agent.is_ok(),
        "local memory backend should initialize without valkey"
    );
}

#[tokio::test]
async fn strict_valkey_memory_backend_fails_when_unreachable() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        persistence_backend: "valkey".to_string(),
        ..MemoryConfig::default()
    };
    match build_agent_with_optional_session_valkey_url(memory, Some("redis://127.0.0.1:1/0")).await
    {
        Ok(_) => panic!("strict valkey backend should fail when redis is unreachable"),
        Err(err) => {
            assert!(
                err.to_string()
                    .contains("strict valkey memory backend failed during startup"),
                "unexpected error: {err}"
            );
        }
    }
}

#[tokio::test]
async fn auto_memory_backend_without_valkey_url_persists_locally() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let table_name = "auto_local".to_string();
    let mut memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "auto".to_string(),
        ..MemoryConfig::default()
    };
    let addr = reserve_local_addr().await;
    let server_handle = spawn_slow_embedding_server(addr, 0, memory.embedding_dim).await;
    memory.embedding_base_url = Some(format!("http://{addr}"));
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = build_agent_with_optional_session_valkey_url(memory, None)
        .await
        .expect("auto backend without redis url should initialize");

    agent
        .append_turn_for_session("auto-local-session", "u1", "a1")
        .await
        .expect("append turn should succeed");

    assert!(
        episodes_path.exists(),
        "auto backend without redis url should persist local episode snapshot"
    );
    assert!(
        q_path.exists(),
        "auto backend without redis url should persist local q-table snapshot"
    );

    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn auto_memory_backend_with_unreachable_valkey_fails_by_default() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let table_name = "auto_valkey".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "auto".to_string(),
        ..MemoryConfig::default()
    };
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    match build_agent_with_optional_session_valkey_url(memory, Some("redis://127.0.0.1:1/0")).await
    {
        Ok(_) => panic!("auto backend with valkey url should fail startup by default"),
        Err(err) => {
            assert!(
                err.to_string()
                    .contains("strict valkey memory backend failed during startup"),
                "unexpected error: {err}"
            );
        }
    }

    assert!(
        !episodes_path.exists(),
        "failed strict startup should not create local episode snapshot files"
    );
    assert!(
        !q_path.exists(),
        "failed strict startup should not create local q-table snapshot files"
    );
}

#[tokio::test]
async fn auto_memory_backend_can_relax_strict_startup_without_local_fallback() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let table_name = "auto_valkey_relaxed".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "auto".to_string(),
        persistence_strict_startup: Some(false),
        ..MemoryConfig::default()
    };
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = build_agent_with_optional_session_valkey_url(memory, Some("redis://127.0.0.1:1/0"))
        .await
        .expect("auto backend should allow relaxed startup when explicitly configured");

    agent
        .append_turn_for_session("auto-valkey-relaxed-session", "u1", "a1")
        .await
        .expect("append turn should still succeed with relaxed startup");

    assert!(
        !episodes_path.exists(),
        "auto backend with configured valkey should not silently fall back to local episode snapshot"
    );
    assert!(
        !q_path.exists(),
        "auto backend with configured valkey should not silently fall back to local q-table snapshot"
    );
}

#[tokio::test]
async fn auto_memory_backend_with_invalid_valkey_url_fails_fast() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        persistence_backend: "auto".to_string(),
        ..MemoryConfig::default()
    };
    match build_agent_with_optional_session_valkey_url(memory, Some("http://127.0.0.1:6379/0"))
        .await
    {
        Ok(_) => panic!("auto backend should fail when valkey url is invalid"),
        Err(err) => {
            assert!(
                err.to_string()
                    .contains("invalid redis url for memory persistence"),
                "unexpected error: {err}"
            );
        }
    }
}

#[tokio::test]
async fn memory_turn_store_skips_episode_when_embedding_endpoint_is_unavailable() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let table_name = "embed_endpoint_down".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        embedding_base_url: Some("http://127.0.0.1:3302".to_string()),
        embedding_model: Some("ollama/qwen3-embedding:0.6b".to_string()),
        embedding_dim: 1024,
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = build_agent_with_optional_session_valkey_url(memory, None)
        .await
        .expect("agent should initialize when embedding endpoint is unavailable");

    let started = Instant::now();
    agent
        .append_turn_for_session("embed-unavailable-session", "u1", "a1")
        .await
        .expect("turn append should still succeed when embedding service is unavailable");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "embedding unavailable path should not block turn append unexpectedly"
    );

    assert!(
        episodes_path.exists(),
        "episode snapshot should be created via hash fallback when embedding is unavailable"
    );
    assert!(
        q_path.exists(),
        "q-table snapshot should be created via hash fallback when embedding is unavailable"
    );
    let metrics = agent.inspect_memory_recall_metrics().await;
    assert_eq!(metrics.embedding_success_total, 0);
    assert_eq!(metrics.embedding_unavailable_total, 1);
    assert_eq!(metrics.embedding_timeout_total, 0);
    assert_eq!(metrics.embedding_cooldown_reject_total, 0);
}

#[tokio::test]
async fn memory_turn_store_skips_episode_when_embedding_unavailable_even_with_tools() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let table_name = "embed_endpoint_down_tool_skip".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        embedding_base_url: Some("http://127.0.0.1:3302".to_string()),
        embedding_model: Some("ollama/qwen3-embedding:0.6b".to_string()),
        embedding_dim: 1024,
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = build_agent_with_optional_session_valkey_url(memory, None)
        .await
        .expect("agent should initialize when embedding endpoint is unavailable");

    agent
        .append_turn_with_tool_count_for_session(
            "embed-unavailable-tool-skip-session",
            "u1",
            "analysis completed with fallback",
            2,
        )
        .await
        .expect("turn append should still succeed when embedding is unavailable");

    assert!(
        episodes_path.exists(),
        "episode snapshot should be created via hash fallback when embedding is unavailable"
    );
    assert!(
        q_path.exists(),
        "q-table snapshot should be created via hash fallback when embedding is unavailable"
    );

    let metrics = agent.inspect_memory_recall_metrics().await;
    assert_eq!(metrics.embedding_success_total, 0);
    assert_eq!(metrics.embedding_unavailable_total, 1);
    assert_eq!(metrics.embedding_timeout_total, 0);
    assert_eq!(metrics.embedding_cooldown_reject_total, 0);
}

#[tokio::test]
async fn memory_embedding_timeout_cooldown_skips_repeated_waits() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let table_name = "embed_timeout_cooldown".to_string();
    let embedding_dim = 64;
    let addr = reserve_local_addr().await;
    let server_handle = spawn_slow_embedding_server(addr, 10_000, embedding_dim).await;
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        embedding_base_url: Some(format!("http://{addr}")),
        embedding_dim,
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let agent = build_agent_with_optional_session_valkey_url(memory, None)
        .await
        .expect("agent should initialize with slow embedding endpoint");

    let first_started = Instant::now();
    agent
        .append_turn_for_session("embed-cooldown-session", "first-timeout-intent", "a1")
        .await
        .expect("first turn append should still succeed when embedding times out");
    let first_elapsed = first_started.elapsed();

    let second_started = Instant::now();
    agent
        .append_turn_for_session("embed-cooldown-session", "second-timeout-intent", "a2")
        .await
        .expect("second turn append should still succeed during cooldown reject");
    let second_elapsed = second_started.elapsed();

    assert!(
        first_elapsed >= Duration::from_secs(2),
        "first turn should include embedding timeout wait; elapsed={first_elapsed:?}"
    );
    assert!(
        second_elapsed < Duration::from_secs(1),
        "second turn should bypass embedding wait during cooldown; elapsed={second_elapsed:?}"
    );
    let metrics = agent.inspect_memory_recall_metrics().await;
    assert_eq!(
        metrics.embedding_timeout_total, 1,
        "first turn should record timeout"
    );
    assert_eq!(
        metrics.embedding_cooldown_reject_total, 1,
        "second turn should record cooldown reject"
    );
    assert_eq!(
        metrics.embedding_success_total, 0,
        "slow server should not produce successful embeddings in this scenario"
    );
    assert_eq!(metrics.embedding_unavailable_total, 0);

    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn memory_decay_policy_applies_on_configured_interval() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let table_name = "decay_interval".to_string();
    let mut memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "local".to_string(),
        decay_enabled: true,
        decay_every_turns: 1,
        decay_factor: 0.5,
        ..MemoryConfig::default()
    };
    let addr = reserve_local_addr().await;
    let server_handle = spawn_slow_embedding_server(addr, 0, memory.embedding_dim).await;
    memory.embedding_base_url = Some(format!("http://{addr}"));
    let (_episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = build_agent_with_optional_session_valkey_url(memory, None)
        .await
        .expect("agent should initialize for decay test");

    agent
        .append_turn_for_session("decay-session", "u1", "a1")
        .await
        .expect("append turn should succeed");

    let raw = std::fs::read_to_string(&q_path).expect("q-table snapshot should exist");
    let q_values: HashMap<String, f32> =
        serde_json::from_str(&raw).expect("q-table json should parse");
    assert_eq!(q_values.len(), 1, "expected one q-table entry");
    let q = q_values.values().next().copied().unwrap_or_default();
    assert!(
        q < 0.6,
        "decay should reduce first-turn q value below non-decay baseline (q={q})"
    );

    server_handle.abort();
    let _ = server_handle.await;
}
