use super::super::QianjiScheduler;
use super::super::types::{
    ConsensusCheckpointView, ConsensusOutcome, EXTERNAL_PROGRESS_TIMEOUT_MS,
};
use crate::consensus::ConsensusResult;
use crate::contracts::NodeStatus;
use crate::error::QianjiError;
use crate::telemetry::ConsensusStatus;
use tokio::time::Duration;

impl QianjiScheduler {
    pub(in crate::scheduler::core) async fn resolve_consensus_output(
        &self,
        node_idx: petgraph::stable_graph::NodeIndex,
        output_data: &serde_json::Value,
        checkpoint: &ConsensusCheckpointView<'_>,
    ) -> Result<ConsensusOutcome, QianjiError> {
        let (node_id, consensus_policy) = {
            let engine = self.engine.read().await;
            (
                engine.graph[node_idx].id.clone(),
                engine.graph[node_idx].consensus.clone(),
            )
        };

        let (Some(policy), Some(manager), Some(sid)) = (
            consensus_policy,
            &self.consensus_manager,
            checkpoint.session_id,
        ) else {
            return Ok(ConsensusOutcome::Proceed(output_data.clone()));
        };

        let output_json = serde_json::to_string(output_data).unwrap_or_default();
        let output_hash = format!("{:x}", md5::compute(&output_json));
        let telemetry_target = Some(consensus_target_progress(&policy));

        match manager
            .submit_vote_with_payload(
                sid,
                &node_id,
                output_hash.clone(),
                Some(&output_json),
                &policy,
            )
            .await
            .map_err(|error| QianjiError::Execution(error.to_string()))?
        {
            ConsensusResult::Agreed(agreed_hash) => {
                self.emit_consensus_spike(
                    sid,
                    &node_id,
                    ConsensusStatus::Agreed,
                    Some(1.0),
                    telemetry_target,
                );
                let agreed_output = self
                    .read_agreed_output(
                        manager,
                        sid,
                        &node_id,
                        &output_hash,
                        &agreed_hash,
                        output_data,
                    )
                    .await?;
                Ok(ConsensusOutcome::Proceed(agreed_output))
            }
            ConsensusResult::Pending => {
                self.emit_consensus_spike(
                    sid,
                    &node_id,
                    ConsensusStatus::Pending,
                    None,
                    telemetry_target,
                );
                self.set_node_status(node_idx, NodeStatus::ConsensusPending)
                    .await;
                self.save_checkpoint_if_needed(
                    Some(sid),
                    checkpoint.redis_url,
                    checkpoint.total_steps,
                    checkpoint.active_branches,
                    checkpoint.context,
                )
                .await;

                let wait_ms = if policy.timeout_ms == 0 {
                    EXTERNAL_PROGRESS_TIMEOUT_MS
                } else {
                    policy.timeout_ms
                };
                let wait_result = manager
                    .wait_for_quorum(sid, &node_id, Duration::from_millis(wait_ms))
                    .await
                    .map_err(|error| QianjiError::Execution(error.to_string()))?;
                if let Some(agreed_hash) = wait_result {
                    let agreed_output = self
                        .read_agreed_output(
                            manager,
                            sid,
                            &node_id,
                            &output_hash,
                            &agreed_hash,
                            output_data,
                        )
                        .await?;
                    return Ok(ConsensusOutcome::Proceed(agreed_output));
                }

                Ok(ConsensusOutcome::Suspend(checkpoint.context.clone()))
            }
            ConsensusResult::Failed(reason) => {
                self.emit_consensus_spike(
                    sid,
                    &node_id,
                    ConsensusStatus::Failed,
                    None,
                    telemetry_target,
                );
                Err(QianjiError::Execution(format!(
                    "Consensus failed for {node_id}: {reason}"
                )))
            }
        }
    }
}

fn consensus_target_progress(policy: &crate::consensus::ConsensusPolicy) -> f32 {
    use crate::consensus::ConsensusMode;
    match policy.mode {
        ConsensusMode::Majority => 0.5,
        ConsensusMode::Unanimous => 1.0,
        ConsensusMode::Weighted => policy.weight_threshold.clamp(0.0, 1.0),
    }
}
