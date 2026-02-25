mod react_loop;
mod shortcut;

#[allow(clippy::wildcard_imports)]
use super::*;

impl Agent {
    /// Execute one user turn with shortcut handling and fallback to the `ReAct` loop.
    ///
    /// # Errors
    /// Returns an error when shortcut execution or `ReAct` loop execution fails.
    pub async fn run_turn(&self, session_id: &str, user_message: &str) -> Result<String> {
        let forced_react_message = parse_react_shortcut(user_message);
        let mut force_react = forced_react_message.is_some();
        let mut user_message_owned =
            forced_react_message.unwrap_or_else(|| user_message.to_string());
        let turn_id = Self::next_runtime_turn_id();

        if let Some(out) = self
            .handle_shortcuts(
                session_id,
                &mut user_message_owned,
                &mut force_react,
                turn_id,
            )
            .await?
        {
            return Ok(out);
        }

        let user_message = user_message_owned.as_str();
        self.run_react_loop(session_id, user_message, force_react, turn_id)
            .await
    }
}
