//! Unified cascading configuration kernel.

mod cache;
mod error;
mod resolve;
mod spec;

pub use error::ConfigCoreError;
pub use resolve::{
    resolve_and_load, resolve_and_load_with_paths, resolve_and_merge_toml,
    resolve_and_merge_toml_with_paths,
};
pub use spec::{ArrayMergeStrategy, ConfigCascadeSpec};
