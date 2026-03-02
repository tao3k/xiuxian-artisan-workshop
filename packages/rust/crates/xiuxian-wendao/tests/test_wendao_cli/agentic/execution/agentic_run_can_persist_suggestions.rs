use super::*;

fn run_agentic_persist(
    tmp: &TempDir,
    config_path: &Path,
) -> Result<Value, Box<dyn std::error::Error>> {
    let run_output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("--conf")
        .arg(config_path)
        .arg("agentic")
        .arg("run")
        .arg("--query")
        .arg("alpha")
        .arg("--persist")
        .output()?;
    let stderr = String::from_utf8_lossy(&run_output.stderr);
    assert!(
        run_output.status.success(),
        "wendao agentic run persist failed: {stderr}"
    );
    let run_stdout = String::from_utf8(run_output.stdout)?;
    Ok(serde_json::from_str(&run_stdout)?)
}

#[test]
fn test_wendao_agentic_run_can_persist_suggestions() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nalpha\n")?;

    let prefix = unique_agentic_prefix();
    if clear_valkey_prefix(&prefix).is_err() {
        return Ok(());
    }

    let config_path = tmp.path().join("wendao.yaml");
    fs::write(
        &config_path,
        format!(
            "link_graph:\n  cache:\n    valkey_url: \"redis://127.0.0.1:6379/0\"\n    key_prefix: \"{prefix}\"\n  agentic:\n    suggested_link:\n      max_entries: 64\n      ttl_seconds: null\n    expansion:\n      max_workers: 1\n      max_candidates: 2\n      max_pairs_per_worker: 1\n      time_budget_ms: 1000.0\n    execution:\n      worker_time_budget_ms: 1000.0\n      persist_suggestions_default: true\n      persist_retry_attempts: 2\n      idempotency_scan_limit: 64\n      relation: \"related_to\"\n      agent_id: \"qianhuan-architect\"\n      evidence_prefix: \"agentic expansion bridge candidate\"\n"
        ),
    )?;

    let run_payload = run_agentic_persist(&tmp, &config_path)?;
    let persisted = run_payload
        .get("persisted_proposals")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    assert!(persisted >= 1);
    assert_eq!(
        run_payload.get("failed_proposals").and_then(Value::as_u64),
        Some(0)
    );

    let run_payload_2 = run_agentic_persist(&tmp, &config_path)?;
    assert_eq!(
        run_payload_2
            .get("persisted_proposals")
            .and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        run_payload_2
            .get("skipped_duplicates")
            .and_then(Value::as_u64),
        Some(1)
    );

    let recent_output = wendao_cmd()
        .arg("--conf")
        .arg(&config_path)
        .arg("agentic")
        .arg("recent")
        .arg("--latest")
        .arg("--state")
        .arg("provisional")
        .arg("--limit")
        .arg("10")
        .output()?;
    let recent_stderr = String::from_utf8_lossy(&recent_output.stderr);
    assert!(
        recent_output.status.success(),
        "wendao agentic recent after run failed: {recent_stderr}"
    );
    let recent_stdout = String::from_utf8(recent_output.stdout)?;
    let rows: Value = serde_json::from_str(&recent_stdout)?;
    let rows = rows.as_array().ok_or("recent payload must be array")?;
    assert!(!rows.is_empty());
    assert!(
        rows.iter()
            .all(|row| row.get("promotion_state").and_then(Value::as_str) == Some("provisional"))
    );

    clear_valkey_prefix(&prefix)?;
    Ok(())
}
