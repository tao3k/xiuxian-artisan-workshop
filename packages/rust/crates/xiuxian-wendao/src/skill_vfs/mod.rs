//! Skill VFS resolver for semantic `wendao://` resource addresses.
//!
//! This module implements ADR-007 addressing:
//! `wendao://skills/<semantic_name>/references/<entity_name>`.

mod asset_request;
mod error;
mod index;
mod resolver;
mod uri;
mod zhixing;

pub use asset_request::{AssetRequest, WendaoAssetHandle};
pub use error::SkillVfsError;
pub use index::{SkillNamespaceIndex, SkillNamespaceMount};
pub use resolver::SkillVfsResolver;
pub use uri::{WENDAO_URI_SCHEME, WendaoResourceUri};
pub use zhixing::{
    ZHIXING_SKILL_DOC_PATH, ZhixingIndexSummary, ZhixingWendaoIndexer,
    build_embedded_wendao_registry, embedded_discover_canonical_uris, embedded_resource_text,
    embedded_resource_text_from_wendao_uri, embedded_skill_links_for_id,
    embedded_skill_links_for_reference_type, embedded_skill_links_index, embedded_skill_markdown,
};
