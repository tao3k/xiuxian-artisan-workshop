//! Skill metadata models and sync helpers used by skill scanners.

mod core;
mod index;
mod records;
mod structure;
mod sync;

pub use core::{
    DecoratorArgs, ReferencePath, SkillMetadata, SnifferRule, ToolAnnotations, ToolRecord,
};
pub use index::{DocsAvailable, IndexToolEntry, SkillIndexEntry};
pub use records::{
    AssetRecord, DataRecord, PromptRecord, ReferenceRecord, ResourceRecord, TemplateRecord,
    TestRecord,
};
pub use structure::{SkillStructure, StructureItem};
pub use sync::{ScanConfig, SyncReport, calculate_sync_ops};
