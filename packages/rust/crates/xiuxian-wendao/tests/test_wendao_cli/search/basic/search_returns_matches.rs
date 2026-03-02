use super::*;

#[test]
fn test_wendao_search_returns_matches() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("notes/alpha.md"),
        "# Alpha Note\n\nReference [[beta]].\n",
    )?;
    write_file(
        &tmp.path().join("notes/beta.md"),
        "---\ntitle: Beta Knowledge\ntags:\n  - rust\n---\n\nReference [[alpha]].\n",
    )?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg("beta")
        .arg("--limit")
        .arg("5")
        .output()?;

    assert!(
        output.status.success(),
        "wendao search failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout)?;
    let payload: Value = serde_json::from_str(&stdout)?;
    assert_eq!(payload.get("query").and_then(Value::as_str), Some("beta"));
    assert_eq!(payload.get("limit").and_then(Value::as_u64), Some(5));
    assert_eq!(
        payload.get("match_strategy").and_then(Value::as_str),
        Some("fts")
    );
    let sort_terms = payload
        .get("sort_terms")
        .and_then(Value::as_array)
        .ok_or("missing sort_terms")?;
    assert_eq!(sort_terms.len(), 1);
    assert_eq!(
        sort_terms[0].get("field").and_then(Value::as_str),
        Some("score")
    );
    assert_eq!(
        sort_terms[0].get("order").and_then(Value::as_str),
        Some("desc")
    );
    assert_eq!(
        payload.get("case_sensitive").and_then(Value::as_bool),
        Some(false)
    );
    let filters = payload.get("filters").ok_or("missing filters payload")?;
    assert_eq!(
        filters
            .get("link_to")
            .and_then(|row| row.get("seeds"))
            .and_then(Value::as_array)
            .map(std::vec::Vec::len),
        None
    );
    assert_eq!(
        filters
            .get("linked_by")
            .and_then(|row| row.get("seeds"))
            .and_then(Value::as_array)
            .map(std::vec::Vec::len),
        None
    );
    assert_eq!(
        filters
            .get("related")
            .and_then(|row| row.get("seeds"))
            .and_then(Value::as_array)
            .map(std::vec::Vec::len),
        None
    );

    let Some(results) = payload.get("results").and_then(Value::as_array) else {
        return Err("missing search results array".into());
    };
    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .any(|row| row.get("stem").and_then(Value::as_str) == Some("beta")),
        "expected search results to include stem=beta; payload={payload}"
    );

    Ok(())
}
