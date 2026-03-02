//! Agent memory-gate flow tests under pressure and recall gating conditions.

use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use axum::{Json, Router, extract::State, routing::post};
use omni_agent::{Agent, AgentConfig, MemoryConfig, SessionStore};
use omni_memory::{Episode, MemoryGatePolicy, MemoryGateVerdict, MemoryUtilityLedger};
use tokio::time::{Duration, sleep};

fn base_agent_config(memory: MemoryConfig) -> AgentConfig {
    AgentConfig {
        inference_url: "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        model: "test-model".to_string(),
        memory: Some(memory),
        ..AgentConfig::default()
    }
}

fn state_paths(memory_path: &str, table_name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root = Path::new(memory_path);
    (
        root.join(format!("{table_name}.episodes.json")),
        root.join(format!("{table_name}.q_table.json")),
    )
}

async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe_result = tokio::net::TcpListener::bind("127.0.0.1:0").await;
    let probe = match probe_result {
        Ok(listener) => listener,
        Err(error) => panic!("reserve local addr: {error}"),
    };
    let addr = match probe.local_addr() {
        Ok(addr) => addr,
        Err(error) => panic!("read reserved local addr: {error}"),
    };
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
        .map_or(1, Vec::len);
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
    let listener_result = tokio::net::TcpListener::bind(addr).await;
    let listener = match listener_result {
        Ok(listener) => listener,
        Err(error) => panic!("bind embedding listener: {error}"),
    };
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    })
}

async fn with_local_embedding_server(
    mut memory: MemoryConfig,
) -> (MemoryConfig, tokio::task::JoinHandle<()>) {
    let addr = reserve_local_addr().await;
    let handle = spawn_embedding_server(addr, memory.embedding_dim).await;
    memory.embedding_base_url = Some(format!("http://{addr}"));
    (memory, handle)
}

fn read_episodes(path: &Path) -> Vec<Episode> {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) => panic!("episodes snapshot should exist: {error}"),
    };
    match serde_json::from_str(&raw) {
        Ok(episodes) => episodes,
        Err(error) => panic!("episodes snapshot should be valid json: {error}"),
    }
}

fn read_q_table(path: &Path) -> HashMap<String, f32> {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) => panic!("q-table snapshot should exist: {error}"),
    };
    match serde_json::from_str(&raw) {
        Ok(values) => values,
        Err(error) => panic!("q-table snapshot should be valid json: {error}"),
    }
}

fn has_metric_key_prefix(metrics: &HashMap<String, String>, prefix: &str) -> bool {
    metrics.keys().any(|key| key.starts_with(prefix))
}

fn live_redis_url() -> Option<String> {
    for key in ["VALKEY_URL"] {
        if let Ok(url) = std::env::var(key)
            && !url.trim().is_empty()
        {
            return Some(url);
        }
    }
    None
}

fn unique_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}-{nanos}")
}

async fn build_agent_with_shared_redis(
    memory: MemoryConfig,
    redis_url: &str,
    key_prefix: &str,
) -> Result<Agent> {
    let config = base_agent_config(memory);
    let session = SessionStore::new_with_redis(
        redis_url.to_string(),
        Some(key_prefix.to_string()),
        Some(120),
    )?;
    Agent::from_config_with_session_backends_for_test(config, session, None).await
}

async fn stream_metrics(
    redis_url: &str,
    key_prefix: &str,
    stream_name: &str,
    session_id: Option<&str>,
) -> Result<HashMap<String, String>> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    let key = match session_id {
        Some(id) if !id.trim().is_empty() => {
            format!("{key_prefix}:metrics:{stream_name}:session:{}", id.trim())
        }
        _ => format!("{key_prefix}:metrics:{stream_name}"),
    };
    let metrics: HashMap<String, String> = redis::cmd("HGETALL")
        .arg(key)
        .query_async(&mut conn)
        .await?;
    Ok(metrics)
}

async fn wait_for_key_xlen(redis_url: &str, key: &str, min_len: usize) -> Result<usize> {
    let client = redis::Client::open(redis_url)?;
    for _ in 0..40 {
        let mut conn = client.get_multiplexed_async_connection().await?;
        let len: usize = redis::cmd("XLEN").arg(key).query_async(&mut conn).await?;
        if len >= min_len {
            return Ok(len);
        }
        sleep(Duration::from_millis(100)).await;
    }
    let mut conn = client.get_multiplexed_async_connection().await?;
    let len: usize = redis::cmd("XLEN").arg(key).query_async(&mut conn).await?;
    Ok(len)
}

async fn wait_for_key_hlen(redis_url: &str, key: &str, min_len: usize) -> Result<usize> {
    let client = redis::Client::open(redis_url)?;
    for _ in 0..40 {
        let mut conn = client.get_multiplexed_async_connection().await?;
        let len: usize = redis::cmd("HLEN").arg(key).query_async(&mut conn).await?;
        if len >= min_len {
            return Ok(len);
        }
        sleep(Duration::from_millis(100)).await;
    }
    let mut conn = client.get_multiplexed_async_connection().await?;
    let len: usize = redis::cmd("HLEN").arg(key).query_async(&mut conn).await?;
    Ok(len)
}

async fn append_memory_turns(
    agent: &Agent,
    session_id: &str,
    user_message: &str,
    assistant_message: &str,
    tool_count: u32,
    repeats: usize,
) -> Result<()> {
    for _ in 0..repeats {
        agent
            .append_turn_with_tool_count_for_session(
                session_id,
                user_message,
                assistant_message,
                tool_count,
            )
            .await?;
    }
    Ok(())
}

fn metric_count(metrics: &HashMap<String, String>, key: &str) -> u64 {
    metrics
        .get(key)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
}

fn assert_global_memory_gate_stream_metrics(global_metrics: &HashMap<String, String>) {
    assert_eq!(
        global_metrics
            .get("kind:memory_gate_event")
            .map(String::as_str),
        Some("2"),
        "memory gate evaluation should emit one stream event per turn"
    );
    assert!(
        has_metric_key_prefix(global_metrics, "react_evidence_count:"),
        "memory gate stream events should include react evidence counts"
    );
    assert!(
        has_metric_key_prefix(global_metrics, "graph_evidence_count:"),
        "memory gate stream events should include graph evidence counts"
    );
    assert!(
        has_metric_key_prefix(global_metrics, "omega_factor_count:"),
        "memory gate stream events should include omega factor counts"
    );
    assert!(
        has_metric_key_prefix(global_metrics, "react_evidence_refs:"),
        "memory gate stream events should include react evidence references"
    );
    assert!(
        has_metric_key_prefix(global_metrics, "graph_evidence_refs:"),
        "memory gate stream events should include graph evidence references"
    );
    assert!(
        has_metric_key_prefix(global_metrics, "omega_factors:"),
        "memory gate stream events should include omega factors"
    );
    assert_eq!(
        global_metrics.get("kind:turn_stored").map(String::as_str),
        Some("6"),
        "turn store events should remain observable for memory gate debugging"
    );
    assert!(
        metric_count(global_metrics, "kind:memory_promoted") >= 1,
        "promoted memory events should be emitted for durable knowledge ingestion"
    );
}

fn assert_scoped_memory_gate_stream_metrics(
    scoped_metrics: &HashMap<String, String>,
    promoted_scoped_metrics: &HashMap<String, String>,
) {
    assert_eq!(
        scoped_metrics
            .get("kind:memory_gate_event")
            .map(String::as_str),
        Some("2")
    );
    assert_eq!(
        scoped_metrics.get("kind:turn_stored").map(String::as_str),
        Some("2")
    );
    assert!(
        metric_count(promoted_scoped_metrics, "kind:memory_promoted") >= 1,
        "promoted session should emit at least one memory_promoted stream record"
    );
}

async fn assert_ingest_candidate_keys(redis_url: &str, key_prefix: &str) -> Result<()> {
    let ingest_stream_key = format!("{key_prefix}:stream:knowledge.ingest.candidates");
    let ingest_ledger_key = format!("{key_prefix}:knowledge:ingest:candidates");
    let queued_candidates = wait_for_key_xlen(redis_url, &ingest_stream_key, 1).await?;
    assert!(
        queued_candidates >= 1,
        "promoted memory should be queued into durable knowledge ingest stream"
    );
    let ledger_candidates = wait_for_key_hlen(redis_url, &ingest_ledger_key, 1).await?;
    assert!(
        ledger_candidates >= 1,
        "promoted memory should be deduplicated in ingest ledger"
    );
    Ok(())
}

#[tokio::test]
async fn repeated_success_turns_reuse_episode_and_reach_promote_threshold() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let table_name = "memory_gate_promote".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let (memory, embedding_server_handle) = with_local_embedding_server(memory).await;
    let expected_gate_promote_threshold = memory.gate_promote_threshold;
    let expected_gate_obsolete_threshold = memory.gate_obsolete_threshold;
    let expected_gate_promote_min_usage = memory.gate_promote_min_usage;
    let expected_gate_obsolete_min_usage = memory.gate_obsolete_min_usage;
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = Agent::from_config(base_agent_config(memory)).await?;

    let session_id = "memory-gate-promote-session";
    for _ in 0..4 {
        agent
            .append_turn_with_tool_count_for_session(
                session_id,
                "compare valkey and postgres tradeoffs",
                "analysis completed successfully",
                6,
            )
            .await?;
    }

    let status = agent.inspect_memory_runtime_status();
    assert_eq!(
        status.episodes_total,
        Some(1),
        "same intent in one session should reuse a single episode for stable gate utility"
    );
    assert_eq!(
        status.q_values_total,
        Some(1),
        "reused episode should keep one q-table entry"
    );
    assert_eq!(
        status.gate_promote_threshold,
        Some(expected_gate_promote_threshold)
    );
    assert_eq!(
        status.gate_obsolete_threshold,
        Some(expected_gate_obsolete_threshold)
    );
    assert_eq!(
        status.gate_promote_min_usage,
        Some(expected_gate_promote_min_usage)
    );
    assert_eq!(
        status.gate_obsolete_min_usage,
        Some(expected_gate_obsolete_min_usage)
    );

    let episodes = read_episodes(&episodes_path);
    assert_eq!(episodes.len(), 1);
    let episode = &episodes[0];
    assert_eq!(episode.scope_key(), session_id);
    assert!(episode.success_count >= 4);
    assert!(episode.failure_count == 0);

    let ledger = MemoryUtilityLedger::from_episode(episode, 0.96, 0.64, 0.78);
    let decision = MemoryGatePolicy::default().evaluate(&ledger, vec![], vec![], vec![]);
    assert_eq!(
        decision.verdict,
        MemoryGateVerdict::Promote,
        "reused successful episode should cross promotion threshold"
    );

    let q_values = read_q_table(&q_path);
    assert_eq!(q_values.len(), 1);
    embedding_server_handle.abort();
    let _ = embedding_server_handle.await;
    Ok(())
}

#[tokio::test]
async fn repeated_failure_turns_trigger_obsolete_and_purge_episode() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let table_name = "memory_gate_obsolete".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let (memory, embedding_server_handle) = with_local_embedding_server(memory).await;
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = Agent::from_config(base_agent_config(memory)).await?;

    let session_id = "memory-gate-obsolete-session";
    let user_intent = "sync valuecell repo";
    let assistant_failure = "error: timed out while fetching remote";

    agent
        .append_turn_with_tool_count_for_session(session_id, user_intent, assistant_failure, 0)
        .await?;
    let after_first = agent.inspect_memory_runtime_status();
    assert_eq!(after_first.episodes_total, Some(1));
    assert_eq!(after_first.q_values_total, Some(1));

    agent
        .append_turn_with_tool_count_for_session(session_id, user_intent, assistant_failure, 0)
        .await?;
    let after_second = agent.inspect_memory_runtime_status();
    assert_eq!(
        after_second.episodes_total,
        Some(0),
        "gate obsolete decision should purge repeatedly failing episode"
    );
    assert_eq!(
        after_second.q_values_total,
        Some(0),
        "purged episode should also remove q-table entry"
    );

    let episodes = read_episodes(&episodes_path);
    assert!(
        episodes.is_empty(),
        "persisted episodes should be empty after purge"
    );
    let q_values = read_q_table(&q_path);
    assert!(
        q_values.is_empty(),
        "persisted q-table should be empty after purge"
    );
    embedding_server_handle.abort();
    let _ = embedding_server_handle.await;
    Ok(())
}

#[tokio::test]
async fn custom_gate_policy_can_purge_after_single_failure_turn() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let table_name = "memory_gate_custom_single_failure_purge".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "local".to_string(),
        gate_obsolete_threshold: 1.0,
        gate_obsolete_min_usage: 1,
        gate_obsolete_failure_rate_floor: 0.0,
        gate_obsolete_max_ttl_score: 1.0,
        ..MemoryConfig::default()
    };
    let (memory, embedding_server_handle) = with_local_embedding_server(memory).await;
    let agent = Agent::from_config(base_agent_config(memory)).await?;

    agent
        .append_turn_with_tool_count_for_session(
            "memory-gate-custom-single-failure",
            "investigate flaky webhook timeout",
            "error: upstream request timed out",
            0,
        )
        .await?;

    let status = agent.inspect_memory_runtime_status();
    assert_eq!(
        status.episodes_total,
        Some(0),
        "custom gate policy should allow obsolete purge after first failure turn"
    );
    assert_eq!(status.q_values_total, Some(0));
    assert_eq!(status.gate_obsolete_threshold, Some(1.0));
    assert_eq!(status.gate_obsolete_min_usage, Some(1));
    assert_eq!(status.gate_obsolete_failure_rate_floor, Some(0.0));
    assert_eq!(status.gate_obsolete_max_ttl_score, Some(1.0));
    embedding_server_handle.abort();
    let _ = embedding_server_handle.await;
    Ok(())
}

#[tokio::test]
async fn custom_gate_policy_can_delay_obsolete_after_repeated_failures() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let table_name = "memory_gate_custom_delay_obsolete".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "local".to_string(),
        gate_obsolete_min_usage: 8,
        ..MemoryConfig::default()
    };
    let (memory, embedding_server_handle) = with_local_embedding_server(memory).await;
    let agent = Agent::from_config(base_agent_config(memory)).await?;

    let session_id = "memory-gate-custom-delay";
    for _ in 0..2 {
        agent
            .append_turn_with_tool_count_for_session(
                session_id,
                "sync indexer snapshots",
                "error: embedding service unavailable",
                0,
            )
            .await?;
    }

    let status = agent.inspect_memory_runtime_status();
    assert_eq!(
        status.episodes_total,
        Some(1),
        "high obsolete_min_usage should keep failing episode until enough evidence accumulates"
    );
    assert_eq!(status.q_values_total, Some(1));
    embedding_server_handle.abort();
    let _ = embedding_server_handle.await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires live valkey server"]
async fn memory_gate_events_are_emitted_into_valkey_stream_metrics() -> Result<()> {
    let Some(redis_url) = live_redis_url() else {
        eprintln!("skip: set VALKEY_URL");
        return Ok(());
    };

    let temp_dir = tempfile::tempdir()?;
    let table_name = unique_id("memory_gate_stream");
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let (memory, embedding_server_handle) = with_local_embedding_server(memory).await;

    let key_prefix = unique_id("memory-gate-stream");
    let session_id = unique_id("memory-gate-stream-session");
    let agent = build_agent_with_shared_redis(memory, &redis_url, &key_prefix).await?;

    append_memory_turns(
        &agent,
        &session_id,
        "retry flaky pipeline",
        "error: timed out while calling upstream",
        0,
        2,
    )
    .await?;

    let promote_session_id = unique_id("memory-gate-promote-stream-session");
    append_memory_turns(
        &agent,
        &promote_session_id,
        "compare valkey and postgres tradeoffs",
        "analysis completed successfully",
        6,
        4,
    )
    .await?;

    let global_metrics = stream_metrics(&redis_url, &key_prefix, "memory.events", None).await?;
    assert_global_memory_gate_stream_metrics(&global_metrics);

    let scoped_metrics =
        stream_metrics(&redis_url, &key_prefix, "memory.events", Some(&session_id)).await?;
    let promoted_scoped_metrics = stream_metrics(
        &redis_url,
        &key_prefix,
        "memory.events",
        Some(&promote_session_id),
    )
    .await?;
    assert_scoped_memory_gate_stream_metrics(&scoped_metrics, &promoted_scoped_metrics);

    assert_ingest_candidate_keys(&redis_url, &key_prefix).await?;
    embedding_server_handle.abort();
    let _ = embedding_server_handle.await;
    Ok(())
}
