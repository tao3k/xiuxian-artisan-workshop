//! Adversarial calibration mechanism (Synapse-Audit).

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use serde_json::json;

/// Mechanism responsible for auditing conclusions against evidence.
pub struct SynapseCalibrator {
    /// ID of the node to be reset if calibration fails.
    pub target_node_id: String,
    /// Maximum allowed drift score before triggering a retry.
    pub drift_threshold: f32,
}

#[async_trait]
impl QianjiMechanism for SynapseCalibrator {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let drift_score = context_f32(context, "drift_score", 0.0);

        if drift_score > self.drift_threshold {
            Ok(QianjiOutput {
                data: json!({ "calibration_status": "failed", "reason": "Drift exceeds threshold" }),
                instruction: FlowInstruction::RetryNodes(vec![self.target_node_id.clone()]),
            })
        } else {
            Ok(QianjiOutput {
                data: json!({ "calibration_status": "passed" }),
                instruction: FlowInstruction::Continue,
            })
        }
    }

    fn weight(&self) -> f32 {
        10.0
    }
}

fn context_f32(context: &serde_json::Value, key: &str, default: f32) -> f32 {
    context
        .get(key)
        .cloned()
        .and_then(|value| serde_json::from_value::<f32>(value).ok())
        .unwrap_or(default)
}
