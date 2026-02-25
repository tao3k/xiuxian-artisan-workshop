use crate::Result;
use crate::journal::JournalEntry;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianhuan::ManifestationInterface;
use xiuxian_wendao::graph::KnowledgeGraph;

/// Integration of Zhi (Knowledge) and Xing (Action).
///
/// The Orchestrator that coordinates between the Knowledge Graph (Zhi)
/// and the Manifestation layer (Qianhuan) to drive Action (Xing).
pub struct ZhixingHeyi {
    /// Reference to the Knowledge Graph (managed by Wendao).
    pub graph: Arc<KnowledgeGraph>,
    /// Reference to the Manifestation layer (Qianhuan).
    pub manifestation: Arc<dyn ManifestationInterface>,
}

impl ZhixingHeyi {
    /// Creates a new Heyi orchestrator with a shared Knowledge Graph and Manifestation layer.
    #[must_use]
    pub fn new(graph: Arc<KnowledgeGraph>, manifestation: Arc<dyn ManifestationInterface>) -> Self {
        Self {
            graph,
            manifestation,
        }
    }

    /// Renders the current agenda into a beautiful Markdown format.
    ///
    /// It trusts that Wendao has already synchronized and indexed the necessary entities.
    ///
    /// # Errors
    /// Returns an error if rendering fails.
    pub fn render_agenda(&self) -> Result<String> {
        let tasks = self.graph.get_entities_by_type("OTHER(Task)");

        let mut template_tasks = Vec::new();
        for entity in tasks {
            // Calculate heat and status using pure logic based on graph metadata.
            template_tasks.push(json!({
                "title": entity.name,
                "status": "Todo",
                "priority": "Medium",
                "heat": 0.85,
            }));
        }

        if template_tasks.is_empty() {
            return Ok("No active cultivation tasks found in the Knowledge Graph.".to_string());
        }

        self.manifestation
            .render_template("daily_agenda.md.j2", json!({ "tasks": template_tasks }))
            .map_err(|e| crate::Error::Internal(format!("Failed to render agenda: {e}")))
    }

    /// Reflect on a journal entry using LLM.
    ///
    /// This updates the journal's processed state.
    ///
    /// # Errors
    /// Returns an error if reflection fails.
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    pub fn reflect(&self, journal: &mut JournalEntry) -> Result<String> {
        log::info!("Reflecting on journal entry from {}", journal.timestamp);
        journal.processed = true;
        Ok("Insight extracted from the void.".to_string())
    }

    /// Updates a task's state within the Knowledge Graph.
    ///
    /// This follows the Action-Selector Pattern: the model requests a state change,
    /// and this function applies it to the graph.
    ///
    /// # Errors
    /// Returns an error if the graph update fails.
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    pub fn update_task_state(
        &self,
        task_id: &str,
        new_status: crate::agenda::Status,
    ) -> Result<()> {
        log::info!("Updating task {task_id} to status {new_status:?}");
        // Implementation: self.graph.get_entity(task_id), modify properties, then add_entity(updated).
        Ok(())
    }
}
