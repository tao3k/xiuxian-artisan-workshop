//! Context annotation mechanism using Qianhuan.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};

/// Mechanism responsible for transmuting raw facts into persona-aligned context snapshots.
pub struct ContextAnnotator {
    /// Reference to the `ThousandFaces` orchestrator.
    pub orchestrator: Arc<ThousandFacesOrchestrator>,
    /// Reference to the Persona Registry.
    pub registry: Arc<PersonaRegistry>,
    /// Target persona ID defined in the registry.
    pub persona_id: String,
}

#[async_trait]
impl QianjiMechanism for ContextAnnotator {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let raw_facts = match context.get("raw_facts") {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let persona = self
            .registry
            .get(&self.persona_id)
            .ok_or(format!("Persona '{}' not found", self.persona_id))?;

        let snapshot = self
            .orchestrator
            .assemble_snapshot(
                persona,
                vec![raw_facts.to_string()],
                "Working History Placeholder",
            )
            .await
            .map_err(|e| format!("Qianhuan annotation failed: {e}"))?;

        Ok(QianjiOutput {
            data: json!({ "annotated_prompt": snapshot }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        8.0
    }
}
