use std::time::{SystemTime, UNIX_EPOCH};

use xiuxian_db_store::{ValkeyQueueEntryId, ValkeyReadSession};

use super::{
    HotStateLeasedActivityTask, HotStateLeasedStep, HotStateSnapshot, ValkeyHotStateStore,
    control_error, decode_activity_lease_hash, decode_heartbeat, decode_lease_hash,
    decode_runnable_activity_task, decode_runnable_step, sort_hot_state_snapshot,
};
use crate::{ControlError, ControlResult, HotStateObservation, HotStateReadUsage};

const OBSERVATION_PIPELINE_SIZE: usize = 32;

impl ValkeyHotStateStore {
    pub(super) async fn collect_observation(
        &self,
        observed_at_ms: u64,
    ) -> ControlResult<HotStateSnapshot> {
        let reader = ValkeyReadSession::new(self.client.clone(), self.config.read_policy);
        let started = std::time::Instant::now();
        let mut observation = HotStateObservation {
            started_at_ms: wall_time_ms()?,
            ..Default::default()
        };
        let mut snapshot = tokio::time::timeout(
            self.config.read_policy.timeout,
            Box::pin(async {
                let pattern = self.config.heartbeat_key_pattern();
                let (pending, leased, activity_pending, activity_leased, heartbeats) =
                    tokio::try_join!(
                        reader.pending_entries(self.step_queue.keys()),
                        reader.lease_entries(self.step_queue.keys()),
                        reader.pending_entries(self.activity_queue.keys()),
                        reader.lease_entries(self.activity_queue.keys()),
                        reader.scan_keys(&pattern),
                    )
                    .map_err(control_error)?;
                let mut snapshot = HotStateSnapshot::new(observed_at_ms);
                self.collect_steps(&reader, &mut snapshot, &mut observation, pending, false)
                    .await?;
                self.collect_steps(&reader, &mut snapshot, &mut observation, leased, true)
                    .await?;
                self.collect_activities(
                    &reader,
                    &mut snapshot,
                    &mut observation,
                    activity_pending,
                    false,
                )
                .await?;
                self.collect_activities(
                    &reader,
                    &mut snapshot,
                    &mut observation,
                    activity_leased,
                    true,
                )
                .await?;
                for keys in heartbeats.chunks(OBSERVATION_PIPELINE_SIZE) {
                    let payloads = reader.string_batch(keys).await.map_err(control_error)?;
                    for payload in payloads {
                        match payload {
                            Some(payload) => {
                                snapshot.worker_heartbeats.push(decode_heartbeat(&payload)?);
                            }
                            None => observation.missing.heartbeats += 1,
                        }
                    }
                }
                sort_hot_state_snapshot(&mut snapshot);
                let usage = reader.finish().map_err(control_error)?;
                observation.usage = HotStateReadUsage {
                    commands: usage.commands,
                    items: usage.items,
                    bytes: usage.bytes,
                };
                Ok::<_, ControlError>(snapshot)
            }),
        )
        .await
        .map_err(|_| ControlError::ObservationBudgetExceeded {
            resource: "deadline",
        })??;
        observation.finished_at_ms = wall_time_ms()?;
        reader.finish().map_err(control_error)?;
        snapshot.collection_elapsed_ms =
            Some(u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX));
        snapshot.observation = Some(observation);
        Ok(snapshot)
    }

    async fn collect_steps(
        &self,
        reader: &ValkeyReadSession,
        snapshot: &mut HotStateSnapshot,
        observation: &mut HotStateObservation,
        entries: Vec<ValkeyQueueEntryId>,
        leased: bool,
    ) -> ControlResult<()> {
        for entries in entries.chunks(OBSERVATION_PIPELINE_SIZE) {
            let payloads = reader
                .payload_batch(self.step_queue.keys(), entries)
                .await
                .map_err(control_error)?;
            let mut leased_steps = Vec::with_capacity(entries.len());
            for (entry, payload) in entries.iter().zip(payloads) {
                let Some(payload) = payload else {
                    if leased {
                        observation.missing.leased_step_payloads += 1;
                    } else {
                        observation.missing.pending_step_payloads += 1;
                    }
                    continue;
                };
                let step = decode_runnable_step(&payload)?;
                if leased {
                    leased_steps.push((entry.clone(), step));
                } else {
                    snapshot.pending_steps.push(step);
                }
            }
            if leased {
                let lease_entries: Vec<_> = leased_steps
                    .iter()
                    .map(|(entry, _)| entry.clone())
                    .collect();
                let lease_fields = reader
                    .lease_batch(self.step_queue.keys(), &lease_entries)
                    .await
                    .map_err(control_error)?;
                for ((_, step), fields) in leased_steps.into_iter().zip(lease_fields) {
                    if let Some(lease) = decode_lease_hash(&step, &fields)? {
                        snapshot
                            .leased_steps
                            .push(HotStateLeasedStep { step, lease });
                    } else {
                        observation.missing.step_leases += 1;
                    }
                }
            }
        }
        Ok(())
    }

    async fn collect_activities(
        &self,
        reader: &ValkeyReadSession,
        snapshot: &mut HotStateSnapshot,
        observation: &mut HotStateObservation,
        entries: Vec<ValkeyQueueEntryId>,
        leased: bool,
    ) -> ControlResult<()> {
        for entries in entries.chunks(OBSERVATION_PIPELINE_SIZE) {
            let payloads = reader
                .payload_batch(self.activity_queue.keys(), entries)
                .await
                .map_err(control_error)?;
            let mut leased_tasks = Vec::with_capacity(entries.len());
            for (entry, payload) in entries.iter().zip(payloads) {
                let Some(payload) = payload else {
                    if leased {
                        observation.missing.leased_activity_payloads += 1;
                    } else {
                        observation.missing.pending_activity_payloads += 1;
                    }
                    continue;
                };
                let activity_task = decode_runnable_activity_task(&payload)?;
                if leased {
                    leased_tasks.push((entry.clone(), activity_task));
                } else {
                    snapshot.pending_activity_tasks.push(activity_task);
                }
            }
            if leased {
                let lease_entries: Vec<_> = leased_tasks
                    .iter()
                    .map(|(entry, _)| entry.clone())
                    .collect();
                let lease_fields = reader
                    .lease_batch(self.activity_queue.keys(), &lease_entries)
                    .await
                    .map_err(control_error)?;
                for ((_, activity_task), fields) in leased_tasks.into_iter().zip(lease_fields) {
                    if let Some(lease) = decode_activity_lease_hash(&activity_task, &fields)? {
                        snapshot
                            .leased_activity_tasks
                            .push(HotStateLeasedActivityTask {
                                activity_task,
                                lease,
                            });
                    } else {
                        observation.missing.activity_leases += 1;
                    }
                }
            }
        }
        Ok(())
    }
}

fn wall_time_ms() -> ControlResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .ok_or_else(|| ControlError::Storage {
            operation: "observation_clock",
            message: "wall clock outside supported epoch".into(),
        })
}
