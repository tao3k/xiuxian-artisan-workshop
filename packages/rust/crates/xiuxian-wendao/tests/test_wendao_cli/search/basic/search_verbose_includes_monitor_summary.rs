use super::*;

#[test]
fn test_wendao_search_verbose_includes_monitor_summary() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# Alpha\n\nbeta phrase.\n")?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "# Beta\n\nbeta phrase again.\n",
    )?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg("beta")
        .arg("--limit")
        .arg("5")
        .arg("--verbose")
        .output()?;

    assert!(
        output.status.success(),
        "wendao search --verbose failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    let phases = payload
        .get("phases")
        .and_then(Value::as_array)
        .ok_or("missing phases")?;
    assert!(phases.iter().any(|row| {
        row.get("phase").and_then(Value::as_str) == Some("link_graph.search.plan_parse")
    }));
    assert!(phases.iter().any(|row| {
        row.get("phase").and_then(Value::as_str) == Some("link_graph.search.execute")
    }));
    assert!(phases.iter().any(|row| {
        row.get("phase").and_then(Value::as_str) == Some("link_graph.search.policy")
    }));
    assert!(phases.iter().any(|row| {
        row.get("phase").and_then(Value::as_str) == Some("link_graph.search.policy")
            && row
                .get("extra")
                .and_then(|extra| extra.get("reason_validated"))
                .and_then(Value::as_bool)
                == Some(true)
    }));
    assert!(phases.iter().any(|row| {
        row.get("phase").and_then(Value::as_str) == Some("link_graph.overlay.promoted")
    }));
    assert!(payload.get("requested_mode").is_some());
    assert!(payload.get("selected_mode").is_some());
    assert!(payload.get("reason").is_some());
    assert!(payload.get("graph_confidence_score").is_some());
    assert!(payload.get("graph_confidence_level").is_some());
    assert!(payload.get("retrieval_plan").is_some());
    assert!(
        payload
            .get("monitor")
            .and_then(|row| row.get("bottlenecks"))
            .and_then(|row| row.get("slowest_phase"))
            .is_some()
    );
    Ok(())
}
