//! `LinkGraph` reference extraction primitives.
//!
//! Provides fast regex-based extraction of entity references from `Markdown`
//! notes.
//! Pattern: `[[EntityName]]` or `[[EntityName#type]]`

mod extract;
mod model;
mod regex;
mod stats;

pub use extract::{
    count_entity_refs, extract_entity_refs, extract_entity_refs_batch,
    find_notes_referencing_entity, is_valid_entity_ref, parse_entity_ref,
};
pub use model::LinkGraphEntityRef;
pub use stats::{LinkGraphRefStats, get_ref_stats};
