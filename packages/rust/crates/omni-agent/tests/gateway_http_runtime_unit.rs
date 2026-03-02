//! Top-level integration harness for `gateway::http::runtime`.

mod config {
    pub(crate) use omni_agent::RuntimeSettings;

    pub(crate) fn load_runtime_settings() -> RuntimeSettings {
        omni_agent::load_runtime_settings()
    }
}

mod embedding {
    pub(crate) use omni_agent::EmbeddingClient;
}

mod gateway {
    pub(crate) mod http {
        pub(crate) mod types {
            use std::sync::Arc;

            use crate::embedding::EmbeddingClient;

            #[derive(Clone)]
            pub(crate) struct GatewayEmbeddingRuntime {
                pub(crate) client: Arc<EmbeddingClient>,
                pub(crate) default_model: Option<String>,
            }
        }

        pub(crate) mod runtime {
            include!("../src/gateway/http/runtime.rs");

            fn lint_symbol_probe() {
                let _ = build_embedding_runtime as fn() -> super::types::GatewayEmbeddingRuntime;
                let _ = build_embedding_runtime_for_gateway;
                let _ = resolve_runtime_embed_base_url
                    as fn(&crate::config::RuntimeSettings, Option<&str>, Option<&str>) -> String;
                let runtime = super::types::GatewayEmbeddingRuntime {
                    client: std::sync::Arc::new(crate::embedding::EmbeddingClient::new(
                        "http://127.0.0.1:1",
                        1,
                    )),
                    default_model: Some("probe".to_string()),
                };
                let _ = (&runtime.client, &runtime.default_model);
            }

            const _: fn() = lint_symbol_probe;

            mod tests {
                include!("gateway/http/runtime.rs");
            }
        }
    }
}
