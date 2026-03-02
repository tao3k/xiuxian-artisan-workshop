use super::super::QianjiScheduler;
use crate::consensus::ConsensusManager;
use crate::error::QianjiError;

impl QianjiScheduler {
    pub(super) async fn read_agreed_output(
        &self,
        manager: &ConsensusManager,
        session_id: &str,
        node_id: &str,
        local_hash: &str,
        agreed_hash: &str,
        local_output: &serde_json::Value,
    ) -> Result<serde_json::Value, QianjiError> {
        if agreed_hash == local_hash {
            return Ok(local_output.clone());
        }

        let payload = manager
            .get_output_payload(session_id, node_id, agreed_hash)
            .await
            .map_err(|error| QianjiError::Execution(error.to_string()))?;
        let Some(payload) = payload else {
            log::warn!(
                "Consensus agreed on hash '{agreed_hash}' for node '{node_id}', but no payload was stored; using local output fallback"
            );
            return Ok(local_output.clone());
        };

        serde_json::from_str::<serde_json::Value>(&payload).map_err(|error| {
            QianjiError::Execution(format!(
                "Failed to parse agreed consensus payload for node '{node_id}': {error}"
            ))
        })
    }
}
