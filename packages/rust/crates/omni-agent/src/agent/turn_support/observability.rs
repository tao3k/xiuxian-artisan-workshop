use std::time::{SystemTime, UNIX_EPOCH};

use crate::contracts::{GraphExecutionPlan, GraphPlanStep, OmegaDecision};
use crate::observability::SessionEvent;
use crate::shortcuts::WorkflowBridgeMode;

use super::super::Agent;
use super::super::omega::ShortcutFallbackAction;

impl Agent {
    pub(crate) fn next_runtime_turn_id() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
            .unwrap_or_default()
    }

    pub(crate) fn record_omega_decision(
        session_id: &str,
        decision: &OmegaDecision,
        workflow_mode: Option<WorkflowBridgeMode>,
        tool_name: Option<&str>,
    ) {
        tracing::debug!(
            event = SessionEvent::RouteDecisionSelected.as_str(),
            session_id,
            workflow_mode = workflow_mode.map(WorkflowBridgeMode::as_str),
            tool_name,
            route = decision.route.as_str(),
            risk_level = decision.risk_level.as_str(),
            confidence = decision.confidence,
            fallback_policy = decision.fallback_policy.as_str(),
            tool_trust_class = decision.tool_trust_class.as_str(),
            reason = %decision.reason,
            policy_id = ?decision.policy_id,
            "omega route decision selected"
        );
    }

    pub(crate) fn record_shortcut_fallback(
        session_id: &str,
        decision: &OmegaDecision,
        workflow_mode: WorkflowBridgeMode,
        tool_name: &str,
        action: ShortcutFallbackAction,
        error: &anyhow::Error,
    ) {
        tracing::warn!(
            event = SessionEvent::RouteFallbackApplied.as_str(),
            session_id,
            workflow_mode = workflow_mode.as_str(),
            tool_name,
            route = decision.route.as_str(),
            fallback_policy = decision.fallback_policy.as_str(),
            fallback_action = action.as_str(),
            error = %error,
            "omega route fallback applied"
        );
    }

    pub(crate) fn record_graph_plan(session_id: &str, plan: &GraphExecutionPlan) {
        tracing::debug!(
            event = SessionEvent::RouteGraphPlanGenerated.as_str(),
            session_id,
            plan_id = %plan.plan_id,
            plan_version = %plan.plan_version,
            route = plan.route.as_str(),
            workflow_mode = plan.workflow_mode.as_str(),
            tool_name = %plan.tool_name,
            fallback_policy = plan.fallback_policy.as_str(),
            step_count = plan.steps.len(),
            "graph execution plan generated"
        );
    }

    pub(in crate::agent) fn record_graph_plan_step_started(
        session_id: &str,
        plan: &GraphExecutionPlan,
        step: &GraphPlanStep,
        attempt: u32,
    ) {
        tracing::debug!(
            event = SessionEvent::RouteGraphStepStarted.as_str(),
            session_id,
            plan_id = %plan.plan_id,
            plan_version = %plan.plan_version,
            step_index = step.index,
            step_id = %step.id,
            step_kind = ?step.kind,
            step_tool_name = step.tool_name.as_deref(),
            step_fallback_action = step.fallback_action.as_deref(),
            attempt,
            "graph plan step started"
        );
    }

    pub(in crate::agent) fn record_graph_plan_step_succeeded(
        session_id: &str,
        plan: &GraphExecutionPlan,
        step: &GraphPlanStep,
        attempt: u32,
        status: &str,
    ) {
        tracing::debug!(
            event = SessionEvent::RouteGraphStepSucceeded.as_str(),
            session_id,
            plan_id = %plan.plan_id,
            plan_version = %plan.plan_version,
            step_index = step.index,
            step_id = %step.id,
            step_kind = ?step.kind,
            step_tool_name = step.tool_name.as_deref(),
            step_fallback_action = step.fallback_action.as_deref(),
            attempt,
            status,
            "graph plan step succeeded"
        );
    }

    pub(in crate::agent) fn record_graph_plan_step_failed(
        session_id: &str,
        plan: &GraphExecutionPlan,
        step: &GraphPlanStep,
        attempt: u32,
        error: &anyhow::Error,
    ) {
        tracing::warn!(
            event = SessionEvent::RouteGraphStepFailed.as_str(),
            session_id,
            plan_id = %plan.plan_id,
            plan_version = %plan.plan_version,
            step_index = step.index,
            step_id = %step.id,
            step_kind = ?step.kind,
            step_tool_name = step.tool_name.as_deref(),
            step_fallback_action = step.fallback_action.as_deref(),
            attempt,
            error = %error,
            "graph plan step failed"
        );
    }

    pub(in crate::agent) fn record_graph_execution_completed(
        session_id: &str,
        plan: &GraphExecutionPlan,
        tool_attempts: u32,
        output_chars: usize,
    ) {
        tracing::debug!(
            event = SessionEvent::RouteGraphExecutionCompleted.as_str(),
            session_id,
            plan_id = %plan.plan_id,
            plan_version = %plan.plan_version,
            route = plan.route.as_str(),
            workflow_mode = plan.workflow_mode.as_str(),
            tool_name = %plan.tool_name,
            tool_attempts,
            output_chars,
            "graph execution completed"
        );
    }

    pub(in crate::agent) fn record_graph_execution_rerouted(
        session_id: &str,
        plan: &GraphExecutionPlan,
        target_route: &str,
        reason: &str,
    ) {
        tracing::info!(
            event = SessionEvent::RouteGraphExecutionRerouted.as_str(),
            session_id,
            plan_id = %plan.plan_id,
            plan_version = %plan.plan_version,
            route = plan.route.as_str(),
            workflow_mode = plan.workflow_mode.as_str(),
            tool_name = %plan.tool_name,
            target_route,
            reason,
            "graph execution rerouted"
        );
    }
}
