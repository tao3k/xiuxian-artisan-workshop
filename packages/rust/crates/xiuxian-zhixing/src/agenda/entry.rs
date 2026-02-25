use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::agenda::status::Status;
use crate::agenda::priority::Priority;

/// A task in the Xiuxian-Zhixing agenda.
/// 
/// Represents a "Vow" (愿) that must be manifested in reality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaEntry {
    /// Unique identifier for the task.
    pub id: Uuid,
    /// Short title of the task.
    pub title: String,
    /// Detailed explanation or context of the task.
    pub description: Option<String>,
    /// Current state in the task lifecycle.
    pub status: Status,
    /// Importance level of the task.
    pub priority: Priority,
    /// Categorization tags for grouping.
    pub tags: Vec<String>,
    
    /// Timestamp when the task was created.
    pub created_at: DateTime<Utc>,
    /// When the task is intended to be worked on.
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Hard deadline for task completion.
    pub deadline: Option<DateTime<Utc>>,
    /// Timestamp when the task reached a terminal state.
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Time-To-Live (hours) before the task is considered "stale" (Heart-Demon).
    pub ttl_hours: i32,
    /// Last interaction timestamp for heat calculation.
    pub last_active_at: DateTime<Utc>,
    
    /// Related entity IDs in the Knowledge Graph (Wendao).
    pub related_entities: Vec<String>,
}

impl AgendaEntry {
    /// Creates a new entry with default TTL and current timestamp.
    #[must_use]
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            status: Status::Todo,
            priority: Priority::Medium,
            tags: Vec::new(),
            created_at: now,
            scheduled_at: None,
            deadline: None,
            completed_at: None,
            ttl_hours: 72, // Default 3 days
            last_active_at: now,
            related_entities: Vec::new(),
        }
    }

    /// Calculate current "Heat" (1.0 = Fresh, 0.0 = Stale/Heart-Demon).
    /// Heat decays based on the time elapsed since `last_active_at`.
    #[must_use]
    pub fn calculate_heat(&self) -> f32 {
        if self.status.is_terminal() {
            return 1.0;
        }
        
        let elapsed = Utc::now().signed_duration_since(self.last_active_at).num_hours();
        if elapsed <= 0 {
            return 1.0;
        }
        
        #[allow(clippy::cast_precision_loss)]
        let heat = 1.0 - (elapsed as f32 / self.ttl_hours as f32);
        heat.clamp(0.0, 1.0)
    }

    /// Refreshes the heat by updating the `last_active_at` timestamp.
    pub fn refresh_heat(&mut self) {
        self.last_active_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_new_entry_is_fresh() {
        let entry = AgendaEntry::new("Test Task".to_string());
        assert!(entry.calculate_heat() > 0.99);
        assert_eq!(entry.status, Status::Todo);
    }

    #[test]
    fn test_heat_decay() {
        let mut entry = AgendaEntry::new("Decay Task".to_string());
        entry.ttl_hours = 10;
        // Mock 5 hours ago
        entry.last_active_at = Utc::now() - Duration::hours(5);
        
        let heat = entry.calculate_heat();
        assert!(heat > 0.4 && heat < 0.6, "Heat should be around 0.5, got {heat}");
    }

    #[test]
    fn test_heart_demon_trigger() {
        let mut entry = AgendaEntry::new("Stale Task".to_string());
        entry.ttl_hours = 24;
        // Mock 25 hours ago
        entry.last_active_at = Utc::now() - Duration::hours(25);
        
        assert_eq!(entry.calculate_heat(), 0.0);
    }
}
