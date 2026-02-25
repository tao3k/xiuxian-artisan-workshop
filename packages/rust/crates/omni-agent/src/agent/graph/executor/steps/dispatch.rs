use crate::agent::Agent;
use crate::contracts::{GraphPlanStep, GraphPlanStepKind};

use super::super::trace::{ordered_steps, push_step_trace};
use super::super::types::{
    GraphPlanExecutionContext, GraphPlanExecutionError, GraphPlanExecutionOutcome,
    GraphPlanExecutionState,
};

impl Agent {
    pub(in crate::agent::graph::executor) async fn resolve_ordered_steps_or_exit<'a>(
        &self,
        context: &GraphPlanExecutionContext<'a>,
        state: &mut GraphPlanExecutionState,
    ) -> std::result::Result<Vec<&'a GraphPlanStep>, GraphPlanExecutionError> {
        match ordered_steps(context.plan) {
            Ok(steps) => Ok(steps),
            Err(error) => Err(self.fail_terminal_and_exit(context, state, error).await),
        }
    }

    pub(in crate::agent::graph::executor) async fn execute_graph_plan_step(
        &self,
        context: &GraphPlanExecutionContext<'_>,
        state: &mut GraphPlanExecutionState,
        step: &GraphPlanStep,
    ) -> std::result::Result<Option<GraphPlanExecutionOutcome>, GraphPlanExecutionError> {
        let step_attempt = state.step_attempt(step);
        let step_started_at = std::time::Instant::now();
        Self::record_graph_plan_step_started(context.session_id, context.plan, step, step_attempt);

        match step.kind {
            GraphPlanStepKind::PrepareInjectionContext => {
                Self::execute_prepare_injection_step(
                    context,
                    state,
                    step,
                    step_attempt,
                    step_started_at,
                );
                Ok(None)
            }
            GraphPlanStepKind::InvokeGraphTool => {
                self.execute_invoke_graph_tool_step(
                    context,
                    state,
                    step,
                    step_attempt,
                    step_started_at,
                )
                .await?;
                Ok(None)
            }
            GraphPlanStepKind::EvaluateFallback => {
                self.execute_evaluate_fallback_step(
                    context,
                    state,
                    step,
                    step_attempt,
                    step_started_at,
                )
                .await
            }
        }
    }

    fn execute_prepare_injection_step(
        context: &GraphPlanExecutionContext<'_>,
        state: &mut GraphPlanExecutionState,
        step: &GraphPlanStep,
        step_attempt: u32,
        step_started_at: std::time::Instant,
    ) {
        Self::record_graph_plan_step_succeeded(
            context.session_id,
            context.plan,
            step,
            step_attempt,
            "prepared",
        );
        push_step_trace(
            &mut state.step_traces,
            step,
            step_attempt,
            step_started_at,
            "prepared",
            None,
        );
    }
}
