#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::implicit_clone,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::manual_async_fn,
    clippy::async_yields_async,
    clippy::no_effect_underscore_binding
)]

use axum::Router;
use axum::routing::get;
use xiuxian_llm::mistral::{MistralServerConfig, derive_models_url, probe_models};

#[test]
fn derive_models_url_normalizes_openai_paths() {
    assert_eq!(
        derive_models_url("http://127.0.0.1:11500"),
        Some("http://127.0.0.1:11500/v1/models".to_string())
    );
    assert_eq!(
        derive_models_url("http://127.0.0.1:11500/v1"),
        Some("http://127.0.0.1:11500/v1/models".to_string())
    );
    assert_eq!(
        derive_models_url("http://127.0.0.1:11500/v1/models"),
        Some("http://127.0.0.1:11500/v1/models".to_string())
    );
    assert_eq!(derive_models_url(" "), None);
}

#[test]
fn mistral_server_config_default_is_stable() {
    let config = MistralServerConfig::default();
    assert_eq!(config.command, "mistralrs-server");
    assert_eq!(config.base_url, "http://127.0.0.1:11500");
    assert_eq!(config.startup_timeout_secs, 45);
    assert_eq!(config.probe_timeout_ms, 1_500);
    assert_eq!(config.probe_interval_ms, 250);
    assert!(config.args.is_empty());
}

#[tokio::test]
async fn probe_models_reports_ready_on_successful_models_endpoint() {
    let app = Router::new().route("/v1/models", get(|| async { "ok" }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let addr = listener.local_addr().expect("resolve local addr");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let base_url = format!("http://{addr}");
    let status = probe_models(&base_url, 1_500).await;
    assert!(status.ready);
    assert_eq!(status.status_code, Some(200));
    assert!(!status.timed_out);
    assert!(!status.transport_error);
}

#[tokio::test]
async fn probe_models_reports_transport_error_on_unreachable_endpoint() {
    let status = probe_models("http://127.0.0.1:1", 200).await;
    assert!(!status.ready);
    assert!(status.transport_error || status.timed_out);
}
