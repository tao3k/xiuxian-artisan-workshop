use std::collections::BTreeSet;
use std::time::Instant;

use crate::agent::ToolExecutionSummary;
use crate::contracts::{
    GraphExecutionPlan, GraphPlanStep, GraphPlanStepKind, OmegaDecision, RouteTraceInjection,
};
use crate::shortcuts::WorkflowBridgeMode;

/// Input bundle for deterministic graph-plan execution.
#[derive(Debug, Clone)]
pub(in crate::agent) struct GraphPlanExecutionInput {
    /// Source shortcut mode (`graph` or `omega`) for telemetry/fallback records.
    pub workflow_mode: WorkflowBridgeMode,
    /// Runtime turn identifier shared with route trace and reflection.
    pub turn_id: u64,
    /// Original shortcut message; used when fallback reroutes into `ReAct`.
    pub shortcut_user_message: String,
    /// First-attempt bridge args (usually enriched with `_omni` metadata).
    pub bridge_arguments_with_metadata: Option<serde_json::Value>,
    /// Metadata-free retry args used by compatibility fallback.
    pub bridge_arguments_without_metadata: Option<serde_json::Value>,
    /// Optional qianhuan injection summary for route trace payload.
    pub injection: Option<RouteTraceInjection>,
}

/// Deterministic graph-plan execution result.
#[derive(Debug, Clone)]
pub(in crate::agent) enum GraphPlanExecutionOutcome {
    Completed {
        output: String,
        tool_summary: ToolExecutionSummary,
    },
    RouteToReact {
        rewritten_user_message: String,
        tool_summary: ToolExecutionSummary,
    },
}

/// Graph-plan execution error carrying tool-attempt summary for memory feedback.
#[derive(Debug)]
pub(in crate::agent) struct GraphPlanExecutionError {
    pub(in crate::agent) error: anyhow::Error,
    pub(in crate::agent) tool_summary: ToolExecutionSummary,
}

#[derive(Debug)]
pub(super) struct GraphPlanExecutionState {
    pub(super) tool_summary: ToolExecutionSummary,
    pub(super) invoke_output: Option<String>,
    pub(super) invoke_error: Option<anyhow::Error>,
    pub(super) invoke_seen: bool,
    pub(super) invoked_tool_name: String,
    pub(super) step_traces: Vec<crate::contracts::RouteTraceGraphStep>,
    pub(super) failure_taxonomy: BTreeSet<String>,
    pub(super) fallback_applied: bool,
}

impl GraphPlanExecutionState {
    pub(super) fn new(plan: &GraphExecutionPlan) -> Self {
        Self {
            tool_summary: ToolExecutionSummary::default(),
            invoke_output: None,
            invoke_error: None,
            invoke_seen: false,
            invoked_tool_name: plan.tool_name.clone(),
            step_traces: Vec::new(),
            failure_taxonomy: BTreeSet::new(),
            fallback_applied: false,
        }
    }

    pub(super) fn step_attempt(&self, step: &GraphPlanStep) -> u32 {
        match step.kind {
            GraphPlanStepKind::InvokeGraphTool => self.tool_summary.attempted.saturating_add(1),
            _ => self.tool_summary.attempted,
        }
    }

    pub(super) fn classify_and_record_failure(&mut self, reason: &str) {
        self.failure_taxonomy
            .insert(super::trace::classify_failure_taxonomy(reason));
    }
}

pub(super) struct GraphPlanExecutionContext<'a> {
    pub(super) session_id: &'a str,
    pub(super) decision: &'a OmegaDecision,
    pub(super) plan: &'a GraphExecutionPlan,
    pub(super) input: &'a GraphPlanExecutionInput,
    pub(super) execution_started: Instant,
}

pub(super) struct StepFailureMeta {
    pub(super) step_attempt: u32,
    pub(super) step_started_at: Instant,
    pub(super) trace_status: &'static str,
    pub(super) is_transport_failure: bool,
}
