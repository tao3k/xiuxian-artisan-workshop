use anyhow::Error;

use crate::agent::Agent;
use crate::agent::graph_bridge;
use crate::agent::omega::ShortcutFallbackAction;
use crate::contracts::GraphPlanStep;

use super::super::trace::{fallback_action_from_step, push_step_trace};
use super::super::types::{
    GraphPlanExecutionContext, GraphPlanExecutionError, GraphPlanExecutionOutcome,
    GraphPlanExecutionState, StepFailureMeta,
};

impl Agent {
    pub(super) async fn execute_evaluate_fallback_step(
        &self,
        context: &GraphPlanExecutionContext<'_>,
        state: &mut GraphPlanExecutionState,
        step: &GraphPlanStep,
        step_attempt: u32,
        step_started_at: std::time::Instant,
    ) -> std::result::Result<Option<GraphPlanExecutionOutcome>, GraphPlanExecutionError> {
        if state.invoke_output.is_some() {
            Self::record_graph_plan_step_succeeded(
                context.session_id,
                context.plan,
                step,
                step_attempt,
                "skipped_no_bridge_error",
            );
            push_step_trace(
                &mut state.step_traces,
                step,
                step_attempt,
                step_started_at,
                "skipped_no_bridge_error",
                None,
            );
            return Ok(None);
        }

        let Some(initial_error) = state.invoke_error.take() else {
            Self::record_graph_plan_step_succeeded(
                context.session_id,
                context.plan,
                step,
                step_attempt,
                "skipped_no_transport_error",
            );
            push_step_trace(
                &mut state.step_traces,
                step,
                step_attempt,
                step_started_at,
                "skipped_no_transport_error",
                None,
            );
            return Ok(None);
        };

        let fallback_action = match fallback_action_from_step(step) {
            Ok(action) => action,
            Err(error) => {
                return Err(self
                    .fail_step_and_exit(
                        context,
                        state,
                        step,
                        StepFailureMeta {
                            step_attempt,
                            step_started_at,
                            trace_status: "failed_invalid_fallback_action",
                            is_transport_failure: false,
                        },
                        error,
                    )
                    .await);
            }
        };

        match fallback_action {
            ShortcutFallbackAction::RetryBridgeWithoutMetadata => {
                self.execute_retry_fallback_step(
                    context,
                    state,
                    step,
                    step_started_at,
                    initial_error,
                )
                .await?;
                Ok(None)
            }
            ShortcutFallbackAction::RouteToReact => Ok(Some(
                self.execute_route_to_react_fallback_step(
                    context,
                    state,
                    step,
                    step_started_at,
                    initial_error,
                )
                .await,
            )),
            ShortcutFallbackAction::Abort => Err(self
                .execute_abort_fallback_step(context, state, step, step_started_at, initial_error)
                .await),
        }
    }

    async fn execute_retry_fallback_step(
        &self,
        context: &GraphPlanExecutionContext<'_>,
        state: &mut GraphPlanExecutionState,
        step: &GraphPlanStep,
        step_started_at: std::time::Instant,
        initial_error: Error,
    ) -> std::result::Result<(), GraphPlanExecutionError> {
        state.fallback_applied = true;
        Self::record_shortcut_fallback(
            context.session_id,
            context.decision,
            context.input.workflow_mode,
            state.invoked_tool_name.as_str(),
            ShortcutFallbackAction::RetryBridgeWithoutMetadata,
            &initial_error,
        );
        match self
            .execute_graph_bridge(graph_bridge::GraphBridgeRequest {
                tool_name: state.invoked_tool_name.clone(),
                arguments: context.input.bridge_arguments_without_metadata.clone(),
            })
            .await
        {
            Ok(result) => {
                state.tool_summary.record_result(result.is_error);
                state.invoke_output = Some(result.output);
                if result.is_error {
                    state
                        .failure_taxonomy
                        .insert("tool_error_payload".to_string());
                }
                let status = if result.is_error {
                    "retry_returned_error_payload"
                } else {
                    "retry_succeeded_without_metadata"
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
                Ok(())
            }
            Err(retry_error) => Err(self
                .fail_step_and_exit(
                    context,
                    state,
                    step,
                    StepFailureMeta {
                        step_attempt: state.tool_summary.attempted.saturating_add(1),
                        step_started_at,
                        trace_status: "retry_transport_failed",
                        is_transport_failure: true,
                    },
                    retry_error,
                )
                .await),
        }
    }

    async fn execute_route_to_react_fallback_step(
        &self,
        context: &GraphPlanExecutionContext<'_>,
        state: &mut GraphPlanExecutionState,
        step: &GraphPlanStep,
        step_started_at: std::time::Instant,
        initial_error: Error,
    ) -> GraphPlanExecutionOutcome {
        state.fallback_applied = true;
        Self::record_shortcut_fallback(
            context.session_id,
            context.decision,
            context.input.workflow_mode,
            state.invoked_tool_name.as_str(),
            ShortcutFallbackAction::RouteToReact,
            &initial_error,
        );
        Self::record_graph_plan_step_succeeded(
            context.session_id,
            context.plan,
            step,
            state.tool_summary.attempted,
            "rerouted_to_react",
        );
        state.classify_and_record_failure(initial_error.to_string().as_str());
        push_step_trace(
            &mut state.step_traces,
            step,
            state.tool_summary.attempted,
            step_started_at,
            "rerouted_to_react",
            Some(initial_error.to_string()),
        );
        Self::record_graph_execution_rerouted(
            context.session_id,
            context.plan,
            "react",
            "bridge_transport_error",
        );
        self.emit_graph_trace_for_state(context, state).await;
        GraphPlanExecutionOutcome::RouteToReact {
            rewritten_user_message: format!(
                "Execute this task with ReAct because workflow bridge failed: {}",
                context.input.shortcut_user_message
            ),
            tool_summary: state.tool_summary,
        }
    }

    async fn execute_abort_fallback_step(
        &self,
        context: &GraphPlanExecutionContext<'_>,
        state: &mut GraphPlanExecutionState,
        step: &GraphPlanStep,
        step_started_at: std::time::Instant,
        initial_error: Error,
    ) -> GraphPlanExecutionError {
        state.fallback_applied = true;
        Self::record_shortcut_fallback(
            context.session_id,
            context.decision,
            context.input.workflow_mode,
            state.invoked_tool_name.as_str(),
            ShortcutFallbackAction::Abort,
            &initial_error,
        );
        self.fail_step_and_exit(
            context,
            state,
            step,
            StepFailureMeta {
                step_attempt: state.tool_summary.attempted,
                step_started_at,
                trace_status: "fallback_abort",
                is_transport_failure: false,
            },
            initial_error,
        )
        .await
    }
}
