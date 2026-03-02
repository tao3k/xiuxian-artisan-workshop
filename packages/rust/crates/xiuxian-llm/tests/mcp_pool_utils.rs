//! MCP pool utility function tests.

use std::time::Duration;

use xiuxian_llm::mcp::{
    call_slow_warn_threshold_ms, call_timeout_for_tool, hit_rate_pct_two_decimals,
    is_expected_long_running_tool, list_tools_cache_ttl_from_config,
};

#[test]
fn list_tools_cache_ttl_is_clamped_to_safe_range() {
    assert_eq!(
        list_tools_cache_ttl_from_config(0),
        Duration::from_millis(1)
    );
    assert_eq!(
        list_tools_cache_ttl_from_config(1_000_000),
        Duration::from_millis(60_000)
    );
}

#[test]
fn call_slow_warn_threshold_scales_for_regular_tools() {
    assert_eq!(
        call_slow_warn_threshold_ms("tools/call:memory.save_memory", Duration::from_secs(3)),
        2_000
    );
    assert_eq!(
        call_slow_warn_threshold_ms("tools/call:skill.discover", Duration::from_secs(60)),
        10_000
    );
}

#[test]
fn call_slow_warn_threshold_scales_for_long_running_tools() {
    assert!(is_expected_long_running_tool(
        "tools/call:crawl4ai.crawl_url"
    ));
    assert!(!is_expected_long_running_tool("tools/call:skill.discover"));
    assert_eq!(
        call_slow_warn_threshold_ms("tools/call:crawl4ai.crawl_url", Duration::from_secs(120)),
        40_000
    );
}

#[test]
fn hit_rate_pct_two_decimals_behaves_as_expected() {
    const EPSILON: f64 = 0.005;

    assert!((hit_rate_pct_two_decimals(0, 0) - 0.0).abs() < EPSILON);
    assert!((hit_rate_pct_two_decimals(1, 3) - 33.33).abs() < EPSILON);
    assert!((hit_rate_pct_two_decimals(10, 10) - 100.0).abs() < EPSILON);
}

#[test]
fn call_timeout_for_tool_uses_short_budget_for_memory_save() {
    assert_eq!(
        call_timeout_for_tool("memory.save_memory", Duration::from_secs(60)),
        Duration::from_secs(5)
    );
    assert_eq!(
        call_timeout_for_tool("skill.discover", Duration::from_secs(60)),
        Duration::from_secs(60)
    );
}
