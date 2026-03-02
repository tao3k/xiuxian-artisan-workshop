//! Embedding module integration harness.

mod config {
    pub(crate) use omni_agent::RuntimeSettings;

    pub(crate) fn load_runtime_settings() -> RuntimeSettings {
        omni_agent::load_runtime_settings()
    }

    fn lint_symbol_probe() {
        let _ = load_runtime_settings as fn() -> RuntimeSettings;
    }

    const _: fn() = lint_symbol_probe;
}

mod embedding {
    #[path = "../../src/embedding/types.rs"]
    pub mod types;

    mod backend_impl {
        include!("../src/embedding/backend.rs");

        #[test]
        fn parse_backend_mode_supports_openai_and_mistral_sdk_aliases() {
            assert_eq!(
                parse_backend_mode(Some("openai_http")),
                EmbeddingBackendMode::OpenAiHttp
            );
            assert_eq!(
                parse_backend_mode(Some("mistral_sdk")),
                EmbeddingBackendMode::MistralSdk
            );
        }

        #[test]
        fn parse_backend_mode_retains_legacy_http_alias() {
            assert_eq!(parse_backend_mode(Some("http")), EmbeddingBackendMode::Http);
            assert_eq!(
                parse_backend_mode(Some("client")),
                EmbeddingBackendMode::Http
            );
        }

        fn lint_symbol_probe() {
            let _ = (
                DEFAULT_EMBED_TIMEOUT_SECS,
                MIN_EMBED_TIMEOUT_SECS,
                MAX_EMBED_TIMEOUT_SECS,
                MAX_EMBED_MAX_IN_FLIGHT,
                MAX_MISTRAL_SDK_EMBED_MAX_NUM_SEQS,
            );

            let settings = EmbeddingBackendSettings {
                mode: EmbeddingBackendMode::Http,
                source: "probe",
                timeout_secs: DEFAULT_EMBED_TIMEOUT_SECS,
                max_in_flight: Some(1),
                default_model: Some("text-embedding-3-small".to_string()),
                mistral_sdk_hf_cache_path: None,
                mistral_sdk_hf_revision: None,
                mistral_sdk_max_num_seqs: Some(1),
            };
            let _ = (
                settings.mode,
                settings.source,
                settings.timeout_secs,
                settings.max_in_flight,
                settings.default_model,
                settings.mistral_sdk_hf_cache_path,
                settings.mistral_sdk_hf_revision,
                settings.mistral_sdk_max_num_seqs,
            );

            let _ = resolve_backend_settings as fn(u64, Option<&str>) -> EmbeddingBackendSettings;
            let _ = parse_backend_mode as fn(Option<&str>) -> EmbeddingBackendMode;
            let _ = default_backend_mode as fn() -> EmbeddingBackendMode;
        }

        const _: fn() = lint_symbol_probe;
    }

    #[cfg(feature = "agent-provider-litellm")]
    mod transport_litellm_impl {
        include!("../src/embedding/transport_litellm.rs");

        #[test]
        fn normalize_openai_base_url_appends_v1_for_plain_host() {
            assert_eq!(
                normalize_openai_compatible_base_url("http://127.0.0.1:11434"),
                "http://127.0.0.1:11434/v1"
            );
        }

        #[test]
        fn normalize_litellm_target_ollama_uses_openai_compat_with_placeholder_key() {
            let (model, base, key, compat) = normalize_litellm_embedding_target(
                "ollama/qwen3-embedding:0.6b",
                "http://127.0.0.1:11434",
                None,
            );
            assert!(compat);
            assert_eq!(model, "openai/qwen3-embedding:0.6b");
            assert_eq!(base, "http://127.0.0.1:11434/v1");
            assert_eq!(key.as_deref(), Some(OLLAMA_PLACEHOLDER_API_KEY));
        }

        #[test]
        fn normalize_litellm_target_non_ollama_is_passthrough() {
            let (model, base, key, compat) = normalize_litellm_embedding_target(
                "minimax/text-embedding",
                "https://api.minimax.io/v1",
                Some("k"),
            );
            assert!(!compat);
            assert_eq!(model, "minimax/text-embedding");
            assert_eq!(base, "https://api.minimax.io/v1");
            assert_eq!(key.as_deref(), Some("k"));
        }

        fn lint_symbol_probe() {
            let _ = embed_litellm;
        }

        const _: fn() = lint_symbol_probe;
    }

    mod transport_openai_impl {
        include!("../src/embedding/transport_openai.rs");

        #[test]
        fn normalize_openai_embeddings_url_appends_v1_for_plain_host() {
            assert_eq!(
                normalize_openai_embeddings_url("http://127.0.0.1:11434"),
                Some("http://127.0.0.1:11434/v1/embeddings".to_string())
            );
        }

        #[test]
        fn normalize_openai_embeddings_url_respects_existing_v1_suffix() {
            assert_eq!(
                normalize_openai_embeddings_url("http://127.0.0.1:18081/v1"),
                Some("http://127.0.0.1:18081/v1/embeddings".to_string())
            );
        }

        fn lint_symbol_probe() {
            let _ = embed_openai_http;
        }

        const _: fn() = lint_symbol_probe;
    }

    mod transport_http_impl {
        use anyhow::{Context, Result};
        use axum::{Json, Router, extract::State, routing::post};
        use serde_json::json;

        include!("../src/embedding/transport_http.rs");

        #[derive(Clone)]
        struct EmbedBatchMockState {
            vectors: Vec<Vec<f32>>,
        }

        async fn handle_embed_batch(
            State(state): State<EmbedBatchMockState>,
        ) -> Json<serde_json::Value> {
            Json(json!({ "vectors": state.vectors }))
        }

        async fn reserve_local_port() -> Result<Option<u16>> {
            let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
                Ok(listener) => listener,
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    return Ok(None);
                }
                Err(error) => {
                    return Err(error).context("failed to reserve local port for embed-http test");
                }
            };

            let port = listener
                .local_addr()
                .context("reserved listener should expose local addr")?
                .port();
            drop(listener);
            Ok(Some(port))
        }

        #[tokio::test]
        async fn embed_http_retries_connection_refused_until_server_is_ready() -> Result<()> {
            let Some(port) = reserve_local_port().await? else {
                return Ok(());
            };

            let state = EmbedBatchMockState {
                vectors: vec![vec![0.1_f32, 0.2_f32, 0.3_f32]],
            };
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(350)).await;
                let Ok(listener) = tokio::net::TcpListener::bind(("127.0.0.1", port)).await else {
                    return;
                };
                let app = Router::new()
                    .route("/embed/batch", post(handle_embed_batch))
                    .with_state(state);
                let _ = axum::serve(listener, app).await;
            });

            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(1))
                .build()
                .context("failed to build reqwest client for embed-http test")?;
            let base_url = format!("http://127.0.0.1:{port}");
            let texts = vec!["hello".to_string()];
            let vectors = embed_http(
                &client,
                &base_url,
                &texts,
                Some("Qwen/Qwen3-Embedding-0.6B"),
            )
            .await;

            assert_eq!(vectors, Some(vec![vec![0.1_f32, 0.2_f32, 0.3_f32]]));
            Ok(())
        }
    }

    fn lint_symbol_probe() {
        let _ = std::mem::size_of::<types::EmbedBatchResponse>();
        let sample = types::EmbedBatchResponse { vectors: None };
        let _ = sample.vectors.as_ref();
    }

    const _: fn() = lint_symbol_probe;
}
