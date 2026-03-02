use crate::entity::{EntityType, RelationType};

/// Parse entity type from string.
pub(crate) fn parse_entity_type(s: &str) -> EntityType {
    match s.to_uppercase().as_str() {
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
        _ => EntityType::Other(s.to_string()),
    }
}

/// Parse relation type from string.
pub(crate) fn parse_relation_type(s: &str) -> RelationType {
    match s.to_uppercase().as_str() {
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
        _ => RelationType::Other(s.to_string()),
    }
}
