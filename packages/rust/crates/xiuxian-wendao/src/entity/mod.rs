//! Entity types for knowledge graph.
//!
//! Provides Entity and Relation types for knowledge graph operations.

mod query;
mod records;
mod types;

pub use query::{EntitySearchQuery, MultiHopOptions};
pub use records::{Entity, GraphStats, Relation};
pub use types::{EntityType, RelationType};
