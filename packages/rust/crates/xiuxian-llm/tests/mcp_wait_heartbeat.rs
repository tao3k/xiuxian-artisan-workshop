//! MCP wait-heartbeat classification tests.

use xiuxian_llm::mcp::{
    HealthProbeStatus, WaitHeartbeatState, classify_wait_heartbeat, degraded_wait_warn_after_secs,
};

fn probe(
    ready: Option<bool>,
    initializing: Option<bool>,
    has_structured_ready_state: bool,
    status_code: Option<u16>,
    timed_out: bool,
    transport_error: bool,
) -> HealthProbeStatus {
    HealthProbeStatus {
        summary: "test".to_string(),
        ready,
        initializing,
        has_structured_ready_state,
        status_code,
        timed_out,
        transport_error,
    }
}

#[test]
fn classify_wait_heartbeat_structured_ready_is_healthy() {
    let status = probe(Some(true), Some(false), true, Some(200), false, false);
    assert_eq!(
        classify_wait_heartbeat(&status),
        WaitHeartbeatState::Healthy
    );
}

#[test]
fn classify_wait_heartbeat_structured_initializing_is_degraded() {
    let status = probe(Some(false), Some(true), true, Some(200), false, false);
    assert_eq!(
        classify_wait_heartbeat(&status),
        WaitHeartbeatState::Degraded
    );
}

#[test]
fn classify_wait_heartbeat_structured_not_ready_is_unhealthy() {
    let status = probe(Some(false), Some(false), true, Some(503), false, false);
    assert_eq!(
        classify_wait_heartbeat(&status),
        WaitHeartbeatState::Unhealthy
    );
}

#[test]
fn classify_wait_heartbeat_non_structured_2xx_is_healthy() {
    let status = probe(None, None, false, Some(200), false, false);
    assert_eq!(
        classify_wait_heartbeat(&status),
        WaitHeartbeatState::Healthy
    );
}

#[test]
fn classify_wait_heartbeat_timeout_is_degraded() {
    let status = probe(None, None, false, None, true, false);
    assert_eq!(
        classify_wait_heartbeat(&status),
        WaitHeartbeatState::Degraded
    );
}

#[test]
fn degraded_wait_warn_after_secs_clamps_to_safe_range() {
    assert_eq!(degraded_wait_warn_after_secs(180), 30);
    assert_eq!(degraded_wait_warn_after_secs(3), 5);
}
