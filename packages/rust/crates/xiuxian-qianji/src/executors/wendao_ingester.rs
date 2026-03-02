//! Native `Wendao` ingestion mechanism for memory-promotion workflows.

use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde_json::{Map, Value, json};
use xiuxian_wendao::{Entity, EntityType, KnowledgeGraph, Relation, RelationType};

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};

const DEFAULT_GRAPH_SCOPE: &str = "qianji:memory_promotion";

/// Promotes validated reflection context into a structured `Wendao` graph entity.
pub struct WendaoIngesterMechanism {
    /// Output key used for the emitted entity payload.
    pub output_key: String,
    /// Static graph scope fallback.
    pub graph_scope: Option<String>,
    /// Optional context key that provides graph scope dynamically.
    pub graph_scope_key: Option<String>,
    /// Vector dimension metadata used by `KnowledgeGraph::save_to_valkey`.
    pub graph_dimension: usize,
    /// Whether persistence should be attempted.
    pub persist: bool,
    /// Whether persistence failures should be recorded as output instead of failing the node.
    pub persist_best_effort: bool,
}

#[async_trait]
impl QianjiMechanism for WendaoIngesterMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let selected_route = context
            .get("selected_route")
            .and_then(Value::as_str)
            .unwrap_or("Promote");
        let decision = selected_route.to_lowercase();

        let graph_scope = resolve_graph_scope(
            context,
            self.graph_scope.as_ref(),
            self.graph_scope_key.as_ref(),
        );
        let entity = build_promotion_entity(context, &decision);
        let topic_entity = build_topic_entity(context);
        let relation = build_promotion_relation(&entity, &topic_entity, &decision);
        let mut persisted = false;
        let mut persist_error: Option<String> = None;

        if self.persist && decision == "promote" {
            let mut graph = KnowledgeGraph::new();
            let persist_result = graph
                .load_from_valkey(&graph_scope)
                .map_err(|error| format!("failed to load existing wendao graph: {error}"))
                .and_then(|()| {
                    graph
                        .add_entity(entity.clone())
                        .map_err(|error| format!("failed to add promotion entity: {error}"))
                })
                .and_then(|_added| {
                    graph
                        .add_entity(topic_entity.clone())
                        .map_err(|error| format!("failed to add topic entity: {error}"))
                })
                .and_then(|_added| {
                    graph
                        .add_relation(&relation)
                        .map_err(|error| format!("failed to add promotion relation: {error}"))
                })
                .and_then(|()| {
                    graph
                        .save_to_valkey(&graph_scope, self.graph_dimension)
                        .map_err(|error| format!("failed to save wendao graph: {error}"))
                });

            match persist_result {
                Ok(()) => persisted = true,
                Err(error) if self.persist_best_effort => {
                    persist_error = Some(error);
                    log::warn!(
                        "qianji wendao ingester best-effort persistence failed: {}",
                        persist_error.as_deref().unwrap_or("")
                    );
                }
                Err(error) => return Err(error),
            }
        }

        let mut data = Map::new();
        data.insert(
            "promotion_decision".to_string(),
            Value::String(decision.clone()),
        );
        data.insert(self.output_key.clone(), json!(entity));
        data.insert(
            "promotion_graph_scope".to_string(),
            Value::String(graph_scope.clone()),
        );
        data.insert("promotion_topic_entity".to_string(), json!(topic_entity));
        data.insert("promotion_relation".to_string(), json!(relation));
        data.insert("promotion_persisted".to_string(), Value::Bool(persisted));
        if let Some(error) = persist_error {
            data.insert("promotion_persist_error".to_string(), Value::String(error));
        }

        Ok(QianjiOutput {
            data: Value::Object(data),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        2.0
    }
}

fn resolve_graph_scope(
    context: &serde_json::Value,
    static_scope: Option<&String>,
    scope_key: Option<&String>,
) -> String {
    let dynamic_scope = scope_key
        .and_then(|key| context.get(key.as_str()))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|scope| !scope.is_empty())
        .map(ToString::to_string);
    let static_scope = static_scope
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .map(ToString::to_string);
    dynamic_scope
        .or(static_scope)
        .unwrap_or_else(|| DEFAULT_GRAPH_SCOPE.to_string())
}

fn build_promotion_entity(context: &serde_json::Value, decision: &str) -> Entity {
    let query = context
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("memory promotion");
    let summary = context
        .get("annotated_prompt")
        .and_then(Value::as_str)
        .unwrap_or("promotion context unavailable");
    let memory_id = context
        .get("memory_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(generate_fallback_memory_id, ToString::to_string);
    let title = context
        .get("memory_title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(
            || format!("Memory Promotion {memory_id}"),
            ToString::to_string,
        );

    let description =
        format!("MemRL promotion decision={decision}; query={query}; summary={summary}",);

    Entity::new(
        format!("memory:{memory_id}"),
        title,
        EntityType::Document,
        description,
    )
    .with_source(Some("qianji://memory_promotion".to_string()))
    .with_metadata("memory_id".to_string(), Value::String(memory_id))
    .with_metadata(
        "promotion_decision".to_string(),
        Value::String(decision.to_string()),
    )
    .with_metadata("query".to_string(), Value::String(query.to_string()))
}

fn build_topic_entity(context: &serde_json::Value) -> Entity {
    let query = context
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("memory promotion");
    let topic_key = normalize_topic_key(query);
    Entity::new(
        format!("topic:{topic_key}"),
        format!("Topic {topic_key}"),
        EntityType::Concept,
        format!("Promotion topic derived from query: {query}"),
    )
    .with_source(Some("qianji://memory_promotion".to_string()))
    .with_metadata("topic_query".to_string(), Value::String(query.to_string()))
}

fn build_promotion_relation(source: &Entity, topic: &Entity, decision: &str) -> Relation {
    Relation::new(
        source.name.clone(),
        topic.name.clone(),
        RelationType::RelatedTo,
        "Memory promotion linkage".to_string(),
    )
    .with_source_doc(Some("qianji://memory_promotion".to_string()))
    .with_metadata(
        "promotion_decision".to_string(),
        Value::String(decision.to_string()),
    )
}

fn normalize_topic_key(raw: &str) -> String {
    let mut normalized = String::with_capacity(raw.len().min(64));
    let mut previous_was_separator = false;

    for ch in raw.chars() {
        if normalized.len() >= 64 {
            break;
        }
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            normalized.push(lower);
            previous_was_separator = false;
        } else if !previous_was_separator {
            normalized.push('-');
            previous_was_separator = true;
        }
    }

    let trimmed = normalized.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "general".to_string()
    } else {
        trimmed
    }
}

fn generate_fallback_memory_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("auto-{millis}")
}
