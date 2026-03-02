use super::ZhixingHeyi;
use super::constants::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_RECIPIENT, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED,
};
use super::schedule_time::{normalize_scheduled_time_input, render_scheduled_time_local};
use crate::Result;
use crate::journal::JournalEntry;
use serde_json::json;
use xiuxian_wendao::{Entity, EntityType};

const TASK_TITLE_LIMIT: usize = 30;
const TASK_TITLE_PREFIX: usize = 27;
const DEFAULT_REMINDER_LEAD_MINUTES: u32 = 15;
const FALLBACK_TASK_TITLE: &str = "Task Created";

fn manifest_task_title(input: &str) -> String {
    let mut chars = input.chars();
    let prefix: String = chars.by_ref().take(TASK_TITLE_PREFIX).collect();
    if chars.next().is_some() {
        format!("{prefix}...")
    } else {
        input.to_string()
    }
}

fn build_task_entity(
    id: String,
    title: String,
    content: String,
    scheduled_at: Option<String>,
    reminder_recipient: Option<String>,
) -> Entity {
    let mut entity = Entity::new(id, title, EntityType::Other("Task".to_string()), content);
    entity
        .metadata
        .insert(ATTR_JOURNAL_CARRYOVER.to_string(), json!(0));
    entity
        .metadata
        .insert(ATTR_TIMER_REMINDED.to_string(), json!(false));
    if let Some(scheduled_at) = scheduled_at {
        entity
            .metadata
            .insert(ATTR_TIMER_SCHEDULED.to_string(), json!(scheduled_at));
    }
    if let Some(reminder_recipient) = reminder_recipient {
        entity
            .metadata
            .insert(ATTR_TIMER_RECIPIENT.to_string(), json!(reminder_recipient));
    }
    entity
}

impl ZhixingHeyi {
    /// Render a structured task-add confirmation from an existing task id.
    ///
    /// This is used by host adapters when an upstream model collapses a rich
    /// tool result into a bare `task:<id>` token.
    ///
    /// # Errors
    /// Returns an error when template rendering fails.
    pub fn render_task_add_response_from_id(&self, task_id: &str) -> Result<String> {
        let mut task_title = FALLBACK_TASK_TITLE.to_string();
        let mut task_detail: Option<String> = None;
        let mut scheduled_local: Option<String> = None;

        if let Some(entity) = self.graph.get_entity(task_id) {
            task_title = entity.name;
            let detail = entity.description.trim();
            if !detail.is_empty() && detail != task_title {
                task_detail = Some(detail.to_string());
            }
            scheduled_local = entity
                .metadata
                .get(ATTR_TIMER_SCHEDULED)
                .and_then(serde_json::Value::as_str)
                .map(|value| render_scheduled_time_local(value, self.time_zone));
        }

        self.render_with_qianhuan_context(
            "task_add_response.md",
            json!({
                "task_title": task_title,
                "task_detail": task_detail,
                "task_id": task_id,
                "scheduled_local": scheduled_local,
                "reminder_lead_minutes": DEFAULT_REMINDER_LEAD_MINUTES,
            }),
            "SUCCESS_STREAK",
        )
    }

    /// Reflects on a journal entry and manifests it as a task.
    ///
    /// # Errors
    /// Returns an error when journal persistence fails.
    pub async fn reflect(&self, journal: &mut JournalEntry) -> Result<String> {
        self.storage.record_journal(journal).await?;

        let task_name = if journal.content.chars().count() > TASK_TITLE_LIMIT {
            manifest_task_title(&journal.content)
        } else {
            journal.content.clone()
        };

        let task_entity = build_task_entity(
            format!("task:{}", journal.id),
            task_name.clone(),
            journal.content.clone(),
            None,
            None,
        );
        if let Err(error) = self.graph.add_entity(task_entity) {
            log::error!("Failed to update graph: {error}");
        }

        journal.processed = true;
        self.render_with_qianhuan_context(
            "journal_reflection.md",
            json!({
                "task_title": task_name,
                "task_id": format!("task:{}", journal.id),
                "journal_id": journal.id,
            }),
            "SUCCESS_STREAK",
        )
    }

    /// Adds a task with an optional scheduled time.
    ///
    /// # Errors
    /// Returns an error when journal persistence fails.
    pub async fn add_task(
        &self,
        title: &str,
        scheduled_at: Option<String>,
        reminder_recipient: Option<String>,
    ) -> Result<String> {
        self.check_heart_demon_blocker()?;
        let normalized_title = title.trim();
        if normalized_title.is_empty() {
            return Err(crate::Error::Logic(
                "task title cannot be empty".to_string(),
            ));
        }

        let journal = JournalEntry::new(normalized_title.to_string());
        self.storage.record_journal(&journal).await?;

        let task_name = normalized_title.to_string();
        let normalized_scheduled_at = scheduled_at
            .as_deref()
            .map(|value| {
                normalize_scheduled_time_input(value, self.time_zone).map_err(crate::Error::Config)
            })
            .transpose()?;
        let task_id = format!("task:{}", journal.id);

        let task_entity = build_task_entity(
            task_id.clone(),
            task_name.clone(),
            normalized_title.to_string(),
            normalized_scheduled_at.clone(),
            reminder_recipient.clone(),
        );
        if let Err(error) = self.graph.add_entity(task_entity) {
            log::error!("Failed to update graph: {error}");
        }

        if let (Some(queue), Some(scheduled_at)) =
            (&self.reminder_queue, normalized_scheduled_at.as_deref())
            && let Err(error) = queue.enqueue_task(
                &task_id,
                &task_name,
                Some(normalized_title),
                scheduled_at,
                reminder_recipient.as_deref(),
            )
        {
            log::warn!(
                "failed to enqueue reminder due queue entry for task_id={task_id} scheduled_at={scheduled_at}: {error}"
            );
        }

        let scheduled_local = normalized_scheduled_at
            .as_deref()
            .map(|value| render_scheduled_time_local(value, self.time_zone));
        self.render_with_qianhuan_context(
            "task_add_response.md",
            json!({
                "task_title": task_name,
                "task_detail": normalized_title,
                "task_id": task_id,
                "scheduled_local": scheduled_local,
                "reminder_lead_minutes": DEFAULT_REMINDER_LEAD_MINUTES,
            }),
            "SUCCESS_STREAK",
        )
    }
}
