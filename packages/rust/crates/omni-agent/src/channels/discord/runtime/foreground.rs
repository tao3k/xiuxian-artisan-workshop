use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;

use super::ForegroundInterruptController;
use super::dispatch::process_discord_message_with_interrupt;
use super::managed::push_background_completion;
use crate::agent::{Agent, DownstreamAdmissionRuntimeSnapshot};
use crate::channels::traits::{Channel, ChannelMessage};
use crate::jobs::{JobCompletion, JobManager, JobManagerConfig, TurnRunner};

pub(super) struct DiscordForegroundRuntime {
    agent: Arc<Agent>,
    channel_for_send: Arc<dyn Channel>,
    job_manager: Arc<JobManager>,
    interrupt_controller: ForegroundInterruptController,
    foreground_gate: Arc<Semaphore>,
    foreground_max_in_flight_messages: usize,
    foreground_tasks: JoinSet<()>,
    turn_timeout_secs: u64,
}

pub(super) struct DiscordForegroundSnapshot {
    pub max_in_flight_messages: usize,
    pub available_permits: usize,
    pub in_flight_messages: usize,
    pub task_count: usize,
}

impl DiscordForegroundRuntime {
    pub(super) async fn spawn_foreground_turn(&mut self, msg: ChannelMessage) {
        let session_key = msg.session_key.clone();
        let channel_name = msg.channel.clone();
        let recipient = msg.recipient.clone();
        let admission = self.agent.evaluate_downstream_admission();
        if !admission.admitted {
            if let Some(reason) = admission.reason {
                tracing::warn!(
                    event = "discord.foreground.admission_reject",
                    reason = reason.as_str(),
                    session_key = %session_key,
                    channel = %channel_name,
                    recipient = %recipient,
                    llm_saturation_pct = ?admission.snapshot.llm.map(|state| state.saturation_pct),
                    embedding_saturation_pct = ?admission.snapshot.embedding.map(|state| state.saturation_pct),
                    llm_reject_threshold_pct = admission.llm_reject_threshold_pct,
                    embedding_reject_threshold_pct = admission.embedding_reject_threshold_pct,
                    "discord foreground turn rejected by downstream admission control"
                );
                if let Err(error) = self
                    .channel_for_send
                    .send(reason.user_message(), &recipient)
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        session_key = %session_key,
                        recipient = %recipient,
                        "failed to send discord admission rejection notice"
                    );
                }
            }
            return;
        }

        let wait_started = Instant::now();
        let Ok(permit) = Arc::clone(&self.foreground_gate).acquire_owned().await else {
            tracing::warn!("discord foreground gate is closed");
            return;
        };
        let gate_wait_ms = u64::try_from(wait_started.elapsed().as_millis()).unwrap_or(u64::MAX);
        if gate_wait_ms >= 50 {
            tracing::warn!(
                event = "discord.foreground.gate_wait",
                wait_ms = gate_wait_ms,
                session_key = %session_key,
                channel = %channel_name,
                recipient = %recipient,
                "discord foreground gate waited before scheduling turn"
            );
        }

        let agent = Arc::clone(&self.agent);
        let channel = Arc::clone(&self.channel_for_send);
        let job_manager = Arc::clone(&self.job_manager);
        let interrupt_controller = self.interrupt_controller.clone();
        let turn_timeout_secs = self.turn_timeout_secs;
        self.foreground_tasks.spawn(async move {
            let _permit = permit;
            process_discord_message_with_interrupt(
                agent,
                channel,
                msg,
                &job_manager,
                turn_timeout_secs,
                &interrupt_controller,
            )
            .await;
        });
    }

    pub(super) async fn push_completion(&self, completion: JobCompletion) {
        push_background_completion(&self.channel_for_send, completion).await;
    }

    pub(super) fn has_foreground_tasks(&self) -> bool {
        !self.foreground_tasks.is_empty()
    }

    pub(super) async fn join_next_foreground_task(&mut self) {
        if let Some(Err(error)) = self.foreground_tasks.join_next().await {
            tracing::warn!(error = %error, "discord foreground worker task join error");
        }
    }

    pub(super) async fn abort_and_drain_foreground_tasks(&mut self) {
        self.foreground_tasks.abort_all();
        while let Some(result) = self.foreground_tasks.join_next().await {
            if let Err(error) = result {
                tracing::warn!(error = %error, "discord foreground worker task join error");
            }
        }
    }

    pub(super) fn snapshot(&self) -> DiscordForegroundSnapshot {
        let available_permits = self.foreground_gate.available_permits();
        let max_in_flight_messages = self.foreground_max_in_flight_messages;
        let in_flight_messages = max_in_flight_messages.saturating_sub(available_permits);
        DiscordForegroundSnapshot {
            max_in_flight_messages,
            available_permits,
            in_flight_messages,
            task_count: self.foreground_tasks.len(),
        }
    }

    pub(super) fn admission_runtime_snapshot(&self) -> DownstreamAdmissionRuntimeSnapshot {
        self.agent.downstream_admission_runtime_snapshot()
    }
}

pub(super) fn build_foreground_runtime(
    agent: Arc<Agent>,
    channel_for_send: Arc<dyn Channel>,
    turn_timeout_secs: u64,
    foreground_max_in_flight_messages: usize,
) -> (DiscordForegroundRuntime, mpsc::Receiver<JobCompletion>) {
    let runner: Arc<dyn TurnRunner> = agent.clone();
    let (job_manager, completion_rx) = JobManager::start(runner, JobManagerConfig::default());
    let runtime = DiscordForegroundRuntime {
        agent,
        channel_for_send,
        job_manager,
        interrupt_controller: ForegroundInterruptController::default(),
        foreground_gate: Arc::new(Semaphore::new(foreground_max_in_flight_messages)),
        foreground_max_in_flight_messages,
        foreground_tasks: JoinSet::new(),
        turn_timeout_secs,
    };
    (runtime, completion_rx)
}
