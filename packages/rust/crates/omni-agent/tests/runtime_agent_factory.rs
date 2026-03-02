//! Runtime-agent factory integration harness.

use omni_agent::{LITELLM_DEFAULT_URL, McpServerEntry, RuntimeSettings};

mod resolve {
    const XIUXIAN_WENDAO_VALKEY_URL_ENV: &str = "XIUXIAN_WENDAO_VALKEY_URL";
    const LEGACY_VALKEY_URL_ENV: &str = "VALKEY_URL";

    pub(crate) fn parse_bool_from_env(name: &str) -> Option<bool> {
        parse_env_value(name, |raw| match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
    }

    pub(crate) fn parse_positive_u32_from_env(name: &str) -> Option<u32> {
        parse_env_value(name, |raw| {
            raw.parse::<u32>().ok().filter(|value| *value > 0)
        })
    }

    pub(crate) fn parse_positive_usize_from_env(name: &str) -> Option<usize> {
        parse_env_value(name, |raw| {
            raw.parse::<usize>().ok().filter(|value| *value > 0)
        })
    }

    pub(crate) fn parse_positive_u64_from_env(name: &str) -> Option<u64> {
        parse_env_value(name, |raw| {
            raw.parse::<u64>().ok().filter(|value| *value > 0)
        })
    }

    pub(crate) fn parse_positive_f32_from_env(name: &str) -> Option<f32> {
        parse_env_value(name, |raw| {
            raw.parse::<f32>().ok().filter(|value| *value > 0.0)
        })
    }

    pub(crate) fn parse_unit_f32_from_env(name: &str) -> Option<f32> {
        parse_env_value(name, |raw| {
            raw.parse::<f32>()
                .ok()
                .filter(|value| (0.0..=1.0).contains(value))
        })
    }

    pub(crate) fn resolve_valkey_url_env() -> Option<String> {
        non_empty_env(XIUXIAN_WENDAO_VALKEY_URL_ENV)
            .or_else(|| non_empty_env(LEGACY_VALKEY_URL_ENV))
    }

    fn non_empty_env(name: &str) -> Option<String> {
        std::env::var(name).ok().and_then(|raw| {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }

    fn parse_env_value<T>(name: &str, parser: impl FnOnce(&str) -> Option<T>) -> Option<T> {
        let raw = std::env::var(name).ok()?;
        parser(raw.as_str())
    }
}

mod types {
    use omni_agent::MemoryConfig;
    use xiuxian_llm::embedding::backend::EmbeddingBackendKind;

    pub(super) type RuntimeEmbeddingBackendMode = EmbeddingBackendKind;

    pub(super) struct MemoryRuntimeOptions {
        pub(super) config: MemoryConfig,
        pub(super) embedding_backend_mode: RuntimeEmbeddingBackendMode,
    }
}

#[path = "../src/runtime_agent_factory/inference.rs"]
mod inference;
#[path = "runtime_agent_factory/memory.rs"]
mod memory;
#[path = "../src/runtime_agent_factory/shared.rs"]
mod shared;

use inference::{
    parse_embedding_backend_mode, resolve_inference_url, resolve_runtime_embedding_backend_mode,
    resolve_runtime_embedding_base_url, resolve_runtime_inference_url,
    validate_inference_url_origin,
};
use memory::resolve_runtime_memory_options;
use types::RuntimeEmbeddingBackendMode;

const _: fn(&RuntimeSettings) -> String = inference::resolve_runtime_model;

#[path = "runtime_agent_factory/inference.rs"]
mod tests;
