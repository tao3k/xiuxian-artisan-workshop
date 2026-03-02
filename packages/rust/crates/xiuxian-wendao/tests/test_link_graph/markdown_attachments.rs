use super::*;

#[test]
fn test_link_graph_extracts_markdown_links_relative_and_anchor()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# A\n\n[B](b.md)\n[C](sub/c.md#section)\n[External](https://example.com)\n",
    )?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nNo links.\n")?;
    write_file(
        &tmp.path().join("docs/sub/c.md"),
        "# C\n\n[A](../a.md)\n[Up](#top)\n",
    )?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let stats = index.stats();
    assert_eq!(stats.total_notes, 3);
    assert_eq!(stats.links_in_graph, 3);

    let neighbors = index.neighbors("a", LinkGraphDirection::Both, 1, 10);
    let stems: Vec<String> = neighbors.into_iter().map(|row| row.stem).collect();
    assert!(stems.contains(&"b".to_string()));
    assert!(stems.contains(&"c".to_string()));
    Ok(())
}

#[test]
fn test_link_graph_extracts_markdown_reference_links() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# A\n\n[B][b-ref]\n[C][]\n[D][missing]\n\n[b-ref]: b.md \"Beta\"\n[C]: sub/c.md#top\n",
    )?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nNo links.\n")?;
    write_file(&tmp.path().join("docs/sub/c.md"), "# C\n\nNo links.\n")?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let stats = index.stats();
    assert_eq!(stats.total_notes, 3);
    assert_eq!(stats.links_in_graph, 2);

    let neighbors = index.neighbors("a", LinkGraphDirection::Both, 1, 10);
    let stems: Vec<String> = neighbors.into_iter().map(|row| row.stem).collect();
    assert!(stems.contains(&"b".to_string()));
    assert!(stems.contains(&"c".to_string()));
    Ok(())
}

#[test]
fn test_link_graph_uses_comrak_for_complex_markdown_links() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# A\n\n[Paren](b(1).md)\n\n`[Nope](c.md)`\n\n```md\n[AlsoNope](c.md)\n```\n",
    )?;
    write_file(&tmp.path().join("docs/b(1).md"), "# B\n\nNo links.\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\nNo links.\n")?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let stats = index.stats();
    assert_eq!(stats.total_notes, 3);
    assert_eq!(stats.links_in_graph, 1);

    let neighbors = index.neighbors("a", LinkGraphDirection::Both, 1, 10);
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].stem, "b(1)");
    Ok(())
}

#[test]
fn test_link_graph_ignores_attachment_links_and_inline_embedded_wikilinks()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# A\n\n\
Inline embed ![[b]] must be ignored.\n\n\
Warning! [[c]] must remain a normal wikilink.\n\n\
![Image](assets/pic.png)\n\
[PDF](files/manual.pdf)\n\
[Absolute Attachment](/tmp/manual.pdf)\n\
[Attachment URI Alt](file:/tmp/manual.pdf)\n\
[Attachment URI](file:///tmp/manual.pdf)\n",
    )?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nNo links.\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\nNo links.\n")?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let stats = index.stats();
    assert_eq!(stats.total_notes, 3);
    assert_eq!(stats.links_in_graph, 1);

    let neighbors = index.neighbors("a", LinkGraphDirection::Both, 1, 10);
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].stem, "c");
    Ok(())
}

#[test]
fn test_link_graph_attachment_search_filters_by_kind_and_extension()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# A\n\n[Paper](files/paper.pdf)\n![Image](assets/pic.png)\n[Key](keys/signing.gpg)\n",
    )?;
    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;

    let image_hits =
        index.search_attachments("", 20, &[], &[LinkGraphAttachmentKind::Image], false);
    assert!(!image_hits.is_empty());
    assert!(
        image_hits
            .iter()
            .all(|row| row.kind == LinkGraphAttachmentKind::Image)
    );
    assert!(image_hits.iter().any(|row| row.attachment_ext == "png"));

    let pdf_hits = index.search_attachments("", 20, &["pdf".to_string()], &[], false);
    assert_eq!(pdf_hits.len(), 1);
    assert_eq!(pdf_hits[0].kind, LinkGraphAttachmentKind::Pdf);
    assert_eq!(pdf_hits[0].attachment_ext, "pdf");
    Ok(())
}
