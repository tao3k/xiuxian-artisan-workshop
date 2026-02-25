mod crawl;
mod workflow_bridge;

use anyhow::Result;

use super::super::Agent;

impl Agent {
    pub(in crate::agent) async fn handle_shortcuts(
        &self,
        session_id: &str,
        user_message_owned: &mut String,
        force_react: &mut bool,
        turn_id: u64,
    ) -> Result<Option<String>> {
        if !*force_react
            && let Some(output) = self
                .handle_workflow_bridge_shortcut(
                    session_id,
                    user_message_owned,
                    force_react,
                    turn_id,
                )
                .await?
        {
            return Ok(Some(output));
        }

        if !*force_react
            && let Some(output) = self
                .handle_crawl_shortcut(session_id, user_message_owned.as_str(), turn_id)
                .await?
        {
            return Ok(Some(output));
        }

        Ok(None)
    }
}
