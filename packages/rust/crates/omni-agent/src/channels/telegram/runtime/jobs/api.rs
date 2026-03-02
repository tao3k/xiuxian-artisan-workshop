use std::sync::Arc;

use tokio::sync::mpsc;

use super::background_completion;
use super::command_router;
#[cfg(test)]
use super::observability;
use crate::agent::Agent;
use crate::channels::managed_runtime::ForegroundQueueMode;
use crate::channels::telegram::runtime::dispatch::ForegroundInterruptController;
use crate::channels::traits::{Channel, ChannelMessage};
use crate::jobs::{JobCompletion, JobManager};

#[cfg(test)]
pub(in crate::channels::telegram::runtime) fn log_preview(s: &str) -> String {
    observability::log_preview(s)
}

pub(in crate::channels::telegram::runtime) async fn handle_inbound_message_with_interrupt(
    msg: ChannelMessage,
    channel: &Arc<dyn Channel>,
    foreground_tx: &mpsc::Sender<ChannelMessage>,
    interrupt_controller: &ForegroundInterruptController,
    job_manager: &Arc<JobManager>,
    agent: &Arc<Agent>,
    queue_mode: ForegroundQueueMode,
) -> bool {
    command_router::handle_inbound_message(
        msg,
        channel,
        foreground_tx,
        interrupt_controller,
        job_manager,
        agent,
        queue_mode,
    )
    .await
}

pub(in crate::channels::telegram::runtime) async fn push_background_completion(
    channel: &Arc<dyn Channel>,
    completion: JobCompletion,
) {
    background_completion::push_background_completion(channel, completion).await;
}
