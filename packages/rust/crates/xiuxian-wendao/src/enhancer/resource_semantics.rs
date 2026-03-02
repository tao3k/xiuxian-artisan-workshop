use std::path::Path;

use crate::{EntityType, RelationType};

/// Classified semantic result for one skill reference edge.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillReferenceSemantics {
    /// Relation type to emit from skill node to reference node.
    pub relation: RelationType,
    /// Entity type to assign to the reference node.
    pub entity: EntityType,
    /// Normalized semantic type-hint when available.
    pub reference_type: Option<String>,
}

/// Classifies one skill reference using type hints and path extension.
#[must_use]
pub fn classify_skill_reference(
    explicit_reference_type: Option<&str>,
    config_type: Option<&str>,
    entity_path: &str,
) -> SkillReferenceSemantics {
    let normalized_reference_type =
        normalize_reference_type(explicit_reference_type, config_type, entity_path);

    let relation_type = match normalized_reference_type.as_deref() {
        Some("persona") => RelationType::Manifests,
        Some("qianji-flow" | "tool") => RelationType::Governs,
        Some("attachment") => RelationType::AttachedTo,
        _ => RelationType::References,
    };

    let entity_type = match normalized_reference_type.as_deref() {
        Some("template") => EntityType::Other("Template".to_string()),
        Some("persona") => EntityType::Other("Persona".to_string()),
        Some("qianji-flow") => EntityType::Other("QianjiFlow".to_string()),
        Some("attachment") => EntityType::Other("Attachment".to_string()),
        Some("tool") => EntityType::Tool,
        Some("api") => EntityType::Api,
        Some("document") => EntityType::Document,
        Some("code") => EntityType::Code,
        _ => infer_entity_type_from_extension(entity_path),
    };

    SkillReferenceSemantics {
        relation: relation_type,
        entity: entity_type,
        reference_type: normalized_reference_type,
    }
}

fn normalize_reference_type(
    explicit_reference_type: Option<&str>,
    config_type: Option<&str>,
    entity_path: &str,
) -> Option<String> {
    explicit_reference_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| config_type.map(str::trim).filter(|value| !value.is_empty()))
        .map(str::to_ascii_lowercase)
        .or_else(|| infer_attachment_type(entity_path))
}

fn infer_entity_type_from_extension(entity_path: &str) -> EntityType {
    let extension = Path::new(entity_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("j2") => EntityType::Other("Template".to_string()),
        Some("toml") => EntityType::Other("Persona".to_string()),
        Some("md" | "markdown") => EntityType::Document,
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "pdf") => {
            EntityType::Other("Attachment".to_string())
        }
        _ => EntityType::Concept,
    }
}

fn infer_attachment_type(entity_path: &str) -> Option<String> {
    let extension = Path::new(entity_path)
        .extension()
        .and_then(|value| value.to_str())?;
    if matches!(
        extension.trim().to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "pdf"
    ) {
        return Some("attachment".to_string());
    }
    None
}
