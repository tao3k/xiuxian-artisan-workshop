//! Discover tool-call read-through cache runtime wiring.
//!
//! Core cache behavior lives in `xiuxian_llm::mcp::discover_cache`; this file
//! only resolves runtime settings/environment for omni-agent.

use std::sync::Arc;

use anyhow::Result;
use xiuxian_llm::mcp::{DiscoverCacheConfig, DiscoverReadThroughCache};
use xiuxian_macros::env_non_empty;

use crate::config::load_runtime_settings;
use crate::env_parse::{parse_bool_from_env, parse_positive_u64_from_env, resolve_valkey_url_env};

const DEFAULT_DISCOVER_CACHE_KEY_PREFIX: &str = "omni-agent:discover";
const DEFAULT_DISCOVER_CACHE_TTL_SECS: u64 = 30;
const MAX_DISCOVER_CACHE_TTL_SECS: u64 = 3_600;

/// Build discover cache from env + runtime settings.
///
/// Returns `Ok(None)` when cache is disabled or no valkey url is configured.
pub(super) fn discover_cache_from_runtime() -> Result<Option<Arc<DiscoverReadThroughCache>>> {
    let Some(config) = resolve_discover_cache_config() else {
        return Ok(None);
    };
    let cache = DiscoverReadThroughCache::from_config(config)?;
    Ok(Some(Arc::new(cache)))
}

fn resolve_discover_cache_config() -> Option<DiscoverCacheConfig> {
    let settings = load_runtime_settings();

    let enabled = parse_bool_from_env("OMNI_AGENT_MCP_DISCOVER_CACHE_ENABLED")
        .or(settings.mcp.discover_cache_enabled)
        .unwrap_or(true);
    if !enabled {
        return None;
    }

    let valkey_url = settings
        .session
        .valkey_url
        .as_deref()
        .map(str::trim)
        .map(str::to_string)
        .filter(|value| !value.is_empty())
        .or_else(resolve_valkey_url_env)?;

    let key_prefix = env_non_empty!("OMNI_AGENT_MCP_DISCOVER_CACHE_KEY_PREFIX")
        .or_else(|| {
            settings
                .mcp
                .discover_cache_key_prefix
                .as_deref()
                .map(str::trim)
                .map(str::to_string)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| DEFAULT_DISCOVER_CACHE_KEY_PREFIX.to_string());

    let ttl_secs = parse_positive_u64_from_env("OMNI_AGENT_MCP_DISCOVER_CACHE_TTL_SECS")
        .or(settings
            .mcp
            .discover_cache_ttl_secs
            .filter(|value| *value > 0))
        .unwrap_or(DEFAULT_DISCOVER_CACHE_TTL_SECS)
        .clamp(1, MAX_DISCOVER_CACHE_TTL_SECS);

    Some(DiscoverCacheConfig {
        valkey_url,
        key_prefix,
        ttl_secs,
    })
}
