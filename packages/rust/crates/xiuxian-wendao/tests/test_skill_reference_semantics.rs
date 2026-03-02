//! Contract tests for skill reference semantic classification.

use xiuxian_wendao::{EntityType, RelationType, classify_skill_reference};

#[test]
fn classify_persona_reference_uses_manifests() {
    let semantics = classify_skill_reference(Some("persona"), None, "steward.md");
    assert_eq!(semantics.relation, RelationType::Manifests);
    assert_eq!(semantics.entity, EntityType::Other("Persona".to_string()));
    assert_eq!(semantics.reference_type.as_deref(), Some("persona"));
}

#[test]
fn classify_qianji_flow_reference_uses_governs() {
    let semantics = classify_skill_reference(Some("qianji-flow"), None, "flow.toml");
    assert_eq!(semantics.relation, RelationType::Governs);
    assert_eq!(
        semantics.entity,
        EntityType::Other("QianjiFlow".to_string())
    );
}

#[test]
fn classify_attachment_by_extension_without_hint() {
    let semantics = classify_skill_reference(None, None, "logo.png");
    assert_eq!(semantics.relation, RelationType::AttachedTo);
    assert_eq!(
        semantics.entity,
        EntityType::Other("Attachment".to_string())
    );
    assert_eq!(semantics.reference_type.as_deref(), Some("attachment"));
}
