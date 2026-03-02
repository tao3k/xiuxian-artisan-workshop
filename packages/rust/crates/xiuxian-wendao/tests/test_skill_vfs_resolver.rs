//! Resolver contract tests for `wendao://skills/.../references/...`.

use std::path::Path;
use std::sync::Arc;

use tempfile::TempDir;
use xiuxian_wendao::{SkillVfsError, SkillVfsResolver};

const SKILL_FRONTMATTER: &str = r#"---
name: agenda-management
description: "Agenda skill"
---

# Agenda Skill
"#;

#[test]
fn resolves_reference_from_semantic_uri() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let root = temp.path().join("internal");
    let skill_dir = root.join("agenda_skill");
    std::fs::create_dir_all(skill_dir.join("references"))?;
    std::fs::write(skill_dir.join("SKILL.md"), SKILL_FRONTMATTER)?;
    std::fs::write(
        skill_dir.join("references").join("steward.md"),
        build_reference_doc("persona: strict-teacher"),
    )?;

    let resolver = SkillVfsResolver::from_roots(&[root])?;
    let content = resolver.read_utf8("wendao://skills/agenda-management/references/steward.md")?;
    assert!(content.contains("persona: strict-teacher"));
    Ok(())
}

#[test]
fn supports_overlay_precedence_by_root_order() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let internal = temp.path().join("internal");
    let user = temp.path().join("user");
    write_skill(
        internal.as_path(),
        "agenda_skill",
        "steward.md",
        "source = internal",
    )?;
    write_skill(
        user.as_path(),
        "agenda_skill",
        "steward.md",
        "source = user",
    )?;

    let resolver = SkillVfsResolver::from_roots(&[user.clone(), internal.clone()])?;
    let path = resolver.resolve_path("wendao://skills/agenda-management/references/steward.md")?;
    assert!(path.starts_with(user.as_path()));
    Ok(())
}

#[test]
fn returns_not_found_for_missing_entity() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let root = temp.path().join("internal");
    write_skill(
        root.as_path(),
        "agenda_skill",
        "steward.md",
        "source = internal",
    )?;

    let resolver = SkillVfsResolver::from_roots(&[root])?;
    assert!(matches!(
        resolver.resolve_path("wendao://skills/agenda-management/references/teacher.md"),
        Err(SkillVfsError::ResourceNotFound { .. })
    ));
    Ok(())
}

#[test]
fn embedded_reference_requires_explicit_mount() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = SkillVfsResolver::from_roots(&[])?;
    assert!(matches!(
        resolver.read_utf8("wendao://skills/agenda-management/references/steward.md"),
        Err(SkillVfsError::UnknownSemanticSkill { .. })
    ));
    Ok(())
}

#[test]
fn resolves_embedded_reference_when_embedded_mount_enabled()
-> Result<(), Box<dyn std::error::Error>> {
    let resolver = SkillVfsResolver::from_roots(&[])?;
    let resolver = resolver.mount_embedded_dir();
    let content = resolver.read_utf8("wendao://skills/agenda-management/references/steward.md")?;
    assert!(content.contains("Pragmatic Agenda Steward"));
    Ok(())
}

#[test]
fn read_utf8_shared_reuses_cached_embedded_arc() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = SkillVfsResolver::from_roots(&[])?;
    let resolver = resolver.mount_embedded_dir();
    let uri = "wendao://skills/agenda-management/references/steward.md";
    let first = resolver.read_utf8_shared(uri)?;
    let second = resolver.read_utf8_shared(uri)?;
    assert!(Arc::ptr_eq(&first, &second));
    Ok(())
}

#[test]
fn read_semantic_alias_reuses_shared_arc() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = SkillVfsResolver::from_roots(&[])?.mount_embedded_dir();
    let uri = "wendao://skills/agenda-management/references/steward.md";
    let first = resolver.read_semantic(uri)?;
    let second = resolver.read_utf8_shared(uri)?;
    assert!(Arc::ptr_eq(&first, &second));
    Ok(())
}

#[test]
fn read_utf8_shared_reuses_cached_local_arc() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let root = temp.path().join("internal");
    write_skill(
        root.as_path(),
        "agenda_skill",
        "steward.md",
        "source = local",
    )?;

    let resolver = SkillVfsResolver::from_roots(&[root])?;
    let uri = "wendao://skills/agenda-management/references/steward.md";
    let first = resolver.read_utf8_shared(uri)?;
    let second = resolver.read_utf8_shared(uri)?;
    assert!(Arc::ptr_eq(&first, &second));
    Ok(())
}

fn write_skill(
    root: &Path,
    folder: &str,
    entity_file: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let skill_dir = root.join(folder);
    std::fs::create_dir_all(skill_dir.join("references"))?;
    std::fs::write(skill_dir.join("SKILL.md"), SKILL_FRONTMATTER)?;
    std::fs::write(
        skill_dir.join("references").join(entity_file),
        build_reference_doc(content),
    )?;
    Ok(())
}

fn build_reference_doc(body: &str) -> String {
    format!(
        r#"---
type: knowledge
metadata:
  title: "Fixture Reference"
---
{}
"#,
        body
    )
}
