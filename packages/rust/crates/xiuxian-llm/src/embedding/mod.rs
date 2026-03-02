//! Embedding transport primitives.

/// Backend mode parsing and normalized backend kinds.
pub mod backend;
/// OpenAI-compatible embedding transport utilities.
pub mod openai_compat;
/// Memory embedding runtime guard (timeout/cooldown/dimension repair).
pub mod runtime;
/// In-process `mistralrs` embedding runtime utilities.
pub mod sdk;
