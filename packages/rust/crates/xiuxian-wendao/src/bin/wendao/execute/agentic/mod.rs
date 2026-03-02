//! Agentic command execution.

mod plan_run;
mod suggested_links;

use crate::types::{AgenticCommand, Cli, Command};
use anyhow::{Context, Result};
use xiuxian_wendao::{LinkGraphIndex, LinkGraphSuggestedLinkRequest};

pub(super) fn handle(cli: &Cli, index: Option<&LinkGraphIndex>) -> Result<()> {
    let Command::Agentic { command } = &cli.command else {
        unreachable!("agentic handler must be called with agentic command");
    };
    match command {
        AgenticCommand::Log {
            source_id,
            target_id,
            relation,
            confidence,
            evidence,
            agent_id,
            created_at_unix,
        } => suggested_links::handle_log(
            cli,
            build_log_request(
                source_id,
                target_id,
                relation,
                *confidence,
                evidence,
                agent_id,
                *created_at_unix,
            ),
        ),
        AgenticCommand::Recent {
            limit,
            latest,
            state,
        } => suggested_links::handle_recent(cli, (*limit).max(1), *latest, state.map(Into::into)),
        AgenticCommand::Decide {
            suggestion_id,
            target_state,
            decided_by,
            reason,
            decided_at_unix,
        } => suggested_links::handle_decide(
            cli,
            suggestion_id,
            (*target_state).into(),
            decided_by,
            reason,
            *decided_at_unix,
        ),
        AgenticCommand::Decisions { limit } => {
            suggested_links::handle_decisions(cli, (*limit).max(1))
        }
        AgenticCommand::Plan { .. } | AgenticCommand::Run { .. } => {
            handle_plan_or_run(cli, index, command)
        }
    }
}

fn build_log_request(
    source_id: &str,
    target_id: &str,
    relation: &str,
    confidence: f64,
    evidence: &str,
    agent_id: &str,
    created_at_unix: Option<f64>,
) -> LinkGraphSuggestedLinkRequest {
    LinkGraphSuggestedLinkRequest {
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        relation: relation.to_string(),
        confidence,
        evidence: evidence.to_string(),
        agent_id: agent_id.to_string(),
        created_at_unix,
    }
}

fn handle_plan_or_run(
    cli: &Cli,
    index: Option<&LinkGraphIndex>,
    command: &AgenticCommand,
) -> Result<()> {
    match command {
        AgenticCommand::Plan {
            query,
            max_workers,
            max_candidates,
            max_pairs_per_worker,
            time_budget_ms,
        } => {
            let index = index.context("link_graph index is required for agentic plan command")?;
            plan_run::handle_plan(
                cli,
                index,
                query.as_deref(),
                *max_workers,
                *max_candidates,
                *max_pairs_per_worker,
                *time_budget_ms,
            )
        }
        AgenticCommand::Run {
            query,
            max_workers,
            max_candidates,
            max_pairs_per_worker,
            time_budget_ms,
            worker_time_budget_ms,
            persist,
            persist_retry_attempts,
            idempotency_scan_limit,
            relation,
            agent_id,
            evidence_prefix,
            created_at_unix,
            verbose,
        } => {
            let index = index.context("link_graph index is required for agentic run command")?;
            plan_run::handle_run(
                cli,
                index,
                &plan_run::AgenticRunOptions {
                    query: query.clone(),
                    max_workers: *max_workers,
                    max_candidates: *max_candidates,
                    max_pairs_per_worker: *max_pairs_per_worker,
                    time_budget_ms: *time_budget_ms,
                    worker_time_budget_ms: *worker_time_budget_ms,
                    persist: *persist,
                    persist_retry_attempts: *persist_retry_attempts,
                    idempotency_scan_limit: *idempotency_scan_limit,
                    relation: relation.clone(),
                    agent_id: agent_id.clone(),
                    evidence_prefix: evidence_prefix.clone(),
                    created_at_unix: *created_at_unix,
                    verbose: *verbose,
                },
            )
        }
        _ => unreachable!("plan/run helper must be called with plan or run command"),
    }
}
