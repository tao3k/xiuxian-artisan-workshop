mod validation;

use serde::{Deserialize, Serialize};

pub use validation::SkillValidationReport;

/// Represents the canonical skill structure as defined in `skills.toml`.
#[xiuxian_macros::xiuxian_config(
    namespace = "skills",
    internal_path = "resources/config/skills.toml",
    orphan_file = "",
    array_merge = "append"
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(from = "SkillStructureDeserialize")]
pub struct SkillStructure {
    /// Required items that must exist in a valid skill.
    pub required: Vec<StructureItem>,
    /// Default items that are created when generating a new skill.
    pub default: Vec<StructureItem>,
    /// Optional items that may be present.
    #[serde(default)]
    pub optional: Vec<StructureItem>,
    /// Validation policy flags loaded from `[validation]`.
    #[serde(default)]
    pub validation: SkillValidationPolicy,
}

/// Validation policy for declarative skill structure checks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SkillValidationPolicy {
    /// Structural filesystem checks (`required/default/references`).
    #[serde(default, flatten)]
    pub structure: SkillStructureValidationPolicy,
    /// `SKILL.md` content checks (logic/frontmatter contract).
    #[serde(default, flatten)]
    pub frontmatter: SkillFrontmatterValidationPolicy,
}

/// Structural validation switches for skill directory layout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SkillStructureValidationPolicy {
    /// When true, default directories are also required to exist.
    #[serde(default)]
    pub strict_mode: bool,
    /// When true, `references/` must exist for every skill.
    #[serde(default)]
    pub enforce_references_folder: bool,
}

/// Frontmatter/content validation switches for `SKILL.md`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SkillFrontmatterValidationPolicy {
    /// When true, forbid Jinja-like logic markers in `SKILL.md`.
    #[serde(default)]
    pub prohibit_logic_in_skill_md: bool,
    /// When true, `SKILL.md` must contain valid YAML frontmatter.
    #[serde(default)]
    pub require_yaml_frontmatter: bool,
}

/// A single item in the skill structure definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructureItem {
    /// Path or pattern for the item.
    pub path: String,
    /// Description of what this item represents.
    #[serde(default)]
    pub description: String,
    /// Type of item (`file` or `dir`).
    #[serde(default)]
    pub item_type: String,
}

impl Default for SkillStructure {
    fn default() -> Self {
        Self::load_effective()
    }
}

impl SkillStructure {
    /// Creates a new `SkillStructure` from configured defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load effective structure from embedded resources + project/user overrides.
    ///
    /// # Panics
    ///
    /// Panics when cascading configuration load fails.
    #[must_use]
    pub fn load_effective() -> Self {
        Self::load().unwrap_or_else(|error| {
            panic!("failed to load cascading skill structure config for [skills]: {error}")
        })
    }

    /// Returns the paths of all default directories.
    #[must_use]
    pub fn script_directories(&self) -> Vec<&str> {
        self.default
            .iter()
            .filter(|item| item.is_dir_kind())
            .map(|item| item.path.trim_end_matches('/'))
            .collect()
    }

    /// Returns immutable validation policy flags.
    #[must_use]
    pub fn validation_policy(&self) -> &SkillValidationPolicy {
        &self.validation
    }

    /// Validate one skill path using the current declarative policy.
    #[must_use]
    pub fn validate_skill_path(&self, skill_path: &std::path::Path) -> SkillValidationReport {
        validation::validate_skill_path(self, skill_path)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SkillStructureDeserialize {
    Canonical(SkillStructureCanonical),
    Embedded(SkillStructureEmbedded),
}

#[derive(Debug, Clone, Deserialize)]
struct SkillStructureCanonical {
    required: Vec<StructureItem>,
    default: Vec<StructureItem>,
    #[serde(default)]
    optional: Vec<StructureItem>,
    #[serde(default)]
    validation: SkillValidationPolicy,
}

#[derive(Debug, Clone, Deserialize)]
struct SkillStructureEmbedded {
    architecture: SkillStructureEmbeddedArchitecture,
    #[serde(default)]
    validation: SkillValidationPolicy,
}

#[derive(Debug, Clone, Deserialize)]
struct SkillStructureEmbeddedArchitecture {
    required: Vec<StructureItem>,
    default: Vec<StructureItem>,
    #[serde(default)]
    optional: Vec<StructureItem>,
}

impl From<SkillStructureDeserialize> for SkillStructure {
    fn from(value: SkillStructureDeserialize) -> Self {
        match value {
            SkillStructureDeserialize::Canonical(config) => Self {
                required: config.required,
                default: config.default,
                optional: config.optional,
                validation: config.validation,
            },
            SkillStructureDeserialize::Embedded(config) => Self {
                required: config.architecture.required,
                default: config.architecture.default,
                optional: config.architecture.optional,
                validation: config.validation,
            },
        }
    }
}

impl StructureItem {
    pub(crate) fn is_dir_kind(&self) -> bool {
        matches!(self.item_kind(), StructureItemKind::Dir)
    }

    pub(crate) fn item_kind(&self) -> StructureItemKind {
        match self.item_type.trim().to_ascii_lowercase().as_str() {
            "dir" | "directory" => StructureItemKind::Dir,
            "file" => StructureItemKind::File,
            _ if self.path.trim_end().ends_with('/') => StructureItemKind::Dir,
            _ => StructureItemKind::File,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StructureItemKind {
    File,
    Dir,
}
