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

pub(super) struct WebhookEventLoopContext<'a> {
    pub(super) inbound_rx: &'a mut mpsc::Receiver<ChannelMessage>,
    pub(super) completion_rx: &'a mut mpsc::Receiver<JobCompletion>,
    pub(super) inbound_tx: &'a mpsc::Sender<ChannelMessage>,
    pub(super) channel_for_send: &'a Arc<dyn Channel>,
    pub(super) foreground_tx: &'a mpsc::Sender<ChannelMessage>,
    pub(super) interrupt_controller: &'a ForegroundInterruptController,
    pub(super) job_manager: &'a Arc<JobManager>,
    pub(super) agent: &'a Arc<Agent>,
    pub(super) webhook_server: &'a mut tokio::task::JoinHandle<std::io::Result<()>>,
    pub(super) runtime_config: TelegramRuntimeConfig,
}

pub(super) async fn run_webhook_event_loop(context: WebhookEventLoopContext<'_>) {
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
            maybe_msg = context.inbound_rx.recv() => {
                let Some(msg) = maybe_msg else {
                    break;
                };
                if !handle_inbound_message_with_interrupt(
                    msg,
                    context.channel_for_send,
                    context.foreground_tx,
                    context.interrupt_controller,
                    context.job_manager,
                    context.agent,
                    context.runtime_config.foreground_queue_mode,
                )
                .await
                {
                    break;
                }
            }
            maybe_completion = context.completion_rx.recv() => {
                let Some(completion) = maybe_completion else {
                    continue;
                };
                push_background_completion(context.channel_for_send, completion).await;
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down...");
                break;
            }
            _ = health_tick.tick() => {
                if drain_finished_webhook_server(context.webhook_server).await {
                    break;
                }
            }
            () = async {
                if let Some(interval) = snapshot_tick.as_mut() {
                    let _ = interval.tick().await;
                }
            }, if snapshot_tick.is_some() => {
                let admission = context.agent.downstream_admission_runtime_snapshot();
                emit_runtime_snapshot(
                    "webhook",
                    context.inbound_tx,
                    context.foreground_tx,
                    context.runtime_config,
                    admission,
                );
            }
        }
    }
}
