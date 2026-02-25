mod steps;
mod trace;
mod types;

use std::collections::BTreeSet;
use std::time::Instant;

use anyhow::anyhow;

use crate::contracts::{
    GraphExecutionPlan, GraphPlanStep, OmegaDecision, RouteTrace, RouteTraceGraphStep,
};

use super::super::Agent;
#[cfg(test)]
use trace::ordered_steps;
use trace::{derive_tool_chain, push_step_trace};
use types::{GraphPlanExecutionContext, GraphPlanExecutionState, StepFailureMeta};

pub(in crate::agent) use types::{
    GraphPlanExecutionError, GraphPlanExecutionInput, GraphPlanExecutionOutcome,
};

impl Agent {
    pub(in crate::agent) async fn execute_graph_shortcut_plan(
        &self,
        session_id: &str,
        decision: &OmegaDecision,
        plan: &GraphExecutionPlan,
        input: GraphPlanExecutionInput,
    ) -> std::result::Result<GraphPlanExecutionOutcome, GraphPlanExecutionError> {
        let context = GraphPlanExecutionContext {
            session_id,
            decision,
            plan,
            input: &input,
            execution_started: Instant::now(),
        };
        let mut state = GraphPlanExecutionState::new(plan);
        let ordered_steps = self
            .resolve_ordered_steps_or_exit(&context, &mut state)
            .await?;

        for step in ordered_steps {
            if let Some(outcome) = self
                .execute_graph_plan_step(&context, &mut state, step)
                .await?
            {
                return Ok(outcome);
            }
        }

        self.finish_graph_plan_execution(&context, &mut state).await
    }

    async fn finish_graph_plan_execution(
        &self,
        context: &GraphPlanExecutionContext<'_>,
        state: &mut GraphPlanExecutionState,
    ) -> std::result::Result<GraphPlanExecutionOutcome, GraphPlanExecutionError> {
        if let Some(output) = state.invoke_output.take() {
            Self::record_graph_execution_completed(
                context.session_id,
                context.plan,
                state.tool_summary.attempted,
                output.len(),
            );
            self.emit_graph_trace_for_state(context, state).await;
            return Ok(GraphPlanExecutionOutcome::Completed {
                output,
                tool_summary: state.tool_summary,
            });
        }

        if let Some(error) = state.invoke_error.take() {
            return Err(self.fail_terminal_and_exit(context, state, error).await);
        }

        if !state.invoke_seen {
            let error = anyhow!(
                "graph plan `{}` did not include invoke_graph_tool step",
                context.plan.plan_id
            );
            return Err(self.fail_terminal_and_exit(context, state, error).await);
        }

        let error = anyhow!(
            "graph plan `{}` finished without bridge output or fallback",
            context.plan.plan_id
        );
        Err(self.fail_terminal_and_exit(context, state, error).await)
    }

    async fn fail_step_and_exit(
        &self,
        context: &GraphPlanExecutionContext<'_>,
        state: &mut GraphPlanExecutionState,
        step: &GraphPlanStep,
        meta: StepFailureMeta,
        error: anyhow::Error,
    ) -> GraphPlanExecutionError {
        if meta.is_transport_failure {
            state.tool_summary.record_transport_failure();
        }
        Self::record_graph_plan_step_failed(
            context.session_id,
            context.plan,
            step,
            meta.step_attempt,
            &error,
        );
        let reason = error.to_string();
        state.classify_and_record_failure(reason.as_str());
        push_step_trace(
            &mut state.step_traces,
            step,
            meta.step_attempt,
            meta.step_started_at,
            meta.trace_status,
            Some(reason),
        );
        self.emit_graph_trace_for_state(context, state).await;
        GraphPlanExecutionError {
            error,
            tool_summary: state.tool_summary,
        }
    }

    async fn fail_terminal_and_exit(
        &self,
        context: &GraphPlanExecutionContext<'_>,
        state: &mut GraphPlanExecutionState,
        error: anyhow::Error,
    ) -> GraphPlanExecutionError {
        state.classify_and_record_failure(error.to_string().as_str());
        self.emit_graph_trace_for_state(context, state).await;
        GraphPlanExecutionError {
            error,
            tool_summary: state.tool_summary,
        }
    }

    async fn emit_graph_trace_for_state(
        &self,
        context: &GraphPlanExecutionContext<'_>,
        state: &GraphPlanExecutionState,
    ) {
        self.emit_graph_route_trace(
            context.session_id,
            context.decision,
            context.plan,
            context.input,
            context.execution_started,
            state.fallback_applied,
            &state.failure_taxonomy,
            &state.step_traces,
        )
        .await;
    }

    #[allow(clippy::too_many_arguments)]
    async fn emit_graph_route_trace(
        &self,
        session_id: &str,
        decision: &OmegaDecision,
        plan: &GraphExecutionPlan,
        input: &GraphPlanExecutionInput,
        execution_started: Instant,
        fallback_applied: bool,
        failure_taxonomy: &BTreeSet<String>,
        step_traces: &[RouteTraceGraphStep],
    ) {
        let trace = RouteTrace {
            session_id: session_id.to_string(),
            turn_id: input.turn_id,
            selected_route: decision.route,
            confidence: decision.confidence,
            risk_level: decision.risk_level,
            tool_trust_class: decision.tool_trust_class,
            fallback_applied: Some(fallback_applied),
            fallback_policy: Some(decision.fallback_policy),
            tool_chain: derive_tool_chain(plan),
            latency_ms: Some(execution_started.elapsed().as_secs_f64() * 1000.0),
            failure_taxonomy: failure_taxonomy.iter().cloned().collect(),
            injection: input.injection.clone(),
            plan_id: Some(plan.plan_id.clone()),
            workflow_mode: Some(plan.workflow_mode),
            graph_steps: (!step_traces.is_empty()).then(|| step_traces.to_vec()),
        };
        self.record_route_trace(&trace).await;
    }
}

#[cfg(test)]
#[path = "../../../../tests/agent/graph_executor.rs"]
mod tests;
