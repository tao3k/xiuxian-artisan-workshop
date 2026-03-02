use thiserror::Error;

/// Graph errors.
#[derive(Debug, Error)]
pub enum GraphError {
    /// The entity payload is invalid.
    #[error("Invalid entity: {0}")]
    InvalidEntity(String),
    /// The requested entity was not found.
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    /// A relation with this ID already exists.
    #[error("Relation already exists: {0}")]
    RelationExists(String),
    /// The relation references invalid source/target entities.
    #[error("Invalid relation: source={0}, target={1}")]
    InvalidRelation(String, String),
}
