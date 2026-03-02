//! Runtime signal payloads emitted by native zhenfa tools.

/// Asynchronous fire-and-forget signal emitted during native tool execution.
#[derive(Clone, Debug, PartialEq)]
pub enum ZhenfaSignal {
    /// Reinforcement-learning reward signal.
    Reward {
        /// Episode identifier to update.
        episode_id: String,
        /// Reward value, typically in `[0.0, 1.0]`.
        value: f32,
        /// Signal source identifier for audit correlation.
        source: String,
    },
    /// Execution trace signal for observability and diagnostics.
    Trace {
        /// Node or component identifier.
        node_id: String,
        /// Trace event payload.
        event: String,
    },
}
