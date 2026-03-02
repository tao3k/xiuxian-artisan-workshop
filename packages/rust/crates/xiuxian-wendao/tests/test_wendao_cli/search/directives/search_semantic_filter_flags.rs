use super::*;

#[test]
fn test_wendao_search_semantic_filter_flags() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "---\ntags:\n  - core\n---\n# A\n\nAlpha signal appears here.\n\n[[b]]\n",
    )?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "---\ntags:\n  - team\n---\n# B\n\nBeta note.\n",
    )?;
    write_file(
        &tmp.path().join("docs/c.md"),
        "# C\n\nAlpha signal appears here too.\n",
    )?;

    let mention_output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg(".md")
        .arg("--limit")
        .arg("10")
        .arg("--mentions-of")
        .arg("Alpha signal")
        .arg("--sort-term")
        .arg("path_asc")
        .output()?;

    assert!(
        mention_output.status.success(),
        "wendao search with mentions-of failed: {}",
        String::from_utf8_lossy(&mention_output.stderr)
    );

    let mention_payload: Value = serde_json::from_str(&String::from_utf8(mention_output.stdout)?)?;
    let mention_rows = mention_payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or("missing mention results")?;
    assert_eq!(mention_rows.len(), 2);
    assert_eq!(
        mention_rows
            .first()
            .and_then(|row| row.get("path"))
            .and_then(Value::as_str),
        Some("docs/a.md")
    );
    assert_eq!(
        mention_rows
            .get(1)
            .and_then(|row| row.get("path"))
            .and_then(Value::as_str),
        Some("docs/c.md")
    );

    let missing_backlink_output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg(".md")
        .arg("--limit")
        .arg("10")
        .arg("--missing-backlink")
        .arg("--sort-term")
        .arg("path_asc")
        .output()?;

    assert!(
        missing_backlink_output.status.success(),
        "wendao search with missing-backlink failed: {}",
        String::from_utf8_lossy(&missing_backlink_output.stderr)
    );

    let missing_backlink_payload: Value =
        serde_json::from_str(&String::from_utf8(missing_backlink_output.stdout)?)?;
    let missing_backlink_rows = missing_backlink_payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or("missing missing-backlink results")?;
    assert_eq!(missing_backlink_rows.len(), 1);
    assert_eq!(
        missing_backlink_rows
            .first()
            .and_then(|row| row.get("path"))
            .and_then(Value::as_str),
        Some("docs/a.md")
    );

    Ok(())
}
