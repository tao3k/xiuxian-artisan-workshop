//! Top-level integration harness for `agent::memory::recall_credit`.

mod agent {
    pub(crate) mod memory_recall_feedback {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum RecallOutcome {
            Success,
            Failure,
        }
    }

    pub(crate) mod memory {
        mod recall_credit_impl {
            include!("../src/agent/memory/recall_credit.rs");
        }

        use recall_credit_impl::*;

        mod tests {
            include!("agent/memory/recall_credit.rs");
        }
    }
}
