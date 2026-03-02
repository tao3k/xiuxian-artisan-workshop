use super::core::{read_lock, write_lock};
use super::{GraphError, KnowledgeGraph};
use crate::entity::Entity;
use log::info;

impl KnowledgeGraph {
    /// Add an entity.
    ///
    /// Returns `true` if newly added, `false` if updated in place.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::InvalidEntity`] when required entity fields are empty.
    pub fn add_entity(&self, entity: Entity) -> Result<bool, GraphError> {
        if entity.id.trim().is_empty() {
            return Err(GraphError::InvalidEntity(
                "entity id must not be empty".to_string(),
            ));
        }
        if entity.name.trim().is_empty() {
            return Err(GraphError::InvalidEntity(
                "entity name must not be empty".to_string(),
            ));
        }

        let mut entities = write_lock(&self.entities);
        let mut entities_by_name = write_lock(&self.entities_by_name);
        let mut entities_by_type = write_lock(&self.entities_by_type);

        let type_str = entity.entity_type.to_string();
        if let Some(existing_id) = entities_by_name.get(&entity.name)
            && let Some(existing) = entities.get_mut(existing_id)
        {
            existing.description = entity.description;
            existing.source = entity.source.or(existing.source.clone());
            existing.aliases = entity.aliases;
            existing.confidence = entity.confidence;
            existing.updated_at = entity.updated_at;
            existing.metadata.extend(entity.metadata);
            info!("Updated entity: {}", entity.name);
            return Ok(false);
        }

        let entity_id = entity.id.clone();
        entities.insert(entity_id.clone(), entity.clone());
        entities_by_name.insert(entity.name.clone(), entity_id.clone());

        entities_by_type
            .entry(type_str)
            .or_default()
            .push(entity_id.clone());

        info!("Added entity: {} ({})", entity.name, entity.entity_type);
        Ok(true)
    }

    /// Get an entity by ID.
    #[must_use]
    pub fn get_entity(&self, entity_id: &str) -> Option<Entity> {
        read_lock(&self.entities).get(entity_id).cloned()
    }

    /// Get an entity by name.
    #[must_use]
    pub fn get_entity_by_name(&self, name: &str) -> Option<Entity> {
        let entities_by_name = read_lock(&self.entities_by_name);
        if let Some(entity_id) = entities_by_name.get(name) {
            return read_lock(&self.entities).get(entity_id).cloned();
        }
        None
    }

    /// Get entities by type.
    #[must_use]
    pub fn get_entities_by_type(&self, entity_type: &str) -> Vec<Entity> {
        let entities_by_type = read_lock(&self.entities_by_type);
        let entities = read_lock(&self.entities);

        if let Some(entity_ids) = entities_by_type.get(entity_type) {
            entity_ids
                .iter()
                .filter_map(|id| entities.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clear all entities and relations.
    pub fn clear(&mut self) {
        write_lock(&self.entities).clear();
        write_lock(&self.entities_by_name).clear();
        write_lock(&self.relations).clear();
        write_lock(&self.outgoing_relations).clear();
        write_lock(&self.incoming_relations).clear();
        write_lock(&self.entities_by_type).clear();
        info!("Knowledge graph cleared");
    }

    /// Get all entities as a vector.
    #[must_use]
    pub fn get_all_entities(&self) -> Vec<Entity> {
        read_lock(&self.entities).values().cloned().collect()
    }

    /// Remove an entity by ID.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::EntityNotFound`] if the entity ID is absent.
    pub fn remove_entity(&self, entity_id: &str) -> Result<(), GraphError> {
        let mut entities = write_lock(&self.entities);
        let mut entities_by_name = write_lock(&self.entities_by_name);
        let mut entities_by_type = write_lock(&self.entities_by_type);
        let mut relations = write_lock(&self.relations);
        let mut outgoing = write_lock(&self.outgoing_relations);
        let mut incoming = write_lock(&self.incoming_relations);

        if let Some(entity) = entities.remove(entity_id) {
            entities_by_name.remove(&entity.name);

            if let Some(ids) = entities_by_type.get_mut(&entity.entity_type.to_string()) {
                ids.retain(|id| id != entity_id);
            }

            let rel_ids_to_remove: Vec<String> = relations
                .keys()
                .filter(|id| {
                    if let Some(rel) = relations.get(id.as_str()) {
                        rel.source == entity.name || rel.target == entity.name
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            for rid in rel_ids_to_remove {
                relations.remove(&rid);
                outgoing.remove(&entity.name);
                incoming.remove(&entity.name);
            }

            info!("Removed entity: {} ({})", entity.name, entity_id);
            Ok(())
        } else {
            Err(GraphError::EntityNotFound(entity_id.to_string()))
        }
    }
}
