//! Integration tests for qianji runtime config layering.

use std::fs;
use std::io;
use std::path::Path;
use tempfile::TempDir;
use xiuxian_qianji::runtime_config::{
    QianjiRuntimeEnv, QianjiRuntimeLlmConfig, QianjiRuntimeWendaoIngesterConfig,
    resolve_qianji_runtime_llm_config_with_env,
    resolve_qianji_runtime_wendao_ingester_config_with_env,
};

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| {
            panic!(
                "failed to create parent directory '{}': {err}",
                parent.display()
            )
        });
    }
    fs::write(path, content)
        .unwrap_or_else(|err| panic!("failed to write file '{}': {err}", path.display()));
}

fn resolve(env: &QianjiRuntimeEnv) -> QianjiRuntimeLlmConfig {
    match resolve_qianji_runtime_llm_config_with_env(env) {
        Ok(cfg) => cfg,
        Err(err) => panic!("runtime config resolve should succeed: {err}"),
    }
}

fn resolve_wendao(env: &QianjiRuntimeEnv) -> QianjiRuntimeWendaoIngesterConfig {
    match resolve_qianji_runtime_wendao_ingester_config_with_env(env) {
        Ok(cfg) => cfg,
        Err(err) => panic!("runtime wendao config resolve should succeed: {err}"),
    }
}

#[test]
fn runtime_config_uses_system_file_defaults() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/conf/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            ("QIANJI_LLM_MODEL".to_string(), String::new()),
            ("OPENAI_API_BASE".to_string(), String::new()),
            ("OPENAI_API_KEY".to_string(), String::new()),
            ("SYSTEM_API_KEY".to_string(), "system-secret".to_string()),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "system-model");
    assert_eq!(cfg.base_url, "http://system.local/v1");
    assert_eq!(cfg.api_key_env, "SYSTEM_API_KEY");
    assert_eq!(cfg.api_key, "system-secret");
}

#[test]
fn runtime_config_user_file_overrides_system_file() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/conf/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );
    write_file(
        &config_home.join("xiuxian-artisan-workshop/qianji.toml"),
        r#"
[llm]
model = "user-model"
base_url = "http://user.local/v1"
api_key_env = "USER_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            ("QIANJI_LLM_MODEL".to_string(), String::new()),
            ("OPENAI_API_BASE".to_string(), String::new()),
            ("OPENAI_API_KEY".to_string(), String::new()),
            ("USER_API_KEY".to_string(), "user-secret".to_string()),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "user-model");
    assert_eq!(cfg.base_url, "http://user.local/v1");
    assert_eq!(cfg.api_key_env, "USER_API_KEY");
    assert_eq!(cfg.api_key, "user-secret");
}

#[test]
fn runtime_config_explicit_path_overrides_user_and_system() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");
    let explicit_path = tmp.path().join("explicit/qianji.toml");

    write_file(
        &project_root.join("packages/conf/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );
    write_file(
        &config_home.join("xiuxian-artisan-workshop/qianji.toml"),
        r#"
[llm]
model = "user-model"
base_url = "http://user.local/v1"
api_key_env = "USER_API_KEY"
"#,
    );
    write_file(
        &explicit_path,
        r#"
[llm]
model = "explicit-model"
base_url = "http://explicit.local/v1"
api_key_env = "EXPLICIT_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        qianji_config_path: Some(explicit_path),
        extra_env: vec![
            ("QIANJI_LLM_MODEL".to_string(), String::new()),
            ("OPENAI_API_BASE".to_string(), String::new()),
            ("OPENAI_API_KEY".to_string(), String::new()),
            (
                "EXPLICIT_API_KEY".to_string(),
                "explicit-secret".to_string(),
            ),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "explicit-model");
    assert_eq!(cfg.base_url, "http://explicit.local/v1");
    assert_eq!(cfg.api_key_env, "EXPLICIT_API_KEY");
    assert_eq!(cfg.api_key, "explicit-secret");
}

#[test]
fn runtime_config_env_overrides_file_layers() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/conf/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        qianji_llm_model: Some("env-model".to_string()),
        openai_api_base: Some("http://env.local/v1".to_string()),
        openai_api_key: Some("env-openai-key".to_string()),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "env-model");
    assert_eq!(cfg.base_url, "http://env.local/v1");
    assert_eq!(cfg.api_key_env, "SYSTEM_API_KEY");
    assert_eq!(cfg.api_key, "env-openai-key");
}

#[test]
fn runtime_config_prefers_openai_api_key_over_named_env_key() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/conf/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            ("OPENAI_API_KEY".to_string(), "openai-secret".to_string()),
            ("SYSTEM_API_KEY".to_string(), "system-secret".to_string()),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.api_key_env, "SYSTEM_API_KEY");
    assert_eq!(cfg.api_key, "openai-secret");
}

#[test]
fn runtime_config_parse_error_surfaces_as_invalid_data() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");
    write_file(
        &project_root.join("packages/conf/qianji.toml"),
        "this is not valid toml = ]",
    );

    let result = resolve_qianji_runtime_llm_config_with_env(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        ..QianjiRuntimeEnv::default()
    });

    let Err(err) = result else {
        panic!("invalid qianji.toml should return error");
    };
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn runtime_config_missing_api_key_returns_not_found() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/conf/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );

    let result = resolve_qianji_runtime_llm_config_with_env(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            ("OPENAI_API_KEY".to_string(), String::new()),
            ("SYSTEM_API_KEY".to_string(), String::new()),
        ],
        ..QianjiRuntimeEnv::default()
    });

    let Err(err) = result else {
        panic!("missing API key should return error");
    };
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
    assert!(err.to_string().contains("SYSTEM_API_KEY"));
}

#[test]
fn runtime_wendao_config_uses_system_defaults() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/conf/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"

[memory_promotion.wendao]
graph_scope = "scope:system"
graph_scope_key = "promotion_scope"
graph_dimension = 2048
persist = true
persist_best_effort = false
"#,
    );

    let cfg = resolve_wendao(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.graph_scope, "scope:system");
    assert_eq!(cfg.graph_scope_key.as_deref(), Some("promotion_scope"));
    assert_eq!(cfg.graph_dimension, 2048);
    assert!(cfg.persist);
    assert!(!cfg.persist_best_effort);
}

#[test]
fn runtime_wendao_config_env_overrides_file() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/conf/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"

[memory_promotion.wendao]
graph_scope = "scope:system"
graph_dimension = 2048
persist = true
persist_best_effort = true
"#,
    );

    let cfg = resolve_wendao(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            (
                "QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE".to_string(),
                "scope:env".to_string(),
            ),
            (
                "QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE_KEY".to_string(),
                "scope_key_env".to_string(),
            ),
            (
                "QIANJI_MEMORY_PROMOTION_GRAPH_DIMENSION".to_string(),
                "4096".to_string(),
            ),
            (
                "QIANJI_MEMORY_PROMOTION_PERSIST".to_string(),
                "false".to_string(),
            ),
            (
                "QIANJI_MEMORY_PROMOTION_PERSIST_BEST_EFFORT".to_string(),
                "false".to_string(),
            ),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.graph_scope, "scope:env");
    assert_eq!(cfg.graph_scope_key.as_deref(), Some("scope_key_env"));
    assert_eq!(cfg.graph_dimension, 4096);
    assert!(!cfg.persist);
    assert!(!cfg.persist_best_effort);
}
