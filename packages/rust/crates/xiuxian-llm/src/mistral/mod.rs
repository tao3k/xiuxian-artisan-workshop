//! Mistral runtime helpers for OpenAI-compatible local serving.
//!
//! This module provides:
//! - configuration parsing and defaults,
//! - lightweight readiness probes for `/v1/models`,
//! - process lifecycle management for `mistralrs-server`.

pub mod config;
pub mod health;
pub mod process;

pub use config::MistralServerConfig;
pub use health::{MistralHealthStatus, derive_models_url, probe_models};
pub use process::{ManagedMistralServer, spawn_mistral_server};
