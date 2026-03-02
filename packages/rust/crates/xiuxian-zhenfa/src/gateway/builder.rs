use std::collections::HashSet;
use std::future::Future;

use axum::{Extension, Router, routing::get, routing::post};
use serde_json::Value;
use thiserror::Error;

use crate::contracts::{JsonRpcErrorObject, JsonRpcMeta};
use crate::router::{MethodRegistry, ZhenfaRouter};

use super::handlers::{health_handler, rpc_handler};

/// Build errors for the Zhenfa gateway.
#[derive(Debug, Error)]
pub enum ZhenfaGatewayBuildError {
    /// Invalid router prefix format.
    #[error("invalid router prefix `{0}` (must start with `/` and cannot be `/`)")]
    InvalidPrefix(String),
    /// Duplicate router prefix.
    #[error("duplicate router prefix `{0}`")]
    DuplicatePrefix(String),
}

/// Builder for the unified Zhenfa gateway.
#[derive(Default)]
pub struct ZhenfaGatewayBuilder {
    routers: Vec<Box<dyn ZhenfaRouter>>,
    methods: MethodRegistry,
}

impl ZhenfaGatewayBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one domain router plugin.
    #[must_use]
    pub fn add_router<R>(mut self, router: R) -> Self
    where
        R: ZhenfaRouter + 'static,
    {
        self.routers.push(Box::new(router));
        self
    }

    /// Register one ad-hoc RPC method handler.
    #[must_use]
    pub fn register_method_fn<F, Fut>(mut self, method: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Value, Option<JsonRpcMeta>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<String, JsonRpcErrorObject>> + Send + 'static,
    {
        self.methods.register_fn(method, handler);
        self
    }

    /// Build the Axum router.
    ///
    /// # Errors
    /// Returns an error when router prefixes are invalid or duplicate.
    pub fn build(mut self) -> Result<Router, ZhenfaGatewayBuildError> {
        let mut prefixes = HashSet::new();
        for router in &self.routers {
            let prefix = router.prefix();
            validate_prefix(prefix)?;
            if !prefixes.insert(prefix.to_string()) {
                return Err(ZhenfaGatewayBuildError::DuplicatePrefix(prefix.to_string()));
            }
            router.register_methods(&mut self.methods);
        }

        let methods = self.methods.clone();
        let mut app = Router::new()
            .route("/healthz", get(health_handler))
            .route("/rpc", post(rpc_handler))
            .layer(Extension(methods));

        for router in &self.routers {
            app = router.mount(app);
        }

        Ok(app)
    }
}

fn validate_prefix(prefix: &str) -> Result<(), ZhenfaGatewayBuildError> {
    let valid = prefix.starts_with('/') && prefix != "/" && !prefix.contains("//");
    if valid {
        Ok(())
    } else {
        Err(ZhenfaGatewayBuildError::InvalidPrefix(prefix.to_string()))
    }
}
