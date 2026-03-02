//! Core metadata types used across skill scanning and indexing.

mod decorator_args;
mod reference_path;
mod skill_metadata;
mod sniffer_rule;
mod tool_annotations;
mod tool_record;

pub use decorator_args::DecoratorArgs;
pub use reference_path::ReferencePath;
pub use skill_metadata::SkillMetadata;
pub use sniffer_rule::SnifferRule;
pub use tool_annotations::ToolAnnotations;
pub use tool_record::{ToolEnrichment, ToolRecord};
