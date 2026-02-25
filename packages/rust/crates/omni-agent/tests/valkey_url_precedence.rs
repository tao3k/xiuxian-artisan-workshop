#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use omni_agent::{SessionGate, SessionStore};
use tempfile::TempDir;

const CHILD_ENV_KEY: &str = "OMNI_AGENT_VALKEY_PRECEDENCE_CHILD";
const CHILD_CASE_KEY: &str = "OMNI_AGENT_VALKEY_PRECEDENCE_CASE";

fn write_runtime_settings(root: &Path, system_yaml: &str) -> Result<()> {
    let system_path = root.join("packages/conf/settings.yaml");
    let user_path = root.join(".config/omni-dev-fusion/settings.yaml");
    if let Some(parent) = system_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = user_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(system_path, system_yaml)?;
    std::fs::write(user_path, "")?;
    Ok(())
}

fn run_child_case(root: &Path, case: &str, valkey_url: &str) -> Result<()> {
    let test_binary = std::env::current_exe().context("resolve current test binary path")?;
    let output = Command::new(test_binary)
        .arg("--exact")
        .arg("valkey_url_precedence_child_probe")
        .arg("--nocapture")
        .env(CHILD_ENV_KEY, "1")
        .env(CHILD_CASE_KEY, case)
        .env("PRJ_ROOT", root)
        .env("PRJ_CONFIG_HOME", root.join(".config"))
        .env("VALKEY_URL", valkey_url)
        .output()
        .with_context(|| format!("spawn child probe for case={case}"))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "child probe failed for case={case} exit_code={:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            stdout,
            stderr
        );
    }
    Ok(())
}

#[test]
fn valkey_url_resolution_prefers_settings_and_keeps_env_fallback() -> Result<()> {
    let case_settings_first = TempDir::new()?;
    write_runtime_settings(
        case_settings_first.path(),
        r#"
session:
  valkey_url: "redis://127.0.0.1:6379/0"
telegram:
  foreground_session_gate_backend: "valkey"
"#,
    )?;
    run_child_case(
        case_settings_first.path(),
        "settings_first",
        "://invalid-url-should-not-win",
    )?;

    let case_env_fallback = TempDir::new()?;
    write_runtime_settings(
        case_env_fallback.path(),
        r#"
session:
  valkey_url: null
telegram:
  foreground_session_gate_backend: "valkey"
"#,
    )?;
    run_child_case(
        case_env_fallback.path(),
        "env_fallback",
        "redis://127.0.0.1:6379/1",
    )?;

    Ok(())
}

#[test]
fn valkey_url_precedence_child_probe() -> Result<()> {
    if std::env::var(CHILD_ENV_KEY).ok().as_deref() != Some("1") {
        return Ok(());
    }

    let case = std::env::var(CHILD_CASE_KEY).unwrap_or_else(|_| "unknown".to_string());
    match case.as_str() {
        "settings_first" | "env_fallback" => {}
        other => bail!("unknown child probe case: {other}"),
    }

    let gate = SessionGate::from_env().context("construct SessionGate from env/settings")?;
    assert_eq!(gate.backend_name(), "valkey");

    let _store = SessionStore::new().context("construct SessionStore from env/settings")?;
    Ok(())
}
