//! MCP discover read-through cache tests.

use anyhow::{Result, anyhow};
use xiuxian_llm::mcp::{DiscoverCacheConfig, DiscoverReadThroughCache};

#[test]
fn discover_cache_build_key_rejects_non_discover_tools() -> Result<()> {
    let cache = DiscoverReadThroughCache::from_config(DiscoverCacheConfig {
        valkey_url: "redis://127.0.0.1:6379/".to_string(),
        key_prefix: "omni-agent:discover".to_string(),
        ttl_secs: 30,
    })?;

    let args = serde_json::json!({"intent": "git commit", "limit": 5});
    assert!(cache.build_cache_key("git.commit", Some(&args)).is_none());
    Ok(())
}

#[test]
fn discover_cache_build_key_is_canonical_for_argument_order() -> Result<()> {
    let cache = DiscoverReadThroughCache::from_config(DiscoverCacheConfig {
        valkey_url: "redis://127.0.0.1:6379/".to_string(),
        key_prefix: "omni-agent:discover".to_string(),
        ttl_secs: 30,
    })?;

    let args_a = serde_json::json!({
        "intent": "research rust mcp",
        "limit": 10,
        "extra": {"b": 2, "a": 1}
    });
    let args_b = serde_json::json!({
        "limit": 10,
        "extra": {"a": 1, "b": 2},
        "intent": "research rust mcp"
    });

    let key_a = cache
        .build_cache_key("skill.discover", Some(&args_a))
        .ok_or_else(|| anyhow!("missing key for args_a"))?;
    let key_b = cache
        .build_cache_key("skill.discover", Some(&args_b))
        .ok_or_else(|| anyhow!("missing key for args_b"))?;
    assert_eq!(key_a, key_b);
    Ok(())
}
