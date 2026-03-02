use super::models::{ConsensusMode, ConsensusPolicy};

pub(super) fn required_weight_threshold(policy: &ConsensusPolicy, total_agents: usize) -> f64 {
    match policy.mode {
        ConsensusMode::Majority => {
            let majority_votes = total_agents.saturating_div(2).saturating_add(1);
            let majority_u32 = u32::try_from(majority_votes).unwrap_or(u32::MAX);
            f64::from(majority_u32)
        }
        ConsensusMode::Unanimous => {
            let agents_u32 = u32::try_from(total_agents).unwrap_or(u32::MAX);
            f64::from(agents_u32)
        }
        ConsensusMode::Weighted => f64::from(policy.weight_threshold.max(0.0)),
    }
}
