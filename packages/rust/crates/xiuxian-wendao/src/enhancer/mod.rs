//! `LinkGraph` note enhancement engine.
//!
//! Secondary analysis for `LinkGraph` query results.
//! - Parse YAML frontmatter into structured metadata
//! - Infer typed relations from note structure
//! - Batch enhance notes (frontmatter + entities + relations)
//! - Extract tagged markdown config blocks into an O(1) memory index
//!
//! The `LinkGraph` backend remains the primary engine for scanning, building the link
//! graph, and querying. This module enriches results with deeper
//! structural analysis at Rust-native speed.

mod frontmatter;
mod markdown_config;
mod pipeline;
mod relations;
mod resource_registry;
mod resource_semantics;
mod types;

pub use frontmatter::parse_frontmatter;
pub use markdown_config::{
    MarkdownConfigBlock, MarkdownConfigLinkTarget, MarkdownConfigMemoryIndex,
    extract_markdown_config_blocks, extract_markdown_config_link_targets_by_id,
    extract_markdown_config_links_by_id,
};
pub use pipeline::{enhance_note, enhance_notes_batch};
pub use relations::infer_relations;
pub use resource_registry::{
    MissingEmbeddedLink, WendaoResourceFile, WendaoResourceLinkTarget, WendaoResourceRegistry,
    WendaoResourceRegistryError,
};
pub use resource_semantics::{SkillReferenceSemantics, classify_skill_reference};
pub use types::{
    EnhancedNote, EntityRefData, InferredRelation, NoteFrontmatter, NoteInput, RefStatsData,
};
