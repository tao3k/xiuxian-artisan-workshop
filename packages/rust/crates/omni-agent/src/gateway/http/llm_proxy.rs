//! LLM Gateway Proxy Handler
//!
//! Provides a reverse proxy for OpenAI-compatible `/v1/chat/completions` requests.
//! It dynamically routes requests to different providers (OpenAI, MiniMax, Anthropic, etc.)
//! based on the model prefix (e.g., `minimax/MiniMax-M2.1`) or falls back to `settings.yaml` configuration.

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::body::Body;
use serde_json::Value;
use std::env;
use std::sync::LazyLock;
use reqwest::Client;
use crate::config::{load_runtime_settings, load_xiuxian_config};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .build()
        .expect("Failed to build HTTP client for LLM proxy")
});

/// Map provider alias to Base URL and API Key Env Var using xiuxian.toml or hardcoded fallbacks
fn resolve_provider_config(provider: &str) -> (String, String) {
    let xiuxian_conf = load_xiuxian_config();
    if let Some(cfg) = xiuxian_conf.llm.providers.get(provider) {
        if let (Some(url), Some(env_key)) = (&cfg.base_url, &cfg.api_key_env) {
            return (url.clone(), env_key.clone());
        }
    }

    match provider {
        "minimax" => (
            "https://api.minimax.io/v1".to_string(),
            "MINIMAX_API_KEY".to_string(),
        ),
        "anthropic" => (
            "https://api.anthropic.com/v1".to_string(),
            "ANTHROPIC_API_KEY".to_string(),
        ),
        "azure" => (
            env::var("AZURE_OPENAI_ENDPOINT").unwrap_or_default(),
            "AZURE_OPENAI_API_KEY".to_string(),
        ),
        "deepseek" => (
            "https://api.deepseek.com/v1".to_string(),
            "DEEPSEEK_API_KEY".to_string(),
        ),
        _ => (
            "https://api.openai.com/v1".to_string(),
            "OPENAI_API_KEY".to_string(),
        ),
    }
}

/// Apply model aliases from xiuxian.toml
fn apply_model_alias(provider: &str, model: &str) -> String {
    let xiuxian_conf = load_xiuxian_config();
    let lower_model = model.to_lowercase();
    
    if let Some(cfg) = xiuxian_conf.llm.providers.get(provider) {
        if let Some(alias) = cfg.model_aliases.get(&lower_model) {
            return alias.clone();
        }
    }
    
    // Hardcoded fallback for minimax casing fix
    if provider == "minimax" {
        if lower_model.starts_with("minimax-") {
            let suffix = &lower_model[8..];
            let mut actual_model = format!("MiniMax-{}", suffix);
            if lower_model == "minimax-m2.1-highspeed" {
                actual_model = "MiniMax-M2.1-lightning".to_string();
            }
            return actual_model;
        }
    }
    
    model.to_string()
}

/// Handle `/v1/chat/completions` request.
/// Parses the JSON body to extract `model`. 
/// Uses `provider/model_name` syntax if present (e.g., `minimax/MiniMax-M2.5`).
/// Otherwise, falls back to the configured provider in `settings.yaml` and `xiuxian.toml`.
pub async fn handle_chat_completions(
    req: Request,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let settings = load_runtime_settings();
    let xiuxian_conf = load_xiuxian_config();
    
    let default_provider = settings.inference.provider
        .or(xiuxian_conf.llm.default_provider)
        .unwrap_or_else(|| "openai".to_string());
        
    // Read body as JSON to modify the model string
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read body: {}", e)))?;
        
    let mut json_body: Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;

    let mut provider = default_provider.clone();
    let mut base_url = settings.inference.base_url.clone();
    let mut api_key_env = settings.inference.api_key_env.clone();

    if let Some(model_val) = json_body.get_mut("model") {
        if let Some(model_str) = model_val.as_str() {
            let mut actual_model = model_str.to_string();
            
            // Check for LiteLLM style prefix (e.g., minimax/MiniMax-M2.5)
            if let Some((prefix, suffix)) = model_str.split_once('/') {
                provider = prefix.to_lowercase();
                actual_model = suffix.to_string();
                
                // Clear custom base_url if we are overriding provider from prefix
                base_url = None;
                api_key_env = None;
            }
            
            actual_model = apply_model_alias(&provider, &actual_model);
            
            // Update the model string in the payload to the actual model
            *model_val = Value::String(actual_model);
        }
    }

    let (resolved_base_url, resolved_key_env) = resolve_provider_config(&provider);
    let target_base_url = base_url.unwrap_or(resolved_base_url);
    let target_api_key_env = api_key_env.unwrap_or(resolved_key_env);
    
    let api_key = env::var(&target_api_key_env).unwrap_or_default();
    if api_key.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Missing API key for provider {}. Set {}", provider, target_api_key_env),
        ));
    }

    let target_url = format!("{}/chat/completions", target_base_url.trim_end_matches('/'));

    let mut proxy_req = HTTP_CLIENT.post(&target_url).json(&json_body);

    // Some providers (like Anthropic) require specific headers.
    if provider == "anthropic" {
        proxy_req = proxy_req
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01");
    } else {
        proxy_req = proxy_req.header("Authorization", format!("Bearer {}", api_key));
    }

    let proxy_res = proxy_req
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Upstream request failed: {}", e)))?;

    let status = proxy_res.status();
    let headers = proxy_res.headers().clone();
    
    // Convert reqwest Response body to axum Body stream
    let body_stream = Body::from_stream(proxy_res.bytes_stream());

    let mut axum_res = body_stream.into_response();
    *axum_res.status_mut() = status;
    
    // Forward essential headers
    for (k, v) in headers.into_iter() {
        if let Some(key) = k {
            if key == axum::http::header::CONTENT_TYPE || key == axum::http::header::CONTENT_ENCODING || key.as_str().starts_with("x-") {
                axum_res.headers_mut().insert(key, v);
            }
        }
    }

    Ok(axum_res)
}
