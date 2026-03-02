//! Runtime contract serialization tests for decision and trace payloads.

use omni_agent::{
    DiscoverConfidence, DiscoverMatch, GraphExecutionPlan, GraphPlanStep, GraphPlanStepKind,
    GraphWorkflowMode, MemoryGateDecision, MemoryGateVerdict, OmegaDecision, OmegaFallbackPolicy,
    OmegaRiskLevel, OmegaRoute, OmegaToolTrustClass, RouteTrace, RouteTraceGraphStep,
    RouteTraceInjection,
};

#[test]
fn omega_decision_serializes_with_snake_case_enums() {
    let decision = OmegaDecision {
        route: OmegaRoute::Graph,
        confidence: 0.91,
        risk_level: OmegaRiskLevel::Medium,
        fallback_policy: OmegaFallbackPolicy::RetryReact,
        tool_trust_class: OmegaToolTrustClass::Verification,
        reason: "Long-horizon task requires graph decomposition.".to_string(),
        policy_id: Some("omega.policy.v1".to_string()),
        drift_tolerance: None,
        next_audit_turn: None,
    };

    let raw = serde_json::to_value(&decision).unwrap_or_else(|error| {
        panic!("failed to serialize omega decision: {error}");
    });

    assert_eq!(raw["route"], "graph");
    assert_eq!(raw["risk_level"], "medium");
    assert_eq!(raw["fallback_policy"], "retry_react");
    assert_eq!(raw["tool_trust_class"], "verification");
}

#[test]
fn memory_gate_decision_roundtrip_stays_stable() {
    let decision = MemoryGateDecision {
        verdict: MemoryGateVerdict::Promote,
        confidence: 0.88,
        react_evidence_refs: vec!["react:tool_retry:42".to_string()],
        graph_evidence_refs: vec!["graph:path:checkout->commit".to_string()],
        omega_factors: vec!["runtime_utility_trend=up".to_string()],
        reason: "Repeatedly validated high-value pattern.".to_string(),
        next_action: "promote".to_string(),
    };

    let raw = serde_json::to_string(&decision).unwrap_or_else(|error| {
        panic!("failed to serialize memory gate decision: {error}");
    });
    let decoded: MemoryGateDecision = serde_json::from_str(&raw).unwrap_or_else(|error| {
        panic!("failed to deserialize memory gate decision: {error}");
    });

    assert_eq!(decoded.verdict, MemoryGateVerdict::Promote);
    assert_eq!(decoded.next_action, "promote");
    assert_eq!(decoded.react_evidence_refs.len(), 1);
}

#[test]
fn discover_match_contract_carries_confidence_and_digest() {
    let row = DiscoverMatch {
        tool: "skill.discover".to_string(),
        usage: "@omni(\"skill.discover\", {\"intent\": \"<intent: string>\"})".to_string(),
        score: 0.73,
        final_score: 0.84,
        confidence: DiscoverConfidence::High,
        ranking_reason: "Strong intent overlap + schema compatibility.".to_string(),
        input_schema_digest: "sha256:abc123".to_string(),
        documentation_path: Some("/tmp/SKILL.md".to_string()),
    };

    let raw = serde_json::to_value(&row).unwrap_or_else(|error| {
        panic!("failed to serialize discover match: {error}");
    });

    assert_eq!(raw["confidence"], "high");
    assert_eq!(raw["input_schema_digest"], "sha256:abc123");
    let final_score = raw["final_score"]
        .as_f64()
        .unwrap_or_else(|| panic!("missing final_score number in serialized payload"));
    assert!((final_score - 0.84).abs() < 1e-6);
}

#[test]
fn graph_execution_plan_contract_is_stable_and_snake_case() {
    let plan = GraphExecutionPlan {
        plan_id: "graph-plan:graph:researcher.run:abort:evidence".to_string(),
        plan_version: "v1".to_string(),
        route: OmegaRoute::Graph,
        workflow_mode: GraphWorkflowMode::Graph,
        tool_name: "researcher.run".to_string(),
        fallback_policy: OmegaFallbackPolicy::Abort,
        steps: vec![
            GraphPlanStep {
                index: 1,
                id: "prepare_injection_context".to_string(),
                kind: GraphPlanStepKind::PrepareInjectionContext,
                description: "prepare".to_string(),
                tool_name: None,
                fallback_action: None,
            },
            GraphPlanStep {
                index: 2,
                id: "invoke_graph_tool".to_string(),
                kind: GraphPlanStepKind::InvokeGraphTool,
                description: "invoke".to_string(),
                tool_name: Some("researcher.run".to_string()),
                fallback_action: None,
            },
            GraphPlanStep {
                index: 3,
                id: "evaluate_fallback".to_string(),
                kind: GraphPlanStepKind::EvaluateFallback,
                description: "fallback".to_string(),
                tool_name: None,
                fallback_action: Some("abort".to_string()),
            },
        ],
    };

    let raw = serde_json::to_value(&plan).unwrap_or_else(|error| {
        panic!("failed to serialize graph execution plan: {error}");
    });
    assert_eq!(raw["route"], "graph");
    assert_eq!(raw["workflow_mode"], "graph");
    assert_eq!(raw["fallback_policy"], "abort");
    assert_eq!(raw["steps"][0]["kind"], "prepare_injection_context");
    assert_eq!(raw["steps"][1]["kind"], "invoke_graph_tool");
    assert_eq!(raw["steps"][2]["kind"], "evaluate_fallback");
}

#[test]
fn graph_execution_plan_contract_validation_accepts_deterministic_v1_shape() {
    let plan = GraphExecutionPlan {
        plan_id: "graph-plan:graph:researcher.run:abort:evidence".to_string(),
        plan_version: "v1".to_string(),
        route: OmegaRoute::Graph,
        workflow_mode: GraphWorkflowMode::Graph,
        tool_name: "researcher.run".to_string(),
        fallback_policy: OmegaFallbackPolicy::Abort,
        steps: vec![
            GraphPlanStep {
                index: 1,
                id: "prepare_injection_context".to_string(),
                kind: GraphPlanStepKind::PrepareInjectionContext,
                description: "prepare".to_string(),
                tool_name: None,
                fallback_action: None,
            },
            GraphPlanStep {
                index: 2,
                id: "invoke_graph_tool".to_string(),
                kind: GraphPlanStepKind::InvokeGraphTool,
                description: "invoke".to_string(),
                tool_name: Some("researcher.run".to_string()),
                fallback_action: None,
            },
            GraphPlanStep {
                index: 3,
                id: "evaluate_fallback".to_string(),
                kind: GraphPlanStepKind::EvaluateFallback,
                description: "fallback".to_string(),
                tool_name: None,
                fallback_action: Some("abort".to_string()),
            },
        ],
    };

    if let Err(error) = plan.validate_shortcut_contract() {
        panic!("deterministic v1 graph plan should be accepted: {error}");
    }
}

#[test]
fn graph_execution_plan_contract_validation_rejects_invalid_fallback_action() {
    let plan = GraphExecutionPlan {
        plan_id: "graph-plan:graph:researcher.run:abort:evidence".to_string(),
        plan_version: "v1".to_string(),
        route: OmegaRoute::Graph,
        workflow_mode: GraphWorkflowMode::Graph,
        tool_name: "researcher.run".to_string(),
        fallback_policy: OmegaFallbackPolicy::Abort,
        steps: vec![
            GraphPlanStep {
                index: 1,
                id: "prepare_injection_context".to_string(),
                kind: GraphPlanStepKind::PrepareInjectionContext,
                description: "prepare".to_string(),
                tool_name: None,
                fallback_action: None,
            },
            GraphPlanStep {
                index: 2,
                id: "invoke_graph_tool".to_string(),
                kind: GraphPlanStepKind::InvokeGraphTool,
                description: "invoke".to_string(),
                tool_name: Some("researcher.run".to_string()),
                fallback_action: None,
            },
            GraphPlanStep {
                index: 3,
                id: "evaluate_fallback".to_string(),
                kind: GraphPlanStepKind::EvaluateFallback,
                description: "fallback".to_string(),
                tool_name: None,
                fallback_action: Some("switch_to_python_loop".to_string()),
            },
        ],
    };

    let error = match plan.validate_shortcut_contract() {
        Ok(()) => panic!("unsupported fallback action must be rejected"),
        Err(error) => error,
    };
    assert!(error.contains("unsupported fallback_action"));
}

#[test]
fn route_trace_contract_supports_plan_and_step_aggregation() {
    let trace = RouteTrace {
        session_id: "telegram:group-1:user-9".to_string(),
        turn_id: 43,
        selected_route: OmegaRoute::Graph,
        confidence: 0.84,
        risk_level: OmegaRiskLevel::Medium,
        tool_trust_class: OmegaToolTrustClass::Evidence,
        fallback_applied: Some(true),
        fallback_policy: Some(OmegaFallbackPolicy::SwitchToGraph),
        tool_chain: vec!["bridge.flaky".to_string()],
        latency_ms: Some(327.1),
        failure_taxonomy: vec!["transport".to_string()],
        injection: Some(RouteTraceInjection {
            blocks_used: 6,
            chars_injected: 3_120,
            dropped_by_budget: 1,
        }),
        plan_id: Some("graph-plan:omega:bridge.flaky:switch_to_graph:verification".to_string()),
        workflow_mode: Some(GraphWorkflowMode::Omega),
        graph_steps: Some(vec![
            RouteTraceGraphStep {
                index: 1,
                id: "prepare_injection_context".to_string(),
                kind: GraphPlanStepKind::PrepareInjectionContext,
                attempt: 0,
                latency_ms: 0.3,
                status: "prepared".to_string(),
                failure_reason: None,
                tool_name: None,
                fallback_action: None,
            },
            RouteTraceGraphStep {
                index: 2,
                id: "invoke_graph_tool".to_string(),
                kind: GraphPlanStepKind::InvokeGraphTool,
                attempt: 1,
                latency_ms: 62.5,
                status: "tool_call_transport_failed".to_string(),
                failure_reason: Some("connection refused".to_string()),
                tool_name: Some("bridge.flaky".to_string()),
                fallback_action: None,
            },
            RouteTraceGraphStep {
                index: 3,
                id: "evaluate_fallback".to_string(),
                kind: GraphPlanStepKind::EvaluateFallback,
                attempt: 2,
                latency_ms: 41.0,
                status: "retry_succeeded_without_metadata".to_string(),
                failure_reason: None,
                tool_name: None,
                fallback_action: Some("retry_bridge_without_metadata".to_string()),
            },
        ]),
    };

    let raw = serde_json::to_value(&trace).unwrap_or_else(|error| {
        panic!("failed to serialize route trace contract: {error}");
    });
    assert_eq!(raw["selected_route"], "graph");
    assert_eq!(raw["workflow_mode"], "omega");
    assert_eq!(raw["fallback_policy"], "switch_to_graph");
    assert_eq!(raw["graph_steps"][0]["kind"], "prepare_injection_context");
    assert_eq!(
        raw["graph_steps"][1]["status"],
        "tool_call_transport_failed"
    );
    assert_eq!(
        raw["graph_steps"][2]["fallback_action"],
        "retry_bridge_without_metadata"
    );
}
