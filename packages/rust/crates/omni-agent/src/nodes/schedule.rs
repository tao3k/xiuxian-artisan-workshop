use std::path::PathBuf;
use std::sync::Arc;

use omni_agent::{
    JobManager, JobManagerConfig, RecurringScheduleConfig, RuntimeSettings, TurnRunner,
    run_recurring_schedule,
};

use crate::runtime_agent_factory::build_agent;

pub(crate) struct ScheduleModeRequest<'a> {
    pub(crate) prompt: String,
    pub(crate) interval_secs: u64,
    pub(crate) max_runs: Option<u64>,
    pub(crate) schedule_id: String,
    pub(crate) session_prefix: String,
    pub(crate) recipient: String,
    pub(crate) wait_for_completion_secs: u64,
    pub(crate) mcp_config_path: PathBuf,
    pub(crate) runtime_settings: &'a RuntimeSettings,
}

pub(crate) async fn run_schedule_mode(request: ScheduleModeRequest<'_>) -> anyhow::Result<()> {
    let ScheduleModeRequest {
        prompt,
        interval_secs,
        max_runs,
        schedule_id,
        session_prefix,
        recipient,
        wait_for_completion_secs,
        mcp_config_path,
        runtime_settings,
    } = request;

    let runner: Arc<dyn TurnRunner> =
        Arc::new(build_agent(&mcp_config_path, runtime_settings).await?);
    let (job_manager, completion_rx) = JobManager::start(runner, JobManagerConfig::default());

    println!(
        "Starting scheduler: schedule_id={schedule_id} interval={}s max_runs={:?}",
        interval_secs.max(1),
        max_runs
    );
    let outcome = run_recurring_schedule(
        job_manager,
        completion_rx,
        RecurringScheduleConfig {
            schedule_id,
            session_prefix,
            recipient,
            prompt,
            interval_secs,
            max_runs,
            wait_for_completion_secs,
        },
    )
    .await?;

    println!(
        "Scheduler finished: submitted={} completed={} succeeded={} failed={} timed_out={} pending={}",
        outcome.submitted,
        outcome.completed,
        outcome.succeeded,
        outcome.failed,
        outcome.timed_out,
        outcome.submitted.saturating_sub(outcome.completed),
    );
    Ok(())
}
