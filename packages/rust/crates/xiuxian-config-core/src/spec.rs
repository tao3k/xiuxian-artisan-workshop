/// Array merge behavior when resolving layered TOML values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ArrayMergeStrategy {
    /// Replace destination arrays with the latest source array.
    #[default]
    Overwrite,
    /// Append source array items to destination arrays.
    Append,
}

/// Immutable runtime spec used by the cascading resolver.
#[derive(Debug, Clone, Copy)]
pub struct ConfigCascadeSpec<'a> {
    /// Namespace key inside `xiuxian.toml` (for example `skills`).
    pub namespace: &'a str,
    /// Embedded baseline TOML payload bundled in the crate binary.
    pub embedded_toml: &'a str,
    /// Optional standalone/orphan config filename (for example `orphan.toml`).
    pub orphan_file: &'a str,
    /// Strategy for merging TOML arrays.
    pub array_merge_strategy: ArrayMergeStrategy,
}

impl<'a> ConfigCascadeSpec<'a> {
    /// Build a new cascade spec.
    #[must_use]
    pub const fn new(namespace: &'a str, embedded_toml: &'a str, orphan_file: &'a str) -> Self {
        Self {
            namespace,
            embedded_toml,
            orphan_file,
            array_merge_strategy: ArrayMergeStrategy::Overwrite,
        }
    }

    /// Override the default array merge strategy.
    #[must_use]
    pub const fn with_array_merge_strategy(self, strategy: ArrayMergeStrategy) -> Self {
        Self {
            array_merge_strategy: strategy,
            ..self
        }
    }
}
