use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Deserialize;

use super::{EmbeddingSettings, MistralSettings, RuntimeSettings};

const EMBEDDED_SYSTEM_SETTINGS_RESOURCE_PATH: &str = "config/xiuxian.toml";
const DEFAULT_SYSTEM_SETTINGS_RELATIVE_PATH: &str =
    "packages/rust/crates/omni-agent/resources/config/xiuxian.toml";
const DEFAULT_USER_SETTINGS_RELATIVE_PATH: &str = "xiuxian-artisan-workshop/xiuxian.toml";
const DEFAULT_CONFIG_HOME_RELATIVE_PATH: &str = ".config";
static CONFIG_HOME_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();
static EMBEDDED_SYSTEM_SETTINGS: OnceLock<RuntimeSettings> = OnceLock::new();

/// Load merged runtime settings from system and user config paths.
#[must_use]
pub fn load_runtime_settings() -> RuntimeSettings {
    let (system_path, user_path) = runtime_settings_paths();
    load_embedded_system_settings()
        .merge(load_one(&system_path))
        .merge(load_one(&user_path))
}

/// Resolve effective system/user runtime settings paths.
#[doc(hidden)]
pub fn runtime_settings_paths() -> (PathBuf, PathBuf) {
    let root = project_root();
    let system_path = root.join(DEFAULT_SYSTEM_SETTINGS_RELATIVE_PATH);
    let user_path = resolve_config_home(&root).join(DEFAULT_USER_SETTINGS_RELATIVE_PATH);
    (system_path, user_path)
}

#[doc(hidden)]
#[must_use]
/// Load and merge runtime settings from explicit system/user paths.
pub fn load_runtime_settings_from_paths(system: &Path, user: &Path) -> RuntimeSettings {
    load_one(system).merge(load_one(user))
}

fn load_one(path: &Path) -> RuntimeSettings {
    if !path.exists() {
        return RuntimeSettings::default();
    }
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) => {
            tracing::warn!(
                path = %path.display(),
                error = %error,
                "failed to read settings file; ignoring"
            );
            return RuntimeSettings::default();
        }
    };
    match toml::from_str::<RuntimeSettingsTomlBridge>(&raw) {
        Ok(bridge) => bridge.into_runtime_settings(),
        Err(error) => {
            tracing::warn!(
                path = %path.display(),
                error = %error,
                "failed to parse settings toml; ignoring file"
            );
            RuntimeSettings::default()
        }
    }
}

fn load_embedded_system_settings() -> RuntimeSettings {
    EMBEDDED_SYSTEM_SETTINGS
        .get_or_init(|| {
            let Some(file) = crate::RESOURCES.get_file(EMBEDDED_SYSTEM_SETTINGS_RESOURCE_PATH)
            else {
                tracing::warn!(
                    resource = EMBEDDED_SYSTEM_SETTINGS_RESOURCE_PATH,
                    "embedded system settings resource missing; falling back to defaults"
                );
                return RuntimeSettings::default();
            };
            let Some(raw) = file.contents_utf8() else {
                tracing::warn!(
                    resource = EMBEDDED_SYSTEM_SETTINGS_RESOURCE_PATH,
                    "embedded system settings resource is not UTF-8; falling back to defaults"
                );
                return RuntimeSettings::default();
            };
            match toml::from_str::<RuntimeSettingsTomlBridge>(raw) {
                Ok(bridge) => bridge.into_runtime_settings(),
                Err(error) => {
                    tracing::warn!(
                        resource = EMBEDDED_SYSTEM_SETTINGS_RESOURCE_PATH,
                        error = %error,
                        "failed to parse embedded system settings; falling back to defaults"
                    );
                    RuntimeSettings::default()
                }
            }
        })
        .clone()
}

#[derive(Debug, Default, Deserialize)]
struct RuntimeSettingsTomlBridge {
    #[serde(flatten)]
    runtime: RuntimeSettings,
    #[serde(default)]
    llm: RuntimeSettingsLlmBridge,
    #[serde(flatten)]
    _extra: HashMap<String, toml::Value>,
}

impl RuntimeSettingsTomlBridge {
    fn into_runtime_settings(self) -> RuntimeSettings {
        let mut runtime = self.runtime;
        if let Some(embedding) = self.llm.embedding.clone() {
            runtime.embedding = merge_embedding_settings(runtime.embedding, embedding);
        }
        if let Some(mistral) = self.llm.mistral.clone() {
            runtime.mistral = merge_mistral_settings(runtime.mistral, mistral);
        }
        apply_llm_inference_defaults(&mut runtime, &self.llm);
        runtime
    }
}

#[derive(Debug, Default, Deserialize)]
struct RuntimeSettingsLlmBridge {
    default_provider: Option<String>,
    default_model: Option<String>,
    #[serde(default)]
    providers: HashMap<String, RuntimeSettingsLlmProviderBridge>,
    embedding: Option<EmbeddingSettings>,
    mistral: Option<MistralSettings>,
    #[serde(flatten)]
    _extra: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RuntimeSettingsLlmProviderBridge {
    base_url: Option<String>,
    api_key_env: Option<String>,
    #[serde(flatten)]
    _extra: HashMap<String, toml::Value>,
}

fn apply_llm_inference_defaults(runtime: &mut RuntimeSettings, llm: &RuntimeSettingsLlmBridge) {
    if runtime.inference.provider.is_none()
        && let Some(provider) = normalize_non_empty(llm.default_provider.as_deref())
    {
        runtime.inference.provider = Some(provider.to_string());
    }

    if runtime.inference.model.is_none()
        && let Some(model) = normalize_non_empty(llm.default_model.as_deref())
    {
        runtime.inference.model = Some(model.to_string());
    }

    let Some(provider_name) = runtime
        .inference
        .provider
        .as_deref()
        .and_then(|value| normalize_non_empty(Some(value)))
    else {
        return;
    };

    let Some(provider_config) = find_provider_config(&llm.providers, provider_name) else {
        return;
    };

    if runtime.inference.base_url.is_none()
        && let Some(base_url) = normalize_non_empty(provider_config.base_url.as_deref())
    {
        runtime.inference.base_url = Some(base_url.to_string());
    }

    if runtime.inference.api_key_env.is_none()
        && let Some(api_key_env) = normalize_non_empty(provider_config.api_key_env.as_deref())
    {
        runtime.inference.api_key_env = Some(api_key_env.to_string());
    }
}

fn find_provider_config<'a>(
    providers: &'a HashMap<String, RuntimeSettingsLlmProviderBridge>,
    provider_name: &str,
) -> Option<&'a RuntimeSettingsLlmProviderBridge> {
    providers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(provider_name))
        .map(|(_, value)| value)
}

fn normalize_non_empty(value: Option<&str>) -> Option<&str> {
    value
        .map(str::trim)
        .and_then(|text| if text.is_empty() { None } else { Some(text) })
}

fn merge_embedding_settings(
    base: EmbeddingSettings,
    overlay: EmbeddingSettings,
) -> EmbeddingSettings {
    EmbeddingSettings {
        backend: overlay.backend.or(base.backend),
        timeout_secs: overlay.timeout_secs.or(base.timeout_secs),
        max_in_flight: overlay.max_in_flight.or(base.max_in_flight),
        batch_max_size: overlay.batch_max_size.or(base.batch_max_size),
        batch_max_concurrency: overlay.batch_max_concurrency.or(base.batch_max_concurrency),
        model: overlay.model.or(base.model),
        litellm_model: overlay.litellm_model.or(base.litellm_model),
        litellm_api_base: overlay.litellm_api_base.or(base.litellm_api_base),
        dimension: overlay.dimension.or(base.dimension),
        client_url: overlay.client_url.or(base.client_url),
    }
}

fn merge_mistral_settings(base: MistralSettings, overlay: MistralSettings) -> MistralSettings {
    MistralSettings {
        enabled: overlay.enabled.or(base.enabled),
        auto_start: overlay.auto_start.or(base.auto_start),
        command: overlay.command.or(base.command),
        args: overlay.args.or(base.args),
        base_url: overlay.base_url.or(base.base_url),
        startup_timeout_secs: overlay.startup_timeout_secs.or(base.startup_timeout_secs),
        probe_timeout_ms: overlay.probe_timeout_ms.or(base.probe_timeout_ms),
        probe_interval_ms: overlay.probe_interval_ms.or(base.probe_interval_ms),
        sdk_hf_cache_path: overlay.sdk_hf_cache_path.or(base.sdk_hf_cache_path),
        sdk_hf_revision: overlay.sdk_hf_revision.or(base.sdk_hf_revision),
        sdk_embedding_max_num_seqs: overlay
            .sdk_embedding_max_num_seqs
            .or(base.sdk_embedding_max_num_seqs),
    }
}

fn project_root() -> PathBuf {
    std::env::var("PRJ_ROOT").ok().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        |value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            } else {
                PathBuf::from(trimmed)
            }
        },
    )
}

/// Set config-home override (used by CLI `--conf`).
///
/// The path can be absolute, or relative to `PRJ_ROOT`/cwd.
pub fn set_config_home_override(path: impl Into<PathBuf>) {
    let path = path.into();
    if path.as_os_str().is_empty() {
        return;
    }
    if CONFIG_HOME_OVERRIDE.set(path.clone()).is_err()
        && let Some(current) = CONFIG_HOME_OVERRIDE.get()
        && current != &path
    {
        tracing::warn!(
            current = %current.display(),
            ignored = %path.display(),
            "config home override already set; ignoring subsequent value"
        );
    }
}

fn resolve_config_home(project_root: &Path) -> PathBuf {
    if let Some(path) = CONFIG_HOME_OVERRIDE.get() {
        return absolutize(project_root, path.clone());
    }

    let configured = std::env::var("PRJ_CONFIG_HOME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_CONFIG_HOME_RELATIVE_PATH.to_string());
    absolutize(project_root, PathBuf::from(configured))
}

fn absolutize(project_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    }
}
