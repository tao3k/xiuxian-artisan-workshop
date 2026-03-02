use super::*;

fn run_attachments_query(
    root: &Path,
    args: &[&str],
    context: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = wendao_cmd()
        .arg("--root")
        .arg(root)
        .arg("attachments")
        .args(args)
        .output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{context}: {stderr}");
    let stdout = String::from_utf8(output.stdout)?;
    Ok(serde_json::from_str(&stdout)?)
}
#[test]
fn test_wendao_attachments_search_filters_by_ext_and_kind() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# Alpha\n\n[Paper](files/paper.pdf)\n![Diagram](assets/diagram.png)\n[Key](security/signing.gpg)\n",
    )?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "# Beta\n\n![Photo](assets/photo.jpg)\n",
    )?;

    let image_payload = run_attachments_query(
        tmp.path(),
        &["--kind", "image", "--limit", "10"],
        "wendao attachments --kind image failed",
    )?;
    let image_hits = image_payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("missing image attachment hits")?;
    assert!(
        image_hits
            .iter()
            .all(|row| row.get("kind").and_then(Value::as_str) == Some("image"))
    );
    assert!(
        image_hits
            .iter()
            .any(|row| { row.get("attachment_ext").and_then(Value::as_str) == Some("png") })
    );
    assert!(
        image_hits
            .iter()
            .any(|row| { row.get("attachment_ext").and_then(Value::as_str) == Some("jpg") })
    );

    let pdf_payload = run_attachments_query(
        tmp.path(),
        &["--ext", "pdf", "--limit", "10"],
        "wendao attachments --ext pdf failed",
    )?;
    let pdf_hits = pdf_payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("missing pdf attachment hits")?;
    assert_eq!(pdf_hits.len(), 1);
    assert_eq!(
        pdf_hits[0].get("attachment_ext").and_then(Value::as_str),
        Some("pdf")
    );
    assert_eq!(pdf_hits[0].get("kind").and_then(Value::as_str), Some("pdf"));
    Ok(())
}

#[test]
fn test_wendao_attachments_search_normalizes_file_scheme_targets()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# Alpha\n\n[Absolute](/tmp/manual.pdf)\n[FileUri](file:///tmp/manual-2.pdf)\n",
    )?;

    let payload = run_attachments_query(
        tmp.path(),
        &["--ext", "pdf", "--limit", "10"],
        "wendao attachments file targets failed",
    )?;
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("missing attachment hits")?;
    assert!(hits.iter().any(|row| {
        row.get("attachment_path").and_then(Value::as_str) == Some("/tmp/manual.pdf")
    }));
    assert!(hits.iter().any(|row| {
        row.get("attachment_path").and_then(Value::as_str) == Some("/tmp/manual-2.pdf")
    }));
    Ok(())
}
