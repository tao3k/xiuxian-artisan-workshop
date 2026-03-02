use crate::skill_vfs::zhixing::{Error, Result};
use crate::{
    Entity, EntityType, Relation, RelationType, WendaoResourceLinkTarget, WendaoResourceUri,
    classify_skill_reference, parse_frontmatter,
};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::Path;

use super::ZhixingWendaoIndexer;

impl ZhixingWendaoIndexer {
    pub(in crate::skill_vfs::zhixing::indexer) fn index_embedded_skill_references(
        &self,
    ) -> Result<(usize, usize)> {
        let registry =
            crate::skill_vfs::zhixing::build_embedded_wendao_registry().map_err(|error| {
                Error::Internal(format!(
                    "failed to build embedded zhixing skill registry for graph indexing: {error}"
                ))
            })?;
        let mut files = registry.files().collect::<Vec<_>>();
        files.sort_by(|left, right| left.path().cmp(right.path()));

        let mut entities_added = 0usize;
        let mut relations_linked = 0usize;

        for file in files {
            if !is_skill_descriptor_path(file.path()) {
                continue;
            }

            let Some(markdown) = crate::skill_vfs::zhixing::embedded_resource_text(file.path())
            else {
                return Err(Error::Internal(format!(
                    "embedded resource `{}` declared in registry but not found in binary",
                    file.path()
                )));
            };

            let frontmatter = parse_frontmatter(markdown);
            let Some(skill_name) = frontmatter
                .name
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_ascii_lowercase)
            else {
                continue;
            };

            if self
                .graph
                .add_entity(build_skill_entity(
                    skill_name.as_str(),
                    file.path(),
                    frontmatter.description.as_deref(),
                    frontmatter.routing_keywords.as_slice(),
                    frontmatter.intents.as_slice(),
                ))
                .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?
            {
                entities_added = entities_added.saturating_add(1);
            }
            let (intent_entities, intent_relations) =
                self.index_skill_intents(skill_name.as_str(), file.path(), &frontmatter.intents)?;
            entities_added = entities_added.saturating_add(intent_entities);
            relations_linked = relations_linked.saturating_add(intent_relations);

            let mut ids = file.link_targets_by_id().iter().collect::<Vec<_>>();
            ids.sort_by(|(left, _), (right, _)| left.cmp(right));

            for (id, targets) in ids {
                let config_type = registry
                    .get(id.as_str())
                    .map(|block| block.config_type.trim().to_ascii_lowercase());
                for target in dedup_targets(targets) {
                    let parsed_uri = WendaoResourceUri::parse(target.target_path.as_str())
                        .map_err(|error| {
                            Error::Internal(format!(
                                "invalid embedded skill link `{}` (id=`{id}` file=`{}`): {error}",
                                target.target_path,
                                file.path()
                            ))
                        })?;

                    let (reference_entity, reference_name) = build_reference_entity(
                        &parsed_uri,
                        file.path(),
                        id.as_str(),
                        target.reference_type.as_deref(),
                        config_type.as_deref(),
                    );

                    if self.graph.add_entity(reference_entity).map_err(|error| {
                        Error::Internal(format!("Graph operation failed: {error}"))
                    })? {
                        entities_added = entities_added.saturating_add(1);
                    }

                    self.graph
                        .add_relation(&build_reference_relation(&ReferenceRelationInput {
                            skill_name: skill_name.as_str(),
                            reference_name: reference_name.as_str(),
                            source_path: file.path(),
                            reference_id: id.as_str(),
                            reference_path: parsed_uri.entity_name(),
                            target_uri: target.target_path.as_str(),
                            explicit_reference_type: target.reference_type.as_deref(),
                            config_type: config_type.as_deref(),
                        }))
                        .map_err(|error| {
                            Error::Internal(format!("Graph operation failed: {error}"))
                        })?;
                    relations_linked = relations_linked.saturating_add(1);
                }
            }
        }

        Ok((entities_added, relations_linked))
    }
}

fn is_skill_descriptor_path(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name == "SKILL.md")
}

fn build_skill_entity(
    skill_name: &str,
    source_path: &str,
    description: Option<&str>,
    routing_keywords: &[String],
    intents: &[String],
) -> Entity {
    let summary = description
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(
            || format!("Skill descriptor for `{skill_name}`"),
            ToString::to_string,
        );
    let mut entity = Entity::new(
        format!("zhixing:skill:{skill_name}"),
        skill_name.to_string(),
        EntityType::Skill,
        summary,
    );
    entity.source = Some(source_path.to_string());
    entity
        .metadata
        .insert("zhixing_domain".to_string(), json!("skill"));
    entity
        .metadata
        .insert("skill_semantic_name".to_string(), json!(skill_name));
    entity
        .metadata
        .insert("source_skill_doc".to_string(), json!(source_path));
    if !routing_keywords.is_empty() {
        entity.metadata.insert(
            "routing_keywords".to_string(),
            json!(routing_keywords.to_vec()),
        );
    }
    if !intents.is_empty() {
        entity
            .metadata
            .insert("intents".to_string(), json!(intents.to_vec()));
    }
    entity
}

impl ZhixingWendaoIndexer {
    fn index_skill_intents(
        &self,
        skill_name: &str,
        source_path: &str,
        intents: &[String],
    ) -> Result<(usize, usize)> {
        let mut entities_added = 0usize;
        let mut relations_added = 0usize;
        let mut normalized_intents = intents
            .iter()
            .map(|intent| intent.trim())
            .filter(|intent| !intent.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        normalized_intents.sort();
        normalized_intents.dedup();

        for intent in normalized_intents {
            let intent_name = format!("intent:{intent}");
            let intent_id = normalize_token(intent.as_str());
            let mut intent_entity = Entity::new(
                format!("zhixing:intent:{intent_id}"),
                intent_name.clone(),
                EntityType::Concept,
                format!("Intent promoted from skill `{skill_name}`"),
            );
            intent_entity.source = Some(source_path.to_string());
            intent_entity
                .metadata
                .insert("zhixing_domain".to_string(), json!("skill_intent"));
            intent_entity
                .metadata
                .insert("skill_semantic_name".to_string(), json!(skill_name));
            intent_entity
                .metadata
                .insert("source_skill_doc".to_string(), json!(source_path));
            intent_entity
                .metadata
                .insert("intent".to_string(), json!(intent.as_str()));
            if self
                .graph
                .add_entity(intent_entity)
                .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?
            {
                entities_added = entities_added.saturating_add(1);
            }
            self.graph
                .add_relation(
                    &Relation::new(
                        skill_name.to_string(),
                        intent_name,
                        RelationType::Governs,
                        format!("Skill `{skill_name}` governs intent `{intent}`"),
                    )
                    .with_source_doc(Some(source_path.to_string()))
                    .with_metadata("intent".to_string(), json!(intent.as_str())),
                )
                .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?;
            relations_added = relations_added.saturating_add(1);
        }

        Ok((entities_added, relations_added))
    }
}

fn dedup_targets(targets: &[WendaoResourceLinkTarget]) -> Vec<WendaoResourceLinkTarget> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for target in targets {
        let key = (
            target.target_path.trim().to_string(),
            target
                .reference_type
                .as_deref()
                .map(str::trim)
                .map(str::to_ascii_lowercase)
                .filter(|value| !value.is_empty()),
        );
        if seen.insert(key) {
            deduped.push(target.clone());
        }
    }
    deduped.sort_by(|left, right| left.target_path.cmp(&right.target_path));
    deduped
}

fn build_reference_entity(
    uri: &WendaoResourceUri,
    source_path: &str,
    reference_id: &str,
    explicit_reference_type: Option<&str>,
    config_type: Option<&str>,
) -> (Entity, String) {
    let reference_name = reference_leaf_name(uri.entity_name());
    let stable_token = normalize_token(uri.entity_name());
    let semantics =
        classify_skill_reference(explicit_reference_type, config_type, uri.entity_name());
    let mut entity = Entity::new(
        format!("zhixing:skill_ref:{}:{stable_token}", uri.semantic_name()),
        reference_name.clone(),
        semantics.entity,
        format!(
            "Semantic skill reference `{}` from `{}`",
            uri.entity_name(),
            uri.semantic_name()
        ),
    );
    entity.source = Some(source_path.to_string());
    entity
        .metadata
        .insert("zhixing_domain".to_string(), json!("skill_reference"));
    entity
        .metadata
        .insert("source_skill_doc".to_string(), json!(source_path));
    entity.metadata.insert(
        "skill_semantic_name".to_string(),
        json!(uri.semantic_name()),
    );
    entity
        .metadata
        .insert("reference_id".to_string(), json!(reference_id));
    entity
        .metadata
        .insert("reference_path".to_string(), json!(uri.entity_name()));
    entity.metadata.insert(
        "reference_uri".to_string(),
        json!(format!(
            "wendao://skills/{}/references/{}",
            uri.semantic_name(),
            uri.entity_name()
        )),
    );
    if let Some(reference_type) = semantics.reference_type {
        entity
            .metadata
            .insert("reference_type".to_string(), json!(reference_type));
    }
    (entity, reference_name)
}

struct ReferenceRelationInput<'a> {
    skill_name: &'a str,
    reference_name: &'a str,
    source_path: &'a str,
    reference_id: &'a str,
    reference_path: &'a str,
    target_uri: &'a str,
    explicit_reference_type: Option<&'a str>,
    config_type: Option<&'a str>,
}

fn build_reference_relation(input: &ReferenceRelationInput<'_>) -> Relation {
    let semantics = classify_skill_reference(
        input.explicit_reference_type,
        input.config_type,
        input.reference_path,
    );
    let relation_type = semantics.relation;
    let relation_label = relation_type.to_string();
    let mut relation = Relation::new(
        input.skill_name.to_string(),
        input.reference_name.to_string(),
        relation_type,
        format!(
            "Skill `{}` {} `{}`",
            input.skill_name, relation_label, input.reference_name
        ),
    )
    .with_source_doc(Some(input.source_path.to_string()))
    .with_metadata("reference_id".to_string(), json!(input.reference_id))
    .with_metadata("reference_uri".to_string(), json!(input.target_uri));

    if let Some(reference_type) = semantics.reference_type {
        relation = relation.with_metadata("reference_type".to_string(), json!(reference_type));
    }

    relation
}

fn reference_leaf_name(entity_path: &str) -> String {
    let path = Path::new(entity_path);
    path.file_stem()
        .or_else(|| path.file_name())
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(|| entity_path.trim().to_string(), ToString::to_string)
}

fn normalize_token(raw: &str) -> String {
    let normalized = raw
        .trim()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_ascii_lowercase();
    if normalized.is_empty() {
        "unknown".to_string()
    } else {
        normalized
    }
}
