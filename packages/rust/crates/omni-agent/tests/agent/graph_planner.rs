#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use crate::contracts::{
    GraphPlanStepKind, GraphWorkflowMode, OmegaDecision, OmegaFallbackPolicy, OmegaRiskLevel,
    OmegaRoute, OmegaToolTrustClass,
};
use crate::shortcuts::WorkflowBridgeMode;

fn build_decision(
    fallback_policy: OmegaFallbackPolicy,
    tool_trust: OmegaToolTrustClass,
) -> OmegaDecision {
    OmegaDecision {
        route: OmegaRoute::Graph,
        confidence: 0.91,
        risk_level: OmegaRiskLevel::Low,
        fallback_policy,
        tool_trust_class: tool_trust,
        reason: "planner-test".to_string(),
        policy_id: Some("omega.unit.graph_planner.v1".to_string()),
        drift_tolerance: None,
        next_audit_turn: None,
    }
}

#[test]
fn build_shortcut_plan_is_deterministic_and_v1_contract_valid() {
    let decision = build_decision(
        OmegaFallbackPolicy::Abort,
        OmegaToolTrustClass::Verification,
    );
    let first = super::build_shortcut_plan(WorkflowBridgeMode::Graph, &decision, " bridge.flaky ");
    let second = super::build_shortcut_plan(WorkflowBridgeMode::Graph, &decision, "bridge.flaky");

    assert_eq!(first, second);
    assert_eq!(
        first.plan_id,
        "graph-plan:graph:bridge.flaky:abort:verification"
    );
    assert_eq!(first.plan_version, "v1");
    assert_eq!(first.workflow_mode, GraphWorkflowMode::Graph);
    assert_eq!(first.route, OmegaRoute::Graph);
    assert_eq!(first.steps.len(), 3);
    assert_eq!(
        first.steps.iter().map(|step| step.kind).collect::<Vec<_>>(),
        vec![
            GraphPlanStepKind::PrepareInjectionContext,
            GraphPlanStepKind::InvokeGraphTool,
            GraphPlanStepKind::EvaluateFallback,
        ]
    );
    assert_eq!(first.steps[2].fallback_action.as_deref(), Some("abort"));

    first
        .validate_shortcut_contract()
        .expect("planner output must pass deterministic graph-plan contract validation");
}

#[test]
fn build_shortcut_plan_maps_policy_to_expected_fallback_actions() {
    let retry_bridge_decision = build_decision(
        OmegaFallbackPolicy::SwitchToGraph,
        OmegaToolTrustClass::Verification,
    );
    let retry_bridge_plan = super::build_shortcut_plan(
        WorkflowBridgeMode::Omega,
        &retry_bridge_decision,
        "bridge.flaky",
    );
    assert_eq!(retry_bridge_plan.workflow_mode, GraphWorkflowMode::Omega);
    assert_eq!(
        retry_bridge_plan.steps[2].fallback_action.as_deref(),
        Some("retry_bridge_without_metadata")
    );
    retry_bridge_plan
        .validate_shortcut_contract()
        .expect("switch_to_graph shortcut plan must be valid");

    let route_react_decision = build_decision(
        OmegaFallbackPolicy::RetryReact,
        OmegaToolTrustClass::Evidence,
    );
    let route_react_plan = super::build_shortcut_plan(
        WorkflowBridgeMode::Omega,
        &route_react_decision,
        "researcher.run",
    );
    assert_eq!(
        route_react_plan.steps[2].fallback_action.as_deref(),
        Some("route_to_react")
    );
    assert_eq!(
        route_react_plan.plan_id,
        "graph-plan:omega:researcher.run:retry_react:evidence"
    );
    route_react_plan
        .validate_shortcut_contract()
        .expect("retry_react shortcut plan must be valid");
}
