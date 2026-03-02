//! Top-level integration harness for `channels::managed_runtime` unit lanes.

mod agent {
    /// Minimal agent shim for managed-runtime turn helper compilation.
    pub(crate) struct Agent;

    impl Agent {
        pub(crate) async fn run_turn(
            &self,
            _session_id: &str,
            _content: &str,
        ) -> Result<String, anyhow::Error> {
            Ok(String::new())
        }
    }
}

mod config {
    use std::path::PathBuf;

    pub(crate) use omni_agent::RuntimeSettings;

    pub(crate) fn load_runtime_settings() -> RuntimeSettings {
        omni_agent::load_runtime_settings()
    }

    pub(crate) fn runtime_settings_paths() -> (PathBuf, PathBuf) {
        (
            PathBuf::from("packages/conf/settings.yaml"),
            PathBuf::from(".config/xiuxian-artisan-workshop/settings.yaml"),
        )
    }
}

mod channels {
    pub(crate) mod managed_runtime {
        pub(crate) mod session_partition_persistence {
            include!("../src/channels/managed_runtime/session_partition_persistence.rs");
        }

        pub(crate) mod turn {
            include!("../src/channels/managed_runtime/turn.rs");
        }

        mod tests {
            include!("unit/channels/managed_runtime/tests/mod.rs");
        }
    }
}
