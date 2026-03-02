//! Contract tests for zhenfa JSON-RPC envelope models.

use serde_json::json;
use xiuxian_zhenfa::{
    INVALID_PARAMS_CODE, INVALID_REQUEST_CODE, JsonRpcErrorObject, JsonRpcId, JsonRpcRequest,
    JsonRpcResponse,
};

#[test]
fn jsonrpc_request_validation_rejects_invalid_version() {
    let request = JsonRpcRequest {
        jsonrpc: "1.0".to_string(),
        method: "wendao.search".to_string(),
        id: JsonRpcId::String("req-1".to_string()),
        params: json!({ "query": "agenda" }),
        meta: None,
    };
    let Err(error) = request.validate() else {
        panic!("invalid jsonrpc version should fail");
    };
    assert_eq!(error.code, INVALID_REQUEST_CODE);
}

#[test]
fn jsonrpc_request_validation_rejects_non_object_params() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "wendao.search".to_string(),
        id: JsonRpcId::String("req-1".to_string()),
        params: json!(["bad"]),
        meta: None,
    };
    let Err(error) = request.validate() else {
        panic!("non-object params should fail");
    };
    assert_eq!(error.code, INVALID_PARAMS_CODE);
}

#[test]
fn jsonrpc_response_success_contains_result_only() {
    let response = JsonRpcResponse::success(
        JsonRpcId::String("req-1".to_string()),
        "<entity id=\"task_01\">Write tests</entity>".to_string(),
        None,
    );
    assert!(response.error.is_none());
    assert!(response.result.is_some());
}

#[test]
fn jsonrpc_response_error_contains_error_only() {
    let response = JsonRpcResponse::error(
        JsonRpcId::String("req-2".to_string()),
        JsonRpcErrorObject::invalid_params("limit must be <= 100"),
    );
    assert!(response.result.is_none());
    assert!(response.error.is_some());
}
