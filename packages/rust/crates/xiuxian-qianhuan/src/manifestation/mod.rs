/// Manifestation manager logic.
pub mod manager;
/// Manifestation render request models.
pub mod request;
/// Template helper logic.
pub mod templates;

pub use manager::{
    ManifestationManager, MemoryTemplateRecord, SessionSystemPromptInjectionSnapshot,
    normalize_session_system_prompt_injection_xml,
};
pub use request::{
    ManifestationRenderRequest, ManifestationRuntimeContext, ManifestationTemplateTarget,
};
