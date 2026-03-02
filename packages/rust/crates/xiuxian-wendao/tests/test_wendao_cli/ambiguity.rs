use super::*;

#[test]
fn test_wendao_metadata_reports_ambiguous_stem_candidates() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("assets/skills/a/SKILL.md"),
        "# Skill A\n\nA.\n",
    )?;
    write_file(
        &tmp.path().join("assets/skills/b/SKILL.md"),
        "# Skill B\n\nB.\n",
    )?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("metadata")
        .arg("SKILL")
        .output()?;

    assert!(
        output.status.success(),
        "wendao metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    assert_eq!(
        payload.get("error").and_then(Value::as_str),
        Some("ambiguous_stem")
    );
    assert_eq!(payload.get("query").and_then(Value::as_str), Some("SKILL"));
    assert_eq!(payload.get("count").and_then(Value::as_u64), Some(2));
    let candidates = payload
        .get("candidates")
        .and_then(Value::as_array)
        .ok_or("missing candidates array")?;
    assert_eq!(candidates.len(), 2);
    assert!(
        candidates
            .iter()
            .any(|row| row.get("path").and_then(Value::as_str) == Some("assets/skills/a/SKILL.md"))
    );
    assert!(
        candidates
            .iter()
            .any(|row| row.get("path").and_then(Value::as_str) == Some("assets/skills/b/SKILL.md"))
    );
    Ok(())
}

#[test]
fn test_wendao_resolve_returns_candidates() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("assets/skills/a/SKILL.md"),
        "# Skill A\n\nA.\n",
    )?;
    write_file(
        &tmp.path().join("assets/skills/b/SKILL.md"),
        "# Skill B\n\nB.\n",
    )?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("resolve")
        .arg("SKILL")
        .arg("--limit")
        .arg("10")
        .output()?;

    assert!(
        output.status.success(),
        "wendao resolve failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    assert_eq!(payload.get("query").and_then(Value::as_str), Some("SKILL"));
    assert_eq!(payload.get("count").and_then(Value::as_u64), Some(2));
    assert_eq!(
        payload.get("returned_count").and_then(Value::as_u64),
        Some(2)
    );
    let candidates = payload
        .get("candidates")
        .and_then(Value::as_array)
        .ok_or("missing candidates array")?;
    assert_eq!(candidates.len(), 2);
    Ok(())
}

#[test]
fn test_wendao_neighbors_reports_ambiguous_stem_candidates()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("assets/skills/a/SKILL.md"),
        "# Skill A\n\n[[other]]\n",
    )?;
    write_file(
        &tmp.path().join("assets/skills/b/SKILL.md"),
        "# Skill B\n\n[[other]]\n",
    )?;
    write_file(&tmp.path().join("assets/skills/other.md"), "# Other\n")?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("neighbors")
        .arg("SKILL")
        .arg("--limit")
        .arg("5")
        .output()?;

    assert!(
        output.status.success(),
        "wendao neighbors failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    assert_eq!(
        payload.get("error").and_then(Value::as_str),
        Some("ambiguous_stem")
    );
    assert_eq!(
        payload.get("command").and_then(Value::as_str),
        Some("neighbors")
    );
    assert_eq!(payload.get("query").and_then(Value::as_str), Some("SKILL"));
    assert_eq!(payload.get("count").and_then(Value::as_u64), Some(2));
    Ok(())
}

#[test]
fn test_wendao_related_reports_ambiguous_stem_candidates() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("assets/skills/a/SKILL.md"),
        "# Skill A\n\n[[other]]\n",
    )?;
    write_file(
        &tmp.path().join("assets/skills/b/SKILL.md"),
        "# Skill B\n\n[[other]]\n",
    )?;
    write_file(&tmp.path().join("assets/skills/other.md"), "# Other\n")?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("related")
        .arg("SKILL")
        .arg("--max-distance")
        .arg("2")
        .arg("--limit")
        .arg("5")
        .output()?;

    assert!(
        output.status.success(),
        "wendao related failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    assert_eq!(
        payload.get("error").and_then(Value::as_str),
        Some("ambiguous_stem")
    );
    assert_eq!(
        payload.get("command").and_then(Value::as_str),
        Some("related")
    );
    assert_eq!(payload.get("query").and_then(Value::as_str), Some("SKILL"));
    assert_eq!(payload.get("count").and_then(Value::as_u64), Some(2));
    Ok(())
}
