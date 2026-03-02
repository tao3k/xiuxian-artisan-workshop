//! Integration tests for the `wendao` CLI binary.

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

static AGENTIC_PREFIX_COUNTER: AtomicU64 = AtomicU64::new(0);

fn write_file(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn wendao_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wendao"));
    cmd.env("VALKEY_URL", "redis://127.0.0.1:6379/0");
    cmd
}

fn unique_agentic_prefix() -> String {
    let seq = AGENTIC_PREFIX_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    format!("xiuxian_wendao:test:wendao_cli:agentic:{pid}:{nanos}:{seq}")
}

fn clear_valkey_prefix(prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open("redis://127.0.0.1:6379/0")?;
    let mut conn = client.get_connection()?;
    let pattern = format!("{prefix}:*");
    let keys: Vec<String> = redis::cmd("KEYS").arg(&pattern).query(&mut conn)?;
    if !keys.is_empty() {
        redis::cmd("DEL").arg(keys).query::<()>(&mut conn)?;
    }
    Ok(())
}

mod agentic;
mod ambiguity;
mod attachments;
mod cli_commands;
mod related;
mod search;
