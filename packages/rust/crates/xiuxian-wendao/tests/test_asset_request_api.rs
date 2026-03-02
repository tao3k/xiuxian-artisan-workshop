//! Tests for Wendao skill-only semantic asset request API.

use xiuxian_wendao::{SkillVfsError, WendaoAssetHandle};

#[test]
fn skill_reference_asset_builds_canonical_uri() -> Result<(), Box<dyn std::error::Error>> {
    let request = WendaoAssetHandle::skill_reference_asset("agenda-management", "teacher.md")?;
    assert_eq!(
        request.uri(),
        "wendao://skills/agenda-management/references/teacher.md"
    );
    Ok(())
}

#[test]
fn skill_reference_asset_rejects_parent_traversal() {
    let error = match WendaoAssetHandle::skill_reference_asset("agenda-management", "../teacher.md")
    {
        Ok(_request) => panic!("parent traversal should be rejected"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        SkillVfsError::InvalidRelativeAssetPath { .. }
    ));
}

#[test]
fn asset_request_can_read_stripped_body_with_callback() -> Result<(), Box<dyn std::error::Error>> {
    let request = WendaoAssetHandle::skill_reference_asset("agenda-management", "rules.md")?;
    let text = request.read_stripped_body_with(|uri| {
        if uri == "wendao://skills/agenda-management/references/rules.md" {
            Some("  content  \n".to_string())
        } else {
            None
        }
    })?;
    assert_eq!(text, "content");
    Ok(())
}

#[test]
fn asset_request_can_read_stripped_body_with_shared_callback()
-> Result<(), Box<dyn std::error::Error>> {
    let request = WendaoAssetHandle::skill_reference_asset("agenda-management", "rules.md")?;
    let text = request.read_stripped_body_with_shared(|uri| {
        if uri == "wendao://skills/agenda-management/references/rules.md" {
            Some("  content  \n".to_string())
        } else {
            None
        }
    })?;
    assert_eq!(text.as_ref(), "content");
    Ok(())
}

#[test]
fn skill_reference_asset_reads_embedded_body_via_builtin_resolver()
-> Result<(), Box<dyn std::error::Error>> {
    let request = WendaoAssetHandle::skill_reference_asset("agenda-management", "teacher.md")?;
    let text = request.read_stripped_body()?;
    assert!(text.contains("Deep Thinking Professor"));
    Ok(())
}

#[test]
fn skill_reference_asset_shared_body_uses_cache() -> Result<(), Box<dyn std::error::Error>> {
    let request = WendaoAssetHandle::skill_reference_asset("agenda-management", "teacher.md")?;
    let first = request.read_stripped_body_shared()?;
    let second = request.read_stripped_body_shared()?;
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    Ok(())
}
