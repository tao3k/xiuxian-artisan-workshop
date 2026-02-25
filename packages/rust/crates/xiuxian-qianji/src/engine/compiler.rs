//! Compiler for declarative Qianji manifests.

use crate::QianjiLlmClient;
use crate::contracts::{EdgeDefinition, NodeDefinition, QianjiManifest, QianjiMechanism};
use crate::engine::QianjiEngine;
use crate::error::QianjiError;
use crate::executors::annotation::ContextAnnotator;
use crate::executors::calibration::SynapseCalibrator;
use crate::executors::knowledge::KnowledgeSeeker;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashMap;
use std::sync::Arc;
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
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
            .map_err(|error| QianjiError::TopologyError(format!("Failed to parse TOML: {error}")))
    }

    fn annotation_persona_id(node_def: &NodeDefinition) -> String {
        node_def
            .params
            .get("persona_id")
            .and_then(|value| value.as_str())
            .unwrap_or("artisan-engineer")
            .to_string()
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
                if let (Some(name), Some(weight)) = (item[0].as_str(), item[1].as_f64()) {
                    branches.push((name.to_string(), to_branch_weight(weight)?));
                }
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
        })
    }

    fn build_calibration_mechanism(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        Arc::new(SynapseCalibrator {
            target_node_id: Self::calibration_target_node_id(node_def),
            drift_threshold: 0.5,
        })
    }

    fn build_formal_audit_mechanism(&self, node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
        let targets = node_def
            .params
            .get("retry_targets")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(std::string::ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();

        Arc::new(crate::executors::formal_audit::FormalAuditMechanism {
            invariants: vec![crate::safety::logic::Invariant::MustBeGrounded],
            retry_target_ids: targets,
        })
    }

    #[cfg_attr(not(feature = "llm"), allow(clippy::unused_self))]
    fn build_llm_mechanism(
        &self,
        node_def: &NodeDefinition,
    ) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
        #[cfg(feature = "llm")]
        {
            let model = node_def
                .params
                .get("model")
                .and_then(|value| value.as_str())
                .unwrap_or("gpt-4o")
                .to_string();
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
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(ToString::to_string))
                        .collect()
                })
                .unwrap_or_else(|| vec!["annotated_prompt".to_string()]);

            let client = self.llm_client.clone().ok_or(QianjiError::TopologyError(
                "LLM client not provided to compiler".to_string(),
            ))?;
            Ok(Arc::new(crate::executors::llm::LlmAnalyzer {
                client,
                model,
                prompt_template,
                output_key,
                context_keys,
            }))
        }
        #[cfg(not(feature = "llm"))]
        {
            let _ = node_def;
            Err(QianjiError::TopologyError(
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
            "formal_audit" => Ok(self.build_formal_audit_mechanism(node_def)),
            "llm" => self.build_llm_mechanism(node_def),
            "mock" => Ok(Self::build_mock_mechanism(node_def)),
            "command" => {
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
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let stop_on_empty_stdout = node_def
                    .params
                    .get("stop_on_empty_stdout")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let empty_reason = node_def
                    .params
                    .get("empty_reason")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string);

                Ok(Arc::new(crate::executors::command::ShellMechanism {
                    cmd,
                    allow_fail,
                    stop_on_empty_stdout,
                    empty_reason,
                    output_key,
                }))
            }
            "suspend" => {
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
                    .map(|s| s.to_string());
                Ok(Arc::new(crate::executors::suspend::SuspendMechanism {
                    reason,
                    prompt,
                    resume_key,
                }))
            }
            "security_scan" => {
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
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let cwd_key = node_def
                    .params
                    .get("cwd_key")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                Ok(Arc::new(
                    crate::executors::security_scan::SecurityScanMechanism {
                        files_key,
                        output_key,
                        abort_on_violation,
                        cwd_key,
                    },
                ))
            }
            "router" => Self::build_router_mechanism(node_def),
            _ => Err(QianjiError::TopologyError(format!(
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
            let mechanism = self.build_mechanism(&node_def)?;
            let idx = engine.add_mechanism(&node_def.id, mechanism);
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
            .ok_or(QianjiError::TopologyError(format!(
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
            return Err(QianjiError::TopologyError(
                "Manifest contains a static cycle".to_string(),
            ));
        }

        Ok(engine)
    }
}

fn to_branch_weight(weight: f64) -> Result<f32, QianjiError> {
    if !weight.is_finite() {
        return Err(QianjiError::TopologyError(
            "Router branch weight must be a finite number".to_string(),
        ));
    }
    if !(f64::from(f32::MIN)..=f64::from(f32::MAX)).contains(&weight) {
        return Err(QianjiError::TopologyError(
            "Router branch weight is out of f32 range".to_string(),
        ));
    }
    #[allow(clippy::cast_possible_truncation)]
    let weight = weight as f32;
    Ok(weight)
}
