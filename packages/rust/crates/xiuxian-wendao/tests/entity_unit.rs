//! Integration tests for `xiuxian_wendao::entity`.

use xiuxian_wendao::{Entity, EntityType, Relation, RelationType};

#[test]
fn test_entity_creation() {
    let entity = Entity::new(
        "entity-001".to_string(),
        "Claude Code".to_string(),
        EntityType::Tool,
        "AI coding assistant".to_string(),
    );

    assert_eq!(entity.id, "entity-001");
    assert_eq!(entity.name, "Claude Code");
    assert_eq!(entity.entity_type, EntityType::Tool);
    assert!((entity.confidence - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_entity_builder() {
    let entity = Entity::new(
        "entity-002".to_string(),
        "Python".to_string(),
        EntityType::Skill,
        "Programming language".to_string(),
    )
    .with_aliases(vec!["py".to_string(), "python3".to_string()])
    .with_confidence(0.95)
    .with_source(Some("docs/lang.md".to_string()));

    assert_eq!(entity.aliases.len(), 2);
    assert!((entity.confidence - 0.95).abs() < f32::EPSILON);
    assert_eq!(entity.source, Some("docs/lang.md".to_string()));
}

#[test]
fn test_relation_creation() {
    let relation = Relation::new(
        "Claude Code".to_string(),
        "Omni-Dev-Fusion".to_string(),
        RelationType::PartOf,
        "Part of the Omni-Dev-Fusion project".to_string(),
    );

    assert!(relation.id.contains("claude_code"));
    assert!(relation.id.contains("part_of"));
    assert!(relation.id.contains("omni-dev-fusion"));
}

#[test]
fn test_entity_type_display() {
    assert_eq!(EntityType::Person.to_string(), "PERSON");
    assert_eq!(EntityType::Organization.to_string(), "ORGANIZATION");
    assert_eq!(EntityType::Concept.to_string(), "CONCEPT");
}

#[test]
fn test_relation_type_display() {
    assert_eq!(RelationType::WorksFor.to_string(), "WORKS_FOR");
    assert_eq!(RelationType::DependsOn.to_string(), "DEPENDS_ON");
    assert_eq!(RelationType::RelatedTo.to_string(), "RELATED_TO");
    assert_eq!(RelationType::References.to_string(), "REFERENCES");
    assert_eq!(RelationType::Governs.to_string(), "GOVERNS");
    assert_eq!(RelationType::Manifests.to_string(), "MANIFESTS");
    assert_eq!(RelationType::AttachedTo.to_string(), "ATTACHED_TO");
}
