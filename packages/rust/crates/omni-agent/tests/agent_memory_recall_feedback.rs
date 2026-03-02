//! Top-level integration harness for `agent::memory_recall_feedback`.

mod agent {
    pub(crate) mod memory_recall {
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct MemoryRecallPlan {
            pub(crate) k1: usize,
            pub(crate) k2: usize,
            pub(crate) lambda: f32,
            pub(crate) min_score: f32,
            pub(crate) max_context_chars: usize,
            pub(crate) budget_pressure: f32,
            pub(crate) window_pressure: f32,
            pub(crate) effective_budget_tokens: Option<usize>,
        }
    }

    mod memory_recall_feedback_impl {
        include!("../src/agent/memory_recall_feedback.rs");

        mod tests {
            include!("agent/memory_recall_feedback.rs");
        }

        fn lint_symbol_probe() {
            let plan = crate::agent::memory_recall::MemoryRecallPlan {
                k1: 1,
                k2: 1,
                lambda: 0.5,
                min_score: 0.1,
                max_context_chars: 256,
                budget_pressure: 0.2,
                window_pressure: 0.3,
                effective_budget_tokens: Some(1000),
            };
            let _ = (
                plan.budget_pressure,
                plan.window_pressure,
                plan.effective_budget_tokens,
            );
            let _ = RECALL_FEEDBACK_SOURCE_COMMAND;
            let _ = update_feedback_bias as fn(f32, RecallOutcome) -> f32;
            let _ = apply_feedback_to_plan
                as fn(
                    crate::agent::memory_recall::MemoryRecallPlan,
                    f32,
                ) -> crate::agent::memory_recall::MemoryRecallPlan;
            let _ = parse_explicit_user_feedback as fn(&str) -> Option<RecallOutcome>;
            let _ = classify_assistant_outcome as fn(&str) -> RecallOutcome;
            let _ = resolve_feedback_outcome
                as fn(&str, Option<&ToolExecutionSummary>, &str) -> (RecallOutcome, &'static str);
            let _ = std::mem::size_of::<ToolExecutionSummary>();
        }

        const _: fn() = lint_symbol_probe;
    }
}
