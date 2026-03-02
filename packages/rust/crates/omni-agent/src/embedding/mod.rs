//! Embedding client runtime.
//!
//! Supports four backends:
//! - `http`: `/embed/batch` HTTP transport.
//! - `openai_http`: generic OpenAI-compatible `/v1/embeddings`.
//! - `mistral_sdk`: in-process `mistralrs` SDK embedding runtime.
//! - `litellm_rs`: Rust-native `LiteLLM` provider path (provider/API-key driven).

mod backend;
mod cache;
mod client;
mod transport_http;
#[cfg(feature = "agent-provider-litellm")]
mod transport_litellm;
mod transport_mistral_sdk;
mod transport_openai;
mod types;

pub use client::{EmbeddingClient, EmbeddingInFlightSnapshot};
