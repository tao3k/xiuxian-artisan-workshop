use thiserror::Error;

/// Errors produced while resolving cascading TOML configuration.
#[derive(Debug, Error)]
pub enum ConfigCoreError {
    /// Embedded baseline TOML failed to parse.
    #[error("failed to parse embedded config for namespace `{namespace}`: {source}")]
    ParseEmbedded {
        /// Namespace for the failing config.
        namespace: String,
        /// TOML parser error.
        source: toml::de::Error,
    },
    /// One config file failed to read.
    #[error("failed to read config file {path}: {source}")]
    ReadFile {
        /// Absolute path of the failing file.
        path: String,
        /// I/O error source.
        source: std::io::Error,
    },
    /// One config file failed to parse.
    #[error("failed to parse TOML {path}: {source}")]
    ParseFile {
        /// Absolute path of the failing file.
        path: String,
        /// TOML parser error.
        source: toml::de::Error,
    },
    /// Both `xiuxian.toml` and orphan config files are present.
    #[error(
        "redundant config detected for namespace `{namespace}`: found orphan config files [{orphans}] while xiuxian.toml is active"
    )]
    RedundantOrphan {
        /// Namespace in conflict.
        namespace: String,
        /// Human-readable orphan path list.
        orphans: String,
    },
    /// Merged TOML failed to deserialize into typed config.
    #[error("failed to deserialize merged config for namespace `{namespace}`: {source}")]
    DeserializeMerged {
        /// Namespace in scope.
        namespace: String,
        /// Deserialization error.
        source: toml::de::Error,
    },
}
