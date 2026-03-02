//! Integration tests for 3-in-1 memory gate determinism and event shape.

use omni_memory::{
    Episode, MemoryGateEvent, MemoryGatePolicy, MemoryGateVerdict, MemoryLifecycleState,
    MemoryUtilityLedger,
};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn episode_with_stats(
    id: &str,
    outcome: &str,
    q_value: f32,
    success_count: u32,
    failure_count: u32,
) -> Episode {
    let mut episode = Episode::new(
        id.to_string(),
        "intent".to_string(),
        vec![0.1; 8],
        "experience".to_string(),
        outcome.to_string(),
    );
    episode.q_value = q_value;
    episode.success_count = success_count;
    episode.failure_count = failure_count;
    episode
}

#[test]
fn gate_policy_promote_decision_is_deterministic() {
    let episode = episode_with_stats("mem-promote", "completed", 0.94, 8, 1);
    let ledger = MemoryUtilityLedger::from_episode(&episode, 0.95, 0.90, 0.92);
    let policy = MemoryGatePolicy::default();

    let decision_a = policy.evaluate(
        &ledger,
        vec!["react:validation:pass".to_string()],
        vec!["graph:path:plan->execute->verify".to_string()],
        vec!["omega:risk=low".to_string()],
    );
    let decision_b = policy.evaluate(
        &ledger,
        vec!["react:validation:pass".to_string()],
        vec!["graph:path:plan->execute->verify".to_string()],
        vec!["omega:risk=low".to_string()],
    );

    assert_eq!(decision_a, decision_b);
    assert_eq!(decision_a.verdict, MemoryGateVerdict::Promote);
    assert_eq!(decision_a.next_action, "promote");
    assert!(decision_a.confidence >= 0.55);
    assert_eq!(
        decision_a.react_evidence_refs,
        vec!["react:validation:pass".to_string()]
    );
    assert_eq!(
        decision_a.graph_evidence_refs,
        vec!["graph:path:plan->execute->verify".to_string()]
    );
    assert!(
        decision_a
            .omega_factors
            .iter()
            .any(|factor| factor.starts_with("utility_score="))
    );
}

#[test]
fn gate_policy_obsolete_requires_threshold_and_min_usage() {
    let policy = MemoryGatePolicy::default();

    let bad_episode = episode_with_stats("mem-obsolete", "error", 0.12, 0, 6);
    let bad_ledger = MemoryUtilityLedger::from_episode(&bad_episode, 0.10, 0.18, 0.12);
    let bad_decision = policy.evaluate(&bad_ledger, vec![], vec![], vec![]);
    assert_eq!(bad_decision.verdict, MemoryGateVerdict::Obsolete);
    assert_eq!(bad_decision.next_action, "obsolete");

    // Same low quality signals but below min usage should not be purged.
    let fresh_episode = episode_with_stats("mem-fresh", "error", 0.12, 0, 1);
    let fresh_ledger = MemoryUtilityLedger::from_episode(&fresh_episode, 0.10, 0.18, 0.12);
    let fresh_decision = policy.evaluate(&fresh_ledger, vec![], vec![], vec![]);
    assert_eq!(fresh_decision.verdict, MemoryGateVerdict::Retain);
}

#[test]
fn gate_event_matches_contract_shape() -> TestResult {
    let episode = episode_with_stats("mem-event", "completed", 0.91, 7, 1);
    let ledger = MemoryUtilityLedger::from_episode(&episode, 0.92, 0.88, 0.90);
    let policy = MemoryGatePolicy::default();
    let decision = policy.evaluate(
        &ledger,
        vec!["react:ref:1".to_string()],
        vec!["graph:ref:1".to_string()],
        vec!["omega:factor:1".to_string()],
    );
    let event = MemoryGateEvent::from_decision(
        "telegram:1304799691:1304799691",
        42,
        &episode.id,
        &ledger,
        decision,
    );

    let value = serde_json::to_value(&event)?;
    assert_eq!(value["session_id"], "telegram:1304799691:1304799691");
    assert_eq!(value["turn_id"], 42);
    assert_eq!(value["state_before"], "active");
    assert!(matches!(
        event.state_after,
        MemoryLifecycleState::Active
            | MemoryLifecycleState::Purged
            | MemoryLifecycleState::Promoted
    ));
    assert!(value["decision"]["next_action"].is_string());
    let react_refs = value["decision"]["react_evidence_refs"]
        .as_array()
        .ok_or_else(|| std::io::Error::other("react evidence refs should be an array"))?;
    let graph_refs = value["decision"]["graph_evidence_refs"]
        .as_array()
        .ok_or_else(|| std::io::Error::other("graph evidence refs should be an array"))?;
    assert_eq!(react_refs.len(), 1);
    assert_eq!(graph_refs.len(), 1);
    let omega_factors = value["decision"]["omega_factors"]
        .as_array()
        .ok_or_else(|| std::io::Error::other("omega factors should be an array"))?;
    assert!(
        omega_factors.len() >= 4,
        "omega factors should include baseline utility metadata"
    );
    Ok(())
}
