use std::path::PathBuf;

use thiserror::Error;

/// Errors produced by skill VFS URI parsing/indexing/resolution.
#[derive(Debug, Error)]
pub enum SkillVfsError {
    /// URI scheme is not `wendao`.
    #[error("unsupported URI scheme `{scheme}` in `{uri}`; expected `wendao://`")]
    UnsupportedScheme {
        /// Full URI string.
        uri: String,
        /// Parsed unsupported scheme.
        scheme: String,
    },
    /// URI payload does not follow `skills/<name>/references/<entity>`.
    #[error("invalid wendao resource URI `{0}`")]
    InvalidUri(String),
    /// One required segment is absent or empty.
    #[error("missing URI segment `{segment}` in `{uri}`")]
    MissingUriSegment {
        /// Full URI string.
        uri: String,
        /// Required segment identifier.
        segment: &'static str,
    },
    /// Entity path includes forbidden traversal or empty components.
    #[error("invalid entity path `{entity}` in URI `{uri}`")]
    InvalidEntityPath {
        /// Full URI string.
        uri: String,
        /// Entity path payload.
        entity: String,
    },
    /// Entity path does not include a file extension.
    #[error("entity path `{entity}` in URI `{uri}` must include a file extension")]
    MissingEntityExtension {
        /// Full URI string.
        uri: String,
        /// Entity path payload.
        entity: String,
    },
    /// Reading one SKILL descriptor file failed.
    #[error("failed to read skill descriptor `{path}`: {source}")]
    ReadSkillDescriptor {
        /// Path of SKILL descriptor.
        path: PathBuf,
        /// I/O error.
        source: std::io::Error,
    },
    /// Parsing SKILL frontmatter failed.
    #[error("failed to parse YAML frontmatter in `{path}`: {source}")]
    ParseSkillFrontmatter {
        /// Path of SKILL descriptor.
        path: PathBuf,
        /// YAML parse error.
        source: serde_yaml::Error,
    },
    /// Parsing skill metadata through xiuxian-skills failed.
    #[error("failed to scan skill metadata for `{path}`: {reason}")]
    ScanSkillMetadata {
        /// Path of SKILL descriptor.
        path: PathBuf,
        /// Scanner failure reason.
        reason: String,
    },
    /// Semantic namespace not found in mounted roots.
    #[error("semantic skill namespace `{semantic_name}` not found")]
    UnknownSemanticSkill {
        /// Semantic namespace from URI.
        semantic_name: String,
    },
    /// References entity not found under known mounts.
    #[error("resource `{entity_name}` not found under semantic skill namespace `{semantic_name}`")]
    ResourceNotFound {
        /// Semantic namespace from URI.
        semantic_name: String,
        /// Entity name from URI.
        entity_name: String,
    },
    /// Reading a resolved resource file failed.
    #[error("failed to read resource `{path}`: {source}")]
    ReadResource {
        /// Resolved file path.
        path: PathBuf,
        /// I/O error.
        source: std::io::Error,
    },
    /// Relative asset path is empty or contains traversal.
    #[error("invalid relative asset path `{path}`")]
    InvalidRelativeAssetPath {
        /// Provided relative path.
        path: String,
    },
    /// External embedded loader failed to resolve one URI.
    #[error("failed to resolve embedded asset for URI `{uri}`")]
    EmbeddedAssetNotFound {
        /// Full URI string.
        uri: String,
    },
}
