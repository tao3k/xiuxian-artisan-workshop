//! Top-level integration harness for `agent::memory_recall`.

#[path = "../src/agent/memory_recall/mod.rs"]
mod memory_recall_impl;

mod session {
    pub(crate) use omni_agent::ChatMessage;
}

mod agent {
    pub(crate) mod memory_recall {
        pub(crate) use crate::memory_recall_impl::{
            MemoryRecallInput, build_memory_context_message, filter_recalled_episodes,
            filter_recalled_episodes_at, plan_memory_recall,
        };

        fn lint_symbol_probe() {
            let _ = crate::memory_recall_impl::MEMORY_RECALL_MESSAGE_NAME;

            let messages = vec![crate::session::ChatMessage {
                role: "user".to_string(),
                content: Some("probe".to_string()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            }];
            let _ = crate::memory_recall_impl::estimate_messages_tokens(&messages);

            let plan = crate::memory_recall_impl::MemoryRecallPlan {
                k1: 1,
                k2: 1,
                lambda: 0.5,
                min_score: 0.1,
                max_context_chars: 128,
                budget_pressure: 0.2,
                window_pressure: 0.3,
                effective_budget_tokens: Some(42),
            };
            let _ = plan.effective_budget_tokens;
        }

        const _: fn() = lint_symbol_probe;

        mod tests {
            include!("agent/memory_recall.rs");
        }
    }
}
