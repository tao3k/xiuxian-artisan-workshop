//! Runtime configuration loader for `qianji.toml`.
//!
//! Resolution order:
//! 1. System config: `<PRJ_ROOT>/packages/conf/qianji.toml`
//! 2. User config: `<PRJ_CONFIG_HOME>/xiuxian-artisan-workshop/qianji.toml`
//! 3. Explicit config path: `$QIANJI_CONFIG_PATH`
//! 4. Environment overrides:
//!    - `QIANJI_LLM_MODEL`
//!    - `OPENAI_API_BASE`
//!    - `OPENAI_API_KEY`
//!    - `QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE`
//!    - `QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE_KEY`
//!    - `QIANJI_MEMORY_PROMOTION_GRAPH_DIMENSION`
//!    - `QIANJI_MEMORY_PROMOTION_PERSIST`
//!    - `QIANJI_MEMORY_PROMOTION_PERSIST_BEST_EFFORT`

use serde::Deserialize;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use xiuxian_macros::{env_first_non_empty, project_config_paths, string_first_non_empty};

const DEFAULT_MODEL: &str = "MiniMax-M2.5";
const DEFAULT_BASE_URL: &str = "http://localhost:3002/v1";
const DEFAULT_API_KEY_ENV: &str = "MINIMAX_API_KEY";
const DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE: &str = "qianji:memory_promotion";
const DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION: usize = 1024;
const DEFAULT_MEMORY_PROMOTION_PERSIST: bool = true;
const DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT: bool = true;

/// Resolved runtime config for Qianji LLM calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiRuntimeLlmConfig {
    /// Effective model name.
    pub model: String,
    /// Effective OpenAI-compatible base URL.
    pub base_url: String,
    /// Effective API key environment variable name.
    pub api_key_env: String,
    /// Effective API key value (resolved from environment).
    pub api_key: String,
}

/// Resolved runtime config for native `Wendao` memory-promotion ingestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiRuntimeWendaoIngesterConfig {
    /// Default graph scope for persisted promotion entities.
    pub graph_scope: String,
    /// Optional context key that can override graph scope at runtime.
    pub graph_scope_key: Option<String>,
    /// Graph dimension metadata passed to `KnowledgeGraph::save_to_valkey`.
    pub graph_dimension: usize,
    /// Whether persistence is enabled by default.
    pub persist: bool,
    /// Whether persistence failures should degrade gracefully by default.
    pub persist_best_effort: bool,
}

impl Default for QianjiRuntimeWendaoIngesterConfig {
    fn default() -> Self {
        Self {
            graph_scope: DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE.to_string(),
            graph_scope_key: None,
            graph_dimension: DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION,
            persist: DEFAULT_MEMORY_PROMOTION_PERSIST,
            persist_best_effort: DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT,
        }
    }
}

/// Explicit runtime environment used by the resolver (test-friendly).
#[derive(Debug, Default, Clone)]
pub struct QianjiRuntimeEnv {
    /// Optional project root override.
    pub prj_root: Option<PathBuf>,
    /// Optional config-home override.
    pub prj_config_home: Option<PathBuf>,
    /// Optional explicit qianji config path override.
    pub qianji_config_path: Option<PathBuf>,
    /// Optional `QIANJI_LLM_MODEL` override.
    pub qianji_llm_model: Option<String>,
    /// Optional `OPENAI_API_BASE` override.
    pub openai_api_base: Option<String>,
    /// Optional `OPENAI_API_KEY` override.
    pub openai_api_key: Option<String>,
    /// Optional `QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE` override.
    pub qianji_memory_promotion_graph_scope: Option<String>,
    /// Optional `QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE_KEY` override.
    pub qianji_memory_promotion_graph_scope_key: Option<String>,
    /// Optional `QIANJI_MEMORY_PROMOTION_GRAPH_DIMENSION` override.
    pub qianji_memory_promotion_graph_dimension: Option<usize>,
    /// Optional `QIANJI_MEMORY_PROMOTION_PERSIST` override.
    pub qianji_memory_promotion_persist: Option<bool>,
    /// Optional `QIANJI_MEMORY_PROMOTION_PERSIST_BEST_EFFORT` override.
    pub qianji_memory_promotion_persist_best_effort: Option<bool>,
    /// Optional values for arbitrary env keys (used for `api_key_env` lookups).
    pub extra_env: Vec<(String, String)>,
}

#[derive(Debug, Default, Deserialize)]
struct QianjiToml {
    #[serde(default)]
    llm: QianjiTomlLlm,
    #[serde(default)]
    memory_promotion: QianjiTomlMemoryPromotion,
}

#[derive(Debug, Default, Deserialize)]
struct QianjiTomlLlm {
    model: Option<String>,
    base_url: Option<String>,
    api_key_env: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct QianjiTomlMemoryPromotion {
    #[serde(default)]
    wendao: QianjiTomlWendaoIngester,
}

#[derive(Debug, Default, Deserialize)]
struct QianjiTomlWendaoIngester {
    graph_scope: Option<String>,
    graph_scope_key: Option<String>,
    graph_dimension: Option<usize>,
    persist: Option<bool>,
    persist_best_effort: Option<bool>,
}

/// Resolve `qianji.toml` and environment into an effective LLM runtime config.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_llm_config() -> io::Result<QianjiRuntimeLlmConfig> {
    resolve_qianji_runtime_llm_config_with_env(&QianjiRuntimeEnv::default())
}

/// Resolve config with explicit runtime environment overrides (for tests and tooling).
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_llm_config_with_env(
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeLlmConfig> {
    let project_root = resolve_project_root(runtime_env);
    let config_home = resolve_prj_config_home(runtime_env, &project_root);
    let file_cfg = load_qianji_toml(runtime_env, &project_root, &config_home)?;

    let model = string_first_non_empty!(
        runtime_env.qianji_llm_model.as_deref(),
        env_var_or_override(runtime_env, "QIANJI_LLM_MODEL").as_deref(),
        file_cfg.llm.model.as_deref(),
        Some(DEFAULT_MODEL),
    );

    let base_url = string_first_non_empty!(
        runtime_env.openai_api_base.as_deref(),
        env_var_or_override(runtime_env, "OPENAI_API_BASE").as_deref(),
        file_cfg.llm.base_url.as_deref(),
        Some(DEFAULT_BASE_URL),
    );

    let api_key_env = string_first_non_empty!(
        file_cfg.llm.api_key_env.as_deref(),
        Some(DEFAULT_API_KEY_ENV),
    );

    let maybe_api_key = string_first_non_empty!(
        runtime_env.openai_api_key.as_deref(),
        resolve_api_key_from_env(runtime_env, api_key_env.as_str()).as_deref(),
    );
    let api_key = if maybe_api_key.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing Qianji API key; set OPENAI_API_KEY or {api_key_env} (resolved from qianji.toml)"
            ),
        ));
    } else {
        maybe_api_key
    };

    Ok(QianjiRuntimeLlmConfig {
        model,
        base_url,
        api_key_env,
        api_key,
    })
}

/// Resolve `qianji.toml` and environment into native `Wendao` ingestion defaults.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_wendao_ingester_config()
-> io::Result<QianjiRuntimeWendaoIngesterConfig> {
    resolve_qianji_runtime_wendao_ingester_config_with_env(&QianjiRuntimeEnv::default())
}

/// Resolve `Wendao` ingestion defaults with explicit runtime environment overrides.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_wendao_ingester_config_with_env(
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeWendaoIngesterConfig> {
    let project_root = resolve_project_root(runtime_env);
    let config_home = resolve_prj_config_home(runtime_env, &project_root);
    let file_cfg = load_qianji_toml(runtime_env, &project_root, &config_home)?;

    let graph_scope = string_first_non_empty!(
        runtime_env.qianji_memory_promotion_graph_scope.as_deref(),
        env_var_or_override(runtime_env, "QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE").as_deref(),
        file_cfg.memory_promotion.wendao.graph_scope.as_deref(),
        Some(DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE),
    );
    let graph_scope_key = normalize_non_empty(Some(string_first_non_empty!(
        runtime_env
            .qianji_memory_promotion_graph_scope_key
            .as_deref(),
        env_var_or_override(runtime_env, "QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE_KEY").as_deref(),
        file_cfg.memory_promotion.wendao.graph_scope_key.as_deref(),
    )));

    let graph_dimension = runtime_env
        .qianji_memory_promotion_graph_dimension
        .or_else(|| {
            parse_usize_env_override(runtime_env, "QIANJI_MEMORY_PROMOTION_GRAPH_DIMENSION")
        })
        .or(file_cfg.memory_promotion.wendao.graph_dimension)
        .unwrap_or(DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION);

    let persist = runtime_env
        .qianji_memory_promotion_persist
        .or_else(|| parse_bool_env_override(runtime_env, "QIANJI_MEMORY_PROMOTION_PERSIST"))
        .or(file_cfg.memory_promotion.wendao.persist)
        .unwrap_or(DEFAULT_MEMORY_PROMOTION_PERSIST);

    let persist_best_effort = runtime_env
        .qianji_memory_promotion_persist_best_effort
        .or_else(|| {
            parse_bool_env_override(runtime_env, "QIANJI_MEMORY_PROMOTION_PERSIST_BEST_EFFORT")
        })
        .or(file_cfg.memory_promotion.wendao.persist_best_effort)
        .unwrap_or(DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT);

    Ok(QianjiRuntimeWendaoIngesterConfig {
        graph_scope,
        graph_scope_key,
        graph_dimension,
        persist,
        persist_best_effort,
    })
}

fn resolve_project_root(runtime_env: &QianjiRuntimeEnv) -> PathBuf {
    if let Some(path) = &runtime_env.prj_root {
        return path.clone();
    }
    if let Some(raw) = env_var_or_override(runtime_env, "PRJ_ROOT") {
        return PathBuf::from(raw);
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn resolve_prj_config_home(runtime_env: &QianjiRuntimeEnv, project_root: &Path) -> PathBuf {
    if let Some(path) = &runtime_env.prj_config_home {
        return path.clone();
    }

    if let Some(raw) = env_var_or_override(runtime_env, "PRJ_CONFIG_HOME") {
        let path = PathBuf::from(raw);
        if path.is_absolute() {
            return path;
        }
        return project_root.join(path);
    }

    project_root.join(".config")
}

fn load_qianji_toml(
    runtime_env: &QianjiRuntimeEnv,
    project_root: &Path,
    config_home: &Path,
) -> io::Result<QianjiToml> {
    let mut merged = QianjiToml::default();

    let candidates =
        if runtime_env_has_path_overrides(runtime_env) {
            let mut manual_candidates = vec![
                project_root.join("packages/conf/qianji.toml"),
                config_home.join("xiuxian-artisan-workshop/qianji.toml"),
            ];
            if let Some(explicit) = runtime_env.qianji_config_path.clone().or_else(|| {
                env_var_or_override(runtime_env, "QIANJI_CONFIG_PATH").map(PathBuf::from)
            }) {
                manual_candidates.push(explicit);
            }
            manual_candidates
        } else {
            project_config_paths!("qianji.toml", "QIANJI_CONFIG_PATH")
        };

    for path in candidates {
        if !path.exists() {
            continue;
        }
        let parsed = read_qianji_toml_file(&path)?;
        apply_llm_overlay(&mut merged.llm, parsed.llm);
        apply_memory_promotion_overlay(&mut merged.memory_promotion, parsed.memory_promotion);
    }

    Ok(merged)
}

fn runtime_env_has_path_overrides(runtime_env: &QianjiRuntimeEnv) -> bool {
    runtime_env.prj_root.is_some()
        || runtime_env.prj_config_home.is_some()
        || runtime_env.qianji_config_path.is_some()
        || runtime_env.extra_env.iter().any(|(key, _)| {
            matches!(
                key.as_str(),
                "PRJ_ROOT" | "PRJ_CONFIG_HOME" | "QIANJI_CONFIG_PATH"
            )
        })
}

fn read_qianji_toml_file(path: &Path) -> io::Result<QianjiToml> {
    let raw = fs::read_to_string(path).map_err(|e| {
        io::Error::other(format!(
            "failed to read qianji config {}: {e}",
            path.display()
        ))
    })?;
    toml::from_str::<QianjiToml>(&raw).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse qianji config {}: {e}", path.display()),
        )
    })
}

fn apply_llm_overlay(target: &mut QianjiTomlLlm, overlay: QianjiTomlLlm) {
    if let Some(model) = normalize_non_empty(overlay.model) {
        target.model = Some(model);
    }
    if let Some(base_url) = normalize_non_empty(overlay.base_url) {
        target.base_url = Some(base_url);
    }
    if let Some(api_key_env) = normalize_non_empty(overlay.api_key_env) {
        target.api_key_env = Some(api_key_env);
    }
}

fn apply_memory_promotion_overlay(
    target: &mut QianjiTomlMemoryPromotion,
    overlay: QianjiTomlMemoryPromotion,
) {
    apply_wendao_overlay(&mut target.wendao, overlay.wendao);
}

fn apply_wendao_overlay(target: &mut QianjiTomlWendaoIngester, overlay: QianjiTomlWendaoIngester) {
    if let Some(graph_scope) = normalize_non_empty(overlay.graph_scope) {
        target.graph_scope = Some(graph_scope);
    }
    if let Some(graph_scope_key) = normalize_non_empty(overlay.graph_scope_key) {
        target.graph_scope_key = Some(graph_scope_key);
    }
    if let Some(graph_dimension) = overlay.graph_dimension {
        target.graph_dimension = Some(graph_dimension);
    }
    if let Some(persist) = overlay.persist {
        target.persist = Some(persist);
    }
    if let Some(persist_best_effort) = overlay.persist_best_effort {
        target.persist_best_effort = Some(persist_best_effort);
    }
}

fn env_var_or_override(runtime_env: &QianjiRuntimeEnv, key: &str) -> Option<String> {
    if let Some(value) = env_override_non_empty(runtime_env, key) {
        return Some(value);
    }
    env_first_non_empty!(key)
}

fn resolve_api_key_from_env(runtime_env: &QianjiRuntimeEnv, api_key_env: &str) -> Option<String> {
    if let Some(value) = env_override_non_empty(runtime_env, "OPENAI_API_KEY") {
        return Some(value);
    }
    if let Some(value) = env_override_non_empty(runtime_env, api_key_env) {
        return Some(value);
    }
    env_first_non_empty!("OPENAI_API_KEY", api_key_env)
}

fn env_override_non_empty(runtime_env: &QianjiRuntimeEnv, key: &str) -> Option<String> {
    runtime_env
        .extra_env
        .iter()
        .find(|(candidate_key, _)| candidate_key == key)
        .and_then(|(_, value)| normalize_non_empty(Some(value.clone())))
}

fn parse_usize_env_override(runtime_env: &QianjiRuntimeEnv, key: &str) -> Option<usize> {
    env_var_or_override(runtime_env, key).and_then(|value| value.trim().parse::<usize>().ok())
}

fn parse_bool_env_override(runtime_env: &QianjiRuntimeEnv, key: &str) -> Option<bool> {
    env_var_or_override(runtime_env, key).and_then(|value| parse_bool_flag(value.as_str()))
}

fn parse_bool_flag(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn normalize_non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}
