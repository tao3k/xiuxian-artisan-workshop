use std::collections::HashMap;

use super::super::telemetry::resolve_snapshot_interval_secs;

#[test]
fn discord_runtime_snapshot_interval_defaults_when_unset() {
    let resolved = resolve_snapshot_interval_secs(|_| None);
    assert_eq!(resolved, Some(30));
}

#[test]
fn discord_runtime_snapshot_interval_uses_positive_override() {
    let values = HashMap::from([(
        "OMNI_AGENT_DISCORD_RUNTIME_SNAPSHOT_INTERVAL_SECS".to_string(),
        "15".to_string(),
    )]);
    let resolved = resolve_snapshot_interval_secs(|name| values.get(name).cloned());
    assert_eq!(resolved, Some(15));
}

#[test]
fn discord_runtime_snapshot_interval_zero_disables_snapshots() {
    let values = HashMap::from([(
        "OMNI_AGENT_DISCORD_RUNTIME_SNAPSHOT_INTERVAL_SECS".to_string(),
        "0".to_string(),
    )]);
    let resolved = resolve_snapshot_interval_secs(|name| values.get(name).cloned());
    assert_eq!(resolved, None);
}

#[test]
fn discord_runtime_snapshot_interval_invalid_falls_back_to_default() {
    let values = HashMap::from([(
        "OMNI_AGENT_DISCORD_RUNTIME_SNAPSHOT_INTERVAL_SECS".to_string(),
        "not-a-number".to_string(),
    )]);
    let resolved = resolve_snapshot_interval_secs(|name| values.get(name).cloned());
    assert_eq!(resolved, Some(30));
}
