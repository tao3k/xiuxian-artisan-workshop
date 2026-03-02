use std::fs;

use super::xiuxian::load_xiuxian_config_from_paths;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn load_xiuxian_config_from_paths_user_overrides_system_defaults() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let system_path = temp_dir.path().join("system.xiuxian.toml");
    let user_path = temp_dir.path().join("user.xiuxian.toml");

    fs::write(
        &system_path,
        r#"
[llm]
default_provider = "openai"
default_model = "gpt-4o-mini"

[llm.providers.openai]
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
"#,
    )?;

    fs::write(
        &user_path,
        r#"
[llm]
default_provider = "minimax"

[llm.providers.minimax]
base_url = "https://api.minimax.io/v1"
api_key_env = "MINIMAX_API_KEY"
"#,
    )?;

    let merged = load_xiuxian_config_from_paths(&system_path, &user_path);

    assert_eq!(merged.llm.default_provider.as_deref(), Some("minimax"));
    assert_eq!(merged.llm.default_model.as_deref(), Some("gpt-4o-mini"));
    assert!(merged.llm.providers.contains_key("openai"));
    assert!(merged.llm.providers.contains_key("minimax"));
    Ok(())
}

#[test]
fn load_xiuxian_config_from_paths_user_provider_overrides_system_provider() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let system_path = temp_dir.path().join("system.xiuxian.toml");
    let user_path = temp_dir.path().join("user.xiuxian.toml");

    fs::write(
        &system_path,
        r#"
[llm.providers.openai]
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
"#,
    )?;

    fs::write(
        &user_path,
        r#"
[llm.providers.openai]
base_url = "https://openai.example.internal/v1"
api_key_env = "OPENAI_API_KEY_ALT"
"#,
    )?;

    let merged = load_xiuxian_config_from_paths(&system_path, &user_path);
    let Some(openai) = merged.llm.providers.get("openai") else {
        panic!("openai provider should exist");
    };
    assert_eq!(
        openai.base_url.as_deref(),
        Some("https://openai.example.internal/v1")
    );
    assert_eq!(openai.api_key_env.as_deref(), Some("OPENAI_API_KEY_ALT"));
    Ok(())
}

#[test]
fn load_xiuxian_config_from_paths_invalid_overlay_keeps_system_config() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let system_path = temp_dir.path().join("system.xiuxian.toml");
    let user_path = temp_dir.path().join("user.xiuxian.toml");

    fs::write(
        &system_path,
        r#"
[llm]
default_provider = "openai"
"#,
    )?;

    fs::write(&user_path, "not a valid toml")?;

    let merged = load_xiuxian_config_from_paths(&system_path, &user_path);
    assert_eq!(merged.llm.default_provider.as_deref(), Some("openai"));
    Ok(())
}

#[test]
fn load_xiuxian_config_merges_reminder_queue_and_link_graph_cache() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let system_path = temp_dir.path().join("system.xiuxian.toml");
    let user_path = temp_dir.path().join("user.xiuxian.toml");

    fs::write(
        &system_path,
        r#"
[wendao.zhixing.reminder_queue]
valkey_url = "redis://127.0.0.1:6379/0"
key_prefix = "xiuxian_zhixing:heyi:reminder"
poll_interval_seconds = 5
poll_batch_size = 128

[wendao.link_graph.cache]
valkey_url = "redis://127.0.0.1:6379/1"
key_prefix = "xiuxian_wendao:link_graph:index"
ttl_seconds = 300
"#,
    )?;

    fs::write(
        &user_path,
        r"
[wendao.zhixing.reminder_queue]
poll_interval_seconds = 3
",
    )?;

    let merged = load_xiuxian_config_from_paths(&system_path, &user_path);
    assert_eq!(
        merged.wendao.zhixing.reminder_queue.valkey_url.as_deref(),
        Some("redis://127.0.0.1:6379/0")
    );
    assert_eq!(
        merged.wendao.zhixing.reminder_queue.key_prefix.as_deref(),
        Some("xiuxian_zhixing:heyi:reminder")
    );
    assert_eq!(
        merged.wendao.zhixing.reminder_queue.poll_interval_seconds,
        Some(3)
    );
    assert_eq!(
        merged.wendao.zhixing.reminder_queue.poll_batch_size,
        Some(128)
    );
    assert_eq!(
        merged.wendao.link_graph.cache.valkey_url.as_deref(),
        Some("redis://127.0.0.1:6379/1")
    );
    Ok(())
}

#[test]
fn load_xiuxian_config_merges_link_graph_watch_extensions() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let system_path = temp_dir.path().join("system.xiuxian.toml");
    let user_path = temp_dir.path().join("user.xiuxian.toml");

    fs::write(
        &system_path,
        r#"
[wendao.link_graph]
watch_patterns = ["**/*"]
watch_extensions = ["md", "markdown", "org"]
"#,
    )?;

    fs::write(
        &user_path,
        r#"
[wendao.link_graph]
watch_extensions = ["orgm", "j2", "toml"]
"#,
    )?;

    let merged = load_xiuxian_config_from_paths(&system_path, &user_path);
    assert_eq!(
        merged.wendao.link_graph.watch_extensions,
        Some(vec![
            "orgm".to_string(),
            "j2".to_string(),
            "toml".to_string()
        ])
    );
    Ok(())
}

#[test]
fn load_xiuxian_config_from_paths_user_overrides_template_paths() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let system_path = temp_dir.path().join("system.xiuxian.toml");
    let user_path = temp_dir.path().join("user.xiuxian.toml");

    fs::write(
        &system_path,
        r#"
[wendao.zhixing]
persona_id = "agenda_steward"
template_paths = ["assets/templates", ".omni/templates"]
"#,
    )?;

    fs::write(
        &user_path,
        r#"
[wendao.zhixing]
persona_id = "planner_master"
template_paths = ["custom/templates"]
"#,
    )?;

    let merged = load_xiuxian_config_from_paths(&system_path, &user_path);
    assert_eq!(
        merged.wendao.zhixing.persona_id.as_deref(),
        Some("planner_master")
    );
    assert_eq!(
        merged.wendao.zhixing.template_paths,
        Some(vec!["custom/templates".to_string()])
    );
    Ok(())
}

#[test]
fn load_xiuxian_config_from_paths_user_overrides_notification_recipient() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let system_path = temp_dir.path().join("system.xiuxian.toml");
    let user_path = temp_dir.path().join("user.xiuxian.toml");

    fs::write(
        &system_path,
        r#"
[wendao.zhixing]
notification_recipient = "telegram:-1001111111111"
"#,
    )?;

    fs::write(
        &user_path,
        r#"
[wendao.zhixing]
notification_recipient = "discord:123456789012345678"
"#,
    )?;

    let merged = load_xiuxian_config_from_paths(&system_path, &user_path);
    assert_eq!(
        merged.wendao.zhixing.notification_recipient.as_deref(),
        Some("discord:123456789012345678")
    );
    Ok(())
}

#[test]
fn load_xiuxian_config_from_paths_user_overrides_qianhuan_persona_dirs() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let system_path = temp_dir.path().join("system.xiuxian.toml");
    let user_path = temp_dir.path().join("user.xiuxian.toml");

    fs::write(
        &system_path,
        r#"
[qianhuan.persona]
persona_dir = "~/.config/xiuxian-artisan-workshop/personas"
persona_dirs = ["assets/personas"]
"#,
    )?;

    fs::write(
        &user_path,
        r#"
[qianhuan.persona]
persona_dirs = ["./custom/personas", "./shared/personas"]
"#,
    )?;

    let merged = load_xiuxian_config_from_paths(&system_path, &user_path);
    assert_eq!(
        merged.qianhuan.persona.persona_dir.as_deref(),
        Some("~/.config/xiuxian-artisan-workshop/personas")
    );
    assert_eq!(
        merged.qianhuan.persona.persona_dirs,
        Some(vec![
            "./custom/personas".to_string(),
            "./shared/personas".to_string()
        ])
    );
    Ok(())
}

#[test]
fn load_xiuxian_config_from_paths_user_overrides_qianhuan_template_dirs() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let system_path = temp_dir.path().join("system.xiuxian.toml");
    let user_path = temp_dir.path().join("user.xiuxian.toml");

    fs::write(
        &system_path,
        r#"
[qianhuan.template]
template_dir = "~/.config/xiuxian-artisan-workshop/qianhuan/templates"
template_dirs = ["packages/rust/crates/xiuxian-qianhuan/resources/qianhuan/templates"]
"#,
    )?;

    fs::write(
        &user_path,
        r#"
[qianhuan.template]
template_dirs = ["./custom/qianhuan/templates", "./team/qianhuan/templates"]
"#,
    )?;

    let merged = load_xiuxian_config_from_paths(&system_path, &user_path);
    assert_eq!(
        merged.qianhuan.template.template_dir.as_deref(),
        Some("~/.config/xiuxian-artisan-workshop/qianhuan/templates")
    );
    assert_eq!(
        merged.qianhuan.template.template_dirs,
        Some(vec![
            "./custom/qianhuan/templates".to_string(),
            "./team/qianhuan/templates".to_string()
        ])
    );
    Ok(())
}

#[test]
fn load_xiuxian_config_from_paths_user_overrides_zhenfa_bridge_config() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let system_path = temp_dir.path().join("system.xiuxian.toml");
    let user_path = temp_dir.path().join("user.xiuxian.toml");

    fs::write(
        &system_path,
        r#"
[zhenfa]
base_url = "http://127.0.0.1:18093"
enabled_tools = ["wendao.search", "qianhuan.reload"]

[zhenfa.valkey]
url = "redis://127.0.0.1:6379/0"
key_prefix = "omni:zhenfa"
cache_ttl_seconds = 120
lock_ttl_seconds = 15
audit_stream = "dispatch.audit"
"#,
    )?;

    fs::write(
        &user_path,
        r#"
[zhenfa]
enabled_tools = ["wendao.search"]

[zhenfa.valkey]
key_prefix = "user:omni:zhenfa"
"#,
    )?;

    let merged = load_xiuxian_config_from_paths(&system_path, &user_path);
    assert_eq!(
        merged.zhenfa.base_url.as_deref(),
        Some("http://127.0.0.1:18093")
    );
    assert_eq!(
        merged.zhenfa.enabled_tools,
        Some(vec!["wendao.search".to_string()])
    );
    assert_eq!(
        merged.zhenfa.valkey.url.as_deref(),
        Some("redis://127.0.0.1:6379/0")
    );
    assert_eq!(
        merged.zhenfa.valkey.key_prefix.as_deref(),
        Some("user:omni:zhenfa")
    );
    assert_eq!(merged.zhenfa.valkey.cache_ttl_seconds, Some(120));
    assert_eq!(merged.zhenfa.valkey.lock_ttl_seconds, Some(15));
    assert_eq!(
        merged.zhenfa.valkey.audit_stream.as_deref(),
        Some("dispatch.audit")
    );
    Ok(())
}
