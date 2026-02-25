use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use crate::agent::Agent;
use crate::channels::telegram::runtime::dispatch::ForegroundInterruptController;
use crate::channels::telegram::runtime::telemetry::{
    emit_runtime_snapshot, snapshot_interval_from_env,
};
use crate::channels::telegram::runtime_config::TelegramRuntimeConfig;
use crate::channels::traits::{Channel, ChannelMessage};
use crate::jobs::{JobCompletion, JobManager};

use super::super::jobs::{handle_inbound_message_with_interrupt, push_background_completion};
use super::server::drain_finished_webhook_server;

#[allow(clippy::too_many_arguments)]
pub(super) async fn run_webhook_event_loop(
    inbound_rx: &mut mpsc::Receiver<ChannelMessage>,
    completion_rx: &mut mpsc::Receiver<JobCompletion>,
    inbound_tx: &mpsc::Sender<ChannelMessage>,
    channel_for_send: &Arc<dyn Channel>,
    foreground_tx: &mpsc::Sender<ChannelMessage>,
    interrupt_controller: &ForegroundInterruptController,
    job_manager: &Arc<JobManager>,
    agent: &Arc<Agent>,
    webhook_server: &mut tokio::task::JoinHandle<std::io::Result<()>>,
    runtime_config: TelegramRuntimeConfig,
) {
    let mut health_tick = tokio::time::interval(Duration::from_secs(1));
    let mut snapshot_tick = snapshot_interval_from_env().map(|period| {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        interval
    });
    if let Some(interval) = snapshot_tick.as_mut() {
        let _ = interval.tick().await;
    }
    loop {
        tokio::select! {
            maybe_msg = inbound_rx.recv() => {
                let Some(msg) = maybe_msg else {
                    break;
                };
                if !handle_inbound_message_with_interrupt(msg, channel_for_send, foreground_tx, interrupt_controller, job_manager, agent).await {
                    break;
                }
            }
            maybe_completion = completion_rx.recv() => {
                let Some(completion) = maybe_completion else {
                    continue;
                };
                push_background_completion(channel_for_send, completion).await;
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down...");
                break;
            }
            _ = health_tick.tick() => {
                if drain_finished_webhook_server(webhook_server).await {
                    break;
                }
            }
            () = async {
                if let Some(interval) = snapshot_tick.as_mut() {
                    let _ = interval.tick().await;
                }
            }, if snapshot_tick.is_some() => {
                let admission = agent.downstream_admission_runtime_snapshot();
                emit_runtime_snapshot("webhook", inbound_tx, foreground_tx, runtime_config, admission);
            }
        }
    }
}
