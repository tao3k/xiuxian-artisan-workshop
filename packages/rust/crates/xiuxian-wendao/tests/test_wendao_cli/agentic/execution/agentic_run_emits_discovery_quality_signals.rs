use super::*;

#[test]
fn test_wendao_agentic_run_emits_discovery_quality_signals()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nalpha momentum\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nalpha breakout\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\nbeta divergence\n")?;

    let prefix = unique_agentic_prefix();
    if clear_valkey_prefix(&prefix).is_err() {
        return Ok(());
    }

    let config_path = tmp.path().join("wendao.yaml");
    write_agentic_execution_config(&config_path, &prefix, 3, 2)?;

    let run_payload = run_agentic_run_persist(
        tmp.path(),
        &config_path,
        "alpha",
        "wendao agentic run failed",
    )?;
    assert!(
        run_payload
            .get("persisted_proposals")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            >= 1
    );
    assert_eq!(
        run_payload.get("failed_proposals").and_then(Value::as_u64),
        Some(0)
    );

    let rows = run_agentic_recent_json(
        &config_path,
        "provisional",
        10,
        "wendao agentic recent failed",
    )?;
    let rows = rows.as_array().ok_or("recent payload must be array")?;
    assert!(!rows.is_empty());
    for row in rows {
        let source_id = row
            .get("source_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let target_id = row
            .get("target_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let relation = row
            .get("relation")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let evidence = row
            .get("evidence")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let confidence = row
            .get("confidence")
            .and_then(Value::as_f64)
            .ok_or("missing confidence")?;
        let agent_id = row
            .get("agent_id")
            .and_then(Value::as_str)
            .unwrap_or_default();

        assert!(!source_id.is_empty());
        assert!(!target_id.is_empty());
        assert_ne!(
            source_id, target_id,
            "unexpected self-loop proposal row={row}"
        );
        assert_eq!(relation, "related_to");
        assert_eq!(agent_id, "qianhuan-architect");
        assert!((0.0..=1.0).contains(&confidence));
        assert!(
            evidence.contains("agentic expansion bridge candidate"),
            "missing evidence prefix in proposal row={row}"
        );
        assert!(
            evidence.contains("query=alpha"),
            "missing query anchor in proposal evidence row={row}"
        );
    }

    clear_valkey_prefix(&prefix)?;
    Ok(())
}
