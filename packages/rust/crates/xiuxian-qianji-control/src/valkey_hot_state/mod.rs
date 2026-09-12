//! Valkey-backed hot scheduling state and bounded observation.

mod store;

pub use store::{ValkeyHotStateConfig, ValkeyHotStateStore, ValkeyKeyNamespace};
