use super::macros::define_native_tool;
use serde_json::json;
use std::sync::Arc;
use xiuxian_zhixing::ZhixingHeyi;

fn reminder_recipient_from_session_id(session_id: Option<&str>) -> Option<String> {
    let raw = session_id?;
    let (channel, key) = raw.split_once(':')?;
    let channel = channel.trim();
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    match channel {
        "telegram" => Some(format!("telegram:{key}")),
        "discord" => Some(format!("discord:{key}")),
        _ => None,
    }
}

define_native_tool! {
    /// Native tool for recording journal entries.
    pub struct JournalRecordTool {
        /// Reference to the Heyi orchestrator.
        pub heyi: Arc<ZhixingHeyi>,
    }
    name: "journal.record",
    description: "Record a daily journal entry or reflection. Use this to compile raw thoughts into structured insights.",
    parameters: json!({
            "type": "object",
            "properties": {
                "content": { "type": "string", "description": "The unstructured journal content" }
            },
            "required": ["content"]
        }),
    call(|tool, arguments, _context| {
        let content = arguments
            .and_then(|a| a["content"].as_str().map(ToString::to_string))
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' argument"))?;

        let mut journal = xiuxian_zhixing::journal::JournalEntry::new(content);
        let insight = tool.heyi.reflect(&mut journal).await?;
        Ok(format!("Journal recorded. Insight: {insight}"))
    })
}

define_native_tool! {
    /// Native tool for adding a specific task to the agenda.
    pub struct TaskAddTool {
        /// Reference to the Heyi orchestrator.
        pub heyi: Arc<ZhixingHeyi>,
    }
    name: "task.add",
    description: "Add a new task or 'Vow' to your cultivation agenda. If the user provides a time, pass it as plain local time in `time` (for example `2026-02-25 10:09 PM`, `2026-02-25 22:09`, `22:09`, `in 30 minutes`). The backend normalizes timezone/RFC3339 internally.",
    parameters: json!({
            "type": "object",
            "properties": {
                "title": { "type": "string", "description": "The title or description of the task" },
                "time": { "type": "string", "description": "Optional: User-facing local scheduled time. Preferred field." },
                "scheduled_at": { "type": "string", "description": "Optional legacy alias. Accepted for compatibility; backend still normalizes." }
            },
            "required": ["title"]
        }),
    call(|tool, arguments, context| {
        let title = arguments
            .as_ref()
            .and_then(|a| a["title"].as_str().map(ToString::to_string))
            .ok_or_else(|| anyhow::anyhow!("Missing 'title' argument"))?;

        let scheduled_at = arguments.as_ref().and_then(|args| {
            args.get("time")
                .or_else(|| args.get("scheduled_at"))
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        });
        let reminder_recipient =
            reminder_recipient_from_session_id(context.session_id.as_deref());

        // Use the specialized add_task method to handle exact time manifestations
        let result = tool
            .heyi
            .add_task(&title, scheduled_at, reminder_recipient)
            .await?;
        Ok(result)
    })
}

define_native_tool! {
    /// Native tool for viewing the agenda.
    pub struct AgendaViewTool {
        /// Reference to the Heyi orchestrator.
        pub heyi: Arc<ZhixingHeyi>,
    }
    name: "agenda.view",
    description: "View the current cultivation agenda, including active vows and critically stale tasks.",
    parameters: json!({ "type": "object", "properties": {} }),
    call(|tool, _args, _context| {
        tool.heyi.render_agenda().map_err(|e| anyhow::anyhow!(e))
    })
}
