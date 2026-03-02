use std::sync::Arc;

use tokio::sync::mpsc;

use super::background;
use super::foreground;
use super::preempt;
use super::session;
use crate::agent::Agent;
use crate::channels::managed_runtime::ForegroundQueueMode;
use crate::channels::managed_runtime::turn::build_session_id;
use crate::channels::telegram::runtime::dispatch::ForegroundInterruptController;
use crate::channels::traits::{Channel, ChannelMessage};
use crate::jobs::JobManager;

pub(in crate::channels::telegram::runtime::jobs) async fn handle_inbound_message(
    msg: ChannelMessage,
    channel: &Arc<dyn Channel>,
    foreground_tx: &mpsc::Sender<ChannelMessage>,
    interrupt_controller: &ForegroundInterruptController,
    job_manager: &Arc<JobManager>,
    agent: &Arc<Agent>,
    queue_mode: ForegroundQueueMode,
) -> bool {
    let session_id = build_session_id(&msg.channel, &msg.session_key);

    if session::try_handle(&msg, channel, agent, interrupt_controller, &session_id).await {
        return true;
    }
    if background::try_handle(&msg, channel, job_manager, &session_id).await {
        return true;
    }

    if queue_mode.should_interrupt_on_new_message() {
        preempt::interrupt_active_turn_for_new_message(interrupt_controller, &session_id, &msg);
    }
    foreground::forward(msg, channel, agent, foreground_tx).await
}
