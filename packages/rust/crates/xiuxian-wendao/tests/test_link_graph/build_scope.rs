use super::*;

#[test]
fn test_link_graph_build_with_excluded_dirs_skips_cache_tree()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# Alpha\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# Beta\n\n[[a]]\n")?;
    write_file(
        &tmp.path().join(".cache/huge.md"),
        "# Should Be Skipped\n\n[[docs/a]]\n",
    )?;

    let excluded = vec![".cache".to_string()];
    let index =
        LinkGraphIndex::build_with_excluded_dirs(tmp.path(), &excluded).map_err(|e| e.clone())?;

    let stats = index.stats();
    assert_eq!(stats.total_notes, 2);
    assert_eq!(stats.links_in_graph, 2);
    assert_eq!(stats.orphans, 0);

    let toc_paths: Vec<String> = index.toc(10).into_iter().map(|row| row.path).collect();
    assert!(!toc_paths.iter().any(|path| path.contains(".cache/")));
    Ok(())
}

#[test]
fn test_link_graph_build_skips_hidden_dirs_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# Alpha\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# Beta\n\n[[a]]\n")?;
    write_file(
        &tmp.path().join(".github/hidden.md"),
        "# Hidden\n\n[[docs/a]]\n",
    )?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let stats = index.stats();
    assert_eq!(stats.total_notes, 2);
    assert_eq!(stats.links_in_graph, 2);

    let toc_paths: Vec<String> = index.toc(10).into_iter().map(|row| row.path).collect();
    assert!(!toc_paths.iter().any(|path| path.starts_with(".github/")));
    Ok(())
}

#[test]
fn test_link_graph_build_with_include_dirs_limits_scope() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# Alpha\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# Beta\n\n[[a]]\n")?;
    write_file(
        &tmp.path().join("assets/knowledge/c.md"),
        "# Gamma\n\n[[docs/a]]\n",
    )?;

    let include = vec!["docs".to_string()];
    let index =
        LinkGraphIndex::build_with_filters(tmp.path(), &include, &[]).map_err(|e| e.clone())?;

    let stats = index.stats();
    assert_eq!(stats.total_notes, 2);
    assert_eq!(stats.links_in_graph, 2);
    assert_eq!(stats.orphans, 0);

    let toc_paths: Vec<String> = index.toc(10).into_iter().map(|row| row.path).collect();
    assert!(toc_paths.iter().all(|path| path.starts_with("docs/")));
    Ok(())
}

#[test]
fn test_link_graph_build_promotes_skill_metadata_into_skill_doc_tags()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("skills/demo/SKILL.md"),
        r#"---
name: demo-skill
description: Demo skill for promotion testing.
metadata:
  routing_keywords:
    - "task planning"
  intents:
    - "Draft a plan"
---

# Demo Skill
"#,
    )?;
    write_file(
        &tmp.path().join("skills/demo/references/rules.md"),
        r#"---
type: knowledge
metadata:
  title: "Rules"
---
# Rules

Follow observable constraints.
"#,
    )?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let skill_docs = index
        .toc(10)
        .into_iter()
        .filter(|doc| doc.path == "skills/demo/SKILL.md")
        .collect::<Vec<_>>();
    assert_eq!(
        skill_docs.len(),
        1,
        "expected one promoted skill descriptor"
    );
    let Some(skill_doc) = skill_docs.first() else {
        return Err(std::io::Error::other("missing promoted skill descriptor").into());
    };
    assert_eq!(skill_doc.doc_type.as_deref(), Some("skill"));
    assert!(
        skill_doc.tags.iter().any(|tag| tag == "skill"),
        "skill descriptor should carry generic skill tag"
    );
    assert!(
        skill_doc.tags.iter().any(|tag| tag == "skill:demo-skill"),
        "skill descriptor should carry semantic skill tag"
    );
    assert!(
        skill_doc
            .tags
            .iter()
            .any(|tag| tag == "routing:task-planning"),
        "routing keywords should be promoted into normalized tags"
    );
    assert!(
        skill_doc
            .tags
            .iter()
            .any(|tag| tag == "intent:draft-a-plan"),
        "intents should be promoted into normalized tags"
    );
    Ok(())
}
