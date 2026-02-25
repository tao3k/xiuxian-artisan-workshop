//! System prompt injection window based on XML Q&A blocks.
//!
//! Contract:
//! - Root tag: `<system_prompt_injection>`
//! - Entry tag: `<qa><q>...</q><a>...</a><source>...</source></qa>`
//! - `<source>` is optional.

/// Synapse-Audit calibration primitives for adversarial alignment checks.
pub mod calibration;
/// Configuration types for injection.
pub mod config;
/// Contract types for snapshots and blocks.
pub mod contracts;
/// Individual Q&A entry logic.
pub mod entry;
/// Error definitions for prompt injection.
pub mod error;
/// Orchestration layer for multi-layer prompt assembly.
pub mod orchestrator;
/// Persona model and registry for role-mix style injection.
pub mod persona;
/// Python bindings for the thin orchestration/persona API surface.
pub mod python_module;
/// Tone transmutation traits and implementations.
pub mod transmuter;
/// Bounded session-level system prompt injection window.
pub mod window;
/// XML parsing and rendering logic.
pub mod xml;

/// Interface definitions for manifestation and context injection.
pub mod interface;
/// Core manifestation manager and template engine.
pub mod manifestation;

pub use config::InjectionWindowConfig;
pub use contracts::{
    InjectionMode, InjectionOrderStrategy, InjectionPolicy, InjectionSnapshot, PromptContextBlock,
    PromptContextCategory, PromptContextSource, RoleMixProfile, RoleMixRole,
};
pub use entry::QaEntry;
pub use error::InjectionError;
pub use orchestrator::{InjectionLayer, ThousandFacesOrchestrator};
pub use persona::{PersonaProfile, PersonaRegistry};
pub use transmuter::{MockTransmuter, ToneTransmuter};
pub use window::SystemPromptInjectionWindow;
pub use xml::SYSTEM_PROMPT_INJECTION_TAG;

pub use interface::ManifestationInterface;
pub use manifestation::ManifestationManager;
