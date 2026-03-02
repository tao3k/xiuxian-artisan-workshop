use super::super::core::read_lock;
use super::super::{GraphError, KnowledgeGraph};
use super::{entity_from_dict, relation_from_dict};
use log::info;
use serde_json::{Value, json, to_string};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

impl KnowledgeGraph {
    /// Save graph to a JSON file.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::InvalidRelation`] when serialization or file I/O fails.
    pub fn save_to_file(&self, path: &str) -> Result<(), GraphError> {
        let entities = read_lock(&self.entities);
        let relations = read_lock(&self.relations);

        let entities_json: Vec<Value> = entities
            .values()
            .map(|entity| {
                json!({
                    "id": entity.id,
                    "name": entity.name,
                    "entity_type": entity.entity_type.to_string(),
                    "description": entity.description,
                    "source": entity.source,
                    "aliases": entity.aliases,
                    "confidence": entity.confidence,
                    "metadata": entity.metadata,
                    "created_at": entity.created_at,
                    "updated_at": entity.updated_at,
                })
            })
            .collect();

        let relations_json: Vec<Value> = relations
            .values()
            .map(|relation| {
                json!({
                    "id": relation.id,
                    "source": relation.source,
                    "target": relation.target,
                    "relation_type": relation.relation_type.to_string(),
                    "description": relation.description,
                    "source_doc": relation.source_doc,
                    "confidence": relation.confidence,
                    "metadata": relation.metadata,
                })
            })
            .collect();

        let export = json!({
            "version": 1,
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "total_entities": entities_json.len(),
            "total_relations": relations_json.len(),
            "entities": entities_json,
            "relations": relations_json,
        });

        let path_buf = PathBuf::from(path);
        if let Some(parent) = path_buf.parent()
            && !parent.exists()
            && let Err(error) = fs::create_dir_all(parent)
        {
            return Err(GraphError::InvalidRelation(
                parent.to_string_lossy().to_string(),
                error.to_string(),
            ));
        }

        let json_str = to_string(&export).map_err(|error| {
            GraphError::InvalidRelation("serialization".to_string(), error.to_string())
        })?;

        let mut file = File::create(path_buf)
            .map_err(|error| GraphError::InvalidRelation(path.to_string(), error.to_string()))?;

        file.write_all(json_str.as_bytes())
            .map_err(|error| GraphError::InvalidRelation(path.to_string(), error.to_string()))?;

        info!(
            "Knowledge graph saved to: {} ({} entities, {} relations)",
            path,
            entities_json.len(),
            relations_json.len()
        );

        Ok(())
    }

    /// Load graph from JSON file.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::InvalidRelation`] when file read or JSON parse fails.
    pub fn load_from_file(&mut self, path: &str) -> Result<(), GraphError> {
        let mut file = File::open(path)
            .map_err(|error| GraphError::InvalidRelation(path.to_string(), error.to_string()))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|error| GraphError::InvalidRelation(path.to_string(), error.to_string()))?;

        let value: Value = serde_json::from_str(&content)
            .map_err(|error| GraphError::InvalidRelation("parse".to_string(), error.to_string()))?;

        self.clear();

        if let Some(entities_arr) = value.get("entities").and_then(|row| row.as_array()) {
            for entity_val in entities_arr {
                if let Some(entity) = entity_from_dict(entity_val) {
                    self.add_entity(entity).ok();
                }
            }
        }

        if let Some(relations_arr) = value.get("relations").and_then(|row| row.as_array()) {
            for relation_val in relations_arr {
                if let Some(relation) = relation_from_dict(relation_val) {
                    self.add_relation(&relation).ok();
                }
            }
        }

        let stats = self.get_stats();
        info!(
            "Knowledge graph loaded from: {} ({} entities, {} relations)",
            path, stats.total_entities, stats.total_relations
        );

        Ok(())
    }
}
