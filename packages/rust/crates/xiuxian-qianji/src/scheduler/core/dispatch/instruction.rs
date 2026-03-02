use super::super::QianjiScheduler;
use crate::contracts::FlowInstruction;
use crate::error::QianjiError;
use std::collections::HashSet;

impl QianjiScheduler {
    pub(in crate::scheduler::core) async fn apply_instruction(
        &self,
        instruction: FlowInstruction,
        active_branches: &mut HashSet<String>,
    ) -> Result<Option<String>, QianjiError> {
        match instruction {
            FlowInstruction::SelectBranch(branch) => {
                active_branches.insert(branch);
                Ok(None)
            }
            FlowInstruction::RetryNodes(node_ids) => {
                self.reset_retry_nodes(&node_ids).await;
                Ok(None)
            }
            FlowInstruction::Suspend(reason) => Ok(Some(reason)),
            FlowInstruction::Abort(reason) => Err(QianjiError::Execution(reason)),
            FlowInstruction::Continue => Ok(None),
        }
    }
}
