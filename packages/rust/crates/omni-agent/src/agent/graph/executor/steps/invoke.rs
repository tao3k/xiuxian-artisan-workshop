use anyhow::{Result, anyhow, bail};

use crate::agent::Agent;
use crate::agent::graph_bridge;
use crate::contracts::{GraphExecutionPlan, GraphPlanStep};

use super::super::trace::push_step_trace;
use super::super::types::{
    GraphPlanExecutionContext, GraphPlanExecutionError, GraphPlanExecutionState, StepFailureMeta,
};

impl Agent {
    pub(super) async fn execute_invoke_graph_tool_step(
        &self,
        context: &GraphPlanExecutionContext<'_>,
        state: &mut GraphPlanExecutionState,
        step: &GraphPlanStep,
        step_attempt: u32,
        step_started_at: std::time::Instant,
    ) -> std::result::Result<(), GraphPlanExecutionError> {
        if state.invoke_seen {
            let error = anyhow!(
                "graph plan `{}` contains duplicate invoke_graph_tool step",
                context.plan.plan_id
            );
            return Err(self
                .fail_step_and_exit(
                    context,
                    state,
                    step,
                    StepFailureMeta {
                        step_attempt,
                        step_started_at,
                        trace_status: "failed_duplicate_invoke_step",
                        is_transport_failure: false,
                    },
                    error,
                )
                .await);
        }
        state.invoke_seen = true;

        let step_tool_name = match Self::resolve_invoked_tool_name(context.plan, step) {
            Ok(name) => name,
            Err(error) => {
                return Err(self
                    .fail_step_and_exit(
                        context,
                        state,
                        step,
                        StepFailureMeta {
                            step_attempt,
                            step_started_at,
                            trace_status: "failed_empty_tool_name",
                            is_transport_failure: false,
                        },
                        error,
                    )
                    .await);
            }
        };
        state.invoked_tool_name.clone_from(&step_tool_name);

        let initial_request = graph_bridge::GraphBridgeRequest {
            tool_name: step_tool_name,
            arguments: context.input.bridge_arguments_with_metadata.clone(),
        };
        match self.execute_graph_bridge(initial_request).await {
            Ok(result) => {
                state.tool_summary.record_result(result.is_error);
                state.invoke_output = Some(result.output);
                if result.is_error {
                    state
                        .failure_taxonomy
                        .insert("tool_error_payload".to_string());
                }
                let status = if result.is_error {
                    "tool_returned_error_payload"
                } else {
                    "tool_call_succeeded"
                };
                Self::record_graph_plan_step_succeeded(
                    context.session_id,
                    context.plan,
                    step,
                    state.tool_summary.attempted,
                    status,
                );
                push_step_trace(
                    &mut state.step_traces,
                    step,
                    state.tool_summary.attempted,
                    step_started_at,
                    status,
                    None,
                );
            }
            Err(error) => {
                state.tool_summary.record_transport_failure();
                Self::record_graph_plan_step_failed(
                    context.session_id,
                    context.plan,
                    step,
                    state.tool_summary.attempted,
                    &error,
                );
                state.classify_and_record_failure(error.to_string().as_str());
                push_step_trace(
                    &mut state.step_traces,
                    step,
                    state.tool_summary.attempted,
                    step_started_at,
                    "tool_call_transport_failed",
                    Some(error.to_string()),
                );
                state.invoke_error = Some(error);
            }
        }
        Ok(())
    }

    fn resolve_invoked_tool_name(
        plan: &GraphExecutionPlan,
        step: &GraphPlanStep,
    ) -> Result<String> {
        let step_tool_name = step
            .tool_name
            .as_deref()
            .unwrap_or(plan.tool_name.as_str())
            .trim();
        if step_tool_name.is_empty() {
            bail!(
                "graph plan `{}` invoke step has empty tool name",
                plan.plan_id
            );
        }
        Ok(step_tool_name.to_string())
    }
}
