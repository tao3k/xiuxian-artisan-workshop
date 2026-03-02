//! Top-level integration harness for `agent::memory_stream_consumer`.

mod config {
    pub(crate) use omni_agent::MemoryConfig;
}

mod observability {
    /// Minimal observability event shim for memory stream-consumer compilation.
    #[derive(Debug, Clone, Copy)]
    pub(crate) enum SessionEvent {
        MemoryStreamConsumerDisabled,
        MemoryStreamConsumerStarted,
        MemoryStreamConsumerReadFailed,
        MemoryStreamConsumerGroupReady,
        MemoryStreamConsumerEventProcessed,
    }

    impl SessionEvent {
        pub(crate) const fn as_str(self) -> &'static str {
            match self {
                Self::MemoryStreamConsumerDisabled => "memory.stream_consumer.disabled",
                Self::MemoryStreamConsumerStarted => "memory.stream_consumer.started",
                Self::MemoryStreamConsumerReadFailed => "memory.stream_consumer.read_failed",
                Self::MemoryStreamConsumerGroupReady => "memory.stream_consumer.group_ready",
                Self::MemoryStreamConsumerEventProcessed => {
                    "memory.stream_consumer.event_processed"
                }
            }
        }
    }
}

mod session {
    /// Minimal redis runtime snapshot shim used by stream-consumer bootstrap.
    #[derive(Debug, Clone)]
    pub(crate) struct RedisSessionRuntimeSnapshot {
        pub(crate) url: String,
        pub(crate) key_prefix: String,
        pub(crate) ttl_secs: Option<u64>,
    }
}

mod agent {
    pub(crate) mod logging {
        include!("../src/agent/logging/repeated_failure.rs");
    }

    pub(crate) mod memory_stream_consumer {
        include!("../src/agent/memory_stream_consumer/mod.rs");

        fn lint_symbol_probe() {
            let _ = spawn_memory_stream_consumer;
        }

        const _: fn() = lint_symbol_probe;

        mod tests {
            include!("agent/memory_stream_consumer.rs");
        }
    }
}
