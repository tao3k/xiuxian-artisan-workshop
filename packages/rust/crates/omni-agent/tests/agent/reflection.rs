/// Reflection policy and runtime state-transition tests for agent orchestration.
use crate::contracts::{OmegaFallbackPolicy, OmegaRiskLevel, OmegaRoute, OmegaToolTrustClass};

use super::{ReflectiveRuntime, ReflectiveRuntimeStage, build_turn_reflection, derive_policy_hint};

type TurnFixture = (&'static str, u32, &'static str);

const LONG_HORIZON_TURNS: [TurnFixture; 15] = [
    ("error", 2, "tool timeout while waiting for dependency"),
    (
        "completed",
        0,
        "summary completed successfully with clear output",
    ),
    (
        "completed",
        5,
        "workflow completed after multiple tool calls",
    ),
    ("error", 3, "permission denied during validation"),
    ("completed", 0, "result generated with stable confidence"),
    (
        "completed",
        4,
        "complex plan resolved with structured steps",
    ),
    ("error", 1, "unexpected failure and retry requested"),
    ("completed", 0, "concise answer completed and verified"),
    (
        "completed",
        6,
        "heavy tool chain completed with checkpoints",
    ),
    ("error", 2, "failed to parse dependency output"),
    (
        "completed",
        0,
        "stable completion with deterministic output",
    ),
    ("completed", 4, "graph-style workflow executed successfully"),
    ("error", 2, "timeout reached while waiting for callback"),
    ("completed", 0, "final summary completed with no tools"),
    (
        "completed",
        5,
        "multi-step execution completed successfully",
    ),
];

fn execute_reflective_runtime_cycle(runtime: &mut ReflectiveRuntime) {
    if let Err(error) = runtime.transition(ReflectiveRuntimeStage::Diagnose) {
        panic!("diagnose transition should succeed: {error}");
    }
    if let Err(error) = runtime.transition(ReflectiveRuntimeStage::Plan) {
        panic!("plan transition should succeed: {error}");
    }
    if let Err(error) = runtime.transition(ReflectiveRuntimeStage::Apply) {
        panic!("apply transition should succeed: {error}");
    }
}

#[test]
fn reflective_runtime_enforces_diagnose_plan_apply_order() {
    let mut runtime = ReflectiveRuntime::default();
    assert!(runtime.transition(ReflectiveRuntimeStage::Diagnose).is_ok());
    assert!(runtime.transition(ReflectiveRuntimeStage::Plan).is_ok());
    assert!(runtime.transition(ReflectiveRuntimeStage::Apply).is_ok());
    assert_eq!(runtime.stage(), Some(ReflectiveRuntimeStage::Apply));
}

#[test]
fn reflective_runtime_rejects_illegal_transition_with_explicit_error() {
    let mut runtime = ReflectiveRuntime::default();
    let error = match runtime.transition(ReflectiveRuntimeStage::Plan) {
        Ok(()) => panic!("plan before diagnose must be rejected"),
        Err(error) => error,
    };
    assert_eq!(error.from, None);
    assert_eq!(error.to, ReflectiveRuntimeStage::Plan);
    assert!(
        error
            .to_string()
            .contains("illegal reflection lifecycle transition"),
        "error message must be explicit for runtime diagnostics"
    );
}

#[test]
fn derive_policy_hint_prefers_verification_after_error() {
    let reflection = build_turn_reflection(
        "react",
        "run regression suite",
        "tool call failed: timeout while waiting",
        "error",
        2,
    );
    let Some(hint) = derive_policy_hint(&reflection, 42) else {
        panic!("error reflection should emit a hint");
    };
    assert_eq!(hint.source_turn_id, 42);
    assert_eq!(hint.preferred_route, OmegaRoute::Graph);
    assert_eq!(hint.risk_floor, OmegaRiskLevel::Medium);
    assert_eq!(
        hint.fallback_override,
        Some(OmegaFallbackPolicy::SwitchToGraph)
    );
    assert_eq!(hint.tool_trust_class, OmegaToolTrustClass::Verification);
}

#[test]
fn derive_policy_hint_prefers_fast_path_for_stable_tool_free_turn() {
    let reflection = build_turn_reflection(
        "react",
        "summarize key points",
        "Summary completed successfully with clear output",
        "completed",
        0,
    );
    let Some(hint) = derive_policy_hint(&reflection, 7) else {
        panic!("stable turn should emit a hint");
    };
    assert_eq!(hint.preferred_route, OmegaRoute::React);
    assert_eq!(hint.tool_trust_class, OmegaToolTrustClass::Evidence);
    assert_eq!(hint.fallback_override, None);
}

#[test]
fn reflective_runtime_long_horizon_quality_thresholds() {
    const MIN_TURNS: usize = 15;
    const MIN_HINT_COVERAGE_PERCENT: usize = 95;
    const MIN_VERIFICATION_HINTS: usize = 10;
    const MIN_FAST_PATH_HINTS: usize = 5;

    assert!(
        LONG_HORIZON_TURNS.len() >= MIN_TURNS,
        "test fixture must maintain long-horizon scale"
    );

    let mut hints_emitted = 0usize;
    let mut verification_hints = 0usize;
    let mut fast_path_hints = 0usize;
    let mut transition_successes = 0usize;

    for (index, (outcome, tool_calls, assistant_message)) in LONG_HORIZON_TURNS.iter().enumerate() {
        let mut runtime = ReflectiveRuntime::default();
        execute_reflective_runtime_cycle(&mut runtime);
        transition_successes += 3;

        let reflection = build_turn_reflection(
            "react",
            &format!("long-horizon objective #{index}"),
            assistant_message,
            outcome,
            *tool_calls,
        );
        let turn_id = u64::try_from(index + 1).unwrap_or(u64::MAX);
        let hint = derive_policy_hint(&reflection, turn_id);
        if let Some(hint) = hint {
            hints_emitted += 1;
            match hint.tool_trust_class {
                OmegaToolTrustClass::Verification => verification_hints += 1,
                OmegaToolTrustClass::Evidence => fast_path_hints += 1,
                OmegaToolTrustClass::Other => {}
            }
        }
    }

    let hint_coverage_percent = if LONG_HORIZON_TURNS.is_empty() {
        0
    } else {
        hints_emitted.saturating_mul(100) / LONG_HORIZON_TURNS.len()
    };
    assert!(
        hints_emitted.saturating_mul(100)
            >= LONG_HORIZON_TURNS
                .len()
                .saturating_mul(MIN_HINT_COVERAGE_PERCENT),
        "hint coverage below threshold: {hint_coverage_percent}% < {MIN_HINT_COVERAGE_PERCENT}%"
    );
    assert!(
        verification_hints >= MIN_VERIFICATION_HINTS,
        "verification hints below threshold: {verification_hints} < {MIN_VERIFICATION_HINTS}"
    );
    assert!(
        fast_path_hints >= MIN_FAST_PATH_HINTS,
        "fast-path hints below threshold: {fast_path_hints} < {MIN_FAST_PATH_HINTS}"
    );
    assert_eq!(
        transition_successes,
        LONG_HORIZON_TURNS.len() * 3,
        "each long-horizon turn must execute diagnose->plan->apply transitions"
    );
}
