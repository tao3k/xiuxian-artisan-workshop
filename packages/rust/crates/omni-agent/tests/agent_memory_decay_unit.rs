//! Top-level integration harness for `agent::memory::decay`.

mod agent {
    pub(crate) mod memory {
        include!("../src/agent/memory/decay.rs");

        mod tests {
            include!("agent/memory/decay.rs");
        }
    }
}
