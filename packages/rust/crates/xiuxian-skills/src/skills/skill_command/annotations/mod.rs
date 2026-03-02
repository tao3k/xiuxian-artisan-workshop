//! Tool annotations builder.
//!
//! Provides heuristics for inferring tool annotations (`read_only`, `destructive`, etc.)
//! from function naming patterns.

mod build;
mod heuristics;

pub use build::build_annotations;
