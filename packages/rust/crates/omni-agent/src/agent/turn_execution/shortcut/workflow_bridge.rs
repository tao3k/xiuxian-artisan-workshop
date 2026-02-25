use anyhow::Result;

use super::super::super::memory_recall_feedback::ToolExecutionSummary;
use super::super::super::{Agent, graph, injection, omega};
use crate::contracts::{OmegaDecision, OmegaRoute, RouteTraceInjection};
use crate::shortcuts::{GraphBridgeShortcut, parse_workflow_bridge_shortcut};

impl Agent {
    pub(super) async fn handle_workflow_bridge_shortcut(
        &self,
        session_id: &str,
        user_message_owned: &mut String,
        force_react: &mut bool,
        turn_id: u64,
    ) -> Result<Option<String>> {
        let Some(shortcut) = parse_workflow_bridge_shortcut(user_message_owned.as_str()) else {
            return Ok(None);
        };
        let decision = omega::apply_quality_gate(omega::decide_for_shortcut(
            shortcut.mode,
            user_message_owned.as_str(),
            &shortcut.tool_name,
        ));
        Self::record_omega_decision(
            session_id,
            &decision,
            Some(shortcut.mode),
            Some(shortcut.tool_name.as_str()),
        );

        if decision.route == OmegaRoute::Graph {
            return self
                .handle_graph_route_shortcut(
                    session_id,
                    user_message_owned,
                    force_react,
                    turn_id,
                    &shortcut,
                    &decision,
                )
                .await;
        }

        *force_react = true;
        Ok(None)
    }

    async fn handle_graph_route_shortcut(
        &self,
        session_id: &str,
        user_message_owned: &mut String,
        force_react: &mut bool,
        turn_id: u64,
        shortcut: &GraphBridgeShortcut,
        decision: &OmegaDecision,
    ) -> Result<Option<String>> {
        let shortcut_snapshot = self
            .build_shortcut_injection_snapshot(session_id, turn_id, user_message_owned.as_str())
            .await?;
        if let Some(snapshot) = &shortcut_snapshot {
            Self::record_injection_snapshot(session_id, snapshot);
        }

        let graph_plan = graph::build_shortcut_plan(shortcut.mode, decision, &shortcut.tool_name);
        Self::record_graph_plan(session_id, &graph_plan);
        let arguments = injection::augment_shortcut_arguments(
            shortcut.arguments.clone(),
            shortcut_snapshot.as_ref(),
            decision,
            shortcut.mode,
            Some(&graph_plan),
        );
        let execution = self
            .execute_graph_shortcut_plan(
                session_id,
                decision,
                &graph_plan,
                graph::GraphPlanExecutionInput {
                    workflow_mode: shortcut.mode,
                    turn_id,
                    shortcut_user_message: user_message_owned.clone(),
                    bridge_arguments_with_metadata: arguments,
                    bridge_arguments_without_metadata: shortcut.arguments.clone(),
                    injection: shortcut_snapshot
                        .as_ref()
                        .map(|snapshot| RouteTraceInjection {
                            blocks_used: snapshot.blocks.len() as u64,
                            chars_injected: snapshot.total_chars as u64,
                            dropped_by_budget: snapshot.dropped_block_ids.len() as u64,
                        }),
                },
            )
            .await;

        self.process_graph_shortcut_execution(
            session_id,
            user_message_owned,
            force_react,
            turn_id,
            decision.route,
            execution,
        )
        .await
    }

    async fn process_graph_shortcut_execution(
        &self,
        session_id: &str,
        user_message_owned: &mut String,
        force_react: &mut bool,
        turn_id: u64,
        route: OmegaRoute,
        execution: std::result::Result<
            graph::GraphPlanExecutionOutcome,
            graph::GraphPlanExecutionError,
        >,
    ) -> Result<Option<String>> {
        match execution {
            Ok(graph::GraphPlanExecutionOutcome::Completed {
                output,
                tool_summary,
            }) => {
                self.finalize_graph_shortcut_completion(
                    session_id,
                    user_message_owned.as_str(),
                    turn_id,
                    route,
                    &output,
                    &tool_summary,
                )
                .await?;
                Ok(Some(output))
            }
            Ok(graph::GraphPlanExecutionOutcome::RouteToReact {
                rewritten_user_message,
                tool_summary: _tool_summary,
            }) => {
                *force_react = true;
                *user_message_owned = rewritten_user_message;
                Ok(None)
            }
            Err(graph::GraphPlanExecutionError {
                error,
                tool_summary,
            }) => {
                self.handle_graph_shortcut_error(
                    session_id,
                    user_message_owned.as_str(),
                    turn_id,
                    route,
                    &error,
                    &tool_summary,
                )
                .await;
                Err(error)
            }
        }
    }

    async fn finalize_graph_shortcut_completion(
        &self,
        session_id: &str,
        user_message: &str,
        turn_id: u64,
        route: OmegaRoute,
        output: &str,
        tool_summary: &ToolExecutionSummary,
    ) -> Result<()> {
        let _ = self
            .update_recall_feedback(session_id, user_message, output, Some(tool_summary))
            .await;
        let effective_tool_count = tool_summary.attempted.max(1);
        self.append_turn_to_session(session_id, user_message, output, effective_tool_count)
            .await?;
        self.reflect_turn_and_update_policy_hint(
            session_id,
            turn_id,
            route,
            user_message,
            output,
            "completed",
            tool_summary.attempted,
        )
        .await;
        Ok(())
    }

    async fn handle_graph_shortcut_error(
        &self,
        session_id: &str,
        user_message: &str,
        turn_id: u64,
        route: OmegaRoute,
        error: &anyhow::Error,
        tool_summary: &ToolExecutionSummary,
    ) {
        let error_text = error.to_string();
        let _ = self
            .update_recall_feedback(session_id, user_message, &error_text, Some(tool_summary))
            .await;
        self.reflect_turn_and_update_policy_hint(
            session_id,
            turn_id,
            route,
            user_message,
            &error_text,
            "error",
            tool_summary.attempted,
        )
        .await;
    }
}
