pub(super) struct ParsedTelegramUpdate<'a> {
    pub(super) message: &'a serde_json::Value,
    pub(super) text: &'a str,
    pub(super) chat_id: String,
    pub(super) chat_title: &'a str,
    pub(super) chat_type: &'a str,
    pub(super) username: Option<&'a str>,
    pub(super) user_id: Option<String>,
    pub(super) message_thread_id: Option<i64>,
    pub(super) message_id: i64,
    pub(super) update_id: i64,
}

impl ParsedTelegramUpdate<'_> {
    pub(super) fn user_identity(&self) -> String {
        self.user_id
            .clone()
            .unwrap_or_else(|| self.username.unwrap_or("unknown").to_string())
    }

    pub(super) fn recipient(&self) -> String {
        if let Some(thread_id) = self.message_thread_id {
            return format!("{}:{thread_id}", self.chat_id);
        }
        self.chat_id.clone()
    }
}
