use anyhow::Result;

use super::super::super::Agent;
use super::super::super::memory_recall_feedback::ToolExecutionSummary;
use crate::contracts::OmegaRoute;
use crate::shortcuts::{CRAWL_TOOL_NAME, parse_crawl_shortcut};

impl Agent {
    pub(super) async fn handle_crawl_shortcut(
        &self,
        session_id: &str,
        user_message: &str,
        turn_id: u64,
    ) -> Result<Option<String>> {
        let Some(shortcut) = parse_crawl_shortcut(user_message) else {
            return Ok(None);
        };
        let mut tool_summary = ToolExecutionSummary::default();
        let output = self
            .call_mcp_tool_with_diagnostics(CRAWL_TOOL_NAME, Some(shortcut.to_arguments()))
            .await;
        let out = match output {
            Ok(output) => {
                tool_summary.record_result(output.is_error);
                output.text
            }
            Err(error) => {
                tool_summary.record_transport_failure();
                self.handle_crawl_shortcut_error(
                    session_id,
                    user_message,
                    turn_id,
                    &error,
                    &tool_summary,
                )
                .await;
                return Err(error);
            }
        };

        let _ = self
            .update_recall_feedback(session_id, user_message, &out, Some(&tool_summary))
            .await;
        self.append_turn_to_session(session_id, user_message, &out, 1)
            .await?;
        self.reflect_turn_and_update_policy_hint(
            session_id,
            turn_id,
            OmegaRoute::React,
            user_message,
            &out,
            "completed",
            tool_summary.attempted,
        )
        .await;
        Ok(Some(out))
    }

    async fn handle_crawl_shortcut_error(
        &self,
        session_id: &str,
        user_message: &str,
        turn_id: u64,
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
            OmegaRoute::React,
            user_message,
            &error_text,
            "error",
            tool_summary.attempted,
        )
        .await;
    }
}
