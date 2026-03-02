//! URI parser contract tests for semantic `wendao://` skill references.

use xiuxian_wendao::{SkillVfsError, WendaoResourceUri};

#[test]
fn parses_valid_wendao_uri() -> Result<(), Box<dyn std::error::Error>> {
    let uri = WendaoResourceUri::parse(
        "wendao://skills/agenda-management/references/personas/steward.md?rev=1#section",
    )?;
    assert_eq!(uri.semantic_name(), "agenda-management");
    assert_eq!(uri.entity_name(), "personas/steward.md");
    assert_eq!(
        uri.candidate_paths(),
        vec![std::path::PathBuf::from("personas/steward.md")]
    );
    Ok(())
}

#[test]
fn rejects_non_wendao_scheme() {
    assert!(matches!(
        WendaoResourceUri::parse("file://skills/agenda/references/steward"),
        Err(SkillVfsError::UnsupportedScheme { .. })
    ));
}

#[test]
fn rejects_entity_path_traversal() {
    assert!(matches!(
        WendaoResourceUri::parse("wendao://skills/agenda/references/../secrets"),
        Err(SkillVfsError::InvalidEntityPath { .. })
    ));
}

#[test]
fn rejects_entity_without_extension() {
    assert!(matches!(
        WendaoResourceUri::parse("wendao://skills/agenda/references/steward"),
        Err(SkillVfsError::MissingEntityExtension { .. })
    ));
}

#[test]
fn parses_entity_with_explicit_extension() -> Result<(), Box<dyn std::error::Error>> {
    let uri = WendaoResourceUri::parse("wendao://skills/agenda/references/steward.md")?;
    assert_eq!(
        uri.candidate_paths(),
        vec![std::path::PathBuf::from("steward.md")]
    );
    Ok(())
}
