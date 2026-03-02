//! xiuxian-qianji: The Thousand Mechanisms Engine.
//!
//! A high-performance, probabilistic DAG executor based on petgraph.
//! Follows Rust 2024 Edition standards.

/// High-level laboratory API for end-to-end workflow execution.
pub mod bootcamp;
/// Distributed consensus management for multi-agent synchronization.
pub mod consensus;
/// Contract definitions for nodes, instructions, and manifests.
pub mod contracts;
/// Core graph engine based on petgraph.
pub mod engine;
/// Unified error handling.
pub mod error;
/// Built-in node execution mechanisms.
pub mod executors;
/// Manifest inspection helpers.
pub mod manifest;
/// Runtime configuration resolver (`packages/conf/qianji.toml` + user overrides).
pub mod runtime_config;
/// Formal logic and safety auditing.
pub mod safety;
/// Asynchronous synaptic-flow scheduler.
pub mod scheduler;
/// Multi-agent swarm orchestration runtime.
pub mod swarm;
/// Real-time swarm telemetry contracts and Valkey emitter.
pub mod telemetry;

#[cfg(feature = "pyo3")]
/// Python bindings via `PyO3`.
pub mod python_module;

pub use bootcamp::{
    BootcampLlmMode, BootcampRunOptions, BootcampVfsMount, WorkflowReport, run_scenario,
    run_workflow, run_workflow_with_mounts,
};
pub use contracts::{
    FlowInstruction, NodeQianhuanExecutionMode, NodeStatus, QianjiManifest, QianjiMechanism,
    QianjiOutput,
};
pub use engine::QianjiEngine;
pub use engine::compiler::QianjiCompiler;
pub use manifest::{manifest_declares_qianhuan_bindings, manifest_requires_llm};
pub use safety::QianjiSafetyGuard;
pub use scheduler::QianjiScheduler;
pub use scheduler::SchedulerAgentIdentity;
pub use scheduler::{RoleAvailabilityRegistry, SchedulerExecutionPolicy};
pub use swarm::{
    ClusterNodeIdentity, ClusterNodeRecord, GlobalSwarmRegistry, RemoteNodeRequest,
    RemoteNodeResponse, RemotePossessionBus, SwarmAgentConfig, SwarmAgentReport, SwarmEngine,
    SwarmExecutionOptions, SwarmExecutionReport, map_execution_error_to_response,
};
pub use telemetry::{
    ConsensusStatus, DEFAULT_PULSE_CHANNEL, NodeTransitionPhase, NoopPulseEmitter, PulseEmitter,
    SwarmEvent, ValkeyPulseEmitter, unix_millis_now,
};

#[cfg(feature = "llm")]
/// Shared LLM client trait object type when `llm` feature is enabled.
pub type QianjiLlmClient = dyn xiuxian_llm::llm::LlmClient;

#[cfg(not(feature = "llm"))]
/// Placeholder trait object type when `llm` feature is disabled.
pub type QianjiLlmClient = dyn std::any::Any + Send + Sync;

/// Built-in research manifest for high-precision calibration.
pub const RESEARCH_TRINITY_TOML: &str = include_str!("../resources/research_trinity.toml");
/// Built-in `MemRL` promotion workflow manifest.
pub const MEMORY_PROMOTION_PIPELINE_TOML: &str =
    include_str!("../resources/memory_promotion_pipeline.toml");

/// Convenient entry point for deploying standard Qianji pipelines.
pub struct QianjiApp;

impl QianjiApp {
    /// Creates a scheduler from one TOML manifest payload.
    ///
    /// # Errors
    ///
    /// Returns [`error::QianjiError`] when manifest compilation fails due to
    /// invalid topology, unsupported mechanisms, or dependency checks.
    pub fn create_pipeline_from_manifest(
        manifest_toml: &str,
        index: std::sync::Arc<xiuxian_wendao::LinkGraphIndex>,
        orchestrator: std::sync::Arc<xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator>,
        registry: std::sync::Arc<xiuxian_qianhuan::persona::PersonaRegistry>,
        llm_client: Option<std::sync::Arc<QianjiLlmClient>>,
    ) -> Result<QianjiScheduler, error::QianjiError> {
        Self::create_pipeline_from_manifest_with_consensus(
            manifest_toml,
            index,
            orchestrator,
            registry,
            llm_client,
            None,
        )
    }

    /// Creates a scheduler from one TOML manifest payload with optional
    /// distributed consensus manager.
    ///
    /// # Errors
    ///
    /// Returns [`error::QianjiError`] when manifest compilation fails due to
    /// invalid topology, unsupported mechanisms, or dependency checks.
    pub fn create_pipeline_from_manifest_with_consensus(
        manifest_toml: &str,
        index: std::sync::Arc<xiuxian_wendao::LinkGraphIndex>,
        orchestrator: std::sync::Arc<xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator>,
        registry: std::sync::Arc<xiuxian_qianhuan::persona::PersonaRegistry>,
        llm_client: Option<std::sync::Arc<QianjiLlmClient>>,
        consensus_manager: Option<std::sync::Arc<crate::consensus::ConsensusManager>>,
    ) -> Result<QianjiScheduler, error::QianjiError> {
        let compiler = QianjiCompiler::new(index, orchestrator, registry, llm_client);
        let engine = compiler.compile(manifest_toml)?;
        Ok(QianjiScheduler::with_consensus_manager(
            engine,
            consensus_manager,
        ))
    }

    /// Creates a standard high-precision research scheduler.
    ///
    /// This pipeline integrates Wendao knowledge search, Qianhuan persona annotation,
    /// and Synapse-Audit adversarial calibration.
    ///
    /// # Errors
    ///
    /// Returns [`error::QianjiError`] when the manifest compilation fails due to invalid
    /// topology, unsupported mechanism configuration, or dependency-related runtime checks.
    pub fn create_research_pipeline(
        index: std::sync::Arc<xiuxian_wendao::LinkGraphIndex>,
        orchestrator: std::sync::Arc<xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator>,
        registry: std::sync::Arc<xiuxian_qianhuan::persona::PersonaRegistry>,
        llm_client: Option<std::sync::Arc<QianjiLlmClient>>,
    ) -> Result<QianjiScheduler, error::QianjiError> {
        Self::create_research_pipeline_with_consensus(
            index,
            orchestrator,
            registry,
            llm_client,
            None,
        )
    }

    /// Creates a standard high-precision research scheduler with optional
    /// distributed consensus manager.
    ///
    /// # Errors
    ///
    /// Returns [`error::QianjiError`] when the manifest compilation fails due to invalid
    /// topology, unsupported mechanism configuration, or dependency-related runtime checks.
    pub fn create_research_pipeline_with_consensus(
        index: std::sync::Arc<xiuxian_wendao::LinkGraphIndex>,
        orchestrator: std::sync::Arc<xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator>,
        registry: std::sync::Arc<xiuxian_qianhuan::persona::PersonaRegistry>,
        llm_client: Option<std::sync::Arc<QianjiLlmClient>>,
        consensus_manager: Option<std::sync::Arc<crate::consensus::ConsensusManager>>,
    ) -> Result<QianjiScheduler, error::QianjiError> {
        let compiler = QianjiCompiler::new(index, orchestrator, registry, llm_client);
        let engine = compiler.compile(RESEARCH_TRINITY_TOML)?;
        Ok(QianjiScheduler::with_consensus_manager(
            engine,
            consensus_manager,
        ))
    }

    /// Creates a standard `MemRL` promotion scheduler.
    ///
    /// # Errors
    ///
    /// Returns [`error::QianjiError`] when manifest compilation fails due to
    /// invalid topology, unsupported mechanisms, or dependency checks.
    pub fn create_memory_promotion_pipeline(
        index: std::sync::Arc<xiuxian_wendao::LinkGraphIndex>,
        orchestrator: std::sync::Arc<xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator>,
        registry: std::sync::Arc<xiuxian_qianhuan::persona::PersonaRegistry>,
        llm_client: Option<std::sync::Arc<QianjiLlmClient>>,
    ) -> Result<QianjiScheduler, error::QianjiError> {
        Self::create_memory_promotion_pipeline_with_consensus(
            index,
            orchestrator,
            registry,
            llm_client,
            None,
        )
    }

    /// Creates a standard `MemRL` promotion scheduler with optional
    /// distributed consensus manager.
    ///
    /// # Errors
    ///
    /// Returns [`error::QianjiError`] when manifest compilation fails due to
    /// invalid topology, unsupported mechanisms, or dependency checks.
    pub fn create_memory_promotion_pipeline_with_consensus(
        index: std::sync::Arc<xiuxian_wendao::LinkGraphIndex>,
        orchestrator: std::sync::Arc<xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator>,
        registry: std::sync::Arc<xiuxian_qianhuan::persona::PersonaRegistry>,
        llm_client: Option<std::sync::Arc<QianjiLlmClient>>,
        consensus_manager: Option<std::sync::Arc<crate::consensus::ConsensusManager>>,
    ) -> Result<QianjiScheduler, error::QianjiError> {
        let compiler = QianjiCompiler::new(index, orchestrator, registry, llm_client);
        let engine = compiler.compile(MEMORY_PROMOTION_PIPELINE_TOML)?;
        Ok(QianjiScheduler::with_consensus_manager(
            engine,
            consensus_manager,
        ))
    }
}
