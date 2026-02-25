//! Core scheduler loop implementation.

use crate::contracts::{FlowInstruction, NodeStatus};
use crate::engine::QianjiEngine;
use crate::error::QianjiError;
use crate::scheduler::checkpoint::QianjiStateSnapshot;
use crate::scheduler::state::{ExecutionState, merge_output_data, spawn_node_execution_task};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Drives the parallel execution of the Qianji Box mechanisms.
pub struct QianjiScheduler {
    /// Thread-safe access to the underlying graph.
    engine: Arc<RwLock<QianjiEngine>>,
    /// Maximum total execution steps to prevent runaway loops.
    max_total_steps: u32,
}

impl QianjiScheduler {
    /// Creates a new scheduler for the given engine.
    #[must_use]
    pub fn new(engine: QianjiEngine) -> Self {
        Self {
            engine: Arc::new(RwLock::new(engine)),
            max_total_steps: 1000, // Generous threshold for complex arrays
        }
    }

    async fn reset_retry_nodes(&self, node_ids: &[String]) {
        let mut engine = self.engine.write().await;
        let mut to_reset = HashSet::new();

        let initial_indices: Vec<_> = engine
            .graph
            .node_indices()
            .filter(|&idx| node_ids.contains(&engine.graph[idx].id))
            .collect();

        for start_idx in initial_indices {
            let mut bfs = petgraph::visit::Bfs::new(&engine.graph, start_idx);
            while let Some(visited) = bfs.next(&engine.graph) {
                to_reset.insert(visited);
            }
        }

        for idx in to_reset {
            engine.graph[idx].status = NodeStatus::Idle;
        }
    }

    /// Execute the graph following Synaptic-Flow: entropy-aware dependency resolution.
    pub async fn run(
        &self,
        initial_context: serde_json::Value,
    ) -> Result<serde_json::Value, QianjiError> {
        self.run_with_checkpoint(initial_context, None, None).await
    }

    /// Execute the graph with state persistence and resumption via Valkey (Redis).
    pub async fn run_with_checkpoint(
        &self,
        initial_context: serde_json::Value,
        session_id: Option<String>,
        redis_url: Option<String>,
    ) -> Result<serde_json::Value, QianjiError> {
        let mut context = initial_context.clone();
        let mut active_branches: HashSet<String> = HashSet::new();
        let mut total_steps = 0;

        // Try to load state if session_id and redis_url are provided
        if let (Some(sid), Some(url)) = (session_id.as_deref(), redis_url.as_deref()) {
            match QianjiStateSnapshot::load(sid, url).await {
                Ok(Some(snapshot)) => {
                    // Start with snapshot context, then merge newly provided initial_context overrides
                    let mut merged_context = snapshot.context;
                    merge_output_data(&mut merged_context, &initial_context);
                    context = merged_context;

                    active_branches = snapshot.active_branches;
                    total_steps = snapshot.total_steps;

                    // Restore graph node statuses
                    let mut engine = self.engine.write().await;
                    let indices: Vec<_> = engine.graph.node_indices().collect();
                    for node_idx in indices {
                        let id = engine.graph[node_idx].id.clone();
                        if let Some(status) = snapshot.node_statuses.get(&id) {
                            engine.graph[node_idx].status = status.clone();
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    log::warn!(
                        "Failed to load checkpoint for session {}: {}. Starting fresh.",
                        sid,
                        e
                    );
                }
            }
        }

        // Build initial execution state
        let mut exec_state = {
            let engine = self.engine.read().await;
            ExecutionState::build(&engine, &active_branches)
        };

        let mut executing_tasks = futures::stream::FuturesUnordered::new();

        loop {
            // Check step budget
            if total_steps > self.max_total_steps {
                return Err(QianjiError::DriftError(
                    "Maximum execution steps exceeded (Potential infinite loop)".to_string(),
                ));
            }

            // Launch ready nodes
            while let Some(node_idx) = exec_state.ready_queue.pop_front() {
                executing_tasks.push(spawn_node_execution_task(
                    self.engine.clone(),
                    node_idx,
                    context.clone(),
                ));
            }

            // If nothing is executing and nothing is ready, we are done (or deadlocked)
            if executing_tasks.is_empty() {
                break;
            }

            // Wait for at least one node to finish
            use futures::StreamExt;
            if let Some(join_result) = executing_tasks.next().await {
                total_steps += 1;

                match join_result {
                    Ok((_node_idx, Ok(output))) => {
                        // Apply Instruction
                        let mut suspend_reason = None;
                        match output.instruction {
                            FlowInstruction::SelectBranch(branch) => {
                                active_branches.insert(branch);
                            }
                            FlowInstruction::RetryNodes(node_ids) => {
                                self.reset_retry_nodes(&node_ids).await;
                            }
                            FlowInstruction::Suspend(reason) => {
                                suspend_reason = Some(reason);
                            }
                            FlowInstruction::Abort(reason) => {
                                return Err(QianjiError::ExecutionError(reason));
                            }
                            FlowInstruction::Continue => {}
                        }
                        merge_output_data(&mut context, &output.data);

                        // State changed (new branch, reset, or node completed), rebuild tracking state
                        let engine = self.engine.read().await;
                        exec_state = ExecutionState::build(&engine, &active_branches);

                        // Save checkpoint
                        if let (Some(sid), Some(url)) =
                            (session_id.as_deref(), redis_url.as_deref())
                        {
                            let mut node_statuses = HashMap::new();
                            for node_idx in engine.graph.node_indices() {
                                let node = &engine.graph[node_idx];
                                node_statuses.insert(node.id.clone(), node.status.clone());
                            }

                            let snapshot = QianjiStateSnapshot {
                                session_id: sid.to_string(),
                                total_steps,
                                active_branches: active_branches.clone(),
                                context: context.clone(),
                                node_statuses,
                            };

                            if let Err(e) = snapshot.save(url).await {
                                log::warn!("Failed to save checkpoint for session {}: {}", sid, e);
                            }
                        }

                        // If suspended, yield control by returning the context.
                        if let Some(reason) = suspend_reason {
                            log::info!("Workflow suspended: {}", reason);
                            return Ok(context);
                        }
                    }
                    Ok((_node_idx, Err(error))) => {
                        return Err(QianjiError::ExecutionError(error));
                    }
                    Err(join_err) => {
                        return Err(QianjiError::ExecutionError(format!(
                            "Task panic: {}",
                            join_err
                        )));
                    }
                }
            }
        }

        // Clean up checkpoint on successful completion
        if let (Some(sid), Some(url)) = (session_id.as_deref(), redis_url.as_deref()) {
            let _ = QianjiStateSnapshot::delete(sid, url).await;
        }

        Ok(context)
    }
}
