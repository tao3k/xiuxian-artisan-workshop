use crate::helpers::{build_agentic_monitor_phases, build_agentic_monitor_summary, emit};
use crate::types::Cli;
use anyhow::{Context, Result};
use serde_json::json;
use xiuxian_wendao::{LinkGraphAgenticExecutionConfig, LinkGraphIndex};

pub(super) struct AgenticRunOptions {
    pub(super) query: Option<String>,
    pub(super) max_workers: Option<usize>,
    pub(super) max_candidates: Option<usize>,
    pub(super) max_pairs_per_worker: Option<usize>,
    pub(super) time_budget_ms: Option<f64>,
    pub(super) worker_time_budget_ms: Option<f64>,
    pub(super) persist: Option<bool>,
    pub(super) persist_retry_attempts: Option<usize>,
    pub(super) idempotency_scan_limit: Option<usize>,
    pub(super) relation: Option<String>,
    pub(super) agent_id: Option<String>,
    pub(super) evidence_prefix: Option<String>,
    pub(super) created_at_unix: Option<f64>,
    pub(super) verbose: bool,
}

pub(super) fn handle_plan(
    cli: &Cli,
    index: &LinkGraphIndex,
    query: Option<&str>,
    max_workers: Option<usize>,
    max_candidates: Option<usize>,
    max_pairs_per_worker: Option<usize>,
    time_budget_ms: Option<f64>,
) -> Result<()> {
    let mut config = LinkGraphIndex::resolve_agentic_expansion_config();
    if let Some(value) = max_workers {
        config.max_workers = value.max(1);
    }
    if let Some(value) = max_candidates {
        config.max_candidates = value.max(1);
    }
    if let Some(value) = max_pairs_per_worker {
        config.max_pairs_per_worker = value.max(1);
    }
    if let Some(value) = time_budget_ms {
        config.time_budget_ms = if value.is_finite() && value > 0.0 {
            value
        } else {
            config.time_budget_ms
        };
    }
    let plan = index.agentic_expansion_plan_with_config(query, config);
    emit(&plan, cli.output)
}

pub(super) fn handle_run(cli: &Cli, index: &LinkGraphIndex, run: &AgenticRunOptions) -> Result<()> {
    let mut config: LinkGraphAgenticExecutionConfig =
        LinkGraphIndex::resolve_agentic_execution_config();
    if let Some(value) = run.max_workers {
        config.expansion.max_workers = value.max(1);
    }
    if let Some(value) = run.max_candidates {
        config.expansion.max_candidates = value.max(1);
    }
    if let Some(value) = run.max_pairs_per_worker {
        config.expansion.max_pairs_per_worker = value.max(1);
    }
    if let Some(value) = run.time_budget_ms {
        config.expansion.time_budget_ms = if value.is_finite() && value > 0.0 {
            value
        } else {
            config.expansion.time_budget_ms
        };
    }
    if let Some(value) = run.worker_time_budget_ms {
        config.worker_time_budget_ms = if value.is_finite() && value > 0.0 {
            value
        } else {
            config.worker_time_budget_ms
        };
    }
    if let Some(value) = run.persist {
        config.persist_suggestions = value;
    }
    if let Some(value) = run.persist_retry_attempts {
        config.persist_retry_attempts = value.max(1);
    }
    if let Some(value) = run.idempotency_scan_limit {
        config.idempotency_scan_limit = value.max(1);
    }
    if let Some(value) = &run.relation {
        config.relation.clone_from(value);
    }
    if let Some(value) = &run.agent_id {
        config.agent_id.clone_from(value);
    }
    if let Some(value) = &run.evidence_prefix {
        config.evidence_prefix.clone_from(value);
    }
    config.created_at_unix = run.created_at_unix;
    let result = index.agentic_expansion_execute_with_config(run.query.as_deref(), config);

    if run.verbose {
        let phases = build_agentic_monitor_phases(&result);
        let monitor = json!({
            "overview": {
                "elapsed_ms": result.elapsed_ms,
                "worker_runs": result.worker_runs.len(),
                "prepared_proposals": result.prepared_proposals,
                "persisted_proposals": result.persisted_proposals,
                "skipped_duplicates": result.skipped_duplicates,
                "failed_proposals": result.failed_proposals,
                "persist_attempts": result.persist_attempts,
                "timed_out": result.timed_out,
            },
            "bottlenecks": build_agentic_monitor_summary(&phases),
        });
        let mut payload = serde_json::to_value(&result)
            .context("failed to serialize agentic execution result")?;
        if let Some(map) = payload.as_object_mut() {
            map.insert("phases".to_string(), json!(phases));
            map.insert("monitor".to_string(), monitor);
        }
        emit(&payload, cli.output)
    } else {
        emit(&result, cli.output)
    }
}
