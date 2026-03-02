//! Top-level integration harness for `agent::zhenfa`.

mod config {
    /// Minimal xiuxian config shim required by zhenfa bridge/hook tests.
    #[derive(Debug, Clone, Default)]
    pub(crate) struct XiuxianConfig {
        pub(crate) wendao: WendaoConfig,
        pub(crate) zhenfa: ZhenfaConfig,
    }

    #[derive(Debug, Clone, Default)]
    pub(crate) struct WendaoConfig {
        pub(crate) zhixing: ZhixingConfig,
    }

    #[derive(Debug, Clone, Default)]
    pub(crate) struct ZhixingConfig {
        pub(crate) notebook_path: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    pub(crate) struct ZhenfaConfig {
        pub(crate) base_url: Option<String>,
        pub(crate) enabled_tools: Option<Vec<String>>,
        pub(crate) valkey: ZhenfaValkeyConfig,
    }

    #[derive(Debug, Clone, Default)]
    pub(crate) struct ZhenfaValkeyConfig {
        pub(crate) url: Option<String>,
        pub(crate) key_prefix: Option<String>,
        pub(crate) cache_ttl_seconds: Option<u64>,
        pub(crate) lock_ttl_seconds: Option<u64>,
        pub(crate) audit_stream: Option<String>,
    }
}

mod agent {
    pub(crate) mod memory_state {
        use omni_memory::{MemoryStateStore, ValkeyMemoryStateStore};

        /// Minimal memory-state backend shim used by zhenfa reward sink path.
        pub(crate) enum MemoryStateBackend {
            Valkey(Box<ValkeyMemoryStateStore>),
        }

        impl MemoryStateBackend {
            pub(crate) fn backend_name(&self) -> &'static str {
                match self {
                    Self::Valkey(store) => store.backend_name(),
                }
            }

            pub(crate) fn update_q_atomic(
                &self,
                episode_id: &str,
                q_value: f32,
            ) -> Result<(), anyhow::Error> {
                match self {
                    Self::Valkey(store) => store.update_q_atomic(episode_id, q_value),
                }
            }
        }
    }

    pub(crate) mod zhenfa {
        include!("../src/agent/zhenfa/mod.rs");

        fn lint_symbol_probe() {
            let _ = std::mem::size_of::<ZhenfaRuntimeDeps>();
            let _ = std::mem::size_of::<ZhenfaToolBridge>();
        }

        const _: fn() = lint_symbol_probe;

        mod tests {
            include!("unit/agent/zhenfa_tests.rs");
        }

        mod valkey_hooks_tests {
            include!("unit/agent/zhenfa/valkey_hooks_tests.rs");
        }
    }
}
