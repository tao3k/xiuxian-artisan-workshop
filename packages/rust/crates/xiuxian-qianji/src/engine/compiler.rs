//! Compiler for declarative Qianji manifests.

use crate::QianjiLlmClient;
#[cfg(feature = "llm")]
use crate::contracts::NodeLlmBinding;
use crate::contracts::{
    EdgeDefinition, NodeDefinition, NodeQianhuanExecutionMode, QianjiManifest, QianjiMechanism,
};
use crate::engine::{NodeExecutionAffinity, QianjiEngine};
use crate::error::QianjiError;
use crate::executors::annotation::ContextAnnotator;
use crate::executors::calibration::SynapseCalibrator;
use crate::executors::knowledge::KnowledgeSeeker;
#[cfg(feature = "llm")]
use crate::runtime_config::resolve_qianji_runtime_llm_config;
use crate::runtime_config::resolve_qianji_runtime_wendao_ingester_config;
use petgraph::stable_graph::NodeIndex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::backend::{LlmBackendKind, parse_llm_backend_kind};
#[cfg(feature = "llm")]
use xiuxian_llm::llm::{LlmClient, OpenAIClient};
use xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator;
use xiuxian_qianhuan::persona::PersonaRegistry;
use xiuxian_wendao::LinkGraphIndex;

/// Orchestrates the conversion of TOML manifests into executable engines.
pub struct QianjiCompiler {
    index: Arc<LinkGraphIndex>,
    orchestrator: Arc<ThousandFacesOrchestrator>,
    registry: Arc<PersonaRegistry>,
    #[cfg_attr(not(feature = "llm"), allow(dead_code))]
    llm_client: Option<Arc<QianjiLlmClient>>,
}

impl QianjiCompiler {
    /// Creates a new compiler with provided trinity dependencies.
    #[must_use]
    pub fn new(
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        llm_client: Option<Arc<QianjiLlmClient>>,
    ) -> Self {
        Self {
            index,
            orchestrator,
            registry,
            llm_client,
        }
    }

    fn parse_manifest(manifest_toml: &str) -> Result<QianjiManifest, QianjiError> {
        toml::from_str(manifest_toml)
            .map_err(|error| QianjiError::Topology(format!("Failed to parse TOML: {error}")))
    }

    fn non_empty(value: Option<&str>) -> Option<String> {
        value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    }

    fn resolve_semantic_placeholder(raw: &str) -> String {
        raw.trim().to_string()
    }

    fn annotation_persona_id(node_def: &NodeDefinition) -> String {
        Self::non_empty(
            node_def
                .qianhuan
                .as_ref()
                .and_then(|binding| binding.persona_id.as_deref()),
        )
        .map_or_else(
            || "artisan-engineer".to_string(),
            |value| Self::resolve_semantic_placeholder(value.as_str()),
        )
    }

    fn annotation_template_target(node_def: &NodeDefinition) -> Option<String> {
        Self::non_empty(
            node_def
                .qianhuan
                .as_ref()
                .and_then(|binding| binding.template_target.as_deref()),
        )
        .map(|value| Self::resolve_semantic_placeholder(value.as_str()))
    }

    fn annotation_execution_mode(node_def: &NodeDefinition) -> NodeQianhuanExecutionMode {
        node_def
            .qianhuan
            .as_ref()
            .map_or(NodeQianhuanExecutionMode::Isolated, |binding| {
                binding.execution_mode.clone()
            })
    }

    fn annotation_history_key(node_def: &NodeDefinition) -> String {
        Self::non_empty(
            node_def
                .qianhuan
                .as_ref()
                .and_then(|binding| binding.history_key.as_deref()),
        )
        .unwrap_or_else(|| "qianhuan_history".to_string())
    }

    fn annotation_output_key(node_def: &NodeDefinition) -> String {
        Self::non_empty(
            node_def
                .qianhuan
                .as_ref()
                .and_then(|binding| binding.output_key.as_deref()),
        )
        .unwrap_or_else(|| "annotated_prompt".to_string())
    }

    fn node_param_string(node_def: &NodeDefinition, key: &str) -> Option<String> {
        Self::non_empty(node_def.params.get(key).and_then(serde_json::Value::as_str))
            .map(|value| Self::resolve_semantic_placeholder(value.as_str()))
    }

    fn derive_role_class_from_persona(node_def: &NodeDefinition) -> Option<String> {
        let persona_id = Self::non_empty(
            node_def
                .qianhuan
                .as_ref()
                .and_then(|binding| binding.persona_id.as_deref()),
        )?;

        let resolved = Self::resolve_semantic_placeholder(persona_id.as_str());
        let stripped = resolved
            .trim_start_matches('$')
            .trim_end_matches('/')
            .trim();
        if stripped.is_empty() {
            return None;
        }

        let file_name = stripped.rsplit('/').next().unwrap_or(stripped);
        let role_name = file_name.strip_suffix(".md").unwrap_or(file_name).trim();
        if role_name.is_empty() {
            None
        } else {
            Some(role_name.to_ascii_lowercase())
        }
    }

    fn node_execution_affinity(node_def: &NodeDefinition) -> NodeExecutionAffinity {
        let agent_id = Self::node_param_string(node_def, "agent_id")
            .or_else(|| Self::node_param_string(node_def, "executor_agent_id"));
        let role_class = Self::node_param_string(node_def, "role_class")
            .or_else(|| Self::node_param_string(node_def, "agent_role"))
            .or_else(|| Self::derive_role_class_from_persona(node_def));

        NodeExecutionAffinity {
            agent_id,
            role_class,
        }
    }

    fn annotation_input_keys(node_def: &NodeDefinition) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut keys = node_def
            .qianhuan
            .as_ref()
            .map(|binding| {
                binding
                    .input_keys
                    .iter()
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .filter(|value| seen.insert((*value).to_string()))
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if keys.is_empty() {
            keys.push("raw_facts".to_string());
        }
        keys
    }

    #[cfg(feature = "llm")]
    fn llm_model(node_def: &NodeDefinition) -> String {
        Self::non_empty(
            node_def
                .llm
                .as_ref()
                .and_then(|binding| binding.model.as_deref()),
        )
        .or_else(|| {
            Self::non_empty(
                node_def
                    .params
                    .get("model")
                    .and_then(serde_json::Value::as_str),
            )
        })
        .unwrap_or_default()
    }

    #[cfg(feature = "llm")]
    fn llm_provider_kind(binding: Option<&NodeLlmBinding>) -> Result<LlmBackendKind, QianjiError> {
        let raw = binding.and_then(|config| config.provider.as_deref());
        if let Some(provider) = raw {
            return parse_llm_backend_kind(Some(provider)).ok_or_else(|| {
                QianjiError::Topology(format!("Unsupported LLM provider for node: {provider}"))
            });
        }
        Ok(LlmBackendKind::OpenAiCompatibleHttp)
    }

    #[cfg(feature = "llm")]
    fn has_dedicated_llm_endpoint(binding: Option<&NodeLlmBinding>) -> bool {
        binding.is_some_and(|config| {
            Self::non_empty(config.base_url.as_deref()).is_some()
                || Self::non_empty(config.api_key_env.as_deref()).is_some()
        })
    }

    #[cfg(feature = "llm")]
    fn resolve_node_llm_endpoint(
        binding: Option<&NodeLlmBinding>,
    ) -> Result<Option<(String, String)>, QianjiError> {
        if !Self::has_dedicated_llm_endpoint(binding) {
            return Ok(None);
        }

        let provider = Self::llm_provider_kind(binding)?;
        if provider == LlmBackendKind::LiteLlmRs {
            return Err(QianjiError::Topology(
                "Node-level provider 'litellm_rs' is not yet supported in xiuxian-qianji; use openai-compatible endpoints for now.".to_string(),
            ));
        }

        let runtime = resolve_qianji_runtime_llm_config().ok();
        let binding_base_url =
            binding.and_then(|config| Self::non_empty(config.base_url.as_deref()));
        let base_url = binding_base_url
            .or_else(|| runtime.as_ref().map(|cfg| cfg.base_url.clone()))
            .ok_or_else(|| {
                QianjiError::Topology(
                    "Node-level LLM endpoint requires `base_url` in [nodes.llm] or global qianji runtime config.".to_string(),
                )
            })?;

        let binding_api_key_env =
            binding.and_then(|config| Self::non_empty(config.api_key_env.as_deref()));
        let api_key_env = binding_api_key_env
            .or_else(|| runtime.as_ref().map(|cfg| cfg.api_key_env.clone()))
            .unwrap_or_else(|| "OPENAI_API_KEY".to_string());

        let api_key = std::env::var(&api_key_env)
            .ok()
            .and_then(|value| Self::non_empty(Some(value.as_str())))
            .or_else(|| {
                std::env::var("OPENAI_API_KEY")
                    .ok()
                    .and_then(|value| Self::non_empty(Some(value.as_str())))
            })
            .or_else(|| runtime.map(|cfg| cfg.api_key))
            .ok_or_else(|| {
                QianjiError::Topology(format!(
                    "Missing API key for node-level LLM endpoint; set {api_key_env} or OPENAI_API_KEY."
                ))
            })?;

        Ok(Some((base_url, api_key)))
    }

    #[cfg(feature = "llm")]
    fn resolve_llm_client_for_node(
        &self,
        node_def: &NodeDefinition,
    ) -> Result<Arc<dyn LlmClient>, QianjiError> {
        let binding = node_def.llm.as_ref();
        if let Some((base_url, api_key)) = Self::resolve_node_llm_endpoint(binding)? {
            let http = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
            return Ok(Arc::new(OpenAIClient {
                api_key,
                base_url,
                http,
            }));
        }

        self.llm_client.clone().ok_or(QianjiError::Topology(
            "LLM client not provided to compiler".to_string(),
        ))
    }

    fn calibration_target_node_id(node_def: &NodeDefinition) -> String {
        node_def
            .params
            .get("target_node_id")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string()
    }

    fn router_branches(node_def: &NodeDefinition) -> Result<Vec<(String, f32)>, QianjiError> {
        let mut branches = Vec::new();
        if let Some(branches_config) = node_def.params["branches"].as_array() {
            for item in branches_config {
                let Some(branch) = item.as_array() else {
                    continue;
                };
                let Some(name) = branch.first().and_then(serde_json::Value::as_str) else {
                    continue;
                };
                let Some(weight) = branch.get(1) else {
                    continue;
                };
                branches.push((name.to_string(), to_branch_weight(weight)?));
            }
        }
        Ok(branches)
    }

    fn build_knowledge_mechanism(&self) -> Arc<dyn QianjiMechanism> {
        Arc::new(KnowledgeSeeker {
            index: self.index.clone(),
        })
    }

    fn build_annotation_mechanism(&self, node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        Arc::new(ContextAnnotator {
            orchestrator: self.orchestrator.clone(),
            registry: self.registry.clone(),
            persona_id: Self::annotation_persona_id(node_def),
            template_target: Self::annotation_template_target(node_def),
            execution_mode: Self::annotation_execution_mode(node_def),
            input_keys: Self::annotation_input_keys(node_def),
            history_key: Self::annotation_history_key(node_def),
            output_key: Self::annotation_output_key(node_def),
        })
    }

    fn build_calibration_mechanism(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        Arc::new(SynapseCalibrator {
            target_node_id: Self::calibration_target_node_id(node_def),
            drift_threshold: 0.5,
        })
    }

    fn formal_audit_retry_targets(node_def: &NodeDefinition) -> Vec<String> {
        node_def
            .params
            .get("retry_targets")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(std::string::ToString::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn formal_audit_uses_llm_controller(node_def: &NodeDefinition) -> bool {
        node_def.qianhuan.is_some() && node_def.llm.is_some()
    }

    #[cfg(feature = "llm")]
    fn formal_audit_threshold_score(node_def: &NodeDefinition) -> Result<f32, QianjiError> {
        let raw = node_def
            .params
            .get("threshold_score")
            .map_or(Ok(0.8_f32), |value| {
                serde_json::from_value::<f32>(value.clone()).map_err(|_error| {
                    QianjiError::Topology(
                        "formal_audit.threshold_score must be a finite number".to_string(),
                    )
                })
            })?;
        if !raw.is_finite() {
            return Err(QianjiError::Topology(
                "formal_audit.threshold_score must be a finite number".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&raw) {
            return Err(QianjiError::Topology(
                "formal_audit.threshold_score must be within [0.0, 1.0]".to_string(),
            ));
        }
        Ok(raw)
    }

    #[cfg(feature = "llm")]
    fn formal_audit_output_key(node_def: &NodeDefinition) -> String {
        node_def
            .params
            .get("output_key")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("audit_critique")
            .to_string()
    }

    #[cfg(feature = "llm")]
    fn formal_audit_max_retries(node_def: &NodeDefinition) -> Result<u32, QianjiError> {
        let raw = node_def
            .params
            .get("max_retries")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(3);
        u32::try_from(raw).map_err(|_error| {
            QianjiError::Topology("formal_audit.max_retries must fit in u32".to_string())
        })
    }

    #[cfg(feature = "llm")]
    fn formal_audit_retry_counter_key(node_def: &NodeDefinition) -> String {
        node_def
            .params
            .get("retry_counter_key")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("audit_retry_count")
            .to_string()
    }

    #[cfg(feature = "llm")]
    fn formal_audit_score_key(node_def: &NodeDefinition) -> String {
        node_def
            .params
            .get("score_key")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("audit_score")
            .to_string()
    }

    #[cfg_attr(not(feature = "llm"), allow(clippy::unused_self))]
    fn build_formal_audit_mechanism(
        &self,
        node_def: &NodeDefinition,
    ) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
        let targets = Self::formal_audit_retry_targets(node_def);

        #[cfg(feature = "llm")]
        if Self::formal_audit_uses_llm_controller(node_def) {
            let threshold_score = Self::formal_audit_threshold_score(node_def)?;
            let max_retries = Self::formal_audit_max_retries(node_def)?;
            let client = self.resolve_llm_client_for_node(node_def)?;
            return Ok(Arc::new(
                crate::executors::formal_audit::LlmAugmentedAuditMechanism {
                    annotator: ContextAnnotator {
                        orchestrator: self.orchestrator.clone(),
                        registry: self.registry.clone(),
                        persona_id: Self::annotation_persona_id(node_def),
                        template_target: Self::annotation_template_target(node_def),
                        execution_mode: Self::annotation_execution_mode(node_def),
                        input_keys: Self::annotation_input_keys(node_def),
                        history_key: Self::annotation_history_key(node_def),
                        output_key: Self::annotation_output_key(node_def),
                    },
                    client,
                    model: Self::llm_model(node_def),
                    threshold_score,
                    max_retries,
                    retry_target_ids: targets.clone(),
                    retry_counter_key: Self::formal_audit_retry_counter_key(node_def),
                    output_key: Self::formal_audit_output_key(node_def),
                    score_key: Self::formal_audit_score_key(node_def),
                },
            ));
        }

        #[cfg(not(feature = "llm"))]
        if Self::formal_audit_uses_llm_controller(node_def) {
            return Err(QianjiError::Topology(
                "Task type `formal_audit` with `[nodes.qianhuan] + [nodes.llm]` requires enabling feature `llm` for xiuxian-qianji.".to_string(),
            ));
        }

        Ok(Arc::new(
            crate::executors::formal_audit::FormalAuditMechanism {
                invariants: vec![crate::safety::logic::Invariant::MustBeGrounded],
                retry_target_ids: targets,
            },
        ))
    }

    fn build_command_mechanism(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        let cmd = node_def
            .params
            .get("cmd")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let output_key = node_def
            .params
            .get("output_key")
            .and_then(|v| v.as_str())
            .unwrap_or("stdout")
            .to_string();
        let allow_fail = node_def
            .params
            .get("allow_fail")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let stop_on_empty_stdout = node_def
            .params
            .get("stop_on_empty_stdout")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let empty_reason = node_def
            .params
            .get("empty_reason")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        Arc::new(crate::executors::command::ShellMechanism {
            cmd,
            allow_fail,
            stop_on_empty_stdout,
            empty_reason,
            output_key,
        })
    }

    fn build_write_file_mechanism(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        let path = node_def
            .params
            .get("path")
            .and_then(|v| v.as_str())
            .or_else(|| node_def.params.get("target_path").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string();
        let content = node_def
            .params
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let output_key = node_def
            .params
            .get("output_key")
            .and_then(|v| v.as_str())
            .unwrap_or("write_file_result")
            .to_string();

        Arc::new(crate::executors::write_file::WriteFileMechanism {
            path,
            content,
            output_key,
        })
    }

    fn build_suspend_mechanism(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        let reason = node_def
            .params
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("suspended")
            .to_string();
        let prompt = node_def
            .params
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("Waiting for input...")
            .to_string();
        let resume_key = node_def
            .params
            .get("resume_key")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        Arc::new(crate::executors::suspend::SuspendMechanism {
            reason,
            prompt,
            resume_key,
        })
    }

    fn build_security_scan_mechanism(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        let files_key = node_def
            .params
            .get("files_key")
            .and_then(|v| v.as_str())
            .unwrap_or("staged_files")
            .to_string();
        let output_key = node_def
            .params
            .get("output_key")
            .and_then(|v| v.as_str())
            .unwrap_or("security_issues")
            .to_string();
        let abort_on_violation = node_def
            .params
            .get("abort_on_violation")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let cwd_key = node_def
            .params
            .get("cwd_key")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        Arc::new(crate::executors::security_scan::SecurityScanMechanism {
            files_key,
            output_key,
            abort_on_violation,
            cwd_key,
        })
    }

    fn build_wendao_ingester_mechanism(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        let runtime_defaults = resolve_qianji_runtime_wendao_ingester_config()
            .unwrap_or_else(|error| {
                log::warn!(
                    "failed to resolve qianji memory promotion runtime config; using hard defaults: {error}"
                );
                crate::runtime_config::QianjiRuntimeWendaoIngesterConfig::default()
            });
        let output_key = node_def
            .params
            .get("output_key")
            .and_then(|v| v.as_str())
            .unwrap_or("promotion_entity")
            .to_string();
        let graph_scope = node_def
            .params
            .get("graph_scope")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .or_else(|| Some(runtime_defaults.graph_scope.clone()));
        let graph_scope_key = node_def
            .params
            .get("graph_scope_key")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .or_else(|| runtime_defaults.graph_scope_key.clone());
        let graph_dimension = node_def
            .params
            .get("graph_dimension")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(runtime_defaults.graph_dimension);
        let persist = node_def
            .params
            .get("persist")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(runtime_defaults.persist);
        let persist_best_effort = node_def
            .params
            .get("persist_best_effort")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(runtime_defaults.persist_best_effort);

        Arc::new(crate::executors::wendao_ingester::WendaoIngesterMechanism {
            output_key,
            graph_scope,
            graph_scope_key,
            graph_dimension,
            persist,
            persist_best_effort,
        })
    }

    fn build_wendao_refresh_mechanism(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        let output_key = node_def
            .params
            .get("output_key")
            .and_then(|v| v.as_str())
            .unwrap_or("wendao_refresh")
            .to_string();
        let changed_paths_key = node_def
            .params
            .get("changed_paths_key")
            .and_then(|v| v.as_str())
            .unwrap_or("changed_paths")
            .to_string();
        let root_dir_key = node_def
            .params
            .get("root_dir_key")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        let root_dir = node_def
            .params
            .get("root_dir")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        let force_full = node_def
            .params
            .get("force_full")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let prefer_incremental = node_def
            .params
            .get("prefer_incremental")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let allow_full_fallback = node_def
            .params
            .get("allow_full_fallback")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let full_rebuild_threshold = node_def
            .params
            .get("full_rebuild_threshold")
            .and_then(serde_json::Value::as_u64)
            .and_then(|raw| usize::try_from(raw).ok());
        let include_dirs = node_def
            .params
            .get("include_dirs")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let excluded_dirs = node_def
            .params
            .get("excluded_dirs")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Arc::new(crate::executors::wendao_refresh::WendaoRefreshMechanism {
            output_key,
            changed_paths_key,
            root_dir_key,
            root_dir,
            force_full,
            prefer_incremental,
            allow_full_fallback,
            full_rebuild_threshold,
            include_dirs,
            excluded_dirs,
        })
    }

    #[cfg_attr(not(feature = "llm"), allow(clippy::unused_self))]
    fn build_llm_mechanism(
        &self,
        node_def: &NodeDefinition,
    ) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
        #[cfg(feature = "llm")]
        {
            let model = Self::llm_model(node_def);
            let prompt_template = node_def
                .params
                .get("prompt")
                .and_then(|value| value.as_str())
                .unwrap_or("You are an expert analyst. Context: {{annotated_prompt}}")
                .to_string();
            let output_key = node_def
                .params
                .get("output_key")
                .and_then(|value| value.as_str())
                .unwrap_or("analysis_conclusion")
                .to_string();
            let context_keys = node_def
                .params
                .get("context_keys")
                .and_then(|v| v.as_array())
                .map_or_else(
                    || vec!["annotated_prompt".to_string()],
                    |arr| {
                        arr.iter()
                            .filter_map(|s| s.as_str().map(ToString::to_string))
                            .collect()
                    },
                );
            let parse_json_output = node_def
                .params
                .get("parse_json_output")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let fallback_repo_tree_on_parse_failure = node_def
                .params
                .get("fallback_repo_tree_on_parse_failure")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);

            let client = self.resolve_llm_client_for_node(node_def)?;
            Ok(Arc::new(crate::executors::llm::LlmAnalyzer {
                client,
                model,
                context_keys,
                prompt_template,
                output_key,
                parse_json_output,
                fallback_repo_tree_on_parse_failure,
            }))
        }
        #[cfg(not(feature = "llm"))]
        {
            let _ = node_def;
            Err(QianjiError::Topology(
                "Task type 'llm' requires enabling feature 'llm' for xiuxian-qianji".to_string(),
            ))
        }
    }

    fn build_mock_mechanism(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        Arc::new(crate::executors::MockMechanism {
            name: node_def.id.clone(),
            weight: node_def.weight,
        })
    }

    fn build_router_mechanism(
        node_def: &NodeDefinition,
    ) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
        let branches = Self::router_branches(node_def)?;
        Ok(Arc::new(crate::executors::router::ProbabilisticRouter {
            branches,
        }))
    }

    fn build_mechanism(
        &self,
        node_def: &NodeDefinition,
    ) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
        match node_def.task_type.as_str() {
            "knowledge" => Ok(self.build_knowledge_mechanism()),
            "annotation" => Ok(self.build_annotation_mechanism(node_def)),
            "calibration" => Ok(Self::build_calibration_mechanism(node_def)),
            "formal_audit" => self.build_formal_audit_mechanism(node_def),
            "llm" => self.build_llm_mechanism(node_def),
            "mock" => Ok(Self::build_mock_mechanism(node_def)),
            "command" => Ok(Self::build_command_mechanism(node_def)),
            "write_file" => Ok(Self::build_write_file_mechanism(node_def)),
            "suspend" => Ok(Self::build_suspend_mechanism(node_def)),
            "security_scan" => Ok(Self::build_security_scan_mechanism(node_def)),
            "wendao_ingester" => Ok(Self::build_wendao_ingester_mechanism(node_def)),
            "wendao_refresh" => Ok(Self::build_wendao_refresh_mechanism(node_def)),
            "router" => Self::build_router_mechanism(node_def),
            _ => Err(QianjiError::Topology(format!(
                "Unknown task type: {}",
                node_def.task_type
            ))),
        }
    }

    fn add_manifest_nodes(
        &self,
        engine: &mut QianjiEngine,
        node_defs: Vec<NodeDefinition>,
    ) -> Result<HashMap<String, NodeIndex>, QianjiError> {
        let mut id_to_index = HashMap::new();
        for node_def in node_defs {
            let consensus = node_def.consensus.clone();
            let mechanism = self.build_mechanism(&node_def)?;
            let execution_affinity = Self::node_execution_affinity(&node_def);
            let idx = engine.add_mechanism_with_affinity(
                &node_def.id,
                mechanism,
                consensus,
                execution_affinity,
            );
            id_to_index.insert(node_def.id, idx);
        }
        Ok(id_to_index)
    }

    fn node_index_by_id(
        id_to_index: &HashMap<String, NodeIndex>,
        node_id: &str,
        role: &str,
    ) -> Result<NodeIndex, QianjiError> {
        id_to_index
            .get(node_id)
            .copied()
            .ok_or(QianjiError::Topology(format!(
                "{role} node not found: {node_id}"
            )))
    }

    fn add_manifest_edges(
        engine: &mut QianjiEngine,
        id_to_index: &HashMap<String, NodeIndex>,
        edge_defs: Vec<EdgeDefinition>,
    ) -> Result<(), QianjiError> {
        for edge_def in edge_defs {
            let from_idx = Self::node_index_by_id(id_to_index, &edge_def.from, "Source")?;
            let to_idx = Self::node_index_by_id(id_to_index, &edge_def.to, "Target")?;
            engine.add_link(from_idx, to_idx, edge_def.label.as_deref(), edge_def.weight);
        }
        Ok(())
    }

    /// Compiles a TOML manifest into a ready-to-run `QianjiEngine`.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when TOML parsing fails, a task type is unsupported,
    /// required dependencies are missing, manifest edges reference unknown nodes,
    /// or the graph contains static cycles.
    pub fn compile(&self, manifest_toml: &str) -> Result<QianjiEngine, QianjiError> {
        let manifest = Self::parse_manifest(manifest_toml)?;
        let mut engine = QianjiEngine::new();
        let id_to_index = self.add_manifest_nodes(&mut engine, manifest.nodes)?;
        Self::add_manifest_edges(&mut engine, &id_to_index, manifest.edges)?;

        if petgraph::algo::is_cyclic_directed(&engine.graph) {
            return Err(QianjiError::Topology(
                "Manifest contains a static cycle".to_string(),
            ));
        }

        Ok(engine)
    }
}

fn to_branch_weight(weight: &serde_json::Value) -> Result<f32, QianjiError> {
    let weight = serde_json::from_value::<f32>(weight.clone()).map_err(|_error| {
        QianjiError::Topology(
            "Router branch weight must be a finite number within f32 range".to_string(),
        )
    })?;
    if !weight.is_finite() {
        return Err(QianjiError::Topology(
            "Router branch weight must be a finite number".to_string(),
        ));
    }
    Ok(weight)
}
