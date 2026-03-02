//! High-level laboratory API for running Qianji workflows end-to-end.

use crate::contracts::QianjiManifest;
use crate::error::QianjiError;
use crate::scheduler::preflight::{RuntimeWendaoMount, install_runtime_wendao_mounts};
use crate::{QianjiApp, QianjiLlmClient};
#[cfg(feature = "llm")]
use async_trait::async_trait;
use include_dir::Dir;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
#[cfg(feature = "llm")]
use xiuxian_llm::llm::{ChatRequest, LlmClient, OpenAIClient};
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_wendao::{LinkGraphIndex, WendaoResourceUri, embedded_resource_text_from_wendao_uri};

#[cfg(feature = "llm")]
use crate::runtime_config::resolve_qianji_runtime_llm_config;

/// Runtime report returned by the Qianji laboratory API.
#[derive(Debug, Clone)]
pub struct WorkflowReport {
    /// Input canonical workflow URI.
    pub flow_uri: String,
    /// Parsed manifest name.
    pub manifest_name: String,
    /// Number of nodes declared by the manifest.
    pub node_count: usize,
    /// Number of edges declared by the manifest.
    pub edge_count: usize,
    /// Whether this manifest requires LLM capability.
    pub requires_llm: bool,
    /// UNIX epoch start timestamp in milliseconds.
    pub started_at_unix_ms: u128,
    /// UNIX epoch finish timestamp in milliseconds.
    pub finished_at_unix_ms: u128,
    /// End-to-end workflow execution duration in milliseconds.
    pub duration_ms: u128,
    /// Final merged workflow context after execution.
    pub final_context: Value,
}

/// LLM runtime mode for bootcamp workflow execution.
#[cfg(feature = "llm")]
#[derive(Clone, Default)]
pub enum BootcampLlmMode {
    /// Disable LLM client injection.
    ///
    /// Workflows requiring LLM nodes will fail at compile-time with a clear
    /// topology error.
    #[default]
    Disabled,
    /// Build an OpenAI-compatible client from `qianji.toml` runtime config.
    RuntimeDefault,
    /// Use one deterministic mock response for every chat completion call.
    Mock {
        /// Static completion payload returned for all requests.
        response: String,
    },
    /// Use one externally managed LLM client.
    External(Arc<QianjiLlmClient>),
}

/// LLM runtime mode for bootcamp workflow execution.
#[cfg(not(feature = "llm"))]
#[derive(Debug, Clone, Copy, Default)]
pub enum BootcampLlmMode {
    /// Disable LLM client injection.
    #[default]
    Disabled,
}

/// Optional runtime overrides for `run_workflow`.
#[derive(Clone)]
pub struct BootcampRunOptions {
    /// Optional project root for `LinkGraph` bootstrap.
    ///
    /// Resolution order when omitted:
    /// 1. `PRJ_ROOT` env var
    /// 2. process current working directory
    pub repo_path: Option<PathBuf>,
    /// Optional session id for checkpoint persistence.
    pub session_id: Option<String>,
    /// Optional `Valkey` URL used with `session_id`.
    pub redis_url: Option<String>,
    /// Genesis rules for default orchestrator construction.
    pub genesis_rules: String,
    /// Optional prebuilt `LinkGraph` index.
    pub index: Option<Arc<LinkGraphIndex>>,
    /// Optional prebuilt `Qianhuan` orchestrator.
    pub orchestrator: Option<Arc<ThousandFacesOrchestrator>>,
    /// Optional prebuilt persona registry.
    pub persona_registry: Option<Arc<PersonaRegistry>>,
    /// LLM runtime selection strategy.
    pub llm_mode: BootcampLlmMode,
    /// Optional manager for distributed consensus voting.
    pub consensus_manager: Option<Arc<crate::consensus::ConsensusManager>>,
}

impl BootcampRunOptions {
    /// Creates options with safe defaults for local bootcamp execution.
    #[must_use]
    pub fn new() -> Self {
        Self {
            repo_path: None,
            session_id: None,
            redis_url: None,
            genesis_rules: "Safety Rules".to_string(),
            index: None,
            orchestrator: None,
            persona_registry: None,
            llm_mode: BootcampLlmMode::Disabled,
            consensus_manager: None,
        }
    }
}

impl Default for BootcampRunOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// External VFS mount descriptor used by `run_scenario`.
#[derive(Debug, Clone, Copy)]
pub struct BootcampVfsMount {
    /// Semantic skill name used in URI host segment:
    /// `wendao://skills/<semantic_name>/references/...`.
    pub semantic_name: &'static str,
    /// References directory path inside `dir`, for example:
    /// `zhixing/skills/agenda-management/references`.
    pub references_dir: &'static str,
    /// Embedded directory exported by the source crate.
    pub dir: &'static Dir<'static>,
}

impl BootcampVfsMount {
    /// Creates one explicit mount descriptor.
    #[must_use]
    pub const fn new(
        semantic_name: &'static str,
        references_dir: &'static str,
        dir: &'static Dir<'static>,
    ) -> Self {
        Self {
            semantic_name,
            references_dir,
            dir,
        }
    }
}

impl From<BootcampVfsMount> for RuntimeWendaoMount {
    fn from(value: BootcampVfsMount) -> Self {
        Self {
            semantic_name: value.semantic_name,
            references_dir: value.references_dir,
            dir: value.dir,
        }
    }
}

/// Runs one workflow manifest resolved from a canonical `wendao://` URI.
///
/// This is the high-level "laboratory" entrypoint:
/// 1. resolve manifest URI from embedded Wendao resources,
/// 2. hydrate compiler dependencies (index/orchestrator/registry),
/// 3. compile and execute through `QianjiScheduler`,
/// 4. return execution metadata plus final context.
///
/// # Errors
///
/// Returns [`QianjiError`] when URI resolution, manifest parsing, dependency
/// bootstrap, workflow compilation, or runtime execution fails.
pub async fn run_workflow(
    flow_uri: &str,
    initial_context: Value,
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    run_workflow_with_mounts(flow_uri, initial_context, &[], options).await
}

/// Runs one workflow manifest with optional extra embedded VFS mounts.
///
/// Mounts are used during initial flow TOML loading. When the same URI exists
/// in both extra mounts and Wendao built-in embedded registry, extra mounts
/// take precedence.
///
/// # Errors
///
/// Returns [`QianjiError`] when URI resolution, manifest parsing, dependency
/// bootstrap, workflow compilation, or runtime execution fails.
pub async fn run_workflow_with_mounts(
    flow_uri: &str,
    initial_context: Value,
    vfs_mounts: &[BootcampVfsMount],
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    let trimmed_flow_uri = flow_uri.trim();
    if trimmed_flow_uri.is_empty() {
        return Err(QianjiError::Topology(
            "bootcamp flow URI must be non-empty".to_string(),
        ));
    }

    let manifest_toml = resolve_flow_manifest_toml(trimmed_flow_uri, vfs_mounts)?;
    let manifest = parse_manifest(manifest_toml.as_str())?;
    let requires_llm = parsed_manifest_requires_llm(&manifest);

    let BootcampRunOptions {
        repo_path,
        session_id,
        redis_url,
        genesis_rules,
        index,
        orchestrator,
        persona_registry,
        llm_mode,
        consensus_manager,
    } = options;

    let index = match index {
        Some(index) => index,
        None => Arc::new(build_link_graph_index(repo_path.as_deref())?),
    };
    let orchestrator = orchestrator
        .unwrap_or_else(|| Arc::new(ThousandFacesOrchestrator::new(genesis_rules, None)));
    let registry = persona_registry.unwrap_or_else(|| Arc::new(PersonaRegistry::with_builtins()));
    let llm_client = resolve_bootcamp_llm_client(requires_llm, llm_mode)?;
    let scheduler = QianjiApp::create_pipeline_from_manifest_with_consensus(
        manifest_toml.as_str(),
        index,
        orchestrator,
        registry,
        llm_client,
        consensus_manager,
    )?;
    let runtime_mounts = vfs_mounts
        .iter()
        .copied()
        .map(RuntimeWendaoMount::from)
        .collect::<Vec<_>>();
    let _mount_guard = install_runtime_wendao_mounts(runtime_mounts);

    let started_at_unix_ms = unix_timestamp_millis()?;
    let started_at = Instant::now();
    let final_context = scheduler
        .run_with_checkpoint(initial_context, session_id, redis_url)
        .await?;
    let finished_at_unix_ms = unix_timestamp_millis()?;
    let duration_ms = started_at.elapsed().as_millis();

    Ok(WorkflowReport {
        flow_uri: trimmed_flow_uri.to_string(),
        manifest_name: manifest.name,
        node_count: manifest.nodes.len(),
        edge_count: manifest.edges.len(),
        requires_llm,
        started_at_unix_ms,
        finished_at_unix_ms,
        duration_ms,
        final_context,
    })
}

/// Compatibility alias of [`run_workflow`] for scenario-style callers.
///
/// This API accepts extra `include_dir` mounts so domain crates can provide
/// embedded resources directly without requiring hardcoded path wiring.
///
/// # Errors
///
/// Returns the same errors as [`run_workflow_with_mounts`].
pub async fn run_scenario(
    flow_uri: &str,
    initial_context: Value,
    vfs_mounts: &[BootcampVfsMount],
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    run_workflow_with_mounts(flow_uri, initial_context, vfs_mounts, options).await
}

fn parse_manifest(manifest_toml: &str) -> Result<QianjiManifest, QianjiError> {
    toml::from_str(manifest_toml)
        .map_err(|error| QianjiError::Topology(format!("Failed to parse TOML: {error}")))
}

fn normalize_relative_path(path: &str) -> String {
    path.trim().trim_start_matches("./").replace('\\', "/")
}

fn resolve_flow_manifest_from_mounts(
    flow_uri: &str,
    vfs_mounts: &[BootcampVfsMount],
) -> Option<String> {
    let parsed = WendaoResourceUri::parse(flow_uri).ok()?;
    let semantic_name = parsed.semantic_name();
    let entity_relative_path =
        normalize_relative_path(parsed.entity_relative_path().to_string_lossy().as_ref());

    for mount in vfs_mounts {
        if !semantic_name.eq_ignore_ascii_case(mount.semantic_name) {
            continue;
        }
        let references_dir = normalize_relative_path(mount.references_dir);
        if references_dir.is_empty() {
            continue;
        }
        let candidate_path = format!("{references_dir}/{entity_relative_path}");
        let Some(content) = mount
            .dir
            .get_file(candidate_path.as_str())
            .and_then(include_dir::File::contents_utf8)
        else {
            continue;
        };
        return Some(content.to_string());
    }
    None
}

fn resolve_flow_manifest_toml(
    flow_uri: &str,
    vfs_mounts: &[BootcampVfsMount],
) -> Result<String, QianjiError> {
    if let Some(content) = resolve_flow_manifest_from_mounts(flow_uri, vfs_mounts) {
        return Ok(content);
    }
    if let Some(content) = embedded_resource_text_from_wendao_uri(flow_uri) {
        return Ok(content.to_string());
    }
    Err(QianjiError::Topology(format!(
        "semantic flow manifest not found for URI `{flow_uri}`"
    )))
}

fn parsed_manifest_requires_llm(manifest: &QianjiManifest) -> bool {
    manifest.nodes.iter().any(|node| {
        if node.task_type.trim().eq_ignore_ascii_case("llm") {
            return true;
        }
        node.task_type.trim().eq_ignore_ascii_case("formal_audit")
            && node.qianhuan.is_some()
            && node.llm.is_some()
    })
}

fn unix_timestamp_millis() -> Result<u128, QianjiError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|error| {
            QianjiError::Execution(format!("system clock drifted before UNIX_EPOCH: {error}"))
        })
}

fn resolve_repo_root_path(explicit: Option<&Path>) -> PathBuf {
    if let Some(path) = explicit {
        return path.to_path_buf();
    }
    if let Ok(path) = std::env::var("PRJ_ROOT") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    std::env::current_dir().unwrap_or_else(|_error| std::env::temp_dir())
}

fn build_link_graph_index(
    explicit_repo_root: Option<&Path>,
) -> Result<LinkGraphIndex, QianjiError> {
    let primary_root = resolve_repo_root_path(explicit_repo_root);
    match LinkGraphIndex::build(primary_root.as_path()) {
        Ok(index) => Ok(index),
        Err(primary_error) => {
            let fallback_root = std::env::temp_dir();
            LinkGraphIndex::build(fallback_root.as_path()).map_err(|fallback_error| {
                QianjiError::Topology(format!(
                    "failed to build LinkGraph index at `{}` ({primary_error}); \
fallback `{}` also failed ({fallback_error})",
                    primary_root.display(),
                    fallback_root.display()
                ))
            })
        }
    }
}

#[cfg(feature = "llm")]
fn resolve_bootcamp_llm_client(
    requires_llm: bool,
    llm_mode: BootcampLlmMode,
) -> Result<Option<Arc<QianjiLlmClient>>, QianjiError> {
    match llm_mode {
        BootcampLlmMode::Disabled => {
            if requires_llm {
                return Err(QianjiError::Topology(
                    "workflow requires LLM, but bootcamp llm_mode is disabled".to_string(),
                ));
            }
            Ok(None)
        }
        BootcampLlmMode::RuntimeDefault => {
            let runtime = resolve_qianji_runtime_llm_config().map_err(|error| {
                QianjiError::Topology(format!(
                    "failed to resolve qianji llm runtime config for bootcamp: {error}"
                ))
            })?;
            let client: Arc<QianjiLlmClient> = Arc::new(OpenAIClient {
                api_key: runtime.api_key,
                base_url: runtime.base_url,
                http: reqwest::Client::new(),
            });
            Ok(Some(client))
        }
        BootcampLlmMode::Mock { response } => {
            let client: Arc<QianjiLlmClient> = Arc::new(MockBootcampLlmClient { response });
            Ok(Some(client))
        }
        BootcampLlmMode::External(client) => Ok(Some(client)),
    }
}

#[cfg(not(feature = "llm"))]
fn resolve_bootcamp_llm_client(
    requires_llm: bool,
    _llm_mode: BootcampLlmMode,
) -> Result<Option<Arc<QianjiLlmClient>>, QianjiError> {
    if requires_llm {
        return Err(QianjiError::Topology(
            "workflow requires LLM; enable feature `llm` for xiuxian-qianji".to_string(),
        ));
    }
    Ok(None)
}

#[cfg(feature = "llm")]
#[derive(Debug, Clone)]
struct MockBootcampLlmClient {
    response: String,
}

#[cfg(feature = "llm")]
#[async_trait]
impl LlmClient for MockBootcampLlmClient {
    async fn chat(&self, _request: ChatRequest) -> anyhow::Result<String> {
        Ok(self.response.clone())
    }
}
