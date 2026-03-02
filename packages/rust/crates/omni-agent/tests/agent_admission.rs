//! Top-level integration harness for `agent::admission`.

mod llm {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct LlmInFlightSnapshot {
        pub(crate) max_in_flight: usize,
        pub(crate) available_permits: usize,
        pub(crate) in_flight: usize,
        pub(crate) saturation_pct: u8,
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub(crate) struct LlmClient {
        pub(crate) snapshot: Option<LlmInFlightSnapshot>,
    }

    impl LlmClient {
        pub(crate) fn in_flight_snapshot(&self) -> Option<LlmInFlightSnapshot> {
            self.snapshot
        }
    }
}

mod embedding {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct EmbeddingInFlightSnapshot {
        pub(crate) max_in_flight: usize,
        pub(crate) available_permits: usize,
        pub(crate) in_flight: usize,
        pub(crate) saturation_pct: u8,
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub(crate) struct EmbeddingClient {
        pub(crate) snapshot: Option<EmbeddingInFlightSnapshot>,
    }

    impl EmbeddingClient {
        pub(crate) fn in_flight_snapshot(&self) -> Option<EmbeddingInFlightSnapshot> {
            self.snapshot
        }
    }
}

mod agent {
    pub(crate) struct Agent {
        pub(crate) llm: crate::llm::LlmClient,
        pub(crate) embedding_client: Option<crate::embedding::EmbeddingClient>,
        pub(crate) downstream_admission_policy: admission_impl::DownstreamAdmissionPolicy,
        pub(crate) downstream_admission_metrics: admission_impl::DownstreamAdmissionMetrics,
    }

    mod admission_impl {
        include!("../src/agent/admission.rs");

        mod tests {
            include!("agent/admission.rs");
        }

        fn lint_symbol_probe() {
            let _ = DownstreamAdmissionRejectReason::as_str
                as fn(DownstreamAdmissionRejectReason) -> &'static str;
            let _ = DownstreamAdmissionRejectReason::user_message
                as fn(DownstreamAdmissionRejectReason) -> &'static str;
            let _ = DownstreamAdmissionPolicy::from_env as fn() -> DownstreamAdmissionPolicy;
            let _ = DownstreamAdmissionPolicy::evaluate
                as fn(
                    DownstreamAdmissionPolicy,
                    DownstreamRuntimeSnapshot,
                ) -> DownstreamAdmissionDecision;
            let _ = DownstreamAdmissionPolicy::runtime_snapshot
                as fn(
                    DownstreamAdmissionPolicy,
                    DownstreamAdmissionMetricsSnapshot,
                ) -> DownstreamAdmissionRuntimeSnapshot;
        }

        const _: fn() = lint_symbol_probe;
    }

    fn lint_symbol_probe() {
        let llm_snapshot = crate::llm::LlmInFlightSnapshot {
            max_in_flight: 8,
            available_permits: 4,
            in_flight: 4,
            saturation_pct: 50,
        };
        let embedding_snapshot = crate::embedding::EmbeddingInFlightSnapshot {
            max_in_flight: 8,
            available_permits: 6,
            in_flight: 2,
            saturation_pct: 25,
        };

        let agent = Agent {
            llm: crate::llm::LlmClient {
                snapshot: Some(llm_snapshot),
            },
            embedding_client: Some(crate::embedding::EmbeddingClient {
                snapshot: Some(embedding_snapshot),
            }),
            downstream_admission_policy: admission_impl::DownstreamAdmissionPolicy::from_env(),
            downstream_admission_metrics: admission_impl::DownstreamAdmissionMetrics::default(),
        };
        let _ = &agent.llm;
        let _ = &agent.embedding_client;
        let _ = &agent.downstream_admission_policy;
        let _ = &agent.downstream_admission_metrics;

        let _ = Agent::evaluate_downstream_admission;
        let _ = Agent::downstream_admission_runtime_snapshot;
    }

    const _: fn() = lint_symbol_probe;
}
