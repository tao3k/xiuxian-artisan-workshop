//! Unified and Modular Configuration Loader for Xiuxian & Wendao.
//!
//! Supports unified `xiuxian.toml` and modular `wendao.toml` with automatic fallback.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::settings::RuntimeSettings;

/// The root configuration structure.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct XiuxianConfig {
    /// Consolidated runtime settings preserved for compatibility with unified `xiuxian.toml`.
    #[serde(flatten)]
    _runtime_settings: RuntimeSettings,

    /// LLM-specific provider configuration.
    #[serde(default)]
    pub llm: LlmConfig,

    /// Wendao (Knowledge Management) configuration.
    #[serde(default)]
    pub wendao: WendaoConfig,

    /// Qianhuan (orchestration/persona) configuration.
    #[serde(default)]
    pub qianhuan: QianhuanConfig,

    /// Zhenfa (HTTP matrix gateway) tool bridge settings.
    #[serde(default)]
    pub zhenfa: ZhenfaConfig,
}

/// LLM routing defaults and provider map for runtime model selection.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct LlmConfig {
    /// Provider key selected when no request-specific provider is supplied.
    pub default_provider: Option<String>,
    /// Optional model alias selected when no request-specific model is supplied.
    pub default_model: Option<String>,
    /// Named provider configurations keyed by provider id.
    #[serde(default)]
    pub providers: HashMap<String, LlmProviderConfig>,
}

/// Connection and model alias configuration for one LLM provider.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct LlmProviderConfig {
    /// Provider API base URL.
    pub base_url: Option<String>,
    /// Environment variable name containing the provider API key.
    pub api_key_env: Option<String>,
    /// Logical model alias to concrete provider model mapping.
    #[serde(default)]
    pub model_aliases: HashMap<String, String>,
}

/// Wendao specific settings.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct WendaoConfig {
    /// Action-oriented configurations (Notebook path, timer settings, etc.)
    #[serde(default)]
    pub zhixing: ZhixingConfig,
    /// Link graph indexing settings.
    #[serde(default)]
    pub link_graph: LinkGraphConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ZhixingConfig {
    /// Root directory for the notebook.
    pub notebook_path: Option<String>,
    /// Time zone for scheduled tasks and reminders (e.g., "Asia/Shanghai").
    pub time_zone: Option<String>,
    /// Active persona id used for Zhixing workflow rendering.
    pub persona_id: Option<String>,
    /// Default notification recipient for timer reminders when task metadata has no recipient.
    pub notification_recipient: Option<String>,
    /// Template directories for Qianhuan manifestations.
    pub template_paths: Option<Vec<String>>,
    /// Optional Valkey-backed reminder queue settings.
    #[serde(default)]
    pub reminder_queue: ZhixingReminderQueueConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ZhixingReminderQueueConfig {
    /// Override Valkey URL for reminder queue.
    pub valkey_url: Option<String>,
    /// Valkey key prefix namespace.
    pub key_prefix: Option<String>,
    /// Queue poll interval in seconds.
    pub poll_interval_seconds: Option<u64>,
    /// Maximum consumed reminders per poll.
    pub poll_batch_size: Option<usize>,
}

/// Qianhuan specific settings.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct QianhuanConfig {
    /// Persona profile resolution settings.
    #[serde(default)]
    pub persona: QianhuanPersonaConfig,
    /// Template resolution settings.
    #[serde(default)]
    pub template: QianhuanTemplateConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct QianhuanPersonaConfig {
    /// Optional single override directory for persona profiles.
    pub persona_dir: Option<String>,
    /// Optional ordered list of persona profile directories.
    pub persona_dirs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct QianhuanTemplateConfig {
    /// Optional single override directory for Qianhuan templates.
    pub template_dir: Option<String>,
    /// Optional ordered list of template directories.
    pub template_dirs: Option<Vec<String>>,
}

/// Zhenfa tool bridge settings.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ZhenfaConfig {
    /// Base URL for zhenfa gateway, for example `http://127.0.0.1:18093`.
    pub base_url: Option<String>,
    /// Explicit enabled RPC tools exposed to LLM (for example `wendao.search`).
    pub enabled_tools: Option<Vec<String>>,
    /// Optional Valkey runtime hooks for zhenfa native orchestrator.
    #[serde(default)]
    pub valkey: ZhenfaValkeyConfig,
}

/// Valkey hook settings for zhenfa native orchestrator.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ZhenfaValkeyConfig {
    /// Valkey URL (for example `redis://127.0.0.1:6379/0`).
    pub url: Option<String>,
    /// Key prefix namespace for zhenfa cache/lock/stream entries.
    pub key_prefix: Option<String>,
    /// TTL for deterministic tool result cache entries.
    pub cache_ttl_seconds: Option<u64>,
    /// TTL for mutation lock lease entries.
    pub lock_ttl_seconds: Option<u64>,
    /// Audit stream suffix name used for `XADD` events.
    pub audit_stream: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
/// Link graph indexing and watch settings for knowledge traversal.
pub struct LinkGraphConfig {
    /// Link graph backend identifier.
    pub backend: Option<String>,
    /// Explicit include directories for indexing.
    pub include_dirs: Option<Vec<String>>,
    /// Enable automatic include directory discovery.
    pub include_dirs_auto: Option<bool>,
    /// Watch roots for incremental updates.
    pub watch_dirs: Option<Vec<String>>,
    /// Glob patterns to include during watch/index operations.
    pub watch_patterns: Option<Vec<String>>,
    /// File extensions allowed for watch/index operations.
    pub watch_extensions: Option<Vec<String>>,
    /// Directories excluded from indexing/watching.
    pub exclude_dirs: Option<Vec<String>>,
    /// Cache backend settings for link graph artifacts.
    #[serde(default)]
    pub cache: LinkGraphCacheConfig,
}

/// Cache configuration for link graph persistence and acceleration.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct LinkGraphCacheConfig {
    /// Valkey URL used for cache reads/writes.
    pub valkey_url: Option<String>,
    /// Key prefix namespace for link graph cache entries.
    pub key_prefix: Option<String>,
    /// Cache entry TTL in seconds.
    pub ttl_seconds: Option<u64>,
}

/// Resolve system and user config paths with the same base logic as runtime settings.
fn resolve_config_paths(filename: &str) -> (PathBuf, PathBuf) {
    let (system_settings_path, user_settings_path) = super::settings::runtime_settings_paths();
    let system_base = system_settings_path
        .parent()
        .map_or_else(|| PathBuf::from("packages/conf"), Path::to_path_buf);
    let user_base = user_settings_path.parent().map_or_else(
        || PathBuf::from(".config/xiuxian-artisan-workshop"),
        Path::to_path_buf,
    );
    (system_base.join(filename), user_base.join(filename))
}

fn read_xiuxian_config(path: &Path) -> Option<XiuxianConfig> {
    let content = fs::read_to_string(path).ok()?;
    toml::from_str::<XiuxianConfig>(&content).ok()
}

fn merge_provider_config(base: &mut LlmProviderConfig, overlay: LlmProviderConfig) {
    if let Some(base_url) = overlay.base_url {
        base.base_url = Some(base_url);
    }
    if let Some(api_key_env) = overlay.api_key_env {
        base.api_key_env = Some(api_key_env);
    }
    base.model_aliases.extend(overlay.model_aliases);
}

fn merge_zhenfa_config(base: &mut ZhenfaConfig, overlay: ZhenfaConfig) {
    if let Some(base_url) = overlay.base_url {
        base.base_url = Some(base_url);
    }
    if let Some(enabled_tools) = overlay.enabled_tools {
        base.enabled_tools = Some(enabled_tools);
    }
    if let Some(url) = overlay.valkey.url {
        base.valkey.url = Some(url);
    }
    if let Some(key_prefix) = overlay.valkey.key_prefix {
        base.valkey.key_prefix = Some(key_prefix);
    }
    if let Some(cache_ttl_seconds) = overlay.valkey.cache_ttl_seconds {
        base.valkey.cache_ttl_seconds = Some(cache_ttl_seconds);
    }
    if let Some(lock_ttl_seconds) = overlay.valkey.lock_ttl_seconds {
        base.valkey.lock_ttl_seconds = Some(lock_ttl_seconds);
    }
    if let Some(audit_stream) = overlay.valkey.audit_stream {
        base.valkey.audit_stream = Some(audit_stream);
    }
}

fn merge_xiuxian_config(base: &mut XiuxianConfig, overlay: XiuxianConfig) {
    if let Some(default_provider) = overlay.llm.default_provider {
        base.llm.default_provider = Some(default_provider);
    }
    if let Some(default_model) = overlay.llm.default_model {
        base.llm.default_model = Some(default_model);
    }
    for (provider, provider_cfg) in overlay.llm.providers {
        if let Some(existing) = base.llm.providers.get_mut(&provider) {
            merge_provider_config(existing, provider_cfg);
        } else {
            base.llm.providers.insert(provider, provider_cfg);
        }
    }

    if let Some(notebook_path) = overlay.wendao.zhixing.notebook_path {
        base.wendao.zhixing.notebook_path = Some(notebook_path);
    }
    if let Some(time_zone) = overlay.wendao.zhixing.time_zone {
        base.wendao.zhixing.time_zone = Some(time_zone);
    }
    if let Some(persona_id) = overlay.wendao.zhixing.persona_id {
        base.wendao.zhixing.persona_id = Some(persona_id);
    }
    if let Some(notification_recipient) = overlay.wendao.zhixing.notification_recipient {
        base.wendao.zhixing.notification_recipient = Some(notification_recipient);
    }
    if let Some(template_paths) = overlay.wendao.zhixing.template_paths {
        base.wendao.zhixing.template_paths = Some(template_paths);
    }
    if let Some(valkey_url) = overlay.wendao.zhixing.reminder_queue.valkey_url {
        base.wendao.zhixing.reminder_queue.valkey_url = Some(valkey_url);
    }
    if let Some(key_prefix) = overlay.wendao.zhixing.reminder_queue.key_prefix {
        base.wendao.zhixing.reminder_queue.key_prefix = Some(key_prefix);
    }
    if let Some(poll_interval_seconds) = overlay.wendao.zhixing.reminder_queue.poll_interval_seconds
    {
        base.wendao.zhixing.reminder_queue.poll_interval_seconds = Some(poll_interval_seconds);
    }
    if let Some(poll_batch_size) = overlay.wendao.zhixing.reminder_queue.poll_batch_size {
        base.wendao.zhixing.reminder_queue.poll_batch_size = Some(poll_batch_size);
    }
    if let Some(backend) = overlay.wendao.link_graph.backend {
        base.wendao.link_graph.backend = Some(backend);
    }
    if let Some(include_dirs) = overlay.wendao.link_graph.include_dirs {
        base.wendao.link_graph.include_dirs = Some(include_dirs);
    }
    if let Some(include_dirs_auto) = overlay.wendao.link_graph.include_dirs_auto {
        base.wendao.link_graph.include_dirs_auto = Some(include_dirs_auto);
    }
    if let Some(watch_dirs) = overlay.wendao.link_graph.watch_dirs {
        base.wendao.link_graph.watch_dirs = Some(watch_dirs);
    }
    if let Some(watch_patterns) = overlay.wendao.link_graph.watch_patterns {
        base.wendao.link_graph.watch_patterns = Some(watch_patterns);
    }
    if let Some(watch_extensions) = overlay.wendao.link_graph.watch_extensions {
        base.wendao.link_graph.watch_extensions = Some(watch_extensions);
    }
    if let Some(exclude_dirs) = overlay.wendao.link_graph.exclude_dirs {
        base.wendao.link_graph.exclude_dirs = Some(exclude_dirs);
    }
    if let Some(valkey_url) = overlay.wendao.link_graph.cache.valkey_url {
        base.wendao.link_graph.cache.valkey_url = Some(valkey_url);
    }
    if let Some(key_prefix) = overlay.wendao.link_graph.cache.key_prefix {
        base.wendao.link_graph.cache.key_prefix = Some(key_prefix);
    }
    if let Some(ttl_seconds) = overlay.wendao.link_graph.cache.ttl_seconds {
        base.wendao.link_graph.cache.ttl_seconds = Some(ttl_seconds);
    }
    if let Some(persona_dir) = overlay.qianhuan.persona.persona_dir {
        base.qianhuan.persona.persona_dir = Some(persona_dir);
    }
    if let Some(persona_dirs) = overlay.qianhuan.persona.persona_dirs {
        base.qianhuan.persona.persona_dirs = Some(persona_dirs);
    }
    if let Some(template_dir) = overlay.qianhuan.template.template_dir {
        base.qianhuan.template.template_dir = Some(template_dir);
    }
    if let Some(template_dirs) = overlay.qianhuan.template.template_dirs {
        base.qianhuan.template.template_dirs = Some(template_dirs);
    }
    merge_zhenfa_config(&mut base.zhenfa, overlay.zhenfa);
}

#[must_use]
pub(super) fn load_xiuxian_config_from_paths(
    system_path: &Path,
    user_path: &Path,
) -> XiuxianConfig {
    let mut config = read_xiuxian_config(system_path).unwrap_or_default();
    if let Some(overlay) = read_xiuxian_config(user_path) {
        merge_xiuxian_config(&mut config, overlay);
    }
    config
}

#[must_use]
pub(super) fn load_xiuxian_config_from_bases(
    system_base: &Path,
    user_base: &Path,
) -> XiuxianConfig {
    let system_xiuxian_path = system_base.join("xiuxian.toml");
    let user_xiuxian_path = user_base.join("xiuxian.toml");
    let mut config = load_xiuxian_config_from_paths(&system_xiuxian_path, &user_xiuxian_path);

    if config.wendao.zhixing.notebook_path.is_none() {
        let system_wendao_path = system_base.join("wendao.toml");
        let user_wendao_path = user_base.join("wendao.toml");
        let wendao_path = if user_wendao_path.exists() {
            user_wendao_path
        } else {
            system_wendao_path
        };
        if let Ok(content) = fs::read_to_string(&wendao_path)
            && let Ok(wendao_only) = toml::from_str::<WendaoConfig>(&content)
        {
            config.wendao = wendao_only;
            tracing::info!(
                path = %wendao_path.display(),
                "Merged modular Wendao configuration."
            );
        }
    }

    config
}

/// Primary loader that merges unified and modular configurations.
#[must_use]
pub fn load_xiuxian_config() -> XiuxianConfig {
    let (system_xiuxian_path, user_xiuxian_path) = resolve_config_paths("xiuxian.toml");
    let system_base = system_xiuxian_path
        .parent()
        .map_or_else(|| PathBuf::from("packages/conf"), Path::to_path_buf);
    let user_base = user_xiuxian_path.parent().map_or_else(
        || PathBuf::from(".config/xiuxian-artisan-workshop"),
        Path::to_path_buf,
    );
    let config = load_xiuxian_config_from_bases(&system_base, &user_base);

    if read_xiuxian_config(&system_xiuxian_path).is_some() {
        tracing::debug!(
            path = %system_xiuxian_path.display(),
            "Loaded system xiuxian configuration."
        );
    }
    if read_xiuxian_config(&user_xiuxian_path).is_some() {
        tracing::debug!(
            path = %user_xiuxian_path.display(),
            "Loaded user xiuxian overlay configuration."
        );
    }

    config
}
