//! Provider mode resolution tests for inference runtime settings.

use crate::config::RuntimeSettings;

use super::providers::{
    DEFAULT_MINIMAX_KEY_ENV, DEFAULT_OPENAI_KEY_ENV, LiteLlmProviderMode,
    resolve_provider_settings_with_env,
};

fn settings_with_inference(
    provider: Option<&str>,
    api_key_env: Option<&str>,
    base_url: Option<&str>,
    model: Option<&str>,
    timeout: Option<u64>,
    max_tokens: Option<u64>,
    max_in_flight: Option<usize>,
) -> RuntimeSettings {
    let mut settings = RuntimeSettings::default();
    settings.inference.provider = provider.map(ToString::to_string);
    settings.inference.api_key_env = api_key_env.map(ToString::to_string);
    settings.inference.base_url = base_url.map(ToString::to_string);
    settings.inference.model = model.map(ToString::to_string);
    settings.inference.timeout = timeout;
    settings.inference.max_tokens = max_tokens;
    settings.inference.max_in_flight = max_in_flight;
    settings
}

#[test]
fn provider_settings_default_to_openai_when_unspecified() {
    let settings = RuntimeSettings::default();
    let resolved = resolve_provider_settings_with_env(&settings, String::new(), None, None);
    assert_eq!(resolved.mode, LiteLlmProviderMode::OpenAi);
    assert_eq!(resolved.source, "default");
    assert_eq!(resolved.api_key_env, DEFAULT_OPENAI_KEY_ENV);
    assert_eq!(resolved.model, "");
    assert_eq!(resolved.minimax_api_base, "https://api.minimax.io/v1");
    assert_eq!(resolved.timeout_secs, 60);
    assert_eq!(resolved.max_tokens, None);
    assert_eq!(resolved.max_in_flight, None);
}

#[test]
fn provider_settings_honor_settings_for_minimax() {
    let settings = settings_with_inference(
        Some("minimax"),
        Some("MINIMAX_CUSTOM_KEY"),
        Some("https://settings.minimax/v1"),
        Some("minimax/MiniMax-M2.5"),
        Some(90),
        Some(4096),
        Some(32),
    );
    let resolved = resolve_provider_settings_with_env(&settings, String::new(), None, None);
    assert_eq!(resolved.mode, LiteLlmProviderMode::Minimax);
    assert_eq!(resolved.source, "settings");
    assert_eq!(resolved.api_key_env, "MINIMAX_CUSTOM_KEY");
    assert_eq!(resolved.model, "MiniMax-M2.5");
    assert_eq!(resolved.minimax_api_base, "https://settings.minimax/v1");
    assert_eq!(resolved.timeout_secs, 90);
    assert_eq!(resolved.max_tokens, Some(4096));
    assert_eq!(resolved.max_in_flight, Some(32));
}

#[test]
fn provider_settings_env_provider_overrides_settings_provider() {
    let settings = settings_with_inference(Some("openai"), None, None, None, None, None, None);
    let resolved =
        resolve_provider_settings_with_env(&settings, String::new(), Some("minimax"), None);
    assert_eq!(resolved.mode, LiteLlmProviderMode::Minimax);
    assert_eq!(resolved.source, "env");
    assert_eq!(resolved.api_key_env, DEFAULT_MINIMAX_KEY_ENV);
    assert_eq!(resolved.model, "MiniMax-M2.5");
}

#[test]
fn provider_settings_requested_model_takes_precedence_and_is_normalized() {
    let settings = settings_with_inference(
        Some("minimax"),
        None,
        None,
        Some("MiniMax-M2.5"),
        None,
        None,
        None,
    );
    let resolved = resolve_provider_settings_with_env(
        &settings,
        "minimax:MiniMax-M2.5-highspeed".to_string(),
        None,
        None,
    );
    assert_eq!(resolved.mode, LiteLlmProviderMode::Minimax);
    assert_eq!(resolved.model, "MiniMax-M2.5-lightning");
}

#[test]
fn provider_settings_env_minimax_api_base_overrides_settings_base_url() {
    let settings = settings_with_inference(
        Some("minimax"),
        None,
        Some("https://settings.minimax/v1"),
        None,
        None,
        None,
        None,
    );
    let resolved = resolve_provider_settings_with_env(
        &settings,
        String::new(),
        None,
        Some("https://env.minimax/v1"),
    );
    assert_eq!(resolved.minimax_api_base, "https://env.minimax/v1");
}

#[test]
fn provider_settings_timeout_and_max_tokens_fallbacks_are_sane() {
    let settings =
        settings_with_inference(Some("minimax"), None, None, None, Some(0), Some(0), Some(0));
    let resolved = resolve_provider_settings_with_env(&settings, String::new(), None, None);
    assert_eq!(resolved.timeout_secs, 60);
    assert_eq!(resolved.max_tokens, None);
    assert_eq!(resolved.max_in_flight, None);
}

#[test]
fn provider_settings_max_tokens_are_clamped_to_u32() {
    let settings = settings_with_inference(
        Some("minimax"),
        None,
        None,
        None,
        Some(120),
        Some(u64::MAX),
        Some(64),
    );
    let resolved = resolve_provider_settings_with_env(&settings, String::new(), None, None);
    assert_eq!(resolved.timeout_secs, 120);
    assert_eq!(resolved.max_tokens, Some(u32::MAX));
    assert_eq!(resolved.max_in_flight, Some(64));
}
