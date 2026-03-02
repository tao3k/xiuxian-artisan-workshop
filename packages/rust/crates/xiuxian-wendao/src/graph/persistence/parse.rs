use crate::entity::{Entity, EntityType, Relation, RelationType};
use serde_json::Value;

/// Create an Entity from a JSON dict.
#[must_use]
pub fn entity_from_dict(data: &Value) -> Option<Entity> {
    let name = data.get("name")?.as_str()?.to_string();
    let entity_type = parse_entity_type_str(data.get("entity_type")?.as_str()?);
    let description = data
        .get("description")
        .map(|value| value.as_str().unwrap_or("").to_string())
        .unwrap_or_default();

    let id = format!(
        "{}:{}",
        entity_type.to_string().to_lowercase(),
        name.to_lowercase().replace(' ', "_")
    );

    let entity = Entity::new(id, name, entity_type, description)
        .with_source(
            data.get("source")
                .and_then(|value| value.as_str().map(str::to_string)),
        )
        .with_aliases(
            data.get("aliases")
                .and_then(|value| {
                    value.as_array().map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect()
                    })
                })
                .unwrap_or_default(),
        )
        .with_confidence(
            data.get("confidence")
                .and_then(|value| serde_json::from_value::<f32>(value.clone()).ok())
                .unwrap_or(1.0),
        );

    Some(entity)
}

/// Create a Relation from a JSON dict.
#[must_use]
pub fn relation_from_dict(data: &Value) -> Option<Relation> {
    let source = data.get("source")?.as_str()?.to_string();
    let target = data.get("target")?.as_str()?.to_string();
    let relation_type = parse_relation_type_str(data.get("relation_type")?.as_str()?);
    let description = data
        .get("description")
        .map(|value| value.as_str().unwrap_or("").to_string())
        .unwrap_or_default();

    let relation = Relation::new(source, target, relation_type, description)
        .with_source_doc(
            data.get("source_doc")
                .and_then(|value| value.as_str().map(str::to_string)),
        )
        .with_confidence(
            data.get("confidence")
                .and_then(|value| serde_json::from_value::<f32>(value.clone()).ok())
                .unwrap_or(1.0),
        );

    Some(relation)
}

pub(crate) fn parse_entity_type_str(raw: &str) -> EntityType {
    match raw.to_uppercase().as_str() {
        "PERSON" => EntityType::Person,
        "ORGANIZATION" => EntityType::Organization,
        "CONCEPT" => EntityType::Concept,
        "PROJECT" => EntityType::Project,
        "TOOL" => EntityType::Tool,
        "SKILL" => EntityType::Skill,
        "LOCATION" => EntityType::Location,
        "EVENT" => EntityType::Event,
        "DOCUMENT" => EntityType::Document,
        "CODE" => EntityType::Code,
        "API" => EntityType::Api,
        "ERROR" => EntityType::Error,
        "PATTERN" => EntityType::Pattern,
        _ => EntityType::Other(raw.to_string()),
    }
}

pub(crate) fn parse_relation_type_str(raw: &str) -> RelationType {
    match raw.to_uppercase().as_str() {
        "WORKS_FOR" => RelationType::WorksFor,
        "PART_OF" => RelationType::PartOf,
        "USES" => RelationType::Uses,
        "DEPENDS_ON" => RelationType::DependsOn,
        "SIMILAR_TO" => RelationType::SimilarTo,
        "LOCATED_IN" => RelationType::LocatedIn,
        "CREATED_BY" => RelationType::CreatedBy,
        "DOCUMENTED_IN" => RelationType::DocumentedIn,
        "RELATED_TO" => RelationType::RelatedTo,
        "IMPLEMENTS" => RelationType::Implements,
        "EXTENDS" => RelationType::Extends,
        "CONTAINS" => RelationType::Contains,
        "REFERENCES" => RelationType::References,
        "GOVERNS" => RelationType::Governs,
        "MANIFESTS" => RelationType::Manifests,
        "ATTACHED_TO" => RelationType::AttachedTo,
        _ => RelationType::Other(raw.to_string()),
    }
}
