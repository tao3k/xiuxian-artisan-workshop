use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use omni_memory::EpisodeStore;
use serde_json::{Value, json};
use xiuxian_qianhuan::ManifestationManager;
use xiuxian_wendao::{LinkGraphIndex, SkillVfsResolver};
use xiuxian_zhenfa::{
    ZhenfaContext, ZhenfaError, ZhenfaOrchestrator, ZhenfaRegistry, ZhenfaSignal, ZhenfaSignalSink,
    ZhenfaTool,
};

use super::valkey_hooks::build_zhenfa_orchestrator_hooks;
use crate::agent::memory_state::MemoryStateBackend;
use crate::config::XiuxianConfig;

const ZHENFA_BASE_URL_ENV: &str = "ZHENFA_BASE_URL";

#[derive(Clone, Default)]
pub(crate) struct ZhenfaRuntimeDeps {
    pub(crate) manifestation_manager: Option<Arc<ManifestationManager>>,
    pub(crate) link_graph_index: Option<Arc<LinkGraphIndex>>,
    pub(crate) skill_vfs_resolver: Option<Arc<SkillVfsResolver>>,
    pub(crate) memory_store: Option<Arc<EpisodeStore>>,
    pub(crate) memory_state_backend: Option<Arc<MemoryStateBackend>>,
}

/// In-process tool bridge that forwards selected calls into `ZhenfaOrchestrator`.
#[derive(Clone)]
pub(crate) struct ZhenfaToolBridge {
    orchestrator: ZhenfaOrchestrator,
    tools: HashMap<String, Value>,
    gateway_base_url: Option<String>,
    valkey_hooks_enabled: bool,
    context_link_graph_index: Option<Arc<LinkGraphIndex>>,
    context_skill_vfs_resolver: Option<Arc<SkillVfsResolver>>,
    context_manifestation_manager: Option<Arc<ManifestationManager>>,
    context_memory_store: Option<Arc<EpisodeStore>>,
}

impl ZhenfaToolBridge {
    /// Build from merged `xiuxian.toml` configuration and runtime dependencies.
    pub(crate) fn from_xiuxian_config(
        config: &XiuxianConfig,
        deps: &ZhenfaRuntimeDeps,
    ) -> Option<Self> {
        let mut registry = ZhenfaRegistry::new();

        for name in resolve_enabled_tool_names(config) {
            let Some(tool) = build_native_tool(name.as_str(), deps) else {
                continue;
            };
            registry.register(tool);
        }

        if registry.is_empty() {
            return None;
        }

        let active_tools = registry.definitions();
        let mut hooks = build_zhenfa_orchestrator_hooks(config).unwrap_or_default();
        let valkey_hooks_enabled =
            hooks.cache.is_some() || hooks.mutation_lock.is_some() || hooks.audit_sink.is_some();
        if let Some(memory_store) = deps.memory_store.as_ref() {
            hooks.signal_sink = Some(Arc::new(MemoryRewardSignalSink::new(
                Arc::clone(memory_store),
                deps.memory_state_backend.as_ref().map(Arc::clone),
            )));
        }
        let orchestrator = ZhenfaOrchestrator::with_hooks(registry, hooks);

        Some(Self {
            orchestrator,
            tools: active_tools,
            gateway_base_url: resolve_zhenfa_base_url(config),
            valkey_hooks_enabled,
            context_link_graph_index: deps.link_graph_index.as_ref().map(Arc::clone),
            context_skill_vfs_resolver: deps.skill_vfs_resolver.as_ref().map(Arc::clone),
            context_manifestation_manager: deps.manifestation_manager.as_ref().map(Arc::clone),
            context_memory_store: deps.memory_store.as_ref().map(Arc::clone),
        })
    }

    /// Optional external gateway endpoint configured for this bridge.
    #[must_use]
    pub(crate) fn base_url(&self) -> Option<&str> {
        self.gateway_base_url.as_deref()
    }

    /// Number of bridged tools.
    #[must_use]
    pub(crate) fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Returns whether Valkey hooks are enabled for this bridge instance.
    #[must_use]
    pub(crate) fn valkey_hooks_enabled(&self) -> bool {
        self.valkey_hooks_enabled
    }

    /// Build LLM-visible tool definitions.
    #[must_use]
    pub(crate) fn list_for_llm(&self) -> Vec<Value> {
        self.tools.values().cloned().collect()
    }

    /// True when the bridge handles this tool name.
    #[must_use]
    pub(crate) fn handles_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Execute one bridged tool call.
    ///
    /// # Errors
    /// Returns an error when params are invalid, tool is disabled, or native dispatch fails.
    pub(crate) async fn call_tool(
        &self,
        session_id: Option<&str>,
        name: &str,
        arguments: Option<Value>,
    ) -> Result<String> {
        if !self.handles_tool(name) {
            return Err(anyhow!("zhenfa tool `{name}` is not enabled"));
        }
        let params = normalize_tool_arguments(arguments)?;
        let mut ctx = ZhenfaContext::new(session_id.map(ToString::to_string), None, HashMap::new());
        if let Some(index) = self.context_link_graph_index.as_ref() {
            let _ = ctx.insert_shared_extension::<LinkGraphIndex>(Arc::clone(index));
        }
        if let Some(resolver) = self.context_skill_vfs_resolver.as_ref() {
            let _ = ctx.insert_shared_extension::<SkillVfsResolver>(Arc::clone(resolver));
        }
        if let Some(manager) = self.context_manifestation_manager.as_ref() {
            let _ = ctx.insert_shared_extension::<ManifestationManager>(Arc::clone(manager));
        }
        if let Some(memory_store) = self.context_memory_store.as_ref() {
            let _ = ctx.insert_shared_extension::<EpisodeStore>(Arc::clone(memory_store));
        }
        match self.orchestrator.dispatch(name, &ctx, params.clone()).await {
            Ok(output) => Ok(output),
            Err(primary_error) => {
                if should_retry_with_null_arguments(&params)
                    && let Ok(output) = self.orchestrator.dispatch(name, &ctx, Value::Null).await
                {
                    return Ok(output);
                }
                tracing::warn!(
                    event = "agent.zhenfa.bridge.dispatch_failed",
                    tool = name,
                    llm_safe_error = primary_error.llm_safe_message(),
                    error = %primary_error,
                    "zhenfa native dispatch failed"
                );
                Err(anyhow!(
                    "zhenfa native dispatch failed for `{name}`: {}",
                    primary_error.llm_safe_message()
                ))
            }
        }
    }
}

#[derive(Clone)]
pub(super) struct MemoryRewardSignalSink {
    memory_store: Arc<EpisodeStore>,
    memory_state_backend: Option<Arc<MemoryStateBackend>>,
}

impl MemoryRewardSignalSink {
    pub(super) fn new(
        memory_store: Arc<EpisodeStore>,
        memory_state_backend: Option<Arc<MemoryStateBackend>>,
    ) -> Self {
        Self {
            memory_store,
            memory_state_backend,
        }
    }
}

#[async_trait]
impl ZhenfaSignalSink for MemoryRewardSignalSink {
    async fn emit(&self, ctx: &ZhenfaContext, signal: ZhenfaSignal) -> Result<(), ZhenfaError> {
        match signal {
            ZhenfaSignal::Reward {
                episode_id,
                value,
                source,
            } => {
                let resolved_episode_id = if episode_id.trim().is_empty() {
                    ctx.correlation_id.clone().unwrap_or_default()
                } else {
                    episode_id
                };
                if resolved_episode_id.trim().is_empty() {
                    tracing::warn!(
                        event = "agent.zhenfa.signal.reward.skipped",
                        session_id = ?ctx.session_id,
                        trace_id = ?ctx.trace_id,
                        source = %source,
                        "skipping reward signal because episode id is missing"
                    );
                    return Ok(());
                }
                let reward = value.clamp(0.0, 1.0);
                let updated_q = self.memory_store.update_q(&resolved_episode_id, reward);
                if let Some(backend) = self.memory_state_backend.as_ref()
                    && let Err(error) = backend.update_q_atomic(&resolved_episode_id, updated_q)
                {
                    tracing::warn!(
                        event = "agent.memory.state.q_atomic_persist_failed",
                        backend = backend.backend_name(),
                        session_id = ?ctx.session_id,
                        reason = "zhenfa_reward_signal",
                        episode_id = %resolved_episode_id,
                        q_value = updated_q,
                        error = %error,
                        "failed to persist zhenfa reward q-value atomically"
                    );
                }
                tracing::debug!(
                    event = "agent.zhenfa.signal.reward.applied",
                    session_id = ?ctx.session_id,
                    trace_id = ?ctx.trace_id,
                    correlation_id = ?ctx.correlation_id,
                    episode_id = %resolved_episode_id,
                    source = %source,
                    reward,
                    updated_q,
                    "applied zhenfa reward signal to memory store"
                );
            }
            ZhenfaSignal::Trace { node_id, event } => {
                tracing::debug!(
                    event = "agent.zhenfa.signal.trace.received",
                    session_id = ?ctx.session_id,
                    trace_id = ?ctx.trace_id,
                    correlation_id = ?ctx.correlation_id,
                    node_id = %node_id,
                    trace_event = %event,
                    "received zhenfa trace signal"
                );
            }
        }
        Ok(())
    }
}

fn normalize_tool_arguments(arguments: Option<Value>) -> Result<Value> {
    match arguments {
        None => Ok(json!({})),
        Some(value) if value.is_object() => Ok(value),
        Some(_) => Err(anyhow!("tool arguments must be a JSON object")),
    }
}

fn should_retry_with_null_arguments(params: &Value) -> bool {
    params.as_object().is_some_and(serde_json::Map::is_empty)
}

fn build_native_tool(name: &str, deps: &ZhenfaRuntimeDeps) -> Option<Arc<dyn ZhenfaTool>> {
    match name {
        "wendao.search" => deps.link_graph_index.as_ref().map(|_index| {
            Arc::new(xiuxian_wendao::zhenfa_router::WendaoSearchTool) as Arc<dyn ZhenfaTool>
        }),
        "qianhuan.render" => deps.manifestation_manager.as_ref().map(|_manager| {
            Arc::new(xiuxian_qianhuan::zhenfa_router::QianhuanRenderTool) as Arc<dyn ZhenfaTool>
        }),
        "qianhuan.reload" => deps.manifestation_manager.as_ref().map(|_manager| {
            Arc::new(xiuxian_qianhuan::zhenfa_router::QianhuanReloadTool) as Arc<dyn ZhenfaTool>
        }),
        _ => None,
    }
}

fn resolve_zhenfa_base_url(config: &XiuxianConfig) -> Option<String> {
    if let Some(url) = config.zhenfa.base_url.as_deref() {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return None;
        }
        return Some(trimmed.to_string());
    }

    std::env::var(ZHENFA_BASE_URL_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn resolve_enabled_tool_names(config: &XiuxianConfig) -> Vec<String> {
    config.zhenfa.enabled_tools.as_ref().map_or_else(
        || vec!["wendao.search".to_string()],
        std::clone::Clone::clone,
    )
}
