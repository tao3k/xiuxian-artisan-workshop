//! Test coverage for omni-agent behavior.

use omni_agent::contracts::{
    OmegaDecision, OmegaFallbackPolicy, OmegaRiskLevel, OmegaRoute, OmegaToolTrustClass,
};

#[test]
fn test_omega_trajectory_drift_contract() {
    let decision = OmegaDecision {
        route: OmegaRoute::React,
        confidence: 0.9,
        risk_level: OmegaRiskLevel::Low,
        fallback_policy: OmegaFallbackPolicy::Abort,
        tool_trust_class: OmegaToolTrustClass::Evidence,
        reason: "Initial".to_string(),
        policy_id: None,
        drift_tolerance: Some(0.5),
        next_audit_turn: Some(3),
    };

    assert!(decision.drift_tolerance.is_some());
    assert_eq!(decision.next_audit_turn, Some(3));
}
