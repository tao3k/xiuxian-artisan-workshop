use axum::{Json, Router, http::StatusCode, routing::post};
use serde_json::{Value, json};
use xiuxian_zhenfa::{MethodRegistry, ZhenfaRouter};

use super::models::{WendaoSearchHttpResponse, WendaoSearchRequest};
use super::rpc::{execute_search, search_from_rpc_params};

const WENDAO_PREFIX: &str = "/v1/wendao";
const WENDAO_SEARCH_ROUTE: &str = "/v1/wendao/search";

/// `xiuxian-wendao` adapter mounted into `xiuxian-zhenfa` gateway.
#[derive(Clone, Default)]
pub struct WendaoZhenfaRouter;

impl WendaoZhenfaRouter {
    /// Create a new Wendao router adapter.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ZhenfaRouter for WendaoZhenfaRouter {
    fn prefix(&self) -> &'static str {
        WENDAO_PREFIX
    }

    fn mount(&self, router: Router) -> Router {
        router.merge(Router::new().route(WENDAO_SEARCH_ROUTE, post(search_http)))
    }

    fn register_methods(&self, registry: &mut MethodRegistry) {
        registry.register_fn("wendao.search", move |params, _meta| async move {
            search_from_rpc_params(params)
        });
    }
}

async fn search_http(
    Json(body): Json<WendaoSearchRequest>,
) -> Result<Json<WendaoSearchHttpResponse>, (StatusCode, Json<Value>)> {
    execute_search(&body)
        .map(|result| Json(WendaoSearchHttpResponse { result }))
        .map_err(|error| internal_http_error(&error))
}

fn internal_http_error(error: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "wendao search failed",
            "details": error
        })),
    )
}
