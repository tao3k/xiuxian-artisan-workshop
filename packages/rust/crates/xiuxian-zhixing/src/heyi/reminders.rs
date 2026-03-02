use super::ZhixingHeyi;
use super::constants::{ATTR_TIMER_RECIPIENT, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED};
use super::schedule_time::render_scheduled_time_local;
use super::templating::escape_markdown_v2;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

/// Notification payload emitted by the timer watcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderSignal {
    /// Task entity ID.
    pub task_id: String,
    /// Task title rendered in reminder message.
    pub title: String,
    /// Optional task detail/body to clarify intended action.
    pub task_brief: Option<String>,
    /// Delivery target (for example `telegram:1304799691`).
    pub recipient: Option<String>,
    /// Human-readable scheduled time in local timezone.
    pub scheduled_local: Option<String>,
}

impl ZhixingHeyi {
    /// Backfill due-queue entries from graph tasks.
    ///
    /// This is intended for process bootstrap so reminders created before queue
    /// enablement are not dropped.
    pub fn backfill_reminder_queue(&self) {
        let Some(queue) = &self.reminder_queue else {
            return;
        };
        let tasks = self.graph.get_entities_by_type("OTHER(Task)");
        for entity in tasks {
            let scheduled = entity
                .metadata
                .get(ATTR_TIMER_SCHEDULED)
                .and_then(serde_json::Value::as_str);
            let reminded = entity
                .metadata
                .get(ATTR_TIMER_REMINDED)
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            if reminded {
                continue;
            }
            let Some(scheduled) = scheduled else {
                continue;
            };
            let recipient = entity
                .metadata
                .get(ATTR_TIMER_RECIPIENT)
                .and_then(serde_json::Value::as_str);
            if let Err(error) = queue.enqueue_task(
                &entity.id,
                &entity.name,
                Some(entity.description.as_str()),
                scheduled,
                recipient,
            ) {
                log::warn!(
                    "failed to backfill reminder queue entry for task_id={} scheduled_at={scheduled}: {error}",
                    entity.id
                );
            }
        }
    }

    /// Starts the background timer watcher to proactively monitor scheduled tasks.
    /// This fully encapsulates the domain logic of Agenda/Journal timeouts
    /// and uses an abstract channel to push notifications back to the host system.
    #[must_use]
    pub fn start_timer_watcher(
        self: Arc<Self>,
        notifier: Sender<ReminderSignal>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let poll_interval_seconds = self.reminder_queue.as_ref().map_or(
                60,
                super::reminder_queue::ReminderQueueStore::poll_interval_seconds,
            );
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(poll_interval_seconds));
            loop {
                interval.tick().await;
                let reminders = self.poll_reminders();
                for signal in reminders {
                    if notifier.send(signal).await.is_err() {
                        log::warn!("Timer watcher notification channel closed, stopping watcher.");
                        break;
                    }
                }
            }
        })
    }

    /// Checks for tasks that need immediate reminders in local time.
    ///
    /// Tasks scheduled within the next 15 minutes are returned once and then
    /// marked with `timer:reminded=true`.
    #[must_use]
    pub fn poll_reminders(&self) -> Vec<ReminderSignal> {
        if self.reminder_queue.is_some() {
            return self.poll_reminders_from_queue();
        }
        self.poll_reminders_from_graph()
    }

    fn poll_reminders_from_queue(&self) -> Vec<ReminderSignal> {
        let Some(queue) = &self.reminder_queue else {
            return Vec::new();
        };
        let due_records = match queue.poll_due(Utc::now().timestamp()) {
            Ok(records) => records,
            Err(error) => {
                log::warn!("Failed to poll reminder queue: {error}");
                return Vec::new();
            }
        };

        let mut reminders = Vec::with_capacity(due_records.len());
        for record in due_records {
            self.mark_task_reminded(record.task_id.as_str());
            reminders.push(record.into_signal(self.time_zone));
        }
        reminders
    }

    fn poll_reminders_from_graph(&self) -> Vec<ReminderSignal> {
        let tasks = self.graph.get_entities_by_type("OTHER(Task)");
        let now_local = Utc::now().with_timezone(&self.time_zone);
        let mut reminders = Vec::new();
        let mut pending_updates = Vec::new();

        for entity in tasks {
            let scheduled = entity
                .metadata
                .get(ATTR_TIMER_SCHEDULED)
                .and_then(serde_json::Value::as_str);
            let reminded = entity
                .metadata
                .get(ATTR_TIMER_REMINDED)
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let recipient = entity
                .metadata
                .get(ATTR_TIMER_RECIPIENT)
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string);

            let Some(scheduled) = scheduled else {
                continue;
            };
            let Ok(scheduled_at_utc) = DateTime::parse_from_rfc3339(scheduled) else {
                continue;
            };

            let scheduled_local = scheduled_at_utc.with_timezone(&self.time_zone);
            let reminder_window_start = scheduled_local - Duration::minutes(15);
            if !reminded && now_local >= reminder_window_start && now_local < scheduled_local {
                reminders.push(ReminderSignal {
                    task_id: entity.id.clone(),
                    title: entity.name.clone(),
                    task_brief: Some(entity.description.clone()),
                    recipient,
                    scheduled_local: Some(render_scheduled_time_local(scheduled, self.time_zone)),
                });
                let mut updated = entity.clone();
                updated
                    .metadata
                    .insert(ATTR_TIMER_REMINDED.to_string(), json!(true));
                pending_updates.push(updated);
            }
        }

        for updated in pending_updates {
            if let Err(error) = self.graph.add_entity(updated) {
                log::warn!("Failed to update reminder state in graph: {error}");
            }
        }

        reminders
    }

    fn mark_task_reminded(&self, task_id: &str) {
        let Some(mut entity) = self.graph.get_entity(task_id) else {
            log::warn!("reminder queue emitted unknown task id: {task_id}");
            return;
        };
        entity
            .metadata
            .insert(ATTR_TIMER_REMINDED.to_string(), json!(true));
        if let Err(error) = self.graph.add_entity(entity) {
            log::warn!(
                "failed to update reminder state in graph after queue poll for task_id={task_id}: {error}"
            );
        }
    }

    /// Render reminder notification in Telegram `MarkdownV2` format.
    ///
    /// The returned payload includes an internal prefix marker consumed by
    /// omni-agent notification providers to switch parse mode.
    ///
    /// # Errors
    /// Returns an error when template rendering fails.
    pub fn render_reminder_notice_markdown_v2(
        &self,
        signal: &ReminderSignal,
    ) -> crate::Result<String> {
        let task_brief_mdv2 = signal
            .task_brief
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty() && *value != signal.title)
            .map(escape_markdown_v2);
        let payload = json!({
            "task_title_mdv2": escape_markdown_v2(signal.title.as_str()),
            "task_brief_mdv2": task_brief_mdv2,
            "task_id_mdv2": escape_markdown_v2(signal.task_id.as_str()),
            "scheduled_local_mdv2": signal.scheduled_local.as_deref().map(escape_markdown_v2),
            "persona_name_mdv2": self
                .active_persona
                .as_ref()
                .map_or_else(
                    || "Agenda Steward".to_string(),
                    |persona| escape_markdown_v2(persona.name.as_str()),
                ),
        });
        let rendered =
            self.render_with_qianhuan_context("reminder_notice.md", payload, "TIMEBOX_EXECUTION")?;
        Ok(format!("[markdown_v2]\n{rendered}"))
    }
}
