use async_trait::async_trait;
use axum::Router;
use serde_json::Value;

use crate::contracts::{JsonRpcErrorObject, JsonRpcMeta};

use super::registry::MethodRegistry;

/// Trait implemented by domain crates to extend the Zhenfa gateway.
pub trait ZhenfaRouter: Send + Sync {
    /// Base URL prefix owned by this router (for example `/v1/wendao`).
    fn prefix(&self) -> &'static str;

    /// Mount domain routes into the shared Axum router.
    fn mount(&self, router: Router) -> Router;

    /// Register JSON-RPC methods handled by this router.
    fn register_methods(&self, _registry: &mut MethodRegistry) {}
}

/// Async handler for one JSON-RPC `method`.
#[async_trait]
pub trait ZhenfaMethodHandler: Send + Sync {
    /// Execute method with JSON params and optional metadata.
    async fn call(
        &self,
        params: Value,
        meta: Option<JsonRpcMeta>,
    ) -> Result<String, JsonRpcErrorObject>;
}
