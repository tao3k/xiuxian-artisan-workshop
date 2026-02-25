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

use std::path::PathBuf;

use omni_agent::{
    build_discord_acl_overrides, build_telegram_acl_overrides, load_runtime_settings_from_paths,
};
use tempfile::TempDir;

fn write_file(path: PathBuf, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
    std::fs::write(path, content).expect("write yaml");
}

#[test]
fn merge_user_overrides_system() {
    let tmp = TempDir::new().expect("tempdir");
    let system = tmp.path().join("packages/conf/settings.yaml");
    let user = tmp.path().join(".config/omni-dev-fusion/settings.yaml");

    write_file(
        system.clone(),
        r#"
agent:
  llm_backend: "http"
inference:
  provider: "openai"
  api_key_env: "OPENAI_API_KEY"
  base_url: "http://127.0.0.1:4000/v1"
  model: "gpt-4o-mini"
  timeout: 45
  max_tokens: 2048
  max_in_flight: 8
mcp:
  agent_pool_size: 4
  agent_handshake_timeout_secs: 30
  agent_connect_retries: 2
  agent_strict_startup: true
  agent_connect_retry_backoff_ms: 500
  agent_tool_timeout_secs: 120
  agent_list_tools_cache_ttl_ms: 900
  agent_discover_cache_enabled: true
  agent_discover_cache_key_prefix: "system-discover"
  agent_discover_cache_ttl_secs: 45
telegram:
  acl:
    allow:
      users: ["1001"]
      groups: ["-2001"]
    admin:
      users: ["1001"]
    control:
      allow_from:
        users: ["1001", "ops"]
      rules:
        - commands: ["/session partition"]
          allow:
            users: ["1001"]
    slash:
      global:
        users: ["1001", "ops"]
      session_status:
        users: ["observer"]
      session_budget:
        users: ["observer"]
      session_memory:
        users: ["observer"]
      session_feedback:
        users: ["editor"]
      job_status:
        users: ["ops"]
      jobs_summary:
        users: ["ops"]
      background_submit:
        users: ["runner"]
  session_admin_persist: true
  mode: "webhook"
  webhook_dedup_backend: "valkey"
  inbound_queue_capacity: 100
  foreground_queue_capacity: 256
  foreground_max_in_flight_messages: 16
  foreground_turn_timeout_secs: 80
  session_partition: "chat"
  max_tool_rounds: 30
discord:
  runtime_mode: "gateway"
  acl:
    role_aliases:
      maintainers: "3009001"
      auditors: "3009002"
    allow:
      users: ["dx1"]
      roles: ["maintainers"]
      guilds: ["3001"]
    admin:
      users: ["dx1"]
      roles: ["maintainers"]
    control:
      allow_from:
        users: ["dx1"]
        roles: ["maintainers"]
      rules:
        - commands: ["/session partition"]
          allow:
            users: ["dx1"]
    slash:
      global:
        users: ["dx1"]
      session_status:
        roles: ["auditors"]
      session_budget:
        roles: ["auditors"]
      session_memory:
        roles: ["auditors"]
      session_feedback:
        users: ["ops"]
      job_status:
        users: ["ops"]
      jobs_summary:
        users: ["ops"]
      background_submit:
        users: ["runner"]
  ingress_bind: "0.0.0.0:8082"
  ingress_path: "/discord/ingress"
  ingress_secret_token: "system-secret"
  session_partition: "guild_channel_user"
  inbound_queue_capacity: 512
  turn_timeout_secs: 120
  foreground_max_in_flight_messages: 16
session:
  window_max_turns: 512
  consolidation_async: true
  context_budget_tokens: 5000
  context_budget_reserve_tokens: 256
  context_budget_strategy: "recent_first"
  summary_max_segments: 6
  summary_max_chars: 256
  redis_prefix: "system-prefix"
  ttl_secs: 3600
embedding:
  backend: "http"
  timeout_secs: 45
  max_in_flight: 12
  batch_max_size: 64
  batch_max_concurrency: 2
  model: "ollama/qwen3-embedding:0.6b"
  litellm_model: "ollama/qwen3-embedding:0.6b"
  litellm_api_base: "http://127.0.0.1:11434"
  dimension: 1024
  client_url: "http://127.0.0.1:3002"
memory:
  embedding_timeout_ms: 7000
  embedding_timeout_cooldown_ms: 18000
  persistence_backend: "local"
  persistence_valkey_url: "redis://127.0.0.1:6379/0"
  persistence_strict_startup: true
  recall_credit_enabled: false
  recall_credit_max_candidates: 2
  decay_enabled: false
  decay_every_turns: 12
  decay_factor: 0.95
  gate_promote_threshold: 0.77
  gate_obsolete_threshold: 0.33
  gate_promote_min_usage: 4
  gate_obsolete_min_usage: 3
  gate_promote_failure_rate_ceiling: 0.24
  gate_obsolete_failure_rate_floor: 0.71
  gate_promote_min_ttl_score: 0.55
  gate_obsolete_max_ttl_score: 0.40
  stream_consumer_enabled: true
  stream_name: "memory.events.system"
  stream_consumer_group: "system-group"
  stream_consumer_name_prefix: "system-agent"
  stream_consumer_batch_size: 12
  stream_consumer_block_ms: 1500
"#,
    );
    write_file(
        user.clone(),
        r#"
agent:
  llm_backend: "litellm_rs"
inference:
  provider: "minimax"
  api_key_env: "MINIMAX_API_KEY"
  base_url: "https://api.minimax.io/v1"
  model: "MiniMax-M2.5"
  timeout: 90
  max_in_flight: 32
mcp:
  agent_pool_size: 8
  agent_connect_retries: 5
  agent_strict_startup: false
  agent_tool_timeout_secs: 240
  agent_list_tools_cache_ttl_ms: 2500
  agent_discover_cache_enabled: false
  agent_discover_cache_key_prefix: "user-discover"
  agent_discover_cache_ttl_secs: 90
telegram:
  acl:
    allow:
      users: ["2002"]
    admin:
      users: ["2002"]
    control:
      allow_from:
        users: []
      rules:
        - commands: ["/reset", "/clear"]
          allow:
            users: ["2002"]
    slash:
      global:
        users: []
      session_status:
        users: ["2002"]
      session_budget:
        users: ["2002"]
      session_memory:
        users: ["2002"]
      session_feedback:
        users: ["ops"]
      job_status:
        users: ["ops"]
      jobs_summary:
        users: ["ops"]
      background_submit:
        users: ["ops"]
  session_admin_persist: false
  mode: "polling"
  inbound_queue_capacity: 120
discord:
  runtime_mode: "ingress"
  acl:
    allow:
      users: ["ux2"]
    admin:
      users: ["ux2"]
    control:
      allow_from:
        users: []
      rules:
        - commands: ["/resume"]
          allow:
            users: ["ux2"]
    slash:
      global:
        users: []
      session_status:
        users: ["ux2"]
      session_budget:
        users: ["ux2"]
      session_memory:
        users: ["ux2"]
      session_feedback:
        users: ["ops"]
      job_status:
        users: ["ops"]
      jobs_summary:
        users: ["ops"]
      background_submit:
        users: ["ops"]
  ingress_bind: "127.0.0.1:9092"
  inbound_queue_capacity: 1024
  foreground_max_in_flight_messages: 64
session:
  window_max_turns: 2048
  consolidation_async: false
  context_budget_tokens: 7000
  context_budget_strategy: "summary_first"
  summary_max_segments: 10
  redis_prefix: "user-prefix"
embedding:
  backend: "litellm_rs"
  timeout_secs: 90
  max_in_flight: 96
  batch_max_size: 256
  batch_max_concurrency: 6
  litellm_api_base: "http://localhost:11434"
  dimension: 768
memory:
  embedding_timeout_ms: 12000
  embedding_timeout_cooldown_ms: 25000
  persistence_valkey_url: "redis://127.0.0.1:6380/0"
  persistence_strict_startup: false
  recall_credit_enabled: true
  recall_credit_max_candidates: 5
  decay_enabled: true
  decay_every_turns: 20
  decay_factor: 0.99
  gate_promote_threshold: 0.88
  gate_obsolete_min_usage: 5
  gate_obsolete_failure_rate_floor: 0.92
  stream_consumer_enabled: false
  stream_name: "memory.events.user"
  stream_consumer_group: "user-group"
  stream_consumer_name_prefix: "user-agent"
  stream_consumer_batch_size: 24
  stream_consumer_block_ms: 900
"#,
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.agent.llm_backend.as_deref(), Some("litellm_rs"));
    assert_eq!(merged.inference.provider.as_deref(), Some("minimax"));
    assert_eq!(
        merged.inference.api_key_env.as_deref(),
        Some("MINIMAX_API_KEY")
    );
    assert_eq!(
        merged.inference.base_url.as_deref(),
        Some("https://api.minimax.io/v1")
    );
    assert_eq!(merged.inference.model.as_deref(), Some("MiniMax-M2.5"));
    assert_eq!(merged.inference.timeout, Some(90));
    assert_eq!(merged.inference.max_tokens, Some(2048));
    assert_eq!(merged.inference.max_in_flight, Some(32));
    assert_eq!(merged.mcp.agent_pool_size, Some(8));
    assert_eq!(merged.mcp.agent_handshake_timeout_secs, Some(30));
    assert_eq!(merged.mcp.agent_connect_retries, Some(5));
    assert_eq!(merged.mcp.agent_strict_startup, Some(false));
    assert_eq!(merged.mcp.agent_connect_retry_backoff_ms, Some(500));
    assert_eq!(merged.mcp.agent_tool_timeout_secs, Some(240));
    assert_eq!(merged.mcp.agent_list_tools_cache_ttl_ms, Some(2500));
    assert_eq!(merged.mcp.agent_discover_cache_enabled, Some(false));
    assert_eq!(
        merged.mcp.agent_discover_cache_key_prefix.as_deref(),
        Some("user-discover")
    );
    assert_eq!(merged.mcp.agent_discover_cache_ttl_secs, Some(90));
    let telegram_overrides = build_telegram_acl_overrides(&merged).expect("telegram acl overrides");
    assert_eq!(telegram_overrides.allowed_users, vec!["2002"]);
    assert_eq!(telegram_overrides.allowed_groups, vec!["-2001"]);
    assert_eq!(merged.telegram.session_admin_persist, Some(false));
    assert_eq!(telegram_overrides.admin_users, vec!["2002"]);
    assert_eq!(
        telegram_overrides.control_command_allow_from,
        Some(Vec::new())
    );
    assert_eq!(telegram_overrides.control_command_rules.len(), 1);
    assert_eq!(
        telegram_overrides.slash_command_allow_from,
        Some(Vec::new())
    );
    assert_eq!(
        telegram_overrides.slash_session_status_allow_from,
        Some(vec!["2002".to_string()])
    );
    assert_eq!(
        telegram_overrides.slash_session_budget_allow_from,
        Some(vec!["2002".to_string()])
    );
    assert_eq!(
        telegram_overrides.slash_session_memory_allow_from,
        Some(vec!["2002".to_string()])
    );
    assert_eq!(
        telegram_overrides.slash_session_feedback_allow_from,
        Some(vec!["ops".to_string()])
    );
    assert_eq!(
        telegram_overrides.slash_job_allow_from,
        Some(vec!["ops".to_string()])
    );
    assert_eq!(
        telegram_overrides.slash_jobs_allow_from,
        Some(vec!["ops".to_string()])
    );
    assert_eq!(
        telegram_overrides.slash_bg_allow_from,
        Some(vec!["ops".to_string()])
    );
    assert_eq!(merged.telegram.mode.as_deref(), Some("polling"));
    assert_eq!(
        merged.telegram.webhook_dedup_backend.as_deref(),
        Some("valkey")
    );
    assert_eq!(merged.telegram.inbound_queue_capacity, Some(120));
    assert_eq!(merged.telegram.foreground_queue_capacity, Some(256));
    assert_eq!(merged.telegram.foreground_max_in_flight_messages, Some(16));
    assert_eq!(merged.telegram.foreground_turn_timeout_secs, Some(80));
    assert_eq!(merged.telegram.session_partition.as_deref(), Some("chat"));
    assert_eq!(merged.telegram.max_tool_rounds, Some(30));
    let discord_overrides = build_discord_acl_overrides(&merged).expect("discord acl overrides");
    assert_eq!(
        discord_overrides.allowed_users,
        vec!["ux2".to_string(), "role:3009001".to_string()]
    );
    assert_eq!(discord_overrides.allowed_guilds, vec!["3001".to_string()]);
    assert_eq!(discord_overrides.admin_users, Some(vec!["ux2".to_string()]));
    assert_eq!(
        discord_overrides.control_command_allow_from,
        Some(Vec::new())
    );
    assert_eq!(discord_overrides.control_command_rules.len(), 1);
    assert_eq!(discord_overrides.slash_command_allow_from, Some(Vec::new()));
    assert_eq!(
        discord_overrides.slash_session_status_allow_from,
        Some(vec!["ux2".to_string()])
    );
    assert_eq!(
        discord_overrides.slash_session_budget_allow_from,
        Some(vec!["ux2".to_string()])
    );
    assert_eq!(
        discord_overrides.slash_session_memory_allow_from,
        Some(vec!["ux2".to_string()])
    );
    assert_eq!(
        discord_overrides.slash_session_feedback_allow_from,
        Some(vec!["ops".to_string()])
    );
    assert_eq!(
        discord_overrides.slash_job_allow_from,
        Some(vec!["ops".to_string()])
    );
    assert_eq!(
        discord_overrides.slash_jobs_allow_from,
        Some(vec!["ops".to_string()])
    );
    assert_eq!(
        discord_overrides.slash_bg_allow_from,
        Some(vec!["ops".to_string()])
    );
    assert_eq!(
        merged.discord.ingress_bind.as_deref(),
        Some("127.0.0.1:9092")
    );
    assert_eq!(merged.discord.runtime_mode.as_deref(), Some("ingress"));
    assert_eq!(
        merged.discord.ingress_path.as_deref(),
        Some("/discord/ingress")
    );
    assert_eq!(
        merged.discord.ingress_secret_token.as_deref(),
        Some("system-secret")
    );
    assert_eq!(
        merged.discord.session_partition.as_deref(),
        Some("guild_channel_user")
    );
    assert_eq!(merged.discord.inbound_queue_capacity, Some(1024));
    assert_eq!(merged.discord.turn_timeout_secs, Some(120));
    assert_eq!(merged.discord.foreground_max_in_flight_messages, Some(64));
    assert_eq!(merged.session.window_max_turns, Some(2048));
    assert_eq!(merged.session.consolidation_async, Some(false));
    assert_eq!(merged.session.context_budget_tokens, Some(7000));
    assert_eq!(merged.session.context_budget_reserve_tokens, Some(256));
    assert_eq!(
        merged.session.context_budget_strategy.as_deref(),
        Some("summary_first")
    );
    assert_eq!(merged.session.summary_max_segments, Some(10));
    assert_eq!(merged.session.summary_max_chars, Some(256));
    assert_eq!(merged.session.redis_prefix.as_deref(), Some("user-prefix"));
    assert_eq!(merged.session.ttl_secs, Some(3600));
    assert_eq!(
        merged.embedding.model.as_deref(),
        Some("ollama/qwen3-embedding:0.6b")
    );
    assert_eq!(merged.embedding.backend.as_deref(), Some("litellm_rs"));
    assert_eq!(merged.embedding.timeout_secs, Some(90));
    assert_eq!(merged.embedding.max_in_flight, Some(96));
    assert_eq!(merged.embedding.batch_max_size, Some(256));
    assert_eq!(merged.embedding.batch_max_concurrency, Some(6));
    assert_eq!(
        merged.embedding.litellm_model.as_deref(),
        Some("ollama/qwen3-embedding:0.6b")
    );
    assert_eq!(
        merged.embedding.litellm_api_base.as_deref(),
        Some("http://localhost:11434")
    );
    assert_eq!(merged.embedding.dimension, Some(768));
    assert_eq!(
        merged.embedding.client_url.as_deref(),
        Some("http://127.0.0.1:3002")
    );
    assert_eq!(merged.memory.persistence_backend.as_deref(), Some("local"));
    assert_eq!(merged.memory.embedding_timeout_ms, Some(12000));
    assert_eq!(merged.memory.embedding_timeout_cooldown_ms, Some(25000));
    assert_eq!(
        merged.memory.persistence_valkey_url.as_deref(),
        Some("redis://127.0.0.1:6380/0")
    );
    assert_eq!(merged.memory.persistence_strict_startup, Some(false));
    assert_eq!(merged.memory.recall_credit_enabled, Some(true));
    assert_eq!(merged.memory.recall_credit_max_candidates, Some(5));
    assert_eq!(merged.memory.decay_enabled, Some(true));
    assert_eq!(merged.memory.decay_every_turns, Some(20));
    assert_eq!(merged.memory.decay_factor, Some(0.99));
    assert_eq!(merged.memory.gate_promote_threshold, Some(0.88));
    assert_eq!(merged.memory.gate_obsolete_threshold, Some(0.33));
    assert_eq!(merged.memory.gate_promote_min_usage, Some(4));
    assert_eq!(merged.memory.gate_obsolete_min_usage, Some(5));
    assert_eq!(merged.memory.gate_promote_failure_rate_ceiling, Some(0.24));
    assert_eq!(merged.memory.gate_obsolete_failure_rate_floor, Some(0.92));
    assert_eq!(merged.memory.gate_promote_min_ttl_score, Some(0.55));
    assert_eq!(merged.memory.gate_obsolete_max_ttl_score, Some(0.40));
    assert_eq!(merged.memory.stream_consumer_enabled, Some(false));
    assert_eq!(
        merged.memory.stream_name.as_deref(),
        Some("memory.events.user")
    );
    assert_eq!(
        merged.memory.stream_consumer_group.as_deref(),
        Some("user-group")
    );
    assert_eq!(
        merged.memory.stream_consumer_name_prefix.as_deref(),
        Some("user-agent")
    );
    assert_eq!(merged.memory.stream_consumer_batch_size, Some(24));
    assert_eq!(merged.memory.stream_consumer_block_ms, Some(900));
}

#[test]
fn merge_telegram_group_policy_overrides_deeply() {
    let tmp = TempDir::new().expect("tempdir");
    let system = tmp.path().join("packages/conf/settings.yaml");
    let user = tmp.path().join(".config/omni-dev-fusion/settings.yaml");

    write_file(
        system.clone(),
        r#"
telegram:
  group_policy: "allowlist"
  group_allow_from: "ops"
  session_admin_persist: true
  require_mention: true
  groups:
    "*":
      admin_users:
        users: ["9090"]
      require_mention: true
      topics:
        "42":
          enabled: false
    "-100":
      group_policy: "disabled"
      allow_from:
        users: ["root"]
      admin_users:
        users: ["3001"]
      topics:
        "10":
          allow_from:
            users: ["ops1"]
          admin_users:
            users: ["7001"]
"#,
    );
    write_file(
        user.clone(),
        r#"
telegram:
  group_policy: "open"
  session_admin_persist: false
  require_mention: false
  groups:
    "-100":
      allow_from:
        users: ["admin2"]
      admin_users:
        users: ["3002"]
      topics:
        "10":
          require_mention: true
          admin_users:
            users: ["7002"]
        "11":
          enabled: true
          admin_users:
            users: ["8001"]
    "-200":
      enabled: true
      admin_users:
        users: ["4001"]
"#,
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.telegram.group_policy.as_deref(), Some("open"));
    assert_eq!(merged.telegram.group_allow_from.as_deref(), Some("ops"));
    assert_eq!(merged.telegram.session_admin_persist, Some(false));
    assert_eq!(merged.telegram.require_mention, Some(false));

    let groups = merged.telegram.groups.expect("merged groups");
    let wildcard = groups.get("*").expect("wildcard group");
    assert_eq!(
        wildcard
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["9090".to_string()])
    );
    assert_eq!(wildcard.require_mention, Some(true));

    let group_100 = groups.get("-100").expect("group -100");
    assert_eq!(group_100.group_policy.as_deref(), Some("disabled"));
    assert_eq!(
        group_100
            .allow_from
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["admin2".to_string()])
    );
    assert_eq!(
        group_100
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["3002".to_string()])
    );
    let topics_100 = group_100.topics.as_ref().expect("group -100 topics");
    let topic_10 = topics_100.get("10").expect("topic 10");
    assert_eq!(
        topic_10
            .allow_from
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["ops1".to_string()])
    );
    assert_eq!(
        topic_10
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["7002".to_string()])
    );
    assert_eq!(topic_10.require_mention, Some(true));
    let topic_11 = topics_100.get("11").expect("topic 11");
    assert_eq!(
        topic_11
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["8001".to_string()])
    );
    assert_eq!(topic_11.enabled, Some(true));

    let group_200 = groups.get("-200").expect("group -200");
    assert_eq!(
        group_200
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["4001".to_string()])
    );
    assert_eq!(group_200.enabled, Some(true));
}

#[test]
fn missing_files_fallback_to_defaults() {
    let tmp = TempDir::new().expect("tempdir");
    let merged = load_runtime_settings_from_paths(
        &tmp.path().join("missing-system.yaml"),
        &tmp.path().join("missing-user.yaml"),
    );
    let telegram_overrides = build_telegram_acl_overrides(&merged).expect("telegram acl overrides");
    assert!(telegram_overrides.allowed_users.is_empty());
    assert!(telegram_overrides.allowed_groups.is_empty());
    assert!(merged.telegram.group_policy.is_none());
    assert!(merged.telegram.session_admin_persist.is_none());
    assert!(merged.telegram.group_allow_from.is_none());
    assert!(merged.telegram.require_mention.is_none());
    assert!(merged.telegram.groups.is_none());
    assert!(telegram_overrides.admin_users.is_empty());
    assert!(telegram_overrides.control_command_allow_from.is_none());
    assert!(telegram_overrides.control_command_rules.is_empty());
    assert!(telegram_overrides.slash_command_allow_from.is_none());
    assert!(telegram_overrides.slash_session_status_allow_from.is_none());
    assert!(telegram_overrides.slash_session_budget_allow_from.is_none());
    assert!(telegram_overrides.slash_session_memory_allow_from.is_none());
    assert!(
        telegram_overrides
            .slash_session_feedback_allow_from
            .is_none()
    );
    assert!(telegram_overrides.slash_job_allow_from.is_none());
    assert!(telegram_overrides.slash_jobs_allow_from.is_none());
    assert!(telegram_overrides.slash_bg_allow_from.is_none());
    assert!(merged.telegram.max_tool_rounds.is_none());
    let discord_overrides = build_discord_acl_overrides(&merged).expect("discord acl overrides");
    assert!(discord_overrides.allowed_users.is_empty());
    assert!(discord_overrides.allowed_guilds.is_empty());
    assert!(discord_overrides.admin_users.is_none());
    assert!(discord_overrides.control_command_allow_from.is_none());
    assert!(discord_overrides.control_command_rules.is_empty());
    assert!(discord_overrides.slash_command_allow_from.is_none());
    assert!(discord_overrides.slash_session_status_allow_from.is_none());
    assert!(discord_overrides.slash_session_budget_allow_from.is_none());
    assert!(discord_overrides.slash_session_memory_allow_from.is_none());
    assert!(
        discord_overrides
            .slash_session_feedback_allow_from
            .is_none()
    );
    assert!(discord_overrides.slash_job_allow_from.is_none());
    assert!(discord_overrides.slash_jobs_allow_from.is_none());
    assert!(discord_overrides.slash_bg_allow_from.is_none());
    assert!(merged.discord.runtime_mode.is_none());
    assert!(merged.discord.session_partition.is_none());
    assert!(merged.discord.foreground_max_in_flight_messages.is_none());
    assert!(merged.mcp.agent_pool_size.is_none());
    assert!(merged.mcp.agent_handshake_timeout_secs.is_none());
    assert!(merged.mcp.agent_connect_retries.is_none());
    assert!(merged.mcp.agent_strict_startup.is_none());
    assert!(merged.mcp.agent_connect_retry_backoff_ms.is_none());
    assert!(merged.mcp.agent_tool_timeout_secs.is_none());
    assert!(merged.mcp.agent_list_tools_cache_ttl_ms.is_none());
    assert!(merged.mcp.agent_discover_cache_enabled.is_none());
    assert!(merged.mcp.agent_discover_cache_key_prefix.is_none());
    assert!(merged.mcp.agent_discover_cache_ttl_secs.is_none());
    assert!(merged.session.window_max_turns.is_none());
    assert!(merged.session.context_budget_strategy.is_none());
    assert!(merged.session.summary_max_segments.is_none());
    assert!(merged.embedding.backend.is_none());
    assert!(merged.embedding.timeout_secs.is_none());
    assert!(merged.embedding.max_in_flight.is_none());
    assert!(merged.embedding.batch_max_size.is_none());
    assert!(merged.embedding.batch_max_concurrency.is_none());
    assert!(merged.embedding.model.is_none());
    assert!(merged.embedding.litellm_api_base.is_none());
    assert!(merged.embedding.dimension.is_none());
    assert!(merged.memory.gate_promote_threshold.is_none());
    assert!(merged.memory.gate_obsolete_threshold.is_none());
    assert!(merged.memory.gate_promote_min_usage.is_none());
    assert!(merged.memory.gate_obsolete_min_usage.is_none());
    assert!(merged.memory.gate_promote_failure_rate_ceiling.is_none());
    assert!(merged.memory.gate_obsolete_failure_rate_floor.is_none());
    assert!(merged.memory.gate_promote_min_ttl_score.is_none());
    assert!(merged.memory.gate_obsolete_max_ttl_score.is_none());
}

#[test]
fn invalid_yaml_is_ignored() {
    let tmp = TempDir::new().expect("tempdir");
    let system = tmp.path().join("packages/conf/settings.yaml");
    let user = tmp.path().join(".config/omni-dev-fusion/settings.yaml");

    write_file(system, "telegram: [");
    write_file(
        user.clone(),
        r#"
telegram:
  acl:
    allow:
      users: ["ok-user"]
"#,
    );

    let merged =
        load_runtime_settings_from_paths(&tmp.path().join("packages/conf/settings.yaml"), &user);
    let telegram_overrides = build_telegram_acl_overrides(&merged).expect("telegram acl overrides");
    assert_eq!(telegram_overrides.allowed_users, vec!["ok-user"]);
}

#[test]
fn embedding_timeout_alias_timeout_is_supported() {
    let tmp = TempDir::new().expect("tempdir");
    let system = tmp.path().join("packages/conf/settings.yaml");
    let user = tmp.path().join(".config/omni-dev-fusion/settings.yaml");

    write_file(
        system,
        r#"
embedding:
  backend: "http"
  timeout: 31
"#,
    );
    write_file(
        user,
        r#"
embedding:
  timeout_secs: 47
"#,
    );

    let merged = load_runtime_settings_from_paths(
        &tmp.path().join("packages/conf/settings.yaml"),
        &tmp.path().join(".config/omni-dev-fusion/settings.yaml"),
    );
    assert_eq!(merged.embedding.backend.as_deref(), Some("http"));
    assert_eq!(merged.embedding.timeout_secs, Some(47));
}
