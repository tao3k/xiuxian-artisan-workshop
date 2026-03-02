use super::hot_reload::start_hot_reload_driver;
use super::memory::build_memory_runtime;
use super::qianhuan::init_persona_registries;
use super::service_mount::{ServiceMountCatalog, ServiceMountMeta};
use super::zhenfa::{build_global_link_graph_index, init_zhenfa_tool_bridge};
use super::zhixing::{init_zhixing_runtime, mount_zhixing_services, resolve_project_root};
use crate::agent::Agent;
use crate::config::{AgentConfig, RuntimeSettings, load_runtime_settings, load_xiuxian_config};
use crate::llm::LlmClient;
use crate::session::{BoundedSessionStore, SessionStore};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::sync::RwLock;

impl Agent {
    /// Build agent from config.
    ///
    /// # Errors
    /// Returns an error when session backends, MCP startup, or memory backends
    /// fail to initialize.
    pub async fn from_config(config: AgentConfig) -> Result<Self> {
        let api_key = config.resolve_api_key();
        let llm = LlmClient::new(config.inference_url.clone(), config.model.clone(), api_key);
        let session = SessionStore::new()?;
        let bounded_session = match config.window_max_turns {
            Some(max_turns) => Some(BoundedSessionStore::new_with_limits(
                max_turns,
                config.summary_max_segments,
                config.summary_max_chars,
            )?),
            None => None,
        };
        Self::build_with_backends(config, llm, session, bounded_session).await
    }

    #[doc(hidden)]
    pub async fn from_config_with_session_backends_for_test(
        config: AgentConfig,
        session: SessionStore,
        bounded_session: Option<BoundedSessionStore>,
    ) -> Result<Self> {
        let api_key = config.resolve_api_key();
        let llm = LlmClient::new(config.inference_url.clone(), config.model.clone(), api_key);
        Self::build_with_backends(config, llm, session, bounded_session).await
    }

    async fn build_with_backends(
        config: AgentConfig,
        llm: LlmClient,
        session: SessionStore,
        bounded_session: Option<BoundedSessionStore>,
    ) -> Result<Self> {
        let mut service_mounts = ServiceMountCatalog::new();
        let mcp_client = init_mcp_client_and_mount(&config, &mut service_mounts).await?;

        let runtime_settings = load_runtime_settings();
        let session_reset_idle_timeout_ms =
            resolve_session_reset_idle_timeout_ms(&runtime_settings);
        let xiuxian_toml_cfg = load_xiuxian_config();
        let project_root = resolve_project_root();
        let persona_registries =
            init_persona_registries(&project_root, &xiuxian_toml_cfg, &mut service_mounts);
        let memory_runtime =
            build_memory_runtime(&config, &session, &runtime_settings, &mut service_mounts)?;

        let mut native_tools = super::super::NativeToolRegistry::new();
        let zhixing_runtime = init_zhixing_runtime(&persona_registries, &mut service_mounts);
        let global_link_graph_index = build_global_link_graph_index(
            &xiuxian_toml_cfg,
            zhixing_runtime.as_ref(),
            &mut service_mounts,
        );
        let zhenfa_tools = init_zhenfa_tool_bridge(
            &xiuxian_toml_cfg,
            zhixing_runtime.as_ref(),
            global_link_graph_index.clone(),
            memory_runtime.memory_store.clone(),
            memory_runtime.memory_state_backend.clone(),
            &mut service_mounts,
        );
        let heyi = if let Some(ref runtime_bundle) = zhixing_runtime {
            mount_zhixing_services(&runtime_bundle.heyi, &mut native_tools, &mut service_mounts);
            Some(Arc::clone(&runtime_bundle.heyi))
        } else {
            service_mounts.skipped(
                "zhixing.native_tools",
                "tooling",
                ServiceMountMeta::default().detail("heyi runtime unavailable"),
            );
            service_mounts.skipped(
                "zhixing.timer_watcher",
                "scheduler",
                ServiceMountMeta::default().detail("heyi runtime unavailable"),
            );
            None
        };
        let hot_reload_driver = start_hot_reload_driver(
            zhixing_runtime.as_ref(),
            &xiuxian_toml_cfg,
            &mut service_mounts,
        )
        .await;

        let service_mount_records = Arc::new(RwLock::new(service_mounts.finish()));

        Ok(Self {
            config,
            session,
            session_reset_idle_timeout_ms,
            session_last_activity_unix_ms: Arc::new(RwLock::new(HashMap::new())),
            bounded_session,
            memory_store: memory_runtime.memory_store,
            memory_state_backend: memory_runtime.memory_state_backend,
            memory_state_load_status: memory_runtime.memory_state_load_status,
            embedding_client: memory_runtime.embedding_client,
            embedding_runtime: memory_runtime.embedding_runtime,
            context_budget_snapshots: Arc::new(RwLock::new(HashMap::new())),
            memory_recall_metrics: Arc::new(RwLock::new(
                super::super::memory_recall_metrics::MemoryRecallMetricsState::default(),
            )),
            manifestation_manager: zhixing_runtime
                .as_ref()
                .map(|runtime| Arc::clone(&runtime.manifestation_manager)),
            reflection_policy_hints: Arc::new(RwLock::new(HashMap::new())),
            memory_decay_turn_counter: Arc::new(AtomicU64::new(0)),
            downstream_admission_policy:
                super::super::admission::DownstreamAdmissionPolicy::from_env(),
            downstream_admission_metrics:
                super::super::admission::DownstreamAdmissionMetrics::default(),
            llm,
            mcp: mcp_client,
            heyi,
            native_tools: Arc::new(native_tools),
            zhenfa_tools,
            memory_stream_consumer_task: memory_runtime.memory_stream_consumer_task,
            _hot_reload_driver: hot_reload_driver,
            service_mount_records,
        })
    }
}

const DEFAULT_SESSION_RESET_IDLE_TIMEOUT_MINS: u64 = 60;

fn resolve_session_reset_idle_timeout_ms(runtime_settings: &RuntimeSettings) -> Option<u64> {
    let idle_timeout_mins: Option<u64> =
        parse_positive_u64_from_env("OMNI_AGENT_SESSION_RESET_IDLE_TIMEOUT_MINS")
            .or(runtime_settings
                .session
                .reset_idle_timeout_mins
                .filter(|value| *value > 0))
            .or(Some(DEFAULT_SESSION_RESET_IDLE_TIMEOUT_MINS));
    idle_timeout_mins.map(|mins| mins.saturating_mul(60_000))
}

fn parse_positive_u64_from_env(name: &str) -> Option<u64> {
    let raw = std::env::var(name).ok()?;
    if let Some(value) = raw.trim().parse::<u64>().ok().filter(|value| *value > 0) {
        Some(value)
    } else {
        tracing::warn!(
            env_var = %name,
            value = %raw,
            "invalid positive integer env value; ignoring override"
        );
        None
    }
}

async fn init_mcp_client_and_mount(
    config: &AgentConfig,
    service_mounts: &mut ServiceMountCatalog,
) -> Result<Option<crate::mcp::McpClientPool>> {
    let mcp_client = super::super::mcp_startup::connect_mcp_pool_if_configured(config).await?;
    if let Some(url) = config
        .mcp_servers
        .iter()
        .find(|server| server.url.is_some())
        .and_then(|server| server.url.clone())
    {
        if mcp_client.is_some() {
            service_mounts.mounted("mcp.pool", "mcp", ServiceMountMeta::default().endpoint(url));
        } else {
            service_mounts.skipped(
                "mcp.pool",
                "mcp",
                ServiceMountMeta::default()
                    .endpoint(url)
                    .detail("startup skipped or unavailable"),
            );
        }
    } else {
        service_mounts.skipped(
            "mcp.pool",
            "mcp",
            ServiceMountMeta::default().detail("no mcp url configured"),
        );
    }
    Ok(mcp_client)
}
