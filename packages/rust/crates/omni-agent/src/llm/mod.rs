//! LLM namespace: backend selection, request mapping, and chat client.

mod backend;
mod client;
mod compat;
#[cfg(feature = "agent-provider-litellm")]
mod converters;
mod providers;
mod tools;
mod types;

pub use client::{LlmClient, LlmInFlightSnapshot};
pub use types::AssistantMessage;

#[cfg(test)]
#[path = "../../tests/llm/backend.rs"]
mod backend_tests;

#[cfg(test)]
#[path = "../../tests/llm/provider_mode.rs"]
mod provider_mode_tests;

#[cfg(test)]
#[path = "../../tests/llm/http_request.rs"]
mod http_request_tests;
