//! Top-level integration harness for `agent::memory_recall_metrics`.

mod agent {
    pub(crate) mod memory_recall_state {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum SessionMemoryRecallDecision {
            Injected,
            Skipped,
        }
    }

    pub(crate) use memory_recall_state::SessionMemoryRecallDecision;

    pub(crate) struct Agent {
        pub(crate) memory_recall_metrics:
            tokio::sync::RwLock<memory_recall_metrics_impl::MemoryRecallMetricsState>,
    }

    mod memory_recall_metrics_impl {
        include!("../src/agent/memory_recall_metrics.rs");

        mod tests {
            include!("agent/memory_recall_metrics.rs");
        }

        fn lint_symbol_probe() {
            let _ = ratio_as_f32 as fn(u64, u64) -> f32;
            let _ = now_unix_ms as fn() -> u64;
            let _ = std::mem::size_of::<MemoryRecallMetricsState>();
            let _ = std::mem::size_of::<MemoryRecallMetricsSnapshot>();
            let _ = std::mem::size_of::<MemoryRecallLatencyBucketsSnapshot>();
        }

        const _: fn() = lint_symbol_probe;
    }

    fn lint_symbol_probe() {
        let agent = Agent {
            memory_recall_metrics: tokio::sync::RwLock::new(
                memory_recall_metrics_impl::MemoryRecallMetricsState::default(),
            ),
        };
        let _ = &agent.memory_recall_metrics;

        let _ = Agent::record_memory_recall_plan_metrics;
        let _ = Agent::record_memory_recall_result_metrics;
        let _ = Agent::record_memory_embedding_success_metric;
        let _ = Agent::record_memory_embedding_timeout_metric;
        let _ = Agent::record_memory_embedding_cooldown_reject_metric;
        let _ = Agent::record_memory_embedding_unavailable_metric;
        let _ = Agent::inspect_memory_recall_metrics;
    }

    const _: fn() = lint_symbol_probe;
}
