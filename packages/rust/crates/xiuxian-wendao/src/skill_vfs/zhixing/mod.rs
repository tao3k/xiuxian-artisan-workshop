//! Zhixing skill resource loading and domain indexer built on Wendao primitives.

mod indexer;
mod resources;

use thiserror::Error as ThisError;

const ATTR_JOURNAL_CARRYOVER: &str = "journal:carryover";
const ATTR_TIMER_SCHEDULED: &str = "timer:scheduled";
const ATTR_TIMER_REMINDED: &str = "timer:reminded";

/// Result type for Zhixing-Wendao resource and indexing operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for Zhixing-Wendao resource and indexing operations.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Internal parsing, filesystem, or graph operation failure.
    #[error("zhixing-wendao integration error: {0}")]
    Internal(String),
}

pub use indexer::{ZhixingIndexSummary, ZhixingWendaoIndexer};
pub(crate) use resources::{
    ZHIXING_EMBEDDED_CRATE_ID, embedded_resource_dir, embedded_semantic_reference_mounts,
};
pub use resources::{
    ZHIXING_SKILL_DOC_PATH, build_embedded_wendao_registry, embedded_discover_canonical_uris,
    embedded_resource_text, embedded_resource_text_from_wendao_uri, embedded_skill_links_for_id,
    embedded_skill_links_for_reference_type, embedded_skill_links_index, embedded_skill_markdown,
};
