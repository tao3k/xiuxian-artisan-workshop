//! Embedding client runtime.
//!
//! Supports three backends:
//! - `http`: `/embed/batch` HTTP transport.
//! - `openai_http`: OpenAI-compatible `/v1/embeddings` (for example local `mistralrs-server`).
//! - `mistral_local`: explicit local `mistralrs-server` runtime mode (also OpenAI-compatible).
//! - `litellm_rs`: Rust-native `LiteLLM` provider path (provider/API-key driven).

mod backend;
mod cache;
mod client;
mod transport_http;
#[cfg(feature = "agent-provider-litellm")]
mod transport_litellm;
mod transport_openai;
mod types;

pub use client::{EmbeddingClient, EmbeddingInFlightSnapshot};
