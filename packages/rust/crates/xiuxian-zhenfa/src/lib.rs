//! Xiuxian-Zhenfa (Matrix Gateway): native-first tool microkernel with an optional JSON-RPC HTTP gateway.

mod client;
mod contracts;
mod gateway;
mod native;
mod router;
mod transmuter;
mod xml_lite;

pub use async_trait;
pub use schemars;
pub use serde_json;
pub use xiuxian_macros::zhenfa_tool;

pub use client::{ZhenfaClient, ZhenfaClientError, ZhenfaClientSuccess};
pub use contracts::{
    INTERNAL_ERROR_CODE, INVALID_PARAMS_CODE, INVALID_REQUEST_CODE, JSONRPC_VERSION,
    JsonRpcErrorObject, JsonRpcId, JsonRpcMeta, JsonRpcRequest, JsonRpcResponse,
    METHOD_NOT_FOUND_CODE, PARSE_ERROR_CODE,
};
pub use gateway::{HealthResponse, ZhenfaGatewayBuildError, ZhenfaGatewayBuilder};
pub use native::{
    ZhenfaAuditSink, ZhenfaContext, ZhenfaDispatchEvent, ZhenfaDispatchOutcome, ZhenfaError,
    ZhenfaMutationGuard, ZhenfaMutationLock, ZhenfaOrchestrator, ZhenfaOrchestratorHooks,
    ZhenfaRegistry, ZhenfaResultCache, ZhenfaSignal, ZhenfaSignalSink, ZhenfaTool,
};
pub use router::{MethodRegistry, ZhenfaMethodHandler, ZhenfaRouter, method_handler};
pub use transmuter::{ZhenfaResolveAndWashError, ZhenfaTransmuter, ZhenfaTransmuterError};
pub use xml_lite::{extract_tag_f32, extract_tag_value};
