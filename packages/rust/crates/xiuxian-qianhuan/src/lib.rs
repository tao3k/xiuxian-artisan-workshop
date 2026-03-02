//! xiuxian-qianhuan - Manifestation layer for Xiuxian system.

/// Calibration profiles and heuristics for prompt manifestation.
pub mod calibration;
/// Runtime configuration models for manifestation pipelines.
pub mod config;
/// Shared contracts for prompt injection and context shaping.
pub mod contracts;
/// Entry points for orchestration-facing APIs.
pub mod entry;
/// Error types for manifestation and injection flows.
pub mod error;
/// Reusable hot-reload runtime primitives.
pub mod hot_reload;
/// Public manifestation interface traits.
pub mod interface;
/// Template manifestation manager and rendering pipeline.
pub mod manifestation;
pub mod orchestrator;
#[path = "persona/mod.rs"]
pub mod persona;
pub mod transmuter;
/// Context-window assembly and truncation utilities.
pub mod window;
/// XML helpers used by prompt materialization workflows.
pub mod xml;
#[cfg(feature = "zhenfa-router")]
/// Zhenfa HTTP/RPC router integration for Qianhuan domain capabilities.
pub mod zhenfa_router;

pub use config::InjectionWindowConfig;
pub use contracts::{
    InjectionMode, InjectionPolicy, InjectionSnapshot, PromptContextBlock, PromptContextCategory,
    PromptContextSource, RoleMixProfile,
};
pub use entry::QaEntry;
pub use error::InjectionError;
pub use hot_reload::{
    HotReloadDriver, HotReloadInvocation, HotReloadOutcome, HotReloadRuntime, HotReloadStatus,
    HotReloadTarget, HotReloadTrigger, HotReloadVersionBackend, InMemoryHotReloadVersionBackend,
    ValkeyHotReloadVersionBackend, resolve_hot_reload_watch_extensions,
    resolve_hot_reload_watch_patterns,
};
pub use interface::ManifestationInterface;
pub use manifestation::{
    ManifestationManager, ManifestationRenderRequest, ManifestationRuntimeContext,
    ManifestationTemplateTarget, MemoryTemplateRecord, SessionSystemPromptInjectionSnapshot,
    normalize_session_system_prompt_injection_xml,
};
pub use orchestrator::ThousandFacesOrchestrator;
pub use persona::{MemoryPersonaRecord, PersonaProfile, PersonaProvider, PersonaRegistry};
pub use transmuter::{MockTransmuter, ToneTransmuter};
pub use window::SystemPromptInjectionWindow;
pub use xml::SYSTEM_PROMPT_INJECTION_TAG;
#[cfg(feature = "zhenfa-router")]
pub use zhenfa_router::QianhuanZhenfaRouter;

/// Mock implementation of `ManifestationInterface` for testing.
#[derive(Default)]
pub struct MockManifestation;

impl interface::ManifestationInterface for MockManifestation {
    fn render_template(
        &self,
        _template_name: &str,
        _data: serde_json::Value,
    ) -> anyhow::Result<String> {
        Ok("Mock Manifestation Content".to_string())
    }

    fn inject_context(&self, state_context: &str) -> String {
        state_context.to_string()
    }
}
